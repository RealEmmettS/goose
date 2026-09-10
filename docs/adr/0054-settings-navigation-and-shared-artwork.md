# ADR 0054: Settings navigation and shared application artwork

Date: 2026-09-09

Status: Accepted; implementation and release qualification in task #g12.

## Context

The user confirms that the installed Windows application now has its logo and opens
the graphical controls. They request visible tab selection, controllable smooth wheel
scrolling, no Wayland switch on unsupported platforms, and one consistent logo in the
application and operating-system control surfaces.

Native SDK 0.5.4 deliberately gives a selected ghost button no persistent fill. Its
default scroll impulse decays by only 14 percent per second, causing one Windows
wheel notch to reach the bottom of the settings page. The tray implementations still
use the earlier monochrome goose, recolored onto a blue circle on Windows and Linux.

## Decision

- Keep the existing five-page layout. Give the current tab the SDK's secondary
  variant and retain the selected semantic state. Other tabs remain ghost buttons.
- Set application scroll tokens explicitly: a small immediate step and short,
  strongly damped momentum. Reversals replace momentum and edges clamp. Reduced
  motion uses a direct logical-pixel step with no kinetic tail. Configuration,
  input routing, and the pinned SDK remain unchanged.
- Offer `platform.wayland` through the settings catalogue only in Linux builds.
  The GUI also hides that field on other platforms when reading an older service
  response. Loading and saving other fields retains its stored value. All shipped
  Linux targets retain the opt-in reduced Wayland backend.
- Use each platform's existing application artwork in its control surface.
  Windows loads resource 1 from its own executable, the exact multi-resolution
  `honk300-app.ico` used by its app, settings window and installer. Windows selects
  the native small-icon size, and the tray retains its own disposable icon handle.
  Linux rasterizes its existing `settings/assets/icon.png` once at tray creation,
  keeping the colors and transparency. StatusNotifier receives straight-alpha ARGB
  in network byte order, as specified by the [StatusNotifier format](https://specifications.freedesktop.org/status-notifier-item/latest-single/).
- Package the exact same PNG as `Goose-status.png` before macOS signing. AppKit
  presents it at 18 points with native template tinting. The template uses the
  Mac application's silhouette and follows menu-bar appearance without a baked background.

This supersedes only the small-icon artwork decision in ADRs 0028 and 0030. Shared
Configure, Update and graceful Quit behavior, receipt ownership, app identities,
permissions and release gates stay as specified by their existing ADRs.

## Verification

Exercise a real native window with an isolated configuration. Check selected tabs
away from hover, a single wheel notch, repeated and reversed scrolling, boundaries,
and platform visibility. Pin bounded settling at several frame rates and no kinetic
motion under reduced motion. Verify native channel/alpha conversion, actual tray
appearance, and Mac AppKit decoding/template behavior. Qualify all release platforms,
signed packages and fresh public downloads at the same source before publication
is considered complete. Hosted checks do not replace physical Mac user acceptance.
