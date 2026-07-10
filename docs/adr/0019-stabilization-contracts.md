# ADR 0019 — v0.3.0 Stabilization Contracts

- Status: Accepted (2026-07-10)
- Relates to: ADR 0014 (renderer), ADR 0015 (platform safety), ADR 0016 (idle life), ADR 0018
  (distribution).

## Context

The pre-v0.3 implementation had a healthy Rust test gate but several user-visible contracts were
implicit or unsafe: malformed configuration could be replaced by defaults, a newer schema could
be rewritten by an older binary, damage bounds accumulated, long-running `f32` clocks lost
precision, display gaps were treated as usable desktop, TUI IPC could block rendering, stopped
runtimes looked unsupported, and platform loops had drifted in tick/reload/damage ordering.

This release is a correctness and distribution-readiness pass. It does not add a new product
surface.

## Decision

### Configuration and control

- Config schema v2 has typed load outcomes: `Missing`, `Loaded`, `Malformed`, and
  `UnsupportedVersion`. Only `Missing` may be created by ordinary setup. Malformed or newer
  files remain untouched; `setup --reset` makes a timestamped backup before replacing one.
- v1 migration maps either legacy mute control to `audio.enabled = false`, removes the duplicate
  `behavior.silence_sounds`, and drops the nonfunctional `stop_radius` setting. Unknown valid
  keys survive load/save and floats serialize stably.
- Saves are atomic in the target directory. A valid config symlink updates its regular-file
  target; dangling or non-file targets fail closed.
- Backend selection (including native Wayland) is restart-required. Live reload reports the
  exact rejected fields rather than pretending they applied.
- Stopped runtime capability state is `Unprobed`, not `Unsupported`. Missing-runtime status has a
  500 ms upper bound.
- The TUI performs IPC on a worker, saves dirty configuration before Start, waits up to ten
  seconds for readiness, and preserves the actual startup error. Terminal restoration guards
  exist before fallible raw-mode work. Small terminals show an explicit size message; normal
  80x24 layouts keep controls and wrapped/scrollable errors reachable.

### Engine, behavior, and rendering

- Simulation time and deadlines are `f64`; only periodic visual phase inputs wrap. A deterministic
  fourteen-day simulation is a regression gate.
- `DesktopLayout` represents real monitor regions and their adjacency. World target sampling and
  clamping exclude L-shaped gaps, and `World::apply_layout` safely rebases active tasks during
  hotplug.
- The renderer retains the current visual bound separately from the previous frame. Damage is
  their union for one present, while only the current bound becomes next-frame history. Scratch
  pixmaps, layers, conversion storage, and platform damage buffers are reused.
- Shared `RuntimeCore` owns fixed-tick clocking, command/config ordering, reload decisions, and
  damage/present cadence. Native event pumps, window systems, permissions, and OS operations stay
  in platform crates.
- Concept C is the shipped side silhouette: distinct back-neck and throat curves, broad neck
  base, restrained long-neck sweep, and oval head. The existing rig, palette, wings, beak, legs,
  facings, and tucked/raised poses remain intact. Crossfades use complementary alpha.
- Puddle-hop return motion is continuous vertically. Manners cancel delayed pranks. Notes and
  memes cancel independently when disabled, on-hour/Hyper honks are deduplicated, and decks
  refresh when runtime capabilities change.
- Renderer goldens are committed inputs; missing goldens fail even in blessing mode.
- Built-in notes and PNGs use explicit descriptors, header validation, lazy decode, and a
  two-entry image cache. User notes and memes merge from external platform media directories.

### Platform boundaries

- Windows retains one PMv2 layered overlay per monitor; topology/DPI reconciliation adds and
  removes only affected windows. The named pipe allows the current user SID and SYSTEM, rejects
  remote clients, and uses bounded I/O. Note typing is target-specific UI Automation only—there
  is no global `SendInput` fallback.
- macOS converts AppKit coordinates through the main-display coordinate space, distinguishes
  clicks from actual foreign-window drags, reconciles topology, and keeps the AppKit pump/present
  path bounded. Audio is in-process and transient capability failures can recover.
- X11/XWayland is the full-mischief Linux default and fails closed unless a composited ARGB visual
  and input shaping are available. Foreign-window work is nonblocking and terminals remain
  protected.
- Native Wayland remains explicit reduced mode. It uses prepare-read/poll/read/dispatch,
  per-output layer surfaces, configure/close/hotplug, integer and fractional scale handling,
  empty input regions, no keyboard interactivity, and a released-buffer pool capped at three
  real buffers per output. Cursor and foreign-window mischief report unsupported.
- Unix IPC lives in a UID-owned `0700` directory with a `0600` socket and validates peers using
  platform credentials. GNU Linux audio is in-process. The musl fallback discovers players
  through `PATH`, reaps children, and has bounded concurrency.
- Until a compatible crates.io release is available, the released `wayland-scanner` 0.31.10 API
  is vendored with upstream revision `d07c4f91f28b42e5a485823ffd9d8d5a210b1053`'s `quick-xml`
  security fix. The complete git snapshot is not pinned because it also contains unreleased
  breaking Wayland API changes incompatible with the released client stack. The patch is removed
  when crates.io ships the fix on the compatible API line.

## Consequences

- Recovery is explicit: corrupt or future configuration is never silently destroyed.
- Multi-monitor behavior follows actual screen geometry and can survive hotplug without a process
  restart.
- Damage and allocation stay proportional to the current visual rather than session length.
- Platform-specific code retains necessary native control while shared runtime semantics are
  testable once.
- Native Wayland is useful and distribution-stable without claiming capabilities the protocol
  intentionally denies.

## Verification

- Config/TUI tests cover all four load states, migration/reset/symlink behavior, restart-required
  reload, 72x20/80x24 layouts, resizing, dead/slow IPC, save-before-start, readiness failure, and
  terminal restoration.
- Engine tests cover fourteen simulated days, monitor gaps/hotplug, bounded damage, complementary
  crossfade, behavior cancellation/deduplication, strict goldens, and allocation reuse.
- Platform tests cover Windows monitor/DPI/pipe/UIA policy, macOS coordinate/drag/permission
  adapters, Unix peer ownership, X11 fail-closed prerequisites, and native Wayland output/scale/
  buffer state. Hosted Linux smoke uses headless Sway because the runtime's required layer-shell
  protocol is not a Weston protocol; it creates multiple virtual outputs with integer and
  fractional scales. Hosted platform smoke remains the authority for APIs unavailable on Windows.
- Hands-on pre-granted macOS Accessibility upgrade evidence remains tracked by `#m16r`; it is not
  a v0.3.1 blocker and no release text promises grant persistence.
