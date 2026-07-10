# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## What this repo is

A from-scratch, cross-platform (Windows/macOS/Linux) **Rust reimplementation of Desktop
Goose** (Samperson's desktop-pet). Target binary: **`honk300`** — a member of this machine's
`*300` tool family (siblings: TR300, ND300, WB300). `README.md` holds the one-paragraph brief.

**Current stage: implementation in progress.** M0-M19 are implemented in-tree, and M16.1 macOS
Accessibility readiness remains gated on pre-granted host evidence rather than Windows-host
claims. The Cargo workspace exists: `honk-engine` (platform-free
core), `honk-control` (shared IPC protocol/client/server), `honk-config` (versioned TOML
configuration), `honk-config-tui` (ratatui reducer UI), `honk-platform-windows` (the layered
overlay), `honk-platform-macos` (AppKit/CoreGraphics backend), `honk-platform-linux` (Linux
control/session helpers), and the root `honk300` binary — a
procedurally-rendered goose roams a transparent Windows overlay, leaves mud trails, runs the
task/FirstUX AI, honks, reacts to pat/click input, can perform bounded cursor nabbing, and can
perch on a user-dragged foreign window, and can collect Notepad/meme windows on Windows.
M10 adds single-instance local control for `start`, `stop`, `reload`, and `do <action>` pokes;
M11 adds the three-name goose-speak grammar; M12 adds durable config and the terminal UI; M13
adds dynamic moods and the local on-hour double honk; M14 adds quiet-hours/DND/fullscreen calm
suppression and built-in procedural Autumn leaves; M15 adds signed virtual-desktop multi-monitor
chase, one Windows overlay HWND per monitor, live Calm Goose, and RGB palette editing (expanded
to the six-tone V2 palette by ADR 0014); M16 adds macOS runtime wiring, `honk300 status`, a TUI Status tab,
bundle-aware assets/start handling, and `script/package_macos_app.sh`; M17 adds the Linux X11
visible overlay path with input shaping, pointer sampling/warp, terminal-filtered foreign-window
snapshots, and Unix IPC control; M18 adds native Wayland reduced mode with layer-shell rendering,
IPC control, and explicit unsupported reporting for blocked mischief. Linux collect-window support
remains unsupported and is reported honestly. `docs/readiness/m16-m18-readiness.md` records the
local gate, CI smoke gates, and pending macOS Accessibility evidence log. M19 adds real
`install`, `uninstall --purge`, and `update` code paths plus cargo-dist shell/PowerShell
installers, Linux archives, and Windows x64/ARM64 Global/Corporate MSI and EXE installers with
sha256 sidecars; `#a8d` is closed from release artifact evidence. Post-M19 rounds **R1/R2**
(ADRs 0014–0016) add Per-Monitor-V2 DPI awareness and the cross-platform reliability contract
(non-blocking collect, Notepad lifecycle, Unix flock singleton, macOS event-drain/present
safety, Linux loud-failure + `overlay` status capability, user-content-preserving uninstall);
replace the renderer with **Procedural Vector V2** — the flat-illustration dual-view goose
adapted from the project's own reference art (`docs/art-reference/`), with plant-and-swing
feet, a six-tone configurable palette, and blink/breath/tail secondary motion; and add the
idle-life behaviors (meandering walks, story-driven mud via off-screen puddle hops, timed
off-screen errands with prank returns) plus `exit`/`quit` stop synonyms.
`honk300_plan.md` is the canonical plan (milestones M0–M19); the two superseded drafts remain as
reference.
R5/v0.3.0 (ADRs 0018–0019) is the distribution-readiness stabilization: config schema v2,
region-aware desktop layouts, bounded damage and shared runtime ordering, Concept C renderer,
platform/IPC hardening, Global MSI as the Windows default, an exact-tag transactional shell
installer for macOS/Linux, and one atomic immutable release workflow. The current release gate is
`docs/readiness/v0.3.0-readiness.md` and task `#r5s`.

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
  the R1 reliability/platform-safety contract (amends ADR 0013's uninstall semantics); ADR 0016
  records the idle-life behaviors (meander, puddle-hop mud, off-screen errands); ADR 0017 records
  the historical R3 macOS packaging slice; ADR 0018 supersedes its advertised-DMG and
  release-mutation decisions; ADR 0019 records the v0.3.0 stabilization contracts.

## Big-picture architecture (original → planned port)

- **The goose is procedurally rendered, not a sprite** — there is no sprite art anywhere.
  It's drawn each frame from the test-pinned geometric rig in `crates/honk-engine/src/rig.rs`.
  Renderer V2 (ADR 0014) draws it in the flat-illustration style of the **project's own
  reference art** (`docs/art-reference/`, Emmett's generated SVGs — design-time reference only;
  path geometry was transcribed into code and no image assets load at runtime). The original
  research inputs were removed after the clean-room rebuild.
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
- Procedural/clean-room goose (no sprite extraction — the visual is drawn from the rig).
  Original sounds, screened original memes, and screened original notes are bundled 1:1 for
  personal-use builds. M9 also adds **one complete custom in-house counterpart per copied
  meme/note original** in the clumsy MS Paint house style. User-supplied `Meme8.png` is approved.
  Old developer donation pages, Patreon links, social handles, and old-project branding do not
  ship.
- Linux: **X11-first** (runs under XWayland); native Wayland behind an opt-in `--wayland`
  flag (reduced mischief).
- Packaging: Windows recommends the x64/ARM64 machine-wide Global MSI. macOS/Linux recommend the
  exact-tag, hash-verifying shell bootstrap; macOS receives a universal2 app in
  `~/Applications`. Corporate/EXE/portable artifacts and the v0.2.1 compatibility DMG remain
  secondary. **No crates.io.**
- macOS artifacts are ad-hoc signed and not notarized. Documentation must not imply that terminal
  installation replaces Gatekeeper approval, Developer ID/notarization, or durable Accessibility
  grant identity.
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
- Keep ADRs in sync with `README.md`, this file, `AGENTS.md`, `.tasks/`, `CHANGELOG.md`, and
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
- v0.3.0 distribution/atomic publication lives in `docs/adr/0018-distribution-and-atomic-release.md`.
- v0.3.0 config/runtime/renderer/platform contracts live in `docs/adr/0019-stabilization-contracts.md`.

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
- **Native Wayland makes the core mischief impossible** (moving other windows, warping the
  cursor, synthesizing keystrokes) — by design. These degrade to no-ops;
  document, don't fight.
- **Terminal windows are never mischief targets.** Backend filters must exclude terminal windows
  before foreign-window ride, collect-window, or future spicy behavior code can target them.
- **macOS needs a real `.app` bundle** (stable bundle-id) for a durable Accessibility grant;
  a bare `~/.cargo/bin` binary can't hold one. The bundle is an LSUIElement agent/permission
  identity only: no native preferences window, menu-bar settings UI, Dock control surface, or
  AppleScript `.sdef` command surface.
- The original `Deck` shuffle is **biased** (`System.Random`, low-bound 0 / exclusive high).
  Decide faithful-port vs. corrected and pin the choice with a test.

## Commands

This is a Rust **1.95** cargo workspace (edition 2021, the TR300/ND300 family default). The
family's local gate:

- `cargo fmt --all -- --check`
- `cargo clippy --all-targets --workspace -- -D warnings`
- `cargo test --workspace`  ·  single test: `cargo test -p honk-engine <name>`
- `cargo build --release`
- `dist plan --tag=v0.3.0`
- `cargo audit`

Release packaging uses **cargo-dist** for portable archives plus project-owned atomic release,
Windows installer, macOS bundle, and bootstrap workflows; **`crates-publish.yml` is intentionally
dropped** (no crates.io).

## Asset & IP rule

Approved personal-use compatibility media are bundled in `Assets/`, with project-created
counterparts where recorded. Redistribution status is documented per entry in
`THIRD_PARTY_ASSETS.md`; do not infer rights from repository presence. The goose **visual** is
clean-room procedural (drawn from the
rig, no sprite art exists to extract). Renderer V2's look is adapted from **Emmett's own
generated reference SVGs** in `docs/art-reference/` (ADR 0014): adapting their path geometry
into code is permitted, and they never ship or load at runtime. Mutable user media remains
outside binaries and installer-owned locations. Do not ship old donate pages or developer
references.

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
