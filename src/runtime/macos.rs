use crate::assets;
use crate::audio;
use crate::runtime::RuntimeOptions;
use honk_config::{BackendCapability, BackendState, Config, EffectiveOptions};
use honk_control::{
    BundleStatus, CapabilityStatus, CommandServer, ControlCommand, ControlResponse, PlatformStatus,
    RuntimeStatus,
};
use honk_engine::render::{
    render_autumn_leaves, render_footmarks_with_timing, render_hearts, render_rig_with_palette,
    render_sleepies, AutumnRenderLayer,
};
use honk_engine::tiny_skia::{Color, Pixmap};
use honk_engine::{
    Accumulator, Clock, CollectWindowCommand, CollectWindowPayload, CursorCommand, Pointer,
    PresenceSnapshot, Rect, Sound, Vec2, World,
};
use honk_platform_macos::{
    accessibility_state, local_time, pointer_state, presence_state, warp_cursor,
    AccessibilityState, CollectWindowController, ForeignWindowWatcher, Overlay,
};

pub fn run(
    options: RuntimeOptions,
    server: &CommandServer,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut config = options.config.clone();
    let assets = assets::AssetCatalog::load();
    println!("honk300: loaded {}", assets.summary());

    let mut overlay = Overlay::new()?;
    let primary_bounds = overlay.primary_monitor_bounds();
    let virtual_bounds = overlay.virtual_desktop_bounds();

    let mut cursor_warp = accessibility_capability();
    let mut window_watch = accessibility_capability();
    let mut collect_window = BackendCapability::Supported;
    let presence = BackendCapability::Unsupported;
    let mut audio_capability = BackendCapability::Supported;

    let mut effective = effective_options(
        &config,
        &options,
        backend_state(
            cursor_warp,
            window_watch,
            collect_window,
            presence,
            audio_capability,
            assets.note_count(),
            assets.meme_count(),
        ),
    );
    let mut audio = if effective.no_sound {
        None
    } else {
        audio::Audio::new()
    };
    if !effective.no_sound && audio.is_none() {
        audio_capability = BackendCapability::Failed;
    }

    let mut warned_window_ride = false;
    let mut window_watcher = if !effective.world.foreign_window.enabled {
        None
    } else {
        match ForeignWindowWatcher::new(&overlay) {
            Ok(watcher) => Some(watcher),
            Err(err) => {
                window_watch = permission_or_failed(&err);
                warned_window_ride = true;
                eprintln!("honk300: macOS window ride unavailable; disabling it ({err})");
                None
            }
        }
    };
    effective = effective_options(
        &config,
        &options,
        backend_state(
            cursor_warp,
            window_watch,
            collect_window,
            presence,
            audio_capability,
            assets.note_count(),
            assets.meme_count(),
        ),
    );

    let world_bounds = world_bounds_for(
        effective.world.multi_monitor_chase,
        primary_bounds,
        virtual_bounds,
    );
    let mut world = World::with_options(world_bounds, seed_from_clock(), effective.world);
    let mut collect_controller = CollectWindowController::new(primary_bounds, virtual_bounds);
    let mut accumulator = Accumulator::new();
    let clock = Clock::start();
    let mut last = clock.elapsed_secs();
    let mut last_present = f32::NEG_INFINITY;
    let mut last_render_bounds: Option<Rect> = None;
    const PRESENT_INTERVAL: f32 = 1.0 / 60.0;
    let mut warned_cursor_warp = false;
    let mut warned_collect_window = false;

    println!("honk300: a macOS goose is loose. Use `honk300 stop` to send it home.");

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
                            let prior_multi_monitor_chase = effective.world.multi_monitor_chase;
                            config = next_config;
                            cursor_warp = refresh_accessibility_capability(cursor_warp);
                            window_watch = refresh_accessibility_capability(window_watch);
                            collect_window = refresh_supported_capability(collect_window);
                            effective = effective_options(
                                &config,
                                &options,
                                backend_state(
                                    cursor_warp,
                                    window_watch,
                                    collect_window,
                                    presence,
                                    audio_capability,
                                    assets.note_count(),
                                    assets.meme_count(),
                                ),
                            );
                            if !effective.world.foreign_window.enabled {
                                window_watcher = None;
                            } else if window_watcher.is_none() && window_watch.active() {
                                match ForeignWindowWatcher::new(&overlay) {
                                    Ok(watcher) => window_watcher = Some(watcher),
                                    Err(err) => {
                                        window_watch = permission_or_failed(&err);
                                        if !warned_window_ride {
                                            warned_window_ride = true;
                                            eprintln!(
                                                "honk300: macOS window ride unavailable after reload ({err})"
                                            );
                                        }
                                    }
                                }
                            }
                            if effective.no_sound {
                                audio = None;
                            } else if audio.is_none() {
                                audio = audio::Audio::new();
                                if audio.is_none() {
                                    audio_capability = BackendCapability::Failed;
                                }
                            }
                            if effective.world.multi_monitor_chase != prior_multi_monitor_chase {
                                eprintln!(
                                    "honk300: multi-monitor chase changed; restart required for bounds/window rebuild"
                                );
                            }
                            world.apply_options(effective.world);
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
                ControlCommand::Status => {
                    request.respond(ControlResponse::Status(runtime_status(
                        cursor_warp,
                        window_watch,
                        collect_window,
                        presence,
                        audio_capability,
                        assets.note_count(),
                        assets.meme_count(),
                    )));
                }
            }
        }

        world.set_local_time(local_time());
        world.set_presence(presence_state().unwrap_or_else(|_| PresenceSnapshot::unsupported()));

        let now = clock.elapsed_secs();
        let dt = now - last;
        last = now;

        let (mx, my, left_down) = pointer_state();
        let pointer = Vec2::new(mx, my);
        world.set_pointer(Pointer {
            pos: pointer,
            present: true,
            left_down,
        });
        overlay.set_interactive(world.goose_hit(pointer));

        let mut disable_window_watcher = false;
        let dragged_window = match window_watcher.as_mut() {
            Some(watcher) => match watcher.active_drag() {
                Ok(snapshot) => snapshot,
                Err(err) => {
                    disable_window_watcher = true;
                    window_watch = permission_or_failed(&err);
                    if !warned_window_ride {
                        warned_window_ride = true;
                        eprintln!("honk300: macOS window ride polling failed ({err})");
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
                collect_window = permission_or_failed(&err);
                world.set_collect_window_supported(false);
                if !warned_collect_window {
                    warned_collect_window = true;
                    eprintln!("honk300: macOS collect-window unavailable; disabling it ({err})");
                }
            }
        }

        if let Some(CursorCommand::WarpTo(pos)) = world.take_cursor_commands().last().copied() {
            if let Err(err) = warp_cursor(pos) {
                cursor_warp = permission_or_failed(&err);
                world.set_cursor_warp_supported(false);
                if !warned_cursor_warp {
                    warned_cursor_warp = true;
                    eprintln!("honk300: macOS cursor warp unavailable; disabling it ({err})");
                }
            }
        }

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
            let dirty = world.render_bounds(last_render_bounds);
            let width = dirty.width().ceil().max(1.0) as u32;
            let height = dirty.height().ceil().max(1.0) as u32;
            let origin = dirty.min;
            let mut canvas =
                Pixmap::new(width, height).ok_or("could not allocate dirty overlay canvas")?;
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
            overlay.present(dirty, &canvas)?;
            last_render_bounds = Some(dirty);
        }

        std::thread::sleep(std::time::Duration::from_millis(2));
    }

    Ok(())
}

