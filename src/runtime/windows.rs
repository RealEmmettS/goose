use crate::assets;
use crate::audio;
use crate::runtime::RuntimeOptions;
use honk_config::{BackendState, Config, EffectiveOptions};
use honk_control::{CommandServer, ControlCommand, ControlResponse};
use honk_engine::render::{
    render_autumn_leaves, render_footmarks_with_timing, render_hearts, render_rig_with_palette,
    render_sleepies, AutumnRenderLayer,
};
use honk_engine::tiny_skia::{Color, Pixmap};
use honk_engine::{
    Accumulator, Clock, CollectWindowCommand, CollectWindowPayload, CursorCommand, LocalTime,
    Pointer, PresenceSnapshot, Sound, Vec2, World,
};
use honk_platform_windows::{
    local_time, pointer_state, presence_state, warp_cursor, CollectWindowController,
    ForeignWindowWatcher, Overlay,
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

    // Backend capability only: Windows can always warp the cursor. The user's mouse-steal
    // preference is applied separately via MouseStealOptions::enabled in effective_options, so
    // it must NOT be folded in here — doing so would keep warp latched off across a reload that
    // re-enables stealing. The flag only flips to false later if a real warp call fails.
    let mut cursor_warp_supported = initial_cursor_warp_supported();
    let mut collect_window_supported = true;
    let initial_effective = effective_options(
        &config,
        &options,
        cursor_warp_supported,
        false,
        collect_window_supported,
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
        collect_window_supported,
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
    const PRESENCE_POLL_INTERVAL: f32 = 0.5;
    let mut warned_cursor_warp = false;
    let mut warned_collect_window = false;
    let mut warned_presence = false;
    let mut last_presence_poll = f32::NEG_INFINITY;

    println!("honk300: a goose is loose on your desktop. Use `honk300 stop` to send it home.");

    loop {
        if !overlay.pump() {
            break;
        }

        while let Some(request) = server.try_recv() {
            match request.command() {
                ControlCommand::Stop => {
                    println!("honk300: stop command received.");
                    request.respond(ControlResponse::Ok);
                    return Ok(());
                }
                ControlCommand::Reload => {
                    let response = match Config::load_existing(&options.config_path) {
                        Ok(next_config) => {
                            config = next_config;
                            effective = effective_options(
                                &config,
                                &options,
                                cursor_warp_supported,
                                window_watcher.is_some(),
                                collect_window_supported,
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
                                    collect_window_supported,
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
                            ControlResponse::Ok
                        }
                        Err(err) => {
                            eprintln!("honk300: reload rejected; keeping prior config ({err})");
                            ControlResponse::Err("RELOAD_REJECTED".into())
                        }
                    };
                    request.respond(response);
                }
                ControlCommand::Do(action) => {
                    let outcome = world.poke(action);
                    println!("honk300: do {action:?} -> {outcome:?}");
                    request.respond(outcome.into());
                }
            }
        }

        let now = clock.elapsed_secs();
        world.set_local_time(runtime_local_time());
        if now - last_presence_poll >= PRESENCE_POLL_INTERVAL {
            last_presence_poll = now;
            match presence_state() {
                Ok(snapshot) => world.set_presence(snapshot),
                Err(err) => {
                    world.set_presence(PresenceSnapshot::unsupported());
                    if !warned_presence {
                        warned_presence = true;
                        eprintln!(
                            "honk300: Windows presence unavailable; DND/fullscreen respect disabled ({err})"
                        );
                    }
                }
            }
        }

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
                // Latch the loss in the runtime capability flag too, so a later reload rebuilds
                // the world with collect still disabled instead of resurrecting it.
                collect_window_supported = false;
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
            render_footmarks_with_timing(
                &mut canvas,
                &world.goose.foot_marks,
                world.now(),
                origin,
                world.footmark_timing(),
            );
            render_autumn_leaves(
                &mut canvas,
                world.autumn(),
                world.now(),
                origin,
                world.goose.position,
                AutumnRenderLayer::BelowGoose,
            );
            render_rig_with_palette(&mut canvas, world.rig(), origin, world.render_palette());
            render_autumn_leaves(
                &mut canvas,
                world.autumn(),
                world.now(),
                origin,
                world.goose.position,
                AutumnRenderLayer::AboveGoose,
            );
            render_hearts(&mut canvas, world.hearts(), world.now(), origin);
            render_sleepies(&mut canvas, world.sleepies(), world.now(), origin);
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
    collect_window_supported: bool,
    note_count: u32,
    meme_count: u32,
) -> EffectiveOptions {
    config.effective_options(
        BackendState {
            cursor_warp_supported,
            window_watch_supported,
            collect_window_supported,
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
        Sound::Honk(_) => config.honk,
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

fn runtime_local_time() -> LocalTime {
    let mut time = local_time();
    if let Some(day) = smoke_local_date() {
        time.day = day;
    }
    time
}

fn smoke_local_date() -> Option<i32> {
    parse_smoke_local_date(&std::env::var("HONK300_SMOKE_LOCAL_DATE").ok()?)
}

fn parse_smoke_local_date(value: &str) -> Option<i32> {
    if value.len() != 8 || !value.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    let day = value.parse::<i32>().ok()?;
    let year = day / 10_000;
    let month = (day / 100) % 100;
    let date = day % 100;
    let max_day = days_in_month(year, month)?;
    (year >= 1900 && (1..=max_day).contains(&date)).then_some(day)
}

fn days_in_month(year: i32, month: i32) -> Option<i32> {
    Some(match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 if is_leap_year(year) => 29,
        2 => 28,
        _ => return None,
    })
}

fn is_leap_year(year: i32) -> bool {
    year % 4 == 0 && (year % 100 != 0 || year % 400 == 0)
}

/// The platform's initial cursor-warp capability, before any runtime warp attempt.
///
/// On Windows this is unconditionally `true` (`SetCursorPos` always exists); it is a backend
/// capability, deliberately independent of the user's mouse-steal preference, which is applied
/// separately via `MouseStealOptions::enabled`.
fn initial_cursor_warp_supported() -> bool {
    true
}

/// A non-deterministic seed for the roam driver, derived from the wall clock.
fn seed_from_clock() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos() as u64)
        .unwrap_or(0x9E37_79B9_7F4A_7C15)
}

#[cfg(test)]
mod tests {
    use super::{initial_cursor_warp_supported, parse_smoke_local_date};

    #[test]
    fn initial_cursor_warp_capability_ignores_mouse_steal_preference() {
        // The cursor-warp capability is a backend trait — Windows can always warp the cursor —
        // not a user choice. Seeding it from the no-mouse-steal preference would wrongly latch
        // warp off and keep it off across a reload that re-enables stealing. The preference is
        // applied separately through MouseStealOptions::enabled, so this stays a pure capability.
        assert!(initial_cursor_warp_supported());
    }

    #[test]
    fn smoke_local_date_accepts_yyyymmdd_only() {
        assert_eq!(parse_smoke_local_date("20261015"), Some(20261015));
        assert_eq!(parse_smoke_local_date("20240229"), Some(20240229));
        assert_eq!(parse_smoke_local_date("20260231"), None);
        assert_eq!(parse_smoke_local_date("20250229"), None);
        assert_eq!(parse_smoke_local_date("2026-10-15"), None);
        assert_eq!(parse_smoke_local_date("20261301"), None);
        assert_eq!(parse_smoke_local_date("18991231"), None);
    }
}
