# ADR 0038 — Cross-Platform Tray Update Helper

- Status: Accepted (2026-07-21)
- Relates to: ADRs 0028, 0030, 0031, 0033, 0034, 0036, and 0037.
- Supersedes: only the Configure/Quit-only limits in ADRs 0028 and 0030. Their shared icon,
  accessible naming, existing-TUI Configure route, graceful Quit, unavailable-host behavior, and
  absence of a native preferences model remain accepted.

## Context

Honk300's native control surfaces expose configuration and graceful shutdown, but a non-technical
user still has to open a terminal and type `honk300 update`. The updater is intentionally
synchronous and provenance-preserving: it may ask the running goose to walk offscreen, retain the
runtime singleton, invoke a native installer or elevation boundary, activate a verified release,
and preserve or roll back the authoritative installation owner.

Running that transaction on the native menu/event thread would deadlock. The updater's graceful
Stop request must be serviced by the same runtime loop that owns the menu, IPC server, overlay,
and final walk-off. Silent background updates would also hide package-manager/elevation prompts
and actionable failures. Terminal applications expose user-controlled close/hold policies, so a
portable promise to close an updater window automatically would be false.

## Decision

The shared native command set is `Configure`, `Update`, and `Quit`. Every supported surface shows
an always-enabled **Update Honk300…** action. Opening the menu performs no network request; the
click alone launches an updater terminal from the exact running executable:

- Windows calls `CreateProcessW` with `CREATE_NEW_CONSOLE` and the literal private argument
  `__control-surface-update`.
- macOS opens a signed, sealed `Update Honk300.command` resource whose only action is to execute
  the exact bundle sibling with that private argument.
- Linux reuses the finite terminal-launcher argument-vector table. The executable and private
  argument are separate literal arguments; no `sh -c` or interpolated command is permitted.

The private command is hidden from public help and runs out of process. It calls the same updater
transaction through a typed internal outcome; the public `update [--json]` formatter and its
single-object stdout contract do not change. A distinct per-user Windows named mutex or owner-only
Unix advisory lock serializes helper transactions and is released by the kernel after a crash.

On verified activation, the helper resolves the newly receipt-owned executable instead of using
PATH or its still-mapped old image, starts the platform's detached app route, and requires bounded
IPC readiness. On failure or cancelled elevation it first accepts an already-running runtime,
then tries the post-transaction receipt owner, and finally may retry the retained pre-transaction
receipt owner. It never reports recovery unless IPC proves a running instance.

Successful activation clears the terminal and shows only:

```text
Update complete.
Honk300 has restarted.

+------------------------------------------+
|              HONK! ALL DONE              |
|     You may now close this window.       |
+------------------------------------------+
```

A no-op also clears the terminal and selects one of exactly 100 unique goose-themed lines, then
always states that nothing was updated, Honk300 is current and running, and the window may be
closed. Flavor text cannot replace those invariant state lines. Failure retains progress and
diagnostics, appends the IPC-proven recovery status, and stays visible.

The v1.3.5 presentation refinement gives every final state the same separated, fixed-width ASCII
panel. Success and no-op use **HONK! ALL DONE**; failures and recovery problems use **HONK! NEEDS
ATTENTION**. The close instruction remains literal in both variants, and color is never required
for hierarchy or meaning.

The helper deliberately remains alive after every final screen until the user closes the terminal
window (or terminates it). Automatic close is out of scope because the terminal profile, not
Honk300, owns close-versus-hold behavior.

## Consequences

- Native-menu users can complete a verified managed update without typing a command.
- The existing install-origin, exact-tag, target, artifact, size, hash, signature, receipt,
  rollback, cleanup-pending, autostart, and user-content rules remain authoritative.
- The updater has a visible home for UAC, `sudo`, package-manager, and failure diagnostics on every
  platform, at the cost of requiring the user to close a completed terminal window.
- A second click cannot race the active transaction; it receives a visible already-open result.
- Packaging and release qualification must prove the signed Mac resource, literal launch
  arguments, hidden help exclusion, lock recovery, terminal result contract, restart readiness,
  failure recovery, and unchanged public JSON/no-op receipt behavior.

## References

- [Apple Terminal profile shell settings](https://support.apple.com/en-lamr/guide/terminal/trmlshll/mac)
- [GNOME Terminal command-exit settings](https://help.gnome.org/gnome-terminal/pref-custom-exit.html)
