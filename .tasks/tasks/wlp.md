TT;DR: Explicit portal pointer permission with live device and terminal safety checks.

## Why
The approved refinement requests opt-in KDE/portal integration while ordinary native
Wayland remains reduced. Window observation cannot confer permission to inject input.

## Scope
Use the native XDG RemoteDesktop grant and libei sender for pointer motion only. Keep
permission ephemeral, expose its live state, cancel on any permission/device/adapter loss,
and require fresh separately qualified terminal exclusion before every action. Do not
request keyboard, button or touch injection, install global input access, or persist a token.
Authoritative sources are ADR 0044, the approved plan, libei/liboeffis APIs and actual
KDE portal/compositor observations. GNOME/Sway/Hyprland retain separate qualification.

## Plan
Start with the real library and native desktop grant. Within two native attempts inspect
the earliest failed premise and adjust its implementation or supported-desktop claim.
Do not build on synthetic acceptance or weaken the live device/terminal safety boundary.

## Acceptance
An explicit native portal grant supplies a live pointer device; bounded permitted motion
reaches the private compositor and stops after cancellation. The production Rust runtime
uses this exact path, and settings clearly distinguish grant, readiness and unsupported states.

## Verification
- [x] Actual portal consent, granted libei devices, motion and cancellation are observed.
- [x] Runtime and native settings use the qualified path without changing saved drafts.
- [x] Fresh target/terminal checks, device pause/removal and disconnects fail closed.

## Status
Done. Final-source native KDE 6 tests pass actual engine motion, native consent
denial/retry, pending/granted cancellation, unchanged drafts, external removal,
stop/crash recovery and backend loss during motion. Repeated healthy CLI/native
setup preserves the active grant. The complete candidate/main/publication and
eight fresh-public lanes pass at d38e4845 in v1.6.0. KDE 5 and musl remain
explicitly unsupported for pointer control; the website record is #rkde.

## Activity
- 2026-09-08 — Complete exact-source native pointer qualification, unchanged main, immutable v1.6.0 publication and all fresh-public lanes; confirm idempotent setup preserves the actual grant and source packages include the companion.
- 2026-09-08: Read back both complete native pointer-runtime results in 34210973568,
  including actual movement and stationary cancellation, all intermediate permission
  states and active-motion backend removal. Retain final-source publication gates.
- 2026-09-08: Run 34209445052 reaches actual engine cursor movement, stationary cancellation, external revocation, graceful stop and crash recovery. KDE backend shutdown emits device removal before disconnect, which production correctly maps to denied; the test had expected failed. Assert the observed removal state and additionally interrupt active native cursor motion, retaining the stationary-pointer and rejected-new-prank checks. Save all intermediate states even if a later gate fails.
- 2026-09-08: Run 34208034861 proves actual settings denial/retry/pending cancellation/grant and retained drafts. The integrated pointer motion identifies Honk300's own full-output, empty-caption layer as an unknown target. Reproduce the production guard failure and recognize only the exact runtime PID and layer identity; foreign lookalikes and protected windows beneath it remain refused. Native observer evidence also stops after a few seconds, so retain timer objects at script scope and require continued native updates before motion.
- 2026-09-08: Integrated run 34205879636 passes both KDE 5 lanes and the window/portal premises on KDE 6, then exposes the settings pointer button being advertised but rejected outside the scroll viewport. Reproduce this through the real SDK accessibility dispatcher, fix bounded ancestor scrolling before activation, and verify the same regression passes while disabled controls remain inert.
- 2026-09-08: Wire the qualified portal into the running Rust owner and native settings without saving configuration drafts. A real engine regression first reproduced an already queued warp surviving capability loss, then passed after immediate queue clearing. Add actual desktop lifecycle qualification; preserve the existing 250 ms and terminal oracles.
- 2026-09-08: Both KDE 6 architectures pass portal/result.json with real native consent, libei readiness, compositor pointer readback, terminal/excessive refusal and explicit cancellation. Preserve the accepted dialog tree instead of overwriting it after dismissal; proceed to runtime and native controls.
- 2026-09-08: Native evidence now separates loaded plugins from failed service authorization. Debian's plasma-workspace file list identifies the missing menu; restore normal service discovery and PipeWire startup order in the private fixture before repeating the actual grant and motion oracle.
- 2026-09-08: Run 34199983732 records empty AvailablePlugins and LoadedPlugins on both KDE 6 desktops. Debian puts screencast/EIS plugins in the separately recommended kwin-common package; include that actual desktop package in the isolated fixture while retaining normal consent and protocol permissions.
- 2026-09-08: The corrected frontend ownership reaches CreateSession. Native KDE reports zkde_screencast_unstable_v1 unavailable. Add the normal isolated KService cache refresh, backend package inventory and compositor plugin/permission diagnostics before repeating the real grant premise; do not disable compositor permission checks.
- 2026-09-08: Keep the static musl archive's pointer control explicitly unsupported with a GNU-build explanation; its static runtime cannot dynamically load the optional input libraries. KDE window support remains separate. Portal qualification now runs independently of the owned-prop test and still blocks the complete native gate on failure.
- 2026-09-08: Run 34196877360 passes all native GUI checks but portal discovery still reaches a previously activated frontend without RemoteDesktop. The replacement frontend exports the interface in its log but had not claimed the shared name. Use its documented --replace option in the private bus and require its exact PID to own the name before requesting permission.
- 2026-09-08: Run 34196518648 proves the fixture checked service ownership before the KDE backend registered. Require both portal owners and the actual RemoteDesktop interface, and initialize D-Bus activation with the private Wayland environment before GTK can auto-start a backend.
- 2026-09-08: Native library compilation and strict checks passed on both architectures, but KDE 6 run 34196079474 disconnected before native consent. Added the library's bounded error detail and exact portal service ownership evidence to identify the failing setup premise; pointer control remains disabled in the runtime.
- 2026-09-08: Recovered the historical board-only spike as the approved implementation task and began the native portal/libei premise. The release gate remains #rkde.
