//! Repeatable CPU renderer workloads; deliberately excludes native compositor/IPC cost.
//! cargo run --release -p honk-engine --example render_bench -- [frames]
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
}
