[CmdletBinding()]
param(
    [Parameter(Mandatory = $true)][string] $Binary,
    [Parameter(Mandatory = $true)][string] $EvidenceDirectory
)

# Run on disposable hosted Windows desktops. This uses the OS UIA provider,
# independently of Native SDK's internal automation tree and input protocol.
$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest
Add-Type -AssemblyName UIAutomationClient
Add-Type -AssemblyName UIAutomationTypes
$binaryPath = (Resolve-Path -LiteralPath $Binary).Path
$evidence = [IO.Path]::GetFullPath($EvidenceDirectory)
New-Item -ItemType Directory -Force -Path $evidence | Out-Null
$config = Join-Path $evidence ('accessibility-' + [guid]::NewGuid().ToString('N') + '.toml')
[IO.File]::WriteAllText($config, "# Native OS accessibility fixture`ngoose_config_version = 2`n", [Text.UTF8Encoding]::new($false))
$process = $null
$window = $null

function Wait-For([scriptblock] $Check, [string] $Description) {
    $deadline = [DateTime]::UtcNow.AddSeconds(20)
    do {
        if ($process.HasExited) { throw "Settings exited while waiting for $Description" }
        if (& $Check) { return }
        Start-Sleep -Milliseconds 100
    } while ([DateTime]::UtcNow -lt $deadline)
    throw "Native accessibility did not provide $Description"
}

function Find-Named([string] $Name) {
    $condition = [Windows.Automation.PropertyCondition]::new([Windows.Automation.AutomationElement]::NameProperty, $Name)
    return $window.FindFirst([Windows.Automation.TreeScope]::Descendants, $condition)
}

function Invoke-Named([string] $Name) {
    Wait-For { $found = Find-Named $Name; $null -ne $found -and $found.Current.IsEnabled } $Name
    $element = Find-Named $Name
    $pattern = $element.GetCurrentPattern([Windows.Automation.InvokePattern]::Pattern)
    $pattern.Invoke()
}

try {
    $start = [Diagnostics.ProcessStartInfo]::new()
    $start.FileName = $binaryPath
    $start.Arguments = '--config "' + $config + '"'
    $start.UseShellExecute = $false
    $start.CreateNoWindow = $true
    $process = [Diagnostics.Process]::Start($start)
    Wait-For { $process.Refresh(); $process.MainWindowHandle -ne [IntPtr]::Zero } 'the native window'
    $window = [Windows.Automation.AutomationElement]::FromHandle($process.MainWindowHandle)
    Wait-For { $null -ne (Find-Named 'First wander (seconds)') } 'loaded settings'
    Invoke-Named 'Appearance'
    Wait-For { $null -ne (Find-Named 'Reduced motion') } 'the appearance page'
    $toggle = (Find-Named 'Reduced motion').GetCurrentPattern([Windows.Automation.TogglePattern]::Pattern)
    if ($toggle.Current.ToggleState -ne [Windows.Automation.ToggleState]::Off) { throw 'Fixture switch did not start off' }
    $toggle.Toggle()
    Wait-For { $toggle.Current.ToggleState -eq [Windows.Automation.ToggleState]::On } 'the changed switch state'
    Invoke-Named 'Save & apply'
    Wait-For { (Get-Content -LiteralPath $config -Raw) -match 'reduced_motion = true' } 'persisted switch state'
    Invoke-Named 'General'
    Wait-For { $null -ne (Find-Named 'First wander (seconds)') } 'the general page'
    # Both a label and a button carry the field name; select the interactive one.
    $condition = [Windows.Automation.AndCondition]::new(
        [Windows.Automation.PropertyCondition]::new([Windows.Automation.AutomationElement]::NameProperty, 'First wander (seconds)'),
        [Windows.Automation.PropertyCondition]::new([Windows.Automation.AutomationElement]::ControlTypeProperty, [Windows.Automation.ControlType]::Button))
    Wait-For { $found = $window.FindFirst([Windows.Automation.TreeScope]::Descendants, $condition); $null -ne $found -and $found.Current.IsEnabled } 'the enabled setting editor'
    $window.FindFirst([Windows.Automation.TreeScope]::Descendants, $condition).GetCurrentPattern([Windows.Automation.InvokePattern]::Pattern).Invoke()
    Wait-For { $null -ne (Find-Named 'Edit setting') } 'the editor dialog'
    if ($null -ne (Find-Named 'General')) { throw 'Modal exposed obscured page controls' }
    $condition = [Windows.Automation.PropertyCondition]::new([Windows.Automation.AutomationElement]::ControlTypeProperty, [Windows.Automation.ControlType]::Edit)
    $field = $window.FindFirst([Windows.Automation.TreeScope]::Descendants, $condition)
    if ($field.Current.Name -ne 'First wander (seconds)') { throw 'Editor has no accessible name' }
    $value = $field.GetCurrentPattern([Windows.Automation.ValuePattern]::Pattern)
    if ($value.Current.IsReadOnly -or $value.Current.Value -ne '20') { throw 'Editor value is unavailable or wrong' }
    $value.SetValue('25')
    Wait-For { $value.Current.Value -eq '25' } 'the edited value'
    Invoke-Named 'Apply to draft'
    Wait-For { $null -eq (Find-Named 'Edit setting') } 'dialog dismissal'
    Invoke-Named 'Save & apply'
    Wait-For { (Get-Content -LiteralPath $config -Raw) -match 'first_wander_time_seconds = 25' } 'the saved numeric value'
    Wait-For { $null -ne (Find-Named 'Saved. The goose will use these settings when it starts.') } 'the readable saved status'
    [ordered]@{
        schema = 'honk300.settings-uia-smoke.v1'
        binary = $binaryPath
        ok = $true
        checks = @('native-names', 'invoke-pattern', 'toggle-pattern', 'value-pattern', 'modal-isolation', 'save-readback')
    } | ConvertTo-Json | Set-Content -LiteralPath (Join-Path $evidence 'result.json') -Encoding utf8
} finally {
    if ($null -ne $process) {
        if (-not $process.HasExited) {
            $process.CloseMainWindow() | Out-Null
            if (-not $process.WaitForExit(10000)) { $process.Kill(); $process.WaitForExit() }
        }
        $process.Dispose()
    }
}
