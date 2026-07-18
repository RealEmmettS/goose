//! Windows overlay backend for honk300.
//!
//! One layered popup window per monitor, each presented via [`Overlay::present`] and
//! `UpdateLayeredWindow`. The engine simulates in signed virtual-desktop coordinates; the
//! backend clips dirty world-space render regions to each monitor window.
//!
//! Click-through is natural per-pixel alpha: we set `WS_EX_LAYERED` but **not**
//! `WS_EX_TRANSPARENT`, so opaque goose pixels receive clicks while transparent margins
//! fall through (plan §6). tiny-skia produces premultiplied RGBA; we feed
//! `UpdateLayeredWindow` premultiplied BGRA with `AC_SRC_ALPHA`. M7 also exposes a thin
//! `SetCursorPos` wrapper for the engine's platform-free cursor commands. M8 adds a
//! foreign-window move/size watcher that feeds platform-free perch-and-ride snapshots to
//! the engine without exposing HWNDs.

#![cfg(windows)]

mod tray;

pub use tray::StatusTray;

use honk_engine::collect_window::{
    collect_note_size, fit_collect_image, CollectWindowCloseOrigin, CollectWindowId,
    CollectWindowKind, CollectWindowRequestId, CollectWindowSnapshot,
};
use honk_engine::math::Rect;
use honk_engine::{ForeignWindowId, ForeignWindowSnapshot, PresenceSnapshot};
use honk_engine::{LocalTime, Vec2};
use std::collections::HashMap;
use std::collections::VecDeque;
use std::ffi::c_void;
use std::fs;
use std::io::Write;
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Mutex, OnceLock};
use tiny_skia::Pixmap;
use windows::core::{w, Error, Result, PCWSTR};
use windows::Win32::Foundation::{
    BOOL, COLORREF, HINSTANCE, HWND, LPARAM, LRESULT, POINT, RECT, SIZE, WPARAM,
};
use windows::Win32::Graphics::Gdi::{
    CreateCompatibleDC, CreateDIBSection, DeleteDC, DeleteObject, EnumDisplayMonitors, GetDC,
    GetMonitorInfoW, ReleaseDC, SelectObject, AC_SRC_ALPHA, AC_SRC_OVER, BITMAPINFO,
    BITMAPINFOHEADER, BI_RGB, BLENDFUNCTION, COLOR_WINDOW, DIB_RGB_COLORS, HBITMAP, HBRUSH, HDC,
    HGDIOBJ, HMONITOR, MONITORINFO,
};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::System::SystemInformation::GetLocalTime;
use windows::Win32::UI::Accessibility::{SetWinEventHook, UnhookWinEvent, HWINEVENTHOOK};
use windows::Win32::UI::HiDpi::{
    SetProcessDpiAwarenessContext, DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2,
};
use windows::Win32::UI::Input::KeyboardAndMouse::{GetAsyncKeyState, VK_LBUTTON};
use windows::Win32::UI::Shell::{
    SHQueryUserNotificationState, QUERY_USER_NOTIFICATION_STATE, QUNS_ACCEPTS_NOTIFICATIONS,
    QUNS_APP, QUNS_BUSY, QUNS_NOT_PRESENT, QUNS_PRESENTATION_MODE, QUNS_QUIET_TIME,
    QUNS_RUNNING_D3D_FULL_SCREEN,
};
use windows::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DefWindowProcW, DestroyWindow, DispatchMessageW, GetAncestor, GetClassNameW,
    GetClientRect, GetCursorPos, GetDlgItem, GetSystemMetrics, GetWindowLongPtrW, GetWindowRect,
    GetWindowTextW, IsIconic, IsWindow, IsWindowVisible, MoveWindow, PeekMessageW,
    RegisterClassExW, SetCursorPos, SetForegroundWindow, SetWindowLongPtrW, SetWindowPos,
    SetWindowTextW, ShowWindow, TranslateMessage, UpdateLayeredWindow, ES_AUTOVSCROLL,
    ES_MULTILINE, ES_WANTRETURN, EVENT_SYSTEM_MOVESIZEEND, EVENT_SYSTEM_MOVESIZESTART, GA_ROOT,
    GWL_EXSTYLE, HMENU, MONITORINFOF_PRIMARY, MSG, OBJID_WINDOW, PM_REMOVE, SM_CXSCREEN,
    SM_CXVIRTUALSCREEN, SM_CYSCREEN, SM_CYVIRTUALSCREEN, SM_XVIRTUALSCREEN, SM_YVIRTUALSCREEN,
    SWP_FRAMECHANGED, SWP_NOACTIVATE, SWP_NOMOVE, SWP_NOSIZE, SWP_NOZORDER, SW_HIDE,
    SW_SHOWNOACTIVATE, ULW_ALPHA, WINDOW_STYLE, WINEVENT_OUTOFCONTEXT, WINEVENT_SKIPOWNPROCESS,
    WM_CLOSE, WM_DESTROY, WM_DISPLAYCHANGE, WM_DPICHANGED, WM_QUIT, WM_SIZE, WNDCLASSEXW,
    WS_BORDER, WS_CHILD, WS_EX_LAYERED, WS_EX_NOACTIVATE, WS_EX_TOOLWINDOW, WS_EX_TOPMOST,
    WS_EX_TRANSPARENT, WS_OVERLAPPEDWINDOW, WS_POPUP, WS_VISIBLE, WS_VSCROLL,
};

/// Set on `WM_DPICHANGED`/`WM_DISPLAYCHANGE` from the overlay wndproc; drained by the runtime
/// via [`Overlay::take_monitors_changed`] so monitor topology changes rebuild the overlay set
/// and world bounds without racing across threads.
static MONITORS_DIRTY: AtomicBool = AtomicBool::new(false);

/// Opt the whole process into Per-Monitor-V2 DPI awareness. Must run **before** any HWND or
/// monitor enumeration (the runtime calls it first, ahead of [`Overlay::new`]).
///
/// With PMv2 the process sees physical pixels everywhere — `GetCursorPos`, `GetWindowRect`,
/// `EnumDisplayMonitors`, `SetCursorPos`, `SetWindowPos`, and `UpdateLayeredWindow` all agree in
/// one physical, signed virtual-desktop coordinate space, matching the engine's world coords with
/// no DPI scaling anywhere. Without this call an unmanifested process is DPI-unaware and Windows
/// silently virtualizes those coordinates on hiDPI monitors, blurring and mis-placing the overlay.
///
/// Idempotent and non-fatal: if awareness is already set for the process the call fails with
/// `ERROR_ACCESS_DENIED`, which we treat as success; any other failure is logged once and the
/// process keeps running (degrading to whatever awareness it already had).
pub fn init_dpi_awareness() {
    unsafe {
        if let Err(err) = SetProcessDpiAwarenessContext(DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2)
        {
            // HRESULT_FROM_WIN32(ERROR_ACCESS_DENIED) => awareness was already established
            // (e.g. by an embedded manifest); that is the desired end state, so stay quiet.
            const ALREADY_SET: windows::core::HRESULT =
                windows::core::HRESULT(0x8007_0005u32 as i32);
            if err.code() != ALREADY_SET {
                eprintln!("honk300: could not set Per-Monitor-V2 DPI awareness ({err})");
            }
        }
    }
}

