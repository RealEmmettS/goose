import importlib.util
import json
from pathlib import Path
import tempfile
import unittest


SPEC = importlib.util.spec_from_file_location(
    "windows_profile_summary", Path(__file__).resolve().parents[1] / "summarize_windows_profile.py"
)
SUMMARY = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(SUMMARY)


class WindowsProfileSummaryTests(unittest.TestCase):
    def test_irregular_sample_intervals_use_total_cpu_and_frame_bounds_exclude_teardown(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            data = {
                "environment.json": {"logical_processors": 20, "warmup_seconds": 30},
                "result.json": {"completed": True, "sample_start_unix_ms": 2000,
                                "sample_end_unix_ms": 3000},
                "samples.json": [
                    {"elapsed_seconds": 0, "cpu_seconds": 0},
                    {"elapsed_seconds": 10, "cpu_seconds": 1},
                    {"elapsed_seconds": 30, "cpu_seconds": 5},
                ],
                "frames.json": {"started_unix_ms": 1000, "retained_frames": 3,
                                "samples": [self.frame(500, 99999), self.frame(1500, 123),
                                            self.frame(2500, 88888)]},
            }
            for name, value in data.items():
                (root / name).write_text(json.dumps(value), encoding="utf-8")
            report = SUMMARY.summarize(root)
            self.assertAlmostEqual(report["cpu"]["average_one_core_percent"], 100 * 5 / 30)
            self.assertAlmostEqual(report["cpu"]["average_machine_percent"], 5 / 6)
            self.assertEqual(report["cpu"]["median_one_core_percent"], 15)
            self.assertEqual(report["frames"]["count"], 1)
            self.assertEqual(report["frames"]["total_us"]["p99"], 123)
            self.assertEqual(report["frames"]["bitmap_allocations"], 1)

            data["result.json"].update(completed=False, sample_end_unix_ms=None)
            (root / "result.json").write_text(json.dumps(data["result.json"]), encoding="utf-8")
            failed = SUMMARY.summarize(root)
            self.assertFalse(failed["result"]["completed"])
            self.assertEqual(failed["frames"]["omitted"], "incomplete_interval")

    @staticmethod
    def frame(elapsed, duration):
        return {"elapsed_ms": elapsed, "render_us": duration, "allocation_us": 0,
                "copy_us": 0, "update_us": 0, "total_us": duration,
                "bitmap_allocations": 1, "presented_pixels": 10}


if __name__ == "__main__":
    unittest.main()
