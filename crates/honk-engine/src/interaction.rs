//! Pointer input and the pat (hover-streak) detector (M6, plan §5.9 / §6).
//!
//! Platform-free: a backend polls the cursor each frame and hands the engine a [`Pointer`]
//! snapshot in world space; the engine decides what it means. Two distinct interactions
//! (plan §5.9):
//! - **Pat** = repeated cursor *hover-sweeps* over the goose (no buttons) → builds a streak,
//!   spawns hearts, and keeps the goose briefly calm. Modelled by [`PatTracker`].
//! - **Click** = a left press while over the goose → the hyper reaction (handled in `world`).

use crate::math::Vec2;

/// A per-frame snapshot of the pointer in world space.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Pointer {
    /// Cursor position in world space.
    pub pos: Vec2,
    /// Whether the cursor is present on this overlay/screen at all.
    pub present: bool,
    /// Whether the left mouse button is held this frame.
    pub left_down: bool,
}

impl Default for Pointer {
    fn default() -> Self {
        Self {
            pos: Vec2::ZERO,
            present: false,
            left_down: false,
        }
    }
}

/// Hover-movement (px) that registers one "pat".
pub const SWEEP_PER_PAT: f32 = 28.0;
/// With no new pat within this many seconds, the streak resets to zero.
pub const STREAK_TIMEOUT: f32 = 1.2;
/// Each registered pat keeps the goose calm for this long (seconds).
pub const CALM_DURATION: f32 = 4.0;

/// Detects pats from successive hovering cursor positions and tracks the happy streak.
#[derive(Debug, Clone)]
pub struct PatTracker {
    streak: u32,
    sweep_accum: f32,
    last_pos: Option<Vec2>,
    last_pat_time: f32,
    calm_until: f32,
}

impl Default for PatTracker {
    fn default() -> Self {
        Self {
            streak: 0,
            sweep_accum: 0.0,
            last_pos: None,
            last_pat_time: f32::NEG_INFINITY,
            calm_until: f32::NEG_INFINITY,
        }
    }
}

impl PatTracker {
    /// A fresh tracker.
    pub fn new() -> Self {
        Self::default()
    }

    /// The current happy streak (consecutive pats before the timeout lapses).
    pub fn streak(&self) -> u32 {
        self.streak
    }

    /// Whether the goose is in its post-pat calm window at `now`.
    pub fn is_calm(&self, now: f32) -> bool {
        now < self.calm_until
    }

    /// Feed one frame of pointer state. `hovering` is whether the cursor is over the goose.
    /// Returns how many pats were registered this frame (0 normally, ≥1 on a long sweep).
    pub fn update(&mut self, hovering: bool, pos: Vec2, now: f32) -> u32 {
        // Lapse the streak after a quiet spell (the in-progress sweep keeps accumulating —
        // it only resets when the cursor actually leaves the goose, below).
        if now - self.last_pat_time > STREAK_TIMEOUT {
            self.streak = 0;
        }

        if !hovering {
            // Off the goose: drop the baseline and the partial sweep so re-entry starts clean.
            self.last_pos = None;
            self.sweep_accum = 0.0;
            return 0;
        }

        let mut pats = 0;
        if let Some(prev) = self.last_pos {
            self.sweep_accum += Vec2::distance(prev, pos);
            while self.sweep_accum >= SWEEP_PER_PAT {
                self.sweep_accum -= SWEEP_PER_PAT;
                self.streak += 1;
                self.last_pat_time = now;
                self.calm_until = now + CALM_DURATION;
                pats += 1;
            }
        }
        self.last_pos = Some(pos);
        pats
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Drag the cursor a total of `dist` px across the goose in one straight sweep.
    fn sweep(t: &mut PatTracker, start: Vec2, dist: f32, now: f32) -> u32 {
        // Prime the baseline position (first hovering frame moves nothing).
        t.update(true, start, now);
        t.update(true, start + Vec2::new(dist, 0.0), now)
    }

    #[test]
    fn sweeping_over_goose_builds_streak_and_calms() {
        let mut t = PatTracker::new();
        let pats = sweep(&mut t, Vec2::new(0.0, 0.0), SWEEP_PER_PAT * 1.5, 0.0);
        assert!(pats >= 1, "a long hover-sweep should register a pat");
        assert!(t.streak() >= 1);
        assert!(t.is_calm(0.0), "patting calms the goose");
    }

    #[test]
    fn not_hovering_registers_no_pats() {
        let mut t = PatTracker::new();
        // Even big cursor movement off the goose does nothing.
        t.update(false, Vec2::new(0.0, 0.0), 0.0);
        let pats = t.update(false, Vec2::new(1000.0, 0.0), 0.1);
        assert_eq!(pats, 0);
        assert_eq!(t.streak(), 0);
    }

    #[test]
    fn first_hover_frame_is_only_a_baseline() {
        let mut t = PatTracker::new();
        // The very first hovering frame has no previous position to measure against.
        let pats = t.update(true, Vec2::new(500.0, 500.0), 0.0);
        assert_eq!(pats, 0);
    }

    #[test]
    fn streak_resets_after_timeout() {
        let mut t = PatTracker::new();
        sweep(&mut t, Vec2::new(0.0, 0.0), SWEEP_PER_PAT * 1.2, 0.0);
        assert!(t.streak() >= 1);
        // Long pause with no pats → the streak lapses.
        t.update(false, Vec2::new(0.0, 0.0), STREAK_TIMEOUT + 0.5);
        assert_eq!(t.streak(), 0, "streak should lapse after the timeout");
    }

    #[test]
    fn calm_window_expires() {
        let mut t = PatTracker::new();
        sweep(&mut t, Vec2::new(0.0, 0.0), SWEEP_PER_PAT * 1.2, 0.0);
        assert!(t.is_calm(0.0));
        assert!(!t.is_calm(CALM_DURATION + 1.0), "calm wears off");
    }
}
