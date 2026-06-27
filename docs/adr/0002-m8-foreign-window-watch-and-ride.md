# 0002 - M8 Foreign-Window Watch-And-Ride Contract

Status: Accepted  
Date: 2026-06-26

## Context

M8 teaches the goose to react when the user drags another application's window. The behavior is intentionally scoped to watch-and-ride: run to the dragged window's title-bar anchor, ride it if the goose arrives before release, and resume the interrupted task when the drag ends. Autonomous window collection, notepad/meme dispatch, and backend-issued window movement remain M9+ work.

M7 established the rule this milestone must preserve: `honk-engine` owns platform-neutral behavior and backend capability gates, while OS-specific handles and calls stay in platform crates. M8 also has to keep signed desktop coordinates viable for negative-position monitors, future per-monitor overlays, and mixed-DPI polish.

## Decision

### Engine Contract

- Keep `honk-engine` platform-free and `#![forbid(unsafe_code)]`.
- Represent a foreign window as `ForeignWindowId(u64)`, an opaque backend token that is not a raw platform handle in engine code.
- Feed the engine `ForeignWindowSnapshot { id, rect, ride_anchor }` while a user drag is active.
- Add `ForeignWindowCapabilities { watch_drag, move_window }` and `ForeignWindowOptions` under `WorldOptions`.
- Treat `move_window` as future readiness data in M8; the engine does not emit move-window commands yet.

### Perch-And-Ride Behavior

- Implement `PerchRideTask` as a transient interrupt that reuses the M6/M7 resume path.
- During seek, the goose uses normal locomotion toward the active window's ride anchor.
- If the drag releases before arrival, or if watch capability is lost, the task ends and the interrupted task resumes.
- Once the goose arrives while the drag is still active, it pins to the moving anchor until release.
- Pat/click-to-hyper interactions are suppressed while perch-and-ride is active so pointer input does not stack interrupts.

### Windows Backend

- Windows observes user move/size drags with `SetWinEventHook(EVENT_SYSTEM_MOVESIZESTART..EVENT_SYSTEM_MOVESIZEEND)` out-of-context.
- The hook callback only queues start/end events. The main loop polls live geometry.
- The backend polls `GetWindowRect` and converts the frame top-center to the M8 ride anchor.
- The backend filters the app's own overlay, invisible windows, minimized windows, invalid handles, and non-root windows.
- Hook setup or active geometry-poll failure disables window riding with a one-time warning instead of crashing or pretending support.
- `--no-window-ride` disables the watcher until the M12 config surface exists.

### Cross-Platform Guardrails

- macOS, X11, and Wayland backends must report `watch_drag` and `move_window` honestly when they arrive.
- macOS support will need Accessibility-aware behavior and a real `.app` bundle.
- X11/XWayland can support fuller original-style behavior.
- Native Wayland remains an explicit degraded/no-op path for foreign-window control.
- Engine APIs continue to use signed world/desktop coordinates.

## Consequences

- M8 is testable headlessly at the engine level without importing Win32/CoreGraphics/X11/Wayland types.
- Windows can ship the first watch-and-ride implementation while non-Windows backends remain honest no-ops until their milestones.
- Future M9 window movement can extend the existing capability contract instead of replacing M8's snapshot path.
- Losing hook support or geometry access produces a clean resume/degrade path, not a stuck task.

## Verification

- Engine tests cover disabled and unsupported paths, drag-start interrupt, release before arrival, riding a moving anchor, capability-loss resume, interaction suppression, and negative-coordinate anchors.
- Windows helper tests cover signed rect conversion and basic invalid/own-window filtering.
- Local workspace checks must pass before M8 is moved to Done: formatting, clippy, workspace tests, release build, and installed cross-target checks where available.

## Follow-Ups

- M9: add autonomous collect-window behavior and window-move commands.
- M12: move `--no-window-ride` into the config/TUI surface.
- M15-M18: recheck watcher geometry and capabilities for per-monitor overlays, macOS, X11, and native Wayland degraded mode.
