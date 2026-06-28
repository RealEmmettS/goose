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
    Do(PokeAction),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ControlResponse {
    Ok,
    Err(String),
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
}
