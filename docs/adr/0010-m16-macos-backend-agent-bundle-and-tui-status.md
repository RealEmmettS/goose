# ADR 0010: M16 macOS Backend, Agent Bundle, And TUI Status

## Status

Accepted.

## Context

M16 moves `honk300` beyond the Windows-only runtime without changing the project control model.
macOS needs a real `.app` identity for durable Accessibility grants, but the app must not grow a
native preferences surface, menu-bar settings UI, Dock controls, or AppleScript command channel.
The user-facing surface remains the CLI and ratatui TUI over local IPC.

The old Boolean backend flags were not expressive enough for macOS. A denied Accessibility grant,
an unsupported public API, and a runtime failure must be distinguishable to the CLI/TUI while the
engine still receives simple active capability flags.

## Decision

- Add `crates/honk-platform-macos` as the AppKit/CoreGraphics/ApplicationServices backend crate.
  `honk-engine` remains OS-free and continues to receive only platform-neutral pointer, presence,
  cursor, foreign-window, collect-window, sound, and render inputs/commands.
- Route macOS `honk300 start` into the desktop runtime rather than the non-Windows placeholder.
  The macOS runtime uses the existing Unix local IPC transport and the same CLI/TUI control model
  as Windows.
- Stage macOS as an agent app bundle with bundle id `dev.emmetts.honk300` and `LSUIElement=true`.
  The `.app` exists for overlay runtime identity and Accessibility permission persistence only.
- Do not add a macOS preferences window, menu-bar settings UI, Dock control surface, or `.sdef`
  AppleScript command surface in M16.
- Extend IPC with `STATUS` and a compact `ControlResponse::Status` payload. Capability state is
  reported as supported, unsupported, denied, or failed. `honk300 status` and the TUI Status tab
  show running state, platform, bundle mode, Accessibility, cursor/window/collect/presence/audio
  capabilities, and asset counts.
- Keep backend state explicit in `honk-config`: the engine-facing `WorldOptions` still receives
  Booleans, but CLI/TUI status can preserve denial/failure reasons.
- Prefer bundled assets at `Contents/Resources/Assets` when running inside a `.app`; fall back to
  current-directory and executable-adjacent assets for development.
- Add `script/package_macos_app.sh` for macOS hosts to build x86_64 and arm64 release slices,
  `lipo` a universal2 binary into `Honk300.app`, copy assets, write `Info.plist`, ad-hoc sign,
  and validate with `plutil`, `codesign`, and `lipo`.

## Consequences

- macOS status and degraded behavior are visible from the terminal without introducing a second
  settings surface.
- A bundled launch from the TUI uses `/usr/bin/open -n <Honk300.app> --args start --config <path>`;
  bare development binaries still launch directly.
- Cursor warp and Accessibility-backed window features report `denied` when the bundle lacks
  Accessibility permission. Public-API gaps such as Focus/DND report `unsupported` until a
  reliable implementation exists.
- The universal2 `.app` staging path is implemented, but `.dmg`, Developer ID signing,
  notarization, installers, update/uninstall flows, and final icon polish remain M19 work.
- macOS-host smoke remains required before declaring the whole M16 readiness pass complete:
  overlay visibility/click-through, Accessibility denial/grant behavior, status/reload/stop,
  audio, terminal protection, and Intel/Apple Silicon slice evidence must be captured on macOS.
