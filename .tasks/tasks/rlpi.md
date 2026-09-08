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
- [x] Relevant production-path regression tests pass.
- [x] Actual behavior or rendered output is inspected in its supported environment.
- [ ] Integration and required package/release checks pass before publication.

## Status
The first complete candidate failed musl prop startup: Ubuntu lacked the musl GTK
libraries required by the packaged companion. Exact musl archives now use a native
Alpine desktop fixture with unchanged readiness and visual checks; the rerun is pending.
The implementation task #lpr is qualified: all eight native settings lanes pass in
34189363866, including real production GNU/musl props and failure/cleanup checks.
Native ARM64 labwc and inspected captures support experimental Pi guidance. This branch
prepares v1.5.0 with the final first-stage fixes integrated. Publication remains queued
behind #rr1; final-source CI, the complete candidate, main and public-byte gates are open.

## Activity
- 2026-09-08: The native integration build catches an incorrect module path in the new prop descriptor launch. Use the companion module's actual exported function; the old candidate is superseded before package execution.
- 2026-09-08: Candidate 34191310461 exposed missing musl GTK dependencies in its Ubuntu fixture (native prop child exited 127). Move exact musl package execution to Alpine on the same native architecture; retain before/after archive identity and all capability checks. Integrate the shared verified-descriptor and invalid-receipt launcher fix for owned props.
- 2026-09-08: Banked all-eight production companion qualification, marked independent implementation complete and prepared the distinct second-stage candidate version/readiness record. Publication remains ordered after #rr1.
- 2026-09-07: Recorded successful native labwc 0.7.1 x64/ARM64 proof, inspected ARM64 captures and added Pi 4/5 64-bit Desktop installation/acceptance guidance with current Raspberry Pi primary documentation. Integrated the qualified first-stage source without publishing either stage early.
- 2026-09-07: Session inspection found a pure Wayland socket could silently select native mode despite the documented opt-in. Enforce the existing command/config choice before native initialization and add a real labwc refusal/readback probe.
- 2026-09-07: Added an optional finite SESSION response without changing existing STATUS bytes. CLI, TUI and the Rust GUI service show live runtime details; untrusted desktop names are reduced to known labels and cannot enable capabilities. Native readback and packaging remain pending.
- 2026-09-07 — created from the user-approved implementation plan (agent: codex).
