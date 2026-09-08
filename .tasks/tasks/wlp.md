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
Active: checking the real library and KDE 6 grant before enabling a runtime capability.
KDE 5 has separate window evidence; no EIS or portal control is claimed for it.

## Activity
- 2026-09-08: Native library compilation and strict checks passed on both architectures, but KDE 6 run 34196079474 disconnected before native consent. Added the library's bounded error detail and exact portal service ownership evidence to identify the failing setup premise; pointer control remains disabled in the runtime.
- 2026-09-08: Recovered the historical board-only spike as the approved implementation task and began the native portal/libei premise. The release gate remains #rkde.
