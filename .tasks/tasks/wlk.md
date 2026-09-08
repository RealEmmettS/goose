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
- [x] Native KDE 5/6 probes exercise actual window data, movement and protected targets.
- [x] The real Rust runtime handles opt-in, identity, capability loss and graceful cleanup.
- [ ] Setup/removal and supported architecture/desktop checks preserve unrelated state.

## Status
Run 34201107432 proves actual owned-note movement on both x64 KDE generations, but
both ARM lanes stop after less than the required four pixels despite correct identity.
The observer also reports sparse frames under the unoptimized runtime. Qualify the real
release build next to distinguish debug rendering load from a production cancellation
defect; preserve the 250 ms expiry and actual movement/revocation requirements.
The full production runtime and setup path passed all four KWin 5/6 x64/ARM64 lanes
in 34195263902. Evidence includes default-off state, separate capabilities, native Rust
settings-service status, live and external removal, crash recovery, graceful cleanup,
stopped removal and unchanged configuration/foreign files. Guided native UI and package
integration remain open. Pointer control is a separate #wlp gate.
Sealed activation and cleanup passed all four native KWin lanes in 34192401152.
The real runtime now consumes explicit private consent, reports distinct live capabilities,
and clears engine observations on removal or lost frames. CLI/native-settings setup and
removal preserve drafts and configuration. New actual-runtime and crash-recovery probes
are pending; portal control and owned-prop positioning remain unsupported.
The actual Rust transport and KWin 5.27/6.3 pass in 34190701777: exact identity,
bounded native movement, terminal/stale/excessive refusal, untrusted peer rejection,
expiry after disconnect, explicit reconnect and stop. The source includes the qualified
Linux implementation and final first-stage review corrections. Current desktop/activity,
fullscreen and actual user-drag boundaries are now implemented with focused Rust tests;
expanded native x64/ARM64 probes are next. Runtime and explicit setup remain open.
No public integration is enabled or advertised.

## Activity
- 2026-09-08: Correct note identity and off-center placement are now observed in all lanes. Both x64 movement checks pass; both ARM checks remain failures. Switch the native runtime and bridge to release builds before repeating, without increasing any freshness deadline or lowering the movement threshold.
- 2026-09-08: Run 34199983732 proves the application-id correction and actual owned movement/revocation on KDE 5 x64. Other approaches can pick up the compositor-centered note already at the engine's center release point, giving no required drag distance. Establish and observe an off-center initial fixture placement before measuring only subsequent engine-driven movement; keep all ownership and revocation assertions.
- 2026-09-08: Native observer evidence identifies the actual child-owned note but its app id remains GTK Application. Upstream GDK confirms the setter ignores pre-map calls. Move the exact per-prop identity assignment after GTK mapping and repeat the actual movement/revocation check.
- 2026-09-08: Native movement run 34198076323 compiles and starts the actual prop host but cannot identify the note. Retain bounded native observer frames, assert the actual note's application token and owned parent process, and use a monotonic frame counter for revocation timing. Independent runtime/UI/portal results remain failures unless every required qualifier passes.
- 2026-09-08: All four native settings lanes pass real AT-SPI consent/cancel/remove/draft preservation in 34196877360. Add KWin placement for actual process-owned props using a per-window application token, unreaped child identity, matching geometry and existing bounded movement checks. A new native observer must prove engine-driven movement and retained-note stability after revocation. Local production-helper ownership and engine cancellation regressions pass.
- 2026-09-08: Extend all four KDE lanes with the production Native SDK binary and real AT-SPI actions for consent, cancellation, removal, modal isolation and a saved draft that retains comments and unknown fields. This is additional native UI qualification, pending its first run.
- 2026-09-08: Banked all-four actual-runtime acceptance from 34195263902. Each lane produced goose/result.json with runtime-connected, revocation, crash recovery and state-preservation proof. Keep native UI/package checks visible and portal access off.
- 2026-09-08: The expanded native gate catches a Linux-only call to a nonexistent vector method before runtime launch. Use the engine's real magnitude operation and repeat native compilation before claiming runtime acceptance.
- 2026-09-08: Add guided Rust-owned setup/removal, native consent UI and separate capability status. Retain a private stable registration identity for owned-only recovery after crashes. Expand the four native KWin lanes to compile and exercise the actual goose runtime, revocation, graceful cleanup, stopped removal and state preservation.
- 2026-09-08: The sealed activation gate stops at native Rust compilation: the introspection proxy retains a borrowed path. Drop that completed proxy before moving the selected path; no native activation result is claimed yet.
- 2026-09-08: All four KWin 5/6 x64/ARM64 lanes pass the expanded native fullscreen, other-desktop, actual user-drag and reconnect checks in 34191906181. Add retained sealed-script activation and owned-only cleanup to the production Rust bridge; its new native gate is next.
- 2026-09-08: Run 34191563854 exposes Plasma 5 desktop-vector conversion, fixture script-id reuse during Plasma 6 reconnect and a fixture responder delayed by native painting. Use the documented numeric membership on Plasma 5, unload test helpers in reverse order, and run the responder in its own GLib context before repeating the same native limits.
- 2026-09-08: Both KWin generations pass the actual Rust transport in 34190701777. Add current-desktop/activity eligibility, separate fullscreen and user-drag observation, and refusal to move a window the user is dragging. Expand the real compositor probe to those states and native ARM64 before runtime integration.
- 2026-09-08: Run 34190444045 passes actual Rust/KWin 5.27 identity, real movement,
  untrusted sender refusal, protected/stale/excessive refusal, expiry, reconnect and stop.
  KWin 6.3's first scripted frame is empty and then stops before fixture window mapping;
  no Rust connection is reached. Add a mapped-window precondition and exact stop-reason
  diagnostics, retaining the existing 250 ms action/revocation deadline.
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
