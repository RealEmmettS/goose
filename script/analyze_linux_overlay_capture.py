#!/usr/bin/env python3
"""Validate Linux compositor captures against paired controlled backgrounds."""

from __future__ import annotations

import argparse
import json
import struct
import zlib
from collections import deque
from dataclasses import asdict, dataclass
from pathlib import Path


DARK_BACKGROUND = (0x20, 0x30, 0x40)
LIGHT_BACKGROUND = (0xD8, 0xE6, 0xF4)


@dataclass(frozen=True)
class CaptureMetrics:
    label: str
    width: int
    height: int
    dark_background_pixels: int
    light_background_pixels: int
    background_transition_pixels: int
    body_pixels: int
    wing_pixels: int
    warm_pixels: int
    largest_near_black_component: int
    largest_unchanged_component: int

    @property
    def pixels(self) -> int:
        return self.width * self.height

    @property
    def has_goose(self) -> bool:
        return self.body_pixels >= 100 and self.wing_pixels >= 25 and self.warm_pixels >= 20


def read_png(path: Path) -> tuple[int, int, bytearray]:
    data = path.read_bytes()
    if not data.startswith(b"\x89PNG\r\n\x1a\n"):
        raise ValueError(f"{path}: invalid PNG signature")
    pos = 8
    width = height = channels = None
    compressed = bytearray()
    while pos + 12 <= len(data):
        length = struct.unpack_from(">I", data, pos)[0]
        kind = data[pos + 4 : pos + 8]
        payload = data[pos + 8 : pos + 8 + length]
        pos += 12 + length
        if kind == b"IHDR":
            width, height, depth, color, compression, filtering, interlace = struct.unpack(
                ">IIBBBBB", payload
            )
            if depth != 8 or color not in (2, 6) or compression or filtering or interlace:
                raise ValueError(f"{path}: unsupported PNG encoding")
            channels = 3 if color == 2 else 4
        elif kind == b"IDAT":
            compressed.extend(payload)
        elif kind == b"IEND":
            break
    if not width or not height or not channels:
        raise ValueError(f"{path}: missing PNG image header")

    raw = zlib.decompress(bytes(compressed))
    stride = width * channels
    expected = height * (stride + 1)
    if len(raw) != expected:
        raise ValueError(f"{path}: unexpected decompressed byte count")
    previous = bytearray(stride)
    offset = 0
    pixels = bytearray()
    for _ in range(height):
        filter_kind = raw[offset]
        offset += 1
        row = bytearray(raw[offset : offset + stride])
        offset += stride
        for index, value in enumerate(row):
            left = row[index - channels] if index >= channels else 0
            up = previous[index]
            up_left = previous[index - channels] if index >= channels else 0
            if filter_kind == 1:
                row[index] = (value + left) & 0xFF
            elif filter_kind == 2:
                row[index] = (value + up) & 0xFF
            elif filter_kind == 3:
                row[index] = (value + ((left + up) // 2)) & 0xFF
            elif filter_kind == 4:
                estimate = left + up - up_left
                distances = (abs(estimate - left), abs(estimate - up), abs(estimate - up_left))
                predictor = (left, up, up_left)[distances.index(min(distances))]
                row[index] = (value + predictor) & 0xFF
            elif filter_kind != 0:
                raise ValueError(f"{path}: unsupported PNG row filter {filter_kind}")
        for index in range(0, stride, channels):
            red, green, blue = row[index : index + 3]
            alpha = row[index + 3] if channels == 4 else 255
            pixels.extend((red, green, blue, alpha))
        previous = row
    return width, height, pixels


def near(red: int, green: int, blue: int, alpha: int, color: tuple[int, int, int], tolerance: int) -> bool:
    return (
        alpha >= 245
        and abs(red - color[0]) <= tolerance
        and abs(green - color[1]) <= tolerance
        and abs(blue - color[2]) <= tolerance
    )


def largest_component(mask: bytearray, width: int, height: int) -> int:
    visited = bytearray(len(mask))
    largest = 0
    for start, enabled in enumerate(mask):
        if not enabled or visited[start]:
            continue
        visited[start] = 1
        queue = deque([start])
        size = 0
        while queue:
            index = queue.popleft()
            size += 1
            x = index % width
            for neighbor in (index - width, index + width, index - 1, index + 1):
                if neighbor < 0 or neighbor >= len(mask) or visited[neighbor] or not mask[neighbor]:
                    continue
                if neighbor == index - 1 and x == 0:
                    continue
                if neighbor == index + 1 and x + 1 == width:
                    continue
                visited[neighbor] = 1
                queue.append(neighbor)
        largest = max(largest, size)
    return largest


def analyze_pair(label: str, dark_path: Path, light_path: Path) -> CaptureMetrics:
    dark_width, dark_height, dark = read_png(dark_path)
    light_width, light_height, light = read_png(light_path)
    if (dark_width, dark_height) != (light_width, light_height):
        raise ValueError(f"{label}: paired screenshots have different dimensions")

    dark_background = light_background = transitions = 0
    body = wing = warm = 0
    near_black_mask = bytearray()
    unchanged_mask = bytearray()
    for offset in range(0, len(dark), 4):
        dark_red, dark_green, dark_blue, dark_alpha = dark[offset : offset + 4]
        red, green, blue, alpha = light[offset : offset + 4]
        dark_is_background = near(
            dark_red, dark_green, dark_blue, dark_alpha, DARK_BACKGROUND, 4
        )
        light_is_background = near(red, green, blue, alpha, LIGHT_BACKGROUND, 4)
        dark_background += dark_is_background
        light_background += light_is_background
        transitions += dark_is_background and light_is_background
        if (
            not light_is_background
            and alpha >= 245
            and min(red, green, blue) >= 205
            and max(red, green, blue) - min(red, green, blue) <= 35
        ):
            body += 1
        if alpha >= 245 and 45 <= red <= 145 and 45 <= green <= 145 and 45 <= blue <= 145 and max(red, green, blue) - min(red, green, blue) <= 30:
            wing += 1
        if alpha >= 245 and red >= 185 and 55 <= green <= 185 and blue <= 105 and red >= green + 35 and green >= blue + 15:
            warm += 1
        near_black_mask.append(
            (alpha >= 245 and red <= 28 and green <= 28 and blue <= 28)
            or (
                dark_alpha >= 245
                and dark_red <= 28
                and dark_green <= 28
                and dark_blue <= 28
            )
        )
        unchanged_mask.append(
            alpha >= 245
            and max(
                abs(dark_red - red),
                abs(dark_green - green),
                abs(dark_blue - blue),
            )
            <= 5
        )

    return CaptureMetrics(
        label=label,
        width=dark_width,
        height=dark_height,
        dark_background_pixels=dark_background,
        light_background_pixels=light_background,
        background_transition_pixels=transitions,
        body_pixels=body,
        wing_pixels=wing,
        warm_pixels=warm,
        largest_near_black_component=largest_component(near_black_mask, dark_width, dark_height),
        largest_unchanged_component=largest_component(unchanged_mask, dark_width, dark_height),
    )


def validate(metrics: CaptureMetrics) -> list[str]:
    failures: list[str] = []
    minimum_background = int(metrics.pixels * 0.65)
    minimum_transition = int(metrics.pixels * 0.60)
    if metrics.dark_background_pixels < minimum_background:
        failures.append("dark controlled background is not visible through the overlay")
    if metrics.light_background_pixels < minimum_background:
        failures.append("light controlled background is not visible through the overlay")
    if metrics.background_transition_pixels < minimum_transition:
        failures.append("paired background transition does not prove per-pixel transparency")
    black_limit = min(60_000, max(1_500, int(metrics.pixels * 0.08)))
    if metrics.largest_near_black_component > black_limit:
        failures.append("near-black compositor component is too large")
    unchanged_limit = max(2_500, int(metrics.pixels * 0.15))
    if metrics.largest_unchanged_component > unchanged_limit:
        failures.append("unchanged opaque compositor component is too large")
    return failures


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument(
        "--pair",
        nargs=3,
        action="append",
        metavar=("LABEL", "DARK_PNG", "LIGHT_PNG"),
        required=True,
    )
    goose = parser.add_mutually_exclusive_group(required=True)
    goose.add_argument("--require-goose-any", action="store_true")
    goose.add_argument("--require-goose-each", action="store_true")
    goose.add_argument("--require-no-goose", action="store_true")
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()

    reports: list[CaptureMetrics] = []
    failures: list[str] = []
    for label, dark_name, light_name in args.pair:
        try:
            metrics = analyze_pair(label, Path(dark_name), Path(light_name))
        except (OSError, ValueError, zlib.error) as error:
            failures.append(f"{label}: {error}")
            continue
        reports.append(metrics)
        failures.extend(f"{label}: {failure}" for failure in validate(metrics))
        if args.require_goose_each and not metrics.has_goose:
            failures.append(f"{label}: articulated goose palette features are missing")
        if args.require_no_goose and metrics.has_goose:
            failures.append(f"{label}: baseline unexpectedly contains goose palette features")

    if args.require_goose_any and not any(report.has_goose for report in reports):
        failures.append("no output contains the articulated goose palette features")
    payload = {
        "schema": "honk300.linux-overlay.v1",
        "pairs": [asdict(report) | {"has_goose": report.has_goose} for report in reports],
        "failures": failures,
    }
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    print(json.dumps(payload, indent=2, sort_keys=True))
    if failures:
        parser.error("; ".join(failures))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
