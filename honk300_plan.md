# honk300 — Cross-Platform Rust Desktop Goose: Hybrid Research & Implementation Plan

> **Status of this document:** This is the **canonical** research + implementation plan for
> `honk300`. It is an adversarially-reviewed *hybrid* that merges the two earlier drafts —
> `claude_plan.md` (the structural spine) and `codex_plan.md` (grafted enrichments) — and folds
> in a round of new, user-approved scope (new autonomous behaviors, a ratatui config TUI, a
> three-name CLI grammar, and a full all-OS/all-arch build matrix). **No application code is
> written yet.** Implementation happens in a later, separate round, starting at milestone M0.
>
> `claude_plan.md` and `codex_plan.md` are **retained** as provenance/reference; where any of
> the three documents conflict, **this file wins**.

---

## How this plan was synthesized (and why the numbers here are trustworthy)

The two drafts disagreed on load-bearing facts. Those disagreements were resolved by
**claim-testing each against ground truth**, not by preference:

- **Engine constants** were verified against the shipped, readable C# modding-API source
  (`Exports.cs`, `SamEngine.cs`). `claude_plan.md` reproduced them **verbatim**; `codex_plan.md`
  read binary *symbol names* but invented *values* (its Appendix B speeds are wrong). **This plan
  uses the verified values.**
- **The Deck RNG** is a *biased* shuffle in `SamEngine.cs` (not a weighted picker). Faithful-port
  decision stands, pinned by tests.
- **The QubeTX family conventions** (editions, targets, cargo-dist version, ICE flags) were
  verified across `qube-machine-report` (TR300), `qube-network-diagnostics` (ND300), and
  `qube-workbranch-view` (WB300).
- **The Windows overlay crux** (`UpdateLayeredWindow` vs softbuffer; per-pixel-alpha hit-testing
  vs `WS_EX_TRANSPARENT`) follows the technically-correct resolution; the alternative spec is a
  click-through-vs-clickable contradiction and is explicitly avoided.

Source-of-truth files (read directly):
`DESKTOP-GOOSE/DesktopGoose v0.31/FOR MOD-MAKERS/GooseMod_DefaultSolution/GooseModdingAPI/{Exports.cs, SamEngine.cs}`,
`…/DefaultMod/TaskDemo_FollowLowAccel.cs`, `…/config.ini`, the Mac `DesktopGoose.sdef` + notes,
and `qube-{machine-report,network-diagnostics,workbranch-view}`.

---

## 1. Context & locked decisions

### 1.1 What we're building
`README.md` asks us to analyze the bundled Desktop Goose files and build **an entirely new
version** — same functionality, same features — but in **Rust**, on **Windows, macOS, and
Linux**, with **native + CLI installers like TR300/ND300/WB300**, **except not distributed via
crates.io** (builds, installers, and scripts only).

Desktop Goose (Samperson) is a desktop-pet: an annoying goose wanders your screen, leaves muddy
footprints, honks, grabs your cursor, drags your windows around, opens Notepad to type at you,
and brings you memes. The original is closed-source C#/.NET (Windows `GooseDesktop.exe`; macOS a
Xamarin/Mono `.app`). **Intended outcome:** a self-contained Rust binary `honk300` (with `honk`
and `goose` aliases) that recreates the original across the three OSes — plus a set of modern,
opt-in behaviors and a terminal config UI — packaged with the full `*300` installer family.

**Scope note (personal use):** installation is on a personal device, which relaxes
code-signing/notarization and copyright-redistribution constraints (documented below), but we
still build the full installer matrix because matching the `*300` family is an explicit goal.

### 1.2 Locked decisions
| Topic | Decision |
|---|---|
| **Names** | Three installed names: `honk300` (primary), `honk`, `goose`. Fresh **permanent** WiX/Inno GUIDs (never reuse TR/ND/WB). |
| **Goose visual** | **Procedural** clean-room render from the public modding-API constants — no sprite extraction. |
| **Sounds** | Bundle original honk/bite/mud (+ Mac pat) audio **1:1**, embedded (personal use). Kept out of any source/package artifact. |
| **Memes** | Copy screened originals 1:1 for personal-use builds, add **one complete custom in-house counterpart per original** in the clumsy MS Paint house style, and keep user-supplied `Meme8.png`. Absent/unsupported slots skip at runtime. |
| **Notes** | Copy screened original notepad messages 1:1 for personal-use builds and add **one custom goose-voiced counterpart per original**. |
| **Config** | **TOML** (`config.toml`), original keys preserved at verified values, versioned + tolerant loader. No `EnableMods` key. |
| **Modding** | **No external mods** (no DLL/`.so`/`.dylib`, no WASM, no third-party data mods). Autumn becomes a **built-in** season/task. Extensibility = **documented internal seams**. |
| **Control** | **No system tray and no global quit key.** Starting, stopping, and configuration are handled by the CLI and TUI over a **single-instance + IPC command channel** (`stop` / `do` / `reload`). |
| **Protected windows** | The goose may visually overlay terminal windows, but must never move, focus, type into, ride, drag, collect, or otherwise manipulate terminal windows — even in spicy/default-off modes. |
| **Default behavior** | Full original **prank, always-on**. `--no-mouse-steal` is opt-in; `pause_on_fullscreen` default on; a **Calm goose** TUI toggle is the opposite pole. |
| **Config UI** | A **ratatui** Claude-Code/QubeTX-family-style TUI at `<name> config`, toggling every behavior incl. Autumn; **hot-apply where cheap**, restart-note otherwise. |
| **Linux** | **X11-first** (runs under XWayland). **Native Wayland** behind an opt-in `--wayland` flag (reduced mischief). |
| **Targets** | **Every OS + arch we advertise:** Windows x64 **and ARM64**, macOS Intel **and Apple Silicon** (universal2), Linux x64 **and ARM** (gnu + musl). |
| **Packaging** | Windows-first 4-installer matrix (Global/Corporate × MSI/EXE) **built per-arch** + shell/PowerShell installers + macOS `.app`/`.dmg` + Linux `.desktop`. **No crates.io** (`crates-publish.yml` dropped). |

---

## 2. Mission, Non-Goals, and Guiding Principles

### 2.1 Mission
Rebuild Desktop Goose as a modern, maintainable, cross-platform desktop companion that
reproduces the *feel* and feature set of the original (wandering, mud, mouse stealing/biting,
honking, patting, meme/notepad windows, config-driven behavior, Autumn leaves), runs
natively on Windows/macOS/Linux, ships via native installers/builds/scripts only, and is **safe
by default**: no telemetry, no network on its own, clean uninstall, explicit user control.

### 2.2 Non-Goals (out of scope for v1)
Bit-for-bit binary compatibility with the original; loading the original .NET/Mono mods; **any
external mod surface** (DLL/WASM/data); mobile platforms; a windowed GUI settings app; a **system
tray**; multi-goose "as a service"; runtime natural-language command parsing (the goose-speak
grammar is a fixed, finite phrase map — no model at runtime); networked features.

### 2.3 Guiding principles
1. **Simulation core is platform-free.** The goose AI, rig, task scheduler, and mood machine are
   a pure crate (`#![forbid(unsafe_code)]`) with no windowing/render/audio/OS dependency —
   headless-testable.
2. **Platform code is an adapter.** Window creation, input, rendering, audio, foreign-window
   control, and IPC live behind capability traits, one impl per backend. No compositor quirk
   leaks into the goose AI.
3. **Honest cross-platform parity.** Windows/macOS/X11 reach full parity; Wayland is documented
   limited-mode. Never *silently* degrade — a disabled capability says so (in the TUI and logs).
4. **Architecture is a build axis, capability is an OS axis.** Every feature available on an OS is
   available on all of that OS's arches; feature differences come only from OS + display server +
   permissions (the `Cap<T>` model). "Build for all arches" is a CI/packaging concern.
