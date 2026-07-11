[CmdletBinding()]
param(
    [Parameter(Mandatory = $true)]
    [ValidatePattern('^v[0-9]+\.[0-9]+\.[0-9]+$')]
    [string] $Tag
)

$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest

$Repository = 'https://github.com/RealEmmettS/goose'
$Version = $Tag.Substring(1)
$Root = Join-Path $env:RUNNER_TEMP "honk300-live-msi-$([Guid]::NewGuid().ToString('N'))"
$MsiExec = Join-Path ([Environment]::GetFolderPath([Environment+SpecialFolder]::System)) 'msiexec.exe'
$Binary = Join-Path $env:ProgramFiles 'honk300\bin\honk300.exe'
$script:MsiInvocation = 0
New-Item -ItemType Directory -Force -Path $Root | Out-Null

function Download-VerifiedReleaseAsset {
    param([string] $ReleaseTag, [string] $Name)
    $path = Join-Path $Root $Name
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
    $CurrentName = 'honk300-x86_64-pc-windows-msvc.msi'
    $ArmName = 'honk300-aarch64-pc-windows-msvc.msi'
    $CurrentMsi = Download-VerifiedReleaseAsset -ReleaseTag $Tag -Name $CurrentName
    $ArmMsi = Download-VerifiedReleaseAsset -ReleaseTag $Tag -Name $ArmName

    $PreviousMsi = Join-Path $Root 'honk300-v0.2.1-x86_64-global.msi'
    Invoke-WebRequest -UseBasicParsing `
        "$Repository/releases/download/v0.2.1/honk300-x86_64-pc-windows-msvc.msi" `
        -OutFile $PreviousMsi -MaximumRetryCount 4 -RetryIntervalSec 2
    $previousHash = (Get-FileHash -LiteralPath $PreviousMsi -Algorithm SHA256).Hash.ToLowerInvariant()
    if ($previousHash -ne '9566f3cc4c97fd16b087f72f16aedf0f80e1044868f2c0694329b4462929e022') {
        throw "v0.2.1 MSI hash mismatch: $previousHash"
    }

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
    Require-MsiSuccess -Mode '/fa' -Path $CurrentMsi

    $downgrade = Start-Msi -Mode '/i' -Path $PreviousMsi
    if ($downgrade.ExitCode -in @(0, 3010)) { throw 'Downgrade unexpectedly succeeded' }
    if ((Reported-Version) -ne $Version) { throw 'rejected downgrade changed the installed version' }

    $ArmStage = Join-Path $Root 'arm64-administrative-extraction'
    Require-MsiSuccess -Mode '/a' -Path $ArmMsi -Extra @("TARGETDIR=`"$ArmStage`"")
    if (-not (Get-ChildItem -LiteralPath $ArmStage -Recurse -Filter honk300.exe)) {
        throw 'ARM64 administrative extraction did not contain honk300.exe'
    }

    Require-MsiSuccess -Mode '/x' -Path $CurrentMsi
    if (Test-Path -LiteralPath $Binary -PathType Leaf) { throw 'MSI uninstall left honk300.exe behind' }
    Write-Output "live Windows $Tag rollback, upgrade, repair, downgrade, ARM extraction, and uninstall smoke passed"
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
    Remove-Item -LiteralPath $Root -Recurse -Force -ErrorAction SilentlyContinue
}
