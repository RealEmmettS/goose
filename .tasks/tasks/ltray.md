TT;DR: Add the shared goose control surface to supported Linux desktops while keeping unsupported tray and native Wayland capabilities explicit.

## Why

The operator directly requested Linux tray parity. Linux desktop environments differ in status-notifier support, terminal launch behavior, packaging dependencies, and session lifecycle, so the implementation must provide useful controls without claiming universal shell support.

## Plan

Select a maintained status-notifier implementation compatible with supported GNU/musl and Debian/archive targets. Use the canonical goose asset and accessible label. Configure launches the exact installed binary's existing TUI in an available normal user terminal; Quit uses the same local IPC/engine graceful stop. Keep tray availability distinct from X11/XWayland/native Wayland mischief capabilities. Integrate package dependencies and clean uninstall ownership, with explicit non-fatal unavailable reporting.

## Impact

Intended: supported Linux desktops gain Configure and graceful Quit parity. Risks include native library dependencies, missing StatusNotifier hosts, terminal selection differences, musl incompatibility, duplicate items, and accidental overclaiming of native Wayland behavior.

## Acceptance

Supported X11/XWayland and native Wayland reduced-mode sessions expose the shared controls when a host is available and report/degrade honestly when it is not. Debian/archive install, update, and uninstall own only their tray assets/integration.

## Verification

- [ ] Native Linux tests cover host available/unavailable behavior, terminal selection, graceful-stop routing, and capability separation.
- [ ] x64/ARM64 GNU and musl builds plus Debian/archive packaging contracts pass without hidden unsupported dependencies.
- [ ] Hosted X11 and native Wayland reduced-mode jobs exercise the tray where the desktop fixture supports it and assert honest degradation otherwise.
- [ ] Install, update, and uninstall preserve user data and remove only owned tray resources.

## Status

To-Do; blocked on #trayc.

## Activity

- 2026-07-17 04:45 — created from the operator's direct Linux tray request under milestone #v110. (agent: codex)
