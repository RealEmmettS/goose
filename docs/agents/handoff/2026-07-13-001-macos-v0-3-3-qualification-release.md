# Handoff: Honk300 v0.3.3 macOS qualification and release

> **Superseded for resume purposes:** read
> `docs/agents/handoff/2026-07-14-001-macos-v0-3-3-alienware-resume.md`. This document preserves
> the earlier session chronology; its open-blocker and next-action sections predate the completed
> shared-behavior and lifecycle re-audits.

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

Emmett subsequently expanded the all-platform motion contract: touching monitor seams must remain
continuous; the goose may occasionally walk fully beyond a genuinely exposed edge and re-enter
from the opposite exposed edge, but only while every rendered pixel is hidden; deliberate
puddle/prank errands must not wrap; startup and shutdown must always be walks from/into an edge,
never visible pops. He also ordered a roughly 30% user-close reaction: if a person closes a note
or meme Honk300 opened, the goose gets visibly annoyed and may then try its existing bounded mouse
nab. Program cleanup must not trigger it, and permission/manners/platform limits still apply.

The repository now contains those shared engine/runtime behaviors, the renderer fix, restrained
gait, semantic dark-mode note color, secure onboarding, universal graphical installer,
transactional lifecycle, cross-platform native compositor gates, Developer ID/notarization
workflow, ADRs, readiness evidence, and a tracked SHAUGHV board. The App Store Connect API key was
created through Emmett's active Chrome session, downloaded only into the ignored owner-only
`.private-release/` directory, authenticated with `notarytool history`, and stored in the three
encrypted GitHub notary secrets. All six signing/notarization secrets now exist. Publication has
still not occurred: final integrated local gates, exact-candidate Mac proof, candidate
notarization, native Windows/Linux execution, immutable publication, and fresh-download checks
remain mandatory. No unnotarized DMG may be published or promoted.

## The Plan & Where It Stands

The canonical checklist is `docs/readiness/v0.3.3-readiness.md`; the board card is `#m20q`.

1. **Done in code; exact candidate repeat open — AppKit pixel bridge and native presentation.**
   tiny-skia's premultiplied RGBA bytes are copied directly into AppKit-owned alpha-last Device-
   RGB storage, and the overlay window uses a stable standard-sRGB destination. Asymmetric tests
   and diagnostic light/dark captures prove the complete goose instead of the translucent blob;
   repeat on the byte-exact signed candidate.
2. **Done — shared renderer/engine audit and restrained gait.** Windows/Linux presenter contracts
   were audited. Normal/moderate gait cadence remains weighted and planted; only Run/Charge
   recovery is shortened enough to prevent rubber-leg trailing. Seven goldens pass; three
   gait-dependent images changed intentionally.
3. **Done diagnostically; exact candidate repeat open — macOS runtime optimization.** Reusable
   AppKit image views/bitmaps, autorelease pools, cached topology, bounded canvas growth, cached
   stipple data, stable standard-sRGB window destination, and 120/60 Hz pacing are in-tree. The
   latest active diagnostic measured 5.55% median CPU, 29.52 MiB maximum RSS, negative 9.89 MiB
   growth, zero leaks, and 20 compositor captures without a large black component. Do not convert
   this diagnostic into a release claim until the exact signed candidate repeats it.
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
6. **Done in code and provisioned; candidate execution open — signing and notarization.** The workflow
   imports one P12 into an ephemeral keychain, signs inside-out with hardened runtime/timestamps,
   notarizes and staples app/DMG separately, and fails closed. The P12 trio and App Store Connect
   trio are configured; local API-key authentication succeeded. No candidate has yet been
   notarized.
7. **In progress and candidate-blocking — shared edge lifecycle and close reaction.** The initial
   implementation adds one-in-five wrap weighting, edge entry/exit tasks, close origin, 30% roll,
   visible annoyance, and separately gated nab. Independent review found seven P1 blockers and one
   P2: missing final transparent-clear acknowledgement, gapped-monitor staging, permission-wait
   loss on hot-plug, Stop cancellation by same-frame permission transitions, close-event identity
   loss, Windows timeout/crash misclassified as user close, user-controlled exit timing without a
   lifecycle deadline, and offscreen reaction consumption. Fix and regression-test every item
   before treating the desired behavior as implemented.
