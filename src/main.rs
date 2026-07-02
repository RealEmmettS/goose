//! honk300 — the binary entry point.
//!
//! M10 adds the local control plane around the current Windows runtime. The root
//! process parses CLI commands, sends stop/do/reload over IPC, or starts the one
//! allowed desktop goose instance.

mod cli;
mod runtime;

#[cfg(any(windows, target_os = "macos", target_os = "linux"))]
mod assets;
#[cfg(any(windows, target_os = "macos", target_os = "linux"))]
mod audio;

use cli::{Cli, Command};
#[cfg(any(windows, target_os = "macos", target_os = "linux"))]
use honk_config::CliOverrides;
use honk_config::Config;
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
        Some(Command::Config) => run_config(cli),
        Some(Command::Install) => lifecycle_placeholder("install"),
        Some(Command::Uninstall) => lifecycle_placeholder("uninstall"),
        Some(Command::Update) => lifecycle_placeholder("update"),
        Some(Command::Setup) => run_setup(cli),
        Some(Command::Start) | None => run_start(cli),
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
            Command::Start
            | Command::Config
            | Command::Install
            | Command::Uninstall
            | Command::Update
            | Command::Setup,
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
    println!("accessibility: {}", status.accessibility.label());
    println!("cursor: {}", status.cursor.label());
    println!("window: {}", status.window.label());
    println!("collect: {}", status.collect.label());
    println!("presence: {}", status.presence.label());
    println!("audio: {}", status.audio.label());
    println!("assets: {} notes, {} memes", status.notes, status.memes);
}

fn run_config(cli: Cli) -> Result<(), Box<dyn std::error::Error>> {
    let loaded = Config::load_or_default(cli.config)?;
    if let Some(warning) = loaded.warning {
        eprintln!("honk300 config: {warning}");
    }
    honk_config_tui::run(loaded.path)?;
    Ok(())
}

fn run_setup(cli: Cli) -> Result<(), Box<dyn std::error::Error>> {
    let loaded = Config::load_or_default(cli.config)?;
    loaded.config.save_atomic(&loaded.path)?;
    println!("honk300: config ready at {}.", loaded.path.display());
    Ok(())
}

fn lifecycle_placeholder(action: &str) -> Result<(), Box<dyn std::error::Error>> {
    println!("honk300 {action}: installer lifecycle commands land in M19.");
    Ok(())
}

#[cfg(windows)]
fn run_start(cli: Cli) -> Result<(), Box<dyn std::error::Error>> {
    let (_singleton, status) = Singleton::acquire()?;
    if status == honk_control::SingletonStatus::AlreadyRunning {
        println!("honk300: a goose is already running. Use `honk300 stop` to stop it.");
        return Ok(());
    }

    let loaded = Config::load_or_default(cli.config.clone())?;
    if let Some(warning) = &loaded.warning {
        eprintln!("honk300: ignoring config problem and using defaults ({warning})");
    }

    let server = CommandServer::start()?;
    runtime::windows::run(
        RuntimeOptions {
            config_path: loaded.path,
            config: loaded.config,
            cli_overrides: CliOverrides {
                no_sound: cli.no_sound,
                no_mouse_steal: cli.no_mouse_steal,
                no_window_ride: cli.no_window_ride,
                wayland: cli.wayland,
            },
        },
        &server,
    )
}

#[cfg(target_os = "macos")]
fn run_start(cli: Cli) -> Result<(), Box<dyn std::error::Error>> {
    let (_singleton, status) = Singleton::acquire()?;
    if status == honk_control::SingletonStatus::AlreadyRunning {
        println!("honk300: a goose is already running. Use `honk300 stop` to stop it.");
        return Ok(());
    }

    let loaded = Config::load_or_default(cli.config.clone())?;
    if let Some(warning) = &loaded.warning {
        eprintln!("honk300: ignoring config problem and using defaults ({warning})");
    }

    let server = CommandServer::start()?;
    runtime::macos::run(
        RuntimeOptions {
            config_path: loaded.path,
            config: loaded.config,
            cli_overrides: CliOverrides {
                no_sound: cli.no_sound,
                no_mouse_steal: cli.no_mouse_steal,
                no_window_ride: cli.no_window_ride,
                wayland: cli.wayland,
            },
        },
        &server,
    )
}

#[cfg(target_os = "linux")]
fn run_start(cli: Cli) -> Result<(), Box<dyn std::error::Error>> {
    let (_singleton, status) = Singleton::acquire()?;
    if status == honk_control::SingletonStatus::AlreadyRunning {
        println!("honk300: a goose is already running. Use `honk300 stop` to stop it.");
        return Ok(());
    }
    let loaded = Config::load_or_default(cli.config.clone())?;
    if let Some(warning) = &loaded.warning {
        eprintln!("honk300: ignoring config problem and using defaults ({warning})");
    }

    let server = CommandServer::start()?;
    runtime::linux::run(
        RuntimeOptions {
            config_path: loaded.path,
            config: loaded.config,
            cli_overrides: CliOverrides {
                no_sound: cli.no_sound,
                no_mouse_steal: cli.no_mouse_steal,
                no_window_ride: cli.no_window_ride,
                wayland: cli.wayland,
            },
        },
        &server,
    )
}

#[cfg(not(any(windows, target_os = "macos", target_os = "linux")))]
fn run_start(cli: Cli) -> Result<(), Box<dyn std::error::Error>> {
    let (_singleton, status) = Singleton::acquire()?;
    if status == honk_control::SingletonStatus::AlreadyRunning {
        println!("honk300: a goose is already running. Use `honk300 stop` to stop it.");
        return Ok(());
    }
    let loaded = Config::load_or_default(cli.config)?;
    if let Some(warning) = loaded.warning {
        eprintln!("honk300: ignoring config problem and using defaults ({warning})");
    }
    eprintln!("honk300: this OS does not have a desktop backend yet.");
    Ok(())
}
