[CmdletBinding()]
param(
    [Parameter(Mandatory=$true)][string]$InstallRoot,
    [Parameter(Mandatory=$true)][string]$EvidenceDirectory
)
$ErrorActionPreference='Stop'
if ($env:GITHUB_ACTIONS -ne 'true') { throw 'The stale-receipt fixture requires a disposable CI host' }
Add-Type -AssemblyName UIAutomationClient
Add-Type -AssemblyName UIAutomationTypes
if (-not ('GooseAppIcons' -as [type])) { Add-Type -TypeDefinition @'
using System; using System.Runtime.InteropServices;
public static class GooseAppIcons {
 [DllImport("shell32.dll",CharSet=CharSet.Unicode)] public static extern uint ExtractIconEx(string path,int index,IntPtr large,IntPtr small,uint count);
 [DllImport("user32.dll",EntryPoint="GetClassLongPtrW")] public static extern IntPtr GetClassLongPtr(IntPtr hwnd,int index);
}
'@
}
$root=(Resolve-Path -LiteralPath $InstallRoot).Path
$evidence=[IO.Path]::GetFullPath($EvidenceDirectory)
[void][IO.Directory]::CreateDirectory($evidence)
$receipt=Get-Content -LiteralPath (Join-Path $root 'install-receipt.json') -Raw | ConvertFrom-Json
$binary=Join-Path $root 'bin/honk300.exe'
$launcher=Join-Path $root 'bin/honk300-app.exe'
$settings=Join-Path $root 'bin/honk300-settings.exe'
function Get-VerifiedIconPath([string]$Descriptor) {
    $iconPath=[Environment]::ExpandEnvironmentVariables(($Descriptor -replace ',\s*-?\d+$','').Trim('"'))
    if (-not $iconPath -or -not (Test-Path -LiteralPath $iconPath -PathType Leaf) -or
        [GooseAppIcons]::ExtractIconEx($iconPath,-1,[IntPtr]::Zero,[IntPtr]::Zero,0) -lt 1) {
        throw "The registered application icon is missing or cannot be read by Windows: '$Descriptor' -> '$iconPath'"
    }
    return $iconPath
}
foreach($file in @($binary,$launcher,$settings)) {
    if ([GooseAppIcons]::ExtractIconEx($file,-1,[IntPtr]::Zero,[IntPtr]::Zero,0) -lt 1) { throw "Missing embedded app icon: $file" }
}
$shortcut=$null
$shell=New-Object -ComObject WScript.Shell
foreach($programs in @([Environment]::GetFolderPath('CommonPrograms'),[Environment]::GetFolderPath('Programs'))) {
    foreach($group in @('Goose','honk300')) {
        $path=Join-Path $programs "$group/Goose.lnk"
        if (-not (Test-Path -LiteralPath $path)) { continue }
        $entry=$shell.CreateShortcut($path)
        if ($entry.TargetPath -eq $launcher -and $entry.Arguments -eq '--settings' -and $entry.IconLocation) { $shortcut=$path }
    }
}
if (-not $shortcut) { throw 'No Goose app-menu shortcut with verified controls target and explicit icon' }
[void](Get-VerifiedIconPath ($shell.CreateShortcut($shortcut).IconLocation))
$registrations=@(foreach($hive in @('HKLM:','HKCU:')) {
    Get-ChildItem "$hive/Software/Microsoft/Windows/CurrentVersion/Uninstall" -ErrorAction SilentlyContinue |
        Where-Object { $_.GetValue('Publisher','') -eq 'Emmett S' -and ([string]$_.GetValue('InstallLocation','')).TrimEnd('\') -eq $root.TrimEnd('\') -and $_.GetValue('DisplayName','') -in @('Goose','Goose (Corporate Edition)') } |
        ForEach-Object { [PSCustomObject]@{Key=$_.PSChildName;DisplayName=$_.GetValue('DisplayName','');DisplayVersion=$_.GetValue('DisplayVersion','');DisplayIcon=$_.GetValue('DisplayIcon','');WindowsInstaller=$_.GetValue('WindowsInstaller',0);IconDescriptor='';VerifiedIcon='';IconSource=''} }
})
if (-not $registrations) { throw 'Goose installed-app registration is missing' }
$msi=New-Object -ComObject WindowsInstaller.Installer
try {
    foreach($registration in $registrations) {
        if ($registration.DisplayVersion -ne $receipt.version) { throw 'Installed-app version does not match the protected receipt' }
        # MSI publishes its primary icon through ProductIcon. DisplayIcon is an EXE
        # uninstall convention and is not a required Windows Installer registry value.
        $descriptor=if($registration.WindowsInstaller -eq 1) {
            $registration.IconSource='Windows Installer ProductIcon'
            $msi.ProductInfo($registration.Key,'ProductIcon')
        } else {
            $registration.IconSource='Uninstall DisplayIcon'
            $registration.DisplayIcon
        }
        $registration.IconDescriptor=$descriptor
        $iconPath=Get-VerifiedIconPath $descriptor
        if ($registration.WindowsInstaller -eq 1) {
            $expectedIcon=Join-Path $PSScriptRoot '../Assets/UI/honk300-app.ico'
            if ((Get-FileHash -LiteralPath $iconPath).Hash -ne (Get-FileHash -LiteralPath $expectedIcon).Hash) { throw 'MSI published unexpected application artwork' }
        } elseif ([IO.Path]::GetFullPath($iconPath) -ne [IO.Path]::GetFullPath($launcher)) {
            throw 'EXE installed-app icon does not use its owned launcher'
        }
        $registration.VerifiedIcon=$iconPath
    }
} finally {
    $registrations | ConvertTo-Json -Depth 4 | Set-Content -LiteralPath (Join-Path $evidence 'registered-apps.json') -Encoding utf8
    [void][Runtime.InteropServices.Marshal]::FinalReleaseComObject($msi)
}

$cache=Join-Path $env:LOCALAPPDATA 'honk300/install-receipt.json'
[void][IO.Directory]::CreateDirectory((Split-Path -Parent $cache))
if (Test-Path -LiteralPath $cache) { $oldCache=[IO.File]::ReadAllBytes($cache) } else { $oldCache=$null }
$stale=$receipt | ConvertTo-Json -Depth 12 | ConvertFrom-Json
$stale.origin=if($receipt.origin -eq 'powershell') {'msi-global'} else {'powershell'}
$stale.installer_family=if($stale.origin -eq 'powershell') {'powershell'} else {'msi'}
$stale.channel=$stale.origin
$stale.PSObject.Properties.Remove('settings_app')
$config=Join-Path $evidence 'controls-config.toml'
[IO.File]::WriteAllText($config,"goose_config_version = 2`n",[Text.UTF8Encoding]::new($false))
$results=@()
try {
    [IO.File]::WriteAllText($cache,($stale | ConvertTo-Json -Depth 12),[Text.UTF8Encoding]::new($false))
    foreach($route in @('settings-command','bare-goose-command','app-menu-target')) {
        $before=@(Get-Process -Name honk300-settings -ErrorAction SilentlyContinue | ForEach-Object Id)
        $gui=$null
        $controller=$null
        try {
            $start=[Diagnostics.ProcessStartInfo]::new()
            $start.FileName=if($route -eq 'app-menu-target') {$launcher} elseif($route -eq 'bare-goose-command') {Join-Path $root 'bin/goose.exe'} else {$binary}
            if($route -eq 'settings-command') { foreach($arg in @('settings','--config',$config)) { [void]$start.ArgumentList.Add($arg) } }
            if($route -eq 'app-menu-target') { [void]$start.ArgumentList.Add('--settings') }
            $start.UseShellExecute=$false
            $start.CreateNoWindow=$true
            $start.RedirectStandardOutput=$true
            $start.RedirectStandardError=$true
            $controller=[Diagnostics.Process]::Start($start)
            $deadline=[DateTime]::UtcNow.AddSeconds(15)
            do {
                $gui=Get-Process -Name honk300-settings -ErrorAction SilentlyContinue | Where-Object { $_.Id -notin $before -and $_.MainWindowHandle -ne 0 } | Select-Object -First 1
                if($gui) { break }
                Start-Sleep -Milliseconds 100
            } while([DateTime]::UtcNow -lt $deadline)
            if (-not $gui) { throw "$route did not open the verified settings window with a stale receipt present" }
            if ($gui.MainWindowTitle -ne 'Goose') { throw "Wrong app title: $($gui.MainWindowTitle)" }
            if ((Get-FileHash -LiteralPath $gui.Path).Hash.ToLowerInvariant() -ne $receipt.settings_app.sha256) { throw 'Unexpected GUI image' }
            if ([GooseAppIcons]::GetClassLongPtr($gui.MainWindowHandle,-14) -eq [IntPtr]::Zero) { throw 'The real settings window has no app icon' }
            $window=[Windows.Automation.AutomationElement]::FromHandle($gui.MainWindowHandle)
            $field=[Windows.Automation.PropertyCondition]::new([Windows.Automation.AutomationElement]::NameProperty,'First wander (seconds)')
            do {
                $loaded=$window.FindFirst([Windows.Automation.TreeScope]::Descendants,$field)
                if($loaded) { break }
                Start-Sleep -Milliseconds 100
            } while([DateTime]::UtcNow -lt $deadline)
            if (-not $loaded) {
                $window.FindAll([Windows.Automation.TreeScope]::Descendants,[Windows.Automation.Condition]::TrueCondition) | ForEach-Object { $_.Current.Name } | Set-Content -LiteralPath (Join-Path $evidence "$route-failed-tree.txt")
                throw 'Native controls did not finish loading through Rust'
            }
            # The Rust launcher retains verification leases through its ten-second input-idle
            # bound. Keep the route within the existing overall deadline, including cold startup.
            $remaining=[Math]::Max(1,[int]($deadline-[DateTime]::UtcNow).TotalMilliseconds)
            if (-not $controller.WaitForExit($remaining) -or $controller.ExitCode -ne 0) { throw "$route controller failed or did not return" }
            $results+=@{route=$route;title=$gui.MainWindowTitle;settings_sha256=$receipt.settings_app.sha256;native_controls=$true;window_icon=$true;stale_receipt_ignored=$true}
        } finally {
            if($gui -and -not $gui.HasExited) { [void]$gui.CloseMainWindow(); if(-not $gui.WaitForExit(5000)) { $gui.Kill(); $gui.WaitForExit() } }
            if($controller) { if(-not $controller.HasExited) { $controller.Kill(); $controller.WaitForExit() }; $controller.Dispose() }
        }
    }
    @{ok=$true;origin=$receipt.origin;shortcut=$shortcut;checks=$results} | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath (Join-Path $evidence 'result.json') -Encoding utf8
} finally {
    if($null -eq $oldCache) { Remove-Item -LiteralPath $cache -ErrorAction SilentlyContinue } else { [IO.File]::WriteAllBytes($cache,$oldCache) }
}
