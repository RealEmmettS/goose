from __future__ import annotations

import importlib.util
import struct
import sys
import tempfile
import unittest
import zlib
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
ANALYZER_PATH = ROOT / "script" / "analyze_linux_overlay_capture.py"
SPEC = importlib.util.spec_from_file_location("linux_overlay_analyzer", ANALYZER_PATH)
assert SPEC and SPEC.loader
ANALYZER = importlib.util.module_from_spec(SPEC)
sys.modules[SPEC.name] = ANALYZER
SPEC.loader.exec_module(ANALYZER)


def png_chunk(kind: bytes, payload: bytes) -> bytes:
    return struct.pack(">I", len(payload)) + kind + payload + struct.pack(">I", zlib.crc32(kind + payload))


def write_png(path: Path, width: int, height: int, pixels: list[tuple[int, int, int]]) -> None:
    rows = bytearray()
    for y in range(height):
        rows.append(0)
        for red, green, blue in pixels[y * width : (y + 1) * width]:
            rows.extend((red, green, blue))
    header = struct.pack(">IIBBBBB", width, height, 8, 2, 0, 0, 0)
    path.write_bytes(
        b"\x89PNG\r\n\x1a\n"
        + png_chunk(b"IHDR", header)
        + png_chunk(b"IDAT", zlib.compress(bytes(rows)))
        + png_chunk(b"IEND", b"")
    )


def valid_pair(width: int = 200, height: int = 160) -> tuple[list[tuple[int, int, int]], list[tuple[int, int, int]]]:
    dark = [ANALYZER.DARK_BACKGROUND] * (width * height)
    light = [ANALYZER.LIGHT_BACKGROUND] * (width * height)
    for y in range(45, 85):
        for x in range(65, 130):
            dark[y * width + x] = light[y * width + x] = (235, 235, 232)
    for y in range(60, 78):
        for x in range(78, 105):
            dark[y * width + x] = light[y * width + x] = (82, 85, 88)
    for y in range(67, 75):
        for x in range(130, 146):
            dark[y * width + x] = light[y * width + x] = (230, 125, 48)
    return dark, light


class LinuxOverlayAnalyzerTests(unittest.TestCase):
    def test_committed_goose_golden_passes_paired_compositor_analysis(self) -> None:
        golden = ROOT / "crates" / "honk-engine" / "tests" / "golden" / "side_mid_stride.png"
        width, height, source = ANALYZER.read_png(golden)

        def composite(background: tuple[int, int, int]) -> list[tuple[int, int, int]]:
            result = []
            for offset in range(0, len(source), 4):
                red, green, blue, alpha = source[offset : offset + 4]
                result.append(
                    tuple(
                        (channel * alpha + background[index] * (255 - alpha) + 127) // 255
                        for index, channel in enumerate((red, green, blue))
                    )
                )
            return result

        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            write_png(root / "dark.png", width, height, composite(ANALYZER.DARK_BACKGROUND))
            write_png(root / "light.png", width, height, composite(ANALYZER.LIGHT_BACKGROUND))
            metrics = ANALYZER.analyze_pair("golden", root / "dark.png", root / "light.png")
            self.assertEqual(ANALYZER.validate(metrics), [])
            self.assertTrue(metrics.has_goose)

    def test_accepts_paired_transparent_background_and_articulated_palette(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            dark, light = valid_pair()
            write_png(root / "dark.png", 200, 160, dark)
            write_png(root / "light.png", 200, 160, light)
            metrics = ANALYZER.analyze_pair("output", root / "dark.png", root / "light.png")
            self.assertEqual(ANALYZER.validate(metrics), [])
            self.assertTrue(metrics.has_goose)

    def test_controlled_backgrounds_alone_do_not_count_as_a_goose(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            write_png(root / "dark.png", 200, 160, [ANALYZER.DARK_BACKGROUND] * 32_000)
            write_png(root / "light.png", 200, 160, [ANALYZER.LIGHT_BACKGROUND] * 32_000)
            metrics = ANALYZER.analyze_pair("empty", root / "dark.png", root / "light.png")
            self.assertEqual(ANALYZER.validate(metrics), [])
            self.assertFalse(metrics.has_goose)

    def test_rejects_large_opaque_black_component(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            dark, light = valid_pair()
            for y in range(15, 145):
                for x in range(15, 185):
                    dark[y * 200 + x] = light[y * 200 + x] = (0, 0, 0)
            write_png(root / "dark.png", 200, 160, dark)
            write_png(root / "light.png", 200, 160, light)
            failures = ANALYZER.validate(
                ANALYZER.analyze_pair("output", root / "dark.png", root / "light.png")
            )
            self.assertTrue(any("near-black" in failure for failure in failures))
            self.assertTrue(any("opaque" in failure for failure in failures))

    def test_rejects_missing_background_transition(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            dark, _light = valid_pair()
            write_png(root / "dark.png", 200, 160, dark)
            write_png(root / "light.png", 200, 160, dark)
            failures = ANALYZER.validate(
                ANALYZER.analyze_pair("output", root / "dark.png", root / "light.png")
            )
            self.assertTrue(any("light controlled background" in failure for failure in failures))
            self.assertTrue(any("transition" in failure for failure in failures))


if __name__ == "__main__":
    unittest.main()
