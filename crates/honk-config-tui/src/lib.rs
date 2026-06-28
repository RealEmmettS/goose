//! Terminal config editor for honk300.

pub mod app;
mod terminal;
pub mod ui;

use app::{Action, AppState, CommandResult, TuiCommand};
use color_eyre::eyre::Result;
use crossterm::event::{self, Event, KeyEvent, KeyEventKind};
use honk_config::Config;
use honk_control::{send_command, ControlCommand, ControlResponse};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::Duration;
use tokio::sync::mpsc;

pub fn run(config_path: PathBuf) -> Result<()> {
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_time()
        .build()?;
    runtime.block_on(run_async(config_path))
}

async fn run_async(config_path: PathBuf) -> Result<()> {
    terminal::install_panic_hook()?;
    let loaded = Config::load_or_default(Some(config_path))?;
    let mut app = AppState::new(loaded.config, loaded.path);
    if let Some(warning) = loaded.warning {
        app.set_status(format!("using defaults: {warning}"), true);
    }

    let (_guard, mut terminal) = terminal::TerminalGuard::enter(terminal::TerminalOptions {
        alt_screen: true,
        mouse: false,
    })?;

    let mut keys = spawn_key_reader();
    let mut tick = tokio::time::interval(Duration::from_millis(100));

    loop {
        terminal.draw(|frame| ui::render(frame, &app))?;
        if app.should_quit {
            break;
        }

        tokio::select! {
            _ = tick.tick() => {}
            maybe_key = keys.recv() => {
                if let Some(key) = maybe_key {
                    let action = app.resolve_key(key);
                    app.apply(action);
                }
            }
        }

        while let Some(command) = app.take_pending_command() {
            let result = handle_command(&app, command);
            app.apply(Action::CommandResult(result));
        }
    }

    Ok(())
}

fn spawn_key_reader() -> mpsc::UnboundedReceiver<KeyEvent> {
    let (tx, rx) = mpsc::unbounded_channel();
    std::thread::spawn(move || loop {
        match event::read() {
            Ok(Event::Key(key)) if key.kind == KeyEventKind::Press => {
                if tx.send(key).is_err() {
                    break;
                }
            }
            Ok(_) => {}
            Err(_) => break,
        }
    });
    rx
}

fn handle_command(app: &AppState, command: TuiCommand) -> CommandResult {
    match command {
        TuiCommand::Save => match app
            .config
            .validate()
            .and_then(|_| app.config.save_atomic(&app.path))
        {
            Ok(()) => match send_command(ControlCommand::Reload) {
                Ok(ControlResponse::Ok) => result("saved; reload sent", false, true),
                Ok(ControlResponse::Err(code)) => {
                    result(format!("saved; reload rejected: {code}"), true, true)
                }
                Err(_) => result("saved; no running goose to reload", false, true),
            },
            Err(err) => result(format!("save failed: {err}"), true, false),
        },
        TuiCommand::Reload => match send_command(ControlCommand::Reload) {
            Ok(ControlResponse::Ok) => result("reload sent", false, false),
            Ok(ControlResponse::Err(code)) => {
                result(format!("reload rejected: {code}"), true, false)
            }
            Err(err) => result(format!("reload failed: {err}"), true, false),
        },
        TuiCommand::Stop => match send_command(ControlCommand::Stop) {
            Ok(ControlResponse::Ok) => result("stop sent", false, false),
            Ok(ControlResponse::Err(code)) => result(format!("stop rejected: {code}"), true, false),
            Err(err) => result(format!("stop failed: {err}"), true, false),
        },
        TuiCommand::Poke(action) => match send_command(ControlCommand::Do(action)) {
            Ok(ControlResponse::Ok) => result(format!("poke sent: {action:?}"), false, false),
            Ok(ControlResponse::Err(code)) => result(format!("poke rejected: {code}"), true, false),
            Err(err) => result(format!("poke failed: {err}"), true, false),
        },
        TuiCommand::Start => match spawn_start(&app.path) {
            Ok(()) => result("start launched", false, false),
            Err(err) => result(format!("start failed: {err}"), true, false),
        },
    }
}

fn result(status: impl Into<String>, is_error: bool, mark_saved: bool) -> CommandResult {
    CommandResult {
        status: status.into(),
        is_error,
        mark_saved,
    }
}

fn spawn_start(config_path: &Path) -> std::io::Result<()> {
    let exe = std::env::current_exe()?;
    let mut command = build_start_command(exe, config_path);
    command.spawn().map(|_| ())
}

fn build_start_command(exe: PathBuf, config_path: &Path) -> Command {
    let mut command = Command::new(exe);
    command
        .arg("start")
        .arg("--config")
        .arg(config_path)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    apply_detached_flags(&mut command);
    command
}

#[cfg(windows)]
fn apply_detached_flags(command: &mut Command) {
    use std::os::windows::process::CommandExt;
    const DETACHED_PROCESS: u32 = 0x0000_0008;
    const CREATE_NEW_PROCESS_GROUP: u32 = 0x0000_0200;
    command.creation_flags(DETACHED_PROCESS | CREATE_NEW_PROCESS_GROUP);
}

#[cfg(not(windows))]
fn apply_detached_flags(_command: &mut Command) {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn start_command_includes_config_path() {
        let command = build_start_command(
            PathBuf::from("honk300.exe"),
            Path::new("C:/tmp/config.toml"),
        );
        let args: Vec<_> = command
            .get_args()
            .map(|arg| arg.to_string_lossy().to_string())
            .collect();
        assert_eq!(args, vec!["start", "--config", "C:/tmp/config.toml"]);
    }

    #[test]
    fn command_result_can_mark_saved() {
        let r = result("saved", false, true);
        assert!(r.mark_saved);
        assert!(!r.is_error);
    }
}
