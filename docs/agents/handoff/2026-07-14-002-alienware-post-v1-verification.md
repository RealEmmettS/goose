# Handoff: Alienware post-v1.0.0 verification passes

**Prepared:** 2026-07-14
**Scope:** Windows-host verification after the stable v1.0.0 release
**Board:** `#m20q`
**Repository:** `https://github.com/RealEmmettS/goose`

## Purpose

This handoff is intentionally limited to extra verification on Emmett's Alienware after v1.0.0
is public. It is not a second release plan and does not block or replace the exact-SHA candidate,
hosted Windows/Linux matrix, native Mac qualification, signing/notarization, publication, or
fresh-download checks completed for v1.0.0.

Use the published v1.0.0 artifacts first. Record any defect with screenshots/logs and fix it
forward in the next patch release; never mutate the immutable `v1.0.0` tag or its assets.

## Start from the published release

```powershell
git clone https://github.com/RealEmmettS/goose.git
cd goose
git fetch --tags --prune
git switch main
git pull --ff-only
git rev-parse v1.0.0
gh release view v1.0.0 --json tagName,targetCommitish,isLatest,url,assets
```

Create a clean evidence directory outside the repository. Download fresh assets from the release,
not from a previous runner artifact or local build. At minimum retain the release manifest, its
signature/checksum material, the x64 and ARM64 portable archives, both Global MSI files, both EXE
installers, both Debian packages, the universal app ZIP, and `honk300-universal2.dmg`.

## Alienware verification matrix

### 1. Repository and release identity

- Confirm `main`, tag `v1.0.0`, the GitHub Release target, and `release-manifest.json` resolve to
  the same full commit.
- Confirm GitHub marks v1.0.0 latest and all stable filenames exist exactly once.
- Verify every downloaded byte count and SHA-256 against the manifest.
- Confirm `honk300 --version`, `honk --version`, and `goose --version` all report `1.0.0`.
- Confirm `latest/download/...` bytes match the immutable `download/v1.0.0/...` bytes.

### 2. Native Windows x64 install and lifecycle

- Test the recommended Global x64 MSI from a clean install state.
- Exercise `honk300`, `honk`, and `goose` command grammar plus `setup`, `config`, `start`, `status`,
  `reload`, `stop`, `exit`, `quit`, every `do` action, audio, and `--no-sound`.
- Verify a second instance is rejected and immediate stop/start is reliable.
- Exercise in-place update through all three aliases. The updater must discover `latest`, pin the
  immutable tag/commit/target/kind/size/hash, stay within x64 Windows provenance, and preserve
  configuration and media.
- Verify normal uninstall preserves user media and purge reports a backup/removes owned state.
- Repeat the important path with the EXE installer and portable archive; do not let one installer
  family replace another family or architecture silently.

### 3. Real compositor and DPI behavior

- Run at 100%, 125%, 150%, and one mixed-DPI multi-monitor layout if available.
- Capture light and dark desktops while the goose is moving. Require transparent margins,
  articulated opaque body, outline, wing, orange beak, two-tone legs, and soft shadow with no
  color-channel swap or gray/opaque full-screen background.
- Exercise eight headings, idle/walk/run/charge, blink/breath/tail motion, mud/puddles, prank
  return, exposed-edge entry, fully-hidden wrap, and animated exit.
- Verify touching monitor seams behave as continuous desktop space. On a truly exposed edge, the
  goose may leave and sometimes re-enter from another exposed edge only after fully hidden.
- Confirm neither startup nor stop causes a visible teleport, pop-in, or disappearance.

### 4. Windows interaction safety

- Verify ordinary-window ride and collect behavior with a normal test window and Notepad.
- Close a goose-created note/meme repeatedly and record that annoyed reactions are plausible but
  not guaranteed; mouse nab remains approximately 30% and must still obey configuration,
  manners, pointer state, and capability gates.
