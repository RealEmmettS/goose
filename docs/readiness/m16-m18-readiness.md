# M16.1-M18.1 Backend Readiness

Last updated: 2026-07-15

## Status

M16-M18 implementation is in-tree. M16.1-M18.1 readiness is a CI-plus-native-evidence gate, not a
cross-target claim. GitHub-hosted runners provide Linux X11/Wayland evidence for `#m17r` and
`#m18r`; hosted Intel and Apple Silicon runners provide the ordinary macOS bundle gate. Managed
macOS Accessibility first-run onboarding is implemented and its focused automated tests pass. On
2026-07-15, one unchanged Developer ID-signed app on a physical M2 passed first-denied safe wait,
non-nagging denied relaunch, same-process live grant, and same-process live revocation against one
recorded binary digest. `#m16r` is closed with explicit source-equivalent, terminal-tooling, absent-
Ghostty, and one-display waivers below; no byte-exact-final-Mac, live Ghostty, or live multi-monitor
claim is made.

## Implemented Evidence

- ADR 0022's macOS onboarding is in-tree: exact managed-app/receipt eligibility, an owner-only
  per-update prompt marker created before native UI, main-thread consent and Settings bridges, a
  platform-neutral calm safe-edge wait, one-second permission polling, same-process FirstUX
  resume after grant, safe re-entry after revocation, and non-prompting unmanaged launches.
- M16 macOS implementation is in-tree: `honk-platform-macos`, macOS runtime wiring, AppKit
  overlay surfaces with an alpha-last Device-RGB bitmap and stable standard-sRGB window
  destination, Accessibility-gated cursor/window behavior, macOS collect windows,
  `honk300 status`, TUI Status, bundle-aware assets/start, and `script/package_macos_app.sh`.
- M17 Linux X11 implementation is in-tree: X11/XWayland session selection, visible transparent
  overlay, XShape/XFixes input-region shaping, Xinerama/root bounds, pointer sampling, cursor
  warp, terminal-filtered foreign-window drag snapshots, Unix IPC status/reload/stop/poke,
  Linux terminal classification, local-time sampling, in-process GNU audio, a bounded and reaped
  PATH-player fallback for musl, and explicit unsupported/failed capability reporting.
- M18 native Wayland reduced mode is in-tree: layer-shell overlay presentation through
  smithay-client-toolkit, one surface per output, real prepare-read/poll/read/dispatch handling,
  output configure/close/hotplug reconciliation, integer and fractional scale support, empty input
  regions, no keyboard interactivity, and a released-buffer pool capped at three buffers per
  output. Unix IPC status/reload/stop/poke and direct honk/mud/wander control remain available;
  cursor warp, foreign-window control, collect-window behavior, and synthetic input report
  explicitly unsupported.
- `honk-engine` remains platform-free; backend crates own OS, display-server, permission, and
  presentation behavior.
- Capability state flows through `BackendCapability` and the compact status protocol instead of
  compile-time assumptions inside engine tasks.
- Linux collect-window support remains unsupported in M17.1 and is reported honestly.

## CI Proof Path

- `.github/workflows/ci.yml` runs:
  - Windows hosted gate: format, workspace tests, clippy, release build, Windows x64/ARM64
    target checks.
  - macOS hosted gate on `macos-15` and `macos-15-intel`: workspace tests,
    `script/smoke_m16_macos.sh`, universal2 `.app` artifact upload, bundle id
    `dev.emmetts.honk300`, `LSUIElement=true`, `plutil`, `codesign`, and `lipo`.
  - Optional macOS Accessibility gate on `[self-hosted, macOS, ARM64, honk300-a11y]` when
    `HONK300_RUN_A11Y_SMOKE=true`: `script/smoke_m16_macos_accessibility.sh`.
  - Linux hosted gate on `ubuntu-latest` and `ubuntu-24.04-arm`: workspace tests, one exact
    prebuilt `HONK300_BIN` through `script/smoke_m17_m18_linux.sh`, X11 root and Sway `grim`
    compositor captures, GNU target checks, and musl target checks.
