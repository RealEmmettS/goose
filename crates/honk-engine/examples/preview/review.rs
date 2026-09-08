//! Actual engine frames; HTML only displays the exported PNGs.
use honk_engine::{
    math::Vec2,
    render::{render_rig_scaled, render_rig_svg, RenderPalette},
    rig::{Rig, RigAnim, RigInput},
    time::DT,
    tiny_skia::{Pixmap, PixmapPaint, Transform},
    CollectWindowCapabilities, CollectWindowCommand, CollectWindowId, CollectWindowKind,
    CollectWindowOptions, CollectWindowSnapshot, Rect, World, WorldOptions,
};
use std::{fs, path::Path};
const CELL: u32 = 128;
const COLS: u32 = 12;
const COUNT: u32 = 120;
const ANCHOR: Vec2 = Vec2 { x: 60.0, y: 93.0 };
const SEQUENCES: [&str; 15] = [
    "walk",
    "walk-front",
    "walk-away",
    "run-stop",
    "stop-front",
    "stop-away",
    "stop-charge",
    "turn",
    "reversal",
    "delivery-reversal",
    "delivery-short-return",
    "pet",
    "honk",
    "anticipation",
    "reduced-motion",
];

pub fn export(out: &str) {
    let out = Path::new(out);
    fs::create_dir_all(out).unwrap();
    fs::create_dir_all(out.join("svg")).unwrap();
    let mut manifest = String::from("{\"developmentPreview\":true,\"fps\":30,\"cell\":128,\"scale\":2,\"columns\":12,\"count\":120,\"sequences\":{");
    for (index, name) in SEQUENCES.iter().enumerate() {
        let delivery = match *name {
            "delivery-reversal" => Some(delivery_frames(320.0, 40)),
            "delivery-short-return" => Some(delivery_frames(312.0, 600)),
            _ => None,
        };
        let mut sheet = Pixmap::new(CELL * 2 * COLS, CELL * 2 * (COUNT / COLS)).unwrap();
        let mut anim = RigAnim::new(Vec2::ZERO, 0.0);
        let mut center = Vec2::ZERO;
        let mut trace = String::new();
        let mut heading = 0.0;
        for frame in 0..COUNT {
            let t = frame as f32 / 30.0;
            let speed = match *name {
                "walk" => 80.0,
                "walk-front" => {
                    heading = 90.0;
                    120.0
                }
                "walk-away" => {
                    heading = 270.0;
                    120.0
                }
                "run-stop" => {
                    if t < 0.5 {
                        t * 400.0
                    } else if t < 2.5 {
                        200.0
                    } else {
                        (3.0 - t).max(0.0) * 400.0
                    }
                }
                "stop-front" | "stop-away" | "stop-charge" => {
                    heading = if *name == "stop-away" { 270.0 } else { 90.0 };
                    if t < 2.0 {
                        if *name == "stop-charge" {
                            400.0
                        } else {
                            120.0
                        }
                    } else {
                        0.0
                    }
                }
                "turn" => {
                    heading = t * 90.0;
                    55.0
                }
                "reversal" => {
                    heading = if (t as u32).is_multiple_of(2) {
                        0.0
                    } else {
                        180.0
                    };
                    85.0
                }
                _ => 0.0,
            };
            if frame == 15 && *name == "pet" {
                anim.pet();
            }
            if [15, 70].contains(&frame) && ["honk", "reduced-motion"].contains(name) {
                anim.flick_tail();
            }
            if [40, 85].contains(&frame) {
                anim.start_blink(f64::from(t));
            }
            anim.anticipate(*name == "anticipation" && (15..60).contains(&frame));
            anim.set_expression_options(true, *name == "reduced-motion");
            let velocity = Vec2::from_angle_degrees(heading) * speed;
            let mut rig = Rig::default();
            if let Some(frames) = &delivery {
                rig = frames[frame as usize];
                center = rig.ground;
            } else {
                for tick in 0..4 {
                    center = center + velocity * DT;
                    rig = anim
                        .update(&RigInput {
                            center,
                            direction_deg: heading,
                            neck_target: 0.45,
                            speed,
                            velocity,
                            step_time: 0.2,
                            now: f64::from(t) + f64::from(tick) * f64::from(DT),
                            dt: DT,
                        })
                        .primary;
                    anim.feet.drain_plants(|_| {});
                }
            }
            let cell = render_rig_scaled(
                &rig,
                rig.ground - ANCHOR,
                CELL as f32,
                CELL as f32,
                2.0,
                RenderPalette::default(),
            )
            .unwrap();
            let vector = render_rig_svg(
                &rig,
                rig.ground - ANCHOR,
                CELL as f32,
                CELL as f32,
                2.0,
                RenderPalette::default(),
            )
            .unwrap();
            fs::write(out.join(format!("svg/{name}-{frame:03}.svg")), &vector).unwrap();
            sheet.draw_pixmap(
                ((frame % COLS) * CELL * 2) as i32,
                ((frame / COLS) * CELL * 2) as i32,
                cell.as_ref(),
                &PixmapPaint::default(),
                Transform::identity(),
                None,
            );
            if frame > 0 {
                trace.push(',');
            }
            trace.push_str(&format!("[{:.2},{:.2}]", center.x, center.y));
            if frame == 30 {
                cell.save_png(out.join(format!("{name}-pose.png"))).unwrap();
                fs::write(out.join(format!("{name}-pose.svg")), &vector).unwrap();
            }
        }
        sheet.save_png(out.join(format!("{name}.png"))).unwrap();
        if index > 0 {
            manifest.push(',');
        }
        manifest.push_str(&format!(
            "\"{name}\":{{\"image\":\"{name}.png\",\"ground\":[{trace}]}}"
        ));
    }
    manifest.push_str("}}");
    fs::write(out.join("manifest.json"), &manifest).unwrap();
    let mut headings = String::new();
    for degrees in (0..360).step_by(30) {
        let rig = Rig::update(Vec2::ZERO, degrees as f32, 0.45, 0.0);
        render_rig_scaled(
            &rig,
            rig.ground - ANCHOR,
            CELL as f32,
            CELL as f32,
            4.0,
            RenderPalette::default(),
        )
        .unwrap()
        .save_png(out.join(format!("heading-{degrees:03}.png")))
        .unwrap();
        fs::write(
            out.join(format!("heading-{degrees:03}.svg")),
            render_rig_svg(
                &rig,
                rig.ground - ANCHOR,
                CELL as f32,
                CELL as f32,
                4.0,
                RenderPalette::default(),
            )
            .unwrap(),
        )
        .unwrap();
        headings.push_str(&format!("<figure><img src='heading-{degrees:03}.png' width='128' height='128'><figcaption>{degrees}&deg;</figcaption></figure>"));
    }
    fs::write(
        out.join("index.html"),
        include_str!("review.html")
            .replace("MANIFEST_DATA", &manifest)
            .replace("HEADING_IMAGES", &headings),
    )
    .unwrap();
    fs::write(out.join("README.md"), "# Actual renderer development preview\n\nOpen index.html to play, pause, scrub, and inspect at 100/150/200 percent on light/dark backgrounds. PNG sheets: 120 frames, 30fps, 12 columns, 256px cells displayed at 128 CSS pixels. Heading PNGs: 512px cells displayed at 128 CSS pixels. Ground anchor: 60,93 world units in every cell. manifest.json records travel for the contact ruler.\n\nEvery named pose and heading also has an editable SVG. Individual motion frames are svg/<sequence>-000.svg through -119.svg. These are the same paths, colors, opacity, strokes and drawing order emitted by the production projected renderer; they contain no embedded raster, fonts, scripts or external resources. The shared rig and geometry remain authoritative. These are development exports, not public release or native desktop acceptance.\n").unwrap();
    println!("wrote actual renderer motion review to {}", out.display());
}

