# ADR 0014 — Renderer V2: Flat-Illustration Procedural Vector with a Dual-View Rig

- Status: Accepted (2026-07-07)
- Supersedes: the "Renderer Direction" section of ADR 0001 (the M7.2 sprite/atlas
  blitter plan recorded there and in board task `#r2v`)
- Folds in: board task `#c0d` (goose rendering/UI polish)
- Owner: Fable (orchestrator), per Emmett's 2026-07-07 direction

## Context

The M0–M19 goose was procedurally drawn from capsules and a hand-tuned bezier body.
The 2026-07-07 full-repo evaluation found the *technique* sound (tiny-skia CPU vector,
anti-aliased, premultiplied RGBA — exactly what every platform present path consumes)
but the *artistry and animation* crude: stateless sine feet that slide ("moonwalk"),
a straight-capsule neck, whole-body rotation at any heading (a side-profile goose
rotated 90° when walking vertically), un-damped mood posture snaps, and no secondary
motion. ADR 0001 had queued a "custom CPU sprite/atlas blitter" as Renderer V2, but
that direction conflicted with the plan §5.2 no-sprite-art guardrail, would re-bake on
every palette edit (breaking M15 live RGB editing), and needs many baked angles for
rotation.

Emmett supplied his own Quiver AI-generated flat-design goose SVGs (side profile, head
variants, and a top-down view) as the quality bar: layered slate wing with scalloped
feather notches, two-tone beak with nostril, soft body shading, separate two-tone legs
with webbed feet, tall tapered S-neck. "Doesn't need to be exact — that level of
effort."

## Decision

### 1. Procedural vector stays; the sprite/atlas direction is superseded

The runtime goose remains drawn per frame with tiny-skia (the goose bounding box is
~120 px; the cost is trivial even supersampled). ADR 0001's other renderer constraints
stand unchanged: premultiplied pixels out, no Vello/wgpu/Skia/game frameworks, platform
crates receive pixels only.

### 2. Reference art: provenance and use

- Emmett's reference SVGs are committed under `docs/art-reference/` (`goose-side-main`,
  `goose-side-head-left`, `goose-side-alt`, `goose-top-down`).
- They are **his own generated art**: adapting their path geometry directly into code
  is permitted and done. The original Desktop Goose app's assets remain off-limits
  (clean-room rule unchanged), and plan §5.2 is reconciled as: *no sprite/bitmap goose
  assets ship or load at runtime; the SVGs are design-time reference only; the code is
  the artifact.* `honk-engine` gains no SVG-parsing dependency — path data was
  transcribed offline (absolutizer script) into `PathBuilder` calls.

### 3. Dual-view rig (replaces rotate-the-profile-anywhere)

- **Side profile** for shallow headings and all idle/interaction: strictly horizontal,
  mirrored left/right by heading sign (dead zone |forward.x| ≤ 0.2 keeps the last
  facing), never rotated.
- **Top-down view** for steep headings: rotates freely to any heading, with a
  gait-synced waddle roll (±~3°).
- Switching uses **hysteresis**: enter top-down past **55°** from horizontal, return
  below **45°**; only a moving goose (speed > 1 px/s) can switch. A **125 ms
  crossfade** (`VIEW_FADE_SECS`) blends views; `GoosePose { primary, fading:
  Option<(Rig, alpha)> }` carries both, and dirty rects use the **union** of both
  views' bounding boxes so the fade is never clipped.

### 4. Layered compositing + 2x supersampling

Each view renders into its own **supersampled layer** (`GOOSE_SUPERSAMPLE = 2.0`) that
is composited onto the destination with per-layer opacity (bilinear, 0.5x). Rationale:
(a) crossfading whole layers blends the goose as one object instead of stacking
translucent parts; (b) the fine detail (feather scallops, nostril, webbed toes) stays
crisp at the 60–90 px on-screen size; (c) the compositor is one `draw_pixmap` — no new
architecture. Painter order per view follows the reference art back-to-front; the thin
outline is kept (reference art is outline-less, but a desktop pet needs contrast on
arbitrary backgrounds) and drawn under fills so overlapping parts cover interior seams.

### 5. Stateful plant-and-swing feet (the moonwalk fix)

`feet.rs` replaces the stateless sine gait with `FeetState`: each foot is **planted in
world space** and swings only when it lags more than `wantStepAtDistance` (5 px) behind
its home under the body, overshooting by `overshootFraction` (0.4), one foot airborne
at a time, swing duration = the entity step interval (task-set, else by speed tier),
sinusoidal lift, smoothstep travel, velocity-led targets, and a 60 px teleport snap.
The C# `ProceduralFeets` update math is closed-source, so this is a **clean-room
stateful reconstruction guided by the verified constants** (6 / 5 / 0.4 — still
pinned by test). The invariant *a planted foot's world position never changes while
the body moves* is pinned by `planted_feet_never_slide`. **Footmarks now stamp at
actual plant events** (`drain_plants`) instead of a gait-phase counter.

