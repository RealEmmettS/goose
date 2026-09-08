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
