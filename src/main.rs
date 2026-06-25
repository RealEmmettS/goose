//! honk300 — the binary entry point.
//!
//! Round 1's M1+M2 slice: open the Windows overlay and run the fixed-timestep loop so a
//! procedurally-rendered goose roams the desktop. Three clocks (plan §7.2): the sim ticks
//! at a fixed 120 Hz via [`Accumulator`], and we present at ~60 Hz, only the goose's
//! bounding box. The CLI grammar, IPC, config TUI, and the macOS/Linux backends arrive in
//! later rounds.

#[cfg(windows)]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    use honk_engine::render::render_rig;
    use honk_engine::tiny_skia::{Color, Pixmap};
    use honk_engine::{Accumulator, Clock, Vec2, World};
    use honk_platform_windows::Overlay;

    // Fixed window size so the layered surface (and its DIB) is allocated once; the goose
    // is centred in it and the whole window is repositioned each frame.
    const FRAME: u32 = 200;

    let mut overlay = Overlay::new()?;
    let bounds = Overlay::virtual_bounds();
    let mut world = World::new(bounds, seed_from_clock());

    let mut canvas = Pixmap::new(FRAME, FRAME).ok_or("could not allocate the goose canvas")?;
    let mut accumulator = Accumulator::new();
    let clock = Clock::start();
    let mut last = clock.elapsed_secs();
    let mut last_present = f32::NEG_INFINITY;
    const PRESENT_INTERVAL: f32 = 1.0 / 60.0;

    println!("honk300: a goose is loose on your desktop. Press Ctrl+C here to send it home.");

    loop {
        if !overlay.pump() {
            break;
        }

        let now = clock.elapsed_secs();
        let dt = now - last;
        last = now;
        for _ in 0..accumulator.pump(dt) {
            world.tick();
        }

        if now - last_present >= PRESENT_INTERVAL {
            last_present = now;
            let rig = world.rig();
            // Centre the goose's bounding box in the fixed FRAME canvas.
            let bb = rig.bounding_box();
            let bb_center = (bb.min + bb.max) * 0.5;
            let origin = bb_center - Vec2::new(FRAME as f32 * 0.5, FRAME as f32 * 0.5);

            canvas.fill(Color::TRANSPARENT);
            render_rig(&mut canvas, rig, origin);
            overlay.present(&canvas, origin.x.floor() as i32, origin.y.floor() as i32)?;
        }

        // Yield so the loop doesn't busy-spin; the accumulator keeps the sim at 120 Hz.
        std::thread::sleep(std::time::Duration::from_millis(2));
    }

    Ok(())
}

/// A non-deterministic seed for the roam driver, derived from the wall clock.
#[cfg(windows)]
fn seed_from_clock() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos() as u64)
        .unwrap_or(0x9E37_79B9_7F4A_7C15)
}

#[cfg(not(windows))]
fn main() {
    eprintln!(
        "honk300: the desktop overlay is Windows-only for now \
         (the macOS and Linux backends land in milestones M16/M17)."
    );
}
