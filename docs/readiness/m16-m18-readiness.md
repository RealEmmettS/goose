# M16.1-M18.1 Backend Readiness

Last updated: 2026-07-13

## Status

M16-M18 implementation is in-tree. M16.1-M18.1 readiness is a CI-plus-native-evidence gate, not a
cross-target claim. GitHub-hosted runners provide Linux X11/Wayland evidence for `#m17r` and
`#m18r`; hosted Intel and Apple Silicon runners provide the ordinary macOS bundle gate. Managed
macOS Accessibility first-run onboarding is implemented and its focused automated tests pass. On
2026-07-12, one unchanged Developer ID-signed app on a physical M2 also passed the earlier denied
and operator-granted command/status smoke. `#m16r` remains open until the exact signed release
candidate records the new first-denied prompt/wait, non-nagging denied relaunch, live grant, and
live revocation sequence against one recorded binary digest, plus the remaining terminal/one-
display observations.

## Implemented Evidence

- ADR 0022's macOS onboarding is in-tree: exact managed-app/receipt eligibility, an owner-only
  per-update prompt marker created before native UI, main-thread consent and Settings bridges, a
  platform-neutral calm safe-edge wait, one-second permission polling, same-process FirstUX
  resume after grant, safe re-entry after revocation, and non-prompting unmanaged launches.
- M16 macOS implementation is in-tree: `honk-platform-macos`, macOS runtime wiring, AppKit
  overlay surfaces with an alpha-last Device-RGB bitmap and stable standard-sRGB window
  destination, Accessibility-gated cursor/window behavior, macOS collect windows,
  `honk300 status`, TUI Status, bundle-aware assets/start, and `script/package_macos_app.sh`.
- M17 Linux X11 implementation is in-tree: X11/XWayland session selection, visible transparent
  overlay, XShape/XFixes input-region shaping, Xinerama/root bounds, pointer sampling, cursor
  warp, terminal-filtered foreign-window drag snapshots, Unix IPC status/reload/stop/poke,
  Linux terminal classification, local-time sampling, in-process GNU audio, a bounded and reaped
  PATH-player fallback for musl, and explicit unsupported/failed capability reporting.
- M18 native Wayland reduced mode is in-tree: layer-shell overlay presentation through
  smithay-client-toolkit, one surface per output, real prepare-read/poll/read/dispatch handling,
  output configure/close/hotplug reconciliation, integer and fractional scale support, empty input
  regions, no keyboard interactivity, and a released-buffer pool capped at three buffers per
  output. Unix IPC status/reload/stop/poke and direct honk/mud/wander control remain available;
  cursor warp, foreign-window control, collect-window behavior, and synthetic input report
  explicitly unsupported.
- `honk-engine` remains platform-free; backend crates own OS, display-server, permission, and
  presentation behavior.
- Capability state flows through `BackendCapability` and the compact status protocol instead of
  compile-time assumptions inside engine tasks.
- Linux collect-window support remains unsupported in M17.1 and is reported honestly.

## CI Proof Path

- `.github/workflows/ci.yml` runs:
  - Windows hosted gate: format, workspace tests, clippy, release build, Windows x64/ARM64
    target checks.
  - macOS hosted gate on `macos-15` and `macos-15-intel`: workspace tests,
    `script/smoke_m16_macos.sh`, universal2 `.app` artifact upload, bundle id
    `dev.emmetts.honk300`, `LSUIElement=true`, `plutil`, `codesign`, and `lipo`.
  - Optional macOS Accessibility gate on `[self-hosted, macOS, ARM64, honk300-a11y]` when
    `HONK300_RUN_A11Y_SMOKE=true`: `script/smoke_m16_macos_accessibility.sh`.
  - Linux hosted gate on `ubuntu-latest` and `ubuntu-24.04-arm`: workspace tests, one exact
    prebuilt `HONK300_BIN` through `script/smoke_m17_m18_linux.sh`, X11 root and Sway `grim`
    compositor captures, GNU target checks, and musl target checks.
- GitHub-hosted runner labels used here match GitHub's current runner table for
  `windows-latest`, `macos-15`, `macos-15-intel`, `ubuntu-latest`, and `ubuntu-24.04-arm`.
- The self-hosted Accessibility job uses cumulative labels so it only runs on a runner with all
  required default/custom labels.

