# ADR 0037 — Windows Integrated-Terminal Job Breakaway

- Status: Accepted (2026-07-19)
- Relates to: ADRs 0033 and 0036 (windowless app launch and typed-start handoff).
- Supersedes: only ADR 0036's hidden-runtime creation flags and assumption that a new process group
  is sufficient terminal detachment. Exact sibling identity, readiness, cleanup, CLI grammar,
  launcher ownership, and runtime behavior remain accepted.

## Context

ADR 0036 made typed Windows starts pass through a transient GUI-subsystem launcher, which starts
the exact sibling `honk300.exe __windows-app-runtime` with null handles,
`CREATE_NO_WINDOW | CREATE_NEW_PROCESS_GROUP`, waits for IPC readiness, and exits. Its public-start
smoke proved the origin shell process exits and the runtime survives its parent.

That does not prove the surrounding command runner is released. Integrated terminals can assign
the shell and all descendants to a Windows job. A new process group changes console-control
delivery but does not leave the job. The Codex app's job explicitly allows breakaway, yet v1.3.2's
hidden runtime did not request it, so the tool call kept tracking the goose after the controller and
app had exited.

The same investigation covered a reported black rectangle around the goose. The public-launch raw
premultiplied BGRA surface contained transparent margins and no opaque-black component. Strict
`CAPTUREBLT` evidence for v1.2.6 direct, v1.3.2 direct, v1.3.2 app-launched, and successive graceful
stop frames all retained the real desktop under the goose. Only a screen-copy path without the
required layered-window capture flag produced the black rectangle. That is capture-tool behavior,
not a renderer or DWM product defect.

## Decision

`honk300-app.exe` remains the only typed-start handoff owner. When it creates the hidden exact
sibling runtime, it first uses:

`CREATE_NO_WINDOW | CREATE_NEW_PROCESS_GROUP | CREATE_BREAKAWAY_FROM_JOB`

The controller-to-app launch remains unchanged. The app still supplies null standard handles,
uses no shell, forwards the argument vector without interpolation, waits for existing IPC
readiness, and kills only its own unready child on failure.

Windows jobs may deliberately disallow breakaway. If and only if the breakaway creation attempt
returns `ERROR_ACCESS_DENIED`, the app retries once with the original
`CREATE_NO_WINDOW | CREATE_NEW_PROCESS_GROUP` flags. Other errors do not retry, so a missing,
unexecutable, or otherwise invalid runtime cannot cause a duplicate or alternate launch. The
fallback respects managed containment while preserving ordinary product startup.

`DETACHED_PROCESS`, a shell intermediary, Task Scheduler, Explorer activation, and a second
runtime executable remain out of scope. Screenshot utilities must use layered-window-aware capture
when their output is treated as alpha evidence; product code does not compensate for a capture
path that omits layered composition.

## Consequences

- Codex and other breakaway-permitted integrated-terminal jobs release their command runner after
  readiness while the hidden runtime and tray remain alive.
- A managed job that forbids breakaway keeps its policy and still receives the original safe
  windowless launch rather than a new startup failure.
- Exact runtime ownership, update receipts, hashes, PATH, aliases, startup integration, and
  graceful/forced lifecycle semantics do not change.
- Qualification must include one real job-contained origin command that returns while the exact
  runtime remains independently status-responsive, followed by explicit cleanup.

## References

- [Microsoft process creation flags](https://learn.microsoft.com/en-us/windows/win32/procthread/process-creation-flags)
- [Microsoft job objects](https://learn.microsoft.com/en-us/windows/win32/procthread/job-objects)
- [Layered window capture requirements](https://learn.microsoft.com/en-us/windows/win32/api/wingdi/nf-wingdi-bitblt)
