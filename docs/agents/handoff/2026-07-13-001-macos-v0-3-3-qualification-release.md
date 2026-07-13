# Handoff: Honk300 v0.3.3 macOS qualification and release

**Date:** 2026-07-13
**Session:** Session 1 of the day
**Agent:** Codex
**Task/ticket ID(s):** `#m20q`, `#m16r`

---

## Session Narrative

Emmett asked for Honk300's first real Mac qualification after the initial AppKit preview showed
the procedurally rendered goose as a nearly transparent white/purple blob. The scope expanded to
the complete native surface: renderer and engine output, restrained cross-platform walking,
dark-mode notes, CLI/TUI/IPC/audio/terminal protection, macOS Accessibility onboarding,
performance, mounted-bundle lifecycle, a graphical per-user DMG installer, Developer ID signing,
notarization, atomic v0.3.3 publication, task-board maintenance, and a later DMG-first website
rollout. The site also received a delegated `shaughv-design` progressive-disclosure pass because
the existing page was visually strong but cognitively busy.

The laptop shut down twice during the work; the task resumed against the same worktree and live
tracked board. Emmett granted Accessibility manually to the unchanged preliminary signed app,
which enabled a denied-then-granted native smoke. Later he approved an installed-release-only
first-run permission flow: one native consent request per update, direct System Settings handoff,
a calm safe-edge goose while denied, one-second live polling, same-process FirstUX resumption on
grant, and safe return to the wait on revocation. He explicitly ruled out a new settings window or
configuration schema. He then narrowed this session: finish Mac-specific work, publish only if
the fail-closed gate permits, push a clean stopping point, and leave the website/final all-platform
release continuation for an Alienware session.

The repository now contains the renderer fix, runtime optimization, restrained shared gait,
semantic dark-mode note color, secure onboarding state machine, universal graphical installer,
transactional lifecycle, Developer ID/notarization workflow, ADRs, readiness evidence, and a
tracked SHAUGHV board. Automated gates are green except where the local Mac lacks a Linux audio
cross-sysroot. Preliminary Developer ID artifacts pass local signature checks. Publication did
not occur because the App Store Connect API-key trio is absent; only the P12/keychain trio exists.
No unnotarized DMG may be published or promoted.

## The Plan & Where It Stands

The canonical checklist is `docs/readiness/v0.3.3-readiness.md`; the board card is `#m20q`.

1. **Done — AppKit pixel bridge and native presentation.** tiny-skia's premultiplied RGBA bytes
   are copied directly into AppKit-owned alpha-last storage. Asymmetric channel/alpha tests and
   real light/dark captures prove the complete articulated goose instead of the translucent blob.
2. **Done — shared renderer/engine audit and restrained gait.** Windows/Linux presenter contracts
   were audited. Normal/moderate gait cadence remains weighted and planted; only Run/Charge
   recovery is shortened enough to prevent rubber-leg trailing. Seven goldens pass; three
   gait-dependent images changed intentionally.
3. **Done — macOS runtime optimization.** Reusable display surfaces, direct mutable bitmap/CGImage
   presentation, autorelease pools, cached topology, bounded canvas growth, cached stipple data,
   and 120 Hz simulation/60 Hz presentation pacing are in-tree. The preliminary 10+60 second run
   measured 8.30% median CPU, 54.48 MiB maximum RSS, -5.12 MiB RSS growth, and zero leaks.
4. **Done in code — first-run Accessibility onboarding.** Managed-install eligibility, safe
   marker creation, main-thread native bridges, calm `PermissionWaitTask`, one-second polling,
   grant/revoke transitions, and smoke contracts are automated-green. Exact-candidate native
   first-denied/non-nag/grant/revoke evidence remains open because macOS security-setting changes
   require an awake operator at action time.
