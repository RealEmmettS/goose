//! Windows overlay backend for honk300.
//!
//! A single **layered popup window**, sized to the goose's bounding box and repositioned
//! every frame by [`Overlay::present`] (via `UpdateLayeredWindow`'s destination point).
//! Because `UpdateLayeredWindow` replaces the entire layered surface, a small moving
//! window IS the dirty-rect optimisation — present cost stays proportional to the goose,
//! not the screen (mitigates the fullscreen-redraw CPU risk, plan §15 E1).
//!
//! Click-through is natural per-pixel alpha: we set `WS_EX_LAYERED` but **not**
//! `WS_EX_TRANSPARENT`, so opaque goose pixels receive clicks while transparent margins
//! fall through (plan §6). tiny-skia produces premultiplied RGBA; we feed
//! `UpdateLayeredWindow` premultiplied BGRA with `AC_SRC_ALPHA`.

#![cfg(windows)]

use honk_engine::math::Rect;
use honk_engine::Vec2;
use std::ffi::c_void;
use tiny_skia::Pixmap;
use windows::core::{w, Result};
use windows::Win32::Foundation::{COLORREF, HINSTANCE, HWND, LPARAM, LRESULT, POINT, SIZE, WPARAM};
use windows::Win32::Graphics::Gdi::{
    CreateCompatibleDC, CreateDIBSection, DeleteDC, DeleteObject, GetDC, ReleaseDC, SelectObject,
    AC_SRC_ALPHA, AC_SRC_OVER, BITMAPINFO, BITMAPINFOHEADER, BI_RGB, BLENDFUNCTION, DIB_RGB_COLORS,
    HBITMAP, HDC, HGDIOBJ,
};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DefWindowProcW, DispatchMessageW, GetSystemMetrics, PeekMessageW,
    PostQuitMessage, RegisterClassExW, ShowWindow, TranslateMessage, UpdateLayeredWindow, MSG,
    PM_REMOVE, SM_CXSCREEN, SM_CXVIRTUALSCREEN, SM_CYSCREEN, SM_CYVIRTUALSCREEN, SM_XVIRTUALSCREEN,
    SM_YVIRTUALSCREEN, SW_SHOWNOACTIVATE, ULW_ALPHA, WM_DESTROY, WM_QUIT, WNDCLASSEXW,
    WS_EX_LAYERED, WS_EX_NOACTIVATE, WS_EX_TOOLWINDOW, WS_EX_TOPMOST, WS_POPUP,
};

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
