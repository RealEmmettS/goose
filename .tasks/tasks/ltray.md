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

- [x] Native Linux tests cover host available/unavailable behavior, terminal selection, graceful-stop routing, and capability separation.
- [x] x64/ARM64 GNU and musl builds plus Debian/archive packaging contracts pass without hidden unsupported dependencies.
- [x] Hosted X11 and native Wayland reduced-mode jobs exercise the tray where the desktop fixture supports it and assert honest degradation otherwise.
- [x] Install, update, and uninstall preserve user data and remove only owned tray resources.

## Status

Done. The pinned pure-Rust StatusNotifier implementation, shared command routing, embedded ARGB
icon, no-shell terminal selection, deterministic unavailable degradation, watcher recovery, and
Debian icon ownership passed the native x64/ARM64 hosted and compositor matrix.

## Activity

- 2026-07-17 08:48 — moved Active → Done after exact-SHA CI 29584610137 passed on native x64 and
  ARM64 Linux. Both architectures exercised the private StatusNotifier watcher/host, accessible
  properties, embedded icon, Configure/Quit menu events, no-host/missing-watcher errors, watcher
  recovery, deterministic no-session-bus runtime, X11 overlay, native Wayland reduced mode, Debian
  contracts, and every GNU/musl target check. (agent: codex)
- 2026-07-17 08:02 — moved To-Do → Active after #wtray closed. Added the hosted private-D-Bus
  watcher/host protocol test for accessible identity, embedded icon, both menu actions, explicit
  no-host/missing-watcher failures, and watcher recovery. The compositor smoke now forces and
  asserts the no-session-bus path independently under both X11 and native Wayland reduced mode.
  Debian owns the canonical icon under hicolor and removes it with the package. Musl test code
  cross-checks cleanly; native x64/ARM64 hosted execution is pending CI. (agent: codex)
- 2026-07-17 04:45 — created from the operator's direct Linux tray request under milestone #v110. (agent: codex)
