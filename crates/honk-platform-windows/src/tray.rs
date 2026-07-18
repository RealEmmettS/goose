use honk_control::ControlSurfaceCommand;
use std::collections::VecDeque;
use std::ffi::{c_void, OsStr};
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::{Mutex, OnceLock};
use tiny_skia::Pixmap;
use windows::core::{w, Error, PCWSTR};
use windows::Win32::Foundation::{BOOL, HINSTANCE, HWND, LPARAM, LRESULT, POINT, WPARAM};
use windows::Win32::Graphics::Gdi::{
    CreateBitmap, CreateDIBSection, DeleteObject, GetDC, ReleaseDC, BITMAPINFO, BITMAPINFOHEADER,
    BI_RGB, DIB_RGB_COLORS, HBITMAP, HGDIOBJ,
};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::Shell::{
    Shell_NotifyIconW, NIF_GUID, NIF_ICON, NIF_MESSAGE, NIF_SHOWTIP, NIF_TIP, NIM_ADD, NIM_DELETE,
    NIM_SETFOCUS, NIM_SETVERSION, NIN_SELECT, NOTIFYICONDATAW, NOTIFYICON_VERSION_4,
};
use windows::Win32::UI::WindowsAndMessaging::{
    AppendMenuW, CreateIconIndirect, CreatePopupMenu, CreateWindowExW, DefWindowProcW, DestroyIcon,
    DestroyMenu, DestroyWindow, EndMenu, GetCursorPos, PostMessageW, RegisterClassExW,
    RegisterWindowMessageW, SetForegroundWindow, TrackPopupMenu, ICONINFO, MF_SEPARATOR, MF_STRING,
    TPM_RETURNCMD, TPM_RIGHTBUTTON, WM_CONTEXTMENU, WM_DESTROY, WM_NULL, WM_USER, WNDCLASSEXW,
    WS_EX_NOACTIVATE, WS_EX_TOOLWINDOW, WS_POPUP,
};

const TRAY_CALLBACK_MESSAGE: u32 = WM_USER + 0x300;
const NIN_KEYSELECT: u32 = NIN_SELECT + 1;
const CONFIGURE_COMMAND_ID: usize = 1;
const QUIT_COMMAND_ID: usize = 2;
const TRAY_ICON_ID: u32 = 1;
const TRAY_ICON_GUID: windows::core::GUID =
    windows::core::GUID::from_u128(0x1282_821f_82b6_42e2_945b_ef2f_e8d9_fbda);
const STATUS_ICON_PNG: &[u8] = include_bytes!("../../../Assets/UI/honk300-status-goose@2x.png");

static TASKBAR_CREATED_MESSAGE: AtomicU32 = AtomicU32::new(0);
static SMOKE_TRAY_QUIT_MESSAGE: AtomicU32 = AtomicU32::new(0);
static TASKBAR_READD_REQUESTED: AtomicBool = AtomicBool::new(false);
static COMMANDS: OnceLock<Mutex<VecDeque<ControlSurfaceCommand>>> = OnceLock::new();

fn command_queue() -> &'static Mutex<VecDeque<ControlSurfaceCommand>> {
    COMMANDS.get_or_init(|| Mutex::new(VecDeque::new()))
}

fn with_commands<R>(f: impl FnOnce(&mut VecDeque<ControlSurfaceCommand>) -> R) -> R {
    let mut commands = command_queue()
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    f(&mut commands)
}

/// Runtime-owned Windows notification-area surface.
///
/// The hidden owner window shares the existing runtime thread's Win32 message pump. Explorer
/// recreation only requests a re-add; the retained Rust owner performs the shell calls on the
/// next runtime iteration and keeps the icon handle alive until the final walk-off completes.
pub struct StatusTray {
    hwnd: HWND,
    icon: windows::Win32::UI::WindowsAndMessaging::HICON,
    added: bool,
}

