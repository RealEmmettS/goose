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
Queued. Upstream IPC reviewed; implementation and native acceptance have not begun.

## Activity
- 2026-09-08 — Split the approved follow-on scope into a separately verifiable adapter task.
