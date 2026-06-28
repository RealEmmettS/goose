use super::Action;
use crossterm::event::{KeyCode, KeyEvent};
use honk_config::Config;
use honk_engine::PokeAction;
use std::collections::VecDeque;
use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Category {
    General,
    Behaviors,
    Mischief,
    Schedule,
    Appearance,
    Audio,
    Commands,
    About,
}

impl Category {
    pub const ALL: [Self; 8] = [
        Self::General,
        Self::Behaviors,
        Self::Mischief,
        Self::Schedule,
        Self::Appearance,
        Self::Audio,
        Self::Commands,
        Self::About,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Self::General => "General",
            Self::Behaviors => "Behaviors",
            Self::Mischief => "Mischief",
            Self::Schedule => "Schedule",
            Self::Appearance => "Appearance",
            Self::Audio => "Audio",
            Self::Commands => "Commands",
            Self::About => "About",
        }
    }

    pub fn next(self) -> Self {
        let idx = Self::ALL.iter().position(|c| *c == self).unwrap_or(0);
        Self::ALL[(idx + 1) % Self::ALL.len()]
    }

    pub fn prev(self) -> Self {
        let idx = Self::ALL.iter().position(|c| *c == self).unwrap_or(0);
        Self::ALL[(idx + Self::ALL.len() - 1) % Self::ALL.len()]
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TuiCommand {
    Save,
    Reload,
    Stop,
    Start,
    Poke(PokeAction),
}

#[derive(Debug, Clone)]
pub struct AppState {
    pub config: Config,
    original: Config,
    pub path: PathBuf,
    pub active_category: Category,
    pub selected_row: usize,
    pub should_quit: bool,
    pub status: String,
    pub status_is_error: bool,
    pending_commands: VecDeque<TuiCommand>,
}

impl AppState {
    pub fn new(config: Config, path: PathBuf) -> Self {
        Self {
            original: config.clone(),
            config,
            path,
            active_category: Category::General,
            selected_row: 0,
            should_quit: false,
            status: "ready".into(),
            status_is_error: false,
            pending_commands: VecDeque::new(),
        }
    }

    pub fn dirty(&self) -> bool {
        self.config != self.original
    }

    pub fn mark_saved(&mut self) {
        self.original = self.config.clone();
    }

    pub fn set_status(&mut self, status: String, is_error: bool) {
        self.status = status;
        self.status_is_error = is_error;
    }

    pub fn take_pending_command(&mut self) -> Option<TuiCommand> {
        self.pending_commands.pop_front()
    }

    pub fn row_count(&self) -> usize {
        match self.active_category {
            Category::General => 4,
            Category::Behaviors => 6,
            Category::Mischief => 5,
            Category::Schedule => 6,
            Category::Appearance => 2,
            Category::Audio => 5,
            Category::Commands => 8,
            Category::About => 5,
        }
    }

    pub fn resolve_key(&self, key: KeyEvent) -> Action {
        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => Action::Quit,
            KeyCode::Tab => Action::NextCategory,
            KeyCode::BackTab => Action::PrevCategory,
            KeyCode::Char(c @ '1'..='8') => {
                let idx = (c as u8 - b'1') as usize;
                Action::SelectCategory(Category::ALL[idx])
            }
            KeyCode::Down | KeyCode::Char('j') => Action::MoveDown,
            KeyCode::Up | KeyCode::Char('k') => Action::MoveUp,
            KeyCode::Enter | KeyCode::Char(' ') => Action::Toggle,
            KeyCode::Right | KeyCode::Char('+') | KeyCode::Char('=') => Action::Adjust(1),
            KeyCode::Left | KeyCode::Char('-') => Action::Adjust(-1),
            KeyCode::Char('s') | KeyCode::Char('S') => Action::Save,
            KeyCode::Char('r') | KeyCode::Char('R') => Action::Reload,
            KeyCode::Char('x') | KeyCode::Char('X') => Action::Stop,
            KeyCode::Char('g') | KeyCode::Char('G') => Action::Start,
            KeyCode::Char('h') => Action::Poke(PokeAction::Honk),
            KeyCode::Char('w') => Action::Poke(PokeAction::Wander),
            KeyCode::Char('m') => Action::Poke(PokeAction::Mud),
            KeyCode::Char('e') => Action::Poke(PokeAction::Meme),
            KeyCode::Char('n') => Action::Poke(PokeAction::Note),
            KeyCode::Char('b') => Action::Poke(PokeAction::Nab),
            _ => Action::None,
        }
    }

    pub fn apply(&mut self, action: Action) {
        match action {
            Action::None => {}
            Action::Quit => self.should_quit = true,
            Action::NextCategory => self.set_category(self.active_category.next()),
            Action::PrevCategory => self.set_category(self.active_category.prev()),
            Action::SelectCategory(category) => self.set_category(category),
            Action::MoveDown => {
                self.selected_row = (self.selected_row + 1).min(self.row_count().saturating_sub(1));
            }
            Action::MoveUp => self.selected_row = self.selected_row.saturating_sub(1),
            Action::Toggle => self.toggle_selected(),
            Action::Adjust(delta) => self.adjust_selected(delta),
            Action::Save => self.pending_commands.push_back(TuiCommand::Save),
            Action::Reload => self.pending_commands.push_back(TuiCommand::Reload),
            Action::Stop => self.pending_commands.push_back(TuiCommand::Stop),
            Action::Start => self.pending_commands.push_back(TuiCommand::Start),
            Action::Poke(action) => self.pending_commands.push_back(TuiCommand::Poke(action)),
        }
    }

    fn set_category(&mut self, category: Category) {
        self.active_category = category;
        self.selected_row = self.selected_row.min(self.row_count().saturating_sub(1));
    }

    fn toggle_selected(&mut self) {
        match (self.active_category, self.selected_row) {
            (Category::General, 0) => self.config.audio.enabled = !self.config.audio.enabled,
            (Category::General, 1) => {
                self.config.safety.no_mouse_steal = !self.config.safety.no_mouse_steal
            }
            (Category::General, 2) => {
                self.config.safety.no_window_ride = !self.config.safety.no_window_ride
            }
            (Category::General, 3) => self.config.platform.wayland = !self.config.platform.wayland,
            (Category::Behaviors, 0) => {
                self.config.behavior.can_attack_mouse = !self.config.behavior.can_attack_mouse
            }
            (Category::Behaviors, 1) => {
                self.config.interaction.pat_streak = !self.config.interaction.pat_streak
            }
            (Category::Mischief, 0) => {
                self.config.mischief.perch_and_ride = !self.config.mischief.perch_and_ride
            }
            (Category::Mischief, 1) => {
                self.config.mischief.collect_windows = !self.config.mischief.collect_windows
            }
            (Category::Mischief, 2) => {
                self.config.mischief.collect_notes = !self.config.mischief.collect_notes
            }
            (Category::Mischief, 3) => {
                self.config.mischief.collect_memes = !self.config.mischief.collect_memes
            }
            (Category::Schedule, 0) => {
                self.config.schedule.quiet_hours_enabled = !self.config.schedule.quiet_hours_enabled
            }
            (Category::Schedule, 3) => {
                self.config.schedule.dnd_respect = !self.config.schedule.dnd_respect
            }
            (Category::Schedule, 4) => {
                self.config.schedule.seasonal = !self.config.schedule.seasonal
            }
            (Category::Schedule, 5) => self.config.schedule.autumn = !self.config.schedule.autumn,
            (Category::Appearance, 0) => {
                self.config.appearance.calm_goose = !self.config.appearance.calm_goose
            }
            (Category::Appearance, 1) => {
                self.config.behavior.use_custom_colors = !self.config.behavior.use_custom_colors
            }
            (Category::Audio, 0) => self.config.audio.enabled = !self.config.audio.enabled,
            (Category::Audio, 1) => self.config.audio.honk = !self.config.audio.honk,
            (Category::Audio, 2) => self.config.audio.bite = !self.config.audio.bite,
            (Category::Audio, 3) => self.config.audio.mud = !self.config.audio.mud,
            (Category::Audio, 4) => self.config.audio.pat = !self.config.audio.pat,
            _ => {}
        }
    }

    fn adjust_selected(&mut self, delta: i8) {
        let delta = delta as f32;
        match (self.active_category, self.selected_row) {
            (Category::Behaviors, 2) => {
                self.config.behavior.first_wander_time_seconds = clamp(
                    self.config.behavior.first_wander_time_seconds + delta,
                    1.0,
                    300.0,
                );
            }
            (Category::Behaviors, 3) => {
                self.config.behavior.min_wandering_time_seconds = clamp(
                    self.config.behavior.min_wandering_time_seconds + delta,
                    1.0,
                    300.0,
                );
                if self.config.behavior.max_wandering_time_seconds
                    < self.config.behavior.min_wandering_time_seconds
                {
                    self.config.behavior.max_wandering_time_seconds =
                        self.config.behavior.min_wandering_time_seconds;
                }
            }
            (Category::Behaviors, 4) => {
                self.config.behavior.max_wandering_time_seconds = clamp(
                    self.config.behavior.max_wandering_time_seconds + delta,
                    1.0,
                    300.0,
                );
                if self.config.behavior.min_wandering_time_seconds
                    > self.config.behavior.max_wandering_time_seconds
                {
                    self.config.behavior.min_wandering_time_seconds =
                        self.config.behavior.max_wandering_time_seconds;
                }
            }
            (Category::Behaviors, 5) => {
                self.config.mouse.succ_time =
                    clamp(self.config.mouse.succ_time + delta * 0.25, 0.25, 10.0);
            }
            _ => {}
        }
    }

    pub fn rows(&self) -> Vec<(String, String)> {
        match self.active_category {
            Category::General => vec![
                row("Sound", on_off(self.config.audio.enabled)),
                row("No mouse steal", on_off(self.config.safety.no_mouse_steal)),
                row("No window ride", on_off(self.config.safety.no_window_ride)),
                row("Wayland backend", planned(self.config.platform.wayland)),
            ],
            Category::Behaviors => vec![
                row(
                    "Can attack mouse",
                    on_off(self.config.behavior.can_attack_mouse),
                ),
                row("Pat streak", on_off(self.config.interaction.pat_streak)),
                row(
                    "First wander time",
                    seconds(self.config.behavior.first_wander_time_seconds),
                ),
                row(
                    "Min wandering time",
                    seconds(self.config.behavior.min_wandering_time_seconds),
                ),
                row(
                    "Max wandering time",
                    seconds(self.config.behavior.max_wandering_time_seconds),
                ),
                row("Mouse succ time", seconds(self.config.mouse.succ_time)),
            ],
            Category::Mischief => vec![
                row(
                    "Perch and ride",
                    on_off(self.config.mischief.perch_and_ride),
                ),
                row(
                    "Collect windows",
                    on_off(self.config.mischief.collect_windows),
                ),
                row("Collect notes", on_off(self.config.mischief.collect_notes)),
                row("Collect memes", on_off(self.config.mischief.collect_memes)),
                row("Terminal protection", "always on".into()),
            ],
            Category::Schedule => vec![
                row(
                    "Quiet hours",
                    planned(self.config.schedule.quiet_hours_enabled),
                ),
                row("Quiet start", self.config.schedule.quiet_start.clone()),
                row("Quiet end", self.config.schedule.quiet_end.clone()),
                row("DND fullscreen", planned(self.config.schedule.dnd_respect)),
                row("Seasonal", planned(self.config.schedule.seasonal)),
                row("Autumn", planned(self.config.schedule.autumn)),
            ],
            Category::Appearance => vec![
                row("Calm goose", planned(self.config.appearance.calm_goose)),
                row(
                    "Custom colors",
                    planned(self.config.behavior.use_custom_colors),
                ),
            ],
            Category::Audio => vec![
                row("Audio enabled", on_off(self.config.audio.enabled)),
                row("Honk sound", on_off(self.config.audio.honk)),
                row("Bite sound", on_off(self.config.audio.bite)),
                row("Mud sound", on_off(self.config.audio.mud)),
                row("Pat sound", on_off(self.config.audio.pat)),
            ],
            Category::Commands => vec![
                row("honk300 / honk / goose", "start".into()),
                row("plz", "start".into()),
                row("stop / bad / no / no honk", "stop".into()),
                row("reload", "reload config".into()),
                row("do honk", "poke honk".into()),
                row("do wander|mud|meme|note|nab", "poke action".into()),
                row("config", "open this TUI".into()),
                row("install/update/uninstall/setup", "M19".into()),
            ],
            Category::About => vec![
                row("honk300", "Desktop Goose in Rust".into()),
                row("Config", self.path.display().to_string()),
                row("Control", "CLI/TUI only over local IPC".into()),
                row("Terminal protection", "not configurable".into()),
                row("Future settings", "persisted, marked planned".into()),
            ],
        }
    }
}

