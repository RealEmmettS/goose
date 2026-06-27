# ADR 0004: M10 CLI/TUI Control Plane and Terminal Protection

- **Status:** Accepted
- **Date:** 2026-06-27
- **Milestone:** M10

## Context

M10 needs a reliable way to control one running goose instance before the three-name M11 CLI
grammar and the M12 config TUI arrive. Earlier planning allowed a keyboard quit path on platforms
that could support it, but that creates inconsistent behavior across Windows, macOS, X11, and
native Wayland and adds unnecessary global-input surface area.

M8 and M9 also introduced window-targeting behavior. That creates a user-safety rule that must be
platform-wide: terminal windows are normal surfaces for visual overlay, but they are never valid
mischief targets.

## Decision

Starting, stopping, poking, reloading, and future configuration are **CLI/TUI-only** over a local
single-instance control channel. There is no system tray and no global quit key.

The M10 command protocol is finite and local-only:

- `STOP`
- `RELOAD`
- `DO HONK|WANDER|MUD|MEME|NOTE|NAB`

The root binary owns command-line parsing and transport. `honk-engine` only sees closed,
platform-neutral command data such as `PokeAction` and `WorldOptions`.

Windows uses a per-user named mutex for singleton ownership and a per-user named pipe for commands.
Unix-family readiness uses a UID-scoped lock file plus Unix domain socket shape so macOS and Linux
can share the same command model when their overlay backends arrive.

Terminal windows are protected. The goose may render over terminal windows as part of the overlay,
but platform backends must exclude terminals before a window can become a target for foreign-window
ride, collect-window movement, synthetic typing, focus changes, drag/move behavior, or future
spicy/default-off behavior. This rule is absolute and does not have a prank-mode override.

## Consequences

- A plain second `honk300` invocation refuses to create a second goose and tells the user to use
  the command channel.
- `honk300 stop`, `honk300 reload`, and `honk300 do <action>` talk to the running instance instead
  of creating new overlays.
- M12 config and TUI work builds on M10 IPC rather than inventing another runtime control path.
- Native Wayland remains honest: unsupported desktop mischief degrades, but stop/reload/poke remain
  transport-level commands.
- New window-targeting features must use the same protected-window filter before touching OS
  handles, focus, movement, or synthetic input.

## Verification

M10 requires:

- Engine tests for poke actions, unsupported actions, busy/active-task behavior, and option reload.
- Protocol tests for valid commands plus malformed, oversized, unknown, and invalid payloads.
- CLI tests for default start, explicit start, stop, reload, and `do <action>`.
- Transport tests for singleton rejection, command round trips, unavailable server behavior, and
  local-only path/name generation where practical.
- Windows protected-window tests proving terminal-like classes/titles are excluded before
  foreign-window snapshots or future window-mischief targets are emitted.
- Full local gate and installed-target checks before moving M10 to Done.
