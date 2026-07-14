# CODEX_PROJECT.md

## TL;DR

Honk300 is a Rust 1.95, cross-platform procedural desktop goose for Windows, macOS, X11, and
native Wayland reduced mode. M0-M19 and the v0.3.x stabilization work are in-tree. The active
v0.3.3 effort is the first complete native macOS qualification: fix AppKit pixel presentation,
refine shared walking, meet runtime budgets, prove denied/granted Accessibility, ship a
Developer ID-signed/notarized per-user DMG, publish atomically, and only then make that DMG the
recommended macOS download at thegoose.app.

## Status

- Active branch/worktree: `codex/macos-v0.3.3` in `.worktrees/macos-v0.3.3`.
- Stable public release remains v0.3.2 until the v0.3.3 readiness checklist is complete.
- Active board card: `#m20q`; live shared task board is tracked under `.tasks/`.
- Version: 0.3.3 in source.
- Renderer: AppKit/CoreGraphics RGBA regressions pass. A custom child layer was found to bypass
  reliable WindowServer capture, so the fixed reusable AppKit image-view presenter is awaiting
  fresh Developer ID-signed light/dark compositor evidence.
- Shared gait: releases planted feet at four pixels, preserves the weighted normal/moderate
  cadence, caps visible lag at 16 px for Walk and 26 px for Run/Charge, and passes cadence guards,
  eight-direction tests, and seven goldens without overcorrecting ordinary walking.
- Performance: a follow-up active-motion profile invalidated the earlier idle-path 8.30% CPU
  baseline. Sampling traced 40.45% median active CPU after a large transient to stale oversized
  transparent canvas/bitmap capacity rather than the engine; bounded shrinking is implemented and
  the exact signed 10-second-warm-up/60-second confirmation remains open.
- Lifecycle: mounted-bundle copy, aliases, autostart, release-bound receipt, preservation, purge
  backup, rollback structure, and isolated-home cleanup are implemented and tested.
- Packaging: a universal x86_64/arm64 installer helper targeting macOS 11.0 in both slices and a
  fail-closed signing/notarization workflow are in-tree. The current universal app and helper pass
  current G2 Developer ID chain, team, hardened-runtime, timestamp, designated-requirement, and
  slice checks; no App Store Connect API key is configured.
- Release/site: the site progressive-disclosure implementation and local tests are complete, but
  candidate notarization, native qualification, release publication, and live-manifest validation
  remain deliberate prerequisites to preview/production deployment.

## Goals

1. Preserve a platform-free 120 Hz simulation engine and shared procedural renderer.
2. Present correct premultiplied-alpha output through each native desktop backend.
3. Keep all CLI/TUI/IPC control local, single-instance, and owner-scoped.
4. Never manipulate terminal windows, even when visual overlays may cover them.
5. Keep install/update/uninstall transactional and preserve foreign files and user media.
6. Publish immutable, complete, machine-verifiable releases with no crates.io distribution.
7. Make the signed, notarized, stapled universal DMG the primary macOS download only after a
   fresh published artifact passes independent checks.

## Architecture

- `crates/honk-engine`: unsafe-free fixed-step simulation, tasks, rig, feet, behaviors, and
  procedural raster renderer. No OS dependencies.
- `crates/honk-control`: closed local IPC protocol and same-user transport.
- `crates/honk-config` / `honk-config-tui`: schema-v2 TOML and ratatui control surface.
- `crates/honk-platform-*`: native Windows, capture-safe AppKit/CoreGraphics, X11, and Wayland
  adapters.
- `src/runtime`: platform event loops built around shared `RuntimeCore` ordering.
- `src/install.rs` / `src/update.rs`: ownership receipts, atomic lifecycle transactions,
  channel-aware updates, and foreign-file preservation.
- `.github/workflows/release.yml`: candidate-first and atomic immutable release orchestration.
- `.github/workflows/macos-packaging.yml`: universal app/helper signing, app+DMG notarization,
  stapling, validation, and evidence.
- `packaging/macos`: native graphical helper source and DMG instructions.
- `docs/adr`: durable decisions. ADR 0020 is current for macOS distribution; ADR 0018 remains
  current for atomic publication and non-macOS install decisions.

## Verification Source Of Truth

