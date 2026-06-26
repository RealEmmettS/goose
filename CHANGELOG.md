# Changelog

All notable changes to this project are documented here. Format based on
[Keep a Changelog](https://keepachangelog.com/); the project will adopt
[Semantic Versioning](https://semver.org/) once it produces releasable artifacts.

> **Project stage: implementation in progress.** Milestones M0-M7 are complete and M8 is
> active. The goose now renders, walks, leaves mud, plays sounds, reacts to the cursor, and can
> perform bounded cursor-nab mischief on Windows; there is no release yet. A plain-English companion lives in
> [HUMAN_CHANGELOG.md](./HUMAN_CHANGELOG.md) and must stay in lockstep.

## [Unreleased]

### Added
- **Cursor mischief: warp + nab sub-states (milestone M7, complete)** — the goose can now steal
  the real cursor on Windows in a bounded, recoverable way. `honk-engine` remains platform-free:
  it owns `CursorCommand::WarpTo(Vec2)`, `MouseStealOptions`, `WorldOptions`, and the
  `NabMouseTask` state machine; platform backends drain cursor commands after each fixed-tick
  update. `TaskCtx` now carries the current platform-neutral pointer plus a cursor-command
  queue, so tasks can request cursor motion without importing Win32/CoreGraphics/X11/Wayland
  APIs. `NabMouseTask` is randomly pickable only when mouse stealing is enabled and the backend
  reports cursor-warp support. A click on the goose also starts `NabMouseTask` when supported;
  when mouse stealing is disabled or unsupported, the older M6 click-to-hyper burst remains the
  fallback. The nab lifecycle seeks the live pointer at charge speed, bites once when the beak
  reaches `grab_distance`, captures the beak-to-cursor offset, then runs a bounded HYPR-style
  retargeting burst while keeping the cursor anchored to the beak until `succ_time` or a
  pull-away/drop threshold ends the grab. While nab owns the cursor, M6 pat/click handling is
  suppressed so synthetic cursor movement does not spawn hearts or interrupt into `HyperTask`.
  The Windows backend now exposes a cursor-warp wrapper, applies only the newest warp command
  after ticking, warns once if warping fails, marks cursor warp unavailable on failure, and the
  binary adds `--no-mouse-steal` as an opt-out. M7 added regression coverage for disabled and
  unsupported paths, click-to-nab, fallback click-to-hyper, seek/grab/drag/drop/timeout, one bite
  sound, drag-offset preservation, HYPR-style retargeting, deterministic command draining, and
  M6 interaction suppression during nab. The full local gate and release build passed before M7
  was moved to Done.
- **M7.0/M7.1/M7.2 completion work** — M7 now includes the completed-milestone audit, the
  mandatory cross-platform `honk-engine` readiness pass, and the renderer/runtime architecture
  spike. The M7.0 audit rechecked M0-M6 against `honk300_plan.md`, fixed stale status docs, and
  created follow-up `#p4d` for fullscreen overlay present-cost measurement. The M7.1 readiness
  pass confirmed the engine stayed platform-free and that current target coverage still respects
  Windows x64/ARM64, macOS Intel/Apple Silicon, Linux x64/ARM GNU, and Linux x64/ARM musl
  expectations. The M7.2 spike selected a future custom CPU sprite/atlas renderer and split that
  implementation into backlog task `#r2v`.
- **Architecture decision records** — added `docs/adr/` with ADR 0001, recording the accepted M7
  cursor-mischief contract, Windows runtime behavior, cross-platform guardrails, renderer
  direction, consequences, verification, and follow-up tasks. `AGENTS.md` and `CLAUDE.md` now
  include ADR maintenance rules so future architecture changes update ADRs, task memory, docs,
  and both changelogs together.
- **Hit-testing: pat (hover-streak + hearts) + click→hyper (milestone M6)** — the goose
  reacts to the cursor. Two distinct interactions (plan §5.9 / §6), built on a new per-frame
  pointer feed (`World::set_pointer` taking a platform-free `interaction::Pointer`; the
  Windows backend polls `GetCursorPos` + `GetAsyncKeyState`). **Pat** = repeated cursor
  *hover-sweeps* over the goose (no buttons): a `PatTracker` accumulates hover-movement into
  a happy streak, each registered pat spawns a rising/fading **heart particle** (new
  `honk-engine::hearts` module + `render::render_hearts`, a clean-room procedural heart) and
  keeps the goose briefly **calm** (a content goose suppresses its spontaneous honks). **Click**
  = a left-press on the goose → a charge-speed **hyper** burst (`task::HyperTask`) that bolts
  around erratically and honks, installed as a transient interrupt that **saves and restores
  the task it suspended** (the resume mechanism perch-and-ride will reuse in M8). Hit-testing
  uses the rig bounding box (`Rect::contains`), naturally click-through everywhere else.
  Engine-side logic is fully unit-tested; the on-screen result was verified visually.
- **Audio (milestone M5)** — the goose honks. A `rodio` backend in the binary plays the
  bundled original sounds (Honk1–4, BITE, MudSquith, Pat1–3) mapped from platform-free
  `Sound` requests the engine emits (`honk-engine::sound::Sound` + a `World` queue drained
  each frame). The goose honks on wander-retarget and squelches while tracking mud.
  `--no-sound` / `--silent` mutes it (the original `SilenceSounds`); a missing audio device
  degrades to a silent no-op. Sounds are embedded via `include_bytes!` from `Assets/Sounds/`.
  Audio is Windows-scoped this round (the macOS/Linux backends wire it in M16/M17).
- **Task state machine + wander + FirstUX intro (milestone M4)** — the M2 roam stand-in is
  replaced by the real AI. A `Task` trait (the documented internal extension seam, plan §18 —
  no external mod ABI), a `TaskCtx`, a registry of randomly-pickable tasks chosen via the
  biased `Deck`, and a `World` task runner. Tasks set targets/params only; the engine
  auto-locomotes. Ships `WanderTask` (roam to random points for a verified 20–40 s dwell, with
  occasional mud-tracking folded in) and a scripted `FirstUxTask` (the goose walks on-stage
  from off the bottom edge and pauses to introduce itself for the verified 20 s
  `FirstWanderTime`, then hands off to roaming). Timings are the verified `config.ini` values
  (20 / 20 / 40); config-driven values arrive with the TOML loader in a later round.
- **Footmarks + mud trail (milestone M3)** — the goose leaves a trail of fading muddy
  footprints while it's "tracking mud," at the verified lifetimes (8.5 s life / 1 s
  shrink-out). To render world-space trails the overlay moved from the small per-goose
  window to a **fullscreen primary-monitor layered overlay** (the plan's intended
  per-monitor architecture; multi-monitor traversal is M15). The engine drops an
  alternating-foot print at each gait half-step while tracking mud; the M2 roam driver
  triggers mud-tracking periodically (M4's `Task_TrackMud` will formalize the trigger).
  Present is capped a touch lower (~40 Hz) since a fullscreen layered present is heavier;
  a dirty-rect optimization (`UpdateLayeredWindowIndirect` + `prcDirty`) is a future perf task.

### Improved
- **Goose look reworked toward the real original — from direct observation and review.** The
  published modding API documents the rig *model* but not the `updateRig`/`Render` maths (closed
  binary; not decompiled, per the clean-room rule), so the goose was re-grounded by running the
  original Desktop Goose and capturing a local reference screenshot, then iterating against
  golden-frame previews and visual-smoke captures. A generated-sprite-style wing-panel/tall-neck
  pass was saved only as a local visual backup and rejected because it drifted from the original's
  charm. The accepted M7 renderer now uses a deliberate single Bezier body silhouette instead of
  stacked capsules, a flatter/thinner oval body closer to the original side-profile mass, the
  neck drawn under the body/head to hide construction seams, a small plain eye instead of a
  ringed cartoon eye, a short rounded orange beak, fuller tiny orange feet, a subtle dotted
  ground shadow, and updated golden frames for rest/reaching/mid-stride. This remains a
  clean-room procedural renderer; Renderer V2 owns the future atlas-based art pipeline.
- **Windows overlay + walking goose (milestones M1 + M2)** — `honk300` now renders the
  procedural goose on a transparent, always-on-top, click-through-where-transparent overlay
  and walks it around the desktop.
  - **Engine (platform-free, tested):** a fixed-120 Hz `Accumulator` (catch-up clamped to
    avoid the spiral of death); clean-room `locomotion` (accelerate toward `target_pos`,
    cap at the speed tier, face the travel direction, stop cleanly on arrival); a `World`
    with a minimal **roam driver** (a temporary stand-in for the M4 task/AI system); and a
    distance-driven **procedural-feet gait** with a subtle body bob.
  - **Windows backend (`honk-platform-windows`):** a layered popup window presented via
    `UpdateLayeredWindow` with premultiplied BGRA (softbuffer can't do per-pixel alpha on a
    Windows layered window). The small window is repositioned every frame, so it *is* the
    dirty rect — present cost stays proportional to the goose, not the screen. `WS_EX_LAYERED`
    **without** `WS_EX_TRANSPARENT` gives natural per-pixel-alpha click-through (opaque goose
    clickable, transparent margins fall through). This presenter shape was superseded by the
    M3 fullscreen primary-monitor overlay so mud/heart/world-space props can render in-place;
    the M7.0 audit tracks dirty-rect/per-monitor optimization as follow-up work.
  - **Renderer reworked to the original's technique:** capsule body / neck / two-segment
    head, an orange beak and webbed feet, a grey outline, and a ground shadow — tuned to
    resemble the real side-profile goose, animated by the neck-lerp + gait + body bob.
  - **Root `honk300` binary:** the three-clock loop (sim 120 Hz, present ~60 Hz on the
    goose's bounding box). Golden frames re-blessed (rest / reaching / mid-stride).
  - **Design note (deviation from plan §4):** the overlay uses raw Win32 (the `windows`
    crate) rather than winit — a small moving layered window via `UpdateLayeredWindow` is the
    canonical low-CPU desktop-pet pattern, and per-backend windowing fits the capability-trait
    design. winit can be revisited at M15 (multi-monitor) / M16 (cross-platform loop). The
    workspace root is now also the `honk300` binary package; added the `honk-platform-windows`
    crate.
- **Cargo workspace + `honk-engine` crate (milestone M0)** — the platform-free
  simulation core: `#![forbid(unsafe_code)]`, no windowing/OS/audio/input dependency,
  fully headless-testable. Ported 1:1 from the verified modding-API source
  (`…/GooseModdingAPI/{Exports.cs, SamEngine.cs}`): `Vec2` + `SamMath`; the 120 Hz
  fixed-timestep constants (`DT = 1/120`); the **faithful biased** `Deck` shuffle-bag
  (decision C8 — a seedable SplitMix64 drives it for deterministic tests; RNG internals
  are clean-room); `GooseEntity` + `ParametersTable` at the verified values (Walk/Run/
  Charge 80/200/400, accel 1300/2300, step 0.2/0.1, mud 15); the rig geometry constants
  with a clean-room `update_rig`; `ProceduralFeet`; the 64-slot `FootMarks` ring buffer
  (lifetime 8.5 s / shrink 1 s); and a clean-room tiny-skia renderer (`Rig → Pixmap`
  with a dirty-rect bounding box). Pinned by 26 unit tests (constants, rig vertices, the
  exact `Deck` sequence + its documented bias, footmark lifetimes) and 3 committed
  golden-frame PNGs. The renderer's proportions are a first clean-room approximation —
  the goldens are a regression baseline, not a fidelity reference (visual tuning is M1+).
- **Workspace scaffold** — root `Cargo.toml` (workspace, edition 2021 / Rust 1.95 via
  `[workspace.package]`, `[profile.dist]`), `rust-toolchain.toml` pinned to 1.95, and
  `crates/honk-engine/Cargo.toml`. The `[workspace.metadata.dist]` / WiX / CI blocks are
  intentionally deferred to the M19 packaging round. Local gate is green
  (`fmt --check`, `clippy -D warnings`, `test --workspace`, `build --release`).
- `honk300_plan.md` — **the canonical, authoritative plan.** A claim-tested *hybrid* that
  synthesizes `claude_plan.md` (structural spine) and `codex_plan.md` (grafts), then folds in an
  approved round of new scope. Each draft's load-bearing claims were verified against ground
  truth: engine constants checked against `…/GooseModdingAPI/Exports.cs` (claude exact; codex's
  Appendix-B speeds wrong), the biased `Deck` against `SamEngine.cs`, and the QubeTX conventions
  (editions, the 6 base targets, `cargo-dist 0.31.0`, ICE flags) across TR300/ND300/WB300. Adds:
  the new autonomous behaviors, a ratatui `<name> config` TUI, a three-name goose-speak CLI, and
  a full all-OS/all-arch build matrix. Build milestones now **M0–M19**.
- `claude_plan.md` — comprehensive, adversarially-reviewed plan for **honk300**, a
  cross-platform (Windows/macOS/Linux) Rust reimplementation of Desktop Goose. Derived
  from analysis of `DESKTOP-GOOSE/` (the original v0.31 Windows + v0.22 macOS builds) and
  the `*300` sibling repos (TR300/ND300/WB300). Captures the reverse-engineered engine
  (rig geometry + physics constants, 120 Hz fixed tick, the biased `Deck` shuffle-bag, the
  Task/`InjectionPoints` model from `…/GooseModdingAPI/{SamEngine,Exports}.cs`), a
  Cargo-workspace architecture (`honk-engine` + capability-trait platform backends), build
  milestones M0–M17, the packaging pipeline (cargo-dist + hand-authored
  `windows-installers.yml`), a per-platform capability matrix, and a ranked risk table.
- `codex_plan.md` — a parallel planning document produced by Codex.
- `CHANGELOG.md` / `HUMAN_CHANGELOG.md` — dual changelogs, mirroring the `*300` family
  convention.
- `CLAUDE.md` — repository guidance for future Claude Code sessions.

### Changed
- M7 is now Done, M8 is now Active, and Renderer V2 is tracked separately as backlog task `#r2v`
  instead of remaining an unfinished M7 subtask. The M7 rich task record now preserves the audit,
  readiness pass, renderer spike, verification, visual acceptance, and follow-up split.
- `README.md`, `AGENTS.md`, and `CLAUDE.md` were updated to reflect M0-M7 complete, M8 active, and
  the new ADR location/maintenance rules.
- `claude_plan.md` and `codex_plan.md` are now **superseded reference drafts**; `honk300_plan.md`
  is canonical. The "Read these first" pointers in **both** `CLAUDE.md` and its Codex twin
  `AGENTS.md` were updated in lockstep (canonical plan, milestone range M0–M19, workspace
  cross-reference → `honk300_plan.md` §7).
- `README.md` gained a **"Status — the decided plan"** section recording `honk300_plan.md` as
  canonical and summarizing the decided direction (three-name goose-speak CLI, ratatui config
  TUI, new autonomous behaviors, no external mods / no tray, all-OS/all-arch builds).

### Decided
- **Renderer V2 direction:** use a custom CPU sprite/atlas blitter that outputs premultiplied
  pixels for each platform backend. Keep `tiny-skia`/`resvg` for vector effects or
  asset-rasterization helpers, but do not make Vello/wgpu, Skia, Bevy, Macroquad, or ggez the
  main runtime renderer for the desktop-pet overlay. Future atlas metadata should include stable
  anchors, beak/cursor attach points, hit masks, frame bounds, and animation tags.
- **Three invocation names** (`honk300` / `honk` / `goose`) with a finite, deterministic
  "goose-speak" grammar (e.g. `goose plz` to start, `honk bad` / `goose no honk` to stop,
  `goose do honk` to poke, `<name> config`, `<name> help`) — a fixed phrase map, **not** runtime
  NL parsing.
- **TOML config** (`config.toml`) replacing the original `.ini`, original keys preserved at the
  verified values, versioned + tolerant loader.
- **No external mod system** (no DLL/WASM/data mods). Autumn becomes a **built-in** season/task;
  extensibility is via documented internal seams (`ARCHITECTURE.md` + rustdoc).
- **No system tray.** Quit via hold-ESC (where the OS allows) or any stop command, over a new
  **single-instance + IPC command channel** (`stop` / `do` / `reload`) that is also the
  Wayland-safe quit and the TUI's hot-apply transport.
- A **ratatui** config TUI at `<name> config` (QubeTX-family architecture: reducer + crossterm +
  `tokio::select!`) toggling every behavior incl. Autumn; **hot-apply where cheap, restart-note
  otherwise**.
- **New autonomous behaviors** (each a toggle, scoped to parameter-modulation of the procedural
  rig — no new art): dynamic moods, seasonal moods, multi-monitor chase, on-the-hour double honk,
  perch-&-ride windows, hover-sweep pat streak + hearts, quiet-hours/DND-fullscreen respect, a
  Calm-goose valve, and manual poke commands. Default = full prank, always-on.
- **Build for every advertised OS and architecture:** Windows x64 **and ARM64**, macOS Intel
  **and Apple Silicon** (universal2 `.app`/`.dmg`), Linux x64 **and ARM** (gnu + musl) — arch is a
  build/packaging axis, capability is an OS/display-server axis (`Cap<T>`).
- App name **honk300** (binary `honk300`, optional `honk` alias); fresh permanent WiX/Inno
  GUIDs (never reuse the sibling repos').
- Clean-room **procedural** goose renderer — no sprite extraction. Original sound effects
  bundled 1:1 (personal use); meme images **regenerated originally** via an
  `Assets/Images/Memes/codex.md` brief (not copied); notepad messages **authored fresh**
  (not paraphrased).
- Linux: **X11-first** (runs under XWayland on Wayland sessions); native Wayland behind an
  opt-in `--wayland` flag with reduced mischief.
- Distribution: Windows-first installer matrix (Global/Corporate × MSI/EXE) + shell/
  PowerShell installers + macOS `.app`/`.dmg` + Linux `.desktop`. **No crates.io** —
  `crates-publish.yml` intentionally dropped from the family pipeline.

### Notes
- There is still no public release or installer artifact. The workspace now builds locally, but
  release packaging remains a later milestone. `DESKTOP-GOOSE/` remains the reference copy of the
  original app and contains third-party copyrighted assets; handle redistribution according to the
  current project asset policy before any public release.
