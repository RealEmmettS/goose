use honk_config::Config;
#[cfg(any(test, target_os = "macos"))]
use honk_engine::DT;
use honk_engine::{Accumulator, Clock, Rect, World};
#[cfg(any(test, target_os = "macos"))]
use std::time::Duration;

const PRESENT_INTERVAL: f64 = 1.0 / 60.0;

#[derive(Debug, Clone, Copy)]
pub(crate) struct RuntimeFrame {
    id: u64,
    now: f64,
    dt: f64,
}

impl RuntimeFrame {
    pub(crate) fn now(self) -> f64 {
        self.now
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Phase {
    Idle,
    Commands,
    Ticked,
}

/// Shared platform-neutral runtime sequencing.
///
/// Native event pumps and capability I/O remain in each platform module. This core pins the
/// common order: sample the clock, process config/control commands, fixed-step the world, then
/// compute current-vs-previous visual damage at the presentation cadence.
pub(crate) struct RuntimeCore {
    clock: Clock,
    accumulator: Accumulator,
    last_frame_time: f64,
    last_present_time: f64,
    last_visual_bounds: Option<Rect>,
    phase: Phase,
    frame_id: u64,
}

impl RuntimeCore {
    pub(crate) fn new() -> Self {
        let clock = Clock::start();
        let now = clock.elapsed_secs();
        Self {
            clock,
            accumulator: Accumulator::new(),
            last_frame_time: now,
            last_present_time: f64::NEG_INFINITY,
            last_visual_bounds: None,
            phase: Phase::Idle,
            frame_id: 0,
        }
    }

    /// Begin a frame after the platform event pump. Config/control requests must be handled after
    /// this call and before [`Self::tick`].
    pub(crate) fn begin_frame(&mut self) -> RuntimeFrame {
        self.begin_at(self.clock.elapsed_secs())
    }

    #[cfg(target_os = "macos")]
    pub(crate) fn next_tick_delay(&self) -> Duration {
        self.next_tick_delay_at(self.clock.elapsed_secs())
    }

    #[cfg(any(test, target_os = "macos"))]
    fn next_tick_delay_at(&self, now: f64) -> Duration {
        let elapsed = (now - self.last_frame_time).max(0.0);
        Duration::from_secs_f64((DT as f64 - elapsed).clamp(0.0, DT as f64))
    }

    fn begin_at(&mut self, now: f64) -> RuntimeFrame {
        assert_eq!(
            self.phase,
            Phase::Idle,
            "previous runtime frame was not finished"
        );
        let dt = (now - self.last_frame_time).max(0.0);
        self.last_frame_time = now;
        self.frame_id = self.frame_id.wrapping_add(1);
        self.phase = Phase::Commands;
        RuntimeFrame {
            id: self.frame_id,
            now,
            dt,
        }
    }

    pub(crate) fn tick(&mut self, world: &mut World, frame: RuntimeFrame) {
        self.assert_frame(frame, Phase::Commands);
        for _ in 0..self.accumulator.pump(frame.dt) {
            world.tick();
        }
        self.phase = Phase::Ticked;
    }

    /// Finish the frame and return the region that must be repainted, if the present cadence is
    /// due and any current/previous visual pixels need drawing or clearing.
    pub(crate) fn damage(&mut self, world: &World, frame: RuntimeFrame) -> Option<Rect> {
        self.assert_frame(frame, Phase::Ticked);
        self.phase = Phase::Idle;
        if frame.now - self.last_present_time < PRESENT_INTERVAL {
            return None;
        }
        self.last_present_time = frame.now;
        let current = world.visual_bounds();
        let damage = World::damage_bounds(self.last_visual_bounds, current);
        self.last_visual_bounds = current;
        damage
    }

    pub(crate) fn restart_required_reason(current: &Config, next: &Config) -> Option<String> {
        let changes = current.restart_required_changes(next);
        (!changes.is_empty()).then(|| changes.join(", "))
    }

    fn assert_frame(&self, frame: RuntimeFrame, phase: Phase) {
        assert_eq!(frame.id, self.frame_id, "stale runtime frame token");
        assert_eq!(self.phase, phase, "runtime phase ordering violation");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use honk_engine::{Vec2, World};

    #[test]
    fn platform_runtimes_share_clock_tick_and_damage_ordering() {
        let mut core = RuntimeCore::new();
        let mut world = World::new(Rect::new(Vec2::ZERO, Vec2::new(1280.0, 720.0)), 7);
        let start = core.last_frame_time;
        let frame = core.begin_at(start + 1.0 / 60.0);
        core.tick(&mut world, frame);
        assert!(world.now() > 0.0);
        assert!(core.damage(&world, frame).is_some());
        assert_eq!(core.phase, Phase::Idle);
    }

    #[test]
    #[should_panic(expected = "runtime phase ordering violation")]
    fn damage_cannot_run_before_fixed_step_tick() {
        let mut core = RuntimeCore::new();
        let world = World::new(Rect::new(Vec2::ZERO, Vec2::new(1280.0, 720.0)), 7);
        let frame = core.begin_at(core.last_frame_time + 1.0 / 60.0);
        let _ = core.damage(&world, frame);
    }

    #[test]
    fn restart_required_config_is_identified_before_assignment() {
        let current = Config::default();
        let mut next = current.clone();
        next.platform.wayland = !current.platform.wayland;
        assert_eq!(
            RuntimeCore::restart_required_reason(&current, &next).as_deref(),
            Some("platform.wayland")
        );
    }

    #[test]
    fn next_tick_delay_paces_the_loop_to_one_hundred_twenty_hz() {
        let core = RuntimeCore::new();
        let tick = honk_engine::DT as f64;
        let start = core.last_frame_time;

        let initial = core.next_tick_delay_at(start).as_secs_f64();
        assert!((initial - tick).abs() < 1e-6);

        let partial = core.next_tick_delay_at(start + tick * 0.25).as_secs_f64();
        assert!((partial - tick * 0.75).abs() < 1e-6);
        assert_eq!(core.next_tick_delay_at(start + tick * 2.0), Duration::ZERO);
    }
}
