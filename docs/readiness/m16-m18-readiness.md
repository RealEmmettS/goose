# M16.1-M18.1 Backend Readiness

Date: 2026-07-01

## Status

M16-M18 implementation is in-tree. M16.1-M18.1 readiness is now a CI-proven gate, not a
Windows-host claim. The local Windows host can prove formatting, tests, release build, and
cross-target compilation; GitHub-hosted and optional self-hosted runners provide the macOS/Linux
runtime evidence required before `#m16r`, `#m17r`, and `#m18r` can move to Done.

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

## Open Readiness Evidence

- `#m16r` can close only after hosted macOS bundle/status smoke has passed and Accessibility
  granted behavior is proven on a self-hosted/pre-granted or manual macOS run. Hosted macOS can
  prove denied/degraded behavior, but it cannot grant durable Accessibility permission.
- `#m17r` can close after Linux x64 and ARM hosted X11 smoke passes and the CI run URL/artifact
  names are recorded.
- `#m18r` can close after Linux x64 and ARM hosted Wayland reduced-mode smoke passes and the CI
  run URL/artifact names are recorded.

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

- Pending push: record GitHub Actions run URL, macOS app artifact names, Linux smoke job names,
  and optional macOS Accessibility run URL here before moving readiness tasks to Done.
