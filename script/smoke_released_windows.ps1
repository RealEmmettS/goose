[CmdletBinding()]
param(
    [Parameter(Mandatory = $true)]
    [ValidatePattern('^v[0-9]+\.[0-9]+\.[0-9]+$')]
    [string] $Tag,

    [ValidateSet('x86_64-pc-windows-msvc', 'aarch64-pc-windows-msvc')]
    [string] $TargetTriple = 'x86_64-pc-windows-msvc',

    [string] $CurrentMsiPath = '',

    [string] $SourceBinaryPath = '',

    [string] $OverlayEvidenceDirectory = '',

    [switch] $AllowUnobservableTrayRecoveryHost
)

$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest

$Repository = 'https://github.com/RealEmmettS/goose'
$Version = $Tag.Substring(1)
$Root = Join-Path $env:RUNNER_TEMP "honk300-live-msi-$([Guid]::NewGuid().ToString('N'))"
$MsiExec = Join-Path ([Environment]::GetFolderPath([Environment+SpecialFolder]::System)) 'msiexec.exe'
$Binary = Join-Path $env:ProgramFiles 'honk300\bin\honk300.exe'
$AppLauncher = Join-Path $env:ProgramFiles 'honk300\bin\honk300-app.exe'
$ArchitectureScript = Join-Path $PSScriptRoot 'verify_binary_architecture.py'
$script:MsiInvocation = 0
$ExpectedHost = if ($TargetTriple -eq 'aarch64-pc-windows-msvc') { 'ARM64' } else { 'AMD64' }
$ExpectedMachine = if ($TargetTriple -eq 'aarch64-pc-windows-msvc') { '0xAA64' } else { '0x8664' }
if ($env:PROCESSOR_ARCHITECTURE -ne $ExpectedHost) {
    throw "Expected native $ExpectedHost host for $TargetTriple, got '$env:PROCESSOR_ARCHITECTURE'"
}
New-Item -ItemType Directory -Force -Path $Root | Out-Null
if (-not $OverlayEvidenceDirectory) {
    $OverlayEvidenceDirectory = Join-Path (Split-Path -Parent $PSScriptRoot) `
        "target\post-release-windows-overlay-evidence-$TargetTriple"
}
$OverlayEvidenceDirectory = [IO.Path]::GetFullPath($OverlayEvidenceDirectory)
New-Item -ItemType Directory -Force -Path $OverlayEvidenceDirectory | Out-Null

function Download-VerifiedReleaseAsset {
    param([string] $ReleaseTag, [string] $Name)
    $directory = Join-Path $Root $ReleaseTag
    New-Item -ItemType Directory -Force -Path $directory | Out-Null
    $path = Join-Path $directory $Name
    $sidecar = "$path.sha256"
    $base = "$Repository/releases/download/$ReleaseTag"
    Invoke-WebRequest -UseBasicParsing "$base/$Name" -OutFile $path -MaximumRetryCount 4 -RetryIntervalSec 2
    Invoke-WebRequest -UseBasicParsing "$base/$Name.sha256" -OutFile $sidecar -MaximumRetryCount 4 -RetryIntervalSec 2
    $expected = ((Get-Content -LiteralPath $sidecar -Raw).Trim() -split '\s+')[0].ToLowerInvariant()
    if ($expected -notmatch '^[0-9a-f]{64}$') { throw "invalid checksum sidecar for $Name" }
    $actual = (Get-FileHash -LiteralPath $path -Algorithm SHA256).Hash.ToLowerInvariant()
    if ($actual -ne $expected) { throw "checksum mismatch for $Name" }
    return $path
}

function Start-Msi {
    param([string] $Mode, [string] $Path, [string[]] $Extra = @())
    $script:MsiInvocation += 1
    $log = Join-Path $Root "msi-$($script:MsiInvocation).log"
    $arguments = @($Mode, "`"$Path`"", '/qn', '/norestart') + $Extra + @('/l*v', "`"$log`"")
    $process = Start-Process -FilePath $MsiExec -ArgumentList $arguments -WindowStyle Hidden -Wait -PassThru
    return [pscustomobject]@{ ExitCode = $process.ExitCode; Log = $log }
}

