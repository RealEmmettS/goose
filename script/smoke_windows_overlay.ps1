[CmdletBinding(DefaultParameterSetName = 'Smoke')]
param(
    [Parameter(Mandatory = $true, ParameterSetName = 'Smoke')]
    [string] $Binary,

    [Parameter(Mandatory = $true, ParameterSetName = 'Smoke')]
    [string] $EvidenceDirectory,

    [Parameter(Mandatory = $true, ParameterSetName = 'Background')]
    [switch] $BackgroundHost,

    [Parameter(Mandatory = $true, ParameterSetName = 'Background')]
    [string] $BackgroundState
)

$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest

function Start-ControlledBackground {
    param([string] $StateDirectory)

    Add-Type -AssemblyName System.Windows.Forms
    try {
        Add-Type -AssemblyName System.Drawing.Common
    }
    catch {
        Add-Type -AssemblyName System.Drawing
    }

    $colorPath = Join-Path $StateDirectory 'color.txt'
    $ackPath = Join-Path $StateDirectory 'color.ack'
    $readyPath = Join-Path $StateDirectory 'ready'
    $stopPath = Join-Path $StateDirectory 'stop'
    $screen = [System.Windows.Forms.SystemInformation]::VirtualScreen
    if ($screen.Width -lt 320 -or $screen.Height -lt 240) {
        throw "interactive virtual screen is unavailable: $($screen.Width)x$($screen.Height)"
    }

    $form = New-Object System.Windows.Forms.Form
    $form.Name = 'Honk300OverlaySmokeBackground'
    $form.Text = 'Honk300 Overlay Smoke Background'
    $form.FormBorderStyle = [System.Windows.Forms.FormBorderStyle]::None
    $form.StartPosition = [System.Windows.Forms.FormStartPosition]::Manual
    $form.Bounds = $screen
    $form.ShowInTaskbar = $false
    $form.TopMost = $true

    $script:BackgroundColor = ''
    $applyColor = {
        if (-not (Test-Path -LiteralPath $colorPath -PathType Leaf)) { return }
        $requested = (Get-Content -LiteralPath $colorPath -Raw).Trim().TrimStart('#').ToUpperInvariant()
        if ($requested -notmatch '^[0-9A-F]{6}$') { return }
        if ($requested -ne $script:BackgroundColor) {
            $form.BackColor = [System.Drawing.ColorTranslator]::FromHtml("#$requested")
            $form.Refresh()
            $script:BackgroundColor = $requested
        }
        # Re-acknowledge the already-active color too. The controller deliberately
        # removes the ack before each capture so the first dark frame cannot race the
        # form's initial paint.
        Set-Content -LiteralPath $ackPath -Value $requested -Encoding ascii -NoNewline
    }

    $timer = New-Object System.Windows.Forms.Timer
    $timer.Interval = 50
    $timer.Add_Tick({
        if (Test-Path -LiteralPath $stopPath) {
            $form.Close()
            return
        }
        & $applyColor
    })
    $form.Add_Shown({
        & $applyColor
        Set-Content -LiteralPath $readyPath -Value $PID -Encoding ascii -NoNewline
        $timer.Start()
    })
    try {
        [System.Windows.Forms.Application]::Run($form)
    }
    finally {
        $timer.Dispose()
        $form.Dispose()
    }
}

if ($BackgroundHost) {
    if (-not $IsWindows) { throw 'the controlled background host requires Windows' }
    Start-ControlledBackground -StateDirectory $BackgroundState
    exit 0
}

if (-not $IsWindows) { throw 'smoke_windows_overlay.ps1 must run on Windows' }

$resolvedBinary = (Resolve-Path -LiteralPath $Binary).Path
if (-not (Test-Path -LiteralPath $resolvedBinary -PathType Leaf)) {
    throw "exact built binary is missing: $resolvedBinary"
}
$evidence = [System.IO.Path]::GetFullPath($EvidenceDirectory)
New-Item -ItemType Directory -Force -Path $evidence | Out-Null

