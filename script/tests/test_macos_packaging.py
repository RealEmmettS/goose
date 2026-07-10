from __future__ import annotations

import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
PACKAGE = (ROOT / "script" / "package_macos_app.sh").read_text(encoding="utf-8")
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
        sign = PACKAGE.index('codesign --force --deep --sign - "$APP_DIR"')
        verify = PACKAGE.index('codesign --verify --deep --strict "$APP_DIR"')
        self.assertLess(sign, verify)
        tail = PACKAGE[verify:]
        self.assertNotIn("ditto ", tail)
        self.assertNotIn("cp ", tail)

    def test_release_emits_real_app_zip_and_keeps_dmg_compatibility_only(self) -> None:
        self.assertIn("honk300-universal2.app.zip", WORKFLOW)
        self.assertIn("honk300-universal2.dmg", WORKFLOW)
        self.assertIn("ditto -c -k", WORKFLOW)
        self.assertIn("codesign --verify --deep --strict", WORKFLOW)
        self.assertNotIn("gh release upload", WORKFLOW)


if __name__ == "__main__":
    unittest.main()
