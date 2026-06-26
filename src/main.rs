//! honk300 — the binary entry point.
//!
//! Windows desktop runtime for the current honk300 milestone slice: overlay, fixed-step
//! simulation, sounds, hit-testing, and M7 cursor mischief. The CLI grammar, IPC, config
//! TUI, and the macOS/Linux backends arrive in later rounds.

#[cfg(windows)]
mod audio;

#[cfg(windows)]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    use honk_engine::render::{render_footmarks, render_hearts, render_rig};
    use honk_engine::tiny_skia::{Color, Pixmap};
    use honk_engine::{
        Accumulator, Clock, CursorCommand, MouseStealOptions, Pointer, Vec2, World, WorldOptions,
    };
    use honk_platform_windows::{pointer_state, warp_cursor, Overlay};

    // `--no-sound` / `--silent` runs the goose mute (the original's SilenceSounds).
    let no_sound = std::env::args().any(|a| a == "--no-sound" || a == "--silent");
    let no_mouse_steal = std::env::args().any(|a| a == "--no-mouse-steal");
    let mut audio = if no_sound { None } else { audio::Audio::new() };

    let mut overlay = Overlay::new()?;
    // Fullscreen primary-monitor overlay so world-space props (footmarks, later
    // meme/notepad windows) render where they belong. World origin maps to the
    // monitor's top-left, so the canvas is the monitor and `origin` is its min corner.
    let bounds = Overlay::primary_bounds();
    let origin = bounds.min;
    let width = bounds.width().ceil().max(1.0) as u32;
    let height = bounds.height().ceil().max(1.0) as u32;

    let mut world = World::with_options(
        bounds,
        seed_from_clock(),
        WorldOptions {
            mouse_steal: MouseStealOptions {
                enabled: !no_mouse_steal,
                warp_supported: !no_mouse_steal,
                ..MouseStealOptions::default()
            },
        },
    );
    let mut canvas = Pixmap::new(width, height).ok_or("could not allocate the overlay canvas")?;
    let mut accumulator = Accumulator::new();
    let clock = Clock::start();
    let mut last = clock.elapsed_secs();
    let mut last_present = f32::NEG_INFINITY;
    // Fullscreen present is heavier than a tiny window, so cap it a little lower.
    const PRESENT_INTERVAL: f32 = 1.0 / 40.0;
    let mut warned_cursor_warp = false;

    println!("honk300: a goose is loose on your desktop. Press Ctrl+C here to send it home.");

    loop {
        if !overlay.pump() {
            break;
        }

        let now = clock.elapsed_secs();
        let dt = now - last;
        last = now;

        // Feed the cursor before ticking: tasks such as nab_mouse chase the newest pointer
        // sample, then emit platform-free cursor commands for the backend to apply below.
        let (mx, my, left_down) = pointer_state();
        world.set_pointer(Pointer {
            pos: Vec2::new(mx, my),
            present: true,
            left_down,
        });

        for _ in 0..accumulator.pump(dt) {
            world.tick();
        }

        // Apply at most the newest warp request. If the OS/session rejects cursor warping,
        // degrade honestly and stop registering further mouse-steal behavior.
        if let Some(CursorCommand::WarpTo(pos)) = world.take_cursor_commands().last().copied() {
            if let Err(err) = warp_cursor(pos) {
                world.set_cursor_warp_supported(false);
                if !warned_cursor_warp {
                    warned_cursor_warp = true;
                    eprintln!("honk300: cursor warp unavailable; disabling mouse stealing ({err})");
                }
            }
        }

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