/// Poll the global cursor position (desktop coordinates) and the left-button state.
/// Returns `(x, y, left_down)`. Desktop coordinates equal engine world coordinates across the
/// signed virtual desktop. Used to feed hit-testing (pat hover-streak + click→hyper) each frame.
pub fn pointer_state() -> (f32, f32, bool) {
    unsafe {
        let mut pt = POINT::default();
        let _ = GetCursorPos(&mut pt);
        // High bit of GetAsyncKeyState ⇒ the key is currently down.
        let left_down = (GetAsyncKeyState(VK_LBUTTON.0 as i32) as u16 & 0x8000) != 0;
        (pt.x as f32, pt.y as f32, left_down)
    }
}

/// Snapshot the local wall clock for the platform-free on-hour honk gate.
pub fn local_time() -> LocalTime {
    unsafe {
        let st = GetLocalTime();
        LocalTime {
            day: (st.wYear as i32 * 10_000) + (st.wMonth as i32 * 100) + st.wDay as i32,
            hour: st.wHour as u8,
            minute: st.wMinute as u8,
            second: st.wSecond as u8,
        }
    }
}

/// Snapshot Windows user notification state for platform-neutral calm-suppression gating.
pub fn presence_state() -> Result<PresenceSnapshot> {
    unsafe { SHQueryUserNotificationState().map(map_notification_state) }
}

fn map_notification_state(state: QUERY_USER_NOTIFICATION_STATE) -> PresenceSnapshot {
    match state {
        QUNS_ACCEPTS_NOTIFICATIONS => PresenceSnapshot::available(),
        QUNS_BUSY | QUNS_RUNNING_D3D_FULL_SCREEN => PresenceSnapshot::fullscreen(),
        QUNS_PRESENTATION_MODE | QUNS_NOT_PRESENT | QUNS_QUIET_TIME | QUNS_APP => {
            PresenceSnapshot::do_not_disturb()
        }
        _ => PresenceSnapshot::unsupported(),
    }
}

/// Warp the global cursor to a desktop/world-space coordinate.
pub fn warp_cursor(pos: Vec2) -> Result<()> {
    unsafe { SetCursorPos(pos.x.round() as i32, pos.y.round() as i32) }
}

/// Windows-side protected-window classes. These windows may be visually overlaid, but
/// goose mischief must not move, focus, type into, drag, ride, or otherwise manipulate them.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProtectedWindowClass {
    Terminal,
}

#[derive(Debug, Clone, Copy)]
struct RawMoveEvent {
    hwnd: isize,
    started: bool,
}

static MOVE_EVENTS: OnceLock<Mutex<VecDeque<RawMoveEvent>>> = OnceLock::new();
static COLLECT_USER_CLOSES: OnceLock<Mutex<VecDeque<isize>>> = OnceLock::new();

fn move_events() -> &'static Mutex<VecDeque<RawMoveEvent>> {
    MOVE_EVENTS.get_or_init(|| Mutex::new(VecDeque::new()))
}

fn collect_user_closes() -> &'static Mutex<VecDeque<isize>> {
    COLLECT_USER_CLOSES.get_or_init(|| Mutex::new(VecDeque::new()))
}

fn record_collect_user_close(hwnd: HWND) {
    if let Ok(mut closes) = collect_user_closes().lock() {
        let key = hwnd_key(hwnd);
        if !closes.contains(&key) {
            closes.push_back(key);
        }
    }
}

fn take_collect_user_close(hwnd: HWND) -> bool {
    let Ok(mut closes) = collect_user_closes().lock() else {
        return false;
    };
    let key = hwnd_key(hwnd);
    let Some(index) = closes.iter().position(|candidate| *candidate == key) else {
        return false;
    };
    closes.remove(index);
    true
}

fn hwnd_key(hwnd: HWND) -> isize {
    hwnd.0 as isize
}

fn hwnd_from_key(key: isize) -> HWND {
    HWND(key as *mut c_void)
}

unsafe extern "system" fn move_event_proc(
    _hook: HWINEVENTHOOK,
    event: u32,
    hwnd: HWND,
    idobject: i32,
    _idchild: i32,
    _ideventthread: u32,
    _dwmseventtime: u32,
) {
    if hwnd.0.is_null() || idobject != OBJID_WINDOW.0 {
        return;
    }

    let started = match event {
        EVENT_SYSTEM_MOVESIZESTART => true,
        EVENT_SYSTEM_MOVESIZEEND => false,
        _ => return,
    };

    if let Ok(mut events) = move_events().lock() {
        events.push_back(RawMoveEvent {
            hwnd: hwnd_key(hwnd),
            started,
        });
        while events.len() > 64 {
            events.pop_front();
        }
    }
}

/// Watches user-initiated foreign-window move/resize operations for M8 perch-and-ride.
pub struct ForeignWindowWatcher {
    hook: HWINEVENTHOOK,
    overlay_hwnds: Vec<isize>,
    active: Option<isize>,
}

impl ForeignWindowWatcher {
    /// Register an out-of-context move/size WinEvent hook.
    pub fn new(overlay: &Overlay) -> Result<Self> {
        let hook = unsafe {
            SetWinEventHook(
                EVENT_SYSTEM_MOVESIZESTART,
                EVENT_SYSTEM_MOVESIZEEND,
                None,
                Some(move_event_proc),
                0,
                0,
                WINEVENT_OUTOFCONTEXT | WINEVENT_SKIPOWNPROCESS,
            )
        };

        if hook.0.is_null() {
            return Err(Error::from_win32());
        }

        Ok(Self {
            hook,
            overlay_hwnds: overlay.hwnd_keys(),
            active: None,
        })
    }

    /// Drain queued move/size events and return the current active drag snapshot, if any.
    pub fn active_drag(&mut self) -> Result<Option<ForeignWindowSnapshot>> {
        self.drain_events();
        let Some(hwnd) = self.active.map(hwnd_from_key) else {
            return Ok(None);
        };
        if !is_foreign_top_level_window(hwnd, &self.overlay_hwnds) {
            self.active = None;
            return Ok(None);
        }

        let rect = window_rect(hwnd)?;
        Ok(Some(ForeignWindowSnapshot::top_center(
            ForeignWindowId(hwnd_key(hwnd) as u64),
            rect,
        )))
    }

    fn drain_events(&mut self) {
        if let Ok(mut events) = move_events().lock() {
            while let Some(event) = events.pop_front() {
                let hwnd = hwnd_from_key(event.hwnd);
                if event.started {
                    if is_foreign_top_level_window(hwnd, &self.overlay_hwnds) {
                        self.active = Some(event.hwnd);
                    }
                } else if self.active == Some(event.hwnd) {
                    self.active = None;
                }
            }
        }
    }
}

impl Drop for ForeignWindowWatcher {
    fn drop(&mut self) {
        unsafe {
            let _ = UnhookWinEvent(self.hook);
        }
    }
}

fn is_foreign_top_level_window(hwnd: HWND, overlay_hwnds: &[isize]) -> bool {
    unsafe {
        if hwnd.0.is_null() || overlay_hwnds.contains(&hwnd_key(hwnd)) {
            return false;
        }
        if !IsWindow(hwnd).as_bool() || !IsWindowVisible(hwnd).as_bool() || IsIconic(hwnd).as_bool()
        {
            return false;
        }
        let root = GetAncestor(hwnd, GA_ROOT);
        !root.0.is_null()
            && hwnd_key(root) == hwnd_key(hwnd)
            && protected_window_class(hwnd).is_none()
    }
}

fn window_rect(hwnd: HWND) -> Result<Rect> {
    unsafe {
        let mut rect = RECT::default();
        GetWindowRect(hwnd, &mut rect)?;
        Ok(rect_from_win32(rect))
    }
}

fn protected_window_class(hwnd: HWND) -> Option<ProtectedWindowClass> {
    classify_protected_window(&window_class_name(hwnd), &window_title(hwnd))
}

