#![cfg_attr(not(windows), allow(dead_code))]

use honk_engine::{PokeAction, PokeOutcome};
use std::error::Error;
use std::fmt;

const VERSION: &str = "HONK300/1";
const MAX_FRAME_BYTES: usize = 128;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ControlCommand {
    Stop,
    Reload,
    Status,
    Do(PokeAction),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ControlResponse {
    Ok,
    Err(String),
    Status(RuntimeStatus),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlatformStatus {
    Windows,
    Macos,
    Linux,
    Other,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BundleStatus {
    App,
    Bare,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CapabilityStatus {
    Supported,
    Unsupported,
    Denied,
    Failed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RuntimeStatus {
    pub running: bool,
    pub platform: PlatformStatus,
    pub bundle: BundleStatus,
    pub accessibility: CapabilityStatus,
    pub cursor: CapabilityStatus,
    pub window: CapabilityStatus,
    pub collect: CapabilityStatus,
    pub presence: CapabilityStatus,
    pub audio: CapabilityStatus,
    pub notes: u32,
    pub memes: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProtocolError {
    Empty,
    TooLarge,
    WrongVersion,
    UnknownCommand,
    UnknownAction,
    ExtraTokens,
    MissingAction,
    MalformedResponse,
}

impl fmt::Display for ProtocolError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let message = match self {
            Self::Empty => "empty control frame",
            Self::TooLarge => "control frame is too large",
            Self::WrongVersion => "unsupported control protocol version",
            Self::UnknownCommand => "unknown control command",
            Self::UnknownAction => "unknown poke action",
            Self::ExtraTokens => "unexpected extra tokens",
            Self::MissingAction => "missing poke action",
            Self::MalformedResponse => "malformed control response",
        };
        f.write_str(message)
    }
}

impl Error for ProtocolError {}

impl ControlCommand {
    pub fn encode(self) -> String {
        match self {
            Self::Stop => format!("{VERSION} STOP\n"),
            Self::Reload => format!("{VERSION} RELOAD\n"),
            Self::Status => format!("{VERSION} STATUS\n"),
            Self::Do(action) => format!("{VERSION} DO {}\n", encode_action(action)),
        }
    }

    pub fn decode(frame: &[u8]) -> Result<Self, ProtocolError> {
        if frame.is_empty() {
            return Err(ProtocolError::Empty);
        }
        if frame.len() > MAX_FRAME_BYTES {
            return Err(ProtocolError::TooLarge);
        }
        let text = std::str::from_utf8(frame).map_err(|_| ProtocolError::UnknownCommand)?;
        let mut parts = text.trim().split_ascii_whitespace();
        if parts.next() != Some(VERSION) {
            return Err(ProtocolError::WrongVersion);
        }
        let Some(command) = parts.next() else {
            return Err(ProtocolError::UnknownCommand);
        };
        match command {
            "STOP" => {
                ensure_end(parts)?;
                Ok(Self::Stop)
            }
            "RELOAD" => {
                ensure_end(parts)?;
                Ok(Self::Reload)
            }
            "STATUS" => {
                ensure_end(parts)?;
                Ok(Self::Status)
            }
            "DO" => {
                let Some(action) = parts.next() else {
                    return Err(ProtocolError::MissingAction);
                };
                ensure_end(parts)?;
                Ok(Self::Do(decode_action(action)?))
            }
            _ => Err(ProtocolError::UnknownCommand),
        }
    }
}

impl From<PokeOutcome> for ControlResponse {
    /// Map an engine poke result onto the wire response so `do <action>` reports the real
    /// outcome instead of a blanket "received". Rejections become error codes the CLI/TUI show.
    fn from(outcome: PokeOutcome) -> Self {
        match outcome {
            PokeOutcome::Applied => Self::Ok,
            PokeOutcome::Busy => Self::Err("BUSY".into()),
            PokeOutcome::Unsupported => Self::Err("UNSUPPORTED".into()),
        }
    }
}

impl ControlResponse {
    pub fn encode(&self) -> String {
        match self {
            Self::Ok => "OK\n".to_string(),
            Self::Err(code) => format!("ERR {code}\n"),
            Self::Status(status) => status.encode(),
        }
    }

    pub fn decode(frame: &[u8]) -> Result<Self, ProtocolError> {
        if frame.is_empty() {
            return Err(ProtocolError::Empty);
        }
        if frame.len() > MAX_FRAME_BYTES {
            return Err(ProtocolError::TooLarge);
        }
        let text = std::str::from_utf8(frame).map_err(|_| ProtocolError::MalformedResponse)?;
        let mut parts = text.trim().split_ascii_whitespace();
        match parts.next() {
            Some("OK") => {
                ensure_end(parts)?;
                Ok(Self::Ok)
            }
            Some("ERR") => {
                let Some(code) = parts.next() else {
                    return Err(ProtocolError::MalformedResponse);
                };
                ensure_end(parts)?;
                Ok(Self::Err(code.to_string()))
            }
            Some("STATUS") => RuntimeStatus::decode(parts).map(Self::Status),
            _ => Err(ProtocolError::MalformedResponse),
        }
    }
}

impl RuntimeStatus {
    pub fn not_running() -> Self {
        Self {
            running: false,
            platform: PlatformStatus::current(),
            bundle: BundleStatus::Unknown,
            accessibility: CapabilityStatus::Unsupported,
            cursor: CapabilityStatus::Unsupported,
            window: CapabilityStatus::Unsupported,
            collect: CapabilityStatus::Unsupported,
            presence: CapabilityStatus::Unsupported,
            audio: CapabilityStatus::Unsupported,
            notes: 0,
            memes: 0,
        }
    }

    fn encode(&self) -> String {
        format!(
            "STATUS G={} P={} B={} A={} C={} W={} K={} R={} O={} N={} M={}\n",
            u8::from(self.running),
            self.platform.encode(),
            self.bundle.encode(),
            self.accessibility.encode(),
            self.cursor.encode(),
            self.window.encode(),
            self.collect.encode(),
            self.presence.encode(),
            self.audio.encode(),
            self.notes,
            self.memes
        )
    }

    fn decode(parts: std::str::SplitAsciiWhitespace<'_>) -> Result<Self, ProtocolError> {
        let mut status = RuntimeStatus::not_running();
        let mut seen = 0u16;
        for token in parts {
            let Some((key, value)) = token.split_once('=') else {
                return Err(ProtocolError::MalformedResponse);
            };
            match key {
                "G" => {
                    status.running = match value {
                        "0" => false,
                        "1" => true,
                        _ => return Err(ProtocolError::MalformedResponse),
                    };
                    seen |= 1 << 0;
                }
                "P" => {
                    status.platform = PlatformStatus::decode(value)?;
                    seen |= 1 << 1;
                }
                "B" => {
                    status.bundle = BundleStatus::decode(value)?;
                    seen |= 1 << 2;
                }
                "A" => {
                    status.accessibility = CapabilityStatus::decode(value)?;
                    seen |= 1 << 3;
                }
                "C" => {
                    status.cursor = CapabilityStatus::decode(value)?;
                    seen |= 1 << 4;
                }
                "W" => {
                    status.window = CapabilityStatus::decode(value)?;
                    seen |= 1 << 5;
                }
                "K" => {
                    status.collect = CapabilityStatus::decode(value)?;
                    seen |= 1 << 6;
                }
                "R" => {
                    status.presence = CapabilityStatus::decode(value)?;
                    seen |= 1 << 7;
                }
                "O" => {
                    status.audio = CapabilityStatus::decode(value)?;
                    seen |= 1 << 8;
                }
                "N" => {
                    status.notes = value
                        .parse()
                        .map_err(|_| ProtocolError::MalformedResponse)?;
                    seen |= 1 << 9;
                }
                "M" => {
                    status.memes = value
                        .parse()
                        .map_err(|_| ProtocolError::MalformedResponse)?;
                    seen |= 1 << 10;
                }
                _ => return Err(ProtocolError::MalformedResponse),
            }
        }
        if seen == 0b111_1111_1111 {
            Ok(status)
        } else {
            Err(ProtocolError::MalformedResponse)
        }
    }
}

impl PlatformStatus {
    pub fn current() -> Self {
        if cfg!(windows) {
            Self::Windows
        } else if cfg!(target_os = "macos") {
            Self::Macos
        } else if cfg!(target_os = "linux") {
            Self::Linux
        } else {
            Self::Other
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Windows => "Windows",
            Self::Macos => "macOS",
            Self::Linux => "Linux",
            Self::Other => "other",
        }
    }

    fn encode(self) -> &'static str {
        match self {
            Self::Windows => "WIN",
            Self::Macos => "MAC",
            Self::Linux => "LIN",
            Self::Other => "OTH",
        }
    }

    fn decode(value: &str) -> Result<Self, ProtocolError> {
        match value {
            "WIN" => Ok(Self::Windows),
            "MAC" => Ok(Self::Macos),
            "LIN" => Ok(Self::Linux),
            "OTH" => Ok(Self::Other),
            _ => Err(ProtocolError::MalformedResponse),
        }
    }
}

impl BundleStatus {
    pub fn label(self) -> &'static str {
        match self {
            Self::App => ".app",
            Self::Bare => "bare binary",
            Self::Unknown => "unknown",
        }
    }

    fn encode(self) -> &'static str {
        match self {
            Self::App => "APP",
            Self::Bare => "BARE",
            Self::Unknown => "UNK",
        }
    }

    fn decode(value: &str) -> Result<Self, ProtocolError> {
        match value {
            "APP" => Ok(Self::App),
            "BARE" => Ok(Self::Bare),
            "UNK" => Ok(Self::Unknown),
            _ => Err(ProtocolError::MalformedResponse),
        }
    }
}

impl CapabilityStatus {
    pub fn active(self) -> bool {
        matches!(self, Self::Supported)
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Supported => "supported",
            Self::Unsupported => "unsupported",
            Self::Denied => "denied",
            Self::Failed => "failed",
        }
    }

    fn encode(self) -> &'static str {
        match self {
            Self::Supported => "S",
            Self::Unsupported => "U",
            Self::Denied => "D",
            Self::Failed => "F",
        }
    }

    fn decode(value: &str) -> Result<Self, ProtocolError> {
        match value {
            "S" => Ok(Self::Supported),
            "U" => Ok(Self::Unsupported),
            "D" => Ok(Self::Denied),
            "F" => Ok(Self::Failed),
            _ => Err(ProtocolError::MalformedResponse),
        }
    }
}

