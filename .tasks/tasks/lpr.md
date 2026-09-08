TT;DR: Add Linux notes and pictures through the existing companion, with bounded ownership and honest positioning support.

## Why
The user-approved September refinement explicitly includes Linux owned props and normal Wayland placement when animated positioning is unavailable. Prepare this implementation in a separate checkout while stage one is qualified; publication remains ordered through #rlpi and #rr1.

## Scope
ADR 0043 records the local implementation boundary. Rust retains assets, behavior, sizing, configuration and runtime ownership. The existing installed companion supplies Linux native GTK prop windows through a bounded private child protocol. Preserve every existing installation, immutable candidate, terminal exclusion and independent user-close reaction.

The new prop work also gives all supported native backends a fixed admission ceiling. Existing Windows/macOS controllers retain completed notes until the user closes them, and their maps have no current count bound. At capacity, defer a new delivery; never delete a person's existing note or latch a healthy backend to failed.

## Plan
First pin admission and placement behavior through real engine tests. Then build one note end to end using the exact companion. Only after that works, add images, user closes and resource/connection failure handling. KDE, portals and foreign-window operations remain their separate approved tasks.

Load-bearing premise: the already-packaged GTK4 host can expose its own X11 window identity and a normal Wayland surface without requiring GTK in the Rust CLI. A native Linux build and an Xvfb/labwc window probe are the earliest falsifiers. If the host cannot do either, stop downstream support claims and revise ADR 0043 before expanding the protocol.

## Impact
Linux users gain owned notes and pictures; source users without the companion retain CLI/TUI control and an explicit unavailable-prop explanation. Prop resource pressure cannot create an unlimited stream of native windows. Additional GUI code stays in the existing independently hashed companion payload.

## Acceptance
Functional: real runtime deliveries create bounded, fully fitted owned windows; X11 moves only those windows; Wayland uses ordinary compositor placement and never invents positioning capability. User close, process loss and graceful shutdown obey the current engine contract.

Evidence: locked Rust tests and native GTK builds plus at least two real delivery/close cycles under Xvfb and native ARM64 labwc. Signed/platform packaging and public publication belong to #rlpi, after #rr1. No physical Pi acceptance is inferred. Checkpoint after the first native note probe and after each distinct failure; two attempts without new evidence require added diagnostics or a changed premise.

## Verification
- [x] Real engine tests cover capacity exhaustion/recovery, placement-only delivery and interrupted actions.
- [x] Native GTK note/image tests verify complete fit, exact owned-window movement, close events and child cleanup.
- [ ] GNU/musl architecture builds preserve CLI/TUI availability and reject broken companion/protocol inputs.

## Status
Active in the isolated `codex/goose-linux-expansion` checkout. Both native GNU architectures
pass real X11 engine delivery/text, child-loss failure latching, reload, graceful cleanup,
standalone controls, native capacity/close/move tests and complete-image pixels. The local
Rust workspace, strict Clippy, release build, 130 Python checks (three host skips), native
settings tests/check/build pass. Workflow 34186038608 passed both GNU architectures,
including visible X11 delivery, actual labwc 0.7.1 placement and scales one/two, input
rejection, child loss and standalone controls. ARM64 note and complete-image captures
were visually reviewed. The exact official GNU companions passed on GTK 4.6 in the full
native settings matrix. Add those same real prop checks to the shipped musl companions;
their native results and complete release integration remain open. Nothing is published.

## Evidence
| Criterion | Oracle / invocation | Raw result or pointer | Interpretation | Limitation | Status |
|---|---|---|---|---|---|
| Bounded engine admission and placement-only behavior | locked workspace suite and native GNU controller tests | 34185456570; local complete workspace suite | Existing notes retained at capacity; no invented Wayland movement | Native full-platform package gates pending | PASS |
| Native host text, pixels, moves, closes, invalid input and EOF | smoke_owned_props_linux.py, scales one and two | 34185456570 GNU x64/ARM64 artifacts | Both cycles and complete-picture pixel checks pass | X11 native host; no Pi hardware | PASS |
| Runtime delivery, failure and shared status | smoke_runtime_props_linux.py | 34186038608 runtime-props-evidence/result.json and session-status.json | Real visible native text, child loss, failure latch, caller-independent status and standalone controls pass | Disposable GNU desktop | PASS |
| Visible runtime note and labwc | strengthened capture and native placement probes | 34186038608 props-labwc/result.json and engine/result.json | Both native architectures pass labwc 0.7.1, scales one/two, normal placement and opt-in enforcement; ARM64 images inspected | Headless pixman, no physical Pi | PASS |
| Oldest GTK and exact GNU release companions | Native settings production-payload prop checks | 34186768042 all eight settings lanes pass | GNU x64/ARM64 prop checks pass with GTK 4.6 after production identity verification | Disposable desktops, no physical Pi | PASS |
| Exact musl release companions | Shared production-payload prop checks | next complete settings matrix | Require real host and runtime behavior under Alpine on both native architectures | Native result pending | NOT RUN |