5. **Distribution mirrors the QubeTX line, minus the crates.io end-user path.**
6. **User sovereignty.** The goose is a guest: CLI/TUI stop commands evict it; uninstall is
   total; nothing happens the user didn't opt into. Default chaos is bounded by quiet-hours,
   fullscreen-pause, `--no-mouse-steal`, protected terminal windows, and Calm goose.
7. **Asset/IP rules are first-class** (see §12): sounds, screened memes, and screened notes are
   bundled 1:1 for personal use; every copied meme/note original gets one complete custom
   in-house counterpart; old donate pages and old developer references do not ship.

---

## 3. Source analysis — the original Desktop Goose

Two distributions are bundled in `DESKTOP-GOOSE/`:
- **`DesktopGoose v0.31/`** (Windows): `GooseDesktop.exe`, `GooseModdingAPI.dll`, `MMQ.dll`,
  `config.ini`, an `Assets/` tree, a `FOR MOD-MAKERS/` Visual Studio solution, readme/text files.
- **`Desktop Goose for Mac v0.22/`**: a Xamarin/Mono `.app` (`Desktop Goose.exe`), `Info.plist`
  (`CFBundleIdentifier=net.namedfork.DesktopGoose`, `LSUIElement=True`, `NSAppleScriptEnabled`),
  `DesktopGoose.sdef`, `runtime-options.plist`, sounds, memes, notes, `Pat1-3.wav`.

### 3.1 The headline finding: the goose is **procedurally rendered**, not a sprite
There is **no goose sprite anywhere**. It is drawn each frame from a geometric **rig** of
primitives, confirmed by the shipped mod-maker source (`SamEngine.cs`, `Exports.cs`). The
`config.ini` colors (`#ffffff` / `#ffa500` / `#d3d3d3`) map to the `brushGooseWhite/Orange/
Outline` solid brushes in `GooseRenderData`. **Consequence:** we reimplement the *renderer*
clean-room; no art extraction.

### 3.2 Engine model (port source-of-truth)
- **Fixed timestep 120 FPS**, `deltaTime = 1/120`, driven by a `Stopwatch` (`SamEngine.Time`).
  Locomotion/step constants are tuned to this rate — the sim must **not** couple to redraw rate.
- **Math:** `Vector2` struct; `SamMath` (`Deg2Rad`, `Rad2Deg`, `RandomRange`, `Lerp`, `Clamp`).
- **`Deck`** — a shuffle-bag (no repeats until exhausted). ⚠️ The original `Reshuffle()` is a
  **biased** shuffle: `otherIndex = (int)RandomRange(0, j)` (low-bound 0, exclusive high `j`),
  seeded by `System.Random` — **not** a correct Fisher–Yates. Task pickability is a
  `canBePickedRandomly` **bool**, not a weight. (Open question: the full binary contains a
  `gooseTaskWeightedList` symbol; verify in M0 whether weighting exists and decide faithful vs.
  corrected. Default: faithful biased shuffle, test-pinned.)
- **Input:** `mouseX`, `mouseY`, `leftMouseButton` (`ButtonState` with `Held`/`Clicked`/
  `Released`), refreshed each frame.

### 3.3 Goose entity + parameters — **exact verified constants** (`Exports.cs`)
`GooseEntity.ParametersTable`:

| Constant | Value | Meaning |
|---|---|---|
| `WalkSpeed` / `RunSpeed` / `ChargeSpeed` | **80 / 200 / 400** | max speed per `SpeedTiers {Walk, Run, Charge}` |
| `AccelerationNormal` / `AccelerationCharged` | **1300 / 2300** | accel in Walk/Run vs Charge |
| `StepTimeNormal` / `StepTimeCharged` | **0.2 / 0.1** | foot-step interval (s) |
| `StopRadius` | **−10** | stop tolerance |
| `DurationToTrackMud` | **15** | seconds the goose leaves muddy prints |

State: `position` (default `(300,300)`), `velocity`, `direction` (deg, default **90**),
`targetDirection`, `targetPos` (default `(300,300)`), `currentSpeed`, `currentAcceleration`,
`stepInterval`, `extendingNeck`, `canDecelerateImmediately` (default true). The engine
**auto-locomotes** toward `targetPos`; tasks only set targets/accel (confirmed by the demo task,
which each frame sets `currentAcceleration = 100` + `targetPos = (mouseX,mouseY)`, then
`setTaskRoaming` after 5 s).
`FootMark`: `Lifetime = 8.5 s`, `ShrinkTime = 1 s`; ring buffer `footMarks[64]`.

### 3.4 The Rig — **exact verified geometry** (`Exports.cs`)
Drawn back-to-front: shadow → underbody → body → neck (two lerped positions) → head (two
segments) → eyes → procedural feet.

- **UnderBody:** radius 15, length 7, elevation 9
- **Body:** radius 22, length 11, elevation 14
- **Neck (`Necc`):** radius 13; pos-1 (height 20, forward 3), pos-2 (height 10, forward 16),
  blended by `neckLerpPercent`
- **Head:** seg-1 (radius 15, length 3), seg-2 (radius 10, length 5)
- **Eyes:** radius 2, elevation 3, IPD 5, forward 5
- **Feet (`ProceduralFeets`):** `feetDistanceApart 6`, `wantStepAtDistance 5`, `overshootFraction 0.4`

("radius + length" ⇒ capsule/stadium shapes; "elevation" is the vertical offset for the
fake-3D + shadow.) The mood system (§5.6) "emotes" by modulating `neckLerpPercent`, speed/accel
tiers, step cadence, tint, and task weights — **never** by adding art.

### 3.5 AI = a Task state machine
- `GooseTaskInfo { canBePickedRandomly, shortName, description, taskID, GetNewTaskData(goose),
  RunTask(goose) }`. A `TaskDatabase` holds tasks; the default **roaming/wandering** state picks
  a random pickable task via the `Deck`; `taskIndexQueue` lets a task queue successors.
- Helpers to reimplement (`Exports.cs` `API.Goose`/`TaskDatabase`): `setSpeed`,
  `setTargetOffscreen(canExitTop)`, `isGooseAtTarget(dist)`, `getDistanceToTarget`,
  `setCurrentTaskByID`, `chooseRandomTask`, `setTaskRoaming`, `playHonckSound`,
  `getTaskIndexByID`, `getAllLoadedTaskIDs`, `getRandomTaskID`.
- **Original task inventory** (binary symbols — the parity target):
  - `Task_Wander` (`FirstWanderTimeSeconds`, `Min/MaxWanderingTimeSeconds`, pause sub-states).
  - `Task_TrackMud` (`DurationToTrackMud`, `trackMudEndTime`, `isTrackingMud`, `AddFootMark`,
    `PlayMudSquith`, mud color).
  - `Task_NabMouse` (`MouseGrabDistance`, `MouseDropDistance`, `MouseSuccTime`, `SeekingMouse`,
    `DraggingMouseAway`, `originalVectorToMouse`, `grabbedOriginalTime`, `WaitingToBringWindowBack`).
  - `Task_CanAttackMouse` / `AttackRandomly` (bite the cursor; `BITE.mp3`).
  - `Task_CollectWindow` dispatcher → `CollectWindow_Notepad` (runs `notepad.exe` with a random
    message), `CollectWindow_Meme`; the original donation task is intentionally excluded from
    honk300. Relevant fields include `windowOffsetToBeak`,
    `SetWindowPassthru`, `DraggingWindowBack`, `ExitWindow`, `OriginalWindowStyle`,
    `PassthruWindowStyle`.
  - `FirstUX_FirstTask` / `FirstUX_SecondTask` — a **scripted first-run intro** (the goose
    introduces itself before going random).
  - Off-screen bolt: `DecideToRun`, `DurationToRunAmok`, `Min/MaxRunTime`, `RunningOffscreen`.
