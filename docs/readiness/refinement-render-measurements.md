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