fn row(label: &str, value: String) -> (String, String) {
    (label.into(), value)
}

fn on_off(v: bool) -> String {
    if v { "on" } else { "off" }.into()
}

fn planned(v: bool) -> String {
    format!("{} (planned)", if v { "on" } else { "off" })
}

fn seconds(v: f32) -> String {
    format!("{v:.2}s")
}

fn clamp(value: f32, min: f32, max: f32) -> f32 {
    value.max(min).min(max)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn app() -> AppState {
        AppState::new(Config::default(), PathBuf::from("config.toml"))
    }

    #[test]
    fn moves_between_categories_and_rows() {
        let mut app = app();
        app.apply(Action::NextCategory);
        assert_eq!(app.active_category, Category::Behaviors);
        app.apply(Action::MoveDown);
        assert_eq!(app.selected_row, 1);
        app.apply(Action::PrevCategory);
        assert_eq!(app.active_category, Category::General);
    }

    #[test]
    fn toggles_boolean_setting_and_tracks_dirty() {
        let mut app = app();
        assert!(app.config.audio.enabled);
        app.apply(Action::Toggle);
        assert!(!app.config.audio.enabled);
        assert!(app.dirty());
    }

    #[test]
    fn adjusts_numeric_setting() {
        let mut app = app();
        app.apply(Action::SelectCategory(Category::Behaviors));
        app.selected_row = 2;
        app.apply(Action::Adjust(1));
        assert_eq!(app.config.behavior.first_wander_time_seconds, 21.0);
    }

    #[test]
    fn save_and_poke_generate_commands() {
        let mut app = app();
        app.apply(Action::Save);
        assert_eq!(app.take_pending_command(), Some(TuiCommand::Save));
        app.apply(Action::Poke(PokeAction::Honk));
        assert_eq!(
            app.take_pending_command(),
            Some(TuiCommand::Poke(PokeAction::Honk))
        );
    }
}