- **Mod injection points** (`InjectionPoints`): `PostModsLoaded`, `Pre/PostTick`,
  `Pre/PostUpdateRig`, `Pre/PostRender`. **We do not expose these as an external ABI** — but the
  same seams exist internally and are documented for the user to add built-in tasks.

### 3.6 The original's command surface (macOS `.sdef`) → our poke commands
The Mac AppleScript dictionary enumerates the original's own "things you can tell the goose to
do": `honk`, `wander [duration]`, `nab mouse`, `track mud`, `collect meme [path] [title]`,
`collect note [text] [title]`, `open memes folder`, `open notes folder`. The original donation
command is not carried forward. This is the basis for our **`<name> do <action>`** pokes and the
TUI **Poke** panel.

### 3.7 Sounds & the goose's voice
- **Sounds:** `Honk1-4.mp3`, `BITE.mp3`, `MudSquith.mp3`, optional `Music.mp3`; Mac adds
  `Pat1-3.wav`. → bundle 1:1, embedded.
- **Voice** (real notes, for custom counterparts in-register): terse, lowercase,
  self-aware menace-but-cute — *"am goose hjonk"*, *"good work"*, *"i cause problems on
  purpose"*, *"peace was never an option" — the goose (me)*, *"nsfdafdsaafsdjl … sorry … hard to
  type withh feet"*, ASCII `>o)` / `(_>`.

### 3.8 Asset inventory
- **Sounds** → bundle 1:1, embedded (excluded from the source package `include`).
- **Images/Memes** (`Meme1-7.png`, `GooseDance.gif`, `MemeAttributions.txt` — third-party) →
  copy screened originals for personal-use builds and add one complete custom clumsy-paint
  counterpart per original. User-supplied `Meme8.png` is approved as an extra meme prop.
- **Images/OtherGfx** (`DonatePage.png`, `heart.png`) → exclude old donate pages; hearts are
  procedural for the pat-streak feature.
- **Text/NotepadMessages** → copy screened originals and add one custom goose-voiced counterpart
  per original.
- **Mods/Autumn/Autumn.dll** + `Autumn.txt` (adds leaf piles to "play in") → reimplement as a
  **built-in** Autumn season/task.
- **Icons** → design an original `honk300` icon.

---

## 4. Cross-platform technical landscape + chosen stack

| Capability | Windows | macOS | Linux X11 / XWayland | Native Wayland |
|---|---|---|---|---|
| Transparent always-on-top borderless overlay | ✅ layered window | ✅ NSWindow | ✅ override/EWMH `_NET_WM_STATE_ABOVE` | ⚠️ wlr-layer-shell only |
| Click-through **but** goose stays clickable | ✅ per-pixel-alpha hit-test | ✅ `ignoresMouseEvents` per-region | ✅ XShape input region = goose bbox | ⚠️ no true click-through protocol |
| Warp the user's cursor | ✅ `SetCursorPos`/enigo | ✅ `CGWarpMouseCursorPosition` (A11y) | ✅ `XWarpPointer`/enigo | ❌ blocked by protocol |
| Move **other** apps' windows | ✅ `SetWindowPos` | ✅ AXUIElement (A11y) | ✅ EWMH `_NET_MOVERESIZE_WINDOW` (X11 windows) | ❌ impossible by design |
| Detect window move-start/end (perch & ride) | ✅ `SetWinEventHook(MOVESIZESTART/END)` | ✅ AX observers | ✅ ConfigureNotify / `_NET_WM_STATE` | ❌ self-skips |
| Synthesize keystrokes (Notepad) | ✅ SendInput/enigo | ✅ CGEvent (A11y/Input Mon.) | ✅ XTEST/enigo | ❌ blocked |
| CLI/TUI control (`start`/`stop`/`config`) | ✅ named pipe | ✅ unix socket | ✅ unix socket | ✅ unix socket |
| DND/fullscreen detect (quiet hours) | ✅ `SHQueryUserNotificationState` | ✅ NSWorkspace / Focus | ✅ EWMH fullscreen / idle | ⚠️ best-effort |
| Single-instance + IPC (stop/do/reload) | ✅ named pipe/event | ✅ unix socket | ✅ unix socket | ✅ unix socket |
| Audio | ✅ `rodio` | ✅ | ✅ | ✅ |

**Chosen crates:** `winit` (window/event loop + monitor enumeration), `tiny-skia` (CPU vector
raster of the procedural goose), `softbuffer` (present on **X11/Wayland only**), `windows` crate
(layered window + `UpdateLayeredWindow` + `GetCursorPos`/`SetWindowPos`/`EnumWindows` +
`SetWinEventHook` + named pipe/event), `objc2`/`objc2-app-kit` (+ Accessibility/
CoreGraphics + AX observers) on macOS, `x11rb` (X11 + XShape + EWMH + XRecord), `smithay-client-
toolkit`/`gtk4-layer-shell` (`--wayland`), `enigo` (cursor warp + keystrokes), `device_query`
(X11 polling), `rodio` (+ `symphonia` MP3 decode) audio, `rust-embed` (assets), `serde` + `toml`
(config), `clap` (CLI), `ratatui` + `crossterm` (config TUI), `tokio` (TUI/IPC/update async),
`ureq` + `sha2` + `serde_json` (self-update), error handling via `thiserror` (engine/CLI) and
`color-eyre` (TUI, matching WB300).

**Hard impossibilities (documented, not fought):** native-Wayland foreign-window move;
native-Wayland cursor-warp / keystroke-synth; softbuffer per-pixel-alpha on a
Windows layered window (use `UpdateLayeredWindow`); a bare (un-bundled) macOS binary holding a
durable Accessibility grant (a real `.app` with a stable bundle-id is mandatory).

---

## 5. Behavior/feature spec — original parity + new behaviors

### 5.1 Original parity set (all reproduced)
Procedural goose; Walk/Run/Charge; autonomous wander with config timing; fading muddy footprints
(8.5 s / 1 s, 15 s duration); recolorable; FirstUX scripted intro; nab/drag the user's cursor;
attack/bite (`Task_CanAttackMouse`/`AttackRandomly`); collect-window dispatcher (Notepad / meme)
with drag-to-beak + passthru; off-screen bolt; honks (4) + bite + mud-squish (+ pat); pat
on interaction; Autumn (now built-in); always-on-top transparent overlay spanning monitors;
CLI/TUI start, stop, and configuration control.

### 5.2 New autonomous behaviors — feasibility guardrail
> The goose is drawn each frame from a geometric rig in tiny-skia. It "emotes" **only** through
> parameters the engine already exposes: posture (`neckLerpPercent`, neck/head height), body
> tilt, speed/accel tier, step cadence, honk frequency/pitch, color tint, footmark behavior, and
> task-weight bias. **No hand-drawn frames, no sprite art.** Every behavior below stays inside
> that envelope, and each is an independent toggle in the config TUI.

### 5.3 Quiet hours + DND / fullscreen respect *(default ON)*
Dim honks / calm mischief during a configurable quiet window and whenever the OS reports
Do-Not-Disturb, presentation, or focused-fullscreen (a superset of `pause_on_fullscreen`).
Config: `quiet_start`, `quiet_end`, `dnd_respect`. Detection per §4 table; Wayland best-effort.

### 5.4 Seasonal moods — generalizes Autumn *(default ON)*
**System-date-driven, no network.** Autumn = leaf-piles (the built-in Autumn reference); winter =
snow-tracks / visible breath; spring/summer variants. Master `seasonal` toggle + per-season
sub-toggles. Autumn ships as the worked example of "a season is a built-in task bundle."

### 5.5 Multi-monitor chase *(default ON)*
The goose exits one monitor's edge and re-enters the adjacent monitor, traversing the continuous
signed virtual-desktop space. Builds directly on the per-monitor-window architecture (§7.3).

