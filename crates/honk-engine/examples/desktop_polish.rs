//! Actual engine/renderer frames for the desktop-polish review, with no native desktop effects.
use honk_engine::autumn::AutumnState;
use honk_engine::entity::GooseEntity;
use honk_engine::interaction::Pointer;
use honk_engine::math::{Rect, Vec2};
use honk_engine::render::{
    render_autumn_leaves, render_hearts, render_pose_with_palette, AutumnRenderLayer,
};
use honk_engine::rng::SplitMix64;
use honk_engine::time::DT;
use honk_engine::tiny_skia::{Color, Pixmap, PixmapPaint, Transform};
use honk_engine::world::World;

fn main() {
    let directory = std::env::args().nth(1).expect("output directory");
    std::fs::create_dir_all(&directory).unwrap();
    let bounds = Rect::new(Vec2::ZERO, Vec2::new(800.0, 600.0));
    let mut autumn = AutumnState::new();
    let mut goose = GooseEntity::new();
    let mut rng = SplitMix64::seed(4);
    autumn.tick(0.0, true, bounds, &goose, &mut rng);
    autumn.tick(10.1, true, bounds, &goose, &mut rng);
    let center = autumn.piles()[0].position;
    let mut sheet = Pixmap::new(960, 640).unwrap();
    sheet.fill(Color::from_rgba8(37, 42, 47, 255));
    let still = autumn.clone();
    for (cell, time) in [(0, 11.2), (1, 38.1), (2, 40.1)] {
        draw(&still, time, center, cell, &directory, &mut sheet);
    }
    goose.position = center;
    goose.velocity = Vec2::new(goose.parameters.charge_speed, 0.0);
    autumn.tick(11.2, true, bounds, &goose, &mut rng);
    for frame in 1..=108 {
        let time = 11.2 + frame as f64 * DT as f64;
        autumn.tick(time, true, bounds, &goose, &mut rng);
        if let Some(cell) = [(18, 3), (54, 4), (108, 5)]
            .iter()
            .find_map(|&(at, cell)| (at == frame).then_some(cell))
        {
            draw(&autumn, time, center, cell, &directory, &mut sheet);
        }
    }
    sheet
        .save_png(format!("{directory}/leaves-contact-sheet.png"))
        .unwrap();
    affection(&directory);
}

fn affection(directory: &str) {
    let mut world = World::new(Rect::new(Vec2::ZERO, Vec2::new(1100.0, 700.0)), 42);
    for _ in 0..6000 {
        world.tick();
        if world.current_task() == "wander" {
            break;
        }
    }
    assert_eq!(world.current_task(), "wander");
    for frame in 0..400 {
        world.set_pointer(Pointer {
            pos: world.rig().body_center + Vec2::new(if frame % 2 == 0 { 6.0 } else { -6.0 }, 0.0),
            present: true,
            left_down: false,
        });
        world.tick();
        if world.current_task() == "affection_follow" {
            break;
        }
    }
    assert_eq!(world.current_task(), "affection_follow");
    let origin = world.goose.position - Vec2::new(120.0, 155.0);
    let cursor = world
        .layout()
        .clamp_point(world.goose.position + Vec2::new(220.0, 0.0));
    let mut sheet = Pixmap::new(1320, 260).unwrap();
    sheet.fill(Color::from_rgba8(37, 42, 47, 255));
    for frame in 0..=300 {
        if let Some(cell) = [0, 100, 300].iter().position(|&at| at == frame) {
            let mut image = Pixmap::new(440, 260).unwrap();
            render_pose_with_palette(&mut image, world.pose(), origin, world.render_palette());
            render_hearts(&mut image, world.hearts(), world.now(), origin);
            let at = cursor - origin;
            let mut arrow = honk_engine::tiny_skia::PathBuilder::new();
            arrow.move_to(at.x, at.y);
            arrow.line_to(at.x, at.y + 18.0);
            arrow.line_to(at.x + 5.0, at.y + 13.0);
            arrow.line_to(at.x + 12.0, at.y + 13.0);
            arrow.close();
            let mut paint = honk_engine::tiny_skia::Paint::default();
            paint.set_color_rgba8(255, 255, 255, 255);
            image.fill_path(
                &arrow.finish().unwrap(),
                &paint,
                honk_engine::tiny_skia::FillRule::Winding,
                Transform::identity(),
                None,
            );
            image
                .save_png(format!("{directory}/affection-{cell}.png"))
                .unwrap();
            sheet.draw_pixmap(
                cell as i32 * 440,
                0,
                image.as_ref(),
                &PixmapPaint::default(),
                Transform::identity(),
                None,
            );
        }
        world.set_pointer(Pointer {
            pos: cursor,
            present: true,
            left_down: false,
        });
        world.tick();
        assert!(world.take_cursor_commands().is_empty());
    }
    sheet
        .save_png(format!("{directory}/affection-contact-sheet.png"))
        .unwrap();
}

fn draw(
    autumn: &AutumnState,
    now: f64,
    center: Vec2,
    cell: usize,
    directory: &str,
    sheet: &mut Pixmap,
) {
    let mut frame = Pixmap::new(320, 320).unwrap();
    render_autumn_leaves(
        &mut frame,
        autumn,
        now,
        center - Vec2::new(140.0, 210.0),
        Vec2::new(10_000.0, 10_000.0),
        AutumnRenderLayer::BelowGoose,
    );
    frame
        .save_png(format!("{directory}/leaf-{cell}.png"))
        .unwrap();
    sheet.draw_pixmap(
        (cell % 3 * 320) as i32,
        (cell / 3 * 320) as i32,
        frame.as_ref(),
        &PixmapPaint::default(),
        Transform::identity(),
        None,
    );
}
