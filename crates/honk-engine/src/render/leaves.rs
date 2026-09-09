//! Bounded supersampled sprites built once from the reviewed SVG leaf family.
use super::geom::paint;
use super::leaf_art;
use crate::math::Vec2;
use std::sync::OnceLock;
use tiny_skia::{FillRule, FilterQuality, Pixmap, PixmapPaint, Transform};

// Sixteen fixed 48px RGBA images occupy 144 KiB, regardless of pile count or lifetime.
// Rasterize at 2x once; preserve smooth rotation without retessellating hundreds of veins
// for every particle on every presentation.
const SPRITE_SIZE: u32 = 48;
const SUPERSAMPLE: f32 = 2.0;
static SPRITES: OnceLock<[[Pixmap; 4]; 4]> = OnceLock::new();

fn sprites() -> [[Pixmap; 4]; 4] {
    let paths = leaf_art::paths();
    std::array::from_fn(|species| {
        std::array::from_fn(|color| {
            let mut image = Pixmap::new(SPRITE_SIZE, SPRITE_SIZE).expect("fixed leaf sprite");
            let rgb = leaf_art::PALETTE[color];
            let vein = (
                rgb.0.saturating_sub(38),
                rgb.1.saturating_sub(28),
                rgb.2.saturating_sub(12),
            );
            for (path, rgb) in paths[species].iter().zip([rgb, vein]) {
                image.fill_path(
                    path,
                    &paint(rgb, 255),
                    FillRule::Winding,
                    Transform::from_row(SUPERSAMPLE, 0.0, 0.0, SUPERSAMPLE, 24.0, 24.0),
                    None,
                );
            }
            image
        })
    })
}

pub(super) fn render(
    pixmap: &mut Pixmap,
    center: Vec2,
    angle: f32,
    size: f32,
    variant: usize,
    color: usize,
    alpha: u8,
) {
    let sprite = &SPRITES.get_or_init(sprites)[variant % 4][color];
    let (sin, cos) = angle.sin_cos();
    let mirror = if variant & 4 == 0 { 1.0 } else { -1.0 };
    let sx = cos * size * mirror / SUPERSAMPLE;
    let ky = sin * size * mirror / SUPERSAMPLE;
    let kx = -sin * size / SUPERSAMPLE;
    let sy = cos * size / SUPERSAMPLE;
    let transform = Transform::from_row(
        sx,
        ky,
        kx,
        sy,
        center.x - (sx + kx) * SPRITE_SIZE as f32 * 0.5,
        center.y - (ky + sy) * SPRITE_SIZE as f32 * 0.5,
    );
    pixmap.draw_pixmap(
        0,
        0,
        sprite.as_ref(),
        &PixmapPaint {
            opacity: f32::from(alpha) / 255.0,
            quality: FilterQuality::Bilinear,
            ..PixmapPaint::default()
        },
        transform,
        None,
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::autumn::LEAF_RENDER_RADIUS;

    #[test]
    fn every_leaf_and_mirror_stays_inside_damage_bounds_while_tumbling() {
        for variant in 0..8 {
            for degrees in (0..360).step_by(5) {
                let mut image = Pixmap::new(64, 64).unwrap();
                render(
                    &mut image,
                    Vec2::new(32.0, 32.0),
                    (degrees as f32).to_radians(),
                    1.11,
                    variant,
                    variant % 4,
                    225,
                );
                let mut visible = 0;
                for (index, pixel) in image.pixels().iter().enumerate() {
                    if pixel.alpha() == 0 {
                        continue;
                    }
                    visible += 1;
                    let x = (index % 64) as f32 - 32.0;
                    let y = (index / 64) as f32 - 32.0;
                    assert!(
                        x.abs() < LEAF_RENDER_RADIUS && y.abs() < LEAF_RENDER_RADIUS,
                        "leaf {variant}, heading {degrees}: pixel outside shared damage bounds"
                    );
                }
                assert!(visible > 40, "a botanical leaf must remain visible");
            }
        }
    }

    #[test]
    fn the_four_species_have_distinct_desktop_silhouettes() {
        let silhouettes: Vec<Vec<bool>> = (0..4)
            .map(|variant| {
                let mut image = Pixmap::new(32, 32).unwrap();
                render(&mut image, Vec2::new(16.0, 16.0), 0.0, 1.0, variant, 0, 255);
                image.pixels().iter().map(|p| p.alpha() > 100).collect()
            })
            .collect();
        for a in 0..4 {
            for b in a + 1..4 {
                let different = silhouettes[a]
                    .iter()
                    .zip(&silhouettes[b])
                    .filter(|(left, right)| left != right)
                    .count();
                assert!(different > 25, "leaf species {a} and {b} look too similar");
            }
        }
    }
}
