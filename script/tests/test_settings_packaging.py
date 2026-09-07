"""Archive integrity tests; native execution is qualified by settings.yml."""
from __future__ import annotations

import hashlib
from io import BytesIO
import json
from pathlib import Path
import stat
import struct
import sys
import tarfile
import tempfile
import tomllib
import unittest
import zipfile

ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT / "script"))
from bundle_settings import add_files, bundle
from verify_settings import verify


class SettingsPackagingTests(unittest.TestCase):
    def test_bundle_preserves_runtime_bytes_and_refreshes_cargo_dist_identity(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            settings = root / "companion"
            settings.mkdir()
            # This bounded PE header is an architecture-parser fixture. It is never
            # executed or used as evidence that the native GUI runs.
            pe = bytearray(256)
            pe[:2] = b"MZ"
            struct.pack_into("<I", pe, 0x3C, 0x80)
            pe[0x80:0x84] = b"PE\0\0"
            struct.pack_into("<H", pe, 0x84, 0x8664)
            struct.pack_into("<H", pe, 0x98, 0x20B)
            struct.pack_into("<H", pe, 0xDC, 2)
            (settings / "honk300-settings.exe").write_bytes(pe)
            (settings / "honk_settings_accessibility.dll").write_bytes(pe)
            for name in ("NATIVE_SDK_LICENSE.txt", "NATIVE_SDK_FONT_LICENSE.txt"):
                (settings / name).write_text("License fixture\n")
            version = tomllib.loads((ROOT / "Cargo.toml").read_text())["package"]["version"]
            target = "x86_64-pc-windows-msvc"
            metadata = {"schema": "honk300.settings-build.v1", "version": version,
                        "target": target, "native_sdk": "0.5.4", "zig": "0.16.0",
                        "automation": False, "name": "honk300-settings.exe",
                        "size": len(pe), "sha256": hashlib.sha256(pe).hexdigest()}
            metadata["accessibility"] = {"name": "honk_settings_accessibility.dll", "size": len(pe), "sha256": hashlib.sha256(pe).hexdigest()}
            metadata_path = settings / "settings-build.json"
            metadata_path.write_text(json.dumps(metadata))
            archive = root / "honk300.zip"
            with zipfile.ZipFile(archive, "w") as z:
                z.writestr("honk300.exe", b"unchanged-runtime-fixture")
                z.writestr("honk300-app.exe", b"unchanged-launcher-fixture")
            manifest = root / "dist-manifest.json"
            manifest.write_text(json.dumps({"dist_version": "0.31.0", "artifacts": {
                archive.name: {"kind": "executable-zip", "target_triples": [target],
                               "assets": [], "checksum": "honk300.zip.sha256"},
                "honk300.zip.sha256": {"kind": "checksum"}}}))
            bundle(archive, settings, target, manifest)
            with zipfile.ZipFile(archive) as z:
                self.assertEqual(z.read("honk300.exe"), b"unchanged-runtime-fixture")
                self.assertEqual(z.read("honk300-app.exe"), b"unchanged-launcher-fixture")
                self.assertEqual(z.read("honk300-settings.exe"), pe)
                self.assertEqual(z.read("honk_settings_accessibility.dll"), pe)
                self.assertEqual(z.read("settings-build.json"), metadata_path.read_bytes())
            identity = json.loads(manifest.read_text())["artifacts"][archive.name]
            checksum = hashlib.sha256(archive.read_bytes()).hexdigest()
            self.assertEqual(identity["checksums"]["sha256"], checksum)
            self.assertEqual((root / "honk300.zip.sha256").read_text(), checksum + " *honk300.zip\n")
            self.assertIn("honk300-settings.exe", [asset["path"] for asset in identity["assets"]])
            before = archive.read_bytes()
            with self.assertRaisesRegex(AssertionError, "already contains"):
                bundle(archive, settings, target, manifest)
            self.assertEqual(archive.read_bytes(), before)
            (settings / "honk_settings_accessibility.dll").write_bytes(pe + b"tampered")
            with self.assertRaises(AssertionError):
                verify(settings, target)
            (settings / "honk_settings_accessibility.dll").write_bytes(pe)
            metadata["automation"] = True
            metadata_path.write_text(json.dumps(metadata))
            with self.assertRaises(AssertionError):
                verify(settings, target)

    def test_tar_preserves_paths_modes_and_payloads(self):
        with tempfile.TemporaryDirectory() as temporary:
            archive = Path(temporary) / "honk300.tar.xz"
            with tarfile.open(archive, "w:xz") as t:
                info = tarfile.TarInfo("honk300/bin/honk300")
                info.mode, info.size = 0o755, 7
                t.addfile(info, BytesIO(b"runtime"))
            add_files(archive, {"honk300/bin/honk300-settings": b"settings", "honk300/bin/license.txt": b"notice"})
            with tarfile.open(archive) as t:
                self.assertEqual(t.extractfile("honk300/bin/honk300").read(), b"runtime")
                self.assertEqual(t.getmember("honk300/bin/honk300").mode, 0o755)
                self.assertEqual(t.getmember("honk300/bin/honk300-settings").mode, 0o755)
                self.assertEqual(t.getmember("honk300/bin/license.txt").mode, 0o644)

    def test_unsafe_or_linked_archive_is_rejected_without_replacing_it(self):
        with tempfile.TemporaryDirectory() as temporary:
            archive = Path(temporary) / "honk300.zip"
            for name, mode in [("../escape", stat.S_IFREG), ("/absolute", stat.S_IFREG),
                               ("C:/absolute", stat.S_IFREG), ("dir\\escape", stat.S_IFREG),
                               ("linked", stat.S_IFLNK)]:
                with self.subTest(name=name):
                    with zipfile.ZipFile(archive, "w") as z:
                        info = zipfile.ZipInfo(name)
                        # ZipInfo's constructor normalizes a backslash on Windows;
                        # write the raw member spelling to exercise archive input.
                        info.filename = name
                        info.external_attr = (mode | 0o644) << 16
                        z.writestr(info, b"data")
                    before = archive.read_bytes()
                    with self.assertRaises(AssertionError):
                        add_files(archive, {"honk300-settings": b"settings"})
                    self.assertEqual(archive.read_bytes(), before)


if __name__ == "__main__":
    unittest.main()
