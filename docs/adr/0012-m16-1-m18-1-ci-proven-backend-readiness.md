# 0012 - M16.1-M18.1 CI-Proven Backend Readiness

## Status

Accepted.

## Context

M16 added the macOS AppKit backend, agent bundle staging, and status protocol. ADR 0011 added the
Linux control-runtime foundation and the honest degraded Wayland contract. The remaining gap is
evidence: macOS bundle/runtime behavior and Linux visible overlay behavior cannot be honestly
closed from a Windows host alone.

The project needs a repeatable completion gate that covers every advertised OS and architecture
axis without moving platform-specific behavior back into `honk-engine`.

## Decision

- Add GitHub Actions CI as the M16.1-M18.1 completion proof path. Hosted CI covers Windows,
  macOS Intel, macOS arm64, Linux x64, and Linux arm64 jobs; a separate opt-in self-hosted macOS
  Accessibility job covers granted Accessibility behavior when such a runner exists.
- Keep `honk-engine` OS-free. X11 and Wayland presentation details live in
  `honk-platform-linux`; macOS AppKit/CoreGraphics details remain in `honk-platform-macos`;
  Windows desktop details remain in `honk-platform-windows`.
- Implement Linux X11 as the default visible Linux path with an always-on-top transparent overlay,
  XShape/XFixes input shaping, Xinerama/root bounds, pointer sampling, cursor warp, and
  terminal-filtered foreign-window drag snapshots.
- Implement native Wayland as a reduced layer-shell overlay. It renders and remains controllable
  over IPC, but cursor warp, synthetic input, foreign-window control, and collect-window behavior
  stay unsupported by design.
- Treat CI artifacts and logs as the readiness evidence. The readiness cards stay open until the
  relevant jobs have passed and their run URL/artifact names are recorded.
- Keep the macOS Accessibility-granted path separate from hosted CI. Hosted macOS can validate the
  bundle, ad-hoc signing, lipo, status, IPC, and denied/degraded behavior; cursor nab,
  foreign-window ride, collect note/meme, and terminal non-targeting under granted Accessibility
  require a pre-granted self-hosted macOS runner or manual evidence.

## Consequences

- `script/smoke_m16_macos.sh` is the hosted macOS smoke gate for the universal2 app, bundle id
  `dev.emmetts.honk300`, `LSUIElement=true`, `plutil`, `codesign`, `lipo`, bundled launch,
  status, and IPC.
- `script/smoke_m16_macos_accessibility.sh` is the optional self-hosted macOS Accessibility gate.
- `script/smoke_m17_m18_linux.sh` is the Linux hosted smoke gate. It runs X11 under Xvfb with a
  window manager, validates visible alpha pixels from the overlay, exercises status/reload/stop
  and direct pokes, then runs forced Wayland reduced mode under headless sway and verifies
  unsupported mischief remains unsupported.
- Linux collect-window support remains unsupported in M17.1; the status protocol reports that
  honestly, and no Linux collect target is emitted.
- M16.1 cannot honestly close if no self-hosted Accessibility-granted macOS evidence exists.

## Verification

- Local Windows-host gate before push: `cargo fmt --all -- --check`, `cargo test --workspace`,
  `cargo clippy --all-targets --workspace -- -D warnings`, `cargo build --release`, and the
  installed Windows/macOS/Linux x64/ARM target checks.
- Hosted CI gate after push: Windows host gate, macOS Intel/arm64 bundle smoke, Linux x64/arm64
  X11 and Wayland smoke, and uploaded macOS app artifacts.
- Optional self-hosted CI gate: macOS Accessibility-granted command smoke on a runner labelled
  `[self-hosted, macOS, ARM64, honk300-a11y]`.
