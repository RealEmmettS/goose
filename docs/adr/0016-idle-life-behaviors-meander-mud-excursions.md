# ADR 0016 — Idle-Life Behaviors: Meander, Story-Driven Mud, Off-Screen Excursions

- Status: Accepted (2026-07-07, Emmett's live review of Renderer V2)
- Relates to: ADR 0014 (Renderer V2), ADR 0004 (control plane), ADR 0007 (moods)
- Owner: Fable (orchestrator), implemented hands-on per Emmett's direction

## Context

With the V2 goose looking right, Emmett's live review turned to how it *behaves*:
straight-line walks with sharp corner turns read as robotic; mud tracking fired on a
50 % coin-flip at every wander waypoint (so the goose was muddy nearly constantly,
with no story for where the mud came from); and the goose never left the screen — no
sense of a life beyond the desktop. Direction: "attention to detail … like you went
and walked in a mud puddle off screen and came back."

## Decisions

### 1. Wander paths meander (goose-chaos walking)

Casual walks (`wander`, `excursion`) get a smoothly-varying lateral offset: a hidden
sine (2.2 rad/s) times a smoothed amplitude that re-rolls every 0.8–2.2 s from the
world RNG, projected perpendicular to the to-target direction, up to ±48 px, fading
to zero within 25–145 px of the target so arrivals still land exactly. Straight
lines become wandering curves; corners round off. Purposeful movement (nab, charge,
perch, collect approach) stays direct. Implemented as a locomotion-step target
perturbation in the world — tasks and the locomotion integrator are untouched, and
everything stays deterministic per seed.

### 2. Mud comes from somewhere (puddle hops)

`WanderTask` no longer starts mud tracking. Instead, every ~70–160 s the goose takes
a **puddle hop**: it waddles just past a screen edge, disappears for 8–15 s, comes
back near where it left — and tracks mud for 30–90 s. The mud now has a narrative
(it found a puddle out there) and muddy feet are an occasional event, not a constant
state. `do mud` still forces mud immediately (explicit poke).

### 3. Long errands with prank returns

Every ~4–7 minutes (240–420 s) the goose takes an **errand**: it waddles off-screen —
preferring the nearest horizontal edge (80 %), occasionally the far side (10 %) or a
vertical edge (10 %) — stays gone 90–120 s, and reappears from a random horizontal
edge point before walking back in. **40 % of errands return with mischief**: a
collect-window prank (note/meme) chains immediately on arrival, when collect is
enabled and supported. On a multi-monitor virtual desktop the "edges" are the outer
union edges, so the goose genuinely leaves the whole desktop.

Mechanics: both excursion kinds are one `ExcursionTask` (Depart → Away → Return)
installed through the existing interrupt slot over plain wandering only — never over
mischief-in-progress, FirstUX, or during manners (quiet hours/DND/fullscreen). The
suspended wander resumes afterwards (after the chained collect, when there is one).
While Away the goose is parked off-screen and still ticks; IPC stays live (`status`,
`stop`, pokes answer normally — a `do honk` while away is a distant honk, which is
charming and allowed). All parameters live in `TimingOptions` (engine defaults; not
yet config-exposed — deliberate, to keep the config surface stable until the cadence
is proven in daily use).

### 4. Stop-synonym grammar

`<name> exit` and `<name> quit` normalize to `stop` alongside `bad` / `no` /
`no honk` (M11 grammar extension). Verified live: `honk300 exit` stops a running
goose; stop-family commands against no instance keep the existing nonzero
"no running goose instance" behavior.

## Consequences

- The goose reads alive: curvy walks, occasional muddy returns, and periodic
  disappearances that set up "what is it doing out there?" — with pranks sometimes
  answering that question.
- The default mud frequency drops dramatically (from ~half of all waypoints to a
  timed 1–2.5/minute *event* cadence with a story).
- The status/render loop is untouched; excursions ride existing task/interrupt
  machinery, so capability gating, manners, and IPC behavior are inherited.
- New deterministic tests pin: a full off-screen round trip, puddle-hop mud
  delivery, wandering-alone-never-muds, and the existing same-seed determinism.

## Verification

- Full local gate green (fmt / clippy -D warnings / 237 workspace tests incl. the
  three new behavior tests).
- Live Windows smoke: meander visible as a serpentine mud trail; `exit`/`quit`
  verified against a running instance; no zombie after stop.
- Cadence values are first-pass; expected to be tuned after a few days of real use.
