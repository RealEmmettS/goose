TL;DR: With v1.0.2 stable/latest, use the Alienware and any later Mac/Linux
hardware for defense-in-depth verification. Findings fix forward; published tags/assets and the
Mac security/presentation contracts are immutable.

## Status

Done 2026-07-17. Exact public v1.0.2 identity, portable/Corporate lifecycle, paired-DWM evidence,
runtime/TUI/audio behavior, forced-failure retry, and every reproduced source finding are closed.
The one-display/absent-Ghostty/no-Linux-host limits and operator-canceled Global elevation remain
explicit waivers. Forward publication belongs to active `#r103`; prior releases remain immutable.

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
- [x] Alienware x64 paired-DWM and lifecycle smoke passes from the published executable.
- [~] Windows portable and Corporate update/uninstall behavior passes on real hardware. Global
  elevation was explicitly canceled before `msiexec` and remains required in hosted release smoke.
- [x] Windows renderer/movement/reaction/audio/TUI/terminal observations are logged; the single-
  display host records multi-monitor/hot-plug as unavailable rather than inferred.
- [~] Deferred Mac exact-SHA profile/menu/contact/lifecycle/terminal checks are run or remain
  explicitly unavailable without changing prior claims.
- [~] No Linux desktop/WSL distribution is available on this host; four cross-target checks pass
  while candidate native X11/Wayland evidence remains authoritative and distinct.
- [x] Findings are reproduced, fixed forward with red/green tests, and assigned only to v1.0.3.

## Activity

- 2026-07-17 07:25 — closed after the complete local source/package gate: optimized v1.0.3
  paired-DWM smoke and direct image inspection, real Corporate install/run/graceful-stop/self-
  uninstall, x64 MSI administrative payload identity, Windows ARM64/Apple/Linux cross-checks,
  workspace tests/Clippy/audit/dist/workflow/script gates all pass. A third Global launch returned
  explicit user cancellation before `msiexec`; this is recorded as an operator waiver and remains
  mandatory in hosted candidate/post-release smoke. (agent: codex)
- 2026-07-17 06:55 — native Windows PowerShell now proves guarded direct-MSI receipt refresh:
  exact owned metadata advances atomically through a path containing an apostrophe while malformed
  and foreign bytes remain unchanged. A live window audit also found Codex exposed as generic
  `Chrome_WidgetWin_1 / ChatGPT`; red/green Windows and Linux regressions now match macOS by
  conservatively protecting Codex and Visual Studio Code surfaces. A second Global elevation
  attempt again ended before `msiexec`, log creation, or byte change, so that physical gate remains
  open rather than inferred. (agent: codex)
- 2026-07-17 06:20 — resolved H5 with an official-contract and injected-failure experiment. A
  Corporate 1.0.3 test MSI scheduled at `afterInstallFinalize` preserved v0.2.1 byte-for-byte on
  forced failure, then cleanly retried and uninstalled with no sampled component clients or old
  product registration left behind. The packaging regression is green; Global keeps its existing
  transactional schedule. (agent: codex)
- 2026-07-17 05:37 — native Corporate lifecycle/run-time matrix passed for clean install, exact-
  byte repair, downgrade refusal, ordinary v0.2.1→v1.0.2 upgrade, side and top-down compositor
  evidence, three-alias control/graceful stop, audio initialization, real 80×24 TUI restoration,
  user-media-preserving uninstall, and backup-first purge. Red/green v1.0.3 fixes now cover Windows
  updater argument/byte decoding, coexisting Global/Corporate provenance and cross-hive ARP proof,
  and already-exited deferred-parent handling. A forced Corporate failure/retry additionally left
  an orphan v0.2.1 MSI component client; H5 remains open for official schedule research and a
  reproduced correction. Global v0.3.1 remains untouched because its UAC prompt was canceled
  before MSI execution. (agent: codex)
- 2026-07-17 05:02 — reproduced a current Windows updater defect: both installed v0.3.1 and exact
  verified portable v1.0.2 fail latest-manifest discovery because the PowerShell `-Command`
  invocation appends the URL as command text instead of binding `param($uri)`. No download/UAC/
  mutation occurred. This is accepted v1.0.3 hardening scope; regression must cover text and file
  download command construction. (agent: codex)
- 2026-07-17 04:58 — exact published x64 v1.0.2 portable executable passed the Alienware paired-
  DWM compositor and IPC lifecycle smoke: semantic alpha/palette/pose thresholds, single-instance,
  reload, graceful stop, immediate restart, and second stop all passed. Direct inspection of all
  six retained PNGs confirmed uniform controlled backgrounds and complete light/dark side poses
  without black rectangles or stale pixels. (agent: codex)
- 2026-07-17 04:54 — independently bound immutable public v1.0.2 to source commit
  `964305869e9ec28768c789465db1b6317dfa3f6f`; immutable/latest manifests are byte-identical, and
  fresh x64 PowerShell/portable/Global/Corporate payloads all match manifest size/SHA, named
  sidecars, `sha256.sum`, and GitHub asset digests. Portable PE is x86_64/version 1.0.2; MSI
  metadata is version 1.0.2. Windows artifacts are presently unsigned, consistently with the
  repository's Mac-only public signing gate. Next is exact-byte DWM smoke and lifecycle mutation
  from the retained v0.3.1 Global MSI fixture. (agent: codex)
- 2026-07-17 04:55 — read-only Alienware baseline confirmed Windows 11 x64 on an Alienware m16 R2,
  one 1920×1080 display, NVIDIA/Intel graphics, Realtek/Shure audio, Windows Terminal, PowerShell,
  cmd, VS Code, and Codex terminals, with Ghostty absent. Found no running goose and one old
  machine-wide v0.3.1 Global MSI with three hash-identical aliases, HKLM msi-global marker, no
  receipt/user-state tree, and no autostart; operator authorized preserving it as the update
  fixture. (agent: codex)
- 2026-07-17 04:40 — moved To-Do → Active on the Alienware; upgraded and relaunched the tracked
  SHAUGHV board at bundle 0.2.2 with full assets and shared hooks; opened the Scientific Inquiry
  evidence canvas before touching published artifacts or installed state. (agent: codex)
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
