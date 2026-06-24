# Codex Plan — Desktop Goose Reimplementation

> **Status:** Research + implementation plan (adversarially reviewed). Not yet implemented.
> **Scope:** A clean-room, cross-platform reimplementation of Desktop Goose, Rust-first, with native
> installers and a self-update path modeled on the QubeTX line (TR-300 / ND-300 / WB-300).
> **Source of truth for requirements:** `README.md` in this repository, plus behavior extracted from
> the shipped Desktop Goose v0.31 (Windows) and v0.22 (macOS) artifacts bundled here for research.
> **README compliance constraint:** Cargo is allowed as the Rust build system and `cargo-dist` may be
> used to build release artifacts, but **end users must not be distributed to through Cargo/crates.io**.
> The supported distribution surface is builds, native installers, and installer scripts only.

---

## 0. Mission, Non-Goals, and Guiding Principles

### 0.1 Mission
Rebuild Desktop Goose as a modern, maintainable, cross-platform desktop companion that:

- Reproduces the *feel* and *feature set* of the original Desktop Goose (wandering goose, mud
  tracks, mouse stealing/biting, honking, patting, meme/notepad/donation window collection,
  customizable memes and notepad messages, config-driven behavior).
- Runs natively on **Windows, macOS, and Linux** (X11 first, Wayland best-effort).
- Ships via **native installers / standalone builds / installer scripts only**. Cargo is for
  development and CI; it is not an end-user distribution channel for this project.
- Is **safe by default**: no telemetry, no network on its own, no arbitrary native-code mod loading
  without explicit user opt-in, clean uninstall, and explicit user control over the goose.

### 0.2 Non-Goals (explicitly out of scope for v1)
- Bit-for-bit binary compatibility with the original `GooseDesktop.exe` / `GooseModdingAPI.dll`.
- Loading the original .NET/Mono mods (Autumn.dll etc.) as native plugins. A new, safer mod surface
  is specified in §9; original mods are reference-only.
- Mobile platforms.
- A GUI settings app (config file + CLI flags for v1; a tray menu is a stretch goal).
- Networked/multi-goose or "goose as a service."

### 0.3 Guiding Principles
1. **Simulation core is platform-free.** The goose AI, rig, and task scheduler must be a pure library
   crate with no windowing/rendering/audio dependencies, so it is unit-testable headless.
2. **Platform code is an adapter.** Window creation, input hooks, rendering, and audio live behind
   traits with one implementation per OS. Never let a Linux compositor quirk leak into the goose AI.
3. **Honest cross-platform parity.** Windows and macOS can reach full parity. Linux will be
   X11-first with documented Wayland fallbacks. Never silently degrade; surface a "limited mode" badge.
4. **Distribution mirrors the QubeTX line, with one explicit divergence.** Reuse the proven
   cargo-dist artifact builder, four-Windows-installer matrix, SHA256 sidecars,
   install-origin markers, updater checks, and uninstall discipline from TR-300 / ND-300 / WB-300;
   **do not** copy their crates.io / `cargo install` end-user path.
5. **User sovereignty.** The goose is a guest. ESC evicts it, a single command/flag kills it, and
   uninstall is total and clean. No behavior the user didn't opt into.
6. **Asset licensing is a first-class decision.** The original assets are included here for research;
   shipping them (or shipping without them) is an explicit, recorded decision in §8.4.

---

## 1. Repository Evidence Inventory

### 1.1 This repository (`goose/`)
- `README.md` — the requirements document (examined; drives this plan). Its single substantive
  requirement is: thoroughly analyze the bundled Desktop Goose files, create an entirely new Desktop
  Goose with the same functionality/features, preferably in Rust, support macOS/Linux/Windows, and
  ship native installers like TR-300 / ND-300 / WB-300 **except without Cargo distribution**.
- `DESKTOP-GOOSE/DesktopGoose v0.31/` — Windows reference build.
  - `GooseDesktop.exe` (226,304 bytes, PE32, .NET WinForms, PDB path
    `C:\Users\Start Minecraft\documents\visual studio 2015\Projects\GooseDesktop\...`).
  - `GooseModdingAPI.dll` (16,384 bytes), `MMQ.dll` (10,752 bytes).
  - `Assets/Images/Memes/` (Meme1–7 plus `GooseDance.gif`), `Assets/Images/MemeAttributions.txt`,
    `Assets/Images/OtherGfx/DonatePage.png` and `heart.png`, `Assets/Sound/NotEmbedded/`
    (Honk1–4.mp3, BITE.mp3, MudSquith.mp3), `Assets/Sound/Music/Rename me to just Music.mp3`,
    `Assets/Text/NotepadMessages/`, `Assets/Mods/Autumn/Autumn.dll`.
  - `config.ini` (created on first run if missing/corrupt/wrong-version), `Read me! Honk.txt`,
    `Autumn.txt`, and `FOR MOD-MAKERS/` (Visual Studio default mod solution + modding API project).
- `DESKTOP-GOOSE/Desktop Goose for Mac v0.22/Desktop Goose.app/` — macOS reference build.
  - `Contents/Info.plist`: `CFBundleIdentifier=net.namedfork.DesktopGoose`,
    `CFBundleShortVersionString=0.22`, `LSMinimumSystemVersion=10.9`, `LSUIElement=True`
    (no Dock icon), `MonoBundleExecutable=Desktop Goose.exe`, `NSAppleScriptEnabled=True`,
    `OSAScriptingDefinition=DesktopGoose.sdef`.
  - `Contents/MonoBundle/Desktop Goose.exe` (Mono, shares the .NET codebase with Windows).
  - `Contents/Resources/Notes/Note1–6.txt` (same notepad messages as Windows).
  - `Contents/Resources/Memes/` mirrors the Windows meme set; `Resources/Pat1.wav`–`Pat3.wav`,
    `Honk1.mp3`–`Honk4.mp3`, `BITE.mp3`, `MudSquith.mp3`, `DonatePage.png`, `MacAbout.png`,
    `PreferencesWindow.nib`, `DesktopGoose.sdef`, and `runtime-options.plist` are bundled.
  - `Contents/Resources/Media.xcassets/AppIcon.appiconset`.

