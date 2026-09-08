# ADR 0047: independently qualified Hyprland observations

Status: accepted for implementation; public qualification pending.

The approved refinement requires a separate Hyprland milestone after KDE and Sway.
Native run [34222607423](https://github.com/RealEmmettS/goose/actions/runs/34222607423)
qualifies Hyprland 0.53.3 and 0.55.2 on native x64 and ARM64. Its privately launched
system compositor supplies the kernel-authenticated socket, real fixture identities,
six-pixel movement, fullscreen transitions and disappearance. The unrelated protected
fixture retains its position. This premise is separate from a production adapter.

The public IPC inventory has no authoritative active user-drag state. Its ability
to dispatch a move therefore does not authorize production movement or animated
owned-prop positioning. This adapter supplies explicit window observations and
fullscreen awareness only. Pointer observation/control, window rides, movement and
DND remain unavailable; no general command dispatcher exists in the Rust boundary.

The transport pins the system-owned Hyprland process, its loaded executable, private
runtime hierarchy and socket identity. Each reply must end within a single bounded
deadline and byte limit. A complete snapshot brackets clients with matching active
monitor/workspace observations under one total deadline. Unknown or replaced peers
receive no query. Shared bounded Unix socket primitives retain Sway's own protocol
and authentication policy; evidence from either adapter cannot qualify the other.

Optional process/title metadata does not erase known fullscreen presence. Hidden,
unmapped and other-workspace clients remain distinct from visible fullscreen windows.
The two qualified versions retain their separate visibility semantics.

Production setup/removal, retained worker ownership, configuration and GUI drafts,
lost authority, lifecycle recovery and every architecture/package gate still need
actual native qualification before a new immutable public release can claim support.
