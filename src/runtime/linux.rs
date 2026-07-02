use crate::assets;
use crate::audio;
use crate::runtime::RuntimeOptions;
use honk_config::{BackendCapability, BackendState, Config, EffectiveOptions};
use honk_control::{
    BundleStatus, CapabilityStatus, CommandServer, ControlCommand, ControlResponse, PlatformStatus,
    RuntimeStatus,
};
use honk_engine::{
    Accumulator, Clock, CollectWindowCommand, CollectWindowPayload, Pointer, PresenceSnapshot,
    Sound, Vec2, World,
};
use honk_platform_linux::{
    collect_window_supported, cursor_mischief_supported, default_world_bounds,
    foreign_window_watch_supported, local_time, presence_supported, DisplayServer, SessionInfo,
};

pub fn run(
    options: RuntimeOptions,
    server: &CommandServer,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut config = options.config.clone();
    let assets = assets::AssetCatalog::load();
    println!("honk300: loaded {}", assets.summary());

    let mut session = SessionInfo::detect(options.cli_overrides.wayland || config.platform.wayland);
    eprintln!(
        "honk300: Linux {} runtime active; unsupported desktop-control features degrade in status.",
        session.display_server.label()
    );

    let mut cursor_warp = cursor_capability(session.display_server);
    let mut window_watch = window_capability(session.display_server);
    let collect_window = collect_capability(session.display_server);
    let presence = presence_capability(session.display_server);
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

    let mut world = World::with_options(
        default_world_bounds(session.display_server),
        seed_from_clock(),
        effective.world,
    );
    let mut accumulator = Accumulator::new();
    let clock = Clock::start();
    let mut last = clock.elapsed_secs();
    let mut warned_collect = false;
    let mut warned_cursor = false;

    println!("honk300: Linux goose control is live. Use `honk300 stop` to send it home.");

    loop {
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
                            let prior_display = session.display_server;
                            config = next_config;
                            session = SessionInfo::detect(
                                options.cli_overrides.wayland || config.platform.wayland,
                            );
                            cursor_warp = cursor_capability(session.display_server);
                            window_watch = window_capability(session.display_server);
                            effective = effective_options(
                                &config,
                                &options,
                                backend_state(
                                    cursor_warp,
                                    window_watch,
                                    collect_capability(session.display_server),
                                    presence_capability(session.display_server),
                                    audio_capability,
                                    assets.note_count(),
                                    assets.meme_count(),
                                ),
                            );
                            if effective.no_sound {
                                audio = None;
                            } else if audio.is_none() {
                                audio = audio::Audio::new();
                                if audio.is_none() {
                                    audio_capability = BackendCapability::Failed;
                                }
                            }
                            if prior_display != session.display_server {
                                eprintln!(
                                    "honk300: Linux display mode changed from {} to {}; restart recommended once display backends are active.",
                                    prior_display.label(),
                                    session.display_server.label()
                                );
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
                ControlCommand::Status => {
                    request.respond(ControlResponse::Status(runtime_status(
                        cursor_warp,
                        window_watch,
                        collect_capability(session.display_server),
                        presence_capability(session.display_server),
                        audio_capability,
                        assets.note_count(),
                        assets.meme_count(),
                    )));
                }
            }
        }

        world.set_local_time(local_time());
        world.set_presence(PresenceSnapshot::unsupported());
        world.set_pointer(Pointer {
            pos: Vec2::ZERO,
            present: false,
            left_down: false,
        });
        world.set_foreign_window_drag(None);
        world.set_collect_window_snapshot(None);

        let now = clock.elapsed_secs();
        let dt = now - last;
        last = now;

        for _ in 0..accumulator.pump(dt) {
            world.tick();
        }

        let collect_commands = world.take_collect_window_commands();
        if !collect_commands.is_empty() {
            observe_collect_assets(&assets, collect_commands);
            world.set_collect_window_supported(false);
            if !warned_collect {
                warned_collect = true;
                eprintln!(
                    "honk300: Linux collect-window commands are unsupported in this runtime mode."
                );
            }
        }

        if !world.take_cursor_commands().is_empty() {
            cursor_warp = BackendCapability::Unsupported;
            world.set_cursor_warp_supported(false);
            if !warned_cursor {
                warned_cursor = true;
                eprintln!("honk300: Linux cursor warp is unsupported in this runtime mode.");
            }
        }

        let sounds = world.take_sounds();
        if let Some(a) = audio.as_mut() {
            for sound in sounds {
                if sound_enabled(effective.audio, sound) {
                    a.play(sound);
                }
            }
        }

        std::thread::sleep(std::time::Duration::from_millis(2));
    }
}

fn observe_collect_assets(assets: &assets::AssetCatalog, commands: Vec<CollectWindowCommand>) {
    for command in commands {
        if let CollectWindowCommand::Spawn { payload, .. } = command {
            match payload {
                CollectWindowPayload::Note { index } => {
                    let _ = assets.note_text(index);
                }
                CollectWindowPayload::Meme { index } => {
                    if let Some(meme) = assets.meme(index) {
                        let _ = (&meme.title, meme.pixmap.width(), meme.pixmap.height());
                    }
                }
            }
        }
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

fn cursor_capability(session: DisplayServer) -> BackendCapability {
    capability_for(session, cursor_mischief_supported)
}

fn window_capability(session: DisplayServer) -> BackendCapability {
    capability_for(session, foreign_window_watch_supported)
}

fn collect_capability(session: DisplayServer) -> BackendCapability {
    capability_for(session, collect_window_supported)
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
        platform: PlatformStatus::Linux,
        bundle: BundleStatus::Bare,
        accessibility: CapabilityStatus::Unsupported,
        cursor: capability_status(cursor),
        window: capability_status(window),
        collect: capability_status(collect),
        presence: capability_status(presence),
        audio: capability_status(audio),
        notes,
        memes,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unknown_display_maps_core_capabilities_to_failed() {
        assert_eq!(
            cursor_capability(DisplayServer::Unknown),
            BackendCapability::Failed
        );
        assert_eq!(
            window_capability(DisplayServer::Unknown),
            BackendCapability::Failed
        );
    }

    #[test]
    fn wayland_reports_core_mischief_unsupported_not_denied() {
        assert_eq!(
            cursor_capability(DisplayServer::Wayland),
            BackendCapability::Unsupported
        );
        assert_eq!(
            window_capability(DisplayServer::Wayland),
            BackendCapability::Unsupported
        );
    }

    #[test]
    fn linux_runtime_status_keeps_platform_and_bundle_stable() {
        let status = runtime_status(
            BackendCapability::Unsupported,
            BackendCapability::Unsupported,
            BackendCapability::Unsupported,
            BackendCapability::Unsupported,
            BackendCapability::Supported,
            2,
            3,
        );
        assert_eq!(status.platform, PlatformStatus::Linux);
        assert_eq!(status.bundle, BundleStatus::Bare);
        assert_eq!(status.audio, CapabilityStatus::Supported);
        assert_eq!(status.notes, 2);
        assert_eq!(status.memes, 3);
    }
}
