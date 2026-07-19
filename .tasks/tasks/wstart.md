TT;DR: Route typed Windows starts through the branded GUI launcher so the terminal becomes only a controller while the existing hidden CLI runtime and tray continue independently.

## Why

A directly typed `honk300 start` currently becomes the long-lived console process. Closing that
PowerShell or terminal window therefore ends the goose even though Windows already ships a
GUI-subsystem launcher for shortcuts and login startup. Every Windows start surface should use
one detached, shell-free process model while retaining the terminal TUI and tray controls.

## Plan

Keep `honk300-app.exe` as the transient branded launch surface. Public `honk300`/`honk`/`goose`
start commands invoke that exact sibling, which forwards all start options to a hidden internal
runtime command on `honk300.exe`. The app waits for bounded IPC readiness and then exits; the
console-free runtime owns the singleton, overlay, IPC, and notification-area icon until graceful
or forced shutdown. Add a real PE icon resource for the user-pinnable launcher and update the
Windows lifecycle smoke to distinguish the transient controller/launcher from the retained
runtime.

## Impact

Intended: the prompt returns after readiness and the goose survives closure of the originating
terminal. Shortcuts, optional login start, and typed CLI starts converge on the same app-launcher
path. Risks include recursive start dispatch, lost start flags, orphaned pre-ready children,
duplicate runtimes, hidden startup failures, broken compositor evidence, or a packaged launcher
without its branded resource.

## Acceptance

All public aliases return only after an existing or new runtime is ready; closing their parent
shell does not end the exact hidden runtime or tray. Missing launchers and bounded startup failures
return nonzero with readable diagnostics. Direct shortcuts and login start remain windowless,
Configure still opens the exact terminal TUI, every stop mode retains its existing semantics, and
Windows x64/ARM64 artifacts contain the GUI-subsystem branded launcher.

## Verification

- [x] Focused Rust tests cover hidden parsing, option forwarding, path resolution, readiness,
  failure cleanup, and singleton behavior.
- [ ] Packaging tests prove PE subsystem, icon resource, shortcut/Run ownership, and archive/MSI
  identity on Windows x64 and ARM64.
- [ ] Disposable lifecycle proof covers short-lived public shells, retained hidden runtime/tray,
  duplicate start, Configure, graceful CLI/tray Quit, force stop, and restart.
- [x] The complete noninteractive local gate passes; any visible physical-Windows check remains
  bounded and operator-authorized.

## Status

Done. v1.3.0 is public stable/latest from exact source
`c350b06d1dbf6c2057eb2d5ebd625da2da9419b8`; candidate, unchanged-main, atomic publication,
all eight fresh-public-byte lanes, exact/latest aliases, and production deployment verification
are complete.

## Activity

- 2026-07-19 — created from the operator's report that a CLI-started goose dies with its terminal.
  Confirmed that shortcuts and login start already use `honk300-app.exe`, while typed `start`
  still runs the foreground runtime directly; the app executable also lacks a branded PE icon.
  (agent: codex)
- 2026-07-19 — implemented the public-controller → transient branded app → hidden exact-sibling
  runtime chain. All start options cross both boundaries as argument vectors; the app owns the
  existing ten-second readiness deadline and distinct spawn/early-exit/timeout results, while
  concurrent launchers observe the singleton winner. Added the app-specific multi-resolution ICO,
  private-command smoke split, and focused parsing/path/readiness/packaging tests. Focused Rust
  tests (104 CLI/runtime, 1 app, 35 TUI), 13 packaging tests, and PowerShell parsing pass. ADR 0036
  and current guidance/changelogs are synchronized; complete gates remain in progress.
  (agent: codex)
- 2026-07-19 — completed the local gate: formatting, strict workspace Clippy, 442-test locked
  workspace suite plus the focused concurrent-launcher regression, locked x64 release build, 122
  Python contracts (three platform skips), PowerShell parsing, actionlint, RustSec over 418 locked
  dependencies, cargo-dist planning, 108-file dirty-tree package-shape verification, x64 PE
  subsystem/icon extraction, and an ARM64 cross-check that compiled the binary-specific resource.
  A bounded physical proof started from short-lived PowerShell PID 31540; the controller and app
  parent exited, hidden runtime PID 2360 retained IPC and its tray-owner window, duplicate start
  preserved that PID, and graceful stop completed. A native ARM64 link cannot run locally because
  the MSVC ARM64 linker workload is absent (the ambient `link.exe` is GNU coreutils); disposable
  x64/native ARM64 CI therefore remain the accurate final gate. (agent: codex)