/// Native-height delivery witnesses use the actual task, locomotion and rig.
/// A bounded in-memory prop supplies only the same native geometry/command feedback
/// as the World regression. Frames span two seconds before and after note typing.
fn delivery_frames(initial_y: f32, delay: usize) -> Vec<Rig> {
    let mut options = WorldOptions::default();
    options.timing.first_wander_time = 600.0;
    options.mood.dynamic_moods = true;
    options.schedule.seasonal = false;
    options.collect_window = CollectWindowOptions::with_backend_support(
        CollectWindowCapabilities {
            spawn_note: true,
            spawn_image: true,
            move_window: true,
            set_passthrough: true,
            synthesize_text: true,
        },
        12,
        15,
    );
    let mut world = World::with_options(
        Rect::new(Vec2::ZERO, Vec2::new(1280.0, 900.0)),
        1_788_900_900_000_000_000,
        options,
    );
    for _ in 0..delay {
        world.tick();
    }
    world.force_collect_window(CollectWindowKind::Note);
    let mut native = None;
    let mut typed_at = None;
    let mut frames = Vec::new();
    let initial = Vec2::new(430.0, initial_y);
    let size = Vec2::new(420.0, 288.0);
    for tick in 0..1800 {
        world.tick();
        for command in world.take_collect_window_commands() {
            match command {
                CollectWindowCommand::Spawn { request, payload } => {
                    native = Some(CollectWindowSnapshot {
                        id: CollectWindowId(1),
                        request,
                        kind: payload.kind(),
                        rect: Rect::new(initial, initial + size),
                        alive: true,
                        close_origin: None,
                    });
                }
                CollectWindowCommand::Move { top_left, .. } => {
                    native.as_mut().unwrap().rect = Rect::new(top_left, top_left + size);
                }
                CollectWindowCommand::TypeNote { .. } => typed_at = Some(tick),
                _ => {}
            }
        }
        if tick > 60 {
            world.set_collect_window_snapshot(native);
        }
        frames.push(world.goose.rig);
        if typed_at.is_some_and(|typed| tick >= typed + COUNT as usize * 2) {
            break;
        }
    }
    let start = typed_at.expect("native-height delivery did not complete") - COUNT as usize * 2;
    (0..COUNT as usize)
        .map(|frame| frames[start + frame * 4])
        .collect()
}
