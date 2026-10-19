//! Repeatable CPU renderer workloads; deliberately excludes native compositor/IPC cost.
//! cargo run --release -p honk-engine --example render_bench -- [frames]
use honk_engine::{
    autumn::{AutumnState, LEAF_RENDER_RADIUS},
    entity::GooseEntity,
    math::Rect,
    rng::SplitMix64,
};
use honk_engine::{footmarks::FootMarks, math::Vec2, render::*, rig::*, time::DT};
use std::time::Instant;

fn main() {
    let frames: usize = std::env::args()
        .nth(1)
        .and_then(|s| s.parse().ok())
        .filter(|&frames| frames > 0)
        .unwrap_or(6_000);
    println!("workload,frames,total_ms,p50_us,p95_us,p99_us,canvas_allocations,retained_bytes");
    for (name, speed, outputs, muddy) in [
        ("idle", 0.0, 1, false),
        ("walk", 80.0, 1, false),
        ("run", 200.0, 1, false),
        ("mud", 80.0, 1, true),
        ("two_outputs", 80.0, 2, false),
    ] {
        let mut anim = RigAnim::new(Vec2::ZERO, 0.0);
        let mut center = Vec2::ZERO;
        let mut marks = FootMarks::new();
        let mut canvases = [DamageCanvas::default(), DamageCanvas::default()];
        let mut samples = Vec::with_capacity(frames);
        for frame in 0..frames + 120 {
            let start = Instant::now();
            let now = frame as f64 * DT as f64;
            let heading = (frame as f32 * 0.15).rem_euclid(360.0);
            let velocity = Vec2::from_angle_degrees(heading) * speed;
            center = center + velocity * DT;
            let pose = anim.update(&RigInput {
                center,
                direction_deg: heading,
                neck_target: 0.45,
                speed,
                velocity,
                step_time: 0.2,
                now,
                dt: DT,
            });
            anim.feet.drain_plants(|p| {
                if muddy {
                    marks.add(p, now);
                }
            });
            let bounds = pose.bounding_box();
            let extra = if muddy { 240.0 } else { 0.0 };
            let origin = bounds.min - Vec2::new(extra, extra);
            for canvas in &mut canvases[..outputs] {
                let surface = canvas
                    .prepare(
                        (bounds.width() + extra * 2.0).ceil() as u32,
                        (bounds.height() + extra * 2.0).ceil() as u32,
                    )
                    .unwrap();
                render_footmarks(surface, &marks, now, origin);
                render_pose_with_palette_at_scale(
                    surface,
                    &pose,
                    origin,
                    RenderPalette::default(),
                    1.0,
                );
                std::hint::black_box(surface.data());
            }
            if frame >= 120 {
                samples.push(start.elapsed().as_secs_f64() * 1e6);
            }
        }
        let total: f64 = samples.iter().sum();
        samples.sort_by(f64::total_cmp);
        let percentile = |p: f64| samples[((frames.saturating_sub(1)) as f64 * p) as usize];
        println!(
            "{name},{frames},{:.2},{:.2},{:.2},{:.2},{},{}",
            total / 1e3,
            percentile(0.5),
            percentile(0.95),
            percentile(0.99),
            canvases
                .iter()
                .map(DamageCanvas::allocations)
                .sum::<usize>(),
            canvases
                .iter()
                .map(DamageCanvas::retained_bytes)
                .sum::<usize>()
        );
    }
    for (name, distant, scatter) in [
        ("leaves", false, false),
        ("leaves_wide", true, false),
        ("leaves_scatter", true, true),
    ] {
        benchmark_leaves(name, frames, distant, scatter);
    }
}

fn benchmark_leaves(name: &str, frames: usize, distant: bool, scatter: bool) {
    // Build a repeatable set of production leaf states outside the timed renderer. The wide
    // scene deliberately separates the pile and goose; the earlier probe omitted this cost.
    let bounds = Rect::new(Vec2::new(80.0, 80.0), Vec2::new(500.0, 500.0));
    let mut autumn = AutumnState::new();
    let mut rng = SplitMix64::seed(300);
    let mut goose = GooseEntity {
        position: Vec2::new(20.0, 20.0),
        ..Default::default()
    };
    autumn.tick(0.0, true, bounds, &goose, &mut rng);
    autumn.tick(10.0, true, bounds, &goose, &mut rng);
    assert!(!autumn.piles().is_empty());
    if scatter {
        goose.position = autumn.piles()[0].position;
        goose.velocity = Vec2::new(400.0, 0.0);
    }
    let mut scenes = Vec::new();
    for tick in 0..240 {
        let now = 11.0 + tick as f64 * DT as f64;
        autumn.tick(now, true, bounds, &goose, &mut rng);
        scenes.push((now, autumn.clone()));
        goose.position = Vec2::new(-1000.0, -1000.0);
    }
    let center = if distant {
        Vec2::new(2400.0, 1400.0)
    } else {
        Vec2::new(400.0, 400.0)
    };
    let mut anim = RigAnim::new(center, 0.0);
    let mut canvas = DamageCanvas::default();
    let mut samples = Vec::with_capacity(frames);
    for frame in 0..frames + 120 {
        let (now, scene) = &scenes[frame % scenes.len()];
        let start = Instant::now();
        let heading = (frame as f32 * 0.15).rem_euclid(360.0);
        let pose = anim.update(&RigInput {
            center,
            direction_deg: heading,
            neck_target: 0.45,
            speed: 80.0,
            velocity: Vec2::from_angle_degrees(heading) * 80.0,
            step_time: 0.2,
            now: frame as f64 * DT as f64,
            dt: DT,
        });
        let mut bounds = pose.bounding_box().grow(3.0);
        for pile in scene.piles() {
            bounds = bounds.union(
                Rect::new(pile.position, pile.position).grow(pile.radius + LEAF_RENDER_RADIUS),
            );
            for leaf in &pile.leaves {
                let point = pile.position + leaf.screen_offset();
                bounds = bounds.union(Rect::new(point, point).grow(LEAF_RENDER_RADIUS));
            }
        }
        let bounds = bounds.pixel_aligned();
        let surface = canvas
            .prepare(bounds.width() as u32, bounds.height() as u32)
            .unwrap();
        render_autumn_leaves(
            surface,
            scene,
            *now,
            bounds.min,
            center,
            AutumnRenderLayer::BelowGoose,
        );
        render_pose_with_palette(surface, &pose, bounds.min, RenderPalette::default());
        render_autumn_leaves(
            surface,
            scene,
            *now,
            bounds.min,
            center,
            AutumnRenderLayer::AboveGoose,
        );
        std::hint::black_box(surface.data());
        if frame >= 120 {
            samples.push(start.elapsed().as_secs_f64() * 1e6);
        }
    }
    let total: f64 = samples.iter().sum();
    samples.sort_by(f64::total_cmp);
    let percentile = |p: f64| samples[((frames.saturating_sub(1)) as f64 * p) as usize];
    println!(
        "{name},{frames},{:.2},{:.2},{:.2},{:.2},{},{}",
        total / 1e3,
        percentile(0.5),
        percentile(0.95),
        percentile(0.99),
        canvas.allocations(),
        canvas.retained_bytes()
    );
}
