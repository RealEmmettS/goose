//! Sound requests emitted by the simulation.
//!
//! Platform-free: the engine only decides *when* a sound should fire and pushes a request;
//! the platform audio backend (e.g. `rodio` in the binary) decides *how* to play it. This
//! keeps `honk-engine` free of any audio dependency.

/// A sound the goose wants played this frame.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Sound {
    /// A honk (one of several variants, chosen by the backend).
    Honk,
    /// Biting the cursor (M7).
    Bite,
    /// A muddy squelch as the goose tracks mud.
    MudSquish,
    /// Contented reaction to being patted (M6).
    Pat,
}
