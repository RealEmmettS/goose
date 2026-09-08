# ADR 0044: explicit and revocable native Wayland adapters

Date: 2026-09-08
Status: accepted design; native KWin script premise qualified, runtime/setup work in progress

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
- Keep terminal protection at both boundaries. Blank, oversized or ambiguous identities do
  not authorize operations. Codex, ChatGPT and Visual Studio Code remain protected.
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

Run the same script and Rust bridge against actual native KDE 5/6 fixture windows. Cover
untrusted D-Bus peers, changed titles/processes/geometry, vanished windows, window overload,
display and desktop changes, delayed replies, owner loss/replacement and explicit disable.
Then connect the real goose runtime, native setup/status/removal and owned prop operations.
Preserve all existing architecture, package, signing, candidate/main and public-byte gates.

The first script premise passed in
[run 34189071395](https://github.com/RealEmmettS/goose/actions/runs/34189071395).
Upstream [KWin scripting APIs](https://develop.kde.org/docs/plasma/kwin/api/) and
[Remote Desktop portal interface](https://github.com/flatpak/xdg-desktop-portal/blob/main/data/org.freedesktop.portal.RemoteDesktop.xml)
define the distinct authority boundaries. Native runtime and portal evidence remain open.
