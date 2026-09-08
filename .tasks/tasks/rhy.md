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
- [x] Rust controls and native settings preserve drafts, ownership and live cancellation.
- [ ] Required source, architecture, package, immutable-publication and website checks pass.

## Status
Active. Run 34231759048 passes the complete actual runtime and native settings
lifecycle on Hyprland 0.53.3/0.55.2, each on native x64/ARM64. Repeated setup retains
one exact worker and consent; live revocation, engine fullscreen/config manners,
drafts, replaced sockets and graceful/crash recovery pass. Qualify the actual
rendered settings and owned note/image delivery before freezing the release source.

## Activity
- 2026-09-08 — Run 34235234824 passes all four native settings and prop lifecycles. Review all settings/full-goose/picture/note captures: complete goose and uncropped pictures are visible, but one note capture precedes text paint despite valid AT-SPI text. Strengthen the real compositor capture to require visible body ink before saving its evidence; retain every existing geometry, alpha and text-readback assertion.
- 2026-09-08 — Native screenshot capture works and the complete runtime lifecycle passes again. The reused prop fixture stops before launch because the minimal container lacks gdbus; install its distribution package. Refresh actual native capability text and capture the entry sequence after closing settings for complete goose review. Prepare matching Rust/SDK metadata and required same-source Hyprland publication gates for v1.8.0; all local versioned Rust/Python/SDK checks pass.
- 2026-09-08 — All four integrated desktop lanes pass at b9219f3 in run 34231759048, including every exact worker assertion. Add actual compositor screenshots and reuse the production prop/engine lifecycle fixture on these specific desktops; prior labwc evidence does not stand in for Hyprland.
- 2026-09-08 — The retained-worker fix passes both older desktop lifecycles, including exact cleanup. The newer ARM fixture fails earlier while unnecessarily launching a second connection/version handshake during fullscreen; diagnostics record a correct bounded timeout while the retained production worker already observes fullscreen. Keep the original authenticated worker across this transition and require its fresh actual fullscreen window identity, PID and geometry. Initial direct transport and impostor checks remain, and no production deadline, identity check or stale-frame rule changes.
- 2026-09-08 — Integrated run 34229133252 passes the complete newer x64 lifecycle and exposes repeated native setup replacing a worker during transient fullscreen recovery on both ARM desktops. Retain any still-running bounded worker with identical consent; freshness continues to control capability independently. The older x64 host reaches external revocation and fails the exact worker-removal check; retain kernel thread identities/state around every assertion to identify that separate failure without weakening the check.
- 2026-09-08 — Connect explicit Hyprland setup/removal to its own private consent record, versioned control protocol, native Wayland runtime and real Native SDK dialogs. Preserve drafts, unknown configuration and unrelated compositor consent. Add native lifecycle qualification for the actual runtime and GUI: default-off behavior, fullscreen/config manners, identical worker ownership after repeated setup, external removal, replaced socket refusal, explicit reconnection and graceful/crash recovery.
- 2026-09-08 — Native diagnostics show all older-compositor queries stall during fullscreen configure while GTK still has its old allocation. Add a retained observation worker that withdraws expired data and retries only timeouts against the same pinned owner; every identity, permission, disconnect or decoding failure stays terminal. Exercise the actual worker across native fullscreen entry/exit and test that expired frames clear, recovery uses one source and permission loss ends it.
- 2026-09-08 — Fixed-batch run 34225958368 passes both newer desktops and still fails the older pair specifically in the fullscreen snapshot. Keep the failed gate and collect individual native client/monitor/version/batch timings plus actual configured GTK geometry before deciding the compatibility boundary.
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
