# ADR 0028 — Shared Goose Control Surface And Tray Parity

- Status: Accepted (2026-07-17)
- Relates to: ADR 0004 (local control and terminal protection), ADR 0010 (macOS agent bundle),
  ADR 0020 (Developer ID distribution), ADR 0022 (Accessibility onboarding), ADR 0023 (rolling
  latest), and ADR 0024 (macOS menu-bar control).
- Supersedes: ADR 0024 only where it requires the visible title **Honk** and treats the lack of
  Windows/Linux tray surfaces as a permanent product boundary. macOS Configure/Quit semantics,
  local IPC, terminal protection, permission behavior, bundle identity, no-native-preferences,
  and graceful shutdown remain accepted. Windows and Linux trays remain unimplemented until
  independently designed and qualified.

## Context

ADR 0024 made the signed macOS agent controllable without a terminal command, but its variable-
width **Honk** title looks like temporary developer UI. The operator wants a small recognizable
goose that can become a consistent product mark when Windows notification-area and Linux desktop
control surfaces are added later.

The icon cannot become a new control plane. Honk300 already has one configuration schema, one
ratatui editor, one same-user local IPC channel, and one engine-owned graceful stop. Future tray
implementations must reproduce the behavior people learn on macOS instead of inventing native
settings that drift or terminating the process abruptly.

Small status artwork also has different platform requirements. AppKit template images are masks
that macOS tints for light, dark, highlighted, increased-contrast, and menu-bar appearances. A
full-color application icon or a baked background would become illegible. Raw SVG loading works
on the qualification Mac but is not the safest dependency across the bundle's macOS 11+ support
range.

## Decision

### One shared icon source

`Assets/UI/honk300-status-goose.svg` is the canonical cross-platform source. It is a transparent,
two-path, monochrome side-profile goose generated with Quiver AI and normalized in-tree. Its
prompt, generation identifier, cleanup, export command, and accessibility requirements live in
`Assets/UI/README.md`. The asset contains no application background, status-bar rectangle,
gradient, text, or runtime network dependency.

macOS seals both the canonical SVG and a deterministic 36×36 RGBA
`honk300-status-goose@2x.png` representation into `Honk300.app`. AppKit loads the PNG as an
18-point `NSImage`, marks it as a template, scales it proportionally down, and installs it on a
square status item. The PNG removes raw-SVG decoder compatibility from the macOS 11 runtime path;
the SVG remains the future Windows/Linux source of truth.

The Rust owner retains the image, menu, status item, and weak AppKit action target for the entire
runtime lifetime. The image-only button has the independent accessibility label **Honk300
controls** and matching tooltip. If an unbundled development run has no decodable resource, the
runtime keeps a variable-width **Honk** text fallback; a cosmetic asset failure must not prevent
the goose, Configure, or Quit from starting.

Packaging copies both resources before inside-out signing. Candidate/release contracts require
them in the staged app, final extracted app ZIP, first DMG mount, and final remounted notarized and
stapled DMG. Missing resources fail production packaging rather than silently publishing the
development fallback.

### Learned behavior is the future parity contract

Every platform control surface must expose a clear accessible name and preserve these semantics:

- **Configure Honk300…** opens the already-shipped terminal TUI for the same installed executable.
  That TUI owns schema validation, save, status, start/stop, and reload. Closing it restores the
  terminal and leaves the running goose alive. A future native tray may launch an equivalent
  terminal command, but it must not add a second preferences schema or hidden save path.
- **Quit Honk300** records the shared graceful-stop intent. The goose keeps simulating and
  presenting while it runs through a real exposed screen edge; the process exits only after the
  complete pose is hidden, the final transparent frame is acknowledged, runtime-owned props are
  cleaned up, and the singleton can be released. Tray handlers must not call an immediate exit,
  terminate API, or kill the process.
- The control surface remains usable while macOS is waiting for Accessibility approval. Opening
  Configure does not grant permission; Quit still works. Permission-bound movement, cursor, and
  window behavior stays suppressed until the same running identity becomes trusted.
- The item exists only while the runtime exists. It does not add a desktop/application background,
  ordinary settings window, network endpoint, global hotkey, Dock process on macOS, or exception
  to protected-terminal rules.