- GitHub-hosted runner labels used here match GitHub's current runner table for
  `windows-latest`, `macos-15`, `macos-15-intel`, `ubuntu-latest`, and `ubuntu-24.04-arm`.
- The self-hosted Accessibility job uses cumulative labels so it only runs on a runner with all
  required default/custom labels.

## Repeatable Smoke Scripts

- `script/smoke_m16_macos.sh` builds and validates the universal2 app, launches the LSUIElement
  bundle, checks status, verifies bundle metadata, and exercises honk/mud/reload/stop IPC.
- `script/smoke_m16_macos_accessibility.sh` supports the exact prebuilt app and separate denied,
  live, and pre-granted phases. Its live phase verifies the owner-only marker, calm denied status,
  reload/honk plus `BUSY` prank rejection, a second denied launch without marker rewrite,
  same-process grant, live revocation back to denied wait, unchanged signed-binary/marker
  fingerprints, operator-observed UI state, and opt-in scoped cleanup. The product-equivalent
  signed candidate passed that four-state native run; exact final-SHA repetition is waived below.
- `script/smoke_m17_m18_linux.sh` accepts an exact `HONK300_BIN` and does not rebuild it. It runs
  a visible X11 overlay under Xvfb/openbox/xcompmgr, checks status, rejects any direct visual whose
  byte order, masks, shifts, bpp, or scanline layout is not the expected little-endian ARGB8888
  ZPixmap contract, captures the actual root window, and proves the controlled background remains
  visible outside the overlay. It exercises honk/mud/wander/nab/reload/stop IPC, then starts
  headless Sway with two virtual outputs at 1.5x and 2x scale. `grim` captures the real composed
  Wayland desktop and requires recognizable body, wing, asymmetric warm beak/legs, and background
  while unsupported mischief remains explicit. Sway is intentional because the runtime requires
  wlr layer-shell; Weston does not provide it.

## Readiness Evidence State

- `#m16r` is closed. Hosted macOS bundle/status smoke passed on arm64 and Intel, automated
  onboarding/classifier/topology contracts are green, and the physical M2 four-state run below
  proved the managed native transition path. The exact-final-SHA repeat and physical ordinary/
  terminal manipulation matrix are accepted source-equivalent/tooling waivers under the
  operator's explicit stable-now/forward-patch direction. Ghostty and multiple displays were not
  present; neither is claimed live.
- `#m17r` is closed from its original readiness scope. Linux x64 and ARM hosted X11 visible smoke passed, including
  internal frame proof, root screenshot proof, IPC status/reload/stop/poke, terminal-filter
  fixture coverage, and GNU/musl target checks. The stricter exact-binary visual-layout gate passed
  candidate `29392439475` on x64/ARM64 GNU/musl and repeats in every release candidate.
- `#m18r` is closed from its original readiness scope. Linux x64 and ARM hosted Wayland reduced-mode smoke passed under
  headless sway, including visible frame proof, IPC status/reload/stop/poke, and explicit
  unsupported mischief status. The semantic `grim` compositor assertion passed candidate
  `29392439475` on both hosted architectures and repeats in every release candidate.

## Physical M2 macOS Evidence (2026-07-12)

- A universal app and graphical helper were signed explicitly inside-out with the current G2
  Developer ID Application identity for team `M9D5379H93`; both slices, hardened runtime,
  timestamp, chain, designated requirement, and strict verification passed.
- The exact signed executable first passed the denied Accessibility smoke. After the operator
  enabled that unchanged Honk300 row in System Settings, status reported `accessibility:
  supported`, `cursor: supported`, and `window: supported`.
- The granted smoke passed honk, mud, reload, nab, meme, note, and stop. A singleton-release fix
  found during that run then passed 60 immediate stop/start cycles on a freshly resealed app.