### 5.6 Dynamic moods — parameter-modulation state machine *(default ON)*
Spontaneous, weighted-timer transitions among `{content, hyper, sad, sleepy, mischievous}`:
- **content** — baseline.
- **hyper** — the existing click→charge reaction, now also self-triggered: charge tier, erratic
  targets, rapid high honks, fast steps.
- **sad** — droopy low neck (`neckLerpPercent` toward pos-2 low), desaturated/cool tint, sparse
  low honks, frequent stops.
- **sleepy** — slow tier, long pauses, occasional Zzz particle.
- **mischievous** — task weights biased toward nab/attack/collect.

Mood modulates posture + speed/accel + honk cadence/pitch + tint + task weights — all existing
knobs. Toggle + a `mood_intensity` setting (`calm | normal | spicy`).

### 5.7 On-the-hour double honk *(default ON)*
Two honks at the top of each local hour. Toggle.

### 5.8 Perch & ride windows *(default ON)*
On a foreign-window **move-start** event, set `targetPos` to that window's title-bar top; the
goose locomotes there using normal locomotion. **If it arrives while you're still dragging → it
sits and rides the title bar.** **If you release the drag before it arrives → it smoothly
abandons and resumes its prior task** (existing task-interrupt/resume; the prior task's data is
preserved and restored). Hooks: `SetWinEventHook(EVENT_SYSTEM_MOVESIZESTART/END)` (Win), AX
move observers (mac), ConfigureNotify tracking (X11). `Cap`-degrades (self-skips) on Wayland.

### 5.9 Pat streak + hearts *(default ON)*
**"Pat" = repeated cursor hover-sweeps over the goose** (not clicks). Sweeping the cursor across
the goose repeatedly builds a happy streak → emits heart particles (procedural; original
`heart.png` as reference) + a temporary calm. Uses the per-frame cursor-over-bbox test already
needed for hit-testing. **Click stays the hyper reaction** — the two interactions are distinct.

### 5.10 Manual poke commands *(default ON)*
`<name> do <honk|wander|mud|meme|note|nab>` from the terminal, and a **Poke** panel in
the config TUI, both routing through the IPC channel to the running goose. Sourced from the
`.sdef` surface (§3.6).

### 5.11 Calm valve *(safety)*
A **Calm goose** master toggle (slows motion, disables jarring mischief, keeps wander/honk/
ambient) plus the `--no-mouse-steal` launch flag. The deliberate opposite pole to always-on.

### 5.12 Available but default-OFF (ship as toggles, low priority)
Clipboard honk (backs up & **restores** the real clipboard), fake-photo flash (captures
nothing), gaggle cameo (rare 2nd goose), `honk`/Konami easter eggs, goose gifts, speech-bubble
notes (lighter cross-platform twin of the Notepad gag), standalone idle-sleep.

### 5.13 Cross-platform parity of the new behaviors
Arch-independent (built for x64 **and** ARM on each OS). Capability differences are
OS/display-server-driven, handled by `Cap<T>`:

| New behavior | Win | macOS | X11/XWayland | Wayland |
|---|---|---|---|---|
| Config TUI + goose-speak CLI + help | ✅ | ✅ | ✅ | ✅ |
| Single-instance + IPC (stop/do/reload, hot-apply) | ✅ pipe/event | ✅ socket | ✅ socket | ✅ socket |
| Dynamic moods · on-hour honk · seasonal · pat-hover+hearts · multi-monitor chase | ✅ | ✅ | ✅ | ✅ (render/sim only) |
| Quiet-hours / DND-fullscreen respect | ✅ | ✅ | ✅ | ⚠️ best-effort |
| Perch & ride | ✅ | ✅ | ✅ | ❌ self-skips |
| Pokes that move windows/cursor (nab/meme/note) | ✅ | ✅ (A11y) | ✅ | ❌ self-disable |

Engine-side behaviors (moods, seasonal, hearts, multi-monitor) are platform-free and identical
everywhere; only OS-interaction behaviors degrade, and they degrade **honestly** (the TUI marks
unavailable toggles "unavailable on this session").

---

## 6. Click-through vs. clickable — the crux, resolved
The goose must let clicks pass through *everywhere except itself*. You cannot be globally
click-through **and** receive clicks. Resolution:
- **Windows (primary):** `UpdateLayeredWindow` per-pixel alpha **without `WS_EX_TRANSPARENT`** —
  Windows naturally routes clicks: opaque goose pixels receive them, transparent pixels fall
  through. Fallback: per-frame toggle of `WS_EX_TRANSPARENT` based on cursor-over-bbox (poll
  `GetCursorPos`, `SetWindowLongPtr`). Style: `WS_EX_LAYERED | WS_EX_TOOLWINDOW | WS_EX_TOPMOST |
  WS_EX_NOACTIVATE`. **Do not** set `WS_EX_TRANSPARENT` globally (that would make pat/click
  impossible — a contradiction in the rejected draft).
- **X11:** set the **XShape input region = goose bbox each frame**, empty elsewhere.
- **macOS:** toggle `ignoresMouseEvents` per cursor-over-goose, or a tracking region.
- This is exactly why `Overlay::set_input_region` exists in the trait.

---

## 7. Target architecture

A Cargo **workspace** (a deliberate divergence from the single-crate `*300` repos — it forces the
platform-agnostic engine to never `use` an OS crate). The shipped artifact remains **one binary**
(installed under three names), keeping cargo-dist/installers happy.

```
honk300/                       (workspace root)
├─ crates/
│  ├─ honk-engine/             # #![forbid(unsafe_code)] — NO winit/OS crates
│  │   ├─ math.rs  time.rs  rng.rs (Deck)
│  │   ├─ entity.rs  rig.rs  feet.rs  footmarks.rs  locomotion.rs
│  │   ├─ task.rs  tasks/{wander,mud,nab,attack,collect_window,first_ux,run_amok,autumn,…}.rs
│  │   ├─ mood.rs              # {content,hyper,sad,sleepy,mischievous} param-modulation FSM
│  │   ├─ schedule.rs          # quiet-hours/season logic (consumes presence flags)
│  │   ├─ render.rs            # Rig -> tiny_skia::Pixmap, dirty-rect aware
│  │   └─ world.rs             # owns sim state; tick(dt, &Input, &Presence, &dyn Platform)
│  ├─ honk-platform/           # capability traits + shared types (ScreenRect, Input, Cap<T>, Presence)
│  ├─ honk-platform-windows/   # layered-window + UpdateLayeredWindow + SetWinEventHook + pipe
│  ├─ honk-platform-macos/     # objc2-app-kit + Accessibility + AX observers + unix socket
│  ├─ honk-platform-linux/     # session detection + degraded runtime helpers + terminal classifier
│  ├─ honk-platform-x11/       # x11rb + XShape + EWMH + XRecord + unix socket
│  ├─ honk-platform-wayland/   # wlr-layer-shell (degraded) backend
│  ├─ honk-assets/             # rust-embed + extraction + override precedence
│  ├─ honk-control/            # shared local IPC protocol/client/server
│  ├─ honk-config/             # versioned TOML load/validate/save + runtime options
│  └─ honk-config-tui/         # ratatui reducer settings UI + Poke panel
└─ src/main.rs                 # honk300 binary: clap + goose-speak normalization, config, install/update, loop
```

### 7.1 Platform capability traits
Split by capability so a backend reports **per-capability** support
(`Cap<T> = Ok | Unsupported | Denied | Failed`) instead of failing wholesale — this is what makes
"Wayland = reduced mischief" and "macOS-without-permission = degraded" fall out of the design.