- Current release gate: `docs/readiness/v0.3.3-readiness.md`.
- Native historical/backend evidence: `docs/readiness/m16-m18-readiness.md`.
- Board handoff and activity: `.tasks/tasks/m20q.md`.
- Canonical product plan: `honk300_plan.md`.
- Required local gate: fmt, workspace clippy with warnings denied, workspace tests, release build,
  universal Apple builds, `dist plan --tag=v0.3.3`, complete Python contracts, cargo-audit
  0.22.2, actionlint, and diff check.

## Current Workspace Tree

Generated 2026-07-13. Build output, Git internals, worktree internals, Python bytecode caches, and
Finder metadata are excluded; every project source/document/configuration path is included.

```text
.
├── .claude
│   └── settings.json
├── .gitattributes
├── .github
│   ├── actionlint.yaml
│   └── workflows
│       ├── ci.yml
│       ├── macos-packaging.yml
│       ├── post-release-smoke.yml
│       ├── release.yml
│       └── windows-installers.yml
├── .gitignore
├── .superpowers
│   └── sdd
│       ├── .gitignore
│       ├── progress.md
│       ├── task-1-brief.md
│       ├── task-1-report.md
│       ├── task-2-brief.md
│       ├── task-2-report.md
│       ├── task-3-brief.md
│       ├── task-3-report.md
│       ├── task-4-brief.md
│       └── task-4-report.md
├── .tasks
│   ├── .board-server.json
│   ├── .board-server.log
│   ├── .gitignore
│   ├── .install-manifest.json
│   ├── CLAUDE.md
│   ├── MILESTONES.md
│   ├── TASKS.md
│   ├── board-server.mjs
│   ├── config.json
│   ├── dashboard.html
│   ├── memory
│   │   ├── context
│   │   │   └── .gitkeep
│   │   ├── glossary.md
│   │   ├── people
│   │   │   └── .gitkeep
│   │   └── projects
│   │       └── .gitkeep
│   ├── milestones
│   ├── secure
│   │   └── README.md
│   ├── tasks
│   │   ├── a6e.md
│   │   ├── a8d.md
│   │   ├── gla.md
│   │   ├── m16r.md
│   │   ├── m17r.md
│   │   ├── m18r.md
│   │   ├── m20q.md
│   │   ├── p4d.md
│   │   └── r5d.md
│   └── vendor
│       ├── animated-brand-mark.js
│       ├── anime.min.js
│       └── fonts
│           ├── ibm-plex-mono
│           │   ├── IBMPlexMono-Medium.woff2
│           │   ├── IBMPlexMono-Regular.woff2
│           │   └── IBMPlexMono-SemiBold.woff2
│           └── unbounded
│               ├── Unbounded-Bold.woff2
│               └── Unbounded-Regular.woff2
├── 2026-06-28-122345-can-you-verify-that-codex-did-m7-and-m8-properly.txt
├── 2026-07-08-221230-examine-this-goose-program-and-evaluate-it-for-s.txt
├── AGENTS.md
├── Assets
│   ├── Images
│   │   └── Memes
│   │       ├── custom
│   │       │   ├── CustomGooseDance.png
│   │       │   ├── CustomMeme1.png
│   │       │   ├── CustomMeme2.png
│   │       │   ├── CustomMeme3.png
│   │       │   ├── CustomMeme4.png
│   │       │   ├── CustomMeme5.png
│   │       │   ├── CustomMeme6.png
│   │       │   ├── CustomMeme7.png
│   │       │   └── prompts.md
│   │       ├── originals
│   │       │   ├── GooseDance.gif
│   │       │   ├── Meme1.png
│   │       │   ├── Meme3.png
│   │       │   ├── Meme4.png
│   │       │   ├── Meme5.png
│   │       │   ├── Meme6.png
│   │       │   ├── Meme7.png
│   │       │   └── SCREENING.md
│   │       └── user
│   │           └── Meme8.png
│   ├── Sounds
│   │   ├── BITE.mp3
│   │   ├── Honk1.mp3
│   │   ├── Honk2.mp3
│   │   ├── Honk3.mp3
│   │   ├── Honk4.mp3
│   │   ├── MudSquith.mp3
│   │   ├── Pat1.wav
│   │   ├── Pat2.wav
│   │   └── Pat3.wav
│   └── Text
│       └── NotepadMessages
│           ├── custom
│           │   ├── custom-am-goose.txt
│           │   ├── custom-good-work.txt
│           │   ├── custom-gooseASCII1.txt
│           │   ├── custom-hard-to-type.txt
│           │   ├── custom-i-cause-problems.txt
│           │   └── custom-peace-was-never.txt
│           └── originals
│               ├── am goose.txt
│               ├── good work.txt
│               ├── gooseASCII1.txt
│               ├── hard to type.txt
│               ├── i cause problems.txt
│               └── peace was never.txt
├── CHANGELOG.md
├── CLAUDE.md
├── CODEX_PROJECT.md
├── Cargo.lock
├── Cargo.toml
├── HUMAN_CHANGELOG.md
├── LICENSE
├── README.md
├── THIRD_PARTY_ASSETS.md
├── claude_plan.md
├── codex_plan.md
├── crates
│   ├── honk-config
│   │   ├── Cargo.toml
│   │   └── src
│   │       └── lib.rs
│   ├── honk-config-tui
│   │   ├── Cargo.toml
│   │   └── src
│   │       ├── app
│   │       │   ├── action.rs
│   │       │   ├── mod.rs
│   │       │   └── state.rs
│   │       ├── lib.rs
│   │       ├── terminal
│   │       │   └── mod.rs
│   │       └── ui
│   │           └── mod.rs
│   ├── honk-control
│   │   ├── Cargo.toml
│   │   └── src
│   │       ├── lib.rs
│   │       ├── platform.rs
│   │       └── protocol.rs
│   ├── honk-engine
│   │   ├── Cargo.toml
│   │   ├── examples
│   │   │   └── preview.rs
│   │   ├── src
│   │   │   ├── autumn.rs
│   │   │   ├── collect_window.rs
│   │   │   ├── command.rs
│   │   │   ├── cursor.rs
│   │   │   ├── entity.rs
│   │   │   ├── feet.rs
│   │   │   ├── footmarks.rs
│   │   │   ├── foreign_window.rs
│   │   │   ├── hearts.rs
│   │   │   ├── interaction.rs
│   │   │   ├── layout.rs
│   │   │   ├── lib.rs
│   │   │   ├── locomotion.rs
│   │   │   ├── math.rs
│   │   │   ├── mood.rs
│   │   │   ├── render
│   │   │   │   ├── geom.rs
│   │   │   │   ├── mod.rs
│   │   │   │   ├── side.rs
│   │   │   │   └── top.rs
│   │   │   ├── rig.rs
│   │   │   ├── rng.rs
│   │   │   ├── schedule.rs
│   │   │   ├── sound.rs
│   │   │   ├── task.rs
│   │   │   ├── time.rs
│   │   │   └── world.rs
│   │   └── tests
│   │       ├── golden
│   │       │   ├── side_left.png
│   │       │   ├── side_mid_stride.png
│   │       │   ├── side_reaching.png
│   │       │   ├── side_rest.png
│   │       │   ├── top_down.png
│   │       │   └── top_down_diag.png
│   │       └── golden.rs
│   ├── honk-platform-linux
│   │   ├── Cargo.toml
│   │   └── src
│   │       └── lib.rs
│   ├── honk-platform-macos
│   │   ├── Cargo.toml
│   │   └── src
│   │       └── lib.rs
│   └── honk-platform-windows
│       ├── Cargo.toml
│       └── src
│           └── lib.rs
├── docs
│   ├── adr
│   │   ├── 0001-m7-cursor-mischief-renderer-and-platform-guardrails.md
│   │   ├── 0002-m8-foreign-window-watch-and-ride.md
│   │   ├── 0003-m9-collect-window-assets-and-no-donate.md
│   │   ├── 0004-m10-cli-tui-control-plane-and-terminal-protection.md
│   │   ├── 0005-m11-cli-grammar-and-poke-outcome-round-trip.md
│   │   ├── 0006-m12-config-tui-and-capability-preference-boundary.md
│   │   ├── 0007-m13-moods-and-local-time-injection.md
│   │   ├── 0008-m14-schedule-presence-and-autumn.md
│   │   ├── 0009-m15-multi-monitor-and-appearance.md
│   │   ├── 0010-m16-macos-backend-agent-bundle-and-tui-status.md
│   │   ├── 0011-m17-m18-linux-control-runtime-and-degraded-wayland.md
│   │   ├── 0012-m16-1-m18-1-ci-proven-backend-readiness.md
│   │   ├── 0013-m19-lifecycle-packaging-and-deferred-macos-distribution.md
│   │   ├── 0014-renderer-v2-flat-illustration-dual-view.md
│   │   ├── 0015-reliability-and-platform-safety-fixes.md
│   │   ├── 0016-idle-life-behaviors-meander-mud-excursions.md
│   │   ├── 0017-macos-packaging-and-lifecycle.md
│   │   ├── 0018-distribution-and-atomic-release.md
│   │   ├── 0019-stabilization-contracts.md
│   │   ├── 0020-macos-developer-id-dmg-distribution.md
│   │   ├── 0021-native-wayland-capability-strata.md
│   │   ├── 0022-macos-accessibility-first-run-onboarding.md
│   │   └── README.md
│   ├── agents
│   │   └── handoff
│   │       └── 2026-07-13-001-macos-v0-3-3-qualification-release.md
│   ├── art-reference
│   │   ├── goose-side-alt.svg
│   │   ├── goose-side-head-left.svg
│   │   ├── goose-side-main.svg
│   │   └── goose-top-down.svg
│   ├── readiness
│   │   ├── m16-m18-readiness.md
│   │   ├── macos-handson-checklist.md
│   │   ├── v0.3.1-readiness.md
│   │   ├── v0.3.2-readiness.md
│   │   └── v0.3.3-readiness.md
│   ├── research
│   │   └── native-wayland-capability-path.md
│   ├── superpowers
│   │   ├── plans
│   │   │   └── 2026-07-13-macos-accessibility-first-run.md
│   │   └── specs
│   │       └── 2026-07-13-macos-accessibility-first-run-design.md
│   └── thinking
│       ├── 2026-06-27-m9-collect-window-plan.md
│       ├── 2026-07-06-active-task-resolution-plan.md
│       └── 2026-07-07-b9e-spicy-behaviors-plan.md
├── honk300_plan.md
├── honk300_planning.txt
├── inno
│   ├── corporate.iss
│   ├── global.iss
│   ├── install-source-exe-corporate.txt
│   └── install-source-exe-global.txt
├── packaging
│   └── macos
│       ├── DMG-README.txt
│       └── InstallHonk300
│           └── main.swift
├── rust-toolchain.toml
├── script
│   ├── honk300-installer.ps1.in
│   ├── honk300-installer.sh.in
│   ├── package_macos_app.sh
│   ├── package_macos_installer_helper.sh
│   ├── release_metadata.py
│   ├── smoke_m16_macos.sh
│   ├── smoke_m16_macos_accessibility.sh
│   ├── smoke_m17_m18_linux.sh
│   ├── smoke_released_unix.sh
│   ├── smoke_released_windows.ps1
│   └── tests
│       ├── test_installer_templates.py
│       ├── test_macos_packaging.py
│       ├── test_macos_smoke_contract.py
│       ├── test_post_release_smoke.py
│       ├── test_release_metadata.py
│       ├── test_release_workflows.py
│       └── test_windows_packaging.py
├── security-audit
│   └── 2026-07-10-1635-diff-review.md
├── src
│   ├── assets.rs
│   ├── audio.rs
│   ├── cli.rs
│   ├── install.rs
│   ├── main.rs
│   ├── runtime
│   │   ├── core.rs
│   │   ├── linux.rs
│   │   ├── macos.rs
│   │   ├── macos_accessibility.rs
│   │   ├── mod.rs
│   │   └── windows.rs
│   └── update.rs
├── thrum5.txt
├── vendor
│   └── wayland-scanner
│       ├── CHANGELOG.md
│       ├── Cargo.lock
│       ├── Cargo.toml
│       ├── Cargo.toml.orig
│       ├── LICENSE.txt
│       ├── README.md
│       ├── UPSTREAM.md
│       ├── src
│       │   ├── c_interfaces.rs
│       │   ├── client_gen.rs
│       │   ├── common.rs
│       │   ├── interfaces.rs
│       │   ├── lib.rs
│       │   ├── parse.rs
│       │   ├── protocol.rs
│       │   ├── server_gen.rs
│       │   ├── token.rs
│       │   └── util.rs
│       └── tests
│           └── scanner_assets
│               ├── test-client-code.rs
│               ├── test-headerless-protocol.xml
│               ├── test-interfaces.rs
│               ├── test-protocol.xml
│               └── test-server-code.rs
├── wix
│   ├── honk300-license.rtf
│   ├── install-source-msi-global.txt
│   └── main.wxs
└── wix-corporate
    ├── corporate.wxs
    └── install-source-msi-corporate.txt

78 directories, 263 files
```
