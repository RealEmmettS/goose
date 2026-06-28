# ADR 0007: M13 Dynamic Moods And Local-Time Injection

- **Status:** Accepted
- **Date:** 2026-06-28
- **Milestone:** M13

## Context

M13 adds two autonomous behaviors from the canonical plan: dynamic moods and the on-hour double
honk. Both features touch the simulation loop, but only one needs wall-clock information. The
engine must remain platform-free and deterministic under tests; platform runtimes already own OS
integration and can sample local time without leaking Windows, macOS, X11, or Wayland APIs into
`honk-engine`.

M12R also closes leftover config/TUI plumbing that M13 depends on: speed, mud, color, mood, and
on-hour settings now map from TOML into runtime options instead of being write-only fields.

## Decision

Dynamic moods are a platform-free state machine owned by `honk-engine`. `MoodMachine` chooses
between `Content`, `Hyper`, `Sad`, `Sleepy`, and `Mischievous` with seeded weighted transitions.
The default `normal` intensity is conservative: long 60-120 second mood windows, content-heavy
weights, and rare hyper/mischievous transitions.

Mood effects post-modulate task output instead of replacing the task system:

- Sad and sleepy moods slow speed/acceleration and lower neck posture.
- Sleepy emits procedural Z particles rendered by the shared renderer.
- Hyper can request the existing `HyperTask` when no long-running mischief task is active.
- Mischievous only biases already-enabled, already-supported nab/collect factories by duplicating
  them in the pickable task list. It never turns on unsupported capabilities.

The on-hour double honk uses runtime-injected local time. Platform runtimes sample local wall time
and call `World::set_local_time(LocalTime)`. The engine then emits one high honk at the top of a
local hour and a second high honk 0.35 seconds later, once per local hour. If the setting is off,
pending second honks are cleared.

`Sound::Honk` now carries a `HonkTone` (`Normal`, `High`, `Low`). The engine requests a tone; the
audio backend maps tones to bundled honk clips while still respecting global audio toggles and
missing-audio-device no-op behavior.

M15 still owns the broader appearance/recolor milestone. M12R only hot-applies the existing
original-style palette triplet when `use_custom_colors = true`; the default palette remains the
golden-frame baseline.

## Consequences

- `honk-engine` remains OS-free and deterministic. Local clock sampling lives outside the engine.
- Mood transitions are testable by seed and do not create new capability paths.
- On-hour honking follows the user's local hour without making the engine depend on `chrono`,
  system APIs, or host time.
- Future macOS/Linux runtimes only need to feed `LocalTime` and map honk tones; they do not need
  to reimplement mood behavior.
- M14 remains responsible for quiet-hours enforcement, DND/fullscreen respect, and seasonal
  Autumn behavior. M13 only edits quiet-hour config values and implements the on-hour honk.

## Verification

- `honk-engine` tests cover hot-applied parameters/footmark timing, deterministic mood
  transitions, disabled moods, sleepy particles, mischievous task bias, and on-hour double honks.
- `honk-config` tests cover unknown-key warnings/preservation, speed/mud/color validation,
  mood-intensity parsing, and option mapping into `WorldOptions`.
- `honk-config-tui` tests cover dynamic row generation, quiet-time 15-minute increments, mood
  intensity cycling, dirty quit confirmation, reducer-owned command results, and detached/null
  stdio start command construction.
- `honk-platform-windows` and root-binary tests compile the Windows local-time snapshot path and
  runtime wiring.