```rust
trait Overlay {
    fn monitors(&self) -> Vec<MonitorId>;
    fn virtual_bounds(&self) -> ScreenRect;            // sim space (signed origin)
    fn monitor_bounds(&self, m: MonitorId) -> ScreenRect;
    fn scale_factor(&self, m: MonitorId) -> f64;       // per-monitor DPI
    fn present(&mut self, m: MonitorId, dirty: ScreenRect, px: &Pixmap) -> Result<()>;
    fn set_input_region(&mut self, m: MonitorId, interactive: &[ScreenRect]) -> Result<()>;
}
trait Pointer { fn global_cursor_pos(&self)->Point; fn warp_cursor(&self,p:Point)->Cap<()>; fn buttons(&self)->ButtonState; }
trait ForeignWindows {
    fn enumerate(&self)->Cap<Vec<ForeignWin>>;
    fn move_to(&self,w:ForeignWin,p:Point)->Cap<()>;
    fn watch_move(&self)->Cap<MoveEvents>;             // move-start/end for perch & ride
}
trait Synth { fn type_text(&self,s:&str)->Cap<()>; fn launch_text_editor(&self)->Cap<EditorHandle>; }
trait Audio { fn play(&self, clip: ClipId); }
trait Presence { fn dnd_or_fullscreen(&self)->Cap<bool>; fn idle_secs(&self)->Cap<u64>; }
trait Control {                                        // single-instance + IPC command channel
    fn acquire_singleton(&self)->Result<Singleton>;    // fails if a goose already runs
    fn serve_commands(&self)->CommandRx;               // receives Stop / Do(action) / Reload
    fn send_command(cmd: Command)->Result<()>;         // used by `<name> stop|do|config`
}
```

### 7.2 The loop — three clocks
- **Sim = fixed 120 Hz** accumulator (`while acc >= dt { world.tick(dt) }`, clamp catch-up to
  ~5 ticks to avoid spiral-of-death).
- **Input poll = 120 Hz** (cursor pos + buttons + presence flags + IPC commands).
- **Present = on-dirty, rate-capped (~60)** — render only the **dirty rect** around the goose +
  active props into a `Pixmap`; idle goose ≈ near-zero present cost.
- Driven by winit 0.30 `about_to_wait` + `ControlFlow::WaitUntil(next_tick)` (precise sleep, no
  busy-spin). Sim is decoupled from `RedrawRequested`.

### 7.3 Per-monitor windows (key correction)
**One overlay window per monitor**, not one giant virtual-screen window. The engine simulates in
one continuous signed `i32` virtual-desktop space; each window presents only its monitor's
region. Single-DPI per surface, monitor-bounded, no negative-coordinate/huge-buffer/mixed-DPI
pain, present cost collapses to the goose bbox. (Enables multi-monitor chase, §5.5.)

---

## 8. CLI grammar, control & quit model

### 8.1 Three names + finite goose-speak grammar
Installed names: **`honk300`** (primary), **`honk`**, **`goose`** — all resolve to the same
binary. A **deterministic, finite phrase→subcommand map** in the clap front-end (no runtime NL/
LLM). `<name>` below = any of the three.

| Intent | Accepted forms |
|---|---|
| **Start** (default) | `<name>` · `<name> start` · `<name> plz` · `honk plz` · `goose plz` |
| **Stop** | `<name> stop` · `honk bad` · `goose no honk` · `<name> no` · `<name> bad` |
| **Poke an action** | `<name> do <honk\|wander\|mud\|meme\|note\|nab>`; pokes stay explicit, including `do honk` |
| **Runtime status** | `<name> status` shows running state, platform, bundle mode, permissions, capabilities, and asset counts |
| **Config TUI** | `<name> config` |
| **Help** (lists every command incl. goose-speak) | `<name> help` · `<name> --help` |
| **Lifecycle** | `<name> install` · `uninstall` · `update` · `setup` · `--version` |
| **Flags** | `--no-mouse-steal` · `--no-sound` · `--config <path>` · `--wayland` |

Every command and alias is discoverable from `<name> help` **and** a **Commands** reference panel
inside the TUI. Implementation: a normalization layer maps the fixed multi-word goose-speak
phrases (e.g. `no honk`, `bad`, `plz`) to canonical subcommands before clap dispatch.

### 8.2 Single-instance + IPC command channel
The goose **acquires a singleton** at startup (Windows named mutex/event; unix socket + lock file
on mac/Linux). A second launch is refused with a friendly message. The running instance **serves
a small command channel** (named pipe/event on Windows; unix domain socket on mac/Linux)
carrying:
- **`Stop`** — graceful shutdown (restore cursor, close spawned windows, flush).
- **`Do(action)`** — trigger a poke immediately.
- **`Reload`** — hot-apply config (§9).
- **`Status`** — report running state plus platform, bundle, permission, capability, and asset
  counts for the CLI/TUI.

`<name> stop` / `<name> do …` / saving in `<name> config` send these. This is the universal,
**Wayland-safe** quit/poke transport. The channel is
local-only, no network, authenticated to the same user (pipe/socket permissions).

### 8.3 Control paths (no tray, no global quit key)
- **Start** uses the CLI (`<name>` / `<name> start`) and later the config TUI entry point.
- **Stop** uses CLI/TUI commands over IPC (`<name> stop` / `honk bad` / `goose no honk`).
- **Configure** uses `<name> config`; saves hot-apply via IPC `Reload` where possible.

---

## 9. Config TUI (`<name> config`)

A ratatui, Claude-Code/QubeTX-family-style terminal settings editor, mirroring WB300's
architecture (`qube-workbranch-view/src/{app,ui}`):

- **Stack:** `ratatui` 0.30 + `crossterm` + a `tokio::select!` event loop + `color-eyre`.
- **Architecture (reducer / Elm-style):** a central `AppState`, an `Action` enum, an
  `apply(Action)` reducer (the *single* place state mutates), pure `ui::render(frame, &state)`.
  Input → resolve to `Action` → `apply` → render. No direct state mutation from input.
- **Layout:** left category rail — **General · Behaviors · Mischief · Schedule · Appearance ·
  Audio · Commands(ref) · About** — and a scrollable form on the right; a **Poke** panel that
  triggers `do <action>` live over IPC; a central `theme.rs` palette (cyan accent, green=enabled,
  yellow=disabled/warn); a persistent footer hint bar (`j/k move · Enter toggle · ←/→ adjust · S
  save · q quit`).
- **Content:** every behavior is a toggle here — original toggles, **Autumn**, and all §5 new
  behaviors; numeric settings (wander times, quiet-hours, mood intensity, mouse-steal tuning) are
  inline-editable with range validation. The **Commands** panel lists the full CLI grammar
  (§8.1).
- **I/O:** reads/writes the versioned `config.toml` via `serde` + `toml`; transient status line
  (`✓ saved` / `⚠ couldn't write config.toml`).
- **Hot-apply where cheap, restart-note otherwise (§decision 8):** on save, behavior / mischief /
  audio / schedule / mood / appearance changes push to a running goose via IPC `Reload` and take
  effect immediately (toggle Autumn off → the leaves stop at once); structural changes (overlay
  backend, `--wayland`, monitor topology) display a "restart to apply" note next to the field.

---

## 10. Asset strategy

