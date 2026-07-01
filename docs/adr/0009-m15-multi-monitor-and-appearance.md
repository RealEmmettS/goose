# ADR 0009: M15 Multi-Monitor And Appearance

- **Status:** Accepted
- **Date:** 2026-07-01
- **Milestone:** M15

## Context

M15 makes the persisted multi-monitor and appearance settings live. The canonical plan requires
continuous signed virtual-desktop simulation with one overlay window per monitor, not one giant
virtual-screen window. The engine must stay platform-free: it can simulate inside whatever bounds
the runtime provides, but it must not enumerate monitors, own HWNDs, or know DPI APIs.

The appearance scope is also intentionally narrow. The original goose renderer exposes only the
white body, orange beak/feet, and outline colors. M15 should make that palette fully usable and
make the Calm Goose valve live without introducing Renderer V2, new copied art, or a new schema.

## Decision

Windows now creates one layered overlay HWND per monitor. Monitor bounds are enumerated through
Win32, preserved as signed desktop coordinates, and unioned into the virtual desktop. The runtime
chooses the engine world bounds at startup:

- `[behaviors].multi_monitor_chase = true` uses the full virtual monitor union.
- `[behaviors].multi_monitor_chase = false` uses the primary/default monitor bounds.

The engine continues to receive only a `Rect` and stays unaware of monitor topology. Runtime
reloads hot-apply normal world options, but a change to `multi_monitor_chase` is restart-required
because changing it means rebuilding world bounds and the overlay window set.

Rendering uses a dirty world-space region from `World::render_bounds(previous)`. The region covers
the current goose rig, previous frame, active footmarks, hearts, sleepy particles, and Autumn
leaves/piles, then clips to world bounds and aligns to whole pixels. The Windows backend clips and
crops that pixmap per monitor before calling `UpdateLayeredWindow`; monitor windows with no
intersection are hidden so stale pixels do not remain visible. Foreign-window watching filters all
overlay HWNDs.

`WorldOptions` now carries the persisted M15 settings:

- `multi_monitor_chase` is a startup/runtime contract flag, default on.
- `appearance.calm_goose` is a master Calm Suppression valve, default off.

Calm Goose reuses the M14 manners path: it suppresses spontaneous honks, on-hour honks, autonomous
cursor/window/collect mischief, and Autumn pile chase while direct clicks and CLI/TUI pokes still
pass through normal config/capability gates.

The TUI makes Calm Goose live, marks multi-monitor chase as restart-required, and edits the
existing original-style palette through RGB channel rows for goose white, goose orange, and goose
outline. The default palette remains the original-style golden baseline unless
`use_custom_colors = true`.

## Consequences

- Multi-monitor traversal works without adding OS concepts to `honk-engine`.
- The Windows backend matches the planned per-monitor-window architecture and reduces present cost
  by avoiding a full primary-monitor repaint every frame.
- Mixed signed coordinates, including negative monitor origins, are explicit test coverage.
- Appearance/recolor is complete for the original renderer scope without starting Renderer V2.
- Future macOS/X11/Wayland backends can reuse the same startup world-bounds decision and dirty
  render contract.

## Verification

- `honk-engine` tests cover Calm Goose honk suppression/direct honk allowance, dirty render bounds,
  and signed rectangle union/intersection helpers.
- `honk-config` tests cover mapping `multi_monitor_chase`, `calm_goose`, and palette values into
  `WorldOptions`.
- `honk-config-tui` tests cover the live Calm Goose row, restart-required multi-monitor row, and
  RGB channel editing.
- `honk-platform-windows` tests cover signed monitor union/primary selection and multi-HWND
  overlay filtering.
- `honk300` runtime tests cover primary-vs-virtual world-bounds selection.