### 1.2 Reference: QubeTX qube-workbranch-view (WB-300)
Path from README: C:/Users/hey/git/qube-workbranch-view.
A mature Rust + cargo-dist + four-Windows-installer project. Key reusable patterns:
- **Stack:** Rust edition 2024, MSRV pinned to 1.95 in both Cargo.toml and rust-toolchain.toml.
- **Architecture invariant:** reducer-driven app architecture, no direct UI state mutation, and unconditional cleanup/restore guards where global process state is modified.
- **Distribution:** cargo-dist artifact builds plus four Windows installers (Global/Corporate x MSI/EXE) with permanent product GUIDs, per-edition install paths, PATH management, and HKCU install-source markers.
- **Self-update:** GitHub releases API, semver compare, install-origin-matched strategy, SHA-256 sidecar verification, installer execution, and post-install --version verification.
- **Lockstep contract:** installer paths/marker values must change together with update/uninstall detection in the same commit.

### 1.3 Reference: TR-300 / ND-300
Paths from README: C:/Users/hey/git/qube-machine-report (TR-300) and C:/Users/hey/git/qube-network-diagnostics (ND-300).

Verified reference patterns:
- TR-300 README: four Windows installer options (Global/Corporate x MSI/EXE), macOS/Linux via cargo-dist shell installer, SmartScreen caveat, matching installer self-update, SHA256 sidecars, and a crates.io path that must not be copied for Goose.
- ND-300 README/CLAUDE: same tag-triggered release plus windows-installers workflow chain, four installer matrix, HKCU/Software/ND300/InstallSource marker, SHA256 verification, post-install version verify, and hidden migrate-cleanup consolidation invoked by installers and silent self-update.
- WB-300 docs: same QubeTX lockstep discipline, but no installer consolidation yet.

Goose should adopt the installer/update/release hardening from all three, especially ND-300 silent consolidation and torn-release pre-flight, while obeying README no-Cargo end-user-distribution requirement.

---

## 2. Desktop Goose Behavior Inventory (extracted from binaries + assets)

### 2.1 The goose rig (procedural animation)
Extracted symbol/string names from `GooseDesktop.exe` and the macOS `Desktop Goose.exe`:

- **Body geometry:** `NeccRadius`, `NeccHeight1`, `NeccHeight2`, `NeccExtendForward1`,
  `NeccExtendForward2`, `HeadRadius1`, `HeadRadius2`, `HeadLength1`, `HeadLength2`, `EyeRadius`,
  `EyeElevation`, `EyesForward`, `UnderBodyRadius`, `UnderBodyLength`, `UnderBodyElevation`,
  `xRadius`, `xyRadius`.
- **Feet:** `lFootPos`, `rFootPos`, `lFootMoveDir`, `lFootMoveOrigin`, `lFootMoveTimeStart`,
  `rFootMoveDir`, `rFootMoveOrigin`, `rFootMoveTimeStart`, `rightFoot`, `feetDistanceApart`,
  `GetFootHome`.
- **Motion:** `currentSpeed`, `currentAcceleration`, `turnSpeed`, `stepTime`, `wantStepAtDistance`,
  `SpeedTiers`, `SetSpeed`/`setSpeed`, `gooseSpeedPercentage`, `goosePos`, `neckHeadPoint`.
- **Speed/accel constants:** `WalkSpeed`, `RunSpeed`, `ChargeSpeed`, `AccelerationNormal`,
  `AccelerationCharged`, `StepTimeNormal`, `StepTimeCharged`, `ShrinkTime`.
- **Targeting:** `GetDistanceToTarget`/`getDistanceToTarget`, `IsGooseAtTarget`, `StopRadius`,
  `GoodEnoughDistance`, `GiveUpTime`, `TickTime`, `DequeueTimeoutNanoSeconds`.

**Implication:** the original renders the goose with vector/shape primitives (neck as a tapered
segment, head as ellipses, feet as procedural stepping). The reimplementation should reproduce this
with a 2D vector renderer (not sprite sheets) so it scales to any DPI and stays tiny.

### 2.2 Task system
Symbols show a registered task database with IDs, weights, and a random picker:

- `Task_Wander` — "Just the goose's wandering around, default state." Fields: `FirstWanderTimeSeconds`,
  `MinWanderingTimeSeconds`, `MaxWanderingTimeSeconds`, `GetRandomWanderDuration`,
  `wanderingDuration`, `wanderingStartTime`, `nextDirChangeTime`, `pauseDuration`, `pauseStartTime`,
  `MaxPauseTime`, `MinPauseTime`, `GetRandomPauseDuration`, `GetRandomWalkTime`.
- `Task_TrackMud` — "The goose runs off the screen, and runs back on leaving MUDDY FOOTPRINTS!"
  Fields: `DurationToTrackMud`, `trackMudEndTime`, `isTrackingMud`, `AddFootMark`, `PlayMudSquith`,
  `MudColorDefault`/`MudColorKey`.
- `Task_NabMouse` — "Make the goose try and steal your mouse!" Fields: `MouseGrabDistance`,
  `MouseDropDistance`, `MouseSuccTime`, `SeekingMouse`, `DraggingMouseAway`,
  `originalVectorToMouse`, `grabbedOriginalTime`, `WaitingToBringWindowBack`.
- `Task_CanAttackMouse` / `AttackRandomly` — the goose can lunge/bite the cursor; `BITE.mp3` plays.
- `Task_CollectWindow` — internal dispatcher with sub-types:
  - `CollectWindow_Notepad` — "Goose Not-epad" (runs `\notepad.exe` with a random message file).
  - `CollectWindow_Meme` — drags an image meme onto the screen from `Assets/Images/Memes/`.
  - `CollectWindow_Donate` — drags the donation page image (`DonatePage.png`).
  - Fields: `CollectWindowTaskData`, `DraggingWindowBack`, `ExitWindow`, `OriginalWindowStyle`,
    `PassthruWindowStyle`, `SetWindowPassthru`, `SetWindowResizableThreadsafe`,
    `SetWindowPositionThreadsafe`, `windowOffsetToBeak`.
- `FirstUX_FirstTask`, `FirstUX_SecondTask` — a scripted first-run sequence (the goose introduces
  itself before going random).
- `DecideToRun`, `DurationToRunAmok`, `MinRunTime`, `MaxRunTime`, `timeToStopRunning`,
  `RunningOffscreen` — the goose occasionally bolts off-screen (used by collect-window and mud tasks).
