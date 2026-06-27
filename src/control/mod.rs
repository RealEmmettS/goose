pub mod platform;
pub mod protocol;

#[cfg(windows)]
pub use platform::CommandServer;
pub use platform::{send_command, Singleton, SingletonStatus};
pub use protocol::{ControlCommand, ControlResponse};
