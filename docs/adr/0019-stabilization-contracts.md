# ADR 0019 — v0.3.0 Stabilization Contracts

- Status: Accepted (2026-07-10; amended 2026-07-13)
- Relates to: ADR 0014 (renderer), ADR 0015 (platform safety), ADR 0016 (idle life), ADR 0018
  (distribution).

## Context

The pre-v0.3 implementation had a healthy Rust test gate but several user-visible contracts were
implicit or unsafe: malformed configuration could be replaced by defaults, a newer schema could
be rewritten by an older binary, damage bounds accumulated, long-running `f32` clocks lost
precision, display gaps were treated as usable desktop, TUI IPC could block rendering, stopped
runtimes looked unsupported, and platform loops had drifted in tick/reload/damage ordering.

The original release was a correctness and distribution-readiness pass without a new control
surface. The v0.3.3 amendment adds shared exposed-edge locomotion and a collect-window close
reaction inside the existing engine/task/config/capability boundaries; it introduces no new
settings schema, native settings UI, or platform privilege.

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
- Install, update, and uninstall mutations must first stop any active runtime and retain the real
  process singleton for the complete owned-file transaction. A probe which immediately drops the
  singleton is insufficient because a concurrent start can reacquire it before replacement.
  In-process lifecycle commands hold a `LifecycleLease`; exact-tag Unix installation transfers
  the lease to a staged binary whose stdin/FIFO lifetime is owned by the installer; Windows
  bootstrap/update uses the manifest-hashed portable binary, and Windows uninstall hands off to a
  private temporary copy before the installed process exits or any integration is mutated.
- Unix termination traps pass explicit nonzero HUP/INT/TERM statuses into cleanup so an interrupt
  after activation rolls back instead of committing. Windows bootstraps and generated updaters
  retain read-only handles which deny replacement while size and SHA-256 are verified from that
  same stream and while the artifact is executed; ambient environment state never bypasses lease
  acquisition. Machine-wide install/uninstall checks each exact installed executable through
  Restart Manager across sessions and rejects reboot-deferred results. Deferred uninstall keeps an
  armed child guard which kills and waits on every pre-READY error, disarming only after the exact
  handshake is observed while the helper is still live.
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
- Ordinary roaming crosses touching-monitor seams naturally. Four ordinary wander entries and
  one exposed-edge wrap entry keep the Pac-Man-style flourish to 20% of baseline deck draws;
  the goose may relocate to the opposite exposed edge only after its complete rendered pose is
  outside every real monitor, then walks back onscreen. Deliberate puddle/prank errands return
  through their own departure edge and never wrap.
- Startup is staged fully beyond a real exposed edge and enters under locomotion. Stop/exit/quit
  cancels transient platform work, chooses the nearest currently exposed edge, and keeps the
  shared simulation and presentation loops alive until the full pose is hidden. Exit speed scales
  within the existing Run/Charge envelope and clients use a finite singleton-release wait, so
  shutdown remains animated, bounded, and safe for an immediate restart.
- When a native note or meme window is closed by its user, a dedicated random stream selects an
  annoyed reaction with 30% probability. The visible reaction always remains safe; only then may
  it chain the existing bounded cursor nab, and only when the backend, live permission/pointer,
  configuration, and manners allow it. Programmatic close/cleanup is never a trigger. Linux has
  no collect-window implementation, so it has no native close event to trigger this behavior.
- Renderer goldens are committed inputs; missing goldens fail even in blessing mode.
- Built-in notes and PNGs use explicit descriptors, header validation, lazy decode, and a
  two-entry image cache. User notes and memes merge from external platform media directories.

### Platform boundaries

- Windows retains one PMv2 layered overlay per monitor; topology/DPI reconciliation adds and
  removes only affected windows. The named pipe allows the current user SID and SYSTEM, rejects
  remote clients, and uses bounded I/O. Note typing is target-specific UI Automation only—there
  is no global `SendInput` fallback. Native collect routing prefers the active request-id/kind
  tuple over older lingering props while still draining user/program close evidence exactly once.
- macOS converts AppKit coordinates through the main-display coordinate space, distinguishes
  clicks from actual foreign-window drags, reconciles topology, and keeps the AppKit pump/present
  path bounded. Its native collect controller follows the same active typed-request rule so an
  older visible note or meme cannot starve a newer task. Audio is in-process and transient
  capability failures can recover.
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
- Lifecycle mutations cannot race an autostart or manual launch after shutdown proof; failure to
  acquire or transfer exclusive ownership occurs before managed files change.
- Native Wayland is useful and distribution-stable without claiming capabilities the protocol
  intentionally denies.

## Verification

- Config/TUI tests cover all four load states, migration/reset/symlink behavior, restart-required
  reload, 72x20/80x24 layouts, resizing, dead/slow IPC, save-before-start, readiness failure, and
  terminal restoration.
- Engine tests cover fourteen simulated days, monitor gaps/hotplug, bounded damage, complementary
  crossfade, exposed-edge selection, touching-monitor continuity, fully hidden wrap/start/exit,
  non-wrapping errands, weighted wrap frequency, user-versus-program close provenance, the 30%
  reaction distribution, manners/capability gating, behavior cancellation/deduplication, strict
  goldens, and allocation reuse.
- Platform tests cover Windows monitor/DPI/pipe/UIA policy, macOS coordinate/drag/permission
  adapters, Unix peer ownership, X11 fail-closed prerequisites, and native Wayland output/scale/
  buffer state. Hosted Linux smoke uses headless Sway because the runtime's required layer-shell
  protocol is not a Weston protocol; it creates multiple virtual outputs with integer and
  fractional scales. Windows native smoke hashes exact x64 and ARM64 builds, freezes one real
  layered surface across controlled light/dark captures, and requires independent semantic
  body/shade/outline/wing ownership, beak/legs, asymmetric color, shadow/edge alpha, and lifecycle
  proof. Native ARM64 execution uses the fail-closed `windows-11-arm` candidate job rather than
  treating a cross-build as runtime qualification. Hosted platform smoke remains the authority
  for native APIs unavailable on the current development host.
- Hands-on pre-granted macOS Accessibility upgrade evidence remains tracked by `#m16r`; it is not
  a v0.3.1 blocker and no release text promises grant persistence.
