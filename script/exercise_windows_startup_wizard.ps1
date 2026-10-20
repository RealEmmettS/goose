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
Add-Type -AssemblyName UIAutomationClientsideProviders
Add-Type -AssemblyName System.Windows.Forms
if (-not ('GooseInstallerWizardInput' -as [type])) {
$automationReferences = @([Windows.Automation.AutomationElement].Assembly.Location,
    [Windows.Automation.ClientSideProviderDescription].Assembly.Location,
    [UIAutomationClientsideProviders.UIAutomationClientSideProviders].Assembly.Location)
Add-Type -ReferencedAssemblies $automationReferences -TypeDefinition @'
using System;
using System.Runtime.InteropServices;
public static class GooseInstallerWizardInput {
    // UIA's default proxy loader walks reflected stack frames. A PowerShell dynamic
    // frame has no reflected type, so initialize from a non-inlined typed frame.
    [System.Runtime.CompilerServices.MethodImpl(System.Runtime.CompilerServices.MethodImplOptions.NoInlining)]
    public static void InitializeProviders() {
        System.Windows.Automation.ClientSettings.RegisterClientSideProviders(
            UIAutomationClientsideProviders.UIAutomationClientSideProviders.ClientSideProviderDescriptionTable);
    }
    [StructLayout(LayoutKind.Sequential)] public struct Point { public int X; public int Y; }
    [DllImport("user32.dll")] public static extern bool SetForegroundWindow(IntPtr window);
    [DllImport("user32.dll")] public static extern IntPtr GetForegroundWindow();
    [DllImport("user32.dll")] public static extern IntPtr WindowFromPoint(Point point);
    [DllImport("user32.dll")] public static extern uint GetWindowThreadProcessId(IntPtr window, out uint process);
    [DllImport("user32.dll")] public static extern bool SetCursorPos(int x, int y);
    [DllImport("user32.dll")] public static extern void mouse_event(uint flags, uint dx, uint dy, uint data, UIntPtr extra);
}
'@
}
[GooseInstallerWizardInput]::InitializeProviders()
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
$failureLocation = $null

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
        [ordered]@{name=$_.Current.Name;type=$_.Current.ControlType.ProgrammaticName;class=$_.Current.ClassName;id=$_.Current.AutomationId;enabled=$_.Current.IsEnabled;process=$_.Current.ProcessId;bounds=$_.Current.BoundingRectangle.ToString();patterns=@($_.GetSupportedPatterns() | ForEach-Object { $_.ProgrammaticName })}
    }) | ConvertTo-Json -Depth 4 | Set-Content -LiteralPath (Join-Path $evidence "$Name.json") -Encoding utf8
}

function Save-MenuDiagnostic {
    $focused = [Windows.Automation.AutomationElement]::FocusedElement
    $elements = [Windows.Automation.AutomationElement]::RootElement.FindAll(
        [Windows.Automation.TreeScope]::Descendants, [Windows.Automation.Condition]::TrueCondition)
    [ordered]@{
        focused = if ($null -eq $focused) { $null } else { [ordered]@{name=$focused.Current.Name;class=$focused.Current.ClassName;process=$focused.Current.ProcessId} }
        elements = @($elements | ForEach-Object {
            [ordered]@{name=$_.Current.Name;type=$_.Current.ControlType.ProgrammaticName;class=$_.Current.ClassName;process=$_.Current.ProcessId;bounds=$_.Current.BoundingRectangle.ToString();patterns=@($_.GetSupportedPatterns() | ForEach-Object { $_.ProgrammaticName })}
        })
    } | ConvertTo-Json -Depth 5 | Set-Content -LiteralPath (Join-Path $evidence 'feature-menu-desktop.json') -Encoding utf8
    $screen = [Windows.Forms.SystemInformation]::VirtualScreen
    $bitmap = [Drawing.Bitmap]::new($screen.Width, $screen.Height)
    $graphics = [Drawing.Graphics]::FromImage($bitmap)
    try {
        $graphics.CopyFromScreen($screen.Left, $screen.Top, 0, 0, $bitmap.Size)
        $bitmap.Save((Join-Path $evidence 'feature-menu-desktop.png'), [Drawing.Imaging.ImageFormat]::Png)
    } finally { $graphics.Dispose(); $bitmap.Dispose() }
}

