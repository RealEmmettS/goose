"""Resolve the pinned Zig compiler without changing the user's global toolchain."""
from __future__ import annotations
import hashlib
import os
from pathlib import Path, PurePosixPath
import platform
import shutil
import subprocess
import tarfile
import urllib.request
import zipfile

# Official https://ziglang.org/download/index.json, Zig 0.16.0 release.
HASHES = {
    "x86_64-windows": "68659eb5f1e4eb1437a722f1dd889c5a322c9954607f5edcf337bc3684a75a7e",
    "aarch64-windows": "aee38316ee4111717900f45dd3130145c39289e105541d737eb8c5ed653c78ef",
    "x86_64-linux": "70e49664a74374b48b51e6f3fdfbf437f6395d42509050588bd49abe52ba3d00",
    "aarch64-linux": "ea4b09bfb22ec6f6c6ceac57ab63efb6b46e17ab08d21f69f3a48b38e1534f17",
    "x86_64-macos": "0387557ed1877bc6a2e1802c8391953baddba76081876301c522f52977b52ba7",
    "aarch64-macos": "b23d70deaa879b5c2d486ed3316f7eaa53e84acf6fc9cc747de152450d401489",
}


def ensure_zig(root: Path) -> str:
    for candidate in [os.environ.get("NATIVE_SDK_ZIG"), shutil.which("zig")]:
        if candidate and subprocess.check_output([candidate, "version"], text=True).strip() == "0.16.0":
            return str(Path(candidate).resolve())
    machine = platform.machine().lower()
    arch = {"amd64": "x86_64", "x86_64": "x86_64", "arm64": "aarch64", "aarch64": "aarch64"}[machine]
    system = {"Windows": "windows", "Linux": "linux", "Darwin": "macos"}[platform.system()]
    host = f"{arch}-{system}"
    name = f"zig-{host}-0.16.0"
    directory = root / "target/settings-toolchain"
    binary = directory / name / ("zig.exe" if system == "windows" else "zig")
    if not binary.is_file():
        directory.mkdir(parents=True, exist_ok=True)
        extension = ".zip" if system == "windows" else ".tar.xz"
        archive = directory / (name + extension)
        with urllib.request.urlopen(f"https://ziglang.org/download/0.16.0/{archive.name}", timeout=120) as response:
            data = response.read(110 * 1024 * 1024)
        if hashlib.sha256(data).hexdigest() != HASHES[host]:
            raise RuntimeError("Zig toolchain SHA-256 mismatch")
        archive.write_bytes(data)
        if extension == ".zip":
            with zipfile.ZipFile(archive) as package:
                for member in package.namelist():
                    path = PurePosixPath(member)
                    if path.is_absolute() or ".." in path.parts or "\\" in member:
                        raise RuntimeError("Unsafe Zig toolchain archive path")
                package.extractall(directory)
        else:
            with tarfile.open(archive) as package:
                # These are exact hash-pinned official bytes. Also retain the
                # standard data filter on Python versions which support it.
                if hasattr(tarfile, "data_filter"):
                    package.extractall(directory, filter="data")
                else:
                    package.extractall(directory)
        archive.unlink()
    if subprocess.check_output([str(binary), "version"], text=True).strip() != "0.16.0":
        raise RuntimeError("Unexpected Zig toolchain version")
    return str(binary.resolve())
