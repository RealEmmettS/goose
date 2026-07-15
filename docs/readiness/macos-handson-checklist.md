# macOS Native Qualification And Release Checklist

Use this runbook on a real Mac to qualify the exact v1.0.1 release candidate and then verify the
published artifacts. It supplies the native evidence required by `#m16r` and the macOS portions
of `#m20q`; `docs/readiness/v1.0.1-readiness.md` remains the release source of truth. ADR 0020
defines the Developer ID/notarized DMG, and ADR 0022 defines managed Accessibility onboarding.

The public macOS deliverable is a Developer ID-signed, Apple-notarized, stapled universal DMG.
The DMG-first graphical path installs per-user into `~/Applications/Honk300.app`; the exact-tag
shell bootstrap is a supported secondary terminal path. Release mode never falls back to ad-hoc
signing. Do not use a locally ad-hoc-signed or unnotarized artifact as release evidence.

## 0. Evidence discipline and prerequisites

- Use a macOS 11+ host. Record Mac model, architecture, macOS version, and display count.
- Start from the full 40-character candidate commit selected for v1.0.1. Candidate mode must
  complete signing, notarization, packaging, assembly, smoke, and rollback gates for that SHA
  without publishing or creating the tag.
- Obtain the candidate's final `honk300-universal2.dmg`, `honk300-universal2.app.zip`,
  `release-manifest.json`, and SHA-256 sidecars from that candidate run. Keep the workflow's
  notarization JSON as internal evidence.
- Create one evidence directory outside the app bundle. Record commands, full output, screenshots,
  artifact hashes, app-binary hash, receipt identity, timings, and process metrics there.
- Do not rebuild, re-sign, replace, or mutate the app after recording its binary hash. Every
  denied/granted Accessibility state, runtime observation, and performance sample below must use
  that unchanged installed identity.
- Permission prompts and protected-window behavior require direct operator observation. Do not
  automate consent clicks or type into System Settings. Reset only Honk300's prompt marker and
  `dev.emmetts.honk300` Accessibility record, and only with the explicit opt-ins described below.

## 1. Validate the candidate DMG before installation

Record the manifest and sidecar checks, then run the native trust checks against the exact DMG:

```sh
shasum -a 256 honk300-universal2.dmg
codesign --verify --strict --verbose=2 honk300-universal2.dmg
xcrun stapler validate honk300-universal2.dmg
spctl --assess --type open --context context:primary-signature --verbose=4 \
  honk300-universal2.dmg
```

Mount the DMG read-only. Confirm its root contains exactly the intended user-facing payload:

- `Honk300.app`,
- `Install Honk300.app`, and
- `Read Me.txt`.

There must be no `/Applications` symlink. For both apps, record strict `codesign` verification,
the designated requirement, authority chain, `TeamIdentifier=M9D5379H93`, hardened-runtime flag,
secure timestamp, and Gatekeeper execution assessment. Also confirm both executables are
universal and that the helper's two slices target macOS 11.0:

```sh
codesign --verify --strict --verbose=4 '/Volumes/Honk300/Honk300.app'
codesign --verify --strict --verbose=4 '/Volumes/Honk300/Install Honk300.app'
codesign -d -r- --verbose=4 '/Volumes/Honk300/Honk300.app'
codesign -d -r- --verbose=4 '/Volumes/Honk300/Install Honk300.app'
spctl --assess --type execute --verbose=4 '/Volumes/Honk300/Honk300.app'
spctl --assess --type execute --verbose=4 '/Volumes/Honk300/Install Honk300.app'
lipo '/Volumes/Honk300/Honk300.app/Contents/MacOS/honk300' -verify_arch x86_64 arm64
lipo '/Volumes/Honk300/Install Honk300.app/Contents/MacOS/Install Honk300' \
  -verify_arch x86_64 arm64
xcrun vtool -show-build \
  '/Volumes/Honk300/Install Honk300.app/Contents/MacOS/Install Honk300'
```

Use the actual volume name and helper executable name reported by the mounted candidate if they
differ. A missing ticket, rejected assessment, wrong team, thin binary, missing timestamp, or
unexpected entitlement is release-blocking.

## 2. Exercise the DMG-first graphical install

Open **Install Honk300** from the mounted DMG. Confirm the signed helper:

1. accepts only its sibling `Honk300.app` with bundle id `dev.emmetts.honk300` and the same
   Developer ID team;
2. installs without `sudo` into `~/Applications/Honk300.app`;
3. presents a native success dialog and opens the installed app; and
4. presents an actionable native failure for a deliberately invalid fixture without partially
   changing the managed installation.

Confirm the installed state:

- `~/Applications/Honk300.app` is a real bundle, not a symlink;
- `~/.local/bin/{honk300,honk,goose}` point into its `Contents/MacOS/honk300`;
- `~/Library/Application Support/honk300/install-receipt.json` uses schema
  `honk300.install.v1`, channel `dmg`, layout `mac-app`, and the exact v1.0.1 tag and full
  candidate commit;
