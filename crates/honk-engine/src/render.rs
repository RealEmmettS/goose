//! Clean-room procedural renderer: a [`Rig`] (and its footmarks) → a `tiny_skia::Pixmap`.
//!
//! Platform-free and offscreen: the same routine feeds the per-monitor overlay in later
//! rounds and the golden-frame test harness here. Body parts are drawn as capsules
//! (a thick round-capped line), back-to-front per the documented order
//! (shadow → underbody → body → neck → head → beak → eyes → feet).
//!
//! World→pixmap mapping is a simple translation: the world point `origin` maps to the
//! pixmap's top-left `(0, 0)`. Colours are the verified defaults
//! (`#ffffff` / `#ffa500` / `#d3d3d3`); the eye/shadow/mud tones are clean-room render
//! details with no constant in the source.
//!
//! The exact proportions here are a **first clean-room approximation**: the original's
//! `updateRig`/render math is closed, so the goldens are a *regression baseline*, not a
//! fidelity reference. Final visual tuning happens on real overlays in M1+ (with the
//! on-screen feedback loop); only the verified geometry *constants* (`rig.rs`) are fixed.

use crate::footmarks::FootMarks;
use crate::math::Vec2;
use crate::rig::{self, Rig};
use tiny_skia::{Color, FillRule, LineCap, Paint, PathBuilder, Pixmap, Stroke, Transform};

const WHITE: (u8, u8, u8) = (0xff, 0xff, 0xff);
const ORANGE: (u8, u8, u8) = (0xff, 0xa5, 0x00);
const OUTLINE: (u8, u8, u8) = (0xd3, 0xd3, 0xd3);
const EYE: (u8, u8, u8) = (0x28, 0x28, 0x28);
const MUD: (u8, u8, u8) = (0x5a, 0x40, 0x28);

/// Extra radius drawn underneath each white part to give it a `#d3d3d3` outline.
const OUTLINE_WIDTH: f32 = 2.0;

fn paint(rgb: (u8, u8, u8), alpha: u8) -> Paint<'static> {
    let mut p = Paint::default();
    p.set_color_rgba8(rgb.0, rgb.1, rgb.2, alpha);
    p.anti_alias = true;
    p
}

/// Stroke a round-capped capsule from `a` to `b` (already in pixmap space).
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

/// Draw a white (or `fill`-coloured) capsule with a `#d3d3d3` outline underneath.
fn outlined_capsule(pixmap: &mut Pixmap, a: Vec2, b: Vec2, radius: f32, fill: (u8, u8, u8)) {
    capsule(pixmap, a, b, radius + OUTLINE_WIDTH, &paint(OUTLINE, 255));
    capsule(pixmap, a, b, radius, &paint(fill, 255));
}

/// Fill a circle at `c` (pixmap space).
fn disc(pixmap: &mut Pixmap, c: Vec2, radius: f32, p: &Paint) {
    if let Some(path) = PathBuilder::from_circle(c.x, c.y, radius) {
        pixmap.fill_path(&path, p, FillRule::Winding, Transform::identity(), None);
    }
}

/// Fill an axis-aligned ellipse at `c` with semi-axes `(rx, ry)` (pixmap space).
fn ellipse(pixmap: &mut Pixmap, c: Vec2, rx: f32, ry: f32, p: &Paint) {
    if let Some(unit) = PathBuilder::from_circle(0.0, 0.0, 1.0) {
        let t = Transform::from_row(rx, 0.0, 0.0, ry, c.x, c.y);
        pixmap.fill_path(&unit, p, FillRule::Winding, t, None);
    }
}

/// Draw an oval body part (goose bodies are wider than tall) with a `#d3d3d3` outline.
fn outlined_ellipse(pixmap: &mut Pixmap, c: Vec2, radius: f32) {
    let (rx, ry) = (radius, radius * 0.78);
    ellipse(
        pixmap,
        c,
        rx + OUTLINE_WIDTH,
        ry + OUTLINE_WIDTH,
        &paint(OUTLINE, 255),
    );
    ellipse(pixmap, c, rx, ry, &paint(WHITE, 255));
}

/// Render the muddy footprints into `pixmap` (call before the goose so it sits on top).
pub fn render_footmarks(pixmap: &mut Pixmap, marks: &FootMarks, now: f32, origin: Vec2) {
    for (mark, scale) in marks.active(now) {
        let c = mark.position - origin;
        let alpha = (180.0 * scale) as u8;
        disc(pixmap, c, 3.5 * scale, &paint(MUD, alpha));
    }
}

/// Render the goose described by `rig` into `pixmap`, with world `origin` at the
/// pixmap's top-left corner.
pub fn render_rig(pixmap: &mut Pixmap, rig: &Rig, origin: Vec2) {
    let t = |p: Vec2| p - origin;
    let fwd = rig.forward;

    // Shadow: a flattened disc on the ground.
    let ground = t(rig.ground);
    if let Some(circle) = PathBuilder::from_circle(0.0, 0.0, rig::BODY_RADIUS * 0.7) {
        let squash = Transform::from_row(1.0, 0.0, 0.0, 0.35, ground.x, ground.y);
        pixmap.fill_path(
            &circle,
            &paint((0, 0, 0), 60),
            FillRule::Winding,
            squash,
            None,
        );
    }

    // Body parts are ovals (a goose body is wider than it is tall), centred on their
    // elevated centres. The `*_LENGTH` constants drive the stepping squash-and-stretch
    // that arrives with locomotion in M2; the resting pose here doesn't need them.
    outlined_ellipse(pixmap, t(rig.underbody_center), rig::UNDERBODY_RADIUS);
    outlined_ellipse(pixmap, t(rig.body_center), rig::BODY_RADIUS);

    // Neck: a white capsule from the body up to the head point.
    outlined_capsule(
        pixmap,
        t(rig.neck_base),
        t(rig.neck_head_point),
        rig::NECC_RADIUS,
        WHITE,
    );

    // Head: both segments are white; only a small beak is orange.
    outlined_capsule(
        pixmap,
        t(rig.neck_head_point),
        t(rig.head1_end),
        rig::HEAD_RADIUS_1,
        WHITE,
    );
    outlined_capsule(
        pixmap,
        t(rig.head1_end),
        t(rig.head2_end),
        rig::HEAD_RADIUS_2,
        WHITE,
    );

    // Beak: a short orange stub poking forward from the front of the head.
    let beak_tip = t(rig.head2_end) + fwd * rig::HEAD_LENGTH_2;
    capsule(
        pixmap,
        t(rig.head2_end),
        beak_tip,
        rig::HEAD_RADIUS_2 * 0.5,
        &paint(ORANGE, 255),
    );

    // Eyes (dark discs on the white head).
    disc(pixmap, t(rig.eye_left), rig::EYE_RADIUS, &paint(EYE, 255));
    disc(pixmap, t(rig.eye_right), rig::EYE_RADIUS, &paint(EYE, 255));

    // Feet (drawn last, per the documented order).
    let foot = |pixmap: &mut Pixmap, c: Vec2| disc(pixmap, c, 3.5, &paint(ORANGE, 255));
    foot(pixmap, t(rig.feet.left));
    foot(pixmap, t(rig.feet.right));
}

/// Convenience for tests/tools: allocate a `width`×`height` transparent pixmap and
/// render the goose so its bounding box is centred. Returns `None` if allocation fails.
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
        let rig = Rig::default();
        let pixmap = render_centered(256, 256, &rig).expect("alloc");
        // The goose drew *something* opaque (alpha byte is index 3 of each RGBA8 px).
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
        // Corners of a centred 256² frame should stay transparent.
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
