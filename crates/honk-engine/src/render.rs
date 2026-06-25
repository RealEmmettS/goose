//! Clean-room procedural renderer: a [`Rig`] (and its footmarks) → a `tiny_skia::Pixmap`.
//!
//! Same *technique* the original uses (`GooseRenderData`): filled stadium/capsule body
//! parts in white, an orange beak and feet, a soft grey outline, and a ground shadow —
//! drawn back-to-front (shadow → legs → under-body → body → neck → head → beak → eye →
//! feet). Platform-free and offscreen: the same routine feeds the per-monitor overlay and
//! the golden-frame tests.
//!
//! World→pixmap mapping is a translation: the world point `origin` maps to the pixmap's
//! top-left. Colours are the verified defaults (`#ffffff` / `#ffa500` / `#d3d3d3`); the
//! eye/shadow/mud tones are clean-room render details. The exact proportions are tuned to
//! resemble the original side-profile goose (the original render maths are closed); the
//! goldens are a regression baseline, not a pixel-fidelity reference.

use crate::footmarks::FootMarks;
use crate::math::Vec2;
use crate::rig::{self, Rig};
use tiny_skia::{Color, FillRule, LineCap, Paint, PathBuilder, Pixmap, Stroke, Transform};

const WHITE: (u8, u8, u8) = (0xff, 0xff, 0xff);
const ORANGE: (u8, u8, u8) = (0xff, 0xa5, 0x00);
const OUTLINE: (u8, u8, u8) = (0xd3, 0xd3, 0xd3);
const EYE: (u8, u8, u8) = (0x20, 0x20, 0x20);
const MUD: (u8, u8, u8) = (0x5a, 0x40, 0x28);

/// Extra radius drawn underneath each white part to give it a `#d3d3d3` outline.
const OUTLINE_WIDTH: f32 = 2.0;

fn paint(rgb: (u8, u8, u8), alpha: u8) -> Paint<'static> {
    let mut p = Paint::default();
    p.set_color_rgba8(rgb.0, rgb.1, rgb.2, alpha);
    p.anti_alias = true;
    p
}

/// Stroke a round-capped capsule from `a` to `b` (pixmap space) of the given radius.
fn capsule(pixmap: &mut Pixmap, a: Vec2, b: Vec2, radius: f32, p: &Paint) {
    let mut pb = PathBuilder::new();
    pb.move_to(a.x, a.y);
    pb.line_to(b.x, b.y);
    if let Some(path) = pb.finish() {
        let stroke = Stroke {
            width: radius * 2.0,
            line_cap: LineCap::Round,
            ..Stroke::default()
        };
        pixmap.stroke_path(&path, p, &stroke, Transform::identity(), None);
    }
}

/// A white capsule with a `#d3d3d3` outline underneath.
fn outlined_capsule(pixmap: &mut Pixmap, a: Vec2, b: Vec2, radius: f32) {
    capsule(pixmap, a, b, radius + OUTLINE_WIDTH, &paint(OUTLINE, 255));
    capsule(pixmap, a, b, radius, &paint(WHITE, 255));
}

/// Fill a circle at `c` (pixmap space).
fn disc(pixmap: &mut Pixmap, c: Vec2, radius: f32, p: &Paint) {
    if let Some(path) = PathBuilder::from_circle(c.x, c.y, radius) {
        pixmap.fill_path(&path, p, FillRule::Winding, Transform::identity(), None);
    }
}

/// Fill a triangle `a–b–c` (pixmap space).
fn triangle(pixmap: &mut Pixmap, a: Vec2, b: Vec2, c: Vec2, p: &Paint) {
    let mut pb = PathBuilder::new();
    pb.move_to(a.x, a.y);
    pb.line_to(b.x, b.y);
    pb.line_to(c.x, c.y);
    pb.close();
    if let Some(path) = pb.finish() {
        pixmap.fill_path(&path, p, FillRule::Winding, Transform::identity(), None);
    }
}

/// Render the muddy footprints into `pixmap` (call before the goose so it sits on top).
pub fn render_footmarks(pixmap: &mut Pixmap, marks: &FootMarks, now: f32, origin: Vec2) {
    for (mark, scale) in marks.active(now) {
        let c = mark.position - origin;
        let alpha = (180.0 * scale) as u8;
        disc(pixmap, c, 3.5 * scale, &paint(MUD, alpha));
    }
}

