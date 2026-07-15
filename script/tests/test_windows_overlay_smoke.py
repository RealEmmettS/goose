import importlib.util
import struct
import tempfile
import unittest
import zlib
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
ANALYZER_PATH = ROOT / "script" / "analyze_windows_overlay_capture.py"
SPEC = importlib.util.spec_from_file_location("windows_overlay_analyzer", ANALYZER_PATH)
assert SPEC and SPEC.loader
ANALYZER = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(ANALYZER)


def composite(pixels, background):
    result = []
    for red, green, blue, alpha in pixels:
        result.append(
            tuple(
                (foreground * alpha + backdrop * (255 - alpha) + 127) // 255
                for foreground, backdrop in zip((red, green, blue), background)
            )
            + (255,)
        )
    return result


def write_rgba_png(path, width, height, pixels):
    rows = []
    for row in range(height):
        payload = bytearray()
        for pixel in pixels[row * width : (row + 1) * width]:
            payload.extend(pixel)
        rows.append(b"\x00" + bytes(payload))
    raw = b"".join(rows)

    def chunk(kind, payload):
        return (
            struct.pack(">I", len(payload))
            + kind
            + payload
            + struct.pack(">I", zlib.crc32(kind + payload) & 0xFFFFFFFF)
        )

    path.write_bytes(
        b"\x89PNG\r\n\x1a\n"
        + chunk("IHDR".encode(), struct.pack(">IIBBBBB", width, height, 8, 6, 0, 0, 0))
        + chunk("IDAT".encode(), zlib.compress(raw))
        + chunk("IEND".encode(), b"")
    )


class WindowsOverlayCaptureAnalyzerTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        golden = ROOT / "crates" / "honk-engine" / "tests" / "golden" / "side_mid_stride.png"
        cls.width, cls.height, cls.source = ANALYZER.read_png_rgba(golden)
        cls.dark_background = (0x20, 0x30, 0x40)
        cls.light_background = (0xF4, 0xED, 0xE4)

    def analyze(self, dark_pixels, light_pixels):
        return ANALYZER.analyze_captures(
            self.width,
            self.height,
            dark_pixels,
            light_pixels,
            self.dark_background,
            self.light_background,
        )

    def test_committed_side_golden_proves_articulated_alpha_composition(self):
        result = self.analyze(
            composite(self.source, self.dark_background),
            composite(self.source, self.light_background),
        )
        self.assertTrue(result["passed"], result)
        self.assertGreaterEqual(len(result["orange_components"]), 2)
        self.assertTrue(result["checks"]["semi_transparent_shadow"])

    def test_red_blue_swap_fails_asymmetric_palette_check(self):
        swapped = [(blue, green, red, alpha) for red, green, blue, alpha in self.source]
        result = self.analyze(
            composite(swapped, self.dark_background),
            composite(swapped, self.light_background),
        )
        self.assertFalse(result["passed"])
        self.assertFalse(result["checks"]["asymmetric_orange_channels"])

    def test_shade_pixels_cannot_double_count_as_outline(self):
        without_outline = []
        for red, green, blue, alpha in self.source:
            candidates = []
            for name, expected in ANALYZER.PALETTE.items():
                distance = sum(
                    (actual - target) ** 2
                    for actual, target in zip((red, green, blue), expected)
                )
                if max(
                    abs(actual - target)
                    for actual, target in zip((red, green, blue), expected)
                ) <= 8:
                    candidates.append((distance, name))
            owner = min(candidates)[1] if candidates else None
            if owner == "outline":
                red, green, blue = ANALYZER.PALETTE["shade"]
            without_outline.append((red, green, blue, alpha))

        result = self.analyze(
            composite(without_outline, self.dark_background),
            composite(without_outline, self.light_background),
        )
        self.assertFalse(result["passed"])
        self.assertTrue(result["checks"]["visible_shade"])
        self.assertFalse(result["checks"]["visible_outline"])

    def test_double_premultiplication_fails_shadow_reconstruction(self):
        double_premultiplied = [
            tuple((channel * alpha + 127) // 255 for channel in (red, green, blue)) + (alpha,)
            for red, green, blue, alpha in self.source
        ]
        result = self.analyze(
            composite(double_premultiplied, self.dark_background),
            composite(double_premultiplied, self.light_background),
        )
        self.assertFalse(result["passed"])
        self.assertFalse(result["checks"]["semi_transparent_shadow"])

    def test_opaque_flattening_fails_alpha_and_background_checks(self):
        flattened = [(red, green, blue, 255) for red, green, blue, _alpha in self.source]
        dark = composite(flattened, self.dark_background)
        result = self.analyze(dark, dark)
        self.assertFalse(result["passed"])
        self.assertFalse(result["checks"]["controlled_transparent_background"])
        self.assertFalse(result["checks"]["semi_transparent_shadow"])

    def test_large_opaque_black_margin_cannot_hide_behind_valid_goose_pixels(self):
        dark = composite(self.source, self.dark_background)
        light = composite(self.source, self.light_background)
        transparent_indices = [
            index for index, (_red, _green, _blue, alpha) in enumerate(self.source) if alpha == 0
        ]
        # Preserve only six percent of the canvas as controlled background.  This
        # reproduces the layered-surface corruption that the former five-percent
        # floor accepted while leaving every articulated goose pixel untouched.
        keep = self.width * self.height * 6 // 100
        for index in transparent_indices[keep:]:
            dark[index] = (0, 0, 0, 255)
            light[index] = (0, 0, 0, 255)

        result = self.analyze(dark, light)
        self.assertFalse(result["passed"], result)
        self.assertFalse(result["checks"]["controlled_transparent_background"])
        self.assertFalse(result["checks"]["no_opaque_black_surface"])
        self.assertGreater(
            result["counts"]["largest_unchanged_near_black_component"],
            self.width * self.height // 100,
        )

    def test_png_file_round_trip_uses_standard_library_decoder(self):
        dark = composite(self.source, self.dark_background)
        light = composite(self.source, self.light_background)
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            dark_path = root / "dark.png"
            light_path = root / "light.png"
            write_rgba_png(dark_path, self.width, self.height, dark)
            write_rgba_png(light_path, self.width, self.height, light)
            result = ANALYZER.analyze_files(
                dark_path,
                light_path,
                self.dark_background,
                self.light_background,
            )
        self.assertTrue(result["passed"], result)


class WindowsOverlaySmokeContractTests(unittest.TestCase):
    def test_controller_and_background_enter_pmv2_before_winforms(self):
        smoke = (ROOT / "script" / "smoke_windows_overlay.ps1").read_text(encoding="utf-8")
        enable = "[Honk300DpiAwareness]::EnablePerMonitorV2()"
        first_winforms_load = "Add-Type -AssemblyName System.Windows.Forms"
        for required in (
            "new IntPtr(-4)",
            "SetProcessDpiAwarenessContext",
            "SetThreadDpiAwarenessContext",
            "AreDpiAwarenessContextsEqual",
            "GetDpiForWindow",
            enable,
        ):
            self.assertIn(required, smoke)
        self.assertLess(smoke.index(enable), smoke.index(first_winforms_load))

    def test_background_channel_is_atomic_versioned_and_retry_safe(self):
        smoke = (ROOT / "script" / "smoke_windows_overlay.ps1").read_text(encoding="utf-8")
        for required in (
            "function Write-TextFileAtomically",
            "function Read-SharedTextFile",
            "[System.IO.FileShare]::Delete",
            "[System.IO.File]::Move($temporaryPath, $Path, $true)",
            "color.request",
            "$requestToken = [Guid]::NewGuid().ToString('N')",
            "Write-TextFileAtomically -Path $colorRequestPath",
            "Write-TextFileAtomically -Path $ackPath",
            "Write-TextFileAtomically -Path $readyPath",
            "Wait-ForBackgroundReady -Expected $Hex -Token $requestToken",
        ):
            self.assertIn(required, smoke)
        self.assertNotIn("color.txt", smoke)
        self.assertNotIn("Remove-Item -LiteralPath $ackPath", smoke)
        self.assertNotIn("Get-Content -LiteralPath $ackPath", smoke)

    def test_pre_runtime_capture_proves_background_and_records_dpi_geometry(self):
        smoke = (ROOT / "script" / "smoke_windows_overlay.ps1").read_text(encoding="utf-8")
        for required in (
            "background-diagnostics.txt",
            "capture-diagnostics.txt",
            "background-proof-dark.png",
            "background-proof-light.png",
            "background-proof.txt",
            "Assert-ControlledBackgroundCapture",
            "overlay_hwnd=0x",
            "overlay_dpi=",
            "virtual_screen=",
        ):
            self.assertIn(required, smoke)
        self.assertLess(
            smoke.index("$darkProofCapture ="),
            smoke.index("$runtime = Start-ExactRuntime -Label 'first'"),
        )

    def test_background_geometry_parser_accepts_windows_crlf_diagnostics(self):
        smoke = (ROOT / "script" / "smoke_windows_overlay.ps1").read_text(encoding="utf-8")
        self.assertIn("$backgroundDiagnostics -split '\\r?\\n'", smoke)
        self.assertIn("$backgroundDiagnosticLines -notcontains $expectedVirtualScreen", smoke)
        self.assertNotIn('(?m)^$([regex]::Escape($expectedVirtualScreen))$', smoke)

    def test_native_smoke_freezes_one_surface_and_fails_closed_on_semantics(self):
        smoke = (ROOT / "script" / "smoke_windows_overlay.ps1").read_text(encoding="utf-8")
        for required in (
            "[Honk300OverlaySmokeNative]::Suspend($runtime.Id)",
            "[Honk300OverlaySmokeNative]::Resume($runtime.Id)",
            "Save-ScreenRect -Rect $rect -Path $darkCapture",
            "Save-ScreenRect -Rect $rect -Path $lightCapture",
            "const uint CAPTUREBLT = 0x40000000",
            "const uint SRCCOPY = 0x00CC0020",
            "[Honk300OverlaySmokeNative]::CaptureScreen(",
            "$destination = $graphics.GetHdc()",
            "$graphics.ReleaseHdc($destination)",
            "[System.Windows.Forms.SystemInformation]::VirtualScreen",
            "'-Sta'",
            "analyze_windows_overlay_capture.py",
            "if (-not $visualPassed)",
            "Get-FileHash -LiteralPath $resolvedBinary",
            "@('reload')",
            "@('stop')",
            "Start-ExactRuntime -Label 'restart'",
            "@('quiet_hours_enabled', 'pause_on_fullscreen')",
        ):
            self.assertIn(required, smoke)
        self.assertNotIn("[System.Drawing.CopyPixelOperation]::CaptureBlt", smoke)
        self.assertNotIn("$graphics.CopyFromScreen(", smoke)

    def test_ci_and_release_candidate_run_the_exact_binary_smoke(self):
        ci = (ROOT / ".github" / "workflows" / "ci.yml").read_text(encoding="utf-8")
        installers = (ROOT / ".github" / "workflows" / "windows-installers.yml").read_text(
            encoding="utf-8"
        )
        post_release = (ROOT / ".github" / "workflows" / "post-release-smoke.yml").read_text(
            encoding="utf-8"
        )
        released_smoke = (ROOT / "script" / "smoke_released_windows.ps1").read_text(
            encoding="utf-8"
        )
        self.assertIn("-Binary target/release/honk300.exe", ci)
        self.assertIn("-EvidenceDirectory target/windows-overlay-evidence", ci)
        self.assertIn("runs-on: windows-11-arm", ci)
        self.assertIn("host: aarch64-pc-windows-msvc", ci)
        self.assertIn(
            "-Binary target/aarch64-pc-windows-msvc/release/honk300.exe",
            ci,
        )
        self.assertIn("if: ${{ always() }}", ci)
        self.assertIn("-Binary target/${{ matrix.triple }}/release/honk300.exe", installers)
        self.assertIn("qualification-windows-overlay-${{ matrix.triple }}", installers)
        self.assertIn("matrix.triple == 'x86_64-pc-windows-msvc'", installers)
        self.assertIn("qualify-windows-arm64-native:", installers)
        self.assertIn("runs-on: windows-11-arm", installers)
        self.assertIn("needs: build-windows-installers", installers)
        self.assertIn(
            "qualification-input-windows-aarch64-pc-windows-msvc",
            installers,
        )
        self.assertIn("actions/download-artifact@v6", installers)
        self.assertIn("$env:PROCESSOR_ARCHITECTURE -ne 'ARM64'", installers)
        self.assertIn(
            "-SourceBinaryPath target/qualification-input-aarch64-pc-windows-msvc/honk300.exe",
            installers,
        )
        self.assertIn("smoke_windows_overlay.ps1", released_smoke)
        self.assertIn("-Binary $Binary", released_smoke)
        self.assertIn("windows-11-arm", post_release)
        self.assertIn("-TargetTriple '${{ matrix.triple }}'", post_release)
        self.assertIn(
            "honk300-windows-overlay-published-${{ matrix.triple }}-${{ inputs.tag }}",
            post_release,
        )


if __name__ == "__main__":
    unittest.main()