### 6. Rig/animation state model

- `RigAnim` (persistent, on `GooseEntity.anim`): feet state, view state + fade,
  side-facing mirror, eased neck, breath phase, blink timing, tail-flick energy.
- `RigInput` (per tick, world → rig): center, heading, neck target, speed/velocity,
  step time, clock. `RigInput::static_pose` + zero `dt` snaps channels for
  deterministic test poses (`Rig::update(center, dir, neck, _)` remains as that shim).
- `Rig` keeps every V1 attach-point field and meaning (`beak_tip` for nab/collect,
  `neck_head` for hearts/pat, `body_center` anchors, `feet`, `bounding_box()`), so
  the task layer needed **zero** changes.
- **Eased neck**: exponential smoothing at 10/s replaces the former direct handoff —
  mood posture changes glide instead of snapping (`neck_eases_instead_of_snapping`).
- **Idle posture**: the world now targets `0.45 + 0.25·speed_frac` before mood
  modifiers (was `0.4·speed_frac`, i.e. fully tucked at idle) — a goose stands tall.
- **Secondary motion**: blink (world schedules from its RNG every 2.0–6.5 s for
  determinism; the rig animates a 60 ms close / 90 ms open lid), idle breathing
  (0.55 Hz, fades out with speed), honk tail-flick (set when a honk sound is emitted,
  0.35 s decay, lifts the tail highlights), body bob from swing progress.

### 7. Six-tone configurable palette (extends M15's three)

`RenderPalette` fields and reference-derived defaults:

| Tone | Default | Used for |
|---|---|---|
| `goose_white` | `#ededed` | body, neck, head, tail highlights |
| `goose_shade` | `#c6c6c6` | throat, belly, thigh, tail-underside shading |
| `goose_wing` | `#515557` | layered wing, eye |
| `goose_orange` | `#fc7927` | beak top, near leg/foot |
| `goose_orange_dark` | `#d1551b` | beak underside/nostril, far leg/foot |
| `goose_outline` | `#c9c9c9` | thin contrast outline |

Config compatibility: `[colors]` keeps `goose_white/orange/outline` (defaults updated
to the reference tones) and gains **optional** `goose_shade`, `goose_wing`,
`goose_orange_dark`. Absent keys derive coherently from the legacy three via
`RenderPalette::from_legacy` (shade = mix(white, outline, 0.55); orange_dark =
darken(orange); wing = default), so pre-V2 config files load unchanged and only gain
explicit keys when the user edits those tones (the TUI materializes on first edit).
The TUI Appearance tab exposes all six tones as R/G/B rows.

### 8. Tests and goldens

Golden frames were **re-blessed** for the new art and expanded from three to six:
`side_rest`, `side_reaching`, `side_left` (mirror), `side_mid_stride` (deterministic
walk), `top_down`, `top_down_diag`. Because goldens are self-generated, behavior is
additionally pinned by numeric tests: no-foot-slide invariant, step alternation +
plant events, view hysteresis (one transition per band crossing), crossfade completes
within `VIEW_FADE_SECS`, pose-union bounding boxes, eased-neck rate, blink lifecycle,
per-tone palette effect, legacy-palette derivation. A preview harness
(`examples/preview.rs`: contact sheet / zoomed strip / walk strip with world-fixed
ground ruler) is the visual-tuning loop.

## Consequences

- The goose's default look changes (softer off-white, slate wing) — intended; the old
  pure-white/orange look remains reachable through the palette config.
- The three old golden PNG names are gone; the website's frame rail (`#r4w` R4.4) and
  hero art will be refreshed to the V2 look.
- Runtimes render via `render_pose_with_palette(world.pose(), …)`;
  `render_rig_with_palette` remains for single-view tools/tests.
- Board `#r2v` is redefined to this ADR; `#c0d` is closed into it.
- Future spicy/gaggle work can reuse `RigAnim`/`GoosePose` for extra geese.

## Verification

- Full local gate green at time of acceptance: `cargo fmt --check`, `clippy
  --all-targets --workspace -D warnings`, `cargo test --workspace` (234 tests, 6
  goldens re-blessed).
- Preview-harness review against the reference art (contact sheet, zoomed strip,
  8-frame walk strip with ground ruler — no foot slide observed).
- Pending before round close: Windows on-screen visual smoke (all 8 directions, nab,
  collect, moods) with screenshots, per repo convention.
