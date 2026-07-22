//! Shared local control channel for honk300.
//!
//! The CLI and config TUI both speak this finite protocol to the one running
//! goose instance. The engine stays below this layer and only receives closed,
//! platform-neutral command data.

mod platform;
mod protocol;

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
