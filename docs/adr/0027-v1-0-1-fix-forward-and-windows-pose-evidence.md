# ADR 0027 — v1.0.1 Fix-Forward And Windows Pose-Complete Evidence

- Status: Accepted (2026-07-15)
- Relates to: ADR 0014 (dual-view renderer), ADR 0018 (immutable atomic publication), ADR 0020
  (Developer ID macOS distribution), ADR 0023 (rolling latest artifacts), ADR 0025 (first stable
  v1 release), and ADR 0026 (hosted Windows ARM64 evidence boundary).
- Supersedes only: ADR 0025's prospective `v1.0.0` public tag and ADR 0026's assumption that a
  qualifying Windows frame must be side-view. All technical release, signing, packaging,
  updater, hardware-verification, and hosted-ARM64 boundaries remain accepted.

## Context

The exact `v1.0.0` tag points to candidate-proven/default-branch commit
`9c5692b32bb256d3008308c83d76ddebd7fb44df`. Its release workflow failed closed before creating
a draft. Every producer except the Windows x64 extra-installer job passed, including the signed,
notarized, and stapled universal macOS app/DMG. The sole failure retained ten complete captures
of a correct top-down goose. Those frames proved controlled transparency, no opaque-black
surface, body, shade, outline, wing, asymmetric warm beak, and antialiased edges, but the analyzer
required side-view-only legs and a ground shadow. Runtime logs showed successful wander commands
and clean lifecycle behavior. No `v1.0.0` GitHub Release, draft, or public asset exists; stable
latest remains v0.3.2.

ADR 0018 forbids moving or rebuilding a consumed immutable tag. Re-running until random movement
selects a side pose would also preserve a probabilistic release gate that rejects one of the
renderer architecture's two intentional views.

## Decision

- The failed `v1.0.0` tag remains immutable and unpublished. The corrected first public stable
  release is `v1.0.1`; source, bundle, receipt, manifest, installer, updater, readiness, handoff,
  and website identities advance together.
- Candidate and default-branch gates must pass again on one exact `v1.0.1` commit before its
  single immutable tag is created. Release mode must still rebuild, verify, and publish the full
  platform matrix atomically. There is no release-mode rerun or asset mutation for `v1.0.0`.
- Windows paired-DWM and hosted-ARM64 presenter-surface analysis accept either of the renderer's
  two strict semantic view profiles after the common bridge/composition checks pass:
  - **Side view** requires the established two-tone asymmetric orange, spatially separated beak
    and leg assembly, and semi-transparent ground shadow.
  - **Top-down view** requires one compact true-orange beak, no dark-orange leg palette, at least
    a 60-percent wing-to-body palette ratio, shade no greater than five percent of body palette,
    and no ground shadow. These properties describe the intentional top-down art and distinguish
    it from a damaged side view.
- Both profiles continue to require controlled transparent margin, no connected opaque-black
  surface, visible body/shade/wing/outline, true-orange channel order, and antialiased pixels.
  Partial entrance frames, missing warm articulation, red/blue swaps, straight alpha, double
  premultiplication, flattened/opaque surfaces, and a side view with removed legs/shadow remain
  failures.
- The analyzer reports `pose_kind`, common pass/fail checks, and separate side/top-down
  diagnostic checks. `passed` means every common check passed and exactly one complete
  view-appropriate profile was proven; false diagnostic fields for the other view are not
  interpreted as failures.
- This changes release evidence only. It does not change renderer geometry, engine movement,
  platform pixel conversion, presentation, or user-visible behavior.

## Consequences

- Release qualification no longer depends on random selection of a side-heading sample while
  remaining strict about the semantic content and alpha/channel contract of the observed view.
- The first public stable version is a patch above the consumed-but-unpublished tag. Existing
  v0.3.2 installations still update normally through the exact-tag, hash-pinned platform path.
- The Mac-signed artifacts produced during the failed workflow remain internal evidence only;
  `v1.0.1` must produce and verify fresh signed/notarized/stapled artifacts from its own exact SHA.
- Post-release Alienware checks remain defense in depth and fix forward to a later patch rather
  than changing `v1.0.1`.

## Verification

- Committed side and top-down renderer goldens pass both paired-background and exact
  premultiplied-presenter analysis.
- Focused regressions reject channel-swapped and warm-beak-free top-down frames, damaged side
  fallthrough, straight/opaque surfaces, double premultiplication, and opaque-black margins.
- Replaying the retained failed-release captures accepts complete attempts 6–12 as strict
  top-down frames while attempts 3–5 remain rejected as partial entrance frames.
- The complete Python packaging/workflow contracts, Windows PowerShell parse, cargo-dist
  `v1.0.1` plan, candidate workflow, same-SHA default-branch CI, atomic release, post-release
  smokes, and fresh-download checks remain required.