### 10.1 Embedding + extraction + precedence
Embed sounds (the procedural goose needs none) via `rust-embed`. On first run or `<name> setup`,
**atomically extract** a writable, user-editable `Assets/` tree + `config.toml` to a per-user
data dir (`%LOCALAPPDATA%\honk300\` / `~/.local/share/honk300/` / `~/Library/Application
Support/honk300/`). **Override precedence:** user-override dir **>** extracted dir **>** embedded
fallback. **Update safety:** store a content-hash manifest; on app update, re-extract only assets
the user hasn't modified (never clobber edits). Missing meme/note → skip, never crash.

### 10.2 Memes → originals plus custom counterparts
Ship screened original meme assets under `Assets/Images/Memes/originals/` for personal-use builds
and one complete custom in-house counterpart per original under `Assets/Images/Memes/custom/`.
The custom images use the clumsy MS Paint house prompt recorded with the assets. Runtime treats
present PNG meme assets as draggable props; GIF animation can be skipped until supported.
User-supplied `Assets/Images/Memes/user/Meme8.png` is approved as an extra prop.
Original meme candidates that fail the no-old-dev/no-donate/no-social-handle screen are excluded
rather than redacted; the reference `Meme2.png` is excluded for a visible handle watermark.

### 10.3 Notes → originals plus custom counterparts
Ship screened original notepad messages under `Assets/Text/NotepadMessages/originals/` and one
custom goose-voiced counterpart per original under `Assets/Text/NotepadMessages/custom/`. Runtime
picks across all present notes via the `Deck`; missing files skip.

---

## 11. Config schema (`config.toml`)

Original keys preserved at **verified values**, plus new sections. Versioned + tolerant loader
(missing → defaults; malformed → warn + defaults; wrong `goose_config_version` → defaults;
unknown keys → ignored with one warning, mirroring WB300's tolerant TOML loader).

```toml
goose_config_version = 1

[behavior]
silence_sounds          = false          # original SilenceSounds
can_attack_mouse        = true           # original Task_CanAttackMouse
attack_randomly         = false          # original AttackRandomly
use_custom_colors       = false          # original UseCustomColors
first_wander_time_seconds = 20.0         # original FirstWanderTimeSeconds
min_wandering_time_seconds = 20.0        # original MinWanderingTimeSeconds
max_wandering_time_seconds = 40.0        # original MaxWanderingTimeSeconds

[colors]                                  # original GooseDefault* (verified)
goose_white   = "#ffffff"
goose_orange  = "#ffa500"
goose_outline = "#d3d3d3"

[speeds]                                   # verified from Exports.cs — DO NOT guess
walk_speed           = 80.0
run_speed            = 200.0
charge_speed         = 400.0
acceleration_normal  = 1300.0
acceleration_charged = 2300.0
step_time_normal     = 0.2
step_time_charged    = 0.1
stop_radius          = -10.0

[mud]
duration_to_track_seconds = 15.0          # verified DurationToTrackMud
footmark_lifetime_seconds = 8.5           # verified FootMark.Lifetime
footmark_shrink_seconds   = 1.0           # verified FootMark.ShrinkTime

[mouse]                                    # nab tuning (values TBD in M7; sensible defaults)
grab_distance = 60.0
drop_distance = 200.0
succ_time     = 1.5

[behaviors]                                # NEW autonomous behaviors (§5)
on_hour_double_honk  = true
multi_monitor_chase  = true

[moods]
dynamic_moods  = true
mood_intensity = "normal"                  # calm | normal | spicy

[mischief]
perch_and_ride = true

[interaction]
pat_streak = true                          # hover-sweep pats + hearts

[schedule]
quiet_hours_enabled = true
quiet_start = "22:00"
quiet_end   = "08:00"
dnd_respect = true
seasonal    = true                         # date-driven; Autumn is the built-in reference season
autumn      = true                         # per-season sub-toggle

[appearance]
calm_goose = false                         # master calm valve

[audio]
enabled = true
honk = true
bite = true
mud  = true
pat  = true

[safety]
pause_on_fullscreen = true
no_mouse_steal      = false                # --no-mouse-steal sets this true
```

---

## 12. Asset & IP rule

`DESKTOP-GOOSE/` contains Samperson's / third-party copyrighted assets and old developer donation
material. This is a personal-use project, so screened original sounds, memes, and notes are
bundled 1:1 for the owner's machines, and each copied meme/note original gets one complete custom
in-house counterpart. Do not publicly redistribute these bundled assets. The goose visual remains
clean-room procedural. Old donate pages, Patreon links, social handles, and old-project branding
do not ship. No crates.io.

---

## 13. Packaging & distribution — all OSes + arches

Reuse the QubeTX family pipeline, minus crates.io, plus GUI bundling, **across every advertised
OS and architecture**.

### 13.1 cargo-dist (`[workspace.metadata.dist]`)
`cargo-dist-version = "0.31.0"`, `ci = "github"`, `installers = ["shell","powershell","msi"]`,
`pr-run-mode = "plan"`, `install-updater = false`, `allow-dirty = ["msi"]`,
`publish-prereleases = false`. `[package.metadata.wix]` with **freshly generated** `upgrade-guid`
+ `path-guid`. `[profile.dist] inherits="release"; lto="thin"`. **No `crates-publish.yml`.**
`windows_subsystem = "windows"` with a CLI fallback path for `--help`/`config`/`install`/`update`/
`stop`/`do` (allocate a console when run from a terminal).

**Target matrix (every OS + arch):**
- **Windows:** `x86_64-pc-windows-msvc`, `aarch64-pc-windows-msvc`
- **macOS:** `x86_64-apple-darwin`, `aarch64-apple-darwin` (shipped as one **universal2** bundle)
- **Linux:** `x86_64-unknown-linux-gnu`, `aarch64-unknown-linux-gnu`,
  `x86_64-unknown-linux-musl` (+ optional `aarch64-unknown-linux-musl` for ARM-static parity)

### 13.2 Windows: 4 installers × 2 arches
The 4-installer matrix built for **both** x64 and ARM64 (WiX + Inno both support ARM64):
`wix/main.wxs` (Global MSI, perMachine, `Program Files\honk300\bin`, system PATH,
`InstallSource=msi-global`), `wix-corporate/corporate.wxs` (perUser, `%LocalAppData%\Programs\
honk300\bin`, `msi-corporate`), `inno/global.iss` (`exe-global`), `inno/corporate.iss`
(`exe-corporate`). **For a GUI pet:** a Start-Menu (and optional desktop) shortcut, and an
**optional** `HKCU\…\Run` autostart entry (checkbox, default off). Install **all three name
aliases** (`honk300`/`honk`/`goose`). Port `windows-installers.yml` (fires via `workflow_run`
after cargo-dist's `Release`, torn-release guard via `dist-manifest.json` + the Global MSI, WiX
`candle`/`light` with `-sice:ICE38 -sice:ICE64 -sice:ICE91`, Inno EXEs, `.sha256` sidecars,
`gh release upload --clobber`), **matrixed over arch** → up to 8 Windows installer artifacts. CI
builds ARM64 via the MSVC ARM64 toolchain (native ARM runner or cross), honestly noting any
emulation-only test gaps.

### 13.3 macOS: universal2 `.app` staging + M19 `.dmg`
A real `.app` bundle with a **stable bundle-id** (`dev.emmetts.honk300`) is **mandatory** for
durable Accessibility grants (mischief features). Ship a **universal2** binary (Intel + Apple
Silicon) in one `.app`; `.dmg`, Developer ID signing, notarization, installer/update/uninstall,
and icon polish remain M19. Unsigned/un-notarized for personal use → document
`xattr -dr com.apple.quarantine`; degrade mischief gracefully until Accessibility is granted.
M16 sets `LSUIElement=true` by default: the bundle is an agent/permission identity only, while
configuration and status stay CLI/TUI-only. No native preferences window, menu-bar settings UI,
Dock controls, or AppleScript `.sdef` commands ship in M16.

### 13.4 Linux: `.desktop` + tarball/AppImage, X11-first, per-arch
Shell installer drops the binary (all three aliases), extracts assets, installs
`~/.local/share/applications/honk300.desktop` + optional `~/.config/autostart/honk300.desktop`,
for **each arch** (x64 + ARM, gnu + musl). Optional per-arch AppImage and `.deb`/`.rpm` via
cargo-dist extras. Default runs X11/XWayland; `--wayland` opts into the degraded layer-shell mode.

### 13.5 In-binary `install / uninstall / update / setup`
Reuse the family's atomic-write + marker-block + symlink-resolution machinery
(`qube-machine-report/src/install/mod.rs`) and SHA-256 self-update (`…/src/update.rs`), **but**
"install" = **login-autostart + shortcut/.desktop/LaunchAgent + asset extraction + `InstallSource`
marker** — **not** a `.bashrc`/PowerShell-profile alias-autorun (a shell-autorun for a windowed
app would wrongly spawn a goose on every shell start). Self-update: read
`HKCU\Software\Honk300\InstallSource` (or path classification) **and the running arch**, download
the **arch-matched** installer, SHA-256-verify (refuse on mismatch), run silently
(`msiexec /i /passive /norestart` or Inno `/SILENT /SUPPRESSMSGBOXES /NORESTART`), handle msiexec
`3010` honestly, post-install `--version` verify. **Never** a `cargo install` strategy.

### 13.6 Uninstall (`<name> uninstall`)
Channel-aware; `--yes` / `--purge` / `--json`. `--purge` removes state/config/registry/PATH and
the `HKCU\Software\Honk300` key, but **backs up user memes/notes first** and tells the user
where. Automated **"leave nothing behind"** script scans PATH/registry/filesystem after uninstall.

### 13.7 CI workflows
`ci.yml` (fmt, clippy `-D warnings`, test, build on push/PR, all targets); cargo-dist
`release.yml` (tag-triggered, full target matrix + GitHub Release); `windows-installers.yml`
(`workflow_run` after Release, per-arch Corporate MSI + 2 Inno EXEs, `.sha256`, `--clobber`).
**No `crates-publish.yml`.**

---

## 14. Build milestones (later round)

Each is independently runnable. M6 (hit-testing) is pulled early (highest-risk primitive);
platforms come after the full Windows feature set so the platform trait is fully shaped before
being implemented three more times.

| # | Milestone | Done-when |
|---|---|---|
| M0 | Workspace + `honk-engine` (math/time/Deck/entity/rig/feet) ported 1:1; unit tests vs verified constants + **golden-frame harness** | `cargo test` green; Deck sequence + rig vertices pinned |
| M1 | Windows transparent click-through topmost overlay renders a **static** procedural goose | goose floats; clicks pass through |
| M2 | 120 Hz accumulator + locomotion + dirty-rect present + procedural feet | goose walks at correct speed; low CPU |
| M3 | Footmarks + mud (8.5/1, 15 s) | fading prints trail the goose |
| M4 | Task state machine + wander + config timing + **FirstUX intro** | scripted intro, then autonomous wander |
| M5 | Audio + honks + bite + mud + **pat** + `SilenceSounds` | sounds play; mute respected |
| M6 | Hit-testing: **pat = hover-streak + hearts**; **click → hyper** | hover pets, click hypes, empty passes through |
| M7 | Cursor mischief (warp + nab sub-states) | goose drags the real cursor |
| M8 | Foreign-window dragging + **perch & ride** (move-start → ride / smooth-abandon) | goose rides a dragged window; abandons cleanly |
| M9 | Collect-window dispatcher: Notepad (faithful keystroke synth) + meme | goose types a note; drags meme windows |
| M10 | **Single-instance + IPC command channel** (stop/do/reload); **no tray, no global quit key** | second launch refused; `honk300 stop` quits; CLI pokes reach the running goose |
| M11 | **CLI grammar** (3 names + goose-speak phrase-map) + `do <action>` pokes + `help` | `goose plz` starts, `honk bad` stops, `goose do honk` honks |
| M12 | **Config TUI** (ratatui reducer; groups + Poke panel; TOML I/O; hot-apply via IPC) | current settings hot-apply where supported; future settings are marked planned/restart-required; save persists |
| M13 | **Dynamic moods** (param-modulation FSM) + **on-hour double honk** | goose spontaneously shifts mood; honks the hour |
| M14 | **Schedule**: quiet hours + DND/fullscreen respect; **seasonal** (Autumn built-in) | goose calms at night/fullscreen; autumn leaves appear in season |
| M15 | **Multi-monitor chase** + full recolor/appearance | goose crosses between monitors; recolor works |
| M16 | macOS backend + universal2 `.app` + permission-gated degradation + CLI/TUI status | runs on Intel + Apple Silicon; degrades gracefully without A11y; no native settings surface |
| M17 | Linux X11 backend (XShape + EWMH + device_query) | full parity on X11/XWayland |
| M18 | `--wayland` layer-shell degraded mode (stop/poke via IPC) | renders on Wayland; mischief self-disables; `goose stop` works |
| M19 | install/update/uninstall(`--purge`)/setup + packaging **all targets** (Win x64+ARM64 ×4 installers, mac universal2 `.dmg`, Linux x64/ARM gnu+musl) + 3 aliases | installers produce working artifacts w/ autostart + shortcut on every OS/arch |

Implementation note (2026-07-01): the Linux control-runtime foundation has landed in
`honk-platform-linux` plus `src/runtime/linux.rs`. Linux `start` now uses Unix IPC, answers
`status`/`reload`/`stop`/`do`, detects X11-first vs. `--wayland`, classifies common terminal apps,
samples local time, and reports unsupported/failed capabilities honestly. This does **not** close
M17 or M18 by itself: X11 still needs visible overlay/input/window parity and Wayland still needs
the reduced layer-shell rendering path plus Linux-host readiness smoke. The M16-M18 active
implementation card is closed to Windows-host evidence in `docs/readiness/m16-m18-readiness.md`;
remaining platform GUI evidence is split into host smoke scripts and follow-up readiness work.

---

## 15. Adversarial red-team — ranked risks & mitigations

**Technical / rendering (from the spine):**
| ID | Risk | L×I | Mitigation |
|---|---|---|---|
| W1 | softbuffer can't do per-pixel alpha on a Windows layered window | HIGH | winit owns the `WS_EX_LAYERED` HWND; tiny-skia → premultiplied BGRA; present via `UpdateLayeredWindow` directly. softbuffer = X11/Wayland only. |
| W2 | Click-through vs clickable conflict | HIGH | Per-pixel-alpha natural hit-test (no `WS_EX_TRANSPARENT`); fallback ex-style toggle; X11 XShape input region. (§6) |
| G1 | AV/SmartScreen flags an unsigned app that warps cursor + synth keys + moves windows | HIGH | Personal use: document "Run anyway"; keep runtime control on local IPC; ship source. Optional Authenticode later. |
| M_perm | macOS Accessibility/Input-Monitoring gates; a bare binary can't hold a stable grant | HIGH | universal2 `.app` (stable bundle-id) mandatory; `AXIsProcessTrusted()`, deep-link to Settings, degrade. |
| E1 | 120 Hz full-screen layered redraw = CPU/battery killer | HIGH→mit | Per-monitor windows + present only the dirty rect; sim 120 Hz, present on-dirty ~60. Idle ≈ near-zero. |
| W_dpi | Per-monitor DPI + signed/negative multi-monitor coords | MED-HIGH | Per-monitor windows (single-DPI each); Per-Monitor-V2 awareness; signed virtual space; handle `WM_DPICHANGED`. |
| L_xwl | XWayland window-move no-ops on native-Wayland windows | MED | `enumerate()`/`watch_move()` return only X11 windows; tasks targeting non-enumerable windows self-skip. |
| T_term | Goose mischief targets a terminal window and disrupts active CLI/TUI work | HIGH | Backend protected-window filters exclude terminal windows before foreign-window ride, collect-window, or spicy behavior code can target them. |
| E_rng | Original Deck shuffle is biased | LOW | Port faithfully with a `// faithful-to-original (biased)` note; M0 pins it. Verify `gooseTaskWeightedList`. |

**Operational / distribution (grafted):**
| ID | Risk | Mitigation |
|---|---|---|
| O_tele | Privacy / phone-home perception | **No telemetry, no network** except `<name> update` (only `update.rs` has an HTTP client). |
| O_ipc | IPC channel abused | Local-only pipe/socket, no network, authenticated to the same user; commands are a closed enum (Stop/Do/Reload). |
| O_supply | Dependency supply chain | Pin `Cargo.lock`; `cargo audit`; prefer pure-Rust crates (tiny-skia, rodio, symphonia, ureq, sha2, ratatui). |
| O_lock | Locked running binary on update | Windows Installer Restart Manager; mac/Linux: ask to quit or detached-helper swap after exit; surface msiexec 3010. |
| O_cargo | Accidental crates.io contamination | No `cargo install` update strategy; test that no strategy invokes `cargo install`; post-install `--version` always. |
| O_arch | ARM build/test gaps | Build all arches in CI; note any emulation-only test coverage honestly; arch-matched self-update. |
| O_race | TUI/engine config races | Reducer-only TUI state + versioned config + the `Reload` hot-apply protocol (engine re-reads atomically). |
| O_mood | Mood-system scope creep | Bounded by the procedural-rig guardrail (§5.2): parameter modulation only, no new art/animation systems. |

**Genuinely impossible (documented, not fought):** native-Wayland foreign-window move /
cursor-warp / keystroke; softbuffer per-pixel alpha on a Windows layered window;
durable macOS Accessibility for an un-bundled binary.

---

## 16. Corrections to naive assumptions (baked in)
- **C1** — Per-monitor overlay windows, not one virtual-screen window. Sim stays one signed space.
- **C2** — Windows present ≠ softbuffer; use `UpdateLayeredWindow`. softbuffer is X11/Wayland-only.
- **C3** — Edition: TR/ND = 2021, WB = 2024. honk300 matches the family at **2021** (1.95
  supports either). The **workspace** is an intentional divergence from the single-crate `*300`
  repos.
- **C4** — Start, stop, and configuration are **CLI/TUI-only over IPC, no tray, no global quit key**.
- **C5** — `<name> install` ≠ "PATH + shell-autorun alias." For a GUI app it means autostart +
  shortcut/.desktop/LaunchAgent + `InstallSource` marker; install all three aliases.
- **C6** — Reconsider `install-path`: dedicated install dirs over `CARGO_HOME`; keep
  `install-updater=false` + our own SHA-256, **arch-matched** `<name> update`.
- **C7** — Cursor pos on Windows via native `GetCursorPos`; device_query for X11.
- **C8** — RNG fidelity is an explicit decision (faithful-biased) pinned by M0 tests.
- **C9** — Config is **TOML**, not `.ini` (family convention; original keys/values preserved).
- **C10** — **No external mods**; Autumn is built-in; extensibility = documented internal seams.
- **C11** — Single-instance + **IPC command channel** (stop/do/reload) underpins quit, pokes, and
  the TUI's hot-apply; a **ratatui** config TUI replaces any tray/GUI settings.
- **C12** — Terminal windows are protected from all goose mischief, including default-off spicy
  behaviors; overlay rendering may cover them visually, but platform backends must never move,
  focus, type into, ride, drag, collect, or otherwise manipulate them.
- **C13** — Build **every OS + arch** (Win x64+ARM64, mac universal2, Linux x64/ARM gnu+musl);
  arch is a build axis, capability is an OS axis.

---

## 17. Verification & testing strategy (implementation round)
- **Engine unit tests (`honk-engine`, `#![forbid(unsafe_code)]`):** assert rig vertex positions,
  locomotion speed/accel, footmark lifetimes, and the **Deck sequence** against hand-computed
  values from the verified C# constants. **Mood-state transitions**, **perch-ride arrive/abandon**,
  **pat-streak hover** logic. CPU-only, no OS.
- **Golden-frame tests:** render `Frame` → PNG via tiny-skia offscreen at fixed timestamps;
  assert against committed goldens (AA tolerance). Catches rig/mood regressions visually.
- **`mock_host` integration tests:** a fake platform recording window/sound/IPC intents — assert
  `collect_window` spawns the right window kind, `nab` issues cursor moves, `Reload` re-applies
  config — without touching the real OS. **Config round-trip + hot-apply IPC test.** **TUI reducer
  tests** (key → action → state).
- **Protected-window tests:** platform backends must classify terminal windows and prove foreign-
  window ride, collect-window, and spicy behavior paths do not target them.
- **Per-platform manual matrix** (`TESTING.md`): overlay transparency + click-through + clickable
  goose; wander; footmarks; honk/mute; cursor-grab; window-drag + perch-ride; notepad-type;
  meme-drop; pat-hover hearts; moods; seasonal; multi-monitor + mixed-DPI; **start/stop grammar**
  (`start`/`stop`/`honk bad`/`goose no honk`); **`goose config` hot-apply**; autostart on/off;
  terminal windows are visually overlaid but not manipulated.
- **Degradation tests:** macOS without Accessibility; Wayland (`--wayland` and default XWayland);
  X11 with a native-Wayland window present (perch-ride/window-drag self-skip).
- **Packaging smoke across every target/arch:** install → `--version` → `<name> stop` on Win
  x64+ARM64, mac Intel + Apple Silicon (universal2), Linux x64 + ARM (gnu + musl); verify
  Start-Menu/.desktop/LaunchAgent, `InstallSource`, arch-matched `<name> update`, `uninstall
  --purge` leaves nothing behind.
- **Local gate (family standard):** `cargo fmt --all -- --check`,
  `cargo clippy --all-targets --workspace -- -D warnings`, `cargo test --workspace`,
  `cargo build --release`, run the binary.

---

## 18. Internal extensibility guide (the replacement for a mod system)
Since there is **no external mod surface**, the supported way to extend the goose is to edit the
source. The repo ships:
- **`ARCHITECTURE.md`** — the core/platform split, the loop, the task/mood/season model, the
  `Cap<T>` degradation pattern.
- **"Adding a task / sound / season"** how-to guides (Autumn is the worked example of a season).
- **Thorough rustdoc** on `honk-engine`'s task-registry seams (the same internal hook points the
  original exposed as `InjectionPoints`, kept clean but not loadable).

