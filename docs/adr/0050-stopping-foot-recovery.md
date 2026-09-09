# ADR 0050: Keep airborne landings current through stopping and reversal

Date: 2026-09-08
Status: accepted; published and fresh-public verified in v1.9.0 and v1.10.1.

## Context

ADR 0041's continuous rig preserves planted contacts and predicts each moving
foot's landing ahead of the body. A native GNOME delivered-note capture in run
34271715326 showed a stretched orange leg after arrival. A production World
delivery probe independently reproduces the same class of stopping defect:
an airborne foot continues toward its frozen walking prediction after velocity
becomes zero. Across 23,040 delivered-note scenarios, the largest ground-space
foot displacement below the body was 31.730 pixels. That probe does not establish
the exact cause of every pixel in the native screenshot.

The first correction still leaves a stretched planted leg in native run
34276905894. A wider production World probe using actual native-height note
placement reproduces that silhouette: seed `1788900900000000000`, forty ticks
before the interrupt, and initial note top-left `(430, 320)`. During pickup the
velocity reverses from approximately `+5` to `-6` pixels/second between ticks,
never triggering a zero-speed sample. The old airborne prediction lands behind
the returning body and steps again, delaying recovery of the other planted foot.
At delivery that foot is 37.240 pixels below the ground point and 35.767 pixels
from its resting home. The existing Run/Charge bound is 26 pixels from home.
The expanded native diagnostic in run 34280092759 independently captures the
same stale-prediction sequence on Debian GNOME x64 and ARM64, with stopped foot
displacements up to 43.600 pixels. Aligned screenshot/state evidence establishes
an engine defect rather than an inference about retained desktop pixels.

## Decision

When velocity changes while airborne and moving, predict the landing from the current home, the
original overshoot bias, and actual velocity through the remaining swing plus
half the following stance. At steady velocity this is the original fixed target;
braking, acceleration and reversal update it before the foot lands behind the
body and unnecessarily steps again. Retain a fixed target between velocity
changes so floating-point re-evaluation does not perturb ordinary straight
walking. Reapply the existing body-travel cap as
speed increases within a swing, shortening future progress intervals without
resetting the accumulated phase. Keep the original weighted duration at steady
speed and never lengthen an active step while slowing down.

When the existing gait switches to stationary recovery, withdraw the airborne
motion lead once. Start its remaining horizontal interpolation at the current
foot position and end at the current resting home. Preserve current step and
vertical lift progress. Do not move planted contacts, add a leg clamp, reset
lift, or change compatibility constants. The existing stationary recovery still
settles the other foot. Guard the near-complete eased fraction so floating-point
rounding cannot introduce an invalid division.

## Verification

Actual FeetState regressions exercise three speed tiers, four headings and forty
stopping phases, checking withdrawn lead, unchanged planted contacts and settled
feet. An actual World regression exercises delayed native note creation,
position feedback, delivery commands and stopping recovery for the original two
production scenarios. Both regression families fail against the previous gait
and pass after the first correction. That broad delivery probe reduced its largest
below-body displacement to 17.913 pixels; it is diagnostic evidence, not a new
geometry threshold. The later velocity-change correction intentionally updates
the two turn fixtures after direct before/after inspection; the four standing
and straight-walking fixtures and every comparison tolerance remain unchanged.

The native-height reversal witness additionally fails against the first stopping
correction because its planted foot exceeds the existing gait bound. Include it
in the actual World regression and review its rendered approach and settling
sequence; do not replace that bound with the failing displacement. The final
World regression covers the original two cases and both native-height witnesses,
with dynamic moods enabled and unchanged planted contacts checked throughout.
The wider 156,672-delivery diagnostic changes its largest below-ground foot
displacement from 37.240 to 20.323 pixels; the original long-leg witness changes
to 2.700 pixels. These are diagnostic measurements, not acceptance thresholds.

The Rust preview adds front, rear and fast abrupt-stop sequences plus actual
World delivery-reversal and short-return sequences. Review their
actual frames and repeat native delivered-note captures before qualification.
The complete same-source architecture, settings, candidate, main, immutable
publication and fresh-public gates remain required by the approved plan.

## Consequences

This shared engine correction belongs in the pending GNOME release and the
following presence release. Candidates 34271545636 and 34271817809 were cancelled
after the visual finding; their completed jobs do not qualify the changed source.
Track native review and publication in task #rst and audit R13. Already published
tags and artifacts remain immutable.

## Publication evidence

The final unchanged-source candidate, main, immutable publication, fresh-download and website results are recorded in [the GNOME release](../readiness/v1.9.0-readiness.md) and [the final manners release](../readiness/v1.10.1-readiness.md). Earlier pending statements and cancelled candidates above describe the original qualification sequence. Physical acceptance retains its separate limits.