fn ensure_end(mut parts: std::str::SplitAsciiWhitespace<'_>) -> Result<(), ProtocolError> {
    if parts.next().is_some() {
        Err(ProtocolError::ExtraTokens)
    } else {
        Ok(())
    }
}

fn encode_action(action: PokeAction) -> &'static str {
    match action {
        PokeAction::Honk => "HONK",
        PokeAction::Wander => "WANDER",
        PokeAction::Mud => "MUD",
        PokeAction::Meme => "MEME",
        PokeAction::Note => "NOTE",
        PokeAction::Nab => "NAB",
    }
}

fn decode_action(action: &str) -> Result<PokeAction, ProtocolError> {
    match action {
        "HONK" => Ok(PokeAction::Honk),
        "WANDER" => Ok(PokeAction::Wander),
        "MUD" => Ok(PokeAction::Mud),
        "MEME" => Ok(PokeAction::Meme),
        "NOTE" => Ok(PokeAction::Note),
        "NAB" => Ok(PokeAction::Nab),
        _ => Err(ProtocolError::UnknownAction),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trips_commands() {
        for command in [
            ControlCommand::Stop,
            ControlCommand::Reload,
            ControlCommand::Status,
            ControlCommand::Do(PokeAction::Honk),
            ControlCommand::Do(PokeAction::Wander),
            ControlCommand::Do(PokeAction::Mud),
            ControlCommand::Do(PokeAction::Meme),
            ControlCommand::Do(PokeAction::Note),
            ControlCommand::Do(PokeAction::Nab),
        ] {
            assert_eq!(
                ControlCommand::decode(command.encode().as_bytes()).unwrap(),
                command
            );
        }
    }

    #[test]
    fn rejects_malformed_commands() {
        assert_eq!(ControlCommand::decode(b""), Err(ProtocolError::Empty));
        assert_eq!(
            ControlCommand::decode(b"HONK299/1 STOP\n"),
            Err(ProtocolError::WrongVersion)
        );
        assert_eq!(
            ControlCommand::decode(b"HONK300/1 DO\n"),
            Err(ProtocolError::MissingAction)
        );
        assert_eq!(
            ControlCommand::decode(b"HONK300/1 DO YELL\n"),
            Err(ProtocolError::UnknownAction)
        );
        assert_eq!(
            ControlCommand::decode(b"HONK300/1 STOP NOW\n"),
            Err(ProtocolError::ExtraTokens)
        );
        assert_eq!(
            ControlCommand::decode(&[b'X'; MAX_FRAME_BYTES + 1]),
            Err(ProtocolError::TooLarge)
        );
    }

    #[test]
    fn maps_poke_outcomes_to_responses() {
        assert_eq!(
            ControlResponse::from(PokeOutcome::Applied),
            ControlResponse::Ok
        );
        assert_eq!(
            ControlResponse::from(PokeOutcome::Busy),
            ControlResponse::Err("BUSY".into())
        );
        assert_eq!(
            ControlResponse::from(PokeOutcome::Unsupported),
            ControlResponse::Err("UNSUPPORTED".into())
        );
    }

    #[test]
    fn round_trips_responses() {
        assert_eq!(
            ControlResponse::decode(ControlResponse::Ok.encode().as_bytes()).unwrap(),
            ControlResponse::Ok
        );
        assert_eq!(
            ControlResponse::decode(ControlResponse::Err("BUSY".into()).encode().as_bytes())
                .unwrap(),
            ControlResponse::Err("BUSY".into())
        );
    }

    #[test]
    fn round_trips_status_response_under_frame_limit() {
        let status = RuntimeStatus {
            running: true,
            platform: PlatformStatus::Macos,
            bundle: BundleStatus::App,
            accessibility: CapabilityStatus::Denied,
            cursor: CapabilityStatus::Denied,
            window: CapabilityStatus::Supported,
            collect: CapabilityStatus::Failed,
            presence: CapabilityStatus::Unsupported,
            audio: CapabilityStatus::Supported,
            notes: 12,
            memes: 34,
        };
        let frame = ControlResponse::Status(status).encode();
        assert!(frame.len() <= MAX_FRAME_BYTES, "{frame}");
        assert_eq!(
            ControlResponse::decode(frame.as_bytes()).unwrap(),
            ControlResponse::Status(status)
        );
    }

    #[test]
    fn rejects_malformed_status_response() {
        assert_eq!(
            ControlResponse::decode(b"STATUS G=1 P=MAC\n"),
            Err(ProtocolError::MalformedResponse)
        );
        assert_eq!(
            ControlResponse::decode(b"STATUS G=1 P=MAC B=APP A=X C=S W=S K=S R=S O=S N=1 M=2\n"),
            Err(ProtocolError::MalformedResponse)
        );
    }
}
