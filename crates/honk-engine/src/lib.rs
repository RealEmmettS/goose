//! `honk-engine` — the platform-free simulation core of honk300.
//!
//! This crate contains the goose's math, fixed-timestep clock, biased `Deck` RNG,
//! entity/parameter state, procedural rig geometry, footmarks, and the clean-room
//! tiny-skia renderer. It has **no** windowing, OS, audio, or input-device dependency
//! and is fully headless-testable — `#![forbid(unsafe_code)]` is enforced below.
//!
//! Scope is milestone **M0** (see `honk300_plan.md` §14): the data, the constants, the
//! Deck, and the renderer, all pinned by tests. Locomotion + the 120 Hz accumulator
//! (M2), the task/AI state machine (M4+), moods, schedule, and every platform backend
//! arrive in later rounds and build on the types defined here.
//!
//! Engine constants are ported verbatim from the verified modding-API source
//! (`GooseModdingAPI/Exports.cs`, `SamEngine.cs`); the `updateRig` placement math and
//! locomotion live only in the closed binary and are reconstructed clean-room.

#![forbid(unsafe_code)]

// Re-exported so downstream crates (platform backends, the binary) and the golden-frame
// tests can construct/inspect the `Pixmap` that the renderer fills, without depending on
// a specific tiny-skia version themselves.
pub use tiny_skia;

pub mod entity;
pub mod feet;
pub mod footmarks;
pub mod locomotion;
pub mod math;
pub mod render;
pub mod rig;
pub mod rng;
pub mod sound;
pub mod task;
pub mod time;
pub mod world;

// A curated surface for downstream crates (the platform backends, the eventual binary).
pub use entity::{GooseEntity, ParametersTable, SpeedTier};
pub use feet::Feet;
pub use footmarks::{FootMark, FootMarks};
pub use math::{Rect, Vec2};
pub use rig::Rig;
pub use rng::{Deck, RandomSource, SplitMix64};
pub use sound::Sound;
pub use task::{FirstUxTask, Task, WanderTask};
pub use time::{Accumulator, Clock, DT, FRAMERATE};
pub use world::World;
