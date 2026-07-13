//! Procedural feet — the stateful plant-and-swing walking gait.
//!
//! Frozen compatibility constants: `feetDistanceApart = 6`, `wantStepAtDistance = 5`,
//! `overshootFraction = 0.4`. [`FeetState`] keeps each foot **planted** in world space
//! and swings it forward (with overshoot) only once it lags too far behind its home
//! under the body. A planted foot never slides: that is the invariant the old stateless
//! sine gait broke (feet "moonwalked" along with the body) and the one pinned by test.
//!
//! `FeetState` lives on the entity across ticks (`GooseEntity::anim`). Each tick it is
//! fed the body's ground point, facing, and velocity; it decides when each foot steps,
//! animates the swing (lift + overshoot), and reports plant events so the world can
//! stamp muddy footprints exactly where a foot actually lands.

use crate::math::Vec2;

/// Lateral spacing between the two feet.
pub const FEET_DISTANCE_APART: f32 = 6.0;
/// How far a foot lags behind its home before it wants to step.
pub const WANT_STEP_AT_DISTANCE: f32 = 5.0;
/// Fraction of the lag distance a stepping foot overshoots by.
pub const OVERSHOOT_FRACTION: f32 = 0.4;

/// Resting fore/aft stagger so the two feet read separately when standing.
const STANCE_STAGGER: f32 = 2.0;
/// Maximum foot lift (pixels) at the top of a swing.
const MAX_LIFT: f32 = 3.5;
/// A foot farther than this from home snaps instead of swinging (teleports/FirstUX).
const SNAP_DISTANCE: f32 = 60.0;
/// Fallback swing duration when the entity has no step interval yet.
const DEFAULT_STEP_TIME: f32 = 0.2;
/// Visual refinement over the compatibility trigger: start the recovery step slightly sooner.
const GAIT_STEP_TRIGGER_DISTANCE: f32 = WANT_STEP_AT_DISTANCE * 0.8;
/// The configured step interval is the gait beat; the airborne recovery occupies only this share.
/// This preserves planted feet while preventing the opposite leg from trailing into a long rubbery
/// stretch before it is allowed to recover.
const GAIT_SWING_DURATION_SCALE: f32 = 0.7;
/// At run/charge speed, cap how far the body can travel during one foot's recovery.
/// Normal/moderate motion (up through 120 px/s × 0.14 s = 16.8 px) stays exactly on the weighted
/// existing cadence; only the Run and Charge tiers shorten their airborne recovery enough to
/// avoid rubber-leg trailing without turning into twitchy bicycle steps.
const MAX_BODY_TRAVEL_DURING_SWING: f32 = 18.0;

const UP: Vec2 = Vec2 { x: 0.0, y: -1.0 };

/// The two foot ground positions (world space) — the compact view used by footmarks,
/// bounding boxes, and tests.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Feet {
    pub left: Vec2,
    pub right: Vec2,
}

/// One foot's full render pose: ground position, current lift (0..1 of [`MAX_LIFT`]
/// pixels, already applied to nothing — the renderer applies it), the direction the
/// foot points, and whether it is mid-swing.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FootPose {
    /// Ground-projected world position of the foot.
    pub pos: Vec2,
    /// Screen-space lift offset (pixels, applied along screen-up by the renderer).
    pub lift: f32,
    /// Unit direction the foot points (swing direction, else body forward).
    pub heading: Vec2,
    /// True while the foot is airborne.
    pub swinging: bool,
}

#[derive(Debug, Clone, PartialEq)]
struct Swing {
    from: Vec2,
    to: Vec2,
    /// Progress 0..1.
    t: f32,
    duration: f32,
}

#[derive(Debug, Clone, PartialEq)]
struct Foot {
    pos: Vec2,
    heading: Vec2,
    swing: Option<Swing>,
}

impl Foot {
    fn planted(pos: Vec2, heading: Vec2) -> Self {
        Self {
            pos,
            heading,
            swing: None,
        }
    }

    fn lift(&self) -> f32 {
        match &self.swing {
            Some(s) => (s.t * std::f32::consts::PI).sin() * MAX_LIFT,
            None => 0.0,
        }
    }

    fn pose(&self) -> FootPose {
        FootPose {
            pos: self.pos,
            lift: self.lift(),
            heading: self.heading,
            swinging: self.swing.is_some(),
        }
    }
}

