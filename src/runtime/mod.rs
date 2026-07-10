#[cfg(windows)]
pub mod windows;

#[cfg(target_os = "macos")]
pub mod macos;

#[cfg(target_os = "linux")]
pub mod linux;

#[cfg(any(windows, target_os = "macos", target_os = "linux"))]
mod core;

#[cfg(any(windows, target_os = "macos", target_os = "linux"))]
use honk_config::BackendCapability;
#[cfg(any(windows, target_os = "macos", target_os = "linux"))]
use honk_config::{CliOverrides, Config};

#[cfg(any(windows, target_os = "macos", target_os = "linux"))]
#[derive(Debug, Clone)]
pub struct RuntimeOptions {
    pub config_path: std::path::PathBuf,
    pub config: Config,
    pub cli_overrides: CliOverrides,
}

#[cfg(any(windows, target_os = "macos", target_os = "linux"))]
pub(crate) fn audio_probe_capability(available: bool) -> BackendCapability {
    if available {
        BackendCapability::Supported
    } else {
        BackendCapability::Failed
    }
}

#[cfg(test)]
mod tests {
    use super::audio_probe_capability;
    use honk_config::BackendCapability;

    #[test]
    fn successful_audio_reprobe_recovers_failed_capability() {
        assert_eq!(audio_probe_capability(true), BackendCapability::Supported);
        assert_eq!(audio_probe_capability(false), BackendCapability::Failed);
    }
}