8. **Implemented; native candidate execution open — Windows/Linux compositor parity.** Exact
   Windows x64/ARM64 binaries are frozen and captured over controlled dark/light desktops; exact
   Linux x64/ARM64 binaries retain X11 root proof and semantic `grim` Sway proof. Local analyzer,
   byte-layout, wiring, cross-target, and workflow contracts pass. The first native runs of these
   strengthened gates remain CI/candidate work, not current evidence.
9. **Mostly done locally; integrated rerun required — repository gate.** Earlier snapshots passed
   fmt, strict clippy, workspace tests, release/universal builds, 50 Python contracts, actionlint,
   cargo-dist, audit, Windows cross checks, and Linux backend checks. New shared behavior and
   workflow integration landed afterward, so rerun the entire gate on the final source commit.
   Full Linux GNU workspace checks on this Mac still need an ALSA/pkg-config cross-sysroot.
10. **Not started by design — candidate/default-branch/tag publication.** Credentials are no
    longer the blocker. Freeze one final source SHA, finish exact Mac proof, run candidate mode,
    and require every native/artifact job before advancing `main` or tagging.
11. **Implemented, pushed, and previewed but production-gated — site rollout.** Exact site SHA
    `85954409a54d88019f29c1209102586cfd497bff` is pushed and built as the protected Vercel preview.
    It intentionally fails closed against the still-live v0.3.2 manifest. Promote that exact SHA
    only after the notarized v0.3.3 DMG is live and independently verified.

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
- Reused per-display bitmap/image/view surfaces in the ordinary AppKit backing store, removed
  swizzle and CFData copies, bounded
  renderer allocations, cached shadow stipple/topology, added native autorelease pools, and fixed
  newly hot-plugged windows inheriting stale click-through state.
- Split the Mac color contract explicitly: premultiplied RGBA in an alpha-last Device-RGB bitmap,
  stable standard-sRGB destination on the overlay window, and final display-profile composition in
  WindowServer. The latest diagnostic clears CPU/RSS/growth/leak limits and 20 capture checks, but
  remains non-candidate evidence.
- Added the initial platform-neutral exposed-edge derivation, one-in-five baseline wrap weighting,
  entry/exit tasks, and adaptive graceful-exit path. Independent review found terminal-clear,
  gapped-topology, state-latch, and deadline blockers; the desired behavior is not release-ready.
- Added initial user/program close provenance on macOS and Windows, the independent 30% user-close
  reaction roll, a visible annoyed task, and separately gated chaining into the existing bounded
  nab. Typed request correlation, positive Windows user-close evidence, and onscreen deferral are
  still blocking. Linux remains honestly no-collect.
- Added exact-binary native compositor/lifecycle gates for Windows x64/ARM64 and strengthened the
  Linux x64/ARM64 X11/Sway gate with exact-binary, strict ARGB8888, and real `grim` evidence.
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
- Created/authenticated the App Store Connect API key, saved its local copy only under ignored
  owner-only `.private-release/`, and configured the three GitHub notary secrets without tracking
  or logging secret contents.
- Pushed and previewed the progressive-disclosure website at exact SHA
  `85954409a54d88019f29c1209102586cfd497bff`; production remains on v0.3.2 by design.
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
- **Mac color spaces are deliberately split, not identical.** The alpha-last bitmap declares
  Device RGB because that is the byte contract. The overlay window declares stable standard sRGB
  to avoid per-frame conversion into a dynamic Display P3 profile; WindowServer owns final
  physical-display conversion. Do not revert to an inherited display profile or the earlier
  all-Device-RGB window experiment without new native evidence.
- **Wraps use exposed geometry, not virtual-bounds guesses.** Touching monitor seams remain
  traversable. Only genuinely exposed edges can wrap, and the only relocation is fully hidden.
  Puddle/prank errands are narrative departures and must never become wrap shortcuts.