impl StatusTray {
    pub fn new() -> windows::core::Result<Self> {
        unsafe {
            with_commands(VecDeque::clear);
            TASKBAR_READD_REQUESTED.store(false, Ordering::Release);

            let taskbar_created = RegisterWindowMessageW(w!("TaskbarCreated"));
            if taskbar_created == 0 {
                return Err(Error::from_win32());
            }
            TASKBAR_CREATED_MESSAGE.store(taskbar_created, Ordering::Release);
            SMOKE_TRAY_QUIT_MESSAGE.store(register_smoke_tray_quit_message()?, Ordering::Release);

            let module = GetModuleHandleW(None)?;
            let instance = HINSTANCE(module.0);
            let class_name = w!("honk300_status_tray_owner");
            let class = WNDCLASSEXW {
                cbSize: std::mem::size_of::<WNDCLASSEXW>() as u32,
                lpfnWndProc: Some(tray_wndproc),
                hInstance: instance,
                lpszClassName: class_name,
                ..Default::default()
            };
            let _ = RegisterClassExW(&class);

            let hwnd = CreateWindowExW(
                WS_EX_TOOLWINDOW | WS_EX_NOACTIVATE,
                class_name,
                w!("Honk300 controls"),
                WS_POPUP,
                0,
                0,
                0,
                0,
                None,
                None,
                instance,
                None,
            )?;
            let icon = match create_status_icon() {
                Ok(icon) => icon,
                Err(error) => {
                    let _ = DestroyWindow(hwnd);
                    return Err(error);
                }
            };
            let mut tray = Self {
                hwnd,
                icon,
                added: false,
            };
            if let Err(error) = tray.add_to_shell() {
                let _ = DestroyIcon(icon);
                let _ = DestroyWindow(hwnd);
                return Err(error);
            }
            Ok(tray)
        }
    }

    /// Re-adds the retained icon after Explorer/taskbar recreation.
    pub fn maintain(&mut self) -> windows::core::Result<()> {
        if TASKBAR_READD_REQUESTED.swap(false, Ordering::AcqRel) {
            self.added = false;
            self.add_to_shell()?;
            eprintln!("honk300: Windows taskbar recreated; restored Honk300 controls.");
        }
        Ok(())
    }

    pub fn take_command(&self) -> Option<ControlSurfaceCommand> {
        with_commands(VecDeque::pop_front)
    }

    fn add_to_shell(&mut self) -> windows::core::Result<()> {
        unsafe {
            let mut data = self.notify_data();
            if !Shell_NotifyIconW(NIM_ADD, &data).as_bool() {
                return Err(Error::from_win32());
            }
            data.Anonymous.uVersion = NOTIFYICON_VERSION_4;
            if !Shell_NotifyIconW(NIM_SETVERSION, &data).as_bool() {
                let _ = Shell_NotifyIconW(NIM_DELETE, &data);
                return Err(Error::from_win32());
            }
            self.added = true;
            Ok(())
        }
    }

    fn notify_data(&self) -> NOTIFYICONDATAW {
        let mut data = NOTIFYICONDATAW {
            cbSize: std::mem::size_of::<NOTIFYICONDATAW>() as u32,
            hWnd: self.hwnd,
            uID: TRAY_ICON_ID,
            uFlags: NIF_MESSAGE | NIF_ICON | NIF_TIP | NIF_SHOWTIP | NIF_GUID,
            uCallbackMessage: TRAY_CALLBACK_MESSAGE,
            hIcon: self.icon,
            guidItem: TRAY_ICON_GUID,
            ..Default::default()
        };
        write_utf16(&mut data.szTip, "Honk300 controls");
        data
    }
}

impl Drop for StatusTray {
    fn drop(&mut self) {
        unsafe {
            SMOKE_TRAY_QUIT_MESSAGE.store(0, Ordering::Release);
            if self.added {
                let _ = Shell_NotifyIconW(NIM_DELETE, &self.notify_data());
            }
            let _ = DestroyWindow(self.hwnd);
            let _ = DestroyIcon(self.icon);
        }
    }
}