## Activity
- 2026-09-08: Both musl jobs stopped during package installation because Alpine does not
  ship xcompmgr. Its native x64/ARM64 picom package supplies the XRender compositor;
  use that in the same private Xvfb session before repeating the unchanged prop assertions.
- 2026-09-07: The entire native settings matrix passed, including real production GNU
  prop delivery on the older GTK baseline. Add the same host/runtime/pixel/failure checks
  for native x64/ARM64 musl payloads, retaining independently verified production identity.
- 2026-09-07: Both GNU architecture jobs passed native X11 and labwc with visible real-runtime notes, scaled complete pictures, owned-only movement, user/program closes, malformed input, child failure and cleanup. Reviewed ARM64 captures. Add the same checks to the official Ubuntu 22.04 production companion gate; tighten the pixel oracle so gray desktop pixels cannot substitute for the note.
- 2026-09-07: Real x64 X11 engine delivery, text, companion failure latching, graceful cleanup, caller-independent session readback and standalone controls pass at eafa11c. Screenshot review found the test compositor's debug mode flattened the transparent overlay over the note; use the existing production-like client-composition mode and require visible note pixels/geometry. Labwc then stopped on the older wlr-randr lacking --json; read its actual text output. Visual and Wayland gates remain open.
- 2026-09-07: Reconciled the independent implementation and dependent publication tasks, preserving release order. Refreshed the versioned task dashboard from the installed skill bundle while retaining its title, settings and stopped server state.
- 2026-09-07: The retained failure log at 54de704 shows the fixture's zero initial-wander duration was rejected before overlay startup. Use a valid long quiet initial wander and fail immediately with the saved runtime log if startup exits. Add real CLI/shared-GUI-service session and positioning readback to native delivery qualification.
- 2026-09-07: Inspection found the direct GTK test desktop had Openbox without the compositor required by the real goose overlay. Add xcompmgr and an explicit live selection-owner check before engine delivery; keep production fail-closed transparency unchanged.
- 2026-09-07: Native ARM64 strict Clippy, protocol/engine tests, release build, both complete-image pixel captures and scaled native close/capacity cycles pass at 13ebb89. The real-delivery fixture incorrectly treated the successful 'not running' status command as an active runtime; inspect its reported state instead. Runtime/labwc/invalid-input/standalone checks remain pending.
- 2026-09-07: Added malformed/oversized-input native cleanup checks and a real standalone runtime/shared-settings-service probe without the companion. Documented the implemented limits and ownership protocol; those expanded hosted assertions are pending.
- 2026-09-07: First native controller compilation stopped only on a stale Linux installer glob import, now limited to Windows/test callers in both release and follow-on branches. Add labwc placement, native display scaling, real Wayland delivery and full-image pixel checks to the next hosted pass; these are new pending probes, not Pi hardware acceptance.
- 2026-09-07: Both native GNU host cycles passed at 35829c9, including Unicode text, capacity, owned movement, Close, recovery and EOF cleanup. Screenshot review found capture before the image's first paint; add a complete four-quadrant pixel oracle. Implemented Rust retained-child/nonblocking protocol, runtime readiness/failure/admission and logical Wayland sizing, with real engine-delivery/child-loss CI next.
- 2026-09-07: Native tree and screenshot evidence isolated the unavailable Close action to GTK announcing the button's multiplication symbol despite its separate label. Make Close the visible button text as well, retaining the exact native action oracle before another run.
- 2026-09-07: Embedded-font GNU builds and text/capacity/movement checks pass, but native Close activation was unavailable on both architectures. Added bounded action-tree and screenshot diagnostics before another attempt, plus a real header drag handle and Cairo-owned pixel lifetime. Rust transport work is local only and does not advertise support before the native gate passes.
- 2026-09-07: Both GNU probes now pass native Unicode text, capacity refusal with retained contents and actual owned movement. The close fixture used XDestroyWindow through xdotool, bypassing GTK close handling and causing a GDK assertion. Replaced it with the real owned GTK Close accessibility action and retain malformed protocol diagnostics. Added embedded approved fonts and bounded dispatch before the next native repeat.
- 2026-09-07: The first GNU x64 and ARM64 production companions compiled and created eight exact 400x250 native notes at the requested positions. The probe then called Noble's deprecated Accessible.get_text instead of the Text interface; corrected the qualified call. This is fixture evidence, not a complete delivery pass.
- 2026-09-07: Added the internal GTK companion host and bounded Zig decoder. Decoder tests pass locally; the first dedicated native GNU x64/ARM64 host build and two-cycle Xvfb note/image/capacity/close probe are next. Do not claim Linux runtime delivery before that earliest native premise is verified.
- 2026-09-07: Implemented bounded admission in the real engine and Windows/macOS controllers, with exhausted/recovered capacity and placement-only note/meme regressions. Added a disposable Windows native exhaustion/close/recovery test. Linux companion implementation and its native evidence remain pending.
- 2026-09-07: Split implementation from the ordered stage-two publication task so work can proceed without changing the first release candidate.