- `ChooseNextTask`, `ChooseRandomTask`, `GetNextRandomTask`, `GetRandomTaskID`,
  `gooseTaskWeightedList`, `taskPickerDeck` — weighted random task selection.

### 2.3 Sounds
- `Honk1.mp3`–`Honk4.mp3` — random honk variants.
- `BITE.mp3` — plays on mouse attack.
- `MudSquith.mp3` — plays during mud tracking.
- `Pat1`/`Pat2`/`Pat3` ("pat on back sound") — plays when the user pats the goose (clicks on it).
- `honkPlayer`, `honkSources`, `biteSource`, `patSoundPool`, `patSources`, `NextHonkSound`,
  `NextPatSound`.

### 2.4 Config (`config.ini` on Windows, `config.goos` on macOS)
Strings reveal a versioned config with a `GOOSE_CONFIG_VERSION` guard (corrupt or wrong-version
config is recreated with defaults). Extracted/defaultable fields:

| Field | Meaning | Notes |
|---|---|---|
| `Version_DoNotEdit=1` | Config schema version | Actual Windows v0.31 file value. |
| `EnableMods=False` | Allow mod loading | Prompts a security warning dialog on first enable. |
| `SilenceSounds=False` | Disable audio | Actual v0.31 toggle from config file. |
| `Task_CanAttackMouse=True` | Enable mouse attack/steal task | Actual v0.31 toggle from config file. |
| `AttackRandomly=False` | Whether attacks can occur randomly | Actual v0.31 toggle from config file. |
| `UseCustomColors=False` | Whether goose colors are overridden | Actual v0.31 toggle from config file. |
| `GooseDefaultWhite=#ffffff` / `GooseDefaultOrange=#ffa500` / `GooseDefaultOutline=#d3d3d3` | Goose palette | Actual v0.31 defaults. |
| `FirstWanderTimeSeconds=20` | Delay before the goose's first wander | Actual v0.31 default. |
| `MinWanderingTimeSeconds=20` / `MaxWanderingTimeSeconds=40` | Wander duration range | Actual v0.31 defaults. |
| `RunSpeed` / `WalkSpeed` / `ChargeSpeed` | Speed tiers | Charge = attack/steal lunge. |
| `AccelerationNormal` / `AccelerationCharged` | Acceleration tiers | |
| `MouseGrabDistance` / `MouseDropDistance` / `MouseSuccTime` | Mouse-steal tuning | |
| `DurationToTrackMud` | Mud-track task length | |
| `DurationToRunAmok` / `MinRunTime` / `MaxRunTime` | Off-screen bolt tuning | |
| `CanAttackAtRandom` (macOS `CanAttackAtRandomKey`) | Whether the goose bites the cursor | |
| `MudColorDefault` (macOS `MudColorKey`) | Mud footprint color | |

### 2.5 Assets and user customization
- `Assets/Images/Memes/` — meme images the goose drags on screen. Users drop their own here.
  Bundled files include `Meme1.png`–`Meme7.png` plus `GooseDance.gif`. `MemeAttributions.txt`
  credits five memes to Reddit / the Untitled Goose Game community, reinforcing the licensing risk.
- `Assets/Images/OtherGfx/DonatePage.png` and `heart.png` — donation/affection UI assets.
- `Assets/Sound/NotEmbedded/` — honks, bite, mud. "NotEmbedded" means they're loadable files, not
  compiled resources, so users can swap them. Windows also includes a music placeholder file named
  `Assets/Sound/Music/Rename me to just Music.mp3`, implying a user-customizable background music hook.
- `Assets/Text/NotepadMessages/*.txt` — random notepad messages. Shipped set:
  - `am goose hjonk`
  - `good work`
  - ASCII goose (`>o)` / `(_>`)
  - `nsfdafdsaafsdjl / asdas sorry / hard to type withh feet`
  - `i cause problems on purpose`
  - `"peace was never an option" -the goose (me)`
- `DonatePage.png` — the donation window image (Patreon + PayPal links in-binary).

### 2.6 Modding surface (reference only — not loaded in v1)
- `GooseModdingAPI.dll` exposes `IMod` with `Init(GooseEntity, GooseRenderData, GooseTaskInfo, …)`,
  `ModEntryPoint`, `ModHelperFunctions`, `GooseFunctionPointers` (function-pointer table for
  `setSpeed`, `setTaskRoaming`, `getDistanceToTarget`, etc.).
- Mods live in `Assets/Mods/<Name>/<Name>.dll` and are discovered by `LoadMods` (which scans
  `Assets/Mods/` and warns if missing).
- `Autumn.dll` (sample mod) adds `Autumn_ChaseLeafPile` / `ChaseLeafPileTaskData` with
  `MaxLeafPiles`, `LEAF_RENDERRAD_W`, `leafBrushes`, `renderAboveGoose` — i.e. a mod can register a
  new task, add renderables, and hook the update rig (`RaisePostModLoad`, `RaisePreUpdateRig`,
  `RaisePostUpdateRig`).
- `FOR MOD-MAKERS/What is this.txt` confirms the bundled Visual Studio solution has two projects:
  the API project (reference/link target, do not edit) and a basic default mod that demonstrates the
  pattern. Installation is "drop the resulting DLL (not the API) into
  `Assets/Mods/YourModFolderName/YourMod.dll` and enable mods in config."
- First enable shows: *"Mods are not created by the maker of Desktop Goose, and *can* contain
  malicious code… Do you still wish to enable mods?"*

### 2.7 Platform-specific notes from the macOS build
- `LSUIElement=True` → runs with no Dock icon (background agent).
- `NSAppleScriptEnabled=True` + `DesktopGoose.sdef` → AppleScript scripting dictionary (commands:
  `HonkCommand`, `WanderCommand`, `CollectMemeCommand`, `CollectNoteCommand`,
  `CollectDonationsCommand`, `NabMouseCommand`, `TrackMudCommand`, `OpenMemesFolderCommand`).
- Uses `NSWindow` with `set_IgnoresMouseEvents`, `NSWindowLevel`, `NSWindowCollectionBehavior`,
  `set_CollectionBehavior` — i.e. a borderless, click-through, above-desktop window.
- `get_CurrentMouseLocation`, `get_CurrentPressedMouseButtons`, `IsLeftMouseDown` for input.
- `MemesDirectory` / `memesDirectory` / `OpenMemesFolder` — the memes folder is openable by the user.
- `Library/Application Support/Desktop Goose` — macOS state directory.

