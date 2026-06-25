//! Clean-room procedural renderer: a [`Rig`] (and its footmarks) → a `tiny_skia::Pixmap`.
//!
//! Reproduces the *technique* of the original (`GooseRenderData`) and is tuned against
//! direct observation of the running goose: a soft rounded **blob** built from overlapping
//! white stadium/capsule forms (under-body, body, neck, head) with a thin grey outline, a
//! short rounded **orange beak**, a small dark eye, orange webbed feet, and a **stippled**
//! ground shadow. Drawn back-to-front. Platform-free and offscreen — the same routine
//! feeds the overlay and the golden-frame tests.
//!
//! Overlap trick: each white form is drawn as [outline-disc-then-white]; where forms
//! overlap, the front form's white covers the back form's outline, leaving only the outer
//! silhouette outlined — the same single-outline blob the original shows. World→pixmap is
//! a translation by `origin`. Colours are the verified defaults (`#ffffff` / `#ffa500` /
//! `#d3d3d3`); eye/shadow/mud tones are clean-room render details.

use crate::footmarks::FootMarks;
use crate::math::Vec2;
use crate::rig::{self, Rig};
use tiny_skia::{Color, FillRule, LineCap, Paint, PathBuilder, Pixmap, Stroke, Transform};

const WHITE: (u8, u8, u8) = (0xff, 0xff, 0xff);
const ORANGE: (u8, u8, u8) = (0xff, 0xa5, 0x00);
const OUTLINE: (u8, u8, u8) = (0xd3, 0xd3, 0xd3);
const EYE: (u8, u8, u8) = (0x1a, 0x1a, 0x1a);
const MUD: (u8, u8, u8) = (0x5a, 0x40, 0x28);

/// Extra radius drawn underneath each white part to give it a `#d3d3d3` outline.
const OUTLINE_WIDTH: f32 = 1.6;

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

