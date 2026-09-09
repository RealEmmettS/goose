# Changelog introductions before refinement closure

Historical text retained from source `0ff0ffa8f70787da13dec8282c1c12d273ac784f`.
These introductions describe an earlier project state, including old settings
navigation and acceptance limits. Current guidance lives in the repository README
and the versioned readiness records. Relative links inside the preserved text
refer to their original repository-root location.

## Technical introduction

```markdown
# Changelog

All notable changes to this project are documented here. Format based on
[Keep a Changelog](https://keepachangelog.com/); release versions follow
[Semantic Versioning](https://semver.org/).

> **Project stage: first stable release.** Milestones M0-M19 are implemented in-tree. The
> managed macOS Accessibility first run is implemented and one unchanged signed executable has
> passed denied, non-nagging relaunch, live-grant, and live-revocation on the physical M2; exact-
> final-SHA and unavailable-hardware/tooling limitations are retained as explicit forward-
> verification waivers. The goose now renders, walks, leaves mud, plays sounds, reacts to the
> cursor, can
> perform bounded cursor-nab mischief, can perch on user-dragged windows, and can collect
> owned note/image windows on Windows. It now enters/leaves through real exposed edges, occasionally
> wraps only while fully hidden, and can react when a user closes one of its collected windows.
> It can be controlled through a single-instance local IPC
> channel. It now has the three-name goose-speak CLI plus durable TOML configuration and the
> ratatui config TUI, dynamic moods, the local on-hour double honk, quiet-hours/DND/fullscreen
> manners, built-in Autumn leaves, Windows multi-monitor chase, and live appearance/recolor
> controls, plus macOS runtime/status/app-bundle staging and a menu-bar shortcut to the existing
> TUI/graceful Quit, Windows notification-area controls, Linux StatusNotifier controls and X11
> visible overlay support,
> native Wayland reduced-mode rendering, CI smoke gates, and M19 Windows/Linux lifecycle plus
> release packaging with artifact evidence. A plain-English companion lives in
> [HUMAN_CHANGELOG.md](./HUMAN_CHANGELOG.md) and must stay in lockstep.

```

## Plain-English introduction

```markdown
# Human Changelog

A plain-English companion to [CHANGELOG.md](./CHANGELOG.md). Every change in the technical
changelog has a layman's-terms version here. No version numbers, no code references — just
what changed and why.

For the technical version with file paths and exact details, see CHANGELOG.md.

> **Where the project is:** the goose is alive on screen. It appears on your desktop, walks
> around, reacts to your mouse, makes sounds, can steal the cursor in a short, bounded prank,
> can hop onto a window while you drag it around, and can now bring in note and meme windows.
> It can now be controlled through a local command channel for starting, stopping, reloading, and
> simple poke commands. It now understands the friendly three-name command grammar and has a
> terminal settings screen backed by a saved config file. It now has dynamic moods and a double
> honk at the top of each hour. It now also respects quiet times, fullscreen/DND manners, and
> seasonal Autumn leaves. It now supports Windows multi-monitor chasing and fuller appearance
> controls. Mac support and the Linux desktop paths are now in the codebase, with repeatable
> CI smoke proof for hosted Mac bundle checks and Linux desktop behavior. The installed Mac app
> now has a calm, non-nagging permission handoff. One unchanged signed copy passed denied,
> repeat-denied, granted, and revoked behavior on the physical Mac; a later fresh-release repeat
> remains visible follow-up work. While running, the Mac app also has a small goose menu that
> opens the same terminal settings screen or starts its animated goodbye.
> Windows and compatible Linux desktops now show the same small goose control as the Mac: it
> opens the one terminal settings screen or starts the Goose's animated goodbye.
> Every desktop now stages the
> Goose's arrival and departure beyond a real screen edge, and a person closing its note or meme
> can provoke a safely bounded annoyed reaction. The Windows/Linux installer
> and update
> work now has release artifact proof, including Windows installers for both regular x64 and ARM64
> machines.

---

```
