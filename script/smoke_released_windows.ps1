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

    Require-MsiSuccess -Mode '/x' -Path $CurrentMsi
    if (Test-Path -LiteralPath $Binary -PathType Leaf) { throw 'MSI uninstall left honk300.exe behind' }
    if (Test-Path -LiteralPath $AppLauncher -PathType Leaf) { throw 'MSI uninstall left honk300-app.exe behind' }
    Write-Output "native Windows $TargetTriple $Tag rollback, slot upgrade, repair, compositor, and uninstall smoke passed"
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
