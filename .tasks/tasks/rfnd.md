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
- [ ] Integration and required package/release checks pass before publication.

## Status

Active: shared bounded rendering, accumulator pacing, production audio/buffer checks and lifecycle extraction are implemented. Full Windows workspace regressions and measured renderer workloads passed. Final native compilation and release gates remain open; the Mac cfg extraction correction awaits hosted verification.

## Activity
- 2026-09-07: Reconciled implemented audit corrections and old/new render costs. Corrected native Mac cfg boundaries discovered by hosted compilation; scoped the goose edge-color oracle to its silhouette after reviewing the actual leaf-pile capture.
- 2026-09-07: Separated provenance, autostart, mutable-media migration, and existing lifecycle regressions into focused modules; current guidance already shortened with complete historical copies retained. Strict workspace clippy, formatting, the full Rust workspace suite, and all 129 Python checks pass (three platform skips). Native cfg checks remain in CI.
- 2026-09-07 — created from the user-approved implementation plan (agent: codex).