## Repeatable Smoke Scripts

- `script/smoke_m16_macos.sh` builds and validates the universal2 app, launches the LSUIElement
  bundle, checks status, verifies bundle metadata, and exercises honk/mud/reload/stop IPC.
- `script/smoke_m16_macos_accessibility.sh` supports the exact prebuilt app and separate denied,
  live, and pre-granted phases. Its live phase verifies the owner-only marker, calm denied status,
  reload/honk plus `BUSY` prank rejection, a second denied launch without marker rewrite,
  same-process grant, live revocation back to denied wait, unchanged signed-binary/marker
  fingerprints, operator-observed UI state, and opt-in scoped cleanup. The exact candidate still
  needs that four-state native run.
- `script/smoke_m17_m18_linux.sh` accepts an exact `HONK300_BIN` and does not rebuild it. It runs
  a visible X11 overlay under Xvfb/openbox/xcompmgr, checks status, rejects any direct visual whose
  byte order, masks, shifts, bpp, or scanline layout is not the expected little-endian ARGB8888
  ZPixmap contract, captures the actual root window, and proves the controlled background remains
  visible outside the overlay. It exercises honk/mud/wander/nab/reload/stop IPC, then starts
  headless Sway with two virtual outputs at 1.5x and 2x scale. `grim` captures the real composed
  Wayland desktop and requires recognizable body, wing, asymmetric warm beak/legs, and background
  while unsupported mischief remains explicit. Sway is intentional because the runtime requires
  wlr layer-shell; Weston does not provide it.

## Readiness Evidence State

- `#m16r` remains open. Hosted macOS bundle/status smoke passed on both arm64 and Intel hosted
  runners, but hosted jobs cannot grant durable Accessibility permission or prove native prompt
  behavior. The onboarding engine, eligibility, secure marker, native bridge, transition, and
  smoke-script contracts are green. The remaining gap is the exact signed candidate's first
  denied prompt and safe-edge observation, non-nagging denied relaunch, live grant without
  restart, live revocation back to the wait, ordinary-window positive behavior, and protected-
  terminal negatives. The native run requires explicit operator confirmations for prompt/no-
  prompt state, timed transitions, the same signed binary digest throughout, and opt-in cleanup.
  Record that evidence here before closing `#m16r`.
- `#m17r` is closed from its original readiness scope. Linux x64 and ARM hosted X11 visible smoke passed, including
  internal frame proof, root screenshot proof, IPC status/reload/stop/poke, terminal-filter
  fixture coverage, and GNU/musl target checks. v0.3.3 adds a stricter exact-binary visual-layout
  gate whose first x64/ARM64 candidate executions are still pending and must not be inferred from
  the older closure.
- `#m18r` is closed from its original readiness scope. Linux x64 and ARM hosted Wayland reduced-mode smoke passed under
  headless sway, including visible frame proof, IPC status/reload/stop/poke, and explicit
  unsupported mischief status. The new semantic `grim` compositor assertion likewise remains a
  v0.3.3 native-candidate gate until both hosted architectures run it.

## Physical M2 macOS Evidence (2026-07-12)

- A universal app and graphical helper were signed explicitly inside-out with the current G2
  Developer ID Application identity for team `M9D5379H93`; both slices, hardened runtime,
  timestamp, chain, designated requirement, and strict verification passed.
- The exact signed executable first passed the denied Accessibility smoke. After the operator
  enabled that unchanged Honk300 row in System Settings, status reported `accessibility:
  supported`, `cursor: supported`, and `window: supported`.
- The granted smoke passed honk, mud, reload, nab, meme, note, and stop. A singleton-release fix
  found during that run then passed 60 immediate stop/start cycles on a freshly resealed app.
- Real Dark appearance note captures now show appearance-aware, high-contrast text. Real light
  and dark overlay captures show the complete opaque body, outline, wing, beak, legs, and shadow.
- A later standard-sRGB window diagnostic measured 5.55% median CPU, 29.52 MiB maximum RSS,
  negative 9.89 MiB growth, zero leaks, and 20 clean compositor captures. It validates the local
  direction but does not replace the byte-exact signed-candidate capture/profile.
- This Mac has one attached display. Live multi-monitor/hot-plug behavior is therefore explicitly
  hardware-waived; signed-coordinate, topology, per-display-window, and hot-plug reconciliation
  tests remain mandatory and pass. No claim of live multi-monitor validation is made.
