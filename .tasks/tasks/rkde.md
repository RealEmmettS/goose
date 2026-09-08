TT;DR: KDE and portal integration.

## Why
Deliver the user-approved refinement while preserving healthy installations and explicit platform claims.

## Scope
Guided opt-in KWin and portal/libei capability adapters; deny terminal targets and cancel on stale identity, revoked access, or disconnect.

Authoritative sources: approved September 7 implementation plan, docs/adr/0041-refinement-native-settings-and-continuous-goose.md, current source/tests, docs/refinement-audit.md.

Preserve saved user settings/media, verified installer ownership, immutable releases, terminal protection, and graceful stop. Native hardware beyond this Windows PC is unavailable; hosted proof is identified separately.

## Plan
Implement the linked board subtasks; retain later release stages as separate tasks. At each failed check record the exact failure and fix or leave the gate open.

## Impact
Guided opt-in KWin and portal/libei capability adapters; deny terminal targets and cancel on stale identity, revoked access, or disconnect.

## Acceptance
The requested behavior runs through its production path and the supporting evidence matches the claimed platform and installation.

## Verification
- [ ] Relevant production-path regression tests pass.
- [ ] Actual behavior or rendered output is inspected in its supported environment.
- [ ] Integration and required package/release checks pass before publication.

## Status
Active. Versioned v1.6.0 metadata and readiness are prepared. Native KWin 5/6 window,
owned-prop and setup/removal checks pass on both architectures; the KDE 6 portal premise
also passes. Integrated pointer runtime/settings checks exposed inaccessible controls
below the scroll viewport. A production SDK regression reproduces that failure before
correction. Publication remains gated on the unchanged desktop tests, all existing
platform/package lanes and same-source candidate/main qualification.

## Activity
- 2026-09-08: Publication-record main run 34209990490 exposes an ARM64 restart-fixture race: the output file exists before its line is written, giving zero fields. Keep the actual restart and exact PID/session/start-argument assertions; wait for the complete line within the existing deadline before readback. Carry the correction into final-source qualification.
- 2026-09-08: Prepare the next distinct release and make exact-commit native KDE qualification a required candidate/publication dependency. Track the integrated accessibility failure without weakening the actual consent or movement checks.
- 2026-09-07 — created from the user-approved implementation plan (agent: codex).
