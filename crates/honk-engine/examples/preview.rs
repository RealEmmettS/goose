//! Renderer V2 preview harness: renders a contact sheet of goose poses to a PNG so
//! the flat-illustration art can be tuned against `docs/art-reference/`.
//!
//! Usage: `cargo run -p honk-engine --example preview -- [out.png]`

use honk_engine::math::Vec2;
use honk_engine::render::{render_pose_with_palette, RenderPalette};
use honk_engine::rig::{GoosePose, RigAnim, RigInput};
use honk_engine::time::DT;
use honk_engine::tiny_skia::{Color, Pixmap};

const CELL: u32 = 170;
const COLS: u32 = 6;
const ROWS: u32 = 3;

struct Sim {
    anim: RigAnim,
    center: Vec2,
    now: f32,
}

impl Sim {
    fn new(dir_deg: f32) -> Self {
        Self {
            anim: RigAnim::new(Vec2::ZERO, dir_deg),
            center: Vec2::ZERO,
            now: 0.0,
        }
    }

    /// Advance `secs` of simulated time walking toward `dir_deg` at `speed`.
    fn run(&mut self, secs: f32, dir_deg: f32, speed: f32, neck: f32, step_time: f32) -> GoosePose {
        let velocity = Vec2::from_angle_degrees(dir_deg) * speed;
        let ticks = (secs / DT).round().max(1.0) as usize;
        let mut pose = None;
        for _ in 0..ticks {
            self.now += DT;
            self.center = self.center + velocity * DT;
            pose = Some(self.anim.update(&RigInput {
                center: self.center,
                direction_deg: dir_deg,
                neck_target: neck,
                speed,
                velocity,
                step_time,
                now: self.now,
                dt: DT,
            }));
        }
        pose.expect("at least one tick")
    }
}

fn main() {
    let out = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "preview.png".to_string());

    // `-- <out.png> big` renders a 3-pose zoomed strip instead of the contact sheet;
    // `-- <out.png> walk` renders 8 sequential walk frames with a ground ruler;
    // `-- <out-dir> frames` exports crisp site/marketing assets (walk spritesheet + poses).
    match std::env::args().nth(2).as_deref() {
        Some("big") => return big_strip(&out),
        Some("walk") => return walk_strip(&out),
        Some("frames") => return export_frames(&out),
        _ => {}
    }

    let mut sheet = Pixmap::new(CELL * COLS, CELL * ROWS).expect("sheet alloc");
    // Neutral checker-ish background so alpha edges are judgeable.
    sheet.fill(Color::from_rgba8(0x2e, 0x33, 0x38, 0xff));

    let pal = RenderPalette::default();
    let mut cells: Vec<(&str, GoosePose)> = Vec::new();

    // Row 1 — side view.
    cells.push(("rest R", Sim::new(0.0).run(1.0, 0.0, 0.0, 0.45, 0.2)));
    {
        let mut s = Sim::new(0.0);
        cells.push(("walk 1", s.run(0.62, 0.0, 80.0, 0.55, 0.2)));
        cells.push(("walk 2", s.run(0.1, 0.0, 80.0, 0.55, 0.2)));
    }
    cells.push(("reach", Sim::new(0.0).run(1.2, 0.0, 0.0, 1.0, 0.2)));
    {
        let mut s = Sim::new(0.0);
        s.run(0.5, 0.0, 0.0, 0.45, 0.2);
        s.anim.start_blink(s.now + DT * 2.0);
        cells.push(("blink", s.run(0.055, 0.0, 0.0, 0.45, 0.2)));
    }
    cells.push(("rest L", Sim::new(180.0).run(1.0, 180.0, 0.0, 0.45, 0.2)));

    // Row 2 — top-down view (walk steeply to engage it, fade completes).
    for (label, deg) in [
        ("top ↑", 270.0),
        ("top ↗", 315.0),
        ("top ↓", 90.0),
        ("top ←*", 250.0),
    ] {
        let mut s = Sim::new(0.0);
        s.run(0.2, 0.0, 80.0, 0.0, 0.2);
        cells.push((label, s.run(0.8, deg, 120.0, 0.55, 0.2)));
    }
    {
        let mut s = Sim::new(0.0);
        s.run(0.2, 0.0, 80.0, 0.0, 0.2);
        s.run(0.8, 90.0, 120.0, 0.0, 0.2);
        cells.push(("top reach", s.run(0.6, 90.0, 120.0, 1.0, 0.2)));
    }
    {
        // Mid-crossfade: switch view and stop after ~60ms.
        let mut s = Sim::new(0.0);
        s.run(0.5, 0.0, 80.0, 0.0, 0.2);
        cells.push(("x-fade", s.run(0.06, 90.0, 120.0, 0.55, 0.2)));
    }

    // Row 3 — motion flavors.
    {
        let mut s = Sim::new(0.0);
        cells.push(("charge", s.run(0.45, 0.0, 400.0, 0.65, 0.1)));
    }
    {
        let mut s = Sim::new(0.0);
        s.run(1.0, 0.0, 0.0, 0.45, 0.2);
        s.anim.flick_tail();
        let pose = s.run(0.05, 0.0, 0.0, 0.45, 0.2);
        cells.push(("tail", pose));
    }
    {
        let mut s = Sim::new(180.0);
        cells.push(("walk L", s.run(0.66, 180.0, 80.0, 0.55, 0.2)));
    }
    {
        let mut s = Sim::new(0.0);
        cells.push(("run", s.run(0.55, 0.0, 200.0, 0.6, 0.2)));
    }
    {
        // Breathing extremes at idle.
        let mut s = Sim::new(0.0);
        cells.push(("breath+", s.run(0.45, 0.0, 0.0, 0.45, 0.2)));
        let mut s2 = Sim::new(0.0);
        cells.push(("breath-", s2.run(1.36, 0.0, 0.0, 0.45, 0.2)));
    }

    for (i, (label, pose)) in cells.iter().enumerate() {
        let col = (i as u32) % COLS;
        let row = (i as u32) / COLS;
        if row >= ROWS {
            break;
        }
        // Center the pose's ground point in the lower-middle of the cell.
        let cell_anchor = Vec2::new(
            (col * CELL) as f32 + CELL as f32 * 0.5,
            (row * CELL) as f32 + CELL as f32 * 0.78,
        );
        let origin = pose.primary.ground - cell_anchor;
        render_pose_with_palette(&mut sheet, pose, origin, pal);
        let _ = label;
    }

    sheet.save_png(&out).expect("write preview png");
    println!("wrote {out} ({}x{})", sheet.width(), sheet.height());
}

