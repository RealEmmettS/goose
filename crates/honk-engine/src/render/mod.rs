//! Clean-room procedural renderer V2 (ADR 0014): a [`GoosePose`] → premultiplied
//! `tiny_skia::Pixmap`, in the flat-illustration style of the project's own reference
//! art (`docs/art-reference/`).
//!
//! Each view (side profile / top-down) renders into its own supersampled layer, then
//! the layers composite onto the destination with per-layer opacity — that is what
//! makes the ~125 ms view crossfade look right (parts blend as a whole goose, not as
//! stacked translucent shapes) and gives the fine detail (feather scallops, nostril,
//! webbed feet) crisp edges at desktop size. Platform-free and offscreen: the same
//! routines feed the overlays and the golden-frame tests.

mod geom;
mod side;
mod top;

use crate::autumn::{AutumnLeafColor, AutumnState};
use crate::footmarks::{FootMarkTiming, FootMarks};
use crate::math::Vec2;
use crate::rig::{GoosePose, Rig, RigView};
use geom::{disc, ellipse, paint};
use std::cell::RefCell;
use tiny_skia::{Color, FilterQuality, Pixmap, PixmapPaint, Transform};

/// Goose raster supersample factor (rendered at 2x, composited down at 0.5x).
pub const GOOSE_SUPERSAMPLE: f32 = 2.0;

const MUD: (u8, u8, u8) = (0x5a, 0x40, 0x28);
const LEAF_GOLD: (u8, u8, u8) = (0xe2, 0xb8, 0x35);
const LEAF_ORANGE: (u8, u8, u8) = (0xd9, 0x6a, 0x21);
const LEAF_RED: (u8, u8, u8) = (0xa9, 0x3b, 0x2a);
const LEAF_BROWN: (u8, u8, u8) = (0x7a, 0x4a, 0x24);

thread_local! {
    /// Render calls on a platform thread reuse one supersampled layer. The two views are
    /// painted/composited sequentially, so a second per-frame allocation is unnecessary.
    static LAYER_SCRATCH: RefCell<LayerScratch> = RefCell::new(LayerScratch::default());
}

#[derive(Default)]
struct LayerScratch {
    pixmap: Option<Pixmap>,
    #[cfg(test)]
    allocations: usize,
}

impl LayerScratch {
    fn prepare(&mut self, width: u32, height: u32) -> &mut Pixmap {
        let width = width.max(1);
        let height = height.max(1);
        let needs_growth = self
            .pixmap
            .as_ref()
            .is_none_or(|pixmap| pixmap.width() < width || pixmap.height() < height);
        if needs_growth {
            // Goose bounds breathe and bob by a handful of supersampled pixels. Power-of-two
            // growth made a 257 px body allocate, clear, paint, filter, and composite 512 px on
            // every frame. Small grow-only quanta retain allocation stability without asking the
            // rasterizer to process a large transparent fringe.
            let current_width = self.pixmap.as_ref().map_or(0, Pixmap::width);
            let current_height = self.pixmap.as_ref().map_or(0, Pixmap::height);
            let width = rounded_scratch_extent(width.max(current_width));
            let height = rounded_scratch_extent(height.max(current_height));
            self.pixmap = Pixmap::new(width, height);
            #[cfg(test)]
            {
                self.allocations += 1;
            }
        }
        let pixmap = self
            .pixmap
            .as_mut()
            .expect("small renderer scratch allocation");
        pixmap.fill(Color::TRANSPARENT);
        pixmap
    }
}

fn rounded_scratch_extent(extent: u32) -> u32 {
    extent.saturating_add(31) / 32 * 32
}

/// User-customizable goose palette — six tones, defaults from the reference art.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RenderPalette {
    /// Base body/head/neck tone.
    pub goose_white: (u8, u8, u8),
    /// Soft shading: throat, underbody, behind-wing, tail underside.
    pub goose_shade: (u8, u8, u8),
    /// The layered dark-slate wing (and the eye).
    pub goose_wing: (u8, u8, u8),
    /// Beak top, legs, feet.
    pub goose_orange: (u8, u8, u8),
    /// Beak underside/mouth-line, nostril, far leg.
    pub goose_orange_dark: (u8, u8, u8),
    /// Thin contrast outline around body-toned forms.
    pub goose_outline: (u8, u8, u8),
}

