# Handoff: Alienware post-v1.0.1 verification

**Prepared:** 2026-07-15

**Scope:** verification-only follow-up on Emmett's Alienware and later native hardware

**Board:** `#v1a` (To-Do); release card `#m20q` is complete

**Repository:** `https://github.com/RealEmmettS/goose`

**Release:** v1.0.1, the first public stable release, exact commit
`de8da8a9dd049286787d20e167bb115ce8afc107`

## TL;DR

Start from fresh published v1.0.1 bytes and repeat real Windows x64 compositor, installer,
updater, lifecycle, terminal-safety, movement, reaction, audio, and TUI checks. Treat those checks
as defense in depth after publication, not retroactive permission to replace assets or move tags.
Any defect gets a narrow regression and a new semantic patch release. Preserve every macOS
presentation, permission, lifecycle, signing, notarization, and menu-bar contract listed below.

The immutable v1.0.0 tag is intentionally unpublished: its atomic workflow failed before draft
creation because a valid complete top-down Windows pose was judged by a side-only screenshot
oracle. ADR 0027 fixes forward to v1.0.1 with strict complete side and top-down profiles; it does
not change renderer, engine, presenter, or user behavior.

## Published authority

- Candidate `29401457634` passed against the exact release commit.
- Ordinary main CI `29401961540` passed on failed-job-only attempt 2; attempt 1 had one pre-render
  hosted Windows desktop-helper startup flake. Source and workflow inputs stayed fixed; the rerun
  rebuilt/re-executed the failed job and retained its own evidence artifact.
- Atomic release `29403056159` and post-release smoke `29403596212` passed. The latest public
  Release contains 22 payloads plus sidecars/manifest for 47 assets.
- Fresh public DMG and app-ZIP downloads passed manifest/sidecar hashes, universal slices,
  Developer ID team `M9D5379H93`, hardened runtime, secure timestamp, designated requirement,
  notarization/stapling, Gatekeeper, isolated install, and cleanup checks.
- A real isolated published v0.3.2 install updated through the CLI to v1.0.1; all three aliases and
  the receipt converged on the exact app ZIP, repeat updates were clean no-ops, and purge restored
  both the fixture and physical account to uninstalled state.
- Site commit `4f4bf426979e6b4e59c850ef39a8eea6a3d08386` passed live-manifest, preview, CI, browser,
  accessibility, responsive, and budget gates. Vercel production deployment `5454715097` made
  the real DMG the primary public macOS action at `thegoose.app`. Documentation-only closure
  `7f20f1c87d1e2e3545bc33779caf67598b1161b2` also passed CI and production deployment without
  changing the tested bundle.

## State to inherit

- Tag v1.0.1, the GitHub Release, and `release-manifest.json` must all resolve to exact release
  commit `de8da8a9dd049286787d20e167bb115ce8afc107`. `main` may be a descendant containing only
  post-tag documentation/board closure; verify the release commit is its ancestor and do not work
  from v1.0.0.
- The complete hosted candidate/default-branch/publication evidence is release authority. This
  handoff adds physical-hardware observations; it does not weaken or replace that authority.
- `#m20q` is closed after atomic publication, fresh-download trust/hash checks, DMG-first
  production deployment, and tracked-board synchronization. The remaining work lives only on
  To-Do card `#v1a`.
- The Mac used for qualification was returned to an uninstalled/stopped state. Honk300's own
  Accessibility toggle was turned off with explicit operator approval; no other Accessibility
  entry was changed.
- Secrets never belong in the repository, board, handoff, evidence bundle, screenshots, or logs.
  GitHub Actions credentials remain repository secrets and the local P8 copy remains untracked.

## Begin from published bytes

```powershell
git clone https://github.com/RealEmmettS/goose.git
cd goose
git fetch --tags --prune
git switch main
git pull --ff-only
git rev-parse v1.0.1^{}
gh release view v1.0.1 --json tagName,targetCommitish,isLatest,url,assets
```

Create an evidence directory outside the repository. Download from the immutable GitHub Release,
not from a runner artifact, browser cache, or local build. Retain at least:

- `release-manifest.json` and aggregate/hash sidecars;
- Windows x64 portable archive, Global MSI, Corporate MSI, and both EXE installers;
- the Windows ARM64 equivalents for identity/archive inspection;
- `honk300-amd64.deb` and `honk300-arm64.deb`;
- `Honk300-universal2.app.zip` and `honk300-universal2.dmg`.

Verify byte counts and SHA-256 before executing anything. Confirm each stable
`/releases/latest/download/...` object is byte-identical to the v1.0.1 immutable-tag object.

## Alienware verification matrix

### Release and updater identity