- Real Dark appearance note captures now show appearance-aware, high-contrast text. Real light
  and dark overlay captures show the complete opaque body, outline, wing, beak, legs, and shadow.
- A later standard-sRGB window diagnostic measured 5.55% median CPU, 29.52 MiB maximum RSS,
  negative 9.89 MiB growth, zero leaks, and 20 clean compositor captures. It validates the local
  direction but does not replace the byte-exact signed-candidate capture/profile.
- This Mac has one attached display. Live multi-monitor/hot-plug behavior is therefore explicitly
  hardware-waived; signed-coordinate, topology, per-display-window, and hot-plug reconciliation
  tests remain mandatory and pass. No claim of live multi-monitor validation is made.
- Terminal.app, Codex, and Visual Studio Code identities were observed on this host and match the
  fail-closed application classifier. Ghostty was not installed. The exact final candidate still
  needs the recorded ordinary-window positive and protected-terminal negative observation before
  `#m16r` closes.
- This preliminary run predates ADR 0022's managed first-run flow. It does not prove the native
  prompt appeared once, the second denied launch did not nag, FirstUX resumed from the wait, or a
  live revocation returned the goose to that wait.

## Physical M2 managed-onboarding closure (2026-07-15)

- Candidate `29391420738` produced the signed/notarized/stapled app whose product code is
  `ba483811176ec5144971c0676cfed54d11d5abe7`. The unchanged signed executable had SHA-256
  `55893d07eae1040096cd97e5cddc0c5a7fb5adf18734eeb3360060c129d74c4f` and stayed process
  `42476` through both live transitions. The later exact release
  source `9c5692b32bb256d3008308c83d76ddebd7fb44df` changed Linux smoke/documentation plus stamped
  commit metadata, not Mac product behavior or designated requirement. Exact `9c5692b` app/DMG
  artifacts separately passed universal slices, Developer ID team `M9D5379H93`, hardened runtime,
  secure timestamp, stable designated requirement, notarization, stapling, and Gatekeeper.
- First denied launch displayed the managed consent/Settings handoff once, created the owner-only
  marker, parked at the safe edge, reported denied cursor/window capability, allowed status,
  reload, honk, and stop, and returned `BUSY` for permission-bound controls. Relaunching the same
  denied app preserved marker/binary identity and did not reopen consent or Settings.
- Enabling the visible Honk300 row changed that same running process to supported Accessibility,
  cursor, and window capability in 102 ms and resumed FirstUX. Disabling only that row returned the
  same process to denied capability/safe wait in 127 ms, cancelled permission-bound work, did not
  reopen Settings, and retained the allowed controls. The official smoke transcript exited zero
  under ignored qualification evidence.
- Granted collect spawned and typed a real note reading “i cause problems on purpose and then bill
  them as features.” The note remained readable in Dark appearance. The native endpoint plus the
  fixed 120 Hz beak-offset regression proves task progress; a decisive visual beak-contact frame
  is deferred hardware verification rather than claimed.
- Terminal.app, Codex, and Visual Studio Code identities are covered by the fail-closed native
  classifier and prior development observations. The desktop-control driver could not physically
  move even the positive TextEdit fixture during this closure pass, so the exact ordinary-window
  positive and terminal-negative manipulation matrix is tooling-waived to those automated/prior
  checks. Ghostty was absent and is software-waived. No live manipulation claim is added.
- This Mac had one display. Live multi-monitor/hot-plug remains hardware-waived while signed-
  coordinate, topology, gapped-layout, per-monitor-window, and hot-plug reconciliation tests pass.
  No live multi-display claim is made.
- Scoped cleanup removed the app, aliases, LaunchAgent/service, receipt/state/media/backups,
  process, socket/runtime directories, mounts, and test fixtures. With fresh action-time approval,
  only Honk300's Accessibility row was switched off; no other privacy entry changed.
