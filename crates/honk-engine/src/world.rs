//! The simulation world: owns the goose and advances it one fixed tick at a time.
//!
//! M2 ships a minimal autonomous **roam** driver — walk to a random on-screen point,
//! pause briefly, repeat — purely so the overlay has something correct-speed to show.
//! It is an explicit stand-in for the real task/AI state machine (wander, mud, nab, …)
//! that arrives in M4; keep behaviour here intentionally thin.

use crate::entity::GooseEntity;
use crate::locomotion;
use crate::math::{Rect, Vec2};
use crate::rig::Rig;
use crate::rng::{RandomSource, SplitMix64};
use crate::time::DT;

/// Distance travelled per full walking-gait cycle (radians of `gait_phase` per `TAU`).
const GAIT_CYCLE_DISTANCE: f32 = 22.0;

#[derive(Debug, Clone, Copy)]
enum Roam {
    Moving,
    Pausing { until: f32 },
}

/// The whole simulation: one goose roaming within `bounds` (the virtual-desktop space).
pub struct World {
    pub goose: GooseEntity,
    pub bounds: Rect,
    rng: SplitMix64,
    roam: Roam,
    elapsed: f32,
    /// Index of the last gait half-step a footmark was considered for (one per foot plant).
    last_step: i64,
}

impl World {
    /// A world bounded by `bounds`, with the goose centred and the roam driver primed to
    /// pick its first target on the first tick. `seed` makes the roam fully deterministic.
    pub fn new(bounds: Rect, seed: u64) -> Self {
        let center = (bounds.min + bounds.max) * 0.5;
        let mut goose = GooseEntity::new();
        goose.position = center;
        goose.target_pos = center;
        goose.current_speed = goose.parameters.walk_speed;
        goose.current_acceleration = goose.parameters.acceleration_normal;
        goose.rig = Rig::update(center, goose.direction, 0.0, 0.0);
        Self {
            goose,
            bounds,
            rng: SplitMix64::seed(seed),
            roam: Roam::Pausing { until: 0.0 }, // → picks a target on tick 1
            elapsed: 0.0,
            last_step: 0,
        }
    }

    /// The world's monotonic clock (seconds), the time base for footmark fade.
    pub fn now(&self) -> f32 {
        self.elapsed
    }

    fn arrived(&self) -> bool {
        Vec2::distance(self.goose.position, self.goose.target_pos) < 1.0
    }

    fn pick_new_target(&mut self) {
        let x = self.rng.range(self.bounds.min.x, self.bounds.max.x);
        let y = self.rng.range(self.bounds.min.y, self.bounds.max.y);
        self.goose.target_pos = Vec2::new(x, y);
        // M3 demo: sometimes the goose "steps in mud" and tracks prints for the next
        // DurationToTrackMud seconds. M4's Task_TrackMud formalises this trigger.
        if self.rng.next_f64() < 0.6 {
            self.goose.track_mud_end_time =
                self.elapsed + self.goose.parameters.duration_to_track_mud;
        }
    }

    /// Advance the world by one fixed [`DT`] tick.
    pub fn tick(&mut self) {
        self.elapsed += DT;

        match self.roam {
            Roam::Pausing { until } if self.elapsed >= until => {
                self.pick_new_target();
                self.roam = Roam::Moving;
            }
            Roam::Moving if self.arrived() => {
                let pause = self.rng.range(0.3, 1.2);
                self.roam = Roam::Pausing {
                    until: self.elapsed + pause,
                };
            }
            _ => {}
        }

        let before = self.goose.position;
        locomotion::step(&mut self.goose, DT);

        // Advance the walking gait by the distance travelled (so a stopped goose stands
        // still); one full waddle cycle per GAIT_CYCLE_DISTANCE of travel.
        let moved = Vec2::distance(before, self.goose.position);
        self.goose.gait_phase += moved * (std::f32::consts::TAU / GAIT_CYCLE_DISTANCE);

        // Keep the rig in sync for rendering; a touch of neck reach while moving fast
        // (cosmetic only — real posture/mood modulation is M13).
        let speed_frac =
            (self.goose.velocity.magnitude() / self.goose.parameters.walk_speed).min(1.0);
        self.goose.rig = Rig::update(
            self.goose.position,
            self.goose.direction,
            speed_frac * 0.4,
            self.goose.gait_phase,
        );

        // Drop a fading muddy print at each foot-plant (half gait cycle) while the goose
        // is tracking mud. Footmarks fade on their own (8.5 s life / 1 s shrink).
        let step = (self.goose.gait_phase / std::f32::consts::PI).floor() as i64;
        if step > self.last_step {
            if self.elapsed < self.goose.track_mud_end_time {
                let foot = if step % 2 == 0 {
                    self.goose.rig.feet.left
                } else {
                    self.goose.rig.feet.right
                };
                self.goose.foot_marks.add(foot, self.elapsed);
            }
            self.last_step = step;
        }
    }

    /// The current rig, for the renderer.
    pub fn rig(&self) -> &Rig {
        &self.goose.rig
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn bounds() -> Rect {
        Rect {
            min: Vec2::new(0.0, 0.0),
            max: Vec2::new(1920.0, 1080.0),
        }
    }

    #[test]
    fn goose_moves_over_time() {
        let mut w = World::new(bounds(), 1);
        let start = w.goose.position;
        for _ in 0..240 {
            w.tick();
        }
        assert!(
            Vec2::distance(start, w.goose.position) > 1.0,
            "the goose should have roamed away from centre"
        );
    }

    #[test]
    fn goose_stays_in_bounds() {
        let mut w = World::new(bounds(), 2);
        for _ in 0..5_000 {
            w.tick();
            let p = w.goose.position;
            // Targets are in-bounds and the goose stops on arrival, so it never leaves.
            assert!(p.x >= -1.0 && p.x <= 1921.0 && p.y >= -1.0 && p.y <= 1081.0);
        }
    }

    #[test]
    fn roam_is_deterministic_for_seed() {
        let mut a = World::new(bounds(), 42);
        let mut b = World::new(bounds(), 42);
        for _ in 0..1_000 {
            a.tick();
            b.tick();
        }
        assert_eq!(a.goose.position, b.goose.position);
    }

    #[test]
    fn tracks_mud_and_drops_fading_prints() {
        let mut w = World::new(bounds(), 5);
        w.goose.track_mud_end_time = 1_000.0; // force mud-tracking on
        for _ in 0..1_200 {
            w.tick();
        }
        // Walking in mud leaves prints.
        assert!(
            w.goose.foot_marks.alive_count(w.now()) > 0,
            "expected muddy prints while tracking mud"
        );
        // With mud-tracking off and enough time elapsed, prints fade away.
        w.goose.track_mud_end_time = -1.0;
        let faded_at = w.now() + 10.0; // past the 8.5 s lifetime
        assert_eq!(w.goose.foot_marks.alive_count(faded_at), 0);
    }
}
