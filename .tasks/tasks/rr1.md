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
- [x] Relevant production-path regression tests pass.
- [x] Actual behavior or rendered output is inspected in its supported environment.
- [x] Integration and required package/release checks pass before publication.

- [x] Website artwork, downloads and platform claims are deployed and verified.

## Status
Complete. The final review defects are corrected, all candidate and unchanged-main gates pass, and all eight fresh-public installation lanes pass. Website PR #6 merged as 2647c55; deployment 6323594580 succeeded, and the live page and recorded renderer SVG provenance were verified. Later Linux guidance belongs to #rlpi.
See [v1.4.0 readiness](../../docs/readiness/v1.4.0-readiness.md): final source `85c9d426f7409800198ee5e89bca7d6042034082`, candidate 34193858719, publication 34197957744 and all-eight fresh-public run 34202201615.
Hosted desktop/package proof does not close the historical installed Windows elevation or unavailable physical Mac/Pi acceptance.

## Evidence
| Criterion | Oracle | Result | Limitation |
|---|---|---|---|
| Production behavior and publication | Final source, native candidate and public download runs linked in readiness | PASS | Hosted proof remains distinct from physical acceptance |
| Website deployment | PR #6, source 2647c55, deployment 6323594580, live page and SVG provenance | PASS | Later Linux guidance remains a separate stage |

## Activity
- 2026-09-08: Completed the renderer SVG handoff and verified the deployed artwork/native-settings guidance from website PR #6. Publication, public-download and website acceptance for this stage are complete.
- 2026-09-08: Reconciled final candidate, main, immutable publication and all-eight fresh-public evidence; website follow-through remains under #rr1.
- 2026-09-08: The stage-two review found two shared companion-launch defects also affecting stage one: pathname re-resolution on Linux and invalid receipt evidence falling through to unmanaged launch. Hold publication, bind exec to the retained descriptor and reject invalid evidence while preserving valid separate installations; add real pathname-replacement and malformed-receipt regressions before new candidate/main gates.
- 2026-09-08: Final PR #9 review exposed a second autostart race and two vendored-board
  transaction gaps. Hold the existing OS config lock through reconciliation and add
  production guard tests. Real disposable HTTP requests reproduced both board failures;
  require the loaded detail revision for deletion and nonempty detail for completion.
  The review's malformed P1 prose supplies no independent actionable evidence; the actual
  Debian timestamp precondition was already diagnosed from native logs and corrected.
- 2026-09-08: The x64 Debian probe stopped at its older-config precondition because
  reproducible packages preserve receipt timestamps. Set the isolated fixture's config
  age explicitly before installation and record both times. The real settings Read/Save,
  false autostart and unchanged protected-receipt assertions remain unchanged; repeat native gates.
- 2026-09-07: Native Mac tests rejected the raw-byte fixture filename with APFS EILSEQ before any service call. Limit that filesystem-specific regression to Linux/Windows, where actual valid non-Unicode files reproduce the original panic. Production fixes are unchanged; repeat final-source qualification.
- 2026-09-07: Reproduced and corrected missing-file reload and native non-Unicode path failures; added installer-intent snapshot ordering, degraded-status and concurrent-autostart regressions plus a real Debian GUI read/save intent probe. Full local Rust/GUI/Python checks pass. Preserve the merged candidate as evidence and qualify the new source before tagging.
- 2026-09-07: Complete candidate 34181368273 passed at e51b76f, and 6ed7666 passed the corrected genuine GUI retry plus CLI/TUI transactions. The separate Linux work exposed a harmless but strict-Clippy-failing import from the installer extraction; correct its cfg and add native Linux strict checks before merging. All public/install state remains unchanged.
- 2026-09-07: All eight Native settings lanes passed again at e51b76f. The genuine CLI/TUI updates passed; GUI offline failure preserved state, but its retry test queued Update before Check had changed the previous enabled button. Await the actual new Ready response and a fresh retained helper before the unchanged transaction assertions.
- 2026-09-07: Source 35c4f48 passes the real Debian CLI/TUI/GUI update, GUI offline failure/retry, receipt, relaunch and no-op gate; all GNU/musl accessibility lanes pass. The prior complete candidate reached final ARM64 Debian qualification but its capture repeatedly froze a partial edge pose after Wander replaced entry. Moved command exercises after startup capture while preserving every pixel oracle; a fresh final-source repeat is required.
- 2026-09-07: Native Windows workload passed note/meme delivery and a three-minute follow-on run, with clean logs and graceful exit. Source ca20ad9 passed the complete local Rust/GUI gates; all six Mac/Linux Native settings lanes passed. Genuine hosted TUI update/receipt/relaunch/no-op passed; fixture teardown killed its PTY before inspecting the helper and mistook the exited child for a foreign process. Reordered teardown and bound its signal to a checked process descriptor.
- 2026-09-07: Candidate 34176867628 passed both native Windows installer builds and signed/notarized/stapled Mac production. ARM64 qualification omitted the companion/DLL from its staged inputs; added those exact build outputs. Fixed SDK fixture reads after queued input/non-atomic snapshots and the TUI cursor-position protocol response; these checks must repeat on the corrected source.
- 2026-09-07: Added audit R11 after both the actual Windows profiling probe and hosted GUI Start exposed inherited output pipes. Exact public v1.3.7 fails the bounded EOF regression; corrected CLI/app return in 0.35/0.30 seconds with runtime alive. Actual native GUI Start completes, closing settings preserves the same runtime, and reopened GUI Stop finishes gracefully. The new hosted Linux CLI updater transaction passed; TUI setup lacked a controlling terminal and is corrected before its next run.
- 2026-09-07: Candidate run 34174916505 passed the signed/notarized universal Mac app and DMG producer, but found invalid multi-file automatic WiX components plus disposable settings fixture failures. Split the notice components, corrected Linux lifecycle environment propagation, and use verified semantic toggle actions for the Mac viewport; the complete candidate must repeat before merge.
- 2026-09-07: Extended the disposable Debian fixture to run genuine updates from CLI, TUI keys and native AT-SPI GUI actions, including an offline helper failure/retry, unchanged failure receipt/runtime, successful relaunch and a public no-op. The terminal adapter records the real helper and does not substitute the installer; hosted execution is pending.
- 2026-09-07: Opened the v1.4.0 readiness record and candidate metadata. Verified actual CLI/TUI/GUI discovery, improved user-facing eligibility wording, and retained machine-readable installer diagnostics. Existing hosted Debian update/relaunch proof will repeat on the final source; immutable publication remains gated.
- 2026-09-07: user explicitly requested CLI/TUI update acceptance and GUI Check for updates / Update now controls; use the existing verified update lifecycle and typed results.
- 2026-09-07 — created from the user-approved implementation plan (agent: codex).
