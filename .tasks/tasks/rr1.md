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
Active: v1.4.0 candidate selected and stamped across Rust/settings metadata; no tag or release created. CLI, real TUI and native GUI update discovery passed against public v1.3.7, correctly identifying the development copy as unmanaged. Exact native settings and full CI are running; final candidate/main/package/public-byte gates remain open in docs/readiness/v1.4.0-readiness.md.

## Activity
- 2026-09-07: Candidate run 34174916505 passed the signed/notarized universal Mac app and DMG producer, but found invalid multi-file automatic WiX components plus disposable settings fixture failures. Split the notice components, corrected Linux lifecycle environment propagation, and use verified semantic toggle actions for the Mac viewport; the complete candidate must repeat before merge.
- 2026-09-07: Extended the disposable Debian fixture to run genuine updates from CLI, TUI keys and native AT-SPI GUI actions, including an offline helper failure/retry, unchanged failure receipt/runtime, successful relaunch and a public no-op. The terminal adapter records the real helper and does not substitute the installer; hosted execution is pending.
- 2026-09-07: Opened the v1.4.0 readiness record and candidate metadata. Verified actual CLI/TUI/GUI discovery, improved user-facing eligibility wording, and retained machine-readable installer diagnostics. Existing hosted Debian update/relaunch proof will repeat on the final source; immutable publication remains gated.
- 2026-09-07: user explicitly requested CLI/TUI update acceptance and GUI Check for updates / Update now controls; use the existing verified update lifecycle and typed results.
- 2026-09-07 — created from the user-approved implementation plan (agent: codex).