fn window_class_name(hwnd: HWND) -> String {
    let mut buf = [0u16; 256];
    let len = unsafe { GetClassNameW(hwnd, &mut buf) };
    String::from_utf16_lossy(&buf[..len.max(0) as usize])
}

fn window_title(hwnd: HWND) -> String {
    let mut buf = [0u16; 512];
    let len = unsafe { GetWindowTextW(hwnd, &mut buf) };
    String::from_utf16_lossy(&buf[..len.max(0) as usize])
}

pub fn classify_protected_window(class_name: &str, title: &str) -> Option<ProtectedWindowClass> {
    let class = class_name.to_ascii_lowercase();
    let title = title.to_ascii_lowercase();

    let terminal_class = [
        "consolewindowclass",
        "cascadia_hosting_window_class",
        "virtualconsoleclass",
        "mintty",
        "wezterm",
        "alacritty",
        "kitty",
        "tabby",
    ]
    .iter()
    .any(|needle| class.contains(needle));
    let terminal_title = [
        "windows terminal",
        "terminal",
        "command prompt",
        "cmd.exe",
        "powershell",
        "pwsh",
        "git bash",
        "mingw64",
        "msys2",
        "wsl",
        "ubuntu",
        "debian",
        "kali",
        "alacritty",
        "wezterm",
        "mintty",
        "visual studio code",
        "codex",
        "chatgpt",
    ]
    .iter()
    .any(|needle| title.contains(needle));

    (terminal_class || terminal_title).then_some(ProtectedWindowClass::Terminal)
}

fn rect_from_win32(rect: RECT) -> Rect {
    Rect {
        min: Vec2::new(rect.left as f32, rect.top as f32),
        max: Vec2::new(rect.right as f32, rect.bottom as f32),
    }
}

enum ControlledWindow {
    Note(NoteWindow),
    Image(ImageWindow),
}

impl ControlledWindow {
    fn hwnd(&self) -> Option<HWND> {
        match self {
            Self::Note(window) => Some(window.hwnd),
            Self::Image(window) => Some(window.hwnd),
        }
    }

    fn request(&self) -> CollectWindowRequestId {
        match self {
            Self::Note(window) => window.request,
            Self::Image(window) => window.request,
        }
    }

    fn kind(&self) -> CollectWindowKind {
        match self {
            Self::Note(_) => CollectWindowKind::Note,
            Self::Image(_) => CollectWindowKind::Meme,
        }
    }
}

const NOTE_EDIT_ID: i32 = 1;

/// A Honk300-owned native text window. Windows 11 Notepad restores a user's prior tab session
/// when launched without a file, so delegating notes to `notepad.exe` can surface and manipulate
/// unrelated documents. Keeping the editable control in-process guarantees exact ownership.
struct NoteWindow {
    request: CollectWindowRequestId,
    hwnd: HWND,
    edit: HWND,
}

impl NoteWindow {
    fn new(request: CollectWindowRequestId, top_left: Vec2, display_bounds: Rect) -> Result<Self> {
        unsafe {
            let hmodule = GetModuleHandleW(None)?;
            let hinstance = HINSTANCE(hmodule.0);
            let class_name = w!("honk300_collect_note");
            let wc = WNDCLASSEXW {
                cbSize: std::mem::size_of::<WNDCLASSEXW>() as u32,
                lpfnWndProc: Some(note_wndproc),
                hInstance: hinstance,
                hbrBackground: HBRUSH((COLOR_WINDOW.0 + 1) as usize as *mut c_void),
                lpszClassName: class_name,
                ..Default::default()
            };
            RegisterClassExW(&wc);

            let size = collect_note_size(display_bounds);
            let (width, height) = (size.x as i32, size.y as i32);
            let hwnd = CreateWindowExW(
                WS_EX_TOOLWINDOW,
                class_name,
                w!("A note from Honk300"),
                WS_OVERLAPPEDWINDOW,
                top_left.x.round() as i32,
                top_left.y.round() as i32,
                width,
                height,
                None,
                None,
                hinstance,
                None,
            )?;
            let edit_style = WS_CHILD
                | WS_VISIBLE
                | WS_BORDER
                | WS_VSCROLL
                | WINDOW_STYLE((ES_MULTILINE | ES_AUTOVSCROLL | ES_WANTRETURN) as u32);
            let edit = CreateWindowExW(
                Default::default(),
                w!("EDIT"),
                w!(""),
                edit_style,
                0,
                0,
                width,
                height,
                hwnd,
                HMENU(NOTE_EDIT_ID as usize as *mut c_void),
                hinstance,
                None,
            )?;
            let _ = ShowWindow(hwnd, SW_SHOWNOACTIVATE);
            resize_note_edit(hwnd);
            Ok(Self {
                request,
                hwnd,
                edit,
            })
        }
    }

    fn move_to(&self, top_left: Vec2) -> Result<()> {
        move_hwnd(self.hwnd, top_left)
    }

    fn set_text(&self, text: &str) -> Result<()> {
        let text_w: Vec<u16> = text.encode_utf16().chain(std::iter::once(0)).collect();
        unsafe { SetWindowTextW(self.edit, PCWSTR(text_w.as_ptr())) }
    }
}

impl Drop for NoteWindow {
    fn drop(&mut self) {
        unsafe {
            let _ = DestroyWindow(self.hwnd);
        }
    }
}

struct ImageWindow {
    request: CollectWindowRequestId,
    hwnd: HWND,
    pixmap: Pixmap,
    dib: Option<Dib>,
}

impl ImageWindow {
    fn new(
        request: CollectWindowRequestId,
        title: &str,
        pixmap: &Pixmap,
        top_left: Vec2,
        display_bounds: Rect,
    ) -> Result<Self> {
        unsafe {
            let pixmap = fit_collect_image(pixmap, display_bounds)
                .ok_or_else(|| error_from_message("could not allocate a bounded collect image"))?;
            let hmodule = GetModuleHandleW(None)?;
            let hinstance = HINSTANCE(hmodule.0);
            let class_name = w!("honk300_collect_image");
            let wc = WNDCLASSEXW {
                cbSize: std::mem::size_of::<WNDCLASSEXW>() as u32,
                lpfnWndProc: Some(image_wndproc),
                hInstance: hinstance,
                lpszClassName: class_name,
                ..Default::default()
            };
            RegisterClassExW(&wc);

            let title_w: Vec<u16> = title.encode_utf16().chain(std::iter::once(0)).collect();
            let hwnd = CreateWindowExW(
                WS_EX_LAYERED | WS_EX_TOOLWINDOW | WS_EX_NOACTIVATE,
                class_name,
                PCWSTR(title_w.as_ptr()),
                WS_POPUP,
                top_left.x.round() as i32,
                top_left.y.round() as i32,
                pixmap.width() as i32,
                pixmap.height() as i32,
                None,
                None,
                hinstance,
                None,
            )?;
            let _ = ShowWindow(hwnd, SW_SHOWNOACTIVATE);

            let mut window = Self {
                request,
                hwnd,
                pixmap,
                dib: None,
            };
            window.present_at(top_left)?;
            Ok(window)
        }
    }

    fn present_at(&mut self, top_left: Vec2) -> Result<()> {
        present_layered(
            self.hwnd,
            &mut self.dib,
            &self.pixmap,
            top_left.x.round() as i32,
            top_left.y.round() as i32,
        )
    }
}

impl Drop for ImageWindow {
    fn drop(&mut self) {
        unsafe {
            let _ = DestroyWindow(self.hwnd);
        }
    }
}

