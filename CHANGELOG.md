# Changelog

All notable changes to this project are documented here. Format based on
[Keep a Changelog](https://keepachangelog.com/); release versions follow
[Semantic Versioning](https://semver.org/).

> **Project stage: first stable release.** Milestones M0-M19 are implemented in-tree. The
> managed macOS Accessibility first run is implemented and one unchanged signed executable has
> passed denied, non-nagging relaunch, live-grant, and live-revocation on the physical M2; exact-
> final-SHA and unavailable-hardware/tooling limitations are retained as explicit forward-
> verification waivers. The goose now renders, walks, leaves mud, plays sounds, reacts to the
> cursor, can
> perform bounded cursor-nab mischief, can perch on user-dragged windows, and can collect
> Notepad/meme windows on Windows. It now enters/leaves through real exposed edges, occasionally
> wraps only while fully hidden, and can react when a user closes one of its collected windows.
> It can be controlled through a single-instance local IPC
> channel. It now has the three-name goose-speak CLI plus durable TOML configuration and the
> ratatui config TUI, dynamic moods, the local on-hour double honk, quiet-hours/DND/fullscreen
> manners, built-in Autumn leaves, Windows multi-monitor chase, and live appearance/recolor
> controls, plus macOS runtime/status/app-bundle staging and a menu-bar shortcut to the existing
> TUI/graceful Quit, Windows notification-area controls, Linux StatusNotifier controls and X11
> visible overlay support,
> native Wayland reduced-mode rendering, CI smoke gates, and M19 Windows/Linux lifecycle plus
> release packaging with artifact evidence. A plain-English companion lives in
> [HUMAN_CHANGELOG.md](./HUMAN_CHANGELOG.md) and must stay in lockstep.

## [Unreleased]

## [1.1.0] - 2026-07-17

### Added
- **Windows and Linux tray control parity (ADR 0030)** - adds one accessible **Honk300 controls**
  surface while the goose is running. Windows uses a fixed-GUID native notification-area icon;
  compatible Linux desktops use a pure-Rust StatusNotifierItem with an embedded contrasting goose
  icon. Both expose only **Configure Honk300…** and **Quit Honk300**.
- **Native lifecycle and recovery qualification** - Windows restores the same icon after Explorer's
  `TaskbarCreated` broadcast and Linux recovers after its StatusNotifier watcher returns. Hosted
  Linux tests exercise watcher/host registration, accessible properties, menu events, recovery,
  and explicit no-host or no-session-bus behavior.

### Changed
- **One control-surface command path** - macOS, Windows, and Linux now emit the same finite
  `Configure`/`Quit` command type. Configure launches the exact running executable's existing
  terminal TUI without shell interpolation; Quit reaches only the engine's animated graceful
  walk-off and never terminates directly from a native callback.
- **Linux package integration** - Debian packages now own the shared icon in the hicolor theme and
  reference it from the existing desktop entry. Portable archives keep the icon embedded and add
  no GTK, AppIndicator, or compositor dependency.

### Fixed
- **Windows TUI launch isolation** - the tray launcher creates a normal new console without
  inheriting the goose process's redirected standard handles, preventing a blank configuration
  screen while leaving the running goose and singleton intact.
- **Explicit tray-unavailable boundaries** - a missing Windows shell owner or Linux session-bus/
  StatusNotifier host is non-fatal and visible while CLI, TUI, IPC, overlays, and supported
  mischief remain independent. Windows ARM64 hosted qualification may use this waiver only when
  an independent stock-icon `Shell_NotifyIconW` registration also fails.
- **Windows Server tray-recovery qualification** - keeps the `TaskbarCreated` recovery check
  strict while allowing the shell's rectangle API a bounded ten seconds to expose a successfully
  restored icon. Failure evidence now includes poll count and runtime restoration diagnostics.

## [1.0.3] - 2026-07-17

### Changed
- **Corporate MSI retry semantics (ADR 0029)** - schedules old-product removal after the new
  per-user installation commits. An injected mid-upgrade failure now leaves the old product and
  bytes untouched; retry and uninstall complete without the reproduced orphaned component client.
  The machine-wide Global MSI keeps its existing transactional schedule.
- **Conservative integrated-terminal protection** - Windows now protects Visual Studio Code,
  Codex, and the observed ChatGPT-titled Codex desktop surface from window mischief. Linux adds
  equivalent `code`/`codex` application-token coverage, matching the existing macOS policy while
  leaving ordinary applications eligible.

### Fixed
- **Native Windows update discovery and download** - keeps URI/output values outside PowerShell
  command source, uses non-interactive stop-on-error requests, decodes Windows PowerShell 5.1 byte
  responses as UTF-8, and preserves exact-tag size/SHA verification. Both installed v0.3.1 and
  exact portable v1.0.2 reproduced the old parser failure; rebuilt source passes live discovery
  and a quoted-path exact download. Because the immutable older executable cannot repair itself,
  Windows users on v1.0.2 or earlier must rerun the supported installer once to reach v1.0.3;
  ordinary installer upgrade preserves user content, and subsequent update checks use the fix.
- **Coexisting Global/Corporate lifecycle identity** - the marker beside the invoked executable
  now wins before registry fallback, and Corporate uninstall discovery searches current-user then
  machine registration while retaining exact product/publisher/ProductCode/root validation.
- **Deferred Windows parent completion** - a helper now treats an already-exited parent as a
  successful wait without suppressing genuine lookup/wait failures.
- **PowerShell-bootstrap receipt freshness** - after an exact-path/version-verified direct Global
  MSI update, only a complete regular non-reparse receipt proving the expected schema, channel,
  layout, and root is atomically refreshed to the new release identity. Missing, malformed,
  foreign, mismatched, and reparse receipts remain untouched.

## [1.0.2] - 2026-07-17

### Added
- **Shared goose control-surface icon (ADR 0028)** - added a Quiver-generated, transparent,
  two-path monochrome SVG as the canonical future macOS/Windows/Linux menu/tray source, with
  prompt/generation provenance, deterministic export guidance, an accessible-name contract, and
  an AppKit-safe 36×36 RGBA representation. v1.0.2 implements only the macOS item; future Windows
  notification-area and Linux desktop surfaces must reuse the same Configure/TUI and engine-owned
  graceful-Quit behavior rather than create another settings model or immediate exit.

### Changed
- **Accessible image-only macOS status item** - replaced the visible `Honk` title in production
  bundles with the 18-point goose template image, square item sizing, proportional scaling, and an
  independent `Honk300 controls` accessibility label/tooltip. The Rust owner retains both image
  and AppKit's weak action target. Missing or undecodable resources keep the variable-width
  `Honk` development fallback without preventing Configure or Quit from launching.
- **v1.0.2 atomic publication closure** - replacement candidate `29565557915`, exact-SHA main CI
  `29566294408`, atomic release `29566759574`, and post-release smoke `29567257622` passed at
  `964305869e9ec28768c789465db1b6317dfa3f6f`. The immutable latest release contains 22 payloads
  and 47 total assets. Fresh public Mac app/DMG hashes, G2 Developer ID identity, hardened runtime,
  notarization, staples, Gatekeeper, resources, universal slices, graphical lifecycle, the real
  v1.0.1→v1.0.2 three-alias managed update, and repeat no-ops passed. The deployed progressive-
  disclosure site reports that exact release and recommended immutable DMG; all published Windows,
  Linux, Debian, and Mac installer smokes passed.
- **v1.0.1 publication closure carried forward** - retained the exact candidate, same-SHA main CI,
  atomic publication, post-release native smoke, independent Developer ID/notarization/stapling/
  Gatekeeper, v0.3.2 upgrade, and DMG-first site evidence without mutating its immutable tag or
  assets. Physical-Mac follow-up additionally proved the published menu/TUI, same-process
  Accessibility grant, readable dark note, 5.415-second animated Quit/restart, and two passing
  process-specific 60-second CPU/RSS profiles; one-display, absent-Ghostty, and hardened-runtime
  leak-attach limitations remain explicit.

### Fixed
- **Old-macOS icon decoder dependency** - the runtime now loads a sealed PNG representation rather
  than depending on raw SVG decoding across the macOS 11+ support range. Native AppKit tests
  require a nonzero representation, 18-point size, and template state.
- **Bounded Windows named-pipe startup reads** - a successful zero-byte poll from a connected
  `PIPE_NOWAIT` command pipe is now treated as a transient scheduler state instead of an empty
  command frame. The existing deadline still fails closed when a peer never supplies data, and
  Windows-target regressions pin both the delayed-frame and bounded-timeout paths. This repairs
  the only failure in the first v1.0.2 candidate without changing the shared command protocol.
- **Release-shape icon enforcement** - macOS packaging copies the canonical SVG and runtime PNG
  before signing and fails closed unless both survive the staged app, extracted final app ZIP,
  initial DMG mount, and final notarized/stapled DMG remount. The updater regression now explicitly
  recognizes v1.0.1→v1.0.2 as a forward in-place update.

## [1.0.1] - 2026-07-15

### Added
- **macOS menu-bar Configure and graceful Quit (2026-07-14, ADR 0024)** - the running
  `LSUIElement` app owns one main-thread AppKit status item labeled `Honk`. Its accessible
  Configure action opens a signed-bundle `Configure Honk300.command` resource which executes the
  same app binary's existing ratatui `config` entry point; Quit sets the shared engine graceful-
  stop intent so simulation/presentation continue through the fully offscreen walk and final
  transparent acknowledgement. The bridge explicitly retains AppKit's weak menu target for the
  item lifetime, removes the item on runtime teardown, and adds no native settings schema/window,
  running Dock control surface, AppleScript API, global key, or Windows/Linux tray. Packaging
  contracts require the executable launcher before signing and in the final app ZIP and both DMG
  validation passes. A local packaged universal app exposed all items through macOS accessibility
  inspection, opened/restored the full TUI in Terminal, and completed animated Quit in four
  seconds; exact-final-SHA interaction repetition is tracked as post-release forward verification.
- **Managed macOS Accessibility first run (2026-07-13, ADR 0022)** - only the exact receipted app
  at `~/Applications/Honk300.app` can open automatic permission UI. Before calling the native
  consent request and opening Accessibility settings, the runtime atomically creates an
  owner-only `accessibility-prompt-v1/<version>` marker; mismatched metadata/receipts, symlinks,
  unsafe state, development binaries, and direct mounted-DMG launches fail closed without UI. A
  denied managed app enters a platform-neutral safe-edge wait which blocks automatic and direct
  pranks except honk while preserving status/reload/stop. One-second polling restores capability
  state and starts FirstUX after a live grant, or abandons permission-bound work and returns to
  the non-nagging wait after revocation. Focused engine, eligibility/marker, native bridge, and
  smoke-contract tests pass. One unchanged signed executable completed denied, non-nag relaunch,
  same-process grant, and same-process revoke on the physical M2; exact-final-SHA repetition is an
  explicit source-equivalent forward-verification waiver rather than a stronger claimed run.
- **Developer ID macOS distribution (v1.0.1, ADR 0020)** - the universal app now carries exact
  version/tag/commit metadata, and the DMG contains `Honk300.app`, a separately signed native
  `Install Honk300.app`, and concise per-user instructions without an `/Applications` symlink.
  The helper verifies the target bundle id, signature, and matching nonempty Developer ID team,
  requires the fixed `M9D5379H93` Developer ID Application certificate identity on both apps,
  delegates to the shared no-sudo install transaction, reports native failures, and opens the
  installed app on success.
- **Native Debian packages (2026-07-14, ADR 0023)** - every release now assembles stable
  `honk300-amd64.deb` and `honk300-arm64.deb` assets from the byte-exact, already-qualified GNU
  archive executables. Each package owns `/usr/lib/honk300/honk300`, stable `/usr/bin` aliases,
  its source marker, desktop entry, license, and exact release metadata while keeping mutable
  media in user XDG storage. Debian provenance adds target/kind/architecture-isolated CLI update,
  real `dpkg` removal, non-purge media preservation, purge backup, ownership proof, and native
  amd64/arm64 post-release compositor/lifecycle smokes.
- **Fail-closed signing and notarization pipeline** - candidate and release macOS jobs now import
  a password-protected Developer ID P12 into an ephemeral keychain, sign nested code inside-out
  with hardened runtime and secure timestamps, notarize/staple/validate the app and DMG, retain
  notarization JSON as internal evidence, and delete temporary credentials. All six certificate
  and App Store Connect secrets are mandatory; release mode has no ad-hoc fallback. The stapled
  final DMG is remounted and Gatekeeper-assessed for both contained apps, proving the user launch
  path rather than only the disk-image container; both the main app and graphical helper sign and
  verify their executable before their outer bundle rather than relying on recursive signing.
- **Native macOS pixel and screenshot contracts** - asymmetric RGBA round trips through AppKit
  and CoreGraphics pin channel/alpha semantics, while semantic captures require a visible body,
  outline, wing, beak, legs, and shadow on light and dark backgrounds.
- **Cross-platform presenter byte-order contracts** - asymmetric color/alpha tests also pin the
  intentional premultiplied RGBA-to-BGRA conversion used by Windows layered windows and both
  Linux X11 and Wayland presenters. This extends the macOS regression audit to every native
  pixel bridge without changing the platform-neutral renderer output.
- **Linux compositor-visible release evidence** - the native smoke can hold an exact
  `HONK300_BIN` unchanged instead of silently rebuilding it. Headless Sway now has two
  fractional-scale outputs and a deterministic background, and `grim` captures the actual
  composed desktop for semantic body/wing/asymmetric-orange assertions rather than checking only
  the renderer's internal PNG. X11 also fails closed unless the selected direct visual is the
  exact little-endian, 32-bpp ARGB8888 ZPixmap layout required by its hard-coded BGRA upload.
- **Windows compositor-visible release evidence** - native CI and the reusable release-candidate
  workflow now run exact x64 and native ARM64 executables through start/status/single-instance/
  reload/stop and immediate-restart lifecycle checks; the published-MSI smoke repeats the x64
  path. The native ARM job consumes the exact executable produced for ARM installer packaging,
  rather than accepting an independent rebuild as equivalent. Each smoke freezes a real
  `UpdateLayeredWindow` surface and captures that unchanged articulated pose over controlled dark
  and light desktops. A dependency-free analyzer assigns every pixel to one nearest palette color
  and fails closed unless the captures independently prove transparent margins, reconstructed
  semantic edge colors, body/shade/outline/wing, the asymmetric orange channel order, and one
  complete renderer view. Side view additionally requires spatially separated beak/two-tone legs
  plus shadow; top-down requires its compact beak, complete wing/body geometry, low shade share,
  and intentional lack of legs/shadow. Screenshots, analysis, logs, and the executable hash are
  retained even on failure.
- **Cross-platform exposed-edge traversal and animated lifecycle** - `DesktopLayout` now derives
  real exposed edges by subtracting touching monitor seams. Four wander entries plus one
  `EdgeWrapTask` keep hidden Pac-Man-style wrapping at 20% of baseline deck draws; relocation is
  legal only after the entire rendered pose is outside every monitor, and deliberate puddle/prank
  excursions return through their own edge without wrapping. Initial startup is staged fully
  offscreen. Stop/exit/quit uses an adaptive Run-to-Charge-bounded `GracefulExitTask`, continues
  ticking/presenting until the full pose is hidden, and gives singleton release a finite 30-second
  client wait for immediate-restart safety on every backend.
- **User-close annoyed reaction** - native macOS and Windows collect snapshots distinguish a user
  closing a spawned note/meme from Honk300 cleanup. An independent deterministic stream selects a
  visible annoyed reaction with 30% probability and may then chain the existing bounded nab only
  when mouse-steal configuration, manners, pointer state, permission, and capability allow it.
  Program cleanup never rolls; Linux remains explicitly collect-unsupported and has no trigger.

### Changed
- **First public stable major release (2026-07-15, ADRs 0025 and 0027)** - the prospective
  unpublished v0.3.3 milestone ships as v1.0.1. The immutable v1.0.0 tag failed closed before a
  draft or public asset existed and remains unchanged under the atomic-release fix-forward rule.
  Package, app-bundle, receipt, manifest, installer, updater, readiness, and website identities
  advance together; v0.3.2-to-v1.0.1 is an explicitly tested normal upgrade. Additional Alienware
  hands-on verification happens after publication and any findings ship in later forward patches
  without rewriting the immutable v1.0.1 tag or assets.
- **Complete rolling-latest release channel (2026-07-14, ADR 0023)** - every general tag now
  requires the complete cross-platform producer matrix, including a fresh GitHub-macOS-built,
  Developer ID-signed/notarized/stapled app and DMG plus both Debian packages regardless of the
  operator's trigger host. Stable unversioned `latest/download` installer names advance only
  after atomic publication; existing tags and their assets remain immutable. All three CLI names
  read the latest manifest only for discovery, then download the platform/provenance-matched
  payload from its exact tag and verify kind, target, size, and SHA-256 before mutation. Managed
  Mac apps update through the exact-tag universal app ZIP selected by the pinned bootstrap, not
  by replacing from the DMG.
- **Quicker shared walk recovery** - the platform-neutral planted-foot trigger now releases at
  four pixels and normal/moderate recovery keeps its weighted 70%-of-beat cadence. A speed-aware
  body-travel cap shortens only Run/Charge recovery: tests bound Walk lag at 16 pixels and
  Run/Charge at 26 pixels while guarding visible airtime and plant cadence against twitchy
  overcorrection. The shared fix applies on Windows, macOS, X11, and Wayland; three gait-dependent
  renderer goldens were deliberately refreshed before the tier cap, which requires no additional
  golden changes.
- **Lower-allocation renderer and macOS runtime** - renderer scratch surfaces grow in bounded
  increments, stippled shadows are cached, and opaque 1x frames can paint directly. macOS reuses
  bitmap/image storage, avoids swizzle buffers and redundant window/display work, drains native
  objects in autorelease pools, caches desktop geometry, retains 120 Hz simulation, and caps
  presentation at 60 Hz. The AppKit view now draws only the active RGBA rectangle and both the
  tiny-skia canvas and native bitmap shrink after an unusually large note, meme, or distant dirty
  region instead of color-converting that stale transparent capacity on every later frame. Its
  alpha-last bitmap remains Device RGB while the overlay window uses a stable standard-sRGB
  destination, avoiding a redundant per-frame Device-RGB-to-Display-P3 ICC/vImage conversion and
  leaving final per-display-profile composition to WindowServer. The active diagnostic measured
  5.55% median CPU, 29.52 MiB maximum RSS, negative 9.89 MiB growth, zero leaks, and 20 clean
  compositor captures. Exact-final-SHA repetition is retained as post-release hardware
  verification under the accepted source-equivalent waiver.
- **Mounted-bundle lifecycle transaction** - `honk300 install` can copy its enclosing mounted
  source app into `~/Applications/Honk300.app` after validating bundle id, stamped release
  identity, strict signature, and both architectures. It preserves the shared aliases/media/
  autostart/receipt/update/uninstall contract and performs an atomic bundle swap with rollback;
  a candidate-only fault hook proves the previous bundle and receipt return after activation.
  Late failures also restore/remove aliases, the LaunchAgent, receipt, and only migration-created
  media while preserving existing user content.
- **Exact-artifact native smoke** - both macOS smoke scripts accept `HONK300_APP` plus
  `HONK300_SKIP_BUILD=1`, reject missing bundles, and create their config only through `setup`,
  allowing denied and granted Accessibility evidence on one unchanged signed identity.

### Fixed
- **Pose-complete Windows release qualification (2026-07-15, ADR 0027)** - release run
  `29398343807` at exact candidate/default-branch SHA `9c5692b` failed atomically before draft
  creation because ten valid top-down entrance captures were judged by a side-only legs/shadow
  oracle. The paired-DWM and hosted-ARM64 raw-presenter analyzers now share strict side and
  top-down profiles plus reconstructed mid-alpha color proof. Red/blue swaps, straight alpha,
  double premultiplication, opaque/black surfaces, missing warm articulation, damaged side
  fallthrough, and partial/cropped top-down frames remain fatal. Retained replay accepts only
  complete attempts 6–12 and rejects partial attempts 3–5. No renderer, engine, presenter, gait,
  or user-visible behavior changed; v1.0.1 rebuilds and requalifies the entire matrix.
- **Hermetic Wayland compositor evidence (2026-07-15)** - exact candidate `29389882143` passed
  every release producer, native compositor/package job, and final assembly at commit `c44b89d`.
  Ordinary main CI then exposed a non-hermetic difference: installing Ubuntu's recommended Sway
  packages loaded `/etc/sway/config`, its wallpaper, and its bar on `HEADLESS-1`; Honk300's
  transparent layer correctly revealed that distro background while `HEADLESS-2` showed the
  requested solid. A first private config still contained its own wildcard background, and the
  exact setters still used `solid_color`. Ubuntu 24.04's `swaybg` represents that color as a
  one-pixel protocol buffer; linear filtering on the 1.5-scale pixman output sampled its outside
  edge and produced a deterministic gradient before Honk300 even launched. The smoke now starts
  with no background rule, creates constant opaque PNG tiles, addresses both discovered outputs
  only by exact name, and proves paired goose-free baselines before launching Honk300. Fractional
  filtering stays enabled. No product background, renderer behavior, or semantic threshold changed.
- **Candidate-native evidence calibration (2026-07-14, ADR 0026)** - candidate `29387569722`
  passed the trusted Mac producer, Windows x64 paired-DWM compositor/lifecycle proof, native
  Windows ARM64 PE/MSI lifecycle, and X11 plus dual-output Wayland on three of four Linux
  variants. Its remaining Linux x64 GNU capture was a valid top-down goose with 710 body, 1,615
  wing, and 13 warm articulation pixels, so the Linux warm floor is now 10 while all body, wing,
  background-transition, transparency, and opaque-surface checks remain unchanged. GitHub's
  hosted ARM64 runner separately returned one byte-identical wallpaper for two visible ordinary-
  window colors. Only that exact GitHub-hosted signature may now use the real process's cropped
  premultiplied-BGRA DIB, atomically recorded after successful `UpdateLayeredWindow` and bound to
  the frozen visible HWND/rectangle. Raw checks reject straight alpha, double premultiplication,
  swapped channels, mostly opaque-black surfaces, missing articulation, and stale metadata.
  Windows x64 plus local/self-hosted ARM64 still require paired live DWM captures; the hosted ARM64
  result is never described as desktop-composition proof. Candidate `29389046641` then passed the
  complete Mac producer, every Linux compositor job, both Debian package jobs, and Windows x64;
  both hosted ARM64 jobs also produced multiple surfaces that passed every raw semantic check but
  rejected them because the moving overlay advanced one or two pixels between the atomic record
  and controller suspension. The binding now keeps exact HWND identity, polls at five
  milliseconds, and permits only a three-physical-pixel origin/dimension delta covering the
  observed single-interval drift; larger movement retries. No pixel, alpha, articulation,
  runner-identity, or paired-DWM requirement changed.
- **Portable native-smoke runtime paths and Windows diagnostics (2026-07-14)** - the first
  v1.0.0 candidate proved all four repaired X11 compositor paths and matching Windows controller/
  background geometry, then failed closed before product capture. Sway's AF_UNIX sockets now use
  a short owner-only temporary runtime directory instead of an evidence path that can exceed
  Linux's 108-byte limit, and cleanup removes it while retaining logs/screenshots. Windows
  geometry validation now splits exact CRLF/LF diagnostic lines instead of applying a Unix-style
  end anchor to a CRLF document. No product background, renderer behavior, or semantic capture
  threshold changed.
- **Final-source native candidate diagnostics (2026-07-14)** - after native `BitBlt` repaired the
  first Windows capture failure, candidate `29384134561` proved the notarized Mac producer and
  Windows x64 path but failed closed because `xcompmgr` held its cached gray root tile on every
  Linux target and the ARM64 Windows controller captured neither its ordinary TopMost background
  nor the overlay before racing the shared color file. X11 qualification now uses a persistent,
  test-only opaque client behind the goose, changes its controlled color through an atomic
  command plus acknowledgement, and forces real compositor damage without adding any background
  to the product. Both Windows smoke processes enter per-monitor-v2 DPI awareness before WinForms
  or HWND creation, exchange tokenized atomic color requests, prove matching physical virtual-
  screen geometry and a real dark/light capture before launch, and retain per-overlay DPI/rect
  evidence. Cross-platform `status` output also treats only a downstream `BrokenPipe` as normal
  while preserving every other write failure. Semantic goose/color/alpha thresholds remain
  unchanged; the source differs from the proven candidate and must rerun exactly before release.
- **Collect pickup now aims locomotion at the beak interaction point** - live macOS qualification
  exposed a side-view deadlock where the body stopped at the prop center but completion measured
  distance from the neck-height beak. Locomotion now recomputes the body target from current beak
  geometry each tick, keeping the beak-distance arrival gate authoritative. A realistic 120 Hz
  world regression reaches passthrough/grab and note typing within 15 seconds without teleporting
  the beak, and the complete integrated local gate passes. A product-equivalent signed app spawned
  and typed a readable native note; exact visual beak-contact capture remains post-release
  verification and is not overstated as completed native evidence.
- **Lingering collect windows cannot starve a newer request** - the macOS and Windows native
  controllers now prefer the most recently spawned typed request over older notes or memes that
  remain open. Dead-window events are still drained first, matched by request id and kind, and
  removed exactly once, so delayed user-close reactions remain truthful without making a newer
  collect task time out behind arbitrary map iteration order.
- **Current macOS hands-on release runbook** - replaced the historical ad-hoc, unnotarized,
  terminal-first checklist with the exact v1.0.1 Developer ID/notarized/stapled DMG-first flow.
  The runbook now preserves one candidate identity across four Accessibility states, native
  terminal protection, renderer and performance evidence, lifecycle rollback, the one-display
  waiver, immutable publication, and fresh-download verification. The README now links the exact
  release DMG and distinguishes the stapled app inside the ZIP from the ZIP container itself.
- **Readable macOS notes in every appearance** - note text now uses AppKit's semantic label color
  instead of absolute black, so it follows light, dark, and increased-contrast appearances. A
  native regression pins the semantic-color contract. The equivalent platform paths were audited:
  Windows delegates note colors to system Notepad, while Linux explicitly reports collect windows
  unsupported rather than drawing a separate note surface.
- **macOS transparent-white/purple goose** - the AppKit bridge now declares tiny-skia's direct
  premultiplied RGBA bytes as alpha-last instead of interpreting them through the former
  BGRA/alpha-first contract. Native captures show the articulated goose rather than a translucent
  color blob; the seven shared engine goldens remain stable apart from the intentional gait poses.
- **macOS overlay capture and post-transient CPU** - replaced the custom child Core Animation
  layer with a reusable `NSImageView` in the ordinary AppKit backing store. WindowServer capture
  and screen sharing now receive the same transparent pixels as the physical display instead of
  omitting the goose or turning unused surface capacity into black rectangles. Capacity-shrink
  regressions prevent a large transient frame from making later normal walking redraw and color-
  convert an oversized image. Product-equivalent active-motion capture and profiling clear the
  budget; exact-final-SHA repetition remains explicit post-release verification.
- **macOS lint debt** - removed the obsolete world-bounds path and corrected the three remaining
  target-specific warning failures so workspace clippy can run with `-D warnings` on macOS.
- **Stop/start singleton race** - a successful `stop`, including the `exit` and `quit` aliases,
  now keeps the runtime alive while the goose walks completely beyond its nearest exposed edge,
  then waits for the shared singleton to be released before returning. Exit speed adapts within
  the existing Run/Charge envelope. An immediate restart can no longer lose the race between the
  old runtime acknowledging shutdown and actually exiting on Windows, macOS, or Linux; the wait
  is bounded and reports a stalled shutdown explicitly.
- **Install/update/uninstall lifecycle race** - managed mutations now stop the active runtime and
  retain its real cross-platform singleton as a `LifecycleLease` until every owned file and
  integration change is finished. The exact-tag Unix installer holds that lease through a staged
  binary and parent-owned FIFO; Windows bootstraps and updaters use the manifest-hashed portable
  executable as a redirected-stdin lease holder. Windows uninstall first performs ownership-only
  preflight, then hands off to a private temporary copy which owns the singleton before the
  installed CLI exits, preventing both running-EXE deletion failures and partial mutation before
  a helper is ready. Verified Windows payloads remain pinned by the same read-only stream through
  execution, generated updates explicitly release/reacquire rather than trusting ambient state,
  Restart Manager checks exact machine-wide paths across sessions, and reboot-deferred MSI results
  fail closed. Unix signals carry explicit rollback statuses, and every pre-READY deferred helper
  is killed and reaped on error. Missing, rejected, timed-out, or interrupted ownership fails or
  rolls back before an unowned partial state can be accepted.
- **Release-board runtime debt** - re-audited the shared `RuntimeCore` and the Windows per-monitor
  dirty-region presenter against their deferred cards. Focused sequencing, 4K bounded-damage, and
  Windows x64/ARM64 cross-target checks confirm the old loop-duplication and fullscreen-redraw
  follow-ups are resolved by the current architecture.
- **Native Wayland capability decision (ADR 0021)** - completed the upstream protocol/compositor
  audit and published a portable/wlroots/KDE/GNOME/portal/XWayland/privileged-helper matrix. The
  portable layer-shell mode remains honestly reduced; future near-parity work is split into
  explicit, permissioned compositor adapters rather than one false universal-support claim.

### Security
- **Local release credentials stay outside Git** - `/.private-release/` is ignored as the
  owner-only location for one-time local signing/notarization material. Credential contents are
  supplied to GitHub only through encrypted Actions secrets and never tracked in the repository.
- **Bound graphical installation** - mounted-source installation preflights foreign aliases,
  receipts, LaunchAgents, symlinks, bundle identity, release metadata, architecture, and code
  signature before mutation. DMG receipts bind updates to the exact tag and full commit, and the
  helper refuses adjacent apps signed by another team. Both graphical and terminal transactions
  also refuse to replace an existing app that lacks a matching Honk300 ownership receipt.
  Receipt checks use no-follow metadata so even a dangling foreign symlink is preserved rather
  than misclassified as an absent path.
- **Embedded-terminal protection on macOS** - the foreign-window classifier now excludes Codex
  and Visual Studio Code by bundle identity in addition to Terminal, Ghostty, iTerm, Warp, and
  other terminal apps. Because AppKit permission-safe filtering happens at application scope,
  conservatively excluding the full editor prevents ride/collect behavior from ever targeting an
  integrated terminal panel.

## [0.3.2] - 2026-07-11

### Added
- **Published installer lifecycle smoke** - a manually dispatched, read-only GitHub Actions gate
  now exercises the immutable stable release on x64/ARM64 Linux, Intel/Apple Silicon macOS, and
  Windows. It installs twice, verifies managed integrations, forces and proves transactional
  rollback, and covers Global MSI upgrade, repair, downgrade refusal, ARM64 extraction, and
  uninstall behavior.

### Fixed
- **Real MSI license plus the Great Honk Accord** - the x64/ARM64 Global and Corporate MSI
  agreement page now displays the authoritative PolyForm Noncommercial License instead of WiX's
  lorem-ipsum placeholder, followed by one shared, long-form, explicitly non-binding ceremonial
  agreement between the installer operator and the Goose. Packaging tests prove the legal wording
  stays aligned with `LICENSE`, the appendix stays within its copy contract, and every MSI variant
  consumes the same simple RTF while the Inno installers remain unchanged.

## [0.3.1] - 2026-07-10

> The immutable `v0.3.0` tag failed in artifact production before any draft or public release was
> created. In accordance with the atomic-release contract, the corrected build ships as the next
> patch version rather than moving or rebuilding that tag.

### Added
- **Stable release and install receipts** - `honk300.release.v1` and `honk300.install.v1`
  record the exact version, tag, commit, target, artifact hash/size, managed layout, aliases, and
  autostart ownership. Windows recommends the x64/ARM64 Global MSI; macOS and Linux use one
  version-stamped, exact-tag, hash-verifying shell bootstrap. macOS installs a real universal
  `~/Applications/Honk300.app`; Linux installs below the user's XDG data directory.
- **Config schema v2 and explicit recovery states** - configuration loads now distinguish
  missing, valid, malformed, and unsupported-newer files. `setup --reset` is the only reset path
  and creates a timestamped backup first; v1 mute controls migrate without losing intent.
- **Distribution notices** - the official PolyForm Noncommercial license, third-party media
  notice, and vendored-code provenance now travel with release payloads.

### Changed
- **Concept-C side renderer** - the goose now uses a single integrated white silhouette with
  distinct back-neck/throat curves, a broad neck base, restrained long-neck sweep, and oval head.
  Both facings, tucked/raised poses, the existing rig, palette, wings, beak, and legs remain.
- **Long-running and multi-display runtime core** - simulation clocks/deadlines use `f64`, visual
  phases wrap independently, display geometry uses real monitor regions/adjacency instead of a
  bounding rectangle, and shared `RuntimeCore` pins command/reload/tick/damage ordering across
  native event pumps. Renderer/platform scratch storage is reused and frame damage is bounded to
  the current and previous visual bounds.
- **Simpler distribution defaults** - Windows is machine-wide MSI-first; macOS/Linux are
  terminal-first and no-sudo. The DMG remains an unadvertised v0.2.1 updater compatibility asset,
  and the ad-hoc-signed, unnotarized macOS limitations are documented honestly.

### Fixed
- **Non-destructive config and responsive control UI** - malformed/future config is never
  overwritten, atomic saves preserve valid symlink targets and stable floats, reload rejects
  restart-only backend changes, stopped runtimes report capabilities as unprobed, and TUI IPC no
  longer blocks drawing/input. Start saves dirty settings first and surfaces actual readiness
  failures while small/error-heavy terminal layouts remain reachable.
- **Behavior and render continuity** - crossfades use complementary alpha, puddle returns stay
  vertically continuous, manners cancel delayed pranks, note/meme toggles cancel independently,
  on-hour/Hyper honks deduplicate, capability decks refresh, and missing renderer goldens fail
  rather than being created silently.
- **Cross-platform backend reliability** - Windows monitor removal no longer quits the process,
  topology/DPI changes reconcile in place, and note typing is target-scoped UI Automation only.
  macOS corrects display-coordinate conversion, drag classification, topology handling, and audio
  recovery. X11 fails closed without ARGB/compositor/input shaping; native Wayland now has a real
  event pump, per-output layer surfaces, hotplug, integer/fractional scale, empty input regions,
  and a released-buffer pool capped at three buffers per output.
- **Transactional lifecycle and publication** - archive traversal/absolute/duplicate/link entries
  are rejected, foreign integrations are preserved, payload swaps and owned integrations roll
  back together, Windows hands privileged replacement to hidden post-exit installer helpers, and
  one draft-only release orchestration now publishes the complete immutable asset set at once.
- **Deterministic hosted X11 readiness** - the Linux visible-overlay smoke gate now waits for the
  compositing manager to own the same X11 selection required by the runtime before starting it,
  and prints the runtime log when readiness fails instead of hiding the actionable cause.
- **X11 input-shape protocol negotiation** - the runtime now negotiates XFixes region support and
  the Shape extension before its required pre-map click-through check. Xorg otherwise rejects the
  first region request with `BadRequest` even when both extensions are installed.
- **Cross-platform release producers** - GNU portable jobs install their ALSA build prerequisite,
  macOS verifies cargo-dist with the native checksum tool and explicitly installs both Rust
  targets, the flat Windows cargo-dist archive is extracted correctly, and WiX legal notices use
  permanent component GUIDs plus an explicit ARM64-compatible installer schema version accepted
  by both x64 and ARM64 MSI builds. A pre-tag candidate mode now builds and verifies the complete
  artifact set without executing publication steps or consuming an immutable tag; hosted MSI
  smoke also exercises an actual v0.2.1 upgrade, repair, downgrade refusal, and uninstall.

### Security
- **Hardened local boundaries and supply chain** - Windows named pipes are limited to the current
  user and SYSTEM, reject remote clients, and use bounded I/O; Unix IPC uses owner-only directories
  and sockets with peer-credential validation, including a short UID-scoped fallback when a macOS
  temporary path exceeds the Unix-socket limit. Elevated Windows lifecycle helpers and note
  pranks resolve only validated system executables rather than the caller's search path. `cargo
  audit` is required, and the compatible vendored `wayland-scanner` security backport carries upstream revision
  `d07c4f91f28b42e5a485823ffd9d8d5a210b1053`'s `quick-xml` fix.

### Removed
- **Obsolete source/config material** - the current `DESKTOP-GOOSE/` reference tree, duplicate
  `behavior.silence_sounds`, nonfunctional `stop_radius`, and old donation/developer material are
  no longer part of the active repository or product surface. Historical transcripts and
  superseded planning records remain untouched.

## [0.2.1] - 2026-07-08

### Changed
- **Side-view neck refinement (renderer polish, per live review)** - the side-profile goose's neck
  no longer bows backward toward the body, and no longer shows a seam where it meets the shoulders.
  The neck spine is now a near-straight, gently forward-leaning curve matching the reference art's
  back edge (`rig.rs` drops the backward bow at `neck_c1`), and its base is buried in the shoulder
  mass so the ribbon outline is covered by the body fill (`neck_base` lowered to reference y92,
  base ribbon width 30→32 in `render/side.rs`). Goldens re-blessed (four side frames plus the
  crossfade-band `top_down_diag`); the website's walk-cycle and pose frames were regenerated to
  match.

## [0.2.0] - 2026-07-08

### Added
- **Renderer V2: flat-illustration dual-view goose (R2, ADR 0014)** - the goose is now drawn in
  the flat-illustration style of the project's own reference art (`docs/art-reference/`): a
  dual-view rig with a mirrored side profile for shallow headings and a freely-rotating top-down
  view for steep ones (125 ms crossfade with 55°/45° hysteresis), stateful plant-and-swing feet
  (planted feet never slide; footmarks stamp at real plant events), a seamless S-curve neck
  ribbon, layered slate wing with scalloped feathers, two-tone beak and webbed-foot legs, eased
  neck posture, blink/breath/honk-tail-flick secondary motion, and 2x-supersampled per-view layer
  compositing. Goldens re-blessed and expanded to six frames; `examples/preview.rs` renders
  contact-sheet/zoom/walk strips for visual tuning.
- **Six-tone configurable palette** - `[colors]` gains optional `goose_shade`, `goose_wing`, and
  `goose_orange_dark` keys (derived coherently from the legacy three when absent, so old config
  files keep working); all six tones edit as R/G/B rows in the TUI, which materializes explicit
  keys on first edit. New defaults follow the reference art.
- **Idle-life behaviors (ADR 0016)** - wander paths meander (rng-driven lateral wobble that fades
  near targets); mud now only comes home from quick off-screen puddle hops (away 8-15 s, tracking
  mud 30-90 s afterward); every 4-7 minutes the goose takes a longer off-screen errand (away
  90-120 s, horizontal-edge preference) and 40% of errands chain a collect-window prank on
  return. All excursion behavior respects manners, never interrupts mischief-in-progress, and
  stays deterministic per seed.
- **`exit` / `quit` stop synonyms** - the goose-speak grammar now accepts `<name> exit` and
  `<name> quit` alongside `bad` / `no` / `no honk`.
- **TUI mouse rows** - `mouse.grab_distance` and `mouse.drop_distance` are now editable.
- **`behavior.attack_randomly` is wired** (was persisted but dead): when enabled, spontaneous
  cursor nabs join the roaming deck (and the mischievous-mood bias), matching the original's
  `AttackRandomly`. Default stays off — without it, nabs come only from clicks and `do nab`.
  New "Attack randomly" TUI toggle.
- **macOS packaging: universal2 `.app` + DMG (R3, ADR 0017)** - `x86_64-apple-darwin` and
  `aarch64-apple-darwin` join `[workspace.metadata.dist].targets`, so cargo-dist emits
  `honk300-<arch>-apple-darwin.tar.xz` (+ sha256) and a Darwin-capable shell installer. A
  hand-authored `.github/workflows/macos-packaging.yml` (chained on the Release workflow, like the
  Windows one) builds a `lipo`-fused universal2 `Honk300.app`, `hdiutil`-packs a compressed
  `honk300-universal2.dmg` (with an `/Applications` symlink), writes a sha256 sidecar, and uploads
  to the tag. `script/package_macos_app.sh` now stamps the real bundle version from
  `HONK300_VERSION` instead of `0.0.0`. Artifacts are **unsigned personal-use** (ad-hoc codesign
  only) — first launch needs a one-time right-click → Open. Supersedes ADR 0013's macOS deferral.
- **macOS `install` / `uninstall` / `update` (R3, ADR 0017)** - `honk300 install` stages
  `~/Applications/Honk300.app` (copying the bundle when run from one, else synthesizing the layout
  from a bare binary), symlinks `honk300`/`honk`/`goose` into `~/.local/bin`, writes a `mac-app`
  install marker, and with `--autostart` writes a `RunAtLoad` LaunchAgent
  (`~/Library/LaunchAgents/dev.emmetts.honk300.plist`). `uninstall` removes those and preserves
  user memes/notes (ADR 0015 §7 semantics); `--purge` also clears
  `~/Library/Application Support/honk300`. `update` replaces the `.app` from the verified DMG for
  bundle installs, and re-runs the cargo-dist shell installer for bare/symlink installs (Linux
  parity, no `cargo install`).

### Changed
- **Wandering no longer starts mud tracking** (previously a 50% roll at every waypoint) — mud is
  an occasional narrative event, not a constant state. `do mud` still forces it.
- **Default goose look** - the default palette moves from pure white/orange to the softer
  reference tones (off-white body, slate wing); the previous look remains reachable via the
  palette config.

### Fixed
- **Windows Per-Monitor-V2 DPI awareness (R1, ADR 0015)** - the process now declares
  PMv2 DPI awareness before creating any window (`init_dpi_awareness()`), and handles
  `WM_DPICHANGED`/`WM_DISPLAYCHANGE` by re-enumerating monitors, rebuilding the per-monitor
  overlay windows, refreshing world bounds, and forcing a repaint. Mixed-DPI overlays are no
  longer blurred or mispositioned, and display/DPI changes no longer need a restart.
- **Non-blocking collect-window + Notepad lifecycle (R1, ADR 0015)** - Notepad spawning is a
  polled state machine instead of a blocking up-to-3s wait, and typing is deferred until the
  spawned window is foreground (best-effort; a focus steal skips the note instead of disabling
  collect-window). The spawned Notepad child is tracked and closed (`WM_CLOSE`, then terminate)
  on close/stop, so `honk300 stop` leaves no notepad.exe zombie and IPC `status`/`do`/`reload`
  stay responsive mid-collect.
- **Unix stale singleton lock (R1, ADR 0015)** - the single-instance guard on macOS/Linux is now
  an advisory `flock` held for the process lifetime (released by the kernel on any death) instead
  of a marker file; a crash can no longer cause a false "already running" refusal. Adds unix-only
  `rustix` to honk-control.
- **macOS event pump + present safety (R1, ADR 0015)** - the overlay pump now drains the
  NSApplication event queue (collect-window close buttons work), and frame presents copy pixels
  into AppKit-owned `NSBitmapImageRep` storage instead of aliasing a reused buffer.
- **Linux degraded-overlay honesty (R1, ADR 0015)** - overlay-creation failure now fails start
  loudly (exit nonzero, clear message) unless `HONK300_ALLOW_HEADLESS=1` explicitly opts into the
  invisible headless mode; a new `overlay` capability rides the status protocol and shows in
  `honk300 status` and the TUI Status tab. X11 also stops re-creating the XFixes input region and
  re-setting the event mask every frame.
- **Non-purge uninstall preserves user content (R1, ADR 0015)** - plain `honk300 uninstall` now
  relocates user-supplied memes/notes to a timestamped `preserved-` folder and prints the
  location instead of deleting them; `--purge` keeps its backup-then-remove behavior.


## [0.1.0] - 2026-07-07

### Added
- **M19 lifecycle commands and update safety** — replaced the placeholder lifecycle commands with
  real `honk300 install [--autostart]`, `honk300 uninstall [--purge]`, and `honk300 update`
  behavior. `install` copies the current executable into the user install location, installs the
  `honk300`/`honk`/`goose` aliases, copies `Assets/` next to the binary, writes install-source
  markers, creates Windows shortcuts or Linux `.desktop` entries, and enables login autostart
  only when requested. `uninstall --purge` backs up user memes/notes before removing config/state.
  `update` reads install-source markers, chooses the matching arch-specific release installer,
  downloads and verifies the `.sha256` sidecar, runs the installer, and verifies post-install
  `--version`; tests pin that no update path uses `cargo install`.
- **M19 cargo-dist and Windows installer release artifacts** — added cargo-dist 0.31 metadata for the
  Windows x64/ARM64 and Linux x64/ARM GNU/musl matrix, a tag-triggered `Release` workflow, a
  chained `Windows Installers` workflow, WiX Global/Corporate MSI manifests, Inno Global/Corporate
  EXE manifests, fresh WiX/Inno GUIDs, per-arch artifact names, install-source marker files,
  optional default-off autostart, alias binaries, asset harvesting, and `.sha256` sidecar upload.
  Release run <https://github.com/RealEmmettS/goose/actions/runs/28842068256> produced the
  cargo-dist shell/PowerShell installers plus Windows and Linux archives; Windows installer run
  <https://github.com/RealEmmettS/goose/actions/runs/28842489497> attached x64/ARM64
  Global/Corporate MSI and EXE artifacts plus sidecars to
  the first release artifacts. macOS DMG/signing/notarization
  remain intentionally deferred behind ADR 0013 and `#m16r`; later macOS packaging defaults to
  unsigned personal-use artifacts unless signing credentials are intentionally added.
- **M16 macOS backend, status, and `.app` staging (implementation in-tree; Accessibility-granted
  smoke pending)** — added `crates/honk-platform-macos` with AppKit/CoreGraphics/ApplicationServices
  dependencies, macOS `start` runtime wiring through the existing Unix IPC transport, one
  AppKit overlay surface per display, CoreGraphics pointer polling/warp, local-time sampling,
  Accessibility-gated focused-window polling for foreign-window ride snapshots, AppKit-owned
  note/meme collect windows, macOS terminal-app classification tests, Accessibility-denied
  capability degradation, and dependency-free macOS audio through `/usr/bin/afplay`. The macOS
  target checks pass for `x86_64-apple-darwin` and `aarch64-apple-darwin`; hosted macOS CI run
  <https://github.com/RealEmmettS/goose/actions/runs/28569332035> now proves the universal2
  bundle/status/IPC gate and uploads `honk300-macos-macos-15` plus
  `honk300-macos-macos-15-intel`. Accessibility-granted cursor/window/collect smoke remains the
  M16.1 readiness blocker.
- **Runtime status protocol and TUI Status tab** — `honk-control` now supports `STATUS` and a
  compact `ControlResponse::Status` payload reporting running state, platform, bundle mode,
  Accessibility, cursor/window/collect/presence/audio capability states, and asset counts.
  `honk300 status` prints the same data, and the config TUI has a Status category plus refresh
  command. `honk-config::BackendState` now preserves supported/unsupported/denied/failed state
  while still collapsing to simple engine options.
- **macOS agent bundle staging** — added `script/package_macos_app.sh` to build x86_64 and arm64
  release slices, `lipo` them into `Honk300.app`, copy `Assets/`, write `Info.plist` with
  bundle id `dev.emmetts.honk300` and `LSUIElement=true`, ad-hoc sign, and validate with
  `plutil`, `codesign`, and `lipo`. Bundle-aware asset discovery now prefers
  `Contents/Resources/Assets`, and TUI Start launches bundled macOS runs through
  `/usr/bin/open -n <Honk300.app> --args start --config <path>`.
- **M17/M18 Linux visible backend and reduced Wayland runtime (CI proven)** — extended
  `crates/honk-platform-linux` beyond session/control plumbing with an X11 visible overlay
  using `x11rb` (`shape`, `xfixes`, `xinerama`, `randr`, `render`), XShape/XFixes input-region
  shaping, Xinerama/root display bounds, pointer sampling, cursor warp, and terminal-filtered
  foreign-window drag snapshots. Native Wayland now has a reduced layer-shell overlay using
  `smithay-client-toolkit` and `wayland-protocols-wlr`; it renders and remains IPC-controllable
  while cursor/window/collect/synthetic-input mischief reports unsupported. Linux collect-window
  support remains unsupported and is visible in status rather than silently attempted.
- **CI-proven M16.1-M18.1 readiness gates** — added `.github/workflows/ci.yml`,
  `script/smoke_m16_macos_accessibility.sh`, and stronger macOS/Linux smoke coverage. Hosted CI
  now covers Windows fmt/test/clippy/release plus Windows x64/ARM64 checks, macOS Intel/arm64
  universal2 bundle smoke with app artifacts, and Linux x64/ARM X11 + Wayland smoke under
  Xvfb/openbox/xcompmgr and headless sway. The X11 smoke verifies both internal frame pixels and
  actual root-window screenshot pixels. The optional self-hosted macOS Accessibility job is gated by
  `HONK300_RUN_A11Y_SMOKE=true` and labels `[self-hosted, macOS, ARM64, honk300-a11y]`.
  `docs/readiness/m16-m18-readiness.md` now records run
  <https://github.com/RealEmmettS/goose/actions/runs/28569332035>. `#m17r` and `#m18r` are Done
  from Linux x64/ARM hosted evidence; `#m16r` remains open because the optional self-hosted
  Accessibility smoke job was skipped.
- **Multi-monitor chase and appearance controls (milestone M15, complete)** — Windows now creates
  one layered overlay HWND per monitor, enumerates signed monitor bounds, chooses the engine world
  bounds from `[behaviors].multi_monitor_chase`, and clips/crops dirty render regions per monitor
  before calling `UpdateLayeredWindow`. With multi-monitor chase off, startup uses the primary
  monitor bounds; with it on, startup uses the full signed virtual desktop. Reloads hot-apply
  normal world options but report multi-monitor chase changes as restart-required.
- **M15 engine/config appearance contract** — `WorldOptions` now carries
  `multi_monitor_chase` and `AppearanceOptions { calm_goose }`. Calm Goose uses the existing Calm
  Suppression/manners path to suppress spontaneous honks, on-hour honks, autonomous
  cursor/window/collect mischief, and Autumn pile chase while leaving direct clicks and CLI/TUI
  pokes under their normal gates. `World::render_bounds(previous)` centralizes dirty-region
  coverage for the goose, previous frame, footmarks, hearts, sleepy particles, and Autumn piles.
- **M15 TUI recolor controls and ADR 0009** — the config TUI now makes Calm Goose live, marks
  multi-monitor chase as restart-required, and edits goose white/orange/outline through separate
  RGB channel rows so hue changes are possible without free-form text input. ADR 0009 records the
  accepted multi-monitor, dirty-render, Calm Goose, and original three-color palette scope.
- **Schedule manners and built-in Autumn (milestone M14, complete)** — added
  `honk-engine::schedule` with `ScheduleOptions`, `LocalMinute`, `PresenceSnapshot`, and
  `PresenceState`, plus `World::set_presence`, `World::manners_active`, and the schedule field on
  `WorldOptions`. Quiet hours are start-inclusive/end-exclusive, support overnight windows, and
  treat `start == end` as no quiet window. Quiet hours, Windows DND, and fullscreen use Calm
  Suppression: spontaneous honks, on-hour honks, autonomous cursor/window/collect mischief, and
  Autumn pile chase are suppressed while direct clicks and CLI/TUI pokes still pass through normal
  config/capability gates. Windows maps `SHQueryUserNotificationState` into platform-neutral
  presence snapshots and polls periodically, warning once and degrading to unsupported if the API
  fails.
- **Procedural Autumn leaf piles** — added platform-free `AutumnState`, piles, leaves, kicked-leaf
  physics, `AutumnLeafPileTask`, render-layer splitting, and Windows render ordering
  (footmarks → Autumn below-goose leaves → goose → Autumn above-goose leaves → hearts → sleepy
  particles). Autumn is active September 1 through November 30 by local runtime-injected date,
  uses recovered reference constants for pile timing/count/physics, and does not copy or load the
  original `Autumn.dll`. The Windows runtime adds `HONK300_SMOKE_LOCAL_DATE=YYYYMMDD` so Autumn can
  be visually smoke-tested outside the season.
- **M14 config and TUI plumbing** — existing version-1 TOML schedule fields now map into
  `WorldOptions.schedule`, `[safety].pause_on_fullscreen` controls fullscreen manners, and the TUI
  removes `(planned)` from live schedule/season rows while adding a separate fullscreen-respect row.
- **Dynamic moods and on-hour double honk (milestone M13, complete)** — added
  `honk-engine::mood` with `MoodKind::{Content,Hyper,Sad,Sleepy,Mischievous}`,
  `MoodIntensity::{Calm,Normal,Spicy}`, seeded weighted transitions, and platform-free
  `LocalTime` injection for schedule-like inputs. Mood effects post-modulate task output:
  sad/sleepy slow movement and lower neck posture, sleepy emits procedural Z particles, hyper
  can request the existing `HyperTask`, and mischievous duplicates only already-enabled
  nab/collect factories in the pickable list. The Windows runtime samples local time outside the
  engine and feeds `World::set_local_time`; the engine emits exactly two high honks at the top
  of a local hour, once per hour. `Sound::Honk` now carries `HonkTone::{Normal,High,Low}` and
  the audio backend maps tones to bundled honk clips while respecting audio toggles.
- **Config TUI and durable configuration (milestone M12, complete)** — added the `honk-config`
  crate for versioned TOML defaults, path resolution, validation, tolerant loading, conversion
  into runtime/world options, and atomic save with practical preservation of unknown keys. The
  default path is `%LOCALAPPDATA%\honk300\config.toml`, `~/Library/Application Support/honk300/config.toml`,
  or `$XDG_DATA_HOME` / `~/.local/share/honk300/config.toml`, with `--config <path>` override.
  Startup falls back to defaults on missing or rejected config and warns without corrupting the
  running state. Reload parses and validates before applying, then hot-applies current M0-M15
  settings for audio, mouse steal/tuning, perch-and-ride, collect-window kinds, pat behavior,
  timing, movement speed, mud/footmark timing, palette, mood intensity, on-hour honking, schedule,
  Autumn, and Calm Goose. Future settings for Wayland/backend and spicy behavior are persisted and
  shown as planned or restart-required.
- **Ratatui reducer UI (milestone M12, complete)** — added the `honk-config-tui` crate with
  reducer-owned state, pure render modules, categories for General, Behaviors, Mischief,
  Schedule, Appearance, Audio, Commands, and About, plus a Poke panel that sends M10 IPC commands.
  Terminal-window protection is shown as always on rather than configurable. Reducer tests cover
  navigation, toggles, numeric edits, dirty/save state, and poke command generation.
- **Shared control crate** — extracted the M10 protocol/client/server code from the binary into
  `honk-control`, reused by the root binary and TUI without changing the wire protocol or adding
  IPC concerns to `honk-engine`.
- **CLI grammar (milestone M11, complete)** — added deterministic pre-clap normalization for
  executable stems `honk300`, `honk`, and `goose`. The binary accepts default start, `start`,
  `plz`, `stop`, `reload`, `do <honk|wander|mud|meme|note|nab>`, `config`, `help`, `--help`,
  `--version`, `--config <path>`, and `--wayland`. `honk plz`, `goose plz`, and `honk300 plz`
  all start; `bad`, `no`, and `no honk` stop; pokes stay explicit through `do <action>`,
  including `do honk`. `install`, `uninstall`, `update`, and `setup` now parse for
  discoverability and M19 implements the lifecycle behavior.
- **CLI/TUI control plane (milestone M10, complete)** — the root binary is now split into
  `src/cli.rs`, `src/control/`, and `src/runtime/windows.rs`. `honk300` defaults to `start`;
  `honk300 start` refuses to create a second goose; and `honk300 stop`, `honk300 reload`, and
  `honk300 do <honk|wander|mud|meme|note|nab>` send finite local IPC commands to the running
  instance. Windows uses a per-user named mutex plus a per-user named pipe. Unix-family readiness
  uses the same protocol over a UID-scoped lock file and Unix domain socket shape for later macOS
  and Linux overlay backends. `honk-engine` gained `PokeAction`, `PokeOutcome`, `World::poke`,
  and `World::apply_options` so stop/reload/poke plumbing stays structured and platform-neutral.
  The protocol rejects malformed, unknown, and oversized payloads. ADR 0004 records the
  CLI/TUI-only control model: no system tray, no global quit key, and no non-IPC stop path.
- **Terminal-window protection** — Windows foreign-window discovery now classifies common terminal
  hosts and excludes them before the goose can ride, collect, move, focus, type into, drag, or
  otherwise manipulate them. The protection rule is documented as permanent and applies to future
  spicy/default-off behavior too; visual overlay over terminal windows remains allowed.
- **Collect-window dispatcher (milestone M9, complete)** — the goose can now drag in Notepad and
  meme windows on Windows. `honk-engine` gained a platform-neutral collect-window contract
  (`CollectWindowId`, `CollectWindowRequestId`, `CollectWindowKind::{Note,Meme}`,
  `CollectWindowCapabilities`, `CollectWindowOptions`, ordered `CollectWindowCommand`s, and
  `CollectWindowSnapshot`) plus `CollectWindowTask` and `World` drain/feed APIs. The task chooses
  note/meme content only when both content and backend capabilities exist, emits ordered spawn /
  move / focus / type / close commands, uses the rig beak tip for drag offset, suppresses
  overlapping pat/click/perch/cursor interrupts while active, leaves Notepad open after typing,
  and closes owned meme windows after a visible dwell. The Windows runtime loads assets from
  provenance-separated `Assets/` directories, spawns and tracks Notepad by PID/HWND, verifies
  foreground focus before Unicode `SendInput`, creates non-topmost owned image windows for memes,
  moves controlled windows with Win32 APIs, toggles pass-through while dragging, feeds snapshots
  back into the engine, and adds `HONK300_SMOKE_COLLECT=note|meme` for visual smoke before M10/M11
  public pokes.
- **M9 assets and ADR 0003** — screened original meme/note assets that pass provenance checks are
  copied 1:1 for personal-use builds, one complete custom in-house counterpart is added per copied
  original, and user-supplied `Meme8.png` is included as an approved meme prop. One original meme
  candidate with a baked-in social handle watermark is excluded rather than redacted. Donate is
  intentionally removed: old donate pages, Patreon links, social handles, and old-project branding
  do not ship. ADR 0003 records the collect-window command/snapshot boundary, asset provenance,
  no-donate decision, cross-platform degradation model, and target expectations.
- **Foreign-window perch & ride (milestone M8, complete)** — the goose now reacts when the
  user drags another application's window on Windows. `honk-engine` gained a platform-neutral
  foreign-window contract (`ForeignWindowId`, `ForeignWindowSnapshot`,
  `ForeignWindowCapabilities`, and `ForeignWindowOptions`) and a transient `PerchRideTask`
  that interrupts the current task, runs to the dragged window's ride anchor, pins to the
  moving anchor if it arrives before release, and resumes the interrupted task on release or
  capability loss. The Windows backend now watches move/size drags with an out-of-context
  `SetWinEventHook`, queues hook events only, polls live geometry via `GetWindowRect`, filters
  the app overlay and invalid/non-root/invisible/minimized windows, unhooks on drop, and exposes
  a temporary `--no-window-ride` opt-out until M12 config exists. `move_window` is reported as
  future capability data only; M8 does not autonomously move windows or start M9
  collect-window/notepad/meme behavior. Added ADR 0002 to pin the engine/backend
  contract and cross-platform guardrails.
- **Cursor mischief: warp + nab sub-states (milestone M7, complete)** — the goose can now steal
  the real cursor on Windows in a bounded, recoverable way. `honk-engine` remains platform-free:
  it owns `CursorCommand::WarpTo(Vec2)`, `MouseStealOptions`, `WorldOptions`, and the
  `NabMouseTask` state machine; platform backends drain cursor commands after each fixed-tick
  update. `TaskCtx` now carries the current platform-neutral pointer plus a cursor-command
  queue, so tasks can request cursor motion without importing Win32/CoreGraphics/X11/Wayland
  APIs. `NabMouseTask` is randomly pickable only when mouse stealing is enabled and the backend
  reports cursor-warp support. A click on the goose also starts `NabMouseTask` when supported;
  when mouse stealing is disabled or unsupported, the older M6 click-to-hyper burst remains the
  fallback. The nab lifecycle seeks the live pointer at charge speed, bites once when the beak
  reaches `grab_distance`, captures the beak-to-cursor offset, then runs a bounded HYPR-style
  retargeting burst while keeping the cursor anchored to the beak until `succ_time` or a
  pull-away/drop threshold ends the grab. While nab owns the cursor, M6 pat/click handling is
  suppressed so synthetic cursor movement does not spawn hearts or interrupt into `HyperTask`.
  The Windows backend now exposes a cursor-warp wrapper, applies only the newest warp command
  after ticking, warns once if warping fails, marks cursor warp unavailable on failure, and the
  binary adds `--no-mouse-steal` as an opt-out. M7 added regression coverage for disabled and
  unsupported paths, click-to-nab, fallback click-to-hyper, seek/grab/drag/drop/timeout, one bite
  sound, drag-offset preservation, HYPR-style retargeting, deterministic command draining, and
  M6 interaction suppression during nab. The full local gate and release build passed before M7
  was moved to Done.
- **M7.0/M7.1/M7.2 completion work** — M7 now includes the completed-milestone audit, the
  mandatory cross-platform `honk-engine` readiness pass, and the renderer/runtime architecture
  spike. The M7.0 audit rechecked M0-M6 against `honk300_plan.md`, fixed stale status docs, and
  created follow-up `#p4d` for fullscreen overlay present-cost measurement. The M7.1 readiness
  pass confirmed the engine stayed platform-free and that current target coverage still respects
  Windows x64/ARM64, macOS Intel/Apple Silicon, Linux x64/ARM GNU, and Linux x64/ARM musl
  expectations. The M7.2 spike selected a future custom CPU sprite/atlas renderer and split that
  implementation into backlog task `#r2v`.
- **Architecture decision records** — added `docs/adr/` with ADR 0001, recording the accepted M7
  cursor-mischief contract, Windows runtime behavior, cross-platform guardrails, renderer
  direction, consequences, verification, and follow-up tasks. `AGENTS.md` and `CLAUDE.md` now
  include ADR maintenance rules so future architecture changes update ADRs, task memory, docs,
  and both changelogs together.
- **Hit-testing: pat (hover-streak + hearts) + click→hyper (milestone M6)** — the goose
  reacts to the cursor. Two distinct interactions (plan §5.9 / §6), built on a new per-frame
  pointer feed (`World::set_pointer` taking a platform-free `interaction::Pointer`; the
  Windows backend polls `GetCursorPos` + `GetAsyncKeyState`). **Pat** = repeated cursor
  *hover-sweeps* over the goose (no buttons): a `PatTracker` accumulates hover-movement into
  a happy streak, each registered pat spawns a rising/fading **heart particle** (new
  `honk-engine::hearts` module + `render::render_hearts`, a clean-room procedural heart) and
  keeps the goose briefly **calm** (a content goose suppresses its spontaneous honks). **Click**
  = a left-press on the goose → a charge-speed **hyper** burst (`task::HyperTask`) that bolts
  around erratically and honks, installed as a transient interrupt that **saves and restores
  the task it suspended** (the resume mechanism perch-and-ride will reuse in M8). Hit-testing
  uses the rig bounding box (`Rect::contains`), naturally click-through everywhere else.
  Engine-side logic is fully unit-tested; the on-screen result was verified visually.
- **Audio (milestone M5)** — the goose honks. A `rodio` backend in the binary plays the
  bundled original sounds (Honk1–4, BITE, MudSquith, Pat1–3) mapped from platform-free
  `Sound` requests the engine emits (`honk-engine::sound::Sound` + a `World` queue drained
  each frame). The goose honks on wander-retarget and squelches while tracking mud.
  `--no-sound` / `--silent` mutes it (the original `SilenceSounds`); a missing audio device
  degrades to a silent no-op. Sounds are embedded via `include_bytes!` from `Assets/Sounds/`.
  Audio is Windows-scoped this round (the macOS/Linux backends wire it in M16/M17).
- **Task state machine + wander + FirstUX intro (milestone M4)** — the M2 roam stand-in is
  replaced by the real AI. A `Task` trait (the documented internal extension seam, plan §18 —
  no external mod ABI), a `TaskCtx`, a registry of randomly-pickable tasks chosen via the
  biased `Deck`, and a `World` task runner. Tasks set targets/params only; the engine
  auto-locomotes. Ships `WanderTask` (roam to random points for a verified 20–40 s dwell, with
  occasional mud-tracking folded in) and a scripted `FirstUxTask` (the goose walks on-stage
  from off the bottom edge and pauses to introduce itself for the verified 20 s
  `FirstWanderTime`, then hands off to roaming). Timings are the verified `config.ini` values
  (20 / 20 / 40); config-driven values arrive with the TOML loader in a later round.
- **Footmarks + mud trail (milestone M3)** — the goose leaves a trail of fading muddy
  footprints while it's "tracking mud," at the verified lifetimes (8.5 s life / 1 s
  shrink-out). To render world-space trails the overlay moved from the small per-goose
  window to a **fullscreen primary-monitor layered overlay** (the plan's intended
  per-monitor architecture; multi-monitor traversal is M15). The engine drops an
  alternating-foot print at each gait half-step while tracking mud; the M2 roam driver
  triggers mud-tracking periodically (M4's `Task_TrackMud` will formalize the trigger).
  Present is capped a touch lower (~40 Hz) since a fullscreen layered present is heavier;
  a dirty-rect optimization (`UpdateLayeredWindowIndirect` + `prcDirty`) is a future perf task.

### Improved
- **M12R config/TUI polish** — `[speeds]`, `[mud]`, `[colors]`, `[moods]`, and on-hour settings
  now validate and map into `WorldOptions` instead of staying write-only. Unknown top-level TOML
  keys and unknown section keys emit a one-shot load warning while still being preserved on save.
  The TUI now uses a row model with scroll support; surfaces movement, mud, color, mood,
  on-hour, and quiet-time rows; edits quiet start/end in 15-minute increments; cycles mood
  intensity through `calm -> normal -> spicy`; confirms dirty quits; routes command outcomes
  through reducer actions; and starts the goose with null stdio plus Windows detached flags.
- **Goose look reworked toward the real original — from direct observation and review.** The
  published modding API documents the rig *model* but not the `updateRig`/`Render` maths (closed
  binary; not decompiled, per the clean-room rule), so the goose was re-grounded by running the
  original Desktop Goose and capturing a local reference screenshot, then iterating against
  golden-frame previews and visual-smoke captures. A generated-sprite-style wing-panel/tall-neck
  pass was saved only as a local visual backup and rejected because it drifted from the original's
  charm. The accepted M7 renderer now uses a deliberate single Bezier body silhouette instead of
  stacked capsules, a flatter/thinner oval body closer to the original side-profile mass, the
  neck drawn under the body/head to hide construction seams, a small plain eye instead of a
  ringed cartoon eye, a short rounded orange beak, fuller tiny orange feet, a subtle dotted
  ground shadow, and updated golden frames for rest/reaching/mid-stride. This remains a
  clean-room procedural renderer; Renderer V2 owns the future atlas-based art pipeline.
- **Windows overlay + walking goose (milestones M1 + M2)** — `honk300` now renders the
  procedural goose on a transparent, always-on-top, click-through-where-transparent overlay
  and walks it around the desktop.
  - **Engine (platform-free, tested):** a fixed-120 Hz `Accumulator` (catch-up clamped to
    avoid the spiral of death); clean-room `locomotion` (accelerate toward `target_pos`,
    cap at the speed tier, face the travel direction, stop cleanly on arrival); a `World`
    with a minimal **roam driver** (a temporary stand-in for the M4 task/AI system); and a
    distance-driven **procedural-feet gait** with a subtle body bob.
  - **Windows backend (`honk-platform-windows`):** a layered popup window presented via
    `UpdateLayeredWindow` with premultiplied BGRA (softbuffer can't do per-pixel alpha on a
    Windows layered window). The small window is repositioned every frame, so it *is* the
    dirty rect — present cost stays proportional to the goose, not the screen. `WS_EX_LAYERED`
    **without** `WS_EX_TRANSPARENT` gives natural per-pixel-alpha click-through (opaque goose
    clickable, transparent margins fall through). This presenter shape was superseded by the
    M3 fullscreen primary-monitor overlay so mud/heart/world-space props can render in-place;
    the M7.0 audit tracks dirty-rect/per-monitor optimization as follow-up work.
  - **Renderer reworked to the original's technique:** capsule body / neck / two-segment
    head, an orange beak and webbed feet, a grey outline, and a ground shadow — tuned to
    resemble the real side-profile goose, animated by the neck-lerp + gait + body bob.
  - **Root `honk300` binary:** the three-clock loop (sim 120 Hz, present ~60 Hz on the
    goose's bounding box). Golden frames re-blessed (rest / reaching / mid-stride).
  - **Design note (deviation from plan §4):** the overlay uses raw Win32 (the `windows`
    crate) rather than winit — a small moving layered window via `UpdateLayeredWindow` is the
    canonical low-CPU desktop-pet pattern, and per-backend windowing fits the capability-trait
    design. winit can be revisited at M15 (multi-monitor) / M16 (cross-platform loop). The
    workspace root is now also the `honk300` binary package; added the `honk-platform-windows`
    crate.
- **Cargo workspace + `honk-engine` crate (milestone M0)** — the platform-free
  simulation core: `#![forbid(unsafe_code)]`, no windowing/OS/audio/input dependency,
  fully headless-testable. Ported 1:1 from the verified modding-API source
  (`…/GooseModdingAPI/{Exports.cs, SamEngine.cs}`): `Vec2` + `SamMath`; the 120 Hz
  fixed-timestep constants (`DT = 1/120`); the **faithful biased** `Deck` shuffle-bag
  (decision C8 — a seedable SplitMix64 drives it for deterministic tests; RNG internals
  are clean-room); `GooseEntity` + `ParametersTable` at the verified values (Walk/Run/
  Charge 80/200/400, accel 1300/2300, step 0.2/0.1, mud 15); the rig geometry constants
  with a clean-room `update_rig`; `ProceduralFeet`; the 64-slot `FootMarks` ring buffer
  (lifetime 8.5 s / shrink 1 s); and a clean-room tiny-skia renderer (`Rig → Pixmap`
  with a dirty-rect bounding box). Pinned by 26 unit tests (constants, rig vertices, the
  exact `Deck` sequence + its documented bias, footmark lifetimes) and 3 committed
  golden-frame PNGs. The renderer's proportions are a first clean-room approximation —
  the goldens are a regression baseline, not a fidelity reference (visual tuning is M1+).
- **Workspace scaffold** — root `Cargo.toml` (workspace, edition 2021 / Rust 1.95 via
  `[workspace.package]`, `[profile.dist]`), `rust-toolchain.toml` pinned to 1.95, and
  `crates/honk-engine/Cargo.toml`. The `[workspace.metadata.dist]` / WiX / CI blocks are
  intentionally deferred to the M19 packaging round. Local gate is green
  (`fmt --check`, `clippy -D warnings`, `test --workspace`, `build --release`).
- `honk300_plan.md` — **the canonical, authoritative plan.** A claim-tested *hybrid* that
  synthesizes `claude_plan.md` (structural spine) and `codex_plan.md` (grafts), then folds in an
  approved round of new scope. Each draft's load-bearing claims were verified against ground
  truth: engine constants checked against `…/GooseModdingAPI/Exports.cs` (claude exact; codex's
  Appendix-B speeds wrong), the biased `Deck` against `SamEngine.cs`, and the QubeTX conventions
  (editions, the 6 base targets, `cargo-dist 0.31.0`, ICE flags) across TR300/ND300/WB300. Adds:
  the new autonomous behaviors, a ratatui `<name> config` TUI, a three-name goose-speak CLI, and
  a full all-OS/all-arch build matrix. Build milestones now **M0–M19**.
- `claude_plan.md` — comprehensive, adversarially-reviewed plan for **honk300**, a
  cross-platform (Windows/macOS/Linux) Rust reimplementation of Desktop Goose. Derived
  from analysis of `DESKTOP-GOOSE/` (the original v0.31 Windows + v0.22 macOS builds) and
  the `*300` sibling repos (TR300/ND300/WB300). Captures the reverse-engineered engine
  (rig geometry + physics constants, 120 Hz fixed tick, the biased `Deck` shuffle-bag, the
  Task/`InjectionPoints` model from `…/GooseModdingAPI/{SamEngine,Exports}.cs`), a
  Cargo-workspace architecture (`honk-engine` + capability-trait platform backends), build
  milestones M0–M17, the packaging pipeline (cargo-dist + hand-authored
  `windows-installers.yml`), a per-platform capability matrix, and a ranked risk table.
- `codex_plan.md` — a parallel planning document produced by Codex.
- `CHANGELOG.md` / `HUMAN_CHANGELOG.md` — dual changelogs, mirroring the `*300` family
  convention.
- `CLAUDE.md` — repository guidance for future Claude Code sessions.

### Changed
- M15 is now Done, M16 implementation has moved into the active backend-readiness track, and
  Renderer V2 remains tracked separately as backlog task
  `#r2v`. The task records now preserve M7's audit/readiness/renderer work, M8's foreign-window
  readiness pass, M9's collect-window asset/ADR/target-readiness work, and M10's IPC/control
  readiness work, plus M11 CLI grammar, M12 config/TUI readiness work, M13 moods/hourly-honk
  closure, M14 schedule/Autumn closure, and M15 multi-monitor/appearance closure.
- `README.md`, `AGENTS.md`, and `CLAUDE.md` were updated to reflect M0-M19 implementation in
  tree, the M16.1 macOS Accessibility evidence gate, Linux X11/Wayland presentation support,
  M19 release evidence, and the ADR
  0001/0002/0003/0004/0007/0008/0009/0010/0011/0012/0013 location and maintenance rules.
- Added **ADR 0005** (M11 three-name CLI, goose-speak, and the poke-outcome round-trip) and
  **ADR 0006** (M12 config TUI, durable TOML, and the capability/preference boundary), recording
  the previously-undocumented M11/M12 decisions and the four contract corrections from the
  adversarial review.
- Added **ADR 0007** (M13 dynamic moods and local-time injection), recording the platform-free
  mood state machine, honk-tone contract, and runtime-owned local-clock sampling boundary.
- Added **ADR 0008** (M14 schedule, presence, and Autumn), recording Calm Suppression, the
  schedule/presence engine boundary, Windows presence polling, and the built-in Autumn constants.
- Added **ADR 0009** (M15 multi-monitor and appearance), recording the per-monitor Windows overlay
  boundary, dirty-region presentation, Calm Goose valve, restart-required multi-monitor reload
  rule, and original three-color recolor scope.
- Added **ADR 0011** (M17/M18 Linux control runtime and degraded Wayland foundation), recording
  the X11-first session rule, forced/native Wayland degradation, Linux Unix IPC runtime, terminal
  classifier, command-player audio, and the remaining Linux-host readiness gates that existed
  before the visible backends landed.
- Added **ADR 0012** (M16.1-M18.1 CI-proven backend readiness), recording the hosted CI matrix,
  optional self-hosted macOS Accessibility gate, Linux X11/Wayland smoke contract, and the rule
  that readiness tasks do not move to Done until CI evidence is recorded.
- `claude_plan.md` and `codex_plan.md` are now **superseded reference drafts**; `honk300_plan.md`
  is canonical. The "Read these first" pointers in **both** `CLAUDE.md` and its Codex twin
  `AGENTS.md` were updated in lockstep (canonical plan, milestone range M0–M19, workspace
  cross-reference → `honk300_plan.md` §7).
- `README.md` gained a **"Status — the decided plan"** section recording `honk300_plan.md` as
  canonical and summarizing the decided direction (three-name goose-speak CLI, ratatui config
  TUI, new autonomous behaviors, no external mods / no tray, all-OS/all-arch builds).

### Fixed
- **Control responses now report the real outcome (M11 round-trip).** `honk300 do <action>` and
  `reload` previously always answered `OK` because the server thread responded at command-enqueue
  time, before the simulation ran. The transport now completes a request/response round-trip:
  `honk-control` gained `ControlRequest`, a bounded (2 s) wait for the sim's answer, and a
  `PokeOutcome`→`ControlResponse` mapping (`Busy` → `ERR BUSY`, `Unsupported` → `ERR UNSUPPORTED`,
  reload failure → `ERR RELOAD_REJECTED`, timeout → `ERR TIMEOUT`). The CLI/TUI "rejected: {code}"
  paths now actually fire. (ADR 0005.)
- **Cursor-warp capability is no longer seeded from the mouse-steal preference (M12 reload).** The
  Windows runtime initialized `cursor_warp_supported` from `!no_mouse_steal`, latching warp off so
  a config edit that re-enabled mouse steal never took effect until restart. It is now a pure
  platform capability (`true` on Windows, via `initial_cursor_warp_supported`) that degrades only
  on a real warp failure; the preference is applied solely through `MouseStealOptions::enabled`.
  (ADR 0006.)
- **Collect-window capability loss now survives reload (M12).** A backend collect-window failure
  was recorded only in engine state and was overwritten by the next reload, so the goose kept
  retrying a dead capability. `BackendState` gained `collect_window_supported`, threaded through
  `Config::effective_options`, so the loss is durable across reloads. (ADR 0006.)
- **Disabling the pat streak no longer disables clicking (M12 interaction).** `interaction.pat_streak`
  gated the click reaction as well as pats. It now scopes to the hover-pat hearts/calm only;
  clicking the goose still triggers a hyper burst (or a cursor nab when mouse steal is supported).
  (ADR 0006.)

### Decided
- **Renderer V2 direction:** use a custom CPU sprite/atlas blitter that outputs premultiplied
  pixels for each platform backend. Keep `tiny-skia`/`resvg` for vector effects or
  asset-rasterization helpers, but do not make Vello/wgpu, Skia, Bevy, Macroquad, or ggez the
  main runtime renderer for the desktop-pet overlay. Future atlas metadata should include stable
  anchors, beak/cursor attach points, hit masks, frame bounds, and animation tags.
- **Three invocation names** (`honk300` / `honk` / `goose`) with a finite, deterministic
  "goose-speak" grammar (e.g. `goose plz` to start, `honk bad` / `goose no honk` to stop,
  `goose do honk` to poke, `<name> config`, `<name> help`) — a fixed phrase map, **not** runtime
  NL parsing.
- **TOML config** (`config.toml`) replacing the original `.ini`, original keys preserved at the
  verified values, versioned + tolerant loader.
- **No external mod system** (no DLL/WASM/data mods). Autumn becomes a **built-in** season/task;
  extensibility is via documented internal seams (`ARCHITECTURE.md` + rustdoc).
- **No system tray and no global quit key.** Start, stop, reload, pokes, and future configuration
  are CLI/TUI-only over the **single-instance + IPC command channel** (`start` / `stop` / `do` /
  `reload`) that is also the Wayland-safe control path and the TUI's hot-apply transport.
- **Terminal windows are protected.** The goose may visually overlay terminals, but terminal
  windows are never valid ride, collect, movement, focus, typing, drag, or spicy-behavior targets.
- A **ratatui** config TUI at `<name> config` (QubeTX-family architecture: reducer + crossterm +
  `tokio::select!`) toggling every behavior incl. Autumn; **hot-apply where cheap, restart-note
  otherwise**.
- **New autonomous behaviors** (each a toggle, scoped to parameter-modulation of the procedural
  rig — no new art): dynamic moods, seasonal moods, multi-monitor chase, on-the-hour double honk,
  perch-&-ride windows, hover-sweep pat streak + hearts, quiet-hours/DND-fullscreen respect, a
  Calm-goose valve, and manual poke commands. Default = full prank, always-on.
- **Build for every advertised OS and architecture:** Windows x64 **and ARM64**, macOS Intel
  **and Apple Silicon** (universal2 `.app`/`.dmg`), Linux x64 **and ARM** (gnu + musl) — arch is a
  build/packaging axis, capability is an OS/display-server axis (`Cap<T>`).
- App name **honk300** (binary `honk300`, optional `honk` alias); fresh permanent WiX/Inno
  GUIDs (never reuse the sibling repos').
- Clean-room **procedural** goose renderer — no sprite extraction. Original sound effects,
  screened original memes, and screened original notes are bundled 1:1 for personal-use builds;
  every copied meme/note original gets one complete custom in-house counterpart. Old donate pages
  and old developer references do not ship.
- Linux: **X11-first** (runs under XWayland on Wayland sessions); native Wayland behind an
  opt-in `--wayland` flag with reduced mischief.
- Distribution: Windows-first installer matrix (Global/Corporate × MSI/EXE) + shell/
  PowerShell installers + macOS `.app`/`.dmg` + Linux `.desktop`. **No crates.io** —
  `crates-publish.yml` intentionally dropped from the family pipeline.

### Notes
- A personal-use `v0.1.0` GitHub release now exists with Windows/Linux archives, shell/
  PowerShell installers, Windows x64/ARM64 MSI/EXE installers, and checksum sidecars.
  `DESKTOP-GOOSE/` remains the reference copy of the original app and contains third-party
  copyrighted assets; do not redistribute those bundled assets publicly.
