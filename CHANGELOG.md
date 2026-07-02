# Changelog

All notable changes to this project are documented here. Format based on
[Keep a Changelog](https://keepachangelog.com/); the project will adopt
[Semantic Versioning](https://semver.org/) once it produces releasable artifacts.

> **Project stage: implementation in progress.** Milestones M0-M18 are implemented in-tree to the
> available Windows-host evidence, with macOS/Linux GUI smoke split into repeatable readiness
> scripts and follow-up host work. The goose now renders, walks, leaves mud, plays sounds, reacts to the cursor, can
> perform bounded cursor-nab mischief, can perch on user-dragged windows, and can collect
> Notepad/meme windows on Windows, and can be controlled through a single-instance local IPC
> channel. It now has the three-name goose-speak CLI plus durable TOML configuration and the
> ratatui config TUI, dynamic moods, the local on-hour double honk, quiet-hours/DND/fullscreen
> manners, built-in Autumn leaves, Windows multi-monitor chase, and live appearance/recolor
> controls, plus macOS runtime/status/app-bundle staging and Linux IPC/status degradation
> plumbing; there is no release yet. A plain-English companion lives in
> [HUMAN_CHANGELOG.md](./HUMAN_CHANGELOG.md) and must stay in lockstep.

## [Unreleased]

### Added
- **M16 macOS backend, status, and `.app` staging (implementation in-tree; macOS-host smoke
  pending)** — added `crates/honk-platform-macos` with AppKit/CoreGraphics/ApplicationServices
  dependencies, macOS `start` runtime wiring through the existing Unix IPC transport, one
  AppKit overlay surface per display, CoreGraphics pointer polling/warp, local-time sampling,
  Accessibility-gated focused-window polling for foreign-window ride snapshots, AppKit-owned
  note/meme collect windows, macOS terminal-app classification tests, Accessibility-denied
  capability degradation, and dependency-free macOS audio through `/usr/bin/afplay`. The macOS
  target checks pass for `x86_64-apple-darwin` and `aarch64-apple-darwin`; physical macOS smoke
  and universal2 bundle validation remain the M16.1 readiness gate.
- **Runtime status protocol and TUI Status tab** — `honk-control` now supports `STATUS` and a
  compact `ControlResponse::Status` payload reporting running state, platform, bundle mode,
  Accessibility, cursor/window/collect/presence/audio capability states, and asset counts.
  `honk300 status` prints the same data, and the config TUI has a Status category plus refresh
  command. `honk-config::BackendState` now preserves supported/unsupported/denied/failed state
  while still collapsing to simple engine options.
- **macOS agent bundle staging** — added `script/package_macos_app.sh` to build x86_64 and arm64
  release slices, `lipo` them into `Honk300.app`, copy `Assets/`, write `Info.plist` with
  bundle id `dev.emmetts.honk300` and `LSUIElement=true`, ad-hoc sign, and validate with
  `plutil`, `codesign`, and `lipo`. Bundle-aware asset discovery now prefers
  `Contents/Resources/Assets`, and TUI Start launches bundled macOS runs through
  `/usr/bin/open -n <Honk300.app> --args start --config <path>`.
- **M17/M18 Linux control-runtime foundation (display backends still pending)** — added
  `crates/honk-platform-linux` for X11-first vs. forced/native Wayland session detection,
  fallback bounds, Linux local-time sampling, and a common-terminal classifier covering Alacritty,
  GNOME Terminal, kitty, Konsole, Ghostty, WezTerm, xfce4-terminal, and Ptyxis without blocking
  regular apps. Linux `start` now routes into a real runtime loop using the existing Unix IPC
  transport for `status`, `reload`, `stop`, and `do <action>` instead of printing the old
  non-Windows placeholder. The degraded runtime ticks the platform-free engine, loads assets,
  plays sounds through `ffplay`/`mpv` when available, reports audio failure when no compatible
  player exists, and reports cursor/window/collect/presence capabilities as unsupported or failed
  until a visible X11 or Wayland backend proves them. ADR 0011 records that M17 full X11 parity
  and M18 visible Wayland reduced mode remain Linux-host readiness gates.
- **M16-M18 readiness scripts and evidence handoff** — added
  `docs/readiness/m16-m18-readiness.md`, `script/smoke_m16_macos.sh`, and
  `script/smoke_m17_m18_linux.sh`. The macOS smoke script builds and validates the universal2
  `.app`, launches the LSUIElement bundle, checks status, and exercises honk/mud/reload/stop IPC.
  The Linux smoke script builds the binary, runs default and forced-Wayland degraded modes, checks
  status, verifies unsupported cursor-nab rejection, and exercises honk/mud/wander/reload/stop IPC.
  The readiness note records the Windows-host gate/cross-target evidence and splits remaining
  macOS/Linux GUI smoke into explicit host follow-up work.
- **Multi-monitor chase and appearance controls (milestone M15, complete)** — Windows now creates
  one layered overlay HWND per monitor, enumerates signed monitor bounds, chooses the engine world
  bounds from `[behaviors].multi_monitor_chase`, and clips/crops dirty render regions per monitor
  before calling `UpdateLayeredWindow`. With multi-monitor chase off, startup uses the primary
  monitor bounds; with it on, startup uses the full signed virtual desktop. Reloads hot-apply
  normal world options but report multi-monitor chase changes as restart-required.
- **M15 engine/config appearance contract** — `WorldOptions` now carries
  `multi_monitor_chase` and `AppearanceOptions { calm_goose }`. Calm Goose uses the existing Calm
  Suppression/manners path to suppress spontaneous honks, on-hour honks, autonomous
  cursor/window/collect mischief, and Autumn pile chase while leaving direct clicks and CLI/TUI
  pokes under their normal gates. `World::render_bounds(previous)` centralizes dirty-region
  coverage for the goose, previous frame, footmarks, hearts, sleepy particles, and Autumn piles.
- **M15 TUI recolor controls and ADR 0009** — the config TUI now makes Calm Goose live, marks
  multi-monitor chase as restart-required, and edits goose white/orange/outline through separate
  RGB channel rows so hue changes are possible without free-form text input. ADR 0009 records the
  accepted multi-monitor, dirty-render, Calm Goose, and original three-color palette scope.
- **Schedule manners and built-in Autumn (milestone M14, complete)** — added
  `honk-engine::schedule` with `ScheduleOptions`, `LocalMinute`, `PresenceSnapshot`, and
  `PresenceState`, plus `World::set_presence`, `World::manners_active`, and the schedule field on
  `WorldOptions`. Quiet hours are start-inclusive/end-exclusive, support overnight windows, and
  treat `start == end` as no quiet window. Quiet hours, Windows DND, and fullscreen use Calm
  Suppression: spontaneous honks, on-hour honks, autonomous cursor/window/collect mischief, and
  Autumn pile chase are suppressed while direct clicks and CLI/TUI pokes still pass through normal
  config/capability gates. Windows maps `SHQueryUserNotificationState` into platform-neutral
  presence snapshots and polls periodically, warning once and degrading to unsupported if the API
  fails.
- **Procedural Autumn leaf piles** — added platform-free `AutumnState`, piles, leaves, kicked-leaf
  physics, `AutumnLeafPileTask`, render-layer splitting, and Windows render ordering
  (footmarks → Autumn below-goose leaves → goose → Autumn above-goose leaves → hearts → sleepy
  particles). Autumn is active September 1 through November 30 by local runtime-injected date,
  uses recovered reference constants for pile timing/count/physics, and does not copy or load the
  original `Autumn.dll`. The Windows runtime adds `HONK300_SMOKE_LOCAL_DATE=YYYYMMDD` so Autumn can
  be visually smoke-tested outside the season.
- **M14 config and TUI plumbing** — existing version-1 TOML schedule fields now map into
  `WorldOptions.schedule`, `[safety].pause_on_fullscreen` controls fullscreen manners, and the TUI
  removes `(planned)` from live schedule/season rows while adding a separate fullscreen-respect row.
- **Dynamic moods and on-hour double honk (milestone M13, complete)** — added
  `honk-engine::mood` with `MoodKind::{Content,Hyper,Sad,Sleepy,Mischievous}`,
  `MoodIntensity::{Calm,Normal,Spicy}`, seeded weighted transitions, and platform-free
  `LocalTime` injection for schedule-like inputs. Mood effects post-modulate task output:
  sad/sleepy slow movement and lower neck posture, sleepy emits procedural Z particles, hyper
  can request the existing `HyperTask`, and mischievous duplicates only already-enabled
  nab/collect factories in the pickable list. The Windows runtime samples local time outside the
  engine and feeds `World::set_local_time`; the engine emits exactly two high honks at the top
  of a local hour, once per hour. `Sound::Honk` now carries `HonkTone::{Normal,High,Low}` and
  the audio backend maps tones to bundled honk clips while respecting audio toggles.
- **Config TUI and durable configuration (milestone M12, complete)** — added the `honk-config`
  crate for versioned TOML defaults, path resolution, validation, tolerant loading, conversion
  into runtime/world options, and atomic save with practical preservation of unknown keys. The
  default path is `%LOCALAPPDATA%\honk300\config.toml`, `~/Library/Application Support/honk300/config.toml`,
  or `$XDG_DATA_HOME` / `~/.local/share/honk300/config.toml`, with `--config <path>` override.
  Startup falls back to defaults on missing or rejected config and warns without corrupting the
  running state. Reload parses and validates before applying, then hot-applies current M0-M15
  settings for audio, mouse steal/tuning, perch-and-ride, collect-window kinds, pat behavior,
  timing, movement speed, mud/footmark timing, palette, mood intensity, on-hour honking, schedule,
  Autumn, and Calm Goose. Future settings for Wayland/backend and spicy behavior are persisted and
  shown as planned or restart-required.
- **Ratatui reducer UI (milestone M12, complete)** — added the `honk-config-tui` crate with
  reducer-owned state, pure render modules, categories for General, Behaviors, Mischief,
  Schedule, Appearance, Audio, Commands, and About, plus a Poke panel that sends M10 IPC commands.
  Terminal-window protection is shown as always on rather than configurable. Reducer tests cover
  navigation, toggles, numeric edits, dirty/save state, and poke command generation.
- **Shared control crate** — extracted the M10 protocol/client/server code from the binary into
  `honk-control`, reused by the root binary and TUI without changing the wire protocol or adding
  IPC concerns to `honk-engine`.
- **CLI grammar (milestone M11, complete)** — added deterministic pre-clap normalization for
  executable stems `honk300`, `honk`, and `goose`. The binary accepts default start, `start`,
  `plz`, `stop`, `reload`, `do <honk|wander|mud|meme|note|nab>`, `config`, `help`, `--help`,
  `--version`, `--config <path>`, and `--wayland`. `honk plz`, `goose plz`, and `honk300 plz`
  all start; `bad`, `no`, and `no honk` stop; pokes stay explicit through `do <action>`,
  including `do honk`. `install`, `uninstall`, `update`, and `setup` now parse for
  discoverability, with M19 placeholder messages where installer/update behavior is not yet
  implemented.
- **CLI/TUI control plane (milestone M10, complete)** — the root binary is now split into
  `src/cli.rs`, `src/control/`, and `src/runtime/windows.rs`. `honk300` defaults to `start`;
  `honk300 start` refuses to create a second goose; and `honk300 stop`, `honk300 reload`, and
  `honk300 do <honk|wander|mud|meme|note|nab>` send finite local IPC commands to the running
  instance. Windows uses a per-user named mutex plus a per-user named pipe. Unix-family readiness
  uses the same protocol over a UID-scoped lock file and Unix domain socket shape for later macOS
  and Linux overlay backends. `honk-engine` gained `PokeAction`, `PokeOutcome`, `World::poke`,
  and `World::apply_options` so stop/reload/poke plumbing stays structured and platform-neutral.
  The protocol rejects malformed, unknown, and oversized payloads. ADR 0004 records the
  CLI/TUI-only control model: no system tray, no global quit key, and no non-IPC stop path.
- **Terminal-window protection** — Windows foreign-window discovery now classifies common terminal
  hosts and excludes them before the goose can ride, collect, move, focus, type into, drag, or
  otherwise manipulate them. The protection rule is documented as permanent and applies to future
  spicy/default-off behavior too; visual overlay over terminal windows remains allowed.
- **Collect-window dispatcher (milestone M9, complete)** — the goose can now drag in Notepad and
  meme windows on Windows. `honk-engine` gained a platform-neutral collect-window contract
  (`CollectWindowId`, `CollectWindowRequestId`, `CollectWindowKind::{Note,Meme}`,
  `CollectWindowCapabilities`, `CollectWindowOptions`, ordered `CollectWindowCommand`s, and
  `CollectWindowSnapshot`) plus `CollectWindowTask` and `World` drain/feed APIs. The task chooses
  note/meme content only when both content and backend capabilities exist, emits ordered spawn /
  move / focus / type / close commands, uses the rig beak tip for drag offset, suppresses
  overlapping pat/click/perch/cursor interrupts while active, leaves Notepad open after typing,
  and closes owned meme windows after a visible dwell. The Windows runtime loads assets from
  provenance-separated `Assets/` directories, spawns and tracks Notepad by PID/HWND, verifies
  foreground focus before Unicode `SendInput`, creates non-topmost owned image windows for memes,
  moves controlled windows with Win32 APIs, toggles pass-through while dragging, feeds snapshots
  back into the engine, and adds `HONK300_SMOKE_COLLECT=note|meme` for visual smoke before M10/M11
  public pokes.
- **M9 assets and ADR 0003** — screened original meme/note assets that pass provenance checks are
  copied 1:1 for personal-use builds, one complete custom in-house counterpart is added per copied
  original, and user-supplied `Meme8.png` is included as an approved meme prop. One original meme
  candidate with a baked-in social handle watermark is excluded rather than redacted. Donate is
  intentionally removed: old donate pages, Patreon links, social handles, and old-project branding
  do not ship. ADR 0003 records the collect-window command/snapshot boundary, asset provenance,
  no-donate decision, cross-platform degradation model, and target expectations.
- **Foreign-window perch & ride (milestone M8, complete)** — the goose now reacts when the
  user drags another application's window on Windows. `honk-engine` gained a platform-neutral
  foreign-window contract (`ForeignWindowId`, `ForeignWindowSnapshot`,
  `ForeignWindowCapabilities`, and `ForeignWindowOptions`) and a transient `PerchRideTask`
  that interrupts the current task, runs to the dragged window's ride anchor, pins to the
  moving anchor if it arrives before release, and resumes the interrupted task on release or
  capability loss. The Windows backend now watches move/size drags with an out-of-context
  `SetWinEventHook`, queues hook events only, polls live geometry via `GetWindowRect`, filters
  the app overlay and invalid/non-root/invisible/minimized windows, unhooks on drop, and exposes
  a temporary `--no-window-ride` opt-out until M12 config exists. `move_window` is reported as
  future capability data only; M8 does not autonomously move windows or start M9
  collect-window/notepad/meme behavior. Added ADR 0002 to pin the engine/backend
  contract and cross-platform guardrails.
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
- **M12R config/TUI polish** — `[speeds]`, `[mud]`, `[colors]`, `[moods]`, and on-hour settings
  now validate and map into `WorldOptions` instead of staying write-only. Unknown top-level TOML
  keys and unknown section keys emit a one-shot load warning while still being preserved on save.
  The TUI now uses a row model with scroll support; surfaces movement, mud, color, mood,
  on-hour, and quiet-time rows; edits quiet start/end in 15-minute increments; cycles mood
  intensity through `calm -> normal -> spicy`; confirms dirty quits; routes command outcomes
  through reducer actions; and starts the goose with null stdio plus Windows detached flags.
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
- M15 is now Done, M16 implementation has moved into the active backend-readiness track, and
  Renderer V2 remains tracked separately as backlog task
  `#r2v`. The task records now preserve M7's audit/readiness/renderer work, M8's foreign-window
  readiness pass, M9's collect-window asset/ADR/target-readiness work, and M10's IPC/control
  readiness work, plus M11 CLI grammar, M12 config/TUI readiness work, M13 moods/hourly-honk
  closure, M14 schedule/Autumn closure, and M15 multi-monitor/appearance closure.
- `README.md`, `AGENTS.md`, and `CLAUDE.md` were updated to reflect M0-M15 complete, M16 active,
  the new M17/M18 Linux control-runtime foundation, and the ADR
  0001/0002/0003/0004/0007/0008/0009/0010/0011 location and maintenance rules.
- Added **ADR 0005** (M11 three-name CLI, goose-speak, and the poke-outcome round-trip) and
  **ADR 0006** (M12 config TUI, durable TOML, and the capability/preference boundary), recording
  the previously-undocumented M11/M12 decisions and the four contract corrections from the
  adversarial review.
- Added **ADR 0007** (M13 dynamic moods and local-time injection), recording the platform-free
  mood state machine, honk-tone contract, and runtime-owned local-clock sampling boundary.
- Added **ADR 0008** (M14 schedule, presence, and Autumn), recording Calm Suppression, the
  schedule/presence engine boundary, Windows presence polling, and the built-in Autumn constants.
- Added **ADR 0009** (M15 multi-monitor and appearance), recording the per-monitor Windows overlay
  boundary, dirty-region presentation, Calm Goose valve, restart-required multi-monitor reload
  rule, and original three-color recolor scope.
- Added **ADR 0011** (M17/M18 Linux control runtime and degraded Wayland foundation), recording
  the X11-first session rule, forced/native Wayland degradation, Linux Unix IPC runtime, terminal
  classifier, command-player audio, and the remaining Linux-host readiness gates for visible X11
  and Wayland backends.
- `claude_plan.md` and `codex_plan.md` are now **superseded reference drafts**; `honk300_plan.md`
  is canonical. The "Read these first" pointers in **both** `CLAUDE.md` and its Codex twin
  `AGENTS.md` were updated in lockstep (canonical plan, milestone range M0–M19, workspace
  cross-reference → `honk300_plan.md` §7).
- `README.md` gained a **"Status — the decided plan"** section recording `honk300_plan.md` as
  canonical and summarizing the decided direction (three-name goose-speak CLI, ratatui config
  TUI, new autonomous behaviors, no external mods / no tray, all-OS/all-arch builds).

### Fixed
- **Control responses now report the real outcome (M11 round-trip).** `honk300 do <action>` and
  `reload` previously always answered `OK` because the server thread responded at command-enqueue
  time, before the simulation ran. The transport now completes a request/response round-trip:
  `honk-control` gained `ControlRequest`, a bounded (2 s) wait for the sim's answer, and a
  `PokeOutcome`→`ControlResponse` mapping (`Busy` → `ERR BUSY`, `Unsupported` → `ERR UNSUPPORTED`,
  reload failure → `ERR RELOAD_REJECTED`, timeout → `ERR TIMEOUT`). The CLI/TUI "rejected: {code}"
  paths now actually fire. (ADR 0005.)
- **Cursor-warp capability is no longer seeded from the mouse-steal preference (M12 reload).** The
  Windows runtime initialized `cursor_warp_supported` from `!no_mouse_steal`, latching warp off so
  a config edit that re-enabled mouse steal never took effect until restart. It is now a pure
  platform capability (`true` on Windows, via `initial_cursor_warp_supported`) that degrades only
  on a real warp failure; the preference is applied solely through `MouseStealOptions::enabled`.
  (ADR 0006.)
- **Collect-window capability loss now survives reload (M12).** A backend collect-window failure
  was recorded only in engine state and was overwritten by the next reload, so the goose kept
  retrying a dead capability. `BackendState` gained `collect_window_supported`, threaded through
  `Config::effective_options`, so the loss is durable across reloads. (ADR 0006.)
- **Disabling the pat streak no longer disables clicking (M12 interaction).** `interaction.pat_streak`
  gated the click reaction as well as pats. It now scopes to the hover-pat hearts/calm only;
  clicking the goose still triggers a hyper burst (or a cursor nab when mouse steal is supported).
  (ADR 0006.)

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
- **No system tray and no global quit key.** Start, stop, reload, pokes, and future configuration
  are CLI/TUI-only over the **single-instance + IPC command channel** (`start` / `stop` / `do` /
  `reload`) that is also the Wayland-safe control path and the TUI's hot-apply transport.
- **Terminal windows are protected.** The goose may visually overlay terminals, but terminal
  windows are never valid ride, collect, movement, focus, typing, drag, or spicy-behavior targets.
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
- Clean-room **procedural** goose renderer — no sprite extraction. Original sound effects,
  screened original memes, and screened original notes are bundled 1:1 for personal-use builds;
  every copied meme/note original gets one complete custom in-house counterpart. Old donate pages
  and old developer references do not ship.
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
