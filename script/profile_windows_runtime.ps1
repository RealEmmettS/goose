[CmdletBinding()]
param(
    [Parameter(Mandatory)][string] $Binary,
    [Parameter(Mandatory)][string] $EvidenceDirectory,
    [ValidateSet('Stopped', 'Walking', 'Leaves', 'Mud', 'Props', 'Soak', 'SettingsIdle', 'SettingsScroll')]
    [string] $Workload = 'Walking',
    [ValidateRange(0, 600)][int] $WarmupSeconds = 30,
    [ValidateRange(1, 7200)][int] $SampleSeconds = 120,
    [switch] $EnableFrameProfile,
    [string] $SettingsBinary,
    [ValidatePattern('^[A-Za-z0-9_.-]{1,80}$')]
    [string] $BackgroundProcessName
)

# A deliberately separate controller: no compilation, desktop calibration, screenshots,
# global input injection or edits to the installed application/configuration occur here.
$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest
if (-not $IsWindows) { throw 'This profiler requires Windows.' }
$binaryPath = (Resolve-Path -LiteralPath $Binary).Path
if ([IO.Path]::GetFileName($binaryPath) -ine 'honk300.exe' -or
    -not (Test-Path -LiteralPath (Join-Path ([IO.Path]::GetDirectoryName($binaryPath)) 'honk300-app.exe'))) {
    throw 'Use the direct honk300.exe in a release bin directory with its sibling honk300-app.exe, not an alias or selector.'
}
$evidenceRoot = [IO.Path]::GetFullPath($EvidenceDirectory)
if (Test-Path -LiteralPath $evidenceRoot) { throw 'Use a new evidence directory; prior samples are immutable.' }
New-Item -ItemType Directory -Path $evidenceRoot -Force | Out-Null
$configPath = Join-Path $evidenceRoot 'profile.toml'
$userConfig = Join-Path $env:LOCALAPPDATA 'honk300/config.toml'
$userHash = if (Test-Path -LiteralPath $userConfig) { (Get-FileHash -LiteralPath $userConfig).Hash } else { $null }
function Get-StartupSnapshot {
    $state = [ordered]@{}
    foreach ($hive in @('HKCU','HKLM')) {
        $entry = Get-ItemProperty -LiteralPath "${hive}:\Software\Microsoft\Windows\CurrentVersion\Run" -Name Honk300 -ErrorAction SilentlyContinue
        $state[$hive] = if ($null -ne $entry) { $entry.Honk300 } else { $null }
    }
    $parent = [IO.DirectoryInfo]::new([IO.Path]::GetDirectoryName($binaryPath))
    while ($null -ne $parent) {
        $receipt = Join-Path $parent.FullName 'install-receipt.json'
        if (Test-Path -LiteralPath $receipt) {
            $state.receipt_path=$receipt
            $state.receipt_sha256=(Get-FileHash -LiteralPath $receipt).Hash
            $state.receipt_modified_utc=(Get-Item -LiteralPath $receipt).LastWriteTimeUtc.ToString('O')
            break
        }
        $parent=$parent.Parent
    }
    return ($state | ConvertTo-Json -Compress)
}
$startupBefore = Get-StartupSnapshot
$configText = @'
goose_config_version = 2
[behavior]
first_wander_time_seconds = 0.1
can_attack_mouse = false
attack_randomly = false
[behaviors]
multi_monitor_chase = true
[safety]
no_mouse_steal = true
no_window_ride = true
pause_on_fullscreen = false
[audio]
enabled = false
[schedule]
quiet_hours_enabled = false
dnd_respect = false
seasonal = SEASONAL
autumn = true
[mischief]
perch_and_ride = false
collect_windows = COLLECT
collect_notes = true
collect_memes = true
'@
$configText = $configText.Replace('SEASONAL', $(if ($Workload -in @('Leaves','Soak')) { 'true' } else { 'false' }))
$configText = $configText.Replace('COLLECT', $(if ($Workload -eq 'Props') { 'true' } else { 'false' }))
[IO.File]::WriteAllText($configPath, $configText, [Text.UTF8Encoding]::new($false))
$configurationHash = (Get-FileHash -LiteralPath $configPath).Hash
$settingsWorkload = $Workload -in @('SettingsIdle','SettingsScroll')
$measuredPath = $binaryPath
if ($settingsWorkload) {
    if (-not $SettingsBinary) { $SettingsBinary=Join-Path ([IO.Path]::GetDirectoryName($binaryPath)) 'honk300-settings.exe' }
    $measuredPath=(Resolve-Path -LiteralPath $SettingsBinary).Path
}

