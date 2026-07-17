TT;DR: Add the shared goose control surface to the Windows notification area and prove the exact installed implementation on the Alienware.

## Why

The operator directly requested Windows tray parity with the macOS menu. Windows users need a discoverable graphical path to the existing TUI and a safe graceful Quit without inventing a second settings system.

## Plan

Implement a native notification-area icon owned for the runtime lifetime, using assets derived from the canonical goose SVG and an accessible Honk300 controls label. Configure launches the exact installed binary's config TUI in a normal unelevated terminal and restores it. Quit enters the existing engine-owned graceful stop, keeps the icon responsive until final clear/process exit, and never calls an abrupt kill API. Handle Explorer restart, DPI, session shutdown, singleton restart, denied capabilities, and installer ownership.

## Impact

Intended: Windows gains discoverable Configure and graceful Quit parity. Risks include duplicate icons, orphaned icons after crashes, incorrect elevation, terminal targeting, shell-restart loss, installer residue, and release artifacts missing icons.

## Acceptance

The installed Global MSI and supported portable/bootstrap paths expose one accessible goose icon, open the authoritative TUI, exit through the full animated lifecycle, recover after Explorer restart, preserve permissions and terminal protection, and remove only owned integration on uninstall.

## Verification

- [ ] Native unit/integration tests cover menu actions, lifetime, unavailable behavior, Explorer recreation, and graceful-stop routing.
- [ ] Windows x64 and ARM64 builds, strict clippy, installer contracts, and exact-binary compositor/lifecycle smokes pass.
- [ ] Alienware hardware proves accessibility/keyboard use, Configure/TUI restoration, animated Quit, immediate restart, DPI, and available multi-monitor behavior.
- [ ] Install, update, repair, and uninstall leave no duplicate or orphaned tray integration.

## Status

Done. Native notification-area ownership, exact-executable Configure, animated Quit, TaskbarCreated
recovery, accessible identity, packaging ownership, and the available Alienware hardware paths are
proven. The host has one display, so multi-monitor behavior retains the explicit test/native-matrix
gate rather than a fabricated local claim.

## Activity

- 2026-07-17 08:16 — CI run 29582777831 proved both Linux architectures and Windows x64, then
  exposed that GitHub's Windows ARM64 interactive session has DWM but no notification-area host;
  the runtime correctly logged its explicit CLI-only degradation. Tightened the smoke to accept
  that path only when `Shell_TrayWnd` is also absent, while continuing to require fixed-GUID
  registration/recreation whenever a real taskbar exists. The revised probe passed locally with
  the Alienware taskbar present. (agent: codex)
- 2026-07-17 08:02 — moved Active → Done. The amended exact-binary Windows smoke removed the
  fixed-GUID icon, proved it absent, delivered TaskbarCreated, and verified the same item/rectangle
  returned. UI Automation found one accessible `Honk300 controls` button; native keyboard menu use
  opened the full TUI from the exact executable and restored it, while Quit completed the walk-off
  in 3.1 seconds and immediate singleton restart succeeded. Packaging contracts passed; the icon is
  embedded in the binary, so no separately owned Windows file or registry integration can orphan.
  The single-display Alienware records the live multi-monitor hardware waiver. (agent: codex)
- 2026-07-17 08:25 — moved To-Do → Active after #trayc closed. Implemented retained
  Shell_NotifyIconW v4 ownership, keyboard/context menus, exact-executable Configure, graceful
  Quit routing, signed coordinates, and TaskbarCreated re-add; focused Windows tests pass.
  (agent: codex)
- 2026-07-17 04:45 — created from the operator's direct Windows tray request under milestone #v110. (agent: codex)