impl Default for RenderPalette {
    fn default() -> Self {
        Self {
            goose_white: (0xed, 0xed, 0xed),
            goose_shade: (0xc6, 0xc6, 0xc6),
            goose_wing: (0x51, 0x55, 0x57),
            goose_orange: (0xfc, 0x79, 0x27),
            goose_orange_dark: (0xd1, 0x55, 0x1b),
            goose_outline: (0xc9, 0xc9, 0xc9),
        }
    }
}

impl RenderPalette {
    /// Build a palette from the three legacy config tones (M15 files predate the
    /// six-tone palette): the missing tones are derived so old configs stay coherent.
    pub fn from_legacy(
        goose_white: (u8, u8, u8),
        goose_orange: (u8, u8, u8),
        goose_outline: (u8, u8, u8),
    ) -> Self {
        let d = Self::default();
        Self {
            goose_white,
            goose_shade: mix(goose_white, goose_outline, 0.55),
            goose_wing: d.goose_wing,
            goose_orange,
            goose_orange_dark: darken(goose_orange),
            goose_outline,
        }
    }
}

fn darken(rgb: (u8, u8, u8)) -> (u8, u8, u8) {
    (
        (rgb.0 as f32 * 0.85) as u8,
        (rgb.1 as f32 * 0.73) as u8,
        (rgb.2 as f32 * 0.55) as u8,
    )
}

fn mix(a: (u8, u8, u8), b: (u8, u8, u8), t: f32) -> (u8, u8, u8) {
    let m = |x: u8, y: u8| (x as f32 + (y as f32 - x as f32) * t) as u8;
    (m(a.0, b.0), m(a.1, b.1), m(a.2, b.2))
}

/// Which side of the goose the Autumn leaves render on.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AutumnRenderLayer {
    BelowGoose,
    AboveGoose,
}

fn leaf_rgb(color: AutumnLeafColor) -> (u8, u8, u8) {
    match color {
        AutumnLeafColor::Gold => LEAF_GOLD,
        AutumnLeafColor::Orange => LEAF_ORANGE,
        AutumnLeafColor::Red => LEAF_RED,
        AutumnLeafColor::Brown => LEAF_BROWN,
    }
}

/// Render built-in Autumn leaves. Call `BelowGoose` before the goose and `AboveGoose` after it.
pub fn render_autumn_leaves(
    pixmap: &mut Pixmap,
    autumn: &AutumnState,
    now: f64,
    origin: Vec2,
    goose_pos: Vec2,
    layer: AutumnRenderLayer,
) {
    for pile in autumn.piles() {
        let spawn = pile.spawn_scale(now);
        let fade = pile.fade_out(now);
        let alpha_scale = 1.0 - fade;
        if layer == AutumnRenderLayer::BelowGoose && pile.kicked_at.is_none() {
            let radius = pile.radius * spawn;
            let center = pile.position - origin;
            ellipse(
                pixmap,
                center,
                radius,
                radius * 0.6,
                &paint((0x31, 0x25, 0x1a), (48.0 * alpha_scale) as u8),
            );
        }

        for leaf in &pile.leaves {
            let world = pile.position + leaf.screen_offset() * spawn;
            let leaf_above_goose = world.y >= goose_pos.y;
            if matches!(layer, AutumnRenderLayer::AboveGoose) != leaf_above_goose {
                continue;
            }
            let alpha = (225.0 * alpha_scale) as u8;
            if alpha == 0 {
                continue;
            }
            let center = world - origin;
            let rgb = leaf_rgb(leaf.color);
            ellipse(pixmap, center, 3.5, 2.0, &paint(rgb, alpha));
        }
    }
}

/// Render the muddy footprints into `pixmap` (call before the goose so it sits on top).
pub fn render_footmarks(pixmap: &mut Pixmap, marks: &FootMarks, now: f64, origin: Vec2) {
    render_footmarks_with_timing(pixmap, marks, now, origin, FootMarkTiming::default());
}

/// Render muddy footprints with runtime timing from config.
pub fn render_footmarks_with_timing(
    pixmap: &mut Pixmap,
    marks: &FootMarks,
    now: f64,
    origin: Vec2,
    timing: FootMarkTiming,
) {
    for (mark, scale) in marks.active_with_timing(now, timing) {
        let c = mark.position - origin;
        let alpha = (180.0 * scale) as u8;
        disc(pixmap, c, 3.5 * scale, &paint(MUD, alpha));
    }
}

