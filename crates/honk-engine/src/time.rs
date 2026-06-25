//! Fixed-timestep clock primitives.
//!
//! The original engine runs a fixed 120 Hz simulation tick driven by a `Stopwatch`;
//! all locomotion/step constants are tuned to that rate, so the sim must never couple
//! to the redraw rate. Round 1 (M0) ships only the constants and a thin [`Clock`]; the
//! accumulator run-loop that consumes them is M2.

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
}
