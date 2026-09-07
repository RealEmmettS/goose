# 0041 — Refinement, native settings, and continuous goose

## Status
Accepted by the user's 2026-09-07 implementation request. Delivery is staged; acceptance is recorded separately from this decision.

## Decision

- Retain the platform-free Rust engine and clean-room procedural vector renderer. Replace the V2 side/overhead opacity switch with a continuous projected rig and bounded task-driven expressions. Geometry changes retain interaction anchors, planted feet, bounds, and terminal safety tests.
- Add a separately launched Native SDK settings executable using native markup and Zig. Rust remains the single configuration validation/persistence and lifecycle authority. The GUI uses a bounded versioned stdio service; existing runtime IPC remains compatible.
- Tray Configure opens graphical settings. `honk300 settings` launches it; `honk300 config` retains the TUI. The settings window never owns the overlay or adds another tray, update channel, or autostart mechanism.
- Preserve previous configuration choices and unknown compatible TOML fields; detect stale edits across cooperating editors before commit.
- Share bounded canvas reuse and accumulator-aware pacing across runtimes. Preserve native presentation, final transparent-clear acknowledgement, and bounded catchup.
- Add owned Linux props and explicit session/capability reporting. Broader Wayland operations require separately qualified, user-enabled compositor/portal integrations. Pi 4/5 64-bit labwc support remains experimental until physical acceptance.
- Publish the core refinement first, Linux props/Pi second, and optional compositor integrations afterward. Existing release tags and bytes stay immutable.

## Supersession
Supersedes ADR 0014 only for the two-view renderer architecture, and ADRs 0024/0028/0030 only for Configure's TUI-only restriction. The existing tray owner, verified updater, permission, terminal-protection, and lifecycle contracts continue unchanged.

## Qualification
Use real rendering and platform/control behavior, not test-only descriptions of those behaviors. GUI build and test are separate required checks. Cross-architecture CI and physical acceptance remain distinct. Failed qualification blocks the affected release stage.

## Settings build and service boundary

The settings binary pins Native SDK 0.5.4 and Zig 0.16.0. Its local dependency is verified and patched to set `create_no_window` on SDK subprocess effects and resolve an explicitly registered bold font through the shared text measurement/rendering path. The upstream version otherwise creates a console for the Rust service and silently substitutes a custom font's base face for bold spans. `settings/scripts/prepare-sdk.mjs` checks exact source hashes before either applying or accepting these patches. No global SDK files change. The user-selected Makira Bold and Light TTFs are embedded; system appearance, high contrast, and reduced motion still feed the design tokens.

The v1 stdio service accepts one JSON request up to 64 KiB and emits one response up to 128 KiB. Its 53-field catalogue covers each editable configuration leaf once. It rejects extra command fields, unknown settings, wrong types, invalid values, oversized input, malformed files and stale revisions. The native client uses the SDK's smaller 4 KiB stdin bound and sends only changed fields. Save preserves comments and unknown compatible TOML; reload errors and pending restart changes remain visible. CLI, TUI and GUI update discovery is read-only, and their explicit Update action retains the existing separate updater window and receipt-owned lifecycle.

Editor saves use the additive `RELOAD_IF` command with a SHA-256 token derived from the canonical selected config path and exact saved file revision. The runtime loads a stable snapshot and applies it only if that token matches, so editing another file or a concurrent external edit cannot be reported as applied. The existing `RELOAD` command remains compatible. Old runtimes reject the new command and editors report the save without claiming live application.

## Additional motion qualification

Front-facing motion exposed both feet trailing behind the new rounded belly. Recovery now predicts the entire swing plus half the following stance, and a foot planted ahead waits for the body to pass over it instead of being lifted again because of absolute distance. The shared body/neck anatomy has four pixels more belly clearance; planted foot and mud coordinates remain unchanged. Existing cadence/lag tests pass, planted-foot invariants cover all twelve sampled headings, and rendered feet remain visible through complete walking/running/charging cycles. Compositor oracles use the new palette and one head/neck/body/feet signature, preserving cropped-pose, channel-swap, opaque-surface, shadow, and alpha rejection.

## Native accessibility qualification remains open

Windows computer use captured and operated the actual settings HWND, including saving an isolated draft and reading back its file. The Windows UI Automation tree exposed only the canvas pane and window chrome. SDK 0.5.4 does not wire its widget-accessibility callback on Windows or GTK; the inspected current 0.10.1 Windows package also lacks that callback. SDK automation semantics are therefore not evidence of a Windows screen-reader bridge. This remains an explicit GUI acceptance issue, separate from the verified font layout and mouse/keyboard behavior.
