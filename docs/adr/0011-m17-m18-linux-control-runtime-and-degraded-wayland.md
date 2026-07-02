# 0011 — M17/M18 Linux Control Runtime And Degraded Wayland Foundation

## Status

Accepted. Extended by ADR 0012. This ADR records the Linux runtime/control foundation and the
honest degradation contract that must remain in place around the visible X11 and Wayland
presentation backends.

## Context

M16 added the macOS backend and the status protocol. Before Linux overlay work can claim parity,
Linux needs the same single-instance IPC, reload/status/poke loop, asset/audio loading, session
detection, terminal-target classifier, and explicit capability-state reporting that Windows and
macOS now have.

Linux has two materially different runtime surfaces:

- X11/XWayland can support visible overlay, input-region shaping, global pointer observation, and
  foreign-window metadata/move paths.
- Native Wayland intentionally withholds global cursor warping, synthetic input, and portable
  foreign-window enumeration/move. The M18 mode must therefore be reduced and explicit, not a
  hidden best-effort prank layer.

## Decision

- Add `crates/honk-platform-linux` for Linux-specific session detection, local-time sampling,
  fallback bounds, and terminal-window classification. `honk-engine` remains OS-free.
- Route Linux `start` through a real runtime loop instead of the old non-Windows placeholder.
- Reuse the existing Unix-domain-socket control transport for Linux `stop`, `reload`, `status`,
  and `do <action>`.
- Report Linux runtime capabilities through the same status protocol as Windows and macOS:
  unsupported/failed desktop-control features stay disabled in `WorldOptions`, while audio can be
  supported or failed independently.
- Default to X11/XWayland when `DISPLAY` exists, even inside a Wayland session. Native Wayland is
  selected only when `--wayland` or `[platform].wayland = true` asks for it, or when no X11 display
  is available.
- Keep terminal windows protected on Linux by classifying common terminal app names/classes before
  any future X11/Wayland foreign-window, collect-window, or spicy-behavior target can be emitted.
- Use command-line audio playback on Linux (`ffplay` or `mpv` if present) to avoid adding native
  audio-stack linker risk to cross-target checks. If no compatible player exists, report audio as
  failed and keep the runtime alive.

## Consequences

- Linux no longer starts as a placeholder: it can run as a single instance, answer status, reload
  config, stop over IPC, tick the platform-free engine, and play direct honks when a compatible
  player exists.
- Cursor warp, foreign-window ride, collect windows, and presence are currently reported as
  unsupported or failed by the Linux runtime until an X11 or Wayland display backend proves each
  capability.
- `do honk`, `do mud`, and `do wander` can be accepted by the degraded Linux runtime; `do nab`,
  `do meme`, and `do note` report unsupported because their backend capabilities are intentionally
  disabled.
- Full M17 closure still requires visible X11/XWayland overlay/input/window support and Linux-host
  smoke evidence.
- Full M18 closure still requires native Wayland reduced-mode rendering, IPC smoke, and explicit
  verification that unavailable mischief remains unsupported rather than crashing or hanging.

## Verification

- Unit coverage: Linux session detection, X11-first selection inside Wayland sessions, forced
  Wayland selection, unknown-display handling, fallback bounds, local-time shape, and terminal-app
  classification.
- Cross-target checks must continue to include Windows x64/ARM64, macOS x64/ARM64, Linux
  x64/ARM64 GNU, and Linux x64/ARM64 musl.
- Linux-host smoke remains required for both the M17 and M18 readiness passes.
