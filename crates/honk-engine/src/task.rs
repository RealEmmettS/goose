//! The goose's task state machine — the AI.
//!
//! Mirrors the original's model (`GooseTaskInfo` + `TaskDatabase`, `Exports.cs`): a default
//! roaming state picks a random *pickable* task via the biased [`Deck`](crate::rng::Deck);
//! a task only sets `target_pos` / speed / acceleration and the engine auto-locomotes
//! (see [`crate::locomotion`]). A scripted **FirstUX** intro runs once before roaming.
//!
//! This `Task` trait is the documented internal extension seam (plan §18) — adding a
//! behavior means adding a `Task` impl and registering it; there is no external mod ABI.
//! Richer autonomous tasks (collect-window/notepad/meme/donate, off-screen bolt) land in M9+.

use crate::cursor::{CursorCommand, MouseStealOptions};
use crate::entity::GooseEntity;
use crate::foreign_window::{ForeignWindowOptions, ForeignWindowSnapshot};
use crate::interaction::Pointer;
use crate::math::{clamp, Rect, Vec2};
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
    /// Cursor commands a task wants the platform backend to apply this frame.
    pub cursor_commands: &'a mut Vec<CursorCommand>,
    /// Last pointer snapshot in world/desktop coordinates.
    pub pointer: Pointer,
    /// Mouse-stealing tuning and backend support.
    pub mouse_steal: MouseStealOptions,
    /// Foreign-window tuning and backend support.
    pub foreign_window: ForeignWindowOptions,
    /// The user-dragged foreign window currently being watched, if any.
    pub dragged_window: Option<ForeignWindowSnapshot>,
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

