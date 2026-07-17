# CODEX_PROJECT.md

## TL;DR

Honk300 is a Rust 1.95, cross-platform procedural desktop goose for Windows, macOS, X11, and
native Wayland reduced mode. M0-M19 and the v0.3.x stabilization work are in-tree. v1.0.3 is the
public stable/latest release at exact source commit
`5192fab9690ff8b6777366a5918c12bbe1ee247a`. It retains v1.0.1's native macOS qualification,
shared gait/edge lifecycle, Developer ID/notarized DMG delivery, Debian packages, rolling updates,
and atomic publication while replacing the temporary Mac menu title with a shared accessible
goose icon. The icon's sealed macOS 11-safe representation and exact Configure/TUI plus animated-
Quit behavior are the contract future Windows/Linux trays must mimic. ADR 0029 adds the completed
Alienware-derived Windows lifecycle/update and integrated-terminal hardening.

## Status

- Default branch `main` contains the post-release v1.0.3 closure; the immutable tag peels to
  `5192fab9690ff8b6777366a5918c12bbe1ee247a`. Prior public tags/releases remain untouched history.
- Completed release/evidence cards: v1.0.3 `#r103`, v1.0.2 `#v102`, v1.0.1 `#m20q`, native Mac
  evidence `#m16r`, and Alienware verification `#v1a`. Shared tray contract `#trayc` is active for
  the separate v1.1.0 release. The shared board lives under `.tasks/`.
- Version: 1.0.3 in source and public stable/latest.
- v1.0.3 release evidence: candidate `29577145711` attempt 2, same-SHA main CI `29577774029`,
  atomic publication `29578238463`, and post-release smoke `29578671930` all passed at the exact
  commit. The immutable GitHub Release contains 22 payloads plus sidecars/manifest for 47 assets.
- macOS control: ADRs 0024/0028 define a visible-while-running image-only goose status item with
  an independent **Honk300 controls** accessibility name. Configure opens the signed bundle's
  existing terminal TUI; Quit requests the same engine-owned walk-off used by CLI/TUI stop. The
  canonical Quiver SVG and deterministic 36×36 PNG are sealed and checked across every app/DMG
  shape, with a resilient text fallback only for malformed/unbundled development runs. Future
  Windows/Linux trays must mimic this icon, TUI launch, same-user boundary, and graceful exit, but
  v1.0.3 does not implement them.
- Renderer: AppKit/CoreGraphics RGBA regressions pass. The reusable AppKit image-view presenter
  now produces opaque, black-rectangle-free WindowServer captures in normal motion and readable
  dark-mode notes. The product-equivalent signed candidate passed live capture and the exact final
  candidate artifacts independently passed their byte/channel/trust contracts.
- Shared gait: releases planted feet at four pixels, preserves the weighted normal/moderate
  cadence, caps visible lag at 16 px for Walk and 26 px for Run/Charge, and passes cadence guards,
  eight-direction tests, and seven goldens without overcorrecting ordinary walking.
- Movement/lifecycle: continuous monitor seams, one-in-five hidden wrapping, non-wrapping errands,
  fully offscreen entry/exit, and the user-only 30% annoyed reaction with separately gated nab pass
  their independent focused review. Final transparent clearing, gapped-monitor staging,
  permission/hot-plug and Stop ordering, typed close correlation, positive Windows provenance,
  bounded exit ownership, and offscreen reaction deferral are pinned by regressions. Exact native
  candidate observations remain open; Linux still has no collect trigger. A live Mac collect run
  exposed that body locomotion could target the prop while completion measured beak arrival,
  making a normal horizontal approach stall. The beak-offset source correction and focused
  realistic 120 Hz regression now pass. The product-equivalent signed candidate spawned and typed
  a readable dark-mode note; an exact visual beak-contact frame remains forward verification.
- Performance: bounded capture-safe surfaces reduced the post-transient regression from 40.45%
  to 12.00% median CPU during unlocked visible motion. The Device-RGB alpha-last bitmap now feeds
  a stable standard-sRGB window destination, leaving final display-profile composition to
  WindowServer. A subsequent active diagnostic measured 5.55% median CPU, 29.52 MiB maximum RSS,
  negative 9.89 MiB growth, zero leaks, and 20 clean compositor captures. This clears the local
  envelope diagnostically. Exact final-SHA repetition is recorded as a source-equivalent waiver;
  later hardware verification must not replace the accepted Mac presentation contracts.
