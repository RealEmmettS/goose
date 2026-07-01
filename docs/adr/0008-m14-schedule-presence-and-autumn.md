# ADR 0008: M14 Schedule, Presence, And Autumn

- **Status:** Accepted
- **Date:** 2026-06-29
- **Milestone:** M14

## Context

M14 makes the previously persisted schedule and seasonal settings live. The engine must continue
to be platform-free: it can reason over local dates, local minutes, and platform-neutral presence
snapshots, but it must not sample host time, call notification APIs, inspect fullscreen windows, or
load platform plugins.

The desired user-facing behavior is not a hard pause. Quiet hours, Do Not Disturb, and fullscreen
should make the goose polite while still allowing direct user commands. Autumn must be built in and
procedural; the original `Autumn.dll` remains reference-only and is not loaded or copied.

## Decision

M14 uses **Calm Suppression** for quiet hours, Do Not Disturb, and fullscreen. When manners are
active, the engine suppresses spontaneous honks and autonomous mischief, skips the on-hour double
honk, treats the current task context as calm, modestly slows autonomous wandering, and rebuilds the
random pickable task list without cursor nab, collect-window, foreign-window ride, or Autumn pile
chase. Direct clicks and CLI/TUI pokes remain allowed if their normal configuration and backend
capability gates allow them.

`honk-engine` owns platform-neutral schedule types:

- `ScheduleOptions` carries quiet-hours, DND/fullscreen respect, and seasonal toggles.
- `LocalMinute` represents a minute in a local day.
- `PresenceSnapshot` and `PresenceState` represent backend-reported user availability.
- `World::set_local_time(LocalTime)` remains the only date/time feed, and `World::set_presence`
  is the new presence feed.

Quiet-hour windows are start-inclusive and end-exclusive. Overnight windows are supported, and
`start == end` means no quiet window. Missing local time does not activate quiet hours. Autumn uses
the local meteorological window: September 1 through November 30, no network and no clock sampling
inside the engine.

The existing version-1 TOML schema stays stable. `honk-config` maps `[schedule]` fields into
`WorldOptions.schedule` and maps `[safety].pause_on_fullscreen` into the fullscreen respect flag.
The TUI shows the M14 rows as implemented controls, while still marking future-only rows such as
Wayland and Calm Goose as planned.

Windows maps `SHQueryUserNotificationState` into `PresenceSnapshot` in `honk-platform-windows`.
`QUNS_BUSY` and `QUNS_RUNNING_D3D_FULL_SCREEN` are fullscreen. `QUNS_PRESENTATION_MODE`,
`QUNS_NOT_PRESENT`, `QUNS_QUIET_TIME`, and `QUNS_APP` are DND-like. `QUNS_ACCEPTS_NOTIFICATIONS`
is available. Runtime polling is periodic because fullscreen changes are not delivered as a
notification stream. On API failure, the runtime warns once and degrades to unsupported presence.

Autumn is a built-in platform-free behavior:

- First leaf pile appears after 10 seconds of active Autumn season.
- Subsequent pile intervals are 4.8-72 seconds.
- At most 6 piles exist.
- Each pile has 128 procedural leaves and 30-50 pixel radius/height.
- Piles use a 1-second spawn animation.
- Kicked leaves live for 10 seconds.
- Leaf physics use gravity `-900`, planar max velocity `200`, and vertical render scale `0.6`.

Rendering order is footmarks, Autumn below-goose leaves, goose, Autumn above-goose leaves, hearts,
then sleepy particles. The Windows runtime accepts `HONK300_SMOKE_LOCAL_DATE=YYYYMMDD` so Autumn
can be visually smoke-tested outside September-November without changing the system clock.

## Consequences

- `honk-engine` stays OS-free and deterministic under tests.
- Future macOS, X11, and Wayland backends only need to feed local time and optional presence
  snapshots; unsupported presence is a first-class degraded state.
- User-initiated actions remain responsive during quiet/DND/fullscreen periods.
- Autonomous mischief has one central manners gate instead of per-platform special cases.
- Autumn is available everywhere the renderer runs and does not depend on original binary mods.

## Verification

- `honk-engine` tests cover quiet-hour windows, overnight semantics, `start == end`, missing local
  time behavior, presence gating, on-hour honk suppression, random-mischief exclusion, direct
  poke/click allowance, Autumn date toggles, deterministic pile spawning, pile cap, kick/expiry
  behavior, inactive-season clearing, and visible procedural leaf pixels in both render layers.
- `honk-config` tests cover HH:MM parsing, schedule option mapping, and fullscreen respect mapping
  into `WorldOptions`.
- `honk-config-tui` tests cover live M14 schedule rows and the fullscreen-respect toggle.
- `honk-platform-windows` tests cover pure mapping from
  `QUERY_USER_NOTIFICATION_STATE` constants into engine presence snapshots.
