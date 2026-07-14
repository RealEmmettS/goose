# AGENTS.md

This file provides guidance to Codex (Codex.ai/code) when working with code in this repository.

## What this repo is

A from-scratch, cross-platform (Windows/macOS/Linux) **Rust reimplementation of Desktop
Goose** (Samperson's desktop-pet). Target binary: **`honk300`** — a member of this machine's
`*300` tool family (siblings: TR300, ND300, WB300). `README.md` holds the one-paragraph brief.

**Current stage: implementation in progress.** M0-M19 are implemented in-tree. M16.1 macOS
Accessibility onboarding is implemented and automated tests are green, while readiness remains
gated on exact signed-candidate denied, non-nagging relaunch, live-grant, and live-revocation
evidence. The repo now has a Cargo workspace, a platform-free
`honk-engine`, shared `honk-control`, versioned TOML `honk-config`, the `honk-config-tui`
terminal UI, Windows, macOS, and Linux platform crates, the `honk300` binary, the approved
built-in media catalog, canonical planning docs, and ADRs under `docs/adr/`. M13's dynamic moods and
on-hour double honk use runtime-injected local time; M14's quiet-hours/DND/fullscreen manners and
built-in Autumn leaves use platform-neutral schedule/presence state; M15's multi-monitor chase
uses signed virtual-desktop bounds and one Windows overlay HWND per monitor while appearance
recolor covers the six-tone V2 goose palette (ADR 0014). M16 adds macOS AppKit/CoreGraphics
runtime wiring, universal2 app staging, `honk300 status`, and a TUI Status tab. M17 adds the
Linux X11 visible overlay path with input shaping, pointer sampling/warp, terminal-filtered
foreign-window snapshots, and Unix IPC control. M18 adds native Wayland reduced mode with
layer-shell rendering, IPC control, and explicit unsupported reporting for blocked mischief.
Linux collect-window support remains unsupported and is reported honestly.
`docs/readiness/m16-m18-readiness.md` records the local gate, CI smoke gates, and pending macOS
Accessibility evidence log. M19 adds real `install`, `uninstall --purge`, and `update` code paths
plus cargo-dist shell/PowerShell installers, Linux archives, and Windows x64/ARM64
Global/Corporate MSI and EXE installers with sha256 sidecars; `#a8d` is closed from release
artifact evidence. Post-M19 rounds R1/R2 (ADRs 0014–0016) add the cross-platform reliability
contract (PMv2 DPI, non-blocking collect, flock singleton, macOS pump/present safety, Linux
loud-failure + `overlay` status, user-content-preserving uninstall), replace the renderer with
the flat-illustration dual-view Procedural Vector V2 goose (six-tone palette, reference art in
`docs/art-reference/`), and add idle-life behaviors (meandering walks, puddle-hop mud,
off-screen errands with prank returns) plus `exit`/`quit` stop synonyms.
R5/v0.3.1 (ADRs 0018–0019) is the distribution-readiness stabilization: config schema v2,
region-aware desktop layouts, bounded damage and shared runtime ordering, Concept C renderer,
platform/IPC hardening, Global MSI as the Windows default, an exact-tag transactional shell
installer for macOS/Linux, and one atomic immutable release workflow. v0.3.3/ADR 0020 adds the
first native macOS qualification, Developer ID signing/notarization, a per-user graphical DMG,
and shared gait refinement. ADR 0022 adds managed one-prompt-per-update Accessibility onboarding,
a calm safe-edge wait, and same-process grant/revocation transitions. The current release gate is
`docs/readiness/v0.3.3-readiness.md` and task `#m20q`. Stable/latest remains v0.3.2 until that
checklist is complete and the immutable v0.3.3 release is independently verified.

## Read these first (source-of-truth pointers)

- `honk300_plan.md` — **the canonical, authoritative plan.** A claim-tested hybrid of the two
  drafts below, plus the approved new scope: architecture, build milestones **M0–M19**, the new
  autonomous behaviors, the ratatui `<name> config` TUI, the three-name (`honk300`/`honk`/`goose`)
  goose-speak CLI, the full all-OS/all-arch build matrix, packaging pipeline, locked decisions,
  and a ranked risk table. **Start here; where the three plans conflict, this one wins.**
- `claude_plan.md` — **superseded draft** (the structural spine of the hybrid). Reference only;
  its exact engine constants and Windows-overlay analysis were verified correct.
- `codex_plan.md` — **superseded draft** (grafts: richer task inventory, FirstUX, TOML, tests,
  `--purge`). Reference only; its Appendix-B speed *values* are wrong — use the constants and
  invariant tests in `crates/honk-engine/src/{entity,rig,task}.rs`.
- `crates/honk-engine/src/` and `crates/honk-engine/tests/` — the active engine source of truth.
  The original research inputs were removed after the full custom rebuild; constants, rig
  geometry, shuffle behavior, and task ordering must now be changed only with their in-tree tests.
- Sibling repos `C:\Users\hey\git\qube-{machine-report,network-diagnostics,workbranch-view}`
  — the conventions to mirror: Cargo layout, `src/install/*`, `src/update.rs`, `build.rs`,
  `.github/workflows/windows-installers.yml`, and the dual-changelog discipline.
- `docs/adr/` — architecture decision records. Read these when a task touches platform
  boundaries, renderer architecture, capability traits, packaging targets, or milestone scope.
  ADR 0001 records the accepted M7 cursor-mischief contract and Renderer V2 direction; ADR 0002
  records the M8 foreign-window watch-and-ride contract; ADR 0003 records the M9 collect-window,
  asset, and no-donate decisions; ADR 0004 records the M10 CLI/TUI-only control plane, local IPC,
  and terminal-window protection rule; ADR 0007 records the M13 dynamic-mood and local-time
  injection contract; ADR 0008 records the M14 schedule/presence/Autumn contract; ADR 0009
  records the M15 multi-monitor/appearance contract; ADR 0010 records the M16 macOS agent-bundle,
  permission degradation, status protocol, and TUI-only control contract; ADR 0011 records the
  M17/M18 Linux control-runtime foundation and degraded Wayland contract; ADR 0012 records the
  M16.1-M18.1 CI-proven readiness contract; ADR 0013 records the M19 lifecycle/release contract
  and deferred macOS distribution slice; ADR 0014 records Renderer V2 (flat-illustration
  dual-view procedural vector — supersedes ADR 0001's sprite/atlas direction); ADR 0015 records
  the R1 reliability/platform-safety contract; ADR 0016 records the idle-life behaviors; ADR 0017
  records the historical R3 macOS packaging slice; ADR 0018 records atomic publication; ADR 0019
  records the v0.3.x stabilization contracts; ADR 0020 replaces only ADR 0018's macOS
  distribution decisions with Developer ID, notarization, and a DMG-first per-user install; ADR
  0021 defines portable native Wayland reduced mode plus explicit compositor capability strata;
  ADR 0022 records the managed macOS Accessibility first-run and live-transition boundary.

## Big-picture architecture (original → planned port)

- **The goose is procedurally rendered, not a sprite** — there is no sprite art anywhere.
  Renderer V2 (ADR 0014) draws it in the flat-illustration style of the project's own reference
  art (`docs/art-reference/`, design-time only; nothing loads at runtime), on the test-pinned rig
  in `crates/honk-engine/src/rig.rs`. The renderer is clean-room procedural (no asset extraction).
- **Engine = fixed 120 Hz tick + a Task state machine.** A default "roaming" state picks
  random tasks via a shuffle-bag (`Deck`); a task only sets `targetPos`/acceleration and the
  engine auto-locomotes toward it. Mod hooks fire Pre/Post Tick / UpdateRig / Render.
- **Rust layout (`honk300_plan.md` §7): a Cargo workspace** — a platform-agnostic `honk-engine`
  crate (`#![forbid(unsafe_code)]`, no OS deps), shared `honk-control`, `honk-config`, and
  `honk-config-tui` crates, plus capability-trait platform backends
  (`windows`/`macos`/`x11`/`wayland`). **One overlay window per monitor** (not one
  virtual-screen window); sim runs at 120 Hz, present is on-dirty/rate-capped.

## Locked decisions (do not re-litigate)

- Name `honk300` (binary `honk300`, optional `honk` alias); fresh permanent WiX/Inno GUIDs.
- Procedural/clean-room goose. Sounds bundled 1:1 (personal use). M9 bundles screened original
  meme/note assets 1:1 for personal-use builds **plus one complete custom in-house counterpart
  per original** in the clumsy MS Paint house style. User-supplied `Meme8.png` is approved.
  Old developer donation pages, Patreon links, social handles, and old-project branding do not
  ship.
- Linux: **X11-first** (runs under XWayland); native Wayland behind an opt-in `--wayland`
  flag (reduced mischief).
- Packaging: Windows recommends the x64/ARM64 machine-wide Global MSI. macOS recommends the
  Developer ID-signed, notarized, stapled universal DMG and installs per-user into
  `~/Applications`; the exact-tag shell bootstrap remains a secondary terminal path. Linux
  recommends that shell bootstrap. Corporate/EXE/portable artifacts remain secondary.
  **No crates.io.**
- Release-mode macOS artifacts must fail closed without Developer ID and App Store Connect API
  credentials. No ad-hoc release fallback, `codesign --deep` signing, or DMG `/Applications`
  symlink is permitted.
- Automatic macOS Accessibility UI is limited to the exact receipted app at
  `~/Applications/Honk300.app`. It prompts at most once per installed update, waits calmly at a
  safe screen edge while denied, and handles grants/revocations in the same process. Development,
  bare, source-tree, and mounted-DMG launches must not open permission UI automatically.
- macOS transparent presentation uses reusable premultiplied-RGBA AppKit image views in the
  ordinary window backing store. Keep canvas/bitmap capacity bounded after transiently large
  damage so screen capture stays alpha-correct and normal walking does not redraw stale space.
  Keep the Device RGB bitmap and overlay-window color spaces aligned; final display-profile
  composition belongs to WindowServer, not a per-frame application-side ICC conversion.
- Starting, stopping, and configuration are **CLI/TUI-only over local IPC**. There is no system
  tray and no global quit key.
- Terminal windows are protected: the goose may visually overlay them, but must never move,
  focus, type into, drag, ride, collect, or otherwise manipulate terminal windows, including in
  spicy/default-off modes.

## Architecture decision records

- Add or update ADRs in `docs/adr/` whenever a change affects platform boundaries, the
  engine/backend contract, renderer architecture, deployment targets, packaging shape,
  permissions, or milestone scope.
- Use a new numbered ADR for changed decisions instead of rewriting history. Mark older ADRs
  as superseded only when a new accepted ADR replaces them.
- Keep ADRs in sync with `README.md`, this file, `CLAUDE.md`, `.tasks/`, `CHANGELOG.md`, and
  `HUMAN_CHANGELOG.md` when they change current guidance.
- M7's accepted decisions live in `docs/adr/0001-m7-cursor-mischief-renderer-and-platform-guardrails.md`.
- M8's accepted decisions live in `docs/adr/0002-m8-foreign-window-watch-and-ride.md`.
- M9's accepted decisions live in `docs/adr/0003-m9-collect-window-assets-and-no-donate.md`.
- M10's accepted decisions live in `docs/adr/0004-m10-cli-tui-control-plane-and-terminal-protection.md`.
- M13's accepted decisions live in `docs/adr/0007-m13-moods-and-local-time-injection.md`.
- M14's accepted decisions live in `docs/adr/0008-m14-schedule-presence-and-autumn.md`.
- M15's accepted decisions live in `docs/adr/0009-m15-multi-monitor-and-appearance.md`.
- M16's accepted decisions live in `docs/adr/0010-m16-macos-backend-agent-bundle-and-tui-status.md`.
- M17/M18's Linux control-runtime foundation lives in `docs/adr/0011-m17-m18-linux-control-runtime-and-degraded-wayland.md`.
- M16.1-M18.1's CI-proven readiness contract lives in `docs/adr/0012-m16-1-m18-1-ci-proven-backend-readiness.md`.
- M19's lifecycle/release split and macOS packaging deferral live in `docs/adr/0013-m19-lifecycle-packaging-and-deferred-macos-distribution.md`.
- Renderer V2's flat-illustration dual-view direction lives in `docs/adr/0014-renderer-v2-flat-illustration-dual-view.md`.
- R1's reliability/platform-safety contract lives in `docs/adr/0015-reliability-and-platform-safety-fixes.md`.
- R2's idle-life behaviors live in `docs/adr/0016-idle-life-behaviors-meander-mud-excursions.md`.
- R3's macOS packaging + lifecycle slice (universal2 `.app`/DMG, macOS `install`/`uninstall`/`update`, unsigned personal-use; supersedes ADR 0013's macOS deferral) lives in `docs/adr/0017-macos-packaging-and-lifecycle.md`.
- v0.3.x distribution/atomic publication lives in `docs/adr/0018-distribution-and-atomic-release.md`.
- v0.3.x config/runtime/renderer/platform contracts live in `docs/adr/0019-stabilization-contracts.md`.
- Developer ID macOS distribution and the per-user graphical DMG live in
  `docs/adr/0020-macos-developer-id-dmg-distribution.md`.
- Native Wayland's portable base and portal/KDE/GNOME/wlroots adapter boundaries live in
  `docs/adr/0021-native-wayland-capability-strata.md`.
- Managed macOS Accessibility first-run onboarding lives in
  `docs/adr/0022-macos-accessibility-first-run-onboarding.md`.

## Task management system

This repo uses the SHAUGHV `tasks-*` system. The board source of truth is `.tasks/TASKS.md`;
each task's rich handoff lives at `.tasks/tasks/<id>.md` with `## Status` and `## Activity`
kept current while work is in flight.

Use proper subtasks for small required steps that should be visible and checkable in the
dashboard modal: indented checkbox rows under the parent task in `.tasks/TASKS.md`, optionally
followed by indented description lines (`    > detail for this subtask`). Do not bury those
board-trackable steps as plain text in the parent task description, and do not call them
"sub-items." Use the parent description for reasoning, context, plan, impact, acceptance, and
resume notes. If related work is large enough to need its own status, activity log, or owner,
make it a separate top-level task and link it with `(needs #id)`.

Relevant skills: `tasks-start`, `tasks-management`, `tasks-update`, `tasks-memory`,
`tasks-remove`. Companion skills such as `ttdr`, `personal-productivity`, `iterative-plan`, or
`git-workflow` are optional if installed.

## Gotchas (cross-platform overlay / desktop-pet)

- **softbuffer cannot do per-pixel alpha on a Windows layered window** — present via
  `UpdateLayeredWindow` directly; softbuffer is X11/Wayland-only.
- **Click-through vs. clickable** — use per-pixel-alpha natural hit-testing (do *not* set
  `WS_EX_TRANSPARENT`); on X11 set the XShape input region to the goose bbox each frame.
- **A normal native Wayland client cannot provide the core mischief portably** (moving other
  windows, global pointer warp, synthetic input). ADR 0021 permits explicit portal or
  compositor-specific adapters, but the portable layer-shell path stays honestly reduced.
- **Terminal windows are never mischief targets.** Backend filters must exclude terminal windows
  before foreign-window ride, collect-window, or future spicy behavior code can target them.
- **macOS needs a real `.app` bundle** (stable bundle-id) for a durable Accessibility grant;
  a bare `~/.cargo/bin` binary can't hold one. The bundle is an LSUIElement agent/permission
  identity only: no native preferences window, menu-bar settings UI, Dock control surface, or
  AppleScript `.sdef` command surface.
- **macOS permission prompting is a managed-install privilege.** Record the owner-only
  per-update marker before opening native UI; if eligibility or secure state fails, retain the
  existing denied/degraded runtime without prompting. A denied managed app enters the engine's
  permission wait, while Windows and Linux never activate that task.
- The original `Deck` shuffle is **biased** (`System.Random`, low-bound 0 / exclusive high).
  Decide faithful-port vs. corrected and pin the choice with a test.

## Commands

This is a Rust **1.95** cargo workspace (edition 2021, the TR300/ND300 family default). The
family's local gate:

- `cargo fmt --all -- --check`
- `cargo clippy --all-targets --workspace -- -D warnings`
- `cargo test --workspace`  ·  single test: `cargo test -p honk-engine <name>`
- `cargo build --release`
- `dist plan --tag=v0.3.3`
- `cargo audit --version 0.22.2`

Release packaging uses **cargo-dist** for portable archives plus project-owned atomic release,
Windows installer, macOS bundle, and bootstrap workflows; **`crates-publish.yml` is intentionally
dropped** (no crates.io).

## Asset & IP rule

`Assets/` contains the approved built-in media catalog. Treat entries identified in
`THIRD_PARTY_ASSETS.md` as personal-use compatibility media: do not assert redistribution rights
or publish them separately. Mutable user media belongs in platform user-data directories, never
inside binaries, MSI-owned directories, or the sealed macOS bundle. The goose visual remains
clean-room procedural; old donation pages and developer branding do not ship.

## Changelog rule

This repo maintains two changelogs in parallel:

- `CHANGELOG.md` — the technical changelog (Keep a Changelog style). Version numbers, file
  references, and details are welcome here.
- `HUMAN_CHANGELOG.md` — a plain-English companion. Every entry in `CHANGELOG.md` has a
  matching entry here for a non-engineer reader: no version numbers, no code references, no
  jargon — just what changed and why it matters.

**When you update `CHANGELOG.md`, you must update `HUMAN_CHANGELOG.md` in the same commit.**
Translate each entry by stripping version numbers, paths, symbol names, metrics, and PR/issue
numbers; replace jargon with everyday words; add a short "why it matters" clause. Use the
labels Added / Improved / Fixed / Removed / Security / Behind the scenes. Purely internal
changes still get a one-line "Behind the scenes" entry — the two files stay in lockstep.
