//! Editable paths emitted by the same projected drawing routine as the desktop.
use super::{geom::VectorCanvas, projected, RenderPalette};
use crate::{math::Vec2, rig::Rig};
use std::fmt::Write;
use tiny_skia::{Paint, Path, PathSegment, Shader, Stroke, Transform};

/// Export one pose without raster images, external resources, scripts or fonts.
/// World-space dimensions and origin have the same meaning as `render_rig_scaled`.
pub fn render_rig_svg(
    rig: &Rig,
    origin: Vec2,
    world_width: f32,
    world_height: f32,
    scale: f32,
    palette: RenderPalette,
) -> Option<String> {
    let width = world_width * scale;
    let height = world_height * scale;
    if ![origin.x, origin.y, width, height, scale]
        .into_iter()
        .all(f32::is_finite)
        || width <= 0.0
        || height <= 0.0
        || scale <= 0.0
    {
        return None;
    }
    let mut canvas = SvgCanvas {
        text: format!(
            "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"{width}\" height=\"{height}\" viewBox=\"0 0 {width} {height}\" fill-rule=\"nonzero\">\n<title>Honk300 goose</title>\n"
        ),
        paths: 0,
    };
    projected::paint_goose(&mut canvas, rig, origin, scale, &palette);
    canvas.text.push_str("</svg>\n");
    Some(canvas.text)
}

struct SvgCanvas {
    text: String,
    paths: usize,
}

impl SvgCanvas {
    fn path(&mut self, path: &Path, paint: &Paint, transform: Transform, stroke: Option<&Stroke>) {
        let Shader::SolidColor(color) = paint.shader else {
            unreachable!("the projected goose uses solid vector paints")
        };
        let rgba = color.to_color_u8();
        let hex = format!("#{:02x}{:02x}{:02x}", rgba.red(), rgba.green(), rgba.blue());
        self.paths += 1;
        write!(self.text, "<path id=\"shape-{}\" d=\"", self.paths).unwrap();
        for segment in path.segments() {
            match segment {
                PathSegment::MoveTo(p) => write!(self.text, "M{} {}", p.x, p.y),
                PathSegment::LineTo(p) => write!(self.text, "L{} {}", p.x, p.y),
                PathSegment::QuadTo(a, b) => write!(self.text, "Q{} {} {} {}", a.x, a.y, b.x, b.y),
                PathSegment::CubicTo(a, b, c) => write!(
                    self.text,
                    "C{} {} {} {} {} {}",
                    a.x, a.y, b.x, b.y, c.x, c.y
                ),
                PathSegment::Close => write!(self.text, "Z"),
            }
            .unwrap();
        }
        self.text.push('"');
        if let Some(stroke) = stroke {
            write!(self.text, " fill=\"none\" stroke=\"{hex}\" stroke-width=\"{}\" stroke-linecap=\"round\" stroke-linejoin=\"miter\" stroke-miterlimit=\"{}\" stroke-opacity=\"{}\"",
                stroke.width, stroke.miter_limit, color.alpha()).unwrap();
        } else {
            write!(
                self.text,
                " fill=\"{hex}\" fill-opacity=\"{}\"",
                color.alpha()
            )
            .unwrap();
        }
        if transform != Transform::identity() {
            write!(
                self.text,
                " transform=\"matrix({} {} {} {} {} {})\"",
                transform.sx, transform.ky, transform.kx, transform.sy, transform.tx, transform.ty
            )
            .unwrap();
        }
        self.text.push_str("/>\n");
    }
}

impl VectorCanvas for SvgCanvas {
    fn fill(&mut self, path: &Path, paint: &Paint, transform: Transform) {
        self.path(path, paint, transform, None);
    }

    fn stroke(&mut self, path: &Path, paint: &Paint, stroke: &Stroke) {
        self.path(path, paint, Transform::identity(), Some(stroke));
    }
}
