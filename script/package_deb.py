#!/usr/bin/env python3
"""Build a deterministic Debian package from an already-qualified GNU binary."""

from __future__ import annotations

import argparse
import json
import os
import re
import shutil
import stat
import subprocess
import tempfile
from pathlib import Path


ARCHITECTURES = {
    "amd64": "x86_64-unknown-linux-gnu",
    "arm64": "aarch64-unknown-linux-gnu",
}
VERSION = re.compile(r"^[0-9]+\.[0-9]+\.[0-9]+$")
COMMIT = re.compile(r"^[0-9a-fA-F]{40}$")
DEPENDENCIES = (
    "libc6 (>= 2.35), "
    "libasound2 | libasound2t64, "
    "libwayland-client0, "
    "libxkbcommon0"
)


def _regular_file(path: Path, label: str) -> Path:
    metadata = path.lstat()
    if not stat.S_ISREG(metadata.st_mode) or path.is_symlink():
        raise ValueError(f"{label} must be a regular non-symlink file: {path}")
    return path


def _write(path: Path, content: str, mode: int = 0o644) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(content, encoding="utf-8", newline="\n")
    path.chmod(mode)


def build_package_tree(
    root: Path,
    binary: Path,
    version: str,
    tag: str,
    commit: str,
    architecture: str,
) -> Path:
    if architecture not in ARCHITECTURES:
        raise ValueError(f"unsupported Debian architecture: {architecture}")
    if not VERSION.fullmatch(version) or tag != f"v{version}":
        raise ValueError("version and tag must agree as stable X.Y.Z / vX.Y.Z values")
    if not COMMIT.fullmatch(commit):
        raise ValueError("commit must be a full 40-character hexadecimal SHA")
    binary = _regular_file(binary, "binary")
    if root.exists():
        raise ValueError(f"package staging root already exists: {root}")

    installed = root / "usr" / "lib" / "honk300" / "honk300"
    installed.parent.mkdir(parents=True)
    shutil.copyfile(binary, installed)
    installed.chmod(0o755)
    _write(installed.parent / "install-source.txt", "deb\n")

    aliases = root / "usr" / "bin"
    aliases.mkdir(parents=True)
    for name in ("honk300", "honk", "goose"):
        (aliases / name).symlink_to("../lib/honk300/honk300")

    target = ARCHITECTURES[architecture]
    metadata = {
        "schema": "honk300.package.v1",
        "version": version,
        "tag": tag,
        "commit": commit.lower(),
        "target": target,
        "architecture": architecture,
    }
    _write(
        root / "usr" / "share" / "honk300" / "release.json",
        json.dumps(metadata, indent=2, sort_keys=True) + "\n",
    )
    _write(
        root / "usr" / "share" / "applications" / "honk300.desktop",
        "[Desktop Entry]\n"
        "Type=Application\n"
        "Name=Honk300\n"
        "Comment=Desktop goose for your screen\n"
        "Exec=/usr/bin/honk300 start\n"
        "Icon=honk300\n"
        "Terminal=false\n"
        "Categories=Utility;Game;\n"
        "StartupNotify=false\n",
    )
    icon_source = (
        Path(__file__).resolve().parents[1]
        / "Assets"
        / "UI"
        / "honk300-status-goose@2x.png"
    )
    icon_destination = (
        root / "usr" / "share" / "icons" / "hicolor" / "36x36" / "apps" / "honk300.png"
    )
    icon_destination.parent.mkdir(parents=True)
    shutil.copyfile(_regular_file(icon_source, "status icon"), icon_destination)
    license_source = Path(__file__).resolve().parents[1] / "LICENSE"
    documentation = root / "usr" / "share" / "doc" / "honk300"
    documentation.mkdir(parents=True)
    shutil.copyfile(_regular_file(license_source, "license"), documentation / "LICENSE")
    _write(
        root / "usr" / "share" / "doc" / "honk300" / "copyright",
        "Format: https://www.debian.org/doc/packaging-manuals/copyright-format/1.0/\n"
        "Upstream-Name: honk300\n"
        "Source: https://github.com/RealEmmettS/goose\n"
        "Files: *\n"
        "Copyright: Emmett S\n"
        "License: PolyForm-Noncommercial-1.0.0\n"
        " See /usr/share/doc/honk300/LICENSE for the complete terms.\n",
    )

    installed_kib = (
        sum(path.stat().st_size for path in root.rglob("*") if path.is_file()) + 1023
    ) // 1024
    _write(
        root / "DEBIAN" / "control",
        f"Package: honk300\n"
        f"Version: {version}\n"
        "Section: games\n"
        "Priority: optional\n"
        f"Architecture: {architecture}\n"
        "Maintainer: Emmett S <hey@emmetts.dev>\n"
        f"Installed-Size: {installed_kib}\n"
        f"Depends: {DEPENDENCIES}\n"
        "Homepage: https://thegoose.app\n"
        "Description: cross-platform procedural desktop goose\n"
        " Honk300 is a playful desktop pet with terminal-first controls and\n"
        " platform-aware safety boundaries.\n",
    )
    return installed


def normalize_mtimes(root: Path, epoch: int) -> None:
    for path in sorted(root.rglob("*"), reverse=True):
        os.utime(path, (epoch, epoch), follow_symlinks=False)
    os.utime(root, (epoch, epoch), follow_symlinks=False)


def build_deb(
    binary: Path,
    output: Path,
    version: str,
    tag: str,
    commit: str,
    architecture: str,
    source_date_epoch: int,
) -> None:
    if output.exists() or output.is_symlink():
        raise ValueError(f"refusing to overwrite Debian package output: {output}")
    output.parent.mkdir(parents=True, exist_ok=True)
    with tempfile.TemporaryDirectory(prefix="honk300-deb-", dir=output.parent) as temporary:
        staging = Path(temporary) / "root"
        build_package_tree(staging, binary, version, tag, commit, architecture)
        normalize_mtimes(staging, source_date_epoch)
        environment = os.environ.copy()
        environment["SOURCE_DATE_EPOCH"] = str(source_date_epoch)
        subprocess.run(
            ["dpkg-deb", "--root-owner-group", "-Zxz", "--build", str(staging), str(output)],
            check=True,
            env=environment,
        )
    _regular_file(output, "Debian package")


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--binary", required=True, type=Path)
    parser.add_argument("--output", required=True, type=Path)
    parser.add_argument("--version", required=True)
    parser.add_argument("--tag", required=True)
    parser.add_argument("--commit", required=True)
    parser.add_argument("--architecture", choices=sorted(ARCHITECTURES), required=True)
    parser.add_argument("--source-date-epoch", type=int, required=True)
    arguments = parser.parse_args()
    try:
        build_deb(
            arguments.binary,
            arguments.output,
            arguments.version,
            arguments.tag,
            arguments.commit,
            arguments.architecture,
            arguments.source_date_epoch,
        )
    except (OSError, ValueError, subprocess.CalledProcessError) as error:
        parser.error(str(error))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
