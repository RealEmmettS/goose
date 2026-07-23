# ADR 0039 — Calmer Default Mud Cadence

- Status: Accepted (2026-07-23, operator-directed v1.3.6 tuning)
- Supersedes in part: ADR 0016 (only its original automatic mud cadence and duration defaults)
- Relates to: ADR 0019 (stable configuration schema), ADR 0022 (permission-wait suppression)

## Context

ADR 0016 deliberately moved mud out of ordinary wandering and into story-driven off-screen
puddle hops. Its first-pass cadence—one hop every 70–160 seconds followed by 30–90 seconds of
tracking—was explicitly expected to be tuned after daily use. In practice, the goose still spends
too much time muddy. The separate direct `do mud` and fresh-config default also lasts 15 seconds.

The operator requested a patch release that makes natural mud substantially less common and
shorter, then selected the strongest proposed reduction and asked that fresh/manual mud shorten as
well. Existing saved configuration remains user intent and must not be rewritten.

## Decision

- Automatic puddle hops default to a randomized **180–300 second** interval.
- A goose returning from an automatic puddle hop tracks new mud for **10–30 seconds**.
- The off-screen puddle visit remains **8–15 seconds**. The puddle narrative, exposed-edge return,
  task interruption rules, manners, permission suppression, and deterministic world RNG are
  unchanged.
- `ParametersTable::default()` and `MudConfig::default()` use **10 seconds** for direct/fresh mud.
  Missing fields and explicit reset therefore materialize 10 seconds consistently.
- An existing explicit `[mud].duration_to_track_seconds` value remains authoritative. There is no
  schema migration and `goose_config_version` remains 2.
- Automatic puddle timing stays engine-internal; this patch adds no TOML or TUI controls for its
  interval or randomized duration.
- Footprint lifetime (8.5 seconds), shrink time (1 second), rendering, and mud audio are unchanged.
  When tracking ends, no new prints are added while existing prints complete their normal fade.

## Consequences

- Mud becomes an occasional desktop event instead of a near-continuous state.
- Direct `do mud` remains available and still honors a user-selected duration.
- Existing configurations keep their current experience until the user changes or resets the
  manual duration; automatic puddle cadence changes for everyone because it was never
  config-exposed.
- The public CLI/TUI/TOML shapes and all platform/backend contracts remain compatible, so this is a
  patch release.

## Verification

- Pin all six puddle timing defaults, including the unchanged away interval.
- Prove the default direct poke creates exactly a ten-second tracking window.
- Prove omitted mud configuration resolves to 10 seconds and an explicit saved 15-second value
  survives load/save and reaches effective world options.
- Retain the deterministic puddle-delivery and plain-wandering-never-muds tests.
- Complete the ordinary immutable release and fresh-public-byte qualification for v1.3.6.
