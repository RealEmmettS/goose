TT;DR: WiX's bundled lorem-ipsum agreement is replaced by the real PolyForm Noncommercial License followed by a lengthy, explicitly non-binding agreement between the installer operator and the Goose. The change is live as immutable stable/latest v0.3.2; local, hosted, public-artifact, and post-release gates passed, and the temporary worktree/branch are removed.

## Why
This is a direct operator order. The released Global MSI currently exposes WiX's stock placeholder license because both hand-authored WiX manifests reference `WixUI_FeatureTree` without overriding `WixUILicenseRtf`. Global and Corporate MSI builds for both x64 and ARM64 share those manifests, so all four artifacts inherit the placeholder. The operator wants the installer to show the project's real license first and then a long sarcastic personal pact with the Goose.

The actual legal terms must remain unambiguous. The root `LICENSE` is the authoritative PolyForm Noncommercial License 1.0.0 and must be reproduced verbatim in the displayed RTF, including the Required Notice. The comedy appendix must say at its beginning and end that it is ceremonial, non-binding, and incapable of changing software rights, third-party terms, privacy behavior, warranties, or obligations. A later operator instruction explicitly expanded the handoff to commit, merge, push, deploy the next immutable production release, and clean all branches created for this work.

## Plan
Add a focused packaging regression test first. It will load a single simple ANSI RTF, extract deterministic plain text, verify that the legal block exactly matches the root license, assert the required section boundaries and non-binding language, enforce an 1,800-word minimum for the Goose appendix, and prove both WiX manifests reference the same source-root-relative file. Confirm the test fails before implementation.

Create `wix/honk300-license.rtf` with a short acceptance notice, a verbatim formatted copy of `LICENSE`, an unmistakable end-of-legal-terms divider, and `THE GREAT HONK ACCORD` as a 1,800-2,200 word original PG-13 mock-legal appendix. Keep RTF control words minimal so the Windows Installer RichEdit control renders immediately. Do not change the authoritative root legal files or add an agreement page to the Inno installers.

Wire both MSI manifests through `WixUILicenseRtf`. Update technical and human changelogs in lockstep. Because the operator asked for production deployment after the v0.3.1 release, bump the package and lockfile to 0.3.2, update current release pointers, and add a v0.3.2 readiness record without changing historical v0.3.1 evidence.

Run focused and full Python tests, RTF parsing checks, the complete Rust gate, cargo-dist planning/audit, and WiX 3.14.1 compilation/linking for x64 and ARM64 Global/Corporate MSIs. Inspect the MSI agreement page in both x64 variants without completing installation. Merge the verified feature branch to `main`, push, run candidate-mode release assembly for v0.3.2, create and push the immutable tag only after candidate success, monitor release publication and live assets, then remove this task's local/remote branches and worktree.

## Impact
Intended: every Windows MSI shows real licensing terms instead of filler and rewards users who keep scrolling with a substantial Goose pact. One shared RTF prevents Global/Corporate and x64/ARM64 copy drift. The next stable release makes the fix available through production download links.

Possible unintended impact: malformed or overly complex RTF could render blank; an inaccurate duplicated license could create ambiguity; a missing source file could break all MSI builds; careless publication could consume an immutable version before the complete artifact set is proven. Tests, minimal RTF, exact legal-text comparison, candidate mode, and the atomic release workflow address those risks.

## Acceptance
All four MSI artifacts embed the same readable combined agreement. The legal section exactly reproduces `LICENSE`; the Goose section is clearly separated, non-binding, long-form, funny, and free of placeholder text. Existing legal notice installation and all Inno behavior remain unchanged. Local gates pass, v0.3.2 is published atomically as stable/latest, public MSI bytes contain the new agreement, and the temporary branch/worktree is removed.

## Verification
- [x] Focused packaging regression proves red before implementation and green afterward.
- [x] Full Python and Rust repository gates, cargo-dist v0.3.2 plan, audit, and diff checks pass.
- [x] WiX 3.14.1 compiles and links Global/Corporate x64 and ARM64 MSIs with the shared RTF.
- [x] Both x64 MSI agreement dialogs visibly render the legal terms and complete Goose appendix.
- [x] Candidate and release workflows pass; v0.3.2 is stable/latest and public artifacts verify.
- [x] `main` is pushed and clean, and this task's temporary branch/worktree no longer exists.

## Status
Complete. Commit `ff08a8e` is on `main` and immutable tag `v0.3.2`; ordinary CI `29173153547`, candidate `29173165932`, release `29173358004`, and post-release smoke `29173553422` are green. Stable/latest production manifests identify the exact tag/commit, all four public MSIs match their recorded hashes and embed the exact RTF, and the live site serves the same manifest. The temporary `codex/goose-eula` worktree and local branch were removed; no remote feature branch existed.

## Activity
- 2026-07-11 18:42 — created and moved directly to Active from operator order; isolated baseline build and workspace tests passed (agent: codex).
- 2026-07-11 18:49 — completed the red-green packaging contract, 16-article combined RTF, and shared Global/Corporate `WixUILicenseRtf` wiring; focused suite is 9/9 green (agent: codex).
- 2026-07-11 18:54 — cut v0.3.2 metadata, paired technical/human changelogs, current release pointers, and a focused immutable-release readiness checklist (agent: codex).
- 2026-07-11 19:01 — local gate complete: 34 Python contracts, Rust/release checks, warnings-as-errors WiX builds, exact MSI RTF inspection, and both x64 agreement dialogs passed; ARM64 CRT host limit documented (agent: codex).
- 2026-07-11 19:04 — committed `ff08a8e`, fast-forwarded and pushed `main`; confirmed GitHub auth and that immutable candidate tag `v0.3.2` does not exist (agent: codex).
- 2026-07-11 19:12 — ordinary CI run `29173153547` and no-publication candidate run `29173165932` passed; downloaded hosted MSIs matched the RTF and real ARM64 payload machine `0xAA64`; pushed immutable annotated tag `v0.3.2` (agent: codex).
- 2026-07-11 19:22 — release run `29173358004` published v0.3.2 stable/latest after remote byte verification; independent public MSI/manifest/site checks and five-job post-release run `29173553422` passed; removed the temporary worktree/branch and moved the task to Done (agent: codex).
