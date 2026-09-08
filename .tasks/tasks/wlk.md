TT;DR: Implement the explicitly enabled KWin bridge and qualify it on real disposable KDE desktops.

## Why
The approved refinement requires KDE-first native Wayland window support, with separate
pointer permission and independently tested compositor adapters.

## Scope
Implement the KWin script, bounded same-user Rust bridge, live identity/terminal checks,
runtime capability mapping and explicit setup/removal. Portal input belongs to #wlp;
release integration follows #rlpi under #rkde. GNOME/Sway/Hyprland retain separate gates.
The authoritative sources are ADRs 0021/0041, upstream KWin APIs, current production
code and actual native compositor probes. Preserve ordinary reduced mode, saved config,
all foreign startup/integration state, immutable installers and terminal protection.

## Plan
First run a thin native KWin script exchange and bounded movement of a fixture-owned
window on real KDE 5/6. If that premise fails, record the native failure before adding
runtime or setup code. After two attempts without new evidence, add diagnostics or
revise the premise. Then connect the production Rust path and expand revocation tests.

## Impact
Explicitly enabled KDE integration can supply individual window/presence capabilities
without treating a desktop hint or portal grant as authority for unrelated operations.

## Acceptance
Actual KDE windows supply fresh identities and geometry. Bounded supported operations
reject terminal, stale, changed, disappeared and out-of-bounds targets. Disconnect,
disable and shutdown stop actions; setup and removal affect only this integration.
The user requires native desktop and every platform/package/public release gates;
this task owns the native implementation evidence and #rkde owns publication.

## Verification
- [ ] Native KDE 5/6 probes exercise actual window data, movement and protected targets.
- [ ] The real Rust runtime handles opt-in, identity, capability loss and graceful cleanup.
- [ ] Setup/removal and supported architecture/desktop checks preserve unrelated state.

## Status
Active in the isolated Wayland integration checkout. The second native probe starts both
compositors and creates their sockets, then queries the D-Bus identity before registration.
Wait for the actual service owner with a fixed deadline; add bus-name and compositor-log
diagnostics for a timeout. Native API feasibility remains unverified; no public integration
is enabled or advertised.

## Activity
- 2026-09-07: Run 34188470703 reaches socket startup on both KWin generations, then fails
  with NameHasNoOwner before the script is loaded. Native logs show no startup crash.
  Require both socket and D-Bus registration, preserving a bounded timeout and its diagnostics.
- 2026-09-07: Run 34187445365 failed at KWin exec with EPERM on both distributions,
  before any script/window assertion. Preserve the file capability metadata, drop the
  private container's realtime request and return artifact ownership to the runner after
  teardown so failure evidence remains readable.
- 2026-09-07: Recovered the historical board-only implementation card, linked it to the
  staged publication task, and started the earliest real KWin API falsifier.
