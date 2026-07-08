//! Top-down goose painter — adapted from `docs/art-reference/goose-top-down.svg`
//! (coordinates pre-anchored at the body centre; art natively heads up/-y). Rotates
//! freely to any heading with a subtle gait-synced waddle roll; the head and beak
//! translate forward with the neck reach.

use super::geom::{self, outline_fill, paint, stipple_shadow, Frame, P};
use super::RenderPalette;
use crate::math::Vec2;
use crate::rig::{Rig, RigView, TOP_SCALE};
use tiny_skia::Pixmap;

const OUTLINE_REF: f32 = 2.4;

pub fn paint_top(layer: &mut Pixmap, rig: &Rig, layer_origin: Vec2, ss: f32, pal: &RenderPalette) {
    let (heading, waddle) = match rig.view {
        RigView::TopDown { heading, waddle } => (heading, waddle),
        _ => return,
    };
    let k = TOP_SCALE * ss;
    let ow = OUTLINE_REF * k;
    let to_layer = |w: Vec2| (w - layer_origin) * ss;

    // Waddle: roll the whole art a few degrees with the gait.
    let (s, c) = waddle.sin_cos();
    let h = Vec2::new(heading.x * c - heading.y * s, heading.x * s + heading.y * c);

    let ground = to_layer(rig.ground);
    let body = Frame::top(ground, k, h, 0.0, 0.0);
    // Head/beak ride the neck reach: their art is body-centre-anchored, so shift the
    // whole head frame by however far the rig pushed the head past its rest position.
    let reach = to_layer(rig.neck_head) - (ground + h * (57.5 * k));
    let head = Frame::top(ground + reach, k, h, 0.0, 0.0);

    // Soft shadow slightly larger than the body.
    stipple_shadow(layer, ground, 36.0 * k, 30.0 * k, ss);

    // Body (path 1).
    {
        let mut p = P::new(&body);
        p.m(9.70, -51.80);
        p.c(18.60, -48.20, 28.00, -38.60, 31.30, -22.80);
        p.l(23.00, 29.70);
        p.c(21.60, 32.90, 19.10, 34.70, 15.10, 39.70);
        p.c(12.00, 45.20, 7.20, 51.60, 0.00, 51.70);
        p.c(-7.20, 51.80, -11.70, 46.00, -14.90, 40.50);
        p.c(-17.70, 37.00, -21.00, 34.30, -23.10, 30.00);
        p.l(-31.30, -21.10);
        p.c(-29.10, -30.30, -23.10, -45.50, -10.10, -51.70);
        p.l(9.70, -51.80);
        p.z();
        if let Some(path) = p.finish() {
            outline_fill(layer, &path, pal.goose_white, pal.goose_outline, ow, 255);
        }
    }

    // Neck-side shades (paths 2, 3).
    for pts in [true, false] {
        let mut p = P::new(&body);
        if pts {
            p.m(-11.10, -51.30);
            p.l(-7.60, -43.40);
            p.c(-6.00, -42.70, -4.80, -43.00, -4.10, -43.40);
            p.c(-6.50, -45.00, -9.10, -49.00, -10.20, -52.10);
            p.l(-11.10, -51.30);
        } else {
            p.m(10.10, -51.80);
            p.c(9.30, -48.70, 6.70, -45.40, 3.70, -43.40);
            p.c(4.80, -43.00, 6.00, -42.90, 7.30, -43.30);
            p.l(10.80, -51.40);
            p.l(10.10, -51.80);
        }
        p.z();
        if let Some(path) = p.finish() {
            geom::fill(layer, &path, &paint(pal.goose_shade, 235));
        }
    }

    // Wings (paths 5, 6) — layered slate with feathered trailing tips.
    {
        let mut p = P::new(&body);
        p.m(-31.30, -21.10);
        p.c(-30.00, -24.20, -27.60, -27.60, -23.70, -29.10);
        p.c(-21.60, -30.00, -18.80, -30.10, -16.80, -28.90);
        p.c(-13.90, -27.20, -11.70, -24.00, -10.70, -21.40);
        p.c(-8.10, -14.50, -7.70, -5.60, -10.40, 7.00);
        p.c(-11.90, 13.60, -13.80, 22.30, -12.80, 36.60);
        p.c(-12.60, 39.00, -13.00, 39.30, -15.40, 38.10);
        p.c(-18.60, 36.40, -21.40, 31.20, -23.10, 21.60);
        p.c(-23.30, 24.90, -22.60, 29.90, -22.60, 30.00);
        p.c(-24.00, 29.60, -24.80, 28.20, -25.90, 26.60);
        p.c(-29.80, 20.00, -30.00, 10.50, -29.70, 5.30);
        p.c(-30.40, 7.80, -30.70, 12.80, -30.80, 14.10);
        p.c(-31.60, 12.30, -32.40, 7.60, -32.50, 6.40);
        p.c(-33.30, -0.20, -33.80, -9.60, -31.30, -21.10);
        p.z();
        if let Some(path) = p.finish() {
            geom::fill(layer, &path, &paint(pal.goose_wing, 255));
        }
        let mut p = P::new(&body);
        p.m(31.40, 14.30);
        p.c(31.60, 13.10, 31.20, 7.80, 29.50, 5.00);
        p.c(29.80, 10.10, 30.20, 19.70, 26.30, 26.60);
        p.c(25.60, 27.70, 24.60, 29.60, 22.70, 30.00);
        p.c(22.80, 27.40, 23.30, 24.90, 23.10, 21.60);
        p.c(22.20, 27.00, 20.80, 34.60, 16.60, 37.50);
        p.c(13.10, 40.00, 12.70, 39.20, 13.00, 36.60);
        p.c(14.40, 23.60, 11.70, 13.60, 10.40, 7.90);
        p.c(7.70, -3.40, 7.10, -18.90, 14.40, -26.40);
        p.c(18.30, -30.40, 21.80, -30.70, 25.00, -29.00);
        p.c(28.40, -27.40, 30.60, -24.20, 31.40, -21.60);
        p.c(32.40, -18.50, 33.30, -10.00, 33.30, -2.50);
        p.c(33.30, 2.90, 32.40, 12.20, 30.70, 14.60);
        p.z();
        if let Some(path) = p.finish() {
            geom::fill(layer, &path, &paint(pal.goose_wing, 255));
        }
    }

    // Neck bridge: a body-toned capsule from the shoulders to the head so a reaching
    // head never detaches, plus a shade ellipse under the head so it reads as a
    // separate mass on top of the body.
    {
        let a = body.pt(0.0, -45.0);
        let b = head.pt(0.0, -57.5);
        let mut pb = tiny_skia::PathBuilder::new();
        pb.move_to(a.x, a.y);
        pb.line_to(b.x, b.y);
        if let Some(path) = pb.finish() {
            geom::stroke(layer, &path, &paint(pal.goose_white, 255), 17.0 * k);
        }
        let under = head.pt(0.0, -55.5);
        geom::ellipse(
            layer,
            under,
            14.8 * k,
            17.5 * k,
            &paint(pal.goose_shade, 160),
        );
    }

    // Beak (path 0), forward of the head.
    {
        let mut p = P::new(&head);
        p.m(-6.60, -69.60);
        p.c(-6.00, -71.30, -5.30, -73.60, -5.00, -75.00);
        p.c(-4.60, -76.90, -3.80, -82.00, -3.30, -83.80);
        p.c(-3.00, -85.10, -1.00, -85.70, 0.00, -85.70);
        p.c(1.00, -85.70, 2.90, -85.10, 3.50, -83.60);
        p.c(4.00, -81.10, 4.60, -75.40, 5.00, -74.00);
        p.l(6.30, -69.60);
        p.l(0.00, -68.40);
        p.l(-6.60, -69.60);
        p.z();
        if let Some(path) = p.finish() {
            geom::fill(layer, &path, &paint(pal.goose_orange, 255));
        }
    }

    // Head (path 4).
    {
        let mut p = P::new(&head);
        p.m(0.00, -72.20);
        p.c(-6.50, -72.20, -10.70, -67.50, -10.80, -57.40);
        p.c(-10.90, -49.80, -5.60, -43.20, -2.00, -43.00);
        p.l(1.90, -43.00);
        p.c(4.40, -42.90, 10.70, -48.60, 10.70, -57.00);
        p.c(10.60, -64.10, 7.90, -72.20, 0.00, -72.20);
        p.z();
        if let Some(path) = p.finish() {
            outline_fill(layer, &path, pal.goose_white, pal.goose_outline, ow, 255);
        }
    }
}