fn preferred_collect_window_id(
    active: Option<(CollectWindowRequestId, CollectWindowKind)>,
    candidates: impl IntoIterator<Item = (CollectWindowId, CollectWindowRequestId, CollectWindowKind)>,
) -> Option<CollectWindowId> {
    // A lingering note or meme may outlive the engine task that created it. Prefer the currently
    // spawned typed request so HashMap iteration order cannot starve a newer task.
    let mut fallback = None;
    for (id, request, kind) in candidates {
        if active == Some((request, kind)) {
            return Some(id);
        }
        fallback.get_or_insert(id);
    }
    fallback
}

/// Applies M9 collect-window commands through Win32 without exposing HWNDs to `honk-engine`.
pub struct CollectWindowController {
    next_id: u64,
    windows: HashMap<CollectWindowId, ControlledWindow>,
    last_rects: HashMap<CollectWindowId, Rect>,
    active_request: Option<(CollectWindowRequestId, CollectWindowKind)>,
    spawn_top_left: Vec2,
    display_bounds: Rect,
}

impl CollectWindowController {
    pub fn new(bounds: Rect) -> Self {
        Self {
            next_id: 1,
            windows: HashMap::new(),
            last_rects: HashMap::new(),
            active_request: None,
            spawn_top_left: Vec2::new(bounds.min.x + 40.0, bounds.min.y + 80.0),
            display_bounds: bounds,
        }
    }

    pub fn update_display_bounds(&mut self, bounds: Rect) {
        self.display_bounds = bounds;
        self.spawn_top_left = Vec2::new(bounds.min.x + 40.0, bounds.min.y + 80.0);
    }

    pub fn spawn_note(&mut self, request: CollectWindowRequestId) -> Result<CollectWindowId> {
        if let Some(id) = self.find_request(request, CollectWindowKind::Note) {
            self.active_request = Some((request, CollectWindowKind::Note));
            return Ok(id);
        }
        let id = self.alloc_id();
        let window = NoteWindow::new(request, self.spawn_top_left, self.display_bounds)?;
        self.windows.insert(id, ControlledWindow::Note(window));
        self.active_request = Some((request, CollectWindowKind::Note));
        Ok(id)
    }

    pub fn spawn_image(
        &mut self,
        request: CollectWindowRequestId,
        title: &str,
        pixmap: &Pixmap,
    ) -> Result<CollectWindowId> {
        if let Some(id) = self.find_request(request, CollectWindowKind::Meme) {
            self.active_request = Some((request, CollectWindowKind::Meme));
            return Ok(id);
        }
        let id = self.alloc_id();
        let window = ImageWindow::new(
            request,
            title,
            pixmap,
            self.spawn_top_left,
            self.display_bounds,
        )?;
        self.windows.insert(id, ControlledWindow::Image(window));
        self.active_request = Some((request, CollectWindowKind::Meme));
        Ok(id)
    }

    pub fn move_window(&mut self, id: CollectWindowId, top_left: Vec2) -> Result<()> {
        match self.windows.get_mut(&id) {
            Some(ControlledWindow::Note(window)) => window.move_to(top_left),
            Some(ControlledWindow::Image(window)) => window.present_at(top_left),
            None => Ok(()),
        }
    }

    pub fn set_passthrough(&mut self, id: CollectWindowId, passthrough: bool) -> Result<()> {
        if let Some(Some(hwnd)) = self.windows.get(&id).map(ControlledWindow::hwnd) {
            set_passthrough(hwnd, passthrough)?;
        }
        Ok(())
    }

    pub fn focus(&self, id: CollectWindowId) -> Result<()> {
        if let Some(Some(hwnd)) = self.windows.get(&id).map(ControlledWindow::hwnd) {
            unsafe {
                if !SetForegroundWindow(hwnd).as_bool() {
                    return Err(Error::from_win32());
                }
            }
        }
        Ok(())
    }

    pub fn type_text(&mut self, id: CollectWindowId, text: &str) -> Result<()> {
        if let Some(ControlledWindow::Note(window)) = self.windows.get_mut(&id) {
            window.set_text(text)?;
        }
        Ok(())
    }

    pub fn close(&mut self, id: CollectWindowId) {
        // Dropping the entry destroys only the Honk300-owned native note/image window.
        if let Some(window) = self.windows.remove(&id) {
            if self.active_request == Some((window.request(), window.kind())) {
                self.active_request = None;
            }
        }
        self.last_rects.remove(&id);
    }

    pub fn snapshot(&mut self) -> Option<CollectWindowSnapshot> {
        // Cull any native collect window that has died, then report the active request's geometry.
        let mut dead = Vec::new();
        for (id, window) in self.windows.iter_mut() {
            let close_origin = match window {
                ControlledWindow::Note(note) => unsafe {
                    (!IsWindow(note.hwnd).as_bool()).then(|| {
                        if take_collect_user_close(note.hwnd) {
                            CollectWindowCloseOrigin::User
                        } else {
                            CollectWindowCloseOrigin::Program
                        }
                    })
                },
                ControlledWindow::Image(image) => unsafe {
                    (!IsWindow(image.hwnd).as_bool()).then(|| {
                        if take_collect_user_close(image.hwnd) {
                            CollectWindowCloseOrigin::User
                        } else {
                            CollectWindowCloseOrigin::Program
                        }
                    })
                },
            };
            if let Some(origin) = close_origin {
                dead.push((*id, origin));
            }
        }
        if let Some((id, close_origin)) = dead.first().copied() {
            let window = self.windows.remove(&id).expect("dead window id");
            if self.active_request == Some((window.request(), window.kind())) {
                self.active_request = None;
            }
            let rect = self
                .last_rects
                .remove(&id)
                .unwrap_or_else(|| Rect::new(Vec2::ZERO, Vec2::ZERO));
            return Some(CollectWindowSnapshot {
                id,
                request: window.request(),
                kind: window.kind(),
                rect,
                alive: false,
                close_origin: Some(close_origin),
            });
        }
        let mut live = Vec::new();
        for (id, window) in self.windows.iter() {
            let Some(hwnd) = window.hwnd() else {
                continue;
            };
            if let Ok(rect) = window_rect(hwnd) {
                self.last_rects.insert(*id, rect);
                live.push((*id, window.request(), window.kind(), rect));
            }
        }
        let id = preferred_collect_window_id(
            self.active_request,
            live.iter()
                .map(|(id, request, kind, _)| (*id, *request, *kind)),
        )?;
        live.into_iter()
            .find(|(candidate, _, _, _)| *candidate == id)
            .map(|(id, request, kind, rect)| CollectWindowSnapshot {
                id,
                request,
                kind,
                rect,
                alive: true,
                close_origin: None,
            })
    }

    fn alloc_id(&mut self) -> CollectWindowId {
        let id = CollectWindowId(self.next_id);
        self.next_id += 1;
        id
    }

    fn find_request(
        &self,
        request: CollectWindowRequestId,
        kind: CollectWindowKind,
    ) -> Option<CollectWindowId> {
        self.windows.iter().find_map(|(id, window)| {
            (window.request() == request && window.kind() == kind).then_some(*id)
        })
    }
}

impl Drop for CollectWindowController {
    fn drop(&mut self) {
        // Clear passthrough before the per-window drops destroy every Honk300-owned note/image.
        for window in self.windows.values() {
            if let Some(hwnd) = window.hwnd() {
                let _ = set_passthrough(hwnd, false);
            }
        }
    }
}

