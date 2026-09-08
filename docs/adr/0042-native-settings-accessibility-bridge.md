# ADR 0042: Native settings accessibility and its Windows payload

Status: accepted implementation refinement of ADR 0041; native qualification in progress.

## Context

Native SDK 0.5.4 produces a widget semantics tree but does not forward it to
Windows UI Automation or Linux AT-SPI. Windows inspection of the first settings
build exposed only its canvas pane. The macOS host already implements the native
accessibility callback. Upgrading the framework did not resolve the missing
Windows callback in the separately inspected current package.

## Decision

Forward the existing bounded widget snapshot to pinned AccessKit adapters on
Windows and Linux. Keep macOS's existing native bridge. Small, exact-source-hash
patches wire the local SDK dependency's publish, focus, destruction, and action
events. The global framework remains untouched.

The bridge retains at most 128 widget nodes, one text run per text widget, and
32 queued actions. It rejects malformed
trees and stale or disabled targets, and applies actions on the owning UI thread.
OS callbacks never enter Zig. Static text uses AccessKit's value representation;
modal dialogs exclude obscured controls from assistive navigation. Windows uses
the real HWND and GTK reports X11 screen bounds when available. Wayland does not
provide a portable global window position; this adapter does not invent one.

Windows uses `honk_settings_accessibility.dll` with its MSVC runtime statically
linked inside the DLL. The Native SDK executable uses Zig's Windows GNU runtime;
the two communicate only through the checked C ABI. Mixing their compiler/C++
runtimes in one static image is unsupported. Linux links the bridge statically.

The Windows DLL is a required settings payload in every portable archive, MSI,
EXE, and immutable release slot. Build metadata and the protected receipt record
its exact name, size, and SHA-256 under `settings_app.accessibility`. Activation
validates its independently supplied hash before and after selecting the slot.
The settings launcher holds read-only file leases on both executable and DLL
until the GUI message loop initializes; a failed or timed-out start kills and
waits for the child. Source distributions that include settings must include its
DLL. Complete dependency notices accompany each native build.

## Verification and limits

Windows OS inspection now exposes named controls, text, edit focus, dialogs, and
the saved result. Actual keyboard editing and Save persisted the isolated test
configuration. Bridge regressions cover roles, states, bounds, malformed trees,
modal filtering, and action admission. Windows file-lease regressions exercise
write/replacement rejection for both files and reject a missing or changed DLL.

The SDK automation harness remains a separate fixture from native assistive
technology acceptance. Hosted GTK/AT-SPI, complete native packaging, and final
release qualification remain required. Physical Mac/Pi acceptance is unavailable
on the current Windows-only setup and must remain explicit.

Actual editing found that the SDK's original 64-node limit truncated Save after
changed-field labels appeared on the Appearance page. The pinned local SDK and
bridge now share a 128-widget ceiling; the OS qualification edits that page and
requires the final Save action and persistent readback. Text runs expose Unicode
and empty field contents to native readers without fabricating glyph positions
or caret selection.
