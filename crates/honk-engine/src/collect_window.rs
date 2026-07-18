//! Platform-free collect-window contract (M9).
//!
//! The engine never sees HWNDs, process handles, image paths, Notepad, or synthetic-input APIs.
//! It chooses note/meme work, emits ordered commands, and consumes opaque snapshots from the
//! platform runtime.

use crate::math::{Rect, Vec2};
use tiny_skia::{FilterQuality, Pixmap, PixmapPaint, Transform};

/// A collect prop may be noticeable without becoming a temporary full-screen surface.
pub const COLLECT_PROP_MAX_SCREEN_FRACTION: f32 = 0.48;
const COLLECT_IMAGE_MAX_WIDTH: u32 = 900;
const COLLECT_IMAGE_MAX_HEIGHT: u32 = 700;

fn monitor_limited_extent(screen: f32, fraction: f32, minimum: f32, maximum: f32) -> f32 {
    let safety_limit = (screen.max(1.0) * COLLECT_PROP_MAX_SCREEN_FRACTION)
        .floor()
        .max(1.0);
    (screen * fraction)
        .round()
        .clamp(minimum, maximum)
        .min(safety_limit)
}

/// A readable note target with a hard ceiling tied to the monitor that receives it.
pub fn collect_note_size(display_bounds: Rect) -> Vec2 {
    Vec2::new(
        monitor_limited_extent(display_bounds.width(), 0.32, 420.0, 720.0),
        monitor_limited_extent(display_bounds.height(), 0.32, 240.0, 420.0),
    )
}

/// Aspect-preserving, downscale-only image dimensions for the receiving monitor.
pub fn fitted_collect_image_size(width: u32, height: u32, display_bounds: Rect) -> (u32, u32) {
    let max_width = (display_bounds.width().max(1.0) * COLLECT_PROP_MAX_SCREEN_FRACTION)
        .floor()
        .max(1.0) as u32;
    let max_height = (display_bounds.height().max(1.0) * COLLECT_PROP_MAX_SCREEN_FRACTION)
        .floor()
        .max(1.0) as u32;
    let max_width = max_width.min(COLLECT_IMAGE_MAX_WIDTH);
    let max_height = max_height.min(COLLECT_IMAGE_MAX_HEIGHT);
    let scale = 1.0_f64
        .min(max_width as f64 / width.max(1) as f64)
        .min(max_height as f64 / height.max(1) as f64);
    (
        ((width as f64 * scale).round() as u32).max(1),
        ((height as f64 * scale).round() as u32).max(1),
    )
}

/// Return the original pixels when they already fit; otherwise resample the entire image using
/// one uniform scale factor. The output therefore cannot crop, stretch, or enlarge the source.
pub fn fit_collect_image(pixmap: &Pixmap, display_bounds: Rect) -> Option<Pixmap> {
    let (width, height) =
        fitted_collect_image_size(pixmap.width(), pixmap.height(), display_bounds);
    if width == pixmap.width() && height == pixmap.height() {
        return Some(pixmap.clone());
    }
    let mut fitted = Pixmap::new(width, height)?;
    fitted.draw_pixmap(
        0,
        0,
        pixmap.as_ref(),
        &PixmapPaint {
            quality: FilterQuality::Bicubic,
            ..PixmapPaint::default()
        },
        Transform::from_scale(
            width as f32 / pixmap.width() as f32,
            height as f32 / pixmap.height() as f32,
        ),
        None,
    );
    Some(fitted)
}

/// Opaque backend token for a window controlled by the collect-window runtime.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CollectWindowId(pub u64);

/// Opaque request token linking a spawn command to the resulting backend snapshot.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CollectWindowRequestId(pub u64);

/// M9 collectable prop classes. Donate is intentionally omitted.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CollectWindowKind {
    Note,
    Meme,
}

/// Why a previously live Honk300-owned collect window disappeared.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CollectWindowCloseOrigin {
    /// The person used the native close control.
    User,
    /// Honk300 closed its own prop as part of normal behavior or cleanup.
    Program,
}

/// A selected content item known to the runtime asset catalog.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CollectWindowPayload {
    Note { index: u32 },
    Meme { index: u32 },
}

impl CollectWindowPayload {
    pub fn kind(self) -> CollectWindowKind {
        match self {
            Self::Note { .. } => CollectWindowKind::Note,
            Self::Meme { .. } => CollectWindowKind::Meme,
        }
    }
}

/// Runtime capabilities reported by the platform backend.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct CollectWindowCapabilities {
    pub spawn_note: bool,
    pub spawn_image: bool,
    pub move_window: bool,
    pub set_passthrough: bool,
    pub synthesize_text: bool,
}

/// User/config preference plus backend/content support for collect-window behavior.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CollectWindowOptions {
    pub enabled: bool,
    pub capabilities: CollectWindowCapabilities,
    pub available_notes: u32,
    pub available_memes: u32,
    pub notes_enabled: bool,
    pub memes_enabled: bool,
}

impl Default for CollectWindowOptions {
    fn default() -> Self {
        Self {
            enabled: true,
            capabilities: CollectWindowCapabilities::default(),
            available_notes: 0,
            available_memes: 0,
            notes_enabled: true,
            memes_enabled: true,
        }
    }
}

impl CollectWindowOptions {
    pub fn with_backend_support(
        capabilities: CollectWindowCapabilities,
        available_notes: u32,
        available_memes: u32,
    ) -> Self {
        Self {
            capabilities,
            available_notes,
            available_memes,
            ..Self::default()
        }
    }