/// Render the rising/fading heart particles (M6 pat-streak; call after the goose so
/// hearts float on top).
pub fn render_hearts(pixmap: &mut Pixmap, hearts: &crate::hearts::Hearts, now: f64, origin: Vec2) {
    const HEART: (u8, u8, u8) = (0xff, 0x5a, 0x7a);
    const LOBE: f32 = 3.4;
    for (pos, alpha) in hearts.active(now) {
        let a = (alpha * 255.0) as u8;
        if a == 0 {
            continue;
        }
        let p = paint(HEART, a);
        let c = pos - origin;
        disc(pixmap, c + Vec2::new(-LOBE * 0.85, -LOBE * 0.45), LOBE, &p);
        disc(pixmap, c + Vec2::new(LOBE * 0.85, -LOBE * 0.45), LOBE, &p);
        let mut pb = tiny_skia::PathBuilder::new();
        let a1 = c + Vec2::new(-LOBE * 1.75, -LOBE * 0.1);
        let b1 = c + Vec2::new(LOBE * 1.75, -LOBE * 0.1);
        let d1 = c + Vec2::new(0.0, LOBE * 1.9);
        pb.move_to(a1.x, a1.y);
        pb.line_to(b1.x, b1.y);
        pb.line_to(d1.x, d1.y);
        pb.close();
        if let Some(path) = pb.finish() {
            geom::fill(pixmap, &path, &p);
        }
    }
}

/// Render sleepy-mood Z particles above the goose.
pub fn render_sleepies(
    pixmap: &mut Pixmap,
    sleepies: &crate::mood::ZParticles,
    now: f64,
    origin: Vec2,
) {
    for (pos, alpha) in sleepies.active(now) {
        let a = (alpha * 220.0) as u8;
        if a == 0 {
            continue;
        }
        let c = pos - origin;
        let p = paint((0x88, 0x99, 0xaa), a);
        let mut pb = tiny_skia::PathBuilder::new();
        pb.move_to(c.x - 4.0, c.y - 5.0);
        pb.line_to(c.x + 4.0, c.y - 5.0);
        pb.line_to(c.x - 4.0, c.y + 5.0);
        pb.line_to(c.x + 4.0, c.y + 5.0);
        if let Some(path) = pb.finish() {
            geom::stroke(pixmap, &path, &p, 1.8);
        }
    }
}

/// Render one view of the goose into a fresh supersampled layer and composite it onto
/// `pixmap` with `opacity`.
fn render_rig_layer(
    pixmap: &mut Pixmap,
    rig: &Rig,
    origin: Vec2,
    palette: RenderPalette,
    opacity: f32,
    supersample: f32,
) {
    let ss = supersample.max(1.0);
    if !layer_needs_offscreen_composite(ss, opacity) {
        match rig.view {
            RigView::Side { .. } => side::paint_side(pixmap, rig, origin, ss, &palette),
            RigView::TopDown { .. } => top::paint_top(pixmap, rig, origin, ss, &palette),
        }
        return;
    }
    let bb = rig.bounding_box();
    let w = (bb.width() * ss).ceil() as u32;
    let h = (bb.height() * ss).ceil() as u32;
    LAYER_SCRATCH.with(|scratch| {
        let mut scratch = scratch.borrow_mut();
        let layer = scratch.prepare(w, h);
        match rig.view {
            RigView::Side { .. } => side::paint_side(layer, rig, bb.min, ss, &palette),
            RigView::TopDown { .. } => top::paint_top(layer, rig, bb.min, ss, &palette),
        }
        let inv = 1.0 / ss;
        let transform = Transform::from_scale(inv, inv)
            .post_translate(bb.min.x - origin.x, bb.min.y - origin.y);
        pixmap.draw_pixmap(
            0,
            0,
            layer.as_ref(),
            &PixmapPaint {
                opacity: opacity.clamp(0.0, 1.0),
                quality: if ss > 1.0 {
                    FilterQuality::Bilinear
                } else {
                    // At 1x there is no downsampling to filter. Nearest preserves the
                    // antialiased tiny-skia pixels and avoids a second bilinear pass.
                    FilterQuality::Nearest
                },
                ..PixmapPaint::default()
            },
            transform,
            None,
        );
    });
}

fn layer_needs_offscreen_composite(supersample: f32, opacity: f32) -> bool {
    (supersample - 1.0).abs() > f32::EPSILON || opacity < 1.0 - f32::EPSILON
}