$repoRoot = Split-Path -Parent $PSScriptRoot
$analyzer = Join-Path $PSScriptRoot 'analyze_windows_overlay_capture.py'
if (-not (Test-Path -LiteralPath $analyzer -PathType Leaf)) {
    throw "overlay analyzer is missing: $analyzer"
}
$pythonCommand = Get-Command python3 -ErrorAction SilentlyContinue
if ($null -eq $pythonCommand) { $pythonCommand = Get-Command python -ErrorAction SilentlyContinue }
if ($null -eq $pythonCommand) { throw 'python3/python is required for capture analysis' }
$python = $pythonCommand.Source

$runnerTemp = if ($env:RUNNER_TEMP) { $env:RUNNER_TEMP } else { [System.IO.Path]::GetTempPath() }
$work = Join-Path $runnerTemp "honk300-windows-overlay-$([Guid]::NewGuid().ToString('N'))"
$backgroundState = Join-Path $work 'background'
New-Item -ItemType Directory -Force -Path $backgroundState | Out-Null
$config = Join-Path $work 'config.toml'
$colorPath = Join-Path $backgroundState 'color.txt'
$ackPath = Join-Path $backgroundState 'color.ack'
$readyPath = Join-Path $backgroundState 'ready'
$stopPath = Join-Path $backgroundState 'stop'
$darkHex = '203040'
$lightHex = 'F4EDE4'
Set-Content -LiteralPath $colorPath -Value $darkHex -Encoding ascii -NoNewline

try {
    Add-Type -AssemblyName System.Drawing.Common
}
catch {
    Add-Type -AssemblyName System.Drawing
}
Add-Type -AssemblyName System.Windows.Forms

Add-Type -TypeDefinition @'
using System;
using System.ComponentModel;
using System.Diagnostics;
using System.Runtime.InteropServices;
using System.Text;

public sealed class Honk300OverlayRect {
    public int X;
    public int Y;
    public int Width;
    public int Height;
}

public static class Honk300OverlaySmokeNative {
    private const uint PROCESS_SUSPEND_RESUME = 0x0800;

    [StructLayout(LayoutKind.Sequential)]
    private struct RECT { public int Left, Top, Right, Bottom; }

    private delegate bool EnumWindowsProc(IntPtr hwnd, IntPtr lParam);

    [DllImport("user32.dll")]
    private static extern bool EnumWindows(EnumWindowsProc callback, IntPtr lParam);
    [DllImport("user32.dll")]
    private static extern bool IsWindowVisible(IntPtr hwnd);
    [DllImport("user32.dll", CharSet = CharSet.Unicode)]
    private static extern int GetClassName(IntPtr hwnd, StringBuilder name, int count);
    [DllImport("user32.dll")]
    private static extern bool GetWindowRect(IntPtr hwnd, out RECT rect);
    [DllImport("user32.dll")]
    private static extern uint GetWindowThreadProcessId(IntPtr hwnd, out uint processId);
    [DllImport("user32.dll", SetLastError = true)]
    private static extern IntPtr GetDC(IntPtr hwnd);
    [DllImport("user32.dll")]
    private static extern int ReleaseDC(IntPtr hwnd, IntPtr deviceContext);
    [DllImport("gdi32.dll", SetLastError = true)]
    private static extern bool BitBlt(
        IntPtr destination,
        int destinationX,
        int destinationY,
        int width,
        int height,
        IntPtr source,
        int sourceX,
        int sourceY,
        uint operation
    );
    [DllImport("kernel32.dll", SetLastError = true)]
    private static extern IntPtr OpenProcess(uint access, bool inherit, int processId);
    [DllImport("kernel32.dll")]
    private static extern bool CloseHandle(IntPtr handle);
    [DllImport("ntdll.dll")]
    private static extern int NtSuspendProcess(IntPtr process);
    [DllImport("ntdll.dll")]
    private static extern int NtResumeProcess(IntPtr process);

    public static Honk300OverlayRect FindLargestVisibleOverlay(int expectedProcessId) {
        Honk300OverlayRect best = null;
        long bestArea = 0;
        EnumWindows(delegate(IntPtr hwnd, IntPtr ignored) {
            if (!IsWindowVisible(hwnd)) return true;
            uint processId;
            GetWindowThreadProcessId(hwnd, out processId);
            if (processId != (uint)expectedProcessId) return true;
            StringBuilder name = new StringBuilder(256);
            GetClassName(hwnd, name, name.Capacity);
            if (!String.Equals(name.ToString(), "honk300_overlay", StringComparison.Ordinal)) return true;
            RECT rect;
            if (!GetWindowRect(hwnd, out rect)) return true;
            int width = rect.Right - rect.Left;
            int height = rect.Bottom - rect.Top;
            long area = (long)width * height;
            if (width > 1 && height > 1 && area > bestArea) {
                bestArea = area;
                best = new Honk300OverlayRect {
                    X = rect.Left, Y = rect.Top, Width = width, Height = height
                };
            }
            return true;
        }, IntPtr.Zero);
        return best;
    }

