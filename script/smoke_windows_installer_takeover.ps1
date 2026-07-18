[CmdletBinding()]
param(
    [Parameter(Mandatory = $true)] [string] $GlobalMsi,
    [Parameter(Mandatory = $true)] [string] $CorporateMsi,
    [Parameter(Mandatory = $true)] [string] $GlobalExe,
    [Parameter(Mandatory = $true)] [string] $CorporateExe,
    [Parameter(Mandatory = $true)] [string] $ExpectedVersion,
    [string] $EvidenceDirectory = 'target/windows-installer-takeover-evidence',
    [switch] $AllowMachineMutation
)

$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest
if (-not $AllowMachineMutation) {
    throw 'This disposable-runner qualification mutates machine/user installer state. Pass -AllowMachineMutation explicitly.'
}

$globalMsiPath = (Resolve-Path -LiteralPath $GlobalMsi).Path
$corporateMsiPath = (Resolve-Path -LiteralPath $CorporateMsi).Path
$globalExePath = (Resolve-Path -LiteralPath $GlobalExe).Path
$corporateExePath = (Resolve-Path -LiteralPath $CorporateExe).Path
$evidence = [IO.Path]::GetFullPath((Join-Path (Get-Location) $EvidenceDirectory))
$globalRoot = Join-Path $env:ProgramFiles 'honk300'
$corporateRoot = Join-Path $env:LOCALAPPDATA 'Programs\honk300'
$msiexec = Join-Path ([Environment]::GetFolderPath([Environment+SpecialFolder]::System)) 'msiexec.exe'
$held = $null
$transitions = [Collections.Generic.List[object]]::new()
New-Item -ItemType Directory -Force -Path $evidence | Out-Null

function Get-HonkRegistrations {
    $items = @()
    foreach ($base in @(
        'HKLM:\Software\Microsoft\Windows\CurrentVersion\Uninstall',
        'HKCU:\Software\Microsoft\Windows\CurrentVersion\Uninstall'
    )) {
        if (-not (Test-Path -LiteralPath $base)) { continue }
        $items += @(Get-ChildItem -LiteralPath $base | ForEach-Object {
            $value = Get-ItemProperty -LiteralPath $_.PSPath
            $publisher = $value.PSObject.Properties['Publisher']
            $displayName = $value.PSObject.Properties['DisplayName']
            if ($null -ne $publisher -and $null -ne $displayName -and
                $publisher.Value -eq 'Emmett S' -and
                $displayName.Value -in @('honk300', 'honk300 (Corporate Edition)')) {
                $location = $value.PSObject.Properties['InstallLocation']
                $windowsInstaller = $value.PSObject.Properties['WindowsInstaller']
                [pscustomobject]@{
                    key = $_.PSChildName
                    display_name = $displayName.Value
                    install_location = if ($null -eq $location) { '' } else { [string]$location.Value }
                    windows_installer = if ($null -eq $windowsInstaller) { 0 } else { [int]$windowsInstaller.Value }
                }
            }
        })
    }
    return @($items)
}

function Invoke-Checked([string] $File, [string[]] $Arguments, [string] $Label) {
    $process = Start-Process -FilePath $File -ArgumentList $Arguments -WindowStyle Hidden -Wait -PassThru
    if ($process.ExitCode -ne 0) {
        throw "$Label failed with exit $($process.ExitCode)"
    }
}

function Read-PeSubsystem([string] $Path) {
    $bytes = [IO.File]::ReadAllBytes($Path)
    if ($bytes.Length -lt 0x40) { throw "PE is truncated: $Path" }
    $peOffset = [BitConverter]::ToInt32($bytes, 0x3c)
    $optional = $peOffset + 24
    if ($optional + 70 -ge $bytes.Length) { throw "PE optional header is truncated: $Path" }
    return [BitConverter]::ToUInt16($bytes, $optional + 68)
}