/// Three zoomed poses (side rest, side reach, top-down), magnified 3x for detail review.
fn big_strip(out: &str) {
    use honk_engine::tiny_skia::{FilterQuality, PixmapPaint, Transform};
    const SMALL: u32 = 120;
    const ZOOM: u32 = 3;
    const BIG: u32 = SMALL * ZOOM;
    let mut sheet = Pixmap::new(BIG * 3, BIG).expect("sheet alloc");
    sheet.fill(Color::from_rgba8(0x2e, 0x33, 0x38, 0xff));
    let pal = RenderPalette::default();

    let poses = [
        Sim::new(0.0).run(1.0, 0.0, 0.0, 0.45, 0.2),
        Sim::new(0.0).run(1.2, 0.0, 0.0, 1.0, 0.2),
        {
            let mut s = Sim::new(0.0);
            s.run(0.2, 0.0, 80.0, 0.0, 0.2);
            s.run(0.8, 270.0, 120.0, 0.55, 0.2)
        },
    ];
    for (i, pose) in poses.iter().enumerate() {
        let mut cell = Pixmap::new(SMALL, SMALL).expect("cell alloc");
        cell.fill(Color::TRANSPARENT);
        let anchor = Vec2::new(SMALL as f32 * 0.5, SMALL as f32 * 0.75);
        let origin = pose.primary.ground - anchor;
        render_pose_with_palette(&mut cell, pose, origin, pal);
        sheet.draw_pixmap(
            0,
            0,
            cell.as_ref(),
            &PixmapPaint {
                quality: FilterQuality::Nearest,
                ..PixmapPaint::default()
            },
            Transform::from_scale(ZOOM as f32, ZOOM as f32)
                .post_translate((i as u32 * BIG) as f32, 0.0),
            None,
        );
    }
    sheet.save_png(out).expect("write big strip");
    println!("wrote {out} ({}x{})", sheet.width(), sheet.height());
}

