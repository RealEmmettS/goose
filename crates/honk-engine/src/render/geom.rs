//! Small shared raster primitives.
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
