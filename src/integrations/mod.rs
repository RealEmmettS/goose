//! Explicit per-user Wayland setup. This is permission state, separate from drafts.
#[cfg(any(target_os = "linux", test))]
mod installed;
#[cfg(target_os = "linux")]
mod runtime;
#[cfg(target_os = "linux")]
pub(crate) use runtime::KwinRuntime;

use serde_json::{json, Value};
type Error = Box<dyn std::error::Error>;

#[cfg(any(target_os = "linux", test))]
const SCRIPT: &[u8] = include_bytes!("../../integrations/kwin/contents/code/main.js");

#[cfg(target_os = "linux")]
fn directory() -> Result<std::path::PathBuf, Error> {
    let config = honk_config::default_config_path().ok_or("User data directory unavailable")?;
    if !config.is_absolute() {
        return Err("The user data directory must be absolute".into());
    }
    Ok(config
        .parent()
        .ok_or("User data directory unavailable")?
        .join("wayland"))
}

pub(crate) fn status() -> Value {
    #[cfg(target_os = "linux")]
    {
        let result = directory().and_then(|path| installed::read(&path).map_err(Into::into));
        let (installed, current, detail) = match result {
            Ok(Some(consent)) => (true, consent.current(), if consent.current() {
                "KDE companion enabled for this user. It connects only in native Wayland mode."
            } else { "The KDE companion needs setup for this installed update." }.to_owned()),
            Ok(None) => (false, false, "KDE integration is off. Setup enables window observation, fullscreen awareness and bounded window actions. Pointer control requires a separate portal grant.".to_owned()),
            Err(error) => (false, false, format!("KDE setup cannot be read: {error}")),
        };
        let capabilities = match honk_control::send_command(
            honk_control::ControlCommand::WaylandStatus,
        ) {
            Ok(honk_control::ControlResponse::Wayland(status)) => json!({
                "windows": status.windows.label(), "movement": status.movement.label(),
                "pointer_observation": status.pointer_observation.label(),
                "pointer_control": status.pointer_control.label(), "fullscreen": status.fullscreen.label(),
                "dnd": status.dnd.label(), "prop_positioning": status.prop_positioning.label(),
            }),
            _ => Value::Null,
        };
        json!({"supported": true, "installed": installed, "current": current,
            "description": detail, "capabilities": capabilities})
    }
    #[cfg(not(target_os = "linux"))]
    json!({"supported": false, "installed": false, "current": false,
        "description": "Optional Wayland integrations are available on Linux."})
}

pub(crate) fn setup() -> Result<Value, Error> {
    #[cfg(target_os = "linux")]
    {
        use std::io::Read;
        let mut random = [0u8; 16];
        std::fs::File::open("/dev/urandom")?.read_exact(&mut random)?;
        let nonce = random
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect::<String>();
        installed::install(&directory()?, &nonce)?;
        let message = match honk_control::send_command(honk_control::ControlCommand::KwinEnable) {
            Ok(honk_control::ControlResponse::Ok) => "KDE setup saved. Connecting the running goose.",
            Ok(honk_control::ControlResponse::Err(_)) => "KDE setup saved. Start the goose in native Wayland mode on KDE to connect.",
            Err(error) if matches!(error.kind(), std::io::ErrorKind::NotFound | std::io::ErrorKind::ConnectionRefused) =>
                "KDE setup saved. Start the goose in native Wayland mode on KDE to connect.",
            _ => "KDE setup saved, but the running goose could not confirm activation. Refresh status or restart it.",
        };
        Ok(json!({"message": message, "integrations": status()}))
    }
    #[cfg(not(target_os = "linux"))]
    Err("KDE integration requires Linux".into())
}

pub(crate) fn remove() -> Result<Value, Error> {
    #[cfg(target_os = "linux")]
    {
        // Remove the saved grant first. The live owner also revalidates it before
        // exposing observations; the explicit IPC revokes immediately when received.
        let directory = directory()?;
        let previous = installed::read(&directory)?;
        installed::remove(&directory)?;
        match honk_control::send_command(honk_control::ControlCommand::KwinDisable) {
            Ok(honk_control::ControlResponse::Ok) => {}
            Err(error)
                if matches!(
                    error.kind(),
                    std::io::ErrorKind::NotFound | std::io::ErrorKind::ConnectionRefused
                ) =>
            {
                if let Some(consent) = previous {
                    honk_platform_linux::kwin::Bridge::retire_inactive_owned_script(
                        &consent.name(),
                    )?;
                }
            }
            _ => return Err(
                "KDE setup removed, but live revocation could not be confirmed. Refresh status."
                    .into(),
            ),
        }
        Ok(json!({"message": "KDE integration removed.", "integrations": status()}))
    }
    #[cfg(not(target_os = "linux"))]
    Err("KDE integration requires Linux".into())
}
