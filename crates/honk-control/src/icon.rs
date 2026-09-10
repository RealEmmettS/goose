//! The application artwork shared by native control surfaces.

use tiny_skia::{FilterQuality, Pixmap, PixmapPaint, Transform};

pub const APP_ICON_PNG: &[u8] = include_bytes!("../../../settings/assets/icon.png");
pub const TRAY_ICON_SIZE: u32 = 36;

/// Rasterize once when the native tray is created. Keep the original colors,
/// transparent margin and complete silhouette, without adding a platform badge.
pub fn tray_pixmap() -> Result<Pixmap, String> {
    let source = Pixmap::decode_png(APP_ICON_PNG).map_err(|error| error.to_string())?;
    let mut output = Pixmap::new(TRAY_ICON_SIZE, TRAY_ICON_SIZE)
        .ok_or_else(|| "could not allocate tray icon".to_owned())?;
    output.draw_pixmap(
        0,
        0,
        source.as_ref(),
        &PixmapPaint {
            quality: FilterQuality::Bicubic,
            ..PixmapPaint::default()
        },
        Transform::from_scale(
            TRAY_ICON_SIZE as f32 / source.width() as f32,
            TRAY_ICON_SIZE as f32 / source.height() as f32,
        ),
        None,
    );
    Ok(output)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tray_keeps_application_colors_and_transparency_at_native_size() {
        let pixmap = tray_pixmap().unwrap();
        assert_eq!((pixmap.width(), pixmap.height()), (36, 36));
        assert_eq!(pixmap.pixel(0, 0).unwrap().alpha(), 0);
        let pixels: Vec<_> = pixmap.pixels().iter().map(|p| p.demultiply()).collect();
        assert!(pixels
            .iter()
            .any(|p| p.alpha() > 240 && p.red() > 220 && p.green() < 170 && p.blue() < 80));
        assert!(pixels
            .iter()
            .any(|p| p.alpha() > 240 && p.red() > 230 && p.green() > 230 && p.blue() > 230));
        // The prior blue disk and white mask must never replace the app art.
        assert!(!pixels
            .iter()
            .any(|p| p.alpha() > 240 && p.blue() > p.red().saturating_add(20)));
    }
}
