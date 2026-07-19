TT;DR: Repair `goose update` on ordinary unelevated Windows sessions by keeping owner-cleanup discovery read-only when the protected receipt already records the correct state.

## Why

The installed public v1.2.6 PowerShell-owned Global layout fails immediately after
`honk300: checking for updates...` with Windows error 5. The user is an administrator, but UAC
correctly runs the terminal without an elevated token. The updater discovers that the retained
Global MSI registration is compatible with the PowerShell owner and finds no conflicting owner,
then unnecessarily replaces the already-current receipt under `C:\Program Files\honk300`.

## Plan

Make protected cleanup-state persistence idempotent so read-only discovery performs no write when
the receipt already records `inactive_releases_retained` (or the same pending state). Preserve the
existing fail-closed conflict journal, exact registration checks, immutable-slot transaction, and
post-install verification. Recheck the v1.3.0 controller to app launcher to hidden runtime change:
the receipt must still bind `honk300-app.exe`, and an update must validate the new launcher hash
alongside all three public aliases.

## Impact

Intended: a normal non-elevated PowerShell can discover and download an update, then request UAC
only at the existing installer/coordinator boundary. Risks are skipping a required cleanup-state
transition, weakening conflicting-owner retirement, or accepting a release whose app launcher does
not match its protected receipt.

## Acceptance

An already-current protected receipt is byte-stable during cleanup discovery, while actual state
changes still use the protected atomic write. Focused updater and packaging tests pass, the live
installed ownership evidence remains authoritative, and the handoff records that an immutable
older updater cannot repair itself before a newer fixed release is installed through the official
versionless bootstrap.

## Status

Done in source and assigned to release task `#r131`. Published v1.2.6 and v1.3.0 remain immutable,
so the installed v1.2.6 updater cannot repair this pre-download failure in place. After v1.3.1 is
published, one official versionless bootstrap install will cross the broken updater; a local
same-version binary must not replace signed public bytes or forge a release receipt.

## Activity

- 2026-07-19 — reproduced `goose update --json` exit 1 with `PermissionDenied` before download.
  The protected receipt is a valid `honk300.install.v2` PowerShell owner at v1.2.6; its active slot,
  aliases, app-launcher SHA-256, target, and immutable artifact identity are present. The matching
  machine-wide Global MSI registration is intentionally compatible with PowerShell ownership, so
  there is no conflicting owner and no cleanup journal. `refresh_windows_owner_cleanup_journal`
  nevertheless rewrites the unchanged receipt under Program Files, which fails in an ordinary UAC
  session. The most recent v1.3.0 Codex task and ADR 0036 were rechecked: the detached launcher
  contract is represented in the receipt and is not the immediate failure source. (agent: codex)
- 2026-07-19 — made cleanup-state persistence return without a filesystem operation when the
  protected receipt already records the requested state. The regression locks the receipt
  read-only, proves the identical-state call preserves its exact bytes and leaves no transaction
  file, then proves a real transition to `cleanup_pending` still persists. Formatting, 105
  Windows CLI/install/update tests, strict workspace all-target/all-feature Clippy, 42 Windows
  packaging/installer/release contracts (two platform skips), and both lifecycle PowerShell
  parsers pass. (agent: codex)
- 2026-07-19 — temporarily exercised the exact fixed discovery function against the live
  unelevated `C:\Program Files\honk300` installation. It found the PowerShell/MSI-compatible owner
  with no conflict and returned successfully; the receipt SHA-256 remained
  `8142140243AB38F1A590DB138F589A9CF94A0E591021737F0570CE36E205F9F1`, its write time was unchanged,
  and the root file set was byte-for-byte stable. Removed the temporary live-only test afterward.
  Existing ADRs 0031, 0034, and 0036 already define the preserved behavior, so no new architecture
  decision is needed. (agent: codex)
