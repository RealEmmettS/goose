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
//! contract, the platform-neutral collect-window command/snapshot contract, dynamic moods,
//! local-time-driven schedule manners, and the built-in Autumn leaf-pile season. IPC/config and
//! the non-Windows platform backends live outside the engine and build on the types defined here.
//!
//! Engine constants are ported verbatim from the verified modding-API source
//! (`GooseModdingAPI/Exports.cs`, `SamEngine.cs`); the `updateRig` placement math and
//! locomotion live only in the closed binary and are reconstructed clean-room.

#![forbid(unsafe_code)]

// Re-exported so downstream crates (platform backends, the binary) and the golden-frame
// tests can construct/inspect the `Pixmap` that the renderer fills, without depending on
// a specific tiny-skia version themselves.
pub use tiny_skia;

pub mod autumn;
pub mod collect_window;
pub mod command;
pub mod cursor;
pub mod entity;
pub mod feet;
pub mod footmarks;
pub mod foreign_window;
pub mod hearts;
pub mod interaction;
pub mod locomotion;
pub mod math;
pub mod mood;
pub mod render;
pub mod rig;
pub mod rng;
pub mod schedule;
pub mod sound;
pub mod task;
pub mod time;
pub mod world;

// A curated surface for downstream crates (the platform backends, the eventual binary).
pub use autumn::{
    AutumnLeaf, AutumnLeafColor, AutumnPile, AutumnPileId, AutumnPileTarget, AutumnState,
    LEAVES_PER_PILE, MAX_LEAF_PILES,
};
pub use collect_window::{
    CollectWindowCapabilities, CollectWindowCommand, CollectWindowId, CollectWindowKind,
    CollectWindowOptions, CollectWindowPayload, CollectWindowRequestId, CollectWindowSnapshot,
};
pub use command::{PokeAction, PokeOutcome};
pub use cursor::{
    CursorCommand, InteractionOptions, MouseStealOptions, TimingOptions, WorldOptions,
};
pub use entity::{GooseEntity, ParametersTable, SpeedTier};
pub use feet::Feet;
pub use footmarks::{FootMark, FootMarkTiming, FootMarks};
pub use foreign_window::{
    ForeignWindowCapabilities, ForeignWindowId, ForeignWindowOptions, ForeignWindowSnapshot,
};
pub use hearts::{Heart, Hearts};
pub use interaction::{PatTracker, Pointer};
pub use math::{Rect, Vec2};
pub use mood::{
    HourlyHonkOptions, LocalHour, LocalTime, MoodIntensity, MoodKind, MoodMachine, MoodOptions,
    ZParticle, ZParticles,
};
pub use render::{AutumnRenderLayer, RenderPalette};
pub use rig::Rig;
pub use rng::{Deck, RandomSource, SplitMix64};
pub use schedule::{LocalMinute, PresenceSnapshot, PresenceState, ScheduleOptions};
pub use sound::{HonkTone, Sound};
pub use task::{
    AutumnLeafPileTask, CollectWindowTask, FirstUxTask, HyperTask, NabMouseTask, PerchRideTask,
    Task, WanderTask,
};
pub use time::{Accumulator, Clock, DT, FRAMERATE};
pub use world::World;
