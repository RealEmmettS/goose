# ADR 0052: Goose application entry points and desktop interaction polish

Date: 2026-09-09

Status: Accepted by the user; implementation and combined release qualification in progress.

## Decision

The native application is the primary control surface. Platform application menus display
**Goose** and open the existing Native SDK controls. Running `goose` without arguments opens
the same controls. Explicit `goose start`/`plz`, graceful stop, settings, update, and the
secondary `goose config` terminal editor continue through the existing Rust services.
The compatible `honk300` and `honk` commands remain. This supersedes earlier guidance that
bare commands start the runtime and that the terminal is the primary user interface.

The implementation changes entry-point routing and presentation, without creating a separate
lifecycle, configuration, update, or persistence owner. Windows shortcuts use the existing
windowless app launcher with `--settings`; login startup and explicit starts retain their
windowless runtime route. The primary executable and aliases retain their console subsystem.
Installer families, permanent upgrade identities, installation roots, immutable slots, receipts,
and the Mac app bundle identifier and managed filesystem location are unchanged.

Windows executable resources, native GUI window icons, installer icons and installed-app icons
use the existing approved goose artwork. Installer discovery accepts the new Goose display names
and the exact historical Honk300 names while preserving publisher, registration, scope, path,
and receipt validation. The Mac keeps `~/Applications/Honk300.app` for its existing ownership
and permission contracts, and uses localized bundle display names for Goose in Finder and
Spotlight. Both bundle-name keys are localized; their base values match the preserved filename,
as specified by [Apple's bundle naming documentation](https://developer.apple.com/library/archive/documentation/General/Reference/InfoPlistKeyReference/Articles/CoreFoundationKeys.html).

## Installed Configure repair

The Global MSI must use Windows Installer's standard `ProgramMenuFolder`, which resolves
to the all-users menu for the existing per-machine scope, as documented by
[Microsoft](https://learn.microsoft.com/en-us/windows/win32/msi/programmenufolder).
The former undefined `CommonProgramsFolder` resolved to the drive root. A real candidate
installation wrote `D:\Goose\Goose.lnk`; the user's old `C:\honk300\Honk300.lnk` confirms
the same original defect. Preserve installation scope and package identities while correcting
the shortcut destination. Qualification must inspect the real Start-menu link, target,
arguments and icon after installation rather than accepting successful MSI exit status.

The user's protected MSI receipt correctly described the installed app and its settings
companion. A stale per-user PowerShell receipt for the same root described an older release
without that companion. Settings launch now follows the updater's existing precedence:
valid current owned evidence wins; external evidence is consulted only when owned evidence
is absent. Invalid or contradictory owned evidence still fails closed, and settings and
accessibility-library size/hash checks and Windows file leases remain mandatory.

The old per-user receipt on the user's machine was preserved under a backup filename after
the protected receipt and companion hashes were checked. The installed GUI then opened.
Native menu selection proved tray Configure and Update, and native GUI controls proved
Start and graceful Stop. This repair did not replace any released executable or tagged asset.

## Leaves and affectionate following

Procedural autumn leaves have pointed/lobed silhouettes, stems, varied orientation and
continuous tumble. Existing leaf counts, spawn behavior, random simulation draws, collision,
gravity, bounce, and velocity-dependent kick strength remain. Every pile expires after
30 seconds, fading during its final four seconds; a kicked pile can expire sooner through
the existing ten-second kick lifetime. A late kick cannot extend a pile's life.

The petting preference gates both hearts and an invited follow. Roughly 2.5 seconds of
actual repeated pointer movement over the goose earns one follow per rubbing streak.
Stationary hovering, clicks, and isolated fast sweeps do not earn it. The goose follows at
its configured walking speed for 15 seconds, stopping about 90 world pixels behind the
pointer. The task changes only the goose's locomotion targets and emits no cursor or
window commands. It ends on pointer loss, a held click, disabling petting, active manners,
permission withdrawal, or shutdown. Existing terminal protection remains in force.

## Qualification

Review actual renderer frames, timed input-to-follow behavior, distance/timeout/cancellation,
and untouched-pile expiry. Qualify the real native Windows app entry points with a stale
external receipt present across installer origins and on both native architectures, including
embedded and actual window icons. Keep ordinary source/CI, installed controls, fresh public
downloads, and physical-device acceptance distinct. Ship these changes together in v1.11.0
after the existing release gates pass.
