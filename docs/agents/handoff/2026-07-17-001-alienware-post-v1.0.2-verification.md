# Handoff: Alienware post-v1.0.2 verification and future tray parity

Date: 2026-07-17
Release target: v1.0.2
Primary task: `#v1a`
Release task: `#v102`
Mac/tray decision: ADR 0028

## TL;DR

Start from fresh published v1.0.2 bytes on the Alienware. Repeat the Windows x64 installer,
update, compositor, movement, audio, TUI, terminal-protection, and multi-monitor checks. Treat any
finding as input to a later forward patch; never alter v1.0.0, v1.0.1, or v1.0.2 tags/assets.

The Mac side is deliberately hardened and must not be simplified from a non-Mac host. Future
Windows/Linux tray work is separate implementation work, but it must copy the learned macOS
behavior: the shared goose icon opens the one existing terminal TUI for Configure, and Quit sends
the shared graceful-stop intent so the goose walks fully offscreen before the process exits. Do
not add another settings schema or call an abrupt process-exit/kill API.

## Resolve the exact public identity first

Record the actual values after publication:

```text
Tag: v1.0.2
Release commit: <fill from published tag>
Candidate workflow: <fill>
Same-SHA main CI: <fill>
Atomic release workflow: <fill>
Post-release smoke: <fill>
```

Then verify:

```powershell
git fetch origin --tags
git rev-parse v1.0.2^{}
gh release view v1.0.2 --json tagName,targetCommitish,isLatest,url,assets
```

Download `release-manifest.json` from both the immutable tag and `latest/download`. Require schema
`honk300.release.v1`, version `1.0.2`, tag `v1.0.2`, the same full commit, unique safe filenames,
and exact hash/size agreement. The latest objects must resolve the complete v1.0.2 release; do not
mix a latest manifest with a different tag's payload.

## What changed on macOS and must be preserved

### Pixel and window presentation

- tiny-skia produces premultiplied RGBA. AppKit consumes an alpha-last Device-RGB bitmap directly;
  do not restore the old BGRA/alpha-first bridge or a per-frame swizzle buffer.
- The overlay uses a stable standard-sRGB destination and lets WindowServer perform final display-
  profile composition. Do not add an application-side Display-P3 conversion.
- Reusable bitmap/image/window objects, bounded capacity after transient damage, autorelease pools,
  cached desktop topology, 120 Hz simulation, and at-most-60 Hz presentation are intentional.
- Dark native notes use `NSColor.labelColor()`; do not replace it with a fixed black foreground.
- The shared renderer/gait refinements are platform-neutral. Do not fork Mac rig geometry or undo
  the four-pixel plant release and speed-aware lag caps from Windows.

### Permission first run

- Only the exact receipted app at `~/Applications/Honk300.app` may prompt automatically.
- The owner-only per-version marker is written before native consent/Settings UI opens.
- Denied mode is a calm engine-owned safe-edge wait: honk/status/reload/stop remain available;
  permission-bound pranks are busy/suppressed.
- Grant or revocation is detected in the same process. Do not rebuild between denied/granted
  evidence, simulate a click into Settings, or broaden prompting to developer/bare/mounted copies.
- The menu remains available while permission is denied.

### Shared goose control surface

- Canonical source: `Assets/UI/honk300-status-goose.svg`.
- AppKit runtime representation: `Assets/UI/honk300-status-goose@2x.png`, 36×36 RGBA rendered as
  an 18-point template. The PNG avoids raw-SVG decoder dependence across macOS 11+.
- Production packaging seals both files before signing and checks both in the staged app, final
  app ZIP, first DMG mount, and final notarized/stapled DMG remount.
- The visible macOS item is image-only and square. Its independent accessible name and tooltip are
  **Honk300 controls**. AppKit supplies light/dark/highlight tint; no background was added.
- Rust retains the image and weak AppKit action target for the full item lifetime. An unbundled
  development copy may fall back to variable-width **Honk** text if the resource is unavailable;
  a cosmetic failure must not stop the runtime.

### Exact action behavior to copy later

1. **Configure Honk300…** opens the sealed `Configure Honk300.command` resource, which executes
   that same bundle's `Contents/MacOS/honk300 config` command. It is the existing 120×30 ratatui
   editor with the same schema, validation, save, status, start/stop, and reload paths. Pressing
   `q` restores the terminal and leaves the goose running.
2. **Quit Honk300** records the same engine-owned graceful-stop intent as CLI/TUI stop. Simulation
   and presentation continue until the full goose/effects are beyond a real exposed edge, the
   final transparent frame is acknowledged, runtime-owned props are cleaned, and the singleton is
   released. Do not use `terminate:`, `exit`, `TerminateProcess`, a tray-library immediate quit,
   or a kill signal as the normal handler.
