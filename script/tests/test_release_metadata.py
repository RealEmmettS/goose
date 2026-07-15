from __future__ import annotations

import hashlib
import json
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

SCRIPT_DIR = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(SCRIPT_DIR))

import release_metadata


class ReleaseMetadataTests(unittest.TestCase):
    def test_manifest_records_exact_tag_commit_hash_size_target_and_kind(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            payload = root / "honk300-x86_64-pc-windows-msvc.msi"
            payload.write_bytes(b"msi-payload")
            release_metadata.write_sha256_sidecars(root)

            manifest = release_metadata.build_manifest(
                str(root),
                "v0.3.0",
                "0123456789abcdef0123456789abcdef01234567",
            )

            self.assertEqual(manifest["schema"], "honk300.release.v1")
            self.assertEqual(manifest["version"], "0.3.0")
            self.assertEqual(manifest["tag"], "v0.3.0")
            self.assertEqual(
                manifest["commit"], "0123456789abcdef0123456789abcdef01234567"
            )
            self.assertEqual(len(manifest["artifacts"]), 1)
            artifact = manifest["artifacts"][0]
            self.assertEqual(artifact["name"], payload.name)
            self.assertEqual(artifact["target"], "x86_64-pc-windows-msvc")
            self.assertEqual(artifact["kind"], "msi-global")
            self.assertEqual(artifact["size"], len(b"msi-payload"))
            self.assertEqual(artifact["checksum"], f"{payload.name}.sha256")
            self.assertEqual(
                artifact["sha256"], hashlib.sha256(b"msi-payload").hexdigest()
            )
            self.assertEqual(
                manifest["layouts"]["windows"]["install_root"],
                r"%ProgramFiles%\honk300",
            )
            self.assertEqual(
                manifest["layouts"]["macos"]["install_root"],
                "~/Applications/Honk300.app",
            )
            self.assertFalse(manifest["layouts"]["linux"]["autostart_default"])

            # The result is a stable public wire shape, not a Python-only object graph.
            json.dumps(manifest, sort_keys=True)

    def test_debian_artifacts_are_architecture_scoped(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            for name in ("honk300-amd64.deb", "honk300-arm64.deb"):
                (root / name).write_bytes(name.encode())
            release_metadata.write_sha256_sidecars(root)
            manifest = release_metadata.build_manifest(
                str(root), "v1.0.0", "0123456789abcdef0123456789abcdef01234567"
            )
            artifacts = {artifact["name"]: artifact for artifact in manifest["artifacts"]}
            self.assertEqual(artifacts["honk300-amd64.deb"]["kind"], "deb")
            self.assertEqual(
                artifacts["honk300-amd64.deb"]["target"],
                "x86_64-unknown-linux-gnu",
            )
            self.assertEqual(
                artifacts["honk300-arm64.deb"]["target"],
                "aarch64-unknown-linux-gnu",
            )

    def test_manifest_ignores_sidecars_and_internal_build_manifests(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            (root / "honk300-installer.sh").write_text("#!/bin/sh\n", encoding="utf-8")
            release_metadata.write_sha256_sidecars(root)
            (root / "dist-manifest.json").write_text("{}", encoding="utf-8")
            (root / "plan-dist-manifest.json").write_text("{}", encoding="utf-8")

            manifest = release_metadata.build_manifest(
                str(root), "v0.3.0", "0123456789abcdef0123456789abcdef01234567"
            )
            self.assertEqual(
                [item["name"] for item in manifest["artifacts"]],
                ["honk300-installer.sh"],
            )

    def test_manifest_requires_a_matching_regular_checksum_sidecar(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            payload = root / "honk300-installer.sh"
            payload.write_text("#!/bin/sh\n", encoding="utf-8")

            with self.assertRaisesRegex(ValueError, "checksum is missing"):
                release_metadata.build_manifest(
                    str(root), "v0.3.0", "0123456789abcdef0123456789abcdef01234567"
                )

            sidecar = payload.with_name(f"{payload.name}.sha256")
            sidecar.write_text("0" * 64 + f" *{payload.name}\n", encoding="utf-8")
            with self.assertRaisesRegex(ValueError, "checksum does not match"):
                release_metadata.build_manifest(
                    str(root), "v0.3.0", "0123456789abcdef0123456789abcdef01234567"
                )

    def test_manifest_rejects_untrusted_tag_commit_and_unknown_artifact(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            (root / "surprise.bin").write_bytes(b"x")
            with self.assertRaisesRegex(ValueError, "tag"):
                release_metadata.build_manifest(str(root), "../main", "0" * 40)
            with self.assertRaisesRegex(ValueError, "commit"):
                release_metadata.build_manifest(str(root), "v0.3.0", "not-a-commit")
            with self.assertRaisesRegex(ValueError, "unknown release artifact"):
                release_metadata.build_manifest(str(root), "v0.3.0", "0" * 40)

    def test_template_rendering_is_strict_and_rejects_unresolved_tokens(self) -> None:
        rendered = release_metadata.render_template(
            "tag=__TAG__ hash=__HASH__", {"TAG": "v0.3.0", "HASH": "a" * 64}
        )
        self.assertEqual(rendered, f"tag=v0.3.0 hash={'a' * 64}")
        with self.assertRaisesRegex(ValueError, "unresolved"):
            release_metadata.render_template("__TAG__ __MISSING__", {"TAG": "v0.3.0"})

    def test_render_installers_embeds_exact_payload_hashes(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            payloads = {
                "honk300-universal2.app.zip": b"app",
                "honk300-x86_64-unknown-linux-gnu.tar.xz": b"x64-gnu",
                "honk300-aarch64-unknown-linux-gnu.tar.xz": b"arm-gnu",
                "honk300-x86_64-unknown-linux-musl.tar.xz": b"x64-musl",
                "honk300-aarch64-unknown-linux-musl.tar.xz": b"arm-musl",
                "honk300-x86_64-pc-windows-msvc.msi": b"x64-msi",
                "honk300-aarch64-pc-windows-msvc.msi": b"arm-msi",
                "honk300-x86_64-pc-windows-msvc.zip": b"x64-portable",
                "honk300-aarch64-pc-windows-msvc.zip": b"arm-portable",
            }
            for name, body in payloads.items():
                (root / name).write_bytes(body)

            release_metadata.render_installers(
                root,
                "v0.3.0",
                "0123456789abcdef0123456789abcdef01234567",
                SCRIPT_DIR / "honk300-installer.sh.in",
                SCRIPT_DIR / "honk300-installer.ps1.in",
            )

            shell = (root / "honk300-installer.sh").read_text(encoding="utf-8")
            powershell = (root / "honk300-installer.ps1").read_text(encoding="utf-8")
            self.assertNotRegex(shell + powershell, r"__(?:SHA|SIZE)_")
            self.assertIn(hashlib.sha256(b"app").hexdigest(), shell)
            self.assertIn(hashlib.sha256(b"x64-gnu").hexdigest(), shell)
            self.assertIn(hashlib.sha256(b"arm-msi").hexdigest(), powershell)
            self.assertIn(hashlib.sha256(b"x64-portable").hexdigest(), powershell)
            self.assertIn('[int64]"7"', powershell)
            self.assertIn('[int64]"12"', powershell)
            self.assertIn('TAG="v0.3.0"', shell)
            self.assertIn('COMMIT="0123456789abcdef0123456789abcdef01234567"', shell)
            self.assertIn("$Tag = 'v0.3.0'", powershell)

    def test_required_asset_validation_and_sidecars_fail_closed(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            for name in release_metadata.REQUIRED_RELEASE_ARTIFACTS:
                (root / name).write_bytes(name.encode("utf-8"))

            release_metadata.validate_required_artifacts(root)
            sidecars = release_metadata.write_sha256_sidecars(root)
            expected = root / "honk300-installer.sh.sha256"
            self.assertIn(expected, sidecars)
            self.assertEqual(
                expected.read_text(encoding="utf-8"),
                f"{hashlib.sha256(b'honk300-installer.sh').hexdigest()} *honk300-installer.sh\n",
            )

            (root / release_metadata.REQUIRED_RELEASE_ARTIFACTS[0]).unlink()
            with self.assertRaisesRegex(ValueError, "missing required release artifacts"):
                release_metadata.validate_required_artifacts(root)

    def test_cli_writes_stable_manifest(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            (root / "honk300-installer.sh").write_text("#!/bin/sh\n", encoding="utf-8")
            release_metadata.write_sha256_sidecars(root)
            output = root / "release-manifest.json"
            result = subprocess.run(
                [
                    sys.executable,
                    str(SCRIPT_DIR / "release_metadata.py"),
                    "manifest",
                    "--directory",
                    str(root),
                    "--tag",
                    "v0.3.0",
                    "--commit",
                    "0123456789abcdef0123456789abcdef01234567",
                    "--output",
                    str(output),
                ],
                check=False,
                capture_output=True,
                text=True,
            )
            self.assertEqual(result.returncode, 0, result.stderr)
            parsed = json.loads(output.read_text(encoding="utf-8"))
            self.assertEqual(parsed["schema"], "honk300.release.v1")
            self.assertEqual(parsed["tag"], "v0.3.0")


if __name__ == "__main__":
    unittest.main()
