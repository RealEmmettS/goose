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
DEBIAN_SMOKE = source("script/smoke_released_deb.sh")


class PostReleaseSmokeTests(unittest.TestCase):
    def test_workflow_executes_live_smoke_on_every_primary_host_architecture(self) -> None:
        for runner in (
            "ubuntu-22.04",
            "ubuntu-22.04-arm",
            "macos-15",
            "macos-15-intel",
            "windows-2022",
            "windows-11-arm",
        ):
            self.assertIn(runner, WORKFLOW)
        self.assertIn("smoke_released_unix.sh", WORKFLOW)
        self.assertIn("smoke_released_windows.ps1", WORKFLOW)
        self.assertIn("permissions:\n  contents: read", WORKFLOW)
        self.assertEqual(WORKFLOW.count("HONK300_SMOKE_TAG: ${{ inputs.tag }}"), 4)
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
        self.assertNotIn("Downgrade unexpectedly succeeded", WINDOWS_SMOKE)
        self.assertIn("honk300.install.v2", WINDOWS_SMOKE)
        self.assertIn("stable current junction", WINDOWS_SMOKE)
        self.assertIn("administrative extraction", WINDOWS_SMOKE)
        self.assertIn("aarch64-pc-windows-msvc", WINDOWS_SMOKE)
        self.assertIn("0xAA64", WINDOWS_SMOKE)
        self.assertIn("0x8664", WINDOWS_SMOKE)
        self.assertIn("MSI-extracted binary does not match the exact qualified build", WINDOWS_SMOKE)
        self.assertIn("installed binary does not match", WINDOWS_SMOKE)

    def test_linux_post_release_smokes_exact_installed_binary_with_persistent_evidence(self) -> None:
        self.assertIn('HONK300_BIN="$BINARY"', UNIX_SMOKE)
        self.assertIn("smoke_m17_m18_linux.sh", UNIX_SMOKE)
        self.assertIn("installed-binary-identity.txt", UNIX_SMOKE)
        self.assertIn("HONK300_RUN_LINUX_OVERLAY_SMOKE", WORKFLOW)
        self.assertIn("Install Linux compositor qualification packages", WORKFLOW)
        self.assertIn("post-release-linux-overlay-${{ matrix.slug }}", WORKFLOW)
        self.assertIn("if-no-files-found: error", WORKFLOW)

    def test_debian_release_smoke_covers_both_native_architectures_and_latest_channel(self) -> None:
        for required in (
            "debian-package:",
            "architecture: amd64",
            "architecture: arm64",
            "smoke_released_deb.sh",
            "post-release-deb-${{ matrix.architecture }}",
        ):
            self.assertIn(required, WORKFLOW)
        for required in (
            "releases/latest/download/honk300-$ARCHITECTURE.deb",
            "cmp \"$PACKAGE\" \"$LATEST_PACKAGE\"",
            "HONK300_DEB_PACKAGE",
            "HONK300_DEB_SKIP_LATEST",
            '[ ! -L "$LOCAL_PACKAGE" ]',
            "local Debian smoke input must explicitly skip",
            "/usr/bin/$name\" update",
            "HONK300_BIN=/usr/lib/honk300/honk300",
            "installed-ldd.txt",
            "runtime library unresolved",
            "/usr/bin/goose uninstall",
            "/usr/bin/honk300 uninstall --purge",
            "dpkg-query --search /usr/lib/honk300/honk300",
        ):
            self.assertIn(required, DEBIAN_SMOKE)


if __name__ == "__main__":
    unittest.main()
