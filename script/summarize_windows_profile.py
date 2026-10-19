#!/usr/bin/env python3
"""Summarize immutable native profiling evidence without changing its raw samples."""
from __future__ import annotations

import argparse
import json
import statistics
from pathlib import Path


def percentile(values: list[float], fraction: float) -> float | None:
    if not values:
        return None
    ordered = sorted(values)
    position = (len(ordered) - 1) * fraction
    lower = int(position)
    upper = min(lower + 1, len(ordered) - 1)
    return ordered[lower] + (ordered[upper] - ordered[lower]) * (position - lower)


def records(path: Path) -> list[dict]:
    if not path.exists() or not path.read_text(encoding="utf-8-sig").strip():
        return []
    value = json.loads(path.read_text(encoding="utf-8-sig"))
    return value if isinstance(value, list) else [value]


def summarize(directory: Path) -> dict:
    environment = records(directory / "environment.json")[0]
    result = records(directory / "result.json")[0]
    samples = records(directory / "samples.json")
    summary = {"directory": str(directory), "environment": environment, "result": result}
    for name in ("cpu_seconds", "dwm_cpu_seconds"):
        rows = [row for row in samples if name in row]
        intervals = []
        for before, after in zip(rows, rows[1:]):
            seconds = after["elapsed_seconds"] - before["elapsed_seconds"]
            delta = after[name] - before[name]
            if seconds > 0 and delta >= 0:
                intervals.append(100 * delta / seconds)
        if not intervals:
            continue
        duration = rows[-1]["elapsed_seconds"] - rows[0]["elapsed_seconds"]
        average = 100 * (rows[-1][name] - rows[0][name]) / duration
        summary[name.removesuffix("_seconds")] = {
            "average_one_core_percent": average,
            "average_machine_percent": average / environment["logical_processors"],
            "median_one_core_percent": statistics.median(intervals),
            "p95_one_core_percent": percentile(intervals, 0.95),
            "measured_seconds": duration,
        }
    for name in ("working_set", "private_bytes", "handles", "gdi_objects", "user_objects"):
        rows = [row for row in samples if name in row]
        if not rows:
            continue
        values = [row[name] for row in rows]
        item = {"first": values[0], "last": values[-1], "minimum": min(values),
                "maximum": max(values), "median": statistics.median(values)}
        end = rows[-1]["elapsed_seconds"]
        if end - rows[0]["elapsed_seconds"] >= 600:
            previous = [row[name] for row in rows if end - 600 <= row["elapsed_seconds"] < end - 300]
            final = [row[name] for row in rows if row["elapsed_seconds"] >= end - 300]
            item["preceding_five_minute_median"] = statistics.median(previous)
            item["final_five_minute_median"] = statistics.median(final)
            item["five_minute_median_growth"] = statistics.median(final) - statistics.median(previous)
        summary[name] = item
    frame_path = directory / "frames.json"
    if frame_path.exists() and frame_path.stat().st_size:
        frames = records(frame_path)[0]
        start = result.get("sample_start_unix_ms")
        end = result.get("sample_end_unix_ms")
        if not isinstance(start, (int, float)) or not isinstance(end, (int, float)) or end < start:
            summary["frames"] = {"omitted": "incomplete_interval", "retained_frames": frames["retained_frames"]}
            return summary
        # Frame and controller clocks start at different times. Use the actual UTC interval,
        # excluding both warm-up and the potentially long graceful walk-off.
        measured = [row for row in frames["samples"]
                    if start <= frames["started_unix_ms"] + row["elapsed_ms"] <= end]
        summary["frames"] = {"count": len(measured), "retained_frames": frames["retained_frames"]}
        oldest = min((row["elapsed_ms"] for row in frames["samples"]), default=0) + frames["started_unix_ms"]
        summary["frames"]["measurement_start_retained"] = oldest <= start
        for stage in ("render_us", "allocation_us", "copy_us", "update_us", "total_us"):
            values = [row[stage] for row in measured]
            summary["frames"][stage] = {"p50": percentile(values, 0.5),
                                          "p95": percentile(values, 0.95),
                                          "p99": percentile(values, 0.99)}
        summary["frames"]["bitmap_allocations"] = sum(row["bitmap_allocations"] for row in measured)
        summary["frames"]["presented_pixels_p95"] = percentile([row["presented_pixels"] for row in measured], 0.95)
    return summary


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("directories", type=Path, nargs="+")
    parser.add_argument("--output", type=Path)
    arguments = parser.parse_args()
    summaries = [summarize(directory) for directory in arguments.directories]
    value = json.dumps(summaries, indent=2)
    if arguments.output:
        # Raw evidence is immutable; require a new derived summary path as well.
        with arguments.output.open("x", encoding="utf-8") as output:
            output.write(value + "\n")
    else:
        print(value)


if __name__ == "__main__":
    main()
