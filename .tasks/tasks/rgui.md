TT;DR: Native SDK graphical settings.

## Why

Deliver the user-approved refinement while preserving healthy installations and explicit platform claims.

## Scope

Separate Zig/native-markup settings app with a versioned bounded Rust stdio service and revision-aware shared config persistence. Keep TUI. Qualify all package identities.

Authoritative sources: approved September 7 implementation plan, docs/adr/0041-refinement-native-settings-and-continuous-goose.md, current source/tests, docs/refinement-audit.md.

Preserve saved user settings/media, verified installer ownership, immutable releases, terminal protection, and graceful stop. Native hardware beyond this Windows PC is unavailable; hosted proof is identified separately.

## Plan

Implement the linked board subtasks; retain later release stages as separate tasks. At each failed check record the exact failure and fix or leave the gate open.

## Impact

Separate Zig/native-markup settings app with a versioned bounded Rust stdio service and revision-aware shared config persistence. Keep TUI. Qualify all package identities.

## Acceptance

The requested behavior runs through its production path and the supporting evidence matches the claimed platform and installation.

## Verification
- [x] Relevant production-path regression tests pass.

- [x] Actual behavior or rendered output is inspected in its supported environment.

- [x] Integration and required package/release checks pass before publication.

## Status
Complete. All five Native SDK pages, revision-aware Rust persistence, GUI/TUI updater actions, independent lifecycle, native accessibility and verified companion packaging passed the final eight architecture lanes and public installation checks.
See [v1.4.0 readiness](../../docs/readiness/v1.4.0-readiness.md): final source `85c9d426f7409800198ee5e89bca7d6042034082`, candidate 34193858719, publication 34197957744 and all-eight fresh-public run 34202201615.
Hosted desktop/package proof does not close the historical installed Windows elevation or unavailable physical Mac/Pi acceptance.

## Evidence
| Criterion | Oracle | Result | Limitation |
|---|---|---|---|
| Production behavior and publication | Final source, native candidate and public download runs linked in readiness | PASS | Hosted proof remains distinct from physical acceptance |

## Activity
- 2026-09-08: Reconciled final candidate, main, immutable publication and all-eight fresh-public evidence; website follow-through remains under #rr1.
- 2026-09-07: Real Debian GUI update qualification exposed an upstream AT-SPI bug: disabled buttons advertised Enabled despite lacking an action. Pinned the minimal provider correction with licenses, real translation regressions, and native disabled-state checks; rerunning all source and candidate gates.
- 2026-09-07: Native macOS builds and both GNU Linux AT-SPI edit/save gates passed at 8a30875. The largest page has 122 semantic widgets before edits, so the final bound is 256 with eight simultaneous native-toggle changes covered on Windows/Linux. CLI, TUI and actual native GUI public discovery passed with the expected unmanaged state.
- 2026-09-07: Reproduced the missing Save accessibility node on an actual changed page; increased the bounded toolkit/bridge capacity and verified native Windows save plus config/status readback. Isolated SDK test modules from platform linkage to repair Windows ARM analysis and DLL loading.

- 2026-09-07: Both Mac and GNU Linux settings builds, native window exercises and production identity checks pass. Musl builds compile but lacked Alpine's separate xvfb-run package; Windows ARM build runner crashed before diagnostics. Added the missing package, pinned x64 Zig host with native ARM64 safety-enabled tests, real Linux AT-SPI qualification and bounded text runs. AccessKit consumer reads Unicode/empty fields; five bridge tests, strict clippy and Windows GUI rebuild pass. Windows OS fixture now selects only actionable buttons and retains failure trees.

- 2026-09-07: user explicitly requested CLI/TUI update acceptance and GUI Check for updates / Update now controls; use the existing verified update lifecycle and typed results.

- 2026-09-07 — created from the user-approved implementation plan (agent: codex).

- 2026-09-07: Shared revision-aware save and bounded versioned stdio service cover all 53 editable fields; Rust protocol/validation/concurrency tests and strict clippy pass. Native settings build and real Windows automation load the five-page app with an isolated config. Layout, conflict, updater, lifecycle, and packaging qualification remain in progress.

- 2026-09-07: Added the bounded native accessibility bridge and independently verified Windows DLL payload. Actual Windows text/control/editor/save checks, Rust bridge and dual-file-lease regressions, production GUI build, and Python packaging tests pass. Updated native CI toolchains; hosted rerun pending.
