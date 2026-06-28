//! Muddy footprints: a fixed 64-slot ring buffer of fading marks.
//!
//! Verified constants (`Exports.cs`, `FootMark`): `Lifetime = 8.5 s`, `ShrinkTime = 1 s`,
//! ring buffer length 64. A mark renders full-size for its first `Lifetime - ShrinkTime`
//! seconds, then shrinks to nothing over the final `ShrinkTime`.

use crate::math::Vec2;

/// Total lifetime of a footmark, in seconds.
pub const LIFETIME: f32 = 8.5;
/// Duration of the shrink-out at the end of life, in seconds.
pub const SHRINK_TIME: f32 = 1.0;
/// Ring-buffer capacity.
pub const CAPACITY: usize = 64;

/// Runtime mud-print lifetime options. Defaults match the verified original constants.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FootMarkTiming {
    pub lifetime: f32,
    pub shrink_time: f32,
}

impl Default for FootMarkTiming {
    fn default() -> Self {
        Self {
            lifetime: LIFETIME,
            shrink_time: SHRINK_TIME,
        }
    }
}

/// A single muddy print: where it was left and when.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FootMark {
    /// World position of the print.
    pub position: Vec2,
    /// Wall-clock time the print was created.
    pub created: f32,
}

impl FootMark {
    /// Remaining-life scale in `[0, 1]` at time `now`: `1.0` while fresh, ramping to
    /// `0.0` across the final [`SHRINK_TIME`] seconds; `0.0` once dead.
    pub fn scale(&self, now: f32) -> f32 {
        self.scale_with_timing(now, FootMarkTiming::default())
    }

    /// Remaining-life scale using explicit runtime timing.
    pub fn scale_with_timing(&self, now: f32, timing: FootMarkTiming) -> f32 {
        let lifetime = timing.lifetime.max(0.001);
        let shrink_time = timing.shrink_time.max(0.001).min(lifetime);
        let remaining = lifetime - (now - self.created);
        if remaining <= 0.0 {
            0.0
        } else if remaining >= shrink_time {
            1.0
        } else {
            remaining / shrink_time
        }
    }

    /// Whether the print is still within its lifetime at `now`.
    pub fn is_alive(&self, now: f32) -> bool {
        self.is_alive_with_timing(now, FootMarkTiming::default())
    }

    /// Whether the print is still alive using explicit runtime timing.
    pub fn is_alive_with_timing(&self, now: f32, timing: FootMarkTiming) -> bool {
        now - self.created <= timing.lifetime.max(0.001)
    }
}

/// A 64-slot ring buffer of footprints. Oldest prints are overwritten once full.
#[derive(Debug, Clone)]
pub struct FootMarks {
    slots: [Option<FootMark>; CAPACITY],
    next: usize,
}

impl Default for FootMarks {
    fn default() -> Self {
        Self::new()
    }
}

impl FootMarks {
    /// An empty buffer.
    pub fn new() -> Self {
        Self {
            slots: [None; CAPACITY],
            next: 0,
        }
    }

    /// Drop a new print at `position`, created at `now`. Overwrites the oldest slot
    /// when the buffer is full.
    pub fn add(&mut self, position: Vec2, now: f32) {
        self.slots[self.next] = Some(FootMark {
            position,
            created: now,
        });
        self.next = (self.next + 1) % CAPACITY;
    }

    /// All prints still alive at `now`, with their current [`FootMark::scale`].
    pub fn active(&self, now: f32) -> impl Iterator<Item = (FootMark, f32)> + '_ {
        self.active_with_timing(now, FootMarkTiming::default())
    }

    /// All prints still alive at `now`, using explicit runtime timing.
    pub fn active_with_timing(
        &self,
        now: f32,
        timing: FootMarkTiming,
    ) -> impl Iterator<Item = (FootMark, f32)> + '_ {
        self.slots
            .iter()
            .filter_map(|s| *s)
            .filter(move |m| m.is_alive_with_timing(now, timing))
            .map(move |m| (m, m.scale_with_timing(now, timing)))
    }

    /// Count of prints alive at `now`.
    pub fn alive_count(&self, now: f32) -> usize {
        self.active(now).count()
    }

    /// Count of prints alive at `now`, using explicit runtime timing.
    pub fn alive_count_with_timing(&self, now: f32, timing: FootMarkTiming) -> usize {
        self.active_with_timing(now, timing).count()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lifetime_constants_match_verified_source() {
        assert_eq!(LIFETIME, 8.5);
        assert_eq!(SHRINK_TIME, 1.0);
        assert_eq!(CAPACITY, 64);
        assert_eq!(
            FootMarkTiming::default(),
            FootMarkTiming {
                lifetime: 8.5,
                shrink_time: 1.0
            }
        );
    }

    #[test]
    fn scale_at_boundaries() {
        let m = FootMark {
            position: Vec2::ZERO,
            created: 0.0,
        };
        assert_eq!(m.scale(0.0), 1.0); // fresh
        assert_eq!(m.scale(7.5), 1.0); // last full-size instant
        assert_eq!(m.scale(8.0), 0.5); // halfway through the shrink
        assert_eq!(m.scale(8.5), 0.0); // dead
        assert_eq!(m.scale(100.0), 0.0); // long dead
    }

    #[test]
    fn custom_timing_controls_scale_and_alive_window() {
        let timing = FootMarkTiming {
            lifetime: 4.0,
            shrink_time: 2.0,
        };
        let m = FootMark {
            position: Vec2::ZERO,
            created: 0.0,
        };
        assert!(m.is_alive_with_timing(4.0, timing));
        assert!(!m.is_alive_with_timing(4.1, timing));
        assert_eq!(m.scale_with_timing(2.0, timing), 1.0);
        assert_eq!(m.scale_with_timing(3.0, timing), 0.5);
    }

    #[test]
    fn alive_window() {
        let m = FootMark {
            position: Vec2::ZERO,
            created: 0.0,
        };
        assert!(m.is_alive(8.5));
        assert!(!m.is_alive(8.6));
    }

    #[test]
    fn ring_buffer_wraps_at_capacity() {
        let mut marks = FootMarks::new();
        // Add 65 marks all "now=0"; the buffer holds at most 64.
        for i in 0..(CAPACITY + 1) {
            marks.add(Vec2::new(i as f32, 0.0), 0.0);
        }
        assert_eq!(marks.alive_count(0.0), CAPACITY);
        // The very first mark (x=0) was overwritten by the 65th (x=64).
        let xs: Vec<f32> = marks.active(0.0).map(|(m, _)| m.position.x).collect();
        assert!(
            !xs.contains(&0.0),
            "oldest mark should have been overwritten"
        );
        assert!(xs.contains(&64.0), "newest mark should be present");
    }
}
