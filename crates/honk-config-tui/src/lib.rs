//! Terminal config editor for honk300.

pub mod app;
mod terminal;
pub mod ui;

use app::{AppState, TuiCommand};
use color_eyre::eyre::Result;
use crossterm::event::{self, Event, KeyEventKind};
use honk_config::Config;
use honk_control::{send_command, ControlCommand, ControlResponse};
use std::path::PathBuf;
use std::process::Command;
use std::time::Duration;

pub fn run(config_path: PathBuf) -> Result<()> {
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

    loop {
        terminal.draw(|frame| ui::render(frame, &app))?;
        if app.should_quit {
            break;
        }
        if event::poll(Duration::from_millis(100))? {
            match event::read()? {
                Event::Key(key) if key.kind == KeyEventKind::Press => {
                    let action = app.resolve_key(key);
                    app.apply(action);
                }
                _ => {}
            }
        }
        while let Some(command) = app.take_pending_command() {
            handle_command(&mut app, command);
        }
    }

    Ok(())
}

fn handle_command(app: &mut AppState, command: TuiCommand) {
    match command {
        TuiCommand::Save => match app
            .config
            .validate()
            .and_then(|_| app.config.save_atomic(&app.path))
        {
            Ok(()) => {
                app.mark_saved();
                match send_command(ControlCommand::Reload) {
                    Ok(ControlResponse::Ok) => app.set_status("saved; reload sent".into(), false),
                    Ok(ControlResponse::Err(code)) => {
                        app.set_status(format!("saved; reload rejected: {code}"), true)
                    }
                    Err(_) => app.set_status("saved; no running goose to reload".into(), false),
                }
            }
            Err(err) => app.set_status(format!("save failed: {err}"), true),
        },
        TuiCommand::Reload => match send_command(ControlCommand::Reload) {
            Ok(ControlResponse::Ok) => app.set_status("reload sent".into(), false),
            Ok(ControlResponse::Err(code)) => {
                app.set_status(format!("reload rejected: {code}"), true)
            }
            Err(err) => app.set_status(format!("reload failed: {err}"), true),
        },
        TuiCommand::Stop => match send_command(ControlCommand::Stop) {
            Ok(ControlResponse::Ok) => app.set_status("stop sent".into(), false),
            Ok(ControlResponse::Err(code)) => {
                app.set_status(format!("stop rejected: {code}"), true)
            }
            Err(err) => app.set_status(format!("stop failed: {err}"), true),
        },
        TuiCommand::Poke(action) => match send_command(ControlCommand::Do(action)) {
            Ok(ControlResponse::Ok) => app.set_status(format!("poke sent: {action:?}"), false),
            Ok(ControlResponse::Err(code)) => {
                app.set_status(format!("poke rejected: {code}"), true)
            }
            Err(err) => app.set_status(format!("poke failed: {err}"), true),
        },
        TuiCommand::Start => match std::env::current_exe()
            .ok()
            .and_then(|exe| Command::new(exe).arg("start").spawn().ok())
        {
            Some(_) => app.set_status("start launched".into(), false),
            None => app.set_status("start failed".into(), true),
        },
    }
}
