# ADR 0031 — Provenance-Preserving Slot Self-Update

- Status: Accepted (2026-07-17)
- Relates to: ADR 0013 (lifecycle packaging), ADR 0018 (atomic publication), ADR 0020
  (Developer ID DMG), ADR 0022 (managed Accessibility identity), ADR 0023 (rolling latest and
  Debian ownership), and ADR 0029 (Windows lifecycle hardening).
- Supersedes: ADR 0029's Windows update handoff, flat-payload replacement, bootstrap-receipt
  refresh, and Global/Corporate major-upgrade transaction ordering. Its web-request hardening,
  conservative terminal protection, uninstall identity checks, and immutable historical-release
  conclusions remain accepted. Earlier v1 receipts remain readable only for one successful
  migration; this ADR makes `honk300.install.v2` authoritative thereafter.

## Context

The released updater selected exact immutable bytes, but Windows returned success after starting
a detached helper that first waited for the invoking CLI to exit. MSI and EXE packages then
replaced flat `bin` payloads. That made the running executable itself the obstacle and conflated
installer registration, payload activation, and cleanup. Path classification also could not
distinguish MSI from EXE ownership, and unknown Windows installs were allowed to converge on
Global MSI. Those behaviors conflict with the user's latest installation intent and cannot
truthfully provide a Claude/Codex-style synchronous `update` command.

The durable pattern used by native self-updaters is an immutable payload store plus a stable
selector. Package managers still own their installations; standalone updaters mutate only their
own roots. macOS graphical installation and managed app update are separate transports, and
Windows has no universal mechanism for overwriting a currently mapped MSI-owned executable.

## Decision

### Authoritative provenance

`honk300.install.v2` records version, tag, commit, origin, installer family, edition, scope,
stable release track, layout, target, exact artifact name/hash/size, owned root, active release,
the three public aliases, autostart ownership, and cleanup state. Detection order is:

1. a regular, non-reparse receipt inside the running installation's owned root;
2. the platform's protected external receipt when the package shape requires it;
3. validated Windows registration or an adjacent owned marker;
4. conservative path evidence only where one path has one possible owner.

Malformed, foreign, or conflicting evidence resolves to unknown and stops. Program Files and
LocalAppData paths never guess MSI versus EXE. Existing v1 receipts are accepted for the exact
root and known channel, but only the successful new transaction writes v2.

An update preserves its exact origin: Global/Corporate MSI, Global/Corporate EXE, PowerShell
bootstrap, DMG-origin managed app, Debian package, or shell bootstrap. Package-managed and
shell-managed installs never convert during `update`. Unknown Windows installs receive the
stable Global-MSI assisted-reinstall URL; foreign or mounted macOS launches receive the stable
DMG URL; unknown Linux launches receive the stable shell-installer command. They never report
update success.

### Windows immutable slots

Every Windows installer stages the three identical binaries under
`channels/<installer-owner>/releases/<version>-<target>/bin`. The installer embeds the exact new
binary as its transaction helper and passes version, tag, commit, target, artifact path, and the
qualified payload SHA. The helper verifies all staged files and their reported version before
activation.

`current` is a directory junction to one immutable release. `bin` is a directory junction to
the lexical `current\bin`, preserving the existing unversioned PATH, shortcut, autostart, and
command locations. Activation retargets the existing junction reparse point in place, verifies
`honk300`, `honk`, and `goose`, then commits the protected receipt. The process that initiated
`update` remains mapped to its untouched old release throughout.

The first slot-aware installer may retire a legacy real `bin` directory into a retained
`legacy-flat` release. No medium-integrity process renames Program Files content. MSI owns
elevation and rollback custom actions; Inno owns its post-install transaction. Installed release
files are retained on ordinary upgrade so neither Windows Installer nor Inno tries to delete a
mapped old image. Cleanup is deferred and never changes activation success.

PowerShell remains a distinct origin even though its bootstrap delegates elevation and
registration to Global MSI. It supplies `HONK300ORIGIN=powershell`; the protected receipt and its
user-state copy therefore continue to select the PowerShell bootstrap on the next update.

Fresh installers allow downgrade. The newest successful activation is authoritative regardless
of semantic version. Channel payloads remain disjoint, and an older owner's uninstall must not
remove neutral selectors when the receipt names a different active origin. Failure before
selector/receipt commit restores the old selector and receipt. Cleanup after commit may fail only
as `cleanup_pending`; it must not roll back the user's new intent.

