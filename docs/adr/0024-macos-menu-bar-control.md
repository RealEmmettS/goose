# ADR 0024 — macOS Menu-Bar Control

- Status: Accepted (2026-07-14)
- Superseded in part: ADR 0028 replaces only the visible **Honk** title and makes this action
  behavior the parity contract for later Windows/Linux trays. All macOS lifecycle/control
  decisions below remain historical and accepted.
- Relates to: ADR 0004 (local control channel and terminal protection), ADR 0010 (macOS agent
  bundle), ADR 0020 (Developer ID app distribution), and ADR 0022 (Accessibility onboarding).
- Supersedes: only the macOS portions of ADR 0004 and ADR 0010 that prohibit any menu-bar
  control, plus ADR 0022's statement that onboarding adds no menu-bar item. Their local IPC,
  terminal-protection, bundle-identity, permission, and no-native-preferences decisions remain
  accepted. Windows and Linux remain tray-free.

## Context

The Developer ID-signed `Honk300.app` can be launched graphically from `~/Applications` or from
a Dock shortcut, but the original control-plane decision required people to remember terminal
commands to configure or stop it. That is needlessly awkward for a normal macOS app and makes an
otherwise graphical DMG installation feel unfinished.

Honk300 already has one validated ratatui settings editor, one local control protocol, and one
engine-owned graceful shutdown. Adding a separate native settings model or calling AppKit's
immediate application termination would split those contracts and could make the goose visibly
disappear instead of walking home.

## Decision

### One runtime-owned status item

The macOS overlay runtime creates one AppKit status item after `NSApplication` finishes launching.
It is present only for the lifetime of the running `Honk300.app` process and is removed when that
runtime is destroyed. The agent remains `LSUIElement=true`: no running Dock control surface,
ordinary app menu, native settings window, or AppleScript command surface is added.

The status item uses the readable title **Honk**, an accessibility tooltip, and two standard menu
items with explicit labels and tooltips:

- **Configure Honk300…**
- **Quit Honk300**

The bridge stays on AppKit's main thread. Because AppKit menu items do not retain their action
target, the Rust owner retains that target for at least as long as the menu and status item.
Commands are recorded as one-shot runtime intents and consumed by the existing macOS event loop;
Quit has priority if both actions are pending.

### Configure reuses the terminal TUI

`Honk300.app` includes a signed-bundle resource named `Configure Honk300.command`. AppKit opens
that resource through the normal workspace association, and the launcher executes the same
bundle's `Contents/MacOS/honk300 config` entry point. The existing schema-v2 TOML, validation,
save, status, start/stop, and IPC reload paths therefore remain authoritative.

There is no native settings schema, preferences window, duplicate save path, or simulated input.
A missing launcher or a failed workspace open produces an actionable error instead of silently
inventing another configuration route. Packaging and notarization gates require the executable
launcher in the staged app, final app ZIP, mounted DMG, and remounted stapled DMG.

### Quit reuses graceful shutdown

Quit sends the same engine-owned graceful-stop intent as the CLI/TUI path. The runtime continues
simulation and presentation until the goose and its effects have walked completely beyond a real
exposed edge and the final transparent frame is acknowledged; only then may the singleton be
released and the process end. The menu action must not call `terminate:`, `exit`, or otherwise
bypass lifecycle cleanup.

The existing terminal-window protection and same-user IPC rules are unchanged. The status item
does not grant Accessibility, weaken denied-mode behavior, or expose a network control surface.

## Consequences

- A graphical macOS user can launch Honk300 from Applications or a Dock shortcut, configure it
  through the existing terminal settings interface, and quit it from the menu bar without
  memorizing commands.
- There is still one configuration model and one graceful-shutdown implementation.
- The item remains available during the managed Accessibility wait, so a denied user retains an
  obvious Configure/Quit path while permission-bound behavior stays suppressed.
- Windows and Linux receive no tray or menu-bar UI; their CLI/TUI/IPC control contract is
  unchanged.
- Native verification must prove status-item lifetime, accessible labels, bundled-launcher
  integrity, terminal TUI launch, and fully animated Quit on the exact signed candidate.

## Verification

Automated contracts cover main-thread AppKit ownership, retained action-target lifetime,
Configure-versus-Quit command routing, graceful-stop intent, launcher contents and executable
mode, pre-sign bundle placement, and presence in every notarized distribution shape.

A local packaged universal app also passed a native interaction smoke: macOS accessibility
inspection exposed `Honk`, **Configure Honk300…**, the separator, and **Quit Honk300**; Configure
opened the complete existing TUI in a 120x30 Terminal and `q` restored it; Quit visibly walked the
goose beyond an exposed edge and the process ended four seconds later. The bundled launcher was
executable and the enclosing app signature still verified.

This ADR does not make v1.0.0 release-ready by itself. One exact signed candidate must still show
the item while running, open the existing TUI from Configure, complete a fully offscreen Quit
before singleton release, and pass the complete Accessibility, lifecycle, packaging, and native
compositor checklist.