    private static IntPtr OpenSuspendHandle(int processId) {
        IntPtr handle = OpenProcess(PROCESS_SUSPEND_RESUME, false, processId);
        if (handle == IntPtr.Zero) throw new Win32Exception(Marshal.GetLastWin32Error());
        return handle;
    }

    public static void Suspend(int processId) {
        IntPtr handle = OpenSuspendHandle(processId);
        try {
            int status = NtSuspendProcess(handle);
            if (status != 0) throw new InvalidOperationException("NtSuspendProcess returned " + status);
        } finally { CloseHandle(handle); }
    }

    public static void Resume(int processId) {
        IntPtr handle = OpenSuspendHandle(processId);
        try {
            int status = NtResumeProcess(handle);
            if (status != 0) throw new InvalidOperationException("NtResumeProcess returned " + status);
        } finally { CloseHandle(handle); }
    }

    public static void CaptureScreen(
        IntPtr destination,
        int sourceX,
        int sourceY,
        int width,
        int height
    ) {
        const uint SRCCOPY = 0x00CC0020;
        const uint CAPTUREBLT = 0x40000000;
        IntPtr source = GetDC(IntPtr.Zero);
        if (source == IntPtr.Zero) throw new Win32Exception(Marshal.GetLastWin32Error());
        try {
            if (!BitBlt(destination, 0, 0, width, height, source, sourceX, sourceY, SRCCOPY | CAPTUREBLT)) {
                throw new Win32Exception(Marshal.GetLastWin32Error());
            }
        } finally {
            ReleaseDC(IntPtr.Zero, source);
        }
    }
}
'@

function Invoke-ExactBinary {
    param(
        [Parameter(Mandatory = $true)] [string[]] $Arguments,
        [string] $EvidenceName
    )
    $output = (& $resolvedBinary @Arguments 2>&1 | Out-String)
    $exitCode = $LASTEXITCODE
    if ($EvidenceName) {
        Set-Content -LiteralPath (Join-Path $evidence $EvidenceName) -Value $output -Encoding utf8NoBOM
    }
    if ($exitCode -ne 0) {
        throw "exact binary command '$($Arguments -join ' ')' failed ($exitCode): $output"
    }
    return $output
}

function Wait-ForRuntime {
    param(
        [Parameter(Mandatory = $true)] [System.Diagnostics.Process] $Process,
        [Parameter(Mandatory = $true)] [string] $EvidenceName
    )
    $last = ''
    for ($attempt = 0; $attempt -lt 100; $attempt += 1) {
        if ($Process.HasExited) {
            throw "runtime exited before status became ready (exit $($Process.ExitCode))"
        }
        try {
            $last = Invoke-ExactBinary -Arguments @('status')
            if (
                $last -match '(?m)^honk300: running\s*$' -and
                $last -match '(?m)^platform: windows\s*$' -and
                $last -match '(?m)^overlay: supported\s*$'
            ) {
                Set-Content -LiteralPath (Join-Path $evidence $EvidenceName) -Value $last -Encoding utf8NoBOM
                return
            }
        }
        catch {
            $last = $_.Exception.Message
        }
        Start-Sleep -Milliseconds 200
    }
    throw "runtime did not report a supported Windows overlay: $last"
}

function Wait-ForBackgroundReady {
    param([string] $Expected)
    for ($attempt = 0; $attempt -lt 100; $attempt += 1) {
        if (Test-Path -LiteralPath $ackPath -PathType Leaf) {
            $actual = (Get-Content -LiteralPath $ackPath -Raw).Trim().ToUpperInvariant()
            if ($actual -eq $Expected.ToUpperInvariant()) {
                # Let DWM present the acknowledged repaint before copying screen pixels.
                Start-Sleep -Milliseconds 200
                return
            }
        }
        Start-Sleep -Milliseconds 50
    }
    throw "controlled background did not acknowledge #$Expected"
}

