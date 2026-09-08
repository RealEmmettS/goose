use crate::assets;
use crate::audio;
use crate::runtime::control_surface;
use crate::runtime::core::RuntimeCore;
use crate::runtime::owned_props::Controller as PropController;
use crate::runtime::{audio_probe_capability, RuntimeOptions};
use honk_config::{BackendCapability, BackendState, Config, EffectiveOptions};
use honk_control::{
    BundleStatus, CapabilityStatus, CommandServer, ControlCommand, ControlResponse, DesktopBackend,
    PlatformStatus, RuntimeStatus, SessionStatus,
};
use honk_engine::render::DamageCanvas;
use honk_engine::render::{
    render_autumn_leaves, render_footmarks_with_timing, render_hearts, render_pose_with_palette,
    render_sleepies, AutumnRenderLayer,
};
use honk_engine::{CursorCommand, DesktopLayout, PresenceSnapshot, Rect, Sound, World};
use honk_platform_linux::{
    display_cursor_mischief_supported, display_foreign_window_watch_supported, local_time,
    presence_supported, DisplayServer, Overlay, OverlayMode, SessionInfo, StatusTray,
};

pub fn run(
    options: RuntimeOptions,
    server: &CommandServer,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut config = options.config.clone();
    let assets = assets::AssetCatalog::load();
    println!("honk300: loaded {}", assets.summary());

    let session = SessionInfo::detect(options.cli_overrides.wayland || config.platform.wayland);
    let mut overlay = Overlay::new(session.display_server)?;
    let mut status_tray = match StatusTray::new() {
        Ok(tray) => Some(tray),
        Err(error) => {
            eprintln!(
                "honk300: Linux StatusNotifier controls are unavailable; CLI controls remain active ({error})"
            );
            None
        }
    };
    let overlay_mode = overlay.mode();
    let display_server = overlay.display_server();
    eprintln!(
        "honk300: Linux {} runtime active; overlay mode is {:?}.",
        display_server.label(),
        overlay_mode
    );

    let mut cursor_warp = cursor_capability(overlay_mode, display_server);
    let mut window_watch = window_capability(overlay_mode, display_server);
    let mut collect_window = BackendCapability::Unsupported;
    let mut props = if overlay_mode == OverlayMode::Headless {
        None
    } else {
        match PropController::start(overlay_mode == OverlayMode::X11) {
            Ok(controller) => Some(controller),
            Err(error) => {
                if error
                    .downcast_ref::<std::io::Error>()
                    .is_none_or(|error| error.kind() != std::io::ErrorKind::NotFound)
                {
                    collect_window = BackendCapability::Failed;
                }
                eprintln!("honk300: native Linux notes and pictures unavailable ({error})");
                None
            }
        }
    };
    let presence = presence_capability(display_server);
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
        audio_capability = audio_probe_capability(false);
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
    }

    let layout = desktop_layout_for(
        effective.world.multi_monitor_chase,
        overlay.monitor_bounds(),
        overlay.bounds(),
    )?;
    let mut world = World::with_layout_and_options(layout, seed_from_clock(), effective.world);
    let mut core = RuntimeCore::new();
    let mut damage_canvas = DamageCanvas::default();
    const AUDIO_RETRY_INTERVAL: f64 = 5.0;
    let mut next_audio_probe = 0.0;
    let mut warned_cursor = false;

    println!("honk300: Linux goose control is live. Use `honk300 stop` to send it home.");

    loop {
        if !overlay.pump() {
            eprintln!("honk300: Linux overlay closed.");
            return Ok(());
        }

        if status_tray.as_ref().is_some_and(StatusTray::is_closed) {
            eprintln!(
                "honk300: Linux StatusNotifier controls stopped; CLI controls remain active."
            );
            status_tray = None;
        }
        while let Some(command) = status_tray.as_ref().and_then(StatusTray::take_command) {
            let action = control_surface::command_name(command);
            if let Err(error) = control_surface::handle_command(
                command,
                &mut world,
                || control_surface::open_configuration(&options.config_path),
                control_surface::open_update_helper,
            ) {
                eprintln!("honk300: {action} action could not start ({error})");
            }
        }

        if overlay.take_topology_changed() {
            let layout = desktop_layout_for(
                effective.world.multi_monitor_chase,
                overlay.monitor_bounds(),
                overlay.bounds(),
            )?;
            world.apply_layout(layout);
        }

        let frame = core.begin_frame();

        if let Some(controller) = props.as_mut() {
            match controller.poll() {
                Ok(()) if controller.ready() && collect_window != BackendCapability::Supported => {
                    collect_window = BackendCapability::Supported;
                    world.set_collect_window_supported(true);
                    world.set_collect_window_positioning(overlay_mode == OverlayMode::X11);
                    eprintln!(
                        "honk300: native Linux notes and pictures ready; animated placement: {}",
                        overlay_mode == OverlayMode::X11
                    );
                }
                Ok(()) => {}
                Err(error) => {
                    eprintln!("honk300: native Linux prop connection failed; disabling delivery ({error})");
                    props = None;
                    collect_window = BackendCapability::Failed;
                    world.set_collect_window_supported(false);
                }
            }
        }
        world.set_collect_window_capacity(props.as_ref().is_some_and(PropController::has_capacity));

        while let Some(request) = server.try_recv() {
            match request.command() {
                ControlCommand::Stop => {
                    println!("honk300: stop command received.");
                    request.respond(ControlResponse::Ok);
                    RuntimeCore::begin_graceful_stop(&mut world);
                }
                ControlCommand::ForceStop => {
                    println!("honk300: forced stop command received; stopping immediately.");
                    request.respond(ControlResponse::Ok);
                    return Ok(());
                }
                ControlCommand::Reload | ControlCommand::ReloadIf(_) => {
                    let response = match RuntimeCore::load_reload_config(
                        request.command(),
                        &options.config_path,
                    ) {
                        Ok(next_config)
                            if RuntimeCore::restart_required_reason(&config, &next_config)
                                .is_some() =>
                        {
                            let reason =
                                RuntimeCore::restart_required_reason(&config, &next_config)
                                    .expect("guard established restart-required changes");
                            eprintln!("honk300: reload rejected; restart required for {reason}");
                            ControlResponse::Err("RESTART_REQUIRED".into())
                        }
                        Ok(next_config) => {
                            config = next_config;
                            cursor_warp = cursor_capability(overlay_mode, display_server);
                            window_watch = window_capability(overlay_mode, display_server);
                            effective = effective_options(
                                &config,
                                &options,
                                backend_state(
                                    cursor_warp,
                                    window_watch,
                                    collect_window,
                                    presence_capability(display_server),
                                    audio_capability,
                                    assets.note_count(),
                                    assets.meme_count(),
                                ),
                            );
                            if effective.no_sound {
                                audio = None;
                            } else if audio.is_none() {
                                audio = audio::Audio::new();
                                audio_capability = audio_probe_capability(audio.is_some());
                            }
                            world.apply_options(effective.world);
                            world.set_collect_window_positioning(
                                collect_window == BackendCapability::Supported
                                    && overlay_mode == OverlayMode::X11,
                            );
                            world.set_collect_window_capacity(
                                props.as_ref().is_some_and(PropController::has_capacity),
                            );
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
                ControlCommand::Status => {
                    request.respond(ControlResponse::Status(runtime_status(
                        overlay_capability(overlay_mode, display_server),
                        backend_state(
                            cursor_warp,
                            window_watch,
                            collect_window,
                            presence_capability(display_server),
                            audio_capability,
                            assets.note_count(),
                            assets.meme_count(),
                        ),
                    )));
                }
                ControlCommand::Session => {
                    let backend = match overlay_mode {
                        OverlayMode::X11
                            if session
                                .xdg_session_type
                                .as_deref()
                                .is_some_and(|value| value.eq_ignore_ascii_case("wayland")) =>
                        {
                            DesktopBackend::X11OnWayland
                        }
                        OverlayMode::X11 => DesktopBackend::X11,
                        OverlayMode::Wayland => DesktopBackend::Wayland,
                        OverlayMode::Headless => DesktopBackend::Headless,
                    };
                    let prop_positioning = if collect_window == BackendCapability::Supported
                        && overlay_mode != OverlayMode::X11
                    {
                        CapabilityStatus::Unsupported
                    } else {
                        capability_status(collect_window)
                    };
                    request.respond(ControlResponse::Session(SessionStatus {
                        backend,
                        desktop: session.desktop,
                        prop_positioning,
                    }));
                }
            }
        }

        world.set_local_time(local_time());
        world.set_presence(PresenceSnapshot::unsupported());
        let pointer = overlay.pointer_state();
        world.set_pointer(pointer);
        world.set_foreign_window_drag(overlay.foreign_window_drag());
        world.set_collect_window_snapshot(props.as_mut().and_then(PropController::snapshot));
        let _ = overlay.set_input_region(Some(world.rig().bounding_box()));

        let now = frame.now();
        core.tick(&mut world, frame);

        let collect_display = world
            .layout()
            .region_at(world.goose.position)
            .and_then(|index| world.layout().regions().get(index).copied())
            .unwrap_or_else(|| overlay.bounds());
        for command in world.take_collect_window_commands() {
            if let Some(controller) = props.as_mut() {
                if let Err(error) = controller.apply(command, &assets, collect_display) {
                    eprintln!(
                        "honk300: native Linux prop command failed; disabling delivery ({error})"
                    );
                    props = None;
                    collect_window = BackendCapability::Failed;
                    world.set_collect_window_supported(false);
                }
            }
        }

        let cursor_commands = world.take_cursor_commands();
        if let Some(CursorCommand::WarpTo(pos)) = cursor_commands.last().copied() {
            if let Err(err) = overlay.warp_cursor(pos) {
                cursor_warp = if err.kind() == std::io::ErrorKind::Unsupported {
                    BackendCapability::Unsupported
                } else {
                    BackendCapability::Failed
                };
                world.set_cursor_warp_supported(false);
                if !warned_cursor {
                    warned_cursor = true;
                    eprintln!("honk300: Linux cursor warp unavailable; disabling it ({err})");
                }
            }
        }

        if let Some(audio) = audio.as_mut() {
            audio.poll();
        }
        if !effective.no_sound && audio.is_none() && now >= next_audio_probe {
            audio = audio::Audio::new();
            audio_capability = audio_probe_capability(audio.is_some());
            next_audio_probe = now + AUDIO_RETRY_INTERVAL;
        }

        let sounds = world.take_sounds();
        let mut audio_failed = false;
        if let Some(a) = audio.as_mut() {
            for sound in sounds {
                if sound_enabled(effective.audio, sound)
                    && a.play(sound) == audio::PlayOutcome::Failed
                {
                    audio_failed = true;
                    break;
                }
            }
        }
        if audio_failed {
            audio = None;
            audio_capability = BackendCapability::Failed;
            next_audio_probe = now + AUDIO_RETRY_INTERVAL;
        }

        if let Some(dirty) = core.damage(&world, frame) {
            let width = dirty.width().ceil().max(1.0) as u32;
            let height = dirty.height().ceil().max(1.0) as u32;
            let origin = dirty.min;
            let canvas = damage_canvas.prepare(width, height)?;
            render_footmarks_with_timing(
                canvas,
                &world.goose.foot_marks,
                world.now(),
                origin,
                world.footmark_timing(),
            );
            render_autumn_leaves(
                canvas,
                world.autumn(),
                world.now(),
                origin,
                world.goose.position,
                AutumnRenderLayer::BelowGoose,
            );
            render_pose_with_palette(canvas, world.pose(), origin, world.render_palette());
            render_autumn_leaves(
                canvas,
                world.autumn(),
                world.now(),
                origin,
                world.goose.position,
                AutumnRenderLayer::AboveGoose,
            );
            render_hearts(canvas, world.hearts(), world.now(), origin);
            render_sleepies(canvas, world.sleepies(), world.now(), origin);
            if overlay.present(dirty, canvas)? {
                core.acknowledge_present();
            } else {
                core.defer_present();
            }
        }

        if core.graceful_stop_complete(&world) {
            println!("honk300: goose walked home; stopping.");
            return Ok(());
        }

        std::thread::sleep(core.next_tick_delay());
    }
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

fn cursor_capability(mode: OverlayMode, session: DisplayServer) -> BackendCapability {
    if display_cursor_mischief_supported(session) {
        return if mode == OverlayMode::X11 {
            BackendCapability::Supported
        } else {
            BackendCapability::Failed
        };
    }
    capability_for(session, display_cursor_mischief_supported)
}

fn window_capability(mode: OverlayMode, session: DisplayServer) -> BackendCapability {
    if display_foreign_window_watch_supported(session) {
        return if mode == OverlayMode::X11 {
            BackendCapability::Supported
        } else {
            BackendCapability::Failed
        };
    }
    capability_for(session, display_foreign_window_watch_supported)
}

fn presence_capability(session: DisplayServer) -> BackendCapability {
    capability_for(session, presence_supported)
}

fn capability_for(
    session: DisplayServer,
    supported: impl FnOnce(DisplayServer) -> bool,
) -> BackendCapability {
    if supported(session) {
        BackendCapability::Supported
    } else if session == DisplayServer::Unknown {
        BackendCapability::Failed
    } else {
        BackendCapability::Unsupported
    }
}

/// Map the live overlay to a status capability so `honk300 status` can tell a visible overlay
/// apart from the invisible headless fallback (which only runs when `HONK300_ALLOW_HEADLESS=1`).
fn overlay_capability(mode: OverlayMode, session: DisplayServer) -> CapabilityStatus {
    match mode {
        OverlayMode::X11 | OverlayMode::Wayland => CapabilityStatus::Supported,
        OverlayMode::Headless => {
            if session == DisplayServer::Unknown {
                // No display server was ever detected: nothing to bring up, not a failure.
                CapabilityStatus::Unsupported
            } else {
                // An X11/Wayland overlay was attempted and fell back headless.
                CapabilityStatus::Failed
            }
        }
    }
}

fn runtime_status(overlay: CapabilityStatus, backend: BackendState) -> RuntimeStatus {
    RuntimeStatus {
        running: true,
        platform: PlatformStatus::Linux,
        bundle: BundleStatus::Bare,
        overlay,
        accessibility: CapabilityStatus::Unsupported,
        cursor: capability_status(backend.cursor_warp),
        window: capability_status(backend.window_watch),
        collect: capability_status(backend.collect_window),
        presence: capability_status(backend.presence),
        audio: capability_status(backend.audio),
        notes: backend.note_count,
        memes: backend.meme_count,
    }
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

fn desktop_layout_for(
    multi_monitor_chase: bool,
    monitor_bounds: Vec<Rect>,
    fallback_bounds: Rect,
) -> Result<DesktopLayout, honk_engine::DesktopLayoutError> {
    if multi_monitor_chase {
        DesktopLayout::new(monitor_bounds)
    } else {
        Ok(DesktopLayout::single(
            monitor_bounds.into_iter().next().unwrap_or(fallback_bounds),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unknown_display_maps_core_capabilities_to_failed() {
        assert_eq!(
            cursor_capability(OverlayMode::Headless, DisplayServer::Unknown),
            BackendCapability::Failed
        );
        assert_eq!(
            window_capability(OverlayMode::Headless, DisplayServer::Unknown),
            BackendCapability::Failed
        );
    }

    #[test]
    fn x11_session_with_headless_fallback_reports_failed_desktop_capabilities() {
        assert_eq!(
            cursor_capability(OverlayMode::Headless, DisplayServer::X11),
            BackendCapability::Failed
        );
        assert_eq!(
            window_capability(OverlayMode::Headless, DisplayServer::X11),
            BackendCapability::Failed
        );
    }

    #[test]
    fn wayland_reports_core_mischief_unsupported_not_denied() {
        assert_eq!(
            cursor_capability(OverlayMode::Wayland, DisplayServer::Wayland),
            BackendCapability::Unsupported
        );
        assert_eq!(
            window_capability(OverlayMode::Wayland, DisplayServer::Wayland),
            BackendCapability::Unsupported
        );
    }

    #[test]
    fn x11_reports_supported_cursor_and_window() {
        assert_eq!(
            cursor_capability(OverlayMode::X11, DisplayServer::X11),
            BackendCapability::Supported
        );
        assert_eq!(
            window_capability(OverlayMode::X11, DisplayServer::X11),
            BackendCapability::Supported
        );
    }

    #[test]
    fn linux_runtime_status_keeps_platform_and_bundle_stable() {
        let status = runtime_status(
            CapabilityStatus::Supported,
            backend_state(
                BackendCapability::Unsupported,
                BackendCapability::Unsupported,
                BackendCapability::Unsupported,
                BackendCapability::Unsupported,
                BackendCapability::Supported,
                2,
                3,
            ),
        );
        assert_eq!(status.platform, PlatformStatus::Linux);
        assert_eq!(status.bundle, BundleStatus::Bare);
        assert_eq!(status.overlay, CapabilityStatus::Supported);
        assert_eq!(status.audio, CapabilityStatus::Supported);
        assert_eq!(status.notes, 2);
        assert_eq!(status.memes, 3);
    }

    #[test]
    fn overlay_capability_distinguishes_visible_from_headless_fallback() {
        // A visible X11/Wayland overlay reports supported.
        assert_eq!(
            overlay_capability(OverlayMode::X11, DisplayServer::X11),
            CapabilityStatus::Supported
        );
        assert_eq!(
            overlay_capability(OverlayMode::Wayland, DisplayServer::Wayland),
            CapabilityStatus::Supported
        );
        // A headless fallback after a real X11/Wayland attempt reports failed, not supported.
        assert_eq!(
            overlay_capability(OverlayMode::Headless, DisplayServer::X11),
            CapabilityStatus::Failed
        );
        assert_eq!(
            overlay_capability(OverlayMode::Headless, DisplayServer::Wayland),
            CapabilityStatus::Failed
        );
        // No display server detected at all is unsupported, not a failure.
        assert_eq!(
            overlay_capability(OverlayMode::Headless, DisplayServer::Unknown),
            CapabilityStatus::Unsupported
        );
    }
}
