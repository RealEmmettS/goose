from __future__ import annotations

import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
PACKAGE = (ROOT / "script" / "package_macos_app.sh").read_text(encoding="utf-8")
HELPER_PACKAGE_PATH = ROOT / "script" / "package_macos_installer_helper.sh"
HELPER_SOURCE_PATH = ROOT / "packaging" / "macos" / "InstallHonk300" / "main.swift"
HELPER_PACKAGE = HELPER_PACKAGE_PATH.read_text(encoding="utf-8")
WORKFLOW = (ROOT / ".github" / "workflows" / "macos-packaging.yml").read_text(
    encoding="utf-8"
)


class MacosPackagingTests(unittest.TestCase):
    def test_bundle_uses_ditto_and_embeds_legal_notices_before_signing(self) -> None:
        self.assertIn('ditto "$ROOT/LICENSE" "$RESOURCES_DIR/LICENSE"', PACKAGE)
        self.assertIn(
            'ditto "$ROOT/THIRD_PARTY_ASSETS.md" "$RESOURCES_DIR/THIRD_PARTY_ASSETS.md"',
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
        self.assertNotIn('ln -s /Applications', WORKFLOW)
        self.assertNotIn("gh release upload", WORKFLOW)

    def test_dmg_contains_only_the_two_apps_and_readme(self) -> None:
        self.assertIn('test -d "$mount/Honk300.app"', WORKFLOW)
        self.assertIn('test -d "$mount/Install Honk300.app"', WORKFLOW)
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
