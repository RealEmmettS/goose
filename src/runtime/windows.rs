use crate::assets;
use crate::audio;
use crate::runtime::RuntimeOptions;
use honk_config::{BackendState, Config, EffectiveOptions};
use honk_control::{CommandServer, ControlCommand};
use honk_engine::render::{render_footmarks, render_hearts, render_rig};
use honk_engine::tiny_skia::{Color, Pixmap};
use honk_engine::{
    Accumulator, Clock, CollectWindowCommand, CollectWindowPayload, CursorCommand, Pointer, Sound,
    Vec2, World,
};
use honk_platform_windows::{
    pointer_state, warp_cursor, CollectWindowController, ForeignWindowWatcher, Overlay,
};

pub fn run(
    options: RuntimeOptions,
    server: &CommandServer,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut config = options.config.clone();
    let assets = assets::AssetCatalog::load();
    println!("honk300: loaded {}", assets.summary());

    let mut overlay = Overlay::new()?;
    // Fullscreen primary-monitor overlay so world-space props render where they belong.
    let bounds = Overlay::primary_bounds();
    let origin = bounds.min;
    let width = bounds.width().ceil().max(1.0) as u32;
    let height = bounds.height().ceil().max(1.0) as u32;

    let mut cursor_warp_supported =
        !(options.cli_overrides.no_mouse_steal || config.safety.no_mouse_steal);
    let initial_effective = effective_options(
        &config,
        &options,
        cursor_warp_supported,
        false,
        assets.note_count(),
        assets.meme_count(),
    );
    if initial_effective.wayland {
        eprintln!("honk300: --wayland is recorded for M18; Windows uses the native overlay.");
    }
    let mut audio = if initial_effective.no_sound {
        None
    } else {
        audio::Audio::new()
    };

    let mut warned_window_ride = false;
    let mut window_watcher = if !initial_effective.world.foreign_window.enabled {
        None
    } else {
        match ForeignWindowWatcher::new(&overlay) {
            Ok(watcher) => Some(watcher),
            Err(err) => {
                warned_window_ride = true;
                eprintln!("honk300: window ride unavailable; disabling perch-and-ride ({err})");
                None
            }
        }
    };
    let mut effective = effective_options(
        &config,
        &options,
        cursor_warp_supported,
        window_watcher.is_some(),
        assets.note_count(),
        assets.meme_count(),
    );

    let mut world = World::with_options(bounds, seed_from_clock(), effective.world);
    if let Some(kind) = smoke_collect_kind() {
        world.force_collect_window(kind);
    }
    let mut collect_controller = CollectWindowController::new(bounds);
    let mut canvas = Pixmap::new(width, height).ok_or("could not allocate the overlay canvas")?;
    let mut accumulator = Accumulator::new();
    let clock = Clock::start();
    let mut last = clock.elapsed_secs();
    let mut last_present = f32::NEG_INFINITY;
    // Fullscreen present is heavier than a tiny window, so cap it a little lower.
    const PRESENT_INTERVAL: f32 = 1.0 / 40.0;
    let mut warned_cursor_warp = false;
    let mut warned_collect_window = false;

    println!("honk300: a goose is loose on your desktop. Use `honk300 stop` to send it home.");

    loop {
        if !overlay.pump() {
            break;
        }

        while let Some(command) = server.try_recv() {
            match command {
                ControlCommand::Stop => {
                    println!("honk300: stop command received.");
                    return Ok(());
                }
                ControlCommand::Reload => match Config::load_existing(&options.config_path) {
                    Ok(next_config) => {
                        config = next_config;
                        effective = effective_options(
                            &config,
                            &options,
                            cursor_warp_supported,
                            window_watcher.is_some(),
                            assets.note_count(),
                            assets.meme_count(),
                        );
                        if !effective.world.foreign_window.enabled {
                            window_watcher = None;
                        } else if window_watcher.is_none() {
                            match ForeignWindowWatcher::new(&overlay) {
                                Ok(watcher) => window_watcher = Some(watcher),
                                Err(err) => {
                                    if !warned_window_ride {
                                        warned_window_ride = true;
                                        eprintln!(
                                            "honk300: window ride unavailable after reload ({err})"
                                        );
                                    }
                                }
                            }
                            effective = effective_options(
                                &config,
                                &options,
                                cursor_warp_supported,
                                window_watcher.is_some(),
                                assets.note_count(),
                                assets.meme_count(),
                            );
                        }
                        if effective.no_sound {
                            audio = None;
                        } else if audio.is_none() {
                            audio = audio::Audio::new();
                        }
                        world.apply_options(effective.world);
                        println!("honk300: reload command applied.");
                    }
                    Err(err) => {
                        eprintln!("honk300: reload rejected; keeping prior config ({err})");
                    }
                },
                ControlCommand::Do(action) => {
                    let outcome = world.poke(action);
                    println!("honk300: do {action:?} -> {outcome:?}");
                }
            }
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

        let mut disable_window_watcher = false;
        let dragged_window = match window_watcher.as_mut() {
            Some(watcher) => match watcher.active_drag() {
                Ok(snapshot) => snapshot,
                Err(err) => {
                    disable_window_watcher = true;
                    if !warned_window_ride {
                        warned_window_ride = true;
                        eprintln!(
                            "honk300: window ride polling failed; disabling perch-and-ride ({err})"
                        );
                    }
                    None
                }
            },
            None => None,
        };
        if disable_window_watcher {
            window_watcher = None;
            world.set_foreign_window_watch_supported(false);
        }
        world.set_foreign_window_drag(dragged_window);
        world.set_collect_window_snapshot(collect_controller.snapshot());

        for _ in 0..accumulator.pump(dt) {
            world.tick();
        }

        for command in world.take_collect_window_commands() {
            let result = match command {
                CollectWindowCommand::Spawn { request, payload } => match payload {
                    CollectWindowPayload::Note { .. } => {
                        collect_controller.spawn_note(request).map(|_| ())
                    }
                    CollectWindowPayload::Meme { index } => {
                        if let Some(meme) = assets.meme(index) {
                            collect_controller
                                .spawn_image(request, &meme.title, &meme.pixmap)
                                .map(|_| ())
                        } else {
                            Ok(())
                        }
                    }
                },
                CollectWindowCommand::Move { id, top_left } => {
                    collect_controller.move_window(id, top_left)
                }
                CollectWindowCommand::SetPassthrough { id, passthrough } => {
                    collect_controller.set_passthrough(id, passthrough)
                }
                CollectWindowCommand::Focus { id } => collect_controller.focus(id),
                CollectWindowCommand::TypeNote { id, note_index } => {
                    if let Some(text) = assets.note_text(note_index) {
                        collect_controller.type_text(id, text)
                    } else {
                        Ok(())
                    }
                }
                CollectWindowCommand::Close { id } => {
                    collect_controller.close(id);
                    Ok(())
                }
            };
            if let Err(err) = result {
                world.set_collect_window_supported(false);
                if !warned_collect_window {
                    warned_collect_window = true;
                    eprintln!("honk300: collect-window unavailable; disabling it ({err})");
                }
            }
        }

        // Apply at most the newest warp request. If the OS/session rejects cursor warping,
        // degrade honestly and stop registering further mouse-steal behavior.
        if let Some(CursorCommand::WarpTo(pos)) = world.take_cursor_commands().last().copied() {
            if let Err(err) = warp_cursor(pos) {
                cursor_warp_supported = false;
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
                if sound_enabled(effective.audio, s) {
                    a.play(s);
                }
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

fn effective_options(
    config: &Config,
    options: &RuntimeOptions,
    cursor_warp_supported: bool,
    window_watch_supported: bool,
    note_count: u32,
    meme_count: u32,
) -> EffectiveOptions {
    config.effective_options(
        BackendState {
            cursor_warp_supported,
            window_watch_supported,
            note_count,
            meme_count,
        },
        options.cli_overrides,
    )
}

fn sound_enabled(config: honk_config::AudioConfig, sound: Sound) -> bool {
    if !config.enabled {
        return false;
    }
    match sound {
        Sound::Honk => config.honk,
        Sound::Bite => config.bite,
        Sound::MudSquish => config.mud,
        Sound::Pat => config.pat,
    }
}

fn smoke_collect_kind() -> Option<honk_engine::CollectWindowKind> {
    match std::env::var("HONK300_SMOKE_COLLECT")
        .ok()?
        .to_ascii_lowercase()
        .as_str()
    {
        "note" => Some(honk_engine::CollectWindowKind::Note),
        "meme" => Some(honk_engine::CollectWindowKind::Meme),
        _ => None,
    }
}

/// A non-deterministic seed for the roam driver, derived from the wall clock.
fn seed_from_clock() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos() as u64)
        .unwrap_or(0x9E37_79B9_7F4A_7C15)
}
