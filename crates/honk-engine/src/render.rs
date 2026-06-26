//! Clean-room procedural renderer: a [`Rig`] (and its footmarks) → a `tiny_skia::Pixmap`.
//!
//! Reproduces the *technique* of the original (`GooseRenderData`) and is tuned against
//! direct observation of the running goose and local preview iteration: a compact rounded
//! white body, tucked head, thin grey outline, short **orange beak**, small dark eye, tiny
//! orange feet, and a soft ground shadow. Drawn back-to-front. Platform-free and offscreen
//! — the same routine
//! feeds the overlay and the golden-frame tests.
//!
//! Overlap trick: each white form is drawn as [outline-then-white]; where forms overlap,
//! the front form's white covers the back form's outline, leaving only the outer silhouette
//! outlined. World->pixmap is a translation by `origin`. Colours are the verified defaults
//! (`#ffffff` / `#ffa500` / `#d3d3d3`); eye/shadow/mud tones are clean-room render details.

use crate::footmarks::FootMarks;
use crate::math::Vec2;
use crate::rig::{self, Rig};
use tiny_skia::{Color, FillRule, LineCap, Paint, PathBuilder, Pixmap, Stroke, Transform};

const WHITE: (u8, u8, u8) = (0xff, 0xff, 0xff);
const ORANGE: (u8, u8, u8) = (0xff, 0xa5, 0x00);
const ORANGE_DARK: (u8, u8, u8) = (0xd8, 0x78, 0x00);
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

fn local(origin: Vec2, fwd: Vec2, across: Vec2, x: f32, y: f32) -> Vec2 {
    origin + fwd * x + across * y
}

fn cubic_to(pb: &mut PathBuilder, p1: Vec2, p2: Vec2, p3: Vec2) {
    pb.cubic_to(p1.x, p1.y, p2.x, p2.y, p3.x, p3.y);
}

fn fill_path(pixmap: &mut Pixmap, path: tiny_skia::Path, p: &Paint) {
    pixmap.fill_path(&path, p, FillRule::Winding, Transform::identity(), None);
}

fn stroke_path(pixmap: &mut Pixmap, path: &tiny_skia::Path, p: &Paint, width: f32) {
    let stroke = Stroke {
        width,
        line_cap: LineCap::Round,
        ..Stroke::default()
    };
    pixmap.stroke_path(path, p, &stroke, Transform::identity(), None);
}

fn capsule(pixmap: &mut Pixmap, a: Vec2, b: Vec2, radius: f32, p: &Paint) {
    let mut pb = PathBuilder::new();
    pb.move_to(a.x, a.y);
    pb.line_to(b.x, b.y);
    if let Some(path) = pb.finish() {
        stroke_path(pixmap, &path, p, radius * 2.0);
    }
}

fn white_capsule(pixmap: &mut Pixmap, a: Vec2, b: Vec2, radius: f32) {
    capsule(pixmap, a, b, radius + OUTLINE_WIDTH, &paint(OUTLINE, 255));
    capsule(pixmap, a, b, radius, &paint(WHITE, 255));
}

