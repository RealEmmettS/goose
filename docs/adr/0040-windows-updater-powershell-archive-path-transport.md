# ADR 0040 — Windows Updater PowerShell Archive-Path Transport

- Status: Accepted (2026-07-23, v1.3.7 immutable fix-forward)
- Relates to: ADR 0029 (Windows lifecycle), ADR 0031 (slot self-update), ADR 0034
  (command-first installation)
- Supersedes: none; this preserves the existing verification and lifecycle contract

## Context

After v1.3.6 passed candidate, same-SHA main, atomic publication, all eight public-byte lanes, and
production verification, the official installed v1.3.5 updater selected it through the public
channel. The generated one-line Windows PowerShell coordinator failed before installer launch:
the quote-adjacent `'\` archive-path literal arrived through `powershell.exe -Command` as an empty
`String.Replace` search value. PowerShell rejected `Replace('','/')`.

The failure was safe. No MSI launched, the active version remained v1.3.5, and the protected
receipt bytes/hash/timestamp remained unchanged. Existing syntax-only tests accepted both source
forms and therefore did not exercise the native argument-transport boundary.

## Decision

- Generated Windows updater PowerShell represents a backslash as `[char]92` when normalizing and
  rejecting lifecycle ZIP entry paths.
- The validation still normalizes only for segment inspection and separately rejects any entry
  containing a backslash, colon, empty segment set, `.` segment, or `..` segment.
- The verified archive, exact single `honk300.exe` entry, pinned stream, lease-holder, installer,
  post-install identity, cleanup, and rollback contracts do not change.
- Tests pin the exact generated expression, prohibit the fragile quote-adjacent form, syntax-parse
  the complete generated coordinator, and execute the replacement/containment expression through
  real Windows PowerShell argument transport.
- Published v1.3.6 assets and tags remain immutable. The correction ships only as v1.3.7.

## Consequences

- Windows self-update can prepare its verified lifecycle lease without an empty-search exception.
- Archive traversal rejection is neither relaxed nor delegated to an ambient extraction default.
- The PowerShell transport boundary has executable coverage in addition to parser coverage.
- v1.3.6 remains an honest published release whose installed-updater failure and safe unchanged
  state are retained in readiness evidence.

## Verification

- Run the focused generated-coordinator and native PowerShell transport regressions.
- Repeat the complete local/candidate/main/public-byte release gates for v1.3.7.
- Use public bytes on the official Windows machine, verify receipt/provenance/runtime identity,
  and repeat isolated-config mud acceptance without touching the real user config.
