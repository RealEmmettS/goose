//! Independent manners observations without changing the legacy STATUS frame.
use crate::{CapabilityStatus, ProtocolError};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PresenceStatus {
    pub fullscreen: CapabilityStatus,
    pub dnd: CapabilityStatus,
}

impl PresenceStatus {
    pub const fn unprobed() -> Self {
        Self {
            fullscreen: CapabilityStatus::Unprobed,
            dnd: CapabilityStatus::Unprobed,
        }
    }

    pub const fn unsupported() -> Self {
        Self {
            fullscreen: CapabilityStatus::Unsupported,
            dnd: CapabilityStatus::Unsupported,
        }
    }

    pub fn aggregate(self) -> CapabilityStatus {
        combine_capabilities([self.fullscreen, self.dnd])
    }

    pub(crate) fn encode(self) -> String {
        format!(
            "PRESENCE {} {}\n",
            self.fullscreen.encode(),
            self.dnd.encode()
        )
    }

    pub(crate) fn decode(
        mut parts: std::str::SplitAsciiWhitespace<'_>,
    ) -> Result<Self, ProtocolError> {
        let mut next =
            || CapabilityStatus::decode(parts.next().ok_or(ProtocolError::MalformedResponse)?);
        let value = Self {
            fullscreen: next()?,
            dnd: next()?,
        };
        if parts.next().is_some() {
            return Err(ProtocolError::MalformedResponse);
        }
        Ok(value)
    }
}

/// A supported independent provider wins; otherwise preserve an actionable loss.
pub fn combine_capabilities<const N: usize>(values: [CapabilityStatus; N]) -> CapabilityStatus {
    use CapabilityStatus::*;
    [Supported, Denied, Failed, Unprobed, Unsupported]
        .into_iter()
        .find(|state| values.contains(state))
        .unwrap_or(Unsupported)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ControlCommand, ControlResponse};

    #[test]
    fn independent_observations_round_trip_without_accepting_partial_or_extra_fields() {
        let command = ControlCommand::PresenceStatus;
        assert_eq!(
            ControlCommand::decode(command.encode().as_bytes()).unwrap(),
            command
        );
        for fullscreen in [
            CapabilityStatus::Supported,
            CapabilityStatus::Denied,
            CapabilityStatus::Failed,
            CapabilityStatus::Unprobed,
            CapabilityStatus::Unsupported,
        ] {
            let reply = ControlResponse::Presence(PresenceStatus {
                fullscreen,
                dnd: CapabilityStatus::Unsupported,
            });
            assert!(reply.encode().len() <= 128);
            assert_eq!(
                ControlResponse::decode(reply.encode().as_bytes()).unwrap(),
                reply
            );
        }
        for bad in ["PRESENCE", "PRESENCE S", "PRESENCE S U X", "PRESENCE X U"] {
            assert!(ControlResponse::decode(bad.as_bytes()).is_err());
        }
        assert_eq!(
            PresenceStatus {
                fullscreen: CapabilityStatus::Supported,
                dnd: CapabilityStatus::Unsupported
            }
            .aggregate(),
            CapabilityStatus::Supported
        );
    }
}
