TT;DR: Ship separately qualified, explicitly enabled Sway window/fullscreen observations.

## Why
The approved refinement requires compositor-specific follow-on integrations after KDE.

## Scope
Native window observations and fullscreen awareness through Sway's own IPC, with explicit
setup/removal and independent capability reporting. ADR 0046 keeps production movement
unsupported because standard Sway IPC supplies no authoritative active-drag state. Pointer
control also requires a separately demonstrated permission and observation path. Preserve current
configuration, foreign desktop settings, terminal protection, graceful shutdown and installer
ownership. KDE evidence does not qualify Sway.

## Plan
First prove socket peer ownership, real native window identity and actual bounded placement
in a disposable Sway desktop on each native Linux architecture. Inspect the first failed
premise after at most two native attempts before expanding runtime or GUI wiring. Then
exercise stale identities, user drags, missing outputs, revocation and restart through Rust.
The existing parent #rwl tracks completion of all compositor adapters; #rkde gates publication.

## Acceptance
The real goose uses only proven Sway observations after explicit setup, reports unsupported
actions accurately and withdraws observations on lost authority. Native owned props retain
normal compositor placement. A new immutable public release and
its website guidance follow all project-required candidate/main and public-byte gates.

## Evidence
Authoritative protocol: https://github.com/swaywm/sway/blob/master/sway/sway-ipc.7.scd
It specifies the native socket framing, tree identities, application PIDs, visible geometry
and command responses.

Native run https://github.com/RealEmmettS/goose/actions/runs/34212710026 passes on
Sway 1.9 (Ubuntu 24.04) and 1.10.1 (Debian trixie), each on x64 and ARM64. Actual
socket peer credentials match the privately launched compositor and Unix user. Native
fixture windows expose their real PID, application identity and floating geometry;
the exact owned target moves six pixels while the protected fixture stays unchanged.
Fullscreen enters/exits and the closed fixture disappears from the live tree.

The native tree/seat results and upstream IPC documentation expose no authoritative
active user-drag state. Window movement is therefore only a native operation premise;
it does not yet qualify animated deliveries. Resolve that safety boundary explicitly
before production movement. Pointer authority remains separately unsupported.

## Verification
- [ ] Actual supported Sway versions on x64/ARM64 pass native movement and refusal scenarios.
- [ ] Rust controls and native settings preserve drafts, ownership and revocation behavior.
- [ ] Required source, architecture, package, immutable-publication and website checks pass.

## Status
Active. The initial socket/window/fullscreen premise passed all four native lanes.
The actual bounded Rust decoder/transport and untrusted-peer refusal pass all four native
lanes in run 34215421415. CLI, settings service, native GUI and owned observation worker
are implemented; integrated desktop lifecycle qualification is now required. User drag
and pointer authority remain unsupported.
Publication remains ordered after the qualified KDE release.

## Activity
- 2026-09-08 — Verify all four production Rust transport lanes, then connect explicit
  Sway consent to its independent bounded worker and native settings controls. Add actual
  engine fullscreen/configuration, native draft, revocation, socket replacement and
  retained-worker/restart qualification before making a public support claim.
- 2026-09-08 — Record ADR 0046 and implement a read-only Rust boundary restricted to
  qualified versions, private socket peer identity, stable ownership and bounded I/O.
  Extend the native fixture to exercise that exact decoder/transport and reject a
  same-user non-compositor listener before it receives any request.
- 2026-09-08 — Record all four native premise results, including the unresolved user-drag
  observation boundary. Do not promote raw movement support into a production capability.
- 2026-09-08 — Begin the isolated native socket/window premise with fixed reply bounds,
  native peer credentials and no modification of the user's compositor configuration.
- 2026-09-08 — Split the approved follow-on scope into a separately verifiable adapter task.
