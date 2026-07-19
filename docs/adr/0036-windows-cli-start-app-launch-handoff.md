# ADR 0036 — Windows CLI Start App-Launcher Handoff

- Status: Accepted (2026-07-19)
- Relates to: ADR 0028 (shared controls), ADR 0030 (Windows tray owner), ADR 0031
  (slot ownership), ADR 0032 (lifecycle), and ADR 0033 (windowless Windows launch).
- Supersedes: only ADR 0033's foreground-lifetime behavior for a typed Windows `start`.
  Public aliases remain console-subsystem commands, the TUI remains terminal-owned, and ADR
  0033's shortcut, login-start, background-helper, and disposable-desktop decisions remain
  accepted.

## Context

ADR 0033 separated the public console executable from the GUI-subsystem launcher used by Windows
shortcuts and login startup. It deliberately kept typed `start` attached to the user's console.
That made the terminal more than a controller: closing its PowerShell, Command Prompt, or terminal
host also ended the runtime, overlay, IPC server, and notification-area owner.

Windows already has the correct app-shaped launch surface. The gap is that typed starts bypass it,
and that the launcher previously returned immediately after spawning instead of proving bounded
runtime readiness. A user-pinnable launcher should also carry the canonical goose artwork rather
than inheriting an unbranded tool icon.

## Decision

### One Windows start chain

Every public Windows start spelling—explicit `start`, bare invocation, `plz`, and all three
`honk300`/`honk`/`goose` aliases—acts as a bounded console controller:

1. Resolve the exact sibling `honk300-app.exe`. A missing launcher is an installation or
   developer-build error; there is no single-binary fallback.
2. Forward every existing start option as an argument vector and synchronously wait for the app
   launcher's result. The controller creates no shell and gives the app null standard handles,
   `CREATE_NO_WINDOW`, and a new process group.
3. Return zero only when the app reports that a runtime was already ready or became ready, then
   verify readiness once more through local IPC and return the user's prompt.

`honk300-app.exe` remains a transient GUI-subsystem launcher rather than the runtime. It resolves
only its exact sibling `honk300.exe`, prepends the hidden Windows-only
`__windows-app-runtime` command, forwards the start options unchanged, and starts the child with
null standard handles plus `CREATE_NO_WINDOW | CREATE_NEW_PROCESS_GROUP`. It never invokes
PowerShell, `cmd.exe`, a shell association, string interpolation, `DETACHED_PROCESS`, or another
fallback executable.

The hidden runtime command is absent from help and documentation. It is used only by the exact app
launcher and qualification harnesses that need a retained process handle or redirected evidence.
That process retains authoritative singleton ownership and owns the engine, overlay, IPC, and
notification-area item for their complete lifetime.

### Readiness, races, and cleanup

The app polls the existing IPC status command for at most ten seconds. It exits zero as soon as a
running status is observed. Spawn failure, a child that exits unsuccessfully before readiness, and
readiness timeout have distinct internal exit codes which the public controller translates into
readable diagnostics.

Concurrent starts are intentionally harmless. More than one transient app may race, but the
hidden runtimes contend for the existing authoritative singleton. A launcher whose child exits
successfully because another runtime won continues observing that winner until it becomes ready or
the shared deadline expires. On timeout or an internal probe failure, a launcher kills and waits
only for its own still-running, not-yet-ready child. It never terminates an existing runtime.

### Branded launch identity

The canonical status-goose artwork is composed into a multi-resolution Windows `.ico` and linked
only into `honk300-app.exe` through a package build script and `embed-resource::compile_for`.
Developer and release builds must build both Windows binaries. Packaged source includes the build
script, resource script, icon, and source artwork. Shortcuts and optional login startup continue
targeting the app launcher, which users may pin normally; the running goose remains tray-only and
does not hold a taskbar window.

Tray Configure still opens the exact console executable's `config` TUI. Tray Quit and ordinary CLI
stop keep the shared graceful walk-off, while force stop remains immediate.

## Consequences

- Closing the originating terminal after a successful Windows start no longer ends the goose or
  its notification-area controls.
- Typed starts now take up to the existing ten-second readiness deadline and surface a meaningful
  failure status instead of leaving hidden startup failures behind.
- A standalone copied `honk300.exe` can still run non-start commands, but starting is deliberately
  rejected without its same-build app sibling.
- Compositor qualification invokes the private runtime command directly when it needs process and
  log ownership. A separate lifecycle phase proves the real public controller/app/runtime chain
  from short-lived shells.
- Public grammar, start flags, configuration, IPC, receipts, update provenance, and installer
  schemas do not change.

## References

- [Cargo build-script linker arguments](https://doc.rust-lang.org/cargo/reference/build-scripts.html)
- [`embed-resource::compile_for`](https://docs.rs/embed-resource/latest/embed_resource/fn.compile_for.html)
- [Microsoft process creation flags](https://learn.microsoft.com/en-us/windows/win32/procthread/process-creation-flags)
