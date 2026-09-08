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
monitor/workspace observations in one fixed read-only batch under one total deadline.
Both qualified upstream implementations separate the three JSON replies with whitespace;
the decoder requires exactly three complete values and rejects trailing data. Unknown or replaced peers
receive no query. Shared bounded Unix socket primitives retain Sway's own protocol
and authentication policy; evidence from either adapter cannot qualify the other.

Optional process/title metadata does not erase known fullscreen presence. Hidden,
unmapped and other-workspace clients remain distinct from visible fullscreen windows.
The two qualified versions retain their separate visibility semantics.

Native run 34226478369 shows the older compositor stalling every query while its
fullscreen target still has the old GTK allocation. Retain the single authenticated
connection owner in a worker; each late read withdraws observations immediately.
Only a timeout may retry another bounded read against that same pinned process,
socket inode and executable. Permission loss, a changed owner, disconnection or
malformed data ends the worker. The presentation loop does not wait for recovery,
and no stale frame or pending action is retained. Shared worker ownership preserves
Sway's existing terminal-error policy; Hyprland's timeout recovery has separate native evidence.

The retained production worker passes all four native lanes in run 34227542105;
Sway separately passes all four shared-worker regression lanes in run 34227540869.

The complete actual runtime and Native SDK lifecycle passes all four desktops in
run 34231759048, including repeated setup with the same kernel thread, live consent
removal, fullscreen/config manners, retained drafts, socket replacement and graceful
and crash recovery. Rendered prop/settings review and final-source publication
gates remain tracked in the release readiness record.
