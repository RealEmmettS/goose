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
        if self.end_time.is_none() {
            goose.current_speed = goose.parameters.walk_speed;
            goose.current_acceleration = goose.parameters.acceleration_normal;
            goose.target_pos = random_point(ctx);
            self.end_time = Some(ctx.now + ctx.rng.range(MIN_WANDERING_TIME, MAX_WANDERING_TIME));
        }
        if arrived(goose, 1.5) {
            goose.target_pos = random_point(ctx);
            // Sometimes the goose tracks mud on its way to the next spot.
            if ctx.rng.next_f64() < 0.5 {
                goose.track_mud_end_time = ctx.now + goose.parameters.duration_to_track_mud;
            }
            // And sometimes it honks for no reason at all.
            if ctx.rng.next_f64() < 0.25 {
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
        };
        assert!(task.run(&mut goose, &mut ctx));
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
        };
        assert!(!task.run(&mut goose, &mut ctx));
        let mut ctx = TaskCtx {
            now: 1.0 + FIRST_WANDER_TIME + 0.1,
            dt: 1.0 / 120.0,
            bounds: b,
            rng: &mut rng,
            sounds: &mut sounds,
        };
        assert!(task.run(&mut goose, &mut ctx));
    }
}
