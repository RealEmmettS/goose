# Optional Wayland integrations

KDE and Sway have separate release stages. The next candidate adds independently
qualified Hyprland observations; publication remains ordered after Sway.
Native owned notes and pictures use normal compositor placement unless the enabled
desktop adapter supplies safe placement.

Honk300 uses X11 or XWayland by default. Native Wayland is an explicit choice:
start with `honk300 start --wayland`, or select the Wayland backend in settings and
restart the goose. The ordinary native backend works without additional desktop
permissions and reports its reduced capabilities.

## KDE window support

In **Platform & status**, choose **Set up KDE** and read the consent dialog. Enabling
it permits the running goose to observe window identities, geometry, current desktops,
fullscreen state and user drags. It also permits bounded placement of its owned notes
and pictures. Terminal windows, including Codex and Visual Studio Code, remain protected.

The equivalent commands are `honk300 integrations kde setup`,
`honk300 integrations kde status`, and `honk300 integrations kde remove`.
Removal takes effect while the goose is running. It stops placement while preserving
delivered notes; an active picture delivery follows the existing cleanup behavior.
You can also remove the integration while
the goose is stopped. Only Honk300's registration and permission record are removed.

Window support is separately exercised on KDE Plasma 5 and 6, on x64 and ARM64.
The native test desktops use Ubuntu 24.04 and Debian 13. A change to the bundled KDE
script requires explicit setup again. No global desktop extension or startup entry
is installed.

## Temporary pointer permission

On a running native Wayland goose with its KDE 6 integration enabled, choose
**Request pointer access**. The desktop presents its own permission dialog. Declining
or cancelling that dialog leaves window support available. Refresh status after the
dialog closes to see whether the granted pointer device is ready.

This requires the GNU Linux build and the optional system libraries `liboeffis.so.1`
and `libei.so.1`, supplied by the distribution. On the qualified Debian desktop their
packages are `liboeffis1` and `libei1`. The musl build and KDE 5 keep their window
capabilities and report pointer permission as unsupported.

Honk300 requests pointer motion only. It does not request keyboard or button injection,
save a permission token, install a global input helper, or grant permission automatically.
Existing mischief opt-outs and manners settings still determine whether the goose acts.
Every actual move requires a fresh, bounded desktop observation and a safe path that
does not cross a terminal or an unknown window.

**Cancel pointer access** ends the current grant, including a pending permission request.
Closing settings leaves the running goose and its current grant intact. Stopping the
goose, restarting it, losing the desktop connection, or removing KDE integration ends
the grant; a later run requires a new explicit request. A paused, removed or disconnected
input device also ends permission.

The equivalent commands are `honk300 integrations pointer request`,
`honk300 integrations pointer status`, and `honk300 integrations pointer cancel`.

## Capability boundaries

Window observation, placement, pointer observation, pointer control and fullscreen
awareness are reported separately. Fullscreen awareness does not establish do-not-disturb
support. Each compositor has separate qualification. Experimental Raspberry Pi guidance does not establish physical
Pi performance or extend the KDE test results to another compositor.

## Sway window and fullscreen observations

On Sway **1.9** or **1.10.1**, start the native Wayland goose from your desktop
session. In **Platform & status**, choose **Set up Sway**, then **Enable Sway
observations**. These exact versions have independent x64 and ARM64 native evidence
on Ubuntu 24.04 and Debian 13. An unqualified version reports that observations
are unavailable instead of assuming the same interface.

Setup permits read-only window identities, visible geometry and fullscreen awareness.
The goose uses the existing **Pause on fullscreen** preference. The adapter does not
move windows or the pointer, watch user drags, or observe do-not-disturb status.
Notes and pictures continue to appear at the compositor's normal chosen position.
Sway's public interface does not expose the current user-drag state needed for safe
animated deliveries; a window movement command alone cannot establish that capability.

Use **Remove Sway observations** to revoke them immediately, or use these commands:

```sh
honk300 integrations sway setup
honk300 integrations sway status
honk300 integrations sway remove
```