function Require-MsiSuccess {
    param([string] $Mode, [string] $Path, [string[]] $Extra = @())
    $result = Start-Msi -Mode $Mode -Path $Path -Extra $Extra
    if ($result.ExitCode -notin @(0, 3010)) {
        Get-Content -LiteralPath $result.Log -Tail 100 -ErrorAction SilentlyContinue | Write-Error
        throw "MSI $Mode failed with $($result.ExitCode)"
    }
}

function Stop-InstalledRuntimeBounded {
    param([Parameter(Mandatory = $true)] [string] $Executable)

    $stopProcess = $null
    $stopStdout = $null
    $stopStderr = $null
    $stopStarted = $false
    try {
        $stopStart = [Diagnostics.ProcessStartInfo]::new()
        $stopStart.FileName = $Executable
        [void]$stopStart.ArgumentList.Add('stop')
        [void]$stopStart.ArgumentList.Add('--force')
        $stopStart.UseShellExecute = $false
        $stopStart.CreateNoWindow = $true
        $stopStart.RedirectStandardOutput = $true
        $stopStart.RedirectStandardError = $true
        $stopProcess = [Diagnostics.Process]::new()
        $stopProcess.StartInfo = $stopStart
        if (-not $stopProcess.Start()) {
            Write-Warning 'final runtime stop controller did not start'
            return
        }
        $stopStarted = $true
        $stopStdout = $stopProcess.StandardOutput.ReadToEndAsync()
        $stopStderr = $stopProcess.StandardError.ReadToEndAsync()
        if (-not $stopProcess.WaitForExit(5000)) {
            try {
                $stopProcess.Kill($true)
            }
            catch [InvalidOperationException] {
                # The controller may exit between the bounded wait and Kill.
            }
            if (-not $stopProcess.WaitForExit(5000)) {
                Write-Warning "final runtime stop controller PID $($stopProcess.Id) survived tree kill"
                return
            }
            Write-Warning 'final runtime stop controller exceeded five seconds and was terminated'
        }
        foreach ($drain in @($stopStdout, $stopStderr)) {
            if ($null -ne $drain -and -not $drain.Wait(5000)) {
                Write-Warning 'final runtime stop controller output did not close within five seconds'
                return
            }
        }
        if ($stopProcess.ExitCode -ne 0) {
            Write-Warning "final runtime stop controller returned $($stopProcess.ExitCode)"
        }
    }
    catch {
        Write-Warning "final bounded runtime stop failed: $($_.Exception.Message)"
    }
    finally {
        if ($stopStarted -and -not $stopProcess.HasExited) {
            try { $stopProcess.Kill($true) } catch { Write-Warning $_ }
            [void]$stopProcess.WaitForExit(5000)
        }
        if ($null -ne $stopProcess) { $stopProcess.Dispose() }
    }
}

function Reported-Version {
    if (-not (Test-Path -LiteralPath $Binary -PathType Leaf)) { return $null }
    return ((& $Binary --version | Select-Object -Last 1).Trim() -replace '^.*\s', '')
}

function Assert-PeMachine {
    param([string] $Path, [int] $Subsystem)
    & python $ArchitectureScript --format pe --machine $ExpectedMachine --subsystem $Subsystem $Path
    if ($LASTEXITCODE -ne 0) { throw "PE identity check failed for $Path" }
}

