//! Fixed-timestep clock primitives.
//!
//! The original engine runs a fixed 120 Hz simulation tick driven by a `Stopwatch`;
//! all locomotion/step constants are tuned to that rate, so the sim must never couple
//! to the redraw rate. The accumulator below is used by the desktop runtime to run the
//! platform-free engine at 120 Hz while presentation happens at its own capped cadence.

/// Simulation tick rate, in Hz.
pub const FRAMERATE: u32 = 120;

/// Seconds per simulation tick (`1 / FRAMERATE`).
pub const DT: f32 = 1.0 / FRAMERATE as f32;

/// A monotonic wall-clock, mirroring the original's `Stopwatch`-backed `Time.time`.
///
/// Headless and side-effect-free apart from reading the monotonic clock; tests that
/// need determinism drive the engine with [`DT`] directly rather than this type.
#[derive(Debug)]
pub struct Clock {
    start: std::time::Instant,
}

impl Clock {
    /// Start a clock at "now".
    pub fn start() -> Self {
        Self {
            start: std::time::Instant::now(),
        }
    }

    /// Seconds elapsed since [`Clock::start`].
    pub fn elapsed_secs(&self) -> f32 {
        self.start.elapsed().as_secs_f32()
    }
}

/// Fixed-timestep accumulator. Feed it real elapsed seconds; it tells you how many whole
/// [`DT`] simulation ticks to run, decoupling the sim from the present rate. Catch-up is
/// clamped so a long stall (debugger pause, sleep) can't trigger a "spiral of death".
#[derive(Debug)]
pub struct Accumulator {
    acc: f32,
    max_catchup_ticks: u32,
}

impl Default for Accumulator {
    fn default() -> Self {
        Self::new()
    }
}

impl Accumulator {
    /// Most sim ticks run from a single [`Accumulator::pump`] before the backlog is dropped.
    pub const DEFAULT_MAX_CATCHUP: u32 = 5;

    /// A fresh accumulator with the default catch-up clamp.
    pub fn new() -> Self {
        Self {
            acc: 0.0,
            max_catchup_ticks: Self::DEFAULT_MAX_CATCHUP,
        }
    }

    /// Add `elapsed` real seconds and return how many [`DT`] ticks to run now. Fractional
    /// leftover carries to the next pump (smooth timing); a backlog beyond the catch-up
    /// clamp is discarded so the sim never falls permanently behind. Negative input is
    /// ignored.
    pub fn pump(&mut self, elapsed: f32) -> u32 {
        self.acc += elapsed.max(0.0);
        let mut ticks = 0;
        while self.acc >= DT && ticks < self.max_catchup_ticks {
            self.acc -= DT;
            ticks += 1;
        }
        // Hit the clamp with ticks still owed ⇒ drop the backlog rather than spiral.
        if self.acc >= DT {
            self.acc = 0.0;
        }
        ticks
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn timestep_matches_original() {
        assert_eq!(FRAMERATE, 120);
        // 1/120 s per tick.
        assert!((DT - 1.0 / 120.0).abs() < 1e-9);
    }

    #[test]
    fn clock_is_monotonic() {
        let c = Clock::start();
        assert!(c.elapsed_secs() >= 0.0);
    }

    #[test]
    fn accumulator_yields_whole_ticks() {
        let mut acc = Accumulator::new();
        assert_eq!(acc.pump(DT * 3.0), 3);
        // Fractional input accumulates across pumps.
        assert_eq!(acc.pump(DT * 0.5), 0);
        assert_eq!(acc.pump(DT * 0.5), 1);
    }

    #[test]
    fn accumulator_clamps_catchup() {
        let mut acc = Accumulator::new();
        // One full second (~120 ticks) is clamped to the catch-up max, backlog dropped.
        assert_eq!(acc.pump(1.0), Accumulator::DEFAULT_MAX_CATCHUP);
        // Backlog was discarded, so the next small pump starts fresh.
        assert_eq!(acc.pump(DT * 0.4), 0);
    }

    #[test]
    fn accumulator_ignores_negative() {
        let mut acc = Accumulator::new();
        assert_eq!(acc.pump(-5.0), 0);
        assert_eq!(acc.pump(DT), 1);
    }
}