5. **Done in code and preliminary native evidence — lifecycle and installer.** Mounted bundles
   install into `~/Applications/Honk300.app` through the shared transaction. The graphical helper
   is universal x86_64/arm64, targets macOS 11.0 in both slices, validates bundle/team/class, and
   invokes `honk300 install` without `sudo`. Rollback now covers app, aliases, LaunchAgent,
   receipt, and migration-created media as one boundary.
6. **Done in code; externally blocked in execution — signing and notarization.** The workflow
   imports one P12 into an ephemeral keychain, signs inside-out with hardened runtime/timestamps,
   notarizes and staples app/DMG separately, and fails closed. The certificate trio is configured;
   `APPLE_NOTARY_KEY_P8_BASE64`, `APPLE_NOTARY_KEY_ID`, and `APPLE_NOTARY_ISSUER_ID` are missing.
7. **Mostly done locally — repository gate.** fmt, strict clippy, workspace tests, release builds,
   both Apple builds, 46 Python contracts, actionlint 1.7.12, cargo-dist plan, Windows x64/ARM64,
   Linux x64 musl, package-scoped Linux GNU x64/ARM64, shell syntax, and diff checks pass. Full
   Linux GNU workspace checks need an ALSA/pkg-config cross-sysroot. ARM64 musl is not installed.
   The final pinned cargo-audit result is recorded in the latest board/readiness activity.
8. **Not started by design — candidate/default-branch/tag publication.** Do not run until the
   notary trio exists and the exact-candidate native permission/terminal gates are complete.
9. **Implemented and locally tested but parked — site rollout.** Uncommitted site changes live at
   `/Users/realemmetts/Downloads/temp_git/desktop-goose-site`. Do not deploy or promote before the
   real notarized v0.3.3 DMG is published and freshly verified.

## What Was Accomplished

- Replaced the broken Mac BGRA/alpha-first interpretation with direct premultiplied RGBA,
  AppKit-owned alpha-last `NSBitmapImageRep` storage and a reusable `NSImage`/CGImage path.
- Added asymmetric AppKit/CoreGraphics, Windows layered-window, Linux X11, and Linux Wayland
  pixel-contract coverage.
- Fixed dark-mode Mac notes by using `NSColor.labelColor()` and added semantic-color regression
  tests for appearance/contrast behavior.
- Refined shared planted feet with a four-pixel trigger, retained 70%-beat normal/moderate swing,
  an 18-pixel body-travel cap that affects the Run/Charge tiers, 16-pixel Walk and 26-pixel
  Run/Charge lag limits, and cadence/airtime guards.
- Reused per-display bitmap/image/layer surfaces, removed swizzle and CFData copies, bounded
  renderer allocations, cached shadow stipple/topology, added native autorelease pools, and fixed
  newly hot-plugged windows inheriting stale click-through state.
- Added secure installed-app-only Accessibility eligibility and an owner-only per-update marker at
  `~/Library/Application Support/honk300/state/accessibility-prompt-v1/<version>`.
- Added the calm permission-wait engine state and same-process grant/revocation recovery.
- Extended `honk300 install` for mounted source bundles and made lifecycle rollback transactional.
- Made receipt preflight/write use no-follow metadata so a dangling foreign receipt symlink is
  preserved; the cumulative-review regression passes in the 95-test root suite.
- Added `packaging/macos/InstallHonk300/main.swift`, `packaging/macos/DMG-README.txt`, and
  `script/package_macos_installer_helper.sh`.
- Hardened `.github/workflows/macos-packaging.yml` and `.github/workflows/release.yml` for
  Developer ID, API-key notarization, stapling, Gatekeeper checks, and internal evidence.
- Created ADR 0020 for Developer ID/notarized DMG distribution and ADR 0022 for onboarding;
  preserved ADR 0018's atomic/no-crates.io decisions.
- Initialized and kept the tracked SHAUGHV board live at `http://127.0.0.1:4317/` in this worktree.
- Updated paired technical/plain-English changelogs, README, AGENTS/CLAUDE guidance, canonical
  plan, backend readiness, v0.3.3 readiness, and `CODEX_PROJECT.md`.

