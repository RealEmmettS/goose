//! Sound requests emitted by the simulation.
//!
//! Platform-free: the engine only decides *when* a sound should fire and pushes a request;
//! the platform audio backend (e.g. `rodio` in the binary) decides *how* to play it. This
//! keeps `honk-engine` free of any audio dependency.

/// Tone hint for a honk request. The backend still chooses from the bundled clips.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HonkTone {
    Normal,
    High,
    Low,
}

/// A sound the goose wants played this frame.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Sound {
    /// A honk (one of several variants, chosen by the backend).
    Honk(HonkTone),
    /// Biting the cursor (M7).
    Bite,
    /// A muddy squelch as the goose tracks mud.
    MudSquish,
    /// Contented reaction to being patted (M6).
    Pat,
}

impl Sound {
    pub const fn honk() -> Self {
        Self::Honk(HonkTone::Normal)
    }

    pub const fn high_honk() -> Self {
        Self::Honk(HonkTone::High)
    }

    pub const fn low_honk() -> Self {
        Self::Honk(HonkTone::Low)
    }
}