- Lifecycle: mounted-bundle copy, aliases, autostart, release-bound receipt, preservation, purge
  backup, rollback, and isolated-home cleanup are implemented. Lifecycle mutation retains the
  singleton; Unix signals carry explicit rollback statuses; Windows verifies from pinned streams,
  has no ambient lease bypass, checks machine-wide paths across sessions, rejects reboot-deferred
  MSI completion, and reaps every deferred helper that has not completed its exact READY handoff.
  Native candidate transactions, final scoped cleanup, a fresh published v0.3.2-to-v1.0.1
  update, and the real public v1.0.1-to-v1.0.2 CLI update passed. All three aliases converge and
  repeat as PID/binary/receipt-stable no-ops. The completed `#v1a` pass records later Windows
  fault-injection and broader interaction repetition without changing the Mac contracts.
- Packaging/candidate: a universal x86_64/arm64 installer helper targeting macOS 11.0 in both
  slices and a fail-closed signing/notarization workflow are in-tree. Candidate run
  `29384134561` at source `39087949731f9a8326d0661182fa4a2dbe89c61b` passed the complete
  Developer ID/notarized/stapled Mac producer and Windows x64 compositor/lifecycle path. It failed
  closed before assembly because all four Linux jobs saw `xcompmgr`'s unchanged gray cached root
  tile, while Windows ARM64 captured neither the controlled background nor goose and then raced
  its shared color file. Current source uses a persistent test-only X11 background client,
  establishes PMv2 before either Windows smoke process creates an HWND, transfers colors through
  an atomic token/ack channel, and proves DPI/geometry and dark/light capture before goose launch.
  Status output also treats only `BrokenPipe` from a closed downstream consumer as success. Those
  source repairs and the new menu resource superseded `3908794`; the paragraph is historical, and
  later candidates below supersede its then-pending rerun. First v1.0.0 candidate
  `29386819926` at `bc3c1d9` then passed the complete trusted Mac producer, both Apple/Windows
  portable pairs, ARM64 Windows installers, and every repaired X11 compositor half. It failed
  closed because evidence-derived Wayland socket paths exceeded Linux's 108-byte AF_UNIX limit
  and the Windows x64 controller treated a matching CRLF geometry document as one line. Current
  source uses a short cleaned Wayland runtime directory and exact CRLF/LF line parsing. Candidate
  `29387569722` at `5d0237fdf23df0abaafaaef74d43cc6acfcd870d` then passed the trusted Mac
  producer, every Apple/Windows portable job, Windows x64 paired-DWM compositor/lifecycle, native
  ARM64 PE/MSI lifecycle, and X11 plus dual-output Wayland on three of four Linux variants. Its
  two remaining blockers were evidence-specific: one valid top-down x64 GNU X11 pose had 13 warm
  pixels against a side-view floor of 20, and GitHub's hosted ARM64 capture API returned one
  byte-identical static wallpaper for both acknowledged visible-window colors. Current source
  sets the Linux warm floor to 10 while retaining all body/wing/background/transparency checks.
  ADR 0026 restricts the ARM64 fallback to the exact GitHub-hosted wallpaper signature and records
  the cropped premultiplied-BGRA DIB only after a successful native present, bound to the frozen
  visible HWND/rectangle; local/self-hosted ARM64 and x64 still require paired DWM. Candidate
  `29389046641` at `414c447077910d8fd05ccdb7c5a5e7cea530c087` passed the complete trusted Mac
  producer, all four Linux X11/Wayland qualifications, both Debian native-package jobs, every
  portable producer, and Windows x64 paired-DWM/lifecycle. Both hosted ARM64 jobs produced several
  exact presenter surfaces that passed every alpha, palette, articulation, and shadow assertion,
  but exact rectangle-string comparison rejected them after the goose advanced one or two pixels
  between atomic record completion and suspension. Current source retains exact HWND binding and
  all semantic checks, polls at five milliseconds, and allows only a three-physical-pixel
  origin/dimension delta for that one presentation interval. That repair required another exact
  candidate. Exact candidate `29389882143` at
  `c44b89d35abb6b30fca5a48064334a79bfcb3839` then passed the complete Mac trust producer, every
  portable target, Windows x64 paired-DWM, both native ARM64 compositor paths, all four Linux
  compositor paths, both Debian native-package jobs, and final candidate assembly. That SHA was
  fast-forwarded to `main`; ordinary CI passed audit/contracts, Windows x64/ARM64, and both Mac
  bundle jobs, but its two Linux jobs inherited Ubuntu's recommended default Sway wallpaper/bar on
  one headless output. Honk300 remained transparent and exposed that backdrop while the wildcard
  update left the other output correctly solid. Current source gives the smoke an isolated minimal
  Sway config, exact per-output colors, an explicit `swaybg` dependency, and paired goose-free
  baselines before launch. Candidate `29391420738` passed that complete release path, but its
  same-SHA Ubuntu 24.04 main run proved Noble's one-pixel swaybg solid-color buffer was linearly
  filtered into a gradient on the 1.5-scale pixman output before Honk300 launched. Current source
  starts with no background rule and tiles constant opaque PNGs only on discovered exact output
  names, preserving fractional filtering. Exact candidate `29392439475` and same-SHA ordinary CI
  `29392827146` then passed at `9c5692b32bb256d3008308c83d76ddebd7fb44df`. The immutable
  `v1.0.0` release run `29398343807` rebuilt every producer but failed before draft creation only
  because Windows x64 sampled complete top-down poses against a side-only beak/legs/shadow oracle.
  Every other producer passed, including the signed/notarized/stapled Mac app/DMG and both native
  Debian packages. ADR 0027 keeps that tag immutable, advances the first public target to v1.0.1,
  and adds strict complete side/top-down profiles plus reconstructed edge-color proof. Product
  renderer/engine/presenter output is unchanged.
