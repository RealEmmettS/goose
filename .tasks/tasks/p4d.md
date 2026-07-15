TT;DR: The old fullscreen-present concern is resolved by bounded visual damage and one clipped layered overlay per monitor.

## Status

Done on 2026-07-12. M15 and the later stabilization work superseded the M7-era fullscreen redraw
shape that originally created this follow-up.

## Evidence

- `World::visual_bounds` and `World::damage_bounds` retain only current and immediately previous
  visible pixels, so damage does not accumulate across frames.
- The Windows runtime paints a dirty-sized canvas rather than a full desktop canvas.
- The Windows backend owns one layered window per monitor and clips each dirty world region to the
  intersecting monitor before `UpdateLayeredWindow`.
- Focused damage tests include a 4K non-accumulation case; Windows x64 and ARM64 target checks pass.
- The v1.0.0 Mac profile independently meets its present-cost envelope at 8.30% median CPU and
  54.48 MiB maximum RSS after warm-up.

## Activity

- 2026-07-12 20:15 - Reconciled the stale card with M15/R5 implementation evidence, reran bounded
  damage and Windows cross-target checks, and recorded the Mac performance result (agent: codex).
