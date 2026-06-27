//! Windows overlay backend for honk300.
//!
//! A single **fullscreen primary-monitor layered popup window** presented via
//! [`Overlay::present`] and `UpdateLayeredWindow`. The fullscreen surface is the current
//! M3+ shape so world-space footmarks/hearts render where they belong; per-monitor windows
//! and tighter dirty-rect presentation remain planned performance work.
//!
//! Click-through is natural per-pixel alpha: we set `WS_EX_LAYERED` but **not**
//! `WS_EX_TRANSPARENT`, so opaque goose pixels receive clicks while transparent margins
//! fall through (plan §6). tiny-skia produces premultiplied RGBA; we feed
//! `UpdateLayeredWindow` premultiplied BGRA with `AC_SRC_ALPHA`. M7 also exposes a thin
//! `SetCursorPos` wrapper for the engine's platform-free cursor commands. M8 adds a
//! foreign-window move/size watcher that feeds platform-free perch-and-ride snapshots to
//! the engine without exposing HWNDs.

#![cfg(windows)]

use honk_engine::math::Rect;
use honk_engine::Vec2;
use honk_engine::{ForeignWindowId, ForeignWindowSnapshot};
use std::collections::VecDeque;
use std::ffi::c_void;
use std::sync::{Mutex, OnceLock};
use tiny_skia::Pixmap;
use windows::core::{w, Error, Result};
use windows::Win32::Foundation::{
    COLORREF, HINSTANCE, HWND, LPARAM, LRESULT, POINT, RECT, SIZE, WPARAM,
};
use windows::Win32::Graphics::Gdi::{
    CreateCompatibleDC, CreateDIBSection, DeleteDC, DeleteObject, GetDC, ReleaseDC, SelectObject,
    AC_SRC_ALPHA, AC_SRC_OVER, BITMAPINFO, BITMAPINFOHEADER, BI_RGB, BLENDFUNCTION, DIB_RGB_COLORS,
    HBITMAP, HDC, HGDIOBJ,
};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::Accessibility::{SetWinEventHook, UnhookWinEvent, HWINEVENTHOOK};
use windows::Win32::UI::Input::KeyboardAndMouse::{GetAsyncKeyState, VK_LBUTTON};
use windows::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DefWindowProcW, DispatchMessageW, GetAncestor, GetCursorPos, GetSystemMetrics,
    GetWindowRect, IsIconic, IsWindow, IsWindowVisible, PeekMessageW, PostQuitMessage,
    RegisterClassExW, SetCursorPos, ShowWindow, TranslateMessage, UpdateLayeredWindow,
    EVENT_SYSTEM_MOVESIZEEND, EVENT_SYSTEM_MOVESIZESTART, GA_ROOT, MSG, OBJID_WINDOW, PM_REMOVE,
    SM_CXSCREEN, SM_CXVIRTUALSCREEN, SM_CYSCREEN, SM_CYVIRTUALSCREEN, SM_XVIRTUALSCREEN,
    SM_YVIRTUALSCREEN, SW_SHOWNOACTIVATE, ULW_ALPHA, WINEVENT_OUTOFCONTEXT,
    WINEVENT_SKIPOWNPROCESS, WM_DESTROY, WM_QUIT, WNDCLASSEXW, WS_EX_LAYERED, WS_EX_NOACTIVATE,
    WS_EX_TOOLWINDOW, WS_EX_TOPMOST, WS_POPUP,
};

/// Poll the global cursor position (desktop coordinates) and the left-button state.
/// Returns `(x, y, left_down)`. Desktop coordinates equal world coordinates because the
/// overlay's origin is the primary monitor's top-left corner. Used to feed the engine's
/// hit-testing (pat hover-streak + click→hyper, plan §6) each frame.
pub fn pointer_state() -> (f32, f32, bool) {
    unsafe {
        let mut pt = POINT::default();
        let _ = GetCursorPos(&mut pt);
        // High bit of GetAsyncKeyState ⇒ the key is currently down.
        let left_down = (GetAsyncKeyState(VK_LBUTTON.0 as i32) as u16 & 0x8000) != 0;
        (pt.x as f32, pt.y as f32, left_down)
    }
}

/// Warp the global cursor to a desktop/world-space coordinate.
pub fn warp_cursor(pos: Vec2) -> Result<()> {
    unsafe { SetCursorPos(pos.x.round() as i32, pos.y.round() as i32) }
}

#[derive(Debug, Clone, Copy)]
struct RawMoveEvent {
    hwnd: isize,
    started: bool,
}

static MOVE_EVENTS: OnceLock<Mutex<VecDeque<RawMoveEvent>>> = OnceLock::new();