- strict signature, Team ID, designated requirement, hardened runtime, timestamp, stapled ticket,
  Gatekeeper assessment, and both architecture slices still pass after installation; and
- mutable media and state are outside the sealed bundle.

The helper opens the app after success, so it may begin managed permission onboarding. Stop it
before the controlled four-state test, then use the scoped reset below. A launch directly from
the mounted `Honk300.app`, a source-tree bundle, or a bare binary must remain denied/degraded
without opening permission UI automatically.

## 3. Prove the four Accessibility states on one unchanged app

Before starting, record these values and retain them through all four states:

```sh
APP="$HOME/Applications/Honk300.app"
BIN="$APP/Contents/MacOS/honk300"
shasum -a 256 "$BIN"
codesign --verify --strict "$APP"
```

Run the interactive live smoke from the candidate checkout. The reset flags are deliberate,
scoped opt-ins: they remove only the v1.0.1 Honk300 prompt marker and reset only bundle id
`dev.emmetts.honk300` in the Accessibility database. Use `HONK300_FINAL_CLEANUP=keep` until the
remaining native tests and profile are complete.

```sh
HONK300_APP="$HOME/Applications/Honk300.app" \
HONK300_SKIP_BUILD=1 \
HONK300_ACCESSIBILITY_PHASE=live \
HONK300_RESET_PROMPT_MARKER=1 \
HONK300_RESET_TCC=1 \
HONK300_FINAL_CLEANUP=keep \
sh script/smoke_m16_macos_accessibility.sh
```

At the script's interactive prompts, type a token only after observing its condition:

- `PROMPTED`: the first denied launch showed one native request, opened Accessibility settings,
  created owner-only `0700` state directories plus a `0600` version marker, and left the goose
  visibly calm at the lower-right safe edge. Status/reload/honk/stop work; wander, mud, nab,
  meme, and note return `BUSY`.
- `NON_NAG`: the same binary and unchanged marker relaunched denied without reopening either the
  native request or Settings, while the safe wait remained controllable.
- `GRANTED`, then `FIRSTUX`: after the operator enables Honk300 in Accessibility, the still-
  running process reports Accessibility/cursor/window support within the recorded deadline,
  leaves the safe edge, and begins a fresh FirstUX without a rebuild or restart.
- `REVOKED`, then `REVOCATION_QUIET`: after the operator disables the same row, the same process
  reports denial, abandons permission-bound work, returns to the safe wait, and does not reopen
  permission UI.

Record the executable hash and marker fingerprint before and after the sequence. They must remain
unchanged. Capture status output and grant/revocation elapsed times. `#m16r` cannot close from
automated adapter tests or a previously granted build alone.

## 4. Verify native functionality and protected windows

Re-enable Accessibility for these granted-mode checks without rebuilding the app. Exercise and
record:

- the `honk300`, `honk`, and `goose` grammar and aliases;
- setup/config/TUI at 80x24, start, status, reload, stop, exit, and quit, including terminal
  restoration;
- honk, wander, mud, nab, meme, note, audio, and `--no-sound`;
- single-instance refusal, owner-only runtime directory/socket, peer ownership, hot reload, and
  restart-required rejection;
- ordinary-window watch/ride and note/meme collection as positive controls; and
- note readability in light, dark, and increased-contrast appearances.

Then use Terminal.app, Ghostty, Codex, a Visual Studio Code integrated terminal, and the shell
running this test as negative controls. The goose may overlap them visually, but must never
focus, type into, move, drag, ride, or collect them. Test every app available on the host and
record any unavailable app as a software-limited waiver backed by the automated application-
identity classifier; never report an uninstalled app as live-tested.

## 5. Capture renderer and behavior evidence

Capture the unchanged candidate on both light and dark backgrounds. Require a visible opaque
body, outline, wing, beak, legs, and shadow; there must be no translucent white/purple blob.
The alpha-last premultiplied-RGBA bitmap must remain Device RGB, the overlay window must report
the stable standard-sRGB destination, and WindowServer—not the app—owns final display-profile
composition.
Record frames for:

- all eight headings and the side/top dual-view transition;
- idle and walk cycles, restrained planted-foot release, blink, breath, and tail motion;
- puddle hop, resulting mud/footmarks, and prank-return frames; and
- note and meme windows, including readable dark-mode note text.

Also record lifecycle/movement without accepting a visible pop:

- launch with the complete pose staged beyond a genuinely exposed edge and watch it walk in;
- cross a touching-monitor seam naturally where hardware permits;
- observe one ordinary exposed-edge wrap, proving the pose is fully hidden before it returns from
  the opposite exposed edge;
- observe a deliberate puddle/prank errand return through its own departure edge without wrapping;
  and
- issue stop/exit/quit and keep capture running until the complete pose has walked out and the
  singleton is released, then immediately restart successfully within the bounded client wait.

