use super::Action;
use crossterm::event::{KeyCode, KeyEvent};
use honk_config::Config;
use honk_control::RuntimeStatus;
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
    Status,
    Commands,
    About,
}

impl Category {
    pub const ALL: [Self; 9] = [
        Self::General,
        Self::Behaviors,
        Self::Mischief,
        Self::Schedule,
        Self::Appearance,
        Self::Audio,
        Self::Status,
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
            Self::Status => "Status",
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
    Status,
    Stop,
    Start,
    Poke(PokeAction),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandResult {
    pub status: String,
    pub is_error: bool,
    pub mark_saved: bool,
    pub runtime_status: Option<RuntimeStatus>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RowKind {
    Static,
    Toggle(ToggleField),
    Adjust(AdjustField),
    CycleMood,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Row {
    pub label: String,
    pub value: String,
    pub kind: RowKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToggleField {
    Sound,
    NoMouseSteal,
    NoWindowRide,
    Wayland,
    OnHourHonk,
    MultiMonitorChase,
    CanAttackMouse,
    PatStreak,
    DynamicMoods,
    PerchAndRide,
    CollectWindows,
    CollectNotes,
    CollectMemes,
    QuietHours,
    DndRespect,
    PauseOnFullscreen,
    Seasonal,
    Autumn,
    CalmGoose,
    CustomColors,
    AudioEnabled,
    HonkSound,
    BiteSound,
    MudSound,
    PatSound,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AdjustField {
    FirstWander,
    MinWander,
    MaxWander,
    WalkSpeed,
    RunSpeed,
    ChargeSpeed,
    AccelNormal,
    AccelCharged,
    StepNormal,
    StepCharged,
    StopRadius,
    MudDuration,
    FootmarkLifetime,
    FootmarkShrink,
    MouseSucc,
    QuietStart,
    QuietEnd,
    Color(ColorSlot, ColorChannel),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColorSlot {
    White,
    Orange,
    Outline,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColorChannel {
    Red,
    Green,
    Blue,
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
    pub runtime_status: RuntimeStatus,
    pending_commands: VecDeque<TuiCommand>,
    confirm_quit: bool,
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
            runtime_status: RuntimeStatus::not_running(),
            pending_commands: VecDeque::new(),
            confirm_quit: false,
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
        self.rows().len()
    }

    pub fn resolve_key(&self, key: KeyEvent) -> Action {
        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => Action::Quit,
            KeyCode::Tab => Action::NextCategory,
            KeyCode::BackTab => Action::PrevCategory,
            KeyCode::Char(c @ '1'..='9') => {
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
            KeyCode::Char('u') | KeyCode::Char('U') => Action::Status,
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
        if !matches!(action, Action::Quit | Action::None) {
            self.confirm_quit = false;
        }
        match action {
            Action::None => {}
            Action::Quit => {
                if self.dirty() && !self.confirm_quit {
                    self.confirm_quit = true;
                    self.set_status("unsaved changes; press q again to quit".into(), true);
                } else {
                    self.should_quit = true;
                }
            }
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
            Action::Status => self.pending_commands.push_back(TuiCommand::Status),
            Action::Stop => self.pending_commands.push_back(TuiCommand::Stop),
            Action::Start => self.pending_commands.push_back(TuiCommand::Start),
            Action::Poke(action) => self.pending_commands.push_back(TuiCommand::Poke(action)),
            Action::CommandResult(result) => {
                if result.mark_saved {
                    self.mark_saved();
                }
                if let Some(status) = result.runtime_status {
                    self.runtime_status = status;
                }
                self.set_status(result.status, result.is_error);
            }
        }
    }

    fn set_category(&mut self, category: Category) {
        self.active_category = category;
        self.selected_row = self.selected_row.min(self.row_count().saturating_sub(1));
    }

    fn selected_kind(&self) -> RowKind {
        self.rows()
            .get(self.selected_row)
            .map(|row| row.kind)
            .unwrap_or(RowKind::Static)
    }

    fn toggle_selected(&mut self) {
        match self.selected_kind() {
            RowKind::Toggle(field) => self.toggle_field(field),
            RowKind::CycleMood => self.cycle_mood(1),
            RowKind::Adjust(_) | RowKind::Static => {}
        }
    }

    fn adjust_selected(&mut self, delta: i8) {
        match self.selected_kind() {
            RowKind::Adjust(field) => self.adjust_field(field, delta),
            RowKind::CycleMood => self.cycle_mood(delta),
            RowKind::Toggle(_) | RowKind::Static => {}
        }
    }

    fn toggle_field(&mut self, field: ToggleField) {
        match field {
            ToggleField::Sound | ToggleField::AudioEnabled => {
                self.config.audio.enabled = !self.config.audio.enabled
            }
            ToggleField::NoMouseSteal => {
                self.config.safety.no_mouse_steal = !self.config.safety.no_mouse_steal
            }
            ToggleField::NoWindowRide => {
                self.config.safety.no_window_ride = !self.config.safety.no_window_ride
            }
            ToggleField::Wayland => self.config.platform.wayland = !self.config.platform.wayland,
            ToggleField::OnHourHonk => {
                self.config.behaviors.on_hour_double_honk =
                    !self.config.behaviors.on_hour_double_honk
            }
            ToggleField::MultiMonitorChase => {
                self.config.behaviors.multi_monitor_chase =
                    !self.config.behaviors.multi_monitor_chase
            }
            ToggleField::CanAttackMouse => {
                self.config.behavior.can_attack_mouse = !self.config.behavior.can_attack_mouse
            }
            ToggleField::PatStreak => {
                self.config.interaction.pat_streak = !self.config.interaction.pat_streak
            }
            ToggleField::DynamicMoods => {
                self.config.moods.dynamic_moods = !self.config.moods.dynamic_moods
            }
            ToggleField::PerchAndRide => {
                self.config.mischief.perch_and_ride = !self.config.mischief.perch_and_ride
            }
            ToggleField::CollectWindows => {
                self.config.mischief.collect_windows = !self.config.mischief.collect_windows
            }
            ToggleField::CollectNotes => {
                self.config.mischief.collect_notes = !self.config.mischief.collect_notes
            }
            ToggleField::CollectMemes => {
                self.config.mischief.collect_memes = !self.config.mischief.collect_memes
            }
            ToggleField::QuietHours => {
                self.config.schedule.quiet_hours_enabled = !self.config.schedule.quiet_hours_enabled
            }
            ToggleField::DndRespect => {
                self.config.schedule.dnd_respect = !self.config.schedule.dnd_respect
            }
            ToggleField::PauseOnFullscreen => {
                self.config.safety.pause_on_fullscreen = !self.config.safety.pause_on_fullscreen
            }
            ToggleField::Seasonal => self.config.schedule.seasonal = !self.config.schedule.seasonal,
            ToggleField::Autumn => self.config.schedule.autumn = !self.config.schedule.autumn,
            ToggleField::CalmGoose => {
                self.config.appearance.calm_goose = !self.config.appearance.calm_goose
            }
            ToggleField::CustomColors => {
                self.config.behavior.use_custom_colors = !self.config.behavior.use_custom_colors
            }
            ToggleField::HonkSound => self.config.audio.honk = !self.config.audio.honk,
            ToggleField::BiteSound => self.config.audio.bite = !self.config.audio.bite,
            ToggleField::MudSound => self.config.audio.mud = !self.config.audio.mud,
            ToggleField::PatSound => self.config.audio.pat = !self.config.audio.pat,
        }
    }

    fn adjust_field(&mut self, field: AdjustField, delta: i8) {
        let delta_f = delta as f32;
        match field {
            AdjustField::FirstWander => {
                self.config.behavior.first_wander_time_seconds = clamp(
                    self.config.behavior.first_wander_time_seconds + delta_f,
                    1.0,
                    300.0,
                );
            }
            AdjustField::MinWander => {
                self.config.behavior.min_wandering_time_seconds = clamp(
                    self.config.behavior.min_wandering_time_seconds + delta_f,
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
            AdjustField::MaxWander => {
                self.config.behavior.max_wandering_time_seconds = clamp(
                    self.config.behavior.max_wandering_time_seconds + delta_f,
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
            AdjustField::WalkSpeed => {
                self.config.speeds.walk_speed =
                    clamp(self.config.speeds.walk_speed + delta_f * 5.0, 10.0, 800.0);
            }
            AdjustField::RunSpeed => {
                self.config.speeds.run_speed =
                    clamp(self.config.speeds.run_speed + delta_f * 5.0, 10.0, 1000.0);
            }
            AdjustField::ChargeSpeed => {
                self.config.speeds.charge_speed = clamp(
                    self.config.speeds.charge_speed + delta_f * 10.0,
                    10.0,
                    1600.0,
                );
            }
            AdjustField::AccelNormal => {
                self.config.speeds.acceleration_normal = clamp(
                    self.config.speeds.acceleration_normal + delta_f * 50.0,
                    50.0,
                    8000.0,
                );
            }
            AdjustField::AccelCharged => {
                self.config.speeds.acceleration_charged = clamp(
                    self.config.speeds.acceleration_charged + delta_f * 50.0,
                    50.0,
                    10000.0,
                );
            }
            AdjustField::StepNormal => {
                self.config.speeds.step_time_normal = clamp(
                    self.config.speeds.step_time_normal + delta_f * 0.01,
                    0.02,
                    1.0,
                );
            }
            AdjustField::StepCharged => {
                self.config.speeds.step_time_charged = clamp(
                    self.config.speeds.step_time_charged + delta_f * 0.01,
                    0.02,
                    1.0,
                );
            }
            AdjustField::StopRadius => {
                self.config.speeds.stop_radius =
                    clamp(self.config.speeds.stop_radius + delta_f, -100.0, 100.0);
            }
            AdjustField::MudDuration => {
                self.config.mud.duration_to_track_seconds = clamp(
                    self.config.mud.duration_to_track_seconds + delta_f,
                    0.5,
                    120.0,
                );
            }
            AdjustField::FootmarkLifetime => {
                self.config.mud.footmark_lifetime_seconds = clamp(
                    self.config.mud.footmark_lifetime_seconds + delta_f * 0.25,
                    0.25,
                    60.0,
                );
                if self.config.mud.footmark_shrink_seconds
                    > self.config.mud.footmark_lifetime_seconds
                {
                    self.config.mud.footmark_shrink_seconds =
                        self.config.mud.footmark_lifetime_seconds;
                }
            }
            AdjustField::FootmarkShrink => {
                self.config.mud.footmark_shrink_seconds = clamp(
                    self.config.mud.footmark_shrink_seconds + delta_f * 0.25,
                    0.05,
                    self.config.mud.footmark_lifetime_seconds,
                );
            }
            AdjustField::MouseSucc => {
                self.config.mouse.succ_time =
                    clamp(self.config.mouse.succ_time + delta_f * 0.25, 0.25, 10.0);
            }
            AdjustField::QuietStart => {
                self.config.schedule.quiet_start =
                    adjust_time_15(&self.config.schedule.quiet_start, delta);
            }
            AdjustField::QuietEnd => {
                self.config.schedule.quiet_end =
                    adjust_time_15(&self.config.schedule.quiet_end, delta);
            }
            AdjustField::Color(slot, channel) => self.adjust_color_channel(slot, channel, delta),
        }
    }

    fn adjust_color_channel(&mut self, slot: ColorSlot, channel: ColorChannel, delta: i8) {
        let color = match slot {
            ColorSlot::White => &mut self.config.colors.goose_white,
            ColorSlot::Orange => &mut self.config.colors.goose_orange,
            ColorSlot::Outline => &mut self.config.colors.goose_outline,
        };
        let next = adjust_color_channel(color.as_str(), channel, delta);
        *color = next;
    }

    fn cycle_mood(&mut self, delta: i8) {
        const VALUES: [&str; 3] = ["calm", "normal", "spicy"];
        let current = VALUES
            .iter()
            .position(|v| *v == self.config.moods.mood_intensity)
            .unwrap_or(1) as i8;
        let next = (current + delta).rem_euclid(VALUES.len() as i8) as usize;
        self.config.moods.mood_intensity = VALUES[next].into();
    }

    pub fn rows(&self) -> Vec<Row> {
        match self.active_category {
            Category::General => vec![
                row(
                    "Sound",
                    on_off(self.config.audio.enabled),
                    RowKind::Toggle(ToggleField::Sound),
                ),
                row(
                    "No mouse steal",
                    on_off(self.config.safety.no_mouse_steal),
                    RowKind::Toggle(ToggleField::NoMouseSteal),
                ),
                row(
                    "No window ride",
                    on_off(self.config.safety.no_window_ride),
                    RowKind::Toggle(ToggleField::NoWindowRide),
                ),
                row(
                    "On-hour honk",
                    on_off(self.config.behaviors.on_hour_double_honk),
                    RowKind::Toggle(ToggleField::OnHourHonk),
                ),
                row(
                    "Multi-monitor chase",
                    restart_required(self.config.behaviors.multi_monitor_chase),
                    RowKind::Toggle(ToggleField::MultiMonitorChase),
                ),
                row(
                    "Wayland backend",
                    planned(self.config.platform.wayland),
                    RowKind::Toggle(ToggleField::Wayland),
                ),
            ],
            Category::Behaviors => vec![
                row(
                    "Can attack mouse",
                    on_off(self.config.behavior.can_attack_mouse),
                    RowKind::Toggle(ToggleField::CanAttackMouse),
                ),
                row(
                    "Pat streak",
                    on_off(self.config.interaction.pat_streak),
                    RowKind::Toggle(ToggleField::PatStreak),
                ),
                row(
                    "Dynamic moods",
                    on_off(self.config.moods.dynamic_moods),
                    RowKind::Toggle(ToggleField::DynamicMoods),
                ),
                row(
                    "Mood intensity",
                    self.config.moods.mood_intensity.clone(),
                    RowKind::CycleMood,
                ),
                row(
                    "First wander time",
                    seconds(self.config.behavior.first_wander_time_seconds),
                    RowKind::Adjust(AdjustField::FirstWander),
                ),
                row(
                    "Min wandering time",
                    seconds(self.config.behavior.min_wandering_time_seconds),
                    RowKind::Adjust(AdjustField::MinWander),
                ),
                row(
                    "Max wandering time",
                    seconds(self.config.behavior.max_wandering_time_seconds),
                    RowKind::Adjust(AdjustField::MaxWander),
                ),
                row(
                    "Walk speed",
                    number(self.config.speeds.walk_speed),
                    RowKind::Adjust(AdjustField::WalkSpeed),
                ),
                row(
                    "Run speed",
                    number(self.config.speeds.run_speed),
                    RowKind::Adjust(AdjustField::RunSpeed),
                ),
                row(
                    "Charge speed",
                    number(self.config.speeds.charge_speed),
                    RowKind::Adjust(AdjustField::ChargeSpeed),
                ),
                row(
                    "Normal accel",
                    number(self.config.speeds.acceleration_normal),
                    RowKind::Adjust(AdjustField::AccelNormal),
                ),
                row(
                    "Charged accel",
                    number(self.config.speeds.acceleration_charged),
                    RowKind::Adjust(AdjustField::AccelCharged),
                ),
                row(
                    "Normal step",
                    seconds(self.config.speeds.step_time_normal),
                    RowKind::Adjust(AdjustField::StepNormal),
                ),
                row(
                    "Charged step",
                    seconds(self.config.speeds.step_time_charged),
                    RowKind::Adjust(AdjustField::StepCharged),
                ),
                row(
                    "Stop radius",
                    number(self.config.speeds.stop_radius),
                    RowKind::Adjust(AdjustField::StopRadius),
                ),
                row(
                    "Mouse succ time",
                    seconds(self.config.mouse.succ_time),
                    RowKind::Adjust(AdjustField::MouseSucc),
                ),
                row(
                    "Mud duration",
                    seconds(self.config.mud.duration_to_track_seconds),
                    RowKind::Adjust(AdjustField::MudDuration),
                ),
                row(
                    "Footmark life",
                    seconds(self.config.mud.footmark_lifetime_seconds),
                    RowKind::Adjust(AdjustField::FootmarkLifetime),
                ),
                row(
                    "Footmark shrink",
                    seconds(self.config.mud.footmark_shrink_seconds),
                    RowKind::Adjust(AdjustField::FootmarkShrink),
                ),
            ],
            Category::Mischief => vec![
                row(
                    "Perch and ride",
                    on_off(self.config.mischief.perch_and_ride),
                    RowKind::Toggle(ToggleField::PerchAndRide),
                ),
                row(
                    "Collect windows",
                    on_off(self.config.mischief.collect_windows),
                    RowKind::Toggle(ToggleField::CollectWindows),
                ),
                row(
                    "Collect notes",
                    on_off(self.config.mischief.collect_notes),
                    RowKind::Toggle(ToggleField::CollectNotes),
                ),
                row(
                    "Collect memes",
                    on_off(self.config.mischief.collect_memes),
                    RowKind::Toggle(ToggleField::CollectMemes),
                ),
                row("Terminal protection", "always on".into(), RowKind::Static),
            ],
            Category::Schedule => vec![
                row(
                    "Quiet hours",
                    on_off(self.config.schedule.quiet_hours_enabled),
                    RowKind::Toggle(ToggleField::QuietHours),
                ),
                row(
                    "Quiet start",
                    self.config.schedule.quiet_start.clone(),
                    RowKind::Adjust(AdjustField::QuietStart),
                ),
                row(
                    "Quiet end",
                    self.config.schedule.quiet_end.clone(),
                    RowKind::Adjust(AdjustField::QuietEnd),
                ),
                row(
                    "DND respect",
                    on_off(self.config.schedule.dnd_respect),
                    RowKind::Toggle(ToggleField::DndRespect),
                ),
                row(
                    "Pause fullscreen",
                    on_off(self.config.safety.pause_on_fullscreen),
                    RowKind::Toggle(ToggleField::PauseOnFullscreen),
                ),
                row(
                    "Seasonal",
                    on_off(self.config.schedule.seasonal),
                    RowKind::Toggle(ToggleField::Seasonal),
                ),
                row(
                    "Autumn",
                    on_off(self.config.schedule.autumn),
                    RowKind::Toggle(ToggleField::Autumn),
                ),
            ],
            Category::Appearance => vec![
                row(
                    "Calm goose",
                    on_off(self.config.appearance.calm_goose),
                    RowKind::Toggle(ToggleField::CalmGoose),
                ),
                row(
                    "Custom colors",
                    on_off(self.config.behavior.use_custom_colors),
                    RowKind::Toggle(ToggleField::CustomColors),
                ),
                color_row(
                    "White R",
                    &self.config.colors.goose_white,
                    ColorSlot::White,
                    ColorChannel::Red,
                ),
                color_row(
                    "White G",
                    &self.config.colors.goose_white,
                    ColorSlot::White,
                    ColorChannel::Green,
                ),
                color_row(
                    "White B",
                    &self.config.colors.goose_white,
                    ColorSlot::White,
                    ColorChannel::Blue,
                ),
                color_row(
                    "Orange R",
                    &self.config.colors.goose_orange,
                    ColorSlot::Orange,
                    ColorChannel::Red,
                ),
                color_row(
                    "Orange G",
                    &self.config.colors.goose_orange,
                    ColorSlot::Orange,
                    ColorChannel::Green,
                ),
                color_row(
                    "Orange B",
                    &self.config.colors.goose_orange,
                    ColorSlot::Orange,
                    ColorChannel::Blue,
                ),
                color_row(
                    "Outline R",
                    &self.config.colors.goose_outline,
                    ColorSlot::Outline,
                    ColorChannel::Red,
                ),
                color_row(
                    "Outline G",
                    &self.config.colors.goose_outline,
                    ColorSlot::Outline,
                    ColorChannel::Green,
                ),
                color_row(
                    "Outline B",
                    &self.config.colors.goose_outline,
                    ColorSlot::Outline,
                    ColorChannel::Blue,
                ),
            ],
            Category::Audio => vec![
                row(
                    "Audio enabled",
                    on_off(self.config.audio.enabled),
                    RowKind::Toggle(ToggleField::AudioEnabled),
                ),
                row(
                    "Honk sound",
                    on_off(self.config.audio.honk),
                    RowKind::Toggle(ToggleField::HonkSound),
                ),
                row(
                    "Bite sound",
                    on_off(self.config.audio.bite),
                    RowKind::Toggle(ToggleField::BiteSound),
                ),
                row(
                    "Mud sound",
                    on_off(self.config.audio.mud),
                    RowKind::Toggle(ToggleField::MudSound),
                ),
                row(
                    "Pat sound",
                    on_off(self.config.audio.pat),
                    RowKind::Toggle(ToggleField::PatSound),
                ),
            ],
            Category::Status => vec![
                row(
                    "Running",
                    if self.runtime_status.running {
                        "yes".into()
                    } else {
                        "no".into()
                    },
                    RowKind::Static,
                ),
                row(
                    "Platform",
                    self.runtime_status.platform.label().into(),
                    RowKind::Static,
                ),
                row(
                    "Bundle",
                    self.runtime_status.bundle.label().into(),
                    RowKind::Static,
                ),
                row(
                    "Accessibility",
                    self.runtime_status.accessibility.label().into(),
                    RowKind::Static,
                ),
                row(
                    "Cursor",
                    self.runtime_status.cursor.label().into(),
                    RowKind::Static,
                ),
                row(
                    "Window ride",
                    self.runtime_status.window.label().into(),
                    RowKind::Static,
                ),
                row(
                    "Collect windows",
                    self.runtime_status.collect.label().into(),
                    RowKind::Static,
                ),
                row(
                    "Presence",
                    self.runtime_status.presence.label().into(),
                    RowKind::Static,
                ),
                row(
                    "Audio",
                    self.runtime_status.audio.label().into(),
                    RowKind::Static,
                ),
                row(
                    "Assets",
                    format!(
                        "{} notes, {} memes",
                        self.runtime_status.notes, self.runtime_status.memes
                    ),
                    RowKind::Static,
                ),
            ],
            Category::Commands => vec![
                row("honk300 / honk / goose", "start".into(), RowKind::Static),
                row("plz", "start".into(), RowKind::Static),
                row("stop / bad / no / no honk", "stop".into(), RowKind::Static),
                row("reload", "reload config".into(), RowKind::Static),
                row("status", "show runtime status".into(), RowKind::Static),
                row("do honk", "poke honk".into(), RowKind::Static),
                row(
                    "do wander|mud|meme|note|nab",
                    "poke action".into(),
                    RowKind::Static,
                ),
                row("config", "open this TUI".into(), RowKind::Static),
                row(
                    "install/update/uninstall/setup",
                    "M19".into(),
                    RowKind::Static,
                ),
            ],
            Category::About => vec![
                row("honk300", "Desktop Goose in Rust".into(), RowKind::Static),
                row("Config", self.path.display().to_string(), RowKind::Static),
                row(
                    "Control",
                    "CLI/TUI only over local IPC".into(),
                    RowKind::Static,
                ),
                row(
                    "Terminal protection",
                    "not configurable".into(),
                    RowKind::Static,
                ),
                row("M16", "macOS backend + status".into(), RowKind::Static),
            ],
        }
    }
}

fn row(label: &str, value: String, kind: RowKind) -> Row {
    Row {
        label: label.into(),
        value,
        kind,
    }
}

fn color_row(label: &str, color: &str, slot: ColorSlot, channel: ColorChannel) -> Row {
    row(
        label,
        color_channel_value(color, channel),
        RowKind::Adjust(AdjustField::Color(slot, channel)),
    )
}

fn on_off(v: bool) -> String {
    if v { "on" } else { "off" }.into()
}

fn planned(v: bool) -> String {
    format!("{} (planned)", if v { "on" } else { "off" })
}

fn restart_required(v: bool) -> String {
    format!("{} (restart)", if v { "on" } else { "off" })
}

fn seconds(v: f32) -> String {
    format!("{v:.2}s")
}

fn number(v: f32) -> String {
    format!("{v:.2}")
}

fn clamp(value: f32, min: f32, max: f32) -> f32 {
    value.max(min).min(max)
}

fn adjust_time_15(value: &str, delta: i8) -> String {
    let Some((hour, minute)) = value.split_once(':') else {
        return "00:00".into();
    };
    let Ok(hour) = hour.parse::<i32>() else {
        return "00:00".into();
    };
    let Ok(minute) = minute.parse::<i32>() else {
        return "00:00".into();
    };
    let total = (hour * 60 + minute + delta as i32 * 15).rem_euclid(24 * 60);
    format!("{:02}:{:02}", total / 60, total % 60)
}

fn adjust_color_channel(value: &str, channel: ColorChannel, delta: i8) -> String {
    let Some((r, g, b)) = parse_hex(value) else {
        return "#ffffff".into();
    };
    let step = delta as i16 * 0x11;
    let (r, g, b) = match channel {
        ColorChannel::Red => (clamp_channel(r as i16 + step), g, b),
        ColorChannel::Green => (r, clamp_channel(g as i16 + step), b),
        ColorChannel::Blue => (r, g, clamp_channel(b as i16 + step)),
    };
    format!("#{r:02x}{g:02x}{b:02x}")
}

fn color_channel_value(value: &str, channel: ColorChannel) -> String {
    let Some((r, g, b)) = parse_hex(value) else {
        return "invalid".into();
    };
    let value = match channel {
        ColorChannel::Red => r,
        ColorChannel::Green => g,
        ColorChannel::Blue => b,
    };
    format!("{value:03}")
}

fn parse_hex(value: &str) -> Option<(u8, u8, u8)> {
    let hex = value.strip_prefix('#')?;
    if hex.len() != 6 || !hex.bytes().all(|b| b.is_ascii_hexdigit()) {
        return None;
    }
    Some((
        u8::from_str_radix(&hex[0..2], 16).ok()?,
        u8::from_str_radix(&hex[2..4], 16).ok()?,
        u8::from_str_radix(&hex[4..6], 16).ok()?,
    ))
}

fn clamp_channel(value: i16) -> u8 {
    value.clamp(0, 255) as u8
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
    fn dynamic_rows_match_active_category() {
        let mut app = app();
        assert_eq!(app.row_count(), app.rows().len());
        app.apply(Action::SelectCategory(Category::Behaviors));
        assert_eq!(app.row_count(), app.rows().len());
        assert!(app.rows().iter().any(|row| row.label == "Dynamic moods"));
        assert!(app.rows().iter().any(|row| row.label == "Walk speed"));
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
        app.selected_row = app
            .rows()
            .iter()
            .position(|row| row.label == "First wander time")
            .unwrap();
        app.apply(Action::Adjust(1));
        assert_eq!(app.config.behavior.first_wander_time_seconds, 21.0);
    }

    #[test]
    fn quiet_time_adjusts_in_fifteen_minute_steps() {
        let mut app = app();
        app.apply(Action::SelectCategory(Category::Schedule));
        app.selected_row = app
            .rows()
            .iter()
            .position(|row| row.label == "Quiet start")
            .unwrap();
        app.apply(Action::Adjust(1));
        assert_eq!(app.config.schedule.quiet_start, "22:15");
        app.apply(Action::Adjust(-1));
        assert_eq!(app.config.schedule.quiet_start, "22:00");
    }

    #[test]
    fn schedule_rows_are_live_m14_controls() {
        let mut app = app();
        app.apply(Action::SelectCategory(Category::Schedule));
        let rows = app.rows();
        for label in [
            "Quiet hours",
            "DND respect",
            "Pause fullscreen",
            "Seasonal",
            "Autumn",
        ] {
            let row = rows.iter().find(|row| row.label == label).unwrap();
            assert!(
                !row.value.contains("planned"),
                "{label} should be a live M14 row"
            );
        }

        app.selected_row = rows
            .iter()
            .position(|row| row.label == "Pause fullscreen")
            .unwrap();
        assert!(app.config.safety.pause_on_fullscreen);
        app.apply(Action::Toggle);
        assert!(!app.config.safety.pause_on_fullscreen);
    }

    #[test]
    fn general_marks_multi_monitor_chase_restart_required() {
        let mut app = app();
        app.apply(Action::SelectCategory(Category::General));
        let rows = app.rows();
        let row = rows
            .iter()
            .find(|row| row.label == "Multi-monitor chase")
            .unwrap();
        assert_eq!(row.value, "on (restart)");

        app.selected_row = rows
            .iter()
            .position(|row| row.label == "Multi-monitor chase")
            .unwrap();
        app.apply(Action::Toggle);
        assert!(!app.config.behaviors.multi_monitor_chase);
    }

    #[test]
    fn appearance_rows_are_live_m15_controls_with_rgb_channel_editing() {
        let mut app = app();
        app.config.colors.goose_white = "#102030".into();
        app.apply(Action::SelectCategory(Category::Appearance));
        let rows = app.rows();
        assert_eq!(
            rows.iter()
                .find(|row| row.label == "Calm goose")
                .unwrap()
                .value,
            "off"
        );
        for label in ["White R", "White G", "White B", "Orange R", "Outline B"] {
            assert!(rows.iter().any(|row| row.label == label), "{label}");
        }

        app.selected_row = rows
            .iter()
            .position(|row| row.label == "Calm goose")
            .unwrap();
        app.apply(Action::Toggle);
        assert!(app.config.appearance.calm_goose);

        app.selected_row = app
            .rows()
            .iter()
            .position(|row| row.label == "White G")
            .unwrap();
        app.apply(Action::Adjust(1));
        assert_eq!(app.config.colors.goose_white, "#103130");
    }

    #[test]
    fn mood_intensity_cycles() {
        let mut app = app();
        app.apply(Action::SelectCategory(Category::Behaviors));
        app.selected_row = app
            .rows()
            .iter()
            .position(|row| row.label == "Mood intensity")
            .unwrap();
        app.apply(Action::Adjust(1));
        assert_eq!(app.config.moods.mood_intensity, "spicy");
        app.apply(Action::Adjust(1));
        assert_eq!(app.config.moods.mood_intensity, "calm");
    }

    #[test]
    fn dirty_quit_requires_confirmation() {
        let mut app = app();
        app.apply(Action::Toggle);
        app.apply(Action::Quit);
        assert!(!app.should_quit);
        assert!(app.status_is_error);
        app.apply(Action::Quit);
        assert!(app.should_quit);
    }

    #[test]
    fn command_result_updates_status_through_reducer() {
        let mut app = app();
        app.apply(Action::Toggle);
        app.apply(Action::CommandResult(CommandResult {
            status: "saved".into(),
            is_error: false,
            mark_saved: true,
            runtime_status: None,
        }));
        assert_eq!(app.status, "saved");
        assert!(!app.status_is_error);
        assert!(!app.dirty());
    }

    #[test]
    fn save_and_poke_generate_commands() {
        let mut app = app();
        app.apply(Action::Save);
        assert_eq!(app.take_pending_command(), Some(TuiCommand::Save));
        app.apply(Action::Status);
        assert_eq!(app.take_pending_command(), Some(TuiCommand::Status));
        app.apply(Action::Poke(PokeAction::Honk));
        assert_eq!(
            app.take_pending_command(),
            Some(TuiCommand::Poke(PokeAction::Honk))
        );
    }

    #[test]
    fn status_rows_show_runtime_capabilities() {
        let mut app = app();
        app.apply(Action::SelectCategory(Category::Status));
        let rows = app.rows();
        for label in [
            "Running",
            "Platform",
            "Bundle",
            "Accessibility",
            "Cursor",
            "Window ride",
            "Collect windows",
            "Presence",
            "Audio",
            "Assets",
        ] {
            assert!(rows.iter().any(|row| row.label == label), "{label}");
        }
    }
}
