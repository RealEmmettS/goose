from __future__ import annotations

import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
GLOBAL_WIX = (ROOT / "wix" / "main.wxs").read_text(encoding="utf-8")
CORPORATE_WIX = (ROOT / "wix-corporate" / "corporate.wxs").read_text(encoding="utf-8")
GLOBAL_INNO = (ROOT / "inno" / "global.iss").read_text(encoding="utf-8")
CORPORATE_INNO = (ROOT / "inno" / "corporate.iss").read_text(encoding="utf-8")
INSTALL_RS = (ROOT / "src" / "install.rs").read_text(encoding="utf-8")
WINDOWS_WORKFLOW = (
    ROOT / ".github" / "workflows" / "windows-installers.yml"
).read_text(encoding="utf-8")


class WindowsPackagingTests(unittest.TestCase):
    def test_global_msi_is_machine_owned_and_all_users(self) -> None:
        self.assertIn("InstallScope='perMachine'", GLOBAL_WIX)
        self.assertIn("Id='CommonProgramsFolder'", GLOBAL_WIX)
        self.assertIn("Root='HKLM'", GLOBAL_WIX)
        self.assertIn("System='yes'", GLOBAL_WIX)
        self.assertNotIn("Root='HKCU'", GLOBAL_WIX)
        self.assertIn("Level='2'", GLOBAL_WIX)  # Autostart remains opt-in.

    def test_corporate_msi_remains_per_user(self) -> None:
        self.assertIn("InstallScope='perUser'", CORPORATE_WIX)
        self.assertIn("Root='HKCU'", CORPORATE_WIX)

    def test_every_windows_installer_carries_license_and_asset_notice(self) -> None:
        for definition in (GLOBAL_WIX, CORPORATE_WIX, GLOBAL_INNO, CORPORATE_INNO):
            self.assertIn("LICENSE", definition)
            self.assertIn("THIRD_PARTY_ASSETS.md", definition)

    def test_embedded_runtime_assets_are_not_duplicated_in_installer_trees(self) -> None:
        self.assertNotIn('Source: "..\\Assets\\*"', GLOBAL_INNO)
        self.assertNotIn('Source: "..\\Assets\\*"', CORPORATE_INNO)
        self.assertNotIn("Harvest Assets for WiX", WINDOWS_WORKFLOW)
        self.assertNotIn("IncludeHarvestedAssets", WINDOWS_WORKFLOW)

    def test_global_exe_uses_machine_registry_and_opt_in_autostart(self) -> None:
        self.assertIn("PrivilegesRequired=admin", GLOBAL_INNO)
        self.assertIn("Root: HKLM", GLOBAL_INNO)
        self.assertIn('Name: "autostart"', GLOBAL_INNO)
        self.assertIn("Flags: unchecked", GLOBAL_INNO)
        self.assertIn("HKEY_LOCAL_MACHINE", INSTALL_RS)


if __name__ == "__main__":
    unittest.main()
