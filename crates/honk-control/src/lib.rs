//! Shared local control channel for honk300.
//!
//! The CLI and config TUI both speak this finite protocol to the one running
//! goose instance. The engine stays below this layer and only receives closed,
//! platform-neutral command data.

mod platform;
mod protocol;

pub use platform::CommandServer;
pub use platform::{send_command, Singleton, SingletonStatus};
pub use protocol::{
    BundleStatus, CapabilityStatus, ControlCommand, ControlResponse, PlatformStatus, ProtocolError,
    RuntimeStatus,
};