### 2.8 User controls
- **ESC** — "Continue holding ESC to evict goose" (hold-to-quit, not single-tap, to avoid accidents).
- **Click on goose** — pat (`PlayPat`).
- **Click elsewhere** — the goose can attack/steal the cursor depending on config.

---

## 3. Target Architecture

### 3.1 Workspace layout
```
goose/
├── Cargo.toml              # workspace
├── rust-toolchain.toml     # pinned (MSRV in lockstep with Cargo.toml rust-version)
├── crates/
│   ├── goose-core/         # pure simulation: rig, tasks, config, math. No I/O, no windowing.
│   ├── goose-platform/     # traits: Window, Input, Renderer, Audio, Time, AssetLoader
│   ├── goose-windows/      # Win32/WinForms-equivalent overlay impl
│   ├── goose-macos/        # NSWindow-equivalent overlay impl
│   ├── goose-linux/        # X11 (primary) + Wayland (fallback) overlay impl
│   ├── goose-app/          # wires core + platform, main loop, tray, CLI
│   └── goose-cli/          # `goose` binary: run, update, uninstall, help
├── assets/                 # shipped assets (license-cleared subset, see §8.4)
├── inno/                   # global.iss, corporate.iss
├── wix/                    # main.wxs (Global MSI)
├── wix-corporate/          # corporate.wxs (Corporate MSI)
├── .github/workflows/      # release.yml, windows-installers.yml, ci.yml (no crates publish workflow for end users)
├── deploy.sh               # two-phase release wrapper (mirrors WB-300)
├── CHANGELOG.md / HUMAN_CHANGELOG.md
└── codex_plan.md           # this file
```

### 3.2 The one architecture rule (mirrors WB-300)
```
Timer tick → core::step(dt, input) → core::Frame → platform::render(frame) + platform::play_audio
```
- The **core** never touches the OS. It takes `Input` (mouse pos, clicks, time) and emits a `Frame`
  (goose pose, active footprints, windows to spawn/drag, sounds to play, task state).
- The **platform** layer owns the overlay window, renders the frame, plays audio, and feeds input.
- State changes happen only in `core::step`. Platform code is a dumb renderer/input pump.
- This makes the entire simulation unit-testable headless (golden-frame tests, see §7).

### 3.3 Core crate modules
- `rig.rs` — the goose skeleton: neck/head/eyes/body/feet with the geometry constants from §2.1,
  procedural foot stepping (`lFootMove*`), and `RenderData` output.
- `tasks/` — one module per task (`wander`, `track_mud`, `nab_mouse`, `can_attack_mouse`,
  `collect_window`), plus `registry.rs` (weighted random picker, `FirstUX` scripted sequence) and
  `scheduler.rs` (task lifecycle, `DecideToRun`, off-screen bolt).
- `config.rs` — versioned config (`GOOSE_CONFIG_VERSION`), TOML on all platforms (drop the
  `.ini`/`.goos` split), missing→defaults, corrupt→defaults+warn, wrong-version→defaults.
- `windows.rs` — the notepad/meme/donation window model (spawn, drag-to-beak, release, exit).
- `audio.rs` — sound *intent* events (Honk, Bite, MudSquith, Pat); the platform layer maps these to
  actual playback.
- `math.rs` — vectors, easing, distance, speed tiers.
- `assets.rs` — asset manifest (memes folder, sounds, notepad messages) as pure data; platform loads.

### 3.4 Platform traits (`goose-platform`)
```rust
pub trait Host {
    fn now(&self) -> Instant;
    fn mouse_location(&self) -> Option<(i32, i32)>;
    fn mouse_buttons(&self) -> MouseButtons;
    fn spawn_window(&self, kind: WindowKind, content: WindowContent) -> WindowHandle;
    fn move_window(&self, handle: WindowHandle, pos: (i32, i32));
    fn close_window(&self, handle: WindowHandle);
    fn play_sound(&self, sound: Sound);
    fn open_user_folder(&self, folder: UserFolder); // memes / sounds / notepad messages
    fn screen_bounds(&self) -> Rect;
    fn quit(&self);
}

pub trait Renderer {
    fn render(&mut self, frame: &Frame);
}
```
Each OS crate provides `Host` + `Renderer`. The app crate's main loop calls `core::step` then
`renderer.render` + drains sound/window intents to the `Host`.

### 3.5 Platform implementations

#### Windows (`goose-windows`)
- **Overlay window:** a layered, transparent, topmost, click-through `WS_EX_LAYERED |
  WS_EX_TRANSPARENT | WS_EX_TOPMOST` window (the original uses `SetLayeredWindowAttributes` and
  `SetWindowLong` with `PassthruWindowStyle`). Render with Direct2D or `wgpu` (2D) for crisp
  DPI-scaled vector drawing.
- **Mouse stealing:** the original literally moves the cursor (`SetCursorPos` equivalent) and
  simulates a drag. This is the riskiest feature on modern Windows (foreground-lock timeout, input
  injection permissions). Implement via `SendInput`; gate behind config and a clear first-run
  consent. Provide a "no mouse stealing" safe mode.
- **Notepad window:** spawn a real `notepad.exe` with a temp .txt (as the original does:
  `\notepad.exe`), or — safer — a small owned borderless window showing the message. Decide in Phase 4.
- **Audio:** `windows-media` / `rodio` (see §4.3).
- **State dir:** `%LOCALAPPDATA%\goose` (config, logs, memes index).

#### macOS (`goose-macos`)
- **Overlay window:** borderless `NSWindow` at `NSWindowLevel` above the desktop,
  `setIgnoresMouseEvents:YES` for click-through, `setCollectionBehavior` to span spaces. Use
  `core-graphics` / `metal` for rendering, or `wgpu`.
- **Input:** `CGEvent` tap for global mouse location/buttons (requires **Accessibility** permission
  on macOS 10.15+). Must prompt the user to grant it in System Settings and degrade gracefully
  (no mouse stealing) if denied.
- **No Dock icon:** `LSUIElement=true` in `Info.plist` (matches the original).
- **AppleScript:** optional v2 stretch goal — ship an `.sdef` exposing the original command set:
  `collect meme` (optional URL/path + title), `collect note` (optional text + title), `nab mouse`,
  `wander` (optional duration), `track mud`, `collect donations`, `honk`, `open memes folder`, and
  `open notes folder`. Not required for v1.