/// Render the full drawable pose — the active view plus, mid-crossfade, the outgoing
/// view underneath at its remaining opacity.
pub fn render_pose_with_palette(
    pixmap: &mut Pixmap,
    pose: &GoosePose,
    origin: Vec2,
    palette: RenderPalette,
) {
    render_pose_with_palette_at_scale(pixmap, pose, origin, palette, GOOSE_SUPERSAMPLE);
}

/// Render the full pose at an explicit raster scale. Native backends may select the
/// 1x antialiased path when their compositor already performs a second image copy;
/// golden/reference renders retain [`GOOSE_SUPERSAMPLE`].
pub fn render_pose_with_palette_at_scale(
    pixmap: &mut Pixmap,
    pose: &GoosePose,
    origin: Vec2,
    palette: RenderPalette,
    supersample: f32,
) {
    let mut incoming_opacity = 1.0;
    if let Some((rig, alpha)) = &pose.fading {
        render_rig_layer(pixmap, rig, origin, palette, *alpha, supersample);
        incoming_opacity = 1.0 - alpha.clamp(0.0, 1.0);
    }
    render_rig_layer(
        pixmap,
        &pose.primary,
        origin,
        palette,
        incoming_opacity,
        supersample,
    );
}

/// Render one goose view with the default palette (tests/tools).
pub fn render_rig(pixmap: &mut Pixmap, rig: &Rig, origin: Vec2) {
    render_rig_with_palette(pixmap, rig, origin, RenderPalette::default());
}

/// Render one goose view with an explicit palette.
pub fn render_rig_with_palette(
    pixmap: &mut Pixmap,
    rig: &Rig,
    origin: Vec2,
    palette: RenderPalette,
) {
    render_rig_layer(pixmap, rig, origin, palette, 1.0, GOOSE_SUPERSAMPLE);
}

/// Render one goose view into a fresh transparent pixmap at an arbitrary vector scale
/// (px per world px) with **no** downsampling — crisp output for tools/marketing/site
/// assets. `origin` is the world point mapped to the pixmap's top-left; the pixmap is
/// `width`×`height` *world units* large before scaling.
pub fn render_rig_scaled(
    rig: &Rig,
    origin: Vec2,
    world_width: f32,
    world_height: f32,
    scale: f32,
    palette: RenderPalette,
) -> Option<Pixmap> {
    let w = (world_width * scale).ceil() as u32;
    let h = (world_height * scale).ceil() as u32;
    let mut pixmap = Pixmap::new(w.max(1), h.max(1))?;
    pixmap.fill(Color::TRANSPARENT);
    match rig.view {
        RigView::Side { .. } => side::paint_side(&mut pixmap, rig, origin, scale, &palette),
        RigView::TopDown { .. } => top::paint_top(&mut pixmap, rig, origin, scale, &palette),
    }
    Some(pixmap)
}