- **Lifecycle never visibly pops.** Startup begins offscreen; stop/exit/quit keeps the engine and
  presenter alive until the full pose is beyond a real edge. Adaptive speed is capped by existing
  Run/Charge parameters and the client wait is finite.
- **Only a human close can provoke retaliation.** Programmatic close/stop/cleanup is marked
  separately and excluded. A positive 30% roll guarantees only a safe annoyed reaction; a cursor
  nab is a second decision under the existing settings, permission, pointer, manners, and backend
  contracts. Linux has no collect trigger.

## How It Works

The Mac presenter allocates an AppKit-owned 8-bit, four-sample, non-planar device-RGB bitmap with
`NSBitmapFormat::empty()`, which is the premultiplied alpha-last contract matching tiny-skia RGBA.
Each dirty frame copies rows without channel swizzling, obtains the reusable CGImage from the
representation, and presents it through the cached `NSImageView` in the ordinary AppKit backing
store. The `NSWindow` destination is standard sRGB; WindowServer handles the final physical
display profile. Surfaces are bucketed for ordinary jitter and shrink after unusually large
transients. Native event/render work is contained by autorelease pools; `RuntimeCore` retains
fixed 120 Hz simulation and dirty/capped 60 Hz presentation.

The initial `DesktopLayout` work subtracts touching monitor corridors from each region's exposed
faces, and the baseline deck carries four wander constructors plus one `EdgeWrapTask`. The task
waits for pose invisibility before its hop; `ExcursionTask` keeps a separate return; Stop installs
`GracefulExitTask`; clients wait at most 30 seconds. Do not assume that is sufficient: a narrow gap
can still mark an edge exposed even when the fixed outward staging point lands on a downstream
monitor, the last sub-cadence exit tick can return before clearing all previously presented
effects, permission hot-plug/transitions can replace terminal state, and tiny valid user speeds can
exceed the client budget. The required fix is a lifecycle-safe exterior staging primitive, an
independent terminal Stop latch and deadline, and a clear-only present acknowledgement covering
all prior visual bounds.

Native collect snapshots initially carry `User` versus `Program` close origin, and the engine uses
a dedicated random stream for the 30% probability. `AnnoyedReactionTask` may chain the existing
gated `NabMouseTask`. The current pending boolean discards id/request/kind, however, so a lingering
A close can abort active B; Windows also reports timeout/crash as User, and a hidden goose can
consume the reaction. Replace it with one-shot typed close events correlated to the active request,
positive user-close evidence, and visibility/transient deferral. Linux's unsupported collect path
must remain outside the roll.

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
   buffer, or an externally aliased `Vec<u8>`. The bitmap is Device RGB; the overlay window is
   stable standard sRGB; WindowServer owns the final display profile. Do not make the window
   inherit Display P3 dynamically or restore the all-Device-RGB window experiment.
2. **Surface lifetime:** keep each window's AppKit-owned `NSBitmapImageRep`, `NSImage`/CGImage, and
   ordinary-backing-store `NSImageView` reusable. Bucket ordinary dimension jitter and shrink after
   unusually large transients. Preserve direct bitmap mutation, autorelease pools, cached virtual-
   desktop coordinates/topology, and 120/60 Hz. Do not restore the capture-omitted child layer.
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
13. **Exposed edges:** preserve touching-monitor continuity, one-in-five baseline wrap weighting,
    fully hidden relocation, non-wrapping errands, fully offscreen startup, and adaptive bounded
    graceful exit. Never clamp or teleport a visible goose as a shortcut. Hot-plug may restage a
    fully invisible pose, but it must not create a visible pop.
14. **User-close reaction:** preserve native User/Program provenance, independent 30% probability,
    safe visible annoyance, and the second-stage existing bounded nab gates. Program cleanup,
    denied/unsupported cursor state, quiet/fullscreen manners, and Linux no-collect must never
    produce an unauthorized pointer move.
15. **Mac-only proof:** Alienware may run Rust/unit/contracts and Windows-native checks, but cannot
    replace AppKit screenshots, codesign chain/timestamp, `vtool` minOS, `notarytool`, stapling,
    Gatekeeper, TCC transition, CoreAudio, leak, or real terminal-window evidence. Use Mac CI or a
    physical Mac and record the limitation honestly.
