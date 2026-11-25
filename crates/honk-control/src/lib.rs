//! Shared local control channel for honk300.
//!
//! The CLI and config TUI both speak this finite protocol to the one running
//! goose instance. The engine stays below this layer and only receives closed,
//! platform-neutral command data.

/// Locate the running image for install ownership and exact-sibling launches.
#[cfg(not(target_os = "macos"))]
pub use std::env::current_exe as current_executable;

/// dyld can report the command symlink used at launch. Ask the kernel for the
/// executing image so an alias (even one subsequently moved) cannot select a sibling.
#[cfg(target_os = "macos")]
pub fn current_executable() -> std::io::Result<std::path::PathBuf> {
    use std::ffi::{CStr, OsStr};
    use std::os::unix::ffi::OsStrExt;
    use std::path::Path;

    let mut buffer = [0u8; libc::PROC_PIDPATHINFO_MAXSIZE as usize];
    // SAFETY: the buffer is writable for the supplied length and this is our own PID.
    let size = unsafe {
        libc::proc_pidpath(
            libc::getpid(),
            buffer.as_mut_ptr().cast(),
            buffer.len() as u32,
        )
    };
    if size <= 0 {
        return Err(std::io::Error::last_os_error());
    }
    let bytes = CStr::from_bytes_until_nul(&buffer).map_err(std::io::Error::other)?;
    let path = Path::new(OsStr::from_bytes(bytes.to_bytes()));
    if !path.is_absolute() {
        return Err(std::io::Error::other("invalid running executable path"));
    }
    Ok(path.to_owned())
}

pub mod icon;
mod platform;
mod presence;
mod protocol;
mod session;
mod wayland;

pub use presence::{combine_capabilities, PresenceStatus};
pub use session::{DesktopBackend, DesktopEnvironment, SessionStatus};
pub use wayland::WaylandStatus;

pub use platform::CommandServer;
pub use platform::{
    send_command, wait_for_shutdown, wait_for_shutdown_lease, LifecycleLease, Singleton,
    SingletonStatus, UpdateGuard,
};
pub use protocol::{
    BundleStatus, CapabilityStatus, ControlCommand, ControlResponse, PlatformStatus, ProtocolError,
    RuntimeStatus,
};

/// A user action emitted by an operating-system control surface.
///
/// This is intentionally smaller than [`ControlCommand`]. Native trays and the macOS menu bar
/// may open the existing configuration TUI, launch the verified updater helper, or request the
/// existing graceful shutdown, but they do not gain a second configuration model or a new IPC
/// command namespace.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ControlSurfaceCommand {
    Configure,
    Update,
    Quit,
}
