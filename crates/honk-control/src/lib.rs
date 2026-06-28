//! Shared local control channel for honk300.
//!
//! The CLI and config TUI both speak this finite protocol to the one running
//! goose instance. The engine stays below this layer and only receives closed,
//! platform-neutral command data.

mod platform;
mod protocol;

#[cfg(windows)]
pub use platform::CommandServer;
pub use platform::{send_command, Singleton, SingletonStatus};
pub use protocol::{ControlCommand, ControlResponse, ProtocolError};
