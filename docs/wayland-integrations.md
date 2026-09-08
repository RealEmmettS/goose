# Optional Wayland integrations

KDE support is the preceding release stage. The next candidate adds separately
qualified Sway observations; publication remains ordered after the KDE stage.
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
support. GNOME and Hyprland adapters have separate implementation and desktop
qualification tasks. Experimental Raspberry Pi guidance does not establish physical
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
