# Native Wayland capability path

Date: 2026-07-12
Decision: [ADR 0021](../adr/0021-native-wayland-capability-strata.md)

## Conclusion

There is no honest, compositor-independent Wayland client path to Honk300's complete X11 feature
set. A normal client can render an overlay, receive input for its own surfaces, and observe output
topology. It cannot portably learn every other surface's geometry, move another client's native
surface, observe the global pointer outside its own surfaces, or inject input without a separate
permission/compositor mechanism.

"Full native Wayland support" must therefore mean a strong portable base plus explicitly selected
capability adapters. It must not mean that every native Wayland compositor can provide every
prank. Honk300's existing layer-shell reduced mode remains the safe portable baseline.

## Upstream facts

- The [Wayland protocol model](https://wayland.freedesktop.org/docs/book/Protocol.html) gives a
  client surface-local pointer events and says clients do not know their surfaces' global
  positions or have access to other clients' surfaces.
- [`ext-foreign-toplevel-list-v1`](https://gitlab.freedesktop.org/wayland/wayland-protocols/-/blob/main/staging/ext-foreign-toplevel-list/ext-foreign-toplevel-list-v1.xml)
  can expose stable mapped-toplevel handles, titles, and app IDs, but is intentionally minimal,
  carries no portable geometry/move requests, is still staging, and may be restricted by
  compositor policy.
- [`pointer-constraints-v1`](https://gitlab.freedesktop.org/wayland/wayland-protocols/-/blob/main/unstable/pointer-constraints/pointer-constraints-unstable-v1.xml)
  constrains a pointer relative to the requesting client's surface. It is not a global
  `SetCursorPos` equivalent, and the compositor is not required to activate a requested lock.
- [`wlr-layer-shell-v1`](https://gitlab.freedesktop.org/wlroots/wlr-protocols/-/blob/master/unstable/wlr-layer-shell-unstable-v1.xml)
  is sufficient for per-output top/overlay surfaces and empty click-through input regions; this is
  the basis of the current native overlay.
- [`wlr-virtual-pointer-v1`](https://gitlab.freedesktop.org/wlroots/wlr-protocols/-/blob/master/unstable/wlr-virtual-pointer-unstable-v1.xml)
  can emulate global relative/absolute pointer events on compositors that advertise and authorize
  it. It is wlroots-specific, unstable, and not a portable Wayland promise.
- The [Remote Desktop portal](https://flatpak.github.io/xdg-desktop-portal/docs/doc-org.freedesktop.portal.RemoteDesktop.html)
  can grant pointer/keyboard control, absolute or relative input, and a libei connection. Starting
  a session normally presents a user approval dialog; even persistent grants can be revoked, and
  restoration tokens are single-use. It is a valid explicit opt-in capability, not a silent
  background-app primitive.
- [KWin scripting](https://develop.kde.org/docs/plasma/kwin/) runs inside the compositor and its
  [API](https://develop.kde.org/docs/plasma/kwin/api/) exposes windows, geometry, cursor position,
  move state, output changes, and window movement. A user-enabled KWin companion can therefore
  bridge near-parity operations that an ordinary client cannot.
- Mutter exposes a compositor-side [`Meta.Window`](https://gnome.pages.gitlab.gnome.org/mutter/meta/class.Window.html)
  model with geometry, move, workspace, monitor, focus, and identity operations. On GNOME, those
  powers belong in a version-compatible Shell extension or compositor component, not a normal
  Wayland client.
- [XWayland](https://wayland.freedesktop.org/docs/book/Xwayland.html) preserves X11 inter-client
  capabilities only for X11 clients. The compositor's built-in X window manager still owns those
  windows, and an X11 client cannot manage native Wayland windows.

## Capability matrix

| Path | Overlay | Global pointer observe/warp | Native window enumerate/geometry | Native window move | Input synthesis | Product posture |
|---|---|---|---|---|---|---|
| Portable Wayland client | Yes, when layer-shell is present | No | Toplevel identity only when compositor exposes staging protocol; no portable geometry | No | No | Current reduced mode; always safe and honest |
| wlroots protocols | Yes | Virtual pointer where advertised/authorized; global observation still compositor-specific | wlr/ext lists vary; geometry is not a general standard | Not by the foreign-toplevel protocol | Virtual pointer; keyboard needs another privileged path | Optional adapter plus compositor capability probe |
| XDG Remote Desktop + libei | Separate from overlay | Yes, after explicit grant | Screen streams, not a general trustworthy window-management model | No | Yes, after explicit grant | Future opt-in; never silent/default and never sufficient alone for ride/collect |
| KDE KWin companion | Yes via normal layer surface | Yes through compositor bridge/portal | Yes | Yes | Portal/libei or compositor bridge | Best near-parity target; user explicitly enables signed/packaged script |
| GNOME Shell companion | Yes via normal layer surface | Compositor/portal dependent | Yes through `Meta.Window` | Yes | Portal/libei | Near-parity but Shell-version coupled; user explicitly enables extension |
| XWayland/X11 backend | Yes | Yes for X11 session path | X11 windows only | X11 windows only | X11 facilities | Existing default when `DISPLAY` exists; never claim native-window parity |
| `/dev/uinput` or privileged daemon | Yes separately | Yes | Still needs compositor-specific window bridge | Still compositor-specific | Yes | Rejected as a default: privilege, security, packaging, and terminal-safety cost |

## Integration path

1. Keep the existing X11-first policy and native layer-shell reduced mode. Continue reporting every
   unavailable capability as `unsupported`, never as silently successful.
2. Add a read-only portable capability probe for `ext-foreign-toplevel-list-v1` only after its
   identity data can preserve terminal exclusions. Do not infer geometry or move support from the
   list protocol.
3. Prototype Remote Desktop/libei behind an explicit opt-in flow. The portal grant is suitable for
   cursor actions only after status/UI makes its scope clear; no background prompt loop.
4. Build KDE first as the near-parity compositor adapter: a small user-enabled KWin script exposes
   a same-user authenticated bridge with exact window identity, geometry, move-state, and bounded
   move commands. Keep terminal filtering on both sides of the bridge.
5. Build GNOME separately as a Shell-versioned extension using `Meta.Window`; do not pretend the
   KDE bridge is portable to Mutter.
6. Treat Sway, Hyprland, and other wlroots compositors as individual IPC/protocol adapters. Require
   capability discovery and integration tests per supported compositor.
7. Preserve engine capability traits and status semantics. Adapters supply existing pointer,
   foreign-window, collect, and presence capabilities; no OS object enters `honk-engine`.

## Release boundary

v0.3.3 does not add a portal grant, compositor plugin, privileged helper, or new configuration
schema. It keeps the already-tested reduced native mode and records the implementation path as
follow-up work. This avoids mixing a security-sensitive Linux expansion into the macOS
qualification release while still closing the research decision.
