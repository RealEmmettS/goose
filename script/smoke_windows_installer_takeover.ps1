[CmdletBinding()]
param(
    [Parameter(Mandatory = $true)] [string] $GlobalMsi,
    [Parameter(Mandatory = $true)] [string] $CorporateMsi,
    [Parameter(Mandatory = $true)] [string] $GlobalExe,
    [Parameter(Mandatory = $true)] [string] $CorporateExe,
    [Parameter(Mandatory = $true)] [string] $ExpectedVersion,
    [string] $EvidenceDirectory = 'target/windows-installer-takeover-evidence',
    [ValidateRange(30, 600)] [int] $ChildTimeoutSeconds = 180,
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
$progressPath = Join-Path $evidence 'progress.log'

function Write-TakeoverProgress([string] $Message) {
    $line = "$([DateTime]::UtcNow.ToString('o')) $Message"
    Write-Host "installer takeover: $Message"
    Add-Content -LiteralPath $progressPath -Value $line -Encoding utf8
}

function Capture-TakeoverTimeout([string] $Label) {
    $safeLabel = (($Label -replace '[^A-Za-z0-9.-]', '-').Trim('-').ToLowerInvariant())
    try {
        $allProcesses = @(Get-CimInstance Win32_Process)
        $msiProcessIds = @($allProcesses | Where-Object Name -eq 'msiexec.exe' | ForEach-Object ProcessId)
        $allProcesses | Where-Object {
            $_.Name -in @('msiexec.exe', 'honk300.exe', 'honk300-app.exe') -or
            $_.Name -like 'MSI*.tmp' -or
            $_.ParentProcessId -in $msiProcessIds -or
            $_.CommandLine -like '*honk300*'
        } | Select-Object ProcessId, ParentProcessId, Name, ExecutablePath, CommandLine |
            ConvertTo-Json -Depth 4 |
            Set-Content -LiteralPath (Join-Path $evidence "$safeLabel.processes.json") -Encoding utf8
    } catch {
        Add-Content -LiteralPath $progressPath -Value "timeout process capture failed: $($_.Exception.Message)" -Encoding utf8
    }
    try {
        @($globalRoot, $corporateRoot) | Where-Object { Test-Path -LiteralPath $_ } |
            ForEach-Object { Get-ChildItem -LiteralPath $_ -Force -Recurse -ErrorAction SilentlyContinue } |
            Select-Object FullName, Mode, Length, LinkTarget |
            ConvertTo-Json -Depth 4 |
            Set-Content -LiteralPath (Join-Path $evidence "$safeLabel.filesystem.json") -Encoding utf8
        foreach ($rootName in @(
            [pscustomobject]@{ Name = 'global'; Root = $globalRoot },
            [pscustomobject]@{ Name = 'corporate'; Root = $corporateRoot }
        )) {
            foreach ($journal in @('.slot-transaction.json', '.slot-committed.json', 'install-receipt.json', '.owner-cleanup-pending.json')) {
                $source = Join-Path $rootName.Root $journal
                if (Test-Path -LiteralPath $source -PathType Leaf) {
                    Copy-Item -LiteralPath $source -Destination (Join-Path $evidence "$safeLabel.$($rootName.Name).$($journal.TrimStart('.'))")
                }
            }
        }
    } catch {
        Add-Content -LiteralPath $progressPath -Value "timeout filesystem capture failed: $($_.Exception.Message)" -Encoding utf8
    }
    try {
        $machinePath = [string](Get-ItemPropertyValue `
            -LiteralPath 'HKLM:\SYSTEM\CurrentControlSet\Control\Session Manager\Environment' `
            -Name Path)
        $userPath = [string](Get-ItemPropertyValue `
            -LiteralPath 'HKCU:\Environment' `
            -Name Path `
            -ErrorAction SilentlyContinue)
        $globalBin = (Join-Path $globalRoot 'bin').TrimEnd('\')
        $corporateBin = (Join-Path $corporateRoot 'bin').TrimEnd('\')
        $contains = {
            param([string] $Value, [string] $Expected)
            return $Value.Split(';').Where({
                $_.Trim().TrimEnd('\').Equals($Expected, [StringComparison]::OrdinalIgnoreCase)
            }).Count -gt 0
        }
        [pscustomobject]@{
            machine_has_global = (& $contains $machinePath $globalBin)
            machine_has_corporate = (& $contains $machinePath $corporateBin)
            user_has_global = (& $contains $userPath $globalBin)
            user_has_corporate = (& $contains $userPath $corporateBin)
            machine_path = $machinePath
            user_path = $userPath
        } | ConvertTo-Json -Depth 3 |
            Set-Content -LiteralPath (Join-Path $evidence "$safeLabel.path.json") -Encoding utf8
    } catch {
        Add-Content -LiteralPath $progressPath -Value "timeout PATH capture failed: $($_.Exception.Message)" -Encoding utf8
    }
    try {
        @(Get-HonkRegistrations) |
            ConvertTo-Json -Depth 4 |
            Set-Content -LiteralPath (Join-Path $evidence "$safeLabel.registrations.json") -Encoding utf8
    } catch {
        Add-Content -LiteralPath $progressPath -Value "timeout registration capture failed: $($_.Exception.Message)" -Encoding utf8
    }
}

function Wait-CheckedProcess(
    [Diagnostics.Process] $Process,
    [string] $Label,
    [switch] $KillTree
) {
    if (-not $Process.WaitForExit($ChildTimeoutSeconds * 1000)) {
        Capture-TakeoverTimeout $Label
        try { $Process.Kill([bool]$KillTree) } catch {}
        try { $Process.WaitForExit() } catch {}
        throw "$Label timed out after $ChildTimeoutSeconds seconds"
    }
    # A second parameterless wait completes redirected stream event handling on .NET.
    $Process.WaitForExit()
}

function Get-HonkRegistrations {
    $items = @()
    $knownInnoKeys = @(
        '{5A94FBD0-DA02-4F63-9363-7D9CE0E280F5}_is1',
        '{A072F01B-0AE8-4ED9-B67F-845ADF7831F9}_is1'
    )
    foreach ($hive in @(
        [pscustomobject]@{ Name = 'HKLM'; Value = [Microsoft.Win32.RegistryHive]::LocalMachine },
        [pscustomobject]@{ Name = 'HKCU'; Value = [Microsoft.Win32.RegistryHive]::CurrentUser }
    )) {
        foreach ($view in @(
            [pscustomobject]@{ Name = '64'; Value = [Microsoft.Win32.RegistryView]::Registry64 },
            [pscustomobject]@{ Name = '32'; Value = [Microsoft.Win32.RegistryView]::Registry32 }
        )) {
            $base = [Microsoft.Win32.RegistryKey]::OpenBaseKey($hive.Value, $view.Value)
            try {
                $uninstall = $base.OpenSubKey('Software\Microsoft\Windows\CurrentVersion\Uninstall')
                if ($null -eq $uninstall) { continue }
                try {
                    foreach ($keyName in $uninstall.GetSubKeyNames()) {
                        $key = $uninstall.OpenSubKey($keyName)
                        if ($null -eq $key) { continue }
                        try {
                            $publisher = [string]$key.GetValue('Publisher', '')
                            $displayName = [string]$key.GetValue('DisplayName', '')
                            if (($publisher -eq 'Emmett S' -and
                                $displayName -in @('honk300', 'honk300 (Corporate Edition)')) -or
                                $keyName -in $knownInnoKeys) {
                                $items += [pscustomobject]@{
                                    hive = $hive.Name
                                    view = $view.Name
                                    key = $keyName
                                    display_name = $displayName
                                    publisher = $publisher
                                    install_location = [string]$key.GetValue('InstallLocation', '')
                                    uninstall_string = [string]$key.GetValue('UninstallString', '')
                                    quiet_uninstall_string = [string]$key.GetValue('QuietUninstallString', '')
                                    windows_installer = [int]$key.GetValue('WindowsInstaller', 0)
                                }
                            }
                        } finally {
                            $key.Dispose()
                        }
                    }
                } finally {
                    $uninstall.Dispose()
                }
            } finally {
                $base.Dispose()
            }
        }
    }
    # HKCU's uninstall inventory is shared between 32/64-bit registry views. Collapse only exact
    # logical duplicates; any differing publisher/root/command evidence remains independently
    # visible and therefore still fails the one-owner assertions below.
    return @($items |
        Group-Object hive, key, display_name, publisher, install_location, uninstall_string, quiet_uninstall_string, windows_installer |
        ForEach-Object { $_.Group | Sort-Object view -Descending | Select-Object -First 1 } |
        Sort-Object hive, key)
}

function Invoke-Checked([string] $File, [string[]] $Arguments, [string] $Label) {
    Write-TakeoverProgress "$Label started"
    $process = Start-Process -FilePath $File -ArgumentList $Arguments -WindowStyle Hidden -PassThru
    Wait-CheckedProcess $process $Label -KillTree
    if ($process.ExitCode -ne 0) {
        throw "$Label failed with exit $($process.ExitCode)"
    }
    Write-TakeoverProgress "$Label completed"
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
    $logName = (($Label -replace '[^A-Za-z0-9.-]', '-').Trim('-').ToLowerInvariant()) + '.msi.log'
    $logPath = Join-Path $evidence $logName
    Invoke-Checked $msiexec @('/i', "`"$Path`"", '/qn', '/norestart', '/l*v', "`"$logPath`"") $Label
}

function Install-Exe([string] $Path, [string] $Label) {
    Invoke-Checked $Path @('/VERYSILENT', '/SUPPRESSMSGBOXES', '/NORESTART') $Label
}

function Start-HeldOldProcess([string] $Binary) {
    Write-TakeoverProgress "held old process starting from $Binary"
    $start = [Diagnostics.ProcessStartInfo]::new()
    $start.FileName = $Binary
    $start.UseShellExecute = $false
    $start.CreateNoWindow = $true
    $start.RedirectStandardInput = $true
    $start.RedirectStandardOutput = $true
    $start.RedirectStandardError = $true
    $start.Environment['HONK300_INTERNAL_HOLD_LIFECYCLE_LEASE'] = '1'
    $process = [Diagnostics.Process]::Start($start)
    $readyTask = $process.StandardOutput.ReadLineAsync()
    if (-not $readyTask.Wait(30000)) {
        try { $process.Kill($true) } catch {}
        throw 'old process did not acquire its lifecycle lease within 30 seconds'
    }
    $ready = $readyTask.Result
    if ($ready -ne 'HONK300_INTERNAL_LIFECYCLE_LEASE_READY') {
        try { $process.Kill($true) } catch {}
        throw "old process did not acquire its lifecycle lease: $ready $($process.StandardError.ReadToEnd())"
    }
    Write-TakeoverProgress "held old process ready at PID $($process.Id)"
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
    Write-TakeoverProgress 'held old process stopped cleanly'
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
    $machineHasActive = (& $contains $machine $activeBin)
    $machineHasGlobal = (& $contains $machine $globalBin)
    $userHasActive = (& $contains $user $activeBin)
    $userHasCorporate = (& $contains $user $corporateBin)
    if ($Origin -in @('msi-global', 'exe-global')) {
        if (-not $machineHasActive -or $userHasCorporate) {
            Capture-TakeoverTimeout "$Origin-path-ownership-failed"
            throw "$Origin does not exclusively own the persisted public PATH (machine_has_active=$machineHasActive; user_has_corporate=$userHasCorporate)"
        }
    }
    else {
        if (-not $userHasActive -or $machineHasGlobal) {
            Capture-TakeoverTimeout "$Origin-path-ownership-failed"
            throw "$Origin does not exclusively own the persisted public PATH (user_has_active=$userHasActive; machine_has_global=$machineHasGlobal)"
        }
    }
}

function Finish-ConflictingOwnerCleanup([string] $Root, [string] $Origin, [string] $Label) {
    Write-TakeoverProgress "$Label cleanup retry started"
    $binary = Join-Path $Root 'bin\honk300.exe'
    $stdout = Join-Path $evidence "$Label.stdout.json"
    $stderr = Join-Path $evidence "$Label.stderr.txt"
    $process = Start-Process -FilePath $binary -ArgumentList @('update', '--json') `
        -WindowStyle Hidden `
        -RedirectStandardOutput $stdout -RedirectStandardError $stderr -PassThru
    Wait-CheckedProcess $process "$Label cleanup retry" -KillTree
    if ($process.ExitCode -ne 0) {
        Capture-TakeoverTimeout "$Label-cleanup-failed"
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
        Capture-TakeoverTimeout "$Label-registration-count-failed"
        throw "$Label expected exactly one Honk300 registration, found $($registrations.Count)"
    }
    $transitions.Add([pscustomobject]@{
        transition = $Label
        origin = $Origin
        old_process_survived = ($null -ne $script:held -and -not $script:held.HasExited)
        registration = $registrations[0]
        update_result = $result
    })
    Write-TakeoverProgress "$Label cleanup retry completed"
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
    if (@(Get-HonkRegistrations).Count -ne 0 -or @($activeState).Count -ne 0) {
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
        Write-TakeoverProgress 'final Global MSI cleanup started'
        $cleanup = Start-Process -FilePath $msiexec `
            -ArgumentList @('/x', "`"$globalMsiPath`"", '/qn', '/norestart') `
            -WindowStyle Hidden -PassThru
        Wait-CheckedProcess $cleanup 'final Global MSI cleanup' -KillTree
        if ($cleanup.ExitCode -notin @(0, 1605)) {
            Write-Error "final Global MSI cleanup failed with $($cleanup.ExitCode)"
        }
        Write-TakeoverProgress 'final Global MSI cleanup completed'
    }
}
