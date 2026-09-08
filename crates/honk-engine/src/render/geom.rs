//! Small shared raster primitives.
use crate::math::Vec2;
use tiny_skia::{FillRule, LineCap, Paint, Path, PathBuilder, Pixmap, Stroke, Transform};

/// Shared vector operations; the desktop remains a statically dispatched Pixmap.
pub(super) trait VectorCanvas {
    fn fill(&mut self, path: &Path, paint: &Paint, transform: Transform);
    fn stroke(&mut self, path: &Path, paint: &Paint, stroke: &Stroke);
}

impl VectorCanvas for Pixmap {
    fn fill(&mut self, path: &Path, paint: &Paint, transform: Transform) {
        self.fill_path(path, paint, FillRule::Winding, transform, None);
    }

    fn stroke(&mut self, path: &Path, paint: &Paint, stroke: &Stroke) {
        self.stroke_path(path, paint, stroke, Transform::identity(), None);
    }
}

pub fn paint(rgb: (u8, u8, u8), alpha: u8) -> Paint<'static> {
    let mut p = Paint::default();
    p.set_color_rgba8(rgb.0, rgb.1, rgb.2, alpha);
    p.anti_alias = true;
    p
}

pub fn fill(pixmap: &mut impl VectorCanvas, path: &Path, p: &Paint) {
    pixmap.fill(path, p, Transform::identity());
}

pub fn stroke(pixmap: &mut impl VectorCanvas, path: &Path, p: &Paint, width: f32) {
    let stroke = Stroke {
        width,
        line_cap: LineCap::Round,
        ..Stroke::default()
    };
    pixmap.stroke(path, p, &stroke);
}

/// Circle/ellipse helpers in layer pixels.
pub fn disc(pixmap: &mut impl VectorCanvas, c: Vec2, radius: f32, p: &Paint) {
    if let Some(path) = PathBuilder::from_circle(c.x, c.y, radius) {
        fill(pixmap, &path, p);
    }
}

pub fn ellipse(pixmap: &mut impl VectorCanvas, center: Vec2, rx: f32, ry: f32, p: &Paint) {
    if rx <= 0.0 || ry <= 0.0 {
        return;
    }
    let transform = Transform::from_row(rx, 0.0, 0.0, ry, center.x, center.y);
    if let Some(path) = PathBuilder::from_circle(0.0, 0.0, 1.0) {
        pixmap.fill(&path, p, transform);
    }
}
