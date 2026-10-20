use super::*;
use std::{
    fs,
    io::{Read, Seek, SeekFrom, Write},
    os::windows::{io::AsRawHandle, process::CommandExt},
    path::PathBuf,
    process::{Command, Stdio},
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc,
    },
    time::Instant,
};
use windows::{
    core::w,
    Win32::{
        Foundation::{HANDLE, HINSTANCE, LRESULT, WAIT_TIMEOUT},
        System::{LibraryLoader::GetModuleHandleW, Threading::WaitForInputIdle},
        UI::WindowsAndMessaging::{
            CreateWindowExW, DefWindowProcW, DestroyWindow, DispatchMessageW, PeekMessageW,
            PostMessageW, RegisterClassExW, MSG, PM_REMOVE, WM_APP, WNDCLASSEXW, WS_EX_NOACTIVATE,
            WS_EX_TOOLWINDOW, WS_POPUP, WS_VISIBLE,
        },
    },
};

const FIXTURE_MODE: &str = "HONK300_SETTINGS_READY_TEST_MODE";
const FIXTURE_READY: &str = "HONK300_SETTINGS_READY_TEST_PATH";
const FIXTURE_TEST: &str = "settings::launch::windows_ready::tests::native_child_fixture";

struct FixtureProgram {
    executable: PathBuf,
    directory: tempfile::TempDir,
}

impl FixtureProgram {
    fn new() -> Arc<Self> {
        let directory = tempfile::Builder::new()
            .prefix("honk300-settings-ready-")
            .tempdir()
            .unwrap();
        let executable = directory.path().join("settings-ready-fixture.exe");
        fs::copy(std::env::current_exe().unwrap(), &executable).unwrap();
        // The real settings companion is a GUI-subsystem process. Make only our
        // private test-binary copy match that property, so the old input-idle
        // regression cannot pass because of console-subsystem special handling.
        let mut image = fs::OpenOptions::new()
            .read(true)
            .write(true)
            .open(&executable)
            .unwrap();
        image.seek(SeekFrom::Start(0x3c)).unwrap();
        let mut offset = [0; 4];
        image.read_exact(&mut offset).unwrap();
        let pe = u32::from_le_bytes(offset) as u64;
        image.seek(SeekFrom::Start(pe)).unwrap();
        let mut signature = [0; 4];
        image.read_exact(&mut signature).unwrap();
        assert_eq!(&signature, b"PE\0\0");
        // Subsystem is at offset 68 in both PE32 and PE32+ optional headers.
        image.seek(SeekFrom::Start(pe + 24 + 68)).unwrap();
        image.write_all(&2_u16.to_le_bytes()).unwrap();
        drop(image);
        Arc::new(Self {
            executable,
            directory,
        })
    }