fn write_utf16<const N: usize>(target: &mut [u16; N], value: &str) {
    for (slot, value) in target
        .iter_mut()
        .zip(value.encode_utf16().chain(std::iter::once(0)))
    {
        *slot = value;
    }
}

unsafe extern "system" fn tray_wndproc(
    hwnd: HWND,
    message: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    if message == TASKBAR_CREATED_MESSAGE.load(Ordering::Acquire) {
        TASKBAR_READD_REQUESTED.store(true, Ordering::Release);
        return LRESULT(0);
    }
    let smoke_tray_quit = SMOKE_TRAY_QUIT_MESSAGE.load(Ordering::Acquire);
    if smoke_tray_quit != 0 && message == smoke_tray_quit {
        // The disposable Windows qualification runner opens the real native menu first. Ending
        // that process-owned menu on its UI thread and enqueueing the same finite command avoids
        // global keyboard/mouse input and proves the exact graceful-Quit route without touching
        // whichever foreign application happens to be focused.
        let _ = EndMenu();
        enqueue_menu_selection(QUIT_COMMAND_ID);
        return LRESULT(0);
    }
    if message == TRAY_CALLBACK_MESSAGE {
        let event = (lparam.0 as u32) & 0xffff;
        if matches!(event, WM_CONTEXTMENU | NIN_SELECT | NIN_KEYSELECT) {
            show_menu(hwnd, point_from_callback(wparam));
        }
        return LRESULT(0);
    }
    if message == WM_DESTROY {
        return LRESULT(0);
    }
    DefWindowProcW(hwnd, message, wparam, lparam)
}

unsafe fn show_menu(hwnd: HWND, mut point: POINT) {
    if point.x == -1 && point.y == -1 && GetCursorPos(&mut point).is_err() {
        return;
    }
    let Ok(menu) = CreatePopupMenu() else {
        return;
    };
    let result = (|| -> windows::core::Result<()> {
        AppendMenuW(
            menu,
            MF_STRING,
            CONFIGURE_COMMAND_ID,
            PCWSTR(wide("Configure Honk300…").as_ptr()),
        )?;
        AppendMenuW(menu, MF_SEPARATOR, 0, PCWSTR::null())?;
        AppendMenuW(
            menu,
            MF_STRING,
            QUIT_COMMAND_ID,
            PCWSTR(wide("Quit Honk300").as_ptr()),
        )?;
        let _ = SetForegroundWindow(hwnd);
        let selected = TrackPopupMenu(
            menu,
            TPM_RETURNCMD | TPM_RIGHTBUTTON,
            point.x,
            point.y,
            0,
            hwnd,
            None,
        );
        enqueue_menu_selection(selected.0 as usize);
        let mut data = NOTIFYICONDATAW {
            cbSize: std::mem::size_of::<NOTIFYICONDATAW>() as u32,
            hWnd: hwnd,
            uID: TRAY_ICON_ID,
            uFlags: NIF_GUID,
            guidItem: TRAY_ICON_GUID,
            ..Default::default()
        };
        data.Anonymous.uVersion = NOTIFYICON_VERSION_4;
        let _ = Shell_NotifyIconW(NIM_SETFOCUS, &data);
        let _ = PostMessageW(hwnd, WM_NULL, WPARAM(0), LPARAM(0));
        Ok(())
    })();
    let _ = DestroyMenu(menu);
    if let Err(error) = result {
        eprintln!("honk300: Windows tray menu could not open ({error})");
    }
}

fn command_for_menu_selection(selection: usize) -> Option<ControlSurfaceCommand> {
    match selection {
        CONFIGURE_COMMAND_ID => Some(ControlSurfaceCommand::Configure),
        QUIT_COMMAND_ID => Some(ControlSurfaceCommand::Quit),
        _ => None,
    }
}

fn enqueue_menu_selection(selection: usize) {
    if let Some(command) = command_for_menu_selection(selection) {
        with_commands(|commands| commands.push_back(command));
    }
}

