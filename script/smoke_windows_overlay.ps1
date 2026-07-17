[CmdletBinding(DefaultParameterSetName = 'Smoke')]
param(
    [Parameter(Mandatory = $true, ParameterSetName = 'Smoke')]
    [string] $Binary,

    [Parameter(Mandatory = $true, ParameterSetName = 'Smoke')]
    [string] $EvidenceDirectory,

    [Parameter(ParameterSetName = 'Smoke')]
    [switch] $AllowUnavailableTrayHost,

    [Parameter(Mandatory = $true, ParameterSetName = 'Background')]
    [switch] $BackgroundHost,

    [Parameter(Mandatory = $true, ParameterSetName = 'Background')]
    [string] $BackgroundState
)

$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest

if ($IsWindows) {
    # PowerShell has no DPI contract of its own. Establish PMv2 before either this
    # controller or the child background host loads WinForms or creates an HWND, so
    # VirtualScreen, GetWindowRect, and BitBlt all use physical-pixel coordinates.
    Add-Type -TypeDefinition @'
using System;
using System.ComponentModel;
using System.Runtime.InteropServices;

public static class Honk300DpiAwareness {
    private static readonly IntPtr PerMonitorV2 = new IntPtr(-4);

    [DllImport("user32.dll", SetLastError = true)]
    private static extern bool SetProcessDpiAwarenessContext(IntPtr value);
    [DllImport("user32.dll", SetLastError = true)]
    private static extern IntPtr SetThreadDpiAwarenessContext(IntPtr value);
    [DllImport("user32.dll")]
    private static extern IntPtr GetThreadDpiAwarenessContext();
    [DllImport("user32.dll")]
    private static extern bool AreDpiAwarenessContextsEqual(IntPtr first, IntPtr second);
    [DllImport("user32.dll")]
    private static extern uint GetDpiForWindow(IntPtr hwnd);

    public static string EnablePerMonitorV2() {
        bool processSet = SetProcessDpiAwarenessContext(PerMonitorV2);
        int processError = processSet ? 0 : Marshal.GetLastWin32Error();

        // A host manifest may have fixed the process default already. A PMv2 thread
        // override is still required and verified before any HWND-affecting API use.
        IntPtr previous = SetThreadDpiAwarenessContext(PerMonitorV2);
        if (previous == IntPtr.Zero) {
            throw new Win32Exception(Marshal.GetLastWin32Error(),
                "SetThreadDpiAwarenessContext(PMv2) failed");
        }
        if (!IsCurrentThreadPerMonitorV2()) {
            throw new InvalidOperationException("current thread did not enter PMv2 DPI awareness");
        }

        return processSet
            ? "process-and-thread-pmv2"
            : "thread-pmv2-process-set-error-" + processError;
    }

    public static bool IsCurrentThreadPerMonitorV2() {
        return AreDpiAwarenessContextsEqual(GetThreadDpiAwarenessContext(), PerMonitorV2);
    }

    public static uint WindowDpi(IntPtr hwnd) {
        return GetDpiForWindow(hwnd);
    }
}
'@
    $script:DpiAwarenessMode = [Honk300DpiAwareness]::EnablePerMonitorV2()

    Add-Type -TypeDefinition @'
using System;
using System.Runtime.InteropServices;

public static class Honk300TraySmoke {
    [StructLayout(LayoutKind.Sequential, CharSet = CharSet.Unicode)]
    public struct NotifyIconData {
        public uint cbSize;
        public IntPtr hWnd;
        public uint uID;
        public uint uFlags;
        public uint uCallbackMessage;
        public IntPtr hIcon;
        [MarshalAs(UnmanagedType.ByValTStr, SizeConst = 128)] public string szTip;
        public uint dwState;
        public uint dwStateMask;
        [MarshalAs(UnmanagedType.ByValTStr, SizeConst = 256)] public string szInfo;
        public uint uVersion;
        [MarshalAs(UnmanagedType.ByValTStr, SizeConst = 64)] public string szInfoTitle;
        public uint dwInfoFlags;
        public Guid guidItem;
        public IntPtr hBalloonIcon;
    }

    [StructLayout(LayoutKind.Sequential)]
    public struct NotifyIdentifier {
        public uint cbSize;
        public IntPtr hWnd;
        public uint uID;
        public Guid guidItem;
    }

    [StructLayout(LayoutKind.Sequential)]
    public struct Rect { public int Left, Top, Right, Bottom; }

    [DllImport("shell32.dll", CharSet = CharSet.Unicode)]
    public static extern bool Shell_NotifyIcon(uint message, ref NotifyIconData data);
    [DllImport("shell32.dll")]
    public static extern int Shell_NotifyIconGetRect(ref NotifyIdentifier id, out Rect rect);
    [DllImport("user32.dll", CharSet = CharSet.Unicode)]
    public static extern IntPtr FindWindow(string className, string title);
    [DllImport("user32.dll", CharSet = CharSet.Unicode)]
    public static extern IntPtr FindWindowEx(IntPtr parent, IntPtr after, string className, string title);
    [DllImport("user32.dll", CharSet = CharSet.Unicode)]
    public static extern uint RegisterWindowMessage(string name);
    [DllImport("user32.dll")]
    public static extern bool PostMessage(IntPtr window, uint message, IntPtr wParam, IntPtr lParam);
    [DllImport("user32.dll", CharSet = CharSet.Unicode)]
    private static extern IntPtr CreateWindowEx(
        uint exStyle, string className, string windowName, uint style,
        int x, int y, int width, int height, IntPtr parent, IntPtr menu,
        IntPtr instance, IntPtr parameter);
    [DllImport("user32.dll")]
    private static extern bool DestroyWindow(IntPtr window);
    [DllImport("user32.dll", CharSet = CharSet.Unicode)]
    private static extern IntPtr LoadIcon(IntPtr instance, IntPtr iconName);

    public static bool ProbeNotificationArea() {
        const uint WS_EX_TOOLWINDOW = 0x00000080;
        const uint WS_POPUP = 0x80000000;
        const uint NIF_MESSAGE = 0x00000001;
        const uint NIF_ICON = 0x00000002;
        const uint NIF_TIP = 0x00000004;
        const uint NIM_ADD = 0;
        const uint NIM_DELETE = 2;
        IntPtr owner = CreateWindowEx(
            WS_EX_TOOLWINDOW, "STATIC", "Honk300 tray API probe", WS_POPUP,
            0, 0, 0, 0, IntPtr.Zero, IntPtr.Zero, IntPtr.Zero, IntPtr.Zero);
        if (owner == IntPtr.Zero) return false;
        try {
            NotifyIconData data = new NotifyIconData();
            data.cbSize = (uint)Marshal.SizeOf<NotifyIconData>();
            data.hWnd = owner;
            data.uID = 0x300;
            data.uFlags = NIF_MESSAGE | NIF_ICON | NIF_TIP;
            data.uCallbackMessage = 0x0401;
            data.hIcon = LoadIcon(IntPtr.Zero, new IntPtr(32512)); // IDI_APPLICATION
            data.szTip = "Honk300 tray API probe";
            bool added = data.hIcon != IntPtr.Zero && Shell_NotifyIcon(NIM_ADD, ref data);
            if (added) Shell_NotifyIcon(NIM_DELETE, ref data);
            return added;
        }
        finally {
            DestroyWindow(owner);
        }
    }
}
'@
}

