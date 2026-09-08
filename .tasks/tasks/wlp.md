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
- [ ] Actual portal consent, granted libei devices, motion and cancellation are observed.
- [ ] Runtime and native settings use the qualified path without changing saved drafts.
- [ ] Fresh target/terminal checks, device pause/removal and disconnects fail closed.

## Status
Run 34198830686 now reaches the actual frontend and KDE backend. KDE denies CreateSession
because its backend cannot see the required screencast protocol. Refresh the private
desktop's standard KService registry and record loaded/available compositor plugins;
retain normal interface permissions and require the genuine consent dialog.
Active: checking the real library and KDE 6 grant before enabling a runtime capability.
KDE 5 has separate window evidence; no EIS or portal control is claimed for it.

## Activity
- 2026-09-08: The corrected frontend ownership reaches CreateSession. Native KDE reports zkde_screencast_unstable_v1 unavailable. Add the normal isolated KService cache refresh, backend package inventory and compositor plugin/permission diagnostics before repeating the real grant premise; do not disable compositor permission checks.
- 2026-09-08: Keep the static musl archive's pointer control explicitly unsupported with a GNU-build explanation; its static runtime cannot dynamically load the optional input libraries. KDE window support remains separate. Portal qualification now runs independently of the owned-prop test and still blocks the complete native gate on failure.
- 2026-09-08: Run 34196877360 passes all native GUI checks but portal discovery still reaches a previously activated frontend without RemoteDesktop. The replacement frontend exports the interface in its log but had not claimed the shared name. Use its documented --replace option in the private bus and require its exact PID to own the name before requesting permission.
- 2026-09-08: Run 34196518648 proves the fixture checked service ownership before the KDE backend registered. Require both portal owners and the actual RemoteDesktop interface, and initialize D-Bus activation with the private Wayland environment before GTK can auto-start a backend.
- 2026-09-08: Native library compilation and strict checks passed on both architectures, but KDE 6 run 34196079474 disconnected before native consent. Added the library's bounded error detail and exact portal service ownership evidence to identify the failing setup premise; pointer control remains disabled in the runtime.
- 2026-09-08: Recovered the historical board-only spike as the approved implementation task and began the native portal/libei premise. The release gate remains #rkde.
