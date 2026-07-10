//! Side-profile goose painter — flat-illustration style adapted from
//! `docs/art-reference/goose-side-main.svg` (the project's own reference art; path
//! geometry transcribed via the offline absolutizer, coordinates in reference units,
//! art natively faces screen-left).
//!
//! Painter order (back to front): shadow → far leg → near leg → body block (tail
//! shade, silhouette, belly/thigh shades, tail highlights, wing feather layers, wing)
//! → neck ribbon → throat shade → head → beak → eye. Legs are parametric (hip →
//! ankle ribbon + webbed foot) so the plant-and-swing gait reads; the neck is a
//! variable-width ribbon on the rig's spine so it raises/tucks smoothly.

use super::geom::{self, disc, ellipse, paint, stipple_shadow, Frame, P};
use super::RenderPalette;
use crate::math::Vec2;
use crate::rig::{Rig, RigView, SIDE_SCALE};
use tiny_skia::Pixmap;

/// Reference anchors.
const BODY_A: (f32, f32) = (80.0, 112.0);
const HEAD_A: (f32, f32) = (44.0, 20.0);

/// Outline width in reference units (thin, flat-design-friendly).
const OUTLINE_REF: f32 = 2.6;

pub fn paint_side(layer: &mut Pixmap, rig: &Rig, layer_origin: Vec2, ss: f32, pal: &RenderPalette) {
    let mirror = match rig.view {
        RigView::Side { mirror } => mirror,
        _ => return,
    };
    let k = SIDE_SCALE * ss;
    let ow = OUTLINE_REF * k;
    let to_layer = |w: Vec2| (w - layer_origin) * ss;

    let body_o = to_layer(rig.body_center);
    let head_o = to_layer(rig.neck_head);
    let ground = to_layer(rig.ground);

    let body = Frame::side(body_o, k, mirror, 0.0, BODY_A.0, BODY_A.1);
    let head = Frame::side(head_o, k, mirror, rig.head_tilt, HEAD_A.0, HEAD_A.1);

    // Ground shadow.
    stipple_shadow(layer, ground, 24.0 * ss, 6.5 * ss, ss);

    // Legs: far (viewer-far, dark) then near (light); the body covers the hip tops.
    let (near, far) = near_far(rig);
    draw_leg(
        layer,
        &body,
        rig,
        far,
        to_layer,
        k,
        mirror,
        pal.goose_orange_dark,
        ss,
    );
    draw_leg(
        layer,
        &body,
        rig,
        near,
        to_layer,
        k,
        mirror,
        pal.goose_orange,
        ss,
    );

    // Tail underside shade (path3).
    {
        let mut p = P::new(&body);
        p.m(90.20, 156.10);
        p.c(98.60, 155.90, 115.90, 153.40, 125.40, 136.90);
        p.c(126.60, 135.00, 127.80, 131.70, 126.30, 130.00);
        p.l(89.90, 150.00);
        p.l(89.90, 156.10);
        p.z();
        if let Some(path) = p.finish() {
            geom::fill(layer, &path, &paint(pal.goose_shade, 255));
        }
    }

    // Concept C white mass: one continuous body → throat → oval head → back-neck
    // silhouette. The two neck contours deliberately differ: the throat is restrained
    // and forward-curving, while the back-neck stays fuller and drops into a broad
    // shoulder base. One path means there is no circular-head/lollipop join to hide.
    {
        let breath = 1.0 + rig.breath * 0.012;
        let mut bf = body;
        bf.fy = bf.fy * breath;
        let point = |x: f32, y: f32| bf.pt(x, y);
        let fwd = Vec2::new(mirror, 0.0);
        let base = to_layer(rig.neck_base);
        let c1 = to_layer(rig.neck_c1);
        let c2 = to_layer(rig.neck_c2);
        let base_throat = base + fwd * (9.0 * k);
        let base_back = base - fwd * (20.0 * k);
        let throat_c1 = c1 + fwd * (7.0 * k);
        let throat_c2 = c2 + fwd * (5.0 * k);
        let back_c2 = c2 - fwd * (12.0 * k);
        let back_c1 = c1 - fwd * (17.0 * k);
        let head_bottom_front = head.pt(36.0, 26.5);
        let head_bottom_back = head.pt(51.0, 27.0);

        let mut pb = tiny_skia::PathBuilder::new();
        let p = point(62.0, 71.9);
        pb.move_to(p.x, p.y);
        let (a, b, p) = (point(62.0, 81.4), point(64.2, 87.5), point(73.0, 88.2));
        pb.cubic_to(a.x, a.y, b.x, b.y, p.x, p.y);
        let (a, b, p) = (point(85.1, 89.2), point(93.8, 94.4), point(101.1, 99.1));
        pb.cubic_to(a.x, a.y, b.x, b.y, p.x, p.y);
        let (a, b, p) = (
            point(107.0, 102.8),
            point(112.8, 107.7),
            point(122.3, 112.5),
        );
        pb.cubic_to(a.x, a.y, b.x, b.y, p.x, p.y);
        let (a, b, p) = (
            point(126.5, 114.5),
            point(132.2, 116.5),
            point(136.8, 117.0),
        );
        pb.cubic_to(a.x, a.y, b.x, b.y, p.x, p.y);
        let p = point(122.4, 131.5);
        pb.line_to(p.x, p.y);
        let (a, b, p) = (point(115.7, 143.3), point(105.7, 147.2), point(96.3, 148.1));
        pb.cubic_to(a.x, a.y, b.x, b.y, p.x, p.y);
        let (a, b, p) = (point(93.2, 154.0), point(90.0, 159.5), point(82.2, 159.4));
        pb.cubic_to(a.x, a.y, b.x, b.y, p.x, p.y);
        let (a, b, p) = (point(79.6, 159.4), point(77.0, 158.3), point(75.0, 156.4));
        pb.cubic_to(a.x, a.y, b.x, b.y, p.x, p.y);
        let (a, b, p) = (point(70.3, 152.4), point(67.0, 144.9), point(63.0, 143.2));
        pb.cubic_to(a.x, a.y, b.x, b.y, p.x, p.y);
        let (a, b, body_front) = (point(51.0, 139.4), point(32.0, 126.0), point(31.4, 106.0));
        pb.cubic_to(a.x, a.y, b.x, b.y, body_front.x, body_front.y);

        // Throat: a concave, narrower climb into the underside of the oval head.
        let throat_join = Vec2::lerp(body_front, base_throat, 0.55) + fwd * (2.0 * k);
        pb.cubic_to(
            body_front.x,
            body_front.y - 10.0 * k,
            throat_join.x,
            throat_join.y,
            base_throat.x,
            base_throat.y,
        );
        pb.cubic_to(
            throat_c1.x,
            throat_c1.y,
            throat_c2.x,
            throat_c2.y,
            head_bottom_front.x,
            head_bottom_front.y,
        );

        // Restrained oval head, continuous with both neck contours.
        let a = head.pt(31.0, 25.0);
        let b = head.pt(29.5, 15.0);
        let p = head.pt(40.0, 12.0);
        pb.cubic_to(a.x, a.y, b.x, b.y, p.x, p.y);
        let a = head.pt(46.0, 9.5);
        let b = head.pt(56.0, 12.0);
        let p = head.pt(57.0, 19.0);
        pb.cubic_to(a.x, a.y, b.x, b.y, p.x, p.y);
        let a = head.pt(57.0, 23.5);
        let b = head.pt(54.0, 26.5);
        pb.cubic_to(a.x, a.y, b.x, b.y, head_bottom_back.x, head_bottom_back.y);

        // Back-neck: straighter and fuller than the throat, widening into the shoulder.
        pb.cubic_to(
            back_c2.x,
            back_c2.y,
            back_c1.x,
            back_c1.y,
            base_back.x,
            base_back.y,
        );
        let body_back = point(62.0, 71.9);
        let shoulder = Vec2::lerp(base_back, body_back, 0.5) - fwd * (4.0 * k);
        pb.cubic_to(
            base_back.x - fwd.x * 4.0 * k,
            base_back.y + 8.0 * k,
            shoulder.x,
            shoulder.y,
            body_back.x,
            body_back.y,
        );
        pb.close();
        if let Some(path) = pb.finish() {
            geom::stroke(layer, &path, &paint(pal.goose_outline, 255), ow * 2.0);
            geom::fill(layer, &path, &paint(pal.goose_white, 255));
        }
    }

    // Belly shade (path6).
    {
        let mut p = P::new(&body);
        p.m(31.40, 109.10);
        p.l(31.40, 106.40);
        p.c(31.10, 116.10, 34.50, 122.90, 38.80, 128.90);
        p.c(43.70, 135.50, 51.10, 140.80, 58.20, 145.40);
        p.c(61.80, 147.30, 66.60, 149.80, 70.50, 151.40);
        p.c(69.20, 149.10, 66.20, 144.70, 65.00, 143.60);
        p.c(54.00, 139.50, 35.00, 131.00, 31.40, 109.10);
        p.z();
        if let Some(path) = p.finish() {
            geom::fill(layer, &path, &paint(pal.goose_shade, 255));
        }
    }

    // Thigh shade (path2).
    {
        let mut p = P::new(&body);
        p.m(58.00, 145.50);
        p.c(61.60, 151.00, 65.60, 158.00, 72.10, 157.90);
        p.c(73.50, 157.90, 74.90, 157.60, 75.30, 157.30);
        p.l(71.20, 151.70);
        p.l(58.00, 145.50);
        p.z();
        if let Some(path) = p.finish() {
            geom::fill(layer, &path, &paint(pal.goose_shade, 255));
        }
    }

    // Tail highlights (paths 7 + 14), lifted slightly by a honk tail-flick.
    {
        let mut tf = body;
        tf.origin = tf.origin + Vec2::new(0.0, -rig.tail_flick * 2.0 * ss);
        let mut p = P::new(&tf);
        p.m(131.00, 116.00);
        p.l(136.80, 117.00);
        p.c(135.30, 122.90, 130.10, 128.10, 126.30, 130.00);
        p.l(115.80, 126.00);
        p.l(131.00, 116.00);
        p.z();
        if let Some(path) = p.finish() {
            geom::fill(layer, &path, &paint(pal.goose_white, 255));
        }
        let mut p = P::new(&tf);
        p.m(122.30, 131.00);
        p.c(123.90, 128.80, 132.90, 126.00, 136.80, 117.20);
        p.l(136.80, 117.00);
        p.l(136.50, 117.00);
        p.l(130.50, 116.30);
        p.l(119.30, 126.00);
        p.l(122.30, 131.00);
        p.z();
        if let Some(path) = p.finish() {
            geom::fill(layer, &path, &paint(pal.goose_white, 255));
        }
    }

    // Wing feather under-layers (paths 16, 15) then the main layered wing (path 8),
    // all in the wing tone with scalloped trailing edges.
    {
        let mut p = P::new(&body);
        p.m(89.10, 122.60);
        p.c(93.20, 125.40, 100.30, 128.20, 106.10, 128.40);
        p.c(110.00, 128.40, 113.20, 127.90, 117.30, 125.50);
        p.l(117.40, 124.90);
        p.l(104.30, 119.00);
        p.l(89.00, 122.10);
        p.l(89.10, 122.60);
        p.z();
        if let Some(path) = p.finish() {
            geom::fill(layer, &path, &paint(pal.goose_wing, 235));
        }
        let mut p = P::new(&body);
        p.m(104.30, 119.00);
        p.c(108.00, 121.20, 114.80, 122.60, 118.70, 122.20);
        p.c(123.00, 122.10, 127.80, 120.90, 131.00, 116.50);
        p.l(131.00, 116.20);
        p.c(126.00, 115.20, 120.60, 112.70, 115.80, 110.00);
        p.l(104.30, 119.00);
        p.z();
        if let Some(path) = p.finish() {
            geom::fill(layer, &path, &paint(pal.goose_wing, 245));
        }
        let mut p = P::new(&body);
        p.m(72.00, 88.00);
        p.c(62.00, 88.00, 55.80, 93.60, 55.80, 102.90);
        p.c(56.00, 114.00, 69.80, 128.90, 90.90, 131.00);
        p.c(94.00, 131.30, 98.00, 131.20, 100.60, 130.80);
        p.c(94.10, 128.50, 90.10, 124.00, 89.10, 122.60);
        p.c(98.30, 128.30, 107.90, 130.90, 117.30, 125.60);
        p.c(111.20, 124.30, 108.20, 121.60, 104.30, 118.90);
        p.c(108.10, 120.60, 120.10, 126.60, 130.70, 116.30);
        p.c(124.00, 114.90, 117.70, 110.80, 113.70, 108.30);
        p.c(106.30, 104.00, 93.00, 91.50, 75.00, 88.50);
        p.l(72.00, 88.00);
        p.z();
        if let Some(path) = p.finish() {
            geom::fill(layer, &path, &paint(pal.goose_wing, 255));
        }
    }

    // A longitudinal throat shade follows the narrower contour; it reinforces the
    // front/back asymmetry without drawing a cross-neck join line.
    {
        let eased = {
            let p = rig.neck_lerp_percent;
            p * p * (3.0 - 2.0 * p)
        };
        let alpha = (55.0 + 105.0 * eased) as u8;
        let fwd = Vec2::new(mirror, 0.0);
        let base = to_layer(rig.neck_base);
        let c1 = to_layer(rig.neck_c1);
        let c2 = to_layer(rig.neck_c2);
        let under_head = head.pt(39.0, 26.0);
        let mut pb = tiny_skia::PathBuilder::new();
        let start = base + fwd * (3.0 * k);
        pb.move_to(start.x, start.y);
        let a = c1 + fwd * (5.0 * k);
        let b = c2 + fwd * (3.5 * k);
        pb.cubic_to(a.x, a.y, b.x, b.y, under_head.x, under_head.y);
        let inner_head = head.pt(43.0, 27.0);
        pb.line_to(inner_head.x, inner_head.y);
        let b = c2 - fwd * (0.5 * k);
        let end = base - fwd * (1.5 * k);
        let a = c1 - fwd * (1.0 * k);
        pb.cubic_to(b.x, b.y, a.x, a.y, end.x, end.y);
        pb.close();
        if let Some(path) = pb.finish() {
            geom::fill(layer, &path, &paint(pal.goose_shade, alpha));
        }
    }

    // Beak (paths 9, 11, 10, 12): orange wedge, corner, darker mouth-line, nostril.
    {
        let mut p = P::new(&head);
        p.m(32.00, 14.00);
        p.c(30.00, 12.80, 30.80, 16.20, 20.10, 21.60);
        p.c(16.10, 23.70, 16.00, 23.40, 14.60, 24.30);
        p.c(11.90, 25.60, 12.80, 28.40, 13.20, 28.90);
        p.c(13.90, 29.10, 15.00, 28.90, 17.10, 29.00);
        p.l(32.00, 29.90);
        p.c(33.10, 29.50, 35.20, 27.60, 34.70, 25.80);
        p.c(34.20, 24.20, 33.70, 23.00, 33.90, 20.60);
        p.c(34.00, 18.30, 34.70, 15.20, 32.00, 14.00);
        p.z();
        if let Some(path) = p.finish() {
            geom::fill(layer, &path, &paint(pal.goose_orange, 255));
        }
        let mut p = P::new(&head);
        p.m(13.30, 28.80);
        p.c(12.60, 29.00, 13.80, 25.60, 16.10, 24.00);
        p.c(15.00, 23.90, 12.90, 25.40, 13.00, 28.70);
        p.c(13.40, 29.00, 13.30, 28.80, 13.30, 28.80);
        p.z();
        if let Some(path) = p.finish() {
            geom::fill(layer, &path, &paint(pal.goose_orange, 255));
        }
        let mut p = P::new(&head);
        p.m(15.40, 28.10);
        p.l(14.50, 28.40);
        p.c(14.90, 28.80, 16.10, 29.00, 19.00, 29.20);
        p.l(32.00, 29.90);
        p.c(32.70, 29.60, 34.50, 28.40, 34.80, 27.10);
        p.c(30.90, 25.90, 23.00, 26.40, 15.40, 28.10);
        p.z();
        if let Some(path) = p.finish() {
            geom::fill(layer, &path, &paint(pal.goose_orange_dark, 255));
        }
        let mut p = P::new(&head);
        p.m(24.10, 21.90);
        p.c(23.10, 22.70, 24.90, 23.50, 27.60, 22.10);
        p.c(29.10, 20.40, 26.40, 19.80, 24.10, 21.90);
        p.z();
        if let Some(path) = p.finish() {
            geom::fill(layer, &path, &paint(pal.goose_orange_dark, 255));
        }
    }

    // Eye (path 13 simplified to a soft disc) + highlight; the lid closes by
    // vertically collapsing the eye toward its centre.
    {
        let eye_c = head.pt(41.2, 17.0);
        let open = (1.0 - rig.blink * 0.92).max(0.06);
        let r = 1.85 * k;
        ellipse(layer, eye_c, r, r * open, &paint(pal.goose_wing, 255));
        if rig.blink < 0.4 {
            disc(
                layer,
                eye_c + Vec2::new(-0.45 * k, -0.4 * k * open),
                0.5 * k,
                &paint(pal.goose_white, 235),
            );
        }
    }
}

