# ADR 0029 — Windows Lifecycle And Terminal Hardening

- Status: Accepted (2026-07-17)
- Relates to: ADR 0004 (terminal protection), ADR 0013 (lifecycle packaging), ADR 0015
  (platform safety), ADR 0018 (atomic publication), ADR 0023 (exact-tag updates), and ADR 0025
  (post-release Alienware verification).
- Supersedes: no prior ADR. It narrows Windows implementation details while preserving the
  existing package shapes, stable GUIDs, release matrix, and immutable-tag policy.

## Context

Exact public v1.0.2 bytes were exercised on a physical Alienware Windows 11 x64 laptop while an
old v0.3.1 Global MSI remained installed as an operator-approved update fixture. The compositor,
three aliases, local IPC, graceful restart, audio initialization, TUI restoration, Corporate
install/repair/ordinary upgrade/uninstall, user-content preservation, and backup-first purge all
passed. The hardware pass also reproduced several Windows lifecycle faults that hosted producer
gates had not reached.

Windows PowerShell treats arguments appended after a `-Command` script as more command text, not
as parameters for the script's `param` block. Both v0.3.1 and exact portable v1.0.2 therefore
failed before update discovery. GitHub responses also reached Windows PowerShell 5.1 as bytes on
this host. A supported Global and Corporate installation could coexist, but registry-first source
detection selected the other installation and Corporate Add/Remove Programs registration existed
under a different hive than the CLI expected. A deferred helper treated an already-exited parent
as failure. Finally, forcing a Corporate major-upgrade failure under the shared early
`RemoveExistingProducts` schedule could leave an old ProductCode as an orphaned component client
after retry.

The live window audit identified this Codex desktop process as a generic Chromium window titled
`ChatGPT`; VS Code uses the same generic class. Windows therefore needed explicit conservative
title coverage to match the existing macOS protection for Codex and VS Code. Linux needed the
equivalent application-token coverage.

## Decision

### Windows update discovery and download

The updater keeps every URI and output path outside PowerShell command source. It passes those
values through child-only environment variables, selects non-interactive stop-on-error execution,
suppresses progress output, and decodes a byte-array response as UTF-8 before parsing. The exact
manifest still selects an immutable tag and the downloaded payload remains size/hash pinned.

After a direct Global-MSI update verifies the installed executable's exact path and version, the
helper may refresh an existing PowerShell-bootstrap receipt. It does nothing unless the receipt
and its parent are regular non-reparse objects and the JSON proves the exact
`honk300.install.v1` schema, `powershell-global-msi` channel, `windows-global-msi` layout, complete
bootstrap shape, and case-insensitive managed install root. A matching receipt advances version,
tag, commit, target, artifact name, and artifact SHA through a create-new sibling plus atomic
replacement. Missing, malformed, foreign, mismatched, directory, and reparse receipts remain
untouched. Delegated bootstrap updates continue to own their own receipt transaction.

### Coexisting Windows installations and deferred helpers

On Windows, the source marker adjacent to the running executable wins over registry fallback.
This binds `update` and `uninstall` to the invoked Global or Corporate installation when both are
present. Corporate uninstall discovery searches the current-user hive first and then the machine
hive, but every candidate must still pass the existing exact product, publisher, ProductCode,
Windows-Installer, install-root, and running-executable validation. Global discovery remains
machine-only.

A deferred helper explicitly treats a missing/already-exited parent as successful completion of
the wait. Real lookup/wait errors remain stop-on-error failures.

### Corporate major-upgrade retry semantics

Global MSI retains `RemoveExistingProducts` at `afterInstallInitialize`. Corporate MSI alone
moves it to `afterInstallFinalize`. The per-user package therefore commits the new installation
before attempting to remove the old product in a separate transaction. If the new install fails
before finalization, the old product and its component clients remain untouched. If later removal
of the old product fails, the new product stays installed and Windows Installer rolls back only
the old-product removal. Stable component GUIDs and feature paths remain mandatory for this late
schedule.

### Integrated terminal protection

Windows classifies Visual Studio Code, Codex, and the observed ChatGPT-titled Codex desktop window
as protected terminal/development surfaces. Linux classifies the `code` and `codex` application
tokens the same way. They may still be visually overlaid, but cannot be focus, typing, drag,
ride, move, or collect targets. Ordinary non-terminal applications remain eligible.

## Consequences

- Windows update discovery works in native Windows PowerShell without interpolating network or
  filesystem values into command source.
- A direct MSI update no longer leaves a validated bootstrap receipt informationally stale, and
  the refresh does not broaden ownership over foreign state.
- Supported Global and Corporate packages can coexist without one executable borrowing the
  other's lifecycle identity.
- Corporate forced-failure retry favors a usable old product before commit and a usable new
  product after commit instead of involving both products in one rollback transaction.
- Codex and VS Code receive conservative cross-platform terminal protection. This may exclude an
  entire development-tool window even when its integrated terminal is hidden; safety is preferred
  over treating that surface as a prank target.
- v1.0.0, v1.0.1, and v1.0.2 tags and assets remain immutable. These changes ship only through
  the forward v1.0.3 release gate.

## Verification

- Red/green Rust tests pin child-only web-request values, PowerShell byte decoding, adjacent-marker
  precedence, Corporate hive order, successful already-exited-parent waiting, guarded receipt
  mutation, and Windows/Linux terminal classification.
- A real Windows PowerShell receipt test uses a path containing an apostrophe, proves exact owned
  metadata advances, and proves malformed and foreign bytes remain unchanged.
- Exact public v1.0.2 network discovery failure was reproduced from both old installed and current
  portable binaries; rebuilt source successfully read the live manifest and downloaded an exact
  artifact to a quoted path with matching SHA-256.
- A modified Corporate MSI injected a post-file-install failure. The new schedule preserved the
  old v0.2.1 executable byte-for-byte; clean retry and uninstall then left no sampled component
  clients or old product registration.
- The complete v1.0.3 readiness gate remains recorded in
  `docs/readiness/v1.0.3-readiness.md`; unexecuted elevation or unavailable hardware observations
  remain explicit rather than inferred.
