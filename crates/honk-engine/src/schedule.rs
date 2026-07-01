//! Platform-free schedule, presence, and season gates (M14).
//!
//! Platform runtimes sample local time and OS presence state, then feed snapshots into the
//! engine. The engine decides whether manners are active without depending on host APIs.

use crate::mood::LocalTime;

/// A minute within a local day, from 00:00 through 23:59.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct LocalMinute(u16);

impl LocalMinute {
    pub const MIN: Self = Self(0);
    pub const MAX: Self = Self(23 * 60 + 59);

    pub const fn new(hour: u8, minute: u8) -> Option<Self> {
        if hour < 24 && minute < 60 {
            Some(Self(hour as u16 * 60 + minute as u16))
        } else {
            None
        }
    }

    pub const fn from_minutes(minutes: u16) -> Option<Self> {
        if minutes < 24 * 60 {
            Some(Self(minutes))
        } else {
            None
        }
    }

    pub const fn minutes(self) -> u16 {
        self.0
    }
}

/// Platform-neutral interpretation of the user's interruption state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PresenceState {
    Available,
    DoNotDisturb,
    Fullscreen,
}

/// Presence state as reported by a platform backend.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PresenceSnapshot {
    pub supported: bool,
    pub state: PresenceState,
}

impl PresenceSnapshot {
    pub const fn available() -> Self {
        Self {
            supported: true,
            state: PresenceState::Available,
        }
    }

    pub const fn unsupported() -> Self {
        Self {
            supported: false,
            state: PresenceState::Available,
        }
    }

    pub const fn do_not_disturb() -> Self {
        Self {
            supported: true,
            state: PresenceState::DoNotDisturb,
        }
    }

    pub const fn fullscreen() -> Self {
        Self {
            supported: true,
            state: PresenceState::Fullscreen,
        }
    }
}

impl Default for PresenceSnapshot {
    fn default() -> Self {
        Self::unsupported()
    }
}

/// Runtime schedule settings consumed by the platform-free world.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ScheduleOptions {
    pub quiet_hours_enabled: bool,
    pub quiet_start: LocalMinute,
    pub quiet_end: LocalMinute,
    pub dnd_respect: bool,
    pub pause_on_fullscreen: bool,
    pub seasonal: bool,
    pub autumn: bool,
}

impl Default for ScheduleOptions {
    fn default() -> Self {
        Self {
            quiet_hours_enabled: true,
            quiet_start: LocalMinute::new(22, 0).expect("valid default quiet start"),
            quiet_end: LocalMinute::new(8, 0).expect("valid default quiet end"),
            dnd_respect: true,
            pause_on_fullscreen: true,
            seasonal: true,
            autumn: true,
        }
    }
}

impl ScheduleOptions {
    /// Whether the configured quiet window contains the provided local time.
    ///
    /// The start minute is inclusive and the end minute is exclusive. A matching start and end
    /// means "no quiet window" rather than all day.
    pub fn quiet_hours_active(self, local_time: Option<LocalTime>) -> bool {
        if !self.quiet_hours_enabled || self.quiet_start == self.quiet_end {
            return false;
        }
        let Some(local_time) = local_time else {
            return false;
        };
        let now = local_time.local_minute();
        if self.quiet_start < self.quiet_end {
            now >= self.quiet_start && now < self.quiet_end
        } else {
            now >= self.quiet_start || now < self.quiet_end
        }
    }

    /// Whether OS presence should put the goose into calm suppression.
    pub fn presence_active(self, presence: PresenceSnapshot) -> bool {
        if !presence.supported {
            return false;
        }
        match presence.state {
            PresenceState::Available => false,
            PresenceState::DoNotDisturb => self.dnd_respect,
            PresenceState::Fullscreen => self.pause_on_fullscreen,
        }
    }

    /// Whether any schedule/presence manners are currently active.
    pub fn manners_active(self, local_time: Option<LocalTime>, presence: PresenceSnapshot) -> bool {
        self.quiet_hours_active(local_time) || self.presence_active(presence)
    }

    /// Whether built-in Autumn should be active for the provided local date.
    pub fn autumn_active(self, local_time: Option<LocalTime>) -> bool {
        self.seasonal
            && self.autumn
            && local_time
                .and_then(LocalTime::month)
                .is_some_and(|month| (9..=11).contains(&month))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn local(hour: u8, minute: u8, month: u8) -> LocalTime {
        LocalTime {
            day: 20260000 + month as i32 * 100 + 15,
            hour,
            minute,
            second: 0,
        }
    }

    #[test]
    fn quiet_hours_support_overnight_windows() {
        let schedule = ScheduleOptions::default();
        assert!(schedule.quiet_hours_active(Some(local(22, 0, 6))));
        assert!(schedule.quiet_hours_active(Some(local(7, 59, 6))));
        assert!(!schedule.quiet_hours_active(Some(local(8, 0, 6))));
        assert!(!schedule.quiet_hours_active(Some(local(21, 59, 6))));
    }

    #[test]
    fn matching_quiet_start_and_end_is_not_all_day() {
        let schedule = ScheduleOptions {
            quiet_start: LocalMinute::new(8, 0).unwrap(),
            quiet_end: LocalMinute::new(8, 0).unwrap(),
            ..ScheduleOptions::default()
        };
        assert!(!schedule.quiet_hours_active(Some(local(8, 0, 6))));
        assert!(!schedule.quiet_hours_active(Some(local(23, 0, 6))));
    }

    #[test]
    fn presence_respects_separate_dnd_and_fullscreen_toggles() {
        let schedule = ScheduleOptions {
            dnd_respect: false,
            pause_on_fullscreen: true,
            ..ScheduleOptions::default()
        };
        assert!(!schedule.presence_active(PresenceSnapshot::do_not_disturb()));
        assert!(schedule.presence_active(PresenceSnapshot::fullscreen()));

        let schedule = ScheduleOptions {
            dnd_respect: true,
            pause_on_fullscreen: false,
            ..ScheduleOptions::default()
        };
        assert!(schedule.presence_active(PresenceSnapshot::do_not_disturb()));
        assert!(!schedule.presence_active(PresenceSnapshot::fullscreen()));
    }

    #[test]
    fn autumn_uses_meteorological_window() {
        let schedule = ScheduleOptions::default();
        assert!(!schedule.autumn_active(Some(local(12, 0, 8))));
        assert!(schedule.autumn_active(Some(local(12, 0, 9))));
        assert!(schedule.autumn_active(Some(local(12, 0, 11))));
        assert!(!schedule.autumn_active(Some(local(12, 0, 12))));
    }
}
