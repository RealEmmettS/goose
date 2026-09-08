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
- [x] Relevant production-path regression tests pass.
- [x] Actual behavior or rendered output is inspected in its supported environment.
- [x] Integration and required package/release checks pass before publication.

## Status
Done. Immutable v1.6.0 is public at d38e4845,
after the complete candidate, unchanged-source main and publication gates.
All eight fresh-public installation lanes and the independent 47-asset audit pass.
Native window, settings and pointer lifecycle regressions pass on final source.
Website PR 8 and deployment 6327655840 are verified, including actual KDE guidance,
all 22 mapped downloads and matching payload hashes. See
docs/readiness/v1.6.0-readiness.md for exact source and run identities.

## Activity
- 2026-09-08 — Complete and read back site PR 8, production deployment 6327655840, all 22 download mappings and matching manifest hashes; both site CI runs and all 12 browser tests pass. Close the third published refinement stage.
- 2026-09-08 — Publish v1.6.0 after candidate 34217330049 and unchanged main qualification. Publication 34222242693 and fresh-public 34225303741 pass; independently verify every public asset and send the authorized website update for final live readback.
- 2026-09-08 — Candidate 34214227317 passes all architecture/package/native/signing gates at 881884a. PR review identifies missing Cargo source-package inclusion and unnecessary pointer revocation on repeated healthy KDE setup. Correct both and require actual Cargo selection plus repeated CLI/native setup during a live native pointer grant before qualifying the changed source.
- 2026-09-08: Publication-record main run 34209990490 exposes an ARM64 restart-fixture race: the output file exists before its line is written, giving zero fields. Keep the actual restart and exact PID/session/start-argument assertions; wait for the complete line within the existing deadline before readback. Carry the correction into final-source qualification.
- 2026-09-08: Prepare the next distinct release and make exact-commit native KDE qualification a required candidate/publication dependency. Track the integrated accessibility failure without weakening the actual consent or movement checks.
- 2026-09-07 — created from the user-approved implementation plan (agent: codex).
