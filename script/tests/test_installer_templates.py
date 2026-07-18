from __future__ import annotations

import re
import os
import shutil
import stat
import subprocess
import tarfile
import tempfile
import unittest
import zipfile
from io import BytesIO
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
SHELL = (ROOT / "script" / "honk300-installer.sh.in").read_text(encoding="utf-8")
POWERSHELL = (ROOT / "script" / "honk300-installer.ps1.in").read_text(encoding="utf-8")
WIX_GLOBAL = (ROOT / "wix" / "main.wxs").read_text(encoding="utf-8")
WIX_CORPORATE = (ROOT / "wix-corporate" / "corporate.wxs").read_text(encoding="utf-8")
INNO_GLOBAL = (ROOT / "inno" / "global.iss").read_text(encoding="utf-8")
INNO_CORPORATE = (ROOT / "inno" / "corporate.iss").read_text(encoding="utf-8")
WINDOWS_WORKFLOW = (ROOT / ".github" / "workflows" / "windows-installers.yml").read_text(encoding="utf-8")


class InstallerTemplateTests(unittest.TestCase):
    def _bash(self) -> str:
        if os.name == "nt":
            candidate = Path(os.environ.get("ProgramFiles", r"C:\Program Files")) / "Git" / "bin" / "bash.exe"
            if candidate.is_file():
                return str(candidate)
        bash = shutil.which("bash")
        if not bash:
            self.skipTest("bash is unavailable")
        return bash

    def _validate_archive(self, kind: str, archive: Path, temp_root: Path) -> subprocess.CompletedProcess[str]:
        work = Path(tempfile.mkdtemp(prefix="validator-", dir=temp_root))
        prefix = work / "installer-functions.sh"
        prefix.write_text(SHELL.split('\nOS="$(uname -s)"', 1)[0] + "\n", encoding="utf-8")
        command = (
            'prefix="$PREFIX"; archive="$ARCHIVE"; work="$WORK"; '
            'if command -v cygpath >/dev/null 2>&1; then '
            'prefix="$(cygpath -u "$prefix")"; archive="$(cygpath -u "$archive")"; work="$(cygpath -u "$work")"; fi; '
            '. "$prefix"; TEMP_ROOT="$work"; '
            f'validate_{kind} "$archive"'
        )
        environment = os.environ.copy()
        environment.update({"PREFIX": str(prefix), "ARCHIVE": str(archive), "WORK": str(work)})
        return subprocess.run(
            [self._bash(), "-c", command],
            capture_output=True,
            text=True,
            check=False,
            env=environment,
        )

    def test_shell_bootstrap_uses_exact_tag_payloads_and_all_target_hashes(self) -> None:
        self.assertIn("releases/download/$TAG/$ARTIFACT", SHELL)
        self.assertNotIn("releases/latest/download/$ARTIFACT", SHELL)
        for token in [
            "__VERSION__",
            "__TAG__",
            "__COMMIT__",
            "__SHA_MAC_APP__",
            "__SHA_LINUX_X64_GNU__",
            "__SHA_LINUX_ARM64_GNU__",
            "__SHA_LINUX_X64_MUSL__",
            "__SHA_LINUX_ARM64_MUSL__",
        ]:
            self.assertIn(token, SHELL)

    def test_macos_success_copy_matches_developer_id_notarized_release(self) -> None:
        self.assertIn("Developer ID-signed, notarized, and stapled", SHELL)
        self.assertNotIn("This build is ad-hoc signed and not notarized", SHELL)

    def test_shell_bootstrap_quiesces_before_swap_and_pins_bundle_release_identity(self) -> None:
        hold = SHELL[SHELL.index("hold_lifecycle_lease() {"):SHELL.index("\nrollback() {")]
        self.assertIn('[ "$waited" -ge 35 ]', hold)
        self.assertIn("did not release lifecycle ownership within 35 seconds", hold)
        self.assertIn("HONK300_INTERNAL_HOLD_LIFECYCLE_LEASE=1", hold)
        self.assertIn("HONK300_INTERNAL_LIFECYCLE_LEASE_READY", hold)
        self.assertIn('exec 9>"$LEASE_FIFO"', hold)
        self.assertIn("release_lifecycle_lease", hold)
        self.assertLess(hold.index("mkfifo"), hold.index("LEASE_FD_OPEN=1"))

        mac_swap = SHELL.index('swap_install "$CANDIDATE" "$DEST"')
        linux_swap = SHELL.index('activate_linux_slot "$STAGE_ROOT/install"')
        self.assertLess(
            SHELL.rindex('hold_lifecycle_lease "$STAGED_BINARY"', 0, mac_swap),
            mac_swap,
        )
        self.assertLess(
            SHELL.rindex(
                'hold_lifecycle_lease "$STAGE_ROOT/install/bin/honk300"',
                0,
                linux_swap,
            ),
            linux_swap,
        )
        cleanup = SHELL[SHELL.index("cleanup() {"):SHELL.index("\nsafe_link() {")]
        self.assertLess(cleanup.index("rollback || true"), cleanup.index("release_lifecycle_lease"))
        self.assertIn('on_exit() { status=$?; cleanup "$status"; }', cleanup)
        self.assertIn("on_hup() { cleanup 129; }", cleanup)
        self.assertIn("on_int() { cleanup 130; }", cleanup)
        self.assertIn("on_term() { cleanup 143; }", cleanup)
        self.assertIn("Print :Honk300ReleaseTag", SHELL)
        self.assertIn('[ "$bundle_tag" = "$TAG" ]', SHELL)
        self.assertIn("Print :Honk300ReleaseCommit", SHELL)
        self.assertIn('[ "$bundle_commit" = "$COMMIT" ]', SHELL)

    @unittest.skipIf(os.name == "nt", "POSIX signal semantics are validated on Unix hosts")
    def test_shell_term_after_swap_restores_previous_install_and_exits_143(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            destination = root / "install"
            previous = root / "install.previous"
            destination.write_text("new\n", encoding="utf-8")
            previous.write_text("old\n", encoding="utf-8")
            prefix = SHELL[: SHELL.index("\nsafe_link() {")]
            harness = root / "interrupt-after-swap.sh"
            harness.write_text(
                prefix
                + "\n"
                + 'DEST="$TEST_DEST"\n'
                + 'PREVIOUS="$TEST_PREVIOUS"\n'
                + "SWAPPED=1\n"
                + 'kill -TERM "$$"\n'
                + "exit 99\n",
                encoding="utf-8",
            )
            environment = os.environ.copy()
            environment.update(
                {"TEST_DEST": str(destination), "TEST_PREVIOUS": str(previous)}
            )
            result = subprocess.run(
                [self._bash(), str(harness)],
                capture_output=True,
                text=True,
                check=False,
                env=environment,
            )
            self.assertEqual(result.returncode, 143, result.stdout + result.stderr)
            self.assertEqual(destination.read_text(encoding="utf-8"), "old\n")
            self.assertFalse(previous.exists())

    @unittest.skipIf(os.name == "nt", "POSIX symlink rollback is validated on Unix hosts")
    def test_shell_failure_before_selector_commit_preserves_old_slot(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            destination = root / "install"
            releases = destination / "releases"
            old_release = releases / "1.0.0-x86_64-unknown-linux-gnu"
            new_release = releases / "1.1.0-x86_64-unknown-linux-gnu"
            old_release.mkdir(parents=True)
            new_release.mkdir()
            (destination / "current").symlink_to(old_release, target_is_directory=True)
            prefix = SHELL[: SHELL.index("\nsafe_link() {")]
            harness = root / "failure-before-selector.sh"
            harness.write_text(
                prefix
                + "\n"
                + 'DEST="$TEST_DEST"\n'
                + 'PREVIOUS="$TEST_OLD"\n'
                + 'NEW_RELEASE="$TEST_NEW"\n'
                + "NEW_RELEASE_CREATED=1\n"
                + 'die "fault injection before selector commit"\n',
                encoding="utf-8",
            )
            environment = os.environ.copy()
            environment.update(
                {
                    "TEST_DEST": str(destination),
                    "TEST_OLD": str(old_release),
                    "TEST_NEW": str(new_release),
                }
            )
            result = subprocess.run(
                [self._bash(), str(harness)],
                capture_output=True,
                text=True,
                check=False,
                env=environment,
            )
            self.assertEqual(result.returncode, 1, result.stdout + result.stderr)
            self.assertEqual((destination / "current").resolve(), old_release.resolve())
            self.assertFalse(new_release.exists())

    def test_shell_bootstrap_is_user_scoped_transactional_and_rejects_bad_archives(self) -> None:
        self.assertIn("$HOME/Applications/Honk300.app", SHELL)
        self.assertIn("${XDG_DATA_HOME:-$HOME/.local/share}/honk300/install", SHELL)
        self.assertIn("$HOME/.local/bin", SHELL)
        self.assertNotRegex(SHELL, r"(^|\s)sudo(\s|$)")
        self.assertIn("reject_archive_entry", SHELL)
        self.assertIn("uniq -d", SHELL)
        self.assertIn(".previous.$PID", SHELL)
        self.assertIn('[ ! -e "$PREVIOUS" ] && [ ! -L "$PREVIOUS" ]', SHELL)
        self.assertIn("rollback", SHELL)
        self.assertIn("HONK300_TEST_LOCAL_DIR", SHELL)
        self.assertIn("HONK300_TEST_FAIL_AFTER_SWAP", SHELL)
        self.assertIn("CREATED_LINKS", SHELL)
        self.assertIn("PATH_PROFILE", SHELL)
        self.assertIn("X-Honk300-Managed=true", SHELL)
        self.assertIn("RECEIPT_BACKUP", SHELL)
        self.assertIn('"schema": "honk300.install.v2"', SHELL)
        self.assertIn('"origin": "$(json_escape "$origin")"', SHELL)
        self.assertIn('"active_release": "$(json_escape "$active_release")"', SHELL)
        self.assertIn('release_id="$VERSION-$TARGET"', SHELL)
        self.assertIn('selector="$DEST/current"', SHELL)
        self.assertIn('"commit": "$(json_escape "$COMMIT")"', SHELL)
        self.assertIn('"owner": "honk300-installer"', SHELL)
        self.assertIn("refusing to replace an unreceipted macOS app bundle", SHELL)
        self.assertIn("codesign --verify --deep --strict", SHELL)
        self.assertIn("dev.emmetts.honk300", SHELL)
        self.assertIn("archive contains links; refusing extraction", SHELL)
        self.assertNotIn('cp -R "$assets_source"', SHELL)

    def test_powershell_bootstrap_installs_only_verified_global_msi(self) -> None:
        for token in [
            "__VERSION__",
            "__TAG__",
            "__COMMIT__",
            "__SHA_WINDOWS_X64_MSI__",
            "__SHA_WINDOWS_ARM64_MSI__",
            "__SHA_WINDOWS_X64_PORTABLE__",
            "__SHA_WINDOWS_ARM64_PORTABLE__",
            "__SIZE_WINDOWS_X64_MSI__",
            "__SIZE_WINDOWS_ARM64_MSI__",
            "__SIZE_WINDOWS_X64_PORTABLE__",
            "__SIZE_WINDOWS_ARM64_PORTABLE__",
        ]:
            self.assertIn(token, POWERSHELL)
        self.assertIn("[IO.FileShare]::Read", POWERSHELL)
        self.assertIn("$sha.ComputeHash($stream)", POWERSHELL)
        self.assertIn("$stream.Length -ne $ExpectedSize", POWERSHELL)
        self.assertIn("msiexec.exe", POWERSHELL)
        self.assertIn("[Environment+SpecialFolder]::System", POWERSHELL)
        self.assertIn("Get-Item -LiteralPath $msiexec -Force", POWERSHELL)
        self.assertIn("[IO.FileAttributes]::ReparsePoint", POWERSHELL)
        self.assertIn("Start-Process -FilePath $msiexec", POWERSHELL)
        self.assertNotIn("Start-Process -FilePath 'msiexec.exe'", POWERSHELL)
        self.assertIn("-Verb RunAs", POWERSHELL)
        self.assertIn("ProgramFiles", POWERSHELL)
        self.assertIn("honk300.install.v2", POWERSHELL)
        self.assertIn("install-receipt.json", POWERSHELL)
        self.assertIn("$Commit", POWERSHELL)
        self.assertIn("refusing to replace a reparse-point install receipt", POWERSHELL)
        self.assertIn("refusing to replace a foreign install receipt", POWERSHELL)
        self.assertIn("HONK300_INTERNAL_HOLD_LIFECYCLE_LEASE", POWERSHELL)
        self.assertIn("HONK300_INTERNAL_LIFECYCLE_LEASE_READY", POWERSHELL)
        self.assertIn("function Get-HiddenProgramVersion", POWERSHELL)
        self.assertIn("$start.CreateNoWindow = $true", POWERSHELL)
        self.assertIn("$reported = Get-HiddenProgramVersion $installed", POWERSHELL)
        self.assertNotIn("& $installed --version", POWERSHELL)
        self.assertIn("RedirectStandardInput", POWERSHELL)
        self.assertIn("portable lifecycle archive must contain exactly one honk300.exe", POWERSHELL)
        self.assertIn("[IO.Compression.ZipArchive]::new", POWERSHELL)
        self.assertNotIn("rstrtmgr.dll", POWERSHELL)
        self.assertNotIn("Assert-NoRestartManagerLocks", POWERSHELL)
        self.assertIn("HONK300ORIGIN=powershell", POWERSHELL)
        self.assertNotIn("@(0, 3010)", POWERSHELL)
        self.assertIn("pending or reboot-deferred replacement is not accepted", POWERSHELL)
        self.assertLess(
            POWERSHELL.index("$leaseProcess = Start-LifecycleLease"),
            POWERSHELL.index("Start-Process -FilePath $msiexec"),
        )
        self.assertLess(
            POWERSHELL.index("Start-Process -FilePath $msiexec"),
            POWERSHELL.rindex("$artifactStream.Dispose()"),
        )
        self.assertNotIn("corporate", POWERSHELL.lower())

    def test_powershell_bootstrap_template_parses_when_powershell_is_available(self) -> None:
        shell = shutil.which("pwsh") or shutil.which("powershell")
        if not shell:
            self.skipTest("PowerShell parser is unavailable")
        parser = (
            "$tokens=$null; $errors=$null; "
            "[System.Management.Automation.Language.Parser]::ParseFile("
            "$env:HONK300_POWERSHELL_TEMPLATE,"
            "[ref]$tokens,[ref]$errors) > $null; "
            "if ($errors.Count) { $errors | ForEach-Object { "
            "[Console]::Error.WriteLine($_.Message) }; exit 1 }"
        )
        environment = os.environ.copy()
        environment["HONK300_POWERSHELL_TEMPLATE"] = str(
            ROOT / "script" / "honk300-installer.ps1.in"
        )
        result = subprocess.run(
            [shell, "-NoProfile", "-Command", parser],
            capture_output=True,
            text=True,
            check=False,
            env=environment,
        )
        self.assertEqual(result.returncode, 0, result.stderr)

    def test_templates_contain_only_release_renderer_tokens(self) -> None:
        shell_tokens = set(re.findall(r"__[A-Z0-9_]+__", SHELL))
        self.assertEqual(
            shell_tokens,
            {
                "__VERSION__",
                "__TAG__",
                "__COMMIT__",
                "__SHA_MAC_APP__",
                "__SHA_LINUX_X64_GNU__",
                "__SHA_LINUX_ARM64_GNU__",
                "__SHA_LINUX_X64_MUSL__",
                "__SHA_LINUX_ARM64_MUSL__",
            },
        )
        self.assertEqual(
            set(re.findall(r"__[A-Z0-9_]+__", POWERSHELL)),
            {
                "__VERSION__",
                "__TAG__",
                "__COMMIT__",
                "__SHA_WINDOWS_X64_MSI__",
                "__SHA_WINDOWS_ARM64_MSI__",
                "__SHA_WINDOWS_X64_PORTABLE__",
                "__SHA_WINDOWS_ARM64_PORTABLE__",
                "__SIZE_WINDOWS_X64_MSI__",
                "__SIZE_WINDOWS_ARM64_MSI__",
                "__SIZE_WINDOWS_X64_PORTABLE__",
                "__SIZE_WINDOWS_ARM64_PORTABLE__",
            },
        )

    def test_archive_validators_execute_and_reject_traversal_duplicates_and_links(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            safe_tar = root / "safe.tar.xz"
            with tarfile.open(safe_tar, "w:xz") as archive:
                item = tarfile.TarInfo("honk300/bin/honk300")
                payload = b"safe"
                item.size = len(payload)
                archive.addfile(item, BytesIO(payload))
            result = self._validate_archive("tar", safe_tar, root)
            self.assertEqual(result.returncode, 0, result.stdout + result.stderr)

            traversal_tar = root / "traversal.tar.xz"
            with tarfile.open(traversal_tar, "w:xz") as archive:
                item = tarfile.TarInfo("../escape")
                item.size = 1
                archive.addfile(item, BytesIO(b"x"))
            self.assertNotEqual(self._validate_archive("tar", traversal_tar, root).returncode, 0)

            duplicate_tar = root / "duplicate.tar.xz"
            with tarfile.open(duplicate_tar, "w:xz") as archive:
                for payload in (b"a", b"b"):
                    item = tarfile.TarInfo("honk300/bin/honk300")
                    item.size = 1
                    archive.addfile(item, BytesIO(payload))
            self.assertNotEqual(self._validate_archive("tar", duplicate_tar, root).returncode, 0)

            link_tar = root / "link.tar.xz"
            with tarfile.open(link_tar, "w:xz") as archive:
                item = tarfile.TarInfo("honk300/link")
                item.type = tarfile.SYMTYPE
                item.linkname = "../../escape"
                archive.addfile(item)
            self.assertNotEqual(self._validate_archive("tar", link_tar, root).returncode, 0)

            link_zip = root / "link.zip"
            with zipfile.ZipFile(link_zip, "w") as archive:
                archive.writestr("Honk300.app/Contents/MacOS/honk300", b"safe")
                link = zipfile.ZipInfo("Honk300.app/link")
                link.create_system = 3
                link.external_attr = (stat.S_IFLNK | 0o777) << 16
                archive.writestr(link, "../../escape")
            self.assertNotEqual(self._validate_archive("zip", link_zip, root).returncode, 0)

            traversal_zip = root / "traversal.zip"
            with zipfile.ZipFile(traversal_zip, "w") as archive:
                archive.writestr("../escape", b"x")
            self.assertNotEqual(self._validate_archive("zip", traversal_zip, root).returncode, 0)

            duplicate_zip = root / "duplicate.zip"
            with zipfile.ZipFile(duplicate_zip, "w") as archive:
                archive.writestr("Honk300.app/Contents/MacOS/honk300", b"a")
                with self.assertWarns(UserWarning):
                    archive.writestr("Honk300.app/Contents/MacOS/honk300", b"b")
            self.assertNotEqual(self._validate_archive("zip", duplicate_zip, root).returncode, 0)

    def test_all_windows_installers_use_slots_preserve_origin_and_allow_latest_intent(self) -> None:
        for wix, origin in [(WIX_GLOBAL, "msi-global"), (WIX_CORPORATE, "msi-corporate")]:
            self.assertIn("AllowDowngrades='yes'", wix)
            self.assertIn("Name='channels'", wix)
            self.assertIn("Name='releases'", wix)
            self.assertIn("Name='$(var.Version)-$(var.TargetTriple)'", wix)
            self.assertGreaterEqual(wix.count("Permanent='yes'"), 6)
            self.assertIn("Name='honk300-app.exe'", wix)
            self.assertNotIn("<Directory Id='Bin' Name='bin'>", wix)
            self.assertIn("Value='[APPLICATIONFOLDER]bin'", wix)
            self.assertIn("Target='[APPLICATIONFOLDER]bin\\honk300-app.exe'", wix)
            self.assertIn("Value='\"[APPLICATIONFOLDER]bin\\honk300-app.exe\"'", wix)
            self.assertNotIn("Target='[Bin]honk300.exe' Arguments='start'", wix)
            self.assertIn("<Component Id='LoginAutostart' Guid='*'>", wix)
            self.assertNotIn("<Component Id='LoginAutostart' Guid='*' Permanent='yes'>", wix)
            self.assertIn("__wsa -r", wix)
            self.assertNotIn("ExeCommand='[CustomActionData]'", wix)
            self.assertNotIn("Id='SetActivateSlotData'", wix)
            self.assertIn("BinaryKey='Honk300SlotHelper' ExeCommand='__windows-slot-uninstall", wix)
            self.assertIn('&quot;[APPLICATIONFOLDER]\\&quot;', wix)
            self.assertNotIn('&quot;[APPLICATIONFOLDER]&quot;', wix)
            self.assertIn('-l &quot;$(var.LauncherSha256)&quot;', wix)
            self.assertIn("__windows-slot-rollback", wix)
            self.assertIn("__windows-slot-commit", wix)
            self.assertIn("__windows-slot-uninstall", wix)
            self.assertIn(origin, wix)
            self.assertNotIn("DowngradeErrorMessage", wix)

        for inno, origin in [(INNO_GLOBAL, "exe-global"), (INNO_CORPORATE, "exe-corporate")]:
            self.assertIn("CloseApplications=no", inno)
            self.assertIn(f"channels\\{origin}\\releases", inno)
            self.assertGreaterEqual(inno.count("uninsneveruninstall"), 5)
            self.assertIn(r'Source: "{#SourceBinDir}\honk300-app.exe"', inno)
            self.assertIn(r'Filename: "{app}\bin\honk300-app.exe"', inno)
            self.assertNotIn(r'Filename: "{app}\bin\honk300.exe"; Parameters: "start"', inno)
            self.assertNotIn("uninsdeletevalue", inno)
            self.assertIn("__windows-slot-activate", inno)
            self.assertIn("__windows-slot-commit", inno)
            self.assertIn("__windows-slot-uninstall", inno)
            self.assertIn("RegWriteStringValue", inno)

        for token in [
            "-dPayloadSha256=$payloadSha256",
            "-dLauncherSha256=$launcherSha256",
            "/DPayloadSha256=$payloadSha256",
            "smoke_windows_slot_update.ps1",
            "smoke_windows_installer_takeover.ps1",
            "honk300.install.v2",
            "stable current junction is missing",
            "all four slot origins with a live native ARM64 process",
        ]:
            self.assertIn(token, WINDOWS_WORKFLOW)

        takeover_smoke = (ROOT / "script" / "smoke_windows_installer_takeover.ps1").read_text(
            encoding="utf-8"
        )
        self.assertIn("@(Get-HonkRegistrations).Count", takeover_smoke)
        self.assertIn("@($activeState).Count", takeover_smoke)
        self.assertIn("[int] $ChildTimeoutSeconds = 180", takeover_smoke)
        self.assertIn("Wait-CheckedProcess", takeover_smoke)
        self.assertIn("progress.log", takeover_smoke)
        self.assertIn("Capture-TakeoverTimeout $Label", takeover_smoke)
        self.assertIn("Get-CimInstance Win32_Process", takeover_smoke)
        self.assertIn("$_.Name -like 'MSI*.tmp'", takeover_smoke)
        self.assertIn(".slot-transaction.json", takeover_smoke)
        self.assertIn("ReadLineAsync()", takeover_smoke)
        self.assertNotIn("if ((Get-HonkRegistrations).Count", takeover_smoke)
        self.assertNotIn("$activeState.Count", takeover_smoke)

        slot_smoke = (ROOT / "script" / "smoke_windows_slot_update.ps1").read_text(
            encoding="utf-8"
        )
        self.assertIn("__wsa -r $root -o $Origin -c $Commit -a $artifact -l $launcherHash", slot_smoke)

        self.assertIn("PREVIOUSHONK300ORIGIN", WIX_GLOBAL)
        self.assertIn("Installed AND PREVIOUSHONK300ORIGIN", WIX_GLOBAL)


if __name__ == "__main__":
    unittest.main()
