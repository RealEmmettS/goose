[CmdletBinding()]
param(
    [Parameter(Mandatory)] [ValidateSet('x86_64-pc-windows-msvc', 'aarch64-pc-windows-msvc')] [string] $TargetTriple,
    [Parameter(Mandatory)] [string] $OutputDirectory
)
$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest
$source = (Resolve-Path -LiteralPath (Join-Path $PSScriptRoot '../wix/startup_action.c')).Path
$output = [IO.Path]::GetFullPath($OutputDirectory)
New-Item -ItemType Directory -Force -Path $output | Out-Null
$vswhere = Join-Path ${env:ProgramFiles(x86)} 'Microsoft Visual Studio/Installer/vswhere.exe'
$components = @('Microsoft.VisualStudio.Component.VC.Tools.x86.x64')
if ($TargetTriple -eq 'aarch64-pc-windows-msvc') { $components += 'Microsoft.VisualStudio.Component.VC.Tools.ARM64' }
$installation = & $vswhere -latest -products '*' -requires $components -property installationPath
if ($LASTEXITCODE -ne 0 -or -not $installation) { throw 'Visual C++ build tools were not found.' }
$vcvars = Join-Path $installation 'VC/Auxiliary/Build/vcvarsall.bat'
$architecture = if ($TargetTriple -eq 'aarch64-pc-windows-msvc') { 'x64_arm64' } else { 'x64' }
$dll = Join-Path $output 'honk300-startup-action.dll'
$object = Join-Path $output 'honk300-startup-action.obj'
$library = Join-Path $output 'honk300-startup-action.lib'
# Literal quoted build paths are passed to cmd only for the compiler's environment setup.
foreach ($path in @($source, $output, $vcvars)) {
    if ($path -match '["%\r\n]') { throw 'Unsupported compiler path characters.' }
}
$command = 'call "{0}" {1} && cl.exe /nologo /LD /O2 /W4 /WX /MT /guard:cf /Fo"{2}" "{3}" /link /OUT:"{4}" /IMPLIB:"{5}" /DYNAMICBASE /NXCOMPAT msi.lib advapi32.lib' -f $vcvars, $architecture, $object, $source, $dll, $library
& $env:ComSpec /d /s /c $command
if ($LASTEXITCODE -ne 0) { throw "Startup custom action compilation failed with exit $LASTEXITCODE." }
$machine = if ($TargetTriple -eq 'aarch64-pc-windows-msvc') { '0xAA64' } else { '0x8664' }
python (Join-Path $PSScriptRoot 'verify_binary_architecture.py') --format pe --machine $machine $dll
if ($LASTEXITCODE -ne 0) { throw 'Startup custom action architecture verification failed.' }