fn register_smoke_tray_quit_message() -> windows::core::Result<u32> {
    let token = std::env::var_os("HONK300_WINDOWS_SMOKE_TRAY_QUIT_TOKEN");
    let Some(message_name) = smoke_tray_quit_message_name(token.as_deref())? else {
        return Ok(0);
    };
    let message_name = wide(&message_name);
    let message = unsafe { RegisterWindowMessageW(PCWSTR(message_name.as_ptr())) };
    if message == 0 {
        Err(Error::from_win32())
    } else {
        Ok(message)
    }
}

fn smoke_tray_quit_message_name(token: Option<&OsStr>) -> windows::core::Result<Option<String>> {
    let Some(token) = token else {
        return Ok(None);
    };
    if token.is_empty() {
        return Ok(None);
    }
    let token = token
        .to_str()
        .filter(|token| {
            (32..=64).contains(&token.len())
                && token
                    .bytes()
                    .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-')
        })
        .ok_or_else(|| failure("invalid Windows tray smoke token"))?;
    Ok(Some(format!("Honk300SmokeTrayQuit-{token}")))
}

fn point_from_callback(value: WPARAM) -> POINT {
    let packed = value.0 as u32;
    POINT {
        x: (packed as u16 as i16) as i32,
        y: ((packed >> 16) as u16 as i16) as i32,
    }
}

fn wide(value: &str) -> Vec<u16> {
    value.encode_utf16().chain(std::iter::once(0)).collect()
}

fn create_status_icon() -> windows::core::Result<windows::Win32::UI::WindowsAndMessaging::HICON> {
    let source = Pixmap::decode_png(STATUS_ICON_PNG)
        .map_err(|error| failure(format!("invalid embedded tray PNG: {error}")))?;
    let width = source.width() as i32;
    let height = source.height() as i32;
    let bgra = compose_tray_bgra(&source);

    unsafe {
        let screen = GetDC(None);
        if screen.0.is_null() {
            return Err(Error::from_win32());
        }
        let mut bits: *mut c_void = std::ptr::null_mut();
        let info = BITMAPINFO {
            bmiHeader: BITMAPINFOHEADER {
                biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
                biWidth: width,
                biHeight: -height,
                biPlanes: 1,
                biBitCount: 32,
                biCompression: BI_RGB.0,
                ..Default::default()
            },
            ..Default::default()
        };
        let color = CreateDIBSection(screen, &info, DIB_RGB_COLORS, &mut bits, None, 0);
        let _ = ReleaseDC(None, screen);
        let color = color?;
        if bits.is_null() {
            let _ = DeleteObject(HGDIOBJ(color.0));
            return Err(failure("tray DIB returned no writable pixels"));
        }
        std::ptr::copy_nonoverlapping(bgra.as_ptr(), bits.cast::<u8>(), bgra.len());

        let mask = CreateBitmap(width, height, 1, 1, None);
        if mask.0.is_null() {
            let _ = DeleteObject(HGDIOBJ(color.0));
            return Err(Error::from_win32());
        }
        let icon_info = ICONINFO {
            fIcon: BOOL(1),
            hbmMask: HBITMAP(mask.0),
            hbmColor: HBITMAP(color.0),
            ..Default::default()
        };
        let icon = CreateIconIndirect(&icon_info);
        let _ = DeleteObject(HGDIOBJ(mask.0));
        let _ = DeleteObject(HGDIOBJ(color.0));
        icon
    }
}

fn compose_tray_bgra(source: &Pixmap) -> Vec<u8> {
    let width = source.width() as f32;
    let height = source.height() as f32;
    let center_x = (width - 1.0) / 2.0;
    let center_y = (height - 1.0) / 2.0;
    let radius = width.min(height) * 0.47;
    let mut output = vec![0; source.data().len()];

    for (index, (input, output)) in source
        .data()
        .chunks_exact(4)
        .zip(output.chunks_exact_mut(4))
        .enumerate()
    {
        let x = (index % source.width() as usize) as f32;
        let y = (index / source.width() as usize) as f32;
        let in_background = (x - center_x).hypot(y - center_y) <= radius;
        let mask = input[3] as u16;
        if in_background {
            let inverse = 255 - mask;
            output[0] = ((255 * mask + 110 * inverse) / 255) as u8;
            output[1] = ((255 * mask + 75 * inverse) / 255) as u8;
            output[2] = ((255 * mask + 24 * inverse) / 255) as u8;
            output[3] = 255;
        } else if mask > 0 {
            output[0] = mask as u8;
            output[1] = mask as u8;
            output[2] = mask as u8;
            output[3] = mask as u8;
        }
    }
    output
}

