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
- [ ] Real engine tests cover capacity exhaustion/recovery, placement-only delivery and interrupted actions.
- [ ] Native GTK note/image tests verify complete fit, exact owned-window movement, close events and child cleanup.
- [ ] GNU/musl architecture builds preserve CLI/TUI availability and reject broken companion/protocol inputs.

## Status
Active in the isolated `codex/goose-linux-expansion` checkout. Architecture and admission implementation are starting; no Linux prop capability is yet implemented or advertised.

## Activity
- 2026-09-07: Added the internal GTK companion host and bounded Zig decoder. Decoder tests pass locally; the first dedicated native GNU x64/ARM64 host build and two-cycle Xvfb note/image/capacity/close probe are next. Do not claim Linux runtime delivery before that earliest native premise is verified.
- 2026-09-07: Implemented bounded admission in the real engine and Windows/macOS controllers, with exhausted/recovered capacity and placement-only note/meme regressions. Added a disposable Windows native exhaustion/close/recovery test. Linux companion implementation and its native evidence remain pending.
- 2026-09-07: Split implementation from the ordered stage-two publication task so work can proceed without changing the first release candidate.