- Verify program cleanup, timeout, crash, and shutdown do not count as a user close.
- Prove Windows Terminal, Command Prompt, PowerShell, Git Bash, VS Code terminal, Codex, and any
  available Ghostty window are never moved, focused, typed into, dragged, ridden, collected, or
  selected for cursor mischief. Ordinary non-terminal positives should still work.

### 5. TUI and terminal restoration

- Run the TUI at 80x24 and at a larger size.
- Exercise save, status, start, reload, stop, and quit flows.
- Interrupt and normally exit it; confirm colors, cursor, alternate screen, input mode, and prompt
  are restored each time.
- Pipe `status` into consumers that close early (for example `Select-Object -First 1`) and confirm
  it exits cleanly without masking unrelated stdout errors.

### 6. Cross-platform artifact review from Windows

- Inspect macOS app ZIP/DMG and Debian package identity, filenames, manifest target/kind, sizes,
  and hashes. Windows can verify archive/package structure, but must not claim AppKit,
  Accessibility, CoreAudio, `codesign`, notarization, stapling, Gatekeeper, or native Debian
  compositor behavior.
- Confirm Windows-triggered future releases still require GitHub's macOS producer and do not
  rewrite an older tagged DMG. Managed Mac updates consume the exact-tag universal app ZIP; the
  DMG is a fresh graphical installer, not the in-place update transport.
- Confirm both Debian packages own the expected architecture-specific executable and stable
  aliases, while user media is outside package ownership.

## Mac-specific guardrails for any patch

Do not undo these v1.0.0 contracts while fixing a Windows or shared issue:

1. tiny-skia output is premultiplied RGBA. AppKit copies it directly into alpha-last bitmap
   storage; do not restore BGRA/alpha-first interpretation, per-frame swizzling, or unpremultiply.
2. AppKit keeps reusable bitmap/image/view storage, bounded shrink after large transients,
   autorelease pools, cached virtual-desktop coordinates, 120 Hz simulation, and at most 60 Hz
   presentation. The ordinary backing store is required for screenshots/screen sharing.
3. Notes use macOS semantic label color for light/dark/high-contrast appearance.
4. Automatic Accessibility UI is limited to the exact managed, receipted, non-symlinked signed
   app. It asks once per installed version, never clicks approval, waits calmly at a safe edge,
   and observes live grant/revocation.
5. The macOS `Honk` menu item is main-thread AppKit UI with an explicitly retained action target.
   Configure opens the existing bundled terminal TUI; Quit enters the shared animated walk-off.
   Do not add a duplicate settings schema or abrupt process termination.
6. The app and installer helper remain universal x86_64/arm64, Developer ID signed inside-out
   with hardened runtime and timestamps, notarized, stapled, and Gatekeeper validated. The DMG
   root remains exactly `Honk300.app`, `Install Honk300.app`, and `Read Me.txt`.
7. Mac install stays per-user at `~/Applications/Honk300.app`; no `sudo` and no misleading
   `/Applications` symlink. In-place updates use exact-tag app ZIP bytes selected by the signed
   manifest, while each tagged DMG remains immutable.
8. A Windows-only patch must not weaken terminal protection, platform/architecture isolation,
   graceful edge entry/exit, shared gait bounds, or the collect beak-contact gate.

## Evidence and patch policy

Record OS build, CPU architecture, display topology/scales, artifact SHA-256, exact tag/commit,
commands, exit codes, screenshots, and relevant logs. Add a board activity entry that separates
passed observations from failures and hardware-waived checks.

For a real defect:

1. reproduce it against the fresh published v1.0.0 artifact;
2. add the narrowest failing automated regression;
3. preserve the Mac guardrails and platform isolation above;
4. run the complete repository gate plus the affected native smoke;
5. publish a new semantic patch version such as v1.0.1 through candidate, exact-SHA CI, immutable
   tag, atomic release, and fresh-download verification.

Never force-update `v1.0.0`, replace its assets, weaken a semantic capture threshold merely to
make CI green, or describe Windows archive inspection as native Mac/Linux proof.
