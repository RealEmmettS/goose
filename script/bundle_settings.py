#!/usr/bin/env python3
"""Add the verified settings companion before sealing a new portable archive.

This runs only in the candidate/release producer, before any artifact is uploaded.
Original Rust payload bytes and paths are preserved. Cargo-dist 0.31.0's artifact
assets and checksum map are refreshed to describe the completed archive.
"""
from __future__ import annotations

import argparse
import hashlib
from io import BytesIO
import json
from pathlib import Path, PurePosixPath
import stat
import tarfile
import tempfile
import zipfile

from verify_settings import verify


def add_files(archive: Path, additions: dict[str, bytes]) -> None:
    with tempfile.TemporaryDirectory(prefix="honk300-bundle-", dir=archive.parent) as temp:
        output = Path(temp) / archive.name
        if archive.suffix == ".zip":
            with zipfile.ZipFile(archive) as source, zipfile.ZipFile(output, "w", compression=zipfile.ZIP_DEFLATED) as destination:
                names = [info.orig_filename for info in source.infolist()]
                validate_names(names, additions)
                for info in source.infolist():
                    assert not stat.S_ISLNK(info.external_attr >> 16), "unexpected archive symlink"
                    destination.writestr(info, source.read(info))
                for name, data in additions.items():
                    info = zipfile.ZipInfo(name, date_time=(2020, 1, 1, 0, 0, 0))
                    info.external_attr = (stat.S_IFREG | (0o755 if "honk300-settings" in name else 0o644)) << 16
                    info.compress_type = zipfile.ZIP_DEFLATED
                    destination.writestr(info, data)
        else:
            with tarfile.open(archive, "r:xz") as source, tarfile.open(output, "w:xz") as destination:
                validate_names(source.getnames(), additions)
                for info in source.getmembers():
                    assert info.isfile() or info.isdir(), "unexpected non-regular tar member"
                    destination.addfile(info, source.extractfile(info) if info.isfile() else None)
                for name, data in additions.items():
                    info = tarfile.TarInfo(name)
                    info.size = len(data)
                    info.mode = 0o755 if "honk300-settings" in name else 0o644
                    info.mtime = 1577836800
                    destination.addfile(info, BytesIO(data))
        output.replace(archive)


def validate_names(names: list[str], additions: dict[str, bytes]) -> None:
    assert len(names) == len(set(names)), "duplicate archive members"
    assert not set(names).intersection(additions), "archive already contains a settings payload"
    for name in [*names, *additions]:
        path = PurePosixPath(name)
        assert name and "\0" not in name and not path.is_absolute() and ".." not in path.parts and "\\" not in name and ":" not in name, "unsafe archive member"


def bundle(archive: Path, directory: Path, target: str, manifest_path: Path) -> None:
    binary = verify(directory, target)
    assert archive.is_file() and not archive.is_symlink()
    expected_runtime = "honk300.exe" if "windows" in target else "honk300"
    if archive.suffix == ".zip":
        with zipfile.ZipFile(archive) as source:
            names = source.namelist()
    else:
        with tarfile.open(archive) as source:
            names = source.getnames()
    runtimes = [PurePosixPath(name) for name in names if PurePosixPath(name).name == expected_runtime]
    assert len(runtimes) == 1, "archive must contain exactly one Rust executable"
    parent = runtimes[0].parent
    files = [binary.name, "settings-build.json", "NATIVE_SDK_LICENSE.txt", "NATIVE_SDK_FONT_LICENSE.txt"]
    additions = {str(parent / name): (directory / name).read_bytes() for name in files}
    manifest = json.loads(manifest_path.read_text(encoding="utf-8"))
    assert manifest["dist_version"] == "0.31.0", "cargo-dist schema version drift"
    artifact = manifest["artifacts"][archive.name]
    assert artifact["kind"] == "executable-zip" and artifact["target_triples"] == [target]
    add_files(archive, additions)
    artifact["assets"].extend({"name": name, "path": name,
                                "kind": "executable" if name == binary.name else "license" if "LICENSE" in name else "unknown"}
                               for name in files)
    algorithms = set(artifact.get("checksums", {})) | {"sha256"}
    payload = archive.read_bytes()
    artifact["checksums"] = {algorithm: hashlib.new(algorithm, payload).hexdigest() for algorithm in algorithms}
    checksum = archive.with_name(artifact["checksum"])
    checksum.write_text(f"{artifact['checksums']['sha256']} *{archive.name}\n", encoding="utf-8")
    checksum_artifact = manifest["artifacts"][artifact["checksum"]]
    for algorithm in checksum_artifact.get("checksums", {}):
        checksum_artifact["checksums"][algorithm] = hashlib.new(algorithm, checksum.read_bytes()).hexdigest()
    manifest_path.write_text(json.dumps(manifest, indent=2) + "\n", encoding="utf-8")
    print(f"Completed portable archive with native settings: {archive}")


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--archive", type=Path, required=True)
    parser.add_argument("--directory", type=Path, required=True)
    parser.add_argument("--target", required=True)
    parser.add_argument("--manifest", type=Path, required=True)
    args = parser.parse_args()
    bundle(args.archive, args.directory, args.target, args.manifest)
