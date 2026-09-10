# ADR 0052: Goose application entry points and desktop interaction polish

Date: 2026-09-09

Status: Published in v1.11.0 with complete release and fresh-public qualification. The user confirms the installed Windows application, logo and graphical controls work; protected receipt and companion hashes agree with the published source. Follow-up settings and tray refinements are in ADR 0054.

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

Linux retains the visible `honk300.desktop` launcher and the Native SDK's existing
`dev.emmetts.honk300.settings` application identifier. A hidden desktop record with that
identifier and a matching full-resolution theme icon let the desktop identify the running
settings window without adding another menu entry. Both records carry `StartupWMClass`.
This follows [GTK's default application-icon lookup](https://docs.gtk.org/gtk4/class.Application.html)
and [GIO's desktop-id lookup](https://docs.gtk.org/gio-unix/ctor.DesktopAppInfo.new.html).
Debian owns the packaged records; shell/manual installation owns only its exact marked
records and icon link. Rollback restores those integrations and uninstall preserves foreign files.
The pinned GTK host's stored `icon_path` alone does not apply a window icon.

## Installed Configure repair

The Global MSI binds Windows Installer's standard `ProgramMenuFolder` to an explicit
`MachineProgramMenuFolder` using WiX `SetDirectory`. The standard property resolves to
the all-users menu for the existing per-machine scope, as documented by
[Microsoft](https://learn.microsoft.com/en-us/windows/win32/msi/programmenufolder).
The explicit machine directory retains the HKLM component key path and avoids treating
the contextual standard directory as per-user component data. WiX schedules the
[directory assignment](https://docs.firegiant.com/wix3/xsd/wix/setdirectory/) before
cost finalization in both installation sequences. Native linker checks must pass without
suppressing ICE43 or ICE57; changing the machine component to HKCU is not a scope-preserving fix.
The former undefined `CommonProgramsFolder` resolved to the drive root. A real candidate
installation wrote `D:\Goose\Goose.lnk`; the user's old `C:\honk300\Honk300.lnk` confirms
the same original defect. Preserve installation scope and package identities while correcting
the shortcut destination. Qualification must inspect the real Start-menu link, target,
arguments and icon after installation rather than accepting successful MSI exit status.

MSI publishes its primary icon through the documented
[Windows Installer ProductIcon API](https://learn.microsoft.com/en-us/windows/win32/api/msi/nf-msi-msigetproductinfow).
Qualification reads that icon and compares the real cached file with the approved artwork.
Resolve environment-variable icon descriptors before native decoding, and retain both the
registered descriptor and verified path in the evidence. Per-user and per-machine icon caches
have [different locations](https://learn.microsoft.com/en-us/windows/win32/msi/installation-context).
The EXE installer continues to use its owned launcher's `DisplayIcon`. A missing
`DisplayIcon` registry value alone is not proof of a missing MSI icon. Icon-table filenames
keep their `.ico` extension for shell interpretation. Mac qualification decodes the actual
ICNS through AppKit; Debian uses the full application image in the 512-pixel hicolor slot,
and manual Linux entries point to an owned copy of the same image.

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

The user rejected generic pointed/lobed shapes and requested distinct botanical SVG designs
in the existing colors. Four reviewed masters provide maple, oak, birch and ginkgo silhouettes;
every design has gold, orange, red and brown variants. A deterministic generator keeps the
editable SVGs, recolors and compiled Rust paths synchronized. The runtime builds sixteen
fixed supersampled sprites once, using 144 KiB, then varies rotation, mirroring and size
independently of species and color. All rotated pixels fit the shared 18-pixel damage margin.
This adds no runtime SVG parser, filesystem access or service dependency.

Existing leaf counts, spawn behavior, random simulation draws, collision,
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

Review the SVG family, actual desktop-size renderer frames, bounded rendering cost,
timed input-to-follow behavior, distance/timeout/cancellation,
and untouched-pile expiry. Qualify the real native Windows app entry points with a stale
external receipt present across installer origins and on both native architectures, including
embedded and actual window icons. Keep ordinary source/CI, installed controls, fresh public
downloads, and physical-device acceptance distinct. Ship these changes together in v1.11.0
after the existing release gates pass.
