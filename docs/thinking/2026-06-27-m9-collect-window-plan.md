# M9 Collect-Window Dispatcher

**Status:** implemented 2026-06-27  
**Scope:** Notepad + meme only; donate removed.  
**Durable decision:** see `docs/adr/0003-m9-collect-window-assets-and-no-donate.md`.

## Final Decisions

- M9 implements `CollectWindowKind::{Note, Meme}` only. No donate enum variant, asset, config
  field, smoke path, or future CLI action is introduced.
- Asset policy is originals plus custom counterparts: screened original meme/note assets are
  copied 1:1 for personal-use builds, and every copied original gets one complete custom
  in-house counterpart in the clumsy MS Paint house style.
- `Assets/Images/Memes/user/Meme8.png` is approved user-supplied content and participates as a
  draggable meme prop without needing a counterpart.
- Old developer donation pages, donation links, Patreon references, social handles, and old-project
  branding are excluded.
- Original candidates that fail screening are excluded rather than redacted; the reference
  `Meme2.png` is excluded because it contains a visible social handle watermark.

## Architecture

- `honk-engine` owns the platform-neutral collect-window contract: opaque IDs, content indexes,
  capabilities, ordered commands, snapshots, `CollectWindowTask`, and world drain/feed APIs.
- The engine never sees HWNDs, process handles, image paths, image decoding, Notepad, Win32 calls,
  or synthetic-input APIs.
- The Windows backend/runtime owns Notepad spawn/discovery, image prop windows, `SetWindowPos`,
  pass-through toggling, focus verification, and Unicode typing.
- Collect commands are drained in order. Spawn/focus/type/close are not coalesced; move commands
  may be coalesced by a future runtime optimization but do not need to be.

## Assets

- Original memes: `Assets/Images/Memes/originals/`
- Custom meme counterparts: `Assets/Images/Memes/custom/`
- User memes: `Assets/Images/Memes/user/`
- Original notes: `Assets/Text/NotepadMessages/originals/`
- Custom note counterparts: `Assets/Text/NotepadMessages/custom/`

The custom image prompt family:

> Draw {MEME_IMAGE_PROMPT} in the most clumsy, scribbly, and utterly pathetic way possible. Use a
> white background, and make it look like it was drawn in an old computer painting program with a
> mouse. It should be vaguely similar but also not really, kind of matching but also off in a
> confusing, awkward way, with that low-quality pixel-by-pixel feel that really emphasizes how
> ridiculously bad it is. Actually, you know what, whatever, just draw it however you want.

## Cross-Platform Requirements

- Windows x64 is the runtime acceptance platform for M9.
- Windows ARM64 must compile with the same backend.
- macOS x64/ARM64, Linux GNU x64/ARM64, and Linux musl x64/ARM64 must continue to compile with
  honest fallback behavior.
- Native Wayland must remain represented as unsupported for foreign-window movement and keystroke
  synthesis through capabilities, not compile-target special cases inside the engine.
- 32-bit x86 is not part of the current canonical target matrix.

## Verification Checklist

- Engine tests cover note/meme success, missing assets, unsupported capabilities, spawn timeout,
  user-closed windows, capability loss, command ordering, and interrupt suppression.
- Windows smoke covers `HONK300_SMOKE_COLLECT=note` and `HONK300_SMOKE_COLLECT=meme`.
- Full gate: `cargo fmt --all -- --check`, `cargo clippy --all-targets --workspace -- -D warnings`,
  `cargo test --workspace`, and `cargo build --release`.
- Installed target checks cover the eight canonical triples currently installed locally.
