# Honk300 settings

The separate native window is built with Zig 0.16.0 and Native SDK 0.5.4. Its five pages edit the same configuration as `honk300 config`. Rust owns validation, atomic revision-aware persistence, runtime requests, and the verified updater. This app has no web layer, tray owner, second update channel, or startup mechanism.

Run `npm ci --ignore-scripts`, `npm run prepare:sdk`, `npm test`, and `npm run build` from this directory. `npm run check` validates the markup and manifest after the typed contract is built. The local SDK patch sets `create_no_window` for subprocess effects on Windows; its exact unpatched source hash is checked before application and on every rebuild. The global SDK installation is never edited. The SDK's Apache-2.0 license remains in the pinned dependency.

For development, build `honk300` and place that exact binary beside `zig-out/bin/honk300-settings`. Launch with an optional `--config PATH`; the GUI always invokes its exact `honk300` sibling through `__settings-service`. It never looks up a runtime on PATH. Release packaging must include and verify both executables. `-Dautomation=true` enables the SDK's file-based native UI harness; production builds leave it disabled.

The stdio protocol uses one request per child, version 1, an increasing request id, and an operation object. The service caps requests at 64 KiB and responses at 128 KiB; the UI respects the SDK's 4 KiB stdin ceiling by sending only changed fields. It retains a draft on validation or revision conflict. Save updates known TOML fields while retaining comments and unknown compatible keys. Runtime reload is reported as requested, and a changed Wayland backend explicitly requires restart.

Update discovery is read-only. Update now opens the established retained terminal helper, which owns installation, cancellation/failure recovery, and receipt-owned relaunch independently of this window.

The two embedded Makira faces use Light for body text and Bold for headings. A pinned patch connects the registered bold face to both measurement and painting. Native theme, high contrast, and reduced motion remain active.

The pinned AccessKit bridge forwards the existing semantics to Windows UI Automation and Linux AT-SPI (ADR 0042); macOS uses the SDK's AppKit provider with the exact-hash dialog and text-input corrections in [ADR 0051](../docs/adr/0051-native-mac-settings-modal-accessibility.md). On Windows, keep `honk_settings_accessibility.dll` beside the executable. The release build helper records and verifies both payload identities and includes dependency notices. Independent native UIA, AT-SPI and Mac Accessibility clients exercise real names, controls, text, focus, modal isolation and isolated saves. The production Mac companion is checked on Intel and Apple Silicon with automation disabled. These hosted provider checks remain distinct from physical screen-reader user acceptance.