function Set-ControlledBackground {
    param([string] $Hex)
    Remove-Item -LiteralPath $ackPath -Force -ErrorAction SilentlyContinue
    Set-Content -LiteralPath $colorPath -Value $Hex.ToUpperInvariant() -Encoding ascii -NoNewline
    Wait-ForBackgroundReady -Expected $Hex
}

function Save-ScreenRect {
    param(
        [Parameter(Mandatory = $true)] [Honk300OverlayRect] $Rect,
        [Parameter(Mandatory = $true)] [string] $Path
    )
    $virtualScreen = [System.Windows.Forms.SystemInformation]::VirtualScreen
    if (
        $Rect.Width -lt 2 -or
        $Rect.Height -lt 2 -or
        $Rect.Width -gt $virtualScreen.Width -or
        $Rect.Height -gt $virtualScreen.Height
    ) {
        throw "implausible overlay rectangle $($Rect.Width)x$($Rect.Height) at $($Rect.X),$($Rect.Y)"
    }
    $bitmap = [System.Drawing.Bitmap]::new(
        $Rect.Width,
        $Rect.Height,
        [System.Drawing.Imaging.PixelFormat]::Format32bppArgb
    )
    $graphics = [System.Drawing.Graphics]::FromImage($bitmap)
    try {
        # CAPTUREBLT is required for layered windows; SRCCOPY alone may omit the exact
        # UpdateLayeredWindow surface this smoke exists to qualify. Graphics.CopyFromScreen
        # rejects the native SRCCOPY | CAPTUREBLT combination as an undefined managed enum,
        # so call BitBlt with the real Win32 raster-operation flags.
        $destination = $graphics.GetHdc()
        try {
            [Honk300OverlaySmokeNative]::CaptureScreen(
                $destination,
                $Rect.X,
                $Rect.Y,
                $Rect.Width,
                $Rect.Height
            )
        }
        finally {
            $graphics.ReleaseHdc($destination)
        }
        $bitmap.Save($Path, [System.Drawing.Imaging.ImageFormat]::Png)
    }
    finally {
        $graphics.Dispose()
        $bitmap.Dispose()
    }
}

function Start-ExactRuntime {
    param([string] $Label)
    $stdout = Join-Path $evidence "$Label-runtime.stdout.log"
    $stderr = Join-Path $evidence "$Label-runtime.stderr.log"
    $process = Start-Process -FilePath $resolvedBinary `
        -ArgumentList @('start', '--config', "`"$config`"", '--no-sound', '--no-mouse-steal', '--no-window-ride') `
        -RedirectStandardOutput $stdout `
        -RedirectStandardError $stderr `
        -PassThru
    Start-Sleep -Milliseconds 100
    $actualPath = (Get-Process -Id $process.Id).Path
    if ([System.IO.Path]::GetFullPath($actualPath) -ne [System.IO.Path]::GetFullPath($resolvedBinary)) {
        throw "runtime process path mismatch: expected $resolvedBinary, got $actualPath"
    }
    return $process
}

