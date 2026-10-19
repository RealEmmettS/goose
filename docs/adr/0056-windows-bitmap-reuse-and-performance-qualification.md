# ADR 0056: Windows bitmap reuse and performance qualification

Date: 2026-10-19

Status: Accepted; implementation and bounded investigation complete, extended qualification deferred by the operator's 2026-10-19 follow-up.

## Context

The operator reported desktop slowdown, runaway settings scrolling and an intermittent
black rectangle around the goose and leaves. The accepted plan permits independently
verified fixes to ship while the original black-rectangle report remains unresolved;
it does not permit a reproducible new regression or failed performance budgets.

The Windows presenter allocated a cropped image for each monitor on every presentation
and replaced its DIB whenever exact drawing dimensions changed. Neither operation is
needed to preserve one overlay per monitor, signed coordinates and per-pixel alpha.

## Decision

Copy checked source rows directly from the shared premultiplied RGBA canvas into the
native premultiplied BGRA bitmap. Keep logical viewport dimensions separate from bitmap
capacity and row stride. Round each capacity extent to 32 pixels, grow when required,
and shrink an extent when it exceeds twice the rounded requirement. Initialize new
storage to transparent and overwrite every presented pixel, including newly exposed
rows and columns, before UpdateLayeredWindow. The window remains exactly the required
size. Retain natural hit-testing, fixed simulation timing and existing presentation caps.

Developer-only frame profiling is selected by HONK300_WINDOWS_PROFILE, with a maximum
of 65,536 retained samples. Ordinary startup does not record or add per-stage clocks.
Profile rendering, allocation, row copying and UpdateLayeredWindow separately. Native
process and available compositor/GPU counters come from the tracked PowerShell driver;
frame timestamps must be trimmed to the controller's actual measurement interval.
Long runs must disclose when their frame record covers only a retained trailing window.

Use isolated configuration without login-startup intent. Verify exact executable and
process ownership, preserve the user's config, startup registration and receipt, and
retain failure evidence. Never build or capture screenshots during timed comparisons.
The benchmark adds deterministic leaf, separated-leaf and scattering scenes to expose
costs hidden by goose-only scenes; renderer timing alone is not native presentation proof.

## Qualification contract

Use 30 seconds of warm-up and at least 120 seconds of samples for three alternating
baseline/candidate pairs per workload; use five pairs when variation exceeds ten percent.
The accepted budgets are at most 15% of one logical core for walking, 30% for sustained
effects, and 5% for settled settings. CPU render/presentation work must meet P95 <=8 ms
and P99 <=16.7 ms. No repeatable >10% median CPU or P95 presentation regression is allowed.
Report whole-machine CPU separately using the actual logical-processor count.

The packaged endurance run lasts at least thirty minutes and includes ten runtime and
settings cycles. Final-five-minute median private memory may grow by at most 5 MiB over
the preceding five-minute median; handles and graphics objects must plateau. Reproduced
desktop slowdown attributable to Goose blocks publication. Failed budgets require a
revised proposal to the operator, without relaxing budgets or reducing animation quality.

Settings wheel input must advance approximately 45-85 logical pixels per ordinary notch,
settle within 350 ms and remain stopped. Reduced motion uses a direct 40-pixel step.
Check fractional/repeated/reversed input, all pages, resizing and current display scaling.
The native probe reproduced stale accessibility geometry when a wheel reset clamped
at the boundary: the pinned SDK skipped publication and had no later kinetic step
to refresh it. A hash-pinned dependency edit requests the existing deferred publication
after changed wheel input. A failing-before regression checks boundary and reduced-motion
geometry after the responding frame, plus no publication churn during unchanged idle frames.

Native measurements also exposed a longer tail under delayed Windows frame delivery. The
SDK used its nominal frame interval for kinetic decay despite carrying actual timestamps.
Use measured completion gaps for scrolling only, rebased by changed wheel input; general
pointer/key input must not reset that clock. Retain nominal cadence for latency accounting.
Integrate delayed frames in bounded small steps before one geometry publication, expiring
old momentum after a long pause so elapsed time cannot cause a large positional jump.
Exercise the real runtime dispatch with irregular timestamps, resolved non-wheel input,
wheel input after idle, and a long pause in addition to the direct physics checks.

Investigate transparency in three five-minute scenario runs and ten start/stop cycles,
correlating accepted source pixels, visible output and native window/buffer metadata.
Capture stays bounded, local and separate from profiling; local full-desktop calibration
remains prohibited. Unreproduced does not mean fixed, and physical desktop acceptance
remains separate from screenshots and hosted compositor tests.

## Consequences

No public CLI, IPC or configuration-schema changes are needed. Preserve existing seasonal
preferences and the earlier approved settings, artwork and petting work. Complete source,
candidate, main, immutable publication and fresh-public qualification before release claims.

## Operator release decision — 2026-10-19

After the implementation and bounded transparency investigation, the operator explicitly
requested pushing and publishing the new version without waiting for extended performance
testing. Proceed after the final native scrolling check and required correctness, source,
candidate, unchanged-main and fresh-public gates. Defer the repeated performance comparison
and packaged thirty-minute endurance run under #g13. This changes their release-gate status;
it does not claim those tests passed or that the budgets above were demonstrated. Keep the
original black-rectangle investigation open under #g14 and preserve all collected evidence.
