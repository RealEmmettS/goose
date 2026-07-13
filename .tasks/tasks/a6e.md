TT;DR: Full native Wayland parity is not portable; retain reduced mode and add explicit portal or compositor adapters as separate future capabilities.

## Status

Done on 2026-07-12. ADR 0021 and the linked research report record the conclusion and integration
path. No v0.3.3 runtime behavior was expanded.

## Evidence

- Reviewed current core Wayland, wayland-protocols staging/unstable, wlroots, XDG Remote Desktop
  and libei, KDE KWin, GNOME Mutter, and XWayland sources.
- Built a matrix for portable, wlroots, KDE, GNOME, portal, XWayland, and privileged-helper paths.
- Separated overlay, pointer, input synthesis, toplevel identity, geometry, and move capabilities.
- Recorded a layered adapter architecture and split implementation into visible follow-up cards.

## Activity

- 2026-07-12 20:16 - Completed the upstream capability audit, accepted ADR 0021, documented the
  matrix and release boundary, and queued distinct portable/portal, KDE, and GNOME/wlroots follow-up
  implementations (agent: codex).
