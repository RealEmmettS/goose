# AGENTS.md

This file provides guidance to Codex (Codex.ai/code) when working with code in this repository.

## What this repo is

A from-scratch, cross-platform (Windows/macOS/Linux) **Rust reimplementation of Desktop
Goose** (Samperson's desktop-pet). Target binary: **`honk300`** — a member of this machine's
`*300` tool family (siblings: TR300, ND300, WB300). `README.md` holds the one-paragraph brief.

**Current stage: implementation in progress.** M0-M14 are complete and M15 (multi-monitor
chase plus full recolor/appearance) is next. The repo now has a Cargo workspace, a platform-free
`honk-engine`, shared `honk-control`, versioned TOML `honk-config`, the `honk-config-tui`
terminal UI, a Windows platform crate, the `honk300` binary, the original app's files as
reference, the canonical planning docs, and ADRs under `docs/adr/`. M13's dynamic moods and
on-hour double honk use runtime-injected local time; M14's quiet-hours/DND/fullscreen manners and
built-in Autumn leaves use platform-neutral schedule/presence state.

## Read these first (source-of-truth pointers)

- `honk300_plan.md` — **the canonical, authoritative plan.** A claim-tested hybrid of the two
  drafts below, plus the approved new scope: architecture, build milestones **M0–M19**, the new
  autonomous behaviors, the ratatui `<name> config` TUI, the three-name (`honk300`/`honk`/`goose`)
  goose-speak CLI, the full all-OS/all-arch build matrix, packaging pipeline, locked decisions,
  and a ranked risk table. **Start here; where the three plans conflict, this one wins.**
- `claude_plan.md` — **superseded draft** (the structural spine of the hybrid). Reference only;
  its exact engine constants and Windows-overlay analysis were verified correct.
- `codex_plan.md` — **superseded draft** (grafts: richer task inventory, FirstUX, TOML, tests,
  `--purge`). Reference only; its Appendix-B speed *values* are wrong — use `honk300_plan.md`/`Exports.cs`.
- `DESKTOP-GOOSE/` — the **original closed-source app**, kept as reference (Windows
  `DesktopGoose v0.31/`, macOS `Desktop Goose for Mac v0.22/`).
- `DESKTOP-GOOSE/DesktopGoose v0.31/FOR MOD-MAKERS/GooseMod_DefaultSolution/GooseModdingAPI/{SamEngine.cs, Exports.cs}`
  — the shipped C# modding API. This is the **engine-port source-of-truth**: exact rig
  geometry, physics constants, the `Deck` RNG, and the Task/`InjectionPoints` model.
- Sibling repos `C:\Users\hey\git\qube-{machine-report,network-diagnostics,workbranch-view}`
  — the conventions to mirror: Cargo layout, `src/install/*`, `src/update.rs`, `build.rs`,
  `.github/workflows/windows-installers.yml`, and the dual-changelog discipline.
- `docs/adr/` — architecture decision records. Read these when a task touches platform
  boundaries, renderer architecture, capability traits, packaging targets, or milestone scope.
  ADR 0001 records the accepted M7 cursor-mischief contract and Renderer V2 direction; ADR 0002
  records the M8 foreign-window watch-and-ride contract; ADR 0003 records the M9 collect-window,
  asset, and no-donate decisions; ADR 0004 records the M10 CLI/TUI-only control plane, local IPC,
  and terminal-window protection rule; ADR 0007 records the M13 dynamic-mood and local-time
  injection contract; ADR 0008 records the M14 schedule/presence/Autumn contract.

## Big-picture architecture (original → planned port)

- **The goose is procedurally rendered, not a sprite** — there is no sprite art anywhere.
  It's drawn each frame from a geometric rig (body/neck/head/eyes/procedural feet) whose
  constants live in `Exports.cs`. The port reimplements this renderer clean-room (no asset
  extraction).
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
- Packaging: Windows-first 4-installer matrix (Global/Corporate × MSI/EXE) + shell/PowerShell
  installers + macOS `.app`/`.dmg` + Linux `.desktop`. **No crates.io.**
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
  a bare `~/.cargo/bin` binary can't hold one.
- The original `Deck` shuffle is **biased** (`System.Random`, low-bound 0 / exclusive high).
  Decide faithful-port vs. corrected and pin the choice with a test.

## Commands

This is a Rust **1.95** cargo workspace (edition 2021, the TR300/ND300 family default). The
family's local gate:

- `cargo fmt --all -- --check`
- `cargo clippy --all-targets --workspace -- -D warnings`
- `cargo test --workspace`  ·  single test: `cargo test -p honk-engine <name>`
- `cargo build --release`

Release packaging uses **cargo-dist** plus the hand-authored `windows-installers.yml` (adapt
from a sibling repo); **`crates-publish.yml` is intentionally dropped** (no crates.io).

## Asset & IP rule

`DESKTOP-GOOSE/` contains Samperson's / third-party copyrighted assets (memes, notes, sounds)
and old developer donation material. This is a personal-use repo, so M9 copies screened original
memes/notes into `Assets/` and adds one complete custom counterpart per original; do not
redistribute those assets publicly. The goose visual remains clean-room procedural. Do not ship
old donate pages or old developer references.

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
