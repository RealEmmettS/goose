# Handoff: Honk300 v0.3.3 Mac completion and Alienware resume

**Date:** 2026-07-14
**Session:** Session 1 of the day
**Agent:** Codex
**Task/ticket ID(s):** `#m20q`, `#m16r`

---

## Session Narrative

Emmett began this work after the first native macOS preview rendered the procedural goose as a
nearly transparent white/purple blob. The request expanded into the first full Mac qualification:
correct AppKit presentation, dark-mode notes, runtime optimization, all CLI/TUI/IPC/audio and
terminal behavior, durable Accessibility onboarding, transactional per-user installation, a
graphical DMG helper, Developer ID signing, Apple notarization, atomic v0.3.3 publication, and a
DMG-first website. He also asked that shared improvements benefit Windows and Linux, that the
walking refinement remain restrained, and that the SHAUGHV board stay live and accurate.

The product scope grew in two explicit ways. First, ordinary roaming may occasionally walk fully
through a genuinely exposed monitor edge and re-enter through the opposite exposed edge, while
touching monitor seams remain continuous, gapped downstream monitors are traversed naturally,
deliberate errands never wrap, and startup/stop never visibly pop. Second, a native user close of a
Honk300 note or meme has an independent approximately 30% chance of a visible annoyed reaction;
only a second, live permission/configuration/manners/capability decision may chain the existing
bounded mouse nab. Program cleanup and Linux's no-collect path never trigger retaliation.

The Mac renderer and note defects were reproduced, fixed, captured, and profiled. The AppKit path
now presents opaque articulated output with readable semantic dark-mode note text. Shared gait was
adjusted only enough to release planted legs before they read as stretching; normal cadence was
preserved. The shared edge/close implementation then received an independent adversarial review.
Its final-clear, gapped-topology, permission/hot-plug, Stop ordering, deadline, typed event,
Windows provenance, and offscreen-reaction findings were repaired and independently closed.

Release review next found five lifecycle races: replacement of a verified Windows artifact before
elevated execution, an ambient PowerShell lease bypass, Unix signal traps which could enter with
status zero, session-local ownership around machine-wide Windows files, and a deferred-uninstall
helper which could outlive a parent-side error. All five are now fixed and independently re-audited
clean. The complete Mac-hosted local gate passes on the resulting stopping tree.

Emmett's final instruction was to stop at a pushed, git-tracked Mac handoff so a fresh Alienware
session can run overnight. Accordingly, no immutable tag, public v0.3.3 release, or production
website promotion was attempted. Exact-candidate interactive TCC evidence also remains open: it
requires one notarized candidate plus operator-visible first-denied/non-nag/live-grant/live-revoke
observations. The board deliberately keeps `#m20q` and `#m16r` Active.

## The Plan & Where It Stands

The canonical checklist is `docs/readiness/v0.3.3-readiness.md`; board source is
`.tasks/TASKS.md`; rich task state is `.tasks/tasks/m20q.md` and `.tasks/tasks/m16r.md`.

1. **Done in code and preliminary native evidence — Mac pixel bridge and notes.** tiny-skia's
   premultiplied RGBA bytes copy directly into AppKit-owned alpha-last storage. The overlay window
   declares stable standard sRGB; WindowServer owns final display conversion. Notes use
   `NSColor.labelColor()`. Native light/dark captures show an opaque goose and readable notes.
2. **Done in code/focused review — shared gait and behavior.** The four-pixel plant trigger,
   weighted normal/moderate cadence, Run/Charge recovery cap, exposed-edge routing, fully hidden
   wrap, non-wrapping errands, locomoted startup/exit, and user-close reaction all pass focused
   tests and independent review. Do not overcorrect the gait or re-bless goldens casually.
3. **Done diagnostically; exact candidate repeat open — Mac performance.** Active diagnostic:
   5.55% median CPU, 29.52 MiB max RSS, negative 9.89 MiB growth, zero leaks, and 20 clean
   compositor captures after a 10-second warm-up/60-second sample. Repeat on the exact candidate.
4. **Done in code; exact four-state evidence open — Accessibility onboarding.** The exact managed
   app/receipt may request consent once per version, open Settings, wait calmly at a safe edge,
   poll once per second, resume FirstUX in-process on grant, and safely return on revocation.
