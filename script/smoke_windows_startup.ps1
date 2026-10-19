[CmdletBinding()]
param(
    [Parameter(Mandatory)] [string] $PreviousMsi,
    [Parameter(Mandatory)] [string] $CandidateMsi,
    [Parameter(Mandatory)] [ValidateSet('global', 'corporate')] [string] $Edition,
    [Parameter(Mandatory)] [string] $ExpectedVersion,
    [string] $EvidenceDirectory = 'target/windows-startup-evidence',
    [switch] $AllowMachineMutation
)

$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest
if (-not $AllowMachineMutation -or $env:CI -ne 'true') {
    throw 'Startup installer qualification is restricted to an explicitly authorized disposable CI runner.'
}
$previous = (Resolve-Path -LiteralPath $PreviousMsi).Path
$candidate = (Resolve-Path -LiteralPath $CandidateMsi).Path
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

function Invoke-Msi([string] $Package, [string] $Label, [string[]] $Extra = @(), [switch] $Uninstall) {
    $operation = if ($Uninstall) { '/x' } else { '/i' }
    $log = Join-Path $evidence "$Label.msi.log"
    # /passive matches the already-published updater. Its command line supplies no feature list.
    $arguments = @($operation, "`"$Package`"", '/passive', '/norestart', '/l*v', "`"$log`"") + $Extra
    Write-Host "Startup qualification: $Label"
    $process = Start-Process -FilePath $msiexec -ArgumentList $arguments -WindowStyle Hidden -PassThru
    try {
        if (-not $process.WaitForExit(180000)) {
            & taskkill.exe /PID $process.Id /T /F | Out-Null
            throw "$Label exceeded 180 seconds; see $log"
        }
        if ($process.ExitCode -ne 0) { throw "$Label failed with MSI exit $($process.ExitCode); see $log" }
    } finally { $process.Dispose() }
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
        receipt_autostart = $receipt.autostart.enabled
        run = $run
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

try {
    if (Test-Path -LiteralPath $receiptPath) { throw 'Disposable runner already has a Goose installation.' }
    Invoke-Msi $previous 'fresh-default'
    $fresh = Read-State 'fresh-default'
    Assert-Startup $fresh $false
    Invoke-Msi $previous 'remove-fresh' -Uninstall
    Invoke-Msi $previous 'explicit-enable' @('ADDLOCAL=Binaries,Environment,Autostart')
    Invoke-Settings @{op='read'} | Out-Null
    Assert-Startup (Read-State 'explicit-enable') $true
    Set-Startup $false
    $before = Read-State 'app-disabled'
    Assert-Startup $before $false
    Invoke-Msi $candidate 'upgrade-after-app-disabled'
    $afterInstall = Read-State 'after-upgrade-before-read'
    # Capture the settings import too, even when the installer already left the Run value wrong.
    Invoke-Settings @{op='read'} | Out-Null
    $afterRead = Read-State 'after-upgrade-after-read'
    Assert-Startup $afterInstall $false
    Assert-Startup $afterRead $false
    if ($afterRead.version -ne $ExpectedVersion) { throw 'Candidate version was not activated.' }
    if ($before.config_sha256 -ne $afterRead.config_sha256) { throw 'Upgrade/settings read changed the disabled saved preference.' }
} catch {
    $failure = $_.Exception.Message
} finally {
    [ordered]@{edition=$Edition;expected_version=$ExpectedVersion;failure=$failure;states=@($records.ToArray())} |
        ConvertTo-Json -Depth 8 | Set-Content -LiteralPath (Join-Path $evidence 'result.json') -Encoding utf8
}
if ($failure) { throw $failure }
Write-Host 'Startup preference regression passed.'