Add-Type -TypeDefinition @'
using System;
using System.Collections.Generic;
using System.Runtime.InteropServices;
using System.Text;
public static class GooseProfileNative {
    [StructLayout(LayoutKind.Sequential)] public struct Rect { public int Left, Top, Right, Bottom; }
    public class Surface { public long hwnd; public int x,y,width,height; public string window_class; }
    private delegate bool EnumProc(IntPtr hwnd, IntPtr parameter);
    private delegate bool MonitorProc(IntPtr monitor,IntPtr dc,ref Rect rect,IntPtr parameter);
    [DllImport("user32.dll")] private static extern bool EnumDisplayMonitors(IntPtr dc,IntPtr clip,MonitorProc callback,IntPtr parameter);
    public static Surface[] Monitors() {
        var result=new List<Surface>();
        var prior=SetThreadDpiAwarenessContext(new IntPtr(-4));
        try { EnumDisplayMonitors(IntPtr.Zero,IntPtr.Zero,(IntPtr m,IntPtr dc,ref Rect r,IntPtr p)=>{
            result.Add(new Surface {x=r.Left,y=r.Top,width=r.Right-r.Left,height=r.Bottom-r.Top,window_class="monitor"});return true;
        },IntPtr.Zero); } finally {if(prior!=IntPtr.Zero)SetThreadDpiAwarenessContext(prior);}
        return result.ToArray();
    }
    [DllImport("user32.dll")] private static extern bool EnumWindows(EnumProc callback, IntPtr parameter);
    [DllImport("user32.dll")] private static extern bool EnumChildWindows(IntPtr parent, EnumProc callback, IntPtr parameter);
    [DllImport("user32.dll")] private static extern bool PostMessage(IntPtr hwnd, uint message, IntPtr wparam, IntPtr lparam);
    public static void Wheel(IntPtr root, uint pid, int delta) {
        uint owner; GetWindowThreadProcessId(root, out owner);
        if (owner != pid) throw new InvalidOperationException("Settings window owner changed.");
        IntPtr canvas=IntPtr.Zero;
        EnumChildWindows(root, (hwnd, parameter) => {
            var name=new StringBuilder(256); GetClassName(hwnd,name,name.Capacity);
            if(name.ToString()=="NativeSdkGpuSurface") canvas=hwnd;
            return true;
        }, IntPtr.Zero);
        if(canvas==IntPtr.Zero) throw new InvalidOperationException("No native settings canvas.");
        var prior=SetThreadDpiAwarenessContext(new IntPtr(-4));
        try {
            Rect rect; if(!GetWindowRect(canvas,out rect)) throw new InvalidOperationException("No canvas bounds.");
            int x=(rect.Left+rect.Right)/2, y=(rect.Top+rect.Bottom)/2;
            if(!PostMessage(canvas,0x020A,new IntPtr(delta<<16),new IntPtr((y<<16)|(x&65535))))
                throw new InvalidOperationException("Wheel input failed.");
        } finally {if(prior!=IntPtr.Zero)SetThreadDpiAwarenessContext(prior);}
    }
    [DllImport("user32.dll")] private static extern uint GetWindowThreadProcessId(IntPtr hwnd, out uint pid);
    [DllImport("user32.dll")] private static extern bool GetWindowRect(IntPtr hwnd, out Rect rect);
    [DllImport("user32.dll")] private static extern bool IsWindowVisible(IntPtr hwnd);
    [DllImport("user32.dll", CharSet=CharSet.Unicode)] private static extern int GetClassName(IntPtr hwnd, StringBuilder text, int length);
    [DllImport("user32.dll")] public static extern uint GetGuiResources(IntPtr process, uint flags);
    [DllImport("user32.dll")] private static extern IntPtr SetThreadDpiAwarenessContext(IntPtr value);
    public static Surface[] Surfaces(uint pid) {
        var result = new List<Surface>();
        var prior = SetThreadDpiAwarenessContext(new IntPtr(-4));
        try {
            EnumWindows((hwnd, parameter) => {
                uint owner; GetWindowThreadProcessId(hwnd, out owner);
                if (owner != pid || !IsWindowVisible(hwnd)) return true;
                var text = new StringBuilder(256); GetClassName(hwnd, text, text.Capacity);
                Rect rect;
                if (text.ToString() == "honk300_overlay" && GetWindowRect(hwnd, out rect))
                    result.Add(new Surface { hwnd=hwnd.ToInt64(), x=rect.Left, y=rect.Top,
                        width=rect.Right-rect.Left, height=rect.Bottom-rect.Top, window_class=text.ToString() });
                return true;
            }, IntPtr.Zero);
        } finally { if (prior != IntPtr.Zero) SetThreadDpiAwarenessContext(prior); }
        return result.ToArray();
    }
}
'@

