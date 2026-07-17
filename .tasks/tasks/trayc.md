TT;DR: Define and build the one shared tray contract and icon pipeline that Windows and Linux will implement without changing the existing macOS behavior.

## Why

This is a direct operator order and the prerequisite for platform tray implementations. ADR 0028 already defines the macOS source behavior: one accessible goose control, Configure opens the existing terminal TUI, and Quit sends the existing graceful-stop intent. A shared contract prevents platform forks, abrupt exits, or duplicate settings models.

## Plan

Research maintained native tray options against the supported Rust 1.95 and Windows/Linux target matrix. Reuse Assets/UI/honk300-status-goose.svg as the canonical source, derive deterministic Windows/Linux representations, and require packaged resources. Define platform-neutral commands and lifetime rules so backends only own native UI integration. Record the accepted boundary in a new ADR rather than rewriting ADR 0028.

## Impact

Intended: Windows and Linux users receive the same discoverable Configure and Quit actions as macOS. Possible unintended impact: a tray dependency can add native library/runtime requirements, conflict with platform event pumps, disappear on shell restart, or make shutdown abrupt. The design must fail visibly but non-fatally when a tray is unavailable.

## Acceptance

One reviewed contract covers accessibility naming, Configure terminal/TUI restoration, graceful Quit, icon lifetime, shell restart, unavailable degradation, singleton behavior, permissions, packaging, and uninstall ownership. Platform implementations consume the same source assets and command semantics.

## Verification

- [x] Maintained dependency choices and supported target implications are documented from primary sources.
- [x] A new ADR records the shared contract, rejected alternatives, packaging shape, and failure behavior.
- [x] Automated tests prove Configure uses the existing config TUI and Quit cannot take an abrupt process-exit path.
- [x] Canonical icon derivation and package-presence checks pass for Windows and Linux artifacts.

## Status

Done. ADR 0030, the shared control enum/router, deterministic embedded icon representations,
Debian package ownership, and platform-neutral regression tests are complete. Windows and Linux
native qualification remain in their own tasks.

## Activity

- 2026-07-17 08:25 — completed all four subtasks. Native Win32 plus pure-Rust StatusNotifierItem
  boundaries are accepted, musl cross-check passes, and shared action/icon/package tests are in
  place. Debian package-tree execution remains correctly assigned to Linux because this
  non-elevated Windows host cannot create its Unix symlinks. (agent: codex)
- 2026-07-17 07:20 — moved To-Do → Active after immutable v1.0.3 and public verification closed
  `#r103`. Scope remains the four board subtasks only. (agent: codex)
- 2026-07-17 04:45 — created as the shared prerequisite for #wtray and #ltray under milestone #v110. (agent: codex)
