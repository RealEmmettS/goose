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

TOP_DOWN_ORANGE_MINIMUM = 5
TOP_DOWN_ORANGE_MAXIMUM = 20

PRESENTER_MAGIC = b"HONK300_LAYERED_BGRA_V1"


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


def _mask_bounds(mask: Sequence[bool], width: int) -> list[int] | None:
    members = [index for index, enabled in enumerate(mask) if enabled]
    if not members:
        return None
    xs = [index % width for index in members]
    ys = [index // width for index in members]
    return [min(xs), min(ys), max(xs) + 1, max(ys) + 1]


def _has_complete_margin(bounds: Sequence[int] | None, width: int, height: int) -> bool:
    return bool(
        bounds
        and bounds[0] > 0
        and bounds[1] > 0
        and bounds[2] < width
        and bounds[3] < height
    )


def _classify_pose(
    palette_counts: dict[str, int],
    orange_components: Sequence[dict],
    shadow_pixels: int,
) -> tuple[str, dict[str, dict[str, bool]]]:
    """Prove either renderer view without treating a valid top-down pose as damaged.

    Side view intentionally exposes a separated beak/two-tone leg assembly and a
    stippled ground shadow. Top-down view intentionally has a single compact beak,
    no visible legs or ground shadow, a larger wing-to-body ratio, and much less
    shade. Requiring the full view-specific signature prevents a damaged side view
    from falling through to the top-down acceptance path.
    """

    orange_y_span = (
        max(component["centroid"][1] for component in orange_components)
        - min(component["centroid"][1] for component in orange_components)
        if orange_components
        else 0.0
    )
    side_checks = {
        "two_tone_orange": (
            palette_counts["orange"] >= PALETTE_MINIMUMS["orange"]
            and palette_counts["orange_dark"] >= PALETTE_MINIMUMS["orange_dark"]
        ),
        "visible_beak_and_two_legs": (
            len(orange_components) >= 2 and orange_y_span >= 15.0
        ),
        "semi_transparent_shadow": shadow_pixels >= 5,
    }
    top_down_checks = {
        "single_compact_beak": (
            len(orange_components) == 1
            and TOP_DOWN_ORANGE_MINIMUM
            <= palette_counts["orange"]
            <= TOP_DOWN_ORANGE_MAXIMUM
        ),
        "no_dark_orange_legs": (
            palette_counts["orange_dark"] < PALETTE_MINIMUMS["orange_dark"]
        ),
        "complete_top_down_palette": (
            palette_counts["body"] >= 400
            and palette_counts["wing"] >= 280
            and palette_counts["outline"] >= 25
        ),
        # The committed top-down view dedicates at least 60% as many opaque
        # palette pixels to its wing as its body. Every committed side view is
        # below that ratio, even mid-stride.
        "top_down_wing_body_ratio": (
            palette_counts["wing"] * 5 >= palette_counts["body"] * 3
        ),
        # Top-down uses only a small neck/body shade. Side view has materially
        # more shade, so this is a second independent view discriminator.
        "top_down_shade_ratio": (
            palette_counts["shade"] * 20 <= palette_counts["body"]
        ),
        # Removing the lower leg/shadow area from a side frame can otherwise
        # leave one orange component. Top-down keeps a much smaller beak and
        # outline share than that damaged side silhouette.
        "top_down_beak_body_ratio": (
            palette_counts["orange"] * 40 <= palette_counts["body"]
        ),
        "top_down_outline_body_ratio": (
            palette_counts["outline"] * 8 <= palette_counts["body"]
        ),
        "no_ground_shadow": shadow_pixels < 5,
    }
    if all(side_checks.values()):
        pose_kind = "side"
    elif all(top_down_checks.values()):
        pose_kind = "top-down"
    else:
        pose_kind = "unknown"
    return pose_kind, {"side": side_checks, "top_down": top_down_checks}


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
    semantic_edge_candidates = 0
    semantic_edge_pixels = 0
    shadow_candidates: list[int] = []
    goose_palette_pixels: list[int] = []
    orange_mask = [False] * (width * height)
    unchanged_near_black_mask = [False] * (width * height)
    content_mask = [False] * (width * height)

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
        if alpha >= 0.02:
            content_mask[index] = True
        if 0.08 <= alpha <= 0.92:
            semi_transparent_pixels += 1
        if not 0.08 <= alpha <= 0.92:
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
        if 0.15 <= alpha <= 0.85:
            semantic_edge_candidates += 1
            if any(_close(reconstructed, expected, 20) for expected in PALETTE.values()):
                semantic_edge_pixels += 1
        if alpha > 0.35:
            continue
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
    content_bounds = _mask_bounds(content_mask, width)
    pose_kind, pose_checks = _classify_pose(
        palette_counts,
        orange_components,
        shadow_pixels,
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
        "complete_content_margin": _has_complete_margin(content_bounds, width, height),
        "visible_body": palette_counts["body"] >= PALETTE_MINIMUMS["body"],
        "visible_shade": palette_counts["shade"] >= PALETTE_MINIMUMS["shade"],
        "visible_wing": palette_counts["wing"] >= PALETTE_MINIMUMS["wing"],
        "visible_outline": palette_counts["outline"] >= PALETTE_MINIMUMS["outline"],
        # Both views contain deliberately asymmetric true-orange pixels, so an
        # R/B bridge swap still fails even though top-down intentionally omits the
        # dark-orange legs used by the side-view proof.
        "asymmetric_orange_channels": (
            palette_counts["orange"] >= TOP_DOWN_ORANGE_MINIMUM
        ),
        "semi_transparent_edges": semi_transparent_pixels >= 20,
        "semantic_edge_colors": (
            semantic_edge_pixels >= 50
            and semantic_edge_pixels * 2 >= semantic_edge_candidates
        ),
        "view_appropriate_articulation": pose_kind != "unknown",
    }
    return {
        "passed": all(checks.values()),
        "pose_kind": pose_kind,
        "dimensions": [width, height],
        "backgrounds": {
            "dark": list(dark_background),
            "light": list(light_background),
        },
        "checks": checks,
        "pose_checks": pose_checks,
        "counts": {
            "transparent": transparent_pixels,
            "semi_transparent": semi_transparent_pixels,
            "semantic_edge_candidates": semantic_edge_candidates,
            "semantic_edge_colors": semantic_edge_pixels,
            "shadow": shadow_pixels,
            "largest_unchanged_near_black_component": largest_near_black_component,
            "palette": palette_counts,
        },
        "orange_components": orange_components,
        "content_bounds": content_bounds,
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


def analyze_surface(
    width: int,
    height: int,
    pixels: Sequence[tuple[int, int, int, int]],
    present: dict | None = None,
) -> dict:
    """Validate exact premultiplied RGBA values recovered from a presented BGRA DIB.

    This is not a replacement for paired DWM capture. It is the strict fallback for GitHub's
    hosted ARM64 runner when that runner demonstrably returns the same static wallpaper for two
    acknowledged ordinary-window colors. The real ARM64 process must still expose its visible
    layered HWND. The record is emitted only after a successful `UpdateLayeredWindow`, retains
    that HWND and rectangle, and bypasses PNG so alpha is never silently demultiplied.
    """

    if len(pixels) != width * height:
        raise ValueError("surface dimensions and pixel count disagree")

    tolerance = 8
    palette_counts = dict.fromkeys(PALETTE, 0)
    transparent_pixels = 0
    semi_transparent_pixels = 0
    semantic_edge_candidates = 0
    semantic_edge_pixels = 0
    invalid_premultiplied_pixels = 0
    opaque_goose_pixels: list[int] = []
    shadow_candidates: list[int] = []
    orange_mask = [False] * (width * height)
    opaque_near_black_mask = [False] * (width * height)
    content_mask = [False] * (width * height)

    for index, (red, green, blue, alpha) in enumerate(pixels):
        if any(channel > alpha for channel in (red, green, blue)):
            invalid_premultiplied_pixels += 1
        if alpha <= 3:
            transparent_pixels += 1
        else:
            content_mask[index] = True
        if 4 <= alpha <= 244:
            semi_transparent_pixels += 1
        if alpha >= 245 and max(red, green, blue) <= 12:
            opaque_near_black_mask[index] = True

        straight = (
            tuple(min(255, round(channel * 255 / alpha)) for channel in (red, green, blue))
            if alpha
            else (0, 0, 0)
        )
        if 38 <= alpha <= 217:
            semantic_edge_candidates += 1
            if any(_close(straight, expected, 20) for expected in PALETTE.values()):
                semantic_edge_pixels += 1

        matches: list[tuple[float, str]] = []
        if alpha >= 245:
            for name, expected in PALETTE.items():
                if _close(straight, expected, tolerance):
                    distance = sum(
                        (actual - target) ** 2
                        for actual, target in zip(straight, expected)
                    )
                    matches.append((distance, name))
        if matches:
            _distance, owner = min(matches)
            palette_counts[owner] += 1
            if owner in {"body", "shade", "wing", "outline"}:
                opaque_goose_pixels.append(index)
            if owner in {"orange", "orange_dark"}:
                orange_mask[index] = True

        if (
            8 <= alpha <= 100
            and all(12 <= channel <= 55 for channel in straight)
            and max(straight) - min(straight) <= 20
        ):
            shadow_candidates.append(index)

    orange_components = _components(orange_mask, width, height, minimum_size=3)
    opaque_near_black_components = _components(
        opaque_near_black_mask,
        width,
        height,
        minimum_size=4,
    )
    opaque_goose_bottom = max(
        (index // width for index in opaque_goose_pixels),
        default=height,
    )
    shadow_pixels = sum(
        1 for index in shadow_candidates if index // width > opaque_goose_bottom
    )
    content_bounds = _mask_bounds(content_mask, width)
    pose_kind, pose_checks = _classify_pose(
        palette_counts,
        orange_components,
        shadow_pixels,
    )
    total = width * height
    largest_opaque_near_black_component = max(
        (component["pixels"] for component in opaque_near_black_components),
        default=0,
    )
    checks = {
        "premultiplied_channel_bounds": invalid_premultiplied_pixels == 0,
        "transparent_surface_margin": transparent_pixels >= max(25, total * 4 // 5),
        "no_opaque_black_surface": (
            largest_opaque_near_black_component <= max(25, total // 100)
        ),
        "complete_content_margin": _has_complete_margin(content_bounds, width, height),
        "visible_body": palette_counts["body"] >= PALETTE_MINIMUMS["body"],
        "visible_shade": palette_counts["shade"] >= PALETTE_MINIMUMS["shade"],
        "visible_wing": palette_counts["wing"] >= PALETTE_MINIMUMS["wing"],
        "visible_outline": palette_counts["outline"] >= PALETTE_MINIMUMS["outline"],
        "asymmetric_orange_channels": (
            palette_counts["orange"] >= TOP_DOWN_ORANGE_MINIMUM
        ),
        "semi_transparent_edges": semi_transparent_pixels >= 20,
        "semantic_edge_colors": (
            semantic_edge_pixels >= 50
            and semantic_edge_pixels * 2 >= semantic_edge_candidates
        ),
        "view_appropriate_articulation": pose_kind != "unknown",
    }
    return {
        "passed": all(checks.values()),
        "mode": "exact-layered-presenter-surface",
        "pose_kind": pose_kind,
        "dimensions": [width, height],
        "present": present,
        "checks": checks,
        "pose_checks": pose_checks,
        "counts": {
            "transparent": transparent_pixels,
            "semi_transparent": semi_transparent_pixels,
            "semantic_edge_candidates": semantic_edge_candidates,
            "semantic_edge_colors": semantic_edge_pixels,
            "invalid_premultiplied": invalid_premultiplied_pixels,
            "shadow": shadow_pixels,
            "largest_opaque_near_black_component": largest_opaque_near_black_component,
            "palette": palette_counts,
        },
        "orange_components": orange_components,
        "content_bounds": content_bounds,
        "opaque_near_black_components": opaque_near_black_components,
    }


def read_presented_surface(
    path: Path,
) -> tuple[int, int, list[tuple[int, int, int, int]], dict]:
    data = path.read_bytes()
    try:
        header, payload = data.split(b"\n\n", 1)
    except ValueError as error:
        raise ValueError(f"{path} has no presenter-record header boundary") from error
    lines = header.splitlines()
    if not lines or lines[0] != PRESENTER_MAGIC:
        raise ValueError(f"{path} has an unknown presenter-record magic")

    fields: dict[str, str] = {}
    for raw_line in lines[1:]:
        try:
            raw_key, raw_value = raw_line.split(b"=", 1)
            key = raw_key.decode("ascii")
            value = raw_value.decode("ascii")
        except (ValueError, UnicodeDecodeError) as error:
            raise ValueError(f"{path} has an invalid presenter-record field") from error
        if key in fields:
            raise ValueError(f"{path} repeats presenter-record field {key}")
        fields[key] = value

    required = {"hwnd", "x", "y", "width", "height", "stride", "bytes"}
    if set(fields) != required:
        raise ValueError(
            f"{path} presenter fields differ: missing={sorted(required - set(fields))}, "
            f"unexpected={sorted(set(fields) - required)}"
        )
    try:
        hwnd = int(fields["hwnd"], 16)
        x = int(fields["x"])
        y = int(fields["y"])
        width = int(fields["width"])
        height = int(fields["height"])
        stride = int(fields["stride"])
        declared_bytes = int(fields["bytes"])
    except ValueError as error:
        raise ValueError(f"{path} has a non-numeric presenter-record field") from error
    if hwnd <= 0 or width <= 0 or height <= 0:
        raise ValueError(f"{path} has an invalid HWND or dimensions")
    expected_bytes = width * height * 4
    if stride != width * 4 or declared_bytes != expected_bytes or len(payload) != expected_bytes:
        raise ValueError(
            f"{path} presenter byte layout differs: stride={stride}, "
            f"declared={declared_bytes}, actual={len(payload)}, expected={expected_bytes}"
        )

    # The selected top-down DIB is premultiplied BGRA. Reorder only; do not demultiply.
    pixels = [
        (red, green, blue, alpha)
        for blue, green, red, alpha in struct.iter_unpack("<BBBB", payload)
    ]
    present = {
        "hwnd": f"0x{hwnd:X}",
        "rect": [x, y, width, height],
        "stride": stride,
        "bytes": declared_bytes,
    }
    return width, height, pixels, present


def analyze_surface_file(path: Path) -> dict:
    width, height, pixels, present = read_presented_surface(path)
    return analyze_surface(width, height, pixels, present)


def _parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser()
    parser.add_argument("--dark", type=Path)
    parser.add_argument("--light", type=Path)
    parser.add_argument("--surface", type=Path)
    parser.add_argument("--dark-bg", default="203040")
    parser.add_argument("--light-bg", default="f4ede4")
    parser.add_argument("--output", type=Path)
    return parser


def main(argv: Iterable[str] | None = None) -> int:
    parser = _parser()
    args = parser.parse_args(argv)
    try:
        if args.surface is not None:
            if args.dark is not None or args.light is not None:
                parser.error("--surface cannot be combined with --dark/--light")
            result = analyze_surface_file(args.surface)
        else:
            if args.dark is None or args.light is None:
                parser.error("paired capture analysis requires --dark and --light")
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
