# honk300 — Cross-Platform Rust Desktop Goose: Research & Implementation Plan

> **Status of this document:** This is the deliverable for *this* round — a thorough, adversarially‑reviewed research + implementation plan. **No application code is written yet.** Implementation happens in a later, separate round.

---

## 1. Context — why this exists

`README.md` asks us to take the files in the `DESKTOP-GOOSE/` folder, analyze them thoroughly, and build **an entirely new version of Desktop Goose** — *same functionality, same features* — but:

- Built in **Rust**.
- Working on **macOS, Linux, and Windows**.
- With **native installers just like TR300, ND300, and WB300** — except **not distributed via cargo/crates.io**; only via builds, installers, and scripts.

"Desktop Goose" (by Samperson) is a desktop‑pet: an annoying goose wanders your screen, leaves muddy footprints, honks, grabs your cursor, drags your windows around, opens Notepad to type at you, and brings you memes. The original is closed‑source C#/.NET (Windows `GooseDesktop.exe`; macOS is a Xamarin/Mono `.app`). The reference `*300` tools are this machine's existing Rust CLI/TUI family (`qube-machine-report` = TR300, `qube-network-diagnostics` = ND300, `qube-workbranch-view` = WB300) with a mature cross‑platform installer pipeline we will mirror.

**Intended outcome:** a single self‑contained Rust binary, `honk300`, that recreates Desktop Goose's behavior across the three OSes, packaged with the same family of installers (4 Windows installers + shell/PowerShell installers + GUI bundles), installable on a personal machine without touching crates.io.

**Scope note (personal use):** The user is installing this on a personal device. That relaxes code‑signing/notarization and copyright‑redistribution constraints (documented below), but we still build the *full* installer matrix because matching the `*300` family is an explicit goal.