fn failure(message: impl AsRef<str>) -> Error {
    Error::new(windows::core::HRESULT(0x8000_4005u32 as i32), message)
}

#[cfg(test)]
mod tests {
    use super::{
        command_for_menu_selection, compose_tray_bgra, point_from_callback,
        smoke_tray_quit_message_name, write_utf16, CONFIGURE_COMMAND_ID, QUIT_COMMAND_ID,
        STATUS_ICON_PNG,
    };
    use honk_control::ControlSurfaceCommand;
    use std::ffi::OsStr;
    use tiny_skia::Pixmap;
    use windows::Win32::Foundation::WPARAM;

    #[test]
    fn embedded_goose_icon_is_valid_contrasting_argb_source() {
        let source = Pixmap::decode_png(STATUS_ICON_PNG).expect("valid canonical runtime PNG");
        assert_eq!((source.width(), source.height()), (36, 36));
        let output = compose_tray_bgra(&source);
        assert_eq!(output.len(), 36 * 36 * 4);
        assert!(output.chunks_exact(4).any(|pixel| pixel[3] == 255));
        assert!(output
            .chunks_exact(4)
            .any(|pixel| pixel[3] == 255 && pixel[0] > pixel[2]));
        assert!(output
            .chunks_exact(4)
            .any(|pixel| pixel[3] == 255 && pixel[0] == 255 && pixel[1] == 255));
    }

    #[test]
    fn v4_callback_coordinates_preserve_signed_virtual_desktop_points() {
        let packed = ((1200u16 as usize) << 16) | ((-320i16 as u16) as usize);
        assert_eq!(point_from_callback(WPARAM(packed)).x, -320);
        assert_eq!(point_from_callback(WPARAM(packed)).y, 1200);
    }

    #[test]
    fn accessible_tooltip_is_nul_terminated() {
        let mut target = [0u16; 128];
        write_utf16(&mut target, "Honk300 controls");
        let length = "Honk300 controls".encode_utf16().count();
        assert_eq!(target[length], 0);
        assert_eq!(
            String::from_utf16(&target[..length]).unwrap(),
            "Honk300 controls"
        );
    }

    #[test]
    fn native_menu_and_ci_hook_share_the_finite_command_mapping() {
        assert_eq!(
            command_for_menu_selection(CONFIGURE_COMMAND_ID),
            Some(ControlSurfaceCommand::Configure)
        );
        assert_eq!(
            command_for_menu_selection(QUIT_COMMAND_ID),
            Some(ControlSurfaceCommand::Quit)
        );
        assert_eq!(command_for_menu_selection(0), None);
        assert_eq!(command_for_menu_selection(usize::MAX), None);
    }

    #[test]
    fn smoke_tray_quit_hook_ignores_absent_or_empty_environment_but_rejects_bad_tokens() {
        assert_eq!(smoke_tray_quit_message_name(None).unwrap(), None);
        assert_eq!(
            smoke_tray_quit_message_name(Some(OsStr::new(""))).unwrap(),
            None
        );
        assert_eq!(
            smoke_tray_quit_message_name(Some(OsStr::new("0123456789abcdef0123456789abcdef")))
                .unwrap()
                .as_deref(),
            Some("Honk300SmokeTrayQuit-0123456789abcdef0123456789abcdef")
        );
        assert!(smoke_tray_quit_message_name(Some(OsStr::new("too-short"))).is_err());
        assert!(
            smoke_tray_quit_message_name(Some(OsStr::new("0123456789abcdef0123456789abcde_")))
                .is_err()
        );
    }
}