16. **Linux visual contract:** `choose_argb_visual` now fails closed unless byte order, masks,
    shifts, bits-per-pixel, scanline pad, and BGRA upload agree. The Sway gate uses real `grim`
    compositor pixels. Do not weaken either to make cross-builds pass; native x64/ARM64 execution
    remains mandatory.
17. **Website:** progressive disclosure is already pushed at exact site SHA
    `85954409a54d88019f29c1209102586cfd497bff`. Keep production on v0.3.2 until the real v0.3.3
    notarized DMG exists and its fresh-download checks pass; then promote this exact tested SHA.

## Known Issues & Limitations

- All six GitHub signing/notarization secrets are configured, and local API-key authentication
  succeeds. The local API key exists only under ignored, owner-only `.private-release/`; never
  print, stage, commit, email, or copy its contents into a handoff. The remaining notarization
  work is candidate execution and evidence, not credential creation.
- The ignored local DMG `target/qualification-evidence/honk300-universal2-signed-unnotarized.dmg`
  is preliminary evidence only. It is signed but has no ticket/staple and must not be published.
- Exact-candidate ADR 0022 four-state permission evidence remains open. Do not change an OS
  Accessibility toggle without the operator awake and confirming at action time.
- Exact-candidate protected-terminal negatives and ordinary-window positive remain open. Ghostty
  was not installed on this Mac; classifier tests cover it.
- Exact-candidate final profile/leak repeat remains open. The 5.55% CPU / 29.52 MiB RSS / negative
  9.89 MiB growth / zero-leak / 20-clean-capture result is diagnostic only.
- **P1 final clear:** runtimes can observe graceful completion on a sub-60-Hz tick and return
  before one transparent clear is presented. Readiness must follow a successful terminal clear-
  only present over all prior goose/effect bounds, including particles and footmarks.
- **P1 gapped topology:** a gap-facing local edge can stage 220 px outward inside a downstream
  monitor. Add a lifecycle-safe exterior ray/staging primitive and prove the complete pose is
  outside every monitor for startup, excursion hops, and exit.
- **P1 permission hot-plug/Stop ordering:** `apply_layout` can replace permission wait, while a
  same-frame grant/revoke can replace an acknowledged graceful exit. Make permission wait and
  terminal Stop independent latches; Stop wins every transition.
- **P1 lifecycle budget:** exit speed/acceleration currently trust arbitrarily small valid user
  settings. Add lifecycle-owned floors plus an internal deadline comfortably below the 30-second
  client wait, using the terminal clear handshake rather than a visible teleport.
- **P1 typed close correlation:** the pending close boolean loses id/request/kind, so a lingering
  close A can abort active collect B and bypass passthrough cleanup. Queue typed one-shot events;
  only a matching close terminates the active collect.
- **P1 Windows provenance:** pending timeout, process failure/crash, and ready-window disappearance
  are all currently labeled User. Program failure/cleanup must be Program; only positive native
  close evidence may be User.
- **P2 visibility:** a positive close roll can consume its visible annoyed beat while the goose is
  hidden. Keep the typed event pending until the pose intersects the desktop and unrelated
  transient work is finished.
- Exact-candidate live entry/wrap/non-wrapping-errand/graceful-exit observations and one real
  user-close annoyed reaction remain open after those blockers are repaired.
- The strengthened Windows x64/ARM64 and Linux x64/ARM64 compositor gates are implemented but
  have not yet produced native candidate artifacts. Cross-target compilation is not native proof.
- This Mac has one display; live multi-monitor/hot-plug is waived, not passed.
- Full Linux GNU workspace checks on this Mac stop in `alsa-sys` because pkg-config has no Linux
  cross-sysroot. Linux package-scoped x64/ARM64 and x64-musl workspace checks pass; native CI owns
  the full proof. ARM64 musl was not installed.
- Site SHA `85954409a54d88019f29c1209102586cfd497bff` is pushed and has a protected preview, but
  production deliberately remains on v0.3.2 until live v0.3.3 artifact verification.

## Important Context for Future Sessions

