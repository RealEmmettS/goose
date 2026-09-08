# ADR 0043: bounded owned Linux props through the existing companion

Date: 2026-09-07
Status: accepted; native X11/labwc implementation qualified, release integration pending

## Context

ADR 0041 approves Linux-owned notes and images and permits normal compositor placement
on native Wayland. The Rust CLI remains useful without GTK, while official packages
already include the Native SDK settings executable and its native GTK4 dependency.
Windows/macOS notes remain open after delivery; their current controller maps do not
bound the number of retained windows. That must not be copied into the Linux host.

## Decision

- Reuse the exact installed `honk300-settings` payload in a separate internal
  `--owned-props` process mode on Linux. Its ordinary settings UI remains Native SDK/Zig.
  A small GTK4 prop host uses native text/picture widgets and no webview. The mode has no
  settings editor, updater, tray, startup mechanism or installation authority.
- Rust verifies the companion using the existing installation receipt boundary, launches
  one private child, validates and bounds messages, and retains its lifetime. Stdin EOF,
  shutdown and a failed protocol end only the child and its owned windows. Missing GTK
  or a companion failure leaves CLI/TUI and overlay behavior usable with honest capability
  reporting.
- Keep at most eight owned props per runtime on every supported native backend. A full
  set refuses new admission as temporary busy and recovers after a close. Existing notes
  are never discarded to make room. An admitted delivery continues at the limit.
- Pass opaque ids, bounded UTF-8 text and already fitted pixels through a versioned
  private protocol. Do not pass arbitrary window ids, shell commands or user filesystem
  paths. A child controls only GTK objects in its own bounded registry.
- On X11, including XWayland, use the GTK surface's own XID for bounded positioning and
  input shaping. On native Wayland, use normal toplevel placement with no global position,
  pointer or foreign-window claim. The engine completes a placement-only delivery without
  pretending to drag a window at an invented desktop coordinate.
- Preserve the existing monitor-relative hard ceiling, full-image downscale and no-upscale
  rules. Include the owned close control within the fitted outer dimensions. Distinguish
  user close from program cleanup; only the former enters the existing reaction policy.

## Compatibility and limits

The helper is already an immutable, independently verified package payload, so this adds
no separately installed executable or lifecycle owner. The Rust runtime and Zig decoder
both reject unknown/oversized malformed protocol input. Configuration does not gain an
ambient permission bypass.

GTK initialization uses [`gtk_init_check`](https://docs.gtk.org/gtk4/func.init_check.html)
so missing display support returns an error. The X11 adapter is explicit because
[`gdk_x11_surface_get_xid`](https://docs.gtk.org/gdk4-x11/method.X11Surface.get_xid.html)
is platform-specific and deprecated in newer GTK4; its continued availability is a native
build/runtime gate, not evidence of a portable Wayland operation. Input regions use the
documented [`GdkSurface` API](https://docs.gtk.org/gdk4/method.Surface.set_input_region.html).

## Session reporting

An additive `HONK300/1 SESSION` request reports the live overlay backend, a closed-set
desktop hint and the separate owned-prop positioning capability. Existing `STATUS`
frames are unchanged, so old clients remain usable and new clients tolerate an old
runtime declining the optional detail. CLI, TUI and the Rust settings service use the
same runtime response; they do not infer runtime state from the caller's environment.

`XDG_CURRENT_DESKTOP` is a bounded, colon-separated hint according to the
[desktop-entry specification](https://xdg.pages.freedesktop.org/xdg-specs/desktop-entry/latest/recognized-keys.html).
It never enables an integration or authorizes a window/pointer operation. An X11
overlay in an environment reporting a Wayland session is labeled accordingly; this
does not claim a native Wayland connection. Unknown values are not echoed into UI.
On a pure Wayland desktop, a launch without the existing command/config opt-in returns
an actionable explanation before creating an overlay or prop child. Merely finding a
Wayland socket no longer silently selects reduced native mode.
Full-screen/DND observations remain explicitly unsupported until a qualified observer
supplies them; a saved manners toggle is not evidence of desktop observation.

## Qualification evidence

Native GNU x64/ARM64 run [34186038608](https://github.com/RealEmmettS/goose/actions/runs/34186038608)
passes real X11 engine delivery and labwc 0.7.1 normal placement, scaled complete
image pixels, native text/actions, malformed-input cleanup, capacity recovery,
retained child failure and graceful shutdown. ARM64 captures were visually reviewed.
Official GTK-baseline/musl payload gates remain part of release qualification.
[Pi guidance](../raspberry-pi.md) retains the physical acceptance boundary.

Exercise actual child protocol, note/image fit, user and program closes, capacity
exhaustion/recovery, disconnect, stale ids, display change and graceful shutdown.
Run GTK behavior under Xvfb and labwc on native Linux architectures, with exact GNU/musl
payload checks. Stage-two publication waits for stage one and the complete existing
release gates. Physical Pi performance remains open under the user's approved experimental
support boundary.