/// Convenience for tests/tools: allocate a `width`×`height` transparent pixmap and
/// render the goose so its bounding box is centred. Returns `None` if allocation fails.
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
    use crate::rig::{Rig, RigAnim, RigInput};

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
    fn each_palette_tone_changes_pixels() {
        let rig = Rig::default();
        let base = render_centered(256, 256, &rig).expect("alloc");
        let tones: [fn(&mut RenderPalette); 6] = [
            |p| p.goose_white = (0x20, 0x60, 0xff),
            |p| p.goose_shade = (0x20, 0xff, 0x60),
            |p| p.goose_wing = (0xff, 0x20, 0x60),
            |p| p.goose_orange = (0x10, 0x10, 0xa0),
            |p| p.goose_orange_dark = (0xa0, 0x10, 0x10),
            |p| p.goose_outline = (0x00, 0x00, 0x00),
        ];
        for (i, tweak) in tones.iter().enumerate() {
            let mut pal = RenderPalette::default();
            tweak(&mut pal);
            let mut custom = Pixmap::new(256, 256).expect("alloc");
            custom.fill(Color::TRANSPARENT);
            let bb = rig.bounding_box();
            let origin = (bb.min + bb.max) * 0.5 - Vec2::new(128.0, 128.0);
            render_rig_with_palette(&mut custom, &rig, origin, pal);
            assert_ne!(base.data(), custom.data(), "tone {i} had no visible effect");
        }
    }

    #[test]
    fn legacy_palette_derives_missing_tones() {
        let p =
            RenderPalette::from_legacy((0xff, 0xff, 0xff), (0xff, 0xa5, 0x00), (0xd3, 0xd3, 0xd3));
        assert_eq!(p.goose_white, (0xff, 0xff, 0xff));
        assert_eq!(p.goose_orange, (0xff, 0xa5, 0x00));
        assert_ne!(p.goose_shade, p.goose_white);
        assert_ne!(p.goose_orange_dark, p.goose_orange);
        assert_eq!(p.goose_wing, RenderPalette::default().goose_wing);
    }

    #[test]
    fn crossfade_draws_both_views() {
        use crate::math::Vec2;
        use crate::time::DT;
        let center = Vec2::new(300.0, 300.0);
        let mut anim = RigAnim::new(center, 0.0);
        anim.update(&RigInput::static_pose(center, 0.0, 0.0));
        let pose = anim.update(&RigInput {
            center,
            direction_deg: 90.0,
            neck_target: 0.0,
            speed: 80.0,
            velocity: Vec2::new(0.0, 80.0),
            step_time: 0.2,
            now: DT as f64,
            dt: DT,
        });
        assert!(pose.fading.is_some(), "expected an active crossfade");
        let bb = pose.bounding_box();
        let w = bb.width().ceil() as u32 + 8;
        let h = bb.height().ceil() as u32 + 8;
        let mut pixmap = Pixmap::new(w, h).expect("alloc");
        pixmap.fill(Color::TRANSPARENT);
        render_pose_with_palette(
            &mut pixmap,
            &pose,
            bb.min - Vec2::new(4.0, 4.0),
            RenderPalette::default(),
        );
        let opaque = pixmap
            .data()
            .chunks_exact(4)
            .filter(|px| px[3] > 60)
            .count();
        assert!(opaque > 800, "expected both views visible, got {opaque}");
    }

    #[test]
    fn crossfade_uses_complementary_layer_opacity() {
        let primary = Rig::update(Vec2::new(90.0, 130.0), 0.0, 0.45, 0.0);
        let outgoing = Rig::update(Vec2::new(270.0, 130.0), 180.0, 0.45, 0.0);
        let pose = GoosePose {
            primary,
            fading: Some((outgoing, 0.25)),
        };
        let mut pixmap = Pixmap::new(360, 240).expect("alloc");
        pixmap.fill(Color::TRANSPARENT);

        render_pose_with_palette(&mut pixmap, &pose, Vec2::ZERO, RenderPalette::default());

        let max_alpha = |x0: usize, x1: usize| {
            let width = pixmap.width() as usize;
            pixmap
                .data()
                .chunks_exact(4)
                .enumerate()
                .filter(|(index, _)| {
                    let x = index % width;
                    x >= x0 && x < x1
                })
                .map(|(_, pixel)| pixel[3])
                .max()
                .unwrap_or(0)
        };
        let incoming_alpha = max_alpha(0, 180);
        let outgoing_alpha = max_alpha(180, 360);
        assert!(
            (185..=200).contains(&incoming_alpha),
            "incoming view should be 1-t opaque, got {incoming_alpha}"
        );
        assert!(
            (58..=70).contains(&outgoing_alpha),
            "outgoing view should be t opaque, got {outgoing_alpha}"
        );
    }

    #[test]
    fn layer_scratch_reuses_its_allocation_for_smaller_frames() {
        let mut scratch = LayerScratch::default();
        let first = scratch.prepare(240, 220).data().as_ptr() as usize;
        let second = scratch.prepare(180, 160).data().as_ptr() as usize;

        assert_eq!(first, second);
        assert_eq!(scratch.allocations, 1);
    }

    #[test]
    fn layer_scratch_growth_wastes_at_most_one_small_quantum_per_axis() {
        let mut scratch = LayerScratch::default();
        let pixmap = scratch.prepare(257, 129);

        assert_eq!(pixmap.width(), 288);
        assert_eq!(pixmap.height(), 160);
    }

    #[test]
    fn stipple_shadow_cache_stays_bounded_during_subpixel_motion() {
        geom::clear_stipple_shadow_cache();
        let mut pixmap = Pixmap::new(256, 256).expect("alloc");
        for frame in 0..2_000 {
            let fraction = frame as f32 / 2_000.0;
            geom::stipple_shadow(
                &mut pixmap,
                Vec2::new(96.0 + fraction, 96.0 + fraction * 0.73),
                24.0,
                6.5,
                1.0,
            );
        }

        assert!(
            geom::stipple_shadow_cache_size() <= 16,
            "subpixel motion must select from a bounded cache"
        );
    }

    #[test]
    fn opaque_one_x_frames_do_not_need_an_offscreen_composite() {
        assert!(!layer_needs_offscreen_composite(1.0, 1.0));
        assert!(layer_needs_offscreen_composite(1.0, 0.5));
        assert!(layer_needs_offscreen_composite(2.0, 1.0));
    }

    #[test]
    fn layer_scratch_allocation_count_plateaus_during_long_render_runs() {
        let mut scratch = LayerScratch::default();
        scratch.prepare(512, 512);

        for frame in 0..2_000_u32 {
            let width = 160 + frame % 353;
            let height = 150 + (frame * 7) % 363;
            scratch.prepare(width, height);
        }

        assert_eq!(
            scratch.allocations, 1,
            "steady-state rendering must reuse the maximum scratch allocation"
        );
    }

    #[test]
    fn side_neck_has_distinct_back_and_throat_contours() {
        let rig = Rig::update(Vec2::new(128.0, 190.0), 0.0, 1.0, 0.0);
        let mut pixmap = Pixmap::new(256, 256).expect("alloc");
        pixmap.fill(Color::TRANSPARENT);
        render_rig(&mut pixmap, &rig, Vec2::ZERO);
        let t = 0.48;
        let u = 1.0 - t;
        let spine = rig.neck_base * (u * u * u)
            + rig.neck_c1 * (3.0 * u * u * t)
            + rig.neck_c2 * (3.0 * u * t * t)
            + rig.neck_head * (t * t * t);
        let row = spine.y.round() as usize;
        let center = spine.x.round() as usize;
        let width = pixmap.width() as usize;
        let visible = |x: usize| {
            let pixel = &pixmap.data()[(row * width + x) * 4..][..4];
            pixel[3] > 128 && pixel[0] > 150 && pixel[1] > 150 && pixel[2] > 150
        };
        let mut left = center;
        while left > 0 && visible(left - 1) {
            left -= 1;
        }
        let mut right = center;
        while right + 1 < width && visible(right + 1) {
            right += 1;
        }
        let back = center - left;
        let throat = right - center;

        assert!(
            back.abs_diff(throat) >= 5,
            "Concept C needs visibly different contours (back={back}, throat={throat})"
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
    fn renders_visible_sleepy_particle() {
        use crate::mood::ZParticles;
        let mut z = ZParticles::new();
        z.add(Vec2::new(64.0, 64.0), 0.0);
        let mut pixmap = Pixmap::new(128, 128).expect("alloc");
        pixmap.fill(Color::TRANSPARENT);
        render_sleepies(&mut pixmap, &z, 0.0, Vec2::ZERO);
        let opaque = pixmap.data().chunks_exact(4).filter(|px| px[3] > 0).count();
        assert!(opaque > 5, "expected a visible Z particle");
    }

    #[test]
    fn renders_visible_autumn_leaves() {
        use crate::autumn::AutumnState;
        use crate::entity::GooseEntity;
        use crate::math::Rect;
        use crate::rng::SplitMix64;

        let mut autumn = AutumnState::new();
        let goose = GooseEntity::new();
        let mut rng = SplitMix64::seed(99);
        autumn.tick(
            0.0,
            true,
            Rect {
                min: Vec2::ZERO,
                max: Vec2::new(128.0, 128.0),
            },
            &goose,
            &mut rng,
        );
        autumn.tick(
            10.1,
            true,
            Rect {
                min: Vec2::ZERO,
                max: Vec2::new(128.0, 128.0),
            },
            &goose,
            &mut rng,
        );

        let mut pixmap = Pixmap::new(128, 128).expect("alloc");
        pixmap.fill(Color::TRANSPARENT);
        render_autumn_leaves(
            &mut pixmap,
            &autumn,
            10.1,
            Vec2::ZERO,
            Vec2::new(64.0, 64.0),
            AutumnRenderLayer::BelowGoose,
        );
        render_autumn_leaves(
            &mut pixmap,
            &autumn,
            10.1,
            Vec2::ZERO,
            Vec2::new(64.0, 64.0),
            AutumnRenderLayer::AboveGoose,
        );

        let leafish = pixmap
            .data()
            .chunks_exact(4)
            .filter(|px| px[3] > 80 && px[0] > px[2] && px[1] > 40)
            .count();
        assert!(leafish > 30, "expected visible Autumn leaves");
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
