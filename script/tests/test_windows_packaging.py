from __future__ import annotations

import re
import struct
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
GLOBAL_WIX = (ROOT / "wix" / "main.wxs").read_text(encoding="utf-8")
CORPORATE_WIX = (ROOT / "wix-corporate" / "corporate.wxs").read_text(encoding="utf-8")
GLOBAL_INNO = (ROOT / "inno" / "global.iss").read_text(encoding="utf-8")
CORPORATE_INNO = (ROOT / "inno" / "corporate.iss").read_text(encoding="utf-8")
INSTALL_RS = (ROOT / "src" / "install.rs").read_text(encoding="utf-8")
MAIN_RS = (ROOT / "src" / "main.rs").read_text(encoding="utf-8")
WINDOWS_APP = (ROOT / "src" / "bin" / "honk300-app.rs").read_text(encoding="utf-8")
CONFIG_TUI = (ROOT / "crates" / "honk-config-tui" / "src" / "lib.rs").read_text(
    encoding="utf-8"
)
CARGO_TOML = (ROOT / "Cargo.toml").read_text(encoding="utf-8")
BUILD_RS = (ROOT / "build.rs").read_text(encoding="utf-8")
APP_ICON = ROOT / "Assets" / "UI" / "honk300-app.ico"
APP_RESOURCE = (ROOT / "Assets" / "UI" / "honk300-app.rc").read_text(
    encoding="utf-8"
)
WINDOWS_OVERLAY_SMOKE = (
    ROOT / "script" / "smoke_windows_overlay.ps1"
).read_text(encoding="utf-8")
WINDOWS_WORKFLOW = (
    ROOT / ".github" / "workflows" / "windows-installers.yml"
).read_text(encoding="utf-8")
WINDOWS_SLOT_SMOKE = (ROOT / "script" / "smoke_windows_slot_update.ps1").read_text(
    encoding="utf-8"
)
MSI_LICENSE = ROOT / "wix" / "honk300-license.rtf"
MSI_LICENSE_REFERENCE = (
    "<WixVariable Id='WixUILicenseRtf' "
    "Value='$(var.SourceRoot)\\wix\\honk300-license.rtf'/>"
)
LICENSE_START = "BEGIN ACTUAL POLYFORM LICENSE TERMS"
LICENSE_END = "END ACTUAL LICENSE TERMS"
GOOSE_START = "THE GREAT HONK ACCORD"


def rtf_to_plain_text(rtf: str) -> str:
    """Extract text from the deliberately minimal installer RTF."""

    def decode_hex(match: re.Match[str]) -> str:
        return bytes.fromhex(match.group(1)).decode("windows-1252")

    text = re.sub(r"\\'([0-9a-fA-F]{2})", decode_hex, rtf)
    text = re.sub(r"\\(?:par|line)\b ?", "\n", text)
    text = re.sub(r"\\tab\b ?", "\t", text)
    text = re.sub(r"\\[a-zA-Z]+-?\d* ?", "", text)
    text = text.replace(r"\{", "{").replace(r"\}", "}")
    text = text.replace(r"\\", "\\")
    text = text.replace("{", "").replace("}", "")
    return "\n".join(line.rstrip() for line in text.splitlines()).strip()


def normalize_license_markdown(markdown: str) -> str:
    """Remove presentation-only Markdown while preserving every license word."""

    lines = []
    for raw_line in markdown.splitlines():
        line = re.sub(r"^#{1,6}\s+", "", raw_line.strip())
        line = re.sub(r"^>\s?", "", line)
        line = re.sub(r"\[([^\]]+)\]\([^)]+\)", r"\1", line)
        line = line.replace("***", "").replace("**", "").replace("`", "")
        if line:
            lines.append(re.sub(r"\s+", " ", line))
    return "\n".join(lines)


