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
        self.assertIn('"schema": "honk300.install.v1"', SHELL)
        self.assertIn('"commit": "$(json_escape "$COMMIT")"', SHELL)
        self.assertIn('"owner": "honk300-installer"', SHELL)
        self.assertIn("codesign --verify --deep --strict", SHELL)
        self.assertIn("dev.emmetts.honk300", SHELL)
        self.assertIn("archive contains links; refusing extraction", SHELL)
        self.assertNotIn('cp -R "$assets_source"', SHELL)

    def test_powershell_bootstrap_installs_only_verified_global_msi(self) -> None:
        for token in ["__VERSION__", "__TAG__", "__COMMIT__", "__SHA_WINDOWS_X64_MSI__", "__SHA_WINDOWS_ARM64_MSI__"]:
            self.assertIn(token, POWERSHELL)
        self.assertIn("Get-FileHash", POWERSHELL)
        self.assertIn("msiexec.exe", POWERSHELL)
        self.assertIn("[Environment+SpecialFolder]::System", POWERSHELL)
        self.assertIn("Get-Item -LiteralPath $msiexec -Force", POWERSHELL)
        self.assertIn("[IO.FileAttributes]::ReparsePoint", POWERSHELL)
        self.assertIn("Start-Process -FilePath $msiexec", POWERSHELL)
        self.assertNotIn("Start-Process -FilePath 'msiexec.exe'", POWERSHELL)
        self.assertIn("-Verb RunAs", POWERSHELL)
        self.assertIn("ProgramFiles", POWERSHELL)
        self.assertIn("honk300.install.v1", POWERSHELL)
        self.assertIn("install-receipt.json", POWERSHELL)
        self.assertIn("$Commit", POWERSHELL)
        self.assertIn("refusing to replace a reparse-point install receipt", POWERSHELL)
        self.assertIn("refusing to replace a foreign install receipt", POWERSHELL)
        self.assertNotIn("corporate", POWERSHELL.lower())
        self.assertNotIn(".zip", POWERSHELL.lower())

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
            {"__VERSION__", "__TAG__", "__COMMIT__", "__SHA_WINDOWS_X64_MSI__", "__SHA_WINDOWS_ARM64_MSI__"},
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


if __name__ == "__main__":
    unittest.main()
