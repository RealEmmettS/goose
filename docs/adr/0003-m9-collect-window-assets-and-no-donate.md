# ADR 0003: M9 Collect-Window, Asset Provenance, and No-Donate Scope

- **Status:** Accepted
- **Date:** 2026-06-27
- **Milestone:** M9

## Context

M9 adds the original goose-style collect-window behavior, but the repo has two constraints that
must be explicit before implementation:

- `honk-engine` must stay platform-free and `#![forbid(unsafe_code)]`.
- This is a personal-use rebuild from an old defunct project, so old developer donation pages and
  old-project identity references must not carry forward.

Earlier planning also conflicted on assets. The accepted M9 policy is now that screened original
memes and notepad messages may be bundled 1:1 for the owner's personal-use builds, but each copied
meme/note original must also have one complete custom in-house counterpart in the project's clumsy
MS Paint house style. User-supplied `Meme8.png` is approved.

## Decision

M9 implements **Notepad + meme only**. Donate is removed from M9 and from future command/config
surfaces unless explicitly reintroduced as a new owner-specific feature.

The engine receives a platform-neutral collect-window contract:

- `CollectWindowKind::{Note, Meme}` only.
- Opaque request/window IDs, content indexes, capabilities, ordered commands, and snapshots.
- No HWNDs, process handles, paths, image decoding, Notepad process control, or synthetic input in
  `honk-engine`.

The Windows runtime/backend owns OS effects:

- Notepad spawn, PID-to-HWND lookup, movement, focus verification, and Unicode `SendInput`.
- Owned non-topmost image windows for meme props.
- `SetWindowPos`, pass-through toggling, snapshot polling, and close handling.

Asset layout records provenance:

- `Assets/Images/Memes/originals/`
- `Assets/Images/Memes/custom/`
- `Assets/Images/Memes/user/`
- `Assets/Text/NotepadMessages/originals/`
- `Assets/Text/NotepadMessages/custom/`

Copied originals must be screened for old developer names, donation/Patreon links, social handles,
and old-project branding. Old donate pages do not ship.

If an original candidate fails screening, it is excluded rather than redacted so the provenance of
the remaining copied originals stays honest. For M9, the reference app's `Meme2.png` is excluded
because it contains a visible social handle watermark; its custom in-house counterpart remains
safe to ship.

## Consequences

- macOS, X11, and native Wayland future backends plug into the same capability model.
- Native Wayland reports unsupported for foreign-window movement and keystroke synthesis rather
  than special-casing behavior inside the engine.
- M10/M11 may route future `do meme` / `do note` commands to this same engine action shape.
- 32-bit x86 remains outside the canonical target matrix; engine types must stay architecture
  neutral.

## Verification

M9 requires:

- Engine tests for note/meme success, missing assets, unsupported capabilities, closed windows,
  capability loss, command ordering, movement coalescing expectations, and interrupt precedence.
- Windows smoke for forced Notepad typing and forced meme dragging, including copied originals,
  custom counterparts, and user-supplied `Meme8`.
- Full local gate plus installed target checks for Windows x64/ARM64, macOS x64/ARM64, Linux GNU
  x64/ARM64, and Linux musl x64/ARM64.
