//! Platform-free foreign-window contract (M8).
//!
//! The engine never sees HWNDs, AX elements, X11 window IDs, or Wayland objects. Platform
//! backends translate their native window handles into opaque IDs and feed the engine a
//! world-space anchor while a user is dragging a foreign window.

use crate::math::{Rect, Vec2};

/// Opaque backend token for a foreign application window.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ForeignWindowId(pub u64);

/// The current geometry of a foreign window being dragged by the user.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ForeignWindowSnapshot {
    pub id: ForeignWindowId,
    /// Window bounds in signed world/desktop coordinates.
    pub rect: Rect,
    /// The point the goose should seek and ride. Windows uses title-bar top-center for M8.
    pub ride_anchor: Vec2,
}

impl ForeignWindowSnapshot {
    /// Build the M8 default ride target from a window rect: top-center of the frame.
    pub fn top_center(id: ForeignWindowId, rect: Rect) -> Self {
        let ride_anchor = Vec2::new((rect.min.x + rect.max.x) * 0.5, rect.min.y);
        Self {
            id,
            rect,
            ride_anchor,
        }
    }
}

/// Runtime capabilities reported by the platform backend.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ForeignWindowCapabilities {
    /// The backend can observe a user's active move/resize drag.
    pub watch_drag: bool,
    /// The backend can move another app's window. M8 reports this for readiness only.
    pub move_window: bool,
}

/// User/config preference plus backend support for foreign-window behavior.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ForeignWindowOptions {
    pub enabled: bool,
    pub capabilities: ForeignWindowCapabilities,
}

impl Default for ForeignWindowOptions {
    fn default() -> Self {
        Self {
            enabled: true,
            capabilities: ForeignWindowCapabilities::default(),
        }
    }
}

impl ForeignWindowOptions {
    /// Whether perch-and-ride is allowed to run in the current runtime configuration.
    pub fn watch_active(self) -> bool {
        self.enabled && self.capabilities.watch_drag
    }

    /// Default M8 options with platform capabilities filled in by the backend.
    pub fn with_backend_support(watch_drag: bool, move_window: bool) -> Self {
        Self {
            capabilities: ForeignWindowCapabilities {
                watch_drag,
                move_window,
            },
            ..Self::default()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn top_center_anchor_supports_signed_coordinates() {
        let rect = Rect {
            min: Vec2::new(-900.0, -40.0),
            max: Vec2::new(-300.0, 360.0),
        };
        let snapshot = ForeignWindowSnapshot::top_center(ForeignWindowId(7), rect);
        assert_eq!(snapshot.ride_anchor, Vec2::new(-600.0, -40.0));
    }

    #[test]
    fn default_options_do_not_assume_backend_support() {
        let options = ForeignWindowOptions::default();
        assert!(options.enabled);
        assert!(!options.watch_active());
        assert!(!options.capabilities.move_window);
    }
}