fn white_shape(pixmap: &mut Pixmap, pb: PathBuilder) {
    if let Some(path) = pb.finish() {
        stroke_path(pixmap, &path, &paint(OUTLINE, 255), OUTLINE_WIDTH * 2.0);
        fill_path(pixmap, path, &paint(WHITE, 255));
    }
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

/// Dotted elliptical ground shadow, matching the original's dithered shadow brush.
fn stipple_shadow(pixmap: &mut Pixmap, ground: Vec2) {
    let rx = rig::BODY_RADIUS * 1.12;
    let ry = rx * 0.25;
    let dot = paint((0x20, 0x20, 0x20), 42);
    let step = 3.0;
    let mut dy = -ry;
    while dy <= ry {
        let row_off = if ((dy / step) as i32) % 2 == 0 {
            0.0
        } else {
            step * 0.5
        };
        let mut dx = -rx;
        while dx <= rx {
            let nx = (dx + row_off) / rx;
            let ny = dy / ry;
            if nx * nx + ny * ny <= 1.0 {
                disc(pixmap, ground + Vec2::new(dx + row_off, dy), 0.85, &dot);
            }
            dx += step;
        }
        dy += step;
    }
}

fn original_foot(pixmap: &mut Pixmap, foot: Vec2, fwd: Vec2, across: Vec2) {
    let p = |x, y| local(foot, fwd, across, x, y);
    let mut web = PathBuilder::new();
    let heel = p(-2.2, 0.0);
    web.move_to(heel.x, heel.y);
    web.line_to(p(3.2, -3.3).x, p(3.2, -3.3).y);
    web.line_to(p(8.2, -0.8).x, p(8.2, -0.8).y);
    web.line_to(p(5.3, 3.4).x, p(5.3, 3.4).y);
    web.line_to(p(1.4, 2.1).x, p(1.4, 2.1).y);
    web.line_to(heel.x, heel.y);
    web.close();
    if let Some(path) = web.finish() {
        fill_path(pixmap, path, &paint(ORANGE, 255));
    }
}

fn original_body(pixmap: &mut Pixmap, center: Vec2, fwd: Vec2, across: Vec2) {
    let p = |x, y| local(center, fwd, across, x, y);
    let mut body = PathBuilder::new();
    let start = p(-36.0, 0.0);
    body.move_to(start.x, start.y);
    cubic_to(&mut body, p(-36.0, -12.0), p(-20.0, -20.0), p(2.0, -19.0));
    cubic_to(&mut body, p(20.0, -18.0), p(30.0, -8.0), p(29.0, 2.0));
    cubic_to(&mut body, p(28.0, 13.0), p(15.0, 20.0), p(-5.0, 20.0));
    cubic_to(&mut body, p(-27.0, 20.0), p(-38.0, 12.0), start);
    body.close();
    white_shape(pixmap, body);
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

    let drawn_feet = [
        t(rig.feet.left) + across * 1.8,
        t(rig.feet.right) + across * 1.8,
    ];

    let neck_base = t(rig.neck_base);
    let neck_head = t(rig.neck_head);
    let snout = t(rig.snout_center);

    // Put the neck under the body/head so the joints do not read as exposed construction
    // circles. The original's charm is a compact silhouette with only soft interior hints.
    white_capsule(pixmap, neck_base, neck_head, rig::NECC_RADIUS * 0.50);

    // Original-style compact body: one cohesive silhouette instead of stacked capsules.
    let bc = t(rig.body_center);
    original_body(pixmap, bc, fwd, across);
    white_capsule(
        pixmap,
        neck_head - fwd * 2.4,
        snout - fwd * 1.7,
        rig::HEAD_RADIUS_2 + 0.2,
    );

    // Tiny orange feet, as in the original desktop goose.
    for foot in drawn_feet {
        original_foot(pixmap, foot, fwd, across);
    }

    // Beak: short rounded orange wedge.
    let beak_base = snout - fwd * 0.6;
    let beak_tip = t(rig.beak_tip) - fwd * 0.8;
    let mut beak = PathBuilder::new();
    let b0 = beak_base - across * 3.5;
    beak.move_to(b0.x, b0.y);
    cubic_to(
        &mut beak,
        beak_base + fwd * 4.4 - across * 4.3,
        beak_tip - fwd * 1.0 - across * 2.4,
        beak_tip,
    );
    cubic_to(
        &mut beak,
        beak_tip - fwd * 1.0 + across * 2.8,
        beak_base + fwd * 4.4 + across * 4.0,
        beak_base + across * 3.5,
    );
    beak.close();
    if let Some(path) = beak.finish() {
        stroke_path(pixmap, &path, &paint(ORANGE_DARK, 230), 1.3);
        fill_path(pixmap, path, &paint(ORANGE, 255));
    }

    disc(
        pixmap,
        beak_base + fwd * 4.2 - across * 1.8,
        0.75,
        &paint(ORANGE_DARK, 150),
    );

    // Eye: the original reads as a tiny dark dot, not a ringed cartoon eye.
    let eye = t(rig.eye);
    disc(pixmap, eye, rig::EYE_RADIUS * 0.92, &paint(EYE, 255));
    disc(
        pixmap,
        eye - fwd * 0.25 - across * 0.25,
        0.35,
        &paint(WHITE, 230),
    );
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
