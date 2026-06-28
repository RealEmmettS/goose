use std::io::{self, Stdout, Write};

use color_eyre::eyre::Result;
use crossterm::{
    cursor,
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};

pub type Tui = Terminal<CrosstermBackend<Stdout>>;

#[derive(Debug, Clone, Copy)]
pub struct TerminalOptions {
    pub alt_screen: bool,
    pub mouse: bool,
}

pub struct TerminalGuard {
    options: TerminalOptions,
    active: bool,
}

impl TerminalGuard {
    pub fn enter(options: TerminalOptions) -> Result<(Self, Tui)> {
        enable_raw_mode()?;
        let mut out = io::stdout();
        if options.alt_screen {
            execute!(out, EnterAlternateScreen)?;
        }
        if options.mouse {
            execute!(out, EnableMouseCapture)?;
        }
        execute!(out, cursor::Hide)?;

        let mut terminal = Terminal::new(CrosstermBackend::new(io::stdout()))?;
        terminal.clear()?;
        Ok((
            Self {
                options,
                active: true,
            },
            terminal,
        ))
    }

    pub fn restore(&mut self) {
        if !self.active {
            return;
        }
        self.active = false;
        let mut out = io::stdout();
        let _ = execute!(out, cursor::Show);
        if self.options.mouse {
            let _ = execute!(out, DisableMouseCapture);
        }
        if self.options.alt_screen {
            let _ = execute!(out, LeaveAlternateScreen);
        }
        let _ = disable_raw_mode();
        let _ = out.flush();
    }
}

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        self.restore();
    }
}

fn force_restore() {
    let mut out = io::stdout();
    let _ = execute!(out, cursor::Show, DisableMouseCapture, LeaveAlternateScreen);
    let _ = disable_raw_mode();
    let _ = out.flush();
}

pub fn install_panic_hook() -> Result<()> {
    let (panic_hook, eyre_hook) = color_eyre::config::HookBuilder::default().into_hooks();
    eyre_hook.install()?;
    let panic_hook = panic_hook.into_panic_hook();
    std::panic::set_hook(Box::new(move |info| {
        force_restore();
        panic_hook(info);
    }));
    Ok(())
}
