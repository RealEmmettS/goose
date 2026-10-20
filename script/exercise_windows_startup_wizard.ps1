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
$packagePath = (Resolve-Path -LiteralPath $Package).Path
$evidence = [IO.Path]::GetFullPath($EvidenceDirectory)
New-Item -ItemType Directory -Force -Path $evidence | Out-Null
$log = Join-Path $evidence 'wizard.msi.log'
$process = Start-Process -FilePath msiexec.exe -ArgumentList @('/i', "`"$packagePath`"", '/qf', '/norestart', '/l*v', "`"$log`"") -WindowStyle Hidden -PassThru
$actions = [Collections.Generic.List[string]]::new()
$selected = $false
$window = $null
$failure = $null

function Find-Control([object] $Parent, [string] $Name, [object] $Type) {
    $condition = [Windows.Automation.AndCondition]::new(
        [Windows.Automation.PropertyCondition]::new([Windows.Automation.AutomationElement]::NameProperty, $Name),
        [Windows.Automation.PropertyCondition]::new([Windows.Automation.AutomationElement]::ControlTypeProperty, $Type))
    return $Parent.FindFirst([Windows.Automation.TreeScope]::Descendants, $condition)
}

function Invoke-Button([string] $Name) {
    $button = Find-Control $window $Name ([Windows.Automation.ControlType]::Button)
    if ($null -eq $button -or -not $button.Current.IsEnabled) { return $false }
    $actions.Add($Name)
    $button.GetCurrentPattern([Windows.Automation.InvokePattern]::Pattern).Invoke()
    return $true
}

function Save-Tree([string] $Name) {
    if ($null -eq $window) { return }
    $tree = $window.FindAll([Windows.Automation.TreeScope]::Descendants, [Windows.Automation.Condition]::TrueCondition)
    @($tree | ForEach-Object {
        [ordered]@{name=$_.Current.Name;type=$_.Current.ControlType.ProgrammaticName;id=$_.Current.AutomationId;enabled=$_.Current.IsEnabled}
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
        $license = Find-Control $window 'I accept the terms in the License Agreement' ([Windows.Automation.ControlType]::CheckBox)
        if ($null -ne $license) {
            $toggle = $license.GetCurrentPattern([Windows.Automation.TogglePattern]::Pattern)
            if ($toggle.Current.ToggleState -ne [Windows.Automation.ToggleState]::On) { $toggle.Toggle() }
        }
        if ($Choice -eq 'repair' -and (Invoke-Button 'Repair')) {
            $selected = $true
        } elseif ($Choice -ne 'repair' -and -not $selected) {
            $tree = Find-Control $window 'Tree of selections' ([Windows.Automation.ControlType]::Tree)
            if ($null -ne $tree) {
                $items = $tree.FindAll([Windows.Automation.TreeScope]::Descendants,
                    [Windows.Automation.PropertyCondition]::new([Windows.Automation.AutomationElement]::ControlTypeProperty, [Windows.Automation.ControlType]::TreeItem))
                $startup = @($items | Where-Object { $_.Current.Name -eq 'Login Autostart' })
                if ($startup.Count -ne 1) { Save-Tree 'missing-startup-item'; throw 'Startup feature was not uniquely exposed by the wizard.' }
                Save-Tree 'before-feature-choice'
                $startup[0].GetCurrentPattern([Windows.Automation.SelectionItemPattern]::Pattern).Select()
                $startup[0].SetFocus()
                [Windows.Forms.SendKeys]::SendWait(' ')
                $menuDeadline = [DateTime]::UtcNow.AddSeconds(5)
                $menuItem = $null
                do {
                    $menus = [Windows.Automation.AutomationElement]::RootElement.FindAll(
                        [Windows.Automation.TreeScope]::Descendants,
                        [Windows.Automation.PropertyCondition]::new([Windows.Automation.AutomationElement]::ControlTypeProperty, [Windows.Automation.ControlType]::MenuItem))
                    $pattern = if ($Choice -eq 'enable') { '^Will be installed on local hard drive' } else { '^Entire feature will be unavailable' }
                    $options = @($menus | Where-Object { $_.Current.Name -match $pattern })
                    if ($options.Count -eq 1) { $menuItem = $options[0]; break }
                    Start-Sleep -Milliseconds 100
                } while ([DateTime]::UtcNow -lt $menuDeadline)
                if ($null -eq $menuItem) { Save-Tree 'missing-feature-menu'; throw 'The native feature selection menu was not exposed.' }
                $menuItem.GetCurrentPattern([Windows.Automation.InvokePattern]::Pattern).Invoke()
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
