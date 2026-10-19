[CmdletBinding()]
param(
    [Parameter(Mandatory)][string]$Binary,
    [Parameter(Mandatory)][string]$EvidenceDirectory,
    [ValidateRange(1,3)][int]$Scenario=1,
    [ValidateRange(30,600)][int]$Seconds=300
)

# Bounded diagnostic capture of Goose's own overlay rectangle. No calibration background.
# Do not run concurrently with performance samples. The desktop is live, so adjacent native
# and screenshot captures are correlated observations, not an atomic compositor snapshot.
$ErrorActionPreference='Stop'
Set-StrictMode -Version Latest
Add-Type -AssemblyName System.Drawing
$binaryPath=(Resolve-Path -LiteralPath $Binary).Path
if([IO.Path]::GetFileName($binaryPath) -ine 'honk300.exe'){throw 'Use the direct runtime binary.'}
$evidence=[IO.Path]::GetFullPath($EvidenceDirectory)
if(Test-Path -LiteralPath $evidence){throw 'Use a new evidence directory.'}
New-Item -ItemType Directory -Path $evidence|Out-Null
$config=Join-Path $evidence 'isolated.toml'
$present=Join-Path $evidence 'current.bgra'
$userConfig=Join-Path $env:LOCALAPPDATA 'honk300/config.toml'
$userHash=if(Test-Path -LiteralPath $userConfig){(Get-FileHash -LiteralPath $userConfig).Hash}else{$null}
$configText=@'
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
seasonal = true
autumn = true
[mischief]
perch_and_ride = false
collect_windows = false
'@
[IO.File]::WriteAllText($config,$configText,[Text.UTF8Encoding]::new($false))
Add-Type -TypeDefinition @'
using System;
using System.Runtime.InteropServices;
public static class GooseTransparencyNative {
    [StructLayout(LayoutKind.Sequential)] public struct Rect{public int Left,Top,Right,Bottom;}
    [DllImport("user32.dll")] public static extern bool GetWindowRect(IntPtr hwnd,out Rect rect);
    [DllImport("user32.dll")] public static extern bool IsWindowVisible(IntPtr hwnd);
    [DllImport("user32.dll")] public static extern uint GetWindowThreadProcessId(IntPtr hwnd,out uint pid);
    [DllImport("user32.dll",EntryPoint="GetWindowLongPtrW")] public static extern IntPtr GetWindowLongPtr(IntPtr hwnd,int index);
    [DllImport("user32.dll")] public static extern IntPtr SetThreadDpiAwarenessContext(IntPtr context);
}
'@
$oldCapture=$env:HONK300_WINDOWS_SMOKE_PRESENT
$oldProfile=$env:HONK300_WINDOWS_PROFILE
$oldDate=$env:HONK300_SMOKE_LOCAL_DATE
$oldDpi=[GooseTransparencyNative]::SetThreadDpiAwarenessContext([IntPtr]::new(-4))
$runtime=$null
$runtimeStart=$null
$observations=[Collections.Generic.List[object]]::new()
$failure=$null
$cleanupFailure=$null
$completed=$false
function Control([string[]]$Arguments){
    $info=[Diagnostics.ProcessStartInfo]::new($binaryPath)
    $info.UseShellExecute=$false;$info.CreateNoWindow=$true
    $info.RedirectStandardError=$true;$info.RedirectStandardOutput=$true
    foreach($argument in $Arguments){$info.ArgumentList.Add($argument)}
    $child=[Diagnostics.Process]::Start($info)
    try{
        $out=$child.StandardOutput.ReadToEndAsync();$err=$child.StandardError.ReadToEndAsync()
        if(-not $child.WaitForExit(30000)){$child.Kill();$child.WaitForExit();throw 'Controller timed out.'}
        if(-not [Threading.Tasks.Task]::WaitAll(@($out,$err),5000)){throw 'Controller retained output pipes.'}
        $text=$out.GetAwaiter().GetResult();$errors=$err.GetAwaiter().GetResult()
        if($child.ExitCode -ne 0){throw "Controller failed: $errors"}
        return $text
    }finally{$child.Dispose()}
}
function Assert-Owned {
    $runtime.Refresh()
    $identity=Get-CimInstance Win32_Process -Filter "ProcessId=$($runtime.Id)"
    if($runtime.HasExited -or $runtime.StartTime -ne $runtimeStart -or $null -eq $identity -or $identity.ExecutablePath -ne $binaryPath -or -not $identity.CommandLine.Contains($config)){throw 'Diagnostic runtime ownership changed.'}
}
try{
    if((Control @('status')) -notmatch 'honk300: not running'){throw 'Preserve the existing runtime.'}
    $env:HONK300_WINDOWS_SMOKE_PRESENT=$present
    $env:HONK300_WINDOWS_PROFILE=$null
    $env:HONK300_SMOKE_LOCAL_DATE='20261019'
    # Keep the directly created process so startup failure cannot lose cleanup ownership.
    $start=[Diagnostics.ProcessStartInfo]::new($binaryPath)
    $start.UseShellExecute=$false;$start.CreateNoWindow=$true
    $start.RedirectStandardError=$true;$start.RedirectStandardOutput=$true
    foreach($arg in @('__windows-app-runtime','--config',$config,'--no-sound','--no-mouse-steal','--no-window-ride')){$start.ArgumentList.Add($arg)}
    $runtime=[Diagnostics.Process]::Start($start);$runtimeStart=$runtime.StartTime
    $stdout=$runtime.StandardOutput.ReadToEndAsync();$stderr=$runtime.StandardError.ReadToEndAsync()
    $timer=[Diagnostics.Stopwatch]::StartNew()
    $nextCapture=5.0;$nextNotice=30.0;$captureIndex=0;$savedBytes=0L
    $disabled=$false;$reenabled=$false;$mud=$false
    while($timer.Elapsed.TotalSeconds -lt $Seconds){
        Assert-Owned
        if($Scenario -eq 2 -and -not $mud -and $timer.Elapsed.TotalSeconds -ge 45){$null=Control @('do','mud');$mud=$true}
        if($Scenario -ge 2 -and -not $disabled -and $timer.Elapsed.TotalSeconds -ge 150){
            [IO.File]::WriteAllText($config,$configText.Replace('autumn = true','autumn = false'),[Text.UTF8Encoding]::new($false))
            $null=Control @('reload');$disabled=$true
        }
        if($Scenario -eq 3 -and -not $reenabled -and $timer.Elapsed.TotalSeconds -ge 210){
            [IO.File]::WriteAllText($config,$configText,[Text.UTF8Encoding]::new($false))
            $null=Control @('reload');$reenabled=$true
        }
        if($timer.Elapsed.TotalSeconds -ge $nextCapture){
            if(Test-Path -LiteralPath $present){Remove-Item -LiteralPath $present}
            $deadline=[DateTime]::UtcNow.AddSeconds(3)
            while(-not (Test-Path -LiteralPath $present) -and [DateTime]::UtcNow -lt $deadline){Start-Sleep -Milliseconds 20}
            if(-not (Test-Path -LiteralPath $present)){
                # On-dirty presentation can legitimately pause while a pose is still or
                # fully beyond an exposed edge. Retain that gap instead of treating an
                # absent new frame as a renderer failure or inventing a fresh capture.
                Assert-Owned
                $observations.Add([ordered]@{elapsed_seconds=$timer.Elapsed.TotalSeconds;utc=[DateTime]::UtcNow.ToString('O');capture_available=$false;status=(Control @('status'));leaves_disabled=$disabled;leaves_reenabled=$reenabled})
                $nextCapture=$timer.Elapsed.TotalSeconds+30
                continue
            }
            $sourceLength=(Get-Item -LiteralPath $present).Length
            if($sourceLength -gt 64MB){throw 'Presented-pixel record exceeds its bound.'}
            $bytes=[IO.File]::ReadAllBytes($present)
            $header=[Text.Encoding]::ASCII.GetString($bytes,0,[Math]::Min(512,$bytes.Length)).Split("`n`n")[0]
            $match=[regex]::Match($header,'hwnd=0x([0-9A-Fa-f]+)')
            if(-not $match.Success){throw 'Invalid presented HWND.'}
            $hwnd=[IntPtr]::new([Convert]::ToInt64($match.Groups[1].Value,16))
            $owner=0u;[GooseTransparencyNative]::GetWindowThreadProcessId($hwnd,[ref]$owner)|Out-Null
            if($owner -ne $runtime.Id){throw 'Presented HWND is not owned by the diagnostic runtime.'}
            $rect=[GooseTransparencyNative+Rect]::new()
            [GooseTransparencyNative]::GetWindowRect($hwnd,[ref]$rect)|Out-Null
            $visible=[GooseTransparencyNative]::IsWindowVisible($hwnd)
            $stem=Join-Path $evidence ('capture-{0:D2}' -f $captureIndex)
            $width=$rect.Right-$rect.Left;$height=$rect.Bottom-$rect.Top
            if($width -le 0 -or $height -le 0 -or [long]$width*$height -gt 16777216){throw 'Unexpected overlay capture size.'}
            # Count current/source and previously stored files, then reserve both the new
            # pixel copy and a conservative PNG bound (including metadata/log allowance).
            $savedBytes=(Get-ChildItem -LiteralPath $evidence -File|Measure-Object -Property Length -Sum).Sum
            $reserved=[long]$bytes.Length+[long]$width*$height*5+2MB
            if($savedBytes+$reserved -gt 256MB -or $captureIndex -ge 24){throw 'Diagnostic capture bound reached.'}
            [IO.File]::WriteAllBytes("$stem.bgra",$bytes)
            $metadata=[IO.Path]::ChangeExtension($present,'metadata.txt')
            if(Test-Path -LiteralPath $metadata){Copy-Item -LiteralPath $metadata -Destination "$stem.metadata.txt"}
            if($visible){
                $bitmap=[Drawing.Bitmap]::new($width,$height)
                $graphics=[Drawing.Graphics]::FromImage($bitmap)
                try{$graphics.CopyFromScreen($rect.Left,$rect.Top,0,0,$bitmap.Size);$bitmap.Save("$stem.png",[Drawing.Imaging.ImageFormat]::Png)}
                finally{$graphics.Dispose();$bitmap.Dispose()}
            }
            $observations.Add([ordered]@{elapsed_seconds=$timer.Elapsed.TotalSeconds;utc=[DateTime]::UtcNow.ToString('O');file="$stem.bgra";hwnd=$hwnd.ToInt64();visible=$visible;rect=@($rect.Left,$rect.Top,($rect.Right-$rect.Left),($rect.Bottom-$rect.Top));extended_style=[GooseTransparencyNative]::GetWindowLongPtr($hwnd,-20).ToInt64();leaves_disabled=$disabled;leaves_reenabled=$reenabled})
            $captureIndex++;$nextCapture=$timer.Elapsed.TotalSeconds+30
        }
        if($timer.Elapsed.TotalSeconds -ge $nextNotice){Write-Output "Transparency scenario $Scenario`: $([int]$timer.Elapsed.TotalSeconds)/$Seconds seconds; $captureIndex captures";$nextNotice+=30}
        Start-Sleep -Milliseconds 200
    }
    $completed=$true
}catch{$failure=$_.Exception.Message;throw}
finally{
    try{
        if($null -ne $runtime -and -not $runtime.HasExited){
            Assert-Owned
            try{
                $null=Control @('stop')
                if(-not $runtime.WaitForExit(60000)){throw 'Graceful diagnostic stop timed out.'}
            }catch{
                $cleanupFailure=$_.Exception.Message;$completed=$false
                if(-not $runtime.HasExited){
                    Assert-Owned
                    $runtime.Kill()
                    if(-not $runtime.WaitForExit(10000)){throw 'Owned diagnostic runtime did not exit after termination.'}
                }
            }
        }
        if($null -ne $runtime){
            if(-not [Threading.Tasks.Task]::WaitAll(@($stdout,$stderr),5000)){throw 'Diagnostic retained output pipes.'}
            $outText=$stdout.GetAwaiter().GetResult();$errText=$stderr.GetAwaiter().GetResult()
            [IO.File]::WriteAllText((Join-Path $evidence 'runtime.stdout.txt'),$outText.Substring(0,[Math]::Min($outText.Length,65536)))
            [IO.File]::WriteAllText((Join-Path $evidence 'runtime.stderr.txt'),$errText.Substring(0,[Math]::Min($errText.Length,65536)))
        }
    }catch{$cleanupFailure=$_.Exception.Message;$completed=$false}
    finally{
        $env:HONK300_WINDOWS_SMOKE_PRESENT=$oldCapture;$env:HONK300_WINDOWS_PROFILE=$oldProfile;$env:HONK300_SMOKE_LOCAL_DATE=$oldDate
        [GooseTransparencyNative]::SetThreadDpiAwarenessContext($oldDpi)|Out-Null
        $unchanged=$userHash -eq $(if(Test-Path -LiteralPath $userConfig){(Get-FileHash -LiteralPath $userConfig).Hash}else{$null})
        [ordered]@{completed=$completed;failure=$failure;cleanup_failure=$cleanupFailure;scenario=$Scenario;seconds=$Seconds;binary=$binaryPath;sha256=(Get-FileHash -LiteralPath $binaryPath).Hash;user_config_unchanged=$unchanged;observations=$observations}|ConvertTo-Json -Depth 7|Set-Content -LiteralPath (Join-Path $evidence 'result.json') -Encoding utf8NoBOM
        if(-not $unchanged){throw 'User config changed during diagnostic run.'}
    }
    if($null -ne $cleanupFailure){throw $cleanupFailure}
}