function Install-Msi([string] $Path, [string] $Label) {
    Invoke-Checked $msiexec @('/i', "`"$Path`"", '/qn', '/norestart') $Label
}

function Install-Exe([string] $Path, [string] $Label) {
    Invoke-Checked $Path @('/VERYSILENT', '/SUPPRESSMSGBOXES', '/NORESTART') $Label
}

function Start-HeldOldProcess([string] $Binary) {
    $start = [Diagnostics.ProcessStartInfo]::new()
    $start.FileName = $Binary
    $start.UseShellExecute = $false
    $start.CreateNoWindow = $true
    $start.RedirectStandardInput = $true
    $start.RedirectStandardOutput = $true
    $start.RedirectStandardError = $true
    $start.Environment['HONK300_INTERNAL_HOLD_LIFECYCLE_LEASE'] = '1'
    $process = [Diagnostics.Process]::Start($start)
    $ready = $process.StandardOutput.ReadLine()
    if ($ready -ne 'HONK300_INTERNAL_LIFECYCLE_LEASE_READY') {
        try { $process.Kill($true) } catch {}
        throw "old process did not acquire its lifecycle lease: $ready $($process.StandardError.ReadToEnd())"
    }
    return $process
}

function Stop-HeldOldProcess {
    if ($null -eq $script:held) { return }
    $script:held.StandardInput.Close()
    if (-not $script:held.WaitForExit(10000)) {
        $script:held.Kill($true)
        throw 'held old process did not exit after its lease pipe closed'
    }
    if ($script:held.ExitCode -ne 0) { throw "held old process exited $($script:held.ExitCode)" }
    $script:held.Dispose()
    $script:held = $null
}

function Assert-Active([string] $Root, [string] $Origin) {
    $receiptPath = Join-Path $Root 'install-receipt.json'
    $receipt = Get-Content -LiteralPath $receiptPath -Raw | ConvertFrom-Json
    if ($receipt.schema -ne 'honk300.install.v2' -or $receipt.origin -ne $Origin) {
        throw "unexpected active receipt at $receiptPath"
    }
    foreach ($name in @('honk300.exe', 'honk.exe', 'goose.exe')) {
        $alias = Join-Path $Root "bin\$name"
        $reported = (& $alias --version | Select-Object -Last 1).Trim()
        if ($reported -notmatch "(^|\s)$([regex]::Escape($ExpectedVersion))$") {
            throw "$alias reports '$reported', expected $ExpectedVersion"
        }
    }
    $launcher = Join-Path $Root 'bin\honk300-app.exe'
    if (-not (Test-Path -LiteralPath $launcher -PathType Leaf) -or (Read-PeSubsystem $launcher) -ne 2) {
        throw "$Origin did not activate the windowless GUI-subsystem app launcher"
    }
    $launcherHash = (Get-FileHash -LiteralPath $launcher -Algorithm SHA256).Hash.ToLowerInvariant()
    if ($receipt.app_launcher.path -ne $launcher -or $receipt.app_launcher.sha256 -ne $launcherHash) {
        throw "$Origin receipt did not bind the exact windowless app launcher"
    }
}

function Assert-PublicPathOwner([string] $Root, [string] $Origin) {
    $machine = [string](Get-ItemPropertyValue -LiteralPath 'HKLM:\SYSTEM\CurrentControlSet\Control\Session Manager\Environment' -Name Path)
    $user = [string](Get-ItemPropertyValue -LiteralPath 'HKCU:\Environment' -Name Path -ErrorAction SilentlyContinue)
    $activeBin = (Join-Path $Root 'bin').TrimEnd('\')
    $globalBin = (Join-Path $globalRoot 'bin').TrimEnd('\')
    $corporateBin = (Join-Path $corporateRoot 'bin').TrimEnd('\')
    $contains = {
        param([string] $Value, [string] $Expected)
        return $Value.Split(';').Where({ $_.Trim().TrimEnd('\').Equals($Expected, [StringComparison]::OrdinalIgnoreCase) }).Count -gt 0
    }
    if ($Origin -in @('msi-global', 'exe-global')) {
        if (-not (& $contains $machine $activeBin) -or (& $contains $user $corporateBin)) {
            throw "$Origin does not exclusively own the persisted public PATH"
        }
    }
    else {
        if (-not (& $contains $user $activeBin) -or (& $contains $machine $globalBin)) {
            throw "$Origin does not exclusively own the persisted public PATH"
        }
    }
}

function Finish-ConflictingOwnerCleanup([string] $Root, [string] $Origin, [string] $Label) {
    $binary = Join-Path $Root 'bin\honk300.exe'
    $stdout = Join-Path $evidence "$Label.stdout.json"
    $stderr = Join-Path $evidence "$Label.stderr.txt"
    $process = Start-Process -FilePath $binary -ArgumentList @('update', '--json') `
        -WindowStyle Hidden `
        -RedirectStandardOutput $stdout -RedirectStandardError $stderr -Wait -PassThru
    if ($process.ExitCode -ne 0) {
        throw "$Label cleanup retry failed with $($process.ExitCode): $(Get-Content -LiteralPath $stdout -Raw)"
    }
    $lines = @(Get-Content -LiteralPath $stdout | Where-Object { $_.Trim() })
    if ($lines.Count -ne 1) { throw "$Label emitted $($lines.Count) stdout objects" }
    $result = $lines[0] | ConvertFrom-Json
    if (-not $result.success -or $result.cleanup_state -eq 'pending') {
        throw "$Label did not finish cleanup: $($lines[0])"
    }
    if (Test-Path -LiteralPath (Join-Path $Root '.owner-cleanup-pending.json')) {
        throw "$Label left the conflicting-owner journal behind"
    }
    Assert-Active $Root $Origin
    Assert-PublicPathOwner $Root $Origin
    if ($null -ne $script:held -and $script:held.HasExited) {
        throw "$Label terminated or replaced the held old process"
    }
    $registrations = @(Get-HonkRegistrations)
    if ($registrations.Count -ne 1) {
        throw "$Label expected exactly one Honk300 registration, found $($registrations.Count)"
    }
    $transitions.Add([pscustomobject]@{
        transition = $Label
        origin = $Origin
        old_process_survived = ($null -ne $script:held -and -not $script:held.HasExited)
        registration = $registrations[0]
        update_result = $result
    })
}

try {
    $activeState = @(
        (Join-Path $globalRoot 'bin'),
        (Join-Path $globalRoot 'install-receipt.json'),
        (Join-Path $corporateRoot 'bin'),
        (Join-Path $corporateRoot 'install-receipt.json')
    ) | Where-Object { Test-Path -LiteralPath $_ }
    # PowerShell unwraps zero or one pipeline results even when the function builds an array.
    # Re-wrap the invocation so StrictMode can always inspect Count on a real array.
    if (@(Get-HonkRegistrations).Count -ne 0 -or $activeState.Count -ne 0) {
        throw 'disposable runner is not clean; refusing to disturb a pre-existing Honk300 install'
    }

    Install-Msi $globalMsiPath 'Global MSI baseline'
    Assert-Active $globalRoot 'msi-global'

    $held = Start-HeldOldProcess (Join-Path $globalRoot 'bin\honk300.exe')
    Install-Exe $globalExePath 'Global EXE takeover'
    Finish-ConflictingOwnerCleanup $globalRoot 'exe-global' 'global-msi-to-global-exe'
    Stop-HeldOldProcess

    $held = Start-HeldOldProcess (Join-Path $globalRoot 'bin\honk300.exe')
    Install-Msi $corporateMsiPath 'Corporate MSI takeover'
    Finish-ConflictingOwnerCleanup $corporateRoot 'msi-corporate' 'global-exe-to-corporate-msi'
    Stop-HeldOldProcess

    $held = Start-HeldOldProcess (Join-Path $corporateRoot 'bin\honk300.exe')
    Install-Exe $corporateExePath 'Corporate EXE takeover'
    Finish-ConflictingOwnerCleanup $corporateRoot 'exe-corporate' 'corporate-msi-to-corporate-exe'
    Stop-HeldOldProcess

    $held = Start-HeldOldProcess (Join-Path $corporateRoot 'bin\honk300.exe')
    Install-Msi $globalMsiPath 'Global MSI reverse-scope takeover'
    Finish-ConflictingOwnerCleanup $globalRoot 'msi-global' 'corporate-exe-to-global-msi'
    Stop-HeldOldProcess

    $transitions | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath (Join-Path $evidence 'takeover.json') -Encoding utf8
}
finally {
    Stop-HeldOldProcess
    if (Test-Path -LiteralPath (Join-Path $globalRoot 'install-receipt.json')) {
        $cleanup = Start-Process -FilePath $msiexec `
            -ArgumentList @('/x', "`"$globalMsiPath`"", '/qn', '/norestart') `
            -WindowStyle Hidden -Wait -PassThru
        if ($cleanup.ExitCode -notin @(0, 1605)) {
            Write-Error "final Global MSI cleanup failed with $($cleanup.ExitCode)"
        }
    }
}
