# ADR 0035 — Hosted Windows Tray Registration Qualification Boundary

- Status: Accepted (2026-07-18)
- Relates to: ADR 0026 (hosted Windows ARM64 evidence), ADR 0028 (shared tray contract), ADR 0030
  (native Windows tray owner), and ADR 0033 (disposable-desktop qualification).
- Supersedes: no product behavior. It narrows only the explicitly opted-in hosted x64
  notification-area observation boundary used by release qualification.

## Context

The immutable v1.2.3 tag passed its complete candidate matrix and ordinary same-SHA Windows x64
CI. The same exact x64 portable ZIP then failed the atomic publication gate before draft release
creation because `Shell_NotifyIconGetRect` returned `S_FALSE` for ten seconds. Honk300's exact
tray-owner window existed, the runtime remained healthy, and the ZIP, executable, launcher, PE,
and version identities all matched. Re-running that hosted job reproduced the shell observation
failure while the strict ordinary Windows lane and candidate lane had already observed the same
product behavior successfully.

Windows Server notification-area materialization is external state owned by Explorer. A release
gate must still distinguish an unobservable hosted shell from a missing or broken Honk300 tray
implementation, and it must not turn a broad retry into evidence.

## Decision

Ordinary Windows x64 CI remains strict. It does not pass an unobservable-tray flag and must prove
the exact fixed-GUID rectangle, exact deletion, `TaskbarCreated` re-registration, and restored
rectangle.

Only a disposable GitHub-hosted x64 workflow that explicitly passes
`AllowUnobservableTrayRecoveryHost` may record initial registration as unobservable, and only
when all of these conditions hold:

1. Honk300's exact `honk300_status_tray_owner` window with accessible name `Honk300 controls`
   exists.
2. The exact Honk300 fixed-GUID rectangle remains unavailable for the full bounded observation
   period.
3. An independent stock-icon probe performs fixed-GUID add, rectangle lookup, delete, re-add,
   and rectangle lookup through the same shell; it must also fail. If it succeeds, Honk300 fails.
4. The evidence records the owner, GUID, accessible name, HRESULT, poll count, runtime output,
   ordinary shell probe result, and independent fixed-GUID probe result.

This state is `registration-unobservable`, not `unavailable`. Therefore the remaining
process-owned native tray menu Quit and modal-menu force tests still execute through the runtime's
exact owner window and registered private message. PE identity, compositor, single-instance,
walk-in, graceful walk-off, immediate force shutdown, and artifact immutability checks also remain
mandatory. The waiver does not establish visible tray placement on that host; strict CI provides
that evidence for the same source change.

## Consequences

- A hosted Explorer failure cannot silently excuse a product failure: an independently created
  fixed-GUID icon must be equally unobservable.
- The narrow branch is evidence-producing and fail-closed. It is not enabled by product code or
  ordinary local/CI execution.
- v1.2.3 remains immutable and unpublished. The correction ships forward as v1.2.4 after a new
  candidate, same-SHA main, atomic publication, and fresh-public-byte matrix.