- Integrated final-source gate: formatting, strict workspace clippy, 432 Rust tests including all
  seven renderer goldens, release and both Apple builds, cargo-dist planning, 95 Python contracts,
  actionlint, shell/PowerShell syntax, pinned cargo-audit over 374 dependencies and 1,160
  advisories, Windows x64/ARM64 strict cross-clippy, Linux x64/ARM64 GNU/musl engine/backend
  checks, and diff validation passed at the exact v1.0.0 SHA. The v1.0.1 fix-forward reruns only
  affected identity/analyzer contracts locally, then repeats the complete hosted candidate matrix.
- Focused v1.0.1 gate: formatting, 107 root tests, 22 native Mac platform tests, optimized
  `honk300 1.0.1`, cargo-dist v1.0.1 planning, 103 Python packaging/workflow contracts,
  retained Windows replay, Python/PowerShell syntax, actionlint, credential scan, and diff
  validation pass. Partial entrance attempts 3–5 reject and complete top-down attempts 6–12 pass.
- Focused v1.0.2 source gate: formatting, strict workspace Clippy, 406 unit tests and seven
  renderer goldens, optimized `honk300 1.0.2`, cargo-dist v1.0.2 planning, 104 Python packaging/
  workflow contracts, pinned audit, actionlint, Python/shell/PowerShell parsing, secret scan, and
  diff validation pass. Both Windows targets cross-check from macOS; all four Linux platform
  targets cross-check, while complete Linux audio/root-binary qualification correctly remains on
  native hosted runners because this Mac has no Linux ALSA sysroot.
- First v1.0.2 candidate `29564261409` failed closed before tagging only in the Windows x64 exact
  overlay smoke: the server applied `Wander`, but a connected `PIPE_NOWAIT` zero-byte poll was
  interpreted as an empty command before the client frame became readable. Mac Developer ID
  signing/notarization/stapling and every other completed producer passed. The bounded Windows
  reader now retries that transient state through the existing deadline and retains a hard
  timeout for peers that never send bytes. Replacement candidate `29565557915`, same-SHA CI
  `29566294408`, atomic release `29566759574`, and post-release smoke `29567257622` then passed
  the complete matrix at `964305869e9ec28768c789465db1b6317dfa3f6f`.
- Update acceptance: all three v1.0.1 aliases first proved PID/binary/receipt-stable latest no-ops.
  After publication, the preserved public v1.0.1 managed app used the real atomic latest manifest
  to converge to the fresh public v1.0.2 app ZIP. Version, exact receipt, aliases, universal G2
  signature, notarization staple, Gatekeeper assessment, immediate restart, and subsequent no-op
  updates through `goose`, `honk`, and `honk300` all passed before final purge.
- Historical v1.0.0 retarget gate: the affected identity paths passed 107 root tests, 22 native
  Mac platform tests, a release binary reporting 1.0.0, cargo-dist v1.0.0 planning, 95 packaging/
  workflow contracts, workflow and script syntax, local link/tree validation, and the live board
  check. The focused v1.0.1 gate rechecks every changed identity/analyzer path, while candidate
  mode freshly rebuilds the complete native matrix on the frozen v1.0.1 SHA.
