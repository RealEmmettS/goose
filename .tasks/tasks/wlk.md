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
Active in the isolated Wayland integration checkout. KWin 6.3.6 now passes the actual script
exchange, native identity, bounded movement, terminal/stale/excessive movement refusals and
disable. KWin 5.27 lacks the newer stacking-order property; use its documented clientList
without claiming occlusion order. The repeat native probe remains open, followed by the
Rust bridge and setup. No public integration is enabled or advertised.

## Activity
- 2026-09-08: The new Rust endpoint compiles and passes strict native Linux checks.
  Extend the actual compositor probe through that endpoint: real movement, untrusted
  peer refusal, expired observation, explicit reconnect and stop. Pin the script to
  one unique bridge owner with a bounded watchdog; native transport results are pending.
- 2026-09-08: Both native KWin generations pass in run 34189071395. Record ADR 0044;
  implement the bounded Rust state and authenticated session-bus endpoint. Six new
  production-state tests pass locally, with actual native transport qualification next.
- 2026-09-08: Run 34188879003 passes the entire earliest KWin 6.3.6 native premise.
  KWin 5.27.11 reaches the script but sends no frame because stackingOrder is unavailable.
  Upstream Plasma/5.27 workspace_wrapper.h exposes clientList instead. Add that compatibility
  source while explicitly withholding any stacking-order inference from the older list.
- 2026-09-07: Run 34188470703 reaches socket startup on both KWin generations, then fails
  with NameHasNoOwner before the script is loaded. Native logs show no startup crash.
  Require both socket and D-Bus registration, preserving a bounded timeout and its diagnostics.
- 2026-09-07: Run 34187445365 failed at KWin exec with EPERM on both distributions,
  before any script/window assertion. Preserve the file capability metadata, drop the
  private container's realtime request and return artifact ownership to the runner after
  teardown so failure evidence remains readable.
- 2026-09-07: Recovered the historical board-only implementation card, linked it to the
  staged publication task, and started the earliest real KWin API falsifier.