function Test-WindowsTrayRecovery {
    param(
        [Parameter(Mandatory = $true)] [string] $EvidencePath,
        [Parameter(Mandatory = $true)] [string] $RuntimeStderrPath
    )

    $guid = [Guid]'1282821f-82b6-42e2-945b-ef2fe8d9fbda'
    $independentProbe = [Honk300TraySmoke]::ProbeNotificationArea()
    $owner = [Honk300TraySmoke]::FindWindow('honk300_status_tray_owner', 'Honk300 controls')
    if ($owner -eq [IntPtr]::Zero) {
        $taskbar = [Honk300TraySmoke]::FindWindow('Shell_TrayWnd', $null)
        $notificationHost = [IntPtr]::Zero
        if ($taskbar -ne [IntPtr]::Zero) {
            $notificationHost = [Honk300TraySmoke]::FindWindowEx(
                $taskbar, [IntPtr]::Zero, 'TrayNotifyWnd', $null
            )
        }
        $runtimeMessage = ''
        for ($attempt = 0; $attempt -lt 40; $attempt += 1) {
            if (Test-Path -LiteralPath $RuntimeStderrPath) {
                $runtimeMessage = Get-Content -LiteralPath $RuntimeStderrPath -Raw
                if ($runtimeMessage -match 'Windows notification-area controls are unavailable; CLI controls remain active') {
                    break
                }
            }
            Start-Sleep -Milliseconds 25
        }
        if ($independentProbe) {
            throw "Honk300 tray registration failed even though an independent Shell_NotifyIconW probe succeeded: $runtimeMessage"
        }
        if (-not $AllowUnavailableTrayHost) {
            throw "Shell_NotifyIconW is unavailable and this runner has no explicit waiver: $runtimeMessage"
        }
        if ($runtimeMessage -notmatch 'Windows notification-area controls are unavailable; CLI controls remain active') {
            throw "Windows tray owner and TrayNotifyWnd are unavailable without explicit runtime degradation: $runtimeMessage"
        }
        Set-Content -LiteralPath $EvidencePath -Encoding utf8NoBOM -Value @"
availability=unavailable
reason=independent stock-icon Shell_NotifyIconW control probe failed in the explicitly waived runner
taskbar=0x$($taskbar.ToInt64().ToString('X'))
notification_host=0x$($notificationHost.ToInt64().ToString('X'))
independent_shell_probe=$($independentProbe.ToString().ToLowerInvariant())
guid=$guid
accessible_name=Honk300 controls
runtime_message=$($runtimeMessage.Trim())
"@
        return
    }

    $identifier = [Honk300TraySmoke+NotifyIdentifier]::new()
    $identifier.cbSize = [Runtime.InteropServices.Marshal]::SizeOf($identifier)
    $identifier.guidItem = $guid
    $beforeRect = [Honk300TraySmoke+Rect]::new()
    $before = [Honk300TraySmoke]::Shell_NotifyIconGetRect([ref] $identifier, [ref] $beforeRect)
    if ($before -ne 0) { throw "Windows tray icon is not registered (HRESULT $before)" }
    if (-not $independentProbe) {
        throw 'Honk300 registered its icon but the independent Shell_NotifyIconW control probe failed'
    }

    # Remove only the exact fixed-GUID item, then deliver the same registered broadcast Explorer
    # sends after taskbar recreation. The runtime must re-add and reapply NOTIFYICON_VERSION_4.
    $data = [Honk300TraySmoke+NotifyIconData]::new()
    $data.cbSize = [Runtime.InteropServices.Marshal]::SizeOf($data)
    $data.hWnd = $owner
    $data.uID = 1
    $data.uFlags = 0x20 # NIF_GUID
    $data.guidItem = $guid
    if (-not [Honk300TraySmoke]::Shell_NotifyIcon(2, [ref] $data)) { # NIM_DELETE
        throw 'could not remove exact tray icon before recreation probe'
    }
    $missingRect = [Honk300TraySmoke+Rect]::new()
    $missing = [Honk300TraySmoke]::Shell_NotifyIconGetRect([ref] $identifier, [ref] $missingRect)
    if ($missing -eq 0) { throw 'tray icon remained registered after exact NIM_DELETE' }

    $taskbarCreated = [Honk300TraySmoke]::RegisterWindowMessage('TaskbarCreated')
    if ($taskbarCreated -eq 0 -or -not [Honk300TraySmoke]::PostMessage(
        $owner, $taskbarCreated, [IntPtr]::Zero, [IntPtr]::Zero
    )) {
        throw 'could not deliver TaskbarCreated to the tray owner'
    }
    $after = -1
    $afterRect = [Honk300TraySmoke+Rect]::new()
    for ($attempt = 0; $attempt -lt 40; $attempt += 1) {
        Start-Sleep -Milliseconds 25
        $after = [Honk300TraySmoke]::Shell_NotifyIconGetRect([ref] $identifier, [ref] $afterRect)
        if ($after -eq 0) { break }
    }
    if ($after -ne 0) { throw "tray icon did not recover after TaskbarCreated (HRESULT $after)" }

    Set-Content -LiteralPath $EvidencePath -Encoding utf8NoBOM -Value @"
owner=0x$($owner.ToInt64().ToString('X'))
guid=$guid
accessible_name=Honk300 controls
independent_shell_probe=$($independentProbe.ToString().ToLowerInvariant())
before_rect=$($beforeRect.Left),$($beforeRect.Top),$($beforeRect.Right),$($beforeRect.Bottom)
missing_hresult=$missing
taskbar_created_message=$taskbarCreated
after_rect=$($afterRect.Left),$($afterRect.Top),$($afterRect.Right),$($afterRect.Bottom)
"@
}

