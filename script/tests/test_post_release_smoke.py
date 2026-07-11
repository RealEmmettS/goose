from __future__ import annotations

import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]


def source(path: str) -> str:
    candidate = ROOT / path
    return candidate.read_text(encoding="utf-8") if candidate.is_file() else ""


WORKFLOW = source(".github/workflows/post-release-smoke.yml")
UNIX_SMOKE = source("script/smoke_released_unix.sh")
WINDOWS_SMOKE = source("script/smoke_released_windows.ps1")


class PostReleaseSmokeTests(unittest.TestCase):
    def test_workflow_executes_live_smoke_on_every_primary_host_architecture(self) -> None:
        for runner in (
            "ubuntu-latest",
            "ubuntu-24.04-arm",
            "macos-15",
            "macos-15-intel",
            "windows-2022",
        ):
            self.assertIn(runner, WORKFLOW)
        self.assertIn("smoke_released_unix.sh", WORKFLOW)
        self.assertIn("smoke_released_windows.ps1", WORKFLOW)
        self.assertIn("permissions:\n  contents: read", WORKFLOW)
        self.assertEqual(WORKFLOW.count("HONK300_SMOKE_TAG: ${{ inputs.tag }}"), 2)
        self.assertNotIn("'${{ inputs.tag }}'", WORKFLOW)

    def test_unix_smoke_proves_idempotence_and_complete_fault_rollback(self) -> None:
        self.assertIn("grep -Eq '^v[0-9]+\\.[0-9]+\\.[0-9]+$'", UNIX_SMOKE)
        self.assertIn("for pass in 1 2", UNIX_SMOKE)
        self.assertIn("HONK300_TEST_FAIL_AFTER_SWAP=1", UNIX_SMOKE)
        self.assertIn("snapshot_state", UNIX_SMOKE)
        self.assertIn('cmp "$BEFORE_STATE" "$AFTER_STATE"', UNIX_SMOKE)
        for owned_state in (
            "install-receipt.json",
            "honk300.desktop",
            "managed PATH",
            "autostart",
            ".local/bin/honk300",
            ".local/bin/honk",
            ".local/bin/goose",
            "codesign --verify --deep --strict",
            "lipo",
        ):
            self.assertIn(owned_state, UNIX_SMOKE)
        self.assertNotIn("-maxdepth", UNIX_SMOKE)
        self.assertIn('"$DEST.previous."*', UNIX_SMOKE)

    def test_windows_smoke_forces_mid_install_failure_and_proves_old_version_returns(self) -> None:
        self.assertIn("Honk300ForceRollback", WINDOWS_SMOKE)
        self.assertIn("InstallExecuteSequence", WINDOWS_SMOKE)
        self.assertIn("Sequence 4010", WINDOWS_SMOKE)
        self.assertIn("v0.2.1", WINDOWS_SMOKE)
        self.assertIn("failed upgrade did not restore", WINDOWS_SMOKE)
        self.assertIn("Downgrade unexpectedly succeeded", WINDOWS_SMOKE)
        self.assertIn("administrative extraction", WINDOWS_SMOKE)


if __name__ == "__main__":
    unittest.main()