def ico_sizes(path: Path) -> set[tuple[int, int]]:
    data = path.read_bytes()
    reserved, kind, count = struct.unpack_from("<HHH", data)
    if (reserved, kind) != (0, 1):
        raise AssertionError("Windows app icon is not an ICO resource")
    sizes = set()
    for index in range(count):
        width, height = struct.unpack_from("<BB", data, 6 + index * 16)
        sizes.add((width or 256, height or 256))
    return sizes


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
        for definition in (GLOBAL_WIX, CORPORATE_WIX):
            self.assertNotIn("<Component Id='LegalNotices' Guid='*'>", definition)
            self.assertIn("InstallerVersion='500'", definition)
        for definition in (GLOBAL_WIX, CORPORATE_WIX):
            self.assertIn("Schedule='afterInstallExecute'", definition)
            self.assertIn("After='RemoveExistingProducts'", definition)

    def test_every_msi_uses_the_same_custom_license_rtf(self) -> None:
        self.assertTrue(MSI_LICENSE.is_file())
        for definition in (GLOBAL_WIX, CORPORATE_WIX):
            self.assertEqual(definition.count(MSI_LICENSE_REFERENCE), 1)

    def test_custom_msi_license_preserves_the_authoritative_terms(self) -> None:
        rtf = MSI_LICENSE.read_text(encoding="ascii")
        self.assertTrue(rtf.startswith(r"{\rtf1\ansi\ansicpg1252"))
        plain_text = rtf_to_plain_text(rtf)
        legal_text = plain_text.split(LICENSE_START, 1)[1].split(LICENSE_END, 1)[0]
        self.assertEqual(
            "\n".join(
                re.sub(r"\s+", " ", line.strip())
                for line in legal_text.splitlines()
                if line.strip()
            ),
            normalize_license_markdown((ROOT / "LICENSE").read_text(encoding="utf-8")),
        )

    def test_goose_accord_is_long_nonbinding_and_placeholder_free(self) -> None:
        plain_text = rtf_to_plain_text(MSI_LICENSE.read_text(encoding="ascii"))
        self.assertNotRegex(plain_text.lower(), r"\b(?:lorem|ipsum)\b")
        goose_text = plain_text.split(GOOSE_START, 1)[1]
        words = re.findall(r"[A-Za-z0-9]+(?:['-][A-Za-z0-9]+)*", goose_text)
        self.assertGreaterEqual(len(words), 1_800)
        self.assertLessEqual(len(words), 2_200)
        opening = " ".join(words[:250]).lower()
        closing = " ".join(words[-250:]).lower()
        self.assertIn("this ceremonial appendix is comedy not a contract", opening)
        self.assertIn(
            "nothing in this appendix changes the polyform noncommercial license",
            closing,
        )

    def test_hosted_global_msi_smoke_covers_legacy_upgrade_and_slot_activation(self) -> None:
        self.assertIn("v0.2.1/honk300-x86_64-pc-windows-msvc.msi", WINDOWS_WORKFLOW)
        self.assertIn(
            "9566f3cc4c97fd16b087f72f16aedf0f80e1044868f2c0694329b4462929e022",
            WINDOWS_WORKFLOW,
        )
        self.assertIn("AllowDowngrades='yes'", GLOBAL_WIX)
        self.assertIn("smoke_windows_slot_update.ps1", WINDOWS_WORKFLOW)
        self.assertIn(".owner-cleanup-pending.json", INSTALL_RS)
        self.assertIn("retry_windows_owner_cleanup", INSTALL_RS)

    def test_every_windows_app_entrypoint_uses_the_gui_subsystem_launcher(self) -> None:
        for wix in (GLOBAL_WIX, CORPORATE_WIX):
            self.assertIn("Name='honk300-app.exe'", wix)
            self.assertIn("Target='[APPLICATIONFOLDER]bin\\honk300-app.exe'", wix)
            self.assertIn("Value='\"[APPLICATIONFOLDER]bin\\honk300-app.exe\"'", wix)
            self.assertNotIn("Target='[APPLICATIONFOLDER]bin\\honk300.exe' Arguments='start'", wix)
            self.assertNotIn("<Directory Id='Bin'", wix)
        for inno in (GLOBAL_INNO, CORPORATE_INNO):
            self.assertIn(r'Source: "{#SourceBinDir}\honk300-app.exe"', inno)
            self.assertIn(r'Filename: "{app}\bin\honk300-app.exe"', inno)
            self.assertNotIn(r'Filename: "{app}\bin\honk300.exe"; Parameters: "start"', inno)
        for required in (
            'WINDOWS_APP_LAUNCHER_NAME: &str = "honk300-app.exe"',
            "windows_autostart_command",
            "legacy_windows_autostart_command",
            'creation_flags(CREATE_NO_WINDOW)',
            '"__windows-retire-owner"',
            "remove_windows_retired_owner_integrations",
            "windows_public_path_contains",
        ):
            self.assertIn(required, INSTALL_RS)
        for required in (
            '#![cfg_attr(windows, windows_subsystem = "windows")]',
            "Command::new(runtime)",
            '.arg("__windows-app-runtime")',
            ".args(forwarded)",
            "READINESS_TIMEOUT",
            "stop_unready_child",
            "CREATE_BREAKAWAY_FROM_JOB",
            "BASE_RUNTIME_CREATION_FLAGS | CREATE_BREAKAWAY_FROM_JOB",
            "error.raw_os_error() == Some(5)",
            ".creation_flags(creation_flags)",
            ".stdin(Stdio::null())",
            ".stdout(Stdio::null())",
            ".stderr(Stdio::null())",
        ):
            self.assertIn(required, WINDOWS_APP)
        self.assertNotIn("powershell", WINDOWS_APP.lower())
        self.assertNotIn("cmd.exe", WINDOWS_APP.lower())
        self.assertNotIn("DETACHED_PROCESS", WINDOWS_APP)
        self.assertIn("windows_app_launcher_path", MAIN_RS)
        self.assertIn("append_windows_start_options", MAIN_RS)
        self.assertIn("Command::WindowsAppRuntime", MAIN_RS)
        self.assertIn(
            'x86_64-pc-windows-msvc = ["honk300", "honk300-app"]', CARGO_TOML
        )
        self.assertIn(
            'aarch64-pc-windows-msvc = ["honk300", "honk300-app"]', CARGO_TOML
        )
        self.assertIn("CREATE_NO_WINDOW | CREATE_NEW_PROCESS_GROUP", CONFIG_TUI)
        self.assertIn("let launcher_may_exit = true;", CONFIG_TUI)
        self.assertNotIn("DETACHED_PROCESS", CONFIG_TUI)

    def test_windows_app_launcher_has_a_branded_multi_resolution_resource(self) -> None:
        self.assertTrue(APP_ICON.is_file())
        self.assertTrue(
            {(16, 16), (32, 32), (48, 48), (256, 256)}.issubset(
                ico_sizes(APP_ICON)
            )
        )
        self.assertIn('1 ICON "honk300-app.ico"', APP_RESOURCE)
        self.assertIn('"Assets/UI/honk300-app.rc"', BUILD_RS)
        self.assertIn('["honk300-app"]', BUILD_RS)
        self.assertIn('ParamsIncludeDirs(["Assets/UI"])', BUILD_RS)
        self.assertIn("manifest_required", BUILD_RS)
        self.assertIn('embed-resource = "3.0.11"', CARGO_TOML)
        self.assertIn('"/build.rs"', CARGO_TOML)

    def test_windows_smoke_separates_public_detachment_from_private_runtime_evidence(self) -> None:
        self.assertIn("function Test-PublicStartDetachment", WINDOWS_OVERLAY_SMOKE)
        self.assertIn("public-start-detachment.txt", WINDOWS_OVERLAY_SMOKE)
        self.assertIn("dead_app_parent_pid", WINDOWS_OVERLAY_SMOKE)
        self.assertIn("singleton=$singletonResult", WINDOWS_OVERLAY_SMOKE)
        self.assertIn("left $($appInfo.Count) transient app launcher", WINDOWS_OVERLAY_SMOKE)
        self.assertIn("$trayOwnerDeadline", WINDOWS_OVERLAY_SMOKE)
        self.assertIn("AddSeconds(5)", WINDOWS_OVERLAY_SMOKE)
        self.assertIn("retained public-start runtime did not complete", WINDOWS_OVERLAY_SMOKE)
        self.assertIn(
            "@('__windows-app-runtime', '--config'", WINDOWS_OVERLAY_SMOKE
        )

    def test_embedded_runtime_assets_are_not_duplicated_in_installer_trees(self) -> None:
        self.assertNotIn('Source: "..\\Assets\\*"', GLOBAL_INNO)
        self.assertNotIn('Source: "..\\Assets\\*"', CORPORATE_INNO)
        self.assertNotIn("Harvest Assets for WiX", WINDOWS_WORKFLOW)
        self.assertNotIn("IncludeHarvestedAssets", WINDOWS_WORKFLOW)

    def test_global_exe_uses_machine_registry_and_opt_in_autostart(self) -> None:
        self.assertIn("PrivilegesRequired=admin", GLOBAL_INNO)
        self.assertIn("RegWriteStringValue(HKEY_LOCAL_MACHINE", GLOBAL_INNO)
        self.assertIn("RegDeleteValue(HKEY_LOCAL_MACHINE", GLOBAL_INNO)
        self.assertIn('Name: "autostart"', GLOBAL_INNO)
        self.assertIn("Flags: unchecked", GLOBAL_INNO)
        self.assertIn("HKEY_LOCAL_MACHINE", INSTALL_RS)

    def test_slot_smoke_supplies_required_autostart_state_to_both_protocols(self) -> None:
        self.assertIn("--autostart false", WINDOWS_SLOT_SMOKE)
        self.assertIn("-u false", WINDOWS_SLOT_SMOKE)


if __name__ == "__main__":
    unittest.main()