After slot commit, the helper inventories only strictly validated Honk300 MSI/Inno registrations.
Any conflicting owner is recorded in a protected cleanup journal and in the receipt. A later
`honk300 update` retries those exact registered uninstallers, verifies that each registration is
gone, and clears the journal only after all conflicts are retired. Machine-wide retirement uses
one hidden elevated active-slot coordinator: it runs the validated native uninstaller, removes
only that retired root's exact persisted PATH and Run entries when roots differ, and verifies the
active root's PATH before returning. This prevents a permanent MSI component from leaving a stale
higher-precedence command without adding a second elevation prompt. A failed or cancelled UAC
grant leaves the new slot and receipt intact but returns nonzero with an assisted cleanup command.
Registration validation follows the native installer rather than inferring ownership from scope:
Global MSI and EXE registrations must be protected machine records, Corporate Inno must be a
per-user record, and Corporate Windows Installer product inventory may legitimately appear in
HKCU or protected HKLM while its payload and receipt remain per-user in LocalAppData. Every form
still requires the exact display name, publisher, product/Inno identity, root, and uninstall
command before it can authorize cleanup.

Windows scope is an authority boundary, not something the updater may bypass. In particular, a
per-user Corporate installer cannot remove or outrank an existing machine-wide PATH/registration
without an administrator grant. Until that post-commit cleanup succeeds, the new Corporate slot
is retained but the install is `cleanup_pending` and must not claim that its public aliases are
authoritative. This is the truthful Windows implementation of latest intent; silently
uninstalling the machine owner before staging, guessing another channel, or reporting success
would violate the rollback and provenance contracts.

### macOS, Debian, and shell-managed Unix

The signed, notarized, stapled universal DMG remains the recommended fresh macOS experience. The
exact managed app at `~/Applications/Honk300.app` keeps the same bundle identity and updates
synchronously with the exact-tag signed universal app ZIP through the pinned bootstrap. A v2
receipt preserves DMG origin and remains eligible for managed Accessibility onboarding. Unknown,
read-only, foreign, source-tree, and mounted-DMG launches cannot claim managed update success.

Debian origin invokes only the exact architecture `.deb`; dpkg remains the owner. The package
contains a protected v2 receipt bound to its qualified payload. Shell-managed Linux uses
`releases/<version>-<target>` plus an atomic `current` symlink. Existing flat shell layouts migrate
on the first successful transaction, and old release directories remain available until deferred
cleanup. macOS shell transport still updates the app bundle rather than creating a second app
layout.

### CLI and publication contract

The public command is `honk300 update [--json]`. Progress is stderr-only. JSON mode emits exactly
one final stdout object with origin, previous and installed versions, target, artifact, result,
activation state, and cleanup state. Exit zero means the selected release is installed,
activated, and verified; refusal, rollback, reboot-deferred replacement, or pending conflicting
owner cleanup is nonzero.

Public filenames, links, and commands remain stable and unversioned. `latest` discovers only
`release-manifest.json`; mutation always uses the manifest's immutable exact-tag URL and requires
kind, target, size, and SHA-256 to match. Internal release directories retain version identity
because integrity and rollback require it.

## Consequences

- A running updater no longer blocks or schedules its own replacement; it waits for the installer
  and verifies the final selector, receipt, aliases, version, target, and artifact identity.
- The receipt, not a path guess or whichever registration is found first, expresses the user's
  chosen installer family and release track.
- Windows disk use grows until inactive releases are safely collected. That is intentional and
  bounded by future installer-owned cleanup rather than live-image deletion.
- An old immutable installer that predates this ADR cannot acquire the new downgrade semantics;
  every slot-aware installer can become authoritative over newer slot-aware installs once any
  required cross-scope administrator cleanup succeeds.
- Package registration and shortcuts remain native MSI/Inno/dpkg responsibilities. Slot helpers
  own staging verification, neutral selector activation, receipt commit, rollback, and a narrowly
  validated conflicting-owner cleanup journal; they never delete arbitrary registrations.

## Verification contract

- Windows qualification holds an old release process open, injects failure after junction
  retargeting, proves complete rollback, then activates another origin without terminating or
  replacing the old image. It verifies the v2 receipt, neutral selector, all aliases, identical
  hashes, and removed transaction state.
- The complete disposable matrix covers four Windows families on x64/ARM64, first legacy
  migration, repair, intentional slot-aware downgrade, cross-channel takeover, registry fallback,
  ambiguous receipt refusal, and pre/post-commit fault points. MSI results that require reboot
  remain failures.
- macOS qualification covers DMG installation, exact app-ZIP update, signing, notarization,
  stapling, Gatekeeper, managed path, Accessibility identity, rollback, and mounted-DMG refusal.
- Debian and shell paths qualify independently for every published architecture. Shell tests prove
  repeated same-version install, atomic selector rollback, external user-content preservation,
  and all three stable aliases.
- Candidate, same-SHA main, atomic release, and post-release jobs compare fresh public latest
  aliases with exact-tag identity before declaring a release complete.