3. The item exists only while the runtime exists. It adds no app background, native preferences
   window, network control, global quit key, or exception to protected-terminal rules.

## Future Windows/Linux tray work

This handoff is verification-first. If a later task implements trays, create/update a platform
ADR and board card rather than folding unreviewed UI into a verification patch.

- Derive Windows `.ico`/notification sizes and Linux PNG/SVG assets from the canonical SVG. Keep
  the source and its provenance discoverable; do not redraw per platform.
- Use each platform's accessible label/name equivalent to **Honk300 controls**.
- Configure must launch `<installed binary> config` in a normal user terminal with no elevation,
  preserve terminal restoration, and never invent a second settings model.
- Quit must enter the existing local IPC/engine graceful stop and keep the tray responsive until
  the process naturally finishes. Never delete the icon first and kill the goose second if that
  makes it visibly disappear.
- The tray does not confer cursor/window permissions. Capability, quiet/fullscreen manners,
  pointer state, and terminal protection remain authoritative.
- Windows/Linux packaging and uninstall must own/remove only their own tray integration. User
  config/media and foreign files remain preserved.
- Qualify keyboard/screen-reader access, explorer/shell restart, session logout, single-instance,
  denied capability behavior, and immediate restart on real hardware before claiming parity.

## Alienware verification pass

### Installation and updates

- Fresh-download x64 Global MSI, portable archive, PowerShell installer, manifest, and sidecars.
- Verify Authenticode/hash/manifest identity before execution.
- Install Global MSI, check Program Files, machine PATH, all-users Start Menu, Add/Remove Programs,
  aliases, source marker, and version.
- From a preserved v1.0.1 install, run each of `honk300 update`, `honk update`, and `goose update`
  in isolated repetitions. Require in-place convergence to v1.0.2 and a clean repeat no-op.
- If the original install came from the PowerShell bootstrap and a later update selects its
  authoritative Global MSI marker, inspect the per-user JSON receipt. Its version/tag/commit are
  currently informational after a direct MSI update: registry/source markers, exact-path update
  proof, and uninstall ownership do not consume them. Do not loosen those authoritative checks.
  A later patch may atomically refresh only a regular, non-reparse receipt whose schema and
  install root already match, but that change requires a Windows regression and hardware proof.
- Exercise repair, downgrade refusal, rollback/failure behavior, ordinary uninstall, and purge
  preservation/backup. Do not manually overwrite a running binary.

### Runtime and compositor

Run the exact published x64 executable:

```powershell
pwsh -File script/smoke_windows_overlay.ps1 `
  -Binary <path-to-published-honk300.exe> `
  -EvidenceDirectory target/windows-overlay-evidence-v1.0.2
```

Inspect both dark/light paired-DWM captures and retained analysis. Require transparent margins,
correct body/shade/outline/wing/orange colors, and one complete side or top-down view. A side view
must show beak, two-tone legs, and shadow; a top-down view must show its complete compact body/wing
without pretending it has visible legs. Do not weaken the oracle for a partial entrance frame.

Then exercise:

- all three CLI names; start/status/reload/stop/exit/quit and immediate restart;
- 80×24 config TUI navigation, save/status/start/stop, `q`, and terminal restoration;
- honk, wander, mud, nab, meme, note, audio, and `--no-sound`;
- entry, ordinary monitor seams, fully hidden wrap, errands, prank return, and graceful exit;
- eight headings, walk/run/charge, planted feet, blink/breath/tail/wing motion, puddles and mud;
- user-close reaction probability/effect without treating cleanup as user closure;
- cursor/window behavior on ordinary windows and strict negatives for Terminal, Codex, VS Code
  terminal, Windows Terminal, PowerShell, cmd, and any other installed terminal;
- real multi-monitor seams, differently scaled displays, negative coordinates, and hot-plug if the
  Alienware setup supports them.

### Linux boundary

Do not infer Linux success from Windows. Hosted v1.0.2 gates still rebuild x64/ARM64 GNU/musl,
X11 and dual-output native Wayland reduced mode, Debian packages, aliases, update, uninstall, and
purge. Any later real Linux observation must keep X11/XWayland and native Wayland capability
claims distinct; collect-window remains unsupported on Linux.

## Reporting and fixes

- Append commands, hashes, screenshots/evidence paths, hardware/display topology, and honest
  unavailable checks to `.tasks/tasks/v1a.md` and a readiness/handoff record.
- Reproduce any finding against fresh published bytes, add a failing regression, and preserve the
  accepted Mac presentation/security/control contracts unless the evidence directly disproves
  them.
- Any source fix after v1.0.2 uses v1.0.3 or newer through candidate mode, exact-SHA main CI, one
  new immutable tag, atomic publication, post-release smoke, and fresh artifact verification.
