//! honk300 — the binary entry point.
//!
//! M10 adds the local control plane around the current Windows runtime. The root
//! process parses CLI commands, sends stop/do/reload over IPC, or starts the one
//! allowed desktop goose instance.

mod cli;
mod install;
mod runtime;
mod update;

#[cfg(any(windows, target_os = "macos", target_os = "linux"))]
mod assets;
#[cfg(any(windows, target_os = "macos", target_os = "linux"))]
mod audio;

use cli::{Cli, Command, StartOptions};
#[cfg(any(windows, target_os = "macos", target_os = "linux"))]
use honk_config::CliOverrides;
use honk_config::{reset_to_defaults, Config, ConfigError, ConfigLoadState};
#[cfg(any(windows, target_os = "macos", target_os = "linux"))]
use honk_control::CommandServer;
use honk_control::{send_command, ControlCommand, ControlResponse, RuntimeStatus, Singleton};
#[cfg(any(windows, target_os = "macos", target_os = "linux"))]
use runtime::RuntimeOptions;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse_normalized();

    if cli.is_client_command() {
        return run_client_command(cli);
    }

    match cli.command {
        Some(Command::Config { config }) => run_config(config),
        Some(Command::Install { autostart }) => install::install(autostart),
        Some(Command::Uninstall { purge }) => install::uninstall(purge),
        Some(Command::Update) => update::run(),
        Some(Command::Setup { config, reset }) => run_setup(config, reset),
        Some(Command::Start { options }) => run_start(options),
        None => run_start(StartOptions::default()),
        Some(Command::Stop | Command::Reload | Command::Status | Command::Do { .. }) => {
            unreachable!()
        }
    }
}

fn run_client_command(cli: Cli) -> Result<(), Box<dyn std::error::Error>> {
    let command = match cli.command {
        Some(Command::Stop) => ControlCommand::Stop,
        Some(Command::Reload) => ControlCommand::Reload,
        Some(Command::Status) => ControlCommand::Status,
        Some(Command::Do { action }) => ControlCommand::Do(action.into_engine()),
        Some(
            Command::Start { .. }
            | Command::Config { .. }
            | Command::Install { .. }
            | Command::Uninstall { .. }
            | Command::Update
            | Command::Setup { .. },
        )
        | None => unreachable!("non-client commands are handled separately"),
    };
    let response = match send_command(command) {
        Ok(response) => response,
        Err(err)
            if matches!(
                err.kind(),
                std::io::ErrorKind::NotFound | std::io::ErrorKind::ConnectionRefused
            ) =>
        {
            if matches!(cli.command, Some(Command::Status)) {
                print_status(RuntimeStatus::not_running());
                return Ok(());
            }
            return Err("honk300: no running goose instance.".into());
        }
        Err(err) => return Err(err.into()),
    };

    match response {
        ControlResponse::Ok => {
            println!("honk300: command accepted.");
            Ok(())
        }
        ControlResponse::Err(code) => Err(format!("honk300 command rejected: {code}").into()),
        ControlResponse::Status(status) => {
            print_status(status);
            Ok(())
        }
    }
}

fn print_status(status: RuntimeStatus) {
    println!(
        "honk300: {}",
        if status.running {
            "running"
        } else {
            "not running"
        }
    );
    println!("platform: {}", status.platform.label());
    println!("bundle: {}", status.bundle.label());
    println!("overlay: {}", status.overlay.label());
    println!("accessibility: {}", status.accessibility.label());
    println!("cursor: {}", status.cursor.label());
    println!("window: {}", status.window.label());
    println!("collect: {}", status.collect.label());
    println!("presence: {}", status.presence.label());
    println!("audio: {}", status.audio.label());
    println!("assets: {} notes, {} memes", status.notes, status.memes);
}