---

## 19. Out of scope / future
Code signing (Authenticode / Apple notarization); App Store / store distribution; macOS
AppleScript `.sdef` surface; **any external/WASM mod system** (explicitly future, low priority —
internal docs are the chosen extensibility path); Music/streaming features; the default-OFF spicy
behaviors (§5.12) remain opt-in extras; exotic tiling-WM polish. Native Wayland mischief stays
intentionally limited.

---

## 20. Appendix
- **Engine port source-of-truth:** `DESKTOP-GOOSE/DesktopGoose v0.31/FOR MOD-MAKERS/
  GooseMod_DefaultSolution/GooseModdingAPI/{SamEngine.cs, Exports.cs}`; `…/DefaultMod/
  {ModMain.cs, TaskDemo_FollowLowAccel.cs}`; `…/config.ini`; Mac `DesktopGoose.sdef` + `Notes/`.
- **Family conventions to mirror:** `qube-workbranch-view/{Cargo.toml, src/{app,ui,config}.rs,
  .github/workflows/windows-installers.yml, wix/, wix-corporate/, inno/}`;
  `qube-machine-report/src/{install/*, update.rs}`, `build.rs`.
- **Constants tables:** §3.3 (physics), §3.4 (rig), §11 (config).
- **File/path lockstep contract:** install paths + `InstallSource` markers in the four installer
  files must change **together with** `update.rs::detect_install_origin()` and `uninstall.rs` in
  the same commit (family discipline).
- **CLI grammar / goose-speak alias table:** §8.1.
- **Control / poke / IPC matrix per OS:** §4 table + §8.

### Document control
- **This round:** produce this plan only. **No goose code written.**
- **Next round (separate):** execute from §14 milestones, starting at M0.
- **Canonical:** this file supersedes `claude_plan.md` and `codex_plan.md` (retained as reference).