- **State dir:** `~/Library/Application Support/goose`.

#### Linux (`goose-linux`)
- **X11 (primary):** a borderless, override-redirect, topmost, input-region-empty (click-through)
  window via `x11rb`/`xcb`. Render with `wgpu`/`tiny-skia` (CPU 2D, no GPU dependency) — tiny-skia is
  preferred for a vector goose on a compositor that may or may not be running.
- **Wayland (fallback):** the protocol deliberately does not allow global input hooks or arbitrary
  topmost windows. On Wayland, the goose can render in a borderless window but **cannot** steal the
  mouse or draw over other windows reliably. Detect Wayland (`XDG_SESSION_TYPE`) and run in
  "limited mode" (wander + honk + in-window memes only), with a clear badge. This is an honest
  limitation, not a bug.
- **Input:** X11 `XQueryPointer` + `XRecord` for global mouse; on Wayland, no global input (limited
  mode).
- **State dir:** `$XDG_CONFIG_HOME/goose` or `~/.config/goose`.

### 3.6 CLI (`goose-cli`)
Mirrors WB-300's `cli.rs` shape:
- `goose` — launch the goose (default).
- `goose help` — full manual (`help.rs`, paged on TTY).
- `goose update` — self-update (registry-aware on Windows, shell-installer elsewhere).
- `goose uninstall` — channel-aware removal (`--yes`, `--purge`, `--json`).
- `goose --version`.
- Flags: `--no-mouse-steal`, `--no-sound`, `--config <path>`, `--no-mods`.

---

## 4. Technology Choices (with alternatives + red-team notes)

### 4.1 Language: Rust
- **Why:** native binaries, fast startup, robust OS FFI, no runtime/dependency hell, matches the
  QubeTX line. The original is .NET/Mono; a Rust rewrite is a clean break, not a port.
- **Edition 2024, MSRV pinned** (start at 1.95 to match WB-300, or the latest stable at bootstrap).
- **Rejected:** C#/.NET (would reproduce the original's Mono-on-Linux pain and runtime dependency);
  C++ (slower to make safe); Go (weaker GUI/FFI story for this).

### 4.2 Rendering
- **Choice: `tiny-skia` (CPU 2D vector) as the default**, with an optional `wgpu` backend.
- **Why:** the goose is vector shapes; tiny-skia has no GPU dependency (critical for Linux
  compositors and locked-down corporate machines), is pure Rust, and produces crisp DPI-scaled
  output. `wgpu` is the fallback for machines where CPU rendering is too slow.
- **Rejected:** sprite-sheet rendering (loses DPI scaling and bloats assets); raw Direct2D/Metal
  (duplicates work per platform); `cairo` (C dependency, painful on Windows).

### 4.3 Audio
- **Choice: `rodio`** (wraps `cpal`), with MP3 decode via `symphonia`.
- **Why:** cross-platform, pure-Rust decode path available, simple "fire and forget" sound events.
- **Rejected:** platform-native audio (duplicates work); `miniaudio` (C dependency).

### 4.4 Windowing / OS integration
- **Windows:** `windows-rs` for Win32 (layered window, `SendInput`, `SetCursorPos`).
- **macOS:** `objc2` + `core-foundation` + `core-graphics` (or `wgpu` for rendering).
- **Linux:** `x11rb` (X11) + `wayland-client` (limited-mode detection only).
- **Rejected:** `winit`/`tao` for the overlay — they target normal application windows, not
  borderless click-through desktop overlays; fighting them is harder than calling the platform APIs
  directly. (They may still be used for a settings window if one is added later.)

### 4.5 Async runtime
- **Choice: `tokio`** for the platform layer (file I/O, self-update HTTP, debounced watchers).
- The **core** is sync and frame-driven (`step(dt)`); no async in the simulation.

