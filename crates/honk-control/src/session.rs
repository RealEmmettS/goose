//! Additive runtime session detail. Existing STATUS frames remain byte-compatible.
use crate::{CapabilityStatus, ProtocolError};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DesktopBackend {
    X11,
    X11OnWayland,
    Wayland,
    Headless,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DesktopEnvironment {
    Kde,
    Gnome,
    Sway,
    Hyprland,
    Labwc,
    Unknown,
}

impl DesktopEnvironment {
    /// Environment hints are display information, never authority to enable an adapter.
    pub fn from_hint(value: Option<&str>) -> Self {
        let Some(value) = value.filter(|value| value.len() <= 256) else {
            return Self::Unknown;
        };
        for name in value.split(':') {
            let desktop = match name.trim().to_ascii_lowercase().as_str() {
                "kde" => Self::Kde,
                "gnome" => Self::Gnome,
                "sway" => Self::Sway,
                "hyprland" => Self::Hyprland,
                "labwc" => Self::Labwc,
                _ => continue,
            };
            return desktop;
        }
        Self::Unknown
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Kde => "KDE",
            Self::Gnome => "GNOME",
            Self::Sway => "Sway",
            Self::Hyprland => "Hyprland",
            Self::Labwc => "labwc",
            Self::Unknown => "unknown",
        }
    }
}

impl DesktopBackend {
    pub fn label(self) -> &'static str {
        match self {
            Self::X11 => "X11",
            Self::X11OnWayland => "X11 (Wayland session)",
            Self::Wayland => "native Wayland",
            Self::Headless => "headless",
            Self::Unknown => "unknown",
        }
    }

    fn wire(self) -> &'static str {
        match self {
            Self::X11 => "X11",
            Self::X11OnWayland => "XWAY",
            Self::Wayland => "WAY",
            Self::Headless => "HEAD",
            Self::Unknown => "UNK",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SessionStatus {
    pub backend: DesktopBackend,
    pub desktop: DesktopEnvironment,
    pub prop_positioning: CapabilityStatus,
}

impl SessionStatus {
    pub(crate) fn encode(self) -> String {
        format!(
            "SESSION {} {} {}\n",
            self.backend.wire(),
            self.desktop.label(),
            self.prop_positioning.encode()
        )
    }

    pub(crate) fn decode(
        mut parts: std::str::SplitAsciiWhitespace<'_>,
    ) -> Result<Self, ProtocolError> {
        let invalid = ProtocolError::MalformedResponse;
        let backend = match parts.next() {
            Some("X11") => DesktopBackend::X11,
            Some("XWAY") => DesktopBackend::X11OnWayland,
            Some("WAY") => DesktopBackend::Wayland,
            Some("HEAD") => DesktopBackend::Headless,
            Some("UNK") => DesktopBackend::Unknown,
            _ => return Err(invalid),
        };
        let desktop = match parts.next() {
            Some("KDE") => DesktopEnvironment::Kde,
            Some("GNOME") => DesktopEnvironment::Gnome,
            Some("Sway") => DesktopEnvironment::Sway,
            Some("Hyprland") => DesktopEnvironment::Hyprland,
            Some("labwc") => DesktopEnvironment::Labwc,
            Some("unknown") => DesktopEnvironment::Unknown,
            _ => return Err(invalid),
        };
        let prop_positioning = CapabilityStatus::decode(parts.next().ok_or(invalid.clone())?)?;
        if parts.next().is_some() {
            return Err(invalid);
        }
        Ok(Self {
            backend,
            desktop,
            prop_positioning,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ControlCommand, ControlResponse};

    #[test]
    fn session_details_round_trip_without_extending_legacy_status() {
        let command = ControlCommand::Session;
        assert_eq!(
            ControlCommand::decode(command.encode().as_bytes()).unwrap(),
            command
        );
        for backend in [
            DesktopBackend::X11,
            DesktopBackend::X11OnWayland,
            DesktopBackend::Wayland,
            DesktopBackend::Headless,
            DesktopBackend::Unknown,
        ] {
            for desktop in [
                DesktopEnvironment::Kde,
                DesktopEnvironment::Gnome,
                DesktopEnvironment::Sway,
                DesktopEnvironment::Hyprland,
                DesktopEnvironment::Labwc,
                DesktopEnvironment::Unknown,
            ] {
                for prop_positioning in [
                    CapabilityStatus::Unprobed,
                    CapabilityStatus::Supported,
                    CapabilityStatus::Unsupported,
                    CapabilityStatus::Denied,
                    CapabilityStatus::Failed,
                ] {
                    let response = ControlResponse::Session(SessionStatus {
                        backend,
                        desktop,
                        prop_positioning,
                    });
                    let encoded = response.encode();
                    assert!(encoded.len() <= 128);
                    assert_eq!(
                        ControlResponse::decode(encoded.as_bytes()).unwrap(),
                        response
                    );
                }
            }
        }
        for invalid in [
            "SESSION WAY",
            "SESSION WAY KDE yes",
            "SESSION WAY forged S",
            "SESSION WAY KDE S extra",
        ] {
            assert!(
                ControlResponse::decode(invalid.as_bytes()).is_err(),
                "{invalid}"
            );
        }
    }

    #[test]
    fn desktop_hints_are_exact_bounded_and_never_echo_untrusted_text() {
        assert_eq!(
            DesktopEnvironment::from_hint(Some("ubuntu:GNOME")),
            DesktopEnvironment::Gnome
        );
        assert_eq!(
            DesktopEnvironment::from_hint(Some(" KDE :other")),
            DesktopEnvironment::Kde
        );
        assert_eq!(
            DesktopEnvironment::from_hint(Some("not-kde")),
            DesktopEnvironment::Unknown
        );
        assert_eq!(
            DesktopEnvironment::from_hint(Some("KDE\x1b[2J")),
            DesktopEnvironment::Unknown
        );
        assert_eq!(
            DesktopEnvironment::from_hint(Some(&"KDE:".repeat(100))),
            DesktopEnvironment::Unknown
        );
        assert_eq!(
            DesktopEnvironment::from_hint(None),
            DesktopEnvironment::Unknown
        );
    }
}
