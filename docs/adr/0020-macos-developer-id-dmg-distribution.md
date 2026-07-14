# ADR 0020 — Developer ID macOS Distribution And Per-User DMG Install

- Status: Accepted (2026-07-12)
- Supersedes: only the macOS primary-install, ad-hoc-signing, hidden-DMG, and macOS artifact
  verification portions of ADR 0018. ADR 0018's atomic publication, immutable-tag,
  release-manifest, Linux bootstrap, Windows MSI, and no-crates.io decisions remain accepted.
- Relates to: ADR 0010 (stable bundle identity), ADR 0015 (lifecycle safety), ADR 0017
  (historical macOS packaging), and ADR 0019 (runtime stabilization).

## Context

The first native Apple Silicon qualification exposed a platform bridge defect: tiny-skia
produced a correct premultiplied-RGBA goose, while the AppKit bitmap declared a different byte and
alpha contract. The resulting window showed a nearly transparent white/purple shape even though
the platform-free engine goldens were correct. The same qualification also established that an
ad-hoc signature is not an acceptable public identity for durable Accessibility permission or a
prominent graphical download.

Honk300 already installs per-user and owns an atomic lifecycle transaction. The public macOS
artifact therefore needs a graphical entry point which reuses that transaction without becoming
a second installer implementation, requesting administrator access, or moving mutable media into
the sealed application bundle.

## Decision

### Native rendering contract

- tiny-skia's premultiplied RGBA bytes are copied directly into an alpha-last
  `NSBitmapImageRep`; no BGRA swizzle buffer is allowed.
- Native tests round-trip asymmetric red, green, blue, and alpha values through AppKit and
  CoreGraphics. Visual qualification requires body, outline, wing, beak, legs, and shadow on
  light and dark backgrounds.
- The engine rig and reference renderer remain platform-neutral. Gait refinements and shared
  renderer allocation improvements apply on every backend; AppKit presentation remains a macOS
  concern.
- macOS reuses each display's bitmap/image storage, presents at at most 60 Hz, advances the
  simulation at 120 Hz, drains native objects inside autorelease pools, and caches display
  geometry until topology changes.
- The reusable `NSImageView` participates in the ordinary AppKit window backing store so
  WindowServer screenshots and screen sharing observe the same alpha-composited pixels as the
  physical display. A custom child layer that bypasses that capture path is not used.
- The bitmap and overlay window both declare Device RGB. AppKit must not repeat a Device RGB to
  display-profile ICC conversion in the application process on every frame; WindowServer owns
  final per-display composition. Native capture and palette checks guard the resulting color.
- Native canvas and bitmap capacity may be bucketed for ordinary frame-size jitter, but must
  shrink after an unusually large transient frame. Only the active RGBA rectangle is presented;
  stale transparent capacity must not be color-converted and composited indefinitely.

### Installation shape

- The recommended macOS download is `honk300-universal2.dmg`, containing `Honk300.app`,
  `Install Honk300.app`, and a short read-me. There is no `/Applications` symlink because the
  managed destination is `~/Applications/Honk300.app`.
- `Install Honk300.app` verifies that its sibling target has bundle identifier
  `dev.emmetts.honk300`, that both bundles have valid signatures, and that both share the same
  nonempty Developer ID team identifier. It then invokes the sibling bundle's existing
  `honk300 install` command without `sudo`, reports native success or actionable failure, and
  opens the installed app on success.
- The graphical helper is itself universal x86_64/arm64, with both slices built for the declared
  macOS 11.0 minimum; a host-native-only or host-deployment-target build is rejected.
- `honk300 install` accepts its exact managed binary or the exact sealed binary inside a mounted
  source `.app`. A source install validates bundle identifier, version, tag, full commit, strict
  signature, and x86_64/arm64 slices before an atomic same-filesystem swap.
- Aliases, autostart, receipt ownership, rollback, foreign-file preservation, media migration,
  update, uninstall, and purge stay in the shared lifecycle code. DMG receipts use
  `honk300.install.v1`, channel `dmg`, layout `mac-app`, and the stamped release identity.
- Bundle activation and external integrations form one rollback boundary. A failure after the
  bundle swap restores/removes aliases, LaunchAgent, receipt, and only newly migrated media,
  preserving foreign files and pre-existing user media.
- The exact-tag, SHA-256-verifying shell bootstrap remains a supported secondary terminal path.

### Signing and notarization

- Stable macOS code is signed inside-out with `Developer ID Application: ES Development LLC
  (M9D5379H93)`, hardened runtime, and secure timestamps. No unnecessary entitlements and no
  `codesign --deep` signing are permitted.
- Candidate and release workflows fail closed unless all six credential secrets exist:
  `MACOS_CERTIFICATE_P12_BASE64`, `MACOS_CERTIFICATE_PASSWORD`,
  `MACOS_KEYCHAIN_PASSWORD`, `APPLE_NOTARY_KEY_P8_BASE64`, `APPLE_NOTARY_KEY_ID`, and
  `APPLE_NOTARY_ISSUER_ID`.
- CI decodes credentials without logging values, imports the P12 into an ephemeral keychain,
  deletes temporary key material, and removes the keychain even on failure.
- The app is signed, zipped temporarily, submitted with `notarytool --wait`, stapled, validated,
  and only then placed in the final app ZIP. The helper and stapled app are placed in the DMG;
  the DMG is signed, notarized, stapled, and validated separately.
- Release-mode validation requires the expected Developer ID team, hardened runtime, stable
  designated requirement, notarization success, stapling, and Gatekeeper acceptance. Notarization
  JSON is retained as internal workflow evidence.

### Publication and promotion

- ADR 0018's candidate-first, exact-SHA, atomic draft publication remains unchanged. Missing
  signing or notarization credentials fail candidate mode; there is no release ad-hoc fallback.
- The website may recommend the DMG only after the immutable v0.3.3 release is live and a fresh
  download independently passes checksum, signature, team, notarization, stapling, Gatekeeper,
  install, and update checks.
- Live multi-monitor testing may be recorded as hardware-waived on a one-display Mac only after
  automated signed-coordinate and topology coverage passes; it must not be claimed as live proof.

## Consequences

- macOS users receive a no-sudo graphical install with a stable Apple-verified identity, while
  operators retain the exact-tag terminal path.
- The helper cannot install an adjacent foreign or differently signed app, and it cannot drift
  from lifecycle ownership because it delegates to the product command.
- A release cannot be assembled on a machine or runner missing notarization credentials.
- Developer ID and App Store Connect account setup are operational prerequisites. An unavailable
  login keychain or absent API key is a release blocker, not a reason to weaken signing.

## Verification

- Native AppKit/CoreGraphics pixel tests and light/dark semantic screenshot assertions.
- Full-compositor and window-capture assertions after a large transient frame, plus an active-
  motion CPU/RSS/leak profile rather than an idle or off-screen sample.
- Shared gait invariants, eight foot directions, renderer goldens, and platform runtime tests.
- Exact prebuilt-app smoke scripts for denied and granted Accessibility without a rebuild between
  identity checks.
- Isolated-home install/autostart/update/uninstall/purge/rollback and foreign-file tests.
- Workflow contract tests plus `codesign`, `notarytool`, `stapler`, `spctl`, `lipo`, `actionlint`,
  and fresh-download verification before website promotion.