5. **Done in code/focused audit — lifecycle.** Install/update/uninstall retain the true singleton.
   Unix interruptions roll back explicitly. Windows pins payload identity through execution,
   reacquires its lease explicitly, checks exact shared paths across sessions, rejects deferred
   reboot state, and kills every helper which has not completed the READY handoff.
6. **Done in code/provisioned; candidate execution open — signing/notarization.** Exact G2
   Developer ID fingerprint and team are pinned. All six GitHub secrets are present. The App Store
   Connect key authenticates locally. Candidate mode must produce notarized/stapled app ZIP and DMG
   plus internal JSON evidence without publishing.
7. **Done locally; hosted native execution open — Windows/Linux compositor parity.** Exact-binary
   x64/ARM64 gates and analyzers are wired into candidate/post-release workflows. Alienware/hosted
   CI must run the first native Windows gates; hosted Linux must run X11/Sway x64/ARM64 gates.
8. **Done for this stopping tree — local Mac-hosted gate.** Formatting, native strict clippy,
   389 workspace tests, release build, both Apple release targets, 71 Python tests, actionlint,
   shell syntax/shellcheck, PowerShell parsing, cargo-dist plan, cargo-audit 0.22.2, diff checks,
   and Windows x64/ARM64 strict clippy pass.
9. **Not done — candidate and exact native qualification.** Commit/push this branch, dispatch the
   candidate against that exact full SHA, then download its private artifacts and perform the Mac
   four-state/capture/profile/terminal/lifecycle checklist without rebuilding.
10. **Not done — publication and site production.** Only after candidate + default-branch CI pass
    the same SHA may `v0.3.3` be tagged. Fresh-download verification follows the atomic release.
    Only then may exact site SHA `85954409a54d88019f29c1209102586cfd497bff` reach production.

## What Was Accomplished

- Corrected and optimized the Mac presenter in `crates/honk-platform-macos/src/lib.rs` and
  `src/runtime/macos.rs`: direct premultiplied RGBA, reusable AppKit-owned bitmap/image/view,
  bounded surface shrink, autorelease pools, cached coordinates, 120/60 Hz pacing, stable sRGB.
- Fixed Mac note appearance with the semantic system label color and native regression coverage.
- Refined shared gait and added cadence/stretch guards in `crates/honk-engine/src/feet.rs` and its
  tests without changing the ordinary walking character.
- Added exposed-edge topology/routing, hidden wrap, offscreen startup, and graceful exit across
  `crates/honk-engine/src/layout.rs`, `task.rs`, `world.rs`, and shared runtime loops.
- Added typed collect-close id/request/kind/origin events, exact-once routing, positive Windows
  native user-close evidence, deferred visible annoyance, and separately gated nab.
- Fixed macOS/Windows active-request multiplexing so lingering props do not starve newer work.
- Added exact-binary native Linux and Windows compositor analyzers/smokes and fail-closed workflow
  evidence under `script/`, `script/tests/`, and `.github/workflows/`.
- Added installed-release-only Mac Accessibility onboarding and ADR 0022 contracts.
- Added mounted-bundle transactional install and universal `Install Honk300.app`; the DMG root is
  exactly the app, helper, and instructions with no `/Applications` symlink.
- Hardened lifecycle ownership in `crates/honk-control/src/platform.rs`, `src/install.rs`,
  `src/update.rs`, and both installer templates.
- Added exact G2 Developer ID, hardened runtime, notarization, stapling, Gatekeeper, final-ZIP,
  final-DMG, and internal evidence requirements to the release workflows.
- Created and authenticated the App Store Connect API key, stored its local copy only under the
  ignored owner-only `.private-release/` directory, and configured all six encrypted GitHub
  Actions secrets without tracking or printing credential contents.
- Completed/pushed the site's progressive-disclosure design at exact SHA
  `85954409a54d88019f29c1209102586cfd497bff`; its protected preview intentionally fails closed
  against the still-live v0.3.2 manifest.
- Updated `README.md`, `AGENTS.md`, `CLAUDE.md`, `CODEX_PROJECT.md`, `honk300_plan.md`, ADRs,
  paired changelogs, readiness evidence, the live board, and this handoff.

