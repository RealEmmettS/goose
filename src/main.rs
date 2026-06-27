//! honk300 — the binary entry point.
//!
//! M10 adds the local control plane around the current Windows runtime. The root
//! process parses CLI commands, sends stop/do/reload over IPC, or starts the one
//! allowed desktop goose instance.

mod cli;
mod control;
mod runtime;

#[cfg(windows)]
mod assets;
#[cfg(windows)]
mod audio;

use clap::Parser;
use cli::{Cli, Command};
#[cfg(windows)]
use control::CommandServer;
use control::{send_command, ControlCommand, ControlResponse, Singleton};
#[cfg(windows)]
use runtime::RuntimeOptions;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    if !cli.is_start() {
        return run_client_command(cli);
    }

    run_start(cli)
}

fn run_client_command(cli: Cli) -> Result<(), Box<dyn std::error::Error>> {
    let command = match cli.command {
        Some(Command::Stop) => ControlCommand::Stop,
        Some(Command::Reload) => ControlCommand::Reload,
        Some(Command::Do { action }) => ControlCommand::Do(action.into_engine()),
        Some(Command::Start) | None => unreachable!("start commands are handled separately"),
    };
    let response = match send_command(command) {
        Ok(response) => response,
        Err(err)
            if matches!(
                err.kind(),
                std::io::ErrorKind::NotFound | std::io::ErrorKind::ConnectionRefused
            ) =>
        {
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
    }
}

#[cfg(windows)]
fn run_start(cli: Cli) -> Result<(), Box<dyn std::error::Error>> {
    let (_singleton, status) = Singleton::acquire()?;
    if status == control::SingletonStatus::AlreadyRunning {
        println!("honk300: a goose is already running. Use `honk300 stop` to stop it.");
        return Ok(());
    }

    let server = CommandServer::start()?;
    runtime::windows::run(
        RuntimeOptions {
            no_sound: cli.no_sound,
            no_mouse_steal: cli.no_mouse_steal,
            no_window_ride: cli.no_window_ride,
        },
        &server,
    )
}

#[cfg(not(windows))]
fn run_start(_cli: Cli) -> Result<(), Box<dyn std::error::Error>> {
    let (_singleton, status) = Singleton::acquire()?;
    if status == control::SingletonStatus::AlreadyRunning {
        println!("honk300: a goose is already running. Use `honk300 stop` to stop it.");
        return Ok(());
    }
    eprintln!(
        "honk300: the desktop overlay is Windows-only for now \
         (the macOS and Linux backends land in milestones M16/M17)."
    );
    Ok(())
}
