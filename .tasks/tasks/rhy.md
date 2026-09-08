TT;DR: Ship a separately qualified, explicitly enabled Hyprland adapter.

## Why
Hyprland's own IPC and version differences require evidence independent of KDE and Sway.

## Scope
Bounded native window observations and fullscreen awareness through supported Hyprland
operations. ADR 0047 keeps movement and owned-prop positioning unavailable without
authoritative active-drag observations. Keep setup/removal explicit and permissions, fullscreen observation, movement
and pointer access separate. Preserve the user's compositor configuration, terminal exclusions,
saved settings and installation lifecycle. No unverified pointer or desktop-parity claim.

## Plan
First prove native socket ownership, exact window identities and a real bounded move on
x64 and ARM64. Retain strict response and connection deadlines: the upstream request socket
must close promptly after each transaction. Inspect the earliest failed premise within two
native attempts before adding Rust runtime and GUI controls. Publication follows #rkde.

## Acceptance
The running goose uses individually proven operations only after setup, cancels actions on
disconnect or changed identity, and removes only its own integration state. Supported versions
earn their own new immutable release and verified website claims through all required gates.

## Evidence
Authoritative IPC reference: https://wiki.hypr.land/IPC/
Exact desktop versions, request semantics and native behavior require direct qualification.

## Verification
- [x] Actual supported Hyprland versions on x64/ARM64 pass native movement and refusal scenarios.
- [ ] Rust controls and native settings preserve drafts, ownership and live cancellation.
- [ ] Required source, architecture, package, immutable-publication and website checks pass.

## Status
Active. All four native premise lanes pass at source 38d1f23 in run 34222607423.
Qualify the actual bounded Rust observation transport next. Runtime/settings and
public support remain pending; native movement does not prove active-drag safety.

## Activity
- 2026-09-08 — Pumping GTK did not resolve the older compositor's fullscreen timeout; the earlier cause attribution was unproven. Replace three socket round trips with the fixed read-only batch supported by both actual upstream versions, retain the 250 ms total bound and exact identity checks, and distinguish version versus snapshot failures. Add incomplete, changed-inventory and trailing-data regressions before repeating native qualification.
- 2026-09-08 — Native Rust transport/decoder and impostor refusal pass both 0.55 architectures. Both 0.53 hosts pass initial observation and fail only when subprocess.run blocks the fixture's own fullscreen configure acknowledgements. Pump GTK while the independent probe executes; retain the exact Rust request and process deadlines.
- 2026-09-08 — Inspect all four passing native premise results from run 34222607423. Record ADR 0047 and implement bounded authenticated Rust observations only: public IPC has no authoritative active user-drag state, so native move capability is not promoted into production authority. Exercise actual decoder/transport output and reject an unrelated listener before sending it any request.
- 2026-09-08 — Native Hyprland 0.55 ARM64 passes the full premise. Both 0.53 hosts now reach ordinary GTK mapping and fail specifically while changing tiling into floating mode; the compositor logs the operation and late reply. Use fixed-size native fixture clients that float at map time, preserving the same actual placement/refusal tests and bounded response deadline without requiring this unrelated layout snapshot.
- 2026-09-08 — The kernel-identified vkms device boots all four real compositors. Actual requests now reach monitor/window operations, but bounded replies time out. Keep the GTK client event loop active during IPC and retry read-only startup waits; preserve each exchange's deadline and EOF check and record transaction timing/bytes for the next native run.
- 2026-09-08 — The third attempt fails in device selection before launching the
  compositor: vkms has no ordinary platform driver symlink on these kernels. Query
  the real DRM version ioctl instead, record all returned driver names, and accept
  exactly one vkms device. No graphics identity is inferred from a card number.
- 2026-09-08 — The second attempt creates the virtual DRM device successfully, but
  Aquamarine chooses the runner's unrelated Hyper-V adapter as its primary allocator.
  Pass only the system-identified vkms device to the private container, select it
  explicitly, and use a named headless output with full backend logging.
- 2026-09-08 — Inspect the first four-lane failure before repeating: the compositor
  lacks a DRM allocator and the Lua generation exceeds the native socket path bound.
  Add an actual disposable virtual graphics device and normal seat service, shorten
  the private runtime path, and preserve backend diagnostics before the second premise.
- 2026-09-08 — Exercise the native headless interface on Ubuntu and Debian x64/ARM64.
  Use each generation's actual configuration and dispatcher syntax, bound each socket
  transaction, attest the launched peer, and touch only exact fixture-owned windows.
- 2026-09-08 — Split the approved follow-on scope into a separately verifiable adapter task.
