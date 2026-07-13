TT;DR: The platform loops now share a tested sequencing skeleton; native event, capability, and presentation work stays in each backend.

## Status

Done on 2026-07-12. The formerly deferred extraction is represented by `RuntimeCore`, which is
used by Windows, macOS, and Linux.

## Evidence

- `RuntimeCore` owns clock sampling, fixed 120 Hz accumulation, 60 Hz presentation cadence,
  current-versus-previous damage, restart-required detection, and frame-order assertions.
- All three runtime modules call `begin_frame`, `tick`, and `damage` in the shared order while
  retaining their genuinely platform-specific event pumps and capability recovery.
- Four focused sequencing tests pass.
- `honk-platform-windows` cross-checks pass for x86_64 and ARM64 Windows targets from this Mac.

## Activity

- 2026-07-12 20:15 - Re-audited the previously deferred card against the current tree, ran the
  focused shared-core tests and both Windows target checks, and closed it without a redundant
  high-risk runtime rewrite (agent: codex).
