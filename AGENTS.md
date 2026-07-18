# AGENTS.md

This file provides guidance to Codex (Codex.ai/code) when working with code in this repository.

## What this repo is

A from-scratch, cross-platform (Windows/macOS/Linux) **Rust reimplementation of Desktop
Goose** (Samperson's desktop-pet). Target binary: **`honk300`** — a member of this machine's
`*300` tool family (siblings: TR300, ND300, WB300). `README.md` holds the one-paragraph brief.

**Current stage: v1.2.6 is the public stable/latest release at exact source commit
`c0ddb2c2c7cd4334040e324c72187eb6f3d4a644`; candidate `29663447227`, same-SHA main CI
`29663774278`, atomic publication `29664104522`, and all-eight-lane fresh-public-byte run
`29664433689` passed under ADRs 0031 and 0034–0035. The immutable release has 47 assets and 22
manifest payloads; exact-tag/latest manifest and command-first bootstrap bytes match. Production
browser QA confirmed v1.2.6, every advertised platform command and native alternative, an empty
warning/error console, and a bounded mobile layout. Task `#cli123` is complete.** M0-M19 are implemented
in-tree. M16.1 macOS Accessibility onboarding is implemented; one unchanged signed executable
passed first-denied, non-nagging relaunch, live-grant, and live-revocation on the physical M2.
Exact-final-SHA, unavailable desktop-driver/Ghostty, and one-display limitations are recorded as
explicit forward-verification waivers rather than stronger native claims. The repo now has a Cargo workspace, a platform-free
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
`docs/readiness/m16-m18-readiness.md` records the local gate, CI smoke gates, completed managed
macOS Accessibility evidence, and its honest hardware/tooling waivers. M19 adds real `install`,
`uninstall --purge`, and `update` code paths
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
installer for macOS/Linux, and one atomic immutable release workflow. R6/v1.0.1 and ADR 0020 add the
first native macOS qualification, Developer ID signing/notarization, a per-user graphical DMG,
and shared gait refinement. ADR 0022 adds managed one-prompt-per-update Accessibility onboarding,
a calm safe-edge wait, and same-process grant/revocation transitions. R6 also adds shared
exposed-edge locomotion (continuous monitor seams, occasional hidden wrap, fully offscreen
startup/graceful exit) and the user-only 30% collect-close annoyed reaction with a separately gated
bounded nab. Independent review closed the final-clear, gapped-topology, state-latch/deadline,
typed-close, Windows-provenance, and reaction-visibility regressions with focused tests. Retained
lifecycle ownership now pins verified Windows artifacts through execution, removes ambient lease
bypasses, rolls Unix signals back explicitly, checks machine-wide Windows paths across sessions,
and kills every unaccepted deferred helper. ADR 0023 adds stable rolling-latest links, exact-tag
platform/provenance-isolated updates, and native amd64/arm64 Debian package ownership. ADR 0024
adds one macOS-only status item whose Configure action launches the existing terminal TUI and
whose Quit action enters the shared graceful walk-off; it adds no native settings model. ADR
0028 replaces its text title with the shared accessible goose template icon and makes those
Configure/TUI and graceful-Quit semantics the platform parity contract. ADR 0030 implements it
with a fixed-GUID native Windows notification-area item and a pure-Rust Linux StatusNotifierItem,
both backed by one shared command router and explicit non-fatal unavailable-host reporting. ADR
0025 established the first stable
v1 release and makes later Alienware hands-on checks post-release input to forward patches. ADR
0027 keeps the failed unpublished
`v1.0.0` tag immutable, makes `v1.0.1` the first public target, and permits either complete
renderer view in Windows qualification without weakening channel/alpha checks. Candidate
`29384134561` proved the complete signed/notarized/stapled Mac
producer and Windows x64 compositor/lifecycle path at commit `3908794`, then failed closed on the
X11 compositor's cached gray root tile and on Windows ARM64 DPI/capture coordination. The current
tree replaces the Linux qualification background with a persistent test-only X11 client, makes
both Windows smoke processes PMv2-aware with an atomic tokenized color channel and preflight
capture proof, and treats a closed downstream status pipe as normal without hiding other write
errors. A live collect run also exposed the body-target-versus-beak-arrival locomotion mismatch;
current source repairs it with a beak-offset target and a realistic fixed-tick regression, and the
complete integrated local gate passes. Candidate `29386819926` at `bc3c1d9` then passed the
complete trusted Mac producer, both Apple and Windows portable builds, the ARM64 Windows
installer producer, and all four repaired X11 compositor halves. It failed closed because the
evidence-derived Wayland socket paths exceeded Linux's 108-byte AF_UNIX limit and the Windows x64
controller treated matching native CRLF diagnostics as one line. Current source gives Wayland a
short owner-only cleaned runtime directory and parses exact CRLF/LF diagnostic lines; neither fix
weakens product behavior. Candidate `29387569722` at `5d0237f` then passed the complete trusted
Mac producer, all Apple/Windows portable jobs, Windows x64's full paired-DWM compositor/lifecycle
gate, native ARM64 PE/MSI lifecycle, and X11 plus dual-output Wayland on three of four Linux
variants. It failed only because one valid top-down x64 GNU X11 pose had 13 warm pixels against a
side-view-derived floor of 20, and GitHub's hosted ARM64 screen API returned the same static
wallpaper for two acknowledged visible-window colors. Current source calibrates the Linux warm
floor to 10 while retaining body/wing/transparency checks. ADR 0026 permits an ARM64 fallback only
for the exact GitHub-hosted wallpaper signature: the native process must expose a visible HWND and
atomically record the cropped premultiplied-BGRA DIB only after `UpdateLayeredWindow` succeeds;
raw alpha/channel, opaque-surface, articulation, shadow, exact-HWND, and bounded fresh-rectangle
checks remain fail closed. Local/self-hosted ARM64 and all ordinary x64 runs still require paired
DWM capture. Exact candidate `29389882143` at `c44b89d` passed the complete Mac producer, every
portable/native Windows and Linux gate, both Debian packages, and final assembly. Same-SHA main CI
then exposed only fixture contamination on Ubuntu 24.04: inherited `/etc/sway/config` wallpaper/bar
remained visible through Honk300's transparent layer on one headless output. A follow-up private
config still carried its own wildcard while exact setters used swaybg's one-pixel solid-color path;
Noble's 1.5-scale linear pixman filter turned that buffer into a pre-launch gradient. Current source
starts Sway with no background rule, tiles opaque PNGs only on discovered exact output names, and
proves both outputs while preserving fractional filtering before launching the goose; it does not
add a product background or weaken the analyzer. Exact candidate `29392439475` and same-SHA main
CI `29392827146` passed at `9c5692b`. The immutable `v1.0.0` release then failed before draft
creation only because Windows x64 sampled valid top-down frames against a side-only oracle; all
other producers, including signed/notarized Mac and native Debian, passed. The strict ADR 0027
side-or-top-down analyzer now rejects partial/cropped, channel-swapped, opaque, straight-alpha,
and double-premultiplied evidence. Exact candidate `29401457634`, same-SHA main CI
`29401961540` attempt 2, atomic release `29403056159`, and post-release smoke `29403596212`
passed at `de8da8a9dd049286787d20e167bb115ce8afc107`. v1.0.2 replacement candidate
`29565557915`, same-SHA main CI `29566294408`, atomic publication `29566759574`, and
post-release smoke `29567257622` then passed at
`964305869e9ec28768c789465db1b6317dfa3f6f`. Public Mac trust, graphical install,
v1.0.1→v1.0.2 managed update, menu/TUI/graceful-Quit, and the DMG-first production site passed.
Candidate `29577145711` attempt 2, same-SHA main CI `29577774029`, atomic publication
`29578238463`, and post-release smoke `29578671930` passed v1.0.3 at
`5192fab9690ff8b6777366a5918c12bbe1ee247a`. Completed evidence is in
`docs/readiness/v1.0.3-readiness.md`. Candidate `29588487072`, same-SHA main CI `29589048598`,
atomic publication `29589698302`, and post-release smoke `29590274819` passed v1.1.0 at
`e58b5ec09ea140e22927e3f8e8cf339b5a7d5bea`. Its completed release gate is
`docs/readiness/v1.1.0-readiness.md`; `#v1a`, `#r103`, `#trayc`, `#wtray`, `#ltray`, and `#r110`
are done.

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
  asset, and no-donate decisions; ADR 0004 records the original M10 CLI/TUI-only control plane,
  local IPC, and terminal-window protection rule (only its macOS menu prohibition is superseded
  by ADR 0024); ADR 0007 records the M13 dynamic-mood and local-time
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
  ADR 0022 records the managed macOS Accessibility first-run and live-transition boundary; ADR
  0023 defines every-release cross-platform production, stable latest names, exact update
  identity, and Debian package lifecycle ownership; ADR 0024 defines the macOS-only menu-bar
  bridge to the existing configuration TUI and graceful shutdown; ADR 0025 records the first
  stable v1 intent and post-release hardware-verification boundary; ADR 0026 records the strict
  hosted Windows ARM64 compositor-evidence boundary without claiming a DWM screenshot; ADR 0027
  records the immutable v1.0.0 failure, v1.0.1 fix-forward, and pose-complete Windows oracle; ADR
  0028 records the shared goose control-surface icon and tray behavior parity contract; ADR 0029
  records the Alienware-derived Windows lifecycle, update, Corporate retry, and integrated-
  terminal hardening contract; ADR 0030 implements native Windows/Linux tray parity and its
  unavailable-host boundaries.

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
- Packaging: recommend the stable versionless official bootstrap on every platform: PowerShell
  `irm ... | iex` on Windows and the no-sudo `curl ... | sh` bootstrap on macOS/Linux. The Mac
  bootstrap installs the real signed universal app in `~/Applications`; the graphical DMG,
  x64/ARM64 Global/Corporate MSI/EXE packages, and architecture-matched Debian packages remain
  supported native alternatives. Raw Cargo/source/portable installs are unmanaged and never
  retire another installer owner. **No crates.io.**
- Every general stable tag builds the complete platform set in GitHub Actions, including a fresh
  GitHub-macOS-produced signed/notarized/stapled app and DMG plus both Debian packages regardless
  of the operator's trigger host. Stable unversioned `latest/download` names advance only after
  atomic publication; existing tagged assets never change. The updater discovers through the
  latest manifest but downloads exact-tag bytes and requires platform, architecture, install
  provenance, artifact kind, size, and SHA-256 to agree before mutation. The DMG is a graphical
  install artifact; managed Mac CLI updates consume the exact-tag universal app ZIP through the
  pinned bootstrap.
- Release-mode macOS artifacts must fail closed without Developer ID and App Store Connect API
  credentials. No ad-hoc release fallback, `codesign --deep` signing, or DMG `/Applications`
  symlink is permitted.
- Automatic macOS Accessibility UI is limited to the exact DMG- or shell-receipted app at
  `~/Applications/Honk300.app`. It prompts at most once per installed update, waits calmly at a
  safe screen edge while denied, and handles grants/revocations in the same process. Development,
  bare, source-tree, and mounted-DMG launches must not open permission UI automatically.
- macOS transparent presentation uses reusable premultiplied-RGBA AppKit image views in the
  ordinary window backing store. Keep canvas/bitmap capacity bounded after transiently large
  damage so screen capture stays alpha-correct and normal walking does not redraw stale space.
  The alpha-last bitmap is Device RGB, the overlay window has a stable standard-sRGB destination,
  and final display-profile composition belongs to WindowServer rather than a per-frame
  application-side Device-RGB-to-Display-P3 conversion.
- Startup and graceful shutdown are locomotion states: stage the goose fully beyond a real exposed
  edge, walk in, and keep ticking/presenting on stop until the full pose has walked out. Shared
  monitor seams are continuous; only genuinely exposed edges may use the occasional 20% hidden
  wrap, and deliberate puddle/prank errands never wrap.
- Ordinary `stop`/`quit`/`exit` and native Quit always use that graceful walk-off. The explicit
  `--force` variants on all three command names terminate immediately through separate IPC.
- Windows collected notes are Honk300-owned native edit windows, never Notepad or global input.
  Windows/macOS notes and images share the ADR 0032 monitor-relative fit: hard 48% per-dimension
  ceiling, aspect-preserving complete-image downscale, no crop, and no upscaling.
- A user—not program cleanup—closing a spawned note or meme gets an independent 30% annoyed
  reaction roll. The reaction may chain only the existing bounded cursor nab and only after live
  capability, permission/pointer, configuration, and manners checks. Linux collect windows remain
  unsupported and therefore produce no native close trigger.
- Install/update/uninstall retain the real runtime singleton for the whole mutation. Unix signals
  roll back with explicit nonzero status. Windows payloads remain pinned from same-stream
  size/hash verification through execution; generated bootstrap delegation must reacquire without
  an ambient bypass, machine-wide paths are checked across sessions, reboot-deferred MSI results
  fail closed, and every pre-READY helper error kills and waits for the child.
- Update provenance is authoritative and never guessed from a Windows install path. A protected
  v2 receipt preserves installer family, edition, scope, stable track, target, owned root, active
  release, and exact artifact identity. Windows and shell-managed Linux activate immutable slots
  through neutral selectors while the initiating process stays mapped to its old release; DMG,
  Debian, and package/bootstrap origins remain distinct. A fresh verified installer may
  intentionally downgrade and becomes the user's latest intent. Conflicting Windows registrations
  are retired only after commit through a protected journal; an opposite-scope cleanup that lacks
  an administrator grant stays nonzero `cleanup_pending` and must not claim public alias takeover.
  One hidden elevated active-slot coordinator runs the validated native uninstall and retires only
  that old root's exact PATH/Run entries before verifying the active PATH; it must not prompt twice.
- `[lifecycle].autostart_on_login` is default-off and reconciles only through the receipt-owned
  Windows Run value, managed Mac LaunchAgent, or per-user Linux XDG entry. Fresh installer intent
  outranks stale config; later explicit config edits update the same owned mechanism. Foreign or
  ambiguous startup ownership fails closed and must never create a duplicate persistence path.
- Windows keeps the three public aliases as console-subsystem commands for intentional CLI use.
  Start Menu/desktop shortcuts and login startup target the independently hashed GUI-subsystem
  `honk300-app.exe`, which starts only its exact sibling with `CREATE_NO_WINDOW`, null handles,
  and no shell intermediary. Background restarts/helpers must never create or focus a terminal.
  Product startup performs no screen calibration; full-desktop compositor surfaces are
  disposable-CI-only.
- Starting, stopping, and configuration remain **CLI/TUI over local IPC** on every platform.
  The macOS menu-bar item, Windows notification-area item, and compatible Linux StatusNotifier
  item expose the same shared icon, accessible naming, existing-TUI Configure action, and engine-
  owned graceful Quit. There is no native preferences model or global quit key; an unavailable
  shell host is non-fatal and does not change overlay or mischief capability claims.
- Terminal windows are protected: the goose may visually overlay them, but must never move,
  focus, type into, drag, ride, collect, or otherwise manipulate terminal windows, including in
  spicy/default-off modes. Conservatively treat Codex and Visual Studio Code surfaces as terminal
  windows across platforms, including the ChatGPT-titled Codex desktop surface observed on
  Windows.

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
- M10's accepted decisions live in `docs/adr/0004-m10-cli-tui-control-plane-and-terminal-protection.md`;
  ADR 0024 supersedes only its no-menu-bar clause on macOS.
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
- Rolling latest artifacts, every-release Mac production, platform-isolated updates, and Debian
  package lifecycle ownership live in
  `docs/adr/0023-rolling-latest-artifacts-and-debian-lifecycle.md`.
- The macOS-only status item, terminal-TUI launcher, and graceful menu Quit live in
  `docs/adr/0024-macos-menu-bar-control.md`.
- The first stable v1 intent and post-release Alienware verification boundary live in
  `docs/adr/0025-first-stable-v1-release.md`.
- The GitHub-hosted Windows ARM64 wallpaper-capture exception and exact post-success presenter-DIB
  evidence boundary live in `docs/adr/0026-hosted-windows-arm64-compositor-evidence.md`.
- The immutable v1.0.0 failure, v1.0.1 fix-forward, and complete side/top-down Windows evidence
  live in `docs/adr/0027-v1-0-1-fix-forward-and-windows-pose-evidence.md`.
- The shared goose menu/tray icon and Configure/TUI plus graceful-Quit parity contract live in
  `docs/adr/0028-shared-goose-control-surface-and-tray-parity.md`.
- The Windows update/lifecycle, Corporate retry, and integrated-terminal hardening contract lives
  in `docs/adr/0029-windows-lifecycle-and-terminal-hardening.md`.
- The native Windows/Linux tray owners, shared command router, recovery, package ownership, and
  explicit unavailable-host boundary live in `docs/adr/0030-windows-linux-tray-control-parity.md`.
- Provenance-preserving v2 receipts, synchronous updates, immutable Windows/Linux release slots,
  latest-intent activation, and DMG-origin app-ZIP updates live in
  `docs/adr/0031-provenance-preserving-slot-self-update.md`; it supersedes only ADR 0029's Windows
  update transaction and receipt-refresh portions.
- Owned monitor-bounded props, graceful versus forced lifecycle, and provenance-owned login start
  live in
  `docs/adr/0032-owned-props-lifecycle-and-login-autostart.md`; it supersedes ADR 0003/0015 only
  where they required an external Windows Notepad process and synthetic input.
- Windowless Windows app/login/background launch and disposable-desktop-only full compositor
  qualification live in
  `docs/adr/0033-windowless-windows-app-launch-and-disposable-desktop-qualification.md`; it
  supersedes ADR 0032's Windows startup command and interactive-local calibration notice.
- Command-first managed installation, native-package alternatives, fresh-intent takeover, and
  unmanaged Cargo boundaries live in `docs/adr/0034-command-first-managed-installation.md`; it
  supersedes only the package-first recommendations in ADRs 0020, 0023, and 0031.
- The narrowly evidence-gated hosted x64 tray-registration observation boundary lives in
  `docs/adr/0035-hosted-windows-tray-registration-qualification-boundary.md`; ordinary Windows
  CI remains strict and unwaived.

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
- **Windows `PIPE_NOWAIT` zero-byte reads are transient, not EOF.** A client can connect before
  its command bytes become readable; keep the bounded retry in `honk-control` and never decode a
  successful zero-byte poll as an empty frame. A peer that stays silent must still hit the
  existing deadline and fail closed.
- **macOS needs a real `.app` bundle** (stable bundle-id) for a durable Accessibility grant;
  a bare `~/.cargo/bin` binary can't hold one. The bundle remains an LSUIElement agent/permission
  identity with no native preferences window, running Dock control surface, or AppleScript
  `.sdef` command surface. Its one macOS status item uses the sealed shared goose template image,
  an independent **Honk300 controls** accessibility label, and only launches the existing terminal
  TUI or requests engine-owned graceful shutdown. Keep its image and weak target retained and all
  AppKit access on the main thread. Windows and Linux implementations preserve those semantics
  through the shared finite command type and router; keep platform callbacks free of engine state.
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
- Windows local lifecycle only, and only when the operator welcomes visible testing: `pwsh -File script/smoke_windows_overlay.ps1 -Binary target/release/honk300.exe -EvidenceDirectory target/windows-overlay-evidence -LifecycleOnly`. Full paired-color compositor proof is disposable-CI-only; `-AllowInteractiveDesktopObscuration` is intentionally rejected locally.
- Linux host only: `HONK300_BIN="$PWD/target/release/honk300" bash script/smoke_m17_m18_linux.sh`
- `dist plan --tag=v1.2.6`
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