$commands = [Collections.Generic.List[object]]::new()
$samples = [Collections.Generic.List[object]]::new()
$runtimeProcess = $null
$runtimeStartedAt = $null
$completed = $false
$failure = $null
$cleanupFailure = $null
$cleanupControlFailure = $null
$terminatedOwnedFallback = $false
$sampleStartUnixMs = $null
$sampleEndUnixMs = $null
$gpuAvailable = $true
$gpuSamples = [Collections.Generic.List[object]]::new()
$backgroundProcesses=@(if($BackgroundProcessName){Get-Process -Name $BackgroundProcessName -ErrorAction SilentlyContinue})
$backgroundPids=@($backgroundProcesses|ForEach-Object{$_.Id})
$oldProfile = $env:HONK300_WINDOWS_PROFILE
$oldDate = $env:HONK300_SMOKE_LOCAL_DATE
$oldCapture = $env:HONK300_WINDOWS_SMOKE_PRESENT
$env:HONK300_WINDOWS_SMOKE_PRESENT = $null
$env:HONK300_SMOKE_LOCAL_DATE = '20261019'
$env:HONK300_WINDOWS_PROFILE = if ($EnableFrameProfile) { Join-Path $evidenceRoot 'frames.json' } else { $null }

function Invoke-Controller([string[]] $Arguments) {
    $info = [Diagnostics.ProcessStartInfo]::new($binaryPath)
    $info.UseShellExecute = $false
    $info.CreateNoWindow = $true
    $info.RedirectStandardOutput = $true
    $info.RedirectStandardError = $true
    foreach ($argument in $Arguments) { $info.ArgumentList.Add($argument) }
    $watch = [Diagnostics.Stopwatch]::StartNew()
    $child = [Diagnostics.Process]::Start($info)
    try {
        $outTask = $child.StandardOutput.ReadToEndAsync()
        $errTask = $child.StandardError.ReadToEndAsync()
        if (-not $child.WaitForExit(30000)) { $child.Kill(); $child.WaitForExit(); throw 'Controller timed out.' }
        if (-not [Threading.Tasks.Task]::WaitAll(@($outTask,$errTask), 5000)) { throw 'Controller retained an output pipe.' }
        $output = $outTask.GetAwaiter().GetResult()
        $errors = $errTask.GetAwaiter().GetResult()
        $commands.Add([ordered]@{arguments=$Arguments;elapsed_ms=$watch.Elapsed.TotalMilliseconds;exit_code=$child.ExitCode;stdout=$output;stderr=$errors})
        if ($child.ExitCode -ne 0) { throw "Controller failed: $output $errors" }
        return $output
    } finally { $child.Dispose() }
}

