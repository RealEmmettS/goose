# ADR 0058: Recognize responsive Windows settings startup

Status: accepted for implementation and qualification in v1.11.3.

## Context

The Windows settings launcher verifies and holds read-only leases on its exact settings
executable and accessibility DLL. ADR 0042 requires those leases through GUI initialization.
The implementation uses `WaitForInputIdle` with a ten-second timeout, followed by child
liveness. On the user's installed v1.11.2, direct commands repeatedly open loaded settings
controls that answer a bounded Windows message, then fail the idle wait and kill the window.

A logging-only build of the original predicate records result 258 (`WAIT_TIMEOUT`), Windows
error 0 and the exact child still alive after the wait. The independent observer confirms
the child belongs to that controller and answers `WM_NULL`. Other direct probes and both
application-menu routes pass. The predicate dates to v1.4.0 and is unchanged across the
recent updates. This is evidence of a readiness condition rejecting a responsive child,
not proof of a new scrolling defect or the underlying Windows scheduling cause.

## Decision

Retain one ten-second absolute deadline and the same verified file leases. Poll child
liveness and look only for a visible, positive-client-area `NativeSdkWindowsHost` window
owned by that exact spawned PID. Require a successful bounded `SendMessageTimeoutW`
`WM_NULL` response, with abort-if-hung, blocking and error-on-exit flags. Clip each nonzero
message timeout to the remaining deadline; recheck window ownership and child liveness
after the response. Window title, visibility alone, other processes and UI Automation
are not readiness signals. Keep failed-child kill-and-wait cleanup, with both leases held
until that wait completes. The ten-second bound covers readiness recognition; exceptional
operating-system termination may extend cleanup. Do not abandon a possibly live child and
release its leases merely to impose a second deadline.

The packaged settings executable must normally import `honk_settings_accessibility.dll`.
Its pinned build already links the DLL import library, and the inspected public executable
has no delay imports. Enforce normal linkage and reject nonempty delay-import directories
in the Windows package verifier, along with unsupported bound imports and malformed
descriptor/thunk/name structures. Windows maps load-time imports before application code
creates and dispatches its window, so the response preserves the lease's mapping boundary.
This does not claim that every settings read or accessibility control has finished loading;
the native acceptance test checks those separately.

No public command, protocol, configuration, GUI or rendering contract changes. Existing
release tags and assets remain immutable; publish a new patch after the normal full gates.

## Qualification

Use real owned child-window fixtures for continuously busy but responsive dispatch,
unresponsive and absent/hidden/wrong-class windows, early exit and foreign-process rejection.
Require prompt cleanup in ordinary failure fixtures and retain the existing executable/DLL
lease and tamper tests. Validate normal imports and malformed/truncated/delayed PE cases on both Windows
architectures. Before source freeze, require three consecutive rounds of all four candidate
settings entry routes with loaded native controls, approved icons and unchanged saved state.
Use a temporary shortcut for candidate shell activation, then verify the actual registered
shortcut after the approved public installation.
Complete all required source, candidate, main, publication, public-byte and installed checks.

## References

- [Microsoft: WaitForInputIdle](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-waitforinputidle)
- [Microsoft: SendMessageTimeoutW](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-sendmessagetimeoutw)
- [Microsoft: Load-time dynamic linking](https://learn.microsoft.com/en-us/windows/win32/dlls/load-time-dynamic-linking)
- [ADR 0042: Native settings accessibility](0042-native-settings-accessibility-bridge.md)
