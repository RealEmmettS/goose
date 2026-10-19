[CmdletBinding()]
param(
    [Parameter(Mandatory)][string] $Binary,
    [Parameter(Mandatory)][string] $EvidenceDirectory
)

# Native input-path evidence: targeted WM_MOUSEWHEEL messages plus OS UIA geometry.
# This does not inject global mouse input or claim a physical-wheel/user acceptance test.
$ErrorActionPreference='Stop'
Set-StrictMode -Version Latest
Add-Type -AssemblyName UIAutomationClient
Add-Type -AssemblyName UIAutomationTypes
$binaryPath=(Resolve-Path -LiteralPath $Binary).Path
$evidence=[IO.Path]::GetFullPath($EvidenceDirectory)
if(Test-Path -LiteralPath $evidence){throw 'Use a new evidence directory.'}
New-Item -ItemType Directory -Path $evidence | Out-Null
$config=Join-Path $evidence 'isolated.toml'
[IO.File]::WriteAllText($config,"goose_config_version = 2`n",[Text.UTF8Encoding]::new($false))
$configHash=(Get-FileHash -LiteralPath $config).Hash
Add-Type -TypeDefinition @'
using System;
using System.Runtime.InteropServices;
using System.Text;
public static class GooseScrollNative {
    [StructLayout(LayoutKind.Sequential)] public struct Rect { public int Left,Top,Right,Bottom; }
    private delegate bool EnumProc(IntPtr hwnd, IntPtr parameter);
    [DllImport("user32.dll")] private static extern bool EnumChildWindows(IntPtr parent,EnumProc callback,IntPtr parameter);
    [DllImport("user32.dll",CharSet=CharSet.Unicode)] private static extern int GetClassName(IntPtr hwnd,StringBuilder text,int length);
    [DllImport("user32.dll")] private static extern uint GetWindowThreadProcessId(IntPtr hwnd,out uint pid);
    [DllImport("user32.dll")] private static extern IntPtr SendMessageTimeout(IntPtr hwnd,uint message,IntPtr wparam,IntPtr lparam,uint flags,uint timeout,out IntPtr result);
    [DllImport("user32.dll")] public static extern bool GetWindowRect(IntPtr hwnd,out Rect rect);
    [DllImport("user32.dll")] public static extern uint GetDpiForWindow(IntPtr hwnd);
    [DllImport("user32.dll")] public static extern bool MoveWindow(IntPtr hwnd,int x,int y,int width,int height,bool repaint);
    [DllImport("user32.dll")] public static extern IntPtr SetThreadDpiAwarenessContext(IntPtr context);
    public static IntPtr Canvas(IntPtr root,uint pid) {
        uint owner; GetWindowThreadProcessId(root,out owner);
        if(owner!=pid) throw new InvalidOperationException("Window ownership changed.");
        IntPtr found=IntPtr.Zero;
        EnumChildWindows(root,(hwnd,p)=>{var name=new StringBuilder(256);GetClassName(hwnd,name,256);
            if(name.ToString()=="NativeSdkGpuSurface")found=hwnd;return true;},IntPtr.Zero);
        if(found==IntPtr.Zero)throw new InvalidOperationException("No settings canvas.");
        return found;
    }
    public static void Wheel(IntPtr canvas,int delta) {
        Rect r;if(!GetWindowRect(canvas,out r))throw new InvalidOperationException("Missing canvas.");
        int x=(r.Left+r.Right)/2,y=(r.Top+r.Bottom)/2;IntPtr result;
        if(SendMessageTimeout(canvas,0x020A,new IntPtr(delta<<16),new IntPtr((y<<16)|(x&65535)),2,1000,out result)==IntPtr.Zero)
            throw new InvalidOperationException("Wheel delivery timed out.");
    }
}
'@
$process=$null
$window=$null
$runs=[Collections.Generic.List[object]]::new()
$failure=$null
$priorDpi=[GooseScrollNative]::SetThreadDpiAwarenessContext([IntPtr]::new(-4))
function Find-Button([string]$name){
    $condition=[Windows.Automation.AndCondition]::new(
        [Windows.Automation.PropertyCondition]::new([Windows.Automation.AutomationElement]::NameProperty,$name),
        [Windows.Automation.PropertyCondition]::new([Windows.Automation.AutomationElement]::ControlTypeProperty,[Windows.Automation.ControlType]::Button))
    return $window.FindFirst([Windows.Automation.TreeScope]::Descendants,$condition)
}
try {
    $start=[Diagnostics.ProcessStartInfo]::new($binaryPath)
    $start.UseShellExecute=$false;$start.CreateNoWindow=$true
    $start.ArgumentList.Add('--config');$start.ArgumentList.Add($config)
    $process=[Diagnostics.Process]::Start($start)
    $deadline=[DateTime]::UtcNow.AddSeconds(20)
    do {
        $process.Refresh()
        if($process.HasExited){throw 'Settings exited before opening.'}
        if($process.MainWindowHandle -ne [IntPtr]::Zero){
            $window=[Windows.Automation.AutomationElement]::FromHandle($process.MainWindowHandle)
            if($null -ne (Find-Button 'First wander (seconds)')){break}
        }
        Start-Sleep -Milliseconds 100
    } while([DateTime]::UtcNow -lt $deadline)
    if($null -eq $window -or $null -eq (Find-Button 'First wander (seconds)')){throw 'Settings did not load.'}
    $canvas=[GooseScrollNative]::Canvas($process.MainWindowHandle,$process.Id)
    $initial=[GooseScrollNative+Rect]::new()
    [GooseScrollNative]::GetWindowRect($process.MainWindowHandle,[ref]$initial)|Out-Null
    foreach($size in @('normal','short')){
        if($size -eq 'short'){
            [GooseScrollNative]::MoveWindow($process.MainWindowHandle,$initial.Left,$initial.Top,$initial.Right-$initial.Left,[int](($initial.Bottom-$initial.Top)*0.8),$true)|Out-Null
            Start-Sleep -Milliseconds 500
        }
        foreach($page in @('General','Appearance','Behavior','Sound & manners','Platform & status')){
            (Find-Button $page).GetCurrentPattern([Windows.Automation.InvokePattern]::Pattern).Invoke()
            Start-Sleep -Milliseconds 400
            [GooseScrollNative]::Wheel($canvas,30000)
            Start-Sleep -Milliseconds 400
            $scale=[GooseScrollNative]::GetDpiForWindow($process.MainWindowHandle)/96.0
            $bounds=$window.Current.BoundingRectangle
            $anchor=$window.FindAll([Windows.Automation.TreeScope]::Descendants,[Windows.Automation.Condition]::TrueCondition) | Where-Object {
                $rect=$_.Current.BoundingRectangle
                $rect.Width -gt 0 -and $rect.Height -gt 0 -and $rect.Top -gt $bounds.Top+260*$scale -and $rect.Bottom -lt $bounds.Bottom-100*$scale
            } | Select-Object -First 1
            if($null -eq $anchor){throw "No visible measurement anchor on $page at $size."}
            $runtimeId=@($anchor.GetRuntimeId())
            foreach($gesture in @('notch','reverse','fractional','rapid','reverse_active')){
                if($gesture -eq 'reverse_active'){
                    [GooseScrollNative]::Wheel($canvas,-120)
                    Start-Sleep -Milliseconds 40
                }
                $before=$anchor.Current.BoundingRectangle.Top
                $watch=[Diagnostics.Stopwatch]::StartNew()
                if($gesture -eq 'notch'){[GooseScrollNative]::Wheel($canvas,-120)}
                elseif($gesture -in @('reverse','reverse_active')){[GooseScrollNative]::Wheel($canvas,120)}
                elseif($gesture -eq 'fractional'){[GooseScrollNative]::Wheel($canvas,-30)}
                else{foreach($n in 1..3){[GooseScrollNative]::Wheel($canvas,-120);Start-Sleep -Milliseconds 25}}
                $points=[Collections.Generic.List[object]]::new()
                while($watch.Elapsed.TotalMilliseconds -lt 900){
                    $rect=$anchor.Current.BoundingRectangle
                    $points.Add([ordered]@{elapsed_ms=$watch.Elapsed.TotalMilliseconds;top=$rect.Top;empty=$rect.IsEmpty;logical_movement=($before-$rect.Top)/$scale})
                    Start-Sleep -Milliseconds 10
                }
                $runs.Add([ordered]@{size=$size;page=$page;gesture=$gesture;dpi=96*$scale;anchor=$anchor.Current.Name;anchor_runtime_id=$runtimeId;window_bounds=@($bounds.Left,$bounds.Top,$bounds.Width,$bounds.Height);before_top=$before;samples=$points})
            }
        }
    }
} catch{$failure=$_.Exception.Message;throw}
finally{
    try{
        if($null -ne $process -and -not $process.HasExited){
            $process.CloseMainWindow()|Out-Null
            if(-not $process.WaitForExit(10000)){$process.Kill();$process.WaitForExit()}
        }
    }finally{
        [GooseScrollNative]::SetThreadDpiAwarenessContext($priorDpi)|Out-Null
        $runs | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath (Join-Path $evidence 'scroll.json') -Encoding utf8NoBOM
        [ordered]@{binary=$binaryPath;sha256=(Get-FileHash -LiteralPath $binaryPath).Hash;failure=$failure;isolated_config_unchanged=((Get-FileHash -LiteralPath $config).Hash -eq $configHash);scope='Targeted native Win32 input and OS UIA geometry; no physical wheel or screen-reader acceptance claimed.'}|ConvertTo-Json|Set-Content -LiteralPath (Join-Path $evidence 'result.json') -Encoding utf8NoBOM
    }
}
