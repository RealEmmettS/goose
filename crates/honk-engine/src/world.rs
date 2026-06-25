//! The simulation world: owns the goose and drives it through the task state machine.
//!
//! A scripted **FirstUX** intro runs once (the goose walks on-stage and introduces itself),
//! then the default roaming state picks a random *pickable* task via the biased
//! [`Deck`](crate::rng::Deck). Tasks set targets/params; [`crate::locomotion`] moves the
//! goose; the gait + footmark logic here is mechanical.

use crate::locomotion;
use crate::math::{Rect, Vec2};
use crate::rig::Rig;
use crate::rng::{Deck, RandomSource, SplitMix64};
use crate::sound::Sound;
use crate::task::{FirstUxTask, Task, TaskCtx, WanderTask};
use crate::time::DT;

/// Distance travelled per full walking-gait cycle (radians of `gait_phase` per `TAU`).
const GAIT_CYCLE_DISTANCE: f32 = 22.0;

/// The whole simulation: one goose roaming within `bounds` (the virtual-desktop space).
pub struct World {
    pub goose: GooseEntity,
    pub bounds: Rect,
    rng: SplitMix64,
    current: Box<dyn Task>,
    /// Factories for the randomly-pickable roaming tasks (the original's `TaskDatabase`).
    pickable: Vec<fn() -> Box<dyn Task>>,
    /// Shuffle-bag over `pickable` indices (no repeats until exhausted).
    deck: Deck<SplitMix64>,
    elapsed: f32,
    /// Index of the last gait half-step a footmark was considered for.
    last_step: i64,
    /// Sound requests produced this tick, drained by the platform audio backend.
    pending_sounds: Vec<Sound>,
}

use crate::entity::GooseEntity;

impl World {
    /// A world bounded by `bounds`, with the goose entering from just off the bottom edge
    /// for the FirstUX intro. `seed` makes the whole simulation deterministic.
    pub fn new(bounds: Rect, seed: u64) -> Self {
        let center = (bounds.min + bounds.max) * 0.5;
        let mut goose = GooseEntity::new();
        // Enter from just off the bottom edge; FirstUX walks the goose on-stage.
        goose.position = Vec2::new(center.x, bounds.max.y + 60.0);
        goose.target_pos = center;
        goose.current_speed = goose.parameters.walk_speed;
        goose.current_acceleration = goose.parameters.acceleration_normal;
        goose.rig = Rig::update(goose.position, goose.direction, 0.0, 0.0);

        let pickable: Vec<fn() -> Box<dyn Task>> =
            vec![|| Box::new(WanderTask::new()) as Box<dyn Task>];
        let deck = Deck::new(pickable.len(), SplitMix64::seed(seed ^ 0x9E37_79B9));

        Self {
            goose,
            bounds,
            rng: SplitMix64::seed(seed),
            current: Box::new(FirstUxTask::new()), // scripted intro runs first
            pickable,
            deck,
            elapsed: 0.0,
            last_step: 0,
            pending_sounds: Vec::new(),
        }
    }

    /// The world's monotonic clock (seconds), the time base for footmark fade.
    pub fn now(&self) -> f32 {
        self.elapsed
    }

    /// Take the sound requests produced since the last call (for the audio backend).
    pub fn take_sounds(&mut self) -> Vec<Sound> {
        std::mem::take(&mut self.pending_sounds)
    }