fn effective_options(
    config: &Config,
    options: &RuntimeOptions,
    backend: BackendState,
) -> EffectiveOptions {
    config.effective_options(backend, options.cli_overrides)
}

fn backend_state(
    cursor_warp: BackendCapability,
    window_watch: BackendCapability,
    collect_window: BackendCapability,
    presence: BackendCapability,
    audio: BackendCapability,
    note_count: u32,
    meme_count: u32,
) -> BackendState {
    BackendState {
        cursor_warp,
        window_watch,
        collect_window,
        presence,
        audio,
        note_count,
        meme_count,
    }
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

fn world_bounds_for(multi_monitor_chase: bool, primary_bounds: Rect, virtual_bounds: Rect) -> Rect {
    if multi_monitor_chase {
        virtual_bounds
    } else {
        primary_bounds
    }
}

fn accessibility_capability() -> BackendCapability {
    match accessibility_state() {
        AccessibilityState::Trusted => BackendCapability::Supported,
        AccessibilityState::Denied => BackendCapability::Denied,
    }
}

fn permission_or_failed(err: &std::io::Error) -> BackendCapability {
    if err.kind() == std::io::ErrorKind::PermissionDenied {
        BackendCapability::Denied
    } else if err.kind() == std::io::ErrorKind::Unsupported {
        BackendCapability::Unsupported
    } else {
        BackendCapability::Failed
    }
}

fn refresh_accessibility_capability(current: BackendCapability) -> BackendCapability {
    match current {
        BackendCapability::Failed | BackendCapability::Unsupported => current,
        BackendCapability::Supported | BackendCapability::Denied => accessibility_capability(),
    }
}

fn refresh_supported_capability(current: BackendCapability) -> BackendCapability {
    match current {
        BackendCapability::Failed | BackendCapability::Unsupported => current,
        BackendCapability::Supported | BackendCapability::Denied => BackendCapability::Supported,
    }
}

fn runtime_status(
    cursor: BackendCapability,
    window: BackendCapability,
    collect: BackendCapability,
    presence: BackendCapability,
    audio: BackendCapability,
    notes: u32,
    memes: u32,
) -> RuntimeStatus {
    RuntimeStatus {
        running: true,
        platform: PlatformStatus::Macos,
        bundle: macos_bundle_status(),
        accessibility: capability_status(accessibility_capability()),
        cursor: capability_status(cursor),
        window: capability_status(window),
        collect: capability_status(collect),
        presence: capability_status(presence),
        audio: capability_status(audio),
        notes,
        memes,
    }
}

fn macos_bundle_status() -> BundleStatus {
    std::env::current_exe()
        .ok()
        .and_then(|exe| {
            exe.ancestors()
                .find(|path| path.extension().and_then(|ext| ext.to_str()) == Some("app"))
                .map(|_| BundleStatus::App)
        })
        .unwrap_or(BundleStatus::Bare)
}

fn capability_status(capability: BackendCapability) -> CapabilityStatus {
    match capability {
        BackendCapability::Supported => CapabilityStatus::Supported,
        BackendCapability::Unsupported => CapabilityStatus::Unsupported,
        BackendCapability::Denied => CapabilityStatus::Denied,
        BackendCapability::Failed => CapabilityStatus::Failed,
    }
}

fn seed_from_clock() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos() as u64)
        .unwrap_or(0x9E37_79B9_7F4A_7C15)
}