Windows and Linux implementations may use their native notification-area,
StatusNotifier/AppIndicator, or desktop-specific mechanisms, but they must preserve the same
actions, accessibility names, graceful lifecycle, same-user control boundary, and failure
behavior. Platform capability differences must be reported honestly. This ADR makes the shared
asset and behavior reusable; it does not claim those surfaces are present in v1.0.2.

## Observed macOS reference behavior

The physical-Mac qualification established the behavior future trays must mimic:

- exact published v1.0.1 exposed its menu during denied and granted Accessibility states;
- Configure opened the bundled `honk300 config` entry point as the complete 120×30 terminal TUI,
  and `q` restored Terminal without stopping the goose;
- the same process changed from denied to supported after the user granted Accessibility;
- Quit produced the engine-owned walk-off and exited after 5.415 seconds, after which the same
  signed app relaunched immediately with its durable grant;
- a dark-mode native note used the semantic system label color and remained readable;
- process-specific 60-second profiles on the loaded M2 host measured 7.80% median CPU while
  denied and 8.60% while actively wandering with Accessibility, with at most 14.66 MiB RSS and no
  positive RSS growth.

These measurements qualify the reference behavior, not future platforms. Exact v1.0.2 icon,
package, update, notarization, and post-publication evidence is recorded in its readiness report.
The Mac has one display and Ghostty is absent, so live multi-monitor/hot-plug and Ghostty evidence
remain explicit hardware/software waivers. Hardened runtime prevents `leaks` from attaching to the
exact signed release; product-equivalent instrumentable zero-leak evidence is retained without
claiming an exact-release attach.

## Release outcome

v1.0.2 shipped from exact commit `964305869e9ec28768c789465db1b6317dfa3f6f`. Replacement
candidate `29565557915`, same-SHA main CI `29566294408`, atomic release `29566759574`, and
post-release installer smoke `29567257622` all passed the complete Windows/macOS/Linux matrix.
The first candidate had failed closed before tagging when Windows treated a transient zero-byte
`PIPE_NOWAIT` named-pipe poll as an empty command; bounded retry through the existing deadline
fixed that transport race without changing this ADR's icon or control semantics.

Fresh public artifacts passed the pinned G2 Developer ID identity, hardened runtime/timestamps,
notarization, stapling, Gatekeeper, universal slices, sealed icon resources, manifest hashes, and
the three-item DMG contract. The public app ZIP hash is
`1c78959543e5860ebd33e5e1a8aac1c73be3c8cf7c2a3465f7478fa822933e98`; the public DMG hash is
`7ee91efd374a5777e43f78a22d652a5847b7087105d3ccbde6569e87b0844ce5`. A preserved public v1.0.1
installation updated through the real latest manifest to these v1.0.2 app bytes, all three aliases
then no-op updated without disturbing the process or receipt, and the public menu completed its
graceful Quit in five seconds before immediate restart. `thegoose.app` resolved the same exact DMG
through its OS-specific progressive disclosure. These are observed release facts; one-display,
absent-Ghostty, and hardened-runtime attach limitations remain explicit.

## Consequences

- macOS users get a compact goose mark that follows system appearance while keeping the exact
  Configure and animated-Quit model they already learned.
- VoiceOver and other accessibility clients receive a stable control name even though the visible
  button has no title.
- The production app cannot quietly omit the icon, while source/development runs remain resilient.
- Future Windows/Linux tray work starts from one asset and behavioral contract, but must still add
  platform ADRs, lifecycle tests, accessibility verification, packaging, and real-hardware proof.
- v1.0.0 and v1.0.1 remain immutable. This change ships through the full v1.0.2 candidate,
  exact-SHA CI, tag, signing, notarization, atomic publication, update, and website-latest path.

## Verification

- Rust tests load the runtime PNG through native AppKit, require a nonzero representation and
  size, set 18-point template mode, and prove the image remains a template.
- Python contracts pin the canonical SVG's transparent two-path shape, PNG signature, copy-before-
  sign ordering, and presence across every signed/notarized distribution shape.
- Physical-Mac verification inspects the actual menu-bar mark in system appearances, accessibility
  label and actions, bundled TUI launch/restoration, permission-wait availability, animated Quit,
  immediate restart, and process-specific resource use.
- Release verification requires Developer ID identity, hardened runtime, timestamp, notarization,
  stapling, Gatekeeper, manifest/hash identity, graphical install, v1.0.1→v1.0.2 CLI update, and
  thegoose.app's stable-latest DMG response.