    /// The id of the currently running task (e.g. `"first_ux"`, `"wander"`).
    pub fn current_task(&self) -> &'static str {
        self.current.id()
    }

    /// Pick the next roaming task from the shuffle-bag.
    fn next_task(&mut self) -> Box<dyn Task> {
        let idx = self.deck.draw();
        (self.pickable[idx])()
    }

    /// Advance the world by one fixed [`DT`] tick.
    pub fn tick(&mut self) {
        self.elapsed += DT;

        // Run the current task (it only sets targets/params); pick the next when it's done.
        let done = {
            let mut ctx = TaskCtx {
                now: self.elapsed,
                dt: DT,
                bounds: self.bounds,
                rng: &mut self.rng,
                sounds: &mut self.pending_sounds,
            };
            self.current.run(&mut self.goose, &mut ctx)
        };
        if done {
            self.current = self.next_task();
        }

        // Auto-locomotion toward the task's target.
        let before = self.goose.position;
        locomotion::step(&mut self.goose, DT);

        // Advance the walking gait by distance travelled (a stopped goose stands still).
        let moved = Vec2::distance(before, self.goose.position);
        self.goose.gait_phase += moved * (std::f32::consts::TAU / GAIT_CYCLE_DISTANCE);

        let speed_frac =
            (self.goose.velocity.magnitude() / self.goose.parameters.walk_speed).min(1.0);
        self.goose.rig = Rig::update(
            self.goose.position,
            self.goose.direction,
            speed_frac * 0.4,
            self.goose.gait_phase,
        );

        // Drop a fading muddy print at each foot-plant (half gait cycle) while tracking mud.
        let step = (self.goose.gait_phase / std::f32::consts::PI).floor() as i64;
        if step > self.last_step {
            if self.elapsed < self.goose.track_mud_end_time {
                let foot = if step % 2 == 0 {
                    self.goose.rig.feet.left
                } else {
                    self.goose.rig.feet.right
                };
                self.goose.foot_marks.add(foot, self.elapsed);
                // A wet squelch now and then while squishing through mud.
                if self.rng.next_f64() < 0.35 {
                    self.pending_sounds.push(Sound::MudSquish);
                }
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
            max: Vec2::new(1000.0, 800.0),
        }
    }

    #[test]
    fn goose_walks_in_during_first_ux() {
        let mut w = World::new(bounds(), 1);
        assert_eq!(w.current_task(), "first_ux");
        let start = w.goose.position;
        for _ in 0..240 {
            w.tick();
        }
        // It walks on-stage (upward) during the intro.
        assert!(Vec2::distance(start, w.goose.position) > 1.0);
    }

    #[test]
    fn roams_within_bounds_after_intro() {
        let mut w = World::new(bounds(), 2);
        // Warm up past the off-stage entrance (it reaches centre within ~1 s of walking).
        for _ in 0..1_000 {
            w.tick();
        }
        for _ in 0..5_000 {
            w.tick();
            let p = w.goose.position;
            assert!(
                p.x >= -1.0 && p.x <= 1001.0 && p.y >= -1.0 && p.y <= 801.0,
                "{p:?}"
            );
        }
    }

    #[test]
    fn hands_off_first_ux_to_roaming() {
        let mut w = World::new(bounds(), 3);
        // FirstUX = walk in + a FIRST_WANDER_TIME pause; well past it we're roaming.
        let mut saw_wander = false;
        for _ in 0..6_000 {
            w.tick();
            if w.current_task() == "wander" {
                saw_wander = true;
                break;
            }
        }
        assert!(saw_wander, "should hand off from first_ux to wander");
    }

    #[test]
    fn deterministic_for_seed() {
        let mut a = World::new(bounds(), 42);
        let mut b = World::new(bounds(), 42);
        for _ in 0..4_000 {
            a.tick();
            b.tick();
        }
        assert_eq!(a.goose.position, b.goose.position);
        assert_eq!(a.current_task(), b.current_task());
    }

    #[test]
    fn tracks_mud_and_drops_fading_prints() {
        let mut w = World::new(bounds(), 5);
        // Force mud-tracking on while the goose walks in (it's moving, so it steps).
        w.goose.track_mud_end_time = 1_000.0;
        for _ in 0..700 {
            w.tick();
        }
        assert!(
            w.goose.foot_marks.alive_count(w.now()) > 0,
            "expected muddy prints while tracking mud and moving"
        );
        // With tracking off and enough time elapsed, prints fade away.
        w.goose.track_mud_end_time = -1.0;
        let faded_at = w.now() + 10.0; // past the 8.5 s lifetime
        assert_eq!(w.goose.foot_marks.alive_count(faded_at), 0);
    }

    #[test]
    fn emits_sound_requests_while_roaming() {
        let mut w = World::new(bounds(), 7);
        let mut heard = false;
        // Run well past FirstUX into roaming; the goose honks on retarget / squishes in mud.
        for _ in 0..30_000 {
            w.tick();
            if !w.take_sounds().is_empty() {
                heard = true;
                break;
            }
        }
        assert!(
            heard,
            "the goose should request sounds (honk/mud) while roaming"
        );
    }
}
