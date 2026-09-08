TT;DR: Ship a separately qualified, explicitly enabled Hyprland adapter.

## Why
Hyprland's own IPC and version differences require evidence independent of KDE and Sway.

## Scope
Bounded native window observations and owned-prop positioning through supported Hyprland
operations. Keep setup/removal explicit and permissions, fullscreen observation, movement
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
- [ ] Actual supported Hyprland versions on x64/ARM64 pass native movement and refusal scenarios.
- [ ] Rust controls and native settings preserve drafts, ownership and live cancellation.
- [ ] Required source, architecture, package, immutable-publication and website checks pass.

## Status
Active. Begin native socket/window qualification on separately packaged Hyprland
generations, with private configuration and no production runtime claims yet.

## Activity
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
