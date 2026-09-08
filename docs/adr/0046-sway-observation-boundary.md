# ADR 0046: explicit Sway observations with unsupported action capabilities

Date: 2026-09-08
Status: accepted and implemented; complete native runtime evidence passed

## Evidence and decision

The independent native probe in run 34212710026 passes on Sway 1.9 and 1.10.1,
on x64 and ARM64. Its private unprivileged compositor supplies actual Unix socket
credentials, per-window process/application identities, floating geometry, fullscreen
changes and disappearance. A fixture-owned window moves exactly six pixels while
the protected fixture remains unchanged.

That raw movement operation does not satisfy the full delivery safety contract.
The public [Sway IPC schema](https://github.com/swaywm/sway/blob/master/sway/sway-ipc.7.scd)
provides no authoritative current drag state. The compositor retains that state in
its internal seat operation, which is not exported in GET_TREE or GET_SEATS. A
focused-window guess, geometry change heuristic or modified user binding cannot
stand in for this evidence. Do not advertise animated delivery or foreign-window
actions through this adapter. Pointer observation/control and DND remain unsupported.

Implement explicit, revocable window/fullscreen observation through the running
Rust owner. The production transport exposes only GET_VERSION and GET_TREE. It has
no arbitrary command, focus, movement, input, configuration or startup operation.
Normal compositor placement of native owned props keeps its existing behavior.

Accept only the independently qualified compositor versions. Require the private
same-user runtime socket, actual peer credentials, a system-owned non-writable Sway
executable, a stable socket inode and bounded requests. Do not follow a replacement
socket or reconnect into a changed compositor owner automatically. The runtime must
report unknown/unavailable observation separately from an observed non-fullscreen
desktop, keep polling outside the presentation loop, and stop its exact worker on
removal or shutdown. No foreign Sway configuration or binding is installed or edited.

## Qualification

The complete [native run 34218062301](https://github.com/RealEmmettS/goose/actions/runs/34218062301)
passes all four desktop lanes, including production transport, setup/removal through
native accessible controls, GUI drafts, unsupported actions, socket replacement,
fullscreen engine manners/config changes and graceful/crash recovery. Native split
fullscreen and powered-off output scenarios exercise the production decoder.
The final-source release repeats this separate desktop workflow as a mandatory gate.
The release follows KDE publication, with separate native versions/architectures,
complete same-source candidate/main gates, immutable artifacts and fresh public-byte
verification. Other compositors and physical Pi behavior inherit no Sway evidence.