## Key Decisions

- **Mac DMG installs per user.** Destination is `~/Applications/Honk300.app`; no `sudo` and no
  misleading `/Applications` symlink.
- **One shared lifecycle implementation.** The graphical helper calls existing `honk300 install`;
  it does not fork aliases, media, receipt, autostart, update, rollback, or uninstall semantics.
- **Developer ID/notarization fail closed.** No release-mode ad-hoc fallback, no `--deep`, and no
  publication without accepted notary JSON, staples, Gatekeeper, and exact leaf fingerprint.
- **Managed-only Accessibility UI.** Bare/source/mounted-DMG/unreceipted apps remain silent and
  degraded. The app may open Settings but never simulate approval clicks.
- **Permission wait remains controllable.** Honk/status/reload/stop work; permission-bound pranks
  report busy. Grant/revoke transitions happen in the same process.
- **Color contract is intentionally split.** AppKit bitmap is alpha-last Device RGB matching byte
  storage; window destination is stable standard sRGB; WindowServer performs physical conversion.
- **No visible teleport.** Only a fully hidden ordinary wrap may relocate. Startup/exit use real
  locomotion; touching seams remain natural and deliberate errands return through their own edge.
- **Close reaction is character, not punishment.** A 30% positive roll guarantees only visible
  annoyance. Cursor motion is a second, bounded, live-gated decision. Cleanup never triggers it.
- **Walking remains restrained.** Normal/moderate weighted cadence stays intact; only excessive
  Run/Charge trailing is shortened. Seven renderer goldens remain authoritative.
- **One-display waiver is narrow.** Automated topology/hot-plug tests pass, but no live
  multi-monitor claim is made.
- **No release at this handoff.** Stable/latest remains v0.3.2. The prospective v0.3.3 README link
  is intentionally qualified until publication.

## How It Works

The Mac overlay owns an `NSBitmapImageRep` configured for 8-bit, four-sample, non-planar,
premultiplied alpha-last pixels. Each dirty frame copies tiny-skia RGBA rows directly and updates a
reused image/view in the normal AppKit backing store. Surface buckets absorb small motion jitter
and shrink after large note/meme transients. An autorelease pool encloses native event/present
work. The shared `RuntimeCore` keeps 120 Hz simulation and dirty/capped 60 Hz presentation.

`DesktopLayout` derives exposed spans from actual monitor regions. Touching corridors remain
connected; gapped downstream regions prevent false exposed staging. Ordinary `EdgeWrapTask` waits
until the whole pose is invisible before relocating to the opposite exposed entry. Excursions keep
their own return. `GracefulExitTask` has lifecycle-owned speed/acceleration/deadline bounds and the
runtime reports ready only after the final transparent clear successfully presents.

Native collect controllers emit typed close events with request id, content kind, and User/Program
origin. The engine drains them exactly once, does not let a lingering old prop abort a new collect,
and defers a positive user reaction until the goose can visibly perform it. Windows treats only
positive native close evidence as User; timeout, crash, process failure, and cleanup are Program.

Every lifecycle mutation first performs ownership preflight, asks the runtime to finish its
animated exit, then retains the singleton. Unix bootstraps hold the staged binary through a
parent-owned FIFO; HUP/INT/TERM call cleanup with 129/130/143. Windows downloads MSI/portable ZIP,
opens them with read sharing only, verifies size/hash through those retained streams, extracts the
lease holder from the pinned ZIP, checks exact installed paths with Restart Manager, and rejects
1641/3010. Generated PowerShell updates close/wait/dispose the outer lease before the public
bootstrap always reacquires. Deferred uninstall owns an armed RAII child guard until exact READY
plus a live-child check succeeds.

The Mac helper validates sibling bundle identifier, Developer ID Application class/team, release
metadata, and universal slices, then invokes shared install. CI signs nested executable and bundle
inside-out, notarizes/staples/validates the app before creating the final ZIP, creates and signs the
DMG, notarizes/staples/remounts it, and Gatekeeper-assesses both contained apps.

## Mac Invariants for the Alienware Session

