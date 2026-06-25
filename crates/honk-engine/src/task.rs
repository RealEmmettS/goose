//! The goose's task state machine — the AI.
//!
//! Mirrors the original's model (`GooseTaskInfo` + `TaskDatabase`, `Exports.cs`): a default
//! roaming state picks a random *pickable* task via the biased [`Deck`](crate::rng::Deck);
//! a task only sets `target_pos` / speed / acceleration and the engine auto-locomotes
//! (see [`crate::locomotion`]). A scripted **FirstUX** intro runs once before roaming.
//!
//! This `Task` trait is the documented internal extension seam (plan §18) — adding a
//! behavior means adding a `Task` impl and registering it; there is no external mod ABI.
//! Richer tasks (nab, attack, collect-window, off-screen bolt) land in M7–M9.

use crate::entity::GooseEntity;
use crate::math::{Rect, Vec2};
use crate::rng::{RandomSource, SplitMix64};
use crate::sound::Sound;

/// Verified wander timings (`config.ini`): seconds. Config-driven values arrive with the
/// TOML loader in a later round; these are the defaults.
pub const FIRST_WANDER_TIME: f32 = 20.0;
pub const MIN_WANDERING_TIME: f32 = 20.0;
pub const MAX_WANDERING_TIME: f32 = 40.0;

/// How long the click→charge "hyper" burst lasts, in seconds (M6, plan §5.6 hyper).
pub const HYPER_DURATION: f32 = 2.5;

/// Per-tick context handed to a running task.
pub struct TaskCtx<'a> {
    /// World clock (seconds).
    pub now: f32,
    /// Fixed tick duration.
    pub dt: f32,
    /// Roaming bounds (the virtual-desktop space).
    pub bounds: Rect,
    /// Shared RNG for target/dwell choices.
    pub rng: &'a mut SplitMix64,
    /// Sound requests a task wants played this frame.
    pub sounds: &'a mut Vec<Sound>,
    /// The goose is in its post-pat calm window (suppresses spontaneous honks; M6 §5.9).
    pub calm: bool,
}

/// A goose behavior. Tasks set targets/params only; locomotion is the engine's job.
pub trait Task {
    /// Stable identifier (for `do <id>` pokes and debugging).
    fn id(&self) -> &'static str;
    /// Advance one tick; return `true` when finished (the engine then picks the next task).
    fn run(&mut self, goose: &mut GooseEntity, ctx: &mut TaskCtx) -> bool;
}

fn arrived(goose: &GooseEntity, tol: f32) -> bool {
    Vec2::distance(goose.position, goose.target_pos) < tol
}

fn random_point(ctx: &mut TaskCtx) -> Vec2 {
    Vec2::new(
        ctx.rng.range(ctx.bounds.min.x, ctx.bounds.max.x),
        ctx.rng.range(ctx.bounds.min.y, ctx.bounds.max.y),
    )
}

/// Roam to random points for a random dwell, occasionally tracking mud. The default
/// pickable task (the original `Task_Wander`, with mud folded in for now).
#[derive(Default)]
pub struct WanderTask {
    end_time: Option<f32>,
}

impl WanderTask {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Task for WanderTask {
    fn id(&self) -> &'static str {
        "wander"
    }

    fn run(&mut self, goose: &mut GooseEntity, ctx: &mut TaskCtx) -> bool {
        // Re-assert walk-tier locomotion every tick so the goose cleanly resumes its stroll
        // after a transient interrupt (e.g. a hyper burst) left a faster tier on it.
        goose.current_speed = goose.parameters.walk_speed;
        goose.current_acceleration = goose.parameters.acceleration_normal;

        if self.end_time.is_none() {
            goose.target_pos = random_point(ctx);
            self.end_time = Some(ctx.now + ctx.rng.range(MIN_WANDERING_TIME, MAX_WANDERING_TIME));
        }
        if arrived(goose, 1.5) {
            goose.target_pos = random_point(ctx);
            // Sometimes the goose tracks mud on its way to the next spot.
            if ctx.rng.next_f64() < 0.5 {
                goose.track_mud_end_time = ctx.now + goose.parameters.duration_to_track_mud;
            }
            // And sometimes it honks for no reason at all — unless it's been freshly patted,
            // when it stays content and quiet for the calm window (§5.9).
            if !ctx.calm && ctx.rng.next_f64() < 0.25 {
                ctx.sounds.push(Sound::Honk);
            }
        }
        ctx.now >= self.end_time.unwrap()
    }
}

/// The click→charge reaction: a short, fast, erratic "hyper" burst (plan §5.6 hyper / M6).
/// Installed as a transient interrupt when you click the goose; when it finishes the world
/// restores whatever task was running before. The full self-triggered mood FSM is M13.
#[derive(Default)]
pub struct HyperTask {
    end_time: Option<f32>,
}