$background = $null
$runtime = $null
$runtimeSuspended = $false
$firstPid = $null
$visualPassed = $false
$initialHash = (Get-FileHash -LiteralPath $resolvedBinary -Algorithm SHA256).Hash.ToLowerInvariant()
Set-Content -LiteralPath (Join-Path $evidence 'exact-binary.sha256.txt') `
    -Value "$initialHash  $resolvedBinary" -Encoding ascii

try {
    $hostExecutable = (Get-Process -Id $PID).Path
    $background = Start-Process -FilePath $hostExecutable `
        -ArgumentList @(
            '-NoLogo', '-NoProfile', '-NonInteractive', '-Sta', '-ExecutionPolicy', 'Bypass',
            '-File', "`"$PSCommandPath`"", '-BackgroundHost', '-BackgroundState', "`"$backgroundState`""
        ) `
        -RedirectStandardOutput (Join-Path $work 'background.stdout.log') `
        -RedirectStandardError (Join-Path $work 'background.stderr.log') `
        -PassThru
    for ($attempt = 0; $attempt -lt 100; $attempt += 1) {
        if ($background.HasExited) {
            $stderr = Get-Content -LiteralPath (Join-Path $work 'background.stderr.log') -Raw -ErrorAction SilentlyContinue
            throw "controlled background host exited early ($($background.ExitCode)): $stderr"
        }
        if (Test-Path -LiteralPath $readyPath -PathType Leaf) { break }
        Start-Sleep -Milliseconds 50
    }
    if (-not (Test-Path -LiteralPath $readyPath -PathType Leaf)) {
        throw 'controlled background host did not expose an interactive desktop'
    }
    Wait-ForBackgroundReady -Expected $darkHex

    # The config path must not be pre-created: setup owns its atomic first write.
    if (Test-Path -LiteralPath $config) { throw "temporary config unexpectedly exists: $config" }
    Invoke-ExactBinary -Arguments @('setup', '--config', $config) -EvidenceName 'setup.txt' | Out-Null
    $smokeConfig = Get-Content -LiteralPath $config -Raw
    foreach ($setting in @('quiet_hours_enabled', 'pause_on_fullscreen')) {
        if ($smokeConfig -notmatch "(?m)^$setting = true\s*$") {
            throw "generated config does not contain expected $setting setting"
        }
        $smokeConfig = $smokeConfig -replace "(?m)^$setting = true\s*$", "$setting = false"
    }
    Set-Content -LiteralPath $config -Value $smokeConfig -Encoding utf8NoBOM

    $runtime = Start-ExactRuntime -Label 'first'
    $firstPid = $runtime.Id
    Wait-ForRuntime -Process $runtime -EvidenceName 'status-start.txt'

    $duplicate = Invoke-ExactBinary -Arguments @('start', '--config', $config, '--no-sound') `
        -EvidenceName 'single-instance.txt'
    if ($duplicate -notmatch 'already running') {
        throw "second start did not prove single-instance enforcement: $duplicate"
    }

    # Each wander picks a fresh target. Retry until the live compositor exposes a side
    # pose (beak plus the two-tone legs), never substituting a generated/golden frame.
    for ($attempt = 1; $attempt -le 12 -and -not $visualPassed; $attempt += 1) {
        Invoke-ExactBinary -Arguments @('do', 'wander') | Out-Null
        Start-Sleep -Milliseconds 900

        $rect = [Honk300OverlaySmokeNative]::FindLargestVisibleOverlay($runtime.Id)
        if ($null -eq $rect) {
            Start-Sleep -Milliseconds 350
            continue
        }

        [Honk300OverlaySmokeNative]::Suspend($runtime.Id)
        $runtimeSuspended = $true
        try {
            # Freeze first, then re-read the rect so a presentation between the
            # discovery probe and NtSuspendProcess cannot offset the capture crop.
            $rect = [Honk300OverlaySmokeNative]::FindLargestVisibleOverlay($runtime.Id)
            if ($null -eq $rect) { throw 'visible overlay disappeared while freezing the runtime' }
            $darkCapture = Join-Path $evidence "overlay-attempt-$attempt-dark.png"
            $lightCapture = Join-Path $evidence "overlay-attempt-$attempt-light.png"
            $analysis = Join-Path $evidence "overlay-attempt-$attempt-analysis.json"
            Set-ControlledBackground -Hex $darkHex
            Save-ScreenRect -Rect $rect -Path $darkCapture
            Set-ControlledBackground -Hex $lightHex
            Save-ScreenRect -Rect $rect -Path $lightCapture

            # A failed semantic pose is a retry, not a skipped assertion. The final
            # attempt still fails the job if no live side-view compositor proof exists.
            $oldNativePreference = $null
            if (Test-Path variable:PSNativeCommandUseErrorActionPreference) {
                $oldNativePreference = $PSNativeCommandUseErrorActionPreference
                $PSNativeCommandUseErrorActionPreference = $false
            }
            try {
                & $python $analyzer `
                    --dark $darkCapture `
                    --light $lightCapture `
                    --dark-bg $darkHex `
                    --light-bg $lightHex `
                    --output $analysis
                $analysisExit = $LASTEXITCODE
            }
            finally {
                if ($null -ne $oldNativePreference) {
                    $PSNativeCommandUseErrorActionPreference = $oldNativePreference
                }
            }
            if ($analysisExit -eq 0) {
                Copy-Item -LiteralPath $darkCapture -Destination (Join-Path $evidence 'overlay-dark.png') -Force
                Copy-Item -LiteralPath $lightCapture -Destination (Join-Path $evidence 'overlay-light.png') -Force
                Copy-Item -LiteralPath $analysis -Destination (Join-Path $evidence 'overlay-analysis.json') -Force
                $visualPassed = $true
            }
        }
        finally {
            [Honk300OverlaySmokeNative]::Resume($runtime.Id)
            $runtimeSuspended = $false
        }
    }
    if (-not $visualPassed) {
        throw 'no live captured pose proved body, shade, outline, wing, asymmetric beak/legs, shadow, and per-pixel alpha'
    }

    $document = Get-Content -LiteralPath $config -Raw
    if ($document -notmatch '(?m)^calm_goose = false\s*$') {
        throw 'generated config does not contain the reload-safe calm_goose setting'
    }
    $document = $document -replace '(?m)^calm_goose = false\s*$', 'calm_goose = true'
    Set-Content -LiteralPath $config -Value $document -Encoding utf8NoBOM
    Invoke-ExactBinary -Arguments @('reload') -EvidenceName 'reload.txt' | Out-Null
    Wait-ForRuntime -Process $runtime -EvidenceName 'status-after-reload.txt'

    Invoke-ExactBinary -Arguments @('stop') -EvidenceName 'stop-first.txt' | Out-Null
    $runtime.WaitForExit(15000)
    if (-not $runtime.HasExited) { throw 'first runtime did not exit after stop' }
    $runtime = $null
    $stopped = Invoke-ExactBinary -Arguments @('status') -EvidenceName 'status-stopped.txt'
    if ($stopped -notmatch '(?m)^honk300: not running\s*$') {
        throw "status did not report stopped: $stopped"
    }

    # No delay or cleanup shim: reacquiring immediately is the regression proof for
    # mutex/IPC teardown. It must be a new process running the same exact file.
    $runtime = Start-ExactRuntime -Label 'restart'
    if ($runtime.Id -eq $firstPid) { throw 'immediate restart unexpectedly reused the first PID' }
    Wait-ForRuntime -Process $runtime -EvidenceName 'status-restart.txt'
    Invoke-ExactBinary -Arguments @('stop') -EvidenceName 'stop-restart.txt' | Out-Null
    $runtime.WaitForExit(15000)
    if (-not $runtime.HasExited) { throw 'restarted runtime did not exit after stop' }
    $runtime = $null

    $finalHash = (Get-FileHash -LiteralPath $resolvedBinary -Algorithm SHA256).Hash.ToLowerInvariant()
    if ($finalHash -ne $initialHash) {
        throw "exact built binary changed during smoke: $initialHash -> $finalHash"
    }
    Set-Content -LiteralPath (Join-Path $evidence 'summary.txt') -Encoding utf8NoBOM -Value @"
exact_binary=$resolvedBinary
sha256=$initialHash
first_pid=$firstPid
visual_capture=passed
lifecycle=start,status,single-instance,reload,stop,immediate-restart,status,stop
"@
    Write-Output "Windows layered-overlay compositor and lifecycle smoke passed for $resolvedBinary ($initialHash)"
}
finally {
    if ($runtimeSuspended -and $null -ne $runtime -and -not $runtime.HasExited) {
        try { [Honk300OverlaySmokeNative]::Resume($runtime.Id) } catch { Write-Warning $_ }
    }
    if ($null -ne $runtime -and -not $runtime.HasExited) {
        try { & $resolvedBinary stop *> $null } catch { Write-Warning $_ }
        if (-not $runtime.WaitForExit(5000)) { $runtime.Kill($true) }
    }
    Set-Content -LiteralPath $stopPath -Value 'stop' -Encoding ascii -NoNewline -ErrorAction SilentlyContinue
    if ($null -ne $background -and -not $background.HasExited) {
        if (-not $background.WaitForExit(5000)) { $background.Kill($true) }
    }
    foreach ($name in @('background.stdout.log', 'background.stderr.log')) {
        $source = Join-Path $work $name
        if (Test-Path -LiteralPath $source -PathType Leaf) {
            Copy-Item -LiteralPath $source -Destination (Join-Path $evidence $name) -Force
        }
    }
    Remove-Item -LiteralPath $work -Recurse -Force -ErrorAction SilentlyContinue
}