/// Persistent plant-and-swing state for both feet.
#[derive(Debug, Clone, PartialEq)]
pub struct FeetState {
    left: Foot,
    right: Foot,
    /// Feet planted since the last drain (world stamps footmarks here).
    plants: Vec<Vec2>,
}

impl FeetState {
    /// Both feet planted at their home positions under `center`, facing `forward`.
    pub fn new(center: Vec2, forward: Vec2) -> Self {
        let (home_l, home_r) = homes(center, forward);
        Self {
            left: Foot::planted(home_l, forward),
            right: Foot::planted(home_r, forward),
            plants: Vec::new(),
        }
    }

    /// Advance the gait one tick. `center` is the entity ground point, `forward` the
    /// facing unit vector, `velocity` the current velocity (px/s), and `step_time` the
    /// entity's per-step interval (`<= 0` falls back to the walk default).
    pub fn tick(&mut self, dt: f32, center: Vec2, forward: Vec2, velocity: Vec2, step_time: f32) {
        let beat = if step_time > 1e-3 {
            step_time
        } else {
            DEFAULT_STEP_TIME
        };
        let weighted_duration = beat * GAIT_SWING_DURATION_SCALE;
        let speed = velocity.magnitude();
        let duration = if speed > 1e-3 {
            weighted_duration.min(MAX_BODY_TRAVEL_DURING_SWING / speed)
        } else {
            weighted_duration
        };
        let (home_l, home_r) = homes(center, forward);

        // Teleport guard: far past any plausible stride, both feet snap home.
        if Vec2::distance(self.left.pos, home_l) > SNAP_DISTANCE
            || Vec2::distance(self.right.pos, home_r) > SNAP_DISTANCE
        {
            self.left = Foot::planted(home_l, forward);
            self.right = Foot::planted(home_r, forward);
            return;
        }

        // Advance any active swing.
        for (foot, home) in [(&mut self.left, home_l), (&mut self.right, home_r)] {
            let _ = home;
            if let Some(swing) = &mut foot.swing {
                swing.t = (swing.t + dt / swing.duration).min(1.0);
                let eased = smoothstep(swing.t);
                foot.pos = Vec2::lerp(swing.from, swing.to, eased);
                let dir = swing.to - swing.from;
                if dir.magnitude() > 1e-3 {
                    foot.heading = dir.normalize();
                }
                if swing.t >= 1.0 {
                    foot.swing = None;
                    self.plants.push(foot.pos);
                }
            }
        }

        // Trigger the next step: one foot in the air at a time, farthest-lagging first.
        let airborne = self.left.swing.is_some() || self.right.swing.is_some();
        if !airborne {
            let lag_l = Vec2::distance(self.left.pos, home_l);
            let lag_r = Vec2::distance(self.right.pos, home_r);
            let (foot, home, lag) = if lag_l >= lag_r {
                (&mut self.left, home_l, lag_l)
            } else {
                (&mut self.right, home_r, lag_r)
            };
            if lag > GAIT_STEP_TRIGGER_DISTANCE {
                // Aim past home (overshoot) and lead a moving body so the foot lands
                // where the home will roughly be, not where it was.
                let dir = (home - foot.pos).normalize();
                let target = home + dir * (lag * OVERSHOOT_FRACTION) + velocity * (duration * 0.5);
                foot.swing = Some(Swing {
                    from: foot.pos,
                    to: target,
                    t: 0.0,
                    duration,
                });
            }
        }
    }

    /// Ground positions of both feet.
    pub fn positions(&self) -> Feet {
        Feet {
            left: self.left.pos,
            right: self.right.pos,
        }
    }

    /// Full render poses, `[left, right]`.
    pub fn poses(&self) -> [FootPose; 2] {
        [self.left.pose(), self.right.pose()]
    }

    /// 0..1 progress of the currently swinging foot (0 when both planted). Drives the
    /// body bob so it rises mid-step.
    pub fn swing_progress(&self) -> f32 {
        [&self.left, &self.right]
            .iter()
            .filter_map(|f| f.swing.as_ref().map(|s| s.t))
            .fold(0.0, f32::max)
    }

    /// Drain foot-plant events accumulated since the last call (world → footmarks).
    pub fn drain_plants(&mut self, mut f: impl FnMut(Vec2)) {
        for p in self.plants.drain(..) {
            f(p);
        }
    }

    /// Screen-up vector shared with the renderer (feet lift along this).
    pub const fn up() -> Vec2 {
        UP
    }
}