- Debian/update: deterministic `honk300-amd64.deb` and `honk300-arm64.deb` packaging reuses the
  exact qualified GNU executables. Package-owned paths, aliases, marker, metadata, platform-kind
  isolation, `dpkg` ownership/elevation, update, preserve-on-uninstall, backup-on-purge, assembly
  evidence, and native amd64/arm64 published smokes are implemented. Stable latest discovery now
  resolves exact immutable tag bytes for every platform and provenance; the DMG is rebuilt by
  GitHub for every release but Mac CLI updates consume the exact-tag app ZIP through the pinned
  bootstrap.
- Release/site: progressive-disclosure site commit
  `aac97943367e767d3de0afdf2c41f1c5002d98fb` passed hosted CI `29559081404` and production
  deployment `5484883674`. Public `thegoose.app` now shows one OS-appropriate recommended path,
  keeps terminal and alternate downloads collapsed, reports verified v1.0.2 from the uncached
  live manifest, and points the primary Mac action at the exact immutable v1.0.2 notarized DMG.
  Live desktop/mobile browser checks found no horizontal overflow, preserved keyboard disclosure,
  and reported zero Axe violations.

## Goals

1. Preserve a platform-free 120 Hz simulation engine and shared procedural renderer.
2. Present correct premultiplied-alpha output through each native desktop backend.
3. Keep all CLI/TUI/IPC control local, single-instance, and owner-scoped; let the macOS status
   item call only the existing TUI and graceful-stop path.
4. Never manipulate terminal windows, even when visual overlays may cover them.
5. Keep install/update/uninstall transactional and preserve foreign files and user media.
6. Publish immutable, complete, machine-verifiable releases with no crates.io distribution.
7. Make the signed, notarized, stapled universal DMG the primary macOS download only after a
   fresh published artifact passes independent checks.
8. Keep multi-monitor seams natural and permit any edge relocation only while the full pose is
   outside real monitor pixels; enter and exit through locomotion rather than visible pops.
9. Treat a user-close annoyed reaction as character only: program cleanup is excluded and any
   cursor nab remains bounded by existing settings, manners, permission, and capability.
10. Publish stable unversioned latest installer URLs backed only by complete immutable tagged
    releases; update only through exact-tag, hash/size/target/provenance-matched artifacts.
11. Offer native amd64/arm64 Debian packages with real package-manager ownership while preserving
    the per-user no-sudo shell path for other Linux users.
12. Give graphical macOS users a visible Configure/Quit path without adding a native settings
    model or abrupt termination; keep its shared icon and behavior reusable for later qualified
    Windows/Linux trays.

## Architecture

- `crates/honk-engine`: unsafe-free fixed-step simulation, tasks, rig, feet, behaviors, and
  procedural raster renderer. No OS dependencies.
- `crates/honk-control`: closed local IPC protocol and same-user transport.
- `crates/honk-config` / `honk-config-tui`: schema-v2 TOML and ratatui control surface.
- `crates/honk-platform-*`: native Windows, capture-safe AppKit/CoreGraphics, X11, and Wayland
  adapters.
- `src/runtime`: platform event loops built around shared `RuntimeCore` ordering.
- `src/install.rs` / `src/update.rs` / `src/debian.rs`: ownership receipts, atomic lifecycle
  transactions, platform/provenance-aware updates, Debian package ownership, and foreign-file
  preservation.
- `.github/workflows/release.yml`: candidate-first and atomic immutable release orchestration.
- `.github/workflows/macos-packaging.yml`: universal app/helper signing, app+DMG notarization,
  stapling, validation, and evidence.
- `Assets/UI`: canonical shared goose control-surface source, AppKit runtime representation, and
  provenance/export/accessibility guidance.
- `packaging/macos`: native graphical helper source, signed terminal-TUI launcher, and DMG
  instructions.
- `docs/adr`: durable decisions. ADR 0020 is current for macOS distribution; ADR 0018 remains
  current for atomic publication, ADR 0023 governs rolling latest and Debian lifecycle, and ADR
  0024 governs the macOS menu-bar bridge. ADR 0025 records the first stable v1 intent and
  post-release Alienware verification boundary. ADR 0026 defines the narrow GitHub-hosted Windows
  ARM64 presenter-evidence path without claiming full DWM composition. ADR 0027 records the
  immutable v1.0.0 failure, v1.0.1 fix-forward, and strict side/top-down evidence profiles. ADR
  0028 defines the shared icon and future tray Configure/TUI plus graceful-Quit parity contract.
  ADR 0029 defines the Windows updater/lifecycle, Corporate retry, and integrated-terminal
  hardening found on the Alienware.

## Verification Source Of Truth

- Completed v1.0.3 release evidence: `docs/readiness/v1.0.3-readiness.md`; v1.0.2 and v1.0.1
  readiness reports remain immutable-release history.
