[CmdletBinding()]
param(
    [Parameter(Mandatory = $true)]
    [string] $Binary,
    [string] $AppLauncher = '',
    [string] $EvidenceDirectory = "target/windows-slot-update-evidence",
    [string] $TargetTriple = "x86_64-pc-windows-msvc"
)

$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest

$binaryPath = (Resolve-Path -LiteralPath $Binary).Path
if (-not $AppLauncher) {
    $AppLauncher = Join-Path (Split-Path -Parent $binaryPath) 'honk300-app.exe'
}
$launcherPath = (Resolve-Path -LiteralPath $AppLauncher).Path
$launcherHash = (Get-FileHash -LiteralPath $launcherPath -Algorithm SHA256).Hash.ToLowerInvariant()
$versionOutput = (& $binaryPath --version | Select-Object -Last 1).Trim()
$version = ($versionOutput.Split()[-1] -replace '[+-].*$', '')
if ($version -notmatch '^[0-9]+\.[0-9]+\.[0-9]+$') {
    throw "could not resolve release version from $versionOutput"
}
$hash = (Get-FileHash -LiteralPath $binaryPath -Algorithm SHA256).Hash.ToLowerInvariant()
$evidence = [IO.Path]::GetFullPath((Join-Path (Get-Location) $EvidenceDirectory))
$root = Join-Path $evidence 'honk300'
$artifact = Join-Path $evidence 'honk300-installer.exe'
$lease = $null

function Read-PeSubsystem([string] $Path) {
    $bytes = [IO.File]::ReadAllBytes($Path)
    if ($bytes.Length -lt 0x40) { throw "PE is truncated: $Path" }
    $peOffset = [BitConverter]::ToInt32($bytes, 0x3c)
    $optional = $peOffset + 24
    if ($optional + 70 -ge $bytes.Length) { throw "PE optional header is truncated: $Path" }
    return [BitConverter]::ToUInt16($bytes, $optional + 68)
}

function Stage-Channel([string] $Channel) {
    $releaseBin = Join-Path $root "channels\$Channel\releases\$version-$TargetTriple\bin"
    New-Item -ItemType Directory -Path $releaseBin -Force | Out-Null
    foreach ($name in @('honk300.exe', 'honk.exe', 'goose.exe')) {
        Copy-Item -LiteralPath $binaryPath -Destination (Join-Path $releaseBin $name)
    }
    Copy-Item -LiteralPath $launcherPath -Destination (Join-Path $releaseBin 'honk300-app.exe')
}

function Invoke-Activation([string] $Origin, [string] $Commit) {
    & $binaryPath __windows-slot-activate `
        --root $root `
        --origin $Origin `
        --version $version `
        --tag "v$version" `
        --commit $Commit `
        --target $TargetTriple `
        --artifact-name 'honk300-installer.exe' `
        --artifact-path $artifact `
        --payload-sha256 $hash `
        --autostart false
    return $LASTEXITCODE
}

function Invoke-CompactActivation([string] $Origin, [string] $Commit) {
    & $binaryPath __wsa -r $root -o $Origin -c $Commit -a $artifact -l $launcherHash -u false
    return $LASTEXITCODE
}

function Assert-Active([string] $Origin, [string] $Channel) {
    $receipt = Get-Content -LiteralPath (Join-Path $root 'install-receipt.json') -Raw | ConvertFrom-Json
    if ($receipt.schema -ne 'honk300.install.v2' -or $receipt.origin -ne $Origin) {
        throw "protected receipt did not preserve $Origin"
    }
    if ($receipt.active_release -notlike "*\channels\$Channel\releases\$version-$TargetTriple") {
        throw "active release does not select $Channel"
    }
    foreach ($name in @('honk300.exe', 'honk.exe', 'goose.exe')) {
        $alias = Join-Path $root "bin\$name"
        $reported = (& $alias --version | Select-Object -Last 1).Trim()
        if ($reported -notmatch ([regex]::Escape($version) + '$')) {
            throw "$name reports $reported instead of $version"
        }
        if ((Get-FileHash -LiteralPath $alias -Algorithm SHA256).Hash.ToLowerInvariant() -ne $hash) {
            throw "$name does not resolve to the staged release bytes"
        }
    }
    $launcher = Join-Path $root 'bin\honk300-app.exe'
    if ((Get-FileHash -LiteralPath $launcher -Algorithm SHA256).Hash.ToLowerInvariant() -ne $launcherHash) {
        throw 'windowless app launcher does not resolve to the staged release bytes'
    }
    if ((Read-PeSubsystem $launcher) -ne 2) {
        throw 'windowless app launcher is not a GUI-subsystem PE'
    }
    if ($receipt.app_launcher.path -ne $launcher -or $receipt.app_launcher.sha256 -ne $launcherHash) {
        throw 'protected receipt does not bind the exact windowless app launcher'
    }
}

