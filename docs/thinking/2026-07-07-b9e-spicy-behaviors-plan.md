# Critical Thinking Session - #b9e Default-Off Spicy Behaviors

**Date:** 2026-07-07 local (-05:00)  
**Framework:** Problem-Solving  
**Mode:** Recommend  
**Stacked skills:** critical-thinking, logical-reasoning

---

## Pre-Flight: Inputs Inspected

| Input | Type | Finding | Confidence |
|---|---|---|---|
| `.tasks/TASKS.md` | board | `#b9e` is To-Do, not active: default-off spicy behaviors with terminal protection. | High |
| `.tasks/tasks/b9e.md` | task detail | Requires generated image assets with the clumsy MS-Paint base prompt; no stock/original-app spicy art. | High |
| `honk300_plan.md` section 5.12 | canonical plan | Scope is clipboard honk, fake-photo flash, gaggle cameo, easter eggs, goose gifts, speech bubbles, standalone idle sleep. | High |
| `docs/adr/0004-...terminal-protection.md` | ADR | Terminal windows are never valid targets for window, focus, typing, drag, collect, or future spicy behavior. | High |
| `crates/honk-engine/src/{world,task,command,cursor,collect_window,foreign_window}.rs` | implementation | Engine already has platform-free tasks, finite pokes, capability-gated window/cursor/collect contracts. | High |
| `src/runtime/{windows,macos,linux}.rs` and platform crates | implementation | Windows/macOS have collect-window controllers; Linux reports collect unsupported; Wayland remains reduced. | High |
| `src/assets.rs` and `Assets/Images/Memes/custom/prompts.md` | implementation/assets | Asset loading currently covers notes and PNG memes only; spicy assets need new catalog slots and prompt records. | High |

### What's Already Decided

- `honk-engine` remains OS-free and `#![forbid(unsafe_code)]`.
- Runtime control remains CLI/TUI-only over local IPC; no tray, no global quit key, and no new global keyboard listener.
- Terminal windows are protected with no prank-mode override.
- Missing or unsupported assets/capabilities skip honestly and must not crash or leave stuck tasks.
- New image assets for #b9e must be generated with the project clumsy-paint base prompt.

---

## Problem Definition

What is happening: `#b9e` is a broad To-Do item with important safety and asset rules, but no implementation exists for clipboard honk, fake photo, cameo, gifts, speech bubbles, easter eggs, or idle sleep.

What should be happening: honk300 should add a default-off spicy behavior family that is opt-in through config/TUI/finite CLI pokes, uses generated house-style assets, degrades by platform capability, and preserves terminal protection by construction.

Gap: the repo has the right task/capability architecture, but no separate spicy domain model, config schema, asset catalog, render path, platform clipboard controller, or verification matrix for these behaviors.

### Reframes Considered

| Framing | Verdict |
|---|---|
| "Add more `CollectWindowKind` variants." | Rejected. It overloads M9 note/meme semantics, inherits default-on collect behavior, and pushes overlay-only jokes into OS-window manipulation. |
| "Implement #b9e as runtime-only effects." | Rejected. It would bypass deterministic engine tests and make config/reload behavior hard to reason about. |
| "Create a separate default-off spicy behavior domain." | Accepted. It matches the task wording and keeps new platform IO explicit. |

---

## Root Cause Analysis

1. Why is #b9e not a small edit? Because it spans engine tasks, config, TUI, control protocol, assets, render composition, platform capabilities, docs, and smoke tests.
2. Why not reuse collect-window? Because collect-window is for note/meme props and defaults enabled when content/capabilities exist; #b9e must be default-off and includes non-window visual effects.
3. Why do capabilities matter? Clipboard and any window-affecting behavior can be unavailable or unsafe on macOS/Linux/Wayland, while overlay visuals can work everywhere.
4. Why does terminal protection remain central? The task explicitly repeats the M10 rule, and several spicy ideas could otherwise tempt direct focus/type/window interactions.
5. Why plan before coding? The wrong architecture would quietly weaken safety guarantees or make future cross-platform behavior dishonest.

---

## Logical Reasoning Check

Argument:

P1. If a feature family contains default-off, capability-sensitive, user-facing mischief, then it needs its own explicit config/options/capability contract rather than borrowing a default-on behavior's contract.

P2. `#b9e` contains default-off, capability-sensitive, user-facing mischief.

P3. The existing collect-window contract is a default-on note/meme prop contract, not a spicy behavior contract.