- Native historical/backend evidence: `docs/readiness/m16-m18-readiness.md`.
- Board handoff and activity: `.tasks/tasks/v102.md` and `.tasks/tasks/v1a.md`.
- Canonical product plan: `honk300_plan.md`.
- Required local gate: fmt, workspace clippy with warnings denied, workspace tests, release build,
  universal Apple builds, `dist plan --tag=v1.0.3`, complete Python contracts, cargo-audit
  0.22.2, actionlint, and diff check.

## Current Workspace Tree

Generated 2026-07-17. Build output, Git internals, worktree internals, private credential storage,
Python bytecode caches, and
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
│   ├── .board-version.json
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
│   │   ├── v103.md
│   │   └── v110.md
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
│   │   ├── r5d.md
│   │   ├── ltray.md
│   │   ├── r103.md
│   │   ├── r110.md
│   │   ├── trayc.md
│   │   ├── v102.md
│   │   ├── v1a.md
│   │   └── wtray.md
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
│   ├── UI
│   │   ├── README.md
│   │   ├── honk300-status-goose.svg
│   │   └── honk300-status-goose@2x.png
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
│   │   ├── 0023-rolling-latest-artifacts-and-debian-lifecycle.md
│   │   ├── 0024-macos-menu-bar-control.md
│   │   ├── 0025-first-stable-v1-release.md
│   │   ├── 0026-hosted-windows-arm64-compositor-evidence.md
│   │   ├── 0027-v1-0-1-fix-forward-and-windows-pose-evidence.md
│   │   ├── 0028-shared-goose-control-surface-and-tray-parity.md
│   │   ├── 0029-windows-lifecycle-and-terminal-hardening.md
│   │   └── README.md
│   ├── agents
│   │   └── handoff
│   │       ├── 2026-07-13-001-macos-v0-3-3-qualification-release.md
│   │       ├── 2026-07-14-001-macos-v0-3-3-alienware-resume.md
│   │       ├── 2026-07-14-002-alienware-post-v1-verification.md
│   │       ├── 2026-07-15-001-alienware-post-v1.0.1-verification.md
│   │       └── 2026-07-17-001-alienware-post-v1.0.2-verification.md
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
│   │   ├── v1.0.1-readiness.md
│   │   ├── v1.0.2-readiness.md
│   │   └── v1.0.3-readiness.md
│   ├── research
│   │   └── native-wayland-capability-path.md
│   ├── superpowers
│   │   ├── plans
│   │   │   └── 2026-07-13-macos-accessibility-first-run.md
│   │   └── specs
│   │       └── 2026-07-13-macos-accessibility-first-run-design.md
│   └── thinking
│       ├── 2026-06-27-m9-collect-window-plan.md
│       ├── 2026-07-17-alienware-v1.0.2-verification.md
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
│       ├── Configure Honk300.command
│       ├── DMG-README.txt
│       └── InstallHonk300
│           └── main.swift
├── rust-toolchain.toml
├── script
│   ├── analyze_linux_overlay_capture.py
│   ├── analyze_windows_overlay_capture.py
│   ├── honk300-installer.ps1.in
│   ├── honk300-installer.sh.in
│   ├── package_deb.py
│   ├── package_macos_app.sh
│   ├── package_macos_installer_helper.sh
│   ├── release_metadata.py
│   ├── smoke_m16_macos.sh
│   ├── smoke_m16_macos_accessibility.sh
│   ├── smoke_m17_m18_linux.sh
│   ├── smoke_released_deb.sh
│   ├── smoke_released_unix.sh
│   ├── smoke_released_windows.ps1
│   ├── smoke_windows_overlay.ps1
│   ├── tests
│   │   ├── test_debian_packaging.py
│   │   ├── test_installer_templates.py
│   │   ├── test_linux_overlay_smoke.py
│   │   ├── test_macos_packaging.py
│   │   ├── test_macos_smoke_contract.py
│   │   ├── test_post_release_smoke.py
│   │   ├── test_release_metadata.py
│   │   ├── test_release_workflows.py
│   │   ├── test_verify_binary_architecture.py
│   │   ├── test_windows_overlay_smoke.py
│   │   └── test_windows_packaging.py
│   ├── verify_binary_architecture.py
│   └── x11_smoke_background.py
├── security-audit
│   └── 2026-07-10-1635-diff-review.md
├── src
│   ├── assets.rs
│   ├── audio.rs
│   ├── cli.rs
│   ├── debian.rs
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

79 directories, 292 files
```
