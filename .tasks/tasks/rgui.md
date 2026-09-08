TT;DR: Native SDK graphical settings.

## Why

Deliver the user-approved refinement while preserving healthy installations and explicit platform claims.

## Scope

Separate Zig/native-markup settings app with a versioned bounded Rust stdio service and revision-aware shared config persistence. Keep TUI. Qualify all package identities.

Authoritative sources: approved September 7 implementation plan, docs/adr/0041-refinement-native-settings-and-continuous-goose.md, current source/tests, docs/refinement-audit.md.

Preserve saved user settings/media, verified installer ownership, immutable releases, terminal protection, and graceful stop. Native hardware beyond this Windows PC is unavailable; hosted proof is identified separately.

## Plan

Implement the linked board subtasks; retain later release stages as separate tasks. At each failed check record the exact failure and fix or leave the gate open.

## Impact

Separate Zig/native-markup settings app with a versioned bounded Rust stdio service and revision-aware shared config persistence. Keep TUI. Qualify all package identities.

## Acceptance

The requested behavior runs through its production path and the supporting evidence matches the claimed platform and installation.

## Verification

- [ ] Relevant production-path regression tests pass.

- [ ] Actual behavior or rendered output is inspected in its supported environment.

- [ ] Integration and required package/release checks pass before publication.

## Status

Active: Rust settings service and native window implemented; interactive and package qualification in progress.
Windows computer-use mouse/save and actual Makira font rendering passed with an isolated config. Native Windows UI Automation now exposes text, controls, dialog/edit focus, and saved status through the bounded AccessKit bridge. Keyboard editing and an isolated save passed. GTK/AT-SPI and final native qualification remain open.

## Activity

- 2026-09-07: Both Mac and GNU Linux settings builds, native window exercises and production identity checks pass. Musl builds compile but lacked Alpine's separate xvfb-run package; Windows ARM build runner crashed before diagnostics. Added the missing package, pinned x64 Zig host with native ARM64 safety-enabled tests, real Linux AT-SPI qualification and bounded text runs. AccessKit consumer reads Unicode/empty fields; five bridge tests, strict clippy and Windows GUI rebuild pass. Windows OS fixture now selects only actionable buttons and retains failure trees.

- 2026-09-07: user explicitly requested CLI/TUI update acceptance and GUI Check for updates / Update now controls; use the existing verified update lifecycle and typed results.

- 2026-09-07 — created from the user-approved implementation plan (agent: codex).


- 2026-09-07: Shared revision-aware save and bounded versioned stdio service cover all 53 editable fields; Rust protocol/validation/concurrency tests and strict clippy pass. Native settings build and real Windows automation load the five-page app with an isolated config. Layout, conflict, updater, lifecycle, and packaging qualification remain in progress.

- 2026-09-07: Added the bounded native accessibility bridge and independently verified Windows DLL payload. Actual Windows text/control/editor/save checks, Rust bridge and dual-file-lease regressions, production GUI build, and Python packaging tests pass. Updated native CI toolchains; hosted rerun pending.
