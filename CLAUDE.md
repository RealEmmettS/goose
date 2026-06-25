# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## What this repo is

A from-scratch, cross-platform (Windows/macOS/Linux) **Rust reimplementation of Desktop
Goose** (Samperson's desktop-pet). Target binary: **`honk300`** — a member of this machine's
`*300` tool family (siblings: TR300, ND300, WB300). `README.md` holds the one-paragraph brief.

**Current stage: implementation in progress (milestones M0–M5 done).** The Cargo workspace
exists: `honk-engine` (platform-free core), `honk-platform-windows` (the layered overlay),
and the root `honk300` binary — a procedurally-rendered goose roams a transparent Windows
overlay, leaves mud trails, runs a task/FirstUX AI, and honks (rodio). `honk300_plan.md` is
the canonical plan (milestones M0–M19); the two superseded drafts remain as reference.

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

## Big-picture architecture (original → planned port)

- **The goose is procedurally rendered, not a sprite** — there is no sprite art anywhere.
  It's drawn each frame from a geometric rig (body/neck/head/eyes/procedural feet) whose
  constants live in `Exports.cs`. The port reimplements this renderer clean-room (no asset
  extraction).
- **Engine = fixed 120 Hz tick + a Task state machine.** A default "roaming" state picks
  random tasks via a shuffle-bag (`Deck`); a task only sets `targetPos`/acceleration and the
  engine auto-locomotes toward it. Mod hooks fire Pre/Post Tick / UpdateRig / Render.
- **Planned Rust layout (`honk300_plan.md` §7): a Cargo workspace** — a platform-agnostic `honk-engine`
  crate (`#![forbid(unsafe_code)]`, no OS deps) plus capability-trait platform backends
  (`windows`/`macos`/`x11`/`wayland`). **One overlay window per monitor** (not one
  virtual-screen window); sim runs at 120 Hz, present is on-dirty/rate-capped.

## Locked decisions (do not re-litigate)

- Name `honk300` (binary `honk300`, optional `honk` alias); fresh permanent WiX/Inno GUIDs.
- Procedural/clean-room goose (no sprite extraction — the visual is drawn from the rig).
  **All other original assets are bundled 1:1 and committed** to `Assets/` (sounds now;
  memes + notes at M9): this is a **personal tool the owner self-distributes to his own
  machines only**, so the earlier "regenerate memes / author fresh notes" plan is
  **superseded — copy the originals.** (Not for public redistribution.)
- Linux: **X11-first** (runs under XWayland); native Wayland behind an opt-in `--wayland`
  flag (reduced mischief).
- Packaging: Windows-first 4-installer matrix (Global/Corporate × MSI/EXE) + shell/PowerShell
  installers + macOS `.app`/`.dmg` + Linux `.desktop`. **No crates.io.**

## Gotchas (cross-platform overlay / desktop-pet)

- **softbuffer cannot do per-pixel alpha on a Windows layered window** — present via
  `UpdateLayeredWindow` directly; softbuffer is X11/Wayland-only.
- **Click-through vs. clickable** — use per-pixel-alpha natural hit-testing (do *not* set
  `WS_EX_TRANSPARENT`); on X11 set the XShape input region to the goose bbox each frame.
- **Native Wayland makes the core mischief impossible** (moving other windows, warping the
  cursor, synthesizing keystrokes, global key grab) — by design. These degrade to no-ops;
  document, don't fight.
- **macOS needs a real `.app` bundle** (stable bundle-id) for a durable Accessibility grant;
  a bare `~/.cargo/bin` binary can't hold one.
- The original `Deck` shuffle is **biased** (`System.Random`, low-bound 0 / exclusive high).
  Decide faithful-port vs. corrected and pin the choice with a test.

## Commands

No build system exists yet. When implementation begins it is a Rust **1.95** cargo workspace
(edition 2021, the TR300/ND300 family default). The family's local gate:

- `cargo fmt --all -- --check`
- `cargo clippy --all-targets --workspace -- -D warnings`
- `cargo test --workspace`  ·  single test: `cargo test -p honk-engine <name>`
- `cargo build --release`

Release packaging uses **cargo-dist** plus the hand-authored `windows-installers.yml` (adapt
from a sibling repo); **`crates-publish.yml` is intentionally dropped** (no crates.io).

## Asset & IP rule

This is a **personal tool the owner builds for and self-distributes to his own machines
only** — not a public release. On that basis the original assets in `DESKTOP-GOOSE/` are
**bundled 1:1 into `Assets/` and committed** (sounds done in M5; memes + notes copied at M9).
The goose **visual** is still clean-room procedural (drawn from the rig, no sprite art exists
to extract). Do **not** publicly redistribute these bundled third-party assets.

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