### Locked decisions (from user)
| Topic | Decision |
|---|---|
| **Name** | `honk300` (binary `honk300`; optional `honk` alias). Joins the `*300` family. Generate **fresh permanent** WiX/Inno GUIDs (never reuse the reference repos'). |
| **Goose visual** | **Procedural** (clean‑room) from the public modding‑API constants — no sprite extraction needed. |
| **Sounds** | Bundle the original honk/bite/mud/pat audio **1:1**, embedded (personal use). Keep them out of any source/package artifact. |
| **Memes** | **Do not copy** the meme images. Ship `Assets/Images/Memes/codex.md` with per‑slot instructions for Codex's image‑gen tools to recreate each meme *originally*. Empty slots are skipped at runtime. |
| **Notes** | Author **brand‑new, original** goose‑voiced notepad messages (a fresh, sarcastic, honking register) — **not** paraphrases of the originals. |
| **Linux** | **X11‑first** default (runs under XWayland on Wayland sessions). **Native Wayland** (wlr‑layer‑shell, reduced mischief) behind an opt‑in `--wayland` flag. |
| **Packaging** | Windows‑first full 4‑installer matrix (Global/Corporate × MSI/EXE) + shell/PowerShell installers that install **and configure**; macOS `.app`/`.dmg` + Linux `.desktop` as best‑effort GUI essentials. **No crates.io** (`crates-publish.yml` dropped). |

---

## 2. Source analysis — the original Desktop Goose

Two distributions are present in `DESKTOP-GOOSE/`:

- **`DesktopGoose v0.31/`** (Windows): `GooseDesktop.exe`, `GooseModdingAPI.dll`, `MMQ.dll`, `config.ini`, an `Assets/` tree, a `FOR MOD-MAKERS/` Visual Studio solution, and readme/text files.
- **`Desktop Goose for Mac v0.22/`**: a Xamarin.Mac `.app` (Mono bundle + `Desktop Goose.exe`), `Info.plist`, `DesktopGoose.sdef` (AppleScript surface), `runtime-options.plist`, sounds, memes, notes.

### 2.1 The single most important finding: the goose is **procedurally rendered**, not a sprite
There is **no goose sprite folder** anywhere in the assets. The goose is drawn at runtime from a geometric **rig** of primitives. This is confirmed by the shipped, mod‑maker‑facing source (intended to be read by modders):

- `…/FOR MOD-MAKERS/GooseMod_DefaultSolution/GooseModdingAPI/SamEngine.cs`
- `…/FOR MOD-MAKERS/GooseMod_DefaultSolution/GooseModdingAPI/Exports.cs`
- `…/DefaultMod/ModMain.cs`, `…/DefaultMod/TaskDemo_FollowLowAccel.cs`

**Consequence:** we reimplement the *renderer* (clean‑room) instead of extracting copyrighted art. The `config.ini` colors (`GooseDefaultWhite=#ffffff`, `GooseDefaultOrange=#ffa500`, `GooseDefaultOutline=#d3d3d3`) map directly to the `brushGooseWhite/Orange/Outline` solid brushes in `GooseRenderData`.

### 2.2 Engine model (port source‑of‑truth: `SamEngine.cs` / `Exports.cs`)
- **Fixed timestep:** 120 FPS, `deltaTime = 1/120`, driven by a `Stopwatch`. *(Locomotion/step constants are tuned to this rate; we must not couple the sim to a variable redraw rate.)*
- **Math:** `Vector2` struct; `SamMath` (`Deg2Rad`, `Rad2Deg`, `RandomRange`, `Lerp`, `Clamp`).
- **`Deck`** — a shuffle‑bag RNG (no repeats until the bag is exhausted), used to pick tasks/props without immediate repetition. ⚠️ The original `Reshuffle()` is a *biased* shuffle (`RandomRange(0, j)` with low‑bound 0 and exclusive high‑bound `j`; seeded by `System.Random`) — **not** a correct Fisher–Yates. See §11 (E_rng) for the fidelity decision.
- **Input:** `mouseX`, `mouseY`, `leftMouseButton` (a `ButtonState` with `Held`/`Clicked`/`Released`), refreshed each frame.

### 2.3 Goose entity + parameters (exact constants to port)
`GooseEntity.ParametersTable`:

| Constant | Value | Meaning |
|---|---|---|
| `WalkSpeed` / `RunSpeed` / `ChargeSpeed` | 80 / 200 / 400 | max speed per `SpeedTiers {Walk, Run, Charge}` |
| `AccelerationNormal` / `AccelerationCharged` | 1300 / 2300 | accel in Walk/Run vs Charge |
| `StepTimeNormal` / `StepTimeCharged` | 0.2 / 0.1 | foot step interval (s) |
| `StopRadius` | −10 | stop tolerance |
| `DurationToTrackMud` | 15 | seconds the goose leaves muddy prints |

State: `position`, `velocity`, `direction` (deg, default 90), `targetDirection`, `targetPos`, `currentSpeed`, `currentAcceleration`, `stepInterval`, `extendingNeck`, `canDecelerateImmediately`. The engine **auto‑locomotes** toward `targetPos`; tasks only set targets/accel.

`FootMark`: `Lifetime = 8.5s`, `ShrinkTime = 1s`; ring buffer `footMarks[64]`.

### 2.4 The Rig (exact geometry — `Exports.cs`)
Drawn back‑to‑front: shadow → underbody → body → neck (two lerped positions) → head (two segments) → eyes → procedural feet. Constants:

- **UnderBody:** radius 15, length 7, elevation 9
- **Body:** radius 22, length 11, elevation 14
- **Neck:** radius 13; pos‑1 (height 20, forward 3), pos‑2 (height 10, forward 16), blended by `neckLerpPercent`
- **Head:** seg‑1 (radius 15, length 3), seg‑2 (radius 10, length 5)
- **Eyes:** radius 2, elevation 3, IPD 5, forward 5
- **Feet (`ProceduralFeets`):** `feetDistanceApart 6`, `wantStepAtDistance 5`, `overshootFraction 0.4`

("radius + length" ⇒ capsule/stadium shapes; "elevation" is a vertical offset for the fake‑3D + shadow.)

### 2.5 AI = a Task state machine
- `GooseTaskInfo { canBePickedRandomly, shortName, description, taskID, GetNewTaskData(goose), RunTask(goose) }`.
- A **`TaskDatabase`** holds all tasks; default **roaming/wandering** state picks a random pickable task via the `Deck`. `taskIndexQueue` lets a task queue successors.
- Helpers (to reimplement): `setSpeed`, `setTargetOffscreen(canExitTop)`, `isGooseAtTarget(dist)`, `getDistanceToTarget`, `setCurrentTaskByID`, `chooseRandomTask`, `setTaskRoaming`, `playHonkSound`, `getModDirectory`, task‑DB queries.
- **Demo task** (`TaskDemo_FollowLowAccel.cs`) shows the contract: each frame set `currentAcceleration` + `targetPos = (mouseX, mouseY)`; after a duration call `setTaskRoaming(goose)`.
- **Mod injection points** (`InjectionPoints`): `PostModsLoaded`, `PreTick`/`PostTick`, `PreUpdateRig`/`PostUpdateRig`, `PreRender`/`PostRender`. Mods implement `IMod.Init()` and subscribe.

### 2.6 Behavior parity list (full set to reproduce)
**Movement/visual:** procedural goose; Walk/Run/Charge; autonomous wander with config timing; fading muddy footprints; recolorable (white/orange/outline); always‑on‑top transparent overlay spanning monitors.
**Mischief / OS‑interaction:** grab & drag the **user's mouse cursor**; **drag the user's other app windows** around; **drop meme images** onto the desktop (draggable props); **open the native text editor and type** a message; aggression toggles (`Task_CanAttackMouse`, `AttackRandomly`).
**Audio:** 4 honks + bite + mud‑squish (+ Mac pat sounds); `SilenceSounds` mute.
**Config/extensibility:** `config.ini` (see §8.4); drop‑in memes & notepad messages; DLL/plugin **mod system** with an **Autumn** leaves mod; **hold ESC ~3s to quit**.

### 2.7 Asset inventory (`DESKTOP-GOOSE/DesktopGoose v0.31/Assets/` + Mac `Resources/`)
- **Sounds:** `Honk1-4.mp3`, `BITE.mp3`, `MudSquith.mp3`, optional `Music.mp3`; Mac adds `Pat1-3.wav`. → **bundle 1:1, embedded.**
- **Images/Memes:** `Meme1-7.png`, `GooseDance.gif`, `MemeAttributions.txt` (third‑party memes). → **regenerate originally via `codex.md`; do not copy.**
- **Images/OtherGfx:** `DonatePage.png`, `heart.png` (pet feedback). → recreate originally / procedural.
- **Text/NotepadMessages:** several `.txt` one‑liners. → **author fresh originals.**
- **Mods/Autumn/Autumn.dll** + `Autumn.txt`. → reimplement Autumn as an in‑tree Rust mod.
- Icons: Windows `.exe` icon, Mac `AppIcon.icns`/`Assets.car`. → design an original `honk300` icon.

---

## 3. Reference‑family conventions to mirror (`TR300` / `ND300` / `WB300`)

Verified directly in `qube-workbranch-view/Cargo.toml`, `qube-workbranch-view/.github/workflows/windows-installers.yml`, and `qube-machine-report/src/install/*`, `…/src/update.rs`.

- **Toolchain:** `rust-version = "1.95"` pinned in lockstep with `rust-toolchain.toml` (`channel = "1.95"`, components rustfmt+clippy). **Edition is `2021`** in TR300/ND300 (WB300 uses `2024`). → see §12 C3.
- **Metadata:** `authors = ["Emmett S <hey@emmetts.dev>"]`, `license = "PolyForm-Noncommercial-1.0.0"`, `repository` under `QubeTX/`. `include = […]` explicitly lists packaged paths (we exclude embedded sounds from it).
- **Profile:** `[profile.dist] inherits="release"; lto="thin"`.
- **Packaging (the headline):** `cargo-dist 0.31.0`, `installers = ["shell","powershell","msi"]`, 6 target triples (`{aarch64,x86_64}-apple-darwin`, `{x86_64,aarch64}-unknown-linux-gnu`, `x86_64-unknown-linux-musl`, `x86_64-pc-windows-msvc`), `install-updater=false`, `pr-run-mode="plan"`, `[package.metadata.wix] upgrade-guid/path-guid`, `allow-dirty=["msi"]`.
- **Hand‑authored `windows-installers.yml`** (fires via `workflow_run` after cargo‑dist's `release.yml`, guards torn releases by probing `dist-manifest.json` + the Global MSI): builds **Corporate MSI** (`wix-corporate/corporate.wxs` via WiX `candle`/`light`, with `-sice:ICE38/64/91`), **Global EXE** (`inno/global.iss`), **Corporate EXE** (`inno/corporate.iss`), each with a `.sha256` sidecar, uploaded with `gh release upload --clobber`.
- **Install locations / registry markers:** Global → `C:\Program Files\<app>\bin\<app>.exe` (system PATH); Corporate → `%LocalAppData%\Programs\<app>\bin\<app>.exe` (user PATH). `HKCU\Software\<APP>\InstallSource ∈ {msi-global, msi-corporate, exe-global, exe-corporate}` — read by `<app> update` to re‑run the matching installer.
- **In‑binary `install/uninstall/update`** (`src/install/{mod,shared,windows,unix}.rs`, `src/update.rs`): atomic writes (temp→fsync→rename), marker‑delimited rc blocks, one‑time backup, SHA‑256‑verified self‑update from the GitHub Releases API, registry `InstallSource` detection on Windows.
- **Docs/process:** `build.rs` man‑page generation (`clap_mangen`); dual `CHANGELOG.md` + `HUMAN_CHANGELOG.md` (lockstep); `AGENTS.md`/`CODEX_PROJECT.md`; `.claude/skills/{release, *-dev-workflow, *-changelog}`; error handling via `thiserror` (TR300/ND300) or `color-eyre` (WB300).
- **Drop for honk300:** `crates-publish.yml` (no crates.io). Reconsider `install-path` (see §12 C6).

---

## 4. Cross‑platform technical landscape + chosen stack

| Capability | Windows | macOS | Linux X11 / XWayland | Native Wayland |
|---|---|---|---|---|
| Transparent always‑on‑top borderless overlay | ✅ layered window | ✅ NSWindow | ✅ override/EWMH `_NET_WM_STATE_ABOVE` | ⚠️ wlr‑layer‑shell only (anchored, no free placement on some compositors) |
| Click‑through **but** goose stays clickable | ✅ per‑pixel‑alpha hit‑test | ✅ `ignoresMouseEvents` toggled per‑region | ✅ XShape input region = goose bbox | ⚠️ no true click‑through protocol |
| Warp the user's cursor | ✅ `SetCursorPos`/enigo | ✅ `CGWarpMouseCursorPosition` (Accessibility) | ✅ `XWarpPointer`/enigo | ❌ blocked by protocol |
| Move **other** apps' windows | ✅ `SetWindowPos` | ✅ AXUIElement (Accessibility) | ✅ EWMH `_NET_MOVERESIZE_WINDOW` (X11 windows only) | ❌ **impossible** by design |
| Synthesize keystrokes (Notepad) | ✅ SendInput/enigo | ✅ CGEvent (Accessibility/Input Monitoring) | ✅ XTEST/enigo | ❌ blocked |
| Global key (hold‑ESC) | ✅ RegisterHotKey/raw input | ⚠️ CGEventTap (Input Monitoring) | ✅ XGrabKey | ❌ no global grab |
| Read global cursor pos under click‑through | ✅ `GetCursorPos` | ✅ CG | ✅ `device_query` | ⚠️ limited |
| Audio | ✅ `rodio` | ✅ | ✅ | ✅ |

**Chosen crates:** `winit` (window/event loop + monitor enumeration), `tiny-skia` (CPU vector raster of the procedural goose), `softbuffer` (present on **X11/Wayland only**), `windows` crate (layered window + `UpdateLayeredWindow` + `GetCursorPos`/`SetWindowPos`/`EnumWindows` + `RegisterHotKey`), `objc2`/`objc2-app-kit` (+ Accessibility/CoreGraphics) on macOS, `x11rb` (X11 + XShape + EWMH), `smithay-client-toolkit`/`gtk4-layer-shell` (`--wayland`), `enigo` (cursor warp + keystrokes), `device_query` (X11 polling), `rodio` (audio), `rust-embed` (assets), `serde`+`ini`/`serde` for config, `clap` (CLI), `thiserror` (errors).

**Hard impossibilities (not just hard) — documented, not fought:**
1. Moving **native‑Wayland** app windows from an external client — no protocol, by design.
2. Cursor warp / keystroke synth / global key grab on **native Wayland** for an unprivileged client.
3. **softbuffer providing per‑pixel alpha on a Windows layered window** — must use `UpdateLayeredWindow` directly.
4. A **bare (un‑bundled) macOS binary** holding a durable Accessibility grant — a real `.app` bundle (stable bundle‑id) is mandatory for the mischief features.

---

## 5. Target architecture

A Cargo **workspace** (a deliberate, justified divergence from the single‑crate `*300` repos — it forces the platform‑agnostic engine to never `use` an OS crate). The shipped artifact remains **one binary**, keeping cargo‑dist/installers happy.

```
honk300/                       (workspace root)
├─ crates/
│  ├─ honk-engine/             # #![forbid(unsafe_code)] — NO winit/OS crates
│  │   ├─ math.rs  time.rs  rng.rs (Deck)
│  │   ├─ entity.rs  rig.rs  feet.rs  footmarks.rs  locomotion.rs
│  │   ├─ task.rs  tasks/{wander,honk,mud,grab_cursor,drag_window,meme,notepad,autumn}.rs
│  │   ├─ render.rs            # Rig -> tiny_skia::Pixmap, dirty-rect aware
│  │   ├─ mods.rs              # injection-point hooks
│  │   └─ world.rs            # owns sim state; tick(dt, &Input, &dyn Platform)
│  ├─ honk-platform/           # traits + shared types (ScreenRect, Input, KeyEvent, Cap<T>)
│  ├─ honk-platform-windows/   # layered-window + UpdateLayeredWindow backend
│  ├─ honk-platform-macos/     # objc2-app-kit + Accessibility backend
│  ├─ honk-platform-x11/       # x11rb + XShape backend
│  ├─ honk-platform-wayland/   # wlr-layer-shell (degraded) backend
│  └─ honk-assets/             # rust-embed + extraction + override precedence
└─ src/main.rs                 # honk300 binary: clap CLI, config, install/update, loop
```

### 5.1 Platform capability traits
Split by capability so a backend can report **per‑capability** support (`Cap<T> = Ok | Unsupported | Denied | Failed`) instead of failing wholesale — this is what makes "Wayland = reduced mischief" and "macOS without permission = degraded" fall out of the design rather than being special‑cased everywhere.

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
trait ForeignWindows { fn enumerate(&self)->Cap<Vec<ForeignWin>>; fn move_to(&self,w:ForeignWin,p:Point)->Cap<()>; }
trait Synth { fn type_text(&self,s:&str)->Cap<()>; fn launch_text_editor(&self)->Cap<EditorHandle>; }
trait Audio { fn play(&self, clip: ClipId); }
trait GlobalKeys { fn poll(&self)->Vec<KeyEvent>; }   // focus-independent
```

### 5.2 The loop — three clocks
- **Sim = fixed 120 Hz** accumulator (`while acc >= dt { world.tick(dt) }`, clamp catch‑up to ~5 ticks to avoid spiral‑of‑death).
- **Input poll = 120 Hz** (cursor pos + buttons + global keys feed `Input`).
- **Present = on‑dirty, rate‑capped (~60)** — render only the **dirty rect** around the goose + active props into a `Pixmap`, present that sub‑region. Idle goose ≈ near‑zero present cost.
- Driven by winit 0.30 `ApplicationHandler::about_to_wait` + `ControlFlow::WaitUntil(next_tick)` (sleep precisely, no busy‑spin). The sim is **decoupled** from `RedrawRequested` (which is vsync/compositor‑bursty).

### 5.3 Per‑monitor windows (key correction)
**One overlay window per monitor**, not one giant virtual‑screen window. The engine simulates in one continuous virtual‑desktop coordinate space (signed `i32`), but each window presents only its monitor's region. This makes each surface single‑DPI, monitor‑bounded, avoids negative‑coordinate/huge‑buffer/mixed‑DPI pain, and collapses present cost to the goose's bbox.

---

## 6. Click‑through vs. clickable — the crux, resolved
The goose must let clicks pass through *everywhere except itself* (you pet/grab it). You can't be globally click‑through **and** receive clicks. Resolution:

- **Windows (primary):** with `UpdateLayeredWindow` per‑pixel alpha **and without `WS_EX_TRANSPARENT`**, Windows naturally routes clicks: opaque goose pixels receive them, fully‑transparent pixels fall through to apps beneath. Fallback: per‑frame toggle of `WS_EX_TRANSPARENT` based on whether the cursor is over the goose bbox (poll `GetCursorPos`, `SetWindowLongPtr`). Use `WS_EX_LAYERED | WS_EX_TOOLWINDOW | WS_EX_TOPMOST | WS_EX_NOACTIVATE`.
- **X11:** set the **XShape input region = goose bbox each frame**, empty elsewhere.
- **macOS:** toggle `ignoresMouseEvents` based on cursor‑over‑goose, or use a tracking region.
- This is exactly why `Overlay::set_input_region` exists in the trait.

---

## 7. Behavior/feature spec → per‑platform capability

| Behavior | Win | macOS | X11/XWayland | `--wayland` |
|---|---|---|---|---|
| Procedural goose + wander + footmarks + honk + recolor + config + mods/Autumn | ✅ | ✅ | ✅ | ✅ |
| Pet/grab the goose (hit‑testing) | ✅ | ✅ | ✅ | ⚠️ best‑effort |
| Grab & drag the **user's cursor** | ✅ | ✅ (Accessibility) | ✅ | ❌ self‑disables |
| Drag the **user's app windows** | ✅ | ✅ (Accessibility) | ✅ (X11 windows only) | ❌ self‑disables |
| Open editor + type message | ✅ | ✅ (Accessibility/Input Mon.) | ✅ | ❌ self‑disables |
| Drop meme props | ✅ | ✅ | ✅ | ✅ (render‑only) |
| Hold‑ESC quit | ✅ RegisterHotKey | ⚠️ Input Monitoring | ✅ XGrabKey | ❌ tray/menu quit |

**Graceful degradation:** any task whose required `Cap` is `Unsupported`/`Denied` self‑skips and returns to roaming. Every platform also ships a **non‑global quit path** (tray/menu‑bar/context gesture) because hold‑ESC is not universally available (§12 C4).

---

## 8. Asset strategy

### 8.1 Embedding + extraction + precedence
- **Embed** sounds (and the procedural goose needs none) via `rust-embed`. On first run or `honk300 setup`, **atomically extract** a writable, user‑editable `Assets/` tree + `config.ini` to a per‑user data dir (`%LOCALAPPDATA%\honk300\` / `~/.local/share/honk300/` / `~/Library/Application Support/honk300/`).
- **Override precedence:** user‑override dir **>** extracted dir **>** embedded fallback.
- **Update safety:** store a content‑hash manifest; on app update, re‑extract only assets the user hasn't modified (never clobber user edits). Missing meme/note → skip, never crash.

### 8.2 Memes → `codex.md` (no copying)
Ship `Assets/Images/Memes/codex.md` enumerating slots `Meme1…Meme7` + `GooseDance` (animated), each with a short prompt brief instructing Codex's image‑gen tools to produce an **original** image in the same role/format (square‑ish desktop meme; looping goose dance for the GIF). Runtime treats any present `MemeN.png`/`.gif` as a draggable prop; absent slots are skipped.

### 8.3 Notes → original goose‑voiced messages
Author a fresh set of **original** one‑liners in a sarcastic, honking, mischievous register (e.g., short typed taunts the goose "writes" in Notepad). These are newly written, **not** paraphrases of the originals. Stored as individual `.txt` files in `Assets/Text/NotepadMessages/` and picked via the `Deck`.

### 8.4 `config.ini` schema parity
Replicate keys: `Version_DoNotEdit`, `EnableMods`, `SilenceSounds`, `Task_CanAttackMouse`, `AttackRandomly`, `UseCustomColors`, `GooseDefaultWhite/Orange/Outline`, `MinWanderingTimeSeconds`, `MaxWanderingTimeSeconds`, `FirstWanderTimeSeconds`. Add honk300‑specific keys as needed (e.g., `Autostart`, `EnableWindowDragging`, `PresentFpsCap`) with safe defaults; unknown keys ignored with a single warning (mirrors WB300's tolerant TOML loader).

---

## 9. Packaging & distribution pipeline

**Reuse the family pipeline, minus crates.io, plus GUI bundling.**

### 9.1 cargo‑dist (`[workspace.metadata.dist]`)
- `cargo-dist-version = "0.31.0"`, `ci = "github"`, `installers = ["shell","powershell","msi"]`, the same 6 target triples, `pr-run-mode = "plan"`, `install-updater = false`, `allow-dirty = ["msi"]`, `publish-prereleases = false`.
- `[package.metadata.wix]` with **freshly generated** `upgrade-guid` + `path-guid` (never reuse TR/ND/WB GUIDs).
- `[profile.dist] inherits="release"; lto="thin"`.
- **No `crates-publish.yml`.** Self‑update pulls installers from **GitHub Releases** (works without crates.io). Reconsider `install-path` (§12 C6) — prefer dedicated install dirs over `CARGO_HOME` for the GUI binary.
- `windows_subsystem = "windows"` on the binary (no console flash), with a CLI fallback path for `--help`/`install`/`update` (allocate a console when run from a terminal, or keep a thin separate behavior).

### 9.2 Windows: 4 installers (Global/Corporate × MSI/EXE)
Adapt `wix/main.wxs` (Global MSI, perMachine, `Program Files\honk300\bin`, system PATH, `InstallSource=msi-global`), `wix-corporate/corporate.wxs` (perUser, `%LocalAppData%\Programs\honk300\bin`, `InstallSource=msi-corporate`), `inno/global.iss` (admin, `exe-global`), `inno/corporate.iss` (lowest, `exe-corporate`). **Add for a GUI app:** a Start‑Menu (and optional desktop) `Shortcut`, and an **optional** `HKCU\…\Run` autostart entry (checkbox, default off). Port `windows-installers.yml` verbatim with renamed artifacts + the torn‑release guard + `.sha256` sidecars + `workflow_run`‑after‑`Release` trigger.

### 9.3 macOS: `.app` + `.dmg` (GUI essential — required, not optional)
A real `.app` bundle with a **stable bundle‑id** (`dev.emmetts.honk300` or similar) is **mandatory** for durable Accessibility grants. Generate `Info.plist`, bundle the binary + embedded assets, wrap in a `.dmg` (`create-dmg` or `hdiutil`). Unsigned/un‑notarized for personal use → document the `xattr -dr com.apple.quarantine` workaround; degrade mischief gracefully until Accessibility is granted. (Notarization is a $99/yr Apple Developer gate — out of scope for personal use, noted as future.)

### 9.4 Linux: `.desktop` + tarball/AppImage, X11‑first
Shell installer drops the binary, extracts assets, and installs `~/.local/share/applications/honk300.desktop` + optional `~/.config/autostart/honk300.desktop`. AppImage is a best‑effort extra. Default runs X11/XWayland; `--wayland` opts into the degraded layer‑shell mode.

### 9.5 In‑binary `honk300 install / uninstall / update / setup` (GUI semantics)
Reuse the family's atomic‑write + marker‑block + symlink‑resolution machinery (`qube-machine-report/src/install/mod.rs`) and SHA‑256 self‑update (`…/src/update.rs`), **but** "install" means **login‑autostart + a launchable shortcut/.desktop/LaunchAgent + asset extraction + `InstallSource` marker** — **not** a `.bashrc`/PowerShell‑profile alias‑autorun (a shell‑autorun for a *windowed* app would wrongly spawn a goose on every shell start). The `honk` PATH alias is a **secondary, flag‑gated convenience**, not the headline.

---

## 10. Build milestones (later round — listed for completeness)
Each is independently runnable; M6 (hit‑testing) is pulled early because it's the highest‑risk primitive; platforms come after the full Windows feature set so the platform trait is fully shaped before being implemented 3 more times.

| # | Milestone | Done‑when |
|---|---|---|
| M0 | Workspace + `honk-engine` (math/time/Deck/entity/rig/feet) ported 1:1, unit‑tested vs hand‑computed constants | `cargo test` green; Deck sequence pinned |
| M1 | Windows transparent click‑through topmost overlay renders a **static** procedural goose (primary monitor) | goose floats; clicks pass through |
| M2 | 120 Hz accumulator + locomotion + dirty‑rect present + procedural feet | goose walks at correct speed; low CPU |
| M3 | Footmarks + mud (8.5s/1s, 15s duration) | fading prints trail the goose |
| M4 | Task state machine + wander + config timing | autonomous wandering with config gaps |
| M5 | Audio + honk + `SilenceSounds` | honks play; mute respected |
| M6 | Input region / hit‑testing: pet + grab‑the‑goose | click goose pets; click empty passes through |
| M7 | Cursor mischief (warp + grab user's cursor) | goose drags the real cursor |
| M8 | Foreign‑window dragging (Windows) | goose drags a real app window |
| M9 | Keystroke synth + Notepad + original messages | goose types an original taunt |
| M10 | Meme props (extracted/codex‑generated) | goose drags a meme onto the desktop |
| M11 | Full config + CLI + man page + recolor | every toggle works |
| M12 | Mod hooks + Autumn (Rust) | Autumn drops leaves |
| M13 | Hold‑ESC quit + tray/menu quit fallback | quits cleanly |
| M14 | macOS backend + `.app` + permission‑gated degradation | runs on macOS, degrades gracefully |
| M15 | Linux X11 backend (XShape + EWMH + device_query) | full parity on X11/XWayland |
| M16 | `--wayland` layer‑shell degraded mode | renders on Wayland; mischief self‑disables |
| M17 | install/update/setup + full packaging pipeline | installers produce working artifacts w/ autostart + shortcut |

---

## 11. Adversarial red‑team — ranked risks & mitigations

| ID | Risk | L×I | Mitigation |
|---|---|---|---|
| W1 | **softbuffer can't do per‑pixel alpha on a Windows layered window** (BitBlt/GDI vs `UpdateLayeredWindow` are mutually exclusive) | HIGH | On Windows, use winit only to *own* the `WS_EX_LAYERED` HWND; tiny‑skia → premultiplied BGRA; present via `UpdateLayeredWindow(Indirect)` directly. softbuffer presents on X11/Wayland only. `present()` is genuinely per‑OS — fine, it's a trait. |
| W2 | **Click‑through vs clickable** conflict | HIGH | Per‑pixel‑alpha natural hit‑test (don't set `WS_EX_TRANSPARENT`); fallback per‑frame ex‑style toggle. X11 = XShape input region = goose bbox. (§6) |
| G1 | **AV/SmartScreen** flags an unsigned app that warps cursor + synthesizes keys + moves windows (RAT‑like profile) | HIGH | Personal use: document the SmartScreen "More info → Run anyway" path; prefer `RegisterHotKey`/raw‑input over `WH_KEYBOARD_LL` global hook; ship source. Optional Authenticode signing dramatically reduces friction (future). Accept some corporate EDR will quarantine — inherent to the category. |
| M_perm | macOS Accessibility/Input‑Monitoring gates; a `~/.cargo/bin` bare binary **can't hold a stable grant** | HIGH | `.app` bundle with stable bundle‑id is **mandatory** for mischief. Detect via `AXIsProcessTrusted()`, deep‑link to Settings, degrade (overlay + wander + honk work without permission). |
| M_gate | Gatekeeper/notarization for an unsigned `.app` doing injection | HIGH | Personal use: ship unsigned `.app`, document `xattr -dr com.apple.quarantine`. Notarization ($99/yr) = future. |
| E1 | **120 Hz full‑screen layered redraw = CPU/battery killer** | HIGH→mitigated | Per‑monitor windows + present **only the goose dirty rect**; sim 120 Hz, present on‑dirty rate‑capped ~60. Idle ≈ near‑zero. |
| W_dpi | Per‑monitor DPI + multi‑monitor signed/negative coords | MED‑HIGH | Per‑monitor windows (single‑DPI each); declare Per‑Monitor‑V2 DPI awareness; sim in signed virtual space; tiny‑skia scale transform; handle `WM_DPICHANGED`/`ScaleFactorChanged`. |
| L_xwl | Under XWayland, window‑move silently no‑ops on **native‑Wayland** windows (works on X11/XWayland windows) | MED | `enumerate()` returns only X11/XWayland windows; tasks targeting non‑enumerable windows self‑skip; document the limitation. |
| Q_global | Hold‑ESC needs a global key listener (overlay never focused); AV‑suspicious / permission‑gated / impossible on Wayland | MED | `RegisterHotKey` (Win), `XGrabKey` (X11), `CGEventTap`+Input‑Mon (mac, with fallback), tray/menu quit everywhere (esp. Wayland). |
| C_dist | cargo‑dist assumes a CLI on PATH/`CARGO_HOME`; a GUI pet wants autostart + shortcut | MED | Keep cargo‑dist for binary payload + self‑update plumbing; `honk300 install/setup` owns shortcuts/autostart/.desktop/LaunchAgent. PATH alias is optional. |
| A_assets | embed/extract/override precedence + update skew | MED | precedence user > extracted > embedded; atomic extract; hash‑manifest re‑extract that never clobbers user edits; missing slot = skip. |
| A_ip | bundling original sounds 1:1; memes‑as‑codex.md; original notes | LOW (personal, user‑chosen) | keep embedded sounds out of the `include=[…]` source‑package list; no crates.io; memes regenerated originally; notes newly authored. |
| E_rng | original `Deck` shuffle is biased (`System.Random`, low‑bound 0/exclusive high) | LOW | **Decision:** port the biased shuffle faithfully for behavior parity, with a `// faithful-to-original (biased)` note; M0 tests pin the chosen behavior. |
| W_warp | enigo/device_query don't work on native Wayland | LOW (known) | `Cap`‑based degradation; `--wayland` reports `Unsupported`; documented. |

**Genuinely impossible (documented, not fought):** native‑Wayland foreign‑window move; native‑Wayland cursor‑warp/keystroke/global‑key; softbuffer per‑pixel alpha on a Windows layered window; durable macOS Accessibility for an un‑bundled binary. (See §4.)

---

## 12. Corrections to naive assumptions (baked into this plan)
- **C1 — Per‑monitor overlay windows, not one virtual‑screen window.** (buffer size, DPI, negative coords, present cost). Sim stays one virtual space.
- **C2 — Windows present ≠ softbuffer.** Use `UpdateLayeredWindow` directly; softbuffer is X11/Wayland‑only.
- **C3 — Edition.** TR300/ND300 are `edition 2021`; WB300 is `2024`. honk300 will **match the family at 2021** unless we deliberately choose 2024 (1.95 supports it). The **workspace** (multi‑crate) is an intentional divergence from the single‑crate `*300` repos — called out, not implied parity.
- **C4 — Hold‑ESC can't be the only quit path** (global hook AV/permission/Wayland limits). Always add a tray/menu/context‑gesture quit.
- **C5 — `honk300 install` ≠ "put on PATH + shell‑autorun alias."** For a GUI app it means autostart + shortcut/.desktop/LaunchAgent + `InstallSource` marker. Reuse the family's atomic/marker machinery, point it at autostart entries.
- **C6 — Reconsider `install-path`.** `CARGO_HOME` is the un‑bundled‑binary trap on macOS; prefer dedicated install dirs driven by our installers; keep `install-updater=false` + our own SHA‑256 `honk300 update`.
- **C7 — Cursor pos on Windows** via native `GetCursorPos` (dependency‑light) rather than device_query; keep device_query for X11.
- **C8 — RNG fidelity is an explicit decision** (faithful‑biased vs corrected) pinned by M0 tests.

---

## 13. Verification & testing strategy (for the implementation round)
- **Engine unit tests (`honk-engine`, `#![forbid(unsafe_code)]`):** assert Rig vertex positions, locomotion speed/accel, footmark lifetimes, and the `Deck` sequence against hand‑computed values from the C# constants. CPU‑only, no OS.
- **Per‑platform manual matrix** (logged in `TESTING.md`, family convention): overlay transparency + click‑through + clickable goose; wander; footmarks; honk/mute; cursor‑grab; window‑drag; notepad‑type; meme‑drop; hold‑ESC + tray quit; multi‑monitor + mixed‑DPI; autostart on/off.
- **Degradation tests:** macOS without Accessibility; Wayland session (`--wayland` and default XWayland); X11 with a native‑Wayland window present (window‑drag self‑skip).
- **Packaging smoke tests:** install via each of the 4 Windows installers → verify install dir, Start‑Menu shortcut, `InstallSource` marker, `honk300 update` re‑runs the matching installer; macOS `.dmg`→`.app` launch + permission prompt; Linux `.desktop` + autostart.
- **Local gate (family standard):** `cargo fmt --check`, `cargo clippy --all-targets --workspace -D warnings`, `cargo test --workspace`, `cargo build --release`, run the binary.
- **CI:** adapt the family's `ci.yml` + cargo‑dist `release.yml` + `windows-installers.yml`; drop `crates-publish.yml`.

---

## 14. Out of scope / future
Code signing (Authenticode/Apple notarization); App Store/store distribution; a full third‑party mod ABI (we ship in‑tree Rust mods + injection hooks first); Music/streaming features beyond the original; ARM Windows; exotic tiling‑WM polish. Native Wayland mischief stays intentionally limited.

---

## 15. Appendix
- **Engine port source‑of‑truth:** `DESKTOP-GOOSE/DesktopGoose v0.31/FOR MOD-MAKERS/GooseMod_DefaultSolution/GooseModdingAPI/{SamEngine.cs, Exports.cs}`; `…/DefaultMod/{ModMain.cs, TaskDemo_FollowLowAccel.cs}`.
- **Family conventions to mirror:** `qube-workbranch-view/{Cargo.toml, .github/workflows/windows-installers.yml, wix/, wix-corporate/, inno/}`; `qube-machine-report/src/{install/*, update.rs}`, `build.rs`.
- **Constants tables:** §2.3 (physics), §2.4 (rig), §8.4 (config keys).
- **Sources (web research):** Desktop Goose itch.io (Samperson) + Autumn devlog; Virtual Pets Wiki; PC Gamer coverage; winit click‑through issue #1434; Microsoft `windows` crate docs (`UpdateLayeredWindow`/`SetWindowPos`/`EnumWindows`); `objc2-app-kit`/`x11rb`/`smithay-client-toolkit`/`enigo`/`device_query`/`rodio` docs; Wayland layer‑shell limitations.

---

### Document control
- **This round:** produce this plan only. **No goose code written.**
- **Next round (separate):** execute from §10 milestones, starting at M0.
