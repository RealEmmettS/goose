TT;DR: Correct stale airborne predictions during delivery stops and reversals.

## Why
Final native delivered-note review exposed a stretched leg after arrival. The
actual production gait continues an airborne walking prediction after stopping.

## Scope
Withdraw that prediction while preserving planted contacts, step timing, lift,
geometry bounds and unchanged compatibility constants. ADR 0050 and the actual
engine are authoritative. Include the correction in the two pending releases;
never replace an existing public artifact. Physical-device acceptance remains
separately tracked. The approved plan and AGENTS own the native and release gates.

## Plan
Reproduce in actual FeetState and World delivery flows, correct the remaining
swing, inspect the Rust motion export, then repeat native captures and complete
same-source publication gates. If two native cycles still show the stretch,
revisit the diagnosis before any further candidate or publication attempt.

## Acceptance
Stopped feet no longer extend the old walking prediction, planted contacts do
not slide, steps finish on time, and actual native deliveries render correctly.
The corrected source passes every required release gate and fresh-public check.

## Evidence
- Criterion: production stopping behavior. Oracle: FeetState phase sweep and
  actual World delivery regression. Both fail against the original gait and
  pass with the final correction. Only the two inspected turn fixtures changed.
- Criterion: visible defect. Oracle: run 34271715326, newer ARM delivered-note
  capture. Independent actual-engine sweep of 23,040 deliveries reproduces the
  stale lead; maximum below-body foot displacement changes from 31.730 to
  17.913 pixels. This is a diagnostic measurement, not an acceptance threshold
  or proof of the native capture's exact pixel cause.
- Criterion: final native/public behavior. Oracle: new native captures and
  same-source publication. Complete in the final readiness records; cancelled
  candidates remain historical evidence and do not qualify later changes.

## Verification
- [x] Original production regressions fail and corrected stopping/contact checks pass.
- [x] Inspect actual Rust stopping motion and complete native delivery captures.
- [x] Pass final-source checks, immutable publication and fresh-public verification.

## Status
Done. Actual World regressions reproduce and correct stale airborne lead
during stops and reversals while preserving planted contacts and step timing.
Eight Rust motion sequences and all 48 repeated native delivery captures are
reviewed; the final diagnostic uses unchanged production engine/runtime code.
The correction is published and fresh-public verified in v1.9.0 and v1.10.1.
Only the two inspected turn fixtures changed; other goldens and all independent
alpha, clipping, channel and geometry limits remain intact.

## Activity
- 2026-09-08 — Complete the ordered publication, fresh-public and website gates. See the final records in docs/readiness/v1.9.0-readiness.md and docs/readiness/v1.10.1-readiness.md. Preserve prior failures below as qualification history and keep unavailable physical acceptance separate.
- 2026-09-08 — Final local formatting, strict workspace clippy, complete Rust
  tests, release build, 138 Python checks (three platform skips), eight production
  JavaScript checks and separate Native SDK test/check/build pass. Review final
  stopping/reversal output and freeze the correction for same-source native gates.
- 2026-09-08 — The expanded native diagnostic 34280092759 independently captures
  the same stale-prediction failure on both Debian architectures, reaching
  43.600 pixels below the stopped body. Keep bounded opt-in foot/capture timing
  in the normal two-cycle fixture. Inspect the exact old/new turn images before
  updating those two fixtures; standing/straight-walking fixtures, tolerances and
  gait bounds remain unchanged. Same-source native qualification remains open.
- 2026-09-08 — Keep the airborne landing current through changing velocity and
  retain the existing travel cap during acceleration. All four actual World
  delivery cases and existing gait/contact/timing checks pass. Review twenty-four
  successive frames in each of eight affected motion sequences, including the
  two native-height World witnesses. The wider 156,672-delivery diagnostic changes
  its largest below-ground foot displacement from 37.240 to 20.323 pixels; the
  original long-leg witness changes to 2.700 pixels. These measurements do not
  replace native visual qualification. Full checks and fresh captures are next.
- 2026-09-08 — Expand the native-height World probe and reproduce the visible
  long rear leg with seed 1788900900000000000, delay forty ticks and note origin
  (430, 320). Reversal skips a zero-speed tick; the old swing lands behind and
  steps twice, delaying the opposite planted contact. The added actual delivery
  regression fails the existing twenty-six-pixel gait bound on the first fix.
  Withdraw the stale prediction on reversed velocity too, preserving contacts
  and lift timing. Native and public qualification remain held.
- 2026-09-08 — Inspect sixteen sequential frames around each front, rear and
  fast abrupt stop plus the existing gradual run-stop sequence. Settling feet
  retract without extending the old lead; no visual golden was changed. Full
  workspace checks and native captures remain the final-source gates.
- 2026-09-08 — Preserve the native art finding, cancel both pending candidates,
  reproduce stale airborne lead in the actual engine, and correct recovery under
  ADR 0050 without changing planted contacts or existing visual goldens.