/// Home (rest) positions for both feet under `center`.
fn homes(center: Vec2, forward: Vec2) -> (Vec2, Vec2) {
    let across = forward.perpendicular() * (FEET_DISTANCE_APART * 0.5);
    let stagger = forward * STANCE_STAGGER;
    (center + across + stagger, center - across - stagger)
}

fn smoothstep(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entity::{ParametersTable, SpeedTier};
    use crate::time::DT;

    #[derive(Debug, Clone, Copy)]
    struct GaitSample {
        max_planted_lag: f32,
        first_swing_ticks: usize,
        plants: usize,
    }

    fn sample_straight_gait(speed: f32, step_time: f32) -> GaitSample {
        let forward = Vec2::new(1.0, 0.0);
        let velocity = forward * speed;
        let mut center = Vec2::ZERO;
        let mut state = FeetState::new(center, forward);
        let mut max_planted_lag: f32 = 0.0;
        let mut first_swing = None;
        let mut first_swing_ticks = 0;
        let mut plants = 0;

        for _ in 0..(120 * 3) {
            center = center + velocity * DT;
            state.tick(DT, center, forward, velocity, step_time);
            let poses = state.poses();
            let (left_home, right_home) = homes(center, forward);
            for (index, (pose, home)) in poses.into_iter().zip([left_home, right_home]).enumerate()
            {
                if !pose.swinging {
                    max_planted_lag = max_planted_lag.max(Vec2::distance(pose.pos, home));
                }
                if first_swing.is_none() && first_swing_ticks == 0 && pose.swinging {
                    first_swing = Some(index);
                }
            }
            if let Some(index) = first_swing {
                first_swing_ticks += 1;
                if !state.poses()[index].swinging {
                    first_swing = None;
                }
            }
            state.drain_plants(|_| plants += 1);
        }

        GaitSample {
            max_planted_lag,
            first_swing_ticks,
            plants,
        }
    }

    #[test]
    fn constants_match_verified_source() {
        assert_eq!(FEET_DISTANCE_APART, 6.0);
        assert_eq!(WANT_STEP_AT_DISTANCE, 5.0);
        assert_eq!(OVERSHOOT_FRACTION, 0.4);
        assert_eq!(GAIT_STEP_TRIGGER_DISTANCE, 4.0);
        assert_eq!(GAIT_SWING_DURATION_SCALE, 0.7);
    }

    #[test]
    fn feet_start_planted_at_homes() {
        let f = FeetState::new(Vec2::new(100.0, 100.0), Vec2::new(1.0, 0.0));
        let feet = f.positions();
        assert!(Vec2::distance(feet.left, feet.right) >= FEET_DISTANCE_APART - 1e-3);
        assert_eq!(f.swing_progress(), 0.0);
    }

    /// The no-foot-slide invariant: while the body moves, a planted (non-swinging)
    /// foot's world position must not change.
    #[test]
    fn planted_feet_never_slide() {
        let forward = Vec2::new(1.0, 0.0);
        let mut center = Vec2::new(100.0, 100.0);
        let velocity = forward * 80.0;
        let mut state = FeetState::new(center, forward);
        let mut prev = state.poses();
        for _ in 0..240 {
            center = center + velocity * DT;
            state.tick(DT, center, forward, velocity, 0.2);
            let now = state.poses();
            for (before, after) in prev.iter().zip(now.iter()) {
                if !before.swinging && !after.swinging {
                    assert!(
                        Vec2::distance(before.pos, after.pos) < 1e-4,
                        "planted foot slid from {:?} to {:?}",
                        before.pos,
                        after.pos
                    );
                }
            }
            prev = now;
        }
    }

    #[test]
    fn walking_alternates_steps_and_reports_plants() {
        let forward = Vec2::new(1.0, 0.0);
        let mut center = Vec2::new(0.0, 0.0);
        let velocity = forward * 80.0;
        let mut state = FeetState::new(center, forward);
        let mut plants = Vec::new();
        for _ in 0..(120 * 3) {
            center = center + velocity * DT;
            state.tick(DT, center, forward, velocity, 0.2);
            state.drain_plants(|p| plants.push(p));
        }
        // Three seconds of walking must produce a steady stream of steps…
        assert!(plants.len() >= 6, "too few steps: {}", plants.len());
        // …that make forward progress.
        assert!(plants.windows(2).all(|w| w[1].x > w[0].x - 1.0));
        // Feet end up near the body, not lagging behind.
        let feet = state.positions();
        assert!(Vec2::distance(feet.left, center) < 30.0);
        assert!(Vec2::distance(feet.right, center) < 30.0);
    }

    #[test]
    fn standing_goose_keeps_feet_still() {
        let forward = Vec2::new(0.0, 1.0);
        let center = Vec2::new(50.0, 50.0);
        let mut state = FeetState::new(center, forward);
        let before = state.positions();
        for _ in 0..120 {
            state.tick(DT, center, forward, Vec2::ZERO, 0.2);
        }
        assert_eq!(state.positions(), before);
        assert_eq!(state.swing_progress(), 0.0);
    }

    #[test]
    fn teleport_snaps_feet_home() {
        let forward = Vec2::new(1.0, 0.0);
        let mut state = FeetState::new(Vec2::ZERO, forward);
        let far = Vec2::new(500.0, 500.0);
        state.tick(DT, far, forward, Vec2::ZERO, 0.2);
        let feet = state.positions();
        assert!(Vec2::distance(feet.left, far) < FEET_DISTANCE_APART + STANCE_STAGGER + 1.0);
        assert!(Vec2::distance(feet.right, far) < FEET_DISTANCE_APART + STANCE_STAGGER + 1.0);
    }

    #[test]
    fn swinging_foot_lifts_and_lands_with_overshoot() {
        let forward = Vec2::new(1.0, 0.0);
        let mut center = Vec2::new(0.0, 0.0);
        let velocity = forward * 80.0;
        let mut state = FeetState::new(center, forward);
        let mut max_lift: f32 = 0.0;
        for _ in 0..120 {
            center = center + velocity * DT;
            state.tick(DT, center, forward, velocity, 0.2);
            for pose in state.poses() {
                max_lift = max_lift.max(pose.lift);
            }
        }
        assert!(max_lift > 1.0, "feet never lifted (max {max_lift})");
    }

    #[test]
    fn walking_planted_foot_does_not_trail_far_enough_to_read_as_stretched() {
        let forward = Vec2::new(1.0, 0.0);
        let mut center = Vec2::ZERO;
        let velocity = forward * 80.0;
        let mut state = FeetState::new(center, forward);
        let mut max_planted_lag: f32 = 0.0;

        for _ in 0..(120 * 3) {
            center = center + velocity * DT;
            state.tick(DT, center, forward, velocity, 0.2);
            let (left_home, right_home) = homes(center, forward);
            for (pose, home) in state.poses().into_iter().zip([left_home, right_home]) {
                if !pose.swinging {
                    max_planted_lag = max_planted_lag.max(Vec2::distance(pose.pos, home));
                }
            }
        }

        assert!(
            max_planted_lag <= 16.0,
            "planted foot trailed {max_planted_lag:.2}px behind home"
        );
    }

    #[test]
    fn actual_speed_tiers_bound_planted_leg_trailing() {
        let parameters = ParametersTable::default();
        for (tier, max_lag) in [
            (SpeedTier::Walk, 16.0_f32),
            (SpeedTier::Run, 26.0),
            (SpeedTier::Charge, 26.0),
        ] {
            let sample = sample_straight_gait(parameters.speed(tier), parameters.step_time(tier));
            assert!(
                sample.max_planted_lag <= max_lag,
                "{tier:?} planted foot trailed {:.2}px (limit {max_lag:.2}px)",
                sample.max_planted_lag
            );
        }
    }

    #[test]
    fn actual_speed_tiers_keep_visible_but_bounded_swing_airtime() {
        let parameters = ParametersTable::default();
        for (tier, min_ticks, max_ticks, max_plants) in [
            (SpeedTier::Walk, 15_usize, 18_usize, 24_usize),
            (SpeedTier::Run, 10, 12, 36),
            (SpeedTier::Charge, 6, 8, 65),
        ] {
            let sample = sample_straight_gait(parameters.speed(tier), parameters.step_time(tier));
            assert!(
                (min_ticks..=max_ticks).contains(&sample.first_swing_ticks),
                "{tier:?} first swing took {} ticks (expected {min_ticks}..={max_ticks})",
                sample.first_swing_ticks
            );
            assert!(
                sample.plants <= max_plants,
                "{tier:?} planted {} times in three seconds (limit {max_plants})",
                sample.plants
            );
        }
    }

    #[test]
    fn moderate_directional_motion_keeps_the_weighted_walk_airtime() {
        let sample = sample_straight_gait(120.0, ParametersTable::default().step_time_normal);

        assert_eq!(sample.first_swing_ticks, 18);
    }
}
