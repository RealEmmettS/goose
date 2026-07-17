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

To-Do; blocked on #trayc.

## Activity

- 2026-07-17 04:45 — created from the operator's direct Windows tray request under milestone #v110. (agent: codex)
