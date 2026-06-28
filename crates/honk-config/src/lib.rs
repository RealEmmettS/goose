//! Versioned TOML configuration for honk300.
//!
//! This crate is intentionally above `honk-engine`: it may know about paths,
//! TOML, and user-facing validation, then converts validated settings into the
//! platform-free option structs consumed by the engine.

use honk_engine::{
    CollectWindowCapabilities, CollectWindowOptions, FootMarkTiming, ForeignWindowOptions,
    HourlyHonkOptions, InteractionOptions, MoodIntensity, MoodOptions, MouseStealOptions,
    ParametersTable, RenderPalette, TimingOptions, WorldOptions,
};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use tempfile::NamedTempFile;
use toml_edit::{value, DocumentMut, Item, Table};

pub const CONFIG_VERSION: u32 = 1;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    pub goose_config_version: u32,
    pub behavior: BehaviorConfig,
    pub colors: ColorConfig,
    pub speeds: SpeedConfig,
    pub mud: MudConfig,
    pub mouse: MouseConfig,
    pub behaviors: FutureBehaviorConfig,
    pub moods: MoodConfig,
    pub mischief: MischiefConfig,
    pub interaction: InteractionConfig,
    pub schedule: ScheduleConfig,
    pub appearance: AppearanceConfig,
    pub audio: AudioConfig,
    pub safety: SafetyConfig,
    pub platform: PlatformConfig,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            goose_config_version: CONFIG_VERSION,
            behavior: BehaviorConfig::default(),
            colors: ColorConfig::default(),
            speeds: SpeedConfig::default(),
            mud: MudConfig::default(),
            mouse: MouseConfig::default(),
            behaviors: FutureBehaviorConfig::default(),
            moods: MoodConfig::default(),
            mischief: MischiefConfig::default(),
            interaction: InteractionConfig::default(),
            schedule: ScheduleConfig::default(),
            appearance: AppearanceConfig::default(),
            audio: AudioConfig::default(),
            safety: SafetyConfig::default(),
            platform: PlatformConfig::default(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct BehaviorConfig {
    pub silence_sounds: bool,
    pub can_attack_mouse: bool,
    pub attack_randomly: bool,
    pub use_custom_colors: bool,
    pub first_wander_time_seconds: f32,
    pub min_wandering_time_seconds: f32,
    pub max_wandering_time_seconds: f32,
}

impl Default for BehaviorConfig {
    fn default() -> Self {
        Self {
            silence_sounds: false,
            can_attack_mouse: true,
            attack_randomly: false,
            use_custom_colors: false,
            first_wander_time_seconds: 20.0,
            min_wandering_time_seconds: 20.0,
            max_wandering_time_seconds: 40.0,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct ColorConfig {
    pub goose_white: String,
    pub goose_orange: String,
    pub goose_outline: String,
}

impl Default for ColorConfig {
    fn default() -> Self {
        Self {
            goose_white: "#ffffff".into(),
            goose_orange: "#ffa500".into(),
            goose_outline: "#d3d3d3".into(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct SpeedConfig {
    pub walk_speed: f32,
    pub run_speed: f32,
    pub charge_speed: f32,
    pub acceleration_normal: f32,
    pub acceleration_charged: f32,
    pub step_time_normal: f32,
    pub step_time_charged: f32,
    pub stop_radius: f32,
}

impl Default for SpeedConfig {
    fn default() -> Self {
        Self {
            walk_speed: 80.0,
            run_speed: 200.0,
            charge_speed: 400.0,
            acceleration_normal: 1300.0,
            acceleration_charged: 2300.0,
            step_time_normal: 0.2,
            step_time_charged: 0.1,
            stop_radius: -10.0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct MudConfig {
    pub duration_to_track_seconds: f32,
    pub footmark_lifetime_seconds: f32,
    pub footmark_shrink_seconds: f32,
}

impl Default for MudConfig {
    fn default() -> Self {
        Self {
            duration_to_track_seconds: 15.0,
            footmark_lifetime_seconds: 8.5,
            footmark_shrink_seconds: 1.0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct MouseConfig {
    pub grab_distance: f32,
    pub drop_distance: f32,
    pub succ_time: f32,
}

impl Default for MouseConfig {
    fn default() -> Self {
        Self {
            grab_distance: 60.0,
            drop_distance: 200.0,
            succ_time: 2.5,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct FutureBehaviorConfig {
    pub on_hour_double_honk: bool,
    pub multi_monitor_chase: bool,
}

impl Default for FutureBehaviorConfig {
    fn default() -> Self {
        Self {
            on_hour_double_honk: true,
            multi_monitor_chase: true,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct MoodConfig {
    pub dynamic_moods: bool,
    pub mood_intensity: String,
}

impl Default for MoodConfig {
    fn default() -> Self {
        Self {
            dynamic_moods: true,
            mood_intensity: "normal".into(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct MischiefConfig {
    pub perch_and_ride: bool,
    pub collect_windows: bool,
    pub collect_notes: bool,
    pub collect_memes: bool,
}

impl Default for MischiefConfig {
    fn default() -> Self {
        Self {
            perch_and_ride: true,
            collect_windows: true,
            collect_notes: true,
            collect_memes: true,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct InteractionConfig {
    pub pat_streak: bool,
}

impl Default for InteractionConfig {
    fn default() -> Self {
        Self { pat_streak: true }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct ScheduleConfig {
    pub quiet_hours_enabled: bool,
    pub quiet_start: String,
    pub quiet_end: String,
    pub dnd_respect: bool,
    pub seasonal: bool,
    pub autumn: bool,
}

impl Default for ScheduleConfig {
    fn default() -> Self {
        Self {
            quiet_hours_enabled: true,
            quiet_start: "22:00".into(),
            quiet_end: "08:00".into(),
            dnd_respect: true,
            seasonal: true,
            autumn: true,
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct AppearanceConfig {
    pub calm_goose: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct AudioConfig {
    pub enabled: bool,
    pub honk: bool,
    pub bite: bool,
    pub mud: bool,
    pub pat: bool,
}

impl Default for AudioConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            honk: true,
            bite: true,
            mud: true,
            pat: true,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct SafetyConfig {
    pub pause_on_fullscreen: bool,
    pub no_mouse_steal: bool,
    pub no_window_ride: bool,
}

impl Default for SafetyConfig {
    fn default() -> Self {
        Self {
            pause_on_fullscreen: true,
            no_mouse_steal: false,
            no_window_ride: false,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct PlatformConfig {
    pub wayland: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct CliOverrides {
    pub no_sound: bool,
    pub no_mouse_steal: bool,
    pub no_window_ride: bool,
    pub wayland: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BackendState {
    pub cursor_warp_supported: bool,
    pub window_watch_supported: bool,
    pub collect_window_supported: bool,
    pub note_count: u32,
    pub meme_count: u32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct EffectiveOptions {
    pub audio: AudioConfig,
    pub no_sound: bool,
    pub no_mouse_steal: bool,
    pub no_window_ride: bool,
    pub wayland: bool,
    pub world: WorldOptions,
}

#[derive(Debug, Clone)]
pub struct LoadedConfig {
    pub path: PathBuf,
    pub config: Config,
    pub warning: Option<String>,
}

#[derive(Debug)]
pub enum ConfigError {
    NoDefaultPath,
    Io(io::Error),
    Parse(toml::de::Error),
    WrongVersion(u32),
    Validation(Vec<String>),
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NoDefaultPath => f.write_str("could not determine a honk300 config path"),
            Self::Io(err) => write!(f, "config I/O error: {err}"),
            Self::Parse(err) => write!(f, "malformed config.toml: {err}"),
            Self::WrongVersion(version) => {
                write!(f, "unsupported goose_config_version {version}")
            }
            Self::Validation(errors) => write!(f, "invalid config: {}", errors.join("; ")),
        }
    }
}

impl std::error::Error for ConfigError {}

impl From<io::Error> for ConfigError {
    fn from(err: io::Error) -> Self {
        Self::Io(err)
    }
}

impl From<toml::de::Error> for ConfigError {
    fn from(err: toml::de::Error) -> Self {
        Self::Parse(err)
    }
}

impl Config {
    pub fn load_or_default(path: Option<PathBuf>) -> Result<LoadedConfig, ConfigError> {
        let path = resolve_path(path)?;
        match Self::load_existing_with_warning(&path) {
            Ok(mut loaded) => {
                loaded.path = path;
                Ok(loaded)
            }
            Err(ConfigError::Io(err)) if err.kind() == io::ErrorKind::NotFound => {
                Ok(LoadedConfig {
                    path,
                    config: Self::default(),
                    warning: None,
                })
            }
            Err(err) => Ok(LoadedConfig {
                path,
                config: Self::default(),
                warning: Some(err.to_string()),
            }),
        }
    }

    pub fn load_existing(path: &Path) -> Result<Self, ConfigError> {
        Self::load_existing_with_warning(path).map(|loaded| loaded.config)
    }

    pub fn load_existing_with_warning(path: &Path) -> Result<LoadedConfig, ConfigError> {
        let text = fs::read_to_string(path)?;
        let warning = unknown_key_warning(&text);
        let config: Self = toml::from_str(&text)?;
        config.validate()?;
        Ok(LoadedConfig {
            path: path.to_path_buf(),
            config,
            warning,
        })
    }

    pub fn validate(&self) -> Result<(), ConfigError> {
        let mut errors = Vec::new();
        if self.goose_config_version != CONFIG_VERSION {
            return Err(ConfigError::WrongVersion(self.goose_config_version));
        }
        positive(
            "behavior.first_wander_time_seconds",
            self.behavior.first_wander_time_seconds,
            &mut errors,
        );
        positive(
            "behavior.min_wandering_time_seconds",
            self.behavior.min_wandering_time_seconds,
            &mut errors,
        );
        positive(
            "behavior.max_wandering_time_seconds",
            self.behavior.max_wandering_time_seconds,
            &mut errors,
        );
        if self.behavior.max_wandering_time_seconds < self.behavior.min_wandering_time_seconds {
            errors.push(
                "behavior.max_wandering_time_seconds must be >= min_wandering_time_seconds".into(),
            );
        }
        positive("mouse.grab_distance", self.mouse.grab_distance, &mut errors);
        positive("mouse.drop_distance", self.mouse.drop_distance, &mut errors);
        positive("mouse.succ_time", self.mouse.succ_time, &mut errors);
        positive("speeds.walk_speed", self.speeds.walk_speed, &mut errors);
        positive("speeds.run_speed", self.speeds.run_speed, &mut errors);
        positive("speeds.charge_speed", self.speeds.charge_speed, &mut errors);
        positive(
            "speeds.acceleration_normal",
            self.speeds.acceleration_normal,
            &mut errors,
        );
        positive(
            "speeds.acceleration_charged",
            self.speeds.acceleration_charged,
            &mut errors,
        );
        positive(
            "speeds.step_time_normal",
            self.speeds.step_time_normal,
            &mut errors,
        );
        positive(
            "speeds.step_time_charged",
            self.speeds.step_time_charged,
            &mut errors,
        );
        finite("speeds.stop_radius", self.speeds.stop_radius, &mut errors);
        positive(
            "mud.duration_to_track_seconds",
            self.mud.duration_to_track_seconds,
            &mut errors,
        );
        positive(
            "mud.footmark_lifetime_seconds",
            self.mud.footmark_lifetime_seconds,
            &mut errors,
        );
        positive(
            "mud.footmark_shrink_seconds",
            self.mud.footmark_shrink_seconds,
            &mut errors,
        );
        if self.mud.footmark_shrink_seconds > self.mud.footmark_lifetime_seconds {
            errors.push("mud.footmark_shrink_seconds must be <= footmark_lifetime_seconds".into());
        }
        validate_hex_color("colors.goose_white", &self.colors.goose_white, &mut errors);
        validate_hex_color(
            "colors.goose_orange",
            &self.colors.goose_orange,
            &mut errors,
        );
        validate_hex_color(
            "colors.goose_outline",
            &self.colors.goose_outline,
            &mut errors,
        );
        validate_time(
            "schedule.quiet_start",
            &self.schedule.quiet_start,
            &mut errors,
        );
        validate_time("schedule.quiet_end", &self.schedule.quiet_end, &mut errors);
        if !matches!(
            self.moods.mood_intensity.as_str(),
            "calm" | "normal" | "spicy"
        ) {
            errors.push("moods.mood_intensity must be calm, normal, or spicy".into());
        }
        if errors.is_empty() {
            Ok(())
        } else {
            Err(ConfigError::Validation(errors))
        }
    }

    pub fn save_atomic(&self, path: &Path) -> Result<(), ConfigError> {
        self.validate()?;
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let mut doc = match fs::read_to_string(path) {
            Ok(text) => text.parse::<DocumentMut>().unwrap_or_default(),
            Err(err) if err.kind() == io::ErrorKind::NotFound => DocumentMut::new(),
            Err(err) => return Err(ConfigError::Io(err)),
        };
        self.write_to_document(&mut doc);
        let parent = path.parent().unwrap_or_else(|| Path::new("."));
        let mut tmp = NamedTempFile::new_in(parent)?;
        tmp.write_all(doc.to_string().as_bytes())?;
        tmp.flush()?;
        tmp.persist(path)
            .map_err(|err| ConfigError::Io(err.error))?;
        Ok(())
    }

    pub fn effective_options(&self, backend: BackendState, cli: CliOverrides) -> EffectiveOptions {
        let no_sound = cli.no_sound || self.behavior.silence_sounds || !self.audio.enabled;
        let no_mouse_steal = cli.no_mouse_steal || self.safety.no_mouse_steal;
        let no_window_ride = cli.no_window_ride || self.safety.no_window_ride;

        let mut foreign_window = ForeignWindowOptions::with_backend_support(
            backend.window_watch_supported,
            !no_window_ride,
        );
        foreign_window.enabled = self.mischief.perch_and_ride && !no_window_ride;

        // Backend capability gates every collect operation: if the runtime reported it can no
        // longer drive collect windows, none of these are available regardless of config. This
        // keeps a runtime capability loss durable across reloads instead of being reset by one.
        let collect_supported = backend.collect_window_supported;
        let collect_capabilities = CollectWindowCapabilities {
            spawn_note: collect_supported,
            spawn_image: collect_supported,
            move_window: collect_supported && self.mischief.collect_windows,
            set_passthrough: collect_supported,
            synthesize_text: collect_supported,
        };
        let mut collect_window = CollectWindowOptions::with_backend_support(
            collect_capabilities,
            backend.note_count,
            backend.meme_count,
        );
        collect_window.enabled = self.mischief.collect_windows;
        collect_window.notes_enabled = self.mischief.collect_notes;
        collect_window.memes_enabled = self.mischief.collect_memes;

        let world = WorldOptions {
            mouse_steal: MouseStealOptions {
                enabled: self.behavior.can_attack_mouse && !no_mouse_steal,
                warp_supported: backend.cursor_warp_supported,
                grab_distance: self.mouse.grab_distance,
                drop_distance: self.mouse.drop_distance,
                succ_time: self.mouse.succ_time,
            },
            foreign_window,
            collect_window,
            interaction: InteractionOptions {
                pat_streak: self.interaction.pat_streak,
            },
            timing: TimingOptions {
                first_wander_time: self.behavior.first_wander_time_seconds,
                min_wandering_time: self.behavior.min_wandering_time_seconds,
                max_wandering_time: self.behavior.max_wandering_time_seconds,
            },
            parameters: ParametersTable {
                walk_speed: self.speeds.walk_speed,
                run_speed: self.speeds.run_speed,
                charge_speed: self.speeds.charge_speed,
                acceleration_normal: self.speeds.acceleration_normal,
                acceleration_charged: self.speeds.acceleration_charged,
                stop_radius: self.speeds.stop_radius,
                step_time_normal: self.speeds.step_time_normal,
                step_time_charged: self.speeds.step_time_charged,
                duration_to_track_mud: self.mud.duration_to_track_seconds,
            },
            footmarks: FootMarkTiming {
                lifetime: self.mud.footmark_lifetime_seconds,
                shrink_time: self.mud.footmark_shrink_seconds,
            },
            palette: if self.behavior.use_custom_colors {
                RenderPalette {
                    goose_white: parse_hex_rgb(&self.colors.goose_white)
                        .unwrap_or(RenderPalette::default().goose_white),
                    goose_orange: parse_hex_rgb(&self.colors.goose_orange)
                        .unwrap_or(RenderPalette::default().goose_orange),
                    goose_outline: parse_hex_rgb(&self.colors.goose_outline)
                        .unwrap_or(RenderPalette::default().goose_outline),
                }
            } else {
                RenderPalette::default()
            },
            mood: MoodOptions {
                dynamic_moods: self.moods.dynamic_moods,
                intensity: parse_mood_intensity(&self.moods.mood_intensity),
            },
            hourly_honk: HourlyHonkOptions {
                on_hour_double_honk: self.behaviors.on_hour_double_honk,
            },
        };

        EffectiveOptions {
            audio: self.audio,
            no_sound,
            no_mouse_steal,
            no_window_ride,
            wayland: cli.wayland || self.platform.wayland,
            world,
        }
    }

    fn write_to_document(&self, doc: &mut DocumentMut) {
        doc["goose_config_version"] = value(self.goose_config_version as i64);
        let behavior = table_mut(doc, "behavior");
        set_bool(behavior, "silence_sounds", self.behavior.silence_sounds);
        set_bool(behavior, "can_attack_mouse", self.behavior.can_attack_mouse);
        set_bool(behavior, "attack_randomly", self.behavior.attack_randomly);
        set_bool(
            behavior,
            "use_custom_colors",
            self.behavior.use_custom_colors,
        );
        set_float(
            behavior,
            "first_wander_time_seconds",
            self.behavior.first_wander_time_seconds,
        );
        set_float(
            behavior,
            "min_wandering_time_seconds",
            self.behavior.min_wandering_time_seconds,
        );
        set_float(
            behavior,
            "max_wandering_time_seconds",
            self.behavior.max_wandering_time_seconds,
        );

        let colors = table_mut(doc, "colors");
        set_str(colors, "goose_white", &self.colors.goose_white);
        set_str(colors, "goose_orange", &self.colors.goose_orange);
        set_str(colors, "goose_outline", &self.colors.goose_outline);

        let speeds = table_mut(doc, "speeds");
        set_float(speeds, "walk_speed", self.speeds.walk_speed);
        set_float(speeds, "run_speed", self.speeds.run_speed);
        set_float(speeds, "charge_speed", self.speeds.charge_speed);
        set_float(
            speeds,
            "acceleration_normal",
            self.speeds.acceleration_normal,
        );
        set_float(
            speeds,
            "acceleration_charged",
            self.speeds.acceleration_charged,
        );
        set_float(speeds, "step_time_normal", self.speeds.step_time_normal);
        set_float(speeds, "step_time_charged", self.speeds.step_time_charged);
        set_float(speeds, "stop_radius", self.speeds.stop_radius);

        let mud = table_mut(doc, "mud");
        set_float(
            mud,
            "duration_to_track_seconds",
            self.mud.duration_to_track_seconds,
        );
        set_float(
            mud,
            "footmark_lifetime_seconds",
            self.mud.footmark_lifetime_seconds,
        );
        set_float(
            mud,
            "footmark_shrink_seconds",
            self.mud.footmark_shrink_seconds,
        );

        let mouse = table_mut(doc, "mouse");
        set_float(mouse, "grab_distance", self.mouse.grab_distance);
        set_float(mouse, "drop_distance", self.mouse.drop_distance);
        set_float(mouse, "succ_time", self.mouse.succ_time);

        let behaviors = table_mut(doc, "behaviors");
        set_bool(
            behaviors,
            "on_hour_double_honk",
            self.behaviors.on_hour_double_honk,
        );
        set_bool(
            behaviors,
            "multi_monitor_chase",
            self.behaviors.multi_monitor_chase,
        );

        let moods = table_mut(doc, "moods");
        set_bool(moods, "dynamic_moods", self.moods.dynamic_moods);
        set_str(moods, "mood_intensity", &self.moods.mood_intensity);

        let mischief = table_mut(doc, "mischief");
        set_bool(mischief, "perch_and_ride", self.mischief.perch_and_ride);
        set_bool(mischief, "collect_windows", self.mischief.collect_windows);
        set_bool(mischief, "collect_notes", self.mischief.collect_notes);
        set_bool(mischief, "collect_memes", self.mischief.collect_memes);

        let interaction = table_mut(doc, "interaction");
        set_bool(interaction, "pat_streak", self.interaction.pat_streak);

        let schedule = table_mut(doc, "schedule");
        set_bool(
            schedule,
            "quiet_hours_enabled",
            self.schedule.quiet_hours_enabled,
        );
        set_str(schedule, "quiet_start", &self.schedule.quiet_start);
        set_str(schedule, "quiet_end", &self.schedule.quiet_end);
        set_bool(schedule, "dnd_respect", self.schedule.dnd_respect);
        set_bool(schedule, "seasonal", self.schedule.seasonal);
        set_bool(schedule, "autumn", self.schedule.autumn);

        let appearance = table_mut(doc, "appearance");
        set_bool(appearance, "calm_goose", self.appearance.calm_goose);

        let audio = table_mut(doc, "audio");
        set_bool(audio, "enabled", self.audio.enabled);
        set_bool(audio, "honk", self.audio.honk);
        set_bool(audio, "bite", self.audio.bite);
        set_bool(audio, "mud", self.audio.mud);
        set_bool(audio, "pat", self.audio.pat);

        let safety = table_mut(doc, "safety");
        set_bool(
            safety,
            "pause_on_fullscreen",
            self.safety.pause_on_fullscreen,
        );
        set_bool(safety, "no_mouse_steal", self.safety.no_mouse_steal);
        set_bool(safety, "no_window_ride", self.safety.no_window_ride);

        let platform = table_mut(doc, "platform");
        set_bool(platform, "wayland", self.platform.wayland);
    }
}

pub fn default_config_path() -> Option<PathBuf> {
    if cfg!(windows) {
        return Some(
            PathBuf::from(std::env::var_os("LOCALAPPDATA")?)
                .join("honk300")
                .join("config.toml"),
        );
    }
    if cfg!(target_os = "macos") {
        return Some(
            PathBuf::from(std::env::var_os("HOME")?)
                .join("Library")
                .join("Application Support")
                .join("honk300")
                .join("config.toml"),
        );
    }
    let base = std::env::var_os("XDG_DATA_HOME")
        .map(PathBuf::from)
        .or_else(|| {
            std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".local/share"))
        })?;
    Some(base.join("honk300").join("config.toml"))
}

pub fn resolve_path(path: Option<PathBuf>) -> Result<PathBuf, ConfigError> {
    match path {
        Some(path) => Ok(path),
        None => default_config_path().ok_or(ConfigError::NoDefaultPath),
    }
}

fn positive(name: &str, value: f32, errors: &mut Vec<String>) {
    if !value.is_finite() || value <= 0.0 {
        errors.push(format!("{name} must be a positive finite number"));
    }
}

fn finite(name: &str, value: f32, errors: &mut Vec<String>) {
    if !value.is_finite() {
        errors.push(format!("{name} must be finite"));
    }
}

fn validate_hex_color(name: &str, value: &str, errors: &mut Vec<String>) {
    if parse_hex_rgb(value).is_none() {
        errors.push(format!("{name} must be a #rrggbb hex color"));
    }
}

fn parse_hex_rgb(value: &str) -> Option<(u8, u8, u8)> {
    let hex = value.strip_prefix('#')?;
    if hex.len() != 6 || !hex.bytes().all(|b| b.is_ascii_hexdigit()) {
        return None;
    }
    let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
    let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
    let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
    Some((r, g, b))
}

fn parse_mood_intensity(value: &str) -> MoodIntensity {
    match value {
        "calm" => MoodIntensity::Calm,
        "spicy" => MoodIntensity::Spicy,
        _ => MoodIntensity::Normal,
    }
}

fn validate_time(name: &str, value: &str, errors: &mut Vec<String>) {
    let Some((hour, minute)) = value.split_once(':') else {
        errors.push(format!("{name} must use HH:MM"));
        return;
    };
    let Ok(hour) = hour.parse::<u8>() else {
        errors.push(format!("{name} hour is invalid"));
        return;
    };
    let Ok(minute) = minute.parse::<u8>() else {
        errors.push(format!("{name} minute is invalid"));
        return;
    };
    if hour > 23 || minute > 59 {
        errors.push(format!("{name} must be within 00:00 through 23:59"));
    }
}

fn unknown_key_warning(text: &str) -> Option<String> {
    let doc = text.parse::<DocumentMut>().ok()?;
    let mut unknown = Vec::new();
    let root_allowed = [
        "goose_config_version",
        "behavior",
        "colors",
        "speeds",
        "mud",
        "mouse",
        "behaviors",
        "moods",
        "mischief",
        "interaction",
        "schedule",
        "appearance",
        "audio",
        "safety",
        "platform",
    ];
    for (key, item) in doc.as_table().iter() {
        if !root_allowed.contains(&key) {
            unknown.push(key.to_string());
            continue;
        }
        let Some(table) = item.as_table() else {
            continue;
        };
        let allowed = known_section_keys(key);
        for (child, _) in table.iter() {
            if !allowed.contains(&child) {
                unknown.push(format!("{key}.{child}"));
            }
        }
    }
    if unknown.is_empty() {
        None
    } else {
        unknown.sort();
        unknown.dedup();
        Some(format!(
            "ignored unknown config key{}: {}",
            if unknown.len() == 1 { "" } else { "s" },
            unknown.join(", ")
        ))
    }
}

fn known_section_keys(section: &str) -> &'static [&'static str] {
    match section {
        "behavior" => &[
            "silence_sounds",
            "can_attack_mouse",
            "attack_randomly",
            "use_custom_colors",
            "first_wander_time_seconds",
            "min_wandering_time_seconds",
            "max_wandering_time_seconds",
        ],
        "colors" => &["goose_white", "goose_orange", "goose_outline"],
        "speeds" => &[
            "walk_speed",
            "run_speed",
            "charge_speed",
            "acceleration_normal",
            "acceleration_charged",
            "step_time_normal",
            "step_time_charged",
            "stop_radius",
        ],
        "mud" => &[
            "duration_to_track_seconds",
            "footmark_lifetime_seconds",
            "footmark_shrink_seconds",
        ],
        "mouse" => &["grab_distance", "drop_distance", "succ_time"],
        "behaviors" => &["on_hour_double_honk", "multi_monitor_chase"],
        "moods" => &["dynamic_moods", "mood_intensity"],
        "mischief" => &[
            "perch_and_ride",
            "collect_windows",
            "collect_notes",
            "collect_memes",
        ],
        "interaction" => &["pat_streak"],
        "schedule" => &[
            "quiet_hours_enabled",
            "quiet_start",
            "quiet_end",
            "dnd_respect",
            "seasonal",
            "autumn",
        ],
        "appearance" => &["calm_goose"],
        "audio" => &["enabled", "honk", "bite", "mud", "pat"],
        "safety" => &["pause_on_fullscreen", "no_mouse_steal", "no_window_ride"],
        "platform" => &["wayland"],
        _ => &[],
    }
}

fn table_mut<'a>(doc: &'a mut DocumentMut, name: &str) -> &'a mut Table {
    if !doc.as_table().contains_key(name) || !doc[name].is_table() {
        doc[name] = Item::Table(Table::new());
    }
    doc[name].as_table_mut().expect("table was just installed")
}

fn set_bool(table: &mut Table, key: &str, v: bool) {
    table[key] = value(v);
}

fn set_float(table: &mut Table, key: &str, v: f32) {
    table[key] = value(v as f64);
}

fn set_str(table: &mut Table, key: &str, v: &str) {
    table[key] = value(v);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn backend() -> BackendState {
        BackendState {
            cursor_warp_supported: true,
            window_watch_supported: true,
            collect_window_supported: true,
            note_count: 1,
            meme_count: 1,
        }
    }

    #[test]
    fn backend_collect_loss_disables_collect_window() {
        // When the backend reports it can no longer spawn/move collect windows, that capability
        // loss must survive into the effective options (and therefore across a reload), even
        // though the user's config still enables collect behavior.
        let mut c = Config::default();
        c.mischief.collect_windows = true;
        c.mischief.collect_notes = true;
        c.mischief.collect_memes = true;

        let mut backend = backend();
        backend.collect_window_supported = false;

        let effective = c.effective_options(backend, CliOverrides::default());
        assert!(
            !effective.world.collect_window.active(),
            "a backend collect-window capability loss must disable collect behavior"
        );
    }

    #[test]
    fn partial_toml_keeps_defaults_for_missing_fields() {
        let c: Config = toml::from_str("[audio]\nenabled = false\n").unwrap();
        assert_eq!(c.goose_config_version, CONFIG_VERSION);
        assert!(!c.audio.enabled);
        assert!(c.audio.honk);
        assert_eq!(c.mouse.grab_distance, 60.0);
    }

    #[test]
    fn unknown_keys_are_ignored_on_load() {
        let c: Config =
            toml::from_str("future_root = true\n[audio]\nfuture = 1\nenabled = false\n").unwrap();
        assert!(!c.audio.enabled);
    }

    #[test]
    fn unknown_keys_warn_on_load() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config.toml");
        fs::write(&path, "future_root = true\n[audio]\nfuture = 1\n").unwrap();
        let loaded = Config::load_existing_with_warning(&path).unwrap();
        let warning = loaded.warning.expect("unknown keys should warn");
        assert!(warning.contains("future_root"));
        assert!(warning.contains("audio.future"));
    }

    #[test]
    fn wrong_version_is_rejected() {
        let c: Config = toml::from_str("goose_config_version = 99\n").unwrap();
        assert!(matches!(c.validate(), Err(ConfigError::WrongVersion(99))));
    }

    #[test]
    fn validation_catches_bad_ranges() {
        let c: Config =
            toml::from_str("goose_config_version = 1\n[mouse]\ngrab_distance = -1.0\n").unwrap();
        assert!(matches!(c.validate(), Err(ConfigError::Validation(_))));
    }

    #[test]
    fn validation_catches_bad_speed_mud_and_color_values() {
        let mut c = Config::default();
        c.speeds.walk_speed = 0.0;
        c.mud.footmark_lifetime_seconds = 1.0;
        c.mud.footmark_shrink_seconds = 2.0;
        c.colors.goose_white = "white".into();
        let Err(ConfigError::Validation(errors)) = c.validate() else {
            panic!("expected validation errors");
        };
        assert!(errors.iter().any(|e| e.contains("speeds.walk_speed")));
        assert!(errors.iter().any(|e| e.contains("footmark_shrink_seconds")));
        assert!(errors.iter().any(|e| e.contains("colors.goose_white")));
    }

    #[test]
    fn effective_options_merge_cli_and_config() {
        let mut c = Config::default();
        c.safety.no_mouse_steal = true;
        c.mischief.collect_memes = false;
        c.mouse.succ_time = 3.5;
        let effective = c.effective_options(backend(), CliOverrides::default());
        assert!(!effective.world.mouse_steal.enabled);
        assert_eq!(effective.world.mouse_steal.succ_time, 3.5);
        assert_eq!(effective.world.parameters.walk_speed, 80.0);
        assert_eq!(effective.world.footmarks.lifetime, 8.5);
        assert_eq!(effective.world.mood.intensity, MoodIntensity::Normal);
        assert!(effective.world.hourly_honk.on_hour_double_honk);
        assert!(!effective
            .world
            .collect_window
            .kind_active(honk_engine::CollectWindowKind::Meme));
    }

    #[test]
    fn effective_options_maps_speed_mud_palette_mood_and_hourly_honk() {
        let mut c = Config::default();
        c.speeds.walk_speed = 91.0;
        c.mud.duration_to_track_seconds = 12.0;
        c.mud.footmark_lifetime_seconds = 6.0;
        c.mud.footmark_shrink_seconds = 2.0;
        c.behavior.use_custom_colors = true;
        c.colors.goose_white = "#ddeeff".into();
        c.colors.goose_orange = "#112233".into();
        c.colors.goose_outline = "#445566".into();
        c.moods.mood_intensity = "spicy".into();
        c.behaviors.on_hour_double_honk = false;

        let effective = c.effective_options(backend(), CliOverrides::default());
        assert_eq!(effective.world.parameters.walk_speed, 91.0);
        assert_eq!(effective.world.parameters.duration_to_track_mud, 12.0);
        assert_eq!(effective.world.footmarks.lifetime, 6.0);
        assert_eq!(effective.world.footmarks.shrink_time, 2.0);
        assert_eq!(effective.world.palette.goose_white, (0xdd, 0xee, 0xff));
        assert_eq!(effective.world.palette.goose_orange, (0x11, 0x22, 0x33));
        assert_eq!(effective.world.palette.goose_outline, (0x44, 0x55, 0x66));
        assert_eq!(effective.world.mood.intensity, MoodIntensity::Spicy);
        assert!(!effective.world.hourly_honk.on_hour_double_honk);
    }

    #[test]
    fn save_preserves_unknown_keys_when_possible() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config.toml");
        fs::write(&path, "custom = 7\n[audio]\nunknown = true\n").unwrap();
        let mut c = Config::default();
        c.audio.enabled = false;
        c.save_atomic(&path).unwrap();
        let text = fs::read_to_string(&path).unwrap();
        assert!(text.contains("custom = 7"));
        assert!(text.contains("unknown = true"));
        assert!(text.contains("enabled = false"));
    }
}