- Goose repo: `/Users/realemmetts/Downloads/temp_git/goose`
- Task worktree: `/Users/realemmetts/Downloads/temp_git/goose/.worktrees/macos-v0.3.3`
- Branch: `codex/macos-v0.3.3`; remote default branch: `main`; stable release: v0.3.2.
- At this handoff update, local HEAD is `be7b5a9eed8759048092d9bcb01e13347045caff`, remote branch
  head is `9331cec6c778b3e0f1053d286676b0f68d64e7e6`, and the shared behavior/workflow/docs integration
  is intentionally uncommitted pending final review. The final consolidation commit/push must
  supersede these historical hashes; a resumed agent should trust `git rev-parse HEAD` on the
  fetched branch and the readiness evidence bound to it, not assume either hash is the candidate.
- Board: `.tasks/TASKS.md`, detail `.tasks/tasks/m20q.md`, `.tasks/tasks/m16r.md`; local dashboard
  resolves to port 4317 from `.tasks/.board-server.json` in this Mac worktree.
- Readiness: `docs/readiness/v0.3.3-readiness.md` and
  `docs/readiness/m16-m18-readiness.md`.
- Decisions: `docs/adr/0020-macos-developer-id-dmg-distribution.md` and
  `docs/adr/0022-macos-accessibility-first-run-onboarding.md`.
- GitHub secret names now present: `MACOS_CERTIFICATE_P12_BASE64`,
  `MACOS_CERTIFICATE_PASSWORD`, `MACOS_KEYCHAIN_PASSWORD`, `APPLE_NOTARY_KEY_P8_BASE64`,
  `APPLE_NOTARY_KEY_ID`, and `APPLE_NOTARY_ISSUER_ID`. Verify names with `gh secret list`; never
  request or print values. The ignored local key is Mac-only operational backup, not a source file.
- Site repo: `/Users/realemmetts/Downloads/temp_git/desktop-goose-site`; branch
  `codex/macos-v0.3.3-dmg-first`; exact pushed/preview-tested SHA is
  `85954409a54d88019f29c1209102586cfd497bff`; protected preview is
  <https://desktop-goose-site-5j5jb7dpi-realemmetts.vercel.app>. Production remains intentionally
  unpromoted.
- The real release path is candidate on one exact SHA, fast-forward/default-branch CI, one
  immutable `v0.3.3` tag, atomic publication, post-release smoke, fresh-download validation, then
  site preview/production promotion.

### Changed-file inventory

Repository/workflow/board/docs: `.github/actionlint.yaml`, `.github/workflows/ci.yml`,
`.github/workflows/macos-packaging.yml`, `.github/workflows/post-release-smoke.yml`,
`.github/workflows/release.yml`, `.github/workflows/windows-installers.yml`, `.gitignore`,
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
`crates/honk-engine/src/collect_window.rs`, `crates/honk-engine/src/layout.rs`,
`crates/honk-engine/src/lib.rs`, `crates/honk-engine/src/render/geom.rs`,
`crates/honk-engine/src/render/mod.rs`, `crates/honk-engine/src/task.rs`,
`crates/honk-engine/src/world.rs`, `crates/honk-engine/tests/golden/side_mid_stride.png`,
`crates/honk-engine/tests/golden/top_down.png`,
`crates/honk-engine/tests/golden/top_down_diag.png`, `crates/honk-platform-linux/src/lib.rs`,
`crates/honk-platform-macos/Cargo.toml`, `crates/honk-platform-macos/src/lib.rs`,
`crates/honk-platform-windows/src/lib.rs`, `src/install.rs`, `src/main.rs`,
`src/runtime/core.rs`, `src/runtime/macos.rs`, `src/runtime/macos_accessibility.rs`,
`src/runtime/linux.rs`, `src/runtime/mod.rs`, `src/runtime/windows.rs`,
`script/analyze_windows_overlay_capture.py`, `script/honk300-installer.sh.in`,
`script/package_macos_app.sh`,
`script/package_macos_installer_helper.sh`, `script/release_metadata.py`,
`script/smoke_m16_macos.sh`, `script/smoke_m16_macos_accessibility.sh`,
`script/smoke_m17_m18_linux.sh`, `script/smoke_released_windows.ps1`,
`script/smoke_windows_overlay.ps1`,
`script/tests/test_installer_templates.py`, `script/tests/test_macos_packaging.py`,
`script/tests/test_macos_smoke_contract.py`, `script/tests/test_release_workflows.py`,
`script/tests/test_windows_overlay_smoke.py`,
`packaging/macos/DMG-README.txt`, and `packaging/macos/InstallHonk300/main.swift`.

