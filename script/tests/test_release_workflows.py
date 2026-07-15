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
        for secret in (
            "MACOS_CERTIFICATE_P12_BASE64",
            "MACOS_CERTIFICATE_PASSWORD",
            "MACOS_KEYCHAIN_PASSWORD",
            "APPLE_NOTARY_KEY_P8_BASE64",
            "APPLE_NOTARY_KEY_ID",
            "APPLE_NOTARY_ISSUER_ID",
        ):
            self.assertIn(secret, RELEASE)
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

    def test_every_general_release_advances_latest_only_after_complete_platform_assembly(self) -> None:
        self.assertIn("tags:\n      - 'v[0-9]+.[0-9]+.[0-9]+'", RELEASE)
        self.assertIn("macos-app:\n    needs: plan", RELEASE)
        assembly = RELEASE[RELEASE.index("assemble-and-publish:") :]
        self.assertIn("- macos-app", assembly)
        self.assertIn("- windows-installers", assembly)
        self.assertIn("- build-debian-packages", assembly)
        self.assertIn("- qualify-debian-packages", assembly)
        self.assertIn('gh release edit "$TAG"', assembly)
        self.assertIn("--draft=false --latest", assembly)
        self.assertNotIn("--clobber", assembly)

    def test_candidate_preflight_assembles_without_consuming_a_tag_or_release(self) -> None:
        for workflow in (RELEASE, WINDOWS, MACOS):
            self.assertIn("candidate:", workflow)
        self.assertIn("Candidate artifact set verified without publication", RELEASE)
        self.assertIn("Preserve the complete verified candidate artifact set", RELEASE)
        self.assertIn("candidate-release-assets-${{ needs.plan.outputs.tag }}", RELEASE)
        self.assertIn("path: release-assets/*", RELEASE)
        candidate_upload = RELEASE.index("Preserve the complete verified candidate artifact set")
        local_verify = RELEASE.index("Verify every local checksum and manifest hash")
        installer_smoke = RELEASE.index("Smoke shell installer twice without sudo")
        self.assertGreater(candidate_upload, local_verify)
        self.assertGreater(candidate_upload, installer_smoke)
        self.assertIn("if: ${{ !inputs.candidate }}", RELEASE)
        self.assertIn("candidate: ${{ inputs.candidate || false }}", RELEASE)
        self.assertIn('$tagExists = [bool](git tag --list "${{ inputs.tag }}")', WINDOWS)
        self.assertNotIn("git show-ref --verify --quiet", WINDOWS)

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
        self.assertIn("Install Linux audio and compositor qualification packages", RELEASE)
        self.assertIn("grim imagemagick libasound2-dev", RELEASE)
        self.assertIn("Join-Path $env:RUNNER_TEMP 'dist.exe'", RELEASE)
        self.assertNotIn(
            "cargo-dist-x86_64-pc-windows-msvc\\dist.exe",
            RELEASE,
        )

    def test_linux_release_payloads_are_exact_native_compositor_qualified(self) -> None:
        for target in (
            "x86_64-unknown-linux-gnu",
            "x86_64-unknown-linux-musl",
            "aarch64-unknown-linux-gnu",
            "aarch64-unknown-linux-musl",
        ):
            self.assertIn(target, RELEASE)
        for required in (
            'archive="target/distrib/honk300-$target.tar.xz"',
            'binary="target/exact-linux-$target/$root/honk300"',
            "--format elf --machine \"$machine\"",
            "x86_64-unknown-linux-gnu|x86_64-unknown-linux-musl) machine=62",
            "aarch64-unknown-linux-gnu|aarch64-unknown-linux-musl) machine=183",
            'HONK300_BIN="$PWD/$binary"',
            'HONK300_EVIDENCE_DIR="$PWD/$evidence/compositor"',
            'archive_sha256_before=$archive_before',
            'archive_sha256_after=$archive_after',
            'binary_sha256_before=$binary_before',
            'binary_sha256_after=$binary_after',
            "if-no-files-found: error",
        ):
            self.assertIn(required, RELEASE)
        self.assertLess(
            RELEASE.index("Qualify exact Linux archive payload before upload"),
            RELEASE.index("Upload portable artifacts to the orchestrator"),
        )

    def test_stable_debian_packages_reuse_exact_qualified_gnu_binaries(self) -> None:
        for required in (
            "build-debian-packages:",
            "Build stable packages from the exact qualified binaries",
            "honk300-amd64.deb",
            "honk300-arm64.deb",
            "python3 script/package_deb.py",
            'cmp "$binary" "$extracted/usr/lib/honk300/honk300"',
            "dpkg-deb --field",
            "verify_binary_architecture.py",
            "release-debian-packages",
            "qualification-debian-package-build",
        ):
            self.assertIn(required, RELEASE)
        self.assertLess(
            RELEASE.index("Build stable packages from the exact qualified binaries"),
            RELEASE.index("Validate exact compatibility and primary artifact set"),
        )

    def test_candidate_natively_qualifies_both_debian_architectures_before_assembly(self) -> None:
        for required in (
            "qualify-debian-packages:",
            "runner: ubuntu-22.04",
            "runner: ubuntu-22.04-arm",
            "HONK300_DEB_PACKAGE:",
            "HONK300_DEB_SKIP_LATEST: '1'",
            "smoke_released_deb.sh",
            "qualification-debian-package-${{ matrix.architecture }}",
        ):
            self.assertIn(required, RELEASE)
        assembly = RELEASE[RELEASE.index("assemble-and-publish:") :]
        self.assertIn("- qualify-debian-packages", assembly)

    def test_windows_cargo_dist_zips_are_native_qualified_before_assembly(self) -> None:
        portable = RELEASE[RELEASE.index("qualify-windows-portable:") :]
        for required in (
            "runs-on: ${{ matrix.runner }}",
            "windows-2022",
            "windows-11-arm",
            "0x8664",
            "0xAA64",
            "release-portable-${{ matrix.triple }}",
            "honk300-${{ matrix.triple }}.zip",
            "cargo-dist ZIP must contain root honk300.exe",
            "verify_binary_architecture.py --format pe",
            "zip_sha256_before",
            "zip_sha256_after",
            "binary_sha256_before",
            "binary_sha256_after",
            "smoke_windows_overlay.ps1",
            "qualification-windows-portable-${{ matrix.triple }}",
        ):
            self.assertIn(required, portable)
        assemble = RELEASE[RELEASE.index("assemble-and-publish:") :]
        self.assertIn("- qualify-windows-portable", assemble)

    def test_windows_msi_payload_identity_and_native_arm_qualification_are_required(self) -> None:
        for required in (
            "Prove Global MSI contains the exact PE build",
            "MSI payload does not match exact qualified build",
            "qualification-windows-msi-identity-${{ matrix.triple }}",
            "qualification-input-windows-aarch64-pc-windows-msvc",
            "runs-on: windows-11-arm",
            "-TargetTriple aarch64-pc-windows-msvc",
            "-CurrentMsiPath target/qualification-input-aarch64-pc-windows-msvc/",
            "-SourceBinaryPath target/qualification-input-aarch64-pc-windows-msvc/honk300.exe",
        ):
            self.assertIn(required, WINDOWS)


if __name__ == "__main__":
    unittest.main()
