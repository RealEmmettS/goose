# ADR 0057: Initialize installer startup choices from the current owned mechanism

Status: accepted for implementation, native qualification pending

## Context

The operator requires login startup to remain default-off. Both Windows MSI editions already
give the optional Autostart feature level 2, above the default install level. However, Windows
Installer remembers feature selection independently from Goose's configuration and Run value.
An installer opt-in followed by an app opt-out leaves Autostart installed in MSI's feature
database. Major-upgrade migration then restores its registry component.

The v1.11.1 packages additionally capture `HONK300AUTOSTART` at sequence 1002, before
`MigrateFeatureStates` at 1200. This records false in the receipt even while migration installs
the startup entry. The next settings read imports the mechanism and changes the configuration
to true. CI 39343205145 reproduced all of these boundaries in both native editions using
the immutable v1.11.0 and v1.11.1 installers. The local user's exact configuration was restored
through the revision-safe settings service; this containment is separate from the repair.

Inno's default `UsePreviousTasks=yes` creates the same stale-selection risk for EXE installs.
Changing all feature/task migration would also reset unrelated PATH or shortcut selections.

## Decision

Keep the existing installer-owned startup mechanisms and rollback. Correct only initialization
of the startup selection, using the current owned Run entry before the installer mutates it.
No entry means off. The exact current GUI launcher or supported legacy command means on.
An unrelated command or unreadable/malformed state fails closed instead of being replaced.

MSI embeds a small native action DLL, built for the package architecture and not installed as
an application component. It reads only the bounded session identity and the owned Run value.
After historical feature migration, it sets only Autostart's requested state. Explicit feature
arguments naming Autostart or ALL retain their ordinary MSI meaning. Unrelated feature arguments
and repair are not a renewed startup opt-in. Full UI may change the seeded selection; execute
revalidates ownership without resetting that choice, except that repair reinitializes from the
current mechanism. Both receipt Boolean outcomes are captured after `InstallValidate`.

Inno initializes only the autostart task after previous/command-line task restoration. It
preserves unrelated selections and explicit `/TASKS`, startup `/MERGETASKS`, and `/LOADINF`
choices. Back/Next does not reset the user's choice. Silent or skipped-page installation uses
the same initialization, and `PrepareToInstall` blocks mutation on ownership errors.

This deliberately repairs the earliest incorrect selection boundary. It does not add registry
mutation to slot activation, duplicate protected receipt parsing inside a second runtime,
disable all MSI feature migration, or require changes in the already-released updater.

## Qualification

Retain the failing published-byte regression. Require native fresh default-off, explicit on/off,
app opt-out then upgrade, app opt-in then upgrade, repair in both directions, unrelated selection
preservation, interactive selection/repair, foreign startup refusal and rollback checks.
Open settings afterward and compare config, receipt and owned Run state. Cover Global/Corporate
MSI and EXE on x64 and ARM64. Preserve all existing source, packaging, release and fresh-public
gates; publish a new immutable patch and never replace existing tagged assets.

## References

- [Microsoft: MigrateFeatureStates](https://learn.microsoft.com/en-us/windows/win32/msi/migratefeaturestates-action)
- [Microsoft: MsiSetFeatureState](https://learn.microsoft.com/en-us/windows/win32/api/msiquery/nf-msiquery-msisetfeaturestatew)
- [Microsoft: ADDLOCAL and feature-property precedence](https://learn.microsoft.com/en-us/windows/win32/msi/addlocal)
- [Inno: previous tasks](https://jrsoftware.org/ishelp/topic_setup_useprevioustasks.htm)
- [Inno: selecting individual tasks](https://jrsoftware.org/ishelp/topic_isxfunc_wizardselecttasks.htm)
