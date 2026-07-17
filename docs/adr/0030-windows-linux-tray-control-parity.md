# ADR 0030 — Windows And Linux Tray Control Parity

- Status: Accepted (2026-07-17)
- Relates to: ADR 0004 (local control and terminal protection), ADR 0015 (platform safety),
  ADR 0021 (Wayland capability strata), ADR 0023 (cross-platform distribution), ADR 0028 (shared
  goose control surface), and ADR 0029 (Windows lifecycle and terminal hardening).
- Supersedes: ADR 0028 only where it says Windows and Linux tray surfaces are unimplemented. It
  preserves ADR 0028's macOS surface, accessible naming, existing-TUI Configure behavior, and
  engine-owned graceful Quit contract.

## Context

Honk300 v1.0.3 has one local configuration schema, one terminal TUI, one same-user IPC protocol,
and one engine-owned graceful shutdown. macOS already exposes those controls through its retained
AppKit status item. Windows and Linux need the same discoverable actions without creating native
preferences, weakening protected-terminal behavior, or overclaiming support on Linux shells that
do not host status notifier items.

The platforms have materially different event and packaging boundaries. The Windows runtime
already pumps Win32 messages on its overlay thread and Explorer can destroy notification icons
when the taskbar is recreated. Linux X11, XWayland, and native Wayland overlay capability does not
predict whether the desktop session has a D-Bus StatusNotifier watcher and host. A GTK-based
cross-platform tray abstraction would add GTK/AppIndicator native dependencies and another event
loop to GNU and musl artifacts that currently avoid them.

The shared monochrome source is optimized as an AppKit template. Windows and Linux hosts do not
promise template tinting, so a raw black silhouette can disappear against a dark panel. Portable
archives also cannot depend on a separately mutable icon file at runtime.

## Decision

### One finite native command contract

`honk-control::ControlSurfaceCommand` is the complete native UI authority:

- `Configure` invokes the existing platform launcher for the exact running executable's `config`
  command; that command opens `honk-config-tui` and remains the only save/validation UI.
- `Quit` is translated only by `runtime::control_surface::handle_command`, which calls
  `RuntimeCore::begin_graceful_stop`. Native callbacks never receive engine state and never call
  `exit`, `TerminateProcess`, signals, or process-kill APIs.

The router propagates Configure launch errors while leaving the goose running. Quit keeps the
control surface alive and responsive until the goose has completely walked through an exposed
edge, the final clear has been presented, props are cleaned up, and normal runtime destruction
releases the singleton.

macOS now emits this same shared enum from its existing AppKit menu target. That is a type-level
consolidation only; its signed launcher and observed behavior do not change.

### Windows notification-area owner

Windows uses the existing `windows` 0.58 dependency and native `Shell_NotifyIconW`, not a new tray
crate. A hidden, non-activating tool-window HWND shares the existing runtime thread and message
pump. One fixed product GUID, one retained HICON, and one notification registration exist for the
runtime lifetime.

Every add is followed by `NIM_SETVERSION` with `NOTIFYICON_VERSION_4`. Mouse context activation,
primary selection, and keyboard selection open a native two-action popup menu. The tooltip and
window title are **Honk300 controls**; menu labels are **Configure Honk300…** and **Quit
Honk300**. Callback coordinates preserve signed virtual-desktop positions, focus is returned with
`NIM_SETFOCUS`, and the exact executable launches Configure in a new unelevated console without
shell interpolation.

The hidden top-level owner receives Explorer's registered `TaskbarCreated` broadcast. Its wndproc
only records a re-add request; the next ordinary runtime iteration re-adds the retained icon and
reapplies version 4. DPI/taskbar recreation therefore cannot create a second engine instance or
duplicate native owner. Initial registration or later restoration failure is explicit and
non-fatal: CLI/TUI/IPC controls remain available. Normal Drop deletes the shell item before
destroying the HWND and HICON.

### Linux StatusNotifierItem owner

Linux uses pinned `ksni` 0.3.6 with default Tokio disabled and the pure-Rust `async-io` plus
blocking facade. It adds no GTK, AppIndicator, XEmbed, compositor, or native widget dependency and
cross-checks under both GNU and musl targets. A retained service handle owns the D-Bus item for the
runtime lifetime; menu callbacks only send the shared enum over a standard channel to the normal
runtime loop.

The item declares an application-status category, active status, **Honk300 controls** title and
tooltip, and the two shared menu labels. It publishes an embedded ARGB pixmap, so portable
archives do not rely on icon-theme installation. Startup fails visibly but non-fatally when the
session bus, StatusNotifierWatcher, or StatusNotifierHost is absent. Later watcher loss is logged
and retained for recovery. Tray availability is not folded into overlay, cursor, foreign-window,
collect-window, presence, Accessibility, X11, XWayland, or native Wayland capability status.

