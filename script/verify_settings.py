#!/usr/bin/env python3
"""Fail closed on a missing, mismatched, or wrong-architecture native companion."""
from __future__ import annotations

import argparse
import hashlib
import json
import os
from pathlib import Path
import subprocess
import sys
try:
    import tomllib
except ModuleNotFoundError:
    import tomli as tomllib

from build_settings import TARGETS


def verify(directory: Path, target: str, execute: bool = False) -> Path:
    root = Path(__file__).resolve().parents[1]
    version = tomllib.loads((root / "Cargo.toml").read_text(encoding="utf-8"))["package"]["version"]
    metadata = json.loads((directory / "settings-build.json").read_text(encoding="utf-8"))
    name = "honk300-settings" + (".exe" if "windows" in target else "")
    binary = directory / name
    assert binary.is_file() and not binary.is_symlink(), "settings executable is not a regular file"
    assert metadata["schema"] == "honk300.settings-build.v1"
    assert metadata["target"] == target and metadata["version"] == version
    assert metadata["name"] == name and metadata["automation"] is False
    assert metadata["size"] == binary.stat().st_size
    assert metadata["sha256"] == hashlib.sha256(binary.read_bytes()).hexdigest()
    assert metadata["native_sdk"] == "0.5.4" and metadata["zig"] == "0.16.0"
    if "windows" in target:
        identity = metadata["accessibility"]
        assert identity["name"] == "honk_settings_accessibility.dll"
        bridge = directory / identity["name"]
        assert bridge.is_file() and not bridge.is_symlink(), "missing native accessibility DLL"
        assert bridge.stat().st_size == identity["size"]
        assert hashlib.sha256(bridge.read_bytes()).hexdigest() == identity["sha256"]
        subprocess.run([sys.executable, str(root / "script/verify_binary_architecture.py"),
                        "--format", "pe", "--machine", "0x8664" if target.startswith("x86") else "0xAA64", str(bridge)], check=True)
        arguments = ["--format", "pe", "--machine", "0x8664" if target.startswith("x86") else "0xAA64", "--subsystem", "2"]
    elif "linux" in target:
        arguments = ["--format", "elf", "--machine", "62" if target.startswith("x86") else "183"]
    else:
        arguments = None
        # lipo verifies both thin and universal Mach-O without executing target code.
        subprocess.run(["lipo", str(binary), "-verify_arch", "x86_64" if target.startswith("x86") else "arm64"], check=True)
    if arguments:
        subprocess.run([sys.executable, str(root / "script/verify_binary_architecture.py"), *arguments, str(binary)], check=True)
    if "linux" in target:
        headers = subprocess.run(["readelf", "-l", str(binary)], check=True, capture_output=True, text=True).stdout
        interpreter = next(line for line in headers.splitlines() if "Requesting program interpreter:" in line)
        assert ("ld-musl-" in interpreter) if target.endswith("musl") else ("ld-linux" in interpreter), "GUI libc does not match archive target"
    if execute:
        result = subprocess.run([str(binary.resolve()), "--version"], capture_output=True,
                                text=True, check=True, timeout=10,
                                creationflags=subprocess.CREATE_NO_WINDOW if os.name == "nt" else 0)
        assert (result.stdout + result.stderr).strip() == f"honk300-settings {version}"
    print(f"Verified production settings: {target} {metadata['sha256']}")
    return binary


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--target", choices=TARGETS, required=True)
    parser.add_argument("--directory", type=Path, required=True)
    parser.add_argument("--execute", action="store_true")
    args = parser.parse_args()
    verify(args.directory, args.target, args.execute)