## Key Decisions

- **DMG install is per-user.** The helper installs to `~/Applications`; no `/Applications`
  symlink, `sudo`, or machine-wide path is introduced.
- **No new user-facing install command.** The helper invokes the existing shared `honk300 install`
  transaction so aliases, receipt, media, autostart, update, rollback, and uninstall do not fork.
- **Automatic permission UI is managed-install-only.** Bare binaries, source builds, symlinks,
  mounted-DMG executions, invalid metadata, and mismatched receipts remain degraded and silent.
- **Prompt marker precedes UI and fails closed.** Unsafe paths, ownership, modes, or marker writes
  prevent prompting; they never create a nag loop.
- **Only direct honk remains available in permission wait.** Ambient pranks, mud, leaves, pats,
  collect, cursor, and window work are cleared/rejected; status/reload/stop remain available.
- **No simulated clicks in System Settings.** AppKit requests consent, opens the Accessibility pane
  or Privacy & Security fallback, and waits for the human.
- **Developer ID is mandatory for release mode.** No release ad-hoc fallback and no `--deep`;
  nested code is signed explicitly inside-out.
- **Publication is atomic and fail-closed.** Missing notary credentials means no candidate, tag,
  release, or website promotion—not a best-effort unsigned release.
- **One display is an honest hardware waiver.** Automated signed-coordinate/topology/hot-plug
  tests pass; no live multi-monitor validation is claimed.
- **Walking was deliberately not overcorrected.** Normal and moderate motion retain the weighted
  planted cadence; only extreme tiers receive the travel cap.

## How It Works

The Mac presenter allocates an AppKit-owned 8-bit, four-sample, non-planar device-RGB bitmap with
`NSBitmapFormat::empty()`, which is the premultiplied alpha-last contract matching tiny-skia RGBA.
Each dirty frame copies rows without channel swizzling, obtains the reusable CGImage from the
representation, and presents it through the cached display window/layer. Surfaces grow/reallocate
only when dimensions require it. Native event/render work is contained by autorelease pools;
`RuntimeCore` retains fixed 120 Hz simulation and dirty/capped 60 Hz presentation.

At managed startup, `src/runtime/macos_accessibility.rs` proves the executable is the exact
non-symlinked `~/Applications/Honk300.app/Contents/MacOS/honk300`, validates bundle id/version/tag/
full commit, and matches the owner-only `honk300.install.v1` receipt. It securely creates the
version marker before any UI. The AppKit main thread requests consent and opens Settings. The
engine walks the goose to a lower-right safe anchor and enters `PermissionWaitTask`. The runtime
polls trust once per second. Grant refreshes capabilities/watcher and starts fresh FirstUX without
restart; revocation drops permission-bound state and returns to the wait without reopening UI.

The helper finds sibling `Honk300.app`, checks both bundles are Developer ID Application code for
team `M9D5379H93`, then runs the sibling binary's shared install transaction. Packaging builds both
helper slices with explicit `*-apple-macos11.0` triples, lipo-joins them, signs executable then
bundle, and verifies architectures/minimum OS. The release workflow notarizes/staples the app
before the final ZIP, then creates/signs/notarizes/staples/remounts the DMG and Gatekeeper-assesses
both contained apps.

## Mac Invariants for the Alienware Session

These are release contracts, not cleanup opportunities. Do not rewrite them from Windows unless a
new failing native/contract test justifies it.

1. **Pixel format:** tiny-skia bytes are premultiplied **RGBA**. AppKit storage is alpha-last with
   `NSBitmapFormat::empty()`. Do not reintroduce BGRA, alpha-first, an unpremultiply step, a swizzle
   buffer, or an externally aliased `Vec<u8>`.
2. **Surface lifetime:** keep each window's AppKit-owned `NSBitmapImageRep`, `NSImage`/CGImage, and
   presentation layer reusable. Reallocate only for dimension growth/change. Preserve direct
   bitmap mutation, autorelease pools, cached virtual-desktop coordinates/topology, and 120/60 Hz.