fn move_hwnd(hwnd: HWND, top_left: Vec2) -> Result<()> {
    unsafe {
        SetWindowPos(
            hwnd,
            None,
            top_left.x.round() as i32,
            top_left.y.round() as i32,
            0,
            0,
            SWP_NOSIZE | SWP_NOZORDER | SWP_NOACTIVATE,
        )
    }
}

fn set_passthrough(hwnd: HWND, passthrough: bool) -> Result<()> {
    unsafe {
        let style = GetWindowLongPtrW(hwnd, GWL_EXSTYLE);
        let transparent = WS_EX_TRANSPARENT.0 as isize;
        let next = if passthrough {
            style | transparent
        } else {
            style & !transparent
        };
        SetWindowLongPtrW(hwnd, GWL_EXSTYLE, next);
        SetWindowPos(
            hwnd,
            None,
            0,
            0,
            0,
            0,
            SWP_NOMOVE | SWP_NOSIZE | SWP_NOZORDER | SWP_NOACTIVATE | SWP_FRAMECHANGED,
        )
    }
}

fn present_layered(
    hwnd: HWND,
    dib: &mut Option<Dib>,
    pixmap: &Pixmap,
    dest_x: i32,
    dest_y: i32,
) -> Result<()> {
    let width = pixmap.width() as i32;
    let height = pixmap.height() as i32;
    if width == 0 || height == 0 {
        return Ok(());
    }

    unsafe {
        if dib
            .as_ref()
            .map(|d| d.width != width || d.height != height)
            .unwrap_or(true)
        {
            *dib = Some(Dib::new(width, height)?);
        }
        let dib = dib.as_ref().expect("dib just set");

        let src = pixmap.data();
        let count = (width * height) as usize;
        let dst = std::slice::from_raw_parts_mut(dib.bits, count * 4);
        copy_premultiplied_rgba_to_bgra(src, dst);

        let screen = GetDC(None);
        let dest = POINT {
            x: dest_x,
            y: dest_y,
        };
        let size = SIZE {
            cx: width,
            cy: height,
        };
        let src_pt = POINT { x: 0, y: 0 };
        let blend = BLENDFUNCTION {
            BlendOp: AC_SRC_OVER as u8,
            BlendFlags: 0,
            SourceConstantAlpha: 255,
            AlphaFormat: AC_SRC_ALPHA as u8,
        };
        let result = UpdateLayeredWindow(
            hwnd,
            screen,
            Some(&dest as *const POINT),
            Some(&size as *const SIZE),
            dib.hdc,
            Some(&src_pt as *const POINT),
            COLORREF(0),
            Some(&blend as *const BLENDFUNCTION),
            ULW_ALPHA,
        );
        ReleaseDC(None, screen);
        result
    }
}

fn copy_premultiplied_rgba_to_bgra(src: &[u8], dst: &mut [u8]) {
    assert_eq!(src.len(), dst.len());
    assert_eq!(src.len() % 4, 0);
    for (rgba, bgra) in src.chunks_exact(4).zip(dst.chunks_exact_mut(4)) {
        bgra.copy_from_slice(&[rgba[2], rgba[1], rgba[0], rgba[3]]);
    }
}

fn error_from_message(message: impl Into<String>) -> Error {
    Error::new(
        windows::core::HRESULT(0x8000_4005u32 as i32),
        message.into(),
    )
}

/// A reusable top-down 32-bpp DIB section we blit the goose into each frame.
struct Dib {
    hdc: HDC,
    bitmap: HBITMAP,
    old: HGDIOBJ,
    bits: *mut u8,
    width: i32,
    height: i32,
}

impl Dib {
    /// Create a `width`×`height` premultiplied-BGRA DIB selected into a memory DC.
    unsafe fn new(width: i32, height: i32) -> Result<Dib> {
        let screen = GetDC(None);
        let hdc = CreateCompatibleDC(screen);
        ReleaseDC(None, screen);

        let bmi = BITMAPINFO {
            bmiHeader: BITMAPINFOHEADER {
                biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
                biWidth: width,
                biHeight: -height, // negative ⇒ top-down rows
                biPlanes: 1,
                biBitCount: 32,
                biCompression: BI_RGB.0,
                ..Default::default()
            },
            ..Default::default()
        };

        let mut bits: *mut c_void = std::ptr::null_mut();
        let bitmap = CreateDIBSection(hdc, &bmi, DIB_RGB_COLORS, &mut bits, None, 0)?;
        let old = SelectObject(hdc, HGDIOBJ(bitmap.0));

        Ok(Dib {
            hdc,
            bitmap,
            old,
            bits: bits as *mut u8,
            width,
            height,
        })
    }
}