fn move_events() -> &'static Mutex<VecDeque<RawMoveEvent>> {
    MOVE_EVENTS.get_or_init(|| Mutex::new(VecDeque::new()))
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
    overlay_hwnd: HWND,
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
            overlay_hwnd: overlay.hwnd,
            active: None,
        })
    }

    /// Drain queued move/size events and return the current active drag snapshot, if any.
    pub fn active_drag(&mut self) -> Result<Option<ForeignWindowSnapshot>> {
        self.drain_events();
        let Some(hwnd) = self.active.map(hwnd_from_key) else {
            return Ok(None);
        };
        if !is_foreign_top_level_window(hwnd, self.overlay_hwnd) {
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
                    if is_foreign_top_level_window(hwnd, self.overlay_hwnd) {
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

fn is_foreign_top_level_window(hwnd: HWND, overlay_hwnd: HWND) -> bool {
    unsafe {
        if hwnd.0.is_null() || hwnd_key(hwnd) == hwnd_key(overlay_hwnd) {
            return false;
        }
        if !IsWindow(hwnd).as_bool() || !IsWindowVisible(hwnd).as_bool() || IsIconic(hwnd).as_bool()
        {
            return false;
        }
        let root = GetAncestor(hwnd, GA_ROOT);
        !root.0.is_null() && hwnd_key(root) == hwnd_key(hwnd)
    }
}

fn window_rect(hwnd: HWND) -> Result<Rect> {
    unsafe {
        let mut rect = RECT::default();
        GetWindowRect(hwnd, &mut rect)?;
        Ok(rect_from_win32(rect))
    }
}

fn rect_from_win32(rect: RECT) -> Rect {
    Rect {
        min: Vec2::new(rect.left as f32, rect.top as f32),
        max: Vec2::new(rect.right as f32, rect.bottom as f32),
    }
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

/// The honk300 desktop overlay: one always-on-top, click-through-where-transparent
/// layered window that the goose lives in.
pub struct Overlay {
    hwnd: HWND,
    dib: Option<Dib>,
}

impl Overlay {
    /// Register the window class and create the (initially hidden) layered window.
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

            let hwnd = CreateWindowExW(
                WS_EX_LAYERED | WS_EX_TOOLWINDOW | WS_EX_TOPMOST | WS_EX_NOACTIVATE,
                class_name,
                w!("honk300"),
                WS_POPUP,
                0,
                0,
                0,
                0,
                None,
                None,
                hinstance,
                None,
            )?;

            let _ = ShowWindow(hwnd, SW_SHOWNOACTIVATE);
            Ok(Overlay { hwnd, dib: None })
        }
    }

    /// The full virtual-desktop bounds (across all monitors). Multi-monitor traversal is
    /// M15; M3's fullscreen overlay covers the primary monitor (see [`Overlay::primary_bounds`]).
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

    /// The primary monitor's bounds (origin `(0, 0)`). The fullscreen overlay covers this
    /// so world-space props (footmarks, later meme/notepad windows) render in place.
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

    /// Drain pending window messages. Returns `false` when the window is closing
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

    /// Present `pixmap` (premultiplied RGBA from the renderer) at desktop position
    /// `(dest_x, dest_y)`. Resizes the backing window/DIB to the pixmap as needed.
    pub fn present(&mut self, pixmap: &Pixmap, dest_x: i32, dest_y: i32) -> Result<()> {
        let width = pixmap.width() as i32;
        let height = pixmap.height() as i32;
        if width == 0 || height == 0 {
            return Ok(());
        }

        unsafe {
            // (Re)allocate the DIB when the size changes.
            if self
                .dib
                .as_ref()
                .map(|d| d.width != width || d.height != height)
                .unwrap_or(true)
            {
                self.dib = Some(Dib::new(width, height)?);
            }
            let dib = self.dib.as_ref().expect("dib just set");

            // Copy premultiplied RGBA → premultiplied BGRA (swap R and B).
            let src = pixmap.data();
            let count = (width * height) as usize;
            let dst = std::slice::from_raw_parts_mut(dib.bits, count * 4);
            for i in 0..count {
                let s = i * 4;
                dst[s] = src[s + 2]; // B
                dst[s + 1] = src[s + 1]; // G
                dst[s + 2] = src[s]; // R
                dst[s + 3] = src[s + 3]; // A
            }

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
                self.hwnd,
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
}

extern "system" fn wndproc(hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    unsafe {
        match msg {
            WM_DESTROY => {
                PostQuitMessage(0);
                LRESULT(0)
            }
            _ => DefWindowProcW(hwnd, msg, wparam, lparam),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
        assert!(!is_foreign_top_level_window(null, null));

        let fake = HWND(std::ptr::dangling_mut::<c_void>());
        assert!(!is_foreign_top_level_window(fake, fake));
    }
}
