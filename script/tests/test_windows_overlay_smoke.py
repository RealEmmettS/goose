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


def premultiply_rgba(pixels):
    return [
        tuple((channel * alpha + 127) // 255 for channel in (red, green, blue)) + (alpha,)
        for red, green, blue, alpha in pixels
    ]


def write_presented_bgra(path, width, height, pixels, hwnd=0x1234, x=-17, y=29):
    payload = b"".join(bytes((blue, green, red, alpha)) for red, green, blue, alpha in pixels)
    header = (
        "HONK300_LAYERED_BGRA_V1\n"
        f"hwnd=0x{hwnd:X}\n"
        f"x={x}\n"
        f"y={y}\n"
        f"width={width}\n"
        f"height={height}\n"
        f"stride={width * 4}\n"
        f"bytes={len(payload)}\n\n"
    ).encode("ascii")
    path.write_bytes(header + payload)


class WindowsOverlayCaptureAnalyzerTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        golden = ROOT / "crates" / "honk-engine" / "tests" / "golden" / "side_mid_stride.png"
        cls.width, cls.height, cls.source = ANALYZER.read_png_rgba(golden)
        cls.presented = premultiply_rgba(cls.source)
        top_down_golden = (
            ROOT / "crates" / "honk-engine" / "tests" / "golden" / "top_down.png"
        )
        cls.top_down_width, cls.top_down_height, cls.top_down_source = (
            ANALYZER.read_png_rgba(top_down_golden)
        )
        cls.top_down_presented = premultiply_rgba(cls.top_down_source)
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
        self.assertEqual(result["pose_kind"], "side")
        self.assertGreaterEqual(len(result["orange_components"]), 2)
        self.assertTrue(result["pose_checks"]["side"]["semi_transparent_shadow"])

    def test_committed_side_golden_proves_exact_layered_presenter_surface(self):
        result = ANALYZER.analyze_surface(self.width, self.height, self.presented)
        self.assertTrue(result["passed"], result)
        self.assertEqual(result["mode"], "exact-layered-presenter-surface")
        self.assertTrue(result["checks"]["premultiplied_channel_bounds"])
        self.assertTrue(result["checks"]["transparent_surface_margin"])
        self.assertTrue(result["checks"]["no_opaque_black_surface"])
        self.assertEqual(result["pose_kind"], "side")
        self.assertTrue(result["pose_checks"]["side"]["visible_beak_and_two_legs"])
        self.assertTrue(result["pose_checks"]["side"]["semi_transparent_shadow"])

    def test_committed_top_down_golden_proves_articulated_alpha_composition(self):
        result = ANALYZER.analyze_captures(
            self.top_down_width,
            self.top_down_height,
            composite(self.top_down_source, self.dark_background),
            composite(self.top_down_source, self.light_background),
            self.dark_background,
            self.light_background,
        )
        self.assertTrue(result["passed"], result)
        self.assertEqual(result["pose_kind"], "top-down")
        self.assertTrue(result["checks"]["asymmetric_orange_channels"])
        self.assertTrue(result["checks"]["semantic_edge_colors"])
        self.assertTrue(result["pose_checks"]["top_down"]["single_compact_beak"])
        self.assertTrue(result["pose_checks"]["top_down"]["no_ground_shadow"])
        self.assertFalse(result["pose_checks"]["side"]["visible_beak_and_two_legs"])

    def test_committed_top_down_golden_proves_exact_layered_presenter_surface(self):
        result = ANALYZER.analyze_surface(
            self.top_down_width,
            self.top_down_height,
            self.top_down_presented,
        )
        self.assertTrue(result["passed"], result)
        self.assertEqual(result["pose_kind"], "top-down")
        self.assertTrue(result["checks"]["premultiplied_channel_bounds"])
        self.assertTrue(result["checks"]["asymmetric_orange_channels"])
        self.assertTrue(result["checks"]["semantic_edge_colors"])
        self.assertTrue(result["pose_checks"]["top_down"]["single_compact_beak"])

    def test_top_down_rejects_double_premultiplication_in_both_evidence_paths(self):
        doubled_straight = premultiply_rgba(self.top_down_source)
        paired_result = ANALYZER.analyze_captures(
            self.top_down_width,
            self.top_down_height,
            composite(doubled_straight, self.dark_background),
            composite(doubled_straight, self.light_background),
            self.dark_background,
            self.light_background,
        )
        self.assertFalse(paired_result["passed"], paired_result)
        self.assertFalse(paired_result["checks"]["semantic_edge_colors"])

        doubled_surface = premultiply_rgba(self.top_down_presented)
        surface_result = ANALYZER.analyze_surface(
            self.top_down_width,
            self.top_down_height,
            doubled_surface,
        )
        self.assertFalse(surface_result["passed"], surface_result)
        self.assertFalse(surface_result["checks"]["semantic_edge_colors"])

    def test_top_down_requires_warm_beak_and_rejects_red_blue_swap(self):
        without_warm = [
            (red, green, blue, 0)
            if (red, green, blue) == ANALYZER.PALETTE["orange"]
            else (red, green, blue, alpha)
            for red, green, blue, alpha in self.top_down_source
        ]
        without_warm_result = ANALYZER.analyze_captures(
            self.top_down_width,
            self.top_down_height,
            composite(without_warm, self.dark_background),
            composite(without_warm, self.light_background),
            self.dark_background,
            self.light_background,
        )
        self.assertFalse(without_warm_result["passed"], without_warm_result)
        self.assertFalse(
            without_warm_result["checks"]["asymmetric_orange_channels"]
        )

        swapped = [
            (blue, green, red, alpha)
            for red, green, blue, alpha in self.top_down_source
        ]
        swapped_result = ANALYZER.analyze_captures(
            self.top_down_width,
            self.top_down_height,
            composite(swapped, self.dark_background),
            composite(swapped, self.light_background),
            self.dark_background,
            self.light_background,
        )
        self.assertFalse(swapped_result["passed"], swapped_result)
        self.assertFalse(swapped_result["checks"]["asymmetric_orange_channels"])

    def test_damaged_side_view_cannot_fall_through_top_down_profile(self):
        damaged = []
        for red, green, blue, alpha in self.source:
            if (red, green, blue) == ANALYZER.PALETTE["orange_dark"]:
                alpha = 0
            if alpha <= 100 and max(red, green, blue) <= 55:
                alpha = 0
            damaged.append((red, green, blue, alpha))

        result = self.analyze(
            composite(damaged, self.dark_background),
            composite(damaged, self.light_background),
        )
        self.assertFalse(result["passed"], result)
        self.assertEqual(result["pose_kind"], "unknown")
        self.assertFalse(result["pose_checks"]["side"]["two_tone_orange"])
        self.assertFalse(
            result["pose_checks"]["top_down"]["top_down_wing_body_ratio"]
        )

    def test_bottom_cropped_side_view_cannot_fall_through_top_down_profile(self):
        first_removed_row = self.height - 73
        cropped = [
            pixel if index // self.width < first_removed_row else (0, 0, 0, 0)
            for index, pixel in enumerate(self.source)
        ]
        result = self.analyze(
            composite(cropped, self.dark_background),
            composite(cropped, self.light_background),
        )
        self.assertFalse(result["passed"], result)
        self.assertEqual(result["pose_kind"], "unknown")
        self.assertFalse(result["pose_checks"]["side"]["two_tone_orange"])
        self.assertFalse(
            result["pose_checks"]["top_down"]["top_down_beak_body_ratio"]
        )

    def test_half_cropped_top_down_view_is_not_complete_evidence(self):
        first_removed_column = 90
        cropped = [
            pixel if index % self.top_down_width < first_removed_column else (0, 0, 0, 0)
            for index, pixel in enumerate(self.top_down_source)
        ]
        result = ANALYZER.analyze_surface(
            self.top_down_width,
            self.top_down_height,
            premultiply_rgba(cropped),
        )
        self.assertFalse(result["passed"], result)
        self.assertEqual(result["pose_kind"], "unknown")
        self.assertFalse(
            result["pose_checks"]["top_down"]["complete_top_down_palette"]
        )

    def test_layered_presenter_surface_rejects_straight_opaque_or_channel_swapped_output(self):
        straight_result = ANALYZER.analyze_surface(self.width, self.height, self.source)
        self.assertFalse(straight_result["passed"], straight_result)
        self.assertFalse(straight_result["checks"]["premultiplied_channel_bounds"])

        opaque = [(red, green, blue, 255) for red, green, blue, _alpha in self.source]
        opaque_result = ANALYZER.analyze_surface(self.width, self.height, opaque)
        self.assertFalse(opaque_result["passed"], opaque_result)
        self.assertFalse(opaque_result["checks"]["transparent_surface_margin"])

        swapped = [(blue, green, red, alpha) for red, green, blue, alpha in self.presented]
        swapped_result = ANALYZER.analyze_surface(self.width, self.height, swapped)
        self.assertFalse(swapped_result["passed"], swapped_result)
        self.assertFalse(swapped_result["checks"]["asymmetric_orange_channels"])

    def test_layered_presenter_surface_rejects_double_premultiplication(self):
        doubled = premultiply_rgba(self.presented)
        result = ANALYZER.analyze_surface(self.width, self.height, doubled)
        self.assertFalse(result["passed"], result)
        self.assertFalse(result["pose_checks"]["side"]["semi_transparent_shadow"])

    def test_layered_presenter_surface_rejects_mostly_opaque_black_margin(self):
        damaged = list(self.presented)
        transparent = [
            index for index, (_red, _green, _blue, alpha) in enumerate(damaged) if alpha == 0
        ]
        keep = self.width * self.height * 5 // 100
        for index in transparent[keep:]:
            damaged[index] = (0, 0, 0, 255)
        result = ANALYZER.analyze_surface(self.width, self.height, damaged)
        self.assertFalse(result["passed"], result)
        self.assertFalse(result["checks"]["transparent_surface_margin"])
        self.assertFalse(result["checks"]["no_opaque_black_surface"])

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
        self.assertFalse(result["pose_checks"]["side"]["semi_transparent_shadow"])

    def test_opaque_flattening_fails_alpha_and_background_checks(self):
        flattened = [(red, green, blue, 255) for red, green, blue, _alpha in self.source]
        dark = composite(flattened, self.dark_background)
        result = self.analyze(dark, dark)
        self.assertFalse(result["passed"])
        self.assertFalse(result["checks"]["controlled_transparent_background"])
        self.assertFalse(result["pose_checks"]["side"]["semi_transparent_shadow"])

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

    def test_raw_presenter_record_round_trip_preserves_bgra_alpha_and_window_binding(self):
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "present.bgra"
            write_presented_bgra(path, self.width, self.height, self.presented)
            result = ANALYZER.analyze_surface_file(path)
        self.assertTrue(result["passed"], result)
        self.assertEqual(result["present"]["hwnd"], "0x1234")
        self.assertEqual(result["present"]["rect"], [-17, 29, self.width, self.height])
        self.assertEqual(result["counts"]["invalid_premultiplied"], 0)


class WindowsOverlaySmokeContractTests(unittest.TestCase):
    def test_taskbar_recovery_allows_bounded_shell_rect_settle_and_records_latency(self):
        smoke = (ROOT / "script" / "smoke_windows_overlay.ps1").read_text(encoding="utf-8")
        self.assertIn("$deletionSettlePolls = 0", smoke)
        self.assertIn("continuously absent GUID state", smoke)
        self.assertIn("deletion_settle_polls=$deletionSettlePolls", smoke)
        self.assertIn("for ($attempt = 0; $attempt -lt 400; $attempt += 1)", smoke)
        self.assertIn("$recoveryPollAttempts = $attempt + 1", smoke)
        self.assertIn("recovery_poll_attempts=$recoveryPollAttempts", smoke)
        self.assertIn("after $recoveryPollAttempts polls; runtime: $runtimeMessage", smoke)

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
            "Measure-ControlledBackgroundCapture",
            "overlay_hwnd=0x",
            "overlay_dpi=",
            "virtual_screen=",
        ):
            self.assertIn(required, smoke)
        self.assertLess(
            smoke.index("$darkProofCapture ="),
            smoke.index("$runtime = Start-ExactRuntime -Label 'first'"),
        )

    def test_hosted_arm64_wallpaper_capture_uses_a_strict_presenter_surface_fallback(self):
        smoke = (ROOT / "script" / "smoke_windows_overlay.ps1").read_text(encoding="utf-8")
        windows_backend = (
            ROOT / "crates" / "honk-platform-windows" / "src" / "lib.rs"
        ).read_text(encoding="utf-8")
        for required in (
            "$env:GITHUB_ACTIONS -eq 'true'",
            "$env:RUNNER_ENVIRONMENT -eq 'github-hosted'",
            "$env:RUNNER_OS -eq 'Windows'",
            "$env:RUNNER_ARCH -eq 'ARM64'",
            "$env:PROCESSOR_ARCHITECTURE -eq 'ARM64'",
            "$darkProof.Coverage -le 0.01",
            "$lightProof.Coverage -le 0.01",
            "$darkProofHash -eq $lightProofHash",
            "hosted-arm64-presenter-surface",
            "HONK300_WINDOWS_SMOKE_PRESENT",
            "--surface",
            "overlay-present.bgra",
            "window_visible=true",
            "$analysisDocument.present.hwnd",
            "$analysisDocument.present.rect",
            "$presenterRectTolerancePixels = 3",
            "$actualHwnd -cne $expectedHwnd",
            "$deltaX -le $presenterRectTolerancePixels",
            "$deltaY -le $presenterRectTolerancePixels",
            "$deltaWidth -le $presenterRectTolerancePixels",
            "$deltaHeight -le $presenterRectTolerancePixels",
            "rect_deltas=$rectDeltas tolerance=$presenterRectTolerancePixels",
            "Remove-Item -LiteralPath $rendererPresentPath -Force -ErrorAction Stop",
            "could not clear the previous presenter record",
            "unknown Windows overlay capture mode",
            "unknown Windows overlay evidence mode",
        ):
            self.assertIn(required, smoke)
        self.assertNotIn("$env:RUNNER_ENVIRONMENT -ne 'self-hosted'", smoke)
        self.assertIn('std::env::var("HONK300_WINDOWS_SMOKE_PRESENT")', windows_backend)
        self.assertIn("HONK300_LAYERED_BGRA_V1", windows_backend)
        self.assertIn("file.write_all(bytes)", windows_backend)
        self.assertIn("fs::rename(&temporary, path)", windows_backend)
        self.assertNotIn("pixmap.save_png", windows_backend)
        self.assertIn("$captureMode = 'paired-dwm'", smoke)
        self.assertIn(
            "'HONK300_WINDOWS_SMOKE_PRESENT',\n                $null,",
            smoke,
        )
        self.assertNotIn("$actualRect -cne $expectedRect", smoke)
        self.assertIn("Start-Sleep -Milliseconds 5", smoke)

        overlay_window = windows_backend[windows_backend.index("impl OverlayWindow") :]
        overlay_present = overlay_window[: overlay_window.index("fn hide(&mut self)")]
        self.assertLess(
            overlay_present.index("present_layered("),
            overlay_present.index("maybe_write_presented_smoke_frame("),
        )

        visual_loop = smoke[smoke.index("for ($attempt = 1;") : smoke.index("if (-not $visualPassed)")]
        self.assertLess(
            visual_loop.index("Start-Sleep -Milliseconds 900"),
            visual_loop.index("Remove-Item -LiteralPath $rendererPresentPath"),
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
        smoke = (ROOT / "script" / "smoke_windows_overlay.ps1").read_text(encoding="utf-8")
        ci = (ROOT / ".github" / "workflows" / "ci.yml").read_text(encoding="utf-8")
        release = (ROOT / ".github" / "workflows" / "release.yml").read_text(
            encoding="utf-8"
        )
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
        self.assertIn("ProbeNotificationArea", smoke)
        self.assertIn("independent_shell_probe", smoke)
        self.assertIn("runs-on: windows-11-arm", ci)
        self.assertIn("host: aarch64-pc-windows-msvc", ci)
        self.assertIn(
            "-Binary target/aarch64-pc-windows-msvc/release/honk300.exe",
            ci,
        )
        self.assertIn("-AllowUnavailableTrayHost", ci)
        self.assertIn("-AllowUnavailableTrayHost:$allowUnavailableTrayHost", release)
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
        self.assertIn("-AllowUnavailableTrayHost:", released_smoke)
        self.assertIn("windows-11-arm", post_release)
        self.assertIn("-TargetTriple '${{ matrix.triple }}'", post_release)
        self.assertIn(
            "honk300-windows-overlay-published-${{ matrix.triple }}-${{ inputs.tag }}",
            post_release,
        )


if __name__ == "__main__":
    unittest.main()
