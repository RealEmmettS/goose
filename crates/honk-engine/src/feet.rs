//! Procedural feet placement.
//!
//! Verified constants (`Exports.cs`, `ProceduralFeets`): `feetDistanceApart = 6`,
//! `wantStepAtDistance = 5`, `overshootFraction = 0.4`. M0 owns the data and the
//! at-rest placement; the stepping/overshoot animation that consumes
//! [`WANT_STEP_AT_DISTANCE`] / [`OVERSHOOT_FRACTION`] lands in M2.

use crate::math::Vec2;

/// Lateral spacing between the two feet.
pub const FEET_DISTANCE_APART: f32 = 6.0;
/// How far a foot may drift from its target before it takes a step (M2).
pub const WANT_STEP_AT_DISTANCE: f32 = 5.0;
/// Fraction a stepping foot overshoots its target by (M2).
pub const OVERSHOOT_FRACTION: f32 = 0.4;

/// The two procedurally-placed feet and their in-flight step state.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ProceduralFeet {
    /// Left foot position.
    pub left: Vec2,
    /// Right foot position.
    pub right: Vec2,
    /// Wall-clock time the left foot's current step started (`< 0` ⇒ planted).
    pub left_step_start: f32,
    /// Wall-clock time the right foot's current step started (`< 0` ⇒ planted).
    pub right_step_start: f32,
}

impl Default for ProceduralFeet {
    fn default() -> Self {
        Self::at_rest(Vec2::new(300.0, 300.0), Vec2::new(0.0, 1.0))
    }
}

impl ProceduralFeet {
    /// Place both feet at rest, straddling `center`, spaced [`FEET_DISTANCE_APART`]
    /// apart perpendicular to `forward`.
    pub fn at_rest(center: Vec2, forward: Vec2) -> Self {
        let lateral = forward.perpendicular() * (FEET_DISTANCE_APART * 0.5);
        Self {
            left: center - lateral,
            right: center + lateral,
            left_step_start: -1.0,
            right_step_start: -1.0,
        }
    }
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
    fn rest_feet_straddle_center() {
        let feet = ProceduralFeet::at_rest(Vec2::new(100.0, 100.0), Vec2::new(0.0, 1.0));
        // forward = +y ⇒ perpendicular = (-1, 0); feet split along x by 6 total.
        assert_eq!(feet.left, Vec2::new(103.0, 100.0));
        assert_eq!(feet.right, Vec2::new(97.0, 100.0));
        assert_eq!(Vec2::distance(feet.left, feet.right), FEET_DISTANCE_APART);
        assert!(feet.left_step_start < 0.0 && feet.right_step_start < 0.0);
    }
}