impl HyperTask {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Task for HyperTask {
    fn id(&self) -> &'static str {
        "hyper"
    }

    fn run(&mut self, goose: &mut GooseEntity, ctx: &mut TaskCtx) -> bool {
        // Charge tier for the whole burst, re-asserted each tick.
        goose.current_speed = goose.parameters.charge_speed;
        goose.current_acceleration = goose.parameters.acceleration_charged;

        if self.end_time.is_none() {
            goose.target_pos = random_point(ctx);
            ctx.sounds.push(Sound::Honk); // an indignant honk at being clicked
            self.end_time = Some(ctx.now + HYPER_DURATION);
        } else if arrived(goose, 3.0) {
            // Bolt to a fresh spot the instant it arrives — erratic, no dwell.
            goose.target_pos = random_point(ctx);
            if ctx.rng.next_f64() < 0.5 {
                ctx.sounds.push(Sound::Honk);
            }
        }
        ctx.now >= self.end_time.unwrap()
    }
}

/// The scripted first-run intro: the goose walks in to centre stage, pauses to "introduce
/// itself" for [`FIRST_WANDER_TIME`], then yields to roaming. (`FirstUX_FirstTask` /
/// `FirstUX_SecondTask` in the original; text/honk flourishes arrive with M5 audio + notes.)
#[derive(Default)]
pub struct FirstUxTask {
    intro_until: Option<f32>,
}

impl FirstUxTask {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Task for FirstUxTask {
    fn id(&self) -> &'static str {
        "first_ux"
    }

