//! The simulation world: owns the goose and drives it through the task state machine.
//!
//! A scripted **FirstUX** intro runs once (the goose walks on-stage and introduces itself),
//! then the default roaming state picks a random *pickable* task via the biased
//! [`Deck`](crate::rng::Deck). Tasks set targets/params; [`crate::locomotion`] moves the
//! goose; the gait + footmark logic here is mechanical.

use crate::cursor::{CursorCommand, WorldOptions};
use crate::foreign_window::ForeignWindowSnapshot;
use crate::hearts::Hearts;
use crate::interaction::{PatTracker, Pointer};
use crate::locomotion;
use crate::math::{Rect, Vec2};
use crate::rig::Rig;
use crate::rng::{Deck, RandomSource, SplitMix64};
use crate::sound::Sound;
use crate::task::{FirstUxTask, HyperTask, NabMouseTask, PerchRideTask, Task, TaskCtx, WanderTask};
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
    /// Cursor requests produced this tick, drained by the platform backend.
    pending_cursor_commands: Vec<CursorCommand>,
    /// Runtime options/capabilities that must stay platform-free.
    options: WorldOptions,
    /// Detects pats from hovering cursor sweeps and tracks the happy/calm streak (M6 §5.9).
    pat: PatTracker,
    /// Heart particles emitted while being patted.
    hearts: Hearts,
    /// Last pointer state fed in via [`World::set_pointer`].
    pointer: Pointer,
    /// Last platform-reported user-dragged foreign window.
    dragged_window: Option<ForeignWindowSnapshot>,
    /// Left button held on the previous pointer update (for click rising-edge detection).
    prev_left_down: bool,
    /// A click landed on the goose; the next tick installs the hyper burst.
    pending_hyper: bool,
    /// A click landed on the goose while mouse stealing is available; the next tick installs
    /// the nab task.
    pending_nab: bool,
    /// The task that was running before a transient interrupt (hyper), restored when it ends.
    interrupted: Option<Box<dyn Task>>,
}

use crate::entity::GooseEntity;

impl World {
    /// A world bounded by `bounds`, with the goose entering from just off the bottom edge
    /// for the FirstUX intro. `seed` makes the whole simulation deterministic.
    pub fn new(bounds: Rect, seed: u64) -> Self {
        Self::with_options(bounds, seed, WorldOptions::default())
    }

