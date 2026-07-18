#!/usr/bin/env python3
"""Fail until compositor captures contain the requested controlled background."""

from __future__ import annotations

import argparse
import json
from pathlib import Path

from analyze_linux_overlay_capture import read_png


def parse_rgb(value: str) -> tuple[int, int, int]:
    normalized = value.removeprefix("#")
    if len(normalized) != 6:
        raise ValueError(f"expected RRGGBB background, got {value!r}")
    try:
        red, green, blue = bytes.fromhex(normalized)
        return red, green, blue
    except ValueError as error:
        raise ValueError(f"expected RRGGBB background, got {value!r}") from error


def sampled_fraction(path: Path, expected: tuple[int, int, int]) -> float:
    width, height, pixels = read_png(path)
    step = max(1, min(width, height) // 64)
    matching = 0
    samples = 0
    for y in range(0, height, step):
        for x in range(0, width, step):
            offset = (y * width + x) * 4
            matching += tuple(pixels[offset : offset + 3]) == expected
            samples += 1
    return matching / samples if samples else 0.0


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--color", required=True)
    parser.add_argument("--minimum", type=float, default=0.90)
    parser.add_argument("png", nargs="+")
    args = parser.parse_args()
    if not 0.0 < args.minimum <= 1.0:
        parser.error("--minimum must be greater than zero and at most one")
    try:
        expected = parse_rgb(args.color)
        fractions = {name: sampled_fraction(Path(name), expected) for name in args.png}
    except (OSError, ValueError) as error:
        parser.error(str(error))
    payload = {"color": list(expected), "minimum": args.minimum, "fractions": fractions}
    print(json.dumps(payload, sort_keys=True))
    if any(fraction < args.minimum for fraction in fractions.values()):
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
