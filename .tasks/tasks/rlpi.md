TT;DR: Linux props and experimental Pi.

## Why
Deliver the user-approved refinement while preserving healthy installations and explicit platform claims.

## Scope
Owned Linux note/meme windows, honest placement capabilities and session detection, native ARM64/labwc proof, experimental Pi 4/5 documentation.

Authoritative sources: approved September 7 implementation plan, docs/adr/0041-refinement-native-settings-and-continuous-goose.md, current source/tests, docs/refinement-audit.md.

Preserve saved user settings/media, verified installer ownership, immutable releases, terminal protection, and graceful stop. Native hardware beyond this Windows PC is unavailable; hosted proof is identified separately.

## Plan
Implement the linked board subtasks; retain later release stages as separate tasks. At each failed check record the exact failure and fix or leave the gate open.

## Impact
Owned Linux note/meme windows, honest placement capabilities and session detection, native ARM64/labwc proof, experimental Pi 4/5 documentation.

## Acceptance
The requested behavior runs through its production path and the supporting evidence matches the claimed platform and installation.

## Verification
- [ ] Relevant production-path regression tests pass.
- [ ] Actual behavior or rendered output is inspected in its supported environment.
- [ ] Integration and required package/release checks pass before publication.

## Status
Queued behind #rr1 and #lpr. Shared runtime session/placement reporting is implemented under
the independent #lpr task and undergoing native tests; real delivery and labwc gates remain
open. Package/publication work begins after its prerequisites qualify.

## Activity
- 2026-09-07: Session inspection found a pure Wayland socket could silently select native mode despite the documented opt-in. Enforce the existing command/config choice before native initialization and add a real labwc refusal/readback probe.
- 2026-09-07: Added an optional finite SESSION response without changing existing STATUS bytes. CLI, TUI and the Rust GUI service show live runtime details; untrusted desktop names are reduced to known labels and cannot enable capabilities. Native readback and packaging remain pending.
- 2026-09-07 — created from the user-approved implementation plan (agent: codex).
