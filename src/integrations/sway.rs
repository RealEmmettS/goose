use super::Error;
use serde_json::{json, Value};

pub(crate) fn sway_status() -> Value {
    #[cfg(target_os = "linux")]
    {
        let saved = super::directory()
            .and_then(|path| super::sway_consent::read(&path).map_err(Into::into));
        let (installed, current, detail) = match saved {
            Ok(Some(consent)) => (true, consent.current(), if consent.current() {
                "Sway observations are enabled for this user. Connect using native Wayland on a qualified Sway version."
            } else { "Repeat setup for this Sway observation update." }.to_owned()),
            Ok(None) => (false, false, "Sway observations are off. Setup allows window observation and fullscreen awareness. Movement, window rides and pointer control remain unavailable.".to_owned()),
            Err(error) => (false, false, format!("Sway setup cannot be read: {error}")),
        };
        let caps = match honk_control::send_command(honk_control::ControlCommand::SwayStatus) {
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
        "description": "Optional Sway observations are available on Linux."})
}

pub(crate) fn sway_setup() -> Result<Value, Error> {
    #[cfg(target_os = "linux")]
    {
        use std::io::Read;
        let mut random = [0u8; 16];
        std::fs::File::open("/dev/urandom")?.read_exact(&mut random)?;
        let nonce: String = random.iter().map(|byte| format!("{byte:02x}")).collect();
        super::sway_consent::install(&super::directory()?, &nonce)?;
        let message = match honk_control::send_command(honk_control::ControlCommand::SwayEnable) {
            Ok(honk_control::ControlResponse::Ok) => "Sway setup saved. Connecting the running goose.",
            Ok(honk_control::ControlResponse::Err(_)) => "Sway setup saved. Start in native Wayland mode on a qualified Sway version to connect.",
            Err(error) if matches!(error.kind(), std::io::ErrorKind::NotFound | std::io::ErrorKind::ConnectionRefused) =>
                "Sway setup saved. Start in native Wayland mode on a qualified Sway version to connect.",
            _ => "Sway setup saved, but live activation could not be confirmed. Refresh status or restart the goose.",
        };
        Ok(json!({"message": message, "sway": sway_status()}))
    }
    #[cfg(not(target_os = "linux"))]
    Err("Sway observations require Linux".into())
}

pub(crate) fn sway_remove() -> Result<Value, Error> {
    #[cfg(target_os = "linux")]
    {
        super::sway_consent::remove(&super::directory()?)?;
        match honk_control::send_command(honk_control::ControlCommand::SwayDisable) {
            Ok(honk_control::ControlResponse::Ok) => {}
            Err(error)
                if matches!(
                    error.kind(),
                    std::io::ErrorKind::NotFound | std::io::ErrorKind::ConnectionRefused
                ) => {}
            _ => return Err(
                "Sway setup removed, but live revocation could not be confirmed. Refresh status."
                    .into(),
            ),
        }
        Ok(json!({"message": "Sway observations removed.", "sway": sway_status()}))
    }
    #[cfg(not(target_os = "linux"))]
    Err("Sway observations require Linux".into())
}
