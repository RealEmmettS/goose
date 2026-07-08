# macOS Hands-On Checklist (R3)

When a real Mac (or a pre-granted self-hosted macOS runner) is available, walk this checklist to
turn the CI-proven macOS packaging into the Accessibility-granted, visually-verified evidence that
board task `#m16r` needs. Everything up to this point is machine-checked in CI; the steps below are
the ones that require a human at a Mac. See ADR 0017 for the packaging contract and ADR 0010 for
the LSUIElement agent-bundle identity.

Scope reminder: artifacts are **unsigned personal-use** (ad-hoc codesign only). First launch is
Gatekeeper-quarantined by design — the right-click-Open step below is expected, not a defect.

## 0. Prerequisites

- A macOS 11+ machine (Apple Silicon or Intel; the app is universal2).
- The release you are validating exists on GitHub with:
  - `honk300-universal2.dmg` + `honk300-universal2.dmg.sha256` (from `macos-packaging.yml`), and
  - `honk300-{x86_64,aarch64}-apple-darwin.tar.xz` + sidecars (from the Release workflow).
- This repo checked out (for the smoke scripts).

## 1. Install from the DMG

1. Download `honk300-universal2.dmg` and its `.sha256`.
2. Verify the checksum: `shasum -a 256 -c honk300-universal2.dmg.sha256` → must print `OK`.
3. Open the DMG (`hdiutil attach honk300-universal2.dmg` or double-click) and drag `Honk300.app`
   onto the `Applications` symlink (or into `~/Applications`).
4. Detach the DMG.

Alternative (lifecycle path, to exercise the Rust installer): from the mounted or copied app, run
`/Applications/Honk300.app/Contents/MacOS/honk300 install --autostart`. Confirm:
- `~/Applications/Honk300.app` exists,
- `~/.local/bin/{honk300,honk,goose}` are symlinks into `Contents/MacOS/honk300`,
- `~/Library/LaunchAgents/dev.emmetts.honk300.plist` exists (only with `--autostart`),
- `honk300 status` reports the macOS platform and bundle.

## 2. First run (right-click → Open)

1. In Finder, **right-click `Honk300.app` → Open**, then confirm the Gatekeeper prompt. This
   approves the unsigned app once; afterwards it launches normally.
2. Confirm the LSUIElement agent starts with no Dock icon and no menu-bar UI (control is CLI/TUI
   only). `honk300 status` should show it running.

## 3. Grant Accessibility

1. Open **System Settings → Privacy & Security → Accessibility**.
2. Enable `Honk300` (add it with `+` if absent, pointing at `~/Applications/Honk300.app`).
3. Restart the goose (`honk300 stop` then start it) so it picks up the grant.
4. `honk300 status` should now report Accessibility as granted and the cursor/window capabilities
   as supported.

## 4. Run the Accessibility smoke

From the repo checkout on the Mac:

```bash
bash script/smoke_m16_macos_accessibility.sh
```

This exercises the pre-granted cursor-nab plus note/meme collect paths. Capture its full output.

## 5. Visually verify Renderer V2

With the goose running and Accessibility granted, confirm by eye (ADR 0014 renderer):

- [ ] The goose walks in all 8 directions; the dual-view rig crossfades to the top-down view for
      steep up/down headings and back to the side profile for shallow ones (no whole-body spin).
- [ ] Feet plant and step (no ice-skating); footmarks stamp at plant events.
- [ ] Blink / breath / honk-tail-flick secondary motion is visible.
- [ ] `honk300 do nab` performs a bounded cursor nab and releases.
- [ ] `honk300 do note` and `honk300 do meme` collect a note/meme window (macOS collect path);
      terminal windows are never targeted.
- [ ] Mud appears only after an off-screen puddle hop, and off-screen errands happen (ADR 0016).

## 6. Record evidence

Append a dated entry to `docs/readiness/m16-m18-readiness.md` (CI Evidence Log) capturing:

- the release tag and machine (arch, macOS version),
- the checksum-verify result,
- the `smoke_m16_macos_accessibility.sh` outcome,
- the visual-verification checklist result above,
- a link to any self-hosted Accessibility CI run if used.

Then flip `#m16r` closed on the board once the Accessibility-granted evidence is recorded. Until
then, `#m16r` stays open with hosted macOS proving only bundle/status/IPC and denied/degraded
behavior.
