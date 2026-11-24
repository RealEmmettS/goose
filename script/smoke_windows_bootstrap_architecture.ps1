[CmdletBinding()]
param(
    [Parameter(Mandatory)] [ValidateSet('x86_64-pc-windows-msvc', 'aarch64-pc-windows-msvc')] [string] $Target,
    [string] $ExpectedRoot,
    [ValidateSet('native', 'x86', 'x64')] [string] $Probe
)
$ErrorActionPreference = 'Stop'
if ($Probe) {
    $template = Join-Path $PSScriptRoot 'honk300-installer.ps1.in'
    $errors = $null
    $syntax = [Management.Automation.Language.Parser]::ParseFile($template, [ref]$null, [ref]$errors)
    if ($errors) { throw $errors[0] }
    foreach ($name in 'Get-NativeWindowsArchitecture', 'Get-NativeProgramFiles') {
        $function = $syntax.Find({param($n) $n -is [Management.Automation.Language.FunctionDefinitionAst] -and $n.Name -eq $name}, $false)
        . ([scriptblock]::Create($function.Extent.Text))
    }
    if ($Probe -eq 'x86' -and [IntPtr]::Size -ne 4) { throw 'Expected a 32-bit host' }
    if ($Probe -eq 'x64' -and [Runtime.InteropServices.RuntimeInformation]::ProcessArchitecture -ne 'X64') {
        throw 'Expected the x64 emulation host'
    }
    $pattern = if ($Target.StartsWith('aarch64')) { '^(arm64|aarch64)$' } else { '^(amd64|x86_64|x64)$' }
    if ((Get-NativeWindowsArchitecture) -notmatch $pattern -or (Get-NativeProgramFiles) -ne $ExpectedRoot) {
        throw "The $Probe host selected an incorrect system architecture or Program Files location"
    }
    Write-Output "$Probe selected $Target under $ExpectedRoot"
    return
}
$ExpectedRoot = [Environment]::GetFolderPath('ProgramFiles')
function Test-Host([string]$Program, [string]$Kind) {
    & $Program -NoProfile -NonInteractive -ExecutionPolicy Bypass -File $PSCommandPath -Target $Target -ExpectedRoot $ExpectedRoot -Probe $Kind
    if ($LASTEXITCODE -ne 0) { throw "Installer selection failed under $Kind PowerShell" }
}
Test-Host (Get-Process -Id $PID).Path 'native'
Test-Host (Join-Path $env:SystemRoot 'SysWOW64\WindowsPowerShell\v1.0\powershell.exe') 'x86'
if ($Target.StartsWith('aarch64')) {
    $archive = Join-Path $env:RUNNER_TEMP 'powershell-7.4.13-x64.zip'
    $folder = Join-Path $env:RUNNER_TEMP 'goose-powershell-x64'
    Invoke-WebRequest 'https://github.com/PowerShell/PowerShell/releases/download/v7.4.13/PowerShell-7.4.13-win-x64.zip' -OutFile $archive -MaximumRetryCount 4
    if ((Get-FileHash $archive).Hash -ne '8fb52d2172d285b230c2857a90ba4dd28ecf6477ba4a91f91b6854a647b33b65') {
        throw 'Unexpected x64 PowerShell archive'
    }
    Expand-Archive -LiteralPath $archive -DestinationPath $folder
    Test-Host (Join-Path $folder 'pwsh.exe') 'x64'
}