3. **Hot-plug input:** a newly reconciled display window must immediately inherit the overlay's
   cached interactive/hover state; otherwise it can remain click-through until another pointer
   transition.
4. **Notes:** Mac note text must use `NSColor.labelColor()`. Do not replace it with absolute black,
   white, or a custom dark-mode guess. Windows uses system Notepad; Linux collect windows are
   explicitly unsupported.
5. **Permission eligibility:** prompt only from the exact managed app and matching receipt. Preserve
   non-symlinked component checks, full metadata validation, current-user ownership, 0700 state
   directories, atomic 0600 marker, and silent fail-closed behavior for bare/DMG/source launches.
6. **Permission order/behavior:** write marker, request native consent on AppKit main thread, open
   Settings fallback, then calm wait. Permit direct honk plus status/reload/stop only. Poll at most
   once/second. Grant resumes FirstUX in-process; revocation safely returns to wait and does not
   reopen UI. Do not add a settings window, menu bar, Dock UI, simulated clicks, or config schema.
7. **Signing:** use Developer ID Application team `M9D5379H93`; the selected local identity SHA was
   `739B04530883FF9B665C66BD464F98C622971B32`. Sign nested executable/code inside-out, hardened,
   timestamped, without `--deep`. Identity names are duplicated locally, so prefer the SHA on Mac.
8. **Helper/deployment:** both `Install Honk300.app` slices are required and both must report
   minimum macOS 11.0. Do not let a host SDK silently create a thin helper or raise minimum OS.
9. **DMG:** root contains exactly `Honk300.app`, `Install Honk300.app`, and concise instructions.
   Do not restore the `/Applications` symlink; installation is intentionally per-user.
10. **Lifecycle:** preserve one rollback boundary for app, aliases, LaunchAgent, receipt, and only
    migration-created media. Preserve pre-existing user media and foreign files. Keep receipt
    schema/tag/version/full-commit/install-root compatible with updater ownership checks.
11. **Gait:** `GAIT_STEP_TRIGGER_DISTANCE` is four pixels, normal/moderate recovery stays at 70% of
    the beat, and `MAX_BODY_TRAVEL_DURING_SWING` is 18 pixels. Tests bound visible lag to 16/26 px
    and guard airtime/cadence. Do not shorten ordinary walking further or create bicycle stepping.
12. **Goldens:** seven renderer goldens pass; only three gait-dependent images changed. Do not
    re-bless them from Windows just because raster output differs. Require a demonstrated engine/
    renderer regression and native/art review.
13. **Mac-only proof:** Alienware may run Rust/unit/contracts and Windows-native checks, but cannot
    replace AppKit screenshots, codesign chain/timestamp, `vtool` minOS, `notarytool`, stapling,
    Gatekeeper, TCC transition, CoreAudio, leak, or real terminal-window evidence. Use Mac CI or a
    physical Mac and record the limitation honestly.
14. **Deferred X11 follow-up:** `crates/honk-platform-linux/src/lib.rs` near `choose_argb_visual`
    accepts a depth-32 visual before validating masks/shifts for the BGRA upload. Treat this as a
    follow-up, not a reason to change the proven Mac RGBA bridge.
15. **Website:** local progressive-disclosure work is in the separate site repo. Keep production on
    v0.3.2 until the real v0.3.3 notarized DMG exists and its fresh-download checks pass.

## Known Issues & Limitations

- GitHub has `MACOS_CERTIFICATE_P12_BASE64`, `MACOS_CERTIFICATE_PASSWORD`, and
  `MACOS_KEYCHAIN_PASSWORD`. It does **not** have `APPLE_NOTARY_KEY_P8_BASE64`,
  `APPLE_NOTARY_KEY_ID`, or `APPLE_NOTARY_ISSUER_ID`. No local `.p8` and no `honk300-notary`
  keychain profile were found. Creating/exporting an App Store Connect API key is the only manual
  release credential prerequisite.
