# ADR 0015 — Reliability and Platform-Safety Contract (R1)

- Status: Accepted (2026-07-07)
- Amends: ADR 0013 (uninstall semantics only — see §6)
- Context: the 2026-07-07 full-repo evaluation (Fable) ranked ten concrete reliability
  defects across the platform backends; this ADR records the fixes and the behavior
  contracts they establish. Implemented by three parallel subagent worktrees, reviewed
  and integrated by the orchestrator.

## Decisions

### 1. Windows is Per-Monitor-V2 DPI aware

`init_dpi_awareness()` calls `SetProcessDpiAwarenessContext(PER_MONITOR_AWARE_V2)`
before any HWND or monitor enumeration (an "already set" `ERROR_ACCESS_DENIED` is
success; other failures log once and continue). `wndproc` handles `WM_DPICHANGED` and
`WM_DISPLAYCHANGE` by flagging monitors-dirty; the runtime then re-enumerates
monitors, reconciles per-monitor overlay windows (reusing HWNDs when the count is
unchanged), refreshes world bounds, recreates the foreign-window watcher, and forces a
full repaint. Contract: every coordinate the process sees (cursor, window rects,
monitor rects, layered-window destinations) is physical pixels in one signed
virtual-desktop space — the engine's coordinate model, now guaranteed rather than
accidental. (Closes plan risk W_dpi, honk300_plan.md:772.)

### 2. Collect-window work never blocks the sim thread

The Notepad path is a polled state machine (`Pending { deadline, next_poll } → Ready`):
spawn returns immediately, window discovery is a throttled (~40 ms) single-pass poll
from the per-frame drive, and a 3 s deadline reclaims the child. Typing is deferred
(`PendingType`, 30 ms settle / 1.5 s give-up) and fires only when the spawned window
is foreground; it is now **best-effort** — a foreground steal skips the note instead
of latching collect-window off (deliberate semantics change from M9, judged more
robust). Contract: no sleep >16 ms on the sim thread, so IPC (`status`/`do`/`reload`,
2 s timeout) stays responsive during collects.

### 3. Spawned Notepad windows never outlive the goose

The child process handle is retained; `close()`, expiry, and `Drop` post `WM_CLOSE`,
wait a bounded ~150 ms grace, then `TerminateProcess` the **tracked child only** —
never a pre-existing user Notepad. `honk300 stop` leaves no notepad.exe zombie.

### 4. Unix single-instance uses an advisory flock

`Singleton` holds `flock(LOCK_EX | LOCK_NB)` on the lock file for the process
lifetime; the kernel releases it on any death, so a crash/SIGKILL can no longer leave
a false "already running" state (the old `create_new` marker file did). `WOULDBLOCK`
⇒ AlreadyRunning. The lock file is **intentionally never unlinked** (unlinking would
let a second process lock a fresh inode at the same path). Stale *socket* files are
still unlinked before bind, which is safe because bind happens only after the flock is
held. Dependency: `rustix` (fs feature, unix-only) in honk-control — no unsafe.

### 5. macOS pump drains events; presents use AppKit-owned pixels

`Overlay::pump` drains `nextEventMatchingMask(…distantPast…)` → `sendEvent` until
empty (collect-window close buttons now work), then `updateWindows()`.
`NSBitmapImageRep`s are allocated with nil planes so **AppKit owns the pixel
storage**, and rows are copied in per frame honoring `bytesPerRow` — the previous rep
aliased a reused/reallocating `Vec` (tearing/use-after-free risk).

### 6. Linux never silently renders nothing; overlay state is reported everywhere

Overlay-creation failure is a **loud, fatal start error** naming the tried backend —
the no-op `HeadlessOverlay` is opt-in via `HONK300_ALLOW_HEADLESS=1` (CI smoke keeps
using real Xvfb/sway and does **not** set it). A new `overlay` capability
(`CapabilityStatus`) rides the status protocol (`V=` key, required field like all
others), reported by all three runtimes, printed by `honk300 status`, and shown as an
"Overlay" row in the TUI Status tab: Supported (visible), Failed (fallback after a
real attempt), Unsupported (no display server). X11 perf nits: the XFixes input
region is cached and re-applied only when the effective rect changes; the event mask
is set once at window creation.

### 7. Plain `uninstall` preserves user content (amends ADR 0013)

Non-purge uninstall relocates the user-provenance meme/note directories to a
timestamped `preserved-<ts>` folder under the backups root and prints the location
(nothing printed when the user added no content); `--purge` keeps its existing
backup-then-remove semantics. ADR 0013's statement that plain uninstall removes the
install tree wholesale is amended accordingly — user-supplied content is never
silently deleted.

## Consequences

- Mixed-DPI/mixed-scale monitor setups get a crisp, correctly-placed overlay; DPI or
  display topology changes no longer require a restart.
- `status` stays responsive during Notepad collects; typing may occasionally skip a
  note under focus contention instead of disabling the feature.
- On a broken Linux display, `start` exits nonzero with a clear message instead of
  zombie-running invisibly; anyone scripting a headless run must opt in explicitly.
- The status wire format gains a required `V=` field (client+server ship in one
  binary, so no cross-version concern).
- A follow-up may remove the now-redundant `ImageWindow._buffer` retention on macOS.

## Verification

- Full local gate green on the integrated branch: fmt --check, clippy --all-targets
  -D warnings, cargo test --workspace (223 tests, incl. new flock crash-survival,
  overlay-capability, pending-notepad, and non-purge-preservation tests).
- Cross-target checks: aarch64-pc-windows-msvc; x86_64/aarch64-apple-darwin;
  x86_64/aarch64-unknown-linux-gnu.
- Pending before round close: Windows smoke — `status` responsive mid-collect, no
  notepad zombie after stop, DPI visual check; WSLg kill-9/restart and loud headless
  failure checks (WSL distro currently not installed on this machine — deferred to
  CI smoke which runs the real Xvfb/sway paths).