1. tiny-skia output is premultiplied **RGBA**. Keep alpha-last `NSBitmapFormat::empty()`; do not
   restore BGRA/alpha-first, unpremultiplication, swizzle storage, or external buffer aliasing.
2. Keep the bitmap Device RGB and the overlay window standard sRGB. Do not inherit Display P3 or
   restore the all-Device-RGB window experiment; both caused capture/performance regressions.
3. Keep the ordinary AppKit image-view backing path, reusable bitmap/image/view objects, bounded
   shrink, autorelease pools, cached virtual coordinates, and 120/60 cadence. Do not restore the
   capture-omitted child CALayer.
4. New display windows must inherit cached click-through/interactive state immediately.
5. Mac note text is `NSColor.labelColor()`, not absolute black/white or a custom dark-mode guess.
6. Permission prompting requires the exact non-symlinked managed app, exact receipt metadata,
   owner-only state directory, atomic 0600 marker, and installed release identity. Fail closed.
7. Prompt order is marker, native AX request on main thread, Settings fallback, calm wait. Never
   simulate consent, add a native settings window, add menu/Dock UI, or change config schema.
8. Grant resumes FirstUX without restart; revoke cancels permission-bound work and returns to wait
   without reopening Settings. Poll no faster than once per second.
9. Sign with exact Developer ID Application team `M9D5379H93`; selected G2 SHA-1 is
   `739B04530883FF9B665C66BD464F98C622971B32`. Sign inside-out, hardened, timestamped, no `--deep`.
10. Both app/helper slices are x86_64+arm64; helper minimum OS is 11.0 in both slices.
11. DMG contains only `Honk300.app`, `Install Honk300.app`, and instructions. Install remains
    per-user. Preserve release tag/version/full commit in bundle and receipt.
12. Preserve one rollback boundary for app, aliases, LaunchAgent, receipt, and only newly migrated
    media. Foreign files and pre-existing user media survive.
13. Keep lifecycle lease/pinned-stream/signal/Restart-Manager/deferred-helper contracts. Do not
    weaken them to accommodate a Windows test; repair the native implementation or fixture.
14. Gait constants/tests and seven goldens protect against overcorrection. Do not re-bless on
    Alienware merely because raster output differs.
15. Terminal windows are absolutely excluded from move/focus/type/drag/ride/collect/nab behavior
    on every platform, including spicy/default-off modes.
16. Alienware cannot replace AppKit capture, TCC, codesign chain/timestamp, `vtool`, CoreAudio,
    `notarytool`, stapler, Gatekeeper, leak, or real Mac terminal evidence. Use the candidate's Mac
    job/physical Mac and record limitations honestly.
17. Do not promote site SHA `8595440` until the real v0.3.3 DMG is published and independently
    fresh-download verified.

## Known Issues & Limitations

- `#m20q` and `#m16r` remain Active. That is intentional, not stale tracking.
- No v0.3.3 candidate was dispatched from this stopping point, so no exact candidate artifact was
  notarized/stapled/downloaded and no publication occurred.
- Exact candidate light/dark/eight-heading/motion/mud/prank/entry/wrap/exit/close captures remain.
- Exact candidate 10+60-second CPU/RSS/growth/leak profile remains. The 5.55% diagnostic is not the
  release claim.
- ADR 0022 first-denied, denied non-nag relaunch, live grant, and live revoke must use one unchanged
  candidate digest. Protected-terminal negatives and an ordinary-window positive remain.
- Ghostty was not installed on this Mac; classifier coverage exists, but live Ghostty evidence
  needs a host with Ghostty or an explicit honest absence record.
- Live multi-monitor/hot-plug is hardware-waived because this Mac has one display. Automated
  topology coverage does not convert that waiver into a live claim.
- Native Windows x64/ARM64 compositor/lifecycle execution and native Linux x64/ARM64 X11/Sway
  execution remain for candidate/hosted CI. Cross-target compilation is green.
- Candidate/default-branch CI, immutable tag, atomic publication, post-release smoke, fresh
  downloads, v0.3.2→v0.3.3 published update, and production site rollout remain open.
- The local App Store Connect key is ignored and owner-only. Never print, stage, commit, email, or
  paste it. GitHub secrets should be referenced only by name.

## Important Context for Future Sessions