fn run_config(config: Option<std::path::PathBuf>) -> Result<(), Box<dyn std::error::Error>> {
    let path = honk_config::resolve_path(config)?;
    honk_config_tui::run(path)?;
    Ok(())
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum SetupResult {
    Created,
    AlreadyExists,
    Reset { backup: Option<std::path::PathBuf> },
}

fn run_setup_at(path: &std::path::Path, reset: bool) -> Result<SetupResult, ConfigError> {
    if reset {
        return reset_to_defaults(path).map(|backup| SetupResult::Reset { backup });
    }
    match Config::load(Some(path.to_path_buf()))? {
        ConfigLoadState::Missing { .. } => {
            Config::default().save_atomic(path)?;
            Ok(SetupResult::Created)
        }
        ConfigLoadState::Loaded(_) => Ok(SetupResult::AlreadyExists),
        ConfigLoadState::Malformed { error, .. } => Err(ConfigError::MalformedDocument(error)),
        ConfigLoadState::UnsupportedVersion { found, .. } => Err(ConfigError::WrongVersion(found)),
    }
}

fn run_setup(
    config: Option<std::path::PathBuf>,
    reset: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let path = honk_config::resolve_path(config)?;
    match run_setup_at(&path, reset)? {
        SetupResult::Created => println!("honk300: config created at {}.", path.display()),
        SetupResult::AlreadyExists => println!(
            "honk300: config already exists at {}; left unchanged (use --reset to replace it).",
            path.display()
        ),
        SetupResult::Reset {
            backup: Some(backup),
        } => println!(
            "honk300: config reset at {}; backup saved to {}.",
            path.display(),
            backup.display()
        ),
        SetupResult::Reset { backup: None } => {
            println!("honk300: config created at {}.", path.display())
        }
    }
    Ok(())
}

#[cfg(windows)]
fn run_start(options: StartOptions) -> Result<(), Box<dyn std::error::Error>> {
    let (_singleton, status) = Singleton::acquire()?;
    if status == honk_control::SingletonStatus::AlreadyRunning {
        println!("honk300: a goose is already running. Use `honk300 stop` to stop it.");
        return Ok(());
    }

    let loaded = Config::load_or_default(options.config.clone())?;
    if let Some(warning) = &loaded.warning {
        eprintln!("honk300 config: {warning}");
    }

    let server = CommandServer::start()?;
    runtime::windows::run(
        RuntimeOptions {
            config_path: loaded.path,
            config: loaded.config,
            cli_overrides: CliOverrides {
                no_sound: options.no_sound,
                no_mouse_steal: options.no_mouse_steal,
                no_window_ride: options.no_window_ride,
                wayland: options.wayland,
            },
        },
        &server,
    )
}

#[cfg(target_os = "macos")]
fn run_start(options: StartOptions) -> Result<(), Box<dyn std::error::Error>> {
    let (_singleton, status) = Singleton::acquire()?;
    if status == honk_control::SingletonStatus::AlreadyRunning {
        println!("honk300: a goose is already running. Use `honk300 stop` to stop it.");
        return Ok(());
    }

    let loaded = Config::load_or_default(options.config.clone())?;
    if let Some(warning) = &loaded.warning {
        eprintln!("honk300 config: {warning}");
    }

    let server = CommandServer::start()?;
    runtime::macos::run(
        RuntimeOptions {
            config_path: loaded.path,
            config: loaded.config,
            cli_overrides: CliOverrides {
                no_sound: options.no_sound,
                no_mouse_steal: options.no_mouse_steal,
                no_window_ride: options.no_window_ride,
                wayland: options.wayland,
            },
        },
        &server,
    )
}

#[cfg(target_os = "linux")]
fn run_start(options: StartOptions) -> Result<(), Box<dyn std::error::Error>> {
    let (_singleton, status) = Singleton::acquire()?;
    if status == honk_control::SingletonStatus::AlreadyRunning {
        println!("honk300: a goose is already running. Use `honk300 stop` to stop it.");
        return Ok(());
    }
    let loaded = Config::load_or_default(options.config.clone())?;
    if let Some(warning) = &loaded.warning {
        eprintln!("honk300 config: {warning}");
    }

    let server = CommandServer::start()?;
    runtime::linux::run(
        RuntimeOptions {
            config_path: loaded.path,
            config: loaded.config,
            cli_overrides: CliOverrides {
                no_sound: options.no_sound,
                no_mouse_steal: options.no_mouse_steal,
                no_window_ride: options.no_window_ride,
                wayland: options.wayland,
            },
        },
        &server,
    )
}

#[cfg(not(any(windows, target_os = "macos", target_os = "linux")))]
fn run_start(options: StartOptions) -> Result<(), Box<dyn std::error::Error>> {
    let (_singleton, status) = Singleton::acquire()?;
    if status == honk_control::SingletonStatus::AlreadyRunning {
        println!("honk300: a goose is already running. Use `honk300 stop` to stop it.");
        return Ok(());
    }
    let loaded = Config::load_or_default(options.config)?;
    if let Some(warning) = loaded.warning {
        eprintln!("honk300 config: {warning}");
    }
    eprintln!("honk300: this OS does not have a desktop backend yet.");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn setup_creates_a_valid_v2_config_only_when_missing() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config.toml");
        assert_eq!(run_setup_at(&path, false).unwrap(), SetupResult::Created);
        assert_eq!(Config::load_existing(&path).unwrap(), Config::default());

        fs::write(
            &path,
            "goose_config_version = 2\nfuture_root = 'preserve me'\n",
        )
        .unwrap();
        let original = fs::read(&path).unwrap();
        assert_eq!(
            run_setup_at(&path, false).unwrap(),
            SetupResult::AlreadyExists
        );
        assert_eq!(fs::read(&path).unwrap(), original);
    }

    #[test]
    fn setup_refuses_malformed_existing_config_without_reset() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config.toml");
        let original = b"[broken\nvalue = true\n";
        fs::write(&path, original).unwrap();
        assert!(run_setup_at(&path, false).is_err());
        assert_eq!(fs::read(&path).unwrap(), original);
    }

    #[test]
    fn setup_reset_backs_up_then_replaces_existing_config() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config.toml");
        let original = b"goose_config_version = 99\nfuture = true\n";
        fs::write(&path, original).unwrap();
        let SetupResult::Reset {
            backup: Some(backup),
        } = run_setup_at(&path, true).unwrap()
        else {
            panic!("reset should report its backup");
        };
        assert_eq!(fs::read(backup).unwrap(), original);
        assert_eq!(Config::load_existing(&path).unwrap(), Config::default());
    }
}
