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
        // GUI-subsystem entry points may have no standard handles at all.
        if let Ok(handle) = unsafe { GetStdHandle(kind) } {
            unsafe { SetHandleInformation(handle, HANDLE_FLAG_INHERIT.0, HANDLE_FLAGS(0))? };
        }
    }
    Ok(())
}