function Write-TextFileAtomically {
    param(
        [Parameter(Mandatory = $true)] [string] $Path,
        [Parameter(Mandatory = $true)] [string] $Value
    )

    $directory = [System.IO.Path]::GetDirectoryName($Path)
    if (-not [System.IO.Directory]::Exists($directory)) {
        [System.IO.Directory]::CreateDirectory($directory) | Out-Null
    }
    $temporaryPath = Join-Path $directory ".$(Split-Path -Leaf $Path).$PID.$([Guid]::NewGuid().ToString('N')).tmp"
    [System.IO.File]::WriteAllText(
        $temporaryPath,
        $Value,
        [System.Text.UTF8Encoding]::new($false)
    )
    try {
        for ($attempt = 0; $attempt -lt 20; $attempt += 1) {
            try {
                [System.IO.File]::Move($temporaryPath, $Path, $true)
                return
            }
            catch [System.IO.IOException] {
                if ($attempt -eq 19) { throw }
            }
            catch [System.UnauthorizedAccessException] {
                if ($attempt -eq 19) { throw }
            }
            Start-Sleep -Milliseconds 10
        }
    }
    finally {
        if ([System.IO.File]::Exists($temporaryPath)) {
            [System.IO.File]::Delete($temporaryPath)
        }
    }
}