fn clamp_point(p: Vec2, bounds: Rect) -> Vec2 {
    Vec2::new(
        clamp(p.x, bounds.min.x, bounds.max.x),
        clamp(p.y, bounds.min.y, bounds.max.y),
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

#[derive(Debug, Clone, Copy, PartialEq)]
enum NabState {
    SeekingMouse,
    DraggingMouseAway {
        original_vector_to_mouse: Vec2,
        grabbed_at: f32,
        target: Vec2,
    },
}

/// Cursor-stealing behavior (M7): chase the live pointer, grab it at the beak, then run a
/// bounded hyper-style burst while emitting platform-free cursor-warp commands.
pub struct NabMouseTask {
    state: NabState,
    bite_played: bool,
}

impl Default for NabMouseTask {
    fn default() -> Self {
        Self::new()
    }
}

impl NabMouseTask {
    pub fn new() -> Self {
        Self {
            state: NabState::SeekingMouse,
            bite_played: false,
        }
    }

    fn hyper_target(ctx: &mut TaskCtx) -> Vec2 {
        random_point(ctx)
    }
}

impl Task for NabMouseTask {
    fn id(&self) -> &'static str {
        "nab_mouse"
    }

    fn run(&mut self, goose: &mut GooseEntity, ctx: &mut TaskCtx) -> bool {
        if !ctx.mouse_steal.active() || !ctx.pointer.present {
            return true;
        }

        goose.current_speed = goose.parameters.charge_speed;
        goose.current_acceleration = goose.parameters.acceleration_charged;
        goose.extending_neck = true;

        match self.state {
            NabState::SeekingMouse => {
                goose.target_pos = clamp_point(ctx.pointer.pos, ctx.bounds);

                if Vec2::distance(goose.rig.beak_tip, ctx.pointer.pos)
                    <= ctx.mouse_steal.grab_distance
                {
                    let original_vector_to_mouse = ctx.pointer.pos - goose.rig.beak_tip;
                    let target = Self::hyper_target(ctx);
                    self.state = NabState::DraggingMouseAway {
                        original_vector_to_mouse,
                        grabbed_at: ctx.now,
                        target,
                    };
                    if !self.bite_played {
                        self.bite_played = true;
                        ctx.sounds.push(Sound::Bite);
                    }
                    ctx.cursor_commands.push(CursorCommand::WarpTo(
                        goose.rig.beak_tip + original_vector_to_mouse,
                    ));
                }
                false
            }
            NabState::DraggingMouseAway {
                original_vector_to_mouse,
                grabbed_at,
                mut target,
            } => {
                if arrived(goose, 3.0) {
                    target = Self::hyper_target(ctx);
                    self.state = NabState::DraggingMouseAway {
                        original_vector_to_mouse,
                        grabbed_at,
                        target,
                    };
                    if ctx.rng.next_f64() < 0.5 {
                        ctx.sounds.push(Sound::Honk);
                    }
                }

                goose.target_pos = target;
                let desired_cursor =
                    clamp_point(goose.rig.beak_tip + original_vector_to_mouse, ctx.bounds);

                if Vec2::distance(ctx.pointer.pos, desired_cursor) > ctx.mouse_steal.drop_distance {
                    return true;
                }

                ctx.cursor_commands
                    .push(CursorCommand::WarpTo(desired_cursor));
                ctx.now - grabbed_at >= ctx.mouse_steal.succ_time
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PerchRideState {
    Seeking,
    Riding,
}

/// Foreign-window perch-and-ride behavior (M8): run to a user-dragged window's title-bar
/// anchor, then ride it until the drag ends. Window discovery and geometry stay in the
/// platform backend; the engine only receives opaque IDs and world-space anchors.
pub struct PerchRideTask {
    window: Option<ForeignWindowSnapshot>,
    state: PerchRideState,
}

impl Default for PerchRideTask {
    fn default() -> Self {
        Self::new()
    }
}

impl PerchRideTask {
    pub fn new() -> Self {
        Self {
            window: None,
            state: PerchRideState::Seeking,
        }
    }
}

impl Task for PerchRideTask {
    fn id(&self) -> &'static str {
        "perch_ride"
    }

    fn run(&mut self, goose: &mut GooseEntity, ctx: &mut TaskCtx) -> bool {
        if !ctx.foreign_window.watch_active() {
            return true;
        }

        let Some(snapshot) = ctx.dragged_window else {
            return true;
        };

        if let Some(current) = self.window {
            if current.id != snapshot.id {
                return true;
            }
        } else {
            self.window = Some(snapshot);
        }

        match self.state {
            PerchRideState::Seeking => {
                goose.current_speed = goose.parameters.run_speed;
                goose.current_acceleration = goose.parameters.acceleration_normal;
                goose.target_pos = snapshot.ride_anchor;

                if Vec2::distance(goose.position, snapshot.ride_anchor) <= 6.0 {
                    self.state = PerchRideState::Riding;
                    goose.position = snapshot.ride_anchor;
                    goose.target_pos = snapshot.ride_anchor;
                    goose.velocity = Vec2::ZERO;
                }
                false
            }
            PerchRideState::Riding => {
                goose.position = snapshot.ride_anchor;
                goose.target_pos = snapshot.ride_anchor;
                goose.velocity = Vec2::ZERO;
                goose.current_speed = 0.0;
                goose.current_acceleration = 0.0;
                false
            }
        }
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

    fn base_ctx<'a>(
        now: f32,
        rng: &'a mut SplitMix64,
        sounds: &'a mut Vec<Sound>,
        cursor_commands: &'a mut Vec<CursorCommand>,
    ) -> TaskCtx<'a> {
        TaskCtx {
            now,
            dt: 1.0 / 120.0,
            bounds: ctx_bounds(),
            rng,
            sounds,
            cursor_commands,
            pointer: Pointer::default(),
            mouse_steal: MouseStealOptions::default(),
            foreign_window: ForeignWindowOptions::default(),
            dragged_window: None,
            calm: false,
        }
    }

    fn ctx_bounds() -> Rect {
        Rect {
            min: Vec2::new(0.0, 0.0),
            max: Vec2::new(1000.0, 800.0),
        }
    }

    fn dragged_window(anchor: Vec2) -> ForeignWindowSnapshot {
        ForeignWindowSnapshot::top_center(
            crate::foreign_window::ForeignWindowId(42),
            Rect {
                min: Vec2::new(anchor.x - 100.0, anchor.y),
                max: Vec2::new(anchor.x + 100.0, anchor.y + 120.0),
            },
        )
    }

    #[test]
    fn wander_picks_in_bounds_targets_and_finishes() {
        let mut rng = SplitMix64::seed(1);
        let mut sounds: Vec<Sound> = Vec::new();
        let mut cursor_commands: Vec<CursorCommand> = Vec::new();
        let b = ctx_bounds();
        let mut goose = GooseEntity::new();
        let mut task = WanderTask::new();
        // First run sets a target inside bounds and arms the dwell timer.
        let mut ctx = base_ctx(0.0, &mut rng, &mut sounds, &mut cursor_commands);
        assert!(!task.run(&mut goose, &mut ctx));
        assert!(goose.target_pos.x >= b.min.x && goose.target_pos.x <= b.max.x);
        assert!(goose.target_pos.y >= b.min.y && goose.target_pos.y <= b.max.y);
        assert_eq!(goose.current_speed, goose.parameters.walk_speed);
        // Well past the max dwell it reports finished.
        let mut ctx = base_ctx(
            MAX_WANDERING_TIME + 1.0,
            &mut rng,
            &mut sounds,
            &mut cursor_commands,
        );
        assert!(task.run(&mut goose, &mut ctx));
    }

    /// Run `WanderTask` through `iters` forced arrivals and count the spontaneous honks.
    fn wander_arrival_honks(calm: bool, seed: u64, iters: usize) -> usize {
        let mut rng = SplitMix64::seed(seed);
        let mut sounds: Vec<Sound> = Vec::new();
        let mut cursor_commands: Vec<CursorCommand> = Vec::new();
        let mut goose = GooseEntity::new();
        let mut task = WanderTask::new();
        let mut now = 0.0;
        for _ in 0..iters {
            // Snap onto the current target so the next run sees an arrival.
            goose.position = goose.target_pos;
            let mut ctx = base_ctx(now, &mut rng, &mut sounds, &mut cursor_commands);
            ctx.calm = calm;
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
        let mut cursor_commands: Vec<CursorCommand> = Vec::new();
        let mut goose = GooseEntity::new();
        let mut task = WanderTask::new();
        // First run arms the task and sets walk speed.
        let mut ctx = base_ctx(0.0, &mut rng, &mut sounds, &mut cursor_commands);
        task.run(&mut goose, &mut ctx);
        // Simulate a hyper burst having left charge-tier speed on the goose.
        goose.current_speed = 999.0;
        goose.current_acceleration = 999.0;
        let mut ctx = base_ctx(1.0, &mut rng, &mut sounds, &mut cursor_commands);
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
        let mut cursor_commands: Vec<CursorCommand> = Vec::new();
        let mut goose = GooseEntity::new();
        let mut task = HyperTask::new();
        let mut ctx = base_ctx(0.0, &mut rng, &mut sounds, &mut cursor_commands);
        assert!(!task.run(&mut goose, &mut ctx), "still hyper at t=0");
        assert_eq!(goose.current_speed, goose.parameters.charge_speed);
        assert_eq!(
            goose.current_acceleration,
            goose.parameters.acceleration_charged
        );
        // Well past the burst it reports finished.
        let mut ctx = base_ctx(
            HYPER_DURATION + 0.1,
            &mut rng,
            &mut sounds,
            &mut cursor_commands,
        );
        assert!(
            task.run(&mut goose, &mut ctx),
            "hyper ends after its duration"
        );
    }

    #[test]
    fn hyper_honks_excitedly_on_enter() {
        let mut rng = SplitMix64::seed(8);
        let mut sounds: Vec<Sound> = Vec::new();
        let mut cursor_commands: Vec<CursorCommand> = Vec::new();
        let mut goose = GooseEntity::new();
        let mut task = HyperTask::new();
        let mut ctx = base_ctx(0.0, &mut rng, &mut sounds, &mut cursor_commands);
        task.run(&mut goose, &mut ctx);
        assert!(
            sounds.contains(&Sound::Honk),
            "clicking the goose makes it honk"
        );
    }

    #[test]
    fn mouse_steal_default_drag_time_matches_hyper_burst() {
        assert_eq!(MouseStealOptions::default().succ_time, HYPER_DURATION);
    }

    #[test]
    fn first_ux_walks_in_then_finishes_after_intro() {
        let mut rng = SplitMix64::seed(2);
        let mut sounds: Vec<Sound> = Vec::new();
        let mut cursor_commands: Vec<CursorCommand> = Vec::new();
        let b = ctx_bounds();
        let center = (b.min + b.max) * 0.5;
        let mut goose = GooseEntity::new();
        goose.position = Vec2::new(center.x, b.max.y + 60.0); // start off-stage
        let mut task = FirstUxTask::new();

        // Before arriving at centre, never finished.
        let mut ctx = base_ctx(0.0, &mut rng, &mut sounds, &mut cursor_commands);
        assert!(!task.run(&mut goose, &mut ctx));
        assert_eq!(goose.target_pos, center);

        // Snap to centre → the intro pause arms; still not finished until it elapses.
        goose.position = center;
        let mut ctx = base_ctx(1.0, &mut rng, &mut sounds, &mut cursor_commands);
        assert!(!task.run(&mut goose, &mut ctx));
        let mut ctx = base_ctx(
            1.0 + FIRST_WANDER_TIME + 0.1,
            &mut rng,
            &mut sounds,
            &mut cursor_commands,
        );
        assert!(task.run(&mut goose, &mut ctx));
    }

    #[test]
    fn nab_finishes_without_cursor_capability() {
        let mut rng = SplitMix64::seed(10);
        let mut sounds = Vec::new();
        let mut cursor_commands = Vec::new();
        let mut goose = GooseEntity::new();
        let mut task = NabMouseTask::new();
        let mut ctx = base_ctx(0.0, &mut rng, &mut sounds, &mut cursor_commands);
        ctx.pointer = Pointer {
            pos: goose.rig.beak_tip,
            present: true,
            left_down: false,
        };
        ctx.mouse_steal.enabled = true;
        ctx.mouse_steal.warp_supported = false;

        assert!(task.run(&mut goose, &mut ctx));
        assert!(ctx.cursor_commands.is_empty());
        assert!(ctx.sounds.is_empty());
    }

    #[test]
    fn nab_seeks_live_pointer_at_charge_speed() {
        let mut rng = SplitMix64::seed(11);
        let mut sounds = Vec::new();
        let mut cursor_commands = Vec::new();
        let mut goose = GooseEntity::new();
        let mut task = NabMouseTask::new();
        let pointer = Vec2::new(700.0, 500.0);
        let mut ctx = base_ctx(0.0, &mut rng, &mut sounds, &mut cursor_commands);
        ctx.pointer = Pointer {
            pos: pointer,
            present: true,
            left_down: false,
        };
        ctx.mouse_steal = MouseStealOptions::with_backend_support(true);

        assert!(!task.run(&mut goose, &mut ctx));
        assert_eq!(goose.target_pos, pointer);
        assert_eq!(goose.current_speed, goose.parameters.charge_speed);
        assert_eq!(
            goose.current_acceleration,
            goose.parameters.acceleration_charged
        );
        assert!(ctx.cursor_commands.is_empty(), "not grabbed yet");
    }

    #[test]
    fn nab_grabs_with_one_bite_and_cursor_warp() {
        let mut rng = SplitMix64::seed(12);
        let mut sounds = Vec::new();
        let mut cursor_commands = Vec::new();
        let mut goose = GooseEntity::new();
        let mut task = NabMouseTask::new();
        let pointer = goose.rig.beak_tip + Vec2::new(3.0, 0.0);
        let mut ctx = base_ctx(0.0, &mut rng, &mut sounds, &mut cursor_commands);
        ctx.pointer = Pointer {
            pos: pointer,
            present: true,
            left_down: false,
        };
        ctx.mouse_steal = MouseStealOptions::with_backend_support(true);

        assert!(!task.run(&mut goose, &mut ctx));
        assert_eq!(&*ctx.sounds, &[Sound::Bite]);
        assert_eq!(&*ctx.cursor_commands, &[CursorCommand::WarpTo(pointer)]);

        ctx.sounds.clear();
        ctx.cursor_commands.clear();
        ctx.now = 0.5;
        ctx.pointer.pos = pointer;
        assert!(!task.run(&mut goose, &mut ctx));
        assert!(
            ctx.sounds.is_empty(),
            "the bite sound should play only when the cursor is first grabbed"
        );
        assert_eq!(ctx.cursor_commands.len(), 1);
    }

    #[test]
    fn nab_drag_preserves_beak_cursor_offset_and_times_out() {
        let mut rng = SplitMix64::seed(13);
        let mut sounds = Vec::new();
        let mut cursor_commands = Vec::new();
        let mut goose = GooseEntity::new();
        let mut task = NabMouseTask::new();
        let offset = Vec2::new(9.0, -4.0);
        let pointer = goose.rig.beak_tip + offset;
        let mut ctx = base_ctx(0.0, &mut rng, &mut sounds, &mut cursor_commands);
        ctx.pointer = Pointer {
            pos: pointer,
            present: true,
            left_down: false,
        };
        ctx.mouse_steal = MouseStealOptions::with_backend_support(true);

        task.run(&mut goose, &mut ctx);
        ctx.cursor_commands.clear();
        ctx.sounds.clear();

        goose.rig.beak_tip = goose.rig.beak_tip + Vec2::new(25.0, 10.0);
        let expected = goose.rig.beak_tip + offset;
        ctx.pointer.pos = expected;
        ctx.now = 0.25;
        assert!(!task.run(&mut goose, &mut ctx));
        assert_eq!(&*ctx.cursor_commands, &[CursorCommand::WarpTo(expected)]);

        ctx.cursor_commands.clear();
        ctx.pointer.pos = expected;
        ctx.now = ctx.mouse_steal.succ_time + 0.01;
        assert!(task.run(&mut goose, &mut ctx), "nab ends after succ_time");
    }

    #[test]
    fn nab_drag_retargets_like_hyper_when_it_arrives() {
        let mut rng = SplitMix64::seed(15);
        let mut sounds = Vec::new();
        let mut cursor_commands = Vec::new();
        let mut goose = GooseEntity::new();
        let mut task = NabMouseTask::new();
        let offset = Vec2::new(6.0, 2.0);
        let pointer = goose.rig.beak_tip + offset;
        let mut ctx = base_ctx(0.0, &mut rng, &mut sounds, &mut cursor_commands);
        ctx.pointer = Pointer {
            pos: pointer,
            present: true,
            left_down: false,
        };
        ctx.mouse_steal = MouseStealOptions::with_backend_support(true);

        assert!(!task.run(&mut goose, &mut ctx));
        let first_target = match task.state {
            NabState::DraggingMouseAway { target, .. } => target,
            NabState::SeekingMouse => panic!("nab should be dragging after the grab"),
        };

        ctx.cursor_commands.clear();
        ctx.sounds.clear();
        goose.position = first_target;
        goose.target_pos = first_target;
        ctx.pointer.pos = goose.rig.beak_tip + offset;
        ctx.now = 0.25;

        assert!(!task.run(&mut goose, &mut ctx));
        let second_target = match task.state {
            NabState::DraggingMouseAway { target, .. } => target,
            NabState::SeekingMouse => panic!("nab should still be dragging after retarget"),
        };
        assert_ne!(
            second_target, first_target,
            "dragging should retarget like hyper instead of pulling in one straight line"
        );
        assert_eq!(goose.target_pos, second_target);
        assert_eq!(
            &*ctx.cursor_commands,
            &[CursorCommand::WarpTo(goose.rig.beak_tip + offset)]
        );
    }

    #[test]
    fn nab_drops_when_cursor_is_pulled_far_away() {
        let mut rng = SplitMix64::seed(14);
        let mut sounds = Vec::new();
        let mut cursor_commands = Vec::new();
        let mut goose = GooseEntity::new();
        let mut task = NabMouseTask::new();
        let pointer = goose.rig.beak_tip;
        let mut ctx = base_ctx(0.0, &mut rng, &mut sounds, &mut cursor_commands);
        ctx.pointer = Pointer {
            pos: pointer,
            present: true,
            left_down: false,
        };
        ctx.mouse_steal = MouseStealOptions::with_backend_support(true);

        task.run(&mut goose, &mut ctx);
        ctx.cursor_commands.clear();
        ctx.pointer.pos = pointer + Vec2::new(ctx.mouse_steal.drop_distance + 20.0, 0.0);
        ctx.now = 0.25;

        assert!(
            task.run(&mut goose, &mut ctx),
            "manual pull-away drops the cursor"
        );
        assert!(ctx.cursor_commands.is_empty());
    }

    #[test]
    fn perch_ride_finishes_without_window_watch_capability() {
        let mut rng = SplitMix64::seed(16);
        let mut sounds = Vec::new();
        let mut cursor_commands = Vec::new();
        let mut goose = GooseEntity::new();
        let mut task = PerchRideTask::new();
        let mut ctx = base_ctx(0.0, &mut rng, &mut sounds, &mut cursor_commands);
        ctx.dragged_window = Some(dragged_window(Vec2::new(500.0, 200.0)));

        assert!(task.run(&mut goose, &mut ctx));
    }

    #[test]
    fn perch_ride_seeks_window_anchor_at_run_speed() {
        let mut rng = SplitMix64::seed(17);
        let mut sounds = Vec::new();
        let mut cursor_commands = Vec::new();
        let mut goose = GooseEntity::new();
        let mut task = PerchRideTask::new();
        let anchor = Vec2::new(800.0, 120.0);
        let mut ctx = base_ctx(0.0, &mut rng, &mut sounds, &mut cursor_commands);
        ctx.foreign_window = ForeignWindowOptions::with_backend_support(true, false);
        ctx.dragged_window = Some(dragged_window(anchor));

        assert!(!task.run(&mut goose, &mut ctx));
        assert_eq!(goose.target_pos, anchor);
        assert_eq!(goose.current_speed, goose.parameters.run_speed);
        assert_eq!(
            goose.current_acceleration,
            goose.parameters.acceleration_normal
        );
    }

    #[test]
    fn perch_ride_abandons_when_drag_releases_before_arrival() {
        let mut rng = SplitMix64::seed(18);
        let mut sounds = Vec::new();
        let mut cursor_commands = Vec::new();
        let mut goose = GooseEntity::new();
        let mut task = PerchRideTask::new();
        let mut ctx = base_ctx(0.0, &mut rng, &mut sounds, &mut cursor_commands);
        ctx.foreign_window = ForeignWindowOptions::with_backend_support(true, false);
        ctx.dragged_window = Some(dragged_window(Vec2::new(900.0, 120.0)));
        assert!(!task.run(&mut goose, &mut ctx));

        ctx.dragged_window = None;
        ctx.now = 0.25;
        assert!(task.run(&mut goose, &mut ctx));
    }

    #[test]
    fn perch_ride_pins_to_moving_anchor_after_arrival() {
        let mut rng = SplitMix64::seed(19);
        let mut sounds = Vec::new();
        let mut cursor_commands = Vec::new();
        let mut goose = GooseEntity::new();
        let mut task = PerchRideTask::new();
        let first_anchor = Vec2::new(-400.0, -20.0);
        goose.position = first_anchor + Vec2::new(2.0, 2.0);
        let mut ctx = base_ctx(0.0, &mut rng, &mut sounds, &mut cursor_commands);
        ctx.foreign_window = ForeignWindowOptions::with_backend_support(true, false);
        ctx.dragged_window = Some(dragged_window(first_anchor));

        assert!(!task.run(&mut goose, &mut ctx));
        assert_eq!(goose.position, first_anchor);
        assert_eq!(goose.velocity, Vec2::ZERO);

        let moved_anchor = Vec2::new(-360.0, -16.0);
        ctx.dragged_window = Some(dragged_window(moved_anchor));
        ctx.now = 0.25;
        goose.velocity = Vec2::new(40.0, 5.0);
        assert!(!task.run(&mut goose, &mut ctx));
        assert_eq!(goose.position, moved_anchor);
        assert_eq!(goose.target_pos, moved_anchor);
        assert_eq!(goose.velocity, Vec2::ZERO);
    }
}
