from __future__ import annotations

import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
WORKFLOWS = ROOT / ".github" / "workflows"
RELEASE = (WORKFLOWS / "release.yml").read_text(encoding="utf-8")
WINDOWS = (WORKFLOWS / "windows-installers.yml").read_text(encoding="utf-8")
MACOS = (WORKFLOWS / "macos-packaging.yml").read_text(encoding="utf-8")
CI = (WORKFLOWS / "ci.yml").read_text(encoding="utf-8")
CARGO = (ROOT / "Cargo.toml").read_text(encoding="utf-8")


class ReleaseWorkflowTests(unittest.TestCase):
    def test_platform_packagers_are_reusable_producers_only(self) -> None:
        for workflow in (WINDOWS, MACOS):
            self.assertIn("workflow_call:", workflow)
            self.assertIn("actions/upload-artifact@", workflow)
            self.assertNotIn("workflow_run:", workflow)
            self.assertNotIn("gh release upload", workflow)
            self.assertNotIn("--clobber", workflow)
            self.assertIn("toolchain: 1.95.0", workflow)

    def test_release_orchestrator_builds_everything_before_one_draft_upload(self) -> None:
        self.assertIn("uses: ./.github/workflows/windows-installers.yml", RELEASE)
        self.assertIn("uses: ./.github/workflows/macos-packaging.yml", RELEASE)
        self.assertIn("release_metadata.py render-installers", RELEASE)
        self.assertIn('--commit "$COMMIT"', RELEASE)
        self.assertIn("release_metadata.py validate", RELEASE)
        self.assertIn("release-manifest.json", RELEASE)
        self.assertEqual(RELEASE.count("release_metadata.py sidecars"), 2)
        self.assertIn("HONK300_TEST_FAIL_AFTER_SWAP=1", RELEASE)
        self.assertIn("gh release create", RELEASE)
        self.assertIn("--draft", RELEASE)
        self.assertEqual(RELEASE.count("gh release upload"), 1)
        self.assertNotIn("--clobber", RELEASE)
        self.assertIn("Verify remote asset set", RELEASE)
        self.assertIn("--draft=false", RELEASE)
        self.assertIn("Delete this run's unpublished draft after failure", RELEASE)
        self.assertIn('if: ${{ failure() && steps.create_draft.outcome == \'success\' }}', RELEASE)

    def test_candidate_preflight_assembles_without_consuming_a_tag_or_release(self) -> None:
        for workflow in (RELEASE, WINDOWS, MACOS):
            self.assertIn("candidate:", workflow)
        self.assertIn("Candidate artifact set verified without publication", RELEASE)
        self.assertIn("if: ${{ !inputs.candidate }}", RELEASE)
        self.assertIn("candidate: ${{ inputs.candidate || false }}", RELEASE)

    def test_ci_requires_audit_and_project_owned_installer_tests(self) -> None:
        self.assertIn("cargo audit", CI)
        self.assertIn("test_release_metadata.py", CI)
        self.assertIn("test_installer_templates.py", CI)
        self.assertIn("test_release_workflows.py", CI)
        self.assertIn('installers = []', CARGO)

    def test_release_tooling_is_immutable_and_checksum_pinned(self) -> None:
        pinned_toolchain = "dtolnay/rust-toolchain@4be7066ada62dd38de10e7b70166bc74ed198c30"
        for workflow in (WINDOWS, MACOS):
            self.assertIn(pinned_toolchain, workflow)
            self.assertNotIn("dtolnay/rust-toolchain@stable", workflow)

        self.assertIn("messense/cargo-xwin@sha256:", RELEASE)
        self.assertIn("cargo-xwin==0.22.0", RELEASE)
        self.assertIn("cd355dab0b4c02fb59038fef87655550021d07f45f1d82f947a34ef98560abb8", RELEASE)
        self.assertIn("a14e17557b269b101405e0cc6b647581d56313c954a51c7fddd423bba21e17b2", RELEASE)
        self.assertNotIn("cargo-dist-installer.sh", RELEASE)
        self.assertNotIn("cargo-dist-installer.ps1", RELEASE)
        self.assertIn("contents: read", RELEASE)
        self.assertIn("contents: write", RELEASE)

    def test_release_portable_jobs_cover_native_host_prerequisites(self) -> None:
        self.assertIn("command -v sha256sum", RELEASE)
        self.assertIn("shasum -a 256 -c -", RELEASE)
        self.assertIn("sudo apt-get install --no-install-recommends -y libasound2-dev", RELEASE)
        self.assertIn("Join-Path $env:RUNNER_TEMP 'dist.exe'", RELEASE)
        self.assertNotIn(
            "cargo-dist-x86_64-pc-windows-msvc\\dist.exe",
            RELEASE,
        )


if __name__ == "__main__":
    unittest.main()
