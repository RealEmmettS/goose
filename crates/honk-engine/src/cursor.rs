//! Platform-free cursor-mischief contract (M7).
//!
//! The engine never moves the operating-system cursor directly. It emits cursor commands
//! in world/desktop coordinates; platform backends decide whether they can honor them
//! (Windows now, macOS/X11 later, native Wayland as an honest no-op).

use crate::collect_window::CollectWindowOptions;
use crate::foreign_window::ForeignWindowOptions;
use crate::math::Vec2;

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

/// Runtime options for the platform-free world.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct WorldOptions {
    pub mouse_steal: MouseStealOptions,
    pub foreign_window: ForeignWindowOptions,
    pub collect_window: CollectWindowOptions,
}
