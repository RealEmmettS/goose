TT;DR: Correct the airborne foot's stale walking target during delivery stops.

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
  pass with the correction; existing visual goldens remain unchanged.
- Criterion: visible defect. Oracle: run 34271715326, newer ARM delivered-note
  capture. Independent actual-engine sweep of 23,040 deliveries reproduces the
  stale lead; maximum below-body foot displacement changes from 31.730 to
  17.913 pixels. This is a diagnostic measurement, not an acceptance threshold
  or proof of the native capture's exact pixel cause.
- Criterion: final native/public behavior. Oracle: new native captures and
  same-source publication. Pending; cancelled candidates do not qualify it.

## Verification
- [x] Original production regressions fail and corrected stopping/contact checks pass.
- [ ] Inspect actual Rust stopping motion and complete native delivery captures.
- [ ] Pass final-source checks, immutable publication and fresh-public verification.

## Status
Active. Production correction and reproducing regressions are implemented.
Native review and both affected releases remain held.

## Activity
- 2026-09-08 — Diagnostic run 34278778497 passes all four native suites. All
  eight first-arrival images and aligned foot traces are normal, so this run
  does not reproduce or explain the earlier intermittent failure. Extend the
  private diagnostic branch to twelve deliveries per desktop with a bounded
  post-arrival capture sequence; keep product behavior and publication held.
- 2026-09-08 — All four native GNOME suites pass in 34276905894, but newer-ARM
  cycle-zero delivery still shows an overextended leg. Keep publication held;
  capture bounded actual foot/velocity state and screenshot timing in a separate
  diagnostic branch before deciding whether another gait or presentation change
  is warranted. The first correction resolves its demonstrated regression but
  does not establish complete native visual acceptance.
- 2026-09-08 — Inspect sixteen sequential frames around each front, rear and
  fast abrupt stop plus the existing gradual run-stop sequence. Settling feet
  retract without extending the old lead; no visual golden was changed. Full
  workspace checks and native captures remain the final-source gates.
- 2026-09-08 — Preserve the native art finding, cancel both pending candidates,
  reproduce stale airborne lead in the actual engine, and correct recovery under
  ADR 0050 without changing planted contacts or existing visual goldens.