impl Drop for Dib {
    fn drop(&mut self) {
        unsafe {
            SelectObject(self.hdc, self.old);
            let _ = DeleteObject(HGDIOBJ(self.bitmap.0));
            let _ = DeleteDC(self.hdc);
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct MonitorBounds {
    id: isize,
    bounds: Rect,
    primary: bool,
}

/// The honk300 desktop overlay: one always-on-top, click-through-where-transparent
/// layered window per monitor.
pub struct Overlay {
    windows: Vec<OverlayWindow>,
    virtual_bounds: Rect,
    primary_bounds: Rect,
}

struct OverlayWindow {
    monitor_id: isize,
    hwnd: HWND,
    dib: Option<Dib>,
    bounds: Rect,
    visible: bool,
}

impl OverlayWindow {
    unsafe fn new(
        hinstance: HINSTANCE,
        class_name: PCWSTR,
        monitor_id: isize,
        bounds: Rect,
    ) -> Result<Self> {
        let hwnd = CreateWindowExW(
            WS_EX_LAYERED | WS_EX_TOOLWINDOW | WS_EX_TOPMOST | WS_EX_NOACTIVATE,
            class_name,
            w!("honk300"),
            WS_POPUP,
            bounds.min.x.floor() as i32,
            bounds.min.y.floor() as i32,
            1,
            1,
            None,
            None,
            hinstance,
            None,
        )?;
        let _ = ShowWindow(hwnd, SW_HIDE);
        Ok(Self {
            monitor_id,
            hwnd,
            dib: None,
            bounds,
            visible: false,
        })
    }

    fn present(&mut self, dirty: Rect, pixmap: &Pixmap) -> Result<()> {
        let Some(intersection) = dirty.intersection(self.bounds).map(Rect::pixel_aligned) else {
            self.hide();
            return Ok(());
        };
        let src_x = (intersection.min.x - dirty.min.x).round().max(0.0) as u32;
        let src_y = (intersection.min.y - dirty.min.y).round().max(0.0) as u32;
        let width = intersection.width().round().max(1.0) as u32;
        let height = intersection.height().round().max(1.0) as u32;
        let cropped = crop_pixmap(pixmap, src_x, src_y, width, height)
            .ok_or_else(|| error_from_message("could not allocate monitor crop"))?;
        present_layered(
            self.hwnd,
            &mut self.dib,
            &cropped,
            intersection.min.x as i32,
            intersection.min.y as i32,
        )?;
        if let Some(dib) = self.dib.as_ref() {
            maybe_write_presented_smoke_frame(
                self.hwnd,
                dib,
                intersection.min.x as i32,
                intersection.min.y as i32,
            );
        }
        if !self.visible {
            unsafe {
                let _ = ShowWindow(self.hwnd, SW_SHOWNOACTIVATE);
            }
            self.visible = true;
        }
        Ok(())
    }

    fn hide(&mut self) {
        if self.visible {
            unsafe {
                let _ = ShowWindow(self.hwnd, SW_HIDE);
            }
            self.visible = false;
        }
    }

    /// Reassign this window's monitor bounds after a topology change. The next `present` clips
    /// and repositions to the new bounds; hide now so a stale surface isn't left at the old
    /// position in the interim.
    fn set_bounds(&mut self, bounds: Rect) {
        if self.bounds == bounds {
            return;
        }
        self.bounds = bounds;
        self.hide();
    }
}

impl Drop for OverlayWindow {
    fn drop(&mut self) {
        unsafe {
            let _ = DestroyWindow(self.hwnd);
        }
    }
}

impl Overlay {
    /// Register the window class and create one initially hidden layered window per monitor.
    pub fn new() -> Result<Overlay> {
        unsafe {
            let hmodule = GetModuleHandleW(None)?;
            let hinstance = HINSTANCE(hmodule.0);
            let class_name = w!("honk300_overlay");

            let wc = WNDCLASSEXW {
                cbSize: std::mem::size_of::<WNDCLASSEXW>() as u32,
                lpfnWndProc: Some(wndproc),
                hInstance: hinstance,
                lpszClassName: class_name,
                ..Default::default()
            };
            RegisterClassExW(&wc);

            let monitors = enumerate_monitor_bounds();
            let virtual_bounds = monitor_union(&monitors);
            let primary_bounds = primary_monitor_bounds(&monitors);
            let mut windows = Vec::with_capacity(monitors.len());
            for monitor in monitors {
                windows.push(OverlayWindow::new(
                    hinstance,
                    class_name,
                    monitor.id,
                    monitor.bounds,
                )?);
            }

            Ok(Overlay {
                windows,
                virtual_bounds,
                primary_bounds,
            })
        }
    }

    /// The full virtual-desktop bounds reported by the current overlay monitor set.
    pub fn virtual_desktop_bounds(&self) -> Rect {
        self.virtual_bounds
    }

    /// The primary monitor bounds reported by the current overlay monitor set.
    pub fn primary_monitor_bounds(&self) -> Rect {
        self.primary_bounds
    }

    /// Current physical-pixel bounds for every active monitor, in overlay-window order.
    pub fn monitor_bounds(&self) -> Vec<Rect> {
        self.windows.iter().map(|window| window.bounds).collect()
    }

    /// Consume the pending monitor-topology-change flag raised by the wndproc on
    /// `WM_DPICHANGED`/`WM_DISPLAYCHANGE`. Returns `true` at most once per change so the runtime
    /// can rebuild in the same loop iteration it observes the flag.
    pub fn take_monitors_changed(&self) -> bool {
        MONITORS_DIRTY.swap(false, Ordering::SeqCst)
    }

    /// Re-enumerate monitors and reconcile the per-monitor overlay windows against the new set,
    /// reusing existing HWNDs where the count is unchanged (the common DPI/resolution case) and
    /// only creating/destroying windows for the added/removed-monitor delta. Reusing HWNDs keeps
    /// [`ForeignWindowWatcher`]'s overlay-HWND filter valid across same-count rebuilds.
    ///
    /// Returns `true` when the virtual-desktop or primary bounds changed, signalling the runtime
    /// to recompute and apply world bounds.
    pub fn rebuild_monitors(&mut self) -> Result<bool> {
        let monitors = enumerate_monitor_bounds();
        let virtual_bounds = monitor_union(&monitors);
        let primary_bounds = primary_monitor_bounds(&monitors);

        unsafe {
            let hmodule = GetModuleHandleW(None)?;
            let hinstance = HINSTANCE(hmodule.0);
            let class_name = w!("honk300_overlay");
            let mut existing = self
                .windows
                .drain(..)
                .map(|window| (window.monitor_id, window))
                .collect::<HashMap<_, _>>();
            let mut reconciled = Vec::with_capacity(monitors.len());
            for monitor in &monitors {
                let mut window = match existing.remove(&monitor.id) {
                    Some(window) => window,
                    None => OverlayWindow::new(hinstance, class_name, monitor.id, monitor.bounds)?,
                };
                window.set_bounds(monitor.bounds);
                reconciled.push(window);
            }
            // Only entries left in `existing` belonged to removed monitors. Dropping that map
            // destroys exactly those HWNDs; surviving monitors retain their window identity.
            self.windows = reconciled;
        }

        let changed =
            virtual_bounds != self.virtual_bounds || primary_bounds != self.primary_bounds;
        self.virtual_bounds = virtual_bounds;
        self.primary_bounds = primary_bounds;
        Ok(changed)
    }

    /// HWND keys for all overlay windows, used to filter them from foreign-window watching.
    fn hwnd_keys(&self) -> Vec<isize> {
        self.windows
            .iter()
            .map(|window| hwnd_key(window.hwnd))
            .collect()
    }

    /// The full virtual-desktop bounds using Win32 system metrics.
    pub fn virtual_bounds() -> Rect {
        unsafe {
            let x = GetSystemMetrics(SM_XVIRTUALSCREEN) as f32;
            let y = GetSystemMetrics(SM_YVIRTUALSCREEN) as f32;
            let w = GetSystemMetrics(SM_CXVIRTUALSCREEN) as f32;
            let h = GetSystemMetrics(SM_CYVIRTUALSCREEN) as f32;
            Rect {
                min: Vec2::new(x, y),
                max: Vec2::new(x + w, y + h),
            }
        }
    }

    /// The primary monitor's bounds from Win32 system metrics.
    pub fn primary_bounds() -> Rect {
        unsafe {
            let w = GetSystemMetrics(SM_CXSCREEN) as f32;
            let h = GetSystemMetrics(SM_CYSCREEN) as f32;
            Rect {
                min: Vec2::new(0.0, 0.0),
                max: Vec2::new(w, h),
            }
        }
    }

    /// Drain pending window messages. Returns `false` when a window is closing
    /// (`WM_QUIT`), signalling the caller to exit the loop.
    pub fn pump(&mut self) -> bool {
        unsafe {
            let mut msg = MSG::default();
            while PeekMessageW(&mut msg, None, 0, 0, PM_REMOVE).as_bool() {
                if msg.message == WM_QUIT {
                    return false;
                }
                let _ = TranslateMessage(&msg);
                DispatchMessageW(&msg);
            }
            true
        }
    }

    /// Present a dirty world-space region. The pixmap's top-left pixel must correspond to
    /// `dirty.min`; this method clips and crops it for each monitor window.
    pub fn present(&mut self, dirty: Rect, pixmap: &Pixmap) -> Result<()> {
        let dirty = dirty.pixel_aligned();
        if dirty.width() <= 0.0 || dirty.height() <= 0.0 {
            return Ok(());
        }
        for window in &mut self.windows {
            window.present(dirty, pixmap)?;
        }
        Ok(())
    }
}

/// Preserve the exact premultiplied-BGRA DIB accepted by `UpdateLayeredWindow` when a native
/// smoke requests it.
///
/// This is called only after the real per-monitor crop, RGBA-to-BGRA bridge, and successful native
/// present. The path is intentionally write-once: the harness removes the completed record only
/// after its pose delay, and the next successful present writes through a sibling temporary before
/// renaming. The custom record retains the raw DIB bytes plus the exact HWND/rectangle instead of
/// passing through PNG's straight-alpha conversion.
fn maybe_write_presented_smoke_frame(hwnd: HWND, dib: &Dib, dest_x: i32, dest_y: i32) {
    let Ok(path) = std::env::var("HONK300_WINDOWS_SMOKE_PRESENT") else {
        return;
    };
    let path = Path::new(path.trim());
    if path.as_os_str().is_empty() || path.exists() {
        return;
    }
    let temporary = path.with_extension(format!("{}.tmp", std::process::id()));
    let _ = fs::remove_file(&temporary);
    if write_presented_smoke_frame(&temporary, hwnd, dib, dest_x, dest_y)
        .and_then(|()| fs::rename(&temporary, path))
        .is_err()
    {
        let _ = fs::remove_file(&temporary);
    }
}

fn write_presented_smoke_frame(
    path: &Path,
    hwnd: HWND,
    dib: &Dib,
    dest_x: i32,
    dest_y: i32,
) -> std::io::Result<()> {
    let width = usize::try_from(dib.width).map_err(std::io::Error::other)?;
    let height = usize::try_from(dib.height).map_err(std::io::Error::other)?;
    let byte_count = width
        .checked_mul(height)
        .and_then(|pixels| pixels.checked_mul(4))
        .ok_or_else(|| std::io::Error::other("presented surface is too large"))?;
    // SAFETY: `Dib::new` allocates exactly width * height * four bytes and keeps that allocation
    // alive until `dib` is dropped. This read occurs synchronously while the immutable borrow is
    // live, after UpdateLayeredWindow has accepted the same selected DIB.
    let bytes = unsafe { std::slice::from_raw_parts(dib.bits, byte_count) };
    let header = format!(
        "HONK300_LAYERED_BGRA_V1\nhwnd=0x{:X}\nx={dest_x}\ny={dest_y}\nwidth={}\nheight={}\nstride={}\nbytes={byte_count}\n\n",
        hwnd_key(hwnd),
        dib.width,
        dib.height,
        width * 4,
    );
    let mut file = fs::File::create(path)?;
    file.write_all(header.as_bytes())?;
    file.write_all(bytes)?;
    file.sync_all()
}

fn crop_pixmap(pixmap: &Pixmap, src_x: u32, src_y: u32, width: u32, height: u32) -> Option<Pixmap> {
    if src_x.checked_add(width)? > pixmap.width() || src_y.checked_add(height)? > pixmap.height() {
        return None;
    }
    let mut cropped = Pixmap::new(width, height)?;
    let src_stride = pixmap.width() as usize * 4;
    let dst_stride = width as usize * 4;
    let src = pixmap.data();
    let dst = cropped.data_mut();
    for row in 0..height as usize {
        let src_start = (src_y as usize + row) * src_stride + src_x as usize * 4;
        let dst_start = row * dst_stride;
        dst[dst_start..dst_start + dst_stride]
            .copy_from_slice(&src[src_start..src_start + dst_stride]);
    }
    Some(cropped)
}

fn enumerate_monitor_bounds() -> Vec<MonitorBounds> {
    let mut monitors = Vec::new();
    unsafe {
        let _ = EnumDisplayMonitors(
            HDC(std::ptr::null_mut()),
            None,
            Some(enum_monitor_proc),
            LPARAM(&mut monitors as *mut Vec<MonitorBounds> as isize),
        );
    }
    if monitors.is_empty() {
        monitors.push(MonitorBounds {
            id: 0,
            bounds: Overlay::primary_bounds(),
            primary: true,
        });
    }
    monitors
}

unsafe extern "system" fn enum_monitor_proc(
    monitor: HMONITOR,
    _hdc: HDC,
    _rect: *mut RECT,
    data: LPARAM,
) -> BOOL {
    let monitors = &mut *(data.0 as *mut Vec<MonitorBounds>);
    let mut info = MONITORINFO {
        cbSize: std::mem::size_of::<MONITORINFO>() as u32,
        ..Default::default()
    };
    if GetMonitorInfoW(monitor, &mut info).as_bool() {
        monitors.push(MonitorBounds {
            id: monitor.0 as isize,
            bounds: rect_from_win32(info.rcMonitor),
            primary: info.dwFlags & MONITORINFOF_PRIMARY != 0,
        });
    }
    BOOL(1)
}

fn monitor_union(monitors: &[MonitorBounds]) -> Rect {
    monitors
        .iter()
        .map(|monitor| monitor.bounds)
        .reduce(Rect::union)
        .unwrap_or_else(Overlay::virtual_bounds)
}

fn primary_monitor_bounds(monitors: &[MonitorBounds]) -> Rect {
    monitors
        .iter()
        .find(|monitor| monitor.primary)
        .or_else(|| monitors.first())
        .map(|monitor| monitor.bounds)
        .unwrap_or_else(Overlay::primary_bounds)
}

#[cfg(test)]
fn retained_monitor_ids(existing: &[isize], desired: &[isize]) -> Vec<isize> {
    desired
        .iter()
        .copied()
        .filter(|id| existing.contains(id))
        .collect()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum OverlayMessageAction {
    MarkTopologyDirty,
    Delegate,
}

fn overlay_message_action(msg: u32) -> OverlayMessageAction {
    match msg {
        WM_DPICHANGED | WM_DISPLAYCHANGE => OverlayMessageAction::MarkTopologyDirty,
        _ => OverlayMessageAction::Delegate,
    }
}

extern "system" fn wndproc(hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    unsafe {
        match overlay_message_action(msg) {
            // A monitor's DPI changed, or the display topology/resolution changed. Under PMv2 we
            // render in physical pixels, so there is no per-window rescale to do — we just flag
            // the monitor set as dirty and let the runtime re-enumerate and rebuild via
            // `Overlay::rebuild_monitors`. Both messages are broadcast to top-level windows.
            OverlayMessageAction::MarkTopologyDirty => {
                MONITORS_DIRTY.store(true, Ordering::SeqCst);
                DefWindowProcW(hwnd, msg, wparam, lparam)
            }
            OverlayMessageAction::Delegate => DefWindowProcW(hwnd, msg, wparam, lparam),
        }
    }
}

extern "system" fn image_wndproc(hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    unsafe {
        match msg {
            WM_CLOSE => {
                // Unlike controller cleanup (which calls DestroyWindow directly), WM_CLOSE is
                // positive evidence that the native/user close path was invoked.
                record_collect_user_close(hwnd);
                DefWindowProcW(hwnd, msg, wparam, lparam)
            }
            WM_DESTROY => LRESULT(0),
            _ => DefWindowProcW(hwnd, msg, wparam, lparam),
        }
    }
}

fn resize_note_edit(hwnd: HWND) {
    unsafe {
        let Ok(edit) = GetDlgItem(hwnd, NOTE_EDIT_ID) else {
            return;
        };
        let mut client = RECT::default();
        if GetClientRect(hwnd, &mut client).is_ok() {
            let _ = MoveWindow(
                edit,
                0,
                0,
                (client.right - client.left).max(1),
                (client.bottom - client.top).max(1),
                true,
            );
        }
    }
}

extern "system" fn note_wndproc(hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    unsafe {
        match msg {
            WM_SIZE => {
                resize_note_edit(hwnd);
                LRESULT(0)
            }
            WM_CLOSE => {
                record_collect_user_close(hwnd);
                DefWindowProcW(hwnd, msg, wparam, lparam)
            }
            WM_DESTROY => LRESULT(0),
            _ => DefWindowProcW(hwnd, msg, wparam, lparam),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use honk_engine::collect_window::{
        fitted_collect_image_size, COLLECT_PROP_MAX_SCREEN_FRACTION,
    };

    #[test]
    fn layered_window_preserves_asymmetric_channels_and_alpha_when_swizzling_to_bgra() {
        let rgba = [17_u8, 83, 149, 211, 7, 61, 203, 0];
        let mut bgra = [0_u8; 8];

        copy_premultiplied_rgba_to_bgra(&rgba, &mut bgra);

        assert_eq!(bgra, [149, 83, 17, 211, 203, 61, 7, 0]);
    }

    #[test]
    fn collect_images_fit_the_monitor_without_cropping_or_upscaling() {
        let display = Rect::new(Vec2::ZERO, Vec2::new(1920.0, 1080.0));

        let landscape = fitted_collect_image_size(2400, 1600, display);
        assert_eq!(landscape, (777, 518));
        assert!(landscape.0 <= 900 && landscape.1 <= 518);
        assert!((landscape.0 as f32 / landscape.1 as f32 - 1.5).abs() < 0.01);

        let portrait = fitted_collect_image_size(1000, 3000, display);
        assert_eq!(portrait, (173, 518));
        assert!((portrait.0 as f32 / portrait.1 as f32 - 1.0 / 3.0).abs() < 0.01);

        assert_eq!(
            fitted_collect_image_size(320, 180, display),
            (320, 180),
            "small images retain their natural readable size"
        );
    }

    #[test]
    fn note_window_is_readable_but_never_dominates_the_monitor() {
        for display in [
            Rect::new(Vec2::ZERO, Vec2::new(3840.0, 2160.0)),
            Rect::new(Vec2::ZERO, Vec2::new(1920.0, 1080.0)),
            Rect::new(Vec2::ZERO, Vec2::new(800.0, 600.0)),
        ] {
            let size = collect_note_size(display);
            assert!(size.x >= 1.0 && size.y >= 1.0);
            assert!(size.x <= display.width() * COLLECT_PROP_MAX_SCREEN_FRACTION);
            assert!(size.y <= display.height() * COLLECT_PROP_MAX_SCREEN_FRACTION);
        }
        assert_eq!(
            collect_note_size(Rect::new(Vec2::ZERO, Vec2::new(1920.0, 1080.0))),
            Vec2::new(614.0, 346.0)
        );
    }

    #[test]
    fn only_native_collect_close_signal_counts_as_user_close() {
        let hwnd = HWND(std::ptr::dangling_mut::<c_void>());
        assert!(!take_collect_user_close(hwnd));
        record_collect_user_close(hwnd);
        record_collect_user_close(hwnd);
        assert!(take_collect_user_close(hwnd));
        assert!(
            !take_collect_user_close(hwnd),
            "native close evidence is one-shot"
        );
    }

    #[test]
    fn collect_controller_prefers_the_active_typed_request_over_a_lingering_window() {
        let lingering = (
            CollectWindowId(1),
            CollectWindowRequestId(100),
            CollectWindowKind::Note,
        );
        let active = (
            CollectWindowId(2),
            CollectWindowRequestId(200),
            CollectWindowKind::Meme,
        );
        let same_request_wrong_kind = (CollectWindowId(3), active.1, CollectWindowKind::Note);

        assert_eq!(
            preferred_collect_window_id(
                Some((active.1, active.2)),
                [lingering, same_request_wrong_kind, active],
            ),
            Some(active.0),
            "an older lingering prop must not starve the currently active typed request"
        );
    }

    #[test]
    fn monitor_reconciliation_preserves_surviving_overlay_identity() {
        let existing_window_monitor_ids = [1_isize, 2, 3];
        let new_monitor_ids = [1_isize, 3];
        assert_eq!(
            retained_monitor_ids(&existing_window_monitor_ids, &new_monitor_ids),
            new_monitor_ids
        );
    }

    #[test]
    fn destroying_one_overlay_is_not_a_process_quit_signal() {
        assert_eq!(
            overlay_message_action(WM_DESTROY),
            OverlayMessageAction::Delegate
        );
    }

    #[test]
    fn note_typing_is_target_scoped_not_global_input() {
        fn requires_targeted_setter(_setter: fn(&NoteWindow, &str) -> Result<()>) {}
        requires_targeted_setter(NoteWindow::set_text);
    }

    #[test]
    fn win32_rect_conversion_preserves_signed_coordinates() {
        let rect = rect_from_win32(RECT {
            left: -900,
            top: -40,
            right: -300,
            bottom: 360,
        });

        assert_eq!(rect.min, Vec2::new(-900.0, -40.0));
        assert_eq!(rect.max, Vec2::new(-300.0, 360.0));
    }

    #[test]
    fn null_or_own_window_is_not_foreign_top_level() {
        let null = HWND(std::ptr::null_mut());
        assert!(!is_foreign_top_level_window(null, &[]));

        let fake = HWND(std::ptr::dangling_mut::<c_void>());
        assert!(!is_foreign_top_level_window(fake, &[hwnd_key(fake)]));
    }

    #[test]
    fn monitor_bounds_union_and_primary_selection_support_negative_coords() {
        let monitors = [
            MonitorBounds {
                id: 1,
                bounds: Rect::new(Vec2::new(-1280.0, 0.0), Vec2::new(0.0, 720.0)),
                primary: false,
            },
            MonitorBounds {
                id: 2,
                bounds: Rect::new(Vec2::new(0.0, -80.0), Vec2::new(1920.0, 1000.0)),
                primary: true,
            },
        ];

        assert_eq!(
            monitor_union(&monitors),
            Rect::new(Vec2::new(-1280.0, -80.0), Vec2::new(1920.0, 1000.0))
        );
        assert_eq!(primary_monitor_bounds(&monitors), monitors[1].bounds);
    }

    #[test]
    fn terminal_windows_are_protected_by_class_or_title() {
        for (class_name, title) in [
            ("ConsoleWindowClass", "Command Prompt"),
            ("CASCADIA_HOSTING_WINDOW_CLASS", "Windows Terminal"),
            ("mintty", "MINGW64:/c/Users/hey/git/goose"),
            ("org.wezfurlong.wezterm", "pwsh"),
            ("GLFW30", "Alacritty"),
            ("Chrome_WidgetWin_1", "Ubuntu - WSL"),
            ("ApplicationFrameWindow", "PowerShell 7"),
            ("Chrome_WidgetWin_1", "goose - Visual Studio Code"),
            ("Chrome_WidgetWin_1", "Codex"),
            ("Chrome_WidgetWin_1", "ChatGPT"),
            ("Notepad", "notes.txt - Notepad"),
        ] {
            let protected = classify_protected_window(class_name, title);
            if title.contains("Notepad") {
                assert_eq!(protected, None, "{class_name} / {title}");
            } else {
                assert_eq!(
                    protected,
                    Some(ProtectedWindowClass::Terminal),
                    "{class_name} / {title}"
                );
            }
        }
    }

    #[test]
    fn notification_state_mapping_feeds_engine_presence() {
        assert_eq!(
            map_notification_state(QUNS_ACCEPTS_NOTIFICATIONS),
            PresenceSnapshot::available()
        );
        assert_eq!(
            map_notification_state(QUNS_BUSY),
            PresenceSnapshot::fullscreen()
        );
        assert_eq!(
            map_notification_state(QUNS_RUNNING_D3D_FULL_SCREEN),
            PresenceSnapshot::fullscreen()
        );
        for state in [
            QUNS_PRESENTATION_MODE,
            QUNS_NOT_PRESENT,
            QUNS_QUIET_TIME,
            QUNS_APP,
        ] {
            assert_eq!(
                map_notification_state(state),
                PresenceSnapshot::do_not_disturb(),
                "{state:?}"
            );
        }
        assert_eq!(
            map_notification_state(QUERY_USER_NOTIFICATION_STATE(-1)),
            PresenceSnapshot::unsupported()
        );
    }
}
