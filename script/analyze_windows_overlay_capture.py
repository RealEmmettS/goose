#!/usr/bin/env python3
"""Fail-closed semantic analysis for paired Windows overlay screenshots.

The smoke harness freezes the exact honk300 process, captures its unchanged layered
window over two controlled backgrounds, and sends both PNGs here.  Comparing the
same surface twice lets us distinguish real per-pixel alpha from a merely colorful
rectangle while the default palette's deliberately asymmetric orange catches an
R/B channel swap.

Only the Python standard library is used so this can run on a stock GitHub-hosted
Windows runner as well as in local packaging verification.
"""

from __future__ import annotations

import argparse
import json
import struct
import sys
import zlib
from collections import deque
from pathlib import Path
from typing import Iterable, Sequence


PALETTE = {
    "body": (0xED, 0xED, 0xED),
    "shade": (0xC6, 0xC6, 0xC6),
    "wing": (0x51, 0x55, 0x57),
    "orange": (0xFC, 0x79, 0x27),
    "orange_dark": (0xD1, 0x55, 0x1B),
    "outline": (0xC9, 0xC9, 0xC9),
}

# Conservative floors beneath every committed side-view golden.  They are high
# enough that a few antialiased pixels or an unrelated desktop icon cannot pass.
PALETTE_MINIMUMS = {
    "body": 100,
    "shade": 5,
    "wing": 50,
    "orange": 10,
    "orange_dark": 3,
    "outline": 10,
}


def _paeth(a: int, b: int, c: int) -> int:
    p = a + b - c
    pa = abs(p - a)
    pb = abs(p - b)
    pc = abs(p - c)
    if pa <= pb and pa <= pc:
        return a
    if pb <= pc:
        return b
    return c


