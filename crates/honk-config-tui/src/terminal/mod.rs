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
    active: bool,
    raw_mode_enabled: bool,
    alt_screen_entered: bool,
    mouse_enabled: bool,
    cursor_hidden: bool,
}

impl TerminalGuard {
    fn armed() -> Self {
        Self {
            active: true,
            raw_mode_enabled: false,
            alt_screen_entered: false,
            mouse_enabled: false,
            cursor_hidden: false,
        }
    }

    pub fn enter(options: TerminalOptions) -> Result<(Self, Tui)> {
        // Arm restoration before raw mode or any other fallible terminal mutation. Every `?`
        // below then drops this guard and unwinds only the steps that actually completed.
        let mut guard = Self::armed();
        enable_raw_mode()?;
        guard.raw_mode_enabled = true;
        let mut out = io::stdout();
        if options.alt_screen {
            execute!(out, EnterAlternateScreen)?;
            guard.alt_screen_entered = true;
        }
        if options.mouse {
            execute!(out, EnableMouseCapture)?;
            guard.mouse_enabled = true;
        }
        execute!(out, cursor::Hide)?;
        guard.cursor_hidden = true;

        let mut terminal = Terminal::new(CrosstermBackend::new(io::stdout()))?;
        terminal.clear()?;
        Ok((guard, terminal))
    }

    pub fn restore(&mut self) {
        if !self.active {
            return;
        }
        self.active = false;
        let mut out = io::stdout();
        if self.cursor_hidden {
            let _ = execute!(out, cursor::Show);
            self.cursor_hidden = false;
        }
        if self.mouse_enabled {
            let _ = execute!(out, DisableMouseCapture);
            self.mouse_enabled = false;
        }
        if self.alt_screen_entered {
            let _ = execute!(out, LeaveAlternateScreen);
            self.alt_screen_entered = false;
        }
        if self.raw_mode_enabled {
            let _ = disable_raw_mode();
            self.raw_mode_enabled = false;
        }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn restoration_guard_is_armed_before_the_first_fallible_terminal_step() {
        let guard = TerminalGuard::armed();
        assert!(guard.active);
        assert!(!guard.raw_mode_enabled);
        assert!(!guard.alt_screen_entered);
        assert!(!guard.mouse_enabled);
        assert!(!guard.cursor_hidden);
    }
}