- Confirm tag, Release target, and manifest `commit` agree exactly. Confirm the default branch
  contains that release commit as an ancestor; it may also include the documented post-tag
  release-evidence checkpoint.
- Confirm GitHub marks v1.0.1 latest and every required stable filename occurs exactly once.
- Confirm `honk300 --version`, `honk --version`, and `goose --version` report 1.0.1.
- From an installed v0.3.2 fixture, run update through each alias. Discovery may use `latest`, but
  mutation must use the manifest-selected immutable v1.0.1 tag, target, platform, architecture,
  provenance kind, size, and SHA-256.
- Confirm Windows updates never select Mac, Debian, ARM64-on-x64, or a different installer family.
  A release pushed from Windows still obtains its fresh Mac app/DMG from GitHub's macOS runner;
  it does not mutate an older tagged DMG.

### Native Windows x64 install and lifecycle

- Start clean and test the recommended machine-wide Global x64 MSI, including elevation,
  repair, upgrade, downgrade refusal, start/status/reload/stop, immediate restart, and uninstall.
- Verify all three aliases and the full grammar: setup, config, start, status, reload, stop, exit,
  quit, every `do` action, audio, and `--no-sound`.
- Prove singleton enforcement, owner-scoped IPC, socket/pipe cleanup, and restart-required config
  rejection.
- Verify normal uninstall preserves mutable media and purge backs up/removes only owned state.
- Repeat the provenance-sensitive path with the EXE installer and portable archive. Never allow
  one family or architecture to silently replace another.

### Native DWM, DPI, and renderer output

- Run `script/smoke_windows_overlay.ps1` against the exact published x64 executable. Retain both
  controlled-background PNGs, semantic JSON, logs, version/hash evidence, and exit codes.
- The Alienware must use paired live DWM capture. ADR 0026's raw presenter path is available only
  under GitHub's exact hosted ARM64 static-wallpaper predicate. Do not imitate runner environment
  variables or enable its diagnostic hook locally.
- Exercise 100%, 125%, and 150% scaling plus mixed-DPI multi-monitor topology if available.
- Accept either a complete side view or complete top-down view. Both require transparent margins,
  opaque articulated body and wing, outline, asymmetric orange articulation, correct channels,
  and reconstructed premultiplied edge colors. Side view also requires separated beak/two-tone
  legs/shadow. Top-down requires one compact beak, complete wing/body proportions, and correctly
  lacks side-only legs/shadow. Partial, cropped, opaque, channel-swapped, straight-alpha, and
  double-premultiplied surfaces must fail.
- Visually inspect eight headings; idle, walk, run, and charge; blink, breath, and tail motion;
  puddles/mud; prank return; and note/meme windows. The gait refinement should release planted
  feet sooner without skating, snapping, or stretching legs.

### Screen-edge and multi-monitor behavior

- Startup must walk in from a real exposed edge. Stop/Quit must walk completely out before the
  process exits. Neither endpoint may visibly pop, teleport, appear, or disappear.
- Touching monitor edges are continuous space; walking through them must move naturally to the
  adjacent monitor. Gaps and truly exposed edges are not monitors.
- A hidden wrap is allowed only after the entire goose is outside real monitor pixels, and only
  on the probabilistic roaming path. Off-screen errands remain non-wrapping and return through an
  edge. Hot-plug/topology changes must not strand or visibly teleport the goose.

### Interaction and terminal safety

- Verify ordinary-window ride and collect with benign test windows/Notepad.
- Close goose-created notes/memes repeatedly. A user close has roughly a 30% chance to trigger an
  annoyed run and separately gated mouse nab; it is not guaranteed. Cleanup, timeout, crash,
  programmatic close, and shutdown must not count as user intent.
- Cursor movement remains bounded and obeys config, quiet/fullscreen manners, pointer state,
  live capability, and permission gates.
- Prove Windows Terminal, Command Prompt, PowerShell, Git Bash, VS Code terminal, Codex, and any
  installed Ghostty window are never moved, focused, typed into, dragged, ridden, collected, or
  chosen for mischief. Demonstrate an ordinary non-terminal positive so a disabled driver is not
  mistaken for protection.

### TUI and terminal restoration

- Test the TUI at 80x24 and a larger size. Exercise save, status, start, reload, stop, and quit.
- Test normal exit and interruption. Restore colors, cursor, alternate screen, input mode, and
  prompt every time.
- Pipe status to a consumer that closes early and verify only downstream broken-pipe is treated as
  success; unrelated output failures remain visible.

### Cross-platform inspection boundaries

- Windows may verify Mac/Debian filenames, manifest identity, sizes, hashes, archive/package
  structure, and universal slice metadata where tools permit.