def read_png_rgba(path: Path) -> tuple[int, int, list[tuple[int, int, int, int]]]:
    """Decode a non-interlaced 8-bit RGB/RGBA PNG without third-party modules."""

    data = path.read_bytes()
    if not data.startswith(b"\x89PNG\r\n\x1a\n"):
        raise ValueError(f"{path} is not a PNG")

    pos = 8
    width = height = bit_depth = color_type = interlace = None
    compressed = bytearray()
    while pos + 12 <= len(data):
        length = struct.unpack(">I", data[pos : pos + 4])[0]
        kind = data[pos + 4 : pos + 8]
        payload = data[pos + 8 : pos + 8 + length]
        pos += 12 + length
        if kind == b"IHDR":
            width, height, bit_depth, color_type, _comp, _filter, interlace = struct.unpack(
                ">IIBBBBB", payload
            )
        elif kind == b"IDAT":
            compressed.extend(payload)
        elif kind == b"IEND":
            break

    if not width or not height:
        raise ValueError(f"{path} has no valid IHDR")
    if bit_depth != 8 or color_type not in (2, 6) or interlace != 0:
        raise ValueError(
            f"{path} must be a non-interlaced 8-bit RGB/RGBA PNG "
            f"(depth={bit_depth}, color={color_type}, interlace={interlace})"
        )

    channels = 3 if color_type == 2 else 4
    stride = width * channels
    raw = zlib.decompress(bytes(compressed))
    expected = height * (stride + 1)
    if len(raw) != expected:
        raise ValueError(f"{path} decoded to {len(raw)} bytes; expected {expected}")

    previous = bytearray(stride)
    rows: list[bytearray] = []
    cursor = 0
    for _ in range(height):
        filter_kind = raw[cursor]
        cursor += 1
        row = bytearray(raw[cursor : cursor + stride])
        cursor += stride
        for index, value in enumerate(row):
            left = row[index - channels] if index >= channels else 0
            up = previous[index]
            upper_left = previous[index - channels] if index >= channels else 0
            if filter_kind == 1:
                row[index] = (value + left) & 0xFF
            elif filter_kind == 2:
                row[index] = (value + up) & 0xFF
            elif filter_kind == 3:
                row[index] = (value + ((left + up) // 2)) & 0xFF
            elif filter_kind == 4:
                row[index] = (value + _paeth(left, up, upper_left)) & 0xFF
            elif filter_kind != 0:
                raise ValueError(f"{path} uses unsupported PNG filter {filter_kind}")
        rows.append(row)
        previous = row

    pixels: list[tuple[int, int, int, int]] = []
    for row in rows:
        if channels == 3:
            pixels.extend((row[i], row[i + 1], row[i + 2], 255) for i in range(0, stride, 3))
        else:
            pixels.extend(
                (row[i], row[i + 1], row[i + 2], row[i + 3])
                for i in range(0, stride, 4)
            )
    return width, height, pixels


def parse_rgb(value: str) -> tuple[int, int, int]:
    value = value.strip().removeprefix("#")
    if len(value) != 6:
        raise ValueError(f"expected RRGGBB background, got {value!r}")
    return tuple(int(value[index : index + 2], 16) for index in (0, 2, 4))  # type: ignore[return-value]


def _close(actual: Sequence[float], expected: Sequence[float], tolerance: float) -> bool:
    return all(abs(a - e) <= tolerance for a, e in zip(actual, expected))


def _components(mask: Sequence[bool], width: int, height: int, minimum_size: int) -> list[dict]:
    seen = bytearray(len(mask))
    result: list[dict] = []
    for start, enabled in enumerate(mask):
        if not enabled or seen[start]:
            continue
        seen[start] = 1
        queue = deque([start])
        members: list[int] = []
        while queue:
            current = queue.popleft()
            members.append(current)
            x = current % width
            y = current // width
            for dy in (-1, 0, 1):
                for dx in (-1, 0, 1):
                    if dx == 0 and dy == 0:
                        continue
                    nx, ny = x + dx, y + dy
                    if nx < 0 or nx >= width or ny < 0 or ny >= height:
                        continue
                    neighbor = ny * width + nx
                    if mask[neighbor] and not seen[neighbor]:
                        seen[neighbor] = 1
                        queue.append(neighbor)
        if len(members) >= minimum_size:
            xs = [member % width for member in members]
            ys = [member // width for member in members]
            result.append(
                {
                    "pixels": len(members),
                    "bounds": [min(xs), min(ys), max(xs) + 1, max(ys) + 1],
                    "centroid": [round(sum(xs) / len(xs), 2), round(sum(ys) / len(ys), 2)],
                }
            )
    return sorted(result, key=lambda component: component["pixels"], reverse=True)


def analyze_captures(
    width: int,
    height: int,
    dark_pixels: Sequence[tuple[int, int, int, int]],
    light_pixels: Sequence[tuple[int, int, int, int]],
    dark_background: tuple[int, int, int],
    light_background: tuple[int, int, int],
) -> dict:
    if len(dark_pixels) != width * height or len(light_pixels) != width * height:
        raise ValueError("capture dimensions and pixel counts disagree")

    tolerance = 8
    palette_counts = dict.fromkeys(PALETTE, 0)
    transparent_pixels = 0
    semi_transparent_pixels = 0
    shadow_candidates: list[int] = []
    goose_palette_pixels: list[int] = []
    orange_mask = [False] * (width * height)
    unchanged_near_black_mask = [False] * (width * height)

    background_delta = [light_background[i] - dark_background[i] for i in range(3)]
    if any(abs(delta) < 64 for delta in background_delta):
        raise ValueError("light and dark backgrounds are not sufficiently distinct")

    for index, (dark_rgba, light_rgba) in enumerate(zip(dark_pixels, light_pixels)):
        dark = dark_rgba[:3]
        light = light_rgba[:3]
        if _close(dark, dark_background, 3) and _close(light, light_background, 3):
            transparent_pixels += 1
        if max((*dark, *light)) <= 12 and _close(dark, light, 3):
            unchanged_near_black_mask[index] = True

        matches: list[tuple[float, str]] = []
        for name, expected in PALETTE.items():
            if _close(dark, expected, tolerance) and _close(light, expected, tolerance):
                # Shade (#c6c6c6) and outline (#c9c9c9) are intentionally close.
                # Assign one nearest palette owner instead of letting a pixel satisfy
                # both semantic checks through the shared tolerance.
                distance = sum(
                    (actual - target) ** 2
                    for actual, target in zip((*dark, *light), (*expected, *expected))
                )
                matches.append((distance, name))
        if matches:
            _distance, owner = min(matches)
            palette_counts[owner] += 1
            if owner in {"body", "shade", "wing", "outline"}:
                goose_palette_pixels.append(index)
            if owner in {"orange", "orange_dark"}:
                orange_mask[index] = True

        channel_transmittance = [
            (light[channel] - dark[channel]) / background_delta[channel]
            for channel in range(3)
        ]
        if max(channel_transmittance) - min(channel_transmittance) > 0.10:
            continue
        transmittance = sum(channel_transmittance) / 3.0
        alpha = 1.0 - transmittance
        if 0.08 <= alpha <= 0.92:
            semi_transparent_pixels += 1
        if not 0.08 <= alpha <= 0.35:
            continue

        reconstructed = []
        for channel in range(3):
            from_dark = (
                dark[channel] - (1.0 - alpha) * dark_background[channel]
            ) / alpha
            from_light = (
                light[channel] - (1.0 - alpha) * light_background[channel]
            ) / alpha
            reconstructed.append((from_dark + from_light) / 2.0)
        if (
            # The stipple source is straight #202020 at alpha 42/255. A second
            # accidental premultiplication reconstructs near #050505 and must fail;
            # the wider bounds only accommodate 8-bit compositor rounding at edges.
            all(12.0 <= channel <= 55.0 for channel in reconstructed)
            and max(reconstructed) - min(reconstructed) <= 20.0
        ):
            shadow_candidates.append(index)

    orange_components = _components(orange_mask, width, height, minimum_size=3)
    unchanged_near_black_components = _components(
        unchanged_near_black_mask,
        width,
        height,
        minimum_size=4,
    )
    # The real stippled ground shadow is below the opaque body/wing/outline palette.
    # Restrict reconstruction to that spatial band so a damaged dark antialiased
    # wing edge cannot masquerade as proof that the shadow survived premultiplication.
    opaque_goose_bottom = max(
        (index // width for index in goose_palette_pixels),
        default=height,
    )
    shadow_pixels = sum(
        1 for index in shadow_candidates if index // width > opaque_goose_bottom
    )
    total = width * height
    largest_near_black_component = max(
        (component["pixels"] for component in unchanged_near_black_components),
        default=0,
    )
    checks = {
        # Every committed side-view golden is more than 94% transparent and a real
        # monitor-sized overlay has still more margin.  An 80% floor leaves ample
        # room for animation/effects while rejecting an opaque or mostly opaque
        # layered-window rectangle.
        "controlled_transparent_background": transparent_pixels >= max(25, total * 4 // 5),
        # A channel/alpha bridge failure can preserve a small colorful goose while
        # turning most of the transparent surface into an opaque black rectangle.
        # The controlled backgrounds contain no near-black patch, so fail closed on
        # any connected unchanged block larger than one percent of the crop.
        "no_opaque_black_surface": largest_near_black_component <= max(25, total // 100),
        "visible_body": palette_counts["body"] >= PALETTE_MINIMUMS["body"],
        "visible_shade": palette_counts["shade"] >= PALETTE_MINIMUMS["shade"],
        "visible_wing": palette_counts["wing"] >= PALETTE_MINIMUMS["wing"],
        "visible_outline": palette_counts["outline"] >= PALETTE_MINIMUMS["outline"],
        "asymmetric_orange_channels": (
            palette_counts["orange"] >= PALETTE_MINIMUMS["orange"]
            and palette_counts["orange_dark"] >= PALETTE_MINIMUMS["orange_dark"]
        ),
        # The side renderer produces a beak component spatially separated from a
        # two-tone leg/foot assembly (near orange + far dark orange). The top-down
        # renderer has only the beak, so this also guarantees CI observed an
        # articulated side-view pose. The feet can overlap mid-stride, which is why
        # the two leg tones are checked separately instead of demanding three blobs.
        "visible_beak_and_two_legs": (
            len(orange_components) >= 2
            and max(component["centroid"][1] for component in orange_components)
            - min(component["centroid"][1] for component in orange_components)
            >= 15.0
            and palette_counts["orange_dark"] >= PALETTE_MINIMUMS["orange_dark"]
        ),
        "semi_transparent_edges": semi_transparent_pixels >= 20,
        "semi_transparent_shadow": shadow_pixels >= 5,
    }
    return {
        "passed": all(checks.values()),
        "dimensions": [width, height],
        "backgrounds": {
            "dark": list(dark_background),
            "light": list(light_background),
        },
        "checks": checks,
        "counts": {
            "transparent": transparent_pixels,
            "semi_transparent": semi_transparent_pixels,
            "shadow": shadow_pixels,
            "largest_unchanged_near_black_component": largest_near_black_component,
            "palette": palette_counts,
        },
        "orange_components": orange_components,
        "unchanged_near_black_components": unchanged_near_black_components,
    }


def analyze_files(
    dark_path: Path,
    light_path: Path,
    dark_background: tuple[int, int, int],
    light_background: tuple[int, int, int],
) -> dict:
    dark_width, dark_height, dark_pixels = read_png_rgba(dark_path)
    light_width, light_height, light_pixels = read_png_rgba(light_path)
    if (dark_width, dark_height) != (light_width, light_height):
        raise ValueError(
            f"capture sizes differ: dark={dark_width}x{dark_height}, "
            f"light={light_width}x{light_height}"
        )
    return analyze_captures(
        dark_width,
        dark_height,
        dark_pixels,
        light_pixels,
        dark_background,
        light_background,
    )


def _parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser()
    parser.add_argument("--dark", type=Path, required=True)
    parser.add_argument("--light", type=Path, required=True)
    parser.add_argument("--dark-bg", default="203040")
    parser.add_argument("--light-bg", default="f4ede4")
    parser.add_argument("--output", type=Path)
    return parser


def main(argv: Iterable[str] | None = None) -> int:
    args = _parser().parse_args(argv)
    try:
        result = analyze_files(
            args.dark,
            args.light,
            parse_rgb(args.dark_bg),
            parse_rgb(args.light_bg),
        )
    except (OSError, ValueError, zlib.error) as error:
        print(f"Windows overlay capture analysis failed: {error}", file=sys.stderr)
        return 2

    rendered = json.dumps(result, indent=2, sort_keys=True)
    print(rendered)
    if args.output:
        args.output.parent.mkdir(parents=True, exist_ok=True)
        args.output.write_text(rendered + "\n", encoding="utf-8")
    return 0 if result["passed"] else 1


if __name__ == "__main__":
    raise SystemExit(main())