/// Order the feet as (near, far) for the viewer: the lower-on-screen foot reads nearer.
fn near_far(rig: &Rig) -> (usize, usize) {
    let [l, r] = rig.feet_pose;
    if l.pos.y >= r.pos.y {
        (0, 1)
    } else {
        (1, 0)
    }
}

#[allow(clippy::too_many_arguments)]
fn draw_leg(
    layer: &mut Pixmap,
    body: &Frame,
    rig: &Rig,
    foot_idx: usize,
    to_layer: impl Fn(Vec2) -> Vec2,
    k: f32,
    mirror: f32,
    color: (u8, u8, u8),
    ss: f32,
) {
    let pose = rig.feet_pose[foot_idx];
    // Hip anchored to the (bobbing) body; the two hips sit slightly apart.
    let hip = if foot_idx == 0 {
        body.pt(74.0, 152.0)
    } else {
        body.pt(80.0, 153.0)
    };
    let up = Vec2::new(0.0, -1.0);
    let ankle = to_layer(pose.pos) + up * ((pose.lift + 1.5) * ss);

    // Shin: a gentle backward-bowed stroke from hip to ankle.
    let mid = Vec2::lerp(hip, ankle, 0.55) + Vec2::new(-mirror, 0.0) * (2.0 * k);
    let mut pb = tiny_skia::PathBuilder::new();
    pb.move_to(hip.x, hip.y);
    pb.quad_to(mid.x, mid.y, ankle.x, ankle.y);
    if let Some(path) = pb.finish() {
        geom::stroke(layer, &path, &paint(color, 255), 5.0 * k);
    }

    // Webbed foot: wedge pointing along travel, toes lifting slightly mid-swing.
    let fwd = Vec2::new(mirror, 0.0);
    let down = Vec2::new(0.0, 1.0);
    let toe_up = pose.lift * 0.10;
    let foot_frame = Frame {
        origin: ankle,
        fx: (fwd * toe_up.cos() + up * toe_up.sin()) * k,
        fy: (down * toe_up.cos() + fwd * toe_up.sin()) * k,
        ax: 0.0,
        ay: 0.0,
    };
    let mut p = P::new(&foot_frame);
    p.m(-5.0, 1.2);
    p.l(6.0, -4.2);
    p.l(17.0, -0.8);
    p.l(11.0, 4.6);
    p.l(2.0, 3.6);
    p.z();
    if let Some(path) = p.finish() {
        geom::fill(layer, &path, &paint(color, 255));
    }
}