- Windows must not claim native AppKit rendering, Accessibility behavior, CoreAudio, `codesign`,
  notarization, stapling, Gatekeeper, macOS install/update, `dpkg`, X11, or Wayland behavior.
- When native Linux is available, keep X11/XWayland and native Wayland reduced mode distinct.
  Native Wayland does not gain foreign-window manipulation or cursor-warp parity merely because
  an archive runs.

## macOS contracts a later patch must preserve

1. tiny-skia emits premultiplied RGBA. AppKit copies it directly into alpha-last bitmap storage.
   Never restore BGRA/alpha-first interpretation, unpremultiplication, or a per-frame swizzle.
2. Each overlay reuses its `NSBitmapImageRep`, `NSImage`, and view storage, shrinks boundedly after
   large transients, uses autorelease pools, caches virtual-desktop coordinates, simulates at
   120 Hz, and presents at no more than 60 Hz. Its ordinary backing store is deliberate for
   screenshots and screen sharing.
3. Mac note text uses AppKit semantic label color so light, dark, increased-contrast, and future
   system appearances remain readable. Do not replace it with absolute black.
4. Automatic Accessibility onboarding runs only for the exact managed, receipted,
   non-symlinked, Developer ID-signed app. It opens the relevant System Settings pane, asks once
   per installed version, never clicks approval, waits calmly at a safe edge, and observes grant
   and revocation in the same process.
5. The macOS-only `Honk` status item is main-thread AppKit UI with a retained action target.
   Configure launches the existing bundled terminal TUI. Quit requests the shared animated
   walk-off. Do not add a duplicate settings schema, abrupt termination, Dock controller, or a
   Windows/Linux tray as an accidental side effect.
6. `Honk300.app` and `Install Honk300.app` remain universal x86_64/arm64, signed explicitly
   inside-out with the G2 Developer ID Application identity for team `M9D5379H93`, hardened
   runtime, secure timestamps, stable designated requirements, notarization, staples, and
   Gatekeeper acceptance. No release-mode ad-hoc fallback is allowed.
7. The DMG root stays exactly `Honk300.app`, `Install Honk300.app`, and `Read Me.txt`; there is no
   misleading `/Applications` symlink. The helper verifies bundle identifier/team and delegates
   to the shared no-sudo transaction into `~/Applications/Honk300.app`.
8. Managed Mac CLI updates use the exact-tag universal app ZIP selected by the verified manifest.
   A DMG is a fresh graphical installer and immutable per tag, while the stable unversioned latest
   URL advances atomically with each complete general release.
9. Preserve the terminal classifier, platform/architecture/provenance isolation, graceful
   entry/exit, monitor-seam logic, gait bounds, collect beak-contact target, and owner-scoped IPC
   when changing any shared code.

## Deferred native Mac checks

When Mac hardware is next available, start with the published v1.0.1 app ZIP/DMG and record:

- a repeated exact-release 10-second warm-up plus 60-second active profile;
- Configure and animated Quit through the status item;
- visible beak contact during collect;
- optional second-hardware repetition of the already-passed published v0.3.2 to v1.0.1 update,
  plus injected rollback, foreign-file preservation, uninstall, and purge;
- ordinary-window positive plus Terminal.app, Codex, VS Code terminal, and Ghostty negatives;
- real multi-monitor/hot-plug only if hardware exists.

The prior physical M2 run measured 5.55% median CPU, 29.52 MiB maximum RSS, negative 9.89 MiB
growth, zero leaks, and 20 clean captures. One unchanged signed executable passed first denied,
non-nag relaunch, same-process grant in 102 ms, and same-process revoke in 127 ms. These are
accepted source-equivalent/product-equivalent evidence with explicit exact-final-SHA,
desktop-driver, absent-Ghostty, and one-display waivers; do not rewrite them as stronger claims.

## Evidence and patch policy

Record OS build, CPU architecture, display topology/scales, artifact SHA-256, exact tag/commit,
commands, exit codes, screenshots, and relevant logs. Update `#v1a` activity with separate passed,
failed, unavailable, and hardware-waived results.

For a real defect:

1. Reproduce against fresh v1.0.1 published bytes.
2. Add the narrowest failing automated regression.
3. Preserve the Mac and cross-platform contracts above.
4. Run the affected native smoke plus repository gates proportionate to the changed surface.
5. Publish a new reviewed semantic patch through candidate, same-SHA default-branch CI, immutable
   tag, atomic release, fresh-download verification, and website manifest validation.

Never force-update v1.0.0 or v1.0.1, replace their assets, weaken a semantic threshold just to
make CI green, expose secrets, or describe archive inspection as native platform proof.