try {
    if (Test-Path -LiteralPath $evidence) {
        $resolved = (Resolve-Path -LiteralPath $evidence).Path
        $expectedParent = [IO.Path]::GetFullPath((Join-Path (Get-Location) 'target'))
        if (-not $resolved.StartsWith($expectedParent, [StringComparison]::OrdinalIgnoreCase)) {
            throw "refusing to replace evidence outside target: $resolved"
        }
        Remove-Item -LiteralPath $resolved -Recurse -Force
    }
    New-Item -ItemType Directory -Path $evidence -Force | Out-Null
    Copy-Item -LiteralPath $binaryPath -Destination $artifact

    Stage-Channel 'msi-global'
    $env:HONK300_TEST_WINDOWS_SLOT_FAIL_AT = 'after_legacy_retirement'
    $emptyFailure = Invoke-Activation 'msi-global' ('0' * 40)
    Remove-Item Env:HONK300_TEST_WINDOWS_SLOT_FAIL_AT -ErrorAction SilentlyContinue
    if ($emptyFailure -eq 0) { throw 'pre-selector fault injection unexpectedly succeeded' }
    foreach ($path in @('current', 'bin', 'install-receipt.json', '.slot-transaction.json')) {
        if (Test-Path -LiteralPath (Join-Path $root $path)) {
            throw "pre-selector rollback left $path behind"
        }
    }

    if ((Invoke-Activation 'msi-global' ('1' * 40)) -ne 0) {
        throw 'initial Global MSI activation failed'
    }
    & $binaryPath __windows-slot-commit --root $root
    if ($LASTEXITCODE -ne 0) { throw 'initial slot commit failed' }
    Assert-Active 'msi-global' 'msi-global'

    $oldAlias = Join-Path $root 'bin\honk300.exe'
    $start = [Diagnostics.ProcessStartInfo]::new()
    $start.FileName = $oldAlias
    $start.UseShellExecute = $false
    $start.CreateNoWindow = $true
    $start.RedirectStandardInput = $true
    $start.RedirectStandardOutput = $true
    $start.RedirectStandardError = $true
    $start.EnvironmentVariables['HONK300_INTERNAL_HOLD_LIFECYCLE_LEASE'] = '1'
    $lease = [Diagnostics.Process]::new()
    $lease.StartInfo = $start
    if (-not $lease.Start()) { throw 'could not start old-slot lease process' }
    if ($lease.StandardOutput.ReadLine() -ne 'HONK300_INTERNAL_LIFECYCLE_LEASE_READY') {
        throw "old-slot process did not become ready: $($lease.StandardError.ReadToEnd())"
    }

    Stage-Channel 'msi-corporate'
    $faultPoints = @(
        'after_current_junction',
        'after_bin_junction',
        'before_receipt_commit',
        'after_receipt_commit',
        'before_alias_verification'
    )
    foreach ($faultPoint in $faultPoints) {
        $env:HONK300_TEST_WINDOWS_SLOT_FAIL_AT = $faultPoint
        $failed = Invoke-Activation 'msi-corporate' ('2' * 40)
        Remove-Item Env:HONK300_TEST_WINDOWS_SLOT_FAIL_AT -ErrorAction SilentlyContinue
        if ($failed -eq 0) { throw "fault-injected activation at $faultPoint unexpectedly succeeded" }
        Assert-Active 'msi-global' 'msi-global'
        if ($lease.HasExited) { throw "old mapped release was terminated by failure at $faultPoint" }
        if (Test-Path -LiteralPath (Join-Path $root '.slot-transaction.json')) {
            throw "rollback at $faultPoint left transaction state behind"
        }
    }

    if ((Invoke-CompactActivation 'msi-corporate' ('3' * 40)) -ne 0) {
        throw 'Corporate MSI activation failed'
    }
    & $binaryPath __windows-slot-commit --root $root
    if ($LASTEXITCODE -ne 0) { throw 'Corporate MSI slot commit failed' }
    Assert-Active 'msi-corporate' 'msi-corporate'
    if ($lease.HasExited) { throw 'old mapped release was terminated by Corporate MSI activation' }

    Stage-Channel 'exe-global'
    if ((Invoke-Activation 'exe-global' ('4' * 40)) -ne 0) { throw 'Global EXE activation failed' }
    & $binaryPath __windows-slot-commit --root $root
    if ($LASTEXITCODE -ne 0) { throw 'Global EXE slot commit failed' }
    Assert-Active 'exe-global' 'exe-global'
    if ($lease.HasExited) { throw 'old mapped release was terminated by Global EXE activation' }

    Stage-Channel 'exe-corporate'
    if ((Invoke-Activation 'exe-corporate' ('5' * 40)) -ne 0) { throw 'Corporate EXE activation failed' }
    $env:HONK300_TEST_WINDOWS_SLOT_FAIL_AT = 'commit_cleanup'
    & $binaryPath __windows-slot-commit --root $root
    Remove-Item Env:HONK300_TEST_WINDOWS_SLOT_FAIL_AT -ErrorAction SilentlyContinue
    if ($LASTEXITCODE -eq 0) { throw 'post-commit cleanup fault unexpectedly succeeded' }
    Assert-Active 'exe-corporate' 'exe-corporate'
    if ($lease.HasExited) { throw 'old mapped release was terminated by pending Corporate EXE cleanup' }
    if (Test-Path -LiteralPath (Join-Path $root '.slot-transaction.json')) {
        throw 'post-commit cleanup failure left a rollbackable transaction journal'
    }
    if (-not (Test-Path -LiteralPath (Join-Path $root '.slot-committed.json'))) {
        throw 'post-commit cleanup failure did not retain its committed cleanup journal'
    }
    & $binaryPath __windows-slot-commit --root $root
    if ($LASTEXITCODE -ne 0) { throw 'Corporate EXE cleanup retry failed' }
    if ((Get-Item -LiteralPath (Join-Path $root 'bin')).LinkTarget -ne (Join-Path $root 'current\bin')) {
        throw 'public bin junction does not follow the neutral current selector'
    }
    if (Test-Path -LiteralPath (Join-Path $root '.slot-transaction.json')) {
        throw 'committed transaction state was not cleaned'
    }

    [ordered]@{
        schema = 'honk300.windows-slot-smoke.v1'
        version = $version
        target = $TargetTriple
        old_process_survived = $true
        rollback_verified = $true
        fault_points = @('after_legacy_retirement') + $faultPoints + @('commit_cleanup')
        exercised_origins = @('msi-global', 'msi-corporate', 'exe-global', 'exe-corporate')
        final_origin = 'exe-corporate'
        aliases = @('honk300.exe', 'honk.exe', 'goose.exe')
        app_launcher = 'honk300-app.exe'
    } | ConvertTo-Json -Depth 4 | Set-Content -LiteralPath (Join-Path $evidence 'result.json') -Encoding utf8
} finally {
    Remove-Item Env:HONK300_TEST_WINDOWS_SLOT_FAIL_AT -ErrorAction SilentlyContinue
    if ($null -ne $lease) {
        try {
            $lease.StandardInput.Close()
            if (-not $lease.WaitForExit(10000)) { $lease.Kill(); $lease.WaitForExit() }
        } finally {
            $lease.Dispose()
        }
    }
}