/// Render the goose described by `rig` into `pixmap`, with world `origin` at the pixmap's
/// top-left corner.
pub fn render_rig(pixmap: &mut Pixmap, rig: &Rig, origin: Vec2) {
    let t = |p: Vec2| p - origin;
    let fwd = rig.forward;
    let across = fwd.perpendicular();

    // Shadow: a flattened disc on the ground.
    let ground = t(rig.ground);
    if let Some(circle) = PathBuilder::from_circle(0.0, 0.0, rig::BODY_RADIUS * 0.85) {
        let squash = Transform::from_row(1.0, 0.0, 0.0, 0.28, ground.x, ground.y);
        pixmap.fill_path(
            &circle,
            &paint((0, 0, 0), 55),
            FillRule::Winding,
            squash,
            None,
        );
    }

    // Legs: thin orange shanks from under the body down to each foot.
    let foot_l = t(rig.feet.left);
    let foot_r = t(rig.feet.right);
    capsule(
        pixmap,
        t(rig.leg_top_left),
        foot_l,
        2.0,
        &paint(ORANGE, 255),
    );
    capsule(
        pixmap,
        t(rig.leg_top_right),
        foot_r,
        2.0,
        &paint(ORANGE, 255),
    );

    // Body mass: under-body (chest) behind, main body on top.
    let ub = t(rig.underbody_center);
    outlined_capsule(
        pixmap,
        ub - fwd * (rig::UNDERBODY_LENGTH * 0.5),
        ub + fwd * (rig::UNDERBODY_LENGTH * 0.5),
        rig::UNDERBODY_RADIUS,
    );
    let bc = t(rig.body_center);
    outlined_capsule(
        pixmap,
        bc - fwd * (rig::BODY_LENGTH * 0.5),
        bc + fwd * (rig::BODY_LENGTH * 0.5),
        rig::BODY_RADIUS,
    );

    // Neck (capsule) → two-segment head (capsules), all white.
    outlined_capsule(pixmap, t(rig.neck_base), t(rig.neck_head), rig::NECC_RADIUS);
    outlined_capsule(
        pixmap,
        t(rig.neck_head),
        t(rig.head1_end),
        rig::HEAD_RADIUS_1,
    );
    outlined_capsule(
        pixmap,
        t(rig.head1_end),
        t(rig.head2_end),
        rig::HEAD_RADIUS_2,
    );

    // Beak: an orange triangle from the snout to the beak tip.
    let snout = t(rig.head2_end);
    let tip = t(rig.beak_tip);
    let half = across * 5.0;
    triangle(pixmap, snout + half, snout - half, tip, &paint(ORANGE, 255));

    // Eye: a small dark disc on the upper-front of the head.
    disc(pixmap, t(rig.eye), rig::EYE_RADIUS + 0.5, &paint(EYE, 255));

    // Feet: orange webbed triangles pointing forward.
    for foot in [foot_l, foot_r] {
        let heel = foot - fwd * 2.0;
        let toe_a = foot + fwd * 5.0 + across * 4.0;
        let toe_b = foot + fwd * 5.0 - across * 4.0;
        triangle(pixmap, heel, toe_a, toe_b, &paint(ORANGE, 255));
    }
}

/// Convenience for tests/tools: allocate a `width`×`height` transparent pixmap and render
/// the goose so its bounding box is centred. Returns `None` if allocation fails.
pub fn render_centered(width: u32, height: u32, rig: &Rig) -> Option<Pixmap> {
    let mut pixmap = Pixmap::new(width, height)?;
    pixmap.fill(Color::TRANSPARENT);
    let bb = rig.bounding_box();
    let bb_center = (bb.min + bb.max) * 0.5;
    let origin = bb_center - Vec2::new(width as f32 * 0.5, height as f32 * 0.5);
    render_rig(&mut pixmap, rig, origin);
    Some(pixmap)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn renders_some_opaque_pixels() {
        let pixmap = render_centered(256, 256, &Rig::default()).expect("alloc");
        let opaque = pixmap
            .data()
            .chunks_exact(4)
            .filter(|px| px[3] > 200)
            .count();
        assert!(
            opaque > 500,
            "expected a visible goose, got {opaque} opaque px"
        );
    }

    #[test]
    fn empty_outside_the_goose() {
        let pixmap = render_centered(256, 256, &Rig::default()).expect("alloc");
        let w = pixmap.width() as usize;
        let h = pixmap.height() as usize;
        let data = pixmap.data();
        for &(x, y) in &[(0usize, 0usize), (w - 1, 0), (0, h - 1), (w - 1, h - 1)] {
            let idx = (y * w + x) * 4;
            assert_eq!(data[idx + 3], 0, "corner ({x},{y}) should be transparent");
        }
    }
}