Spawn a note or meme and close it manually enough times to observe the probabilistic user-close
path. Record the visible annoyed reaction when it occurs. If the existing bounded nab follows,
verify live Accessibility, mouse-steal configuration, pointer availability, and manners allow it;
program cleanup must never trigger the reaction. Linux's no-collect limitation is outside this
native Mac observation and must remain documented rather than emulated.

Ordinary walking must keep its weighted cadence and must not look stretched or overcorrected.
Do not refresh the shared engine goldens merely to hide a native presentation defect.

## 6. Run the exact-candidate performance profile

With the same signed binary running in its normal granted mode, allow a 10-second warm-up and
then sample for 60 seconds. Record the binary hash again plus:

- median CPU at or below 10%;
- maximum RSS at or below 80 MiB;
- RSS growth at or below 10 MiB;
- zero leaks/leaked bytes from Apple's `leaks` tool; and
- unchanged 120 Hz simulation and at-most-60 Hz presentation rates.

The earlier development profile is useful baseline evidence but does not replace this final
signed-candidate run.

The latest standard-sRGB development diagnostic measured 5.55% median CPU, 29.52 MiB maximum
RSS, negative 9.89 MiB growth, zero leaks, and 20 clean compositor captures. Treat those numbers
only as a comparison point: repeat every metric above on the exact candidate and do not copy the
diagnostic result into the candidate evidence.

## 7. Complete lifecycle and rollback qualification

Use isolated homes/fixtures where the scripts support them, and keep the real user install only
for native UI observations. On the exact candidate package, prove:

- graphical install, opt-in autostart, stop/start, normal uninstall, and purge;
- a receipted published v0.3.2 app updates to v1.0.1 through `honk300 update`;
- injected failure restores the previous app, receipt, aliases, LaunchAgent, and user content;
- normal uninstall preserves foreign files and user media; purge backs media up before removal;
  and
- a foreign app, alias, receipt, LaunchAgent, symlink, or dangling symlink is refused and
  preserved.

After all native evidence is captured, run the live smoke once more with the separate destructive
cleanup opt-in, or perform the equivalent verified commands:

```sh
HONK300_APP="$HOME/Applications/Honk300.app" \
HONK300_SKIP_BUILD=1 \
HONK300_ACCESSIBILITY_PHASE=live \
HONK300_RESET_PROMPT_MARKER=1 \
HONK300_RESET_TCC=1 \
HONK300_FINAL_CLEANUP=purge-managed-install \
sh script/smoke_m16_macos_accessibility.sh
```

Confirm the Mac is returned to an uninstalled state: no Honk300 app, aliases, LaunchAgent,
socket, receipt, prompt marker, or test media. Reset only Honk300's Accessibility record where
macOS permits.

## 8. Multi-monitor evidence and waiver

Run all automated signed-coordinate, display-union, topology-change, and primary-display tests.
If this Mac still has one display, record live multi-monitor and hot-plug as hardware-waived with
the detected topology. Do not claim live multi-monitor validation. If a second display becomes
available, separately record crossing negative coordinates, per-display presentation, and
hot-plug recovery.

## 9. Publish and independently verify the immutable release

Only after the exact commit passes the pre-tag release checklist:

1. advance the default branch to the candidate-proven SHA and wait for ordinary CI on that SHA;
2. create and push the single immutable `v1.0.1` tag;
3. wait for atomic publication and post-release smoke; and
4. confirm the GitHub release is latest, complete, and bound to the intended commit.

Fresh-download the public app ZIP, DMG, manifest, and sidecars instead of reusing candidate files.
Independently repeat hashes, Developer ID team/chain, designated requirement, hardened runtime,
notarization, stapling, Gatekeeper, DMG layout, graphical install, and the live v0.3.2-to-v1.0.1
update. The app inside the final ZIP must already carry its stapled ticket.

Only after those checks may the website promote the signed/notarized universal DMG above the
secondary terminal install. Verify the production macOS link returns the real immutable v1.0.1
DMG and that release-manifest validation fails closed if it is absent.

## 10. Record and close

Append a dated evidence entry to `docs/readiness/v1.0.1-readiness.md` and the granted native
evidence to `docs/readiness/m16-m18-readiness.md`. Include:

- exact tag, full commit, workflow run, artifact and binary hashes;
- Mac/macOS/display topology and any explicit hardware/software waiver;
- signature, notarization, stapling, Gatekeeper, receipt, and universal-slice results;
- all four Accessibility states and operator confirmations;
- ordinary-window positives and terminal negatives;
- renderer captures, performance/leak metrics, lifecycle/update/rollback results; and
- fresh-download and production-link verification.

Close `#m16r` only after the exact unchanged signed-app Accessibility evidence is recorded. Close
`#m20q` only after publication, fresh-download verification, website production promotion, final
cleanup, and every release-blocking verification item is complete or explicitly waived.
