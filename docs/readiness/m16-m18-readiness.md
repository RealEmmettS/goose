# M16-M18 Backend Readiness

Date: 2026-07-01

## Status

M16-M18 implementation work is closed to the best evidence available from the Windows host. The
remaining platform-host observations are split into repeatable smoke scripts and follow-up
readiness work instead of being represented as completed local evidence.

## Completed Evidence

- M16 macOS implementation is in-tree: `honk-platform-macos`, macOS runtime wiring, AppKit overlay
  surfaces, Accessibility-gated cursor/window behavior, macOS collect windows, `honk300 status`,
  TUI Status, bundle-aware assets/start, and `script/package_macos_app.sh`.
- M17/M18 Linux control-runtime foundation is in-tree: `honk-platform-linux`, Linux `start`,
  Unix IPC status/reload/stop/poke, X11-first vs. forced/native Wayland detection, terminal
  classifier, local-time sampling, command-player audio, and explicit unsupported/failed status
  for desktop-control capabilities that are not implemented yet.
- `honk-engine` remains platform-free; the backend crates own OS-specific code.
- Capability state flows through `BackendCapability` and the compact status protocol instead of
  compile-time assumptions inside engine tasks.
- Terminal classifiers exist for Windows, macOS, and Linux before backend code can emit
  foreign-window or collect-window targets.
- Cross-target checks passed for:
  - `x86_64-pc-windows-msvc`
  - `aarch64-pc-windows-msvc`
  - `x86_64-apple-darwin`
  - `aarch64-apple-darwin`
  - `x86_64-unknown-linux-gnu`
  - `aarch64-unknown-linux-gnu`
  - `x86_64-unknown-linux-musl`
  - `aarch64-unknown-linux-musl`

## Repeatable Host Smoke

- `script/smoke_m16_macos.sh` builds and validates the universal2 app, launches the LSUIElement
  bundle, checks status, and exercises honk/mud/reload/stop IPC on a macOS host.
- `script/smoke_m17_m18_linux.sh` builds the Linux binary, runs default and forced-Wayland
  degraded modes, checks status, confirms unsupported nab rejection, and exercises
  honk/mud/wander/reload/stop IPC on a Linux host.

## Split Follow-Up Evidence

- `#m16r` macOS host manual smoke: grant Accessibility, verify cursor nab, foreign-window ride,
  collect note/meme, terminal non-targeting, status/reload/stop, audio, multi-monitor behavior,
  and Intel/Apple Silicon runtime evidence.
- `#m17r` Linux X11 host work: visible transparent overlay, input shaping/click-through, pointer and
  window support where X11 allows it, terminal target filtering against live X11 metadata, and
  x64/ARM GNU/musl runtime evidence.
- `#m18r` Linux Wayland host work: visible reduced-mode layer-shell overlay, IPC stop/poke/reload/status,
  unsupported cursor/window/keystroke behavior, and compositor-specific notes where needed.

## Verification Commands

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
