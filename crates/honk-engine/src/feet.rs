//! Procedural feet — the walking gait.
//!
//! Verified constants (`Exports.cs`, `ProceduralFeets`): `feetDistanceApart = 6`,
//! `wantStepAtDistance = 5`, `overshootFraction = 0.4`. The original plants each foot and
//! swings it forward (with overshoot) once it lags too far behind the body. We reproduce
//! the *look* of that gait clean-room with a distance-driven phase: the two feet swing in
//! antiphase along the facing direction and lift on their forward swing, giving a waddle.
//! The phase only advances while the goose moves, so a stopped goose stands still.

use crate::math::Vec2;

/// Lateral spacing between the two feet (for the slight two-foot offset in side view).
pub const FEET_DISTANCE_APART: f32 = 6.0;
/// How far a foot lags before it wants to step — drives the stride length.
pub const WANT_STEP_AT_DISTANCE: f32 = 5.0;
/// Fraction a stepping foot overshoots by — drives how far the swing reaches.
pub const OVERSHOOT_FRACTION: f32 = 0.4;

/// Stride half-reach along the facing direction.
const STRIDE: f32 = WANT_STEP_AT_DISTANCE * (1.0 + OVERSHOOT_FRACTION); // 7.0

/// Maximum foot lift during a forward swing.
const LIFT: f32 = 3.0;

const UP: Vec2 = Vec2 { x: 0.0, y: -1.0 };

/// The two foot positions for a given gait phase.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Feet {
    pub left: Vec2,
    pub right: Vec2,
}

/// Foot positions for a goose whose feet sit at `center`, facing `forward`, at gait
/// `phase` (radians, advanced by distance travelled). The feet swing in antiphase along
/// `forward`, offset slightly across it so both read in side view, and lift while swinging.
pub fn gait(center: Vec2, forward: Vec2, phase: f32) -> Feet {
    // Resting fore/aft stagger so both feet read even when the goose stands still.
    const STANCE: f32 = 3.0;
    let across = forward.perpendicular() * (FEET_DISTANCE_APART * 0.5);
    let swing_l = phase.sin();
    let swing_r = (phase + std::f32::consts::PI).sin();
    let left =
        center + across + forward * (STANCE + swing_l * STRIDE) + UP * (swing_l.max(0.0) * LIFT);
    let right =
        center - across + forward * (-STANCE + swing_r * STRIDE) + UP * (swing_r.max(0.0) * LIFT);
    Feet { left, right }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn constants_match_verified_source() {
        assert_eq!(FEET_DISTANCE_APART, 6.0);
        assert_eq!(WANT_STEP_AT_DISTANCE, 5.0);
        assert_eq!(OVERSHOOT_FRACTION, 0.4);
    }

    #[test]
    fn feet_swing_in_antiphase() {
        // Quarter phase: left foot forward, right foot back (along +x facing).
        let f = gait(Vec2::ZERO, Vec2::new(1.0, 0.0), std::f32::consts::FRAC_PI_2);
        assert!(f.left.x > 0.5, "left should be swung forward");
        assert!(f.right.x < -0.5, "right should be swung back");
    }

    #[test]
    fn feet_are_deterministic() {
        let a = gait(Vec2::new(10.0, 10.0), Vec2::new(0.0, 1.0), 1.3);
        let b = gait(Vec2::new(10.0, 10.0), Vec2::new(0.0, 1.0), 1.3);
        assert_eq!(a, b);
    }

    #[test]
    fn feet_straddle_across_facing() {
        // At phase 0 both feet are at their neutral fore/aft, separated across `forward`.
        let f = gait(Vec2::ZERO, Vec2::new(1.0, 0.0), 0.0);
        assert!(Vec2::distance(f.left, f.right) >= FEET_DISTANCE_APART - 1e-3);
    }
}