Honk300 connects only to the session's private `SWAYSOCK`, verifies the actual
same-user system compositor, and bounds replies and waiting time. It writes only
its own permission record; it does not edit your Sway configuration or bindings.
Repeating setup keeps a healthy connection. Removing permission also works while
the goose is stopped and preserves other integration records and unsaved settings.

A replaced socket, disconnected compositor, stale reply or unavailable powered
output withdraws observations. After restoring the desktop, repeat setup to reconnect.
A normal goose restart may use its saved read-only permission; it never grants pointer
access. Closing settings leaves the goose running, and stopping the goose preserves
unsaved settings in its independent window.

## Hyprland window and fullscreen observations

The Hyprland candidate qualifies **0.53.3** and **0.55.2**, each on x64 and ARM64.
Start the native Wayland goose from your desktop session, open **Platform & status**,
choose **Set up Hyprland**, and confirm **Enable Hyprland observations**. A version
outside that qualified set reports unavailable observations.

The adapter reads current visible windows and fullscreen presence for the existing
**Pause on fullscreen** preference. Notes and pictures use normal compositor
placement. Hyprland's public interface does not report authoritative current user
drags, so animated placement, foreign-window movement and window rides remain
unavailable. Pointer observation/control and do-not-disturb are also unavailable.

Choose **Remove Hyprland observations** for immediate revocation, or use:

```sh
honk300 integrations hyprland setup
honk300 integrations hyprland status
honk300 integrations hyprland remove
```

Setup writes Honk300's own private permission record and leaves your Hyprland
configuration and bindings intact. Repeating setup keeps the same live worker.
The worker verifies the actual system compositor and private socket, with bounded
replies and waiting time. A late reply immediately withdraws observations; another
bounded read may recover only against that same authenticated owner. A replaced
socket, malformed reply or disconnected compositor ends the worker. Restore the
desktop and repeat setup to reconnect.

Removing permission also works while the goose is stopped. Normal restarts may
reuse saved observation permission. Closing settings keeps the goose running;
setup, removal and stopping the goose preserve the independent settings draft.

## GNOME window, fullscreen and user-drag observations

The GNOME candidate targets **GNOME Shell 46.0** and **48.7**, each on x64 and ARM64.
Its full native qualification is tracked in [the release record](readiness/v1.9.0-readiness.md).
Other Shell versions report unavailable observations. Start the goose normally in
your GNOME Wayland session; GNOME uses the compatible XWayland overlay.

Open **Platform & status**, choose **Set up GNOME**, then confirm **Enable GNOME
observations**. First setup or an updated companion may require signing out and
back in so Shell discovers the installed extension. If requested, enable
**Honk300 desktop observations** in GNOME Extensions and repeat setup. Honk300
never changes the global extension switch or enables another extension.

The companion observes current windows, fullscreen apps and actual user-held window
drags. The goose can ride an ordinary window while you drag it and respects
**Pause on fullscreen**. It excludes terminal windows and stops the ride when the
drag ends, its window closes, window rides are disabled or observations are lost.
The companion never moves another app's window. Pointer observation/control and
do-not-disturb awareness remain unavailable. Honk300-owned notes and pictures use
their separate XWayland positioning support.

Choose **Remove GNOME observations** to revoke access, or use:

```sh
honk300 integrations gnome setup
honk300 integrations gnome status
honk300 integrations gnome remove
```

Setup installs only the versioned Honk300 companion and its private permission
record. Rust verifies the system Shell; the companion accepts observations only
from the executable you approved. An update requires explicit setup again.
Changed or unrelated extension files are preserved and reported during removal.
Removing observations works with the goose stopped and retains your other
extensions, desktop settings and independent unsaved settings draft.

Lost access, a disabled extension, a replaced Shell owner or invalid data clears
observations and cancels any window ride. Restore the desktop and repeat setup
to reconnect. A normal restart may reuse valid saved observations; it never grants
pointer access.