- Closure decision: exact-final-SHA live repetition and the manipulation matrix are explicit
  verification waivers, not silent passes. The operator directed stable publication now and
  later Alienware/native checks as forward-patch input. These waivers do not authorize weakening
  any Mac pixel, Accessibility, signing, lifecycle, terminal-protection, or cleanup contract.

## Local Verification Commands

```powershell
cargo fmt --all -- --check
cargo test --workspace
cargo clippy --all-targets --workspace -- -D warnings
cargo build --release
$targets = @(
  'x86_64-pc-windows-msvc',
  'aarch64-pc-windows-msvc',
  'x86_64-apple-darwin',
  'aarch64-apple-darwin',
  'x86_64-unknown-linux-gnu',
  'aarch64-unknown-linux-gnu',
  'x86_64-unknown-linux-musl',
  'aarch64-unknown-linux-musl'
)
foreach ($target in $targets) {
  cargo check --workspace --target $target
  if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
}
```

## CI Evidence Log

- 2026-07-14 - Mac stopping-tree gate passed all 423 workspace tests, 22 native Mac platform
  tests, strict clippy, both Apple release builds, 71 packaging/workflow contracts, actionlint,
  shell/PowerShell validation, cargo-dist planning, cargo-audit, and cross-target checks. The host
  is clean/uninstalled. No candidate was dispatched, so ADR 0022's four-state exact-candidate,
  terminal, capture, profile, and lifecycle evidence remains open and `#m16r` stays Active.
- 2026-07-13 - strengthened the Linux x64/ARM64 smoke contracts so one exact binary is retained,
  X11 byte layout fails closed, and `grim` checks the real two-output fractional-scale Sway
  composition for body/wing/asymmetric warm pixels. Focused local contracts pass; first hosted
  execution of this stronger v1.0.0 gate remains pending.
- 2026-07-13 - focused automated onboarding gates passed: five engine permission-wait tests,
  thirteen managed-install/marker/transition tests, all 17 macOS platform tests (including bundle
  metadata, prompt-thread, Settings failure, and hot-plug interactivity contracts), and five
  smoke-script contract tests. No
  exact signed-app denied/non-nag/grant/revoke smoke was run by this documentation pass; that
  four-state native evidence remains pending.
- 2026-07-12 - physical M2 preliminary Accessibility evidence passed on one unchanged current G2
  Developer ID-signed app: denied smoke first, operator grant, supported accessibility/cursor/
  window status, then honk/mud/reload/nab/meme/note/stop and 60 stop/start cycles. This is native
  pre-candidate evidence; the immutable release candidate must repeat it before closure.
- 2026-07-02 - GitHub Actions run
  <https://github.com/RealEmmettS/goose/actions/runs/28569332035> completed successfully for
  hosted Windows/macOS/Linux readiness.
  - Windows host gate: passed fmt, workspace tests, clippy, release build, and Windows x64/ARM64
    target checks.
  - macOS bundle smoke (`macos-15` arm64): passed workspace tests, universal2 app packaging,
    `plutil`, `codesign`, `lipo -verify_arch x86_64 arm64`, bundle launch/status/IPC smoke, and
    artifact upload.
  - macOS bundle smoke (`macos-15-intel`): passed the same hosted bundle/status gate and artifact
    upload on Intel-hosted macOS.
  - macOS app artifacts: `honk300-macos-macos-15` and `honk300-macos-macos-15-intel`.
  - Linux visible smoke (`ubuntu-latest`): passed X11 visible overlay smoke, Wayland reduced-mode
    smoke, workspace tests, and Linux x64 GNU/musl target checks.
  - Linux visible smoke (`ubuntu-24.04-arm`): passed X11 visible overlay smoke, Wayland
    reduced-mode smoke, workspace tests, and Linux ARM GNU/musl target checks.
  - macOS Accessibility smoke:
    <https://github.com/RealEmmettS/goose/actions/runs/28569332035/job/84703318760> was skipped,
    so Accessibility-granted cursor/window/collect evidence is still missing and `#m16r` stays
    open.