Configure uses no shell. It prefers the proposed `xdg-terminal-exec` argument-vector interface
when present, then explicit common-terminal executable/argument pairs. Every path receives the
exact current executable and literal `config` argument. If no supported terminal is installed,
Configure reports that limitation and the running goose remains controllable through CLI/IPC.

### Shared asset and package ownership

`Assets/UI/honk300-status-goose.svg` remains the canonical source and
`honk300-status-goose@2x.png` remains its deterministic 36x36 transparent representation. Both
Windows and Linux binaries embed that PNG. Each backend applies the same bounded transform: the
source alpha becomes a white goose over an opaque dark-blue circular field, preserving
antialiasing and contrast on light and dark shells. Windows converts the result to a retained BGRA
HICON; Linux sends network-order ARGB. Neither performs runtime file lookup or SVG decoding.

All portable, MSI, EXE, bootstrap, and Debian forms therefore carry the pixels inside the exact
qualified binary. Native Debian packages additionally install the tracked PNG byte-for-byte at
`/usr/share/icons/hicolor/36x36/apps/honk300.png`, reference `Icon=honk300`, and let dpkg own its
removal. No installer writes into mutable user media, and uninstall preserves existing user-data
rules.

## Rejected alternatives

- `tray-icon` for both platforms: rejected because its Linux implementation requires GTK and
  AppIndicator/Ayatana native libraries and a GTK loop, expanding every supported Linux artifact.
- Legacy XEmbed fallback: rejected because it is X11-specific, does not solve native Wayland, and
  would turn one explicit unavailable state into a second unqualified integration.
- `assume_sni_available`: rejected because it can hide permanent watcher/host absence. Startup
  uses the library's explicit `Watcher` and `WontShow` failures.
- A native configuration window: rejected because it would duplicate schema, validation, status,
  reload, and terminal-protection behavior.
- `$SHELL -c`, interpolated `$TERMINAL`, or desktop-file command strings: rejected because exact
  executable/argument identity is available without parsing or injection risk.
- Immediate process exit from menu callbacks: rejected because it bypasses final-clear,
  prop cleanup, graceful locomotion, and singleton release.

## Consequences

- Windows and compatible Linux desktops gain the same discoverable Configure and graceful Quit
  model as macOS.
- Explorer restart and Linux watcher restart have explicit retained-lifetime recovery paths.
- Linux sessions without StatusNotifier support remain fully usable through CLI/TUI/IPC and say
  why the tray is absent.
- The Linux dependency graph grows by a pure-Rust D-Bus stack but retains the no-GTK GNU/musl
  packaging contract.
- All three native surfaces share a compile-time command type and one shutdown router, reducing
  the chance of platform behavior drift.

## Verification contract

- Unit tests prove the shared Configure/Quit routing, error separation, menu command emission,
  accessible tooltip, signed Windows callback coordinates, and contrasting embedded icon bytes.
- Windows qualification covers keyboard/menu accessibility, exact-executable TUI launch and
  restoration, graceful animated Quit, immediate singleton restart, `TaskbarCreated` recovery,
  DPI, available display topology, and installed/portable artifact identity. A hosted runner may
  claim the explicit unavailable path only when a second hidden owner using a stock icon also
  fails a minimal `Shell_NotifyIconW` control registration; if that independent probe succeeds,
  Honk300's registration failure remains fatal. This waiver is enabled only for the Windows ARM64
  GitHub-hosted qualification session and is recorded in its evidence. The Windows Server 2022
  release runner may record recovery as unobservable only after Honk300's fixed GUID was initially
  observable, its removal remained settled, the runtime logged a successful TaskbarCreated re-add,
  and a second hidden owner with a stock icon and fresh GUID independently also fails the same
  add/observe/delete/settle/re-add/observe control sequence. If that control recovers, Honk300's
  failure remains fatal. Windows Latest, local/self-hosted Windows, and the physical Alienware do
  not receive this recovery-observation waiver.
- Linux qualification covers StatusNotifier watcher/host registration, menu actions, terminal
  selection, explicit no-host degradation, X11 and native Wayland capability separation, GNU and
  musl builds, Debian icon ownership, and archive/package lifecycle.
- v1.1.0 remains gated on the complete all-platform exact-SHA candidate, ordinary main CI, atomic
  immutable publication, post-release installers, fresh-download hashes/updates, stable/latest,
  and production-site checks.
