# macOS Accessibility First-Run Design

Date: 2026-07-13  
Status: Approved  
Release: Honk300 v0.3.3  
Board: `#m20q` / `#m16r`

## Goal

Give a person who launches the installed, signed macOS app a clear and characterful path to
grant Accessibility permission without creating a nag loop. While permission is absent, the
goose walks to a safe screen-edge perch and waits calmly. The running process continues to
serve status, reload, stop, and IPC requests. As soon as macOS reports the grant, the same
process resumes the normal FirstUX sequence without requiring a restart.

This behavior is macOS onboarding. The engine primitive used to hold the goose safely remains
platform-neutral and dormant on Windows and Linux.

## Scope

- Trigger the native Accessibility consent request and open the Accessibility pane once for
  each installed Honk300 version that starts without permission.
- Restrict automatic prompting to the exact managed app installed at
  `~/Applications/Honk300.app`, backed by a matching release receipt. Development binaries,
  bare command-line copies, source-tree bundles, and an app launched directly from a mounted
  DMG retain the existing non-prompting degraded behavior.
- Keep the goose visible in a calm permission-wait state near a safe edge of the primary
  display while Accessibility is denied.
- Detect both grants and revocations while the process is running.
- Preserve normal control-plane behavior and truthful capability reporting.
- Record and verify the experience on the exact signed release candidate.

## Non-goals

- No settings window, menu-bar UI, Dock control surface, AppleScript API, or configuration
  schema change.
- No attempt to click the permission switch, bypass macOS consent, or automate System Settings.
- No attempt to discover or follow the System Settings window before Accessibility is granted;
  its reliable geometry is itself behind the permission boundary.
- No repeated prompt on every launch, background notification loop, or prompt from developer
  builds.
- No change to Windows or Linux permission UX.

## Eligibility and durable state

The runtime derives a `ManagedInstalledRelease` decision from all of these facts:

1. `current_exe` is the exact canonical executable under
   `~/Applications/Honk300.app/Contents/MacOS/honk300`.
2. The containing bundle identifier is `dev.emmetts.honk300`.
3. `~/Library/Application Support/honk300/install-receipt.json` is owned by Honk300, names the
   same install root, and matches the binary's embedded version, tag, and commit metadata.

The installer and release gates remain responsible for Developer ID verification. Runtime
eligibility does not shell out to `codesign` on every launch; the exact path and matching
release receipt distinguish the signed managed installation from local development runs.

Prompt history lives outside the TOML configuration schema at:

```text
~/Library/Application Support/honk300/state/accessibility-prompt-v1/<version>
```

The state directory is owner-only and the marker is atomically created with owner-only
permissions. A marker is created before any user-visible prompt is attempted. If secure marker
creation fails, Honk300 logs an actionable error and does not open UI, preventing an
unrecorded prompt loop. A normal uninstall preserves this user state; `uninstall --purge`
removes it with the existing Honk300 state tree.

## Startup and transition state machine

| Installed identity | Accessibility | Version marker | Result |
| --- | --- | --- | --- |
| No | Denied | Any | Existing degraded startup; no prompt and no permission-wait override |
| No | Granted | Any | Existing normal startup |
| Yes | Granted | Any | Normal FirstUX; no prompt |
| Yes | Denied | Missing | Securely create marker, request native consent, open Accessibility settings, then wait |
| Yes | Denied | Present | Wait calmly without reopening settings |

The runtime probes Accessibility at most once per second rather than every frame.

- `Denied -> Granted`: rebuild cursor and foreign-window capabilities, create the window
  watcher, reapply effective options, leave permission-wait, and start the normal FirstUX task
  in the same process.
- `Granted -> Denied`: disable cursor/window capabilities, release any active permission-bound
  behavior, enter permission-wait, and do not reopen settings for the already-marked version.
- Probe or settings-opening failures: keep the overlay and control plane alive, report denied or
  failed capability truthfully, and log one actionable diagnostic.

The native request uses `AXIsProcessTrustedWithOptions` with the prompt option. AppKit then asks
`NSWorkspace` to open the Privacy & Security Accessibility pane. The direct Accessibility URL
is attempted first, with the general Privacy & Security pane as a fallback. Honk300 never
simulates input in either pane.

## Permission-wait behavior

The engine gains an explicit, platform-neutral permission-wait mode with three small operations:
enter at an anchor, update the anchor after topology changes, and leave into FirstUX. macOS is
the only runtime that invokes it.