/// Eight sequential walk frames, 50 ms apart, 2x zoom, with world-fixed ground ticks —
/// planted feet must stay aligned to the same tick across frames (no moonwalk).
fn walk_strip(out: &str) {
    use honk_engine::tiny_skia::{FilterQuality, PixmapPaint, Transform};
    const SMALL: u32 = 150;
    const ZOOM: u32 = 2;
    const BIG: u32 = SMALL * ZOOM;
    const FRAMES: u32 = 8;
    let mut sheet = Pixmap::new(BIG * FRAMES, BIG).expect("sheet alloc");
    sheet.fill(Color::from_rgba8(0x2e, 0x33, 0x38, 0xff));
    let pal = RenderPalette::default();

    let mut s = Sim::new(0.0);
    s.run(0.4, 0.0, 80.0, 0.55, 0.2); // settle into the cycle
    for i in 0..FRAMES {
        let pose = s.run(0.05, 0.0, 80.0, 0.55, 0.2);
        let mut cell = Pixmap::new(SMALL, SMALL).expect("cell alloc");
        cell.fill(Color::TRANSPARENT);
        let anchor = Vec2::new(SMALL as f32 * 0.5, SMALL as f32 * 0.7);
        let origin = pose.primary.ground - anchor;
        // World-fixed ground ticks every 10 px so foot slippage is visible.
        {
            use honk_engine::tiny_skia::Paint;
            let mut tick = Paint::default();
            tick.set_color_rgba8(0xff, 0xd7, 0x00, 200);
            let gy = pose.primary.ground.y;
            let mut wx = (origin.x / 10.0).floor() * 10.0;
            while wx < origin.x + SMALL as f32 {
                let px = wx - origin.x;
                let rect =
                    honk_engine::tiny_skia::Rect::from_xywh(px, gy - origin.y + 4.0, 1.5, 6.0);
                if let Some(r) = rect {
                    cell.fill_rect(r, &tick, Transform::identity(), None);
                }
                wx += 10.0;
            }
        }
        render_pose_with_palette(&mut cell, &pose, origin, pal);
        sheet.draw_pixmap(
            0,
            0,
            cell.as_ref(),
            &PixmapPaint {
                quality: FilterQuality::Nearest,
                ..PixmapPaint::default()
            },
            Transform::from_scale(ZOOM as f32, ZOOM as f32).post_translate((i * BIG) as f32, 0.0),
            None,
        );
    }
    sheet.save_png(out).expect("write walk strip");
    println!("wrote {out} ({}x{})", sheet.width(), sheet.height());
}

/// Export crisp, registration-consistent renderer output for the website/marketing:
/// a 12-frame walk-cycle spritesheet (one stride, loopable), the individual frames,
/// and large rest / reach / left / top-down poses — all true vector renders.
fn export_frames(out_dir: &str) {
    use honk_engine::render::render_rig_scaled;
    std::fs::create_dir_all(out_dir).expect("create out dir");

    // World-space window around the goose: the ground point sits at (W/2, H*0.82).
    const W: f32 = 130.0;
    const H: f32 = 110.0;
    const SCALE: f32 = 3.0;
    let anchor = |ground: Vec2| ground - Vec2::new(W * 0.5, H * 0.82);
    let pal = RenderPalette::default();

    // One full stride is two step intervals (~0.4 s at walk tier): 12 frames sample a
    // loopable cycle after the gait has settled.
    let mut s = Sim::new(0.0);
    s.run(1.0, 0.0, 80.0, 0.55, 0.2);
    let mut frames = Vec::new();
    for _ in 0..12 {
        let pose = s.run(0.4 / 12.0, 0.0, 80.0, 0.55, 0.2);
        let rig = pose.primary;
        let px = render_rig_scaled(&rig, anchor(rig.ground), W, H, SCALE, pal).expect("frame");
        frames.push(px);
    }
    let fw = frames[0].width();
    let fh = frames[0].height();
    let mut sheet = Pixmap::new(fw * frames.len() as u32, fh).expect("sheet alloc");
    sheet.fill(Color::TRANSPARENT);
    for (i, f) in frames.iter().enumerate() {
        use honk_engine::tiny_skia::{PixmapPaint, Transform};
        sheet.draw_pixmap(
            (i as u32 * fw) as i32,
            0,
            f.as_ref(),
            &PixmapPaint::default(),
            Transform::identity(),
            None,
        );
        f.save_png(format!("{out_dir}/walk-{i:02}.png"))
            .expect("frame png");
    }
    sheet
        .save_png(format!("{out_dir}/walk-sheet.png"))
        .expect("sheet png");
    println!(
        "walk-sheet.png: {} frames of {}x{} (cycle 0.4s)",
        frames.len(),
        fw,
        fh
    );

    // Large poses at 6x (~450 px tall), same registration window.
    for (name, pose) in [
        ("pose-rest", Sim::new(0.0).run(1.0, 0.0, 0.0, 0.45, 0.2)),
        ("pose-reach", Sim::new(0.0).run(1.2, 0.0, 0.0, 1.0, 0.2)),
        (
            "pose-rest-left",
            Sim::new(180.0).run(1.0, 180.0, 0.0, 0.45, 0.2),
        ),
        ("pose-top", {
            let mut s = Sim::new(0.0);
            s.run(0.2, 0.0, 80.0, 0.45, 0.2);
            s.run(0.8, 270.0, 120.0, 0.55, 0.2)
        }),
    ] {
        let rig = pose.primary;
        let px = render_rig_scaled(&rig, anchor(rig.ground), W, H, 6.0, pal).expect("pose");
        px.save_png(format!("{out_dir}/{name}.png"))
            .expect("pose png");
        println!("{name}.png: {}x{}", px.width(), px.height());
    }
}
