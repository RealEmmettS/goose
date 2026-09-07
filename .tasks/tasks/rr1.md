TT;DR: First refinement release.

## Why
Deliver the user-approved refinement while preserving healthy installations and explicit platform claims.

## Scope
Qualify and publish the reliability, art, and GUI stage through immutable candidate/main/release/public-byte gates.

Authoritative sources: approved September 7 implementation plan, docs/adr/0041-refinement-native-settings-and-continuous-goose.md, current source/tests, docs/refinement-audit.md.

Preserve saved user settings/media, verified installer ownership, immutable releases, terminal protection, and graceful stop. Native hardware beyond this Windows PC is unavailable; hosted proof is identified separately.

## Plan
Implement the linked board subtasks; retain later release stages as separate tasks. At each failed check record the exact failure and fix or leave the gate open.

## Impact
Qualify and publish the reliability, art, and GUI stage through immutable candidate/main/release/public-byte gates.

## Acceptance
The requested behavior runs through its production path and the supporting evidence matches the claimed platform and installation.

## Verification
- [ ] Relevant production-path regression tests pass.
- [ ] Actual behavior or rendered output is inspected in its supported environment.
- [ ] Integration and required package/release checks pass before publication.

## Status
Queued; not implemented or qualified.

## Activity
- 2026-09-07: user explicitly requested CLI/TUI update acceptance and GUI Check for updates / Update now controls; use the existing verified update lifecycle and typed results.
- 2026-09-07 — created from the user-approved implementation plan (agent: codex).
