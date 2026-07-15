# ADR 0025 — First Stable v1 Release And Post-Release Hardware Verification

- Status: Accepted (2026-07-14)
- Superseded in part: ADR 0027 replaces only the prospective `v1.0.0` public tag after that
  immutable tag failed closed before draft creation. `v1.0.1` is the first public stable target;
  the release and post-release hardware-verification contracts below remain accepted.
- Relates to: ADR 0018 (candidate-first atomic publication), ADR 0020 (Developer ID macOS
  distribution), ADR 0022 (macOS Accessibility evidence), ADR 0023 (complete rolling releases),
  and ADR 0024 (macOS menu-bar control).
- Supersedes: the prospective v0.3.3 release label in current plans and readiness material. It
  does not change any technical, signing, packaging, update, or verification contract in those
  records.

## Context

Honk300's native macOS qualification grew into the first complete stable product release: the
platform-neutral engine and renderer, three-name CLI/TUI/IPC control plane, Windows/macOS/Linux
runtimes, managed lifecycle, architecture-isolated updater, Windows installers, native Debian
packages, and Developer ID/notarized universal Mac distribution are all present. The prospective
v0.3.3 candidate was never tagged or published; its workflow runs were diagnostic and therefore
do not constrain the public version number.

The operator designated this milestone as the first major stable release. They will later use an
Alienware for additional hands-on Windows verification and ship forward patches if that hardware
finds defects.

## Decision

- The public release is `v1.0.0`; package metadata, bundle metadata, receipts, manifests,
  installers, updater fixtures, readiness material, website fallback/tests, and release commands
  must agree on `1.0.0` before the candidate is frozen.
- Updating an installed v0.3.2 copy to v1.0.0 remains a supported ordinary semantic-version
  upgrade through all three CLI aliases and the existing platform/provenance-isolated updater.
- The unpublished v0.3.3 candidate run remains historical diagnostic evidence only. It may prove
  a workflow stage or expose a defect, but it cannot satisfy exact-v1.0.0 identity, publication,
  or fresh-download checks.
- v1.0.0 still requires one exact candidate commit to pass the complete hosted Windows/Linux
  artifact and native-smoke matrix plus the physical-Mac Accessibility, rendering, menu, audio,
  lifecycle, performance, signing, notarization, stapling, and Gatekeeper gates. It must then pass
  ordinary default-branch CI on that same commit before the immutable tag is created.
- Additional Alienware hands-on verification is post-release defense in depth. It does not replace
  the hosted Windows candidate gates and does not block v1.0.0 once the defined exact-SHA gates
  pass. Findings are fixed in a new semantic patch release; `v1.0.0` and its assets are never
  rewritten.
- The tracked Alienware handoff contains verification work only. It must preserve the macOS pixel,
  AppKit lifetime, Accessibility, menu, signing/notarization, per-user install, graceful-exit,
  terminal-protection, and platform-isolation contracts when suggesting a patch.

## Consequences

- Public versioning communicates that this is the supported stable baseline rather than another
  pre-1.0 iteration.
- Every release artifact and primary website link can be checked against one unambiguous version,
  tag, commit, size, and hash.
- Later native-machine observations improve the product through v1.0.1 or newer without weakening
  immutable publication or retroactively changing what v1.0.0 contained.

## Verification

- `Cargo.toml`, the local package entry in `Cargo.lock`, `honk300 --version`, app metadata,
  lifecycle receipts, release manifest, installer/package metadata, and website fallback/tests
  all report 1.0.0/v1.0.0.
- `dist plan --tag=v1.0.0` and workflow contracts reject version/tag disagreement.
- A focused updater regression proves v0.3.2 is older than v1.0.0.
- Repository scans distinguish intentionally historical unpublished-v0.3.3 evidence and unrelated
  dependency versions from current release identity.
- The exact candidate, default-branch CI, immutable tag, atomic publication, fresh downloads, and
  production website resolve to the same full commit and verified artifact set.
