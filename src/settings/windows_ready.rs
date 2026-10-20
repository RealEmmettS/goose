//! Bounded startup of the exact verified native settings child.
//!
//! The SDK can keep its input queue occupied while it renders and handles input.
//! An empty queue is therefore not a readiness signal. Its real host window must
//! answer a harmless message instead. The accessibility bridge is a required
//! load-time DLL import, so reaching this window's procedure also follows mapping
//! both payloads, whose verified file leases remain held by the caller.

use std::{io, process::Child, time::Duration};
use windows::Win32::{
    Foundation::{BOOL, HWND, LPARAM, RECT, WPARAM},
    UI::WindowsAndMessaging::{
        EnumWindows, GetClassNameW, GetClientRect, GetWindowThreadProcessId, IsWindowVisible,
        SendMessageTimeoutW, SMTO_ABORTIFHUNG, SMTO_BLOCK, SMTO_ERRORONEXIT, WM_NULL,
    },
};

const HOST_CLASS: &str = "NativeSdkWindowsHost";
const STARTUP_TIMEOUT: Duration = Duration::from_secs(10);
const MESSAGE_TIMEOUT: Duration = Duration::from_millis(100);
const POLL_INTERVAL: Duration = Duration::from_millis(20);

pub(super) fn await_settings_start(mut child: Child) -> io::Result<()> {
    await_settings_start_with_timeout(&mut child, STARTUP_TIMEOUT)
}

fn await_settings_start_with_timeout(child: &mut Child, timeout: Duration) -> io::Result<()> {
    let result = wait_for_responsive_window(child, timeout);
    if let Err(error) = result {
        // Always request termination and reap the exact child before the caller
        // releases its verified executable/DLL leases. It may already have exited.
        let kill_result = child.kill();
        // Recognition has a deadline; cleanup must not abandon a possibly-live
        // child and release those leases merely to impose a second deadline.
        child.wait().map_err(|wait_error| {
            io::Error::other(format!(
                "{error}; could not reap the settings child (termination: {kill_result:?}): {wait_error}"
            ))
        })?;
        return Err(error);
    }
    Ok(())
}

fn wait_for_responsive_window(child: &mut Child, timeout: Duration) -> io::Result<()> {
    let deadline = std::time::Instant::now() + timeout;
    let mut observation = "no visible settings window";
    loop {
        require_running(child)?;
        let Some(remaining) = deadline.checked_duration_since(std::time::Instant::now()) else {
            return Err(startup_timeout(timeout, observation));
        };
        if remaining.is_zero() {
            return Err(startup_timeout(timeout, observation));
        }
        if let Some(hwnd) = settings_window(child.id())? {
            observation = "the settings window did not answer messages";
            let Some(remaining) = deadline.checked_duration_since(std::time::Instant::now()) else {
                return Err(startup_timeout(timeout, observation));
            };
            let milliseconds = remaining.min(MESSAGE_TIMEOUT).as_millis();
            if milliseconds == 0 {
                return Err(startup_timeout(timeout, observation));
            }
            let mut message_result = 0;
            // SAFETY: Only an HWND belonging to this exact child receives WM_NULL.
            // This system message has no pointer payload and causes no user input
            // or focus changes. Its nonzero API return, not the WM_NULL result
            // (normally zero), reports response. No flag may extend the timeout.
            let responded = unsafe {
                SendMessageTimeoutW(
                    hwnd,
                    WM_NULL,
                    WPARAM(0),
                    LPARAM(0),
                    SMTO_ABORTIFHUNG | SMTO_BLOCK | SMTO_ERRORONEXIT,
                    milliseconds as u32,
                    Some(&mut message_result),
                )
            }
            .0 != 0;
            require_running(child)?;
            if std::time::Instant::now() >= deadline {
                return Err(startup_timeout(timeout, observation));
            }
            if responded && is_settings_window(hwnd, child.id()) {
                return Ok(());
            }
        }
        let remaining = deadline.saturating_duration_since(std::time::Instant::now());
        std::thread::sleep(remaining.min(POLL_INTERVAL));
    }
}

fn require_running(child: &mut Child) -> io::Result<()> {
    match child.try_wait() {
        Ok(None) => Ok(()),
        Ok(Some(status)) => Err(io::Error::other(format!(
            "the settings application exited before becoming ready ({status})"
        ))),
        Err(error) => Err(io::Error::new(
            error.kind(),
            format!("could not inspect the settings child: {error}"),
        )),
    }
}

fn startup_timeout(timeout: Duration, observation: &str) -> io::Error {
    io::Error::new(
        io::ErrorKind::TimedOut,
        format!(
            "the settings application did not become responsive within {} seconds ({observation})",
            timeout.as_secs_f64()
        ),
    )
}

struct WindowSearch {
    process_id: u32,
    found: Option<HWND>,
}

fn settings_window(process_id: u32) -> io::Result<Option<HWND>> {
    let mut search = WindowSearch {
        process_id,
        found: None,
    };
    // SAFETY: EnumWindows invokes the callback synchronously. The stack-owned
    // search remains exclusively borrowed and valid until enumeration returns.
    let enumeration = unsafe {
        EnumWindows(
            Some(find_settings_window),
            LPARAM((&mut search as *mut WindowSearch) as isize),
        )
    };
    if search.found.is_some() {
        // The callback deliberately stops enumeration with FALSE on a match.
        return Ok(search.found);
    }
    enumeration.map_err(|error| {
        io::Error::other(format!("could not inspect settings windows: {error}"))
    })?;
    Ok(None)
}

unsafe extern "system" fn find_settings_window(hwnd: HWND, parameter: LPARAM) -> BOOL {
    // SAFETY: settings_window supplies this exclusive pointer for the synchronous callback.
    let search = unsafe { &mut *(parameter.0 as *mut WindowSearch) };
    if is_settings_window(hwnd, search.process_id) {
        search.found = Some(hwnd);
        BOOL(0)
    } else {
        BOOL(1)
    }
}

fn is_settings_window(hwnd: HWND, process_id: u32) -> bool {
    // SAFETY: These read-only HWND queries tolerate destruction between calls;
    // ownership and all readiness properties are checked again after messaging.
    unsafe {
        let mut owner = 0;
        if GetWindowThreadProcessId(hwnd, Some(&mut owner)) == 0
            || owner != process_id
            || !IsWindowVisible(hwnd).as_bool()
        {
            return false;
        }
        let mut class = [0; 64];
        let length = GetClassNameW(hwnd, &mut class);
        if length <= 0
            || !class[..length as usize]
                .iter()
                .copied()
                .eq(HOST_CLASS.encode_utf16())
        {
            return false;
        }
        let mut client = RECT::default();
        GetClientRect(hwnd, &mut client).is_ok()
            && client.right > client.left
            && client.bottom > client.top
    }
}

#[cfg(test)]
#[path = "windows_ready_tests.rs"]
mod tests;
