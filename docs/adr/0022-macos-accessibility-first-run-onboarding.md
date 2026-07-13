# ADR 0022 — macOS Accessibility First-Run Onboarding

- Status: Accepted (2026-07-13)
- Relates to: ADR 0010 (macOS bundle identity and capability status), ADR 0015 (platform safety),
  ADR 0019 (runtime ordering), and ADR 0020 (Developer ID distribution and managed DMG install).
- Supersedes: nothing. This decision adds onboarding to the existing macOS identity, runtime,
  and distribution contracts.

## Context

Honk300 needs macOS Accessibility permission for cursor warp and foreign-window behavior. The
stable agent bundle and status protocol already made denial honest, while ADR 0020 established a
Developer ID-signed managed install at `~/Applications/Honk300.app`. A first launch that merely
reported `denied` still left the person to discover the correct System Settings pane, and a
restart-only transition would make the normal FirstUX introduction needlessly fragile.

Automatic permission UI must not come from development binaries, bare command-line copies,
source-tree bundles, or an app launched directly from a mounted DMG. It must also avoid reopening
System Settings every time a person intentionally leaves permission denied.

## Decision

### Managed-release eligibility

Automatic onboarding is restricted to an exact managed release. All of these facts must agree:

- the canonical executable is
  `~/Applications/Honk300.app/Contents/MacOS/honk300` and is not a symlink;
- the containing bundle identifier is `dev.emmetts.honk300`;
- the bundle supplies a safe semantic version, matching `v<version>` tag, and 40-hex release
  commit; and
- the owner-controlled `honk300.install.v1` receipt names the same install root and matches that
  version, tag, and commit.

A missing or malformed identity, a foreign-owned app root or receipt, a symlinked app root,
receipt, or executable, or any metadata mismatch makes the launch ineligible for automatic UI.
The existing truthful degraded startup remains available to those development and unmanaged
copies.

### One prompt per installed version

Before requesting any user-visible UI, the runtime atomically creates this owner-only marker:

```text
~/Library/Application Support/honk300/state/accessibility-prompt-v1/<version>
```

The existing Honk300 state root must be a real current-user-owned directory. Newly created state
and prompt directories must also have mode `0700`; the marker must be a real current-user-owned
file with mode `0600`. Symlinks, foreign ownership, unsafe permissions, and marker-write failures
fail closed without opening System Settings. Normal uninstall preserves this user state; the
existing purge path removes it with the Honk300 state tree.

After the marker is secure, the AppKit main thread calls `AXIsProcessTrustedWithOptions` with the
native prompt option and asks `NSWorkspace` to open the Accessibility pane. The direct
Accessibility URL is preferred and the general Privacy & Security pane is the fallback. Honk300
does not click a consent control or simulate input in System Settings. A later denied launch of
the same installed version enters the wait state without reopening the prompt or Settings pane.

### Calm permission wait

The platform-neutral engine owns a `permission_wait` task, but only the managed macOS runtime
activates it. The task walks the goose to a deterministic lower-right anchor inside the primary
display's safe edge, holds a calm idle pose, and updates the anchor when display topology changes.
Entering the mode clears interrupted or queued cursor, collect-window, mud, Autumn, mood, pat,
particle, and sound work so permission-bound behavior cannot leak across the boundary.

While permission is denied:

- automatic roaming, cursor/window pranks, collect windows, mud excursions, seasonal play,
  moods, and scheduled honks cannot start;
- direct `do honk` remains available, while `wander`, `mud`, `nab`, `meme`, and `note` return
  `BUSY`; and
- status, reload, stop, exit, and quit continue over the existing owner-scoped IPC channel.

The existing status protocol continues to report Accessibility, cursor, and window denial; no
configuration or IPC schema is added for onboarding.

### Live grant and revocation

The managed runtime checks Accessibility at most once per second.

- On `denied -> trusted`, it refreshes cursor/window capability state, recreates the foreign-
  window watcher when enabled, reapplies effective options, leaves permission wait, and starts a
  fresh FirstUX introduction in the same process.
- On `trusted -> denied`, it drops the watcher, marks permission-bound capabilities denied,
  abandons their active work, and returns to permission wait without reopening UI for the
  already-marked version.

Overlay rendering, audio policy, local IPC, and terminal-window protection remain active through
both transitions. Terminal applications are filtered before any newly restored watch, ride, or
collect path can use them.

## Consequences

- The managed app gives a visible, characterful permission handoff without adding a settings
  window, menu-bar item, Dock control, AppleScript API, config field, or new command protocol.
- Intentional denial is respected: one installed version can ask once, then wait without a nag
  loop.
- A grant can resume the intended introduction without a rebuild or restart, and a later
  revocation fails safely while leaving control available.
- Windows and Linux behavior is unchanged because the engine primitive remains dormant there.
- Website deployment and production promotion remain outside this decision and stay parked
  behind the immutable-release and fresh-download gates.

## Verification

Automated coverage is green for the permission-wait task and suppression rules, exact installed
eligibility, receipt and metadata mismatches, secure marker creation, symlink rejection,
one-prompt-per-version policy, native prompt/thread and Settings URL contracts, safe anchors,
deterministic grant/revocation transitions, and the signed-app smoke-script contract.

This ADR does not close macOS readiness by itself. One recorded digest of the exact unchanged
signed release candidate must still cover all four native states before release: first denied
launch, denied relaunch without another prompt, live grant with same-process FirstUX/capability
recovery, and live revocation back to the calm wait. Prompt/non-prompt observations require an
explicit operator confirmation; destructive marker, Accessibility-record, install, and fixture
cleanup requires separate opt-ins. Ordinary-window behavior and protected-terminal observations
remain part of that candidate evidence.