- 2026-07-19 — operator authorized pushing the new version. Chose stable `v1.3.0` because detached
  typed start and the branded app identity are user-facing functionality rather than a corrective
  `v1.2.x` patch. Opened the exact candidate/main/publication/public-byte closure sequence while
  preserving immutable v1.2.6. (agent: codex)
- 2026-07-19 — exact candidate `29698609939` at immutable failed commit `451103d` failed closed
  before publication. The ARM64 portable lane found that LLVM-RC resolves icon paths from the
  resource directory, and the x64 lifecycle lane found that IPC can become ready just before its
  runtime-owned tray window is observable on a back-to-back start. Fixed forward with an explicit
  resource include directory and a strict bounded tray-owner wait; no waiver was broadened.
  Focused packaging/smoke contracts, x64 release build, ARM64 target check, and the complete local
  Windows compositor/lifecycle smoke now pass. A new exact candidate remains required.
  (agent: codex)
- 2026-07-19 — replacement candidate `29698948076` at immutable failed commit `ef6f76b` proved
  the repaired ARM64 resource producer and all Windows/Apple portable lanes, then failed closed
  before publication because the new harness force-restarted between aliases. That sequencing
  contradicted the required singleton convergence and raced Explorer's asynchronous fixed-GUID
  cleanup. The public phase now keeps one runtime/tray across all three short-lived alias shells,
  proves both duplicate starts preserve its PID and every app parent exits, then performs one
  graceful cleanup. The separate lifecycle matrix retains force-stop and restart coverage. The
  focused contracts and complete 79-second physical x64 smoke pass. (agent: codex)
- 2026-07-19 — third candidate `29699310086` at immutable failed commit `f927cf2` passed the
  signed Mac producer, every portable producer, and both Windows builds, then both x64 consumers
  independently exposed a real immediate-restart defect. Explorer retained the old fixed tray
  GUID after process exit, Honk300's single add failed, and a stock icon succeeded moments later.
  Fixed the runtime with 31 bounded 100 ms registration attempts inside the existing ten-second
  readiness deadline; permanent shell failure still degrades explicitly. Windows backend tests,
  strict Clippy, focused packaging contracts, release build, and the complete physical lifecycle
  smoke—including the exact failed transition—now pass. (agent: codex)
- 2026-07-19 — fourth candidate `29699787617` at immutable failed commit `fe0700c` passed every
  producer, then showed that synchronous retry was still the wrong boundary: both x64 consumers
  retained the old Explorer GUID beyond three seconds, and native ARM64 consumed enough of the
  launcher deadline to return status 10. Moved recovery into nonblocking runtime maintenance:
  retain the exact owner immediately, preserve IPC readiness, retry every 100 ms, report explicit
  degradation after three seconds, and keep recovering afterward. ARM64's unavailable-host proof
  still requires an independently failed stock fixed-GUID probe; strict x64 remains unwaived.
  Focused Windows tests, strict Clippy, x64 release and ARM64 target builds, and the complete
  physical lifecycle smoke pass with this nonblocking implementation.
  (agent: codex)
- 2026-07-19 — exact candidate `29700210929` passed all 19 jobs at `c350b06d`, including signed
  Mac production, strict x64 recovery, native ARM64 degradation/compositor/MSI/lifecycle, Linux,
  Debian, and final assembly. Unchanged-main CI `29700548745` then passed all 11 required jobs.
  Atomic publication `29700897715` passed all 19 jobs and published 47 immutable assets with 22
  manifest payloads. Fresh-public-byte run `29701222408` passed all eight Windows/Mac/Linux/Debian
  lanes. Exact/latest manifest and installer bytes match; the production deployment serves the
  same v1.3.0 manifest and versionless commands. Task complete. (agent: codex)
