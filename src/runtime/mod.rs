#[cfg(windows)]
pub mod windows;

#[cfg(windows)]
use honk_config::{CliOverrides, Config};

#[cfg(windows)]
#[derive(Debug, Clone)]
pub struct RuntimeOptions {
    pub config_path: std::path::PathBuf,
    pub config: Config,
    pub cli_overrides: CliOverrides,
}
