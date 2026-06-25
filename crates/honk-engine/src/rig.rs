//! The goose rig: geometry constants and the per-frame body-point computation.
//!
//! The **constants** below are verbatim from the verified source (`Exports.cs`, `Rig`).
//! The actual `updateRig` placement math lives only in the closed binary, so the
//! point computation here is a **clean-room reconstruction** that uses every constant
//! consistently (this is the intended approach — the renderer is reimplemented
//! clean-room, no asset/code extraction). Its output is pinned by tests as a
//! regression wall, not asserted against the original binary.
//!
//! Interpretation: `direction` gives the `forward` unit vector; `up = (0, -1)` is
//! screen-up; "elevation"/"height" raise a part by `up`, "forward" extends it along
//! `forward`. "radius + length" parts are capsules; "elevation" also drives the shadow.

use crate::feet::ProceduralFeet;
use crate::math::{Rect, Vec2};

// UnderBody
pub const UNDERBODY_RADIUS: f32 = 15.0;
pub const UNDERBODY_LENGTH: f32 = 7.0;
pub const UNDERBODY_ELEVATION: f32 = 9.0;

// Body
pub const BODY_RADIUS: f32 = 22.0;
pub const BODY_LENGTH: f32 = 11.0;
pub const BODY_ELEVATION: f32 = 14.0;

// Neck (Necc): one radius, blended between two (height, forward) positions.
pub const NECC_RADIUS: f32 = 13.0;
pub const NECC_HEIGHT_1: f32 = 20.0;
pub const NECC_FORWARD_1: f32 = 3.0;
pub const NECC_HEIGHT_2: f32 = 10.0;
pub const NECC_FORWARD_2: f32 = 16.0;

// Head: two forward segments (segment 2 is the beak).
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

/// Computed positions of every body part for one frame, plus the inputs that produced
/// them. Drawn back-to-front: shadow → underbody → body → neck → head → eyes → feet.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rig {
    /// Ground point (shadow centre) — the entity position.
    pub ground: Vec2,
    /// Forward (facing) unit vector.
    pub forward: Vec2,
    /// Neck blend: `0.0` = raised pose (height 20 / forward 3), `1.0` = lowered/extended
    /// pose (height 10 / forward 16).
    pub neck_lerp_percent: f32,

    pub underbody_center: Vec2,
    pub body_center: Vec2,
    pub neck_base: Vec2,
    pub neck_center: Vec2,
    pub neck_head_point: Vec2,
    pub head1_end: Vec2,
    pub head2_end: Vec2,
    pub eye_left: Vec2,
    pub eye_right: Vec2,
    pub feet: ProceduralFeet,
}

impl Default for Rig {
    fn default() -> Self {
        Rig::update(Vec2::new(300.0, 300.0), 90.0, 0.0)
    }
}

impl Rig {
    /// Recompute all body points for a goose at `center` facing `direction_deg`, with
    /// the neck blended by `neck_lerp_percent` (clamped to `[0, 1]`).
    pub fn update(center: Vec2, direction_deg: f32, neck_lerp_percent: f32) -> Self {
        let p = crate::math::clamp(neck_lerp_percent, 0.0, 1.0);
        let forward = Vec2::from_angle_degrees(direction_deg);

        let underbody_center = center + UP * UNDERBODY_ELEVATION;
        let body_center = center + UP * BODY_ELEVATION;
        let neck_base = body_center;

        let neck_height = crate::math::lerp(NECC_HEIGHT_1, NECC_HEIGHT_2, p);
        let neck_forward = crate::math::lerp(NECC_FORWARD_1, NECC_FORWARD_2, p);
        let neck_head_point = center + UP * neck_height + forward * neck_forward;
        let neck_center = Vec2::lerp(neck_base, neck_head_point, 0.5);

        let head1_end = neck_head_point + forward * HEAD_LENGTH_1;
        let head2_end = head1_end + forward * HEAD_LENGTH_2;

        let eye_base = neck_head_point + forward * EYES_FORWARD + UP * EYE_ELEVATION;
        let lateral = forward.perpendicular() * (IPD * 0.5);
        let eye_left = eye_base - lateral;
        let eye_right = eye_base + lateral;

        Self {
            ground: center,
            forward,
            neck_lerp_percent: p,
            underbody_center,
            body_center,
            neck_base,
            neck_center,
            neck_head_point,
            head1_end,
            head2_end,
            eye_left,
            eye_right,
            feet: ProceduralFeet::at_rest(center, forward),
        }
    }

    /// World-space bounding box covering the whole goose (for dirty-rect present),
    /// padded by the largest body radius.
    pub fn bounding_box(&self) -> Rect {
        let points = [
            self.ground,
            self.underbody_center,
            self.body_center,
            self.neck_base,
            self.neck_head_point,
            self.head1_end,
            self.head2_end,
            self.eye_left,
            self.eye_right,
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

    fn approx(a: Vec2, b: Vec2) -> bool {
        (a.x - b.x).abs() < 1e-3 && (a.y - b.y).abs() < 1e-3
    }

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
    fn rig_points_pinned_for_known_state() {
        // Goose at (300,300) facing 90° (forward = +y), neck fully raised (p = 0).
        let rig = Rig::update(Vec2::new(300.0, 300.0), 90.0, 0.0);
        assert!(approx(rig.ground, Vec2::new(300.0, 300.0)));
        // body/underbody are raised by their elevations (up 14 / up 9).
        assert!(approx(rig.body_center, Vec2::new(300.0, 286.0)));
        assert!(approx(rig.underbody_center, Vec2::new(300.0, 291.0)));
        // neck top: up 20, forward (+y) 3 ⇒ (300, 300 - 20 + 3) = (300, 283)
        assert!(approx(rig.neck_head_point, Vec2::new(300.0, 283.0)));
        // head segments extend forward (+y): +3 then +5
        assert!(approx(rig.head1_end, Vec2::new(300.0, 286.0)));
        assert!(approx(rig.head2_end, Vec2::new(300.0, 291.0)));
        // eyes: from neck top, forward 5 (+y) and up 3, split laterally by IPD/2 on x.
        // forward=(0,1) ⇒ perpendicular=(-1,0); base=(300, 283+5-3)=(300,285)
        assert!(approx(rig.eye_left, Vec2::new(302.5, 285.0)));
        assert!(approx(rig.eye_right, Vec2::new(297.5, 285.0)));
    }

    #[test]
    fn neck_lerp_blends_between_poses() {
        let raised = Rig::update(Vec2::new(300.0, 300.0), 90.0, 0.0);
        let lowered = Rig::update(Vec2::new(300.0, 300.0), 90.0, 1.0);
        // Lowered pose: height 10 (less up), forward 16 (more forward in +y).
        assert!(approx(
            lowered.neck_head_point,
            Vec2::new(300.0, 300.0 - 10.0 + 16.0)
        ));
        // The two poses differ, and clamping holds for out-of-range input.
        assert_ne!(raised.neck_head_point, lowered.neck_head_point);
        let over = Rig::update(Vec2::new(300.0, 300.0), 90.0, 5.0);
        assert_eq!(over.neck_lerp_percent, 1.0);
    }

    #[test]
    fn bounding_box_contains_the_goose() {
        let rig = Rig::default();
        let bb = rig.bounding_box();
        assert!(bb.width() > 0.0 && bb.height() > 0.0);
        assert!(bb.min.x <= rig.ground.x && bb.max.x >= rig.ground.x);
        assert!(bb.min.y <= rig.neck_head_point.y);
    }
}