function Read-SharedTextFile {
    param([Parameter(Mandatory = $true)] [string] $Path)

    $share = [System.IO.FileShare]::ReadWrite -bor [System.IO.FileShare]::Delete
    for ($attempt = 0; $attempt -lt 20; $attempt += 1) {
        $stream = $null
        $reader = $null
        try {
            $stream = [System.IO.FileStream]::new(
                $Path,
                [System.IO.FileMode]::Open,
                [System.IO.FileAccess]::Read,
                $share
            )
            $reader = [System.IO.StreamReader]::new($stream)
            return $reader.ReadToEnd()
        }
        catch [System.IO.FileNotFoundException] {
            # Atomic replacement can make a probe briefly observe no current name.
        }
        catch [System.IO.IOException] {
            if ($attempt -eq 19) { throw }
        }
        catch [System.UnauthorizedAccessException] {
            if ($attempt -eq 19) { throw }
        }
        finally {
            if ($null -ne $reader) { $reader.Dispose() }
            if ($null -ne $stream) { $stream.Dispose() }
        }
        Start-Sleep -Milliseconds 10
    }
    return $null
}

function Start-ControlledBackground {
    param([string] $StateDirectory)

    Add-Type -AssemblyName System.Windows.Forms
    try {
        Add-Type -AssemblyName System.Drawing.Common
    }
    catch {
        Add-Type -AssemblyName System.Drawing
    }

    $colorRequestPath = Join-Path $StateDirectory 'color.request'
    $ackPath = Join-Path $StateDirectory 'color.ack'
    $readyPath = Join-Path $StateDirectory 'ready'
    $stopPath = Join-Path $StateDirectory 'stop'
    $diagnosticsPath = Join-Path $StateDirectory 'background-diagnostics.txt'
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

    $script:BackgroundRequestToken = ''
    $applyColor = {
        if (-not (Test-Path -LiteralPath $colorRequestPath -PathType Leaf)) { return }
        $requestDocument = Read-SharedTextFile -Path $colorRequestPath
        if ($null -eq $requestDocument) { return }
        $request = $requestDocument.Trim()
        if ($request -notmatch '^(?<Token>[0-9A-Fa-f]{32}) (?<Color>[0-9A-Fa-f]{6})$') {
            throw "invalid controlled-background request: $request"
        }
        $requestToken = $Matches.Token.ToLowerInvariant()
        $requested = $Matches.Color.ToUpperInvariant()
        if ($requestToken -eq $script:BackgroundRequestToken) { return }

        $form.BackColor = [System.Drawing.ColorTranslator]::FromHtml("#$requested")
        $form.Refresh()
        Write-TextFileAtomically -Path $ackPath -Value "$requestToken $requested"
        $script:BackgroundRequestToken = $requestToken
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
        if (-not $script:BackgroundRequestToken) {
            throw 'controlled background did not receive its initial color request'
        }
        $windowDpi = [Honk300DpiAwareness]::WindowDpi($form.Handle)
        if ($windowDpi -eq 0) { throw 'GetDpiForWindow returned zero for controlled background' }
        $bounds = $form.Bounds
        if (
            $bounds.X -ne $screen.X -or
            $bounds.Y -ne $screen.Y -or
            $bounds.Width -ne $screen.Width -or
            $bounds.Height -ne $screen.Height
        ) {
            throw "controlled background bounds $($bounds.X),$($bounds.Y),$($bounds.Width),$($bounds.Height) do not match virtual screen $($screen.X),$($screen.Y),$($screen.Width),$($screen.Height)"
        }
        $threadPmv2 = [Honk300DpiAwareness]::IsCurrentThreadPerMonitorV2().ToString().ToLowerInvariant()
        $windowHandle = $form.Handle.ToInt64().ToString('X')
        Write-TextFileAtomically -Path $diagnosticsPath -Value (@(
            'process=background'
            "dpi_awareness=$script:DpiAwarenessMode"
            "thread_pmv2=$threadPmv2"
            "virtual_screen=$($screen.X),$($screen.Y),$($screen.Width),$($screen.Height)"
            "window_hwnd=0x$windowHandle"
            "window_visible=$($form.Visible.ToString().ToLowerInvariant())"
            "window_dpi=$windowDpi"
            "window_bounds=$($bounds.X),$($bounds.Y),$($bounds.Width),$($bounds.Height)"
        ) -join [Environment]::NewLine)
        Write-TextFileAtomically -Path $readyPath -Value "$PID"
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
$colorRequestPath = Join-Path $backgroundState 'color.request'
$ackPath = Join-Path $backgroundState 'color.ack'
$readyPath = Join-Path $backgroundState 'ready'
$stopPath = Join-Path $backgroundState 'stop'
$backgroundDiagnosticsPath = Join-Path $backgroundState 'background-diagnostics.txt'
$darkHex = '203040'
$lightHex = 'F4EDE4'
$initialColorToken = [Guid]::NewGuid().ToString('N')
Write-TextFileAtomically -Path $colorRequestPath -Value "$initialColorToken $darkHex"

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
    public long Hwnd;
    public uint Dpi;
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
    [DllImport("user32.dll")]
    private static extern uint GetDpiForWindow(IntPtr hwnd);
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
                    X = rect.Left,
                    Y = rect.Top,
                    Width = width,
                    Height = height,
                    Hwnd = hwnd.ToInt64(),
                    Dpi = GetDpiForWindow(hwnd)
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

$controllerVirtualScreen = [System.Windows.Forms.SystemInformation]::VirtualScreen
if ($controllerVirtualScreen.Width -lt 320 -or $controllerVirtualScreen.Height -lt 240) {
    throw "interactive virtual screen is unavailable to controller: $($controllerVirtualScreen.Width)x$($controllerVirtualScreen.Height)"
}
$captureDiagnosticsPath = Join-Path $evidence 'capture-diagnostics.txt'
$controllerThreadPmv2 = [Honk300DpiAwareness]::IsCurrentThreadPerMonitorV2().ToString().ToLowerInvariant()
$controllerDiagnostics = (@(
    'process=controller'
    "dpi_awareness=$script:DpiAwarenessMode"
    "thread_pmv2=$controllerThreadPmv2"
    "virtual_screen=$($controllerVirtualScreen.X),$($controllerVirtualScreen.Y),$($controllerVirtualScreen.Width),$($controllerVirtualScreen.Height)"
) -join [Environment]::NewLine) + [Environment]::NewLine
Write-TextFileAtomically -Path $captureDiagnosticsPath -Value $controllerDiagnostics

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
    param(
        [Parameter(Mandatory = $true)] [string] $Expected,
        [Parameter(Mandatory = $true)] [string] $Token
    )
    $expectedAck = "$($Token.ToLowerInvariant()) $($Expected.ToUpperInvariant())"
    $last = '<missing>'
    for ($attempt = 0; $attempt -lt 100; $attempt += 1) {
        if (Test-Path -LiteralPath $ackPath -PathType Leaf) {
            $ackDocument = Read-SharedTextFile -Path $ackPath
            if ($null -eq $ackDocument) {
                Start-Sleep -Milliseconds 50
                continue
            }
            $last = $ackDocument.Trim()
            if ($last -ceq $expectedAck) {
                # Let DWM present the acknowledged repaint before copying screen pixels.
                Start-Sleep -Milliseconds 200
                return
            }
        }
        Start-Sleep -Milliseconds 50
    }
    throw "controlled background did not acknowledge '$expectedAck' (last '$last')"
}

function Set-ControlledBackground {
    param([string] $Hex)
    $requestToken = [Guid]::NewGuid().ToString('N')
    Write-TextFileAtomically -Path $colorRequestPath -Value "$requestToken $($Hex.ToUpperInvariant())"
    Wait-ForBackgroundReady -Expected $Hex -Token $requestToken
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

function Measure-ControlledBackgroundCapture {
    param(
        [Parameter(Mandatory = $true)] [string] $Path,
        [Parameter(Mandatory = $true)] [string] $ExpectedHex
    )

    $expected = [System.Drawing.ColorTranslator]::FromHtml("#$ExpectedHex")
    $bitmap = [System.Drawing.Bitmap]::new($Path)
    $matching = 0
    $total = $bitmap.Width * $bitmap.Height
    try {
        for ($y = 0; $y -lt $bitmap.Height; $y += 1) {
            for ($x = 0; $x -lt $bitmap.Width; $x += 1) {
                $pixel = $bitmap.GetPixel($x, $y)
                if (
                    $pixel.R -eq $expected.R -and
                    $pixel.G -eq $expected.G -and
                    $pixel.B -eq $expected.B
                ) {
                    $matching += 1
                }
            }
        }
    }
    finally {
        $bitmap.Dispose()
    }

    return [pscustomobject]@{
        Coverage = [double]$matching / $total
        Matching = $matching
        Total = $total
    }
}

function Start-ExactRuntime {
    param([string] $Label)
    $stdout = Join-Path $evidence "$Label-runtime.stdout.log"
    $stderr = Join-Path $evidence "$Label-runtime.stderr.log"
    $previousSmokePresent = [Environment]::GetEnvironmentVariable(
        'HONK300_WINDOWS_SMOKE_PRESENT',
        'Process'
    )
    try {
        if ($captureMode -eq 'hosted-arm64-presenter-surface') {
            [Environment]::SetEnvironmentVariable(
                'HONK300_WINDOWS_SMOKE_PRESENT',
                $rendererPresentPath,
                'Process'
            )
        }
        else {
            # Paired-DWM evidence must not activate a diagnostic hook inherited from the caller.
            [Environment]::SetEnvironmentVariable(
                'HONK300_WINDOWS_SMOKE_PRESENT',
                $null,
                'Process'
            )
        }
        $process = Start-Process -FilePath $resolvedBinary `
            -ArgumentList @('start', '--config', "`"$config`"", '--no-sound', '--no-mouse-steal', '--no-window-ride') `
            -RedirectStandardOutput $stdout `
            -RedirectStandardError $stderr `
            -PassThru
    }
    finally {
        [Environment]::SetEnvironmentVariable(
            'HONK300_WINDOWS_SMOKE_PRESENT',
            $previousSmokePresent,
            'Process'
        )
    }
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
$captureMode = 'paired-dwm'
$presenterRectTolerancePixels = 3
$rendererPresentPath = Join-Path $work 'renderer-present.bgra'
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
    Wait-ForBackgroundReady -Expected $darkHex -Token $initialColorToken
    if (-not (Test-Path -LiteralPath $backgroundDiagnosticsPath -PathType Leaf)) {
        throw 'controlled background did not publish DPI and geometry diagnostics'
    }
    $backgroundDiagnostics = Read-SharedTextFile -Path $backgroundDiagnosticsPath
    $expectedVirtualScreen = "virtual_screen=$($controllerVirtualScreen.X),$($controllerVirtualScreen.Y),$($controllerVirtualScreen.Width),$($controllerVirtualScreen.Height)"
    $backgroundDiagnosticLines = @(
        $backgroundDiagnostics -split '\r?\n' | Where-Object { $_.Length -gt 0 }
    )
    if ($backgroundDiagnosticLines -notcontains $expectedVirtualScreen) {
        throw "controller/background virtual-screen mismatch: expected '$expectedVirtualScreen', got '$backgroundDiagnostics'"
    }
    if ($backgroundDiagnosticLines -notcontains 'window_visible=true') {
        throw "controlled background HWND is not visible: '$backgroundDiagnostics'"
    }
    Copy-Item -LiteralPath $backgroundDiagnosticsPath `
        -Destination (Join-Path $evidence 'background-diagnostics.txt') -Force

    # Prove the controller and the ordinary TopMost background agree on physical
    # coordinates before the goose starts. This turns wallpaper-only captures into
    # an immediate infrastructure failure instead of twelve misleading pose retries.
    $proofScreen = [System.Windows.Forms.Screen]::PrimaryScreen.Bounds
    $proofRect = [Honk300OverlayRect]::new()
    $proofRect.Width = 64
    $proofRect.Height = 64
    $proofRect.X = $proofScreen.X + [Math]::Floor(($proofScreen.Width - $proofRect.Width) / 2)
    $proofRect.Y = $proofScreen.Y + [Math]::Floor(($proofScreen.Height - $proofRect.Height) / 2)
    $darkProofCapture = Join-Path $evidence 'background-proof-dark.png'
    $lightProofCapture = Join-Path $evidence 'background-proof-light.png'
    Save-ScreenRect -Rect $proofRect -Path $darkProofCapture
    try {
        Set-ControlledBackground -Hex $lightHex
        Save-ScreenRect -Rect $proofRect -Path $lightProofCapture
    }
    finally {
        Set-ControlledBackground -Hex $darkHex
    }
    $darkProofHash = (Get-FileHash -LiteralPath $darkProofCapture -Algorithm SHA256).Hash.ToLowerInvariant()
    $lightProofHash = (Get-FileHash -LiteralPath $lightProofCapture -Algorithm SHA256).Hash.ToLowerInvariant()
    $darkProof = Measure-ControlledBackgroundCapture -Path $darkProofCapture -ExpectedHex $darkHex
    $lightProof = Measure-ControlledBackgroundCapture -Path $lightProofCapture -ExpectedHex $lightHex
    $pairedBackgroundVisible = (
        $darkProof.Coverage -ge 0.95 -and
        $lightProof.Coverage -ge 0.95 -and
        $darkProofHash -ne $lightProofHash
    )
    $hostedArm64WallpaperCapture = (
        $env:GITHUB_ACTIONS -eq 'true' -and
        $env:RUNNER_ENVIRONMENT -eq 'github-hosted' -and
        $env:RUNNER_OS -eq 'Windows' -and
        $env:RUNNER_ARCH -eq 'ARM64' -and
        $env:PROCESSOR_ARCHITECTURE -eq 'ARM64' -and
        $darkProof.Coverage -le 0.01 -and
        $lightProof.Coverage -le 0.01 -and
        $darkProofHash -eq $lightProofHash
    )
    if ($pairedBackgroundVisible) {
        $captureMode = 'paired-dwm'
    }
    elseif ($hostedArm64WallpaperCapture) {
        # GitHub's public-preview ARM64 hosted display currently returns the same static
        # wallpaper from GetDC(NULL)+BitBlt while ordinary HWNDs remain visible and repaint.
        # Keep this exception exact and auditable: only that hosted ARM64 signature may use the
        # real process's post-success layered-presenter bytes plus visible-HWND evidence below.
        $captureMode = 'hosted-arm64-presenter-surface'
    }
    else {
        throw "controlled background proof failed: dark=$($darkProof.Matching)/$($darkProof.Total) ($($darkProof.Coverage)); light=$($lightProof.Matching)/$($lightProof.Total) ($($lightProof.Coverage)); identical=$($darkProofHash -eq $lightProofHash)"
    }
    Set-Content -LiteralPath (Join-Path $evidence 'background-proof.txt') -Encoding utf8NoBOM -Value @"
capture_mode=$captureMode
rect=$($proofRect.X),$($proofRect.Y),$($proofRect.Width),$($proofRect.Height)
dark_hex=$darkHex
dark_coverage=$($darkProof.Coverage.ToString('F6', [System.Globalization.CultureInfo]::InvariantCulture))
dark_sha256=$darkProofHash
light_hex=$lightHex
light_coverage=$($lightProof.Coverage.ToString('F6', [System.Globalization.CultureInfo]::InvariantCulture))
light_sha256=$lightProofHash
"@

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
    Test-WindowsTrayRecovery `
        -EvidencePath (Join-Path $evidence 'tray-recovery.txt') `
        -RuntimeStderrPath (Join-Path $evidence 'first-runtime.stderr.log')

    $duplicate = Invoke-ExactBinary -Arguments @('start', '--config', $config, '--no-sound') `
        -EvidenceName 'single-instance.txt'
    if ($duplicate -notmatch 'already running') {
        throw "second start did not prove single-instance enforcement: $duplicate"
    }

    # Each wander picks a fresh target. Retry until the exact running binary exposes one complete
    # renderer view: side (beak, two-tone legs, shadow) or top-down (compact beak and complete
    # view-specific body/wing geometry). Normal hosts must prove the frozen DWM composition over
    # paired backgrounds. Only the exact GitHub-hosted ARM wallpaper signature may instead analyze
    # the premultiplied BGRA DIB accepted by its successful visible layered-window present.
    for ($attempt = 1; $attempt -le 12 -and -not $visualPassed; $attempt += 1) {
        Invoke-ExactBinary -Arguments @('do', 'wander') | Out-Null
        Start-Sleep -Milliseconds 900

        if ($captureMode -eq 'hosted-arm64-presenter-surface') {
            # Request only after the pose delay. The backend atomically records the next exact DIB
            # after UpdateLayeredWindow succeeds, so the evidence cannot be an early stale frame.
            if (Test-Path -LiteralPath $rendererPresentPath) {
                Remove-Item -LiteralPath $rendererPresentPath -Force -ErrorAction Stop
            }
            if (Test-Path -LiteralPath $rendererPresentPath) {
                throw 'could not clear the previous presenter record before requesting a fresh one'
            }
            for ($surfaceAttempt = 0; $surfaceAttempt -lt 400; $surfaceAttempt += 1) {
                if (Test-Path -LiteralPath $rendererPresentPath -PathType Leaf) { break }
                Start-Sleep -Milliseconds 5
            }
            if (-not (Test-Path -LiteralPath $rendererPresentPath -PathType Leaf)) {
                continue
            }
            # Freeze immediately after the completed atomic record appears. Its embedded HWND must
            # still equal the frozen native window below. The observed race advances one
            # presentation interval between the atomic rename and NtSuspendProcess, so geometry
            # may agree within the tightly bounded physical-pixel tolerance; larger drift retries.
            [Honk300OverlaySmokeNative]::Suspend($runtime.Id)
            $runtimeSuspended = $true
        }

        $rect = [Honk300OverlaySmokeNative]::FindLargestVisibleOverlay($runtime.Id)
        if ($null -eq $rect) {
            if ($runtimeSuspended) {
                [Honk300OverlaySmokeNative]::Resume($runtime.Id)
                $runtimeSuspended = $false
            }
            Start-Sleep -Milliseconds 350
            continue
        }

        if (-not $runtimeSuspended) {
            [Honk300OverlaySmokeNative]::Suspend($runtime.Id)
            $runtimeSuspended = $true
        }
        try {
            # Freeze first, then re-read the rect so a presentation between the
            # discovery probe and NtSuspendProcess cannot offset the capture crop.
            $rect = [Honk300OverlaySmokeNative]::FindLargestVisibleOverlay($runtime.Id)
            if ($null -eq $rect) { throw 'visible overlay disappeared while freezing the runtime' }
            if ($rect.Dpi -eq 0) { throw 'GetDpiForWindow returned zero for the frozen overlay' }
            $overlayHandle = $rect.Hwnd.ToString('X')
            Add-Content -LiteralPath $captureDiagnosticsPath -Encoding utf8NoBOM -Value `
                "attempt=$attempt capture_mode=$captureMode overlay_hwnd=0x$overlayHandle overlay_rect=$($rect.X),$($rect.Y),$($rect.Width),$($rect.Height) overlay_dpi=$($rect.Dpi)"
            $darkCapture = Join-Path $evidence "overlay-attempt-$attempt-dark.png"
            $lightCapture = Join-Path $evidence "overlay-attempt-$attempt-light.png"
            $surfaceCapture = Join-Path $evidence "overlay-attempt-$attempt-present.bgra"
            $analysis = Join-Path $evidence "overlay-attempt-$attempt-analysis.json"
            if ($captureMode -eq 'paired-dwm') {
                Set-ControlledBackground -Hex $darkHex
                Save-ScreenRect -Rect $rect -Path $darkCapture
                Set-ControlledBackground -Hex $lightHex
                Save-ScreenRect -Rect $rect -Path $lightCapture
                $analysisArguments = @(
                    '--dark', $darkCapture,
                    '--light', $lightCapture,
                    '--dark-bg', $darkHex,
                    '--light-bg', $lightHex,
                    '--output', $analysis
                )
            }
            elseif ($captureMode -eq 'hosted-arm64-presenter-surface') {
                Copy-Item -LiteralPath $rendererPresentPath -Destination $surfaceCapture -Force
                $analysisArguments = @('--surface', $surfaceCapture, '--output', $analysis)
            }
            else {
                throw "unknown Windows overlay capture mode: $captureMode"
            }

            # A failed or incomplete semantic pose is a retry, not a skipped assertion. The final
            # attempt still fails the job if neither exact renderer-view proof exists.
            $oldNativePreference = $null
            if (Test-Path variable:PSNativeCommandUseErrorActionPreference) {
                $oldNativePreference = $PSNativeCommandUseErrorActionPreference
                $PSNativeCommandUseErrorActionPreference = $false
            }
            try {
                & $python $analyzer @analysisArguments
                $analysisExit = $LASTEXITCODE
            }
            finally {
                if ($null -ne $oldNativePreference) {
                    $PSNativeCommandUseErrorActionPreference = $oldNativePreference
                }
            }
            if ($analysisExit -eq 0 -and $captureMode -eq 'hosted-arm64-presenter-surface') {
                $analysisDocument = Get-Content -LiteralPath $analysis -Raw | ConvertFrom-Json
                $expectedHwnd = "0x$overlayHandle"
                $expectedRect = "$($rect.X),$($rect.Y),$($rect.Width),$($rect.Height)"
                $actualHwnd = [string]$analysisDocument.present.hwnd
                $actualRectValues = @($analysisDocument.present.rect)
                $actualRect = $actualRectValues -join ','
                $rectAgreement = $false
                $rectDeltas = 'unavailable'
                if ($actualRectValues.Count -eq 4) {
                    $deltaX = [Math]::Abs([long]$actualRectValues[0] - [long]$rect.X)
                    $deltaY = [Math]::Abs([long]$actualRectValues[1] - [long]$rect.Y)
                    $deltaWidth = [Math]::Abs([long]$actualRectValues[2] - [long]$rect.Width)
                    $deltaHeight = [Math]::Abs([long]$actualRectValues[3] - [long]$rect.Height)
                    $rectDeltas = "$deltaX,$deltaY,$deltaWidth,$deltaHeight"
                    $rectAgreement = (
                        $deltaX -le $presenterRectTolerancePixels -and
                        $deltaY -le $presenterRectTolerancePixels -and
                        $deltaWidth -le $presenterRectTolerancePixels -and
                        $deltaHeight -le $presenterRectTolerancePixels
                    )
                }
                if (
                    $actualHwnd -cne $expectedHwnd -or
                    -not $rectAgreement
                ) {
                    Add-Content -LiteralPath $captureDiagnosticsPath -Encoding utf8NoBOM -Value `
                        "attempt=$attempt stale_present_record expected_hwnd=$expectedHwnd actual_hwnd=$actualHwnd expected_rect=$expectedRect actual_rect=$actualRect rect_deltas=$rectDeltas tolerance=$presenterRectTolerancePixels"
                    $analysisExit = 3
                }
            }
            if ($analysisExit -eq 0) {
                if ($captureMode -eq 'paired-dwm') {
                    Copy-Item -LiteralPath $darkCapture -Destination (Join-Path $evidence 'overlay-dark.png') -Force
                    Copy-Item -LiteralPath $lightCapture -Destination (Join-Path $evidence 'overlay-light.png') -Force
                }
                elseif ($captureMode -eq 'hosted-arm64-presenter-surface') {
                    Copy-Item -LiteralPath $surfaceCapture -Destination (Join-Path $evidence 'overlay-present.bgra') -Force
                }
                else {
                    throw "unknown Windows overlay evidence mode: $captureMode"
                }
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
        throw "no exact $captureMode pose proved a complete side or top-down body, shade, outline, wing, warm articulation, and per-pixel alpha"
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
visual_capture=$captureMode
lifecycle=start,status,single-instance,reload,stop,immediate-restart,status,stop
"@
    Write-Output "Windows layered-overlay $captureMode and lifecycle smoke passed for $resolvedBinary ($initialHash)"
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
