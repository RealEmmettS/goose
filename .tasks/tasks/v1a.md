TL;DR: With v1.0.2 stable/latest, use the Alienware and any later Mac/Linux
hardware for defense-in-depth verification. Findings fix forward; published tags/assets and the
Mac security/presentation contracts are immutable.

## Status

To-Do. This card intentionally does not block the operator-approved first stable publication once
the repository's exact-SHA candidate/default-branch/atomic-release gates pass. Start from fresh
published bytes, not a local rebuild. The tracked operator handoff is
`docs/agents/handoff/2026-07-17-001-alienware-post-v1.0.2-verification.md`.

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
   v1.0.1 update, rollback injection, foreign-file preservation, and any available Ghostty or
   multi-display checks. Preserve the accepted AppKit RGBA, Accessibility, Developer ID,
   notarization/stapling, per-user install, semantic note color, and graceful-exit contracts.
5. Record real Linux hardware observations when available. Native Wayland remains reduced mode;
   do not infer window/cursor parity from X11/XWayland or compositor-specific capabilities.
6. Reproduce any finding against fresh published bytes, add a failing regression, and ship only a
   new reviewed semantic patch. Never force-update a tag or replace published assets.

## Verification

- [x] Published v1.0.2 tag, release target, manifest commit, hashes, and latest aliases agree on
  the physical Mac and hosted release; the Alienware should independently repeat this preflight.
- [ ] Alienware x64 paired-DWM and lifecycle smoke passes from the published executable.
- [ ] Windows Global MSI and portable update/uninstall behavior passes on real hardware.
- [ ] Windows renderer/movement/reaction/audio/TUI/terminal/multi-monitor observations are logged.
- [~] Deferred Mac exact-SHA profile/menu/contact/lifecycle/terminal checks are run or remain
  explicitly unavailable without changing prior claims.
- [ ] Any Linux real-hardware evidence keeps X11, XWayland, and native Wayland capability claims
  distinct.
- [ ] Findings, if any, are fixed forward with tests and a new immutable patch release.

## Activity

- 2026-07-17 09:30 - public v1.0.2 was independently resolved to exact source
  `964305869e9ec28768c789465db1b6317dfa3f6f`. Candidate `29565557915`, same-SHA CI
  `29566294408`, atomic release `29566759574`, and post-release smoke `29567257622` passed.
  Fresh public Mac app/DMG trust, v1.0.1→v1.0.2 managed update, three-alias repeat no-op, menu
  icon/accessibility, five-second graceful Quit/restart, live progressive-disclosure website, and
  final host purge passed. Remaining work is verification-only: Alienware x64 hardware, later
  Linux observations, visible beak contact, broader terminal/fault injection, and any future
  multi-display/Ghostty Mac availability. (agent: codex)
- 2026-07-17 02:10 - fresh published v1.0.1 DMG/app trust, graphical install, receipt/aliases,
  menu Configure into the complete 120×30 TUI, same-process Accessibility grant, semantic dark-
  mode note contrast, animated 5.415-second menu Quit, and immediate restart passed on the physical
  M2. Two isolated 60-second profiles measured 7.80% and 8.60% median CPU, at most 14.66 MiB RSS,
  and no positive growth on the heavily loaded host. The one-display Mac cannot supply live
  multi-monitor/hot-plug evidence, Ghostty is absent, and hardened runtime blocks `leaks` attachment
  to the exact signed release; those limitations remain explicit. Remaining beak-contact,
  fault-injection, and broader terminal observations stay open rather than becoming inferred
  passes. (agent: codex)
- 2026-07-15 03:15 - created as the verification-only continuation for the Alienware and later
  native hardware. It owns accepted closure waivers from `#m20q`/`#m16r`; it does not authorize
  changing published tags, weakening gates, or undoing the Mac-specific implementation (agent:
  codex).
