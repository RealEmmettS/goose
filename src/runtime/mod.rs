#[cfg(windows)]
pub mod windows;

#[cfg(target_os = "macos")]
pub mod macos;

#[cfg(target_os = "linux")]
pub mod linux;

#[cfg(any(windows, target_os = "macos", target_os = "linux"))]
use honk_config::{CliOverrides, Config};

#[cfg(any(windows, target_os = "macos", target_os = "linux"))]
#[derive(Debug, Clone)]
pub struct RuntimeOptions {
    pub config_path: std::path::PathBuf,
    pub config: Config,
    pub cli_overrides: CliOverrides,
}
