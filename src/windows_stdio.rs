//! Keep a controller's inherited pipes out of independently launched processes.
use windows::Win32::{
    Foundation::{SetHandleInformation, HANDLE_FLAGS, HANDLE_FLAG_INHERIT},
    System::Console::{GetStdHandle, STD_ERROR_HANDLE, STD_INPUT_HANDLE, STD_OUTPUT_HANDLE},
};

pub(crate) fn prevent_inheritance() -> windows::core::Result<()> {
    // Run before starting any threads or children. Stdio::null() selects the child's
    // standard streams, but Rust's Windows spawn still inherits other handles whose
    // inherit bit is set. Otherwise our original stdout/stderr pipes survive in the
    // detached runtime and output-capturing CLI/GUI callers never receive EOF.
    // Clearing this bit neither closes nor replaces our own streams. Explicit
    // Stdio::inherit() remains valid: std creates its own inheritable duplicates.
    for kind in [STD_INPUT_HANDLE, STD_OUTPUT_HANDLE, STD_ERROR_HANDLE] {
        // GUI-subsystem entry points may have no standard handles at all. The
        // pinned windows binding returns Err for both NULL and INVALID_HANDLE_VALUE.
        if let Ok(handle) = unsafe { GetStdHandle(kind) } {
            unsafe { SetHandleInformation(handle, HANDLE_FLAG_INHERIT.0, HANDLE_FLAGS(0))? };
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::os::windows::io::AsRawHandle;
    use windows::Win32::{
        Foundation::{GetHandleInformation, HANDLE},
        System::Console::SetStdHandle,
    };

    #[test]
    fn absent_standard_handles_are_skipped_and_valid_inheritance_is_cleared() {
        const CHILD: &str = "HONK300_TEST_STDIO_CHILD";
        if std::env::var_os(CHILD).is_none() {
            let result = std::process::Command::new(std::env::current_exe().unwrap())
                .args([
                    "--exact",
                    "windows_stdio::tests::absent_standard_handles_are_skipped_and_valid_inheritance_is_cleared",
                    "--nocapture",
                ])
                .env(CHILD, "1")
                .output()
                .unwrap();
            assert!(result.status.success(), "{result:?}");
            return;
        }
        // Only this isolated test process changes its standard handle table.
        let kinds = [STD_INPUT_HANDLE, STD_OUTPUT_HANDLE, STD_ERROR_HANDLE];
        let original = kinds.map(|kind| unsafe { GetStdHandle(kind) }.unwrap_or_default());
        for kind in kinds {
            unsafe { SetStdHandle(kind, HANDLE::default()).unwrap() };
        }
        let absent = prevent_inheritance();
        let file = std::fs::OpenOptions::new().write(true).open("NUL").unwrap();
        let handle = HANDLE(file.as_raw_handle());
        unsafe {
            SetHandleInformation(handle, HANDLE_FLAG_INHERIT.0, HANDLE_FLAG_INHERIT).unwrap();
            SetStdHandle(STD_OUTPUT_HANDLE, handle).unwrap();
        }
        let present = prevent_inheritance();
        let mut flags = 0;
        let information = unsafe { GetHandleInformation(handle, &mut flags) };
        for (kind, handle) in kinds.into_iter().zip(original) {
            unsafe { SetStdHandle(kind, handle).unwrap() };
        }
        absent.unwrap();
        present.unwrap();
        information.unwrap();
        assert_eq!(flags & HANDLE_FLAG_INHERIT.0, 0);
    }
}
