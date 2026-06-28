//! Platform-free collect-window contract (M9).
//!
//! The engine never sees HWNDs, process handles, image paths, Notepad, or synthetic-input APIs.
//! It chooses note/meme work, emits ordered commands, and consumes opaque snapshots from the
//! platform runtime.

use crate::math::{Rect, Vec2};

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
}

impl CollectWindowSnapshot {
    pub fn center(self) -> Vec2 {
        (self.rect.min + self.rect.max) * 0.5
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
}
