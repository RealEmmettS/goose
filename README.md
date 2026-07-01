# goose
Take the files in the Desktop Goose folder, analyze them thoroughly, and we're going to create an entirely new version of Desktop Goose. Same functionality, same features, but preferably built in Rust, works with both macOS, Linux, and Windows, and has native and CLI installers just like TR300, ND300, and WB300 (except we won't distribute with cargo, just via the builds and installers and scripts)

FOR REFERENCE:
"C:\Users\hey\git\qube-machine-report"
"C:\Users\hey\git\qube-network-diagnostics"
"C:\Users\hey\git\qube-workbranch-view"

---

## Status

**Stage:** implementation in progress. Milestones **M0-M15** are complete; **M16 macOS backend
and universal2 app packaging** is next. The current Windows build renders the procedural goose on the desktop,
walks it, leaves mud, plays sounds, reacts to pat/click input, can perform bounded cursor
nabbing when cursor warping is enabled, and can perch on a user-dragged foreign window until
release. It can also drag in Notepad and meme windows through the M9 collect-window dispatcher,
M10 adds a single-instance local control channel for `start`, `stop`, `reload`, and
`do <action>` pokes, M11 adds the three-name goose-speak CLI grammar, and M12 adds durable TOML
configuration plus the terminal config TUI. M13 adds deterministic dynamic moods and the local
on-hour double honk; M14 adds quiet-hours/DND/fullscreen calm suppression and built-in procedural
Autumn leaves. M15 adds Windows multi-monitor chase, per-monitor dirty-region presentation, live
Calm Goose, and full RGB editing for the original three-color goose palette. There is no installer
or release artifact yet.

**Canonical plan → [`honk300_plan.md`](./honk300_plan.md). Start here.** It is a claim-tested
*hybrid* that synthesizes the two earlier drafts — [`claude_plan.md`](./claude_plan.md) (the
structural spine) and [`codex_plan.md`](./codex_plan.md) (grafts) — after verifying each draft's
load-bearing claims against the original's shipped C# source and the `*300` sibling repos. Both
drafts are now **superseded reference only**; where the three conflict, `honk300_plan.md` wins.
(Why the hybrid is lopsided: `claude_plan.md`'s engine constants match `Exports.cs` verbatim,
while `codex_plan.md`'s guessed speed values were wrong.)

**Architecture decisions → [`docs/adr/`](./docs/adr/README.md).** ADRs record durable decisions
that should survive individual task-board updates. The first ADR closes M7's cursor-mischief
contract, cross-platform guardrails, and Renderer V2 direction.

**Decided direction (see `honk300_plan.md` for the full detail):**

- **Binary `honk300`**, installed under three names — `honk300` / `honk` / `goose` — with a
  finite "goose-speak" CLI (`honk plz` / `goose plz` to start, `honk bad` /
  `goose no honk` to stop, `goose do honk` to poke, `<name> config`, `<name> help`).
- **Clean-room procedural goose** (no sprite extraction); engine ported 1:1 from the verified
  constants. Sounds, screened original memes, and screened original notes are bundled 1:1 for
  personal use; M9 adds one complete custom in-house counterpart per copied meme/note original.
  Old donate pages and old developer references do not ship.
- **TOML config** + a **ratatui terminal config TUI** at `<name> config`. Current M0-M15
  settings hot-apply through reload where supported; future settings are persisted and shown
  as planned or restart-required until their milestones land.
- **New autonomous behaviors**, each an optional toggle, scoped to parameter-modulation of the
  procedural rig (no new copied goose art): dynamic moods, on-the-hour double honk,
  quiet-hours/DND/fullscreen manners, built-in Autumn leaves, multi-monitor chase, full
  original-palette recolor, and the Calm Goose valve are implemented. Default = full original
  prank, always-on.
- **No external mods** (Autumn is built-in; extensibility via documented internal seams),
  **no system tray**, and **no global quit key**. Starting, stopping, and configuration are
  CLI/TUI-only over the single-instance IPC channel.
- **Terminal windows are protected:** the goose may visually pass over terminals, but it must
  never move, focus, type into, drag, ride, collect, or otherwise manipulate terminal windows.
- **Built for every OS + architecture:** Windows x64 **and ARM64**, macOS Intel **and Apple
  Silicon** (universal2), Linux x64 **and ARM** (gnu + musl where packaging supports it).
  Native + CLI installers like TR300/ND300/WB300; **no crates.io**.
- Linux is **X11-first** (runs under XWayland); native Wayland is an opt-in `--wayland` mode with
  reduced mischief.
