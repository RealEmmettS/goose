TT;DR: After the first public stable release is live, use the Alienware and any later Mac/Linux
hardware for defense-in-depth verification. Findings fix forward; published tags/assets and the
Mac security/presentation contracts are immutable.

## Status

To-Do. This card intentionally does not block the operator-approved first stable publication once
the repository's exact-SHA candidate/default-branch/atomic-release gates pass. Start from fresh
published bytes, not a local rebuild. The tracked operator handoff is
`docs/agents/handoff/2026-07-15-001-alienware-post-v1.0.1-verification.md`.

## Why

Hosted native gates and the physical-M2 qualification cover the release contract, but additional
real Windows hardware and later Mac availability can expose driver, monitor, terminal, audio, or
installer differences. The operator explicitly chose stable publication now and forward patches
for later findings. This card keeps that verification visible without silently turning waived
observations into claimed passes.

## Plan

1. Resolve `main`, the published stable tag, GitHub Release, and `release-manifest.json` to the
   same full commit. Compare immutable-tag and stable-latest bytes/hashes before installation.
2. On the Alienware, exercise all three command names, portable and Global MSI provenance,
   install/update/repair/downgrade-refusal/uninstall, owner/lifecycle behavior, audio/no-sound,
   IPC, TUI restoration, terminal protection, collected windows, and user-close reactions.
3. Run `script/smoke_windows_overlay.ps1` against the exact published x64 executable. Retain its
   paired-DWM images/JSON/logs and visually inspect both renderer views, edge entry/wrap/exit,
   planted gait, secondary motion, mud/prank returns, and multi-monitor seams/hot-plug.
4. When a Mac is available again, fresh-download the published app ZIP/DMG and repeat the exact-
   SHA 10+60 profile, menu Configure/Quit, visible beak contact, terminal/ordinary-window matrix,
   v0.3.2 update, rollback injection, foreign-file preservation, and any available Ghostty or
   multi-display checks. Preserve the accepted AppKit RGBA, Accessibility, Developer ID,
   notarization/stapling, per-user install, semantic note color, and graceful-exit contracts.
5. Record real Linux hardware observations when available. Native Wayland remains reduced mode;
   do not infer window/cursor parity from X11/XWayland or compositor-specific capabilities.
6. Reproduce any finding against fresh published bytes, add a failing regression, and ship only a
   new reviewed semantic patch. Never force-update a tag or replace published assets.

## Verification

- [ ] Published tag, release target, manifest commit, hashes, and latest aliases agree.
- [ ] Alienware x64 paired-DWM and lifecycle smoke passes from the published executable.
- [ ] Windows Global MSI and portable update/uninstall behavior passes on real hardware.
- [ ] Windows renderer/movement/reaction/audio/TUI/terminal/multi-monitor observations are logged.
- [ ] Deferred Mac exact-SHA profile/menu/contact/lifecycle/terminal checks are run or remain
  explicitly unavailable without changing prior claims.
- [ ] Any Linux real-hardware evidence keeps X11, XWayland, and native Wayland capability claims
  distinct.
- [ ] Findings, if any, are fixed forward with tests and a new immutable patch release.

## Activity

- 2026-07-15 03:15 - created as the verification-only continuation for the Alienware and later
  native hardware. It owns accepted closure waivers from `#m20q`/`#m16r`; it does not authorize
  changing published tags, weakening gates, or undoing the Mac-specific implementation (agent:
  codex).
