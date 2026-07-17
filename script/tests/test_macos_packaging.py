from __future__ import annotations

import struct
import unittest
import zlib
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
PACKAGE = (ROOT / "script" / "package_macos_app.sh").read_text(encoding="utf-8")
HELPER_PACKAGE_PATH = ROOT / "script" / "package_macos_installer_helper.sh"
HELPER_SOURCE_PATH = ROOT / "packaging" / "macos" / "InstallHonk300" / "main.swift"
CONFIGURE_LAUNCHER_PATH = ROOT / "packaging" / "macos" / "Configure Honk300.command"
STATUS_ICON_PATH = ROOT / "Assets" / "UI" / "honk300-status-goose.svg"
STATUS_ICON_RUNTIME_PATH = ROOT / "Assets" / "UI" / "honk300-status-goose@2x.png"
HELPER_PACKAGE = HELPER_PACKAGE_PATH.read_text(encoding="utf-8")
WORKFLOW = (ROOT / ".github" / "workflows" / "macos-packaging.yml").read_text(
    encoding="utf-8"
)


def decode_rgba_png(path: Path) -> tuple[int, int, bytes]:
    data = path.read_bytes()
    if data[:8] != b"\x89PNG\r\n\x1a\n":
        raise AssertionError("status icon is not a PNG")

    chunks: dict[bytes, list[bytes]] = {}
    offset = 8
    while offset < len(data):
        length = struct.unpack(">I", data[offset : offset + 4])[0]
        kind = data[offset + 4 : offset + 8]
        payload = data[offset + 8 : offset + 8 + length]
        chunks.setdefault(kind, []).append(payload)
        offset += 12 + length

    width, height, depth, color_type, compression, filtering, interlace = struct.unpack(
        ">IIBBBBB", chunks[b"IHDR"][0]
    )
    if (depth, color_type, compression, filtering, interlace) != (8, 6, 0, 0, 0):
        raise AssertionError("status icon must be a non-interlaced 8-bit RGBA PNG")

    raw = zlib.decompress(b"".join(chunks[b"IDAT"]))
    stride = width * 4
    previous = bytearray(stride)
    pixels = bytearray()
    position = 0
    for _ in range(height):
        filter_kind = raw[position]
        position += 1
        scanline = bytearray(raw[position : position + stride])
        position += stride
        for index, value in enumerate(scanline):
            left = scanline[index - 4] if index >= 4 else 0
            above = previous[index]
            upper_left = previous[index - 4] if index >= 4 else 0
            if filter_kind == 1:
                predictor = left
            elif filter_kind == 2:
                predictor = above
            elif filter_kind == 3:
                predictor = (left + above) // 2
            elif filter_kind == 4:
                estimate = left + above - upper_left
                distances = (
                    abs(estimate - left),
                    abs(estimate - above),
                    abs(estimate - upper_left),
                )
                predictor = (left, above, upper_left)[distances.index(min(distances))]
            elif filter_kind == 0:
                predictor = 0
            else:
                raise AssertionError(f"unsupported PNG filter {filter_kind}")
            scanline[index] = (value + predictor) & 0xFF
        pixels.extend(scanline)
        previous = scanline

    return width, height, bytes(pixels)


