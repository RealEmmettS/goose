//! honk300 — the binary entry point.
//!
//! Round 1's M1+M2 slice: open the Windows overlay and run the fixed-timestep loop so a
//! procedurally-rendered goose roams the desktop. Three clocks (plan §7.2): the sim ticks
//! at a fixed 120 Hz via [`Accumulator`], and we present at ~60 Hz, only the goose's
//! bounding box. The CLI grammar, IPC, config TUI, and the macOS/Linux backends arrive in
//! later rounds.

#[cfg(windows)]
mod audio;

#[cfg(windows)]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    use honk_engine::render::{render_footmarks, render_hearts, render_rig};
    use honk_engine::tiny_skia::{Color, Pixmap};
    use honk_engine::{Accumulator, Clock, Pointer, Vec2, World};
    use honk_platform_windows::{pointer_state, Overlay};

    // `--no-sound` / `--silent` runs the goose mute (the original's SilenceSounds).
    let no_sound = std::env::args().any(|a| a == "--no-sound" || a == "--silent");
    let mut audio = if no_sound { None } else { audio::Audio::new() };

    let mut overlay = Overlay::new()?;
    // Fullscreen primary-monitor overlay so world-space props (footmarks, later
    // meme/notepad windows) render where they belong. World origin maps to the
    // monitor's top-left, so the canvas is the monitor and `origin` is its min corner.
    let bounds = Overlay::primary_bounds();
    let origin = bounds.min;
    let width = bounds.width().ceil().max(1.0) as u32;
    let height = bounds.height().ceil().max(1.0) as u32;

    let mut world = World::new(bounds, seed_from_clock());
    let mut canvas = Pixmap::new(width, height).ok_or("could not allocate the overlay canvas")?;
    let mut accumulator = Accumulator::new();
    let clock = Clock::start();
    let mut last = clock.elapsed_secs();
    let mut last_present = f32::NEG_INFINITY;
    // Fullscreen present is heavier than a tiny window, so cap it a little lower.
    const PRESENT_INTERVAL: f32 = 1.0 / 40.0;

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

        // Feed the cursor for hit-testing: hover-sweeps pat the goose (hearts + calm),
        // a left-click on it sends it hyper (plan §5.9 / §6). The overlay origin is the
        // monitor's top-left, so desktop cursor coordinates are world coordinates.
        let (mx, my, left_down) = pointer_state();
        world.set_pointer(Pointer {
            pos: Vec2::new(mx, my),
            present: true,
            left_down,
        });

        // Drain and play any sounds the sim requested this frame (silently dropped if muted).
        let sounds = world.take_sounds();
        if let Some(a) = audio.as_mut() {
            for s in sounds {
                a.play(s);
            }
        }

        if now - last_present >= PRESENT_INTERVAL {
            last_present = now;
            canvas.fill(Color::TRANSPARENT);
            render_footmarks(&mut canvas, &world.goose.foot_marks, world.now(), origin);
            render_rig(&mut canvas, world.rig(), origin);
            render_hearts(&mut canvas, world.hearts(), world.now(), origin);
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
