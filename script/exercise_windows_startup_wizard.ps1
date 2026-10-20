[CmdletBinding()]
param(
    [Parameter(Mandatory)] [string] $Package,
    [Parameter(Mandatory)] [ValidateSet('enable', 'disable', 'repair')] [string] $Choice,
    [Parameter(Mandatory)] [string] $EvidenceDirectory,
    [switch] $AllowMachineMutation
)
$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest
if (-not $AllowMachineMutation -or $env:CI -ne 'true') {
    throw 'Installer wizard qualification is restricted to an authorized disposable CI runner.'
}
Add-Type -AssemblyName UIAutomationClient
Add-Type -AssemblyName UIAutomationTypes
Add-Type -AssemblyName System.Windows.Forms
if (-not ('GooseInstallerWizardInput' -as [type])) {
Add-Type -TypeDefinition @'
using System;
using System.Runtime.InteropServices;
public static class GooseInstallerWizardInput {
    [StructLayout(LayoutKind.Sequential)] public struct Point { public int X; public int Y; }
    [DllImport("user32.dll")] public static extern bool SetForegroundWindow(IntPtr window);
    [DllImport("user32.dll")] public static extern IntPtr WindowFromPoint(Point point);
    [DllImport("user32.dll")] public static extern uint GetWindowThreadProcessId(IntPtr window, out uint process);
    [DllImport("user32.dll")] public static extern bool SetCursorPos(int x, int y);
    [DllImport("user32.dll")] public static extern void mouse_event(uint flags, uint dx, uint dy, uint data, UIntPtr extra);
}
'@
}
$packagePath = (Resolve-Path -LiteralPath $Package).Path
$evidence = [IO.Path]::GetFullPath($EvidenceDirectory)
New-Item -ItemType Directory -Force -Path $evidence | Out-Null
$log = Join-Path $evidence 'wizard.msi.log'
# This script only runs on disposable CI desktops; the full wizard must be visible.
$process = Start-Process -FilePath msiexec.exe -ArgumentList @('/i', "`"$packagePath`"", '/qf', '/norestart', '/l*v', "`"$log`"") -WindowStyle Normal -PassThru
$actions = [Collections.Generic.List[string]]::new()
$selected = $false
$window = $null
$failure = $null

function Find-Control([object] $Parent, [string] $Name) {
    # The native MSI provider exposes some named controls as Pane, including buttons.
    $condition = [Windows.Automation.PropertyCondition]::new([Windows.Automation.AutomationElement]::NameProperty, $Name)
    $matches = $Parent.FindAll([Windows.Automation.TreeScope]::Descendants, $condition)
    if ($matches.Count -gt 1) { throw "Ambiguous wizard control: $Name" }
    if ($matches.Count -eq 1) { return $matches[0] }
    return $null
}

function Click-Control([object] $Control) {
    if (-not $Control.Current.IsEnabled -or $Control.Current.IsOffscreen) { throw 'Wizard control is unavailable.' }
    [void][GooseInstallerWizardInput]::SetForegroundWindow([IntPtr]$window.Current.NativeWindowHandle)
    $bounds = $Control.Current.BoundingRectangle
    if ($bounds.IsEmpty -or $bounds.Width -le 0 -or $bounds.Height -le 0) { throw 'Wizard control has no visible bounds.' }
    $point = [GooseInstallerWizardInput+Point]::new()
    $point.X = [int]($bounds.Left + $bounds.Width / 2)
    $point.Y = [int]($bounds.Top + $bounds.Height / 2)
    $hit = [GooseInstallerWizardInput]::WindowFromPoint($point)
    [uint32]$ownerId = 0
    [void][GooseInstallerWizardInput]::GetWindowThreadProcessId($hit, [ref]$ownerId)
    if ($ownerId -ne $window.Current.ProcessId) { throw 'Wizard click would target another process.' }
    if (-not [GooseInstallerWizardInput]::SetCursorPos($point.X, $point.Y)) { throw 'Cannot position the CI pointer on the wizard.' }
    [GooseInstallerWizardInput]::mouse_event(2, 0, 0, 0, [UIntPtr]::Zero)
    [GooseInstallerWizardInput]::mouse_event(4, 0, 0, 0, [UIntPtr]::Zero)
}

function Invoke-Control([object] $Control) {
    $pattern = $null
    if ($Control.TryGetCurrentPattern([Windows.Automation.InvokePattern]::Pattern, [ref]$pattern)) { $pattern.Invoke() }
    else { Click-Control $Control }
}

function Invoke-Button([string] $Name) {
    $button = Find-Control $window $Name
    if ($null -eq $button -or -not $button.Current.IsEnabled) { return $false }
    $actions.Add($Name)
    Invoke-Control $button
    return $true
}

function Save-Tree([string] $Name) {
    if ($null -eq $window) { return }
    $tree = $window.FindAll([Windows.Automation.TreeScope]::Descendants, [Windows.Automation.Condition]::TrueCondition)
    @($tree | ForEach-Object {
        [ordered]@{name=$_.Current.Name;type=$_.Current.ControlType.ProgrammaticName;class=$_.Current.ClassName;id=$_.Current.AutomationId;enabled=$_.Current.IsEnabled;patterns=@($_.GetSupportedPatterns() | ForEach-Object { $_.ProgrammaticName })}
    }) | ConvertTo-Json -Depth 4 | Set-Content -LiteralPath (Join-Path $evidence "$Name.json") -Encoding utf8
}

try {
    $deadline = [DateTime]::UtcNow.AddSeconds(150)
    while ([DateTime]::UtcNow -lt $deadline) {
        $process.Refresh()
        if ($process.HasExited) { break }
        $windows = [Windows.Automation.AutomationElement]::RootElement.FindAll(
            [Windows.Automation.TreeScope]::Children, [Windows.Automation.Condition]::TrueCondition)
        $matches = @($windows | Where-Object {
            $owner = Get-Process -Id $_.Current.ProcessId -ErrorAction SilentlyContinue
            $_.Current.Name -match '^(Goose|honk300).*Setup$' -and
            $null -ne $owner -and $owner.ProcessName -eq 'msiexec'
        })
        if ($matches.Count -gt 1) { throw 'Multiple installer wizard windows are ambiguous.' }
        if ($matches.Count -eq 0) { Start-Sleep -Milliseconds 150; continue }
        $window = $matches[0]
        $license = Find-Control $window 'I accept the terms in the License Agreement'
        if ($null -ne $license) {
            $toggle = $null
            if ($license.TryGetCurrentPattern([Windows.Automation.TogglePattern]::Pattern, [ref]$toggle)) {
                if ($toggle.Current.ToggleState -ne [Windows.Automation.ToggleState]::On) { $toggle.Toggle() }
            } else {
                $next = Find-Control $window 'Next'
                if ($null -ne $next -and -not $next.Current.IsEnabled) { Click-Control $license }
            }
        }
        if ($Choice -eq 'repair' -and (Invoke-Button 'Repair')) {
            $selected = $true
        } elseif ($Choice -ne 'repair' -and -not $selected) {
            $tree = Find-Control $window 'Tree of selections'
            if ($null -ne $tree) {
                $items = $tree.FindAll([Windows.Automation.TreeScope]::Descendants, [Windows.Automation.Condition]::TrueCondition)
                $startup = @($items | Where-Object { $_.Current.Name -eq 'Login Autostart' })
                if ($startup.Count -ne 1) { Save-Tree 'missing-startup-item'; throw 'Startup feature was not uniquely exposed by the wizard.' }
                Save-Tree 'before-feature-choice'
                $selection = $null
                if ($startup[0].TryGetCurrentPattern([Windows.Automation.SelectionItemPattern]::Pattern, [ref]$selection)) {
                    $selection.Select()
                    $startup[0].SetFocus()
                } else { Click-Control $startup[0] }
                [Windows.Forms.SendKeys]::SendWait(' ')
                $menuDeadline = [DateTime]::UtcNow.AddSeconds(5)
                $menuItem = $null
                do {
                    $menus = [Windows.Automation.AutomationElement]::RootElement.FindAll(
                        [Windows.Automation.TreeScope]::Descendants,
                        [Windows.Automation.PropertyCondition]::new([Windows.Automation.AutomationElement]::ProcessIdProperty, $window.Current.ProcessId))
                    $pattern = if ($Choice -eq 'enable') { '^Will be installed on local hard drive' } else { '^Entire feature will be unavailable' }
                    $options = @($menus | Where-Object { $_.Current.Name -match $pattern })
                    if ($options.Count -eq 1) { $menuItem = $options[0]; break }
                    Start-Sleep -Milliseconds 100
                } while ([DateTime]::UtcNow -lt $menuDeadline)
                if ($null -eq $menuItem) { Save-Tree 'missing-feature-menu'; throw 'The native feature selection menu was not exposed.' }
                Invoke-Control $menuItem
                $actions.Add("startup-$Choice")
                $selected = $true
                Save-Tree 'after-feature-choice'
            } elseif (Invoke-Button 'Change') { Start-Sleep -Milliseconds 150; continue }
        }
        if (Invoke-Button 'Finish') { Start-Sleep -Milliseconds 150; continue }
        if (Invoke-Button 'Install') { Start-Sleep -Milliseconds 150; continue }
        [void](Invoke-Button 'Next')
        Start-Sleep -Milliseconds 150
    }
    if (-not $process.HasExited) { throw 'The full installer wizard exceeded 150 seconds.' }
    if ($process.ExitCode -ne 0) { throw "The full installer wizard failed with exit $($process.ExitCode)." }
    if (-not $selected) { throw 'The wizard completed without exercising the requested selection.' }
} catch {
    $failure = $_.Exception.Message
    try { Save-Tree 'failure-tree' } catch { }
} finally {
    if (-not $process.HasExited) {
        & taskkill.exe /PID $process.Id /T /F | Out-Null
        [void]$process.WaitForExit(5000)
    }
    [ordered]@{choice=$Choice;failure=$failure;actions=@($actions.ToArray())} | ConvertTo-Json -Depth 4 |
        Set-Content -LiteralPath (Join-Path $evidence 'result.json') -Encoding utf8
    $process.Dispose()
}
if ($failure) { throw $failure }
