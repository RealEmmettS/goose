# ADR 0044: explicit and revocable native Wayland adapters

Date: 2026-09-08
Status: accepted design; native window/runtime/setup and portal premises qualified, integrated pointer lifecycle in progress

## Context

ADRs 0021 and 0041 require KDE-first window integration and separately granted pointer
control. The real isolated KWin 5.27/6.3 script probe now passes native identity, bounded
movement, terminal exclusion, stale/excessive movement refusal and disable. This permits
implementation of the Rust boundary, not a public capability claim.

## Decision

- Keep every integration default-off. Guided setup installs only Honk300's named companion,
  records the user's explicit choice, and provides status and removal. A desktop environment
  hint never installs, activates or grants an adapter. Ordinary native Wayland remains reduced.
- Keep configuration, lifecycle and actions in Rust. The KWin script supplies bounded fresh
  observations and executes at most one narrowly typed geometry change per exchange. It has
  no focus, keyboard, shell, arbitrary-code, window-close or unbounded-action operation.
- Use a session-bus service owned only while the authorized Rust runtime is alive. The Rust
  service checks the sender against the live unique owner of `org.kde.KWin` and verifies the
  compositor's Unix user. Name replacement, invalid frames and expiry discard pending actions.
  The script pins its bridge owner and stops on loss/replacement; delayed responses cannot act.
- Bound snapshots to 64 windows and 64 KiB, one pending action, a 4 KiB response and a 250 ms
  maximum observation age. Validate schema, unique identities, geometry, target process/app,
  current desktop/activity, terminal exclusion and the current window again before moving.
  Movement stays within the target's real output and within 24 logical pixels per exchange.
- Preserve exact native floating-point geometry through JSON decoding and replies.
  The captured native delivery regression proves that a one-ULP parse change can
  invalidate the script's exact stale-position check. Enable float roundtrips at
  the Rust boundary; never add tolerance to the native identity/geometry guard.
- Keep terminal protection at both boundaries. Blank, oversized or ambiguous identities do
  not authorize operations. Codex, ChatGPT and Visual Studio Code remain protected.
  The pointer guard recognizes only the running process's own immovable `honk300`
  layer surface, whose native KWin caption is empty. Its exact compositor-reported PID
  is required; a foreign process using the same application name receives no exemption.
  Every underlying window still undergoes the full terminal and unknown-identity checks.
- Report window observation, foreign movement, owned-prop positioning, pointer observation,
  pointer control, fullscreen and DND individually. KWin window evidence supplies no portal
  grant. KWin fullscreen evidence supplies no DND claim. Plasma 5's client list supplies no
  stacking-order authority; do not infer pointer occlusion from it.
- Request portal pointer access only through the desktop's explicit grant flow. Use only
  capabilities actually supplied by the granted session, cancel on close/device removal,
  and require adequate live terminal-target exclusion before enabling cursor mischief.
  No uinput helper or global input permission is installed implicitly.
- GNOME, Sway and Hyprland remain separate implementation and native desktop gates. No KDE
  result promotes another adapter. Pi hardware performance remains unverified.

## Qualification

The pointer premise uses optional system `liboeffis.so.1` for the native RemoteDesktop
grant and `libei.so.1` for the granted sender. Rust retains both contexts and bounded
device references. Only absolute pointer motion is bound; no keyboard/button/touch API
is exposed. Missing libraries or unsupported loading remains unsupported. Devices must
resume, provide an unambiguous virtual region and pass a newly authenticated KWin frame
for every bounded movement. Paused/removed devices end the session and need a new grant.
Native grant and actual pointer motion passed on both KDE 6 architectures before runtime
wiring. CLI and Native SDK request/cancel operations now reach the running Rust owner,
which polls the retained session, exposes pending/granted/ended state and cancels on
companion expiry, permission/device loss, explicit removal or shutdown. No grant is
restored after restart. Each engine command rechecks the current terminal-safe path;
capability loss clears already queued engine cursor commands immediately.
The [liboeffis API](https://libinput.pages.freedesktop.org/libei/api/group__liboeffis.html)
and [libei sender API](https://libinput.pages.freedesktop.org/libei/api/group__libei-sender.html)
define the context, descriptor and device ownership used at this boundary.

Explicit setup stores the exact companion bytes and a random registration identity in a
bounded private user record beside the existing configuration. Permission state is separate
from an unsaved settings draft. Updated script bytes require explicit setup again; the
record retains its registration identity so crash recovery can retire that exact stopped
script. The runtime loads sealed bytes, removes its registration on graceful exit, and
rechecks the saved grant while running. Removal deletes only that record and registration.
Normal uninstall retains user data as before; purge follows the existing user-data policy.
No global KWin package, autostart entry or foreign script is installed or removed.

Owned native Wayland props set a per-window application token and retain the existing
private process transport. KWin placement requires that token, the unreaped child PID,
matching native dimensions and a fresh eligible window. The same script-side movement
bounds apply. The prop retains its own input region and text ownership; permission loss
restores input and cancels pending motion while preserving retained notes.

Run the same script and Rust bridge against actual native KDE 5/6 fixture windows. Cover
untrusted D-Bus peers, changed titles/processes/geometry, vanished windows, window overload,
display and desktop changes, delayed replies, owner loss/replacement and explicit disable.
Then connect the real goose runtime, native setup/status/removal and owned prop operations.
Preserve all existing architecture, package, signing, candidate/main and public-byte gates.

The first script premise passed in
[run 34189071395](https://github.com/RealEmmettS/goose/actions/runs/34189071395).
Upstream [KWin scripting APIs](https://develop.kde.org/docs/plasma/kwin/api/) and
[Remote Desktop portal interface](https://github.com/flatpak/xdg-desktop-portal/blob/main/data/org.freedesktop.portal.RemoteDesktop.xml)
define the distinct authority boundaries. Run 34203908233 passes both KDE generations and
architectures, including actual owned-note movement and the separate KDE 6 native portal
premise. The integrated pointer runtime/settings lifecycle and final release qualification
remain open until their production desktop and package evidence passes.
