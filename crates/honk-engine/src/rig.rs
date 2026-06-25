//! The goose rig: geometry constants and the per-frame body-point computation.
//!
//! The **constants** are verbatim from the verified source (`Exports.cs`, `Rig`). The
//! original draws the goose with the same *technique* (`GooseRenderData`): a stack of
//! filled stadium/capsule forms in white with a thin grey outline pen, an orange beak and
//! feet, and a stippled ground shadow. The exact `updateRig` placement maths live only in
//! the closed binary, so the assembly here is a clean-room reconstruction — but it is
//! **tuned against direct observation of the running original**: the real procedural goose
//! is a soft rounded *blob* whose head is tucked into the front-top of the body (a short
//! neck), not a tall-necked silhouette. The neck-lerp raises the head out of the tuck.
//!
//! Frame: `forward` is the facing/travel unit vector, `up = (0, -1)` is screen-up, and
//! `across = forward.perpendicular()` separates the two feet in side view. The whole rig
//! rotates with `direction`, so the goose faces where it walks.

use crate::feet::{self, Feet};
use crate::math::{Rect, Vec2};

// UnderBody (the lower-front belly mass)
pub const UNDERBODY_RADIUS: f32 = 15.0;
pub const UNDERBODY_LENGTH: f32 = 7.0;
pub const UNDERBODY_ELEVATION: f32 = 9.0;

// Body (the main mass)
pub const BODY_RADIUS: f32 = 22.0;
pub const BODY_LENGTH: f32 = 11.0;
pub const BODY_ELEVATION: f32 = 14.0;

// Neck (Necc): one radius, blended between two (height, forward) positions.
pub const NECC_RADIUS: f32 = 13.0;
pub const NECC_HEIGHT_1: f32 = 20.0;
pub const NECC_FORWARD_1: f32 = 3.0;
pub const NECC_HEIGHT_2: f32 = 10.0;
pub const NECC_FORWARD_2: f32 = 16.0;

// Head: two forward segments (skull, then the snout the beak sits on).
pub const HEAD_RADIUS_1: f32 = 15.0;
pub const HEAD_LENGTH_1: f32 = 3.0;
pub const HEAD_RADIUS_2: f32 = 10.0;
pub const HEAD_LENGTH_2: f32 = 5.0;

// Eyes
pub const EYE_RADIUS: f32 = 2.0;
pub const EYE_ELEVATION: f32 = 3.0;
pub const IPD: f32 = 5.0;
pub const EYES_FORWARD: f32 = 5.0;

/// Screen-up unit vector (elevation/height direction).
const UP: Vec2 = Vec2 { x: 0.0, y: -1.0 };

/// How far the body centre floats above the feet/ground point.
const BODY_LIFT: f32 = 23.0;

/// Computed positions of every body part for one frame. Drawn back-to-front:
/// shadow → under-body → body → neck → head → snout → beak → eye → feet.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rig {
    /// Ground point (feet level / shadow centre) — the entity position.
    pub ground: Vec2,
    /// Forward (facing) unit vector.
    pub forward: Vec2,
    /// Neck blend: `0.0` = head tucked into the body, `1.0` = neck raised/reaching.
    pub neck_lerp_percent: f32,

    pub body_center: Vec2,
    pub underbody_center: Vec2,
    pub neck_base: Vec2,
    pub neck_head: Vec2,
    pub snout_center: Vec2,
    pub beak_tip: Vec2,
    pub eye: Vec2,
    pub feet: Feet,
}

impl Default for Rig {
    fn default() -> Self {
        Rig::update(Vec2::new(300.0, 300.0), 0.0, 0.0, 0.0)
    }
}

impl Rig {
    /// Recompute all body points for a goose standing at `center` (feet/ground), facing
    /// `direction_deg`, with the neck blended by `neck_lerp_percent` (`0` tucked → `1`
    /// raised) and the gait at `gait_phase` (radians, advanced by distance travelled).
    pub fn update(
        center: Vec2,
        direction_deg: f32,
        neck_lerp_percent: f32,
        gait_phase: f32,
    ) -> Self {
        let p = crate::math::clamp(neck_lerp_percent, 0.0, 1.0);
        let forward = Vec2::from_angle_degrees(direction_deg);

        // A small body bob synced to the gait (twice per stride).
        let bob = UP * ((gait_phase * 2.0).sin().abs() * 1.5);
        let body_center = center + UP * BODY_LIFT + bob;
        // Belly: lower and forward of the body, filling out the blob's lower-front.
        let underbody_center = center + UP * (UNDERBODY_ELEVATION + 4.0) + forward * 9.0 + bob;

        // The head tucks into the body's front-top at p=0 and lifts/forward as p→1. Scaled
        // from the verified neck constants so the tuck reads as the observed blob.
        let neck_base = body_center + forward * 11.0 + UP * 5.0;
        let lift = crate::math::lerp(11.0, NECC_HEIGHT_1, p); // 11 (tucked) → 20 (raised)
        let reach = crate::math::lerp(NECC_FORWARD_1 + 11.0, NECC_FORWARD_2, p); // 14 → 16
        let neck_head = body_center + forward * reach + UP * lift + bob;

        // Snout extends forward of the skull; the beak is a short rounded orange stub.
        let snout_center = neck_head + forward * 8.0;
        let beak_tip = snout_center + forward * 9.0 + UP * (-2.0);

        // Eye on the upper-front of the skull.
        let eye = neck_head + forward * EYES_FORWARD + UP * EYE_ELEVATION;

        // Procedural feet, slightly forward of the ground point.
        let feet = feet::gait(center + forward * 2.0, forward, gait_phase);

        Self {
            ground: center,
            forward,
            neck_lerp_percent: p,
            body_center,
            underbody_center,
            neck_base,
            neck_head,
            snout_center,
            beak_tip,
            eye,
            feet,
        }
    }

