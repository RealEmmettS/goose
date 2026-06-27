# 0001 — M7 Cursor Mischief, Renderer Direction, And Cross-Platform Guardrails

Status: Accepted  
Date: 2026-06-26

## Context

M7 implements Desktop Goose cursor stealing for the Rust `honk300` port. The project targets Windows first, but every design choice must remain valid for macOS, Linux X11, native Wayland-degraded mode, x64, ARM64, and Linux ARM targets.

Before adding cursor warping, M7 performed a completed-milestone review of M0-M6. That review confirmed the engine/task architecture is usable, found stale status docs, and identified a performance follow-up: the M3+ fullscreen primary-monitor layered overlay is correct for world-space trails, but no longer matches the earlier small-window dirty-rect/low-CPU claim.

M7 also exposed two renderer facts:

- The current clean-room procedural goose can be improved, but tuning per-frame path geometry is slow and subjective.
- The future renderer should be more customizable and efficient without forcing GPU/windowing complexity into the platform backends.

## Decision

### Cursor Mischief Contract

- Keep `honk-engine` platform-free and architecture-neutral.
- Represent cursor movement as engine commands, not direct OS calls.
- Add a cursor command queue owned by the engine and drained by platform backends.
- Add engine options for mouse stealing so unsupported platforms and user opt-out paths are explicit.
- Register `NabMouseTask` only when mouse stealing is enabled and the backend reports cursor warping support.
- Clicking the goose starts `NabMouseTask` when supported. When unsupported or disabled, the older click-to-hyper behavior remains the fallback.
- Suppress pat/click-to-hyper handling while nab owns the cursor so synthetic cursor motion does not spawn hearts or interrupt the grab.

### Nab Behavior

- Seek the live cursor at charge speed.
- Bite once when the beak reaches the cursor.
- Capture the beak-to-cursor offset.
- During drag, keep the cursor anchored to the beak plus the captured offset.
- Move through a bounded HYPR-style retargeting burst instead of dragging in a single straight line.
- End after the configured success time or if the user/system pulls the cursor beyond the drop threshold.

### Windows Backend

- Windows implements cursor polling and cursor warping first.
- Cursor polling remains in the Windows platform crate.
- Cursor warping is a thin backend wrapper around Windows cursor positioning.
- If cursor warping fails at runtime, the app warns once, marks cursor warping unavailable, and continues without crashing.
- `--no-mouse-steal` disables cursor stealing without disabling the rest of the goose.

### Cross-Platform Guardrails

- macOS, X11, and Wayland backends must report capabilities honestly.
- macOS cursor/window mischief will need permission-aware integration, especially Accessibility and a real app bundle.
- Linux X11 can support more of the original prank model.
- Native Wayland should degrade explicitly for cursor warping, synthetic input, and foreign-window
  control. Runtime start/stop/configuration control is handled separately by local IPC; see ADR
  0004.
- Engine math continues to use signed desktop coordinates so negative monitors, mixed DPI, and future per-monitor overlays remain possible.

### Renderer Direction

- Keep the current improved procedural renderer for M7.
- Record Renderer V2 as a separate backlog task, not an M7 blocker.
- Renderer V2 should be a custom CPU sprite/atlas blitter that outputs premultiplied pixels for platform backends.
- Keep `tiny-skia`/`resvg` as useful vector/effect or asset-rasterization helpers.
- Do not switch the desktop-pet runtime to Vello/wgpu, Skia, Bevy, Macroquad, or ggez as the primary renderer. Those tools are not a clean fit for the current layered-overlay pixel path and cross-target packaging constraints.
- Future atlas metadata should include stable anchors, beak/cursor attach points, hit masks, frame bounds, and animation tags.

## Consequences

- M7 can be tested headlessly at the engine level and demonstrated on Windows with real cursor movement.
- Non-Windows builds keep compiling without fake cursor-warp behavior.
- M8 and later mischief tasks can reuse the same capability-gated command pattern.
- The app avoids GPU readback and game-framework windowing complexity for the overlay path.
- Renderer V2 has a clear future contract while M7 remains complete with the accepted procedural goose.
- Fullscreen overlay present cost remains a known follow-up and is tracked separately.

## Verification

- Engine/world unit coverage for disabled/unsupported mouse stealing, seek/grab/drag/drop/timeout, click-to-nab, fallback click-to-hyper, cursor-command drain, HYPR-style retargeting, and M6 interaction suppression during nab.
- Golden-frame coverage for rest, reaching, and mid-stride goose poses.
- Local gate passed on Windows: formatting, clippy, workspace tests, and release build.
- Windows runtime smoke completed with mouse stealing enabled for 60 seconds.
- Additional visual smoke runs confirmed the overlay still renders after goose-shape tuning.

## Follow-Ups

- `#p4d`: measure and optimize fullscreen overlay present cost.
- `#r2v`: implement Renderer V2 sprite-atlas renderer.
- M8: add a readiness pass for foreign-window capability abstractions before marking that milestone complete.
