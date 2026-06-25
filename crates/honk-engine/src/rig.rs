//! The goose rig: geometry constants and the per-frame body-point computation.
//!
//! The **constants** are verbatim from the verified source (`Exports.cs`, `Rig`). The
//! original draws the goose with the same *technique* we use here — a stack of filled
//! stadium/capsule shapes (under-body, body, a neck blended between two positions, a
//! two-segment head), plus eyes and procedural feet, outlined and shadowed (see
//! `GooseRenderData`). The exact `updateRig` placement maths live only in the closed
//! binary, so the assembly below is a clean-room reconstruction tuned to look like the
//! original side-profile goose; it is pinned by the golden frames as a regression
//! baseline, and customised per `honk300_plan.md`.
//!
//! Frame: `forward` is the facing/travel unit vector, `up = (0, -1)` is screen-up, and
//! `across = forward.perpendicular()` separates the two feet in side view. The whole rig
//! rotates with `direction`, so the goose faces where it walks.

use crate::feet::{self, Feet};
use crate::math::{Rect, Vec2};

// UnderBody (the chest mass, low and forward)
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

// Head: two forward segments (seg 1 is the skull, seg 2 the snout the beak sits on).
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
const BODY_LIFT: f32 = 30.0;

/// Computed positions of every body part for one frame. Drawn back-to-front:
/// shadow → legs → body → under-body → neck → head → beak → eye → feet.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rig {
    /// Ground point (feet level / shadow centre) — the entity position.
    pub ground: Vec2,
    /// Forward (facing) unit vector.
    pub forward: Vec2,
    /// Neck blend: `0.0` = raised/upright (height 20 / forward 3), `1.0` = lowered/reaching
    /// (height 10 / forward 16).
    pub neck_lerp_percent: f32,

    pub body_center: Vec2,
    pub underbody_center: Vec2,
    pub neck_base: Vec2,
    pub neck_head: Vec2,
    pub head1_end: Vec2,
    pub head2_end: Vec2,
    pub beak_tip: Vec2,
    pub eye: Vec2,
    pub leg_top_left: Vec2,
    pub leg_top_right: Vec2,
    pub feet: Feet,
}

impl Default for Rig {
    fn default() -> Self {
        Rig::update(Vec2::new(300.0, 300.0), 90.0, 0.0, 0.0)
    }
}

impl Rig {
    /// Recompute all body points for a goose standing at `center` (feet/ground), facing
    /// `direction_deg`, with the neck blended by `neck_lerp_percent` and the gait at
    /// `gait_phase` (radians, advanced by distance travelled).
    pub fn update(
        center: Vec2,
        direction_deg: f32,
        neck_lerp_percent: f32,
        gait_phase: f32,
    ) -> Self {
        let p = crate::math::clamp(neck_lerp_percent, 0.0, 1.0);
        let forward = Vec2::from_angle_degrees(direction_deg);
        let across = forward.perpendicular();

        // A small body bob synced to the gait (twice per stride).
        let bob = (gait_phase * 2.0).sin().abs() * 1.5;
        let body_center = center + UP * (BODY_LIFT + bob);
        // Chest: lower and forward of the body.
        let underbody_center =
            body_center + forward * 8.0 + UP * (-(BODY_ELEVATION - UNDERBODY_ELEVATION) - 4.0);

        // Neck rises from the front-top of the body; head point blends the two poses.
        let neck_base = body_center + forward * 16.0 + UP * 8.0;
        let neck_height = crate::math::lerp(NECC_HEIGHT_1, NECC_HEIGHT_2, p);
        let neck_forward = crate::math::lerp(NECC_FORWARD_1, NECC_FORWARD_2, p);
        let neck_head = neck_base + UP * neck_height + forward * neck_forward;

        // Two-segment head extending forward from the neck top.
        let head1_end = neck_head + forward * HEAD_LENGTH_1;
        let head2_end = head1_end + forward * HEAD_LENGTH_2;
        // A short orange beak past the snout, angled slightly down.
        let beak_tip = head2_end + forward * 11.0 + UP * (-3.0);

        // Eye on the upper-front of the skull.
        let eye = neck_head + forward * EYES_FORWARD + UP * (EYE_ELEVATION + 4.0);

        // Legs drop from under the body to the feet.
        let leg_top = underbody_center + UP * (-6.0);
        let leg_top_left = leg_top + across * 3.0;
        let leg_top_right = leg_top - across * 3.0;

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
            head1_end,
            head2_end,
            beak_tip,
            eye,
            leg_top_left,
            leg_top_right,
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
            self.head1_end,
            self.head2_end,
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
        // Body floats above the feet/ground.
        assert!(rig.body_center.y < rig.ground.y);
        // Neck top is above the body; head is at/above the neck top.
        assert!(rig.neck_head.y < rig.body_center.y);
        assert!(rig.head1_end.y <= rig.neck_base.y);
        // Head and beak reach forward (+x) past the body.
        assert!(rig.head2_end.x > rig.body_center.x);
        assert!(rig.beak_tip.x > rig.head2_end.x);
        // Feet straddle and sit near the ground, not up by the head.
        assert!(rig.feet.left.y > rig.body_center.y);
        assert!(Vec2::distance(rig.feet.left, rig.feet.right) > 1.0);
    }

    #[test]
    fn neck_lerp_blends_between_poses() {
        let upright = Rig::update(Vec2::new(300.0, 300.0), 0.0, 0.0, 0.0);
        let reaching = Rig::update(Vec2::new(300.0, 300.0), 0.0, 1.0, 0.0);
        // Reaching pose: lower neck (height 10) and further forward (16) than upright.
        assert!(reaching.neck_head.y > upright.neck_head.y);
        assert!(reaching.neck_head.x > upright.neck_head.x);
        // Out-of-range neck lerp is clamped.
        assert_eq!(
            Rig::update(Vec2::ZERO, 0.0, 5.0, 0.0).neck_lerp_percent,
            1.0
        );
    }

    #[test]
    fn rotates_with_direction() {
        // Facing +y (down): the neck still rises (up), then head + beak extend forward
        // along +y from the neck top — i.e. the rig rotated to face the travel direction.
        let down = Rig::update(Vec2::new(300.0, 300.0), 90.0, 0.0, 0.0);
        assert!(
            down.neck_head.y < down.body_center.y,
            "neck rises above the body"
        );
        assert!(
            down.head2_end.y > down.neck_head.y,
            "head leads downward (forward)"
        );
        assert!(
            down.beak_tip.y > down.head2_end.y,
            "beak leads furthest forward"
        );
    }

    #[test]
    fn bounding_box_contains_the_goose() {
        let rig = Rig::default();
        let bb = rig.bounding_box();
        assert!(bb.width() > 0.0 && bb.height() > 0.0);
        assert!(bb.min.y <= rig.neck_head.y && bb.max.y >= rig.feet.left.y);
    }
}