- The ignored local DMG `target/qualification-evidence/honk300-universal2-signed-unnotarized.dmg`
  is preliminary evidence only. It is signed but has no ticket/staple and must not be published.
- Exact-candidate ADR 0022 four-state permission evidence remains open. Do not change an OS
  Accessibility toggle without the operator awake and confirming at action time.
- Exact-candidate protected-terminal negatives and ordinary-window positive remain open. Ghostty
  was not installed on this Mac; classifier tests cover it.
- Exact-candidate final profile/leak repeat remains open; preliminary metrics pass comfortably.
- This Mac has one display; live multi-monitor/hot-plug is waived, not passed.
- Full Linux GNU workspace checks on this Mac stop in `alsa-sys` because pkg-config has no Linux
  cross-sysroot. Linux package-scoped x64/ARM64 and x64-musl workspace checks pass; native CI owns
  the full proof. ARM64 musl was not installed.
- Site changes are not committed/deployed in this repository and must remain behind release.

## Important Context for Future Sessions

- Goose repo: `/Users/realemmetts/Downloads/temp_git/goose`
- Task worktree: `/Users/realemmetts/Downloads/temp_git/goose/.worktrees/macos-v0.3.3`
- Branch: `codex/macos-v0.3.3`; remote default branch: `main`; stable release: v0.3.2.
- Accessibility design commit: `41afcaf`.
- Integrated implementation/docs/board commit before this handoff: `483a6db`.
- Board: `.tasks/TASKS.md`, detail `.tasks/tasks/m20q.md`, `.tasks/tasks/m16r.md`; local dashboard
  resolves to port 4317 from `.tasks/.board-server.json` in this Mac worktree.
- Readiness: `docs/readiness/v0.3.3-readiness.md` and
  `docs/readiness/m16-m18-readiness.md`.
- Decisions: `docs/adr/0020-macos-developer-id-dmg-distribution.md` and
  `docs/adr/0022-macos-accessibility-first-run-onboarding.md`.
- Site repo: `/Users/realemmetts/Downloads/temp_git/desktop-goose-site`; progressive-disclosure
  edits were locally tested but intentionally not deployed.
- The real release path is candidate on one exact SHA, fast-forward/default-branch CI, one
  immutable `v0.3.3` tag, atomic publication, post-release smoke, fresh-download validation, then
  site preview/production promotion.

### Changed-file inventory

Repository/workflow/board/docs: `.github/actionlint.yaml`,
`.github/workflows/macos-packaging.yml`, `.github/workflows/release.yml`, `.gitignore`,
`.claude/settings.json`, `.tasks/.gitignore`, `.tasks/CLAUDE.md`, `.tasks/MILESTONES.md`,
`.tasks/TASKS.md`, `.tasks/board-server.mjs`, `.tasks/config.json`, `.tasks/dashboard.html`,
`.tasks/memory/context/.gitkeep`, `.tasks/memory/glossary.md`, `.tasks/memory/people/.gitkeep`,
`.tasks/memory/projects/.gitkeep`, `.tasks/tasks/a6e.md`, `.tasks/tasks/m20q.md`,
`.tasks/tasks/m16r.md`, `.tasks/tasks/p4d.md`, `.tasks/tasks/r5d.md`, `AGENTS.md`, `CLAUDE.md`,
`CHANGELOG.md`, `HUMAN_CHANGELOG.md`, `README.md`, `CODEX_PROJECT.md`, `Cargo.toml`, `Cargo.lock`,
`honk300_plan.md`, `docs/adr/0018-distribution-and-atomic-release.md`, `docs/adr/README.md`,
`docs/adr/0020-macos-developer-id-dmg-distribution.md`,
`docs/adr/0021-native-wayland-capability-strata.md`,
`docs/adr/0022-macos-accessibility-first-run-onboarding.md`,
`docs/readiness/m16-m18-readiness.md`, `docs/readiness/v0.3.3-readiness.md`,
`docs/research/native-wayland-capability-path.md`, and
`docs/superpowers/plans/2026-07-13-macos-accessibility-first-run.md`.

