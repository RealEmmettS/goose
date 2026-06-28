# ADR 0006: M12 Config TUI, Durable TOML, and the Capability/Preference Boundary

- **Status:** Accepted
- **Date:** 2026-06-28
- **Milestone:** M12

## Context

M12 adds the ratatui config TUI and the durable, versioned TOML config: `goose_config_version`,
`serde(default)` (unknown keys preserved, missing keys defaulted), validate-then-apply, and an
atomic save (temp file + persist). Saving hot-applies to the running goose through the M10 reload
IPC.

Reloading rebuilds the world options from disk. The runtime separately tracks **backend
capabilities** — cursor warp, foreign-window watch, collect-window control — which can degrade at
runtime when an OS call fails. A correct reload must keep real capability losses while honoring
the user's config-preference changes.

M12 landed in the same single commit as M11, without an ADR. An adversarial review against
`honk300_plan.md` found the capability/preference boundary blurred in two places and a third,
unrelated interaction conflation:

1. The cursor-warp **capability** was seeded from the no-mouse-steal **preference**, so a config
   edit that re-enabled stealing never took effect until the next restart (the flag was latched
   off and never reset).
2. A collect-window backend failure was recorded only in engine state and was overwritten by the
   next reload, which rebuilt collect capabilities as all-supported — the goose would keep
   retrying a dead capability.
3. `interaction.pat_streak` gated the click reaction as well as pats, so disabling the hover-pat
   streak silently disabled clicking the goose.

## Decision

A **backend capability** is distinct from a **user preference**. Behavior is active only when
both agree: `active = enabled(preference) && supported(capability)`.

- The runtime seeds each `*_supported` flag from the actual platform capability, never from a
  preference. On Windows the cursor can always be warped, so cursor-warp support initializes to
  `true`; the no-mouse-steal preference is applied solely through the config-derived
  `MouseStealOptions::enabled`. A `*_supported` flag only flips to `false` when a real OS call
  fails at runtime.
- Runtime capability losses are **durable across reload**. Cursor-warp, foreign-window watch, and
  collect-window each thread their backend-supported flag through `BackendState` /
  `effective_options`, so a reload rebuilds the world with the loss intact instead of resurrecting
  a capability the backend already reported dead. `BackendState` carries
  `collect_window_supported` alongside `cursor_warp_supported` and `window_watch_supported`; all
  three degrade uniformly.
- `interaction.pat_streak` scopes to the hover-pat streak (hearts and the post-pat calm window)
  **only**. The click reaction — a hyper burst, or a cursor nab when mouse-stealing is supported —
  is a separate interaction and is never disabled by turning pats off.
- Config stays versioned, `serde(default)`, validate-then-apply, atomic save. The TUI is the
  reducer-driven editor; hot-apply rides the M10 reload IPC (see ADR 0004, ADR 0005).

## Consequences

- Editing config and reloading re-enables mouse steal without a restart.
- A collect-window backend failure stays disabled across reloads, so the goose stops re-attempting
  a capability the platform cannot honor.
- Disabling pats leaves click-to-hyper (and click-to-nab) working.
- The capability/preference split is now uniform across all three degradable behaviors, which keeps
  future backends (macOS, X11, native Wayland) honest: an unsupported behavior degrades without
  masking the user's preference.

## Verification

- `honk-config` tests: `effective_options` merges CLI overrides and config; a backend
  collect-window capability loss disables collect behavior even when config still enables it.
- `honk-engine` tests: clicking the goose triggers hyper even with `pat_streak` off; `pat_streak`
  off still disables hearts/calm; click triggers nab when mouse-stealing is supported.
- Root-binary test: the initial cursor-warp capability ignores the mouse-steal preference.
- Functional smoke over the real IPC: start with mouse steal disabled in config → `do nab` reports
  `UNSUPPORTED`; flip the config and `reload` → `do nab` is accepted, proving the capability was
  not latched off.
