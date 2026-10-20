[CmdletBinding()]
param(
    [Parameter(Mandatory)] [Alias('PreviousMsi')] [string] $PreviousInstaller,
    [Parameter(Mandatory)] [Alias('CandidateMsi')] [string] $CandidateInstaller,
    [ValidateSet('msi', 'exe')] [string] $Family = 'msi',
    [Parameter(Mandatory)] [ValidateSet('global', 'corporate')] [string] $Edition,
    [Parameter(Mandatory)] [string] $ExpectedVersion,
    [string] $EvidenceDirectory = 'target/windows-startup-evidence',
    [switch] $ExerciseWizard,
    [switch] $AllowMachineMutation
)

$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest
if (-not $AllowMachineMutation -or $env:CI -ne 'true') {
    throw 'Startup installer qualification is restricted to an explicitly authorized disposable CI runner.'
}
$previous = (Resolve-Path -LiteralPath $PreviousInstaller).Path
$candidate = (Resolve-Path -LiteralPath $CandidateInstaller).Path
$evidence = [IO.Path]::GetFullPath((Join-Path (Get-Location) $EvidenceDirectory))
New-Item -ItemType Directory -Force -Path $evidence | Out-Null
$root = if ($Edition -eq 'global') { Join-Path $env:ProgramFiles 'honk300' } else { Join-Path $env:LOCALAPPDATA 'Programs/honk300' }
$exe = Join-Path $root 'bin/honk300.exe'
$receiptPath = Join-Path $root 'install-receipt.json'
$configPath = Join-Path $env:LOCALAPPDATA 'honk300/config.toml'
$hive = if ($Edition -eq 'global') { [Microsoft.Win32.RegistryHive]::LocalMachine } else { [Microsoft.Win32.RegistryHive]::CurrentUser }
$msiexec = Join-Path ([Environment]::GetFolderPath([Environment+SpecialFolder]::System)) 'msiexec.exe'
$records = [Collections.Generic.List[object]]::new()
$failure = $null
$activePackage = $null
$installer = New-Object -ComObject WindowsInstaller.Installer