Therefore, `#b9e` should be implemented as a separate spicy behavior contract, not as extra collect-window variants.

Support type: practical deductive/architectural argument. P1 is a project safety rule derived from ADR 0004 and prior capability/preference separation. P2 and P3 are directly supported by the task and code. Confidence: High.

---

## Design Direction

### New Domain Model

Add a platform-free `crates/honk-engine/src/spicy.rs` module and export it from `honk-engine`:

- `SpicyOptions`: master `enabled` plus individual default-false toggles:
  `clipboard_honk`, `fake_photo_flash`, `gaggle_cameo`, `easter_eggs`, `goose_gifts`,
  `speech_bubbles`, and `idle_sleep`.
- `SpicyCapabilities`: backend/session support for `clipboard`, `overlay_visuals`, and any future
  target/window operation. Defaults unsupported except pure overlay visuals when the runtime can render.
- `SpicyKind` / `SpicyPayload`: closed, typed behavior identifiers.
- `SpicyCommand`: platform/runtime intents such as `ClipboardHonk`, `ShowImage`, `ShowFlash`,
  `ShowSpeechBubble`, and `ClearVisual`, with no OS handles or paths.
- `SpicySnapshot` or `SpicyVisual`: active overlay effects the runtime can draw.

`WorldOptions` should gain `spicy: SpicyOptions`. `TaskCtx` should gain read-only spicy options and a mutable `spicy_commands` drain, parallel to collect-window commands.

### Behavior Split

| Behavior | Best implementation path | Notes |
|---|---|---|
| Fake-photo flash | Overlay-only visual | Captures nothing. No screenshot API. Render a flash/card/frame from generated asset. |
| Speech bubbles | Overlay visual with generated bubble PNGs | Avoid adding text-render/font complexity in the first pass by using baked/generated bubble notes. |
| Goose gifts | Overlay visual/task | Goose carries or drops generated gift props. Missing props skip. |
| Gaggle cameo | Mostly procedural overlay | Render a rare second goose rig; use generated cameo props only if needed. |
| Easter eggs | CLI/TUI/local sequence only | No global keyboard listener. Konami can live in TUI input state or explicit CLI/control surface. |
| Standalone idle sleep | Engine/render only | Reuse sleepy posture/Z-particle style without new OS capability. |
| Clipboard honk | Platform clipboard controller | Backup in memory, set a short honk message, restore after timeout/best-effort on exit; never log clipboard contents. |

### Config and TUI

Add `SpicyConfig` to `honk-config` with all fields defaulting to `false`. Prefer a separate TOML section:

```toml
[spicy]
enabled = false
clipboard_honk = false
fake_photo_flash = false
gaggle_cameo = false
easter_eggs = false
goose_gifts = false
speech_bubbles = false
idle_sleep = false
```

The TUI should expose these in the Mischief category or a new Spicy category. Rows should show unavailable capability state where applicable, but config toggles remain editable so a cross-platform config file can travel.

### CLI and IPC

Extend `PokeAction` and `CliPokeAction` only with finite variants needed for smoke and direct opt-in use, for example:

- `Photo`
- `Gift`
- `Bubble`
- `Cameo`
- `Clipboard`
- `Sleep`

Update `honk-control` encode/decode tests. Keep phrases explicit through `do <action>`; do not make bare `honk` mean a spicy action.

### Assets

Add a provenance-separated spicy asset layout:

```text
Assets/Images/Spicy/
  fake-photo/custom/
  gifts/custom/
  speech-bubbles/custom/
  easter-eggs/custom/
  cameo/custom/
```

Each folder gets a `prompts.md` with the verbatim base prompt and final subject prompts. Generated PNGs should be downscaled to practical overlay sizes, like M9 did, and all image slots must be optional.

The asset catalog should load counts and pixmaps for spicy asset classes separately from memes. Do not fold these into `meme_count`.

---

## Implementation Plan

1. Add ADR 0014 for the spicy behavior contract.
   - Record default-off semantics, no screenshot capture, no global keyboard hook, clipboard backup/restore rules, generated-asset policy, and terminal protection.

2. Add the engine contract.
   - New `spicy.rs`, `SpicyOptions`, `SpicyCapabilities`, `SpicyCommand`, active visual state, and world drain/feed methods.
   - Integrate into `World::pickable_for`, `poke`, `apply_options`, `render_bounds`, and tests.
   - Ensure default `WorldOptions` leaves the pickable task set unchanged.