- Repository: `/Users/realemmetts/Downloads/temp_git/goose` on this Mac; task worktree used here:
  `/Users/realemmetts/Downloads/temp_git/goose/.worktrees/macos-v0.3.3`.
- Branch: `codex/macos-v0.3.3`. This handoff is committed on the branch; obtain the exact pushed SHA
  with `git rev-parse origin/codex/macos-v0.3.3` after fetching.
- Base at the start of final consolidation: `be7b5a9eed8759048092d9bcb01e13347045caff`.
- Default remote branch: `main`; stable/latest release is v0.3.2.
- Board: run `node .tasks/board-server.mjs ensure`; source of truth is `.tasks/TASKS.md`.
- Signing/notary secret names: `MACOS_CERTIFICATE_P12_BASE64`,
  `MACOS_CERTIFICATE_PASSWORD`, `MACOS_KEYCHAIN_PASSWORD`,
  `APPLE_NOTARY_KEY_P8_BASE64`, `APPLE_NOTARY_KEY_ID`,
  `APPLE_NOTARY_ISSUER_ID`. All six exist in `RealEmmettS/goose`.
- Local key path exists only on the Mac under ignored `.private-release/`; the handoff never stores
  its contents. Directory mode is 0700 and key mode is 0600.
- Website repo: `/Users/realemmetts/Downloads/temp_git/desktop-goose-site`; branch
  `codex/macos-v0.3.3-dmg-first`; exact pushed SHA
  `85954409a54d88019f29c1209102586cfd497bff`; production remains on stable v0.3.2.
- Protected site preview:
  `https://desktop-goose-site-5j5jb7dpi-realemmetts.vercel.app`.
- This Mac was returned to an uninstalled state at handoff: no app, three aliases, LaunchAgent,
  receipt, running Honk300 process, or scoped temp fixture was found.
- Local GNU full-workspace cross-check may fail on macOS for missing Linux ALSA/pkg-config sysroot;
  native hosted Linux is authoritative. Targeted GNU backend checks and musl cross-checks are the
  appropriate Mac-hosted evidence.

## What's Next

1. **Exact first action on Alienware:** clone/fetch the branch and verify the handoff SHA:

   ```powershell
   git fetch origin --prune
   git switch codex/macos-v0.3.3
   git pull --ff-only
   git status --short --branch
   Get-Content docs/agents/handoff/2026-07-14-001-macos-v0-3-3-alienware-resume.md
   ```

2. Run native Windows x64 strict gate and exact layered-compositor/lifecycle smoke. On ARM64, let
   the hosted/candidate ARM runner execute the exact ARM artifact; do not substitute x64 emulation
   for the required ARM result.
3. Confirm ordinary branch CI on the pushed stopping SHA. Fix forward on this branch if native
   Windows finds a real issue; preserve the Mac invariants above.
4. Dispatch candidate mode against the exact full SHA and `tag=v0.3.3`, `candidate=true`. Candidate
   must build/sign/notarize/package every artifact without publishing.
5. Wait for native Windows x64/ARM64 and Linux x64/ARM64 candidate gates. Download the candidate's
   private macOS app ZIP/DMG/evidence artifacts.
6. On this Mac, install that unchanged candidate and complete ADR 0022 four-state TCC evidence,
   exact captures, performance/leak sample, terminal/ordinary-window behavior, helper/autostart,
   v0.3.2 update, rollback injection, preservation, uninstall/purge, and final cleanup.
7. Commit/push evidence updates; ensure candidate/default CI bind the same release code SHA. Do not
   change release code after candidate proof without rerunning candidate.
8. Fast-forward default `main` only after the exact candidate passes. Wait for ordinary CI, create
   the single immutable `v0.3.3` tag, and wait for atomic publication/post-release smoke.
9. Fresh-download app ZIP and DMG; recheck manifest hashes, G2 identity/team, hardened runtime,
   designated requirement, tickets, staples, Gatekeeper, install, and published v0.3.2 update.
10. Validate the protected site preview against the live v0.3.3 manifest, then promote exact site
    SHA `8595440` to the site's default branch and verify thegoose.app returns the real DMG bytes.
11. Close `#m16r` and `#m20q` only after every remaining checkbox is proven or narrowly waived.