function Invoke-Msi([string] $Package, [string] $Label, [string[]] $Extra = @(), [switch] $Uninstall, [switch] $ExpectFailure) {
    $operation = if ($Uninstall) { '/x' } else { '/i' }
    $log = Join-Path $evidence "$Label.msi.log"
    # /passive matches the already-published updater. Its command line supplies no feature list.
    $ui = if ($ExpectFailure) { '/qn' } else { '/passive' }
    $arguments = @($operation, "`"$Package`"", $ui, '/norestart', '/l*v', "`"$log`"") + $Extra
    Write-Host "Startup qualification: $Label"
    $process = Start-Process -FilePath $msiexec -ArgumentList $arguments -WindowStyle Hidden -PassThru
    try {
        if (-not $process.WaitForExit(180000)) {
            & taskkill.exe /PID $process.Id /T /F | Out-Null
            throw "$Label exceeded 180 seconds; see $log"
        }
        if ($ExpectFailure) {
            if ($process.ExitCode -ne 1603) { throw "$Label expected MSI failure 1603, got $($process.ExitCode); see $log" }
        } elseif ($process.ExitCode -ne 0) { throw "$Label failed with MSI exit $($process.ExitCode); see $log" }
    } finally { $process.Dispose() }
    if (-not $ExpectFailure) { $script:activePackage = if ($Uninstall) { $null } else { $Package } }
}

function Invoke-Installer([string] $Package, [string] $Label, [string[]] $Extra = @(), [switch] $Uninstall, [switch] $ExpectFailure) {
    if ($Family -eq 'msi') {
        Invoke-Msi $Package $Label $Extra -Uninstall:$Uninstall -ExpectFailure:$ExpectFailure
        return
    }
    $file = $Package
    if ($Uninstall) {
        $uninstallers = @(Get-ChildItem -LiteralPath $root -Filter 'unins*.exe' -File)
        if ($uninstallers.Count -ne 1) { throw 'Expected one owned EXE uninstaller.' }
        $file = $uninstallers[0].FullName
    }
    $log = Join-Path $evidence "$Label.inno.log"
    $arguments = @('/VERYSILENT', '/SUPPRESSMSGBOXES', '/NORESTART', "/LOG=`"$log`"") + $Extra
    Write-Host "Startup qualification: $Label"
    $process = Start-Process -FilePath $file -ArgumentList $arguments -WindowStyle Hidden -PassThru
    try {
        if (-not $process.WaitForExit(180000)) {
            & taskkill.exe /PID $process.Id /T /F | Out-Null
            throw "$Label exceeded 180 seconds; see $log"
        }
        if ($ExpectFailure) {
            if ($process.ExitCode -eq 0) { throw "$Label unexpectedly succeeded; see $log" }
        } elseif ($process.ExitCode -ne 0) { throw "$Label failed with EXE exit $($process.ExitCode); see $log" }
    } finally { $process.Dispose() }
}

function Feature-State([string] $Feature) {
    if ($Family -ne 'msi' -or $null -eq $activePackage) { return $null }
    $database = $installer.OpenDatabase($activePackage, 0)
    $view = $null
    $record = $null
    try {
        $view = $database.OpenView('SELECT `Value` FROM `Property` WHERE `Property` = ''ProductCode''')
        [void]$view.Execute()
        $record = $view.Fetch()
        if ($null -eq $record) { throw 'MSI product identity is missing.' }
        $productCode = $record.StringData(1)
        return $installer.FeatureState($productCode, $Feature)
    } finally {
        if ($null -ne $record) { [void][Runtime.InteropServices.Marshal]::FinalReleaseComObject($record) }
        if ($null -ne $view) { [void][Runtime.InteropServices.Marshal]::FinalReleaseComObject($view) }
        [void][Runtime.InteropServices.Marshal]::FinalReleaseComObject($database)
    }
}

function Invoke-Settings([object] $Command) {
    $start = [Diagnostics.ProcessStartInfo]::new($exe)
    $start.UseShellExecute = $false
    $start.CreateNoWindow = $true
    $start.RedirectStandardInput = $true
    $start.RedirectStandardOutput = $true
    $start.RedirectStandardError = $true
    $start.ArgumentList.Add('__settings-service')
    $process = [Diagnostics.Process]::Start($start)
    try {
        $output = $process.StandardOutput.ReadToEndAsync()
        $errors = $process.StandardError.ReadToEndAsync()
        $process.StandardInput.Write((@{protocol=1;request_id=1;command=$Command} | ConvertTo-Json -Depth 12 -Compress))
        $process.StandardInput.Close()
        if (-not $process.WaitForExit(60000)) {
            $process.Kill($true)
            throw 'Settings service exceeded 60 seconds.'
        }
        $errorText = $errors.GetAwaiter().GetResult()
        if ($process.ExitCode -ne 0) { throw "Settings service failed: $errorText" }
        $response = $output.GetAwaiter().GetResult() | ConvertFrom-Json
        if (-not $response.ok) { throw ($response.error | ConvertTo-Json -Compress) }
        if ($response.data.PSObject.Properties['warning'] -and $response.data.warning) { throw $response.data.warning }
        return $response.data
    } finally { $process.Dispose() }
}

function Set-Startup([bool] $Enabled) {
    $read = Invoke-Settings @{op='read'}
    Invoke-Settings @{op='save';revision=$read.revision;patch=@{'lifecycle.autostart_on_login'=$Enabled}} | Out-Null
}

function Read-State([string] $Label) {
    $receipt = Get-Content -LiteralPath $receiptPath -Raw | ConvertFrom-Json
    $base = [Microsoft.Win32.RegistryKey]::OpenBaseKey($hive, [Microsoft.Win32.RegistryView]::Registry64)
    try {
        $key = $base.OpenSubKey('Software\Microsoft\Windows\CurrentVersion\Run')
        try { $run = if ($null -eq $key) { $null } else { $key.GetValue('Honk300', $null) } }
        finally { if ($null -ne $key) { $key.Dispose() } }
    } finally { $base.Dispose() }
    $state = [ordered]@{
        label = $Label
        version = $receipt.version
        commit = $receipt.commit
        receipt_autostart = $receipt.autostart.enabled
        run = $run
        startup_feature = Feature-State 'Autostart'
        path_feature = Feature-State 'Environment'
        config_sha256 = if (Test-Path -LiteralPath $configPath) { (Get-FileHash -LiteralPath $configPath).Hash } else { $null }
        at = [DateTime]::UtcNow.ToString('o')
    }
    $records.Add($state)
    $state | ConvertTo-Json | Set-Content -LiteralPath (Join-Path $evidence "$Label.json") -Encoding utf8
    return $state
}

function Assert-Startup([object] $State, [bool] $Enabled) {
    $expectedRun = if ($Enabled) { '"' + (Join-Path $root 'bin\honk300-app.exe') + '"' } else { $null }
    if ($State.receipt_autostart -ne $Enabled -or $State.run -ne $expectedRun) {
        throw "$($State.label): expected startup=$Enabled, receipt=$($State.receipt_autostart), Run=$($State.run)"
    }
}

function Set-RunFixture([object] $Value, [Microsoft.Win32.RegistryValueKind] $Kind = 'String') {
    # Test fixtures only: this script is restricted to an authorized disposable runner.
    $base = [Microsoft.Win32.RegistryKey]::OpenBaseKey($hive, [Microsoft.Win32.RegistryView]::Registry64)
    try {
        $key = $base.CreateSubKey('Software\Microsoft\Windows\CurrentVersion\Run')
        try {
            if ($null -eq $Value) { $key.DeleteValue('Honk300', $false) }
            else { $key.SetValue('Honk300', $Value, $Kind) }
        } finally { $key.Dispose() }
    } finally { $base.Dispose() }
}

function Assert-RefusesForeignStartup([string[]] $ExplicitArguments) {
    $before = Read-State 'before-foreign-fixtures'
    $receiptHash = (Get-FileHash -LiteralPath $receiptPath).Hash
    foreach ($fixture in @(
        @{name='foreign-command';value='"C:\Unrelated\application.exe"';kind='String';message='belongs to another command'},
        @{name='unsupported-type';value=123;kind='DWord';message='unsupported type or encoding'}
    )) {
        try {
            Set-RunFixture $fixture.value $fixture.kind
            Invoke-Installer $candidate $fixture.name $ExplicitArguments -ExpectFailure
            $after = Read-State $fixture.name
            if ($after.run -ne $fixture.value -or (Get-FileHash -LiteralPath $receiptPath).Hash -ne $receiptHash -or
                $after.config_sha256 -ne $before.config_sha256) {
                throw "$($fixture.name): refused installation changed startup, receipt or configuration."
            }
            $suffix = if ($Family -eq 'msi') { 'msi' } else { 'inno' }
            $log = Get-Content -LiteralPath (Join-Path $evidence "$($fixture.name).$suffix.log") -Raw
            if (-not $log.Contains($fixture.message)) { throw "$($fixture.name): refusal did not report its ownership failure." }
        } finally { Set-RunFixture $before.run }
    }
}

function New-RollbackProbe {
    $probeDirectory = Join-Path $evidence 'rollback-input'
    New-Item -ItemType Directory -Force -Path $probeDirectory | Out-Null
    $path = Join-Path $probeDirectory ([IO.Path]::GetFileName($candidate))
    Copy-Item -LiteralPath $candidate -Destination $path
    $database = $installer.OpenDatabase($path, 1)
    try {
        # Fail after the old product is retired and the new slot/Run state is active,
        # while the MSI transaction and slot rollback action can still restore it.
        # This modifies a private test copy, never the candidate or public MSI.
        $sequences = @{}
        foreach ($action in @('ActivateWindowsSlot', 'CommitWindowsSlot', 'InstallFinalize')) {
            $view = $database.OpenView('SELECT `Sequence` FROM `InstallExecuteSequence` WHERE `Action` = ''' + $action + '''')
            $record = $null
            try {
                [void]$view.Execute()
                $record = $view.Fetch()
                if ($null -eq $record) { throw "Missing rollback-boundary action: $action" }
                $sequences[$action] = $record.IntegerData(1)
            } finally {
                if ($null -ne $record) { [void][Runtime.InteropServices.Marshal]::FinalReleaseComObject($record) }
                [void][Runtime.InteropServices.Marshal]::FinalReleaseComObject($view)
            }
        }
        $probeSequence = $sequences.CommitWindowsSlot + 1
        if ($probeSequence -le $sequences.ActivateWindowsSlot -or $probeSequence -ge $sequences.InstallFinalize) {
            throw 'No valid post-activation rollback probe boundary.'
        }
        $view = $database.OpenView('SELECT `Action` FROM `InstallExecuteSequence` WHERE `Sequence` = ' + $probeSequence)
        $record = $null
        try {
            [void]$view.Execute()
            $record = $view.Fetch()
            if ($null -ne $record) { throw 'Rollback probe sequence is already occupied.' }
        } finally {
            if ($null -ne $record) { [void][Runtime.InteropServices.Marshal]::FinalReleaseComObject($record) }
            [void][Runtime.InteropServices.Marshal]::FinalReleaseComObject($view)
        }
        $actionType = if ($Edition -eq 'global') { 3106 } else { 1058 }
        foreach ($sql in @(
            ('INSERT INTO `CustomAction` (`Action`, `Type`, `Source`, `Target`) VALUES (''GooseStartupRollbackProbe'', ' + $actionType + ', ''APPLICATIONFOLDER'', ''"[SystemFolder]WindowsPowerShell\v1.0\powershell.exe" -NoProfile -NonInteractive -ExecutionPolicy Bypass -File "[GOOSESTARTUPPROBESCRIPT]" -OutputPath "[GOOSESTARTUPPROBEOUTPUT]" -ReceiptPath "[APPLICATIONFOLDER]install-receipt.json"'')'),
            ('INSERT INTO `InstallExecuteSequence` (`Action`, `Condition`, `Sequence`) VALUES (''GooseStartupRollbackProbe'', ''NOT REMOVE~="ALL"'', ' + $probeSequence + ')')
        )) {
            $view = $database.OpenView($sql)
            try { [void]$view.Execute() } finally { [void][Runtime.InteropServices.Marshal]::FinalReleaseComObject($view) }
        }
        # Changed MSI bytes need a different package code even in a private fixture.
        $summary = $database.SummaryInformation(1)
        try {
            $probePackageCode = [Guid]::NewGuid().ToString('B').ToUpperInvariant()
            [void]$summary.GetType().InvokeMember('Property', [Reflection.BindingFlags]::SetProperty, $null, $summary, @(9, $probePackageCode))
            [void]$summary.Persist()
        } finally { [void][Runtime.InteropServices.Marshal]::FinalReleaseComObject($summary) }
        [void]$database.Commit()
    } finally { [void][Runtime.InteropServices.Marshal]::FinalReleaseComObject($database) }
    $probeScript = @'
param([string] $OutputPath, [string] $ReceiptPath)
$ErrorActionPreference = 'Stop'
$base = [Microsoft.Win32.RegistryKey]::OpenBaseKey([Microsoft.Win32.RegistryHive]::__HIVE__, [Microsoft.Win32.RegistryView]::Registry64)
try {
    $key = $base.OpenSubKey('Software\Microsoft\Windows\CurrentVersion\Run')
    try { $run = if ($null -eq $key) { $null } else { $key.GetValue('Honk300', $null) } }
    finally { if ($null -ne $key) { $key.Dispose() } }
} finally { $base.Dispose() }
$receipt = Get-Content -LiteralPath $ReceiptPath -Raw | ConvertFrom-Json
@{run=$run;version=$receipt.version;commit=$receipt.commit;at=[DateTime]::UtcNow.ToString('o')} |
    ConvertTo-Json | Set-Content -LiteralPath $OutputPath -Encoding utf8
exit 1
'@
    $probeScript.Replace('__HIVE__', $hive.ToString()) | Set-Content -LiteralPath (Join-Path $evidence 'rollback-probe.ps1') -Encoding utf8
    return $path
}

try {
    if (Test-Path -LiteralPath $receiptPath) { throw 'Disposable runner already has a Goose installation.' }
    Invoke-Installer $candidate 'fresh-default'
    $fresh = Read-State 'fresh-default'
    Assert-Startup $fresh $false
    Invoke-Installer $candidate 'remove-fresh' -Uninstall
    $enableArguments = if ($Family -eq 'msi') { @('ADDLOCAL=Binaries,Autostart', 'REMOVE=Environment') } else { @('/TASKS=autostart') }
    Invoke-Installer $previous 'explicit-enable' $enableArguments
    Invoke-Settings @{op='read'} | Out-Null
    Assert-Startup (Read-State 'explicit-enable') $true
    Set-Startup $false
    $before = Read-State 'app-disabled'
    Assert-Startup $before $false
    if ($Family -eq 'msi' -and ($before.startup_feature -ne 3 -or $before.path_feature -ne 2)) {
        throw 'The stale startup/PATH feature fixture was not established.'
    }
    Invoke-Installer $candidate 'upgrade-after-app-disabled'
    $afterInstall = Read-State 'after-upgrade-before-read'
    # Capture the settings import too, even when the installer already left the Run value wrong.
    Invoke-Settings @{op='read'} | Out-Null
    $afterRead = Read-State 'after-upgrade-after-read'
    Assert-Startup $afterInstall $false
    Assert-Startup $afterRead $false
    if ($afterRead.version -ne $ExpectedVersion) { throw 'Candidate version was not activated.' }
    if ($before.config_sha256 -ne $afterRead.config_sha256) { throw 'Upgrade/settings read changed the disabled saved preference.' }
    if ($Family -eq 'msi' -and $afterRead.path_feature -ne $before.path_feature) { throw 'Startup correction changed the unrelated PATH selection.' }

    # The reverse drift also matters: settings can opt in when MSI remembers absent.
    Set-Startup $true
    $enabled = Read-State 'app-enabled'
    $repairArguments = if ($Family -eq 'msi') { @('REINSTALL=ALL', 'REINSTALLMODE=vomus') } else { @('/MERGETASKS=desktopicon') }
    Invoke-Installer $candidate 'repair-preserves-enabled' $repairArguments
    Invoke-Settings @{op='read'} | Out-Null
    $enabledRepair = Read-State 'after-enabled-repair'
    Assert-Startup $enabledRepair $true
    if ($enabled.config_sha256 -ne $enabledRepair.config_sha256) { throw 'Repair changed the enabled saved preference.' }
    Set-Startup $false
    $disabled = Read-State 'disabled-before-repair'
    Invoke-Installer $candidate 'repair-preserves-disabled' $repairArguments
    Invoke-Settings @{op='read'} | Out-Null
    $disabledRepair = Read-State 'after-disabled-repair'
    Assert-Startup $disabledRepair $false
    if ($disabled.config_sha256 -ne $disabledRepair.config_sha256) { throw 'Repair changed the disabled saved preference.' }

    # Explicit installer intent still wins over the current preference.
    Invoke-Installer $candidate 'candidate-explicit-enable' $enableArguments
    Invoke-Settings @{op='read'} | Out-Null
    Assert-Startup (Read-State 'candidate-explicit-enable') $true
    $disableArguments = if ($Family -eq 'msi') { @('REMOVE=Autostart') } else { @('/TASKS=desktopicon') }
    Invoke-Installer $candidate 'candidate-explicit-disable' $disableArguments
    Invoke-Settings @{op='read'} | Out-Null
    Assert-Startup (Read-State 'candidate-explicit-disable') $false
    if ($Family -eq 'exe') {
        Invoke-Installer $candidate 'candidate-starred-enable' @('/MERGETASKS=*autostart')
        Invoke-Settings @{op='read'} | Out-Null
        Assert-Startup (Read-State 'candidate-starred-enable') $true
        Invoke-Installer $candidate 'candidate-merged-disable' @('/MERGETASKS=!autostart')
        Invoke-Settings @{op='read'} | Out-Null
        Assert-Startup (Read-State 'candidate-merged-disable') $false
    }
    Assert-RefusesForeignStartup $disableArguments

    # Use another real two-generation upgrade for app opt-in, with an unrelated option.
    Invoke-Installer $candidate 'remove-before-enabled-upgrade' -Uninstall
    Invoke-Installer $previous 'previous-default-off'
    Set-Startup $true
    $beforeEnabledUpgrade = Read-State 'enabled-before-upgrade'
    $unrelatedArguments = if ($Family -eq 'msi') { @('ADDLOCAL=Binaries,Environment') } else { @('/MERGETASKS=desktopicon') }
    Invoke-Installer $candidate 'upgrade-preserves-enabled' $unrelatedArguments
    Invoke-Settings @{op='read'} | Out-Null
    $afterEnabledUpgrade = Read-State 'enabled-after-upgrade'
    Assert-Startup $afterEnabledUpgrade $true
    if ($beforeEnabledUpgrade.config_sha256 -ne $afterEnabledUpgrade.config_sha256) { throw 'Upgrade changed the enabled saved preference.' }
    if ($Family -eq 'msi') {
        if ($afterEnabledUpgrade.commit -notmatch '^[0-9a-f]{40}$') { throw 'Candidate receipt commit is missing or invalid.' }
        $rollbackProbe = New-RollbackProbe
        Invoke-Installer $candidate 'remove-before-rollback-upgrades' -Uninstall
        foreach ($enabledBefore in @($true, $false)) {
            $label = if ($enabledBefore) { 'rollback-enabled' } else { 'rollback-disabled' }
            [string[]]$beforeArguments = if ($enabledBefore) { $enableArguments } else { @('ADDLOCAL=Binaries,Environment', 'REMOVE=Autostart') }
            Invoke-Installer $previous "prepare-$label" $beforeArguments
            Set-Startup $enabledBefore
            $beforeRollback = Read-State "before-$label"
            Assert-Startup $beforeRollback $enabledBefore
            $receiptHash = (Get-FileHash -LiteralPath $receiptPath).Hash
            [string[]]$oppositeArguments = if ($enabledBefore) { @('ADDLOCAL=Binaries', 'REMOVE=Autostart') } else { $enableArguments }
            $probeOutput = Join-Path $evidence ("$label-during-transaction-" + [Guid]::NewGuid().ToString('N') + '.json')
            $oppositeArguments += @(
                ('GOOSESTARTUPPROBESCRIPT="' + (Join-Path $evidence 'rollback-probe.ps1') + '"'),
                ('GOOSESTARTUPPROBEOUTPUT="' + $probeOutput + '"')
            )
            Invoke-Installer $rollbackProbe $label $oppositeArguments -ExpectFailure
            $log = Get-Content -LiteralPath (Join-Path $evidence "$label.msi.log") -Raw
            if ($log -notmatch 'GooseStartupRollbackProbe.*returned actual error code 1') {
                throw "$label did not execute the intended deferred failure."
            }
            $duringRollback = Get-Content -LiteralPath $probeOutput -Raw | ConvertFrom-Json
            $oppositeRun = if ($enabledBefore) { $null } else { '"' + (Join-Path $root 'bin\honk300-app.exe') + '"' }
            if ($duringRollback.run -ne $oppositeRun) { throw "$label never applied the opposite Run state before failing." }
            if ($duringRollback.version -ne $ExpectedVersion -or $duringRollback.commit -ne $afterEnabledUpgrade.commit) {
                throw "$label did not reach the candidate slot before failing."
            }
            $afterRollback = Read-State "after-$label"
            Assert-Startup $afterRollback $enabledBefore
            if ((Get-FileHash -LiteralPath $receiptPath).Hash -ne $receiptHash -or
                $afterRollback.config_sha256 -ne $beforeRollback.config_sha256) {
                throw "$label did not restore the receipt/configuration."
            }
            Invoke-Installer $previous "remove-after-$label" -Uninstall
        }
        Invoke-Installer $candidate 'restore-candidate-after-rollback'
    }
    if ($ExerciseWizard) {
        if ($Family -ne 'msi') { throw 'Full wizard qualification currently targets MSI.' }
        Set-Startup $false
        foreach ($choice in @('enable', 'disable')) {
            & (Join-Path $PSScriptRoot 'exercise_windows_startup_wizard.ps1') -Package $candidate -Choice $choice `
                -EvidenceDirectory (Join-Path $evidence "wizard-$choice") -AllowMachineMutation
            Invoke-Settings @{op='read'} | Out-Null
            Assert-Startup (Read-State "wizard-$choice") ($choice -eq 'enable')
        }
        Invoke-Installer $candidate 'before-wizard-repair-enable' $enableArguments
        Invoke-Settings @{op='read'} | Out-Null
        Set-Startup $false
        $beforeWizardRepair = Read-State 'before-wizard-repair'
        if ($beforeWizardRepair.startup_feature -ne 3) { throw 'Full-UI repair fixture did not retain a stale installed feature.' }
        & (Join-Path $PSScriptRoot 'exercise_windows_startup_wizard.ps1') -Package $candidate -Choice repair `
            -EvidenceDirectory (Join-Path $evidence 'wizard-repair') -AllowMachineMutation
        Invoke-Settings @{op='read'} | Out-Null
        $afterWizardRepair = Read-State 'after-wizard-repair'
        Assert-Startup $afterWizardRepair $false
        if ($beforeWizardRepair.config_sha256 -ne $afterWizardRepair.config_sha256) { throw 'Wizard repair changed the saved preference.' }
    }
} catch {
    $failure = $_.Exception.Message
} finally {
    [ordered]@{family=$Family;edition=$Edition;expected_version=$ExpectedVersion;failure=$failure;states=@($records.ToArray())} |
        ConvertTo-Json -Depth 8 | Set-Content -LiteralPath (Join-Path $evidence 'result.json') -Encoding utf8
}
if ($failure) { throw $failure }
Write-Host 'Startup preference regression passed.'
