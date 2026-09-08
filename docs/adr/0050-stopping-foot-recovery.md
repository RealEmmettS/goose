# ADR 0050: Withdraw airborne motion lead when locomotion stops

Date: 2026-09-08
Status: Accepted; native qualification and publication pending.

## Context

ADR 0041's continuous rig preserves planted contacts and predicts each moving
foot's landing ahead of the body. A native GNOME delivered-note capture in run
34271715326 showed a stretched orange leg after arrival. A production World
delivery probe independently reproduces the same class of stopping defect:
an airborne foot continues toward its frozen walking prediction after velocity
becomes zero. Across 23,040 delivered-note scenarios, the largest ground-space
foot displacement below the body was 31.730 pixels. That probe does not establish
the exact cause of every pixel in the native screenshot.

## Decision

When the existing gait switches to stationary recovery, withdraw the motion lead
of an already airborne step once. Start its remaining horizontal interpolation
at the current foot position and end at the current resting home. Preserve the
original step progress, duration and vertical lift phase. Do not move planted
contacts, add a leg clamp, reset the lift, or change compatibility constants.

Ordinary moving steps retain their interpolation and cadence. The existing
stationary recovery still settles the other foot. Guard the near-complete eased
fraction so floating-point rounding cannot introduce an invalid division.

## Verification

Actual FeetState regressions exercise three speed tiers, four headings and forty
stopping phases, checking withdrawn lead, unchanged planted contacts and settled
feet. An actual World regression exercises delayed native note creation,
position feedback, delivery commands and stopping recovery for two captured
production scenarios. Both regression families fail against the previous gait
and pass after correction. The same broad delivery probe reduces its largest
below-body displacement to 17.913 pixels; it is diagnostic evidence, not a new
geometry threshold. Existing visual goldens remain unchanged.

The Rust preview adds front, rear and fast abrupt-stop sequences. Review their
actual frames and repeat native delivered-note captures before qualification.
The complete same-source architecture, settings, candidate, main, immutable
publication and fresh-public gates remain required by the approved plan.

## Consequences

This shared engine correction belongs in the pending GNOME release and the
following presence release. Candidates 34271545636 and 34271817809 were cancelled
after the visual finding; their completed jobs do not qualify the changed source.
Track native review and publication in task #rst and audit R13. Already published
tags and artifacts remain immutable.