    pub fn kind_active(self, kind: CollectWindowKind) -> bool {
        if !self.enabled || !self.capabilities.move_window {
            return false;
        }
        match kind {
            CollectWindowKind::Note => {
                self.notes_enabled
                    && self.available_notes > 0
                    && self.capabilities.spawn_note
                    && self.capabilities.synthesize_text
            }
            CollectWindowKind::Meme => {
                self.memes_enabled && self.available_memes > 0 && self.capabilities.spawn_image
            }
        }
    }

    pub fn active(self) -> bool {
        self.kind_active(CollectWindowKind::Note) || self.kind_active(CollectWindowKind::Meme)
    }
}

/// A platform operation requested by the simulation.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CollectWindowCommand {
    Spawn {
        request: CollectWindowRequestId,
        payload: CollectWindowPayload,
    },
    Move {
        id: CollectWindowId,
        top_left: Vec2,
    },
    SetPassthrough {
        id: CollectWindowId,
        passthrough: bool,
    },
    Focus {
        id: CollectWindowId,
    },
    TypeNote {
        id: CollectWindowId,
        note_index: u32,
    },
    Close {
        id: CollectWindowId,
    },
}

/// Backend-reported state for a controlled collect window.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CollectWindowSnapshot {
    pub id: CollectWindowId,
    pub request: CollectWindowRequestId,
    pub kind: CollectWindowKind,
    pub rect: Rect,
    pub alive: bool,
    /// Set only for a one-shot dead-window snapshot.
    pub close_origin: Option<CollectWindowCloseOrigin>,
}

impl CollectWindowSnapshot {
    pub fn center(self) -> Vec2 {
        (self.rect.min + self.rect.max) * 0.5
    }
}

/// One-shot evidence that a specific Honk300-owned prop stopped existing.
///
/// This is deliberately separate from the latest live snapshot. A backend can report a close
/// for a lingering note while a newer collect request is active, and the engine must retain both
/// facts without allowing the unrelated close to terminate the newer request.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CollectWindowCloseEvent {
    pub id: CollectWindowId,
    pub request: CollectWindowRequestId,
    pub kind: CollectWindowKind,
    pub rect: Rect,
    pub origin: CollectWindowCloseOrigin,
}

impl CollectWindowCloseEvent {
    pub fn from_dead_snapshot(snapshot: CollectWindowSnapshot) -> Option<Self> {
        (!snapshot.alive).then_some(Self {
            id: snapshot.id,
            request: snapshot.request,
            kind: snapshot.kind,
            rect: snapshot.rect,
            origin: snapshot.close_origin?,
        })
    }

    pub fn matches(self, request: CollectWindowRequestId, kind: CollectWindowKind) -> bool {
        self.request == request && self.kind == kind
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn options_distinguish_content_and_capabilities() {
        let caps = CollectWindowCapabilities {
            spawn_note: true,
            spawn_image: true,
            move_window: true,
            set_passthrough: true,
            synthesize_text: true,
        };
        let options = CollectWindowOptions::with_backend_support(caps, 1, 0);
        assert!(options.kind_active(CollectWindowKind::Note));
        assert!(!options.kind_active(CollectWindowKind::Meme));
        assert!(options.active());
    }

    #[test]
    fn options_distinguish_user_enabled_kinds() {
        let caps = CollectWindowCapabilities {
            spawn_note: true,
            spawn_image: true,
            move_window: true,
            set_passthrough: true,
            synthesize_text: true,
        };
        let mut options = CollectWindowOptions::with_backend_support(caps, 1, 1);
        options.notes_enabled = false;
        assert!(!options.kind_active(CollectWindowKind::Note));
        assert!(options.kind_active(CollectWindowKind::Meme));

        options.memes_enabled = false;
        assert!(!options.active());
    }

    #[test]
    fn payload_reports_kind() {
        assert_eq!(
            CollectWindowPayload::Note { index: 3 }.kind(),
            CollectWindowKind::Note
        );
        assert_eq!(
            CollectWindowPayload::Meme { index: 4 }.kind(),
            CollectWindowKind::Meme
        );
    }

    #[test]
    fn collect_images_fit_without_cropping_stretching_or_upscaling() {
        let display = Rect::new(Vec2::ZERO, Vec2::new(1620.0, 1080.0));
        assert_eq!(fitted_collect_image_size(2400, 1600, display), (777, 518));
        assert_eq!(fitted_collect_image_size(1000, 3000, display), (173, 518));
        assert_eq!(fitted_collect_image_size(320, 180, display), (320, 180));

        let panorama = fitted_collect_image_size(8000, 500, display);
        assert_eq!(panorama, (777, 49));
        let tiny_display = Rect::new(Vec2::ZERO, Vec2::new(640.0, 480.0));
        let portrait = fitted_collect_image_size(500, 4000, tiny_display);
        assert!(portrait.0 <= 307 && portrait.1 <= 230);
        assert!((portrait.0 as f64 / portrait.1 as f64 - 0.125).abs() < 0.01);
    }

    #[test]
    fn collect_notes_are_readable_but_bounded_on_every_display() {
        for display in [
            Rect::new(Vec2::ZERO, Vec2::new(640.0, 480.0)),
            Rect::new(Vec2::ZERO, Vec2::new(1920.0, 1080.0)),
            Rect::new(Vec2::ZERO, Vec2::new(3840.0, 2160.0)),
        ] {
            let size = collect_note_size(display);
            assert!(size.x <= display.width() * COLLECT_PROP_MAX_SCREEN_FRACTION);
            assert!(size.y <= display.height() * COLLECT_PROP_MAX_SCREEN_FRACTION);
            assert!(size.x > 0.0 && size.y > 0.0);
        }
        assert_eq!(
            collect_note_size(Rect::new(Vec2::ZERO, Vec2::new(1920.0, 1080.0))),
            Vec2::new(614.0, 346.0)
        );
    }
}
