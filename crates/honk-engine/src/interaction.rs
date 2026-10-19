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
pub const STREAK_TIMEOUT: f64 = 1.2;
/// Each registered pat keeps the goose calm for this long (seconds).
pub const CALM_DURATION: f64 = 4.0;
/// Actual repeated rubbing needed before petting reactions and the friendly follow invitation.
pub const AFFECTION_RUB_SECONDS: f64 = 2.5;
const RUB_PAUSE_SECONDS: f64 = 0.3;

/// Detects pats from successive hovering cursor positions and tracks the happy streak.
#[derive(Debug, Clone)]
pub struct PatTracker {
    streak: u32,
    sweep_accum: f32,
    last_pos: Option<Vec2>,
    last_pat_time: f64,
    calm_until: f64,
    last_motion_time: f64,
    rub_seconds: f64,
    follow_requested: bool,
    follow_issued: bool,
}

impl Default for PatTracker {
    fn default() -> Self {
        Self {
            streak: 0,
            sweep_accum: 0.0,
            last_pos: None,
            last_pat_time: f64::NEG_INFINITY,
            calm_until: f64::NEG_INFINITY,
            last_motion_time: f64::NEG_INFINITY,
            rub_seconds: 0.0,
            follow_requested: false,
            follow_issued: false,
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
    pub fn is_calm(&self, now: f64) -> bool {
        now < self.calm_until
    }

    /// One invitation per sustained rubbing streak; holding still never starts a follow.
    pub fn take_follow_request(&mut self) -> bool {
        std::mem::take(&mut self.follow_requested)
    }

    /// Feed one frame of pointer state. `hovering` is whether the cursor is over the goose.
    /// Returns pats only after sustained rubbing has earned the first petting reaction.
    pub fn update(&mut self, hovering: bool, pos: Vec2, now: f64) -> u32 {
        if now - self.last_motion_time > RUB_PAUSE_SECONDS {
            self.rub_seconds = 0.0;
            self.sweep_accum = 0.0;
            self.follow_requested = false;
            self.follow_issued = false;
        }
        // Let the visible happy streak expire after the last earned pat.
        if now - self.last_pat_time > STREAK_TIMEOUT {
            self.streak = 0;
        }

        if !hovering {
            // Off the goose: drop the baseline and the partial sweep so re-entry starts clean.
            self.last_pos = None;
            self.sweep_accum = 0.0;
            self.rub_seconds = 0.0;
            self.last_motion_time = f64::NEG_INFINITY;
            self.follow_requested = false;
            self.follow_issued = false;
            return 0;
        }

        let mut pats = 0;
        if let Some(prev) = self.last_pos {
            let moved = Vec2::distance(prev, pos);
            if moved >= 0.25 {
                let delta = now - self.last_motion_time;
                if delta <= RUB_PAUSE_SECONDS {
                    self.rub_seconds += delta.clamp(0.0, 0.05);
                }
                self.last_motion_time = now;
                if self.rub_seconds >= AFFECTION_RUB_SECONDS && !self.follow_issued {
                    self.follow_requested = true;
                    self.follow_issued = true;
                    // Start with one visible reaction when intentional rubbing is earned.
                    // Earlier movement must never accumulate a burst of deferred hearts.
                    self.sweep_accum = SWEEP_PER_PAT;
                }
            }
            if self.rub_seconds < AFFECTION_RUB_SECONDS {
                self.last_pos = Some(pos);
                return 0;
            }
            self.sweep_accum += moved;
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

    fn rub(t: &mut PatTracker, start: f64, frames: usize) -> u32 {
        (0..frames)
            .map(|frame| {
                t.update(
                    true,
                    Vec2::new((frame % 2) as f32 * 10.0, 0.0),
                    start + frame as f64 / 120.0,
                )
            })
            .sum()
    }

    #[test]
    fn sustained_rubbing_builds_streak_and_calms() {
        let mut t = PatTracker::new();
        let pats = rub(&mut t, 0.0, 360);
        assert!(pats >= 1, "sustained rubbing should register pats");
        assert!(t.streak() >= 1);
        assert!(t.is_calm(3.0), "patting calms the goose");
    }

    #[test]
    fn quick_swipes_and_stationary_hover_never_start_petting() {
        let mut tracker = PatTracker::new();
        for frame in 0..120 {
            assert_eq!(
                tracker.update(
                    true,
                    Vec2::new(frame as f32 * 30.0, 0.0),
                    frame as f64 / 120.0
                ),
                0
            );
        }
        for frame in 120..720 {
            assert_eq!(
                tracker.update(true, Vec2::new(3570.0, 0.0), frame as f64 / 120.0),
                0
            );
        }
        assert_eq!(tracker.streak(), 0);
        assert!(!tracker.is_calm(6.0));
        assert!(!tracker.take_follow_request());
    }

    #[test]
    fn first_pet_requires_two_and_a_half_seconds_at_supported_poll_rates() {
        for hz in [30, 60, 120, 144] {
            let mut tracker = PatTracker::new();
            let mut first_pat = None;
            for frame in 0..hz * 3 {
                let now = frame as f64 / hz as f64;
                let pats = tracker.update(true, Vec2::new((frame % 2) as f32 * 10.0, 0.0), now);
                if pats > 0 {
                    first_pat = Some(now);
                    assert!(pats <= 1, "no backlog of unearned pats");
                    break;
                }
                assert_eq!(tracker.streak(), 0);
                assert!(!tracker.is_calm(now));
            }
            let first_pat = first_pat.expect("intentional rubbing earns a reaction");
            assert!((2.5..2.6).contains(&first_pat), "{hz} Hz: {first_pat}");
        }
    }

    #[test]
    fn leaving_the_goose_or_pausing_requires_a_fresh_rub() {
        for leave in [false, true] {
            let mut tracker = PatTracker::new();
            assert_eq!(rub(&mut tracker, 0.0, 240), 0);
            tracker.update(!leave, Vec2::new(10.0, 0.0), if leave { 2.0 } else { 2.5 });
            assert_eq!(rub(&mut tracker, 2.6, 240), 0);
            assert_eq!(tracker.streak(), 0);
            assert!(!tracker.take_follow_request());
            assert!(rub(&mut tracker, 4.6, 120) > 0);
        }
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
        rub(&mut t, 0.0, 360);
        assert!(t.streak() >= 1);
        // Long pause with no pats → the streak lapses.
        t.update(false, Vec2::new(0.0, 0.0), 3.0 + STREAK_TIMEOUT + 0.5);
        assert_eq!(t.streak(), 0, "streak should lapse after the timeout");
    }

    #[test]
    fn calm_window_expires() {
        let mut t = PatTracker::new();
        rub(&mut t, 0.0, 360);
        assert!(t.is_calm(3.0));
        assert!(!t.is_calm(3.0 + CALM_DURATION + 1.0), "calm wears off");
    }

    #[test]
    fn affection_requires_timed_motion_and_only_invites_once_per_streak() {
        let mut tracker = PatTracker::new();
        for frame in 0..400 {
            tracker.update(true, Vec2::ZERO, frame as f64 / 120.0);
            assert!(!tracker.take_follow_request());
        }
        let mut invitations = 0;
        for frame in 0..600 {
            tracker.update(
                true,
                Vec2::new((frame % 2) as f32 * 10.0, 0.0),
                4.0 + frame as f64 / 120.0,
            );
            let invited = tracker.take_follow_request();
            if frame < 300 {
                assert!(!invited);
            }
            invitations += usize::from(invited);
        }
        assert_eq!(invitations, 1);
        tracker.update(false, Vec2::ZERO, 10.0);
        for frame in 0..240 {
            tracker.update(
                true,
                Vec2::new((frame % 2) as f32 * 10.0, 0.0),
                10.0 + frame as f64 / 120.0,
            );
            assert!(
                !tracker.take_follow_request(),
                "a new streak must earn its own duration"
            );
        }
    }
}
