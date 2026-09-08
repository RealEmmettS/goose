use super::Error;
use serde_json::{json, Value};

pub(crate) fn gnome_status() -> Value {
    #[cfg(target_os = "linux")]
    {
        let saved = super::directory()
            .and_then(|path| super::gnome_consent::read(&path).map_err(Into::into));
        let (installed, current, detail) = match saved {
            Ok(Some(consent)) => {
                let current = std::env::current_exe().is_ok_and(|path| consent.current(&path));
                (true, current, if current {
                    "GNOME companion setup is saved. Use the compatible XWayland overlay. After first setup or a companion update, sign out and back in, then enable Honk300 desktop observations in GNOME Extensions if needed."
                } else { "GNOME companion setup is incomplete or belongs to another update. Repeat setup, or finish removing it before changing versions." }.to_owned())
            }
            Ok(None) => (false, false, "GNOME observations are off. Setup allows window, fullscreen and user-drag observations. The goose keeps its compatible XWayland overlay and owned notes. Pointer control and foreign-window movement remain unavailable.".to_owned()),
            Err(error) => (false, false, format!("GNOME setup cannot be read: {error}")),
        };
        let caps = match honk_control::send_command(honk_control::ControlCommand::GnomeStatus) {
            Ok(honk_control::ControlResponse::Wayland(status)) => json!({
                "windows": status.windows.label(), "fullscreen": status.fullscreen.label(),
                "movement": status.movement.label(), "pointer_control": status.pointer_control.label(),
                "pointer_observation": status.pointer_observation.label(), "dnd": status.dnd.label(),
                "prop_positioning": status.prop_positioning.label(),
                "user_drag": status.windows.label(),
            }),
            _ => Value::Null,
        };
        json!({"supported": true, "installed": installed, "current": current,
            "description": detail, "capabilities": caps})
    }
    #[cfg(not(target_os = "linux"))]
    json!({"supported": false, "installed": false, "current": false,
        "description": "Optional GNOME observations are available on Linux."})
}

pub(crate) fn gnome_setup() -> Result<Value, Error> {
    #[cfg(target_os = "linux")]
    {
        use std::io::Read;
        let mut random = [0_u8; 16];
        std::fs::File::open("/dev/urandom")?.read_exact(&mut random)?;
        let nonce: String = random.iter().map(|byte| format!("{byte:02x}")).collect();
        let directory = super::directory()?;
        let consent = super::gnome_consent::install(&directory, &nonce, &std::env::current_exe()?)?;
        let enabled = honk_platform_linux::gnome::Connection::connect(
            consent.nonce,
            super::gnome_consent::build_identity(),
        )
        .and_then(|connection| connection.set_companion_enabled(true));
        let message = if matches!(enabled, Ok(true)) {
            match honk_control::send_command(honk_control::ControlCommand::GnomeEnable) {
                Ok(honk_control::ControlResponse::Ok) => "GNOME companion enabled. Connecting the running goose; refresh status to confirm observations.",
                _ => "GNOME companion enabled. Start the goose using its compatible XWayland overlay to connect.",
            }
        } else {
            "GNOME companion setup saved. Sign out and back in so GNOME can discover the extension, then enable Honk300 desktop observations in GNOME Extensions and repeat setup. The goose keeps running independently."
        };
        Ok(json!({"message": message, "gnome": gnome_status()}))
    }
    #[cfg(not(target_os = "linux"))]
    Err("GNOME observations require Linux".into())
}

pub(crate) fn gnome_remove() -> Result<Value, Error> {
    #[cfg(target_os = "linux")]
    {
        let directory = super::directory()?;
        let previous = super::gnome_consent::revoke(&directory)?;
        // The durable revoking phase withdraws permission before any live call.
        match honk_control::send_command(honk_control::ControlCommand::GnomeDisable) {
            Ok(honk_control::ControlResponse::Ok) => {},
            Err(error) if matches!(error.kind(), std::io::ErrorKind::NotFound | std::io::ErrorKind::ConnectionRefused) => {},
            _ => return Err("GNOME consent revoked, but the running goose did not confirm removal. Refresh status.".into()),
        }
        if let Some(consent) = previous {
            // Refuse edited or unrelated contents before changing the loaded UUID.
            super::gnome_consent::known_files(&directory, &consent)?;
            // Missing Shell leaves the recorded grant revoked. Any loaded copy
            // rejects that grant even if the user's current bus is unavailable.
            if let Ok(connection) = honk_platform_linux::gnome::Connection::connect(
                consent.nonce.clone(),
                super::gnome_consent::build_identity(),
            ) {
                if !connection.set_companion_enabled(false)? {
                    return Err("GNOME consent revoked. Finish disabling Honk300 desktop observations in GNOME Extensions, then remove again.".into());
                }
            }
            super::gnome_consent::remove_files(&directory, &consent)?;
        }
        Ok(json!({"message": "GNOME companion and observations removed.", "gnome": gnome_status()}))
    }
    #[cfg(not(target_os = "linux"))]
    Err("GNOME observations require Linux".into())
}
