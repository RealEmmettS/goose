# Windows/Linux Tray Design Evidence Canvas

Date opened: 2026-07-17
Task: `#trayc`
Release: v1.1.0
Decision predecessor: ADR 0028
Method: critical-thinking architecture/constraint inquiry; append-only evidence log

## Decision question

How should Honk300 expose the existing Configure/TUI and graceful-Quit controls through native
Windows and Linux tray surfaces without creating a second settings model, abrupt shutdown path,
new network/control authority, or unsupported desktop-shell claim?

## Hard constraints

- Preserve the macOS ADR 0028 behavior and shared local IPC semantics.
- Configure launches the existing terminal TUI and restores terminal state on exit.
- Quit sends the existing graceful stop intent; it never terminates the process directly.
- The tray is visible only while the runtime is alive and shares runtime/singleton ownership.
- Windows must survive Explorer/taskbar recreation and remain DPI/accessibility aware.
- Linux must use a maintained status-notifier path where available and degrade explicitly when
  the desktop shell has no compatible host.
- X11, XWayland, and native Wayland compositor capabilities remain distinct from tray availability.
- Archives, Debian packages, installers, update, repair, and uninstall own every shipped tray
  resource without owning mutable user content.
- Rust 1.95 and the complete existing Windows/Linux architecture matrix remain supported.

## Evidence ledger

- 2026-07-17 — v1.0.3 is the independently verified public baseline at
  `5192fab9690ff8b6777366a5918c12bbe1ee247a`; tray work begins only after its closure. (fact)
- 2026-07-17 — ADR 0028 supplies the behavioral oracle: one accessible goose control, Configure
  opens the existing TUI, Quit enters graceful engine-owned walk-off, and no platform gets a
  second preferences model. (fact)

## Candidate hypotheses

- H1: one small platform-neutral tray action router plus backend-owned UI lifetimes can preserve
  command semantics without making the engine depend on native tray libraries.
- H2: Windows can use the existing `windows` crate and `Shell_NotifyIconW` on the runtime thread
  or a retained tray thread, avoiding an additional cross-platform tray dependency.
- H3: Linux should prefer a StatusNotifierItem implementation over legacy XEmbed so the same
  path can work under supported X11 and Wayland desktop shells; unavailability must be reported
  rather than treated as runtime failure.
- H4: canonical SVG-derived packaged PNG/ICO assets can be deterministic across source, archive,
  MSI/EXE, Debian, and runtime validation without introducing runtime SVG rendering.

## Exit conditions

- Primary-source dependency/event-loop research is recorded.
- One accepted contract and rejected alternatives are written into a new ADR.
- Asset derivation and package ownership are testable and deterministic.
- Platform-neutral action-routing tests prove Configure and graceful Quit parity before either
  native backend task begins.

## Primary-source findings

- 2026-07-17 — Microsoft requires `NIM_SETVERSION` after every `NIM_ADD`; version 4 provides
  `NIN_SELECT`, `NIN_KEYSELECT`, and `WM_CONTEXTMENU` activation semantics. `NIM_SETFOCUS`
  returns keyboard focus to the notification area after a menu closes. (fact; Microsoft
  `Shell_NotifyIcon` documentation)
- 2026-07-17 — Explorer broadcasts the registered `TaskbarCreated` message after recreating the
  taskbar, and notification icons must be re-added. Windows also uses that taskbar contract around
  primary-display DPI changes. (fact; Microsoft taskbar documentation)
- 2026-07-17 — the freedesktop StatusNotifierWatcher owns item/host registration and exposes
  whether a host exists. An item cannot honestly claim visibility without a watcher and host.
  (fact; freedesktop StatusNotifierItem specification)
- 2026-07-17 — `tray-icon` 0.24.1 supports the existing Windows pump but its Linux path requires
  GTK plus AppIndicator/Ayatana system libraries and a GTK event loop. That would add native
  runtime/build requirements to every GNU/musl target and is rejected. (fact from maintained
  crate source and README; inference about this repository's matrix)
- 2026-07-17 — `ksni` 0.3.6 has MSRV 1.80, a blocking facade over pure Rust D-Bus, explicit
  `Watcher`/`WontShow` startup errors, host online/offline callbacks, retained shutdown handles,
  pixmap icons, and no GTK dependency. `async-io` plus `blocking`, with default Tokio disabled,
  compiles for `x86_64-unknown-linux-musl` under the pinned Rust 1.95 toolchain. (fact from crate
  metadata/source and local cross-target check)
- 2026-07-17 — `xdg-terminal-exec` is still a proposal, not a universal desktop contract. It is
  safe as the first no-shell launcher when present, followed by explicit argument-vector fallbacks
  for common terminals; absence stays a visible Configure error and never becomes Quit. (fact
  from proposal; decision)

## Adversarial checks

- A native callback cannot receive a `World`, config path, IPC handle, or process-kill authority;
  it can only enqueue `ControlSurfaceCommand::{Configure, Quit}`. (accepted invariant)
- The one shared runtime router is the only translation point and maps Quit exclusively to
  `RuntimeCore::begin_graceful_stop`. Automated tests inject a panicking Configure launcher to
  prove the Quit branch cannot invoke it. (accepted invariant and test evidence)
- Windows keeps its hidden owner HWND, generated HICON, shell registration, and fixed GUID for the
  runtime lifetime. `TaskbarCreated` sets a flag; the normal runtime thread re-adds and reapplies
  version 4 instead of making shell calls from a foreign thread. (accepted design)
- Linux advertises an embedded ARGB pixmap rather than relying on an installed icon theme. A
  Debian theme copy is still package-owned for application launchers. Missing D-Bus, watcher, or
  host is logged non-fatally and leaves CLI/IPC controls intact. (accepted design)
- Both native icons derive from the same sealed 36x36 PNG representation of the canonical SVG and
  apply the same white-goose/dark-blue-circle contrast transform. There is no runtime SVG parser,
  file lookup, mutable icon override, or network fetch. (accepted design)

## Decision

Accept H1, H2, and H3. Refine H4: the canonical SVG and deterministic 36x36 PNG remain the tracked
asset pair; Windows and Linux embed the PNG and deterministically compose their required BGRA/ARGB
bytes in memory. Debian additionally owns `/usr/share/icons/hicolor/36x36/apps/honk300.png`.
Reject a shared GTK tray crate, legacy XEmbed fallback, shell-interpolated terminal commands, a
second preferences UI, and any immediate-exit handler. ADR 0030 records the durable boundary.
