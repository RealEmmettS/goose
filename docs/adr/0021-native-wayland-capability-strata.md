# 0021 — Native Wayland Capability Strata

## Status

Accepted. Extends ADR 0011's reduced-mode contract without changing v0.3.3 runtime behavior.

## Context

ADR 0011 correctly states that a normal native Wayland client cannot portably move other windows,
warp the global cursor, or synthesize input. The active research card asked whether newer core,
staging, wlroots, portal, KDE, or GNOME facilities now make honest full support possible.

The 2026-07-12 upstream review is recorded in
[`docs/research/native-wayland-capability-path.md`](../research/native-wayland-capability-path.md).
The available mechanisms belong to different trust and portability layers: normal client
protocols, explicit desktop portals, compositor-specific protocols, and compositor-hosted
extensions. No one layer supplies all Honk300 capabilities everywhere.

## Decision

- Define native Wayland support as capability strata, not a single full-parity boolean.
- Retain the visible layer-shell overlay, IPC, audio, honk, wander, and mud as the portable base.
- Keep cursor warp/input synthesis and foreign-window geometry/move `unsupported` unless a probed,
  explicit adapter proves the individual capability.
- Permit a future user-approved XDG Remote Desktop/libei adapter for cursor/input capability. It
  must be opt-in, revocable, status-visible, and must never be treated as window-management proof.
- Prefer a user-enabled KWin script as the first near-parity foreign-window adapter. Treat a GNOME
  Shell extension and wlroots compositor IPC adapters as separate implementations with separate
  compatibility/testing claims.
- Do not ship a privileged `/dev/uinput` helper by default. It does not solve native window
  management and creates disproportionate privilege, packaging, and terminal-safety risk.
- Preserve the platform-free engine contract. Compositor adapters translate into the existing
  capability traits and never expose Wayland/compositor objects to `honk-engine`.
- Terminal-window protection remains absolute at both the client and adapter boundary.

## Consequences

- Documentation no longer describes native full parity as one future protocol away.
- The existing reduced mode remains correct and can improve incrementally without false claims.
- Near parity is feasible on selected desktops only with explicit companion integration.
- Linux packaging and support claims must identify the exact compositor/adapter and tested version.
- This decision adds no v0.3.3 dependency, setting, permission prompt, or release artifact.

## Verification

- Keep native reduced-mode visible-overlay and explicit unsupported-status smoke gates.
- Add protocol/global discovery tests before any portable observation work.
- Require same-user authentication, terminal-negative tests, revocation/failure recovery, and live
  compositor evidence for each future adapter.
- Never promote a compositor adapter from experimental until its supported version matrix and
  uninstall/cleanup path are proven.