    fn run(&mut self, goose: &mut GooseEntity, ctx: &mut TaskCtx) -> bool {
        match self.intro_until {
            None => {
                // Walk in to centre stage.
                goose.current_speed = goose.parameters.walk_speed;
                goose.current_acceleration = goose.parameters.acceleration_normal;
                goose.target_pos = (ctx.bounds.min + ctx.bounds.max) * 0.5;
                if arrived(goose, 2.0) {
                    self.intro_until = Some(ctx.now + FIRST_WANDER_TIME);
                }
                false
            }
            // Pause centre stage for the intro beat, then hand off to roaming.
            Some(until) => ctx.now >= until,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ctx_bounds() -> Rect {
        Rect {
            min: Vec2::new(0.0, 0.0),
            max: Vec2::new(1000.0, 800.0),
        }
    }

    #[test]
    fn wander_picks_in_bounds_targets_and_finishes() {
        let mut rng = SplitMix64::seed(1);
        let mut sounds: Vec<Sound> = Vec::new();
        let b = ctx_bounds();
        let mut goose = GooseEntity::new();
        let mut task = WanderTask::new();
        // First run sets a target inside bounds and arms the dwell timer.
        let mut ctx = TaskCtx {
            now: 0.0,
            dt: 1.0 / 120.0,
            bounds: b,
            rng: &mut rng,
            sounds: &mut sounds,
            calm: false,
        };
        assert!(!task.run(&mut goose, &mut ctx));
        assert!(goose.target_pos.x >= b.min.x && goose.target_pos.x <= b.max.x);
        assert!(goose.target_pos.y >= b.min.y && goose.target_pos.y <= b.max.y);
        assert_eq!(goose.current_speed, goose.parameters.walk_speed);
        // Well past the max dwell it reports finished.
        let mut ctx = TaskCtx {
            now: MAX_WANDERING_TIME + 1.0,
            dt: 1.0 / 120.0,
            bounds: b,
            rng: &mut rng,
            sounds: &mut sounds,
            calm: false,
        };
        assert!(task.run(&mut goose, &mut ctx));
    }

    /// Run `WanderTask` through `iters` forced arrivals and count the spontaneous honks.
    fn wander_arrival_honks(calm: bool, seed: u64, iters: usize) -> usize {
        let mut rng = SplitMix64::seed(seed);
        let mut sounds: Vec<Sound> = Vec::new();
        let b = ctx_bounds();
        let mut goose = GooseEntity::new();
        let mut task = WanderTask::new();
        let mut now = 0.0;
        for _ in 0..iters {
            // Snap onto the current target so the next run sees an arrival.
            goose.position = goose.target_pos;
            let mut ctx = TaskCtx {
                now,
                dt: 1.0 / 120.0,
                bounds: b,
                rng: &mut rng,
                sounds: &mut sounds,
                calm,
            };
            task.run(&mut goose, &mut ctx);
            now += 0.1;
        }
        sounds.iter().filter(|s| **s == Sound::Honk).count()
    }

    #[test]
    fn calm_suppresses_spontaneous_honks() {
        let seed = 12_345;
        let noisy = wander_arrival_honks(false, seed, 80);
        assert!(noisy > 0, "control: an un-calm goose honks on arrivals");
        let calm = wander_arrival_honks(true, seed, 80);
        assert_eq!(
            calm, 0,
            "a calm (post-pat) goose suppresses spontaneous honks"
        );
    }

    #[test]
    fn wander_reasserts_speed_each_run() {
        let mut rng = SplitMix64::seed(99);
        let mut sounds: Vec<Sound> = Vec::new();
        let b = ctx_bounds();
        let mut goose = GooseEntity::new();
        let mut task = WanderTask::new();
        // First run arms the task and sets walk speed.
        let mut ctx = TaskCtx {
            now: 0.0,
            dt: 1.0 / 120.0,
            bounds: b,
            rng: &mut rng,
            sounds: &mut sounds,
            calm: false,
        };
        task.run(&mut goose, &mut ctx);
        // Simulate a hyper burst having left charge-tier speed on the goose.
        goose.current_speed = 999.0;
        goose.current_acceleration = 999.0;
        let mut ctx = TaskCtx {
            now: 1.0,
            dt: 1.0 / 120.0,
            bounds: b,
            rng: &mut rng,
            sounds: &mut sounds,
            calm: false,
        };
        task.run(&mut goose, &mut ctx);
        assert_eq!(
            goose.current_speed, goose.parameters.walk_speed,
            "wander should restore walk speed after a hyper burst"
        );
        assert_eq!(
            goose.current_acceleration,
            goose.parameters.acceleration_normal
        );
    }

    #[test]
    fn hyper_sets_charge_tier_and_finishes() {
        let mut rng = SplitMix64::seed(4);
        let mut sounds: Vec<Sound> = Vec::new();
        let b = ctx_bounds();
        let mut goose = GooseEntity::new();
        let mut task = HyperTask::new();
        let mut ctx = TaskCtx {
            now: 0.0,
            dt: 1.0 / 120.0,
            bounds: b,
            rng: &mut rng,
            sounds: &mut sounds,
            calm: false,
        };
        assert!(!task.run(&mut goose, &mut ctx), "still hyper at t=0");
        assert_eq!(goose.current_speed, goose.parameters.charge_speed);
        assert_eq!(
            goose.current_acceleration,
            goose.parameters.acceleration_charged
        );
        // Well past the burst it reports finished.
        let mut ctx = TaskCtx {
            now: HYPER_DURATION + 0.1,
            dt: 1.0 / 120.0,
            bounds: b,
            rng: &mut rng,
            sounds: &mut sounds,
            calm: false,
        };
        assert!(
            task.run(&mut goose, &mut ctx),
            "hyper ends after its duration"
        );
    }

    #[test]
    fn hyper_honks_excitedly_on_enter() {
        let mut rng = SplitMix64::seed(8);
        let mut sounds: Vec<Sound> = Vec::new();
        let b = ctx_bounds();
        let mut goose = GooseEntity::new();
        let mut task = HyperTask::new();
        let mut ctx = TaskCtx {
            now: 0.0,
            dt: 1.0 / 120.0,
            bounds: b,
            rng: &mut rng,
            sounds: &mut sounds,
            calm: false,
        };
        task.run(&mut goose, &mut ctx);
        assert!(
            sounds.contains(&Sound::Honk),
            "clicking the goose makes it honk"
        );
    }

    #[test]
    fn first_ux_walks_in_then_finishes_after_intro() {
        let mut rng = SplitMix64::seed(2);
        let mut sounds: Vec<Sound> = Vec::new();
        let b = ctx_bounds();
        let center = (b.min + b.max) * 0.5;
        let mut goose = GooseEntity::new();
        goose.position = Vec2::new(center.x, b.max.y + 60.0); // start off-stage
        let mut task = FirstUxTask::new();

        // Before arriving at centre, never finished.
        let mut ctx = TaskCtx {
            now: 0.0,
            dt: 1.0 / 120.0,
            bounds: b,
            rng: &mut rng,
            sounds: &mut sounds,
            calm: false,
        };
        assert!(!task.run(&mut goose, &mut ctx));
        assert_eq!(goose.target_pos, center);

        // Snap to centre → the intro pause arms; still not finished until it elapses.
        goose.position = center;
        let mut ctx = TaskCtx {
            now: 1.0,
            dt: 1.0 / 120.0,
            bounds: b,
            rng: &mut rng,
            sounds: &mut sounds,
            calm: false,
        };
        assert!(!task.run(&mut goose, &mut ctx));
        let mut ctx = TaskCtx {
            now: 1.0 + FIRST_WANDER_TIME + 0.1,
            dt: 1.0 / 120.0,
            bounds: b,
            rng: &mut rng,
            sounds: &mut sounds,
            calm: false,
        };
        assert!(task.run(&mut goose, &mut ctx));
    }
}