- Terminal.app, Codex, and Visual Studio Code identities were observed on this host and match the
  fail-closed application classifier. Ghostty was not installed. The exact final candidate still
  needs the recorded ordinary-window positive and protected-terminal negative observation before
  `#m16r` closes.
- This preliminary run predates ADR 0022's managed first-run flow. It does not prove the native
  prompt appeared once, the second denied launch did not nag, FirstUX resumed from the wait, or a
  live revocation returned the goose to that wait.

## Local Verification Commands

```powershell
cargo fmt --all -- --check
cargo test --workspace
cargo clippy --all-targets --workspace -- -D warnings
cargo build --release
$targets = @(
  'x86_64-pc-windows-msvc',
  'aarch64-pc-windows-msvc',
  'x86_64-apple-darwin',
  'aarch64-apple-darwin',
  'x86_64-unknown-linux-gnu',
  'aarch64-unknown-linux-gnu',
  'x86_64-unknown-linux-musl',
  'aarch64-unknown-linux-musl'
)
foreach ($target in $targets) {
  cargo check --workspace --target $target
  if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
}
```

## CI Evidence Log

- 2026-07-14 - Mac stopping-tree gate passed all 389 workspace tests, 22 native Mac platform
  tests, strict clippy, both Apple release builds, 71 packaging/workflow contracts, actionlint,
  shell/PowerShell validation, cargo-dist planning, cargo-audit, and cross-target checks. The host
  is clean/uninstalled. No candidate was dispatched, so ADR 0022's four-state exact-candidate,
  terminal, capture, profile, and lifecycle evidence remains open and `#m16r` stays Active.
- 2026-07-13 - strengthened the Linux x64/ARM64 smoke contracts so one exact binary is retained,
  X11 byte layout fails closed, and `grim` checks the real two-output fractional-scale Sway
  composition for body/wing/asymmetric warm pixels. Focused local contracts pass; first hosted
  execution of this stronger v0.3.3 gate remains pending.
- 2026-07-13 - focused automated onboarding gates passed: five engine permission-wait tests,
  thirteen managed-install/marker/transition tests, all 17 macOS platform tests (including bundle
  metadata, prompt-thread, Settings failure, and hot-plug interactivity contracts), and five
  smoke-script contract tests. No
  exact signed-app denied/non-nag/grant/revoke smoke was run by this documentation pass; that
  four-state native evidence remains pending.
- 2026-07-12 - physical M2 preliminary Accessibility evidence passed on one unchanged current G2
  Developer ID-signed app: denied smoke first, operator grant, supported accessibility/cursor/
  window status, then honk/mud/reload/nab/meme/note/stop and 60 stop/start cycles. This is native
  pre-candidate evidence; the immutable release candidate must repeat it before closure.
- 2026-07-02 - GitHub Actions run
  <https://github.com/RealEmmettS/goose/actions/runs/28569332035> completed successfully for
  hosted Windows/macOS/Linux readiness.
  - Windows host gate: passed fmt, workspace tests, clippy, release build, and Windows x64/ARM64
    target checks.
  - macOS bundle smoke (`macos-15` arm64): passed workspace tests, universal2 app packaging,
    `plutil`, `codesign`, `lipo -verify_arch x86_64 arm64`, bundle launch/status/IPC smoke, and
    artifact upload.
  - macOS bundle smoke (`macos-15-intel`): passed the same hosted bundle/status gate and artifact
    upload on Intel-hosted macOS.
  - macOS app artifacts: `honk300-macos-macos-15` and `honk300-macos-macos-15-intel`.
  - Linux visible smoke (`ubuntu-latest`): passed X11 visible overlay smoke, Wayland reduced-mode
    smoke, workspace tests, and Linux x64 GNU/musl target checks.
  - Linux visible smoke (`ubuntu-24.04-arm`): passed X11 visible overlay smoke, Wayland
    reduced-mode smoke, workspace tests, and Linux ARM GNU/musl target checks.
  - macOS Accessibility smoke:
    <https://github.com/RealEmmettS/goose/actions/runs/28569332035/job/84703318760> was skipped,
    so Accessibility-granted cursor/window/collect evidence is still missing and `#m16r` stays
    open.
