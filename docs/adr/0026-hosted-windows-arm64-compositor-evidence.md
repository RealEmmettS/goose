# ADR 0026 — Hosted Windows ARM64 Compositor Evidence Boundary

- Status: Accepted (2026-07-14)
- Relates to: ADR 0012 (CI-proven backend readiness), ADR 0015 (Windows layered-overlay safety),
  ADR 0018 (candidate-first publication), and ADR 0025 (v1.0.0 release gate).
- Supersedes: only the assumption that GitHub's `windows-11-arm` public-preview runner can always
  expose ordinary or layered HWNDs through `GetDC(NULL)` plus `BitBlt(CAPTUREBLT)`. It does not
  relax real-machine Windows capture, renderer, installer, lifecycle, or architecture checks.

## Context

The v1.0.0 candidate runs the same paired-background native compositor smoke on Windows x64 and
ARM64. Candidate `29387569722` proved the full x64 path, but the public-preview ARM64 hosted
runner returned one static wallpaper image for both acknowledged colors of a visible, full-screen,
PMv2 WinForms HWND. The controller and background agreed on physical `0,0,1024,768` geometry; the
background HWND was visible and changed colors; source, MSI-extracted, installed, and repaired
ARM64 binaries were byte-identical; yet both pre-runtime `BitBlt` captures had zero controlled-
color pixels and identical hashes. Failing later overlay pixels under that capture source would
measure the runner's wallpaper rather than Honk300.

Silently accepting the wallpaper would be dishonest. Requiring that public-preview capture API
to report pixels it does not expose would also make native ARM64 packaging and lifecycle
unreleasable without testing the product. The Windows backend owns the cropped, premultiplied
BGRA DIB that `UpdateLayeredWindow` accepts, so a tightly identified hosted-runner fallback can
retain meaningful native proof without pretending it is DWM screen evidence.

## Decision

- Paired dark/light DWM capture remains mandatory on Windows x64 and on any Windows ARM64 host
  where ordinary windows appear in screen capture. It continues to prove transparent margins,
  opaque body/shade/outline/wing, asymmetric orange channels, beak and two legs, antialiased
  edges, ground shadow, and absence of an opaque black surface.
- A fallback is permitted only when all of these facts hold together:
  - `GITHUB_ACTIONS=true`, `RUNNER_ENVIRONMENT=github-hosted`, `RUNNER_OS=Windows`,
    `RUNNER_ARCH=ARM64`, and the native process reports `PROCESSOR_ARCHITECTURE=ARM64`;
  - controller and background are PMv2 and agree on physical virtual-screen geometry;
  - the background HWND reports visible and acknowledges both tokenized color requests;
  - each controlled color covers at most one percent of its capture; and
  - the two captures are byte-identical, matching the observed static-wallpaper failure.
- Under that exact signature, the running ARM64 binary must still expose a visible
  `honk300_overlay` HWND with a nonzero DPI and plausible rectangle. Only after the real monitor
  crop, premultiplied RGBA-to-BGRA conversion, and a successful `UpdateLayeredWindow`, its Windows
  backend may atomically record the exact selected DIB bytes plus that present's HWND and physical
  rectangle. The smoke requests the record only after its pose delay, freezes the process as soon
  as the completed record appears, and rejects stale metadata unless it names the exact same HWND
  and its recorded origin and dimensions remain within three physical pixels of the independently
  queried frozen rectangle. That bound covers the observed one-presentation-interval, one-to-two-
  pixel drift between the atomic rename and `NtSuspendProcess`; larger drift retries and remains
  fatal if no bounded fresh record passes. It does not relax any surface-pixel assertion.
- The raw premultiplied BGRA record must independently prove channel values bounded by alpha, at
  least 80-percent transparent margin, no connected opaque-black surface, body, shade, outline,
  wing, asymmetric near/far orange, spatially separated beak/legs, antialiased pixels, and a
  semi-transparent ground shadow. It bypasses PNG because ordinary PNG encoding demultiplies
  alpha. A generated fixture or repository golden cannot substitute for the exact process DIB.
- ARM64 PE identity, MSI extraction/install/repair byte identity, rollback, upgrade, downgrade
  refusal, uninstall, start/status/singleton/reload/stop/immediate-restart, and exact-binary hash
  stability remain mandatory.
- Evidence must name the mode `hosted-arm64-presenter-surface`. Documentation must not call this
  a live ARM64 DWM screenshot. A local/self-hosted ARM64 capture failure remains fatal, and a
  future GitHub runner that exposes the controlled colors automatically returns to paired DWM
  capture.

## Consequences

- v1.0.0 can test the native ARM64 executable and package lifecycle on the available hosted
  architecture without converting static wallpaper into false compositor evidence.
- The exact native presenter DIB and a real visible layered HWND are proven on ARM64;
  final desktop composition remains explicitly runner-limited. Windows x64 still supplies the
  full DWM alpha/composition proof for the shared Windows backend.
- The `HONK300_WINDOWS_SMOKE_PRESENT` hook is dormant unless explicitly set, writes at most one
  completed post-success present per harness request, and has no user-facing command or
  configuration surface.
- Post-release physical Windows checks remain useful defense in depth. Any future real ARM64
  machine result is recorded as additional evidence or fixed forward; it never rewrites v1.0.0.

## Verification

- Python contracts reject straight-alpha, double-premultiplied, opaque, mostly opaque-black,
  channel-swapped, incomplete, stale/unbound, or non-articulated presenter records and pin the
  exact GitHub-hosted ARM64 predicate.
- Windows x64 candidate evidence still contains distinct 100-percent controlled-background
  proofs and paired live overlay captures.
- Hosted ARM64 evidence records both identical wallpaper hashes, zero controlled-color coverage,
  visible background and overlay HWND metadata, the exact HWND plus bounded rectangle deltas,
  `hosted-arm64-presenter-surface`, exact surface analysis, native PE/MSI identity, and lifecycle
  results.
- Cross-compilation checks both Windows targets after the backend handoff changes.