function Assert-OwnedRuntime {
    if ($null -eq $runtimeProcess) { throw 'No owned runtime.' }
    $runtimeProcess.Refresh()
    if ($runtimeProcess.HasExited) { throw 'Measured runtime exited unexpectedly.' }
    $identity = Get-CimInstance Win32_Process -Filter "ProcessId=$($runtimeProcess.Id)"
    if ($null -eq $identity -or $identity.ExecutablePath -ne $measuredPath -or
        -not $identity.CommandLine.Contains($configPath) -or $runtimeProcess.StartTime -ne $runtimeStartedAt) {
        throw 'Measured runtime identity changed; do not send control commands.'
    }
}

try {
    if ((Invoke-Controller @('status')) -notmatch 'honk300: not running') { throw 'An existing runtime must remain untouched.' }
    $version = (Invoke-Controller @('--version')).Trim()
    $hostData = [ordered]@{
        binary=$measuredPath;sha256=(Get-FileHash -LiteralPath $measuredPath).Hash.ToLowerInvariant();version=$version
        controller=$binaryPath;controller_sha256=(Get-FileHash -LiteralPath $binaryPath).Hash.ToLowerInvariant()
        workload=$Workload;warmup_seconds=$WarmupSeconds;sample_seconds=$SampleSeconds
        config_sha256=$configurationHash.ToLowerInvariant();logical_processors=[Environment]::ProcessorCount
        started_utc=[DateTime]::UtcNow.ToString('O');frame_profile=[bool]$EnableFrameProfile
        adapters=@(Get-CimInstance Win32_VideoController | Select-Object Name,DriverVersion,CurrentHorizontalResolution,CurrentVerticalResolution,CurrentRefreshRate)
        monitors=@([GooseProfileNative]::Monitors())
        power_scheme=((& powercfg /getactivescheme | Out-String).Trim())
        processors=@(Get-CimInstance Win32_Processor | Select-Object Name,NumberOfCores,NumberOfLogicalProcessors)
        background_processes=@($backgroundProcesses|ForEach-Object{[ordered]@{name=$_.ProcessName;pid=$_.Id;started_utc=$_.StartTime.ToUniversalTime().ToString('O')}})
        scope='Native runtime on the current desktop; sound and external pointer/window manipulation disabled; no capture/calibration during sampling.'
    }
    $hostData | ConvertTo-Json -Depth 6 | Set-Content -LiteralPath (Join-Path $evidenceRoot 'environment.json') -Encoding utf8NoBOM
    if ($settingsWorkload) {
        $start=[Diagnostics.ProcessStartInfo]::new($measuredPath)
        $start.UseShellExecute=$false
        $start.CreateNoWindow=$true
        $start.ArgumentList.Add('--config')
        $start.ArgumentList.Add($configPath)
        $runtimeProcess=[Diagnostics.Process]::Start($start)
        $runtimeStartedAt=$runtimeProcess.StartTime
        $deadline=[DateTime]::UtcNow.AddSeconds(20)
        do {
            $runtimeProcess.Refresh()
            if($runtimeProcess.HasExited){throw 'Settings exited before its window opened.'}
            if($runtimeProcess.MainWindowHandle -ne [IntPtr]::Zero){break}
            Start-Sleep -Milliseconds 100
        } while([DateTime]::UtcNow -lt $deadline)
        if($runtimeProcess.MainWindowHandle -eq [IntPtr]::Zero){throw 'Settings window did not open.'}
        if($Workload -eq 'SettingsScroll') {
            Add-Type -AssemblyName UIAutomationClient
            Add-Type -AssemblyName UIAutomationTypes
            $window=[Windows.Automation.AutomationElement]::FromHandle($runtimeProcess.MainWindowHandle)
            $condition=[Windows.Automation.AndCondition]::new(
                [Windows.Automation.PropertyCondition]::new([Windows.Automation.AutomationElement]::NameProperty,'Appearance'),
                [Windows.Automation.PropertyCondition]::new([Windows.Automation.AutomationElement]::ControlTypeProperty,[Windows.Automation.ControlType]::Button))
            $appearance=$window.FindFirst([Windows.Automation.TreeScope]::Descendants,$condition)
            if($null -eq $appearance){throw 'Settings scrolling requires the overflowing Appearance page.'}
            $appearance.GetCurrentPattern([Windows.Automation.InvokePattern]::Pattern).Invoke()
            $hostData.settings_page='Appearance'
            $hostData | ConvertTo-Json -Depth 6 | Set-Content -LiteralPath (Join-Path $evidenceRoot 'environment.json') -Encoding utf8NoBOM
        }
        Write-Output "Started $Workload PID $($runtimeProcess.Id); warmup $WarmupSeconds seconds."
    } elseif ($Workload -ne 'Stopped') {
        try {
            $null = Invoke-Controller @('start','--config',$configPath,'--no-sound','--no-mouse-steal','--no-window-ride')
        } finally {
            # A controller timeout can happen after the independent app has launched.
            # Retain its exact isolated identity for cleanup even when readiness failed.
            $matching = @(Get-CimInstance Win32_Process -Filter "Name='honk300.exe'" | Where-Object {
                $_.ExecutablePath -eq $binaryPath -and $_.CommandLine.Contains($configPath) -and $_.CommandLine.Contains('__windows-app-runtime')
            })
            if($matching.Count -eq 1){
                $runtimeProcess=Get-Process -Id $matching[0].ProcessId
                $runtimeStartedAt=$runtimeProcess.StartTime
            }
        }
        if ($matching.Count -ne 1) { throw 'Cannot identify exactly one owned runtime.' }
        Write-Output "Started $Workload PID $($runtimeProcess.Id); warmup $WarmupSeconds seconds."
    }
    $warmup = [Diagnostics.Stopwatch]::StartNew()
    while ($warmup.Elapsed.TotalSeconds -lt $WarmupSeconds) { Start-Sleep -Milliseconds 250 }
    if ($Workload -eq 'Mud') { Assert-OwnedRuntime; $null=Invoke-Controller @('do','mud') }
    if ($Workload -eq 'Props') { Assert-OwnedRuntime; $null=Invoke-Controller @('do','note') }
    $desktopProcess = @(Get-Process dwm -ErrorAction SilentlyContinue | Where-Object SessionId -eq (Get-Process -Id $PID).SessionId) | Select-Object -First 1
    $timer = [Diagnostics.Stopwatch]::StartNew()
    $nextSample = 0.0
    $nextNotice = 30.0
    $nextGpu = 0.0
    $nextWheel = 0.0
    $nextMud = 20.0
    $memeRequested = $false
    $sampleStartUnixMs=[DateTimeOffset]::UtcNow.ToUnixTimeMilliseconds()
    while ($timer.Elapsed.TotalSeconds -lt $SampleSeconds) {
        if($Workload -eq 'Mud' -and $timer.Elapsed.TotalSeconds -ge $nextMud) {
            Assert-OwnedRuntime
            $null=Invoke-Controller @('do','mud')
            $nextMud=$timer.Elapsed.TotalSeconds+20
        }
        if($Workload -eq 'SettingsScroll' -and $timer.Elapsed.TotalSeconds -ge $nextWheel) {
            # Exercise Win32 wheel handling only in this owned settings window. No global input.
            $delta=if(([int][Math]::Floor($timer.Elapsed.TotalSeconds/5))%2 -eq 0){-120}else{120}
            [GooseProfileNative]::Wheel($runtimeProcess.MainWindowHandle,$runtimeProcess.Id,$delta)
            $nextWheel=$timer.Elapsed.TotalSeconds+0.5
        }
        if ($timer.Elapsed.TotalSeconds -lt $nextSample) { Start-Sleep -Milliseconds 50; continue }
        $row = [ordered]@{elapsed_seconds=$timer.Elapsed.TotalSeconds;utc=[DateTime]::UtcNow.ToString('O')}
        if ($null -ne $runtimeProcess) {
            $runtimeProcess.Refresh()
            if ($runtimeProcess.HasExited) { throw 'Measured runtime exited during sampling.' }
            $row.cpu_seconds=$runtimeProcess.TotalProcessorTime.TotalSeconds
            $row.working_set=$runtimeProcess.WorkingSet64
            $row.private_bytes=$runtimeProcess.PrivateMemorySize64
            $row.handles=$runtimeProcess.HandleCount
            $row.threads=$runtimeProcess.Threads.Count
            $row.gdi_objects=[GooseProfileNative]::GetGuiResources($runtimeProcess.Handle,0)
            $row.user_objects=[GooseProfileNative]::GetGuiResources($runtimeProcess.Handle,1)
            $row.surfaces=@([GooseProfileNative]::Surfaces($runtimeProcess.Id))
        }
        if ($null -ne $desktopProcess) {
            $desktopProcess.Refresh()
            # DWM may deny direct process-time queries to an ordinary user. Its published
            # performance counter still provides cumulative 100-nanosecond CPU time.
            $desktopCounters=Get-CimInstance Win32_PerfRawData_PerfProc_Process -Filter "IDProcess=$($desktopProcess.Id)" -ErrorAction SilentlyContinue
            if ($null -ne $desktopCounters) {
                $row.dwm_cpu_seconds=[double]$desktopCounters.PercentProcessorTime/10000000.0
                $row.dwm_working_set=[long]$desktopCounters.WorkingSet
            }
        }
        $backgroundRows=@(foreach($background in $backgroundProcesses){
            $background.Refresh()
            if(-not $background.HasExited){[ordered]@{pid=$background.Id;cpu_seconds=$background.TotalProcessorTime.TotalSeconds;working_set=$background.WorkingSet64}}
        })
        if($backgroundRows.Count -gt 0){$row.background_processes=$backgroundRows}
        $samples.Add($row)
        if ($gpuAvailable -and $timer.Elapsed.TotalSeconds -ge $nextGpu) {
            try {
                $gpuRows = @(Get-CimInstance Win32_PerfFormattedData_GPUPerformanceCounters_GPUEngine -ErrorAction Stop | Where-Object {
                    ($null -ne $runtimeProcess -and $_.Name -match "^pid_$($runtimeProcess.Id)_") -or
                    ($null -ne $desktopProcess -and $_.Name -match "^pid_$($desktopProcess.Id)_") -or
                    ($_.Name -match '^pid_(\d+)_' -and [int]$Matches[1] -in $backgroundPids)
                } | Select-Object Name,UtilizationPercentage)
                $gpuSamples.Add([ordered]@{elapsed_seconds=$timer.Elapsed.TotalSeconds;engines=$gpuRows})
            } catch { $gpuAvailable=$false; Write-Output 'GPU engine counters unavailable; recording that limitation.' }
            $nextGpu=$timer.Elapsed.TotalSeconds+5
        }
        if ($Workload -eq 'Props' -and -not $memeRequested -and $timer.Elapsed.TotalSeconds -ge ($SampleSeconds/2)) {
            Assert-OwnedRuntime
            $null=Invoke-Controller @('do','meme')
            $memeRequested=$true
        }
        if ($timer.Elapsed.TotalSeconds -ge $nextNotice) {
            Write-Output ("{0}: {1:N0}/{2}s; {3} samples" -f $Workload,$timer.Elapsed.TotalSeconds,$SampleSeconds,$samples.Count)
            $nextNotice+=30
        }
        $nextSample=$timer.Elapsed.TotalSeconds+1
    }
    $sampleEndUnixMs=[DateTimeOffset]::UtcNow.ToUnixTimeMilliseconds()
    if ($null -ne $runtimeProcess) {
        Assert-OwnedRuntime
        if($settingsWorkload){ $runtimeProcess.CloseMainWindow() | Out-Null }
        else { $null=Invoke-Controller @('stop') }
        if (-not $runtimeProcess.WaitForExit(60000)) { throw 'Graceful stop exceeded sixty seconds.' }
        if ((Invoke-Controller @('status')) -notmatch 'honk300: not running') { throw 'Runtime remained registered after stopping.' }
    }
    $completed=$true
} catch { $failure=$_.Exception.Message; throw }
finally {
    try {
        if ($null -ne $runtimeProcess) {
            $runtimeProcess.Refresh()
            if (-not $runtimeProcess.HasExited) {
                Assert-OwnedRuntime
                if($settingsWorkload){$runtimeProcess.Kill()}
                else {
                    try{$null=Invoke-Controller @('stop','--force')}
                    catch{$cleanupControlFailure=$_.Exception.Message}
                    $runtimeProcess.Refresh()
                    if(-not $runtimeProcess.HasExited){
                        # Failed readiness may mean there is no working IPC server.
                        # Terminate only the retained exact isolated process in that case.
                        Assert-OwnedRuntime
                        $runtimeProcess.Kill()
                        $terminatedOwnedFallback=$true
                    }
                }
                if (-not $runtimeProcess.WaitForExit(10000)) { throw 'Owned runtime did not stop during cleanup.' }
            }
        }
    } catch { $cleanupFailure=$_.Exception.Message; $completed=$false }
    finally {
        $env:HONK300_WINDOWS_PROFILE=$oldProfile
        $env:HONK300_SMOKE_LOCAL_DATE=$oldDate
        $env:HONK300_WINDOWS_SMOKE_PRESENT=$oldCapture
        $currentHash=if(Test-Path -LiteralPath $userConfig){(Get-FileHash -LiteralPath $userConfig).Hash}else{$null}
        $unchanged=$currentHash -eq $userHash
        $startupAfter=Get-StartupSnapshot
        $startupUnchanged=$startupBefore -eq $startupAfter
        $samples | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath (Join-Path $evidenceRoot 'samples.json') -Encoding utf8NoBOM
        $gpuSamples | ConvertTo-Json -Depth 6 | Set-Content -LiteralPath (Join-Path $evidenceRoot 'gpu.json') -Encoding utf8NoBOM
        $commands | ConvertTo-Json -Depth 5 | Set-Content -LiteralPath (Join-Path $evidenceRoot 'commands.json') -Encoding utf8NoBOM
        [ordered]@{completed=($completed -and $unchanged -and $startupUnchanged);failure=$failure;cleanup_failure=$cleanupFailure;cleanup_control_failure=$cleanupControlFailure;terminated_owned_fallback=$terminatedOwnedFallback;user_config_unchanged=$unchanged;startup_unchanged=$startupUnchanged;startup_before=($startupBefore|ConvertFrom-Json);startup_after=($startupAfter|ConvertFrom-Json);gpu_counters_available=$gpuAvailable;samples=$samples.Count;sample_start_unix_ms=$sampleStartUnixMs;sample_end_unix_ms=$sampleEndUnixMs} |
            ConvertTo-Json -Depth 5 | Set-Content -LiteralPath (Join-Path $evidenceRoot 'result.json') -Encoding utf8NoBOM
        if (-not $unchanged -or -not $startupUnchanged) { throw 'User configuration or installed startup state changed during the trial.' }
    }
    if ($null -ne $cleanupFailure) { throw $cleanupFailure }
}
Write-Output "Completed $Workload; evidence: $evidenceRoot"
