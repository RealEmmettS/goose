//! Platform-free cursor-mischief contract (M7).
//!
//! The engine never moves the operating-system cursor directly. It emits cursor commands
//! in world/desktop coordinates; platform backends decide whether they can honor them
//! (Windows now, macOS/X11 later, native Wayland as an honest no-op).

use crate::collect_window::CollectWindowOptions;
use crate::entity::ParametersTable;
use crate::footmarks::FootMarkTiming;
use crate::foreign_window::ForeignWindowOptions;
use crate::math::Vec2;
use crate::mood::{HourlyHonkOptions, MoodOptions};
use crate::render::RenderPalette;
use crate::schedule::ScheduleOptions;

/// A cursor operation requested by the simulation for the platform backend to apply.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CursorCommand {
    /// Warp the real cursor to this world/desktop coordinate.
    WarpTo(Vec2),
}

/// Tuning and capability flags for mouse-stealing behavior.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MouseStealOptions {
    /// User/config preference: whether mouse stealing is allowed at all.
    pub enabled: bool,
    /// Backend capability: whether this OS/session can warp the real cursor.
    pub warp_supported: bool,
    /// Distance from beak to cursor that counts as a successful grab.
    pub grab_distance: f32,
    /// Distance threshold for dropping the grab when the user/system pulls away.
    pub drop_distance: f32,
    /// Seconds to keep dragging before the goose lets go.
    pub succ_time: f32,
}

impl Default for MouseStealOptions {
    fn default() -> Self {
        Self {
            enabled: true,
            warp_supported: false,
            grab_distance: 60.0,
            drop_distance: 200.0,
            succ_time: 2.5,
        }
    }
}

impl MouseStealOptions {
    /// Whether a nab task is allowed to run in the current runtime configuration.
    pub fn active(self) -> bool {
        self.enabled && self.warp_supported
    }

    /// Default M7 tuning with the backend capability filled in by the platform layer.
    pub fn with_backend_support(warp_supported: bool) -> Self {
        Self {
            warp_supported,
            ..Self::default()
        }
    }
}

/// Runtime appearance/personality options that stay platform-free.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct AppearanceOptions {
    /// Master calm valve: suppress autonomous disruption while leaving direct commands live.
    pub calm_goose: bool,
}

/// Runtime options for the platform-free world.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WorldOptions {
    /// Whether the runtime should use the full virtual desktop instead of the primary monitor.
    pub multi_monitor_chase: bool,
    pub mouse_steal: MouseStealOptions,
    pub foreign_window: ForeignWindowOptions,
    pub collect_window: CollectWindowOptions,
    pub interaction: InteractionOptions,
    pub appearance: AppearanceOptions,
    pub timing: TimingOptions,
    pub parameters: ParametersTable,
    pub footmarks: FootMarkTiming,
    pub palette: RenderPalette,
    pub mood: MoodOptions,
    pub hourly_honk: HourlyHonkOptions,
    pub schedule: ScheduleOptions,
}

impl Default for WorldOptions {
    fn default() -> Self {
        Self {
            multi_monitor_chase: true,
            mouse_steal: MouseStealOptions::default(),
            foreign_window: ForeignWindowOptions::default(),
            collect_window: CollectWindowOptions::default(),
            interaction: InteractionOptions::default(),
            appearance: AppearanceOptions::default(),
            timing: TimingOptions::default(),
            parameters: ParametersTable::default(),
            footmarks: FootMarkTiming::default(),
            palette: RenderPalette::default(),
            mood: MoodOptions::default(),
            hourly_honk: HourlyHonkOptions::default(),
            schedule: ScheduleOptions::default(),
        }
    }
}

/// User-facing interaction toggles that affect platform-free input handling.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InteractionOptions {
    /// Whether hover sweeps over the goose register pats, hearts, and calm.
    pub pat_streak: bool,
}

impl Default for InteractionOptions {
    fn default() -> Self {
        Self { pat_streak: true }
    }
}

/// Runtime timing values that were historically constants in the original config.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TimingOptions {
    pub first_wander_time: f32,
    pub min_wandering_time: f32,
    pub max_wandering_time: f32,
}

impl Default for TimingOptions {
    fn default() -> Self {
        Self {
            first_wander_time: 20.0,
            min_wandering_time: 20.0,
            max_wandering_time: 40.0,
        }
    }
}
