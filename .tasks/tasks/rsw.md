TT;DR: Ship a separately qualified, explicitly enabled Sway adapter.

## Why
The approved refinement requires compositor-specific follow-on integrations after KDE.

## Scope
Native window observations, fullscreen awareness and bounded owned-prop placement through
Sway's own IPC, with explicit setup/removal and independent capability reporting. Pointer
control requires a separately demonstrated permission and observation path. Preserve current
configuration, foreign desktop settings, terminal protection, graceful shutdown and installer
ownership. KDE evidence does not qualify Sway.

## Plan
First prove socket peer ownership, real native window identity and actual bounded placement
in a disposable Sway desktop on each native Linux architecture. Inspect the first failed
premise after at most two native attempts before expanding runtime or GUI wiring. Then
exercise stale identities, user drags, missing outputs, revocation and restart through Rust.
The existing parent #rwl tracks completion of all compositor adapters; #rkde gates publication.

## Acceptance
The real goose uses only proven Sway operations after explicit setup, reports unsupported
operations accurately and cancels work on lost authority. A new immutable public release and
its website guidance follow all project-required candidate/main and public-byte gates.

## Evidence
Authoritative protocol: https://github.com/swaywm/sway/blob/master/sway/sway-ipc.7.scd
It specifies the native socket framing, tree identities, application PIDs, visible geometry
and command responses. Exact installed compositor versions and native behavior remain unverified.

## Verification
- [ ] Actual supported Sway versions on x64/ARM64 pass native movement and refusal scenarios.
- [ ] Rust controls and native settings preserve drafts, ownership and revocation behavior.
- [ ] Required source, architecture, package, immutable-publication and website checks pass.

## Status
Queued. Upstream protocol reviewed; implementation and native acceptance have not begun.

## Activity
- 2026-09-08 — Split the approved follow-on scope into a separately verifiable adapter task.