class MacosPackagingTests(unittest.TestCase):
    def test_bundle_uses_ditto_and_embeds_legal_notices_before_signing(self) -> None:
        self.assertIn('ditto "$ROOT/LICENSE" "$RESOURCES_DIR/LICENSE"', PACKAGE)
        self.assertIn(
            'ditto "$ROOT/THIRD_PARTY_ASSETS.md" "$RESOURCES_DIR/THIRD_PARTY_ASSETS.md"',
            PACKAGE,
        )
        self.assertIn(
            'ditto "$ROOT/packaging/macos/Configure Honk300.command"',
            PACKAGE,
        )
        self.assertIn('chmod 755 "$RESOURCES_DIR/Configure Honk300.command"', PACKAGE)
        self.assertIn(
            'ditto "$ROOT/Assets/UI/honk300-status-goose.svg"',
            PACKAGE,
        )
        self.assertIn(
            'ditto "$ROOT/Assets/UI/honk300-status-goose@2x.png"',
            PACKAGE,
        )
        self.assertNotIn('"$ROOT/Assets"', PACKAGE)
        self.assertIn("MACOS_SIGN_IDENTITY", PACKAGE)
        self.assertIn('codesign --force --options runtime --sign - "$BIN"', PACKAGE)
        self.assertIn('codesign --force --options runtime --timestamp --sign "$IDENTITY" "$BIN"', PACKAGE)
        self.assertNotIn("--deep", PACKAGE)
        sign = PACKAGE.index('codesign --force --options runtime --sign - "$BIN"')
        verify = PACKAGE.index('codesign --verify --strict "$APP_DIR"')
        self.assertLess(sign, verify)
        tail = PACKAGE[verify:]
        self.assertNotIn("ditto ", tail)
        self.assertNotIn("cp ", tail)

    def test_menu_bar_configure_launcher_runs_the_bundled_tui(self) -> None:
        self.assertTrue(CONFIGURE_LAUNCHER_PATH.is_file())
        launcher = CONFIGURE_LAUNCHER_PATH.read_text(encoding="utf-8")
        self.assertIn('#!/bin/sh', launcher)
        self.assertIn('CONTENTS_DIR=', launcher)
        self.assertIn('MacOS/honk300" config', launcher)
        self.assertNotIn("sudo", launcher)
        self.assertLess(
            PACKAGE.index('chmod 755 "$RESOURCES_DIR/Configure Honk300.command"'),
            PACKAGE.index('codesign --force --options runtime --sign - "$BIN"'),
        )

    def test_status_icon_is_shared_monochrome_source_sealed_before_signing(self) -> None:
        self.assertTrue(STATUS_ICON_PATH.is_file())
        self.assertTrue(STATUS_ICON_RUNTIME_PATH.is_file())
        self.assertEqual(STATUS_ICON_RUNTIME_PATH.read_bytes()[:8], b"\x89PNG\r\n\x1a\n")
        width, height, pixels = decode_rgba_png(STATUS_ICON_RUNTIME_PATH)
        self.assertEqual((width, height), (36, 36))
        alpha = pixels[3::4]
        self.assertEqual(min(alpha), 0)
        self.assertEqual(max(alpha), 255)
        self.assertLess(sum(value > 0 for value in alpha), len(alpha) // 2)
        self.assertTrue(
            all(
                pixels[index : index + 3] == b"\0\0\0"
                for index in range(0, len(pixels), 4)
            )
        )
        icon = STATUS_ICON_PATH.read_text(encoding="utf-8")
        self.assertIn('<svg xmlns="http://www.w3.org/2000/svg"', icon)
        self.assertIn("Honk300 status and tray goose", icon)
        self.assertEqual(icon.count("<path "), 2)
        self.assertNotIn("gradient", icon.lower())
        self.assertNotIn("<rect", icon.lower())
        icon_copy = PACKAGE.index(
            'ditto "$ROOT/Assets/UI/honk300-status-goose.svg"'
        )
        binary_sign = PACKAGE.index(
            'codesign --force --options runtime --sign - "$BIN"'
        )
        self.assertLess(icon_copy, binary_sign)
        runtime_copy = PACKAGE.index(
            'ditto "$ROOT/Assets/UI/honk300-status-goose@2x.png"'
        )
        self.assertLess(runtime_copy, binary_sign)

    def test_release_emits_signed_notarized_stapled_app_and_primary_dmg(self) -> None:
        self.assertIn("honk300-universal2.app.zip", WORKFLOW)
        self.assertIn("honk300-universal2.dmg", WORKFLOW)
        self.assertIn("Install Honk300.app", WORKFLOW)
        self.assertIn("ditto -c -k", WORKFLOW)
        self.assertIn("codesign --verify --strict", WORKFLOW)
        self.assertIn("notarytool submit", WORKFLOW)
        self.assertIn("stapler staple", WORKFLOW)
        self.assertIn("stapler validate", WORKFLOW)
        self.assertIn("spctl --assess", WORKFLOW)
        self.assertIn("verify_developer_id", WORKFLOW)
        self.assertIn("Authority=Developer ID Application: ES Development LLC (M9D5379H93)", WORKFLOW)
        self.assertIn("TeamIdentifier=M9D5379H93", WORKFLOW)
        self.assertIn("flags=0x10000(runtime)", WORKFLOW)
        self.assertIn("^Timestamp=", WORKFLOW)
        self.assertIn("codesign -d -r-", WORKFLOW)
        self.assertIn("anchor apple generic", WORKFLOW)
        self.assertIn("certificate leaf[subject.OU] = M9D5379H93", WORKFLOW)
        self.assertIn("739B04530883FF9B665C66BD464F98C622971B32", WORKFLOW)
        self.assertIn("CN=Developer ID Certification Authority", WORKFLOW)
        self.assertIn("OU=G2", WORKFLOW)
        self.assertIn('security list-keychains -d user -s "$keychain"', WORKFLOW)
        self.assertNotIn('"$keychain" "$login_keychain"', WORKFLOW)
        self.assertIn("MACOS_SIGN_KEYCHAIN", WORKFLOW)
        self.assertIn('codesign --keychain "$MACOS_SIGN_KEYCHAIN"', WORKFLOW)
        self.assertIn("codesign -d --extract-certificates=", WORKFLOW)
        self.assertIn("honk300-sealed-certificate-", WORKFLOW)
        self.assertIn("honk300-final-zip-certificate-", WORKFLOW)
        self.assertIn("honk300-dmg-certificate-", WORKFLOW)
        self.assertIn('openssl x509 -inform DER -in "${certificate_prefix}0"', WORKFLOW)
        self.assertNotIn('ln -s /Applications', WORKFLOW)
        self.assertNotIn("gh release upload", WORKFLOW)

    def test_dmg_contains_only_the_two_apps_and_readme(self) -> None:
        self.assertIn('test -d "$mount/Honk300.app"', WORKFLOW)
        self.assertIn('test -d "$mount/Install Honk300.app"', WORKFLOW)
        self.assertIn(
            'test -x "$mount/Honk300.app/Contents/Resources/Configure Honk300.command"',
            WORKFLOW,
        )
        self.assertIn(
            'test -f "$mount/Honk300.app/Contents/Resources/honk300-status-goose.svg"',
            WORKFLOW,
        )
        self.assertIn(
            'test -f "$mount/Honk300.app/Contents/Resources/honk300-status-goose@2x.png"',
            WORKFLOW,
        )
        self.assertEqual(
            WORKFLOW.count(
                'test -f "$mount/Honk300.app/Contents/Resources/honk300-status-goose.svg"'
            ),
            2,
        )
        self.assertEqual(
            WORKFLOW.count(
                'test -f "$mount/Honk300.app/Contents/Resources/honk300-status-goose@2x.png"'
            ),
            2,
        )
        self.assertIn(
            'status_icon="$app/Contents/Resources/honk300-status-goose.svg"',
            WORKFLOW,
        )
        self.assertIn('test -f "$status_icon"', WORKFLOW)
        self.assertIn(
            'status_icon_runtime="$app/Contents/Resources/honk300-status-goose@2x.png"',
            WORKFLOW,
        )
        self.assertIn('test -f "$status_icon_runtime"', WORKFLOW)
        self.assertIn(
            'test -f "$extracted/Honk300.app/Contents/Resources/honk300-status-goose.svg"',
            WORKFLOW,
        )
        self.assertIn(
            'test -f "$extracted/Honk300.app/Contents/Resources/honk300-status-goose@2x.png"',
            WORKFLOW,
        )
        self.assertIn('test -f "$mount/Read Me.txt"', WORKFLOW)
        self.assertIn('test ! -e "$mount/Applications"', WORKFLOW)
        self.assertIn("-eq 3", WORKFLOW)
        self.assertIn('spctl --assess --type execute --verbose=4 "$mount/Honk300.app"', WORKFLOW)
        self.assertIn(
            'spctl --assess --type execute --verbose=4 "$mount/Install Honk300.app"',
            WORKFLOW,
        )

    def test_release_credentials_fail_closed_and_use_ephemeral_keychain(self) -> None:
        for name in (
            "MACOS_CERTIFICATE_P12_BASE64",
            "MACOS_CERTIFICATE_PASSWORD",
            "MACOS_KEYCHAIN_PASSWORD",
            "APPLE_NOTARY_KEY_P8_BASE64",
            "APPLE_NOTARY_KEY_ID",
            "APPLE_NOTARY_ISSUER_ID",
        ):
            self.assertIn(name, WORKFLOW)
        self.assertIn("security create-keychain", WORKFLOW)
        self.assertIn("security delete-keychain", WORKFLOW)
        self.assertIn('${{ runner.temp }}/honk300-developer-id.p12', WORKFLOW)
        self.assertIn('${{ runner.temp }}/AuthKey.p8', WORKFLOW)
        self.assertNotIn("ad-hoc fallback", WORKFLOW)
        self.assertNotIn("--entitlements", PACKAGE)
        self.assertIn("MACOS_SIGN_KEYCHAIN", PACKAGE)
        self.assertIn("MACOS_SIGN_KEYCHAIN", HELPER_PACKAGE)
        self.assertIn('codesign --keychain "$KEYCHAIN"', PACKAGE)
        self.assertIn('codesign --keychain "$KEYCHAIN"', HELPER_PACKAGE)

    def test_final_zip_and_notarization_evidence_fail_closed(self) -> None:
        self.assertIn('ditto -x -k "$output/honk300-universal2.app.zip" "$extracted"', WORKFLOW)
        self.assertIn('codesign --verify --strict --verbose=4 "$extracted/Honk300.app"', WORKFLOW)
        self.assertIn('xcrun stapler validate "$extracted/Honk300.app"', WORKFLOW)
        self.assertIn('spctl --assess --type execute --verbose=4 "$extracted/Honk300.app"', WORKFLOW)
        self.assertIn("Require complete notarization evidence", WORKFLOW)
        self.assertIn("target/notarization-evidence/app-log.json", WORKFLOW)
        self.assertIn("target/notarization-evidence/dmg-log.json", WORKFLOW)
        evidence_tail = WORKFLOW[WORKFLOW.index("Preserve notarization evidence for this workflow run") :]
        self.assertIn("if-no-files-found: error", evidence_tail)
        self.assertNotIn("if-no-files-found: warn", evidence_tail)

    def test_release_explicitly_installs_both_apple_targets(self) -> None:
        self.assertIn(
            "rustup target add x86_64-apple-darwin aarch64-apple-darwin",
            WORKFLOW,
        )

    def test_bundle_stamps_updater_compatible_release_identity(self) -> None:
        self.assertIn("HONK300_TAG", PACKAGE)
        self.assertIn("HONK300_COMMIT", PACKAGE)
        self.assertIn("<key>Honk300ReleaseTag</key>", PACKAGE)
        self.assertIn("<key>Honk300ReleaseCommit</key>", PACKAGE)

    def test_graphical_helper_verifies_target_identity_and_invokes_shared_install(self) -> None:
        self.assertTrue(HELPER_PACKAGE_PATH.is_file())
        self.assertTrue(HELPER_SOURCE_PATH.is_file())
        helper = HELPER_SOURCE_PATH.read_text(encoding="utf-8")
        self.assertIn("dev.emmetts.honk300", helper)
        self.assertIn("SecStaticCodeCheckValidity", helper)
        self.assertIn("kSecCodeInfoTeamIdentifier", helper)
        self.assertIn("M9D5379H93", helper)
        self.assertIn("kSecCodeInfoCertificates", helper)
        self.assertIn("SecCertificateCopyCommonName", helper)
        self.assertIn("Developer ID Application:", helper)
        self.assertIn('["install"]', helper)
        self.assertNotIn("sudo", helper)

    def test_graphical_helper_signs_its_executable_and_bundle_inside_out(self) -> None:
        self.assertNotIn("--deep", HELPER_PACKAGE)
        self.assertIn("x86_64-apple-macos11.0", HELPER_PACKAGE)
        self.assertIn("arm64-apple-macos11.0", HELPER_PACKAGE)
        self.assertIn("lipo -create", HELPER_PACKAGE)
        self.assertIn('lipo "$EXECUTABLE" -verify_arch x86_64 arm64', HELPER_PACKAGE)
        binary_sign = HELPER_PACKAGE.index(
            'codesign --force --options runtime --timestamp --sign "$IDENTITY" "$EXECUTABLE"'
        )
        bundle_sign = HELPER_PACKAGE.index(
            'codesign --force --options runtime --timestamp --sign "$IDENTITY" "$APP_DIR"'
        )
        self.assertLess(binary_sign, bundle_sign)
        self.assertIn(
            'codesign --verify --strict --verbose=2 "$EXECUTABLE"',
            HELPER_PACKAGE,
        )

    def test_release_verifies_helper_architectures_and_macos_11_deployment(self) -> None:
        self.assertIn('lipo "$helper_bin" -verify_arch x86_64 arm64', WORKFLOW)
        self.assertIn("vtool -show-build", WORKFLOW)
        self.assertIn("helper-deployment.txt", WORKFLOW)
        self.assertIn("minos 11.0", WORKFLOW)


if __name__ == "__main__":
    unittest.main()