## What's Next

1. **Exact next action on the current Mac:** fix the shared-behavior review blockers before any
   release gate. Start with a `RuntimeCore` terminal clear-only present plan plus successful-present
   acknowledgement; then add gap-safe lifecycle staging, independent permission-wait/terminal-Stop
   latches, lifecycle-owned speed/deadline bounds, typed request-correlated close events, positive
   Windows user-close provenance, and onscreen reaction deferral. Add every regression listed in
   `docs/readiness/v0.3.3-readiness.md`. Then finish the workflow review and run the complete
   integrated gate. Do not reuse the earlier clean result.
2. Review `git diff`, ensure no secret contents or ignored qualification artifacts are staged,
   update this handoff's branch hash if useful, then stage/commit/push every source, workflow,
   documentation, changelog, and tracked-board change to `codex/macos-v0.3.3`.
3. Dispatch the full candidate workflow against that exact 40-character SHA with `candidate=true`.
   All six secrets are ready. Require notarized/stapled app+DMG evidence, exact Windows x64/ARM64
   layered-compositor evidence, exact Linux x64/ARM64 X11/Sway compositor evidence, artifact
   assembly, and zero publication. A failure means fix forward to a new SHA and repeat candidate;
   never move a tag.
4. Download the candidate's exact notarized Mac app/DMG to this Mac. With the operator present,
   run ADR 0022's four states on one recorded executable digest: first denied prompt/Settings/safe
   wait, second denied non-nag, live grant and FirstUX without restart, live revoke and safe return.
   Record the ordinary-window positive and Terminal.app/Codex/VS Code/Ghostty/test-shell negatives.
5. On that same candidate, record light/dark renderer semantics, fully offscreen startup, natural
   seam behavior where hardware permits, hidden-only wrap, non-wrapping errand, one user-close
   annoyed reaction, and graceful exit without a pop. Repeat the 10+60-second active profile and
   `leaks`; the diagnostic metrics are comparison only. Preserve the one-display live-multi-
   monitor waiver unless hardware actually changes.
6. Complete exact-candidate graphical install, opt-in autostart, stop/start, v0.3.2-to-v0.3.3
   update, injected rollback, uninstall/purge, and foreign-file preservation. Finish this Mac with
   no managed app, aliases, LaunchAgent, socket, receipt, test media, or scoped fixtures; reset only
   Honk300's Accessibility record where permitted.
7. On the Alienware, fetch the pushed branch and read this handoff plus both readiness files before
   changing code. Run the native Windows overlay/lifecycle smoke on the exact candidate and the
   ordinary Rust/contracts. Preserve every Mac invariant above; do not “simplify” alpha-last
   Device RGB plus standard-sRGB window, AppKit capture, TCC, signing, helper, or graceful edge
   logic from Windows. GitHub secrets are sufficient for CI; the ignored local Mac API key does
   not need to be copied to Alienware.
8. Only after candidate and native evidence are complete, fast-forward `main` to the proven SHA,
   wait for ordinary CI on that exact commit, create/push the one immutable `v0.3.3` tag, and wait
   for atomic publication plus post-release smoke.
9. Fresh-download the public app ZIP/DMG/manifest/sidecars and independently verify hashes,
   Developer ID/team/designated requirement/hardened runtime, tickets, staples, Gatekeeper,
   install, and update.
10. Re-run site tests against the live v0.3.3 manifest, then promote exact site SHA
    `85954409a54d88019f29c1209102586cfd497bff` to the default branch. Verify thegoose.app's primary
    Mac action returns the real notarized DMG; do not rebuild or redesign the already-proven site
    during promotion.
