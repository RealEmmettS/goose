"""Build and validate honk300's immutable release metadata."""

from __future__ import annotations

import argparse
import hashlib
import json
import os
import re
import sys
import tempfile
from pathlib import Path


SCHEMA = "honk300.release.v1"
_STABLE_TAG = re.compile(r"^v(\d+)\.(\d+)\.(\d+)$")
_COMMIT = re.compile(r"^[0-9a-fA-F]{40}$")
_TOKEN = re.compile(r"__[A-Z0-9_]+__")

WINDOWS_TARGETS = ("x86_64-pc-windows-msvc", "aarch64-pc-windows-msvc")
MACOS_TARGETS = ("x86_64-apple-darwin", "aarch64-apple-darwin")
LINUX_TARGETS = (
    "x86_64-unknown-linux-gnu",
    "aarch64-unknown-linux-gnu",
    "x86_64-unknown-linux-musl",
    "aarch64-unknown-linux-musl",
)

# The release is atomic: these files must exist before any draft can be published. The
# compatibility entries preserve every v0.2.1 installer filename used by existing clients.
REQUIRED_RELEASE_ARTIFACTS = (
    "honk300-installer.sh",
    "honk300-installer.ps1",
    "honk300-universal2.app.zip",
    "honk300-universal2.dmg",
    *(f"honk300-{target}.tar.xz" for target in MACOS_TARGETS),
    *(f"honk300-{target}.tar.xz" for target in LINUX_TARGETS),
    *(f"honk300-{target}.zip" for target in WINDOWS_TARGETS),
    *(f"honk300-{target}.msi" for target in WINDOWS_TARGETS),
    *(f"honk300-{target}-corporate.msi" for target in WINDOWS_TARGETS),
    *(f"honk300-{target}-setup.exe" for target in WINDOWS_TARGETS),
    *(f"honk300-{target}-corporate-setup.exe" for target in WINDOWS_TARGETS),
)

LAYOUTS = {
    "windows": {
        "install_root": r"%ProgramFiles%\honk300",
        "aliases": ["honk300.exe", "honk.exe", "goose.exe"],
        "autostart_owner": "msi",
        "autostart_default": False,
    },
    "macos": {
        "install_root": "~/Applications/Honk300.app",
        "aliases": ["~/.local/bin/honk300", "~/.local/bin/honk", "~/.local/bin/goose"],
        "autostart_owner": "honk300-installer",
        "autostart_default": False,
    },
    "linux": {
        "install_root": "${XDG_DATA_HOME:-~/.local/share}/honk300/install",
        "aliases": ["~/.local/bin/honk300", "~/.local/bin/honk", "~/.local/bin/goose"],
        "autostart_owner": "honk300-installer",
        "autostart_default": False,
    },
}