On entry, the mode clears pending interruptions and permission-bound commands, walks the goose
to a deterministic point inset from the lower-right safe edge of the primary display, then
holds a normal calm idle pose. Display changes recompute that anchor. The renderer, gait, fixed
120 Hz simulation, and maximum 60 Hz presentation contract remain unchanged.

While waiting:

- automatic roaming, cursor nabbing, foreign-window riding, collect windows, mud excursions,
  hourly honks, and other pranks cannot start;
- `status`, `reload`, `stop`, `exit`, and `quit` continue to work normally;
- a directly requested `do honk` remains available, while `wander`, `mud`, `nab`, `meme`, and
  `note` return `BUSY`;
- status continues to report overlay support and Accessibility/cursor/window denial using the
  existing protocol, so no IPC schema change is required;
- audio follows the existing `--no-sound` and configuration rules.

Leaving the mode starts `FirstUxTask` from the current safe-edge position. The goose therefore
walks naturally toward center stage, performs the usual intro beat, and enters normal roaming.

## Error handling and security

- Marker directory traversal rejects symlinks and foreign ownership using the lifecycle's
  existing secure-directory patterns.
- Marker creation is atomic and never overwrites foreign content.
- Receipt parsing, path canonicalization, or metadata mismatch makes the launch ineligible for
  automatic UI rather than weakening the check.
- Native UI requests occur on the AppKit main thread inside an autorelease pool.
- Only permission-dependent capabilities transition; overlay rendering, IPC, terminal
  protection, and stop semantics remain available.
- Terminal windows remain excluded before every watch, ride, collect, or future mischief path,
  including immediately after a live grant.

## Verification

### Automated

- Failing-first policy tests cover exact installed eligibility, receipt/version mismatches,
  source/DMG/bare launches, existing markers, secure marker permissions, symlink rejection, and
  one-prompt-per-version behavior.
- State-machine tests cover denied startup, non-nagging restart, live grant, live revocation,
  watcher initialization failure, and settings-open failure.
- Engine tests prove the goose reaches and remains at the safe anchor, topology changes update
  it, automatic tasks and permission-bound commands remain empty, only honk is accepted, and a
  grant resumes FirstUX.
- Existing macOS runtime, IPC, lifecycle, renderer, performance, and terminal-classification
  tests remain green. The complete Windows and Linux gates prove the dormant engine primitive
  causes no behavior change on those platforms.

### Exact signed-app smoke

1. Install one exact universal Developer ID-signed candidate at the managed path and reset only
   Honk300's Accessibility record where macOS permits.
2. Launch denied. Confirm the native request and Accessibility pane appear once, the marker is
   owner-only, the goose waits visibly at the safe edge, status is truthful, blocked actions are
   busy, and stop/restart work.
3. Relaunch the same denied version. Confirm no settings pane or prompt is reopened and the goose
   still waits safely.
4. Grant Accessibility to that unchanged signed identity. Confirm the running process detects it
   within the polling interval, walks into FirstUX, and enables the supported cursor/window
   capabilities without rebuilding or restarting.
5. Exercise CLI/TUI/IPC/audio, ordinary-window behavior, and protected Terminal, Codex, VS Code,
   Terminal.app, and available third-party terminal identities.
6. Revoke the grant while running and confirm the goose returns to permission-wait without a new
   prompt or permission-bound command.
7. Complete candidate/release verification and return the Mac to the required uninstalled state.

## Documentation and decision records

Implementation adds a new ADR for the permission-onboarding boundary rather than rewriting the
historical M16 decision. `README.md`, `AGENTS.md`, `CLAUDE.md`, the canonical plan, readiness
reports, `CHANGELOG.md`, and `HUMAN_CHANGELOG.md` will describe the final tested behavior. The
technical and plain-English changelogs remain in lockstep.

## Alternatives considered

- **Freeze the renderer in the runtime only.** Rejected because task scheduling and direct pokes
  could still enqueue side effects; an explicit engine mode is deterministic and testable.
- **Roam in degraded mode while settings is open.** Rejected because it weakens the visual cue
  and permits non-permission pranks before onboarding is complete.
- **Repeatedly open System Settings until granted.** Rejected as intrusive and hostile to an
  intentional denial.
- **Sit beside the exact Settings window before permission.** Rejected because reliable foreign
  window geometry is unavailable at precisely that point. A safe-edge perch preserves the
  character without pretending the capability exists.