function Add-ForcedRollbackFailure {
    param([string] $Path)
    $installer = New-Object -ComObject WindowsInstaller.Installer
    $database = $null
    try {
        $database = $installer.OpenDatabase($Path, 1)
        $queries = @(
            'INSERT INTO `CustomAction` (`Action`, `Type`, `Source`, `Target`) VALUES (''Honk300ForceRollback'', 19, '''', ''Intentional rollback smoke failure'')',
            # Sequence 4010 runs after InstallFiles (4000) and before InstallFinalize.
            'INSERT INTO `InstallExecuteSequence` (`Action`, `Condition`, `Sequence`) VALUES (''Honk300ForceRollback'', ''NOT Installed'', 4010)'
        )
        foreach ($query in $queries) {
            $view = $database.OpenView($query)
            try { $view.Execute() } finally { [void][Runtime.InteropServices.Marshal]::FinalReleaseComObject($view) }
        }
        $database.Commit()
    }
    finally {
        if ($null -ne $database) { [void][Runtime.InteropServices.Marshal]::FinalReleaseComObject($database) }
        [void][Runtime.InteropServices.Marshal]::FinalReleaseComObject($installer)
    }
}

$CurrentMsi = $null
$PreviousMsi = $null
try {
    $CurrentName = "honk300-$TargetTriple.msi"
    if ($CurrentMsiPath) {
        $CurrentMsi = (Resolve-Path -LiteralPath $CurrentMsiPath).Path
    } else {
        $CurrentMsi = Download-VerifiedReleaseAsset -ReleaseTag $Tag -Name $CurrentName
    }
    $PreviousMsi = Download-VerifiedReleaseAsset -ReleaseTag 'v0.2.1' -Name $CurrentName

    $SourceHash = $null
    if ($SourceBinaryPath) {
        $SourceBinaryPath = (Resolve-Path -LiteralPath $SourceBinaryPath).Path
        Assert-PeMachine -Path $SourceBinaryPath -Subsystem 3
        $SourceHash = (Get-FileHash -LiteralPath $SourceBinaryPath -Algorithm SHA256).Hash.ToLowerInvariant()
    }

    $AdministrativeStage = Join-Path $Root 'administrative-extraction'
    Require-MsiSuccess -Mode '/a' -Path $CurrentMsi -Extra @("TARGETDIR=`"$AdministrativeStage`"")
    $ExtractedBinaries = @(Get-ChildItem -LiteralPath $AdministrativeStage -Recurse -Filter honk300.exe -File)
    if ($ExtractedBinaries.Count -ne 1) {
        throw "administrative extraction expected one honk300.exe, got $($ExtractedBinaries.Count)"
    }
    $ExtractedBinary = $ExtractedBinaries[0].FullName
    Assert-PeMachine -Path $ExtractedBinary -Subsystem 3
    $ExtractedHash = (Get-FileHash -LiteralPath $ExtractedBinary -Algorithm SHA256).Hash.ToLowerInvariant()
    if ($SourceHash -and $ExtractedHash -ne $SourceHash) {
        throw 'MSI-extracted binary does not match the exact qualified build'
    }
    $ExtractedLaunchers = @(
        Get-ChildItem -LiteralPath $AdministrativeStage -Recurse -Filter honk300-app.exe -File
    )
    if ($ExtractedLaunchers.Count -ne 1) {
        throw "administrative extraction expected one honk300-app.exe, got $($ExtractedLaunchers.Count)"
    }
    $ExtractedLauncher = $ExtractedLaunchers[0].FullName
    Assert-PeMachine -Path $ExtractedLauncher -Subsystem 2
    $ExtractedLauncherHash = (
        Get-FileHash -LiteralPath $ExtractedLauncher -Algorithm SHA256
    ).Hash.ToLowerInvariant()

    foreach ($package in @($CurrentMsi, $PreviousMsi)) {
        $cleanup = Start-Msi -Mode '/x' -Path $package
        if ($cleanup.ExitCode -notin @(0, 1605, 1614, 3010)) {
            throw "pre-smoke cleanup failed with $($cleanup.ExitCode)"
        }
    }

    Require-MsiSuccess -Mode '/i' -Path $PreviousMsi
    if ((Reported-Version) -ne '0.2.1') { throw 'v0.2.1 did not install before rollback smoke' }

    $FailingMsi = Join-Path $Root 'honk300-forced-rollback.msi'
    Copy-Item -LiteralPath $CurrentMsi -Destination $FailingMsi
    Add-ForcedRollbackFailure -Path $FailingMsi
    $failedUpgrade = Start-Msi -Mode '/i' -Path $FailingMsi
    if ($failedUpgrade.ExitCode -in @(0, 3010)) { throw 'forced rollback upgrade unexpectedly succeeded' }
    if ((Reported-Version) -ne '0.2.1') {
        Get-Content -LiteralPath $failedUpgrade.Log -Tail 150 -ErrorAction SilentlyContinue | Write-Error
        throw 'failed upgrade did not restore v0.2.1'
    }

    Require-MsiSuccess -Mode '/i' -Path $CurrentMsi
    if ((Reported-Version) -ne $Version) { throw "upgrade did not install $Version" }
    Assert-PeMachine -Path $Binary -Subsystem 3
    Assert-PeMachine -Path $AppLauncher -Subsystem 2
    $InstalledHash = (Get-FileHash -LiteralPath $Binary -Algorithm SHA256).Hash.ToLowerInvariant()
    if ($InstalledHash -ne $ExtractedHash) {
        throw 'installed binary does not match the administratively extracted MSI payload'
    }
    $InstalledLauncherHash = (
        Get-FileHash -LiteralPath $AppLauncher -Algorithm SHA256
    ).Hash.ToLowerInvariant()
    if ($InstalledLauncherHash -ne $ExtractedLauncherHash) {
        throw 'installed app launcher does not match the administratively extracted MSI payload'
    }
    Require-MsiSuccess -Mode '/fa' -Path $CurrentMsi
    $RepairedHash = (Get-FileHash -LiteralPath $Binary -Algorithm SHA256).Hash.ToLowerInvariant()
    if ($RepairedHash -ne $ExtractedHash) { throw 'MSI repair changed the installed executable' }
    $RepairedLauncherHash = (
        Get-FileHash -LiteralPath $AppLauncher -Algorithm SHA256
    ).Hash.ToLowerInvariant()
    if ($RepairedLauncherHash -ne $ExtractedLauncherHash) {
        throw 'MSI repair changed the windowless app launcher'
    }

    @(
        "target=$TargetTriple",
        "pe_machine=$ExpectedMachine",
        "msi_sha256=$((Get-FileHash -LiteralPath $CurrentMsi -Algorithm SHA256).Hash.ToLowerInvariant())",
        "source_sha256=$SourceHash",
        "extracted_sha256=$ExtractedHash",
        "installed_sha256=$InstalledHash",
        "repaired_sha256=$RepairedHash"
        "extracted_launcher_sha256=$ExtractedLauncherHash"
        "installed_launcher_sha256=$InstalledLauncherHash"
        "repaired_launcher_sha256=$RepairedLauncherHash"
    ) | Set-Content -LiteralPath (Join-Path $OverlayEvidenceDirectory 'msi-identity.txt') -Encoding utf8

    & (Join-Path $PSScriptRoot 'smoke_windows_overlay.ps1') `
        -Binary $Binary `
        -EvidenceDirectory (Join-Path $OverlayEvidenceDirectory 'compositor') `
        -AllowUnavailableTrayHost:($TargetTriple -eq 'aarch64-pc-windows-msvc') `
        -AllowUnobservableTrayRecoveryHost:$AllowUnobservableTrayRecoveryHost

    $receiptPath = Join-Path $env:ProgramFiles 'honk300\install-receipt.json'
    $receipt = Get-Content -LiteralPath $receiptPath -Raw | ConvertFrom-Json
    if ($receipt.schema -ne 'honk300.install.v2' -or $receipt.origin -ne 'msi-global') {
        throw 'published MSI did not commit the protected Global MSI receipt'
    }
    if ($receipt.app_launcher.path -ne $AppLauncher -or $receipt.app_launcher.sha256 -ne $InstalledLauncherHash) {
        throw 'published MSI receipt did not bind the exact windowless app launcher'
    }
    if (-not (Test-Path (Join-Path $env:ProgramFiles 'honk300\current') -PathType Container)) {
        throw 'published MSI did not activate the stable current junction'
    }
    foreach ($alias in @('honk300.exe', 'honk.exe', 'goose.exe')) {
        $aliasPath = Join-Path $env:ProgramFiles "honk300\bin\$alias"
        if ((& $aliasPath --version | Select-Object -Last 1).Trim() -notmatch "$Version$") {
            throw "published MSI did not activate $alias"
        }
    }

    # Exercise the private terminal helper against the real public no-op path. Redirecting its
    # terminal streams lets the smoke inspect the invariant result while proving the helper stays
    # alive after rendering. Native menu tests own click routing; this lane owns released bytes,
    # protected-receipt immutability, runtime readiness, and the deliberate terminal hold.
    $ReceiptHashBeforeHelper = (Get-FileHash -LiteralPath $receiptPath -Algorithm SHA256).Hash
    $ReceiptTimeBeforeHelper = (Get-Item -LiteralPath $receiptPath).LastWriteTimeUtc.Ticks
    $HelperStdout = Join-Path $Root 'update-helper.stdout.txt'
    $HelperStderr = Join-Path $Root 'update-helper.stderr.txt'
    $HelperProcess = $null
    $HelperStdoutStream = $null
    $HelperStderrStream = $null
    $HelperStdoutCopy = $null
    $HelperStderrCopy = $null
    $HelperStarted = $false
    try {
        # The compositor/lifecycle smoke above ends after force-stopping and waiting for its final
        # runtime. Leave it stopped here so the UpToDate helper itself must resolve the receipt-owned
        # executable, launch the detached app path, and wait for IPC readiness.
        Write-Output 'Starting public update-helper no-op smoke from the stopped runtime state.'

        # The helper deliberately never exits after rendering. Start the exact binary through
        # ProcessStartInfo and drain both pipes asynchronously so its lifetime cannot prevent this
        # parent from enforcing the result deadline or inspecting the retained screen.
        $HelperStart = [Diagnostics.ProcessStartInfo]::new()
        $HelperStart.FileName = $Binary
        [void]$HelperStart.ArgumentList.Add('__control-surface-update')
        $HelperStart.UseShellExecute = $false
        $HelperStart.CreateNoWindow = $true
        $HelperStart.RedirectStandardOutput = $true
        $HelperStart.RedirectStandardError = $true
        $HelperProcess = [Diagnostics.Process]::new()
        $HelperProcess.StartInfo = $HelperStart
        if (-not $HelperProcess.Start()) {
            throw 'public update helper process did not start'
        }
        $HelperStarted = $true
        $HelperStdoutStream = [IO.FileStream]::new(
            $HelperStdout,
            [IO.FileMode]::Create,
            [IO.FileAccess]::Write,
            [IO.FileShare]::ReadWrite,
            1,
            [IO.FileOptions]::Asynchronous
        )
        $HelperStderrStream = [IO.FileStream]::new(
            $HelperStderr,
            [IO.FileMode]::Create,
            [IO.FileAccess]::Write,
            [IO.FileShare]::ReadWrite,
            1,
            [IO.FileOptions]::Asynchronous
        )
        $HelperStdoutCopy = $HelperProcess.StandardOutput.BaseStream.CopyToAsync($HelperStdoutStream)
        $HelperStderrCopy = $HelperProcess.StandardError.BaseStream.CopyToAsync($HelperStderrStream)
        Write-Output "Public update helper launched as PID $($HelperProcess.Id); waiting up to two minutes for its retained result screen."
        $HelperDeadline = [DateTime]::UtcNow.AddMinutes(2)
        do {
            Start-Sleep -Milliseconds 250
            $HelperProcess.Refresh()
            $HelperText = if (Test-Path -LiteralPath $HelperStdout) {
                Get-Content -LiteralPath $HelperStdout -Raw
            } else { '' }
            if ($HelperProcess.HasExited -and $HelperText -notmatch 'There was nothing to update\.') {
                $HelperError = if (Test-Path -LiteralPath $HelperStderr) {
                    Get-Content -LiteralPath $HelperStderr -Raw
                } else { '' }
                throw "public update helper exited before its result screen: $HelperError"
            }
        } until (
            $HelperText -match 'There was nothing to update\.' -or
            [DateTime]::UtcNow -ge $HelperDeadline
        )
        if ($HelperText -notmatch 'There was nothing to update\.' -or
            $HelperText -notmatch 'Honk300 is already up to date and running\.' -or
            $HelperText -notmatch 'HONK! ALL DONE' -or
            $HelperText -notmatch 'You may now close this window\.') {
            $HelperError = if (Test-Path -LiteralPath $HelperStderr) {
                Get-Content -LiteralPath $HelperStderr -Raw
            } else { '' }
            throw "public update helper did not render the explicit no-op result contract before its two-minute deadline: $HelperError"
        }
        Write-Output 'Public update helper rendered the retained no-op result screen.'
        $HelperProcess.Refresh()
        if ($HelperProcess.HasExited) {
            throw 'public update helper did not remain alive after rendering its result'
        }
        if ((& $Binary status | Out-String) -notmatch 'honk300: running') {
            throw 'public update helper did not preserve a ready runtime after a no-op'
        }
        $ReceiptHashAfterHelper = (Get-FileHash -LiteralPath $receiptPath -Algorithm SHA256).Hash
        $ReceiptTimeAfterHelper = (Get-Item -LiteralPath $receiptPath).LastWriteTimeUtc.Ticks
        if ($ReceiptHashAfterHelper -ne $ReceiptHashBeforeHelper -or
            $ReceiptTimeAfterHelper -ne $ReceiptTimeBeforeHelper) {
            throw 'public no-op update helper mutated the protected receipt'
        }
        @(
            "receipt_sha256=$ReceiptHashAfterHelper",
            "receipt_timestamp_ticks=$ReceiptTimeAfterHelper",
            "helper_pid=$($HelperProcess.Id)",
            'helper_result=up_to_date_restarted_and_held'
        ) | Set-Content -LiteralPath (Join-Path $OverlayEvidenceDirectory 'update-helper.txt') -Encoding utf8
    }
    finally {
        try {
            if ($HelperStarted) {
                $HelperProcess.Refresh()
                if (-not $HelperProcess.HasExited) {
                    try {
                        $HelperProcess.Kill($true)
                    }
                    catch [InvalidOperationException] {
                        # The helper may exit between Refresh and Kill; the bounded wait below
                        # still proves that no process remains.
                    }
                }
                if (-not $HelperProcess.WaitForExit(5000)) {
                    throw "public update helper process tree did not exit within five seconds (PID $($HelperProcess.Id))"
                }
            }
            foreach ($copy in @($HelperStdoutCopy, $HelperStderrCopy)) {
                if ($null -eq $copy) { continue }
                try {
                    if (-not $copy.Wait(5000)) {
                        Write-Warning 'public update helper output drain remained pending after process-tree cleanup; disposing its streams'
                    }
                }
                catch {
                    Write-Warning "public update helper output drain ended during process-tree cleanup: $($_.Exception.Message)"
                }
            }
        }
        finally {
            if ($null -ne $HelperStdoutStream) { $HelperStdoutStream.Dispose() }
            if ($null -ne $HelperStderrStream) { $HelperStderrStream.Dispose() }
            if ($null -ne $HelperProcess) { $HelperProcess.Dispose() }
            Stop-InstalledRuntimeBounded -Executable $Binary
        }
    }

    Require-MsiSuccess -Mode '/x' -Path $CurrentMsi
    if (Test-Path -LiteralPath $Binary -PathType Leaf) { throw 'MSI uninstall left honk300.exe behind' }
    if (Test-Path -LiteralPath $AppLauncher -PathType Leaf) { throw 'MSI uninstall left honk300-app.exe behind' }
    Write-Output "native Windows $TargetTriple $Tag rollback, slot upgrade, repair, compositor, update-helper, and uninstall smoke passed"
}
finally {
    foreach ($package in @($CurrentMsi, $PreviousMsi)) {
        if ($null -ne $package -and (Test-Path -LiteralPath $package)) {
            $cleanup = Start-Msi -Mode '/x' -Path $package
            if ($cleanup.ExitCode -notin @(0, 1605, 1614, 3010)) {
                Write-Warning "post-smoke cleanup returned $($cleanup.ExitCode)"
            }
        }
    }
    $MsiLogDirectory = Join-Path $OverlayEvidenceDirectory 'msi-logs'
    New-Item -ItemType Directory -Force -Path $MsiLogDirectory | Out-Null
    Copy-Item -Path (Join-Path $Root 'msi-*.log') -Destination $MsiLogDirectory -ErrorAction SilentlyContinue
    Remove-Item -LiteralPath $Root -Recurse -Force -ErrorAction SilentlyContinue
}
