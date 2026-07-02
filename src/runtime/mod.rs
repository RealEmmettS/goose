#[cfg(windows)]
pub mod windows;

#[cfg(target_os = "macos")]
pub mod macos;

#[cfg(any(windows, target_os = "macos"))]
use honk_config::{CliOverrides, Config};

#[cfg(any(windows, target_os = "macos"))]
#[derive(Debug, Clone)]
pub struct RuntimeOptions {
    pub config_path: std::path::PathBuf,
    pub config: Config,
    pub cli_overrides: CliOverrides,
}