/// A white capsule with a `#d3d3d3` outline underneath (internal outlines are covered by
/// overlapping white forms, leaving only the outer silhouette).
fn white_capsule(pixmap: &mut Pixmap, a: Vec2, b: Vec2, radius: f32) {
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

/// A stippled (dotted) elliptical ground shadow, matching the original's dither-brush look.
fn stipple_shadow(pixmap: &mut Pixmap, ground: Vec2) {
    let rx = rig::BODY_RADIUS * 0.95;
    let ry = rx * 0.32;
    let dot = paint((0x20, 0x20, 0x20), 70);
    let step = 3.0;
    let mut dy = -ry;
    while dy <= ry {
        let mut dx = -rx;
        // Offset alternate rows for a denser dither.
        let row_off = if ((dy / step) as i32) % 2 == 0 {
            0.0
        } else {
            step * 0.5
        };
        while dx <= rx {
            let nx = (dx + row_off) / rx;
            let ny = dy / ry;
            if nx * nx + ny * ny <= 1.0 {
                disc(pixmap, ground + Vec2::new(dx + row_off, dy), 0.95, &dot);
            }
            dx += step;
        }
        dy += step;
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

/// Render the rising/fading heart particles (M6 pat-streak; call after the goose so hearts
/// float on top). Each heart is a small procedural pink heart at its current alpha.
pub fn render_hearts(pixmap: &mut Pixmap, hearts: &crate::hearts::Hearts, now: f32, origin: Vec2) {
    const HEART: (u8, u8, u8) = (0xff, 0x5a, 0x7a); // soft pink-red
    const LOBE: f32 = 3.4;
    for (pos, alpha) in hearts.active(now) {
        let a = (alpha * 255.0) as u8;
        if a == 0 {
            continue;
        }
        let p = paint(HEART, a);
        let c = pos - origin;
        // Two top lobes…
        disc(pixmap, c + Vec2::new(-LOBE * 0.85, -LOBE * 0.45), LOBE, &p);
        disc(pixmap, c + Vec2::new(LOBE * 0.85, -LOBE * 0.45), LOBE, &p);
        // …over a downward point.
        triangle(
            pixmap,
            c + Vec2::new(-LOBE * 1.75, -LOBE * 0.1),
            c + Vec2::new(LOBE * 1.75, -LOBE * 0.1),
            c + Vec2::new(0.0, LOBE * 1.9),
            &p,
        );
    }
}

/// Render the goose described by `rig` into `pixmap`, with world `origin` at the pixmap's
/// top-left corner.
pub fn render_rig(pixmap: &mut Pixmap, rig: &Rig, origin: Vec2) {
    let t = |p: Vec2| p - origin;
    let fwd = rig.forward;
    let across = fwd.perpendicular();

    stipple_shadow(pixmap, t(rig.ground));

    // Body blob: belly behind, main body on top (both white stadiums).
    let ub = t(rig.underbody_center);
    white_capsule(
        pixmap,
        ub - fwd * (rig::UNDERBODY_LENGTH * 0.5),
        ub + fwd * (rig::UNDERBODY_LENGTH * 0.5),
        rig::UNDERBODY_RADIUS,
    );
    let bc = t(rig.body_center);
    white_capsule(
        pixmap,
        bc - fwd * (rig::BODY_LENGTH * 0.5),
        bc + fwd * (rig::BODY_LENGTH * 0.5),
        rig::BODY_RADIUS,
    );

    // Neck (thinner) → head (a rounded stadium ending at the snout). When tucked these sit
    // inside/atop the body and merge into the blob; raising the neck lifts the head out.
    let neck_head = t(rig.neck_head);
    let snout = t(rig.snout_center);
    white_capsule(pixmap, t(rig.neck_base), neck_head, rig::NECC_RADIUS * 0.82);
    white_capsule(pixmap, neck_head, snout, rig::HEAD_RADIUS_2 + 2.0);

    // Beak: a short, rounded orange stub forward of the snout.
    capsule(pixmap, snout, t(rig.beak_tip), 5.0, &paint(ORANGE, 255));

    // Eye: a small dark disc on the upper-front of the head.
    disc(pixmap, t(rig.eye), rig::EYE_RADIUS + 0.6, &paint(EYE, 255));

    // Feet: orange webbed triangles pointing forward (drawn last, at the bottom-front).
    for foot in [t(rig.feet.left), t(rig.feet.right)] {
        let heel = foot - fwd * 2.0;
        let toe_a = foot + fwd * 5.0 + across * 3.5;
        let toe_b = foot + fwd * 5.0 - across * 3.5;
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
    fn renders_a_visible_heart() {
        use crate::hearts::Hearts;
        let mut hearts = Hearts::new();
        hearts.add(Vec2::new(64.0, 64.0), 0.0);
        let mut pixmap = Pixmap::new(128, 128).expect("alloc");
        pixmap.fill(Color::TRANSPARENT);
        render_hearts(&mut pixmap, &hearts, 0.0, Vec2::ZERO);
        // Count clearly-pink opaque pixels (the heart colour: high R, low-ish G/B).
        let pink = pixmap
            .data()
            .chunks_exact(4)
            .filter(|px| px[3] > 120 && px[0] > 150 && px[1] < 170 && px[2] < 210)
            .count();
        assert!(pink > 15, "expected a visible heart, got {pink} pink px");
    }

    #[test]
    fn no_hearts_draws_nothing() {
        use crate::hearts::Hearts;
        let hearts = Hearts::new();
        let mut pixmap = Pixmap::new(64, 64).expect("alloc");
        pixmap.fill(Color::TRANSPARENT);
        render_hearts(&mut pixmap, &hearts, 0.0, Vec2::ZERO);
        let opaque = pixmap.data().chunks_exact(4).filter(|px| px[3] > 0).count();
        assert_eq!(opaque, 0, "no hearts → nothing drawn");
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