def _sha256(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as source:
        for chunk in iter(lambda: source.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def _stable_version(tag: str) -> str:
    match = _STABLE_TAG.fullmatch(tag)
    if not match:
        raise ValueError(f"tag must be a stable vMAJOR.MINOR.PATCH value: {tag}")
    return ".".join(match.groups())


def _validated_commit(commit: str) -> str:
    if not _COMMIT.fullmatch(commit):
        raise ValueError("commit must be a full 40-character hexadecimal SHA")
    return commit.lower()


def _regular_file(directory: Path, name: str) -> Path:
    path = directory / name
    if path.is_symlink() or not path.is_file():
        raise ValueError(f"required release artifact is missing or not a regular file: {name}")
    return path


def _write_text_atomic(path: Path, content: str) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    descriptor, temporary = tempfile.mkstemp(prefix=f".{path.name}.", dir=path.parent)
    try:
        with os.fdopen(descriptor, "w", encoding="utf-8", newline="\n") as target:
            target.write(content)
            target.flush()
            os.fsync(target.fileno())
        os.replace(temporary, path)
    except BaseException:
        try:
            os.unlink(temporary)
        except FileNotFoundError:
            pass
        raise


def _classify(name: str) -> tuple[str, str]:
    exact = {
        "honk300-installer.sh": ("universal-unix", "bootstrap-shell"),
        "honk300-installer.ps1": ("windows", "bootstrap-powershell"),
        "honk300-universal2.app.zip": ("universal2-apple-darwin", "mac-app"),
        "honk300-universal2.dmg": ("universal2-apple-darwin", "compat-mac-dmg"),
    }
    if name in exact:
        return exact[name]

    windows = re.fullmatch(
        r"honk300-(x86_64|aarch64)-pc-windows-msvc"
        r"(?P<corporate>-corporate)?(?P<setup>-setup)?\.(?P<ext>msi|exe|zip)",
        name,
    )
    if windows:
        arch = windows.group(1)
        target = f"{arch}-pc-windows-msvc"
        ext = windows.group("ext")
        corporate = bool(windows.group("corporate"))
        if ext == "zip":
            return target, "portable"
        channel = "corporate" if corporate else "global"
        return target, f"{ext}-{channel}"

    archive = re.fullmatch(
        r"honk300-(?P<target>(?:x86_64|aarch64)-(?:unknown-linux-(?:gnu|musl)|apple-darwin))"
        r"\.tar\.(?:gz|xz)",
        name,
    )
    if archive:
        return archive.group("target"), "portable"

    raise ValueError(f"unknown release artifact: {name}")


def build_manifest(directory: str, tag: str, commit: str) -> dict:
    version = _stable_version(tag)
    commit = _validated_commit(commit)

    root = Path(directory)
    if not root.is_dir():
        raise ValueError(f"release directory does not exist: {root}")

    artifacts = []
    for path in sorted((item for item in root.iterdir() if item.is_file()), key=lambda p: p.name):
        name = path.name
        if (
            name.endswith(".sha256")
            or name.endswith("dist-manifest.json")
            or name in {"release-manifest.json", "sha256.sum"}
        ):
            continue
        target, kind = _classify(name)
        artifact_hash = _sha256(path)
        checksum = path.with_name(f"{name}.sha256")
        expected_checksum = f"{artifact_hash} *{name}\n"
        if checksum.is_symlink() or not checksum.is_file():
            raise ValueError(f"release artifact checksum is missing or unsafe: {checksum.name}")
        if checksum.read_text(encoding="utf-8") != expected_checksum:
            raise ValueError(f"release artifact checksum does not match: {checksum.name}")
        artifacts.append(
            {
                "name": name,
                "target": target,
                "kind": kind,
                "sha256": artifact_hash,
                "size": path.stat().st_size,
                "checksum": checksum.name,
            }
        )

    return {
        "schema": SCHEMA,
        "version": version,
        "tag": tag,
        "commit": commit,
        "layouts": LAYOUTS,
        "artifacts": artifacts,
    }


def render_template(template: str, replacements: dict[str, str]) -> str:
    rendered = template
    for key, value in replacements.items():
        token = f"__{key}__"
        if not re.fullmatch(r"__[A-Z0-9_]+__", token):
            raise ValueError(f"invalid replacement token: {key}")
        rendered = rendered.replace(token, value)
    unresolved = sorted(set(_TOKEN.findall(rendered)))
    if unresolved:
        raise ValueError(f"unresolved template tokens: {', '.join(unresolved)}")
    return rendered


def render_installers(
    directory: Path,
    tag: str,
    commit: str,
    shell_template: Path,
    powershell_template: Path,
) -> tuple[Path, Path]:
    root = Path(directory)
    version = _stable_version(tag)
    payloads = {
        "SHA_MAC_APP": "honk300-universal2.app.zip",
        "SHA_LINUX_X64_GNU": "honk300-x86_64-unknown-linux-gnu.tar.xz",
        "SHA_LINUX_ARM64_GNU": "honk300-aarch64-unknown-linux-gnu.tar.xz",
        "SHA_LINUX_X64_MUSL": "honk300-x86_64-unknown-linux-musl.tar.xz",
        "SHA_LINUX_ARM64_MUSL": "honk300-aarch64-unknown-linux-musl.tar.xz",
        "SHA_WINDOWS_X64_MSI": "honk300-x86_64-pc-windows-msvc.msi",
        "SHA_WINDOWS_ARM64_MSI": "honk300-aarch64-pc-windows-msvc.msi",
    }
    replacements = {"TAG": tag, "VERSION": version, "COMMIT": _validated_commit(commit)}
    replacements.update(
        {token: _sha256(_regular_file(root, name)) for token, name in payloads.items()}
    )

    shell = render_template(shell_template.read_text(encoding="utf-8"), replacements)
    powershell = render_template(
        powershell_template.read_text(encoding="utf-8"), replacements
    )
    shell_output = root / "honk300-installer.sh"
    powershell_output = root / "honk300-installer.ps1"
    _write_text_atomic(shell_output, shell)
    _write_text_atomic(powershell_output, powershell)
    if os.name != "nt":
        shell_output.chmod(0o755)
    return shell_output, powershell_output


def validate_required_artifacts(directory: Path) -> None:
    root = Path(directory)
    missing = [name for name in REQUIRED_RELEASE_ARTIFACTS if not (root / name).is_file()]
    unsafe = [name for name in REQUIRED_RELEASE_ARTIFACTS if (root / name).is_symlink()]
    if missing or unsafe:
        details = []
        if missing:
            details.append("missing: " + ", ".join(missing))
        if unsafe:
            details.append("symlinks: " + ", ".join(unsafe))
        raise ValueError("missing required release artifacts (" + "; ".join(details) + ")")


def write_sha256_sidecars(directory: Path) -> list[Path]:
    root = Path(directory)
    outputs = []
    for artifact in sorted(root.iterdir(), key=lambda path: path.name):
        if (
            not artifact.is_file()
            or artifact.is_symlink()
            or artifact.name.endswith(".sha256")
            or artifact.name == "sha256.sum"
            or artifact.name.endswith("-dist-manifest.json")
        ):
            continue
        sidecar = artifact.with_name(f"{artifact.name}.sha256")
        _write_text_atomic(sidecar, f"{_sha256(artifact)} *{artifact.name}\n")
        outputs.append(sidecar)
    return outputs


def _parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(description=__doc__)
    commands = parser.add_subparsers(dest="command", required=True)

    manifest = commands.add_parser("manifest", help="write release-manifest.json")
    manifest.add_argument("--directory", required=True, type=Path)
    manifest.add_argument("--tag", required=True)
    manifest.add_argument("--commit", required=True)
    manifest.add_argument("--output", required=True, type=Path)

    render = commands.add_parser("render-installers", help="render version-stamped bootstraps")
    render.add_argument("--directory", required=True, type=Path)
    render.add_argument("--tag", required=True)
    render.add_argument("--commit", required=True)
    render.add_argument(
        "--shell-template", type=Path, default=Path(__file__).with_name("honk300-installer.sh.in")
    )
    render.add_argument(
        "--powershell-template",
        type=Path,
        default=Path(__file__).with_name("honk300-installer.ps1.in"),
    )

    validate = commands.add_parser("validate", help="validate the atomic release asset set")
    validate.add_argument("--directory", required=True, type=Path)

    sidecars = commands.add_parser("sidecars", help="write SHA-256 sidecars")
    sidecars.add_argument("--directory", required=True, type=Path)
    return parser


def main(argv: list[str] | None = None) -> int:
    parser = _parser()
    arguments = parser.parse_args(argv)
    try:
        if arguments.command == "manifest":
            manifest = build_manifest(
                str(arguments.directory), arguments.tag, arguments.commit
            )
            _write_text_atomic(
                arguments.output, json.dumps(manifest, indent=2, sort_keys=True) + "\n"
            )
        elif arguments.command == "render-installers":
            render_installers(
                arguments.directory,
                arguments.tag,
                arguments.commit,
                arguments.shell_template,
                arguments.powershell_template,
            )
        elif arguments.command == "validate":
            validate_required_artifacts(arguments.directory)
        elif arguments.command == "sidecars":
            write_sha256_sidecars(arguments.directory)
        else:  # argparse enforces this, but keeping the branch makes future edits fail closed.
            raise ValueError(f"unsupported command: {arguments.command}")
    except (OSError, ValueError) as error:
        print(f"release_metadata: {error}", file=sys.stderr)
        return 2
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
