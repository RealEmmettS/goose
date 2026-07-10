# macOS Hands-On Accessibility Checklist

When a real Mac (or a pre-granted self-hosted macOS runner) is available, walk this checklist to
turn the CI-proven macOS packaging into the Accessibility-granted, visually-verified evidence that
board task `#m16r` needs. Everything up to this point is machine-checked in CI; the steps below are
the ones that require a human at a Mac. See ADR 0018 for the current distribution contract and
ADR 0010 for the LSUIElement agent-bundle identity.

Scope reminder: the app is ad-hoc signed and **not notarized**. macOS may require explicit
approval under Privacy & Security. Terminal installation does not replace Developer ID signing or
notarization, and an upgrade may require Accessibility reauthorization.

## 0. Prerequisites

- A macOS 11+ machine (Apple Silicon or Intel; the app is universal2).
- The release you are validating exists on GitHub with `honk300-installer.sh`,
  `honk300-universal2.app.zip`, `release-manifest.json`, and their sidecars.
- This repo checked out (for the smoke scripts).

## 1. Install through the supported terminal path

Run:

```sh
curl --proto '=https' --tlsv1.2 -fsSL https://github.com/RealEmmettS/goose/releases/latest/download/honk300-installer.sh | sh
```

Confirm:

- `~/Applications/Honk300.app` exists,
- `~/.local/bin/{honk300,honk,goose}` are symlinks into `Contents/MacOS/honk300`,
- `~/Library/Application Support/honk300/install-receipt.json` names the exact tag, commit,
  artifact, and SHA-256,
- `codesign --verify --deep --strict ~/Applications/Honk300.app` succeeds,
- `lipo ~/Applications/Honk300.app/Contents/MacOS/honk300 -verify_arch x86_64 arm64` succeeds,
- `honk300 status` reports the macOS platform and bundle.

The DMG is a hidden v0.2.1 updater-compatibility asset and is not part of this user install test.

## 2. First run and Privacy & Security approval

1. Start with `honk300 start`. If macOS blocks the unnotarized app, follow the approval shown in
   **System Settings → Privacy & Security**, then start it again. Record the exact prompt and
   approval path; do not assume it persists across upgrades.
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
- the receipt, exact artifact checksum, bundle-slice, and code-sign verification results,
- the `smoke_m16_macos_accessibility.sh` outcome,
- the visual-verification checklist result above,
- a link to any self-hosted Accessibility CI run if used.

Then flip `#m16r` closed on the board once the Accessibility-granted evidence is recorded. Until
then, `#m16r` stays open with hosted macOS proving only bundle/status/IPC and denied/degraded
behavior.
