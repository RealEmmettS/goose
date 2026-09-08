# Refinement renderer measurements

Windows release-build CPU renderer probe, 2026-09-07. Original dual-view art with new shared DamageCanvas, before art replacement. 120 warmup ticks followed by 6,000 sampled ticks per workload; deterministic rotating heading. Values include pose, buffer clear, rasterization, and mud marks. They exclude native compositor, actual props, IPC latency, process CPU utilization, and native desktop acceptance. Those require separate runtime measurements.

| Workload | Total ms | Median us | P95 us | P99 us | Canvas allocations | Retained bytes |
|---|---:|---:|---:|---:|---:|---:|
| Idle | 834.44 | 132.00 | 178.10 | 264.80 | 1 | 49152 |
| Walking | 787.12 | 132.50 | 184.50 | 416.90 | 2 | 81920 |
| Running | 797.97 | 132.70 | 199.80 | 393.00 | 2 | 81920 |
| Mud | 1700.19 | 272.70 | 413.20 | 597.30 | 2 | 1556480 |
| Two output copies | 1549.20 | 263.80 | 377.30 | 825.40 | 4 | 163840 |

Reproduce: `cargo run --release --locked -p honk-engine --example render_bench -- 6000`.

## Continuous projected goose

Repeated on the same Windows machine after the complete rig and visible-foot correction, with no Rust/Zig build running. Same warmup, sample count, headings, and release probe. Raw result: `target/refinement-render-quiet.csv`.

| Workload | Total ms | Median us | P95 us | P99 us | Canvas allocations | Retained bytes |
|---|---:|---:|---:|---:|---:|---:|
| Idle | 1385.24 | 215.40 | 336.00 | 472.50 | 1 | 49152 |
| Walking | 1345.24 | 215.30 | 303.40 | 409.10 | 1 | 49152 |
| Running | 1341.61 | 213.80 | 300.70 | 403.00 | 1 | 49152 |
| Mud | 2366.39 | 378.80 | 546.50 | 730.60 | 1 | 1400832 |
| Two output copies | 2712.96 | 433.00 | 615.10 | 822.30 | 2 | 98304 |

The new geometry costs about 62 percent more median CPU render time for ordinary walking than the old art using the same canvas owner. It is a visual redesign, not a render-speed improvement. Retained walking capacity and allocation growth are lower; all sampled workloads have sub-millisecond P99 CPU rendering on this machine. These measurements do not establish native compositor cost, overall runtime CPU use, battery use, or Raspberry Pi performance.

The final exports in `target/goose-refinement-motion/` were inspected in Chrome at normal and enlarged sizes on light and dark backgrounds, including front/rear steps and the continuous heading sheet. The website task received these actual renderer outputs marked as development assets.

## Native Windows workload comparison

Measured the exact public v1.3.7 binary and the v1.4.0 `ca20ad9` runtime on the same
1920x1080 physical display with 22 logical processors. The public archive's manifest
identity, size and SHA-256 were verified before extraction. Each binary used an isolated
config, muted audio and disabled pointer/window manipulation. Both started through their
same hidden runtime entry point; captured controller launch/EOF is qualified separately
in audit R11. No Rust/Zig build ran during sampling and no compositor calibration occurred.

After a 15-second settling period, each phase sampled the actual runtime process every
second for 45 iterations, including an IPC status call every fifth iteration. The elapsed
phase duration was about 46 seconds. Headings and ordinary scheduling remain random;
these are individual workload observations, not a controlled statistical speed claim.

| Phase | Public CPU, percent of one core | New CPU, percent of one core | Public ending working set, MiB | New ending working set, MiB |
|---|---:|---:|---:|---:|
| Startup hold | 6.41 | 5.63 | 16.40 | 16.50 |
| Roaming | 9.09 | 6.41 | 16.66 | 16.56 |
| Mud errand | 9.36 | 7.29 | 16.69 | 16.64 |
| Note errand | 7.81 | 6.65 | 18.86 | 18.89 |
| Meme errand | Unsupported after note failure | 7.19 | Not measured | 22.64 |

The public baseline twice latched collect capability to failed after a normal foreground
denial (R12). The new runtime accepted both errands, retained collect support and emitted
no backend errors. The following 184-second run averaged 3.25 percent of one core, ended
at 33.21 MiB working set / 13.87 MiB private memory and exited gracefully in 5.00 seconds.
Its private-memory high water was 14.18 MiB and handles ranged from 263 to 270. Newly
selected image assets account for changing cache contents; this short run alone does not
prove a steady-state memory plateau or unlimited-session prop behavior.

Status-command median/P95 was 33.20/38.55 ms for the public baseline and 35.28/44.59 ms
for the new runtime. The slowest observed call was 494 ms. These observations do not
show a command-latency improvement, and the new art's per-frame renderer cost above is
still higher. Actual multi-monitor/Pi CPU performance remains unavailable on this machine;
the separately measured two-output raster workload and hosted topology tests cover their
stated scopes only.

Raw samples and retained logs are in `target/runtime-profile-v137-direct/` and
`target/runtime-profile-v140/`; the local measurement driver is
`target/measure-runtime-windows.ps1`. The new binary SHA-256 is
`1319b83783b45c4784064d39a909bdcac2255c885cc95e81e5a12c5bd7624ba8`.