    fn spawn(self: &Arc<Self>, mode: &str) -> Fixture {
        static NEXT: AtomicU64 = AtomicU64::new(0);
        let ready = self.directory.path().join(format!(
            "{mode}-{}.ready",
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        let child = Command::new(&self.executable)
            .args(["--exact", FIXTURE_TEST, "--ignored", "--nocapture"])
            .env(FIXTURE_MODE, mode)
            .env(FIXTURE_READY, &ready)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .creation_flags(0x0800_0000)
            .spawn()
            .unwrap();
        Fixture {
            child,
            ready,
            _program: self.clone(),
        }
    }
}

struct Fixture {
    child: Child,
    ready: PathBuf,
    _program: Arc<FixtureProgram>,
}

impl Fixture {
    fn await_fixture(&mut self) {
        let deadline = Instant::now() + Duration::from_secs(5);
        while !self.ready.exists() {
            assert!(
                self.child.try_wait().unwrap().is_none(),
                "native fixture exited before creating its window"
            );
            assert!(Instant::now() < deadline, "native fixture did not start");
            std::thread::sleep(Duration::from_millis(10));
        }
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

#[test]
fn busy_window_is_ready_without_an_idle_queue() {
    let program = FixtureProgram::new();
    let mut fixture = program.spawn("busy");
    fixture.await_fixture();
    // SAFETY: The fixture retains the exact, live GUI-subsystem child handle.
    let idle = unsafe { WaitForInputIdle(HANDLE(fixture.child.as_raw_handle()), 200) };
    assert_eq!(
        idle, WAIT_TIMEOUT.0,
        "fixture must reproduce the old idle gate"
    );
    let began = Instant::now();
    await_settings_start_with_timeout(&mut fixture.child, Duration::from_secs(2)).unwrap();
    assert!(began.elapsed() < Duration::from_secs(2));
    assert!(fixture.child.try_wait().unwrap().is_none());
}

#[test]
fn hung_window_times_out_and_exact_child_is_reaped() {
    let program = FixtureProgram::new();
    let mut fixture = program.spawn("hung");
    fixture.await_fixture();
    assert!(settings_window(fixture.child.id()).unwrap().is_some());
    let began = Instant::now();
    let error = await_settings_start_with_timeout(&mut fixture.child, Duration::from_millis(250))
        .unwrap_err();
    assert_eq!(error.kind(), io::ErrorKind::TimedOut);
    assert!(error.to_string().contains("did not answer messages"));
    assert!(began.elapsed() < Duration::from_secs(2));
    assert!(fixture.child.try_wait().unwrap().is_some());
}

#[test]
fn hidden_wrong_class_empty_and_missing_windows_are_not_ready() {
    let program = FixtureProgram::new();
    for mode in ["hidden", "wrong_class", "empty_client", "no_window"] {
        let mut fixture = program.spawn(mode);
        fixture.await_fixture();
        assert!(
            settings_window(fixture.child.id()).unwrap().is_none(),
            "{mode}"
        );
        let began = Instant::now();
        let error =
            await_settings_start_with_timeout(&mut fixture.child, Duration::from_millis(150))
                .unwrap_err();
        assert_eq!(error.kind(), io::ErrorKind::TimedOut, "{mode}: {error}");
        assert!(error.to_string().contains("no visible settings window"));
        assert!(began.elapsed() < Duration::from_secs(2), "{mode}");
        assert!(fixture.child.try_wait().unwrap().is_some(), "{mode}");
    }
}

#[test]
fn another_process_host_window_cannot_satisfy_readiness() {
    let program = FixtureProgram::new();
    let mut foreign = program.spawn("busy");
    foreign.await_fixture();
    assert!(settings_window(foreign.child.id()).unwrap().is_some());
    let mut subject = program.spawn("no_window");
    subject.await_fixture();
    let error = await_settings_start_with_timeout(&mut subject.child, Duration::from_millis(150))
        .unwrap_err();
    assert_eq!(error.kind(), io::ErrorKind::TimedOut);
    assert!(subject.child.try_wait().unwrap().is_some());
    assert!(foreign.child.try_wait().unwrap().is_none());
}

#[test]
fn child_exit_is_reported_and_reaped_without_waiting_for_deadline() {
    let program = FixtureProgram::new();
    let mut fixture = program.spawn("early_exit");
    let began = Instant::now();
    let error =
        await_settings_start_with_timeout(&mut fixture.child, Duration::from_secs(2)).unwrap_err();
    assert_ne!(error.kind(), io::ErrorKind::TimedOut);
    assert!(error.to_string().contains("exited before becoming ready"));
    assert!(began.elapsed() < Duration::from_secs(2));
    assert_eq!(fixture.child.try_wait().unwrap().unwrap().code(), Some(7));
}

#[test]
#[ignore = "bounded offscreen child fixture, invoked only by the readiness tests"]
fn native_child_fixture() {
    let Some(mode) = std::env::var_os(FIXTURE_MODE) else {
        return;
    };
    let mode = mode.to_str().unwrap();
    if mode == "early_exit" {
        std::process::exit(7);
    }
    let ready = PathBuf::from(std::env::var_os(FIXTURE_READY).unwrap());
    if mode == "no_window" {
        fs::write(ready, "no-window fixture started").unwrap();
        std::thread::sleep(Duration::from_secs(5));
        return;
    }
    // SAFETY: Every fixture owns only its own offscreen, nonactivating tool
    // window, class and message loop. No fixture reads or injects global input.
    unsafe {
        let instance = HINSTANCE(GetModuleHandleW(None).unwrap().0);
        let class_name = if mode == "wrong_class" {
            w!("HonkReadyWrongClass")
        } else {
            w!("NativeSdkWindowsHost")
        };
        let class = WNDCLASSEXW {
            cbSize: std::mem::size_of::<WNDCLASSEXW>() as u32,
            lpfnWndProc: Some(fixture_window_proc),
            hInstance: instance,
            lpszClassName: class_name,
            ..Default::default()
        };
        assert_ne!(RegisterClassExW(&class), 0);
        let visible = if mode == "hidden" {
            WS_POPUP
        } else {
            WS_POPUP | WS_VISIBLE
        };
        let (width, height) = if mode == "empty_client" {
            (0, 0)
        } else {
            (200, 140)
        };
        let hwnd = CreateWindowExW(
            WS_EX_NOACTIVATE | WS_EX_TOOLWINDOW,
            class_name,
            w!("Goose"),
            visible,
            -30_000,
            -30_000,
            width,
            height,
            None,
            None,
            instance,
            None,
        )
        .unwrap();
        // Two queued messages stay pending as each removed one is replaced.
        // PeekMessage still dispatches sent WM_NULL messages, without ever
        // waiting for an empty input queue as the old readiness gate requires.
        PostMessageW(hwnd, WM_APP, WPARAM(0), LPARAM(0)).unwrap();
        PostMessageW(hwnd, WM_APP, WPARAM(0), LPARAM(0)).unwrap();
        fs::write(ready, "offscreen fixture window created").unwrap();
        let watchdog = Instant::now() + Duration::from_secs(5);
        if mode == "hung" {
            std::thread::sleep(Duration::from_secs(5));
        } else {
            while Instant::now() < watchdog {
                let mut message = MSG::default();
                if PeekMessageW(&mut message, None, 0, 0, PM_REMOVE).as_bool() {
                    PostMessageW(hwnd, WM_APP, WPARAM(0), LPARAM(0)).unwrap();
                    DispatchMessageW(&message);
                }
                std::thread::yield_now();
            }
        }
        let _ = DestroyWindow(hwnd);
    }
}

unsafe extern "system" fn fixture_window_proc(
    hwnd: HWND,
    message: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    if message == WM_NULL || message == WM_APP {
        LRESULT(0)
    } else {
        // SAFETY: The window class invokes this procedure with its own arguments.
        unsafe { DefWindowProcW(hwnd, message, wparam, lparam) }
    }
}
