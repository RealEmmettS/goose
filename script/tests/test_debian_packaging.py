from __future__ import annotations

import importlib.util
import json
import stat
import tempfile
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
SCRIPT = ROOT / "script" / "package_deb.py"
SPEC = importlib.util.spec_from_file_location("package_deb", SCRIPT)
assert SPEC and SPEC.loader
PACKAGE_DEB = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(PACKAGE_DEB)


class DebianPackagingTests(unittest.TestCase):
    def test_package_tree_has_stable_aliases_identity_and_desktop_entry(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            temp = Path(directory)
            binary = temp / "input-honk300"
            binary.write_bytes(b"qualified-elf")
            binary.chmod(0o755)
            staging = temp / "staging"
            installed = PACKAGE_DEB.build_package_tree(
                staging,
                binary,
                "1.0.0",
                "v1.0.0",
                "0123456789abcdef0123456789abcdef01234567",
                "amd64",
            )

            self.assertEqual(installed.read_bytes(), binary.read_bytes())
            self.assertTrue(installed.stat().st_mode & stat.S_IXUSR)
            self.assertEqual((installed.parent / "install-source.txt").read_text(), "deb\n")
            for name in ("honk300", "honk", "goose"):
                alias = staging / "usr" / "bin" / name
                self.assertTrue(alias.is_symlink())
                self.assertEqual(alias.readlink(), Path("../lib/honk300/honk300"))
            metadata = json.loads(
                (staging / "usr" / "share" / "honk300" / "release.json").read_text()
            )
            self.assertEqual(metadata["target"], "x86_64-unknown-linux-gnu")
            self.assertEqual(metadata["tag"], "v1.0.0")
            control = (staging / "DEBIAN" / "control").read_text()
            self.assertIn("Package: honk300\n", control)
            self.assertIn("Architecture: amd64\n", control)
            self.assertIn("libasound2 | libasound2t64", control)
            desktop = (staging / "usr" / "share" / "applications" / "honk300.desktop").read_text()
            self.assertIn("Exec=/usr/bin/honk300 start", desktop)

    def test_rejects_mismatched_release_identity_and_unsafe_input(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            temp = Path(directory)
            binary = temp / "binary"
            binary.write_bytes(b"elf")
            with self.assertRaisesRegex(ValueError, "version and tag"):
                PACKAGE_DEB.build_package_tree(
                    temp / "one", binary, "1.0.0", "v1.0.1", "0" * 40, "amd64"
                )
            link = temp / "link"
            link.symlink_to(binary)
            with self.assertRaisesRegex(ValueError, "regular non-symlink"):
                PACKAGE_DEB.build_package_tree(
                    temp / "two", link, "1.0.0", "v1.0.0", "0" * 40, "arm64"
                )


if __name__ == "__main__":
    unittest.main()
