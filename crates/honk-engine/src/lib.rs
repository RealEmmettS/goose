//! `honk-engine` — the platform-free simulation core of honk300.
//!
//! This crate contains the goose's math, fixed-timestep clock, biased `Deck` RNG,
//! entity/parameter state, procedural rig geometry, footmarks, and the clean-room
//! tiny-skia renderer. It has **no** windowing, OS, audio, or input-device dependency
//! and is fully headless-testable — `#![forbid(unsafe_code)]` is enforced below.
//!
//! Scope through milestone **M9** (see `honk300_plan.md` §14): data/constants, the Deck,
//! renderer, locomotion, task/AI state machine, pointer interactions, sound/cursor intent
//! queues, cursor-mischief task state, the platform-neutral foreign-window perch/ride
//! contract, and the platform-neutral collect-window command/snapshot contract. Moods,
//! schedule, IPC/config, and the non-Windows platform backends arrive in later rounds and build
//! on the types defined here.
//!
//! Engine constants are ported verbatim from the verified modding-API source
//! (`GooseModdingAPI/Exports.cs`, `SamEngine.cs`); the `updateRig` placement math and
//! locomotion live only in the closed binary and are reconstructed clean-room.

#![forbid(unsafe_code)]

// Re-exported so downstream crates (platform backends, the binary) and the golden-frame
// tests can construct/inspect the `Pixmap` that the renderer fills, without depending on
// a specific tiny-skia version themselves.
pub use tiny_skia;

pub mod collect_window;
pub mod cursor;
pub mod entity;
pub mod feet;
pub mod footmarks;
pub mod foreign_window;
pub mod hearts;
pub mod interaction;
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
pub use collect_window::{
    CollectWindowCapabilities, CollectWindowCommand, CollectWindowId, CollectWindowKind,
    CollectWindowOptions, CollectWindowPayload, CollectWindowRequestId, CollectWindowSnapshot,
};
pub use cursor::{CursorCommand, MouseStealOptions, WorldOptions};
pub use entity::{GooseEntity, ParametersTable, SpeedTier};
pub use feet::Feet;
pub use footmarks::{FootMark, FootMarks};
pub use foreign_window::{
    ForeignWindowCapabilities, ForeignWindowId, ForeignWindowOptions, ForeignWindowSnapshot,
};
pub use hearts::{Heart, Hearts};
pub use interaction::{PatTracker, Pointer};
pub use math::{Rect, Vec2};
pub use rig::Rig;
pub use rng::{Deck, RandomSource, SplitMix64};
pub use sound::Sound;
pub use task::{
    CollectWindowTask, FirstUxTask, HyperTask, NabMouseTask, PerchRideTask, Task, WanderTask,
};
pub use time::{Accumulator, Clock, DT, FRAMERATE};
pub use world::World;
