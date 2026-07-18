# ADR 0033 — Windowless Windows App Launch And Disposable-Desktop Qualification

- Status: Accepted (2026-07-18)
- Relates to: ADR 0015 (platform safety), ADR 0028 (shared controls), ADR 0029
  (Windows lifecycle), ADR 0031 (slot ownership), and ADR 0032 (login-start preference).
- Supersedes: ADR 0032's Windows login-start command and interactive-local calibration-notice
  decision. The cross-platform config preference, owned prop sizing, and graceful/forced
  lifecycle decisions remain accepted.

## Context

Windows decides whether to create or attach a console from the executable's PE subsystem and
process-creation flags. `honk300.exe` must remain a console program: when a user intentionally
types a command, it must block, print, and return an exit status in that existing terminal. The
same executable is the wrong direct target for Explorer shortcuts and login startup because a
console host can appear, take foreground focus, and interrupt another application.

The Windows compositor qualifier also used full-virtual-desktop dark and near-white surfaces.
Those surfaces were test infrastructure, but on a person's desktop they were indistinguishable
from an app that had blanked the screen. The qualification does not justify disrupting a local
interactive session.

Microsoft documents the GUI/console PE subsystem distinction through `/SUBSYSTEM`, and documents
that `CREATE_NO_WINDOW` prevents a console application from inheriting or creating a console.
The `Run` key accepts a direct command line; it does not require PowerShell or another shell.

## Decision

### Two exact Windows entry points

Windows packages carry two same-release binaries in every immutable slot:

- `honk300.exe` is the public console-subsystem CLI/runtime. `honk300`, `honk`, and `goose` on
  `PATH` continue to resolve to those console bytes.
- `honk300-app.exe` is an internal GUI-subsystem launcher. It is not public CLI surface and is
  not placed on `PATH`. It resolves only the exact sibling `honk300.exe`, starts it with the
  literal `start` argument, null standard handles, `CREATE_NO_WINDOW`, and a new process group,
  then exits. It never invokes a shell, PowerShell, `cmd.exe`, a script association, or a fallback
  dialog.

MSI/Inno Start Menu and optional desktop shortcuts, Windows login-start values, and manual
per-user shortcuts target `honk300-app.exe` directly with no arguments. An exact legacy owned
`"honk300.exe" start` login value may be migrated; any other existing value is foreign and is
refused. The protected receipt records the launcher's stable path and SHA-256. Slot activation,
update verification, repair, takeover, and uninstall qualify that identity independently from
the three public aliases.

Background restarts and lifecycle helpers use direct executable paths, null or redirected
handles, and `CREATE_NO_WINDOW`; no background operation may allocate or activate a terminal.
The only expected terminal surfaces are the terminal the user deliberately used for a CLI/TUI,
and the terminal explicitly requested by **Configure Honk300…**. An authorized installer/update
may still show Windows' native elevation consent or installer UI; that is not a hidden helper
console.

### Disposable desktop qualification only

Product startup performs no screen calibration. Local Windows qualification no longer creates
full-desktop dark/white surfaces, even behind an opt-in flag. It may run non-obscuring lifecycle
or prop checks only when the operator welcomes visible testing. Strict paired-color compositor
proof runs on disposable GitHub Actions desktops.

The native tray-Quit smoke opens the exact process-owned menu, proves that popup belongs to the
runtime, and uses a randomized environment-scoped registered message to make the owner thread
end that menu and enqueue the same finite `Quit` command. It never sends global keyboard or mouse
input and cannot target a foreign foreground application.

## Consequences

- Login startup, shortcuts, and background recovery cannot flash a console or steal foreground
  focus through a console window.
- Intentional CLI behavior stays conventional and scriptable instead of becoming a detached GUI
  command with lost output/status.
- Every Windows archive and installer contains and independently verifies the internal launcher;
  this adds one non-public payload and receipt field.
- Local developer proof is less visually exhaustive by design. Native x64/ARM64 disposable CI
  owns the full compositor/lifecycle matrix, while local compile, PE-header, parser, unit, and
  packaging checks remain non-interactive.

## References

- [Microsoft `/SUBSYSTEM` linker option](https://learn.microsoft.com/en-us/cpp/build/reference/subsystem?view=msvc-170)
- [Microsoft process creation flags](https://learn.microsoft.com/en-us/windows/win32/procthread/process-creation-flags)
- [Microsoft Run and RunOnce registry keys](https://learn.microsoft.com/en-us/windows/win32/setupapi/run-and-runonce-registry-keys)