    /// World-space bounding box covering the whole goose (for dirty-rect present),
    /// padded by the largest body radius.
    pub fn bounding_box(&self) -> Rect {
        let points = [
            self.ground,
            self.body_center,
            self.underbody_center,
            self.neck_base,
            self.neck_head,
            self.snout_center,
            self.beak_tip,
            self.eye,
            self.feet.left,
            self.feet.right,
        ];
        // `points` is non-empty, so `bounding` always returns `Some`.
        Rect::bounding(points, BODY_RADIUS).expect("rig always has points")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn geometry_constants_match_verified_source() {
        assert_eq!(BODY_RADIUS, 22.0);
        assert_eq!(BODY_LENGTH, 11.0);
        assert_eq!(BODY_ELEVATION, 14.0);
        assert_eq!(UNDERBODY_RADIUS, 15.0);
        assert_eq!(NECC_RADIUS, 13.0);
        assert_eq!((NECC_HEIGHT_1, NECC_FORWARD_1), (20.0, 3.0));
        assert_eq!((NECC_HEIGHT_2, NECC_FORWARD_2), (10.0, 16.0));
        assert_eq!((HEAD_RADIUS_1, HEAD_LENGTH_1), (15.0, 3.0));
        assert_eq!((HEAD_RADIUS_2, HEAD_LENGTH_2), (10.0, 5.0));
        assert_eq!(
            (EYE_RADIUS, EYE_ELEVATION, IPD, EYES_FORWARD),
            (2.0, 3.0, 5.0, 5.0)
        );
    }

    #[test]
    fn goose_is_assembled_sanely() {
        // Facing +x; up is -y, so "higher" parts have *smaller* y.
        let rig = Rig::update(Vec2::new(300.0, 300.0), 0.0, 0.0, 0.0);
        // Body floats above the feet/ground; head is at/above the body.
        assert!(rig.body_center.y < rig.ground.y);
        assert!(rig.neck_head.y <= rig.body_center.y);
        // Head, snout, and beak reach forward (+x) past the body centre.
        assert!(rig.snout_center.x > rig.body_center.x);
        assert!(rig.beak_tip.x > rig.snout_center.x);
        // Feet straddle and sit near the ground, not up by the head.
        assert!(rig.feet.left.y > rig.body_center.y);
        assert!(Vec2::distance(rig.feet.left, rig.feet.right) > 1.0);
    }

    #[test]
    fn neck_lerp_raises_the_head() {
        let tucked = Rig::update(Vec2::new(300.0, 300.0), 0.0, 0.0, 0.0);
        let raised = Rig::update(Vec2::new(300.0, 300.0), 0.0, 1.0, 0.0);
        // Raising the neck lifts the head (smaller y = higher on screen).
        assert!(raised.neck_head.y < tucked.neck_head.y);
        // Out-of-range neck lerp is clamped.
        assert_eq!(
            Rig::update(Vec2::ZERO, 0.0, 5.0, 0.0).neck_lerp_percent,
            1.0
        );
    }

    #[test]
    fn rotates_with_direction() {
        // Facing +y (down): head, snout, and beak lead downward (+y) past the body.
        let down = Rig::update(Vec2::new(300.0, 300.0), 90.0, 0.0, 0.0);
        assert!(down.snout_center.y > down.body_center.y);
        assert!(down.beak_tip.y > down.snout_center.y);
    }

    #[test]
    fn bounding_box_contains_the_goose() {
        let rig = Rig::default();
        let bb = rig.bounding_box();
        assert!(bb.width() > 0.0 && bb.height() > 0.0);
        assert!(bb.min.y <= rig.neck_head.y && bb.max.y >= rig.feet.left.y);
    }
}