Source/tests/packaging: `crates/honk-config-tui/src/lib.rs`, `crates/honk-control/src/lib.rs`,
`crates/honk-control/src/platform.rs`, `crates/honk-engine/src/feet.rs`,
`crates/honk-engine/src/lib.rs`, `crates/honk-engine/src/render/geom.rs`,
`crates/honk-engine/src/render/mod.rs`, `crates/honk-engine/src/task.rs`,
`crates/honk-engine/src/world.rs`, `crates/honk-engine/tests/golden/side_mid_stride.png`,
`crates/honk-engine/tests/golden/top_down.png`,
`crates/honk-engine/tests/golden/top_down_diag.png`, `crates/honk-platform-linux/src/lib.rs`,
`crates/honk-platform-macos/Cargo.toml`, `crates/honk-platform-macos/src/lib.rs`,
`crates/honk-platform-windows/src/lib.rs`, `src/install.rs`, `src/main.rs`,
`src/runtime/core.rs`, `src/runtime/macos.rs`, `src/runtime/macos_accessibility.rs`,
`src/runtime/mod.rs`, `script/honk300-installer.sh.in`, `script/package_macos_app.sh`,
`script/package_macos_installer_helper.sh`, `script/release_metadata.py`,
`script/smoke_m16_macos.sh`, `script/smoke_m16_macos_accessibility.sh`,
`script/tests/test_installer_templates.py`, `script/tests/test_macos_packaging.py`,
`script/tests/test_macos_smoke_contract.py`, `script/tests/test_release_workflows.py`,
`packaging/macos/DMG-README.txt`, and `packaging/macos/InstallHonk300/main.swift`.

## What's Next

1. **Exact next action:** create/export an App Store Connect API key for team `M9D5379H93`, then
   configure the three missing repository secrets without logging values:

   ```bash
   gh secret set APPLE_NOTARY_KEY_P8_BASE64 --repo RealEmmettS/goose
   gh secret set APPLE_NOTARY_KEY_ID --repo RealEmmettS/goose
   gh secret set APPLE_NOTARY_ISSUER_ID --repo RealEmmettS/goose
   gh secret list --repo RealEmmettS/goose
   ```

2. On the Alienware, fetch `codex/macos-v0.3.3`, read this document and the two readiness files,
   run Windows-native/full repository gates, and fix only demonstrated failures. Preserve every
   Mac invariant above.
3. Return to this Mac or an equivalent trusted Mac for the exact signed commit. Run
   `script/smoke_m16_macos_accessibility.sh` in its live phase with the operator present: reset only
   Honk300 state, confirm first prompt/Settings/safe wait, confirm non-nag relaunch, grant in place,
   verify same-process recovery, revoke in place, and verify safe return. Record one binary digest.
4. On that exact app, record the ordinary-window positive, Terminal.app/Codex/VS Code/Ghostty/test-
   shell negatives, final renderer behavior sheet, 10+60 second profile, and leaks. Preserve the
   one-display waiver unless real extra hardware is attached.
5. Test exact candidate install, autostart, v0.3.2-to-v0.3.3 update, injected rollback, uninstall,
   purge, and final clean host state.
6. Commit/push the exact SHA and run candidate mode. Only after every signing/notarization/native
   gate passes should `main` be advanced and ordinary CI awaited. Then create the single immutable
   `v0.3.3` tag.
7. Fresh-download the release app ZIP/DMG and independently verify hashes, identity/team,
   designated requirement, hardened runtime, tickets, staples, Gatekeeper, install, and update.
8. Finish and commit the separate site repo, test against the live manifest, deploy a Vercel
   preview, promote the exact site commit, and verify thegoose.app returns the notarized DMG.
