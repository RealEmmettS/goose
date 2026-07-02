# M16.1-M18.1 Backend Readiness

Date: 2026-07-01

## Status

M16-M18 implementation is in-tree. M16.1-M18.1 readiness is now a CI-proven gate, not a
Windows-host claim. The local Windows host can prove formatting, tests, release build, and
cross-target compilation; GitHub-hosted runners have now provided the Linux X11/Wayland evidence
for `#m17r` and `#m18r`, while `#m16r` still requires optional self-hosted or manual macOS
Accessibility-granted evidence.

## Implemented Evidence

- M16 macOS implementation is in-tree: `honk-platform-macos`, macOS runtime wiring, AppKit
  overlay surfaces, Accessibility-gated cursor/window behavior, macOS collect windows,
  `honk300 status`, TUI Status, bundle-aware assets/start, and `script/package_macos_app.sh`.
- M17 Linux X11 implementation is in-tree: X11/XWayland session selection, visible transparent
  overlay, XShape/XFixes input-region shaping, Xinerama/root bounds, pointer sampling, cursor
  warp, terminal-filtered foreign-window drag snapshots, Unix IPC status/reload/stop/poke,
  Linux terminal classification, local-time sampling, command-player audio, and explicit
  unsupported/failed capability reporting.
- M18 native Wayland reduced mode is in-tree: layer-shell overlay presentation through
  smithay-client-toolkit, Unix IPC status/reload/stop/poke, direct honk/mud/wander control, and
  explicit unsupported status for cursor warp, foreign-window control, collect-window behavior,
  and synthetic input.
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
  - Linux hosted gate on `ubuntu-latest` and `ubuntu-24.04-arm`: workspace tests,
    `script/smoke_m17_m18_linux.sh`, GNU target checks, and musl target checks.
- GitHub-hosted runner labels used here match GitHub's current runner table for
  `windows-latest`, `macos-15`, `macos-15-intel`, `ubuntu-latest`, and `ubuntu-24.04-arm`.
- The self-hosted Accessibility job uses cumulative labels so it only runs on a runner with all
  required default/custom labels.

## Repeatable Smoke Scripts

- `script/smoke_m16_macos.sh` builds and validates the universal2 app, launches the LSUIElement
  bundle, checks status, verifies bundle metadata, and exercises honk/mud/reload/stop IPC.
- `script/smoke_m16_macos_accessibility.sh` is the pre-granted Accessibility smoke for cursor
  nab plus note/meme command paths on macOS.
- `script/smoke_m17_m18_linux.sh` builds the Linux binary, runs a visible X11 overlay under
  Xvfb/openbox/xcompmgr, checks status, captures an internal PNG smoke frame, verifies non-zero
  alpha pixels, captures the actual X11 root window, verifies the known background color remains
  visible outside the overlay, exercises honk/mud/wander/nab/reload/stop IPC, then starts
  headless sway and verifies native Wayland reduced mode renders while mischief remains
  unsupported.

## Readiness Evidence State

- `#m16r` remains open. Hosted macOS bundle/status smoke passed on both arm64 and Intel hosted
  runners, but the Accessibility-granted smoke job was skipped because the self-hosted
  pre-granted runner gate was not enabled. Hosted macOS proves app bundle/status/IPC and
  denied/degraded behavior; it cannot grant durable Accessibility permission.
- `#m17r` is closed. Linux x64 and ARM hosted X11 visible smoke passed, including
  internal frame proof, root screenshot proof, IPC status/reload/stop/poke, terminal-filter
  fixture coverage, and GNU/musl target checks.
- `#m18r` is closed. Linux x64 and ARM hosted Wayland reduced-mode smoke passed under
  headless sway, including visible frame proof, IPC status/reload/stop/poke, and explicit
  unsupported mischief status.

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
