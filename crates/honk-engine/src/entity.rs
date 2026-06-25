//! The goose entity: its tunable parameter table and per-frame state.
//!
//! Constants are ported **verbatim** from the verified source
//! (`GooseModdingAPI/Exports.cs`, `GooseEntity.ParametersTable`). Do not "tidy" these
//! values — they are tuned to the 120 Hz tick and pinned by tests.

use crate::footmarks::FootMarks;
use crate::math::Vec2;
use crate::rig::Rig;

/// The three speed tiers the goose moves at.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpeedTier {
    Walk,
    Run,
    Charge,
}

/// Maximum speeds, accelerations, and timings. Verified values from `Exports.cs`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ParametersTable {
    /// Max speed in the Walk tier.
    pub walk_speed: f32,
    /// Max speed in the Run tier.
    pub run_speed: f32,
    /// Max speed in the Charge tier.
    pub charge_speed: f32,
    /// Acceleration in the Walk and Run tiers.
    pub acceleration_normal: f32,
    /// Acceleration in the Charge tier.
    pub acceleration_charged: f32,
    /// Stop tolerance radius (negative in the original).
    pub stop_radius: f32,
    /// Foot-step interval in the Walk and Run tiers (seconds).
    pub step_time_normal: f32,
    /// Foot-step interval in the Charge tier (seconds).
    pub step_time_charged: f32,
    /// Seconds the goose leaves muddy prints after stepping in mud.
    pub duration_to_track_mud: f32,
}

impl Default for ParametersTable {
    fn default() -> Self {
        Self {
            walk_speed: 80.0,
            run_speed: 200.0,
            charge_speed: 400.0,
            acceleration_normal: 1300.0,
            acceleration_charged: 2300.0,
            stop_radius: -10.0,
            step_time_normal: 0.2,
            step_time_charged: 0.1,
            duration_to_track_mud: 15.0,
        }
    }
}

impl ParametersTable {
    /// Max speed for a given tier.
    pub fn speed(&self, tier: SpeedTier) -> f32 {
        match tier {
            SpeedTier::Walk => self.walk_speed,
            SpeedTier::Run => self.run_speed,
            SpeedTier::Charge => self.charge_speed,
        }
    }

    /// Acceleration for a given tier (Charge is faster).
    pub fn acceleration(&self, tier: SpeedTier) -> f32 {
        match tier {
            SpeedTier::Charge => self.acceleration_charged,
            _ => self.acceleration_normal,
        }
    }

    /// Step interval for a given tier (Charge is quicker).
    pub fn step_time(&self, tier: SpeedTier) -> f32 {
        match tier {
            SpeedTier::Charge => self.step_time_charged,
            _ => self.step_time_normal,
        }
    }
}

/// The full mutable state of the goose. Defaults mirror the original's field
/// initializers (position/target `(300, 300)`, `direction` 90°, etc.).
///
/// The engine auto-locomotes toward `target_pos`; tasks (M4+) only set targets and
/// acceleration. The locomotion integration itself lands in M2 — M0 owns the state
/// shape, the parameter table, and the rig.
#[derive(Debug, Clone)]
pub struct GooseEntity {
    /// Current position (rig origin / shadow point).
    pub position: Vec2,
    /// Current velocity.
    pub velocity: Vec2,
    /// Facing direction in degrees.
    pub direction: f32,
    /// Desired facing as a unit vector.
    pub target_direction: Vec2,
    /// Whether the neck is forced extended this frame (resets each tick).
    pub extending_neck: bool,
    /// Auto-locomotion target.
    pub target_pos: Vec2,
    /// Current max speed (set from a tier).
    pub current_speed: f32,
    /// Current acceleration magnitude.
    pub current_acceleration: f32,
    /// Foot-step interval in seconds.
    pub step_interval: f32,
    /// Whether the goose stops sharply at its target or drifts around it.
    pub can_decelerate_immediately: bool,
    /// Wall-clock time at which mud-tracking ends (`< 0` ⇒ not tracking).
    pub track_mud_end_time: f32,
    /// Muddy footprints left behind.
    pub foot_marks: FootMarks,
    /// Computed body geometry for rendering.
    pub rig: Rig,
    /// Tunable parameters.
    pub parameters: ParametersTable,
}

impl Default for GooseEntity {
    fn default() -> Self {
        Self {
            position: Vec2::new(300.0, 300.0),
            velocity: Vec2::ZERO,
            direction: 90.0,
            target_direction: Vec2::ZERO,
            extending_neck: false,
            target_pos: Vec2::new(300.0, 300.0),
            current_speed: 0.0,
            current_acceleration: 0.0,
            step_interval: 0.0,
            can_decelerate_immediately: true,
            track_mud_end_time: -1.0,
            foot_marks: FootMarks::new(),
            rig: Rig::default(),
            parameters: ParametersTable::default(),
        }
    }
}

impl GooseEntity {
    /// A goose with default state and parameters.
    pub fn new() -> Self {
        Self::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parameter_table_matches_verified_source() {
        let p = ParametersTable::default();
        assert_eq!(p.walk_speed, 80.0);
        assert_eq!(p.run_speed, 200.0);
        assert_eq!(p.charge_speed, 400.0);
        assert_eq!(p.acceleration_normal, 1300.0);
        assert_eq!(p.acceleration_charged, 2300.0);
        assert_eq!(p.stop_radius, -10.0);
        assert_eq!(p.step_time_normal, 0.2);
        assert_eq!(p.step_time_charged, 0.1);
        assert_eq!(p.duration_to_track_mud, 15.0);
    }

    #[test]
    fn tier_lookups() {
        let p = ParametersTable::default();
        assert_eq!(p.speed(SpeedTier::Walk), 80.0);
        assert_eq!(p.speed(SpeedTier::Run), 200.0);
        assert_eq!(p.speed(SpeedTier::Charge), 400.0);
        assert_eq!(p.acceleration(SpeedTier::Walk), 1300.0);
        assert_eq!(p.acceleration(SpeedTier::Charge), 2300.0);
        assert_eq!(p.step_time(SpeedTier::Run), 0.2);
        assert_eq!(p.step_time(SpeedTier::Charge), 0.1);
    }

    #[test]
    fn entity_defaults_match_original_initializers() {
        let g = GooseEntity::new();
        assert_eq!(g.position, Vec2::new(300.0, 300.0));
        assert_eq!(g.target_pos, Vec2::new(300.0, 300.0));
        assert_eq!(g.direction, 90.0);
        assert!(g.can_decelerate_immediately);
        assert_eq!(g.track_mud_end_time, -1.0);
        assert_eq!(g.velocity, Vec2::ZERO);
    }
}
