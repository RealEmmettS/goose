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
BACKGROUND_PATH = ROOT / "script" / "x11_smoke_background.py"
BACKGROUND_SPEC = importlib.util.spec_from_file_location("x11_smoke_background", BACKGROUND_PATH)
assert BACKGROUND_SPEC and BACKGROUND_SPEC.loader
BACKGROUND = importlib.util.module_from_spec(BACKGROUND_SPEC)
sys.modules[BACKGROUND_SPEC.name] = BACKGROUND
BACKGROUND_SPEC.loader.exec_module(BACKGROUND)


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
    def test_accepts_proven_small_top_down_pose_but_still_requires_warm_articulation(self) -> None:
        common = {
            "label": "candidate-top-down",
            "width": 1280,
            "height": 720,
            "dark_background_pixels": 918_768,
            "light_background_pixels": 918_704,
            "background_transition_pixels": 918_689,
            "body_pixels": 710,
            "wing_pixels": 1_615,
            "largest_near_black_component": 0,
            "largest_unchanged_component": 991,
        }
        self.assertTrue(ANALYZER.CaptureMetrics(warm_pixels=13, **common).has_goose)
        self.assertFalse(ANALYZER.CaptureMetrics(warm_pixels=9, **common).has_goose)

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

    def test_cli_accepts_an_explicit_background_only_baseline(self) -> None:
        source = (ROOT / "script" / "analyze_linux_overlay_capture.py").read_text(
            encoding="utf-8"
        )
        self.assertIn('goose.add_argument("--require-no-goose"', source)
        self.assertIn("baseline unexpectedly contains goose palette features", source)

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


class LinuxOverlaySmokeContractTests(unittest.TestCase):
    def test_wayland_runtime_socket_directory_stays_short_and_is_cleaned(self) -> None:
        smoke = (ROOT / "script" / "smoke_m17_m18_linux.sh").read_text(encoding="utf-8")
        self.assertIn('WAYLAND_RUNTIME_DIR="$(mktemp -d /tmp/honk300-wl.XXXXXX)"', smoke)
        self.assertIn('export XDG_RUNTIME_DIR="${WAYLAND_RUNTIME_DIR}"', smoke)
        self.assertIn('rm -rf "${WAYLAND_RUNTIME_DIR}"', smoke)
        self.assertNotIn('export XDG_RUNTIME_DIR="${WORK}/runtime"', smoke)

    def test_x11_capture_uses_client_compositing_and_proves_a_clean_baseline(self) -> None:
        smoke = (ROOT / "script" / "smoke_m17_m18_linux.sh").read_text(encoding="utf-8")
        for required in (
            "xcompmgr -n",
            "validate_x11_capture_baseline",
            "x11-baseline-dark.png",
            "x11-baseline-light.png",
            "--require-no-goose",
            "X11 compositor capture baseline is invalid before launch",
            "capture_x11_background_pairs",
            "--require-goose-each",
        ):
            self.assertIn(required, smoke)
        self.assertNotIn("xcompmgr -a", smoke)

    def test_wayland_capture_owns_a_minimal_compositor_and_exact_output_backgrounds(self) -> None:
        smoke = (ROOT / "script" / "smoke_m17_m18_linux.sh").read_text(encoding="utf-8")
        for required in (
            "need_cmd swaybg",
            "need_cmd convert",
            "sway-smoke.conf",
            "# Honk300 owned compositor; exact output rules are applied over IPC.",
            'sway -c "${WORK}/sway-smoke.conf" -d',
            "wayland-bg-dark.png",
            "wayland-bg-light.png",
            "convert -size 32x32 'xc:#203040'",
            'case "${color}" in',
            "set_wayland_background",
            'swaymsg output "${WAYLAND_FIRST_OUTPUT}" bg "${image}" tile',
            'swaymsg output "${WAYLAND_SECOND_OUTPUT}" bg "${image}" tile',
            "wait_for_wayland_background",
            "matching / len(samples) < 0.90",
            "Wayland background did not settle",
            "validate_wayland_capture_baseline",
            "wayland-baseline-first-dark.png",
            "wayland-baseline-second-light.png",
            "Wayland compositor capture baseline is invalid before launch",
            "--require-no-goose",
        ):
            self.assertIn(required, smoke)
        self.assertNotIn("output * bg", smoke)
        self.assertNotIn('bg "${color}" solid_color', smoke)
        self.assertNotIn(
            'swaymsg output "${WAYLAND_SECOND_OUTPUT}" bg "${image}" tile >/dev/null\n  sleep 0.20',
            smoke,
        )
        self.assertNotIn("WLR_LIBINPUT_NO_DEVICES=1 sway -d", smoke)

    def test_x11_background_is_a_disposable_lowered_client_not_an_app_surface(self) -> None:
        smoke = (ROOT / "script" / "smoke_m17_m18_linux.sh").read_text(encoding="utf-8")
        helper = BACKGROUND_PATH.read_text(encoding="utf-8")
        for required in (
            "script/x11_smoke_background.py",
            'start_x11_background "#203040"',
            'set_x11_background "#203040"',
            'set_x11_background "#d8e6f4"',
            'kill "${X11_BACKGROUND_PID}"',
        ):
            self.assertIn(required, smoke)
        for required in (
            "CW_OVERRIDE_REDIRECT",
            "XChangeWindowAttributes",
            "XLowerWindow",
            "XSetWindowBackground",
            "XClearWindow",
            "honk300-smoke-background",
        ):
            self.assertIn(required, helper)

    def test_x11_background_command_and_ready_files_are_strict(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            command = root / "command"
            ready = root / "ready"
            command.write_text("#203040\n", encoding="ascii")
            self.assertEqual(BACKGROUND.read_color(command), "#203040")
            command.write_text("red\n", encoding="ascii")
            with self.assertRaises(ValueError):
                BACKGROUND.read_color(command)
            BACKGROUND.write_ready(ready, "#d8e6f4")
            self.assertEqual(ready.read_text(encoding="ascii"), "#d8e6f4\n")
            self.assertEqual(ready.stat().st_mode & 0o777, 0o600)


if __name__ == "__main__":
    unittest.main()
