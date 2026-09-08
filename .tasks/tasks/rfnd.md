TT;DR: Audit and reliability foundation.

## Why
Deliver the user-approved refinement while preserving healthy installations and explicit platform claims.

## Scope
Record actionable findings, correct shared rendering/pacing and false tests, preserve lifecycle semantics, and validate real behavior.

Authoritative sources: approved September 7 implementation plan, docs/adr/0041-refinement-native-settings-and-continuous-goose.md, current source/tests, docs/refinement-audit.md.

Preserve saved user settings/media, verified installer ownership, immutable releases, terminal protection, and graceful stop. Native hardware beyond this Windows PC is unavailable; hosted proof is identified separately.

## Plan
Implement the linked board subtasks; retain later release stages as separate tasks. At each failed check record the exact failure and fix or leave the gate open.

## Impact
Record actionable findings, correct shared rendering/pacing and false tests, preserve lifecycle semantics, and validate real behavior.

## Acceptance
The requested behavior runs through its production path and the supporting evidence matches the claimed platform and installation.

## Verification
- [x] Relevant production-path regression tests pass.
- [x] Actual behavior or rendered output is inspected in its supported environment.
- [x] Integration and required package/release checks pass before publication.

## Status
Complete. The shared resource, pacing, audio, delivery and lifecycle corrections passed final native builds and the complete immutable release. Actual isolated Windows workloads and before/after renderer measurements are retained in the audit.
See [v1.4.0 readiness](../../docs/readiness/v1.4.0-readiness.md): final source `85c9d426f7409800198ee5e89bca7d6042034082`, candidate 34193858719, publication 34197957744 and all-eight fresh-public run 34202201615.
Hosted desktop/package proof does not close the historical installed Windows elevation or unavailable physical Mac/Pi acceptance.

## Evidence
| Criterion | Oracle | Result | Limitation |
|---|---|---|---|
| Production behavior and publication | Final source, native candidate and public download runs linked in readiness | PASS | Hosted proof remains distinct from physical acceptance |

## Activity
- 2026-09-08: Reconciled final candidate, main, immutable publication and all-eight fresh-public evidence; website follow-through remains under #rr1.
- 2026-09-07: Complete candidate e51b76f passed every portable/native package and Mac signing/notarization gate. The follow-on strict Linux compilation found an unused installer glob import; limit it to Windows/test callers and add strict Clippy to both Linux CI lanes before the final main/release repeat.
- 2026-09-07: Physical before/after workload qualification reproduced a baseline delivery failure twice. Added R12 and made owned prop activation best effort without bypassing focus policy; release build, strict workspace clippy and native Windows platform regressions pass. Repeating the full isolated native workload before closing verification.
- 2026-09-07: Reconciled implemented audit corrections and old/new render costs. Corrected native Mac cfg boundaries discovered by hosted compilation; scoped the goose edge-color oracle to its silhouette after reviewing the actual leaf-pile capture.
- 2026-09-07: Separated provenance, autostart, mutable-media migration, and existing lifecycle regressions into focused modules; current guidance already shortened with complete historical copies retained. Strict workspace clippy, formatting, the full Rust workspace suite, and all 129 Python checks pass (three platform skips). Native cfg checks remain in CI.
- 2026-09-07 — created from the user-approved implementation plan (agent: codex).