3. Add config/TUI/control plumbing.
   - `honk-config`: default-false `[spicy]`, known-key preservation, validation, `effective_options`.
   - `honk-config-tui`: rows and toggles, status text, command help.
   - `honk-control` and `src/cli.rs`: finite action variants and protocol tests.

4. Add asset generation and loading.
   - Generate only the needed first-pass PNGs via `image_gen` using the base prompt from `.tasks/tasks/b9e.md`.
   - Store prompts beside assets.
   - Extend `src/assets.rs` with spicy asset loading and optional lookup APIs.

5. Add overlay visual runtime.
   - Runtime draws engine-reported visual effects after the goose using loaded pixmaps or procedural fallback shapes.
   - Fake photo is a flash/frame, not a screen capture.
   - Speech/gift/cameo visuals are bounded, timed, and included in dirty/render bounds.

6. Add platform clipboard support last.
   - Windows first, macOS if straightforward, Linux only if a safe session-specific path exists.
   - Unsupported sessions report unsupported rather than attempting shell tools blindly.
   - Clipboard backups are in memory only, size-bounded, never logged, and restored after timeout and on normal runtime shutdown.

7. Update docs and task state.
   - README/AGENTS/CLAUDE if behavior or guidance changes.
   - `CHANGELOG.md` and `HUMAN_CHANGELOG.md` in lockstep for user-facing changes.
   - `.tasks/tasks/b9e.md` with evidence and acceptance checks.

---

## Acceptance Gates

- Default config and `WorldOptions::default()` do not enable any #b9e behavior.
- Enabling one spicy toggle cannot implicitly enable all spicy behavior.
- Calm/quiet-hours/DND/fullscreen manners suppress autonomous spicy behavior; explicit `do` pokes may still be accepted when the matching capability exists.
- Terminal windows remain non-targets; no new code path focuses, types into, moves, drags, rides, or collects terminal windows.
- Fake photo uses no screen capture API and stores no pixels from the user's desktop.
- Clipboard honk restores the prior clipboard on the happy path and never logs the prior value.
- Missing generated assets skip cleanly.
- Linux/Wayland unsupported states are explicit and visible through status/TUI where relevant.

---

## Verification Plan

Local gate:

- `cargo fmt --all -- --check`
- `cargo clippy --all-targets --workspace -- -D warnings`
- `cargo test --workspace`
- `cargo build --release`

Target checks:

- Windows x64 and ARM64 cargo checks.
- macOS x64/ARM64 cargo checks.
- Linux GNU/musl x64/ARM64 cargo checks.

Focused tests:

- `honk-engine`: defaults off, enabled task selection, `poke` outcomes, visual lifetimes, render bounds, manners suppression.
- `honk-config`: default-false TOML, unknown-key warning preservation, effective options with backend capability states.
- `honk-config-tui`: toggle rows and command/status rows.
- `honk-control`: protocol round trips and malformed action rejection.
- Platform crates: terminal classifier regression remains green; clipboard capability tests where possible.
- Runtime smoke: enable each behavior individually, run direct pokes, capture visual evidence, verify clipboard restore, and verify unsupported reporting on Linux/Wayland paths.

---

## Steel-Manned Dissent

The case against this plan: #b9e is low-priority flavor work, so a quick implementation using existing collect-window image windows would ship visible jokes faster.

What would have to be true: the task would need to be merely "show more funny images" and not a default-off, safety-sensitive behavior family with clipboard and terminal-protection requirements.

How handled: rejected. The speed gain is not worth blurring the established capability/preference boundary or weakening terminal protection. The separate spicy contract is more work, but it keeps the behavior auditable.

Confidence: High.

---

## Closing

Exit state: Directed.

Recommended next move: add ADR 0014 and the default-off `spicy` engine/config skeleton first, with tests proving all defaults are off and the existing behavior deck is unchanged. Then layer overlay-only visuals, generated assets, direct pokes, and finally clipboard support.

Confidence: High for architecture and sequencing; Medium for exact asset count and platform clipboard scope until implementation tests the OS APIs.

Open questions:

- Should the first implementation include all #b9e items, or land a smaller slice of overlay-only visuals first?
- Should Konami be TUI-only, CLI-only, or both? It should not be a global keyboard listener.
- Should clipboard support be Windows-first only in the first pass, with macOS/Linux marked unsupported until separately proven?

Revisit trigger: start implementation or decide to split #b9e into child tasks.
