use super::Error;
use serde_json::{json, Value};

pub(crate) fn hyprland_status() -> Value {
    #[cfg(target_os = "linux")]
    {
        let saved = super::directory()
            .and_then(|path| super::hyprland_consent::read(&path).map_err(Into::into));
        let (installed, current, detail) = match saved {
            Ok(Some(consent)) => (true, consent.current(), if consent.current() {
                "Hyprland observations are enabled for this user. Connect using native Wayland on a qualified Hyprland version."
            } else { "Repeat setup for this Hyprland observation update." }.to_owned()),
            Ok(None) => (false, false, "Hyprland observations are off. Setup allows window observation and fullscreen awareness. Movement, window rides and pointer control remain unavailable.".to_owned()),
            Err(error) => (false, false, format!("Hyprland setup cannot be read: {error}")),
        };
        let caps = match honk_control::send_command(honk_control::ControlCommand::HyprlandStatus) {
            Ok(honk_control::ControlResponse::Wayland(status)) => json!({
                "windows": status.windows.label(), "fullscreen": status.fullscreen.label(),
                "movement": status.movement.label(), "pointer_control": status.pointer_control.label(),
                "pointer_observation": status.pointer_observation.label(), "dnd": status.dnd.label(),
                "prop_positioning": status.prop_positioning.label(),
            }),
            _ => Value::Null,
        };
        json!({"supported": true, "installed": installed, "current": current,
            "description": detail, "capabilities": caps})
    }
    #[cfg(not(target_os = "linux"))]
    json!({"supported": false, "installed": false, "current": false,
        "description": "Optional Hyprland observations are available on Linux."})
}

pub(crate) fn hyprland_setup() -> Result<Value, Error> {
    #[cfg(target_os = "linux")]
    {
        use std::io::Read;
        let mut random = [0u8; 16];
        std::fs::File::open("/dev/urandom")?.read_exact(&mut random)?;
        let nonce: String = random.iter().map(|byte| format!("{byte:02x}")).collect();
        super::hyprland_consent::install(&super::directory()?, &nonce)?;
        let message = match honk_control::send_command(honk_control::ControlCommand::HyprlandEnable) {
            Ok(honk_control::ControlResponse::Ok) => "Hyprland setup saved. Connecting the running goose.",
            Ok(honk_control::ControlResponse::Err(_)) => "Hyprland setup saved. Start in native Wayland mode on a qualified Hyprland version to connect.",
            Err(error) if matches!(error.kind(), std::io::ErrorKind::NotFound | std::io::ErrorKind::ConnectionRefused) =>
                "Hyprland setup saved. Start in native Wayland mode on a qualified Hyprland version to connect.",
            _ => "Hyprland setup saved, but live activation could not be confirmed. Refresh status or restart the goose.",
        };
        Ok(json!({"message": message, "hyprland": hyprland_status()}))
    }
    #[cfg(not(target_os = "linux"))]
    Err("Hyprland observations require Linux".into())
}

pub(crate) fn hyprland_remove() -> Result<Value, Error> {
    #[cfg(target_os = "linux")]
    {
        super::hyprland_consent::remove(&super::directory()?)?;
        match honk_control::send_command(honk_control::ControlCommand::HyprlandDisable) {
            Ok(honk_control::ControlResponse::Ok) => {}
            Err(error)
                if matches!(
                    error.kind(),
                    std::io::ErrorKind::NotFound | std::io::ErrorKind::ConnectionRefused
                ) => {}
            _ => return Err(
                "Hyprland setup removed, but live revocation could not be confirmed. Refresh status."
                    .into(),
            ),
        }
        Ok(json!({"message": "Hyprland observations removed.", "hyprland": hyprland_status()}))
    }
    #[cfg(not(target_os = "linux"))]
    Err("Hyprland observations require Linux".into())
}
