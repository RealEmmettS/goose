//! Shared geometry/drawing helpers for the flat-illustration goose renderer.
//!
//! Part shapes are authored in **reference-art coordinates** (see
//! `docs/art-reference/`). A [`Frame`] maps those coordinates into layer pixels via an
//! affine basis, so the same authored path serves both facings (mirroring), any head
//! tilt (rotation), and the top-down view (heading rotation) without touching the
//! coordinates themselves.

use crate::math::Vec2;
use tiny_skia::{FillRule, LineCap, Paint, Path, PathBuilder, Pixmap, Stroke, Transform};

pub fn paint(rgb: (u8, u8, u8), alpha: u8) -> Paint<'static> {
    let mut p = Paint::default();
    p.set_color_rgba8(rgb.0, rgb.1, rgb.2, alpha);
    p.anti_alias = true;
    p
}

pub fn fill(pixmap: &mut Pixmap, path: &Path, p: &Paint) {
    pixmap.fill_path(path, p, FillRule::Winding, Transform::identity(), None);
}

pub fn stroke(pixmap: &mut Pixmap, path: &Path, p: &Paint, width: f32) {
    let stroke = Stroke {
        width,
        line_cap: LineCap::Round,
        ..Stroke::default()
    };
    pixmap.stroke_path(path, p, &stroke, Transform::identity(), None);
}

/// The reference art is outline-less flat design; the goose keeps a thin outline for
/// contrast against arbitrary desktops. Drawn under the fill (the classic
/// outline-under-fill trick: overlapping parts cover each other's inner outlines).
pub fn outline_fill(
    pixmap: &mut Pixmap,
    path: &Path,
    fill_rgb: (u8, u8, u8),
    outline_rgb: (u8, u8, u8),
    outline_w: f32,
    alpha: u8,
) {
    stroke(pixmap, path, &paint(outline_rgb, alpha), outline_w * 2.0);
    fill(pixmap, path, &paint(fill_rgb, alpha));
}

/// Affine part frame: maps a reference-art point `(x, y)` (relative to the part anchor
/// `(ax, ay)`) into layer pixels: `pt(x, y) = origin + fx·(x-ax) + fy·(y-ay)`.
#[derive(Debug, Clone, Copy)]
pub struct Frame {
    pub origin: Vec2,
    pub fx: Vec2,
    pub fy: Vec2,
    pub ax: f32,
    pub ay: f32,
}

impl Frame {
    /// Side-view frame. `k` = pixels per ref unit (incl. supersample), `mirror` = -1
    /// facing screen-left (art native) / +1 facing right, `tilt` = head tilt radians
    /// (positive dips the beak).
    pub fn side(origin: Vec2, k: f32, mirror: f32, tilt: f32, ax: f32, ay: f32) -> Self {
        let (s, c) = tilt.sin_cos();
        // Derived from: forward = (ax-x), up = (ay-y), rotated by tilt, then mapped
        // onto screen axes fwd=(mirror,0), up=(0,-1).
        Self {
            origin,
            fx: Vec2::new(-mirror * k * c, -k * s),
            fy: Vec2::new(-mirror * k * s, k * c),
            ax,
            ay,
        }
    }

    /// Top-down frame rotated to `heading` (unit). Art native heading is up (-y);
    /// art +x maps to the goose's right.
    pub fn top(origin: Vec2, k: f32, heading: Vec2, ax: f32, ay: f32) -> Self {
        let right = Vec2::new(-heading.y, heading.x);
        Self {
            origin,
            fx: right * k,
            fy: heading * -k,
            ax,
            ay,
        }
    }

    pub fn pt(&self, x: f32, y: f32) -> Vec2 {
        self.origin + self.fx * (x - self.ax) + self.fy * (y - self.ay)
    }
}

/// Path builder in a [`Frame`]: authored reference coordinates go straight in.
pub struct P<'f> {
    pb: PathBuilder,
    f: &'f Frame,
}

impl<'f> P<'f> {
    pub fn new(f: &'f Frame) -> Self {
        Self {
            pb: PathBuilder::new(),
            f,
        }
    }

    pub fn m(&mut self, x: f32, y: f32) {
        let p = self.f.pt(x, y);
        self.pb.move_to(p.x, p.y);
    }

    pub fn l(&mut self, x: f32, y: f32) {
        let p = self.f.pt(x, y);
        self.pb.line_to(p.x, p.y);
    }

    pub fn c(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, x: f32, y: f32) {
        let p1 = self.f.pt(x1, y1);
        let p2 = self.f.pt(x2, y2);
        let p = self.f.pt(x, y);
        self.pb.cubic_to(p1.x, p1.y, p2.x, p2.y, p.x, p.y);
    }

    pub fn z(&mut self) {
        self.pb.close();
    }

    pub fn finish(self) -> Option<Path> {
        self.pb.finish()
    }
}

/// Circle/ellipse helpers in layer pixels.
pub fn disc(pixmap: &mut Pixmap, c: Vec2, radius: f32, p: &Paint) {
    if let Some(path) = PathBuilder::from_circle(c.x, c.y, radius) {
        fill(pixmap, &path, p);
    }
}

pub fn ellipse(pixmap: &mut Pixmap, center: Vec2, rx: f32, ry: f32, p: &Paint) {
    if rx <= 0.0 || ry <= 0.0 {
        return;
    }
    let transform = Transform::from_row(rx, 0.0, 0.0, ry, center.x, center.y);
    if let Some(path) = PathBuilder::from_circle(0.0, 0.0, 1.0) {
        pixmap.fill_path(&path, p, FillRule::Winding, transform, None);
    }
}

/// Dotted elliptical ground shadow (the original's dithered look), supersample-aware.
pub fn stipple_shadow(pixmap: &mut Pixmap, center: Vec2, rx: f32, ry: f32, ss: f32) {
    let dot = paint((0x20, 0x20, 0x20), 42);
    let step = 3.0 * ss;
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
                disc(
                    pixmap,
                    center + Vec2::new(dx + row_off, dy),
                    0.85 * ss,
                    &dot,
                );
            }
            dx += step;
        }
        dy += step;
    }
}
