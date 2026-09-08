//! Optional adapter capabilities, separate from the legacy aggregate STATUS frame.
use crate::{CapabilityStatus, ProtocolError};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WaylandStatus {
    pub windows: CapabilityStatus,
    pub movement: CapabilityStatus,
    pub pointer_observation: CapabilityStatus,
    pub pointer_control: CapabilityStatus,
    pub fullscreen: CapabilityStatus,
    pub dnd: CapabilityStatus,
    pub prop_positioning: CapabilityStatus,
}

impl Default for WaylandStatus {
    fn default() -> Self {
        Self {
            windows: CapabilityStatus::Unsupported,
            movement: CapabilityStatus::Unsupported,
            pointer_observation: CapabilityStatus::Unsupported,
            pointer_control: CapabilityStatus::Unsupported,
            fullscreen: CapabilityStatus::Unsupported,
            dnd: CapabilityStatus::Unsupported,
            prop_positioning: CapabilityStatus::Unsupported,
        }
    }
}

impl WaylandStatus {
    pub(crate) fn encode(self) -> String {
        let fields = [
            self.windows,
            self.movement,
            self.pointer_observation,
            self.pointer_control,
            self.fullscreen,
            self.dnd,
            self.prop_positioning,
        ];
        format!(
            "WAYLAND {}\n",
            fields.map(CapabilityStatus::encode).join(" ")
        )
    }

    pub(crate) fn decode(
        mut parts: std::str::SplitAsciiWhitespace<'_>,
    ) -> Result<Self, ProtocolError> {
        let mut next =
            || CapabilityStatus::decode(parts.next().ok_or(ProtocolError::MalformedResponse)?);
        let status = Self {
            windows: next()?,
            movement: next()?,
            pointer_observation: next()?,
            pointer_control: next()?,
            fullscreen: next()?,
            dnd: next()?,
            prop_positioning: next()?,
        };
        if parts.next().is_some() {
            return Err(ProtocolError::MalformedResponse);
        }
        Ok(status)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ControlCommand, ControlResponse};

    #[test]
    fn adapter_capabilities_and_controls_have_bounded_distinct_frames() {
        for command in [
            ControlCommand::WaylandStatus,
            ControlCommand::KwinEnable,
            ControlCommand::KwinDisable,
            ControlCommand::SwayStatus,
            ControlCommand::SwayEnable,
            ControlCommand::SwayDisable,
            ControlCommand::GnomeStatus,
            ControlCommand::GnomeEnable,
            ControlCommand::GnomeDisable,
            ControlCommand::HyprlandStatus,
            ControlCommand::HyprlandEnable,
            ControlCommand::HyprlandDisable,
            ControlCommand::PointerRequest,
            ControlCommand::PointerCancel,
        ] {
            assert_eq!(
                ControlCommand::decode(command.encode().as_bytes()).unwrap(),
                command
            );
        }
        let response = ControlResponse::Wayland(WaylandStatus {
            windows: CapabilityStatus::Supported,
            movement: CapabilityStatus::Supported,
            pointer_observation: CapabilityStatus::Supported,
            pointer_control: CapabilityStatus::Denied,
            fullscreen: CapabilityStatus::Supported,
            dnd: CapabilityStatus::Unsupported,
            prop_positioning: CapabilityStatus::Unprobed,
        });
        assert!(response.encode().len() <= 128);
        assert_eq!(
            ControlResponse::decode(response.encode().as_bytes()).unwrap(),
            response
        );
        for frame in [
            "WAYLAND S S",
            "WAYLAND S S S D S U ?",
            "WAYLAND S S S D S U U extra",
        ] {
            assert!(ControlResponse::decode(frame.as_bytes()).is_err());
        }
        assert!(ControlCommand::decode(b"HONK300/1 KWIN_ENABLE extra").is_err());
    }
}