try {
    $deadline = [DateTime]::UtcNow.AddSeconds(150)
    while ([DateTime]::UtcNow -lt $deadline) {
        $process.Refresh()
        if ($process.HasExited) { break }
        $windows = [Windows.Automation.AutomationElement]::RootElement.FindAll(
            [Windows.Automation.TreeScope]::Children, [Windows.Automation.Condition]::TrueCondition)
        $matches = @($windows | Where-Object {
            # A native window may disappear between enumeration and inspection.
            # Read its identity once and never pass an absent PID to Get-Process.
            if ($null -eq $_) { return $false }
            try {
                $windowPid = [int]$_.Current.ProcessId
                $windowTitle = [string]$_.Current.Name
            }
            catch [Windows.Automation.ElementNotAvailableException] { return $false }
            if ($windowPid -le 0 -or $windowTitle -notmatch '^(Goose|honk300).*Setup$') {
                return $false
            }
            $owner = Get-Process -Id $windowPid -ErrorAction SilentlyContinue
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
                if ($null -eq $selection -or -not $selection.Current.IsSelected) {
                    throw 'Login Autostart was not selected before opening its feature menu.'
                }
                [uint32]$keyboardOwner = 0
                [void][GooseInstallerWizardInput]::GetWindowThreadProcessId([GooseInstallerWizardInput]::GetForegroundWindow(), [ref]$keyboardOwner)
                $focused = [Windows.Automation.AutomationElement]::FocusedElement
                if ($keyboardOwner -ne $window.Current.ProcessId -or $null -eq $focused -or $focused.Current.ProcessId -ne $window.Current.ProcessId) {
                    throw 'The installer does not own keyboard focus for feature selection.'
                }
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
                    # MSI draws this popup itself: the native UIA provider can expose
                    # four invokable items with empty names. The retained CI screenshot
                    # establishes local-install first and unavailable last. Require that
                    # exact owned, visible layout; final Run/receipt assertions still apply.
                    $popups = @($menus | Where-Object {
                        $_.Current.ControlType -eq [Windows.Automation.ControlType]::Menu -and
                        $_.Current.ClassName -eq '#32768' -and -not $_.Current.IsOffscreen
                    })
                    if ($popups.Count -eq 1) {
                        $popupBounds = $popups[0].Current.BoundingRectangle
                        $popupItems = @($popups[0].FindAll([Windows.Automation.TreeScope]::Children,
                            [Windows.Automation.PropertyCondition]::new([Windows.Automation.AutomationElement]::ControlTypeProperty, [Windows.Automation.ControlType]::MenuItem)) |
                            Sort-Object { $_.Current.BoundingRectangle.Top })
                        $available = @($popupItems | Where-Object {
                            $invokePattern = $null
                            $itemBounds = $_.Current.BoundingRectangle
                            $_.Current.ProcessId -eq $window.Current.ProcessId -and
                            $_.Current.Name -eq '' -and $_.Current.IsEnabled -and -not $_.Current.IsOffscreen -and
                            $itemBounds.Width -gt 0 -and $itemBounds.Height -gt 0 -and
                            $popupBounds.Contains($itemBounds) -and
                            $_.TryGetCurrentPattern([Windows.Automation.InvokePattern]::Pattern, [ref]$invokePattern)
                        })
                        if ($popupItems.Count -eq 4 -and $available.Count -eq 4) {
                            $popupIdentity = [string]::Join(',', $popups[0].GetRuntimeId())
                            $focus = [Windows.Automation.AutomationElement]::FocusedElement
                            $popupFocused = $false
                            for ($ancestor = 0; $ancestor -lt 4 -and $null -ne $focus; $ancestor++) {
                                if ([string]::Join(',', $focus.GetRuntimeId()) -eq $popupIdentity) { $popupFocused = $true; break }
                                $focus = [Windows.Automation.TreeWalker]::RawViewWalker.GetParent($focus)
                            }
                            if (-not $popupFocused) { throw 'The native feature popup does not own the current focus.' }
                            for ($itemIndex = 1; $itemIndex -lt 4; $itemIndex++) {
                                if ($popupItems[$itemIndex].Current.BoundingRectangle.Top -lt $popupItems[$itemIndex - 1].Current.BoundingRectangle.Bottom) {
                                    throw 'The unnamed native feature menu has overlapping items.'
                                }
                            }
                            Save-MenuDiagnostic
                            $menuItem = if ($Choice -eq 'enable') { $popupItems[0] } else { $popupItems[3] }
                            $actions.Add('verified-unnamed-feature-menu')
                            break
                        }
                    }
                    Start-Sleep -Milliseconds 100
                } while ([DateTime]::UtcNow -lt $menuDeadline)
                if ($null -eq $menuItem) { Save-Tree 'missing-feature-menu'; Save-MenuDiagnostic; throw 'The native feature selection menu was not exposed.' }
                Invoke-Control $menuItem
                $actions.Add("startup-$Choice")
                $selected = $true
                Save-Tree 'after-feature-choice'
            } elseif (Invoke-Button 'Change') { Start-Sleep -Milliseconds 150; continue }
        }
        # VerifyReadyDlg uses Change for maintenance feature edits, as well as
        # MaintenanceTypeDlg's earlier Change button. Commit only after selection.
        if ($selected -and $Choice -ne 'repair' -and (Invoke-Button 'Change')) { Start-Sleep -Milliseconds 150; continue }
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
    $failureLocation = $_.ScriptStackTrace
    try { Save-Tree 'failure-tree' } catch { }
} finally {
    if (-not $process.HasExited) {
        & taskkill.exe /PID $process.Id /T /F | Out-Null
        [void]$process.WaitForExit(5000)
    }
    [ordered]@{choice=$Choice;failure=$failure;failure_location=$failureLocation;actions=@($actions.ToArray())} | ConvertTo-Json -Depth 4 |
        Set-Content -LiteralPath (Join-Path $evidence 'result.json') -Encoding utf8
    $process.Dispose()
}
if ($failure) { throw $failure }