    /// Build a world with explicit runtime options/capabilities.
    pub fn with_options(bounds: Rect, seed: u64, options: WorldOptions) -> Self {
        let center = (bounds.min + bounds.max) * 0.5;
        let mut goose = GooseEntity::new();
        // Enter from just off the bottom edge; FirstUX walks the goose on-stage.
        goose.position = Vec2::new(center.x, bounds.max.y + 60.0);
        goose.target_pos = center;
        goose.current_speed = goose.parameters.walk_speed;
        goose.current_acceleration = goose.parameters.acceleration_normal;
        goose.rig = Rig::update(goose.position, goose.direction, 0.0, 0.0);

        let mut pickable: Vec<fn() -> Box<dyn Task>> =
            vec![|| Box::new(WanderTask::new()) as Box<dyn Task>];
        if options.mouse_steal.active() {
            pickable.push(|| Box::new(NabMouseTask::new()) as Box<dyn Task>);
        }
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
            pending_cursor_commands: Vec::new(),
            options,
            pat: PatTracker::new(),
            hearts: Hearts::new(),
            pointer: Pointer::default(),
            dragged_window: None,
            prev_left_down: false,
            pending_hyper: false,
            pending_nab: false,
            interrupted: None,
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

    /// Take cursor commands emitted since the last call (for the platform backend).
    pub fn take_cursor_commands(&mut self) -> Vec<CursorCommand> {
        std::mem::take(&mut self.pending_cursor_commands)
    }

    /// Reflect a backend capability change after startup, e.g. cursor warp failed.
    pub fn set_cursor_warp_supported(&mut self, supported: bool) {
        self.options.mouse_steal.warp_supported = supported;
    }

    /// Reflect a backend capability change after startup, e.g. move-size hook setup failed.
    pub fn set_foreign_window_watch_supported(&mut self, supported: bool) {
        self.options.foreign_window.capabilities.watch_drag = supported;
        if !supported {
            self.dragged_window = None;
        }
    }

    /// Feed one frame of foreign-window drag state in world/desktop coordinates.
    pub fn set_foreign_window_drag(&mut self, dragged_window: Option<ForeignWindowSnapshot>) {
        self.dragged_window = dragged_window;
    }

    /// The live heart particles (for the renderer).
    pub fn hearts(&self) -> &Hearts {
        &self.hearts
    }

    /// Whether the goose is currently in its post-pat calm window.
    pub fn is_calm(&self) -> bool {
        self.pat.is_calm(self.elapsed)
    }

    /// Whether the world-space `point` is over the goose (its rig bounding box; plan §6).
    pub fn goose_hit(&self, point: Vec2) -> bool {
        self.goose.rig.bounding_box().contains(point)
    }

    /// Whether the active task is controlling the real cursor.
    pub fn is_cursor_mischief_active(&self) -> bool {
        self.current.id() == "nab_mouse"
    }

    /// Whether the active task is reacting to a foreign-window drag.
    pub fn is_perch_ride_active(&self) -> bool {
        self.current.id() == "perch_ride"
    }

    /// Feed one frame of pointer state (cursor + buttons, world space). Detects pats
    /// (hover sweeps → hearts + calm) and a click on the goose (→ a hyper burst next tick).
    pub fn set_pointer(&mut self, pointer: Pointer) {
        if self.is_cursor_mischief_active() || self.is_perch_ride_active() {
            self.pointer = pointer;
            self.prev_left_down = pointer.left_down;
            return;
        }

        let hovering = pointer.present && self.goose_hit(pointer.pos);

        // Pat = hovering hover-sweeps. Each registered pat spawns a heart above the goose.
        let pats = self.pat.update(hovering, pointer.pos, self.elapsed);
        if pats > 0 {
            let head = self.goose.rig.neck_head;
            for _ in 0..pats.min(3) {
                let jitter = Vec2::new(self.rng.range(-7.0, 7.0), self.rng.range(-3.0, 3.0));
                self.hearts.add(head + jitter, self.elapsed);
            }
            self.pending_sounds.push(Sound::Pat);
        }

        // Click = left-button rising edge while hovering → a hyper burst on the next tick.
        let clicked = hovering && pointer.left_down && !self.prev_left_down;
        if clicked {
            if self.options.mouse_steal.active() {
                self.pending_nab = true;
            } else {
                self.pending_hyper = true;
            }
        }

        self.prev_left_down = pointer.left_down;
        self.pointer = pointer;
    }

    /// Interrupt the current task with a hyper burst, saving the prior task to resume later.
    fn start_hyper(&mut self) {
        if self.current.id() == "hyper" {
            return; // already mid-burst; don't stack
        }
        let prev = std::mem::replace(&mut self.current, Box::new(HyperTask::new()));
        self.interrupted = Some(prev);
    }

    /// Interrupt the current task with a cursor nab, saving the prior task to resume later.
    fn start_nab(&mut self) {
        if self.current.id() == "nab_mouse" {
            return; // already stealing the cursor
        }
        let prev = std::mem::replace(&mut self.current, Box::new(NabMouseTask::new()));
        self.interrupted = Some(prev);
    }

    /// Interrupt the current task with a foreign-window perch/ride.
    fn start_perch_ride(&mut self) {
        if self.current.id() == "perch_ride" || self.interrupted.is_some() {
            return; // do not stack transient interrupts
        }
        let prev = std::mem::replace(&mut self.current, Box::new(PerchRideTask::new()));
        self.interrupted = Some(prev);
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

        // A click landed last frame: when cursor stealing is available it takes precedence
        // over the older M6 hyper reaction; otherwise fall back to hyper.
        if self.pending_nab {
            self.pending_nab = false;
            if self.options.mouse_steal.active() && !self.is_cursor_mischief_active() {
                self.pending_hyper = false;
                self.start_nab();
            }
        }

        // Install the hyper burst only when nab did not consume the click.
        if self.pending_hyper && !self.is_cursor_mischief_active() {
            self.pending_hyper = false;
            self.start_hyper();
        } else if self.pending_hyper {
            self.pending_hyper = false;
        }

        if self.options.foreign_window.watch_active()
            && self.dragged_window.is_some()
            && !self.is_cursor_mischief_active()
            && !self.is_perch_ride_active()
        {
            self.start_perch_ride();
        }

        // Run the current task (it only sets targets/params); pick the next when it's done.
        let calm = self.pat.is_calm(self.elapsed);
        let done = {
            let mut ctx = TaskCtx {
                now: self.elapsed,
                dt: DT,
                bounds: self.bounds,
                rng: &mut self.rng,
                sounds: &mut self.pending_sounds,
                cursor_commands: &mut self.pending_cursor_commands,
                pointer: self.pointer,
                mouse_steal: self.options.mouse_steal,
                foreign_window: self.options.foreign_window,
                dragged_window: self.dragged_window,
                calm,
            };
            self.current.run(&mut self.goose, &mut ctx)
        };
        if done {
            // A finished interrupt resumes the task it suspended; otherwise draw next.
            self.current = match self.interrupted.take() {
                Some(prev) => prev,
                None => self.next_task(),
            };
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
    use crate::cursor::MouseStealOptions;
    use crate::foreign_window::{ForeignWindowId, ForeignWindowOptions};

    fn bounds() -> Rect {
        Rect {
            min: Vec2::new(0.0, 0.0),
            max: Vec2::new(1000.0, 800.0),
        }
    }

    fn window_snapshot(id: u64, anchor: Vec2) -> ForeignWindowSnapshot {
        ForeignWindowSnapshot::top_center(
            ForeignWindowId(id),
            Rect {
                min: Vec2::new(anchor.x - 150.0, anchor.y),
                max: Vec2::new(anchor.x + 150.0, anchor.y + 180.0),
            },
        )
    }

    fn world_with_window_watch(seed: u64) -> World {
        let mut w = World::with_options(
            bounds(),
            seed,
            WorldOptions {
                foreign_window: ForeignWindowOptions::with_backend_support(true, false),
                ..WorldOptions::default()
            },
        );
        w.current = Box::new(WanderTask::new());
        w
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

    /// Sweep the cursor back and forth over the goose `strokes` times, hovering throughout.
    fn pat_the_goose(w: &mut World, strokes: usize) {
        let anchor = w.goose.rig.body_center;
        // Baseline frame so the first real move has a previous position to measure from.
        w.set_pointer(Pointer {
            pos: anchor,
            present: true,
            left_down: false,
        });
        for i in 0..strokes {
            let dx = if i % 2 == 0 { 6.0 } else { -6.0 };
            w.set_pointer(Pointer {
                pos: anchor + Vec2::new(dx, 0.0),
                present: true,
                left_down: false,
            });
        }
    }

    #[test]
    fn hovering_sweeps_pat_the_goose_spawning_hearts_and_calm() {
        let mut w = World::new(bounds(), 1);
        pat_the_goose(&mut w, 12);
        assert!(
            w.hearts().alive_count(w.now()) >= 1,
            "patting spawns heart particles"
        );
        assert!(w.is_calm(), "patting calms the goose");
    }

    #[test]
    fn cursor_off_the_goose_does_not_pat() {
        let mut w = World::new(bounds(), 1);
        let away = w.bounds.max + Vec2::new(50.0, 50.0); // well outside the goose
        w.set_pointer(Pointer {
            pos: away,
            present: true,
            left_down: false,
        });
        for i in 0..12 {
            let dx = if i % 2 == 0 { 20.0 } else { -20.0 };
            w.set_pointer(Pointer {
                pos: away + Vec2::new(dx, 0.0),
                present: true,
                left_down: false,
            });
        }
        assert_eq!(w.hearts().alive_count(w.now()), 0, "no pats off the goose");
        assert!(!w.is_calm());
    }

    #[test]
    fn clicking_the_goose_triggers_hyper_then_resumes_prior_task() {
        let mut w = World::new(bounds(), 5);
        // Warm up into the roaming wander task.
        for _ in 0..6_000 {
            w.tick();
            if w.current_task() == "wander" {
                break;
            }
        }
        assert_eq!(w.current_task(), "wander");

        // Default engine options do not assume cursor warp support, so click falls back to
        // the M6 hyper behavior: release/idle baseline frame, then the press edge.
        let anchor = w.goose.rig.body_center;
        w.set_pointer(Pointer {
            pos: anchor,
            present: true,
            left_down: false,
        });
        w.set_pointer(Pointer {
            pos: anchor,
            present: true,
            left_down: true,
        });
        w.tick();
        assert_eq!(w.current_task(), "hyper", "a click sends the goose hyper");

        // After the burst it resumes the task it interrupted.
        for _ in 0..(120 * 3) {
            w.tick();
        }
        assert_eq!(
            w.current_task(),
            "wander",
            "the hyper burst resumes the prior task"
        );
    }

    #[test]
    fn clicking_the_goose_triggers_nab_when_mouse_steal_is_supported() {
        let mut w = World::with_options(
            bounds(),
            8,
            WorldOptions {
                mouse_steal: MouseStealOptions::with_backend_support(true),
                ..WorldOptions::default()
            },
        );
        // Warm up into roaming so this verifies a normal user click, not first-run setup.
        for _ in 0..6_000 {
            w.tick();
            if w.current_task() == "wander" {
                break;
            }
        }
        assert_eq!(w.current_task(), "wander");

        let anchor = w.goose.rig.body_center;
        w.set_pointer(Pointer {
            pos: anchor,
            present: true,
            left_down: false,
        });
        w.set_pointer(Pointer {
            pos: anchor,
            present: true,
            left_down: true,
        });
        w.tick();

        assert_eq!(
            w.current_task(),
            "nab_mouse",
            "with cursor warp support, clicking the goose should steal the cursor instead of hyper"
        );
        assert!(
            !w.take_cursor_commands().is_empty(),
            "click-triggered nab should emit a cursor warp command"
        );
        assert_eq!(
            w.take_sounds(),
            vec![Sound::Bite],
            "click-triggered nab bites when it catches the cursor"
        );
    }

    #[test]
    fn clicking_away_from_the_goose_does_not_trigger_hyper() {
        let mut w = World::new(bounds(), 6);
        let away = w.bounds.max + Vec2::new(50.0, 50.0);
        w.set_pointer(Pointer {
            pos: away,
            present: true,
            left_down: false,
        });
        w.set_pointer(Pointer {
            pos: away,
            present: true,
            left_down: true,
        });
        w.tick();
        assert_ne!(
            w.current_task(),
            "hyper",
            "clicks off the goose pass through"
        );
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

    #[test]
    fn nab_is_pickable_only_when_enabled_and_supported() {
        let default_world = World::new(bounds(), 1);
        assert_eq!(
            default_world.pickable.len(),
            1,
            "default engine options do not assume cursor warp support"
        );

        let disabled = World::with_options(
            bounds(),
            1,
            WorldOptions {
                mouse_steal: MouseStealOptions {
                    enabled: false,
                    warp_supported: true,
                    ..MouseStealOptions::default()
                },
                ..WorldOptions::default()
            },
        );
        assert_eq!(disabled.pickable.len(), 1);

        let supported = World::with_options(
            bounds(),
            1,
            WorldOptions {
                mouse_steal: MouseStealOptions::with_backend_support(true),
                ..WorldOptions::default()
            },
        );
        assert_eq!(
            supported.pickable.len(),
            2,
            "nab_mouse joins roaming only when the backend can warp the cursor"
        );
    }

    #[test]
    fn cursor_commands_are_queued_and_drained_once() {
        let mut w = World::with_options(
            bounds(),
            9,
            WorldOptions {
                mouse_steal: MouseStealOptions::with_backend_support(true),
                ..WorldOptions::default()
            },
        );
        w.current = Box::new(NabMouseTask::new());
        let pointer = w.goose.rig.beak_tip;
        w.set_pointer(Pointer {
            pos: pointer,
            present: true,
            left_down: false,
        });
        w.tick();

        assert_eq!(
            w.take_sounds(),
            vec![Sound::Bite],
            "nab emits the bite sound when it grabs"
        );
        assert_eq!(
            w.take_cursor_commands(),
            vec![CursorCommand::WarpTo(pointer)],
            "nab emits a platform-free cursor warp"
        );
        assert!(
            w.take_cursor_commands().is_empty(),
            "cursor commands drain exactly once"
        );
    }

    #[test]
    fn nab_suppresses_pat_and_click_hyper_interactions() {
        let mut w = World::with_options(
            bounds(),
            10,
            WorldOptions {
                mouse_steal: MouseStealOptions::with_backend_support(true),
                ..WorldOptions::default()
            },
        );
        w.current = Box::new(NabMouseTask::new());

        let anchor = w.goose.rig.body_center;
        for i in 0..12 {
            let dx = if i % 2 == 0 { 6.0 } else { -6.0 };
            w.set_pointer(Pointer {
                pos: anchor + Vec2::new(dx, 0.0),
                present: true,
                left_down: false,
            });
        }
        assert_eq!(
            w.hearts().alive_count(w.now()),
            0,
            "synthetic cursor movement during nab must not pat the goose"
        );

        w.set_pointer(Pointer {
            pos: anchor,
            present: true,
            left_down: true,
        });
        w.tick();
        assert_ne!(
            w.current_task(),
            "hyper",
            "click edges during nab must not interrupt into hyper"
        );
    }

    #[test]
    fn foreign_window_drag_does_not_start_without_watch_capability() {
        let mut w = World::new(bounds(), 11);
        w.current = Box::new(WanderTask::new());
        w.set_foreign_window_drag(Some(window_snapshot(1, Vec2::new(600.0, 100.0))));
        w.tick();
        assert_ne!(
            w.current_task(),
            "perch_ride",
            "default engine options do not assume foreign-window watch support"
        );
    }

    #[test]
    fn foreign_window_drag_interrupts_and_release_before_arrival_resumes() {
        let mut w = world_with_window_watch(12);
        assert_eq!(w.current_task(), "wander");

        w.set_foreign_window_drag(Some(window_snapshot(2, Vec2::new(900.0, 80.0))));
        w.tick();
        assert_eq!(w.current_task(), "perch_ride");

        w.set_foreign_window_drag(None);
        w.tick();
        assert_eq!(
            w.current_task(),
            "wander",
            "releasing before arrival resumes the interrupted task"
        );
    }

    #[test]
    fn foreign_window_drag_rides_moving_anchor_until_release() {
        let mut w = world_with_window_watch(13);
        let first_anchor = Vec2::new(420.0, 90.0);
        w.goose.position = first_anchor + Vec2::new(1.0, 1.0);

        w.set_foreign_window_drag(Some(window_snapshot(3, first_anchor)));
        w.tick();
        assert_eq!(w.current_task(), "perch_ride");
        assert_eq!(w.goose.position, first_anchor);

        let moved_anchor = Vec2::new(500.0, 110.0);
        w.set_foreign_window_drag(Some(window_snapshot(3, moved_anchor)));
        w.tick();
        assert_eq!(w.current_task(), "perch_ride");
        assert_eq!(w.goose.position, moved_anchor);
        assert_eq!(w.goose.velocity, Vec2::ZERO);

        w.set_foreign_window_drag(None);
        w.tick();
        assert_eq!(w.current_task(), "wander");
    }

    #[test]
    fn foreign_window_watch_capability_loss_abandons_cleanly() {
        let mut w = world_with_window_watch(14);
        let anchor = Vec2::new(430.0, 100.0);
        w.goose.position = anchor + Vec2::new(1.0, 0.0);
        w.set_foreign_window_drag(Some(window_snapshot(4, anchor)));
        w.tick();
        assert_eq!(w.current_task(), "perch_ride");

        w.set_foreign_window_watch_supported(false);
        w.tick();
        assert_eq!(w.current_task(), "wander");
    }

    #[test]
    fn perch_ride_suppresses_pat_and_click_hyper_interactions() {
        let mut w = world_with_window_watch(15);
        let anchor = w.goose.rig.body_center;
        w.set_foreign_window_drag(Some(window_snapshot(5, anchor)));
        w.tick();
        assert_eq!(w.current_task(), "perch_ride");

        for i in 0..12 {
            let dx = if i % 2 == 0 { 6.0 } else { -6.0 };
            w.set_pointer(Pointer {
                pos: anchor + Vec2::new(dx, 0.0),
                present: true,
                left_down: false,
            });
        }
        assert_eq!(
            w.hearts().alive_count(w.now()),
            0,
            "cursor motion during perch/ride must not pat the goose"
        );

        w.set_pointer(Pointer {
            pos: anchor,
            present: true,
            left_down: true,
        });
        w.tick();
        assert_eq!(
            w.current_task(),
            "perch_ride",
            "click edges during perch/ride must not interrupt into hyper"
        );
    }
}