### 4.6 HTTP / self-update
- **Choice: `ureq` + `sha2` + `serde_json`** (exactly WB-300's stack) for GitHub releases API,
  SHA-256 sidecar verification, and install-origin detection.

### 4.7 Config
- **Choice: TOML** via `toml` + `serde`. Versioned with `GOOSE_CONFIG_VERSION`. One format on all
  platforms (the original's `.ini`/`.goos` split is an accident of history).

---

## 5. Installer & Release Plan (mirrors QubeTX)

### 5.1 Release model
Tag-triggered: `bump → merge to main → push vX.Y.Z tag`. `release.yml` builds the multi-target
artifacts + GitHub Release; `windows-installers.yml` attaches the Windows add-on installers. `deploy.sh`
scripts the two phases (bump on workbranch, tag on main) with a lockstep changelog guard.

### 5.2 Targets (cargo-dist)
- `x86_64-pc-windows-msvc`
- `aarch64-pc-windows-msvc` (ARM64 Windows — modern; the original never shipped this)
- `x86_64-apple-darwin`
- `aarch64-apple-darwin` (Apple Silicon)
- `x86_64-unknown-linux-gnu` (glibc)
- `x86_64-unknown-linux-musl` (static, for locked-down distros)
- (optional) `aarch64-unknown-linux-gnu`

### 5.3 Windows installer matrix (four first-class installers)
Directly modeled on WB-300's `wix/main.wxs`, `wix-corporate/corporate.wxs`, `inno/global.iss`,
`inno/corporate.iss`:

| Installer | Scope | Admin? | Install path | Marker |
|---|---|---|---|---|
| Global MSI | perMachine | yes (UAC) | `C:\Program Files\goose\bin` | `msi-global` |
| Corporate MSI | perUser | no | `%LocalAppData%\Programs\goose\bin` | `msi-corporate` |
| Global EXE (Inno) | perMachine | yes (UAC) | `C:\Program Files\goose\bin` | `exe-global` |
| Corporate EXE (Inno) | perUser | no | `%LocalAppData%\Programs\goose\bin` | `exe-corporate` |

- Each writes `HKCU\Software\Goose\InstallSource = <marker>`.
- Each adds `bin` to the appropriate PATH (system HKLM for Global, user HKCU for Corporate).
- Each ships a `.sha256` sidecar.
- Product GUIDs (MSI `UpgradeCode`, Inno `AppId`) are **permanent** — never regenerate.
- **Lockstep contract:** install paths + marker values in the four installer files must change
  together with `crates/goose-cli/src/update.rs::detect_install_origin()` and `uninstall.rs` in the
  same commit (copy WB-300's discipline verbatim).

### 5.4 macOS / Linux installers
- **macOS:** a `.app` bundle inside a `.dmg` (or a `.pkg`). Code-signed and notarized (requires an
  Apple Developer ID — a real prerequisite, see §8.5). The bundle sets `LSUIElement=true`.
- **Linux:** a shell installer (`curl … | sh`) for glibc, plus a static-musl binary for
  locked-down distros. Optionally distribute `.deb`/`.rpm` via `cargo-dist` extras. **No Snap/Flatpak
  for v1** (sandboxing breaks the overlay/mouse-stealing model — an honest limitation).

### 5.5 Self-update (`goose update`)
Port WB-300's `src/update.rs` almost verbatim:
1. `fetch_latest_version()` from GitHub releases API (prerelease-aware `is_newer`).
2. `detect_install_origin()` — read `HKCU\Software\Goose\InstallSource`; fall back to path
   classification (`\Program Files\goose\` → Global, `\AppData\Local\Programs\goose\` → Corporate).
3. Download the matching installer to `%TEMP%`.
4. **`verify_checksum`** — fetch `<url>.sha256`, parse, compare with `compute_sha256`. Refuse on
   mismatch (defense against corporate TLS-interception proxies / MITM).
5. Run the installer silently (`msiexec /i /passive /norestart` or Inno `/SILENT
   /SUPPRESSMSGBOXES /NORESTART`).
6. Handle msiexec `3010` (reboot required) without claiming success.
7. `verify_post_install` — re-exec `goose --version` and confirm the on-disk binary actually changed.
8. `--json` mode for scripting/agents.
9. On macOS/Linux: shell-installer only (`curl|sh`, then `wget|sh` fallback) with the same
   post-install version verify. **Do not** add a `cargo install` update strategy for Goose; the README
   explicitly says distribution is builds/installers/scripts, not Cargo.

### 5.6 Uninstall (`goose uninstall`)
- Windows MSI/EXE: find the Add/Remove Programs entry, launch the uninstaller detached (quiet), so it
  can delete the running exe after exit.
- macOS: delete the `.app` bundle (delayed-detached if running from it).
- Linux: delete the binary + shell-installer manifest.
- `--purge`: also remove state/config dirs (`%LOCALAPPDATA%\goose`, `~/Library/Application
  Support/goose`, `~/.config/goose`) and the `HKCU\Software\Goose` registry key.
- Never touches user assets (memes, notepad messages) the user added — move them to a backup folder
  on `--purge` and tell the user where.

### 5.7 CI workflows
- `ci.yml` — fmt, clippy `-D warnings`, test, build on push/PR.
- `release.yml` — tag-triggered; cargo-dist builds the 6+ targets + GitHub Release.
- `windows-installers.yml` — `workflow_run` after `release.yml`; builds the Corporate MSI
  (`candle`/`light`, `-sice:ICE38/64/91`) and the two Inno EXEs, writes `.sha256` sidecars,
  `gh release upload --clobber`s the 6 add-on assets.
- No `crates-publish.yml` for Goose unless a future, separate library crate is intentionally
  published for developers. The end-user product must not depend on crates.io distribution.

---

## 6. Implementation Phases

### Phase 0 — Bootstrap (repo + license + skeleton)
- Create the workspace, `rust-toolchain.toml`, CI stubs, `CHANGELOG.md` + `HUMAN_CHANGELOG.md`,
  `deploy.sh` (copied from WB-300 and adapted).
- **Decide the license** (§8.4) and the asset-clearance strategy.
- Empty `goose-core` with a `step()` that returns a static pose; empty platform traits; a `goose-cli`
  that prints the version.
- **Exit gate:** `cargo test` passes; `cargo clippy -D warnings` clean; CI green.

### Phase 1 — Simulation core (headless)
- Implement `rig.rs` (geometry + procedural feet), `math.rs`, `config.rs` (versioned TOML).
- Implement `tasks/wander.rs` (the default state) with the wander-duration/pause model.
- `Frame`/`RenderData` output; golden-frame tests (assert the pose at t=0, t=1s, on a turn).
- **Exit gate:** golden frames locked; wander feels right in a headless render-to-PNG harness.

### Phase 2 — Desktop overlay MVP (Windows first)
- `goose-windows`: layered transparent topmost window, tiny-skia render loop, `Host` impl.
- Wire `core::step` → render. The goose wanders on top of the desktop, click-through.
- ESC-to-evict (hold for 1s). Tray icon with "Quit" and "Open memes folder".
- **Exit gate:** goose visibly wanders on Windows; clean exit restores the desktop; no input lag.

### Phase 3 — Behavior parity
- Implement `track_mud`, `nab_mouse`, `can_attack_mouse`, `collect_window` (notepad/meme/donation).
- Weighted task picker + `FirstUX` scripted intro.
- Audio (rodio): honks, bite, mud, pat.
- Pat-on-click.
- **Exit gate:** feature checklist from §2.2 all demonstrable; config fields all honored.

### Phase 4 — Assets, customization, config UI
- Asset loading from `<state>/assets/` with shipped defaults; user-drop folders for memes/sounds/
  notepad messages.
- `goose help` manual.
- Config hot-reload (watch the TOML file).
- Tray menu: pause, mute, no-mouse-steal, open folders, quit.
- **Exit gate:** user can drop a meme and see it appear; config changes apply live.

### Phase 5 — macOS + Linux platform parity
- `goose-macos`: NSWindow overlay, CGEvent input (with accessibility permission prompt), `LSUIElement`.
- `goose-linux`: X11 overlay (full), Wayland limited-mode detection + badge.
- Per-OS state dirs; honest "limited mode" on Wayland.
- **Exit gate:** wander + honk + memes on all three OSes; mouse-steal on Windows + macOS (with
  permission); documented Wayland limits.

### Phase 6 — Installers, self-update, release
- Port the four Windows installers (WiX ×2, Inno ×2) from WB-300; adapt paths/GUIDs/markers to `goose`.
- macOS `.dmg` (signed + notarized) and Linux shell installer.
- `goose update` + `goose uninstall` (port from WB-300 `update.rs` / `uninstall.rs`).
- `release.yml` + `windows-installers.yml` + `deploy.sh`.
- First tagged release (`v0.1.0`).
- **Exit gate:** a clean machine can install via Corporate EXE (no admin), `goose update` to a new
  tag, and `goose uninstall --purge` leaves nothing behind.

### Phase 7 — (Optional) safe modding layer
- A **data-driven** mod surface first: mods are TOML/JSON + assets that declare new tasks, sounds,
  memes, or config overrides — no native code.
- A **WASM** mod surface as a stretch goal: mods compile to `wasm32-unknown-unknown` and call a
  versioned host API (`setSpeed`, `setTaskRoaming`, `getDistanceToTarget`, …) via WIT. Sandboxed,
  no filesystem/network access by default.
- **Never** load arbitrary `.dll`/`.so`/`.dylib` mods (the original's model) without an explicit,
  scary warning and per-mod opt-in — and even then, prefer WASM.
- **Exit gate:** a sample "Autumn" mod reimplemented as a data/WASM mod demonstrating a new task.

---

## 7. Testing & Acceptance Criteria

### 7.1 Unit tests (core)
- `rig.rs`: foot-stepping produces monotonic forward motion; neck/head track the target within
  `GoodEnoughDistance`.
- `tasks/`: each task transitions through its documented states; `FirstUX` runs the scripted pair
  before random selection; the weighted picker respects weights (statistical test over 10k draws).
- `config.rs`: missing→defaults, corrupt→defaults+warn, wrong-version→defaults, unknown keys ignored.
- `math.rs`: speed tiers, acceleration, easing.

### 7.2 Golden-frame tests
- Render `Frame` to a PNG via tiny-skia's offscreen path at fixed timestamps; assert against
  committed golden PNGs (with a tolerance for anti-aliasing). Catches rig regressions visually.

### 7.3 Integration tests (platform)
- A `mock_host` implementing the `Host` trait records window/sound intents; assert that
  `collect_window` spawns the right window kind, `nab_mouse` issues cursor moves, etc. — without
  touching the real OS.

### 7.4 Manual QA matrix
| OS | Wander | Honk | Mud | Meme | Notepad | Mouse steal | Pat | Install | Update | Uninstall |
|---|---|---|---|---|---|---|---|---|---|---|
| Windows 11 x64 | | | | | | | | | | |
| Windows 11 ARM64 | | | | | | | | | | |
| macOS (Intel + Apple Silicon) | | | | | | (w/ A11y) | | | | |
| Linux X11 (GNOME/KDE/XFce) | | | | | | | | | | |
| Linux Wayland (limited mode) | wander | honk | — | in-window | in-window | — | — | | | |

### 7.5 Installer/update verification
- `goose update` between two tags on a clean VM for each installer variant.
- SHA-256 mismatch refusal (point the updater at a tampered sidecar; assert it refuses).
- `goose uninstall --purge` leaves no files/registry/PATH entries (automated check script).

---

## 8. Adversarial Red-Team Review

### 8.1 Permission & privacy failure modes
- **macOS Accessibility permission:** mouse stealing and global mouse position require it. If denied,
  the goose must degrade to "no mouse interaction" and say so — never silently fail. **Mitigation:**
  prompt on first run, link to System Settings, show a badge while denied.
- **Windows input injection:** `SendInput` is blocked when the foreground process runs at a higher
  integrity level (UAC dialog, Task Manager). **Mitigation:** detect failure and surface "couldn't
  grab the mouse this time" rather than spinning.
- **Wayland:** no global input/draw-over-everything. **Mitigation:** detect `XDG_SESSION_TYPE=wayland`
  and run limited mode with a badge. Do not attempt XWayland hacks that break under GNOME/KDE.
- **Telemetry:** none. The goose must not phone home. The only network access is `goose update`
  hitting GitHub releases on explicit user action. **Mitigation:** no HTTP client in the core or
  platform layers; only in `update.rs`.

### 8.2 User-hostile behavior guardrails
- The original goose is *intentionally* annoying (steals mouse, drags memes, bites). This is the
  product. But it must be **bounded**:
  - **Hold-ESC-to-quit** (not single-tap) — already in the original; keep it.
  - **Tray icon always present** with an unambiguous "Quit" — a panic exit is never the only way out.
  - **`--no-mouse-steal` flag + config** for users who want the goose without the chaos.
  - **Rate limits** on meme/notepad spam (the original has task durations; keep them).
  - **Never** interfere with full-screen apps / games. Detect foreground fullscreen on Windows
    (`SHQueryUserNotificationState`) and macOS (`NSWorkspace.frontmostApplication`) and pause the
    goose while a fullscreen app is focused. (The original doesn't do this; we should — it's a
    genuine improvement that prevents the goose from disrupting a game/stream.)

### 8.3 Mod security
- The original loads `.dll` mods — arbitrary native code with full process privileges. **Do not
  reproduce this as the default.** §6 Phase 7 specifies data-driven mods first, WASM second.
- If native mods are ever added (v2+), require: (a) `EnableMods=true` in config, (b) a per-mod
  allowlist with a scary consent dialog naming the mod file and its SHA-256, (c) a signature check
  if a signing key is established. Default off.
- WASM mods are sandboxed by construction (no I/O unless explicitly granted via WIT interfaces).

### 8.4 Asset licensing & supply chain
- **The original Desktop Goose assets are included in this repo for research only.** Shipping them in
  a release requires determining their license. Desktop Goose is by Samperson (person behind "Start
  Minecraft" / the Patreon in-binary). **Decision required:** either (a) get written permission to
  redistribute the original assets, (b) ship only newly-created CC0/CC-BY assets (a "goose" that
  looks different), or (c) ship no assets and prompt the user to supply their own. **Default for
  v0.1.0: option (b)** — original-style but newly-drawn assets + synthesized honk SFX — to avoid any
  licensing ambiguity.
- **Dependency supply chain:** pin all Cargo dependencies in `Cargo.lock`; audit with `cargo audit`;
  prefer pure-Rust crates (tiny-skia, rodio, symphonia, ureq, sha2) over C bindings to minimize the
  attack surface.
- **Installer integrity:** SHA-256 sidecars on every release asset; `goose update` refuses on
  mismatch (§5.5). This is the WB-300 pattern, proven.

### 8.5 Release prerequisites (real costs)
- **Apple Developer Program** ($99/yr) for code signing + notarization. Without it, macOS users get
  Gatekeeper warnings. **Mitigation:** ship unsigned-with-instructions for v0.1.0; add signing in
  Phase 6 if budget allows.
- **Windows code signing** (optional but recommended): an EV cert avoids SmartScreen warnings. For
  v0.1.0, ship unsigned with SmartScreen caveats; the Corporate (perUser) installer at least needs
  no admin.
- **GitHub Actions** (free tier sufficient for cargo-dist + windows-installers).
- **No crates.io requirement for the end-user product.** A developer-only library publication can be
  reconsidered later, but the README explicitly excludes Cargo distribution for Goose.

### 8.6 Cross-platform parity gaps (honest limitations)
- **Wayland:** no global mouse steal, no reliable draw-over-all. Limited mode only. Documented.
- **macOS mouse steal:** requires Accessibility permission; degrades without it.
- **Linux audio:** PulseAudio/PipeWire via `cpal`/`rodio` is reliable; ALSA-only systems may need
  the user to install `libasound2`. Document the dependency.
- **Sandboxed package formats (Snap/Flatpak):** not supported for v1; the overlay model is
  incompatible with their sandbox. Document that users should use the shell installer or `.deb`/`.rpm`.

### 8.7 Update robustness
- **Locked binary:** the running `goose.exe` can't be overwritten in place. Windows Installer's
  Restart Manager handles this (rename locked file, schedule delete-on-reboot). Surface msiexec
  `3010` honestly (§5.5). On macOS/Linux, the updater should ask the user to quit the goose first
  (or self-restart via a detached helper that swaps the binary after exit — port WB-300's
  `uninstall.rs` delayed-detached pattern).
- **Cargo-path contamination:** TR-300 / ND-300 / WB-300 support `cargo install`, but Goose should
  not. The updater must not claim a Cargo strategy, must not fall through to crates.io, and should
  test that no update strategy invokes `cargo install`. Post-install `--version` verification still
  applies to every installer/script strategy.
- **Re-tagging:** never re-tag a released version; always bump patch and re-run (in `deploy.sh`).

### 8.8 Uninstall completeness
- The four installers remove their own `InstallSource` markers (WiX component teardown / Inno
  `uninsdeletevalue`). `goose uninstall --purge` removes state + config + the `HKCU\Software\Goose`
  key + the toast AUMID key (if added). **Verify with an automated "leave nothing behind" script**
  that scans PATH, registry, and the filesystem after uninstall.

---

## 9. Final Recommended Path (MVP → Production)

1. **Phase 0–2 first:** get a wandering, click-through goose on Windows with tiny-skia. This is the
   smallest vertical slice that proves the architecture (core/platform split, overlay window, render
   loop, clean exit). Ship as `v0.1.0-alpha` (unsigned, Windows-only, no installer — just a zip).
2. **Phase 3:** add the behaviors (mud, mouse, memes, notepad, honk, pat). Ship `v0.1.0` with the
   four Windows installers + self-update.
3. **Phase 5:** macOS + Linux. Ship `v0.2.0` cross-platform with honest Wayland limits.
4. **Phase 6 hardening:** code signing (macOS first), `deploy.sh`, full CI. Ship `v1.0.0`.
5. **Phase 7 (optional):** safe modding (data-driven, then WASM). Ship `v1.1.0`.

**The single most important risk** is the platform overlay + input-injection surface across three
OSes. The core/platform split (§3.2) is the load-bearing decision that contains this risk: if a
platform's overlay is hard, the core still ships and can be rendered into any window. **Do not let
platform difficulty block the simulation core.**

---

## Appendix A — File/Path Contract (lockstep)

| What | Windows | macOS | Linux |
|---|---|---|---|
| Binary | `goose.exe` | `goose` (in `.app/Contents/MacOS/`) | `goose` |
| Install (Global) | `C:\Program Files\goose\bin` | `/Applications/Goose.app` | `/usr/local/bin/goose` |
| Install (Corporate/perUser) | `%LocalAppData%\Programs\goose\bin` | `~/Applications/Goose.app` | `~/.local/bin/goose` |
| Config | `%LOCALAPPDATA%\goose\config.toml` | `~/Library/Application Support/goose/config.toml` | `$XDG_CONFIG_HOME/goose/config.toml` |
| State/log | `%LOCALAPPDATA%\goose\` | `~/Library/Application Support/goose/` | `$XDG_DATA_HOME/goose/` |
| User memes | `<state>/assets/memes/` | same | same |
| User sounds | `<state>/assets/sounds/` | same | same |
| User notepad msgs | `<state>/assets/notepad/` | same | same |
| Install marker (Win) | `HKCU\Software\Goose\InstallSource` | n/a | n/a |

**Lockstep rule:** any change to install paths or marker values must update the four Windows
installer files, `update.rs::detect_install_origin()`, and `uninstall.rs` in the same commit.

---

## Appendix B — Config Schema (v1, TOML)

```toml
goose_config_version = 1

[behavior]
enable_mods = false
silence_sounds = false
can_attack_mouse = true
attack_randomly = false
use_custom_colors = false
first_wander_time_seconds = 20.0
min_wandering_time_seconds = 20.0
max_wandering_time_seconds = 40.0

[colors]
goose_default_white = "#ffffff"
goose_default_orange = "#ffa500"
goose_default_outline = "#d3d3d3"

[speeds]
walk = 30.0
run = 90.0
charge = 200.0
acceleration_normal = 120.0
acceleration_charged = 400.0

[mouse]
grab_distance = 60.0
drop_distance = 200.0
succ_time = 1.5

[mud]
duration_seconds = 6.0
color = "#5a3a1a"

[run_amok]
duration_seconds = 3.0
min_run_time = 2.0
max_run_time = 5.0

[audio]
enabled = true
honk = true
bite = true
mud = true
pat = true

[safety]
pause_on_fullscreen = true   # new: don't bother games/streams
no_mouse_steal = false       # --no-mouse-steal sets this true
```

Missing → defaults. Malformed → warn + defaults. Unknown keys → ignored. `GOOSE_CONFIG_VERSION`
mismatch → defaults (mirrors the original's "wrong version, recreate" behavior).

---

*End of plan. This document is the research + implementation blueprint; implementation begins at
Phase 0 once this plan is approved.*