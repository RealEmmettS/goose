#!/usr/bin/env python3
"""Build the pinned native settings companion for one declared release target."""
from __future__ import annotations

import argparse
import hashlib
import json
import os
from pathlib import Path
import shutil
import subprocess
try:
    import tomllib
except ModuleNotFoundError:  # Ubuntu 22.04's supported system Python.
    import tomli as tomllib
from settings_toolchain import ensure_zig
from settings_licenses import accessibility_notices

TARGETS = {
    "x86_64-pc-windows-msvc": "x86_64-windows",
    "aarch64-pc-windows-msvc": "aarch64-windows",
    "x86_64-apple-darwin": "x86_64-macos",
    "aarch64-apple-darwin": "aarch64-macos",
    "x86_64-unknown-linux-gnu": "x86_64-linux-gnu",
    "aarch64-unknown-linux-gnu": "aarch64-linux-gnu",
    "x86_64-unknown-linux-musl": "x86_64-linux-musl",
    "aarch64-unknown-linux-musl": "aarch64-linux-musl",
}


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--target", choices=TARGETS, required=True)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--automation", action="store_true")
    parser.add_argument("--skip-install", action="store_true")
    parser.add_argument("--test", action="store_true", help="execute on a matching native host")
    args = parser.parse_args()
    root = Path(__file__).resolve().parents[1]
    project = root / "settings"
    os.environ["NATIVE_SDK_ZIG"] = ensure_zig(root)
    version = tomllib.loads((root / "Cargo.toml").read_text(encoding="utf-8"))["package"]["version"]
    manifest = (project / "app.zon").read_text(encoding="utf-8")
    source = (project / "src/main.zig").read_text(encoding="utf-8")
    assert f'.version = "{version}"' in manifest, "settings manifest version mismatch"
    assert f'pub const version = "{version}";' in source, "settings binary version mismatch"
    assert json.loads((project / "package.json").read_text())["version"] == version
    npm = shutil.which("npm.cmd" if os.name == "nt" else "npm")
    assert npm, "Node.js and npm are required"

    def run(*command: str) -> None:
        subprocess.run(command, cwd=project, check=True)

    if not args.skip_install:
        run(npm, "ci", "--ignore-scripts", "--no-audit", "--no-fund")
    run(npm, "run", "prepare:sdk")
    if args.test:
        if "windows" in args.target or "linux" in args.target:
            run("cargo", "test", "--locked", "--manifest-path", "accessibility/Cargo.toml",
                "--target", args.target)
        run(npm, "test")
        run(npm, "run", "check")
    run(npm, "run", "build", "--", "-Dtarget=" + TARGETS[args.target],
        "-Dautomation=" + str(args.automation).lower())
    name = "honk300-settings" + (".exe" if "windows" in args.target else "")
    built = project / "zig-out/bin" / name
    assert built.is_file() and not built.is_symlink(), "missing regular settings binary"
    output = args.output.resolve()
    output.mkdir(parents=True, exist_ok=True)
    installed = output / name
    if built.resolve() != installed:
        shutil.copyfile(built, installed)
    installed.chmod(0o755)
    accessibility = None
    if "windows" in args.target:
        bridge_name = "honk_settings_accessibility.dll"
        bridge = project / "zig-out/bin" / bridge_name
        assert bridge.is_file() and not bridge.is_symlink(), "missing native accessibility DLL"
        if bridge.resolve() != (output / bridge_name).resolve():
            shutil.copyfile(bridge, output / bridge_name)
        accessibility = {"name": bridge_name, "size": bridge.stat().st_size,
                         "sha256": hashlib.sha256(bridge.read_bytes()).hexdigest()}
    # The SDK and bundled typefaces retain their third-party license notices.
    license_source = project / "node_modules/@native-sdk/cli/LICENSE"
    (output / "NATIVE_SDK_LICENSE.txt").write_text(
        license_source.read_text(encoding="utf-8") + accessibility_notices(project, args.target),
        encoding="utf-8")
    (output / "NATIVE_SDK_FONT_LICENSE.txt").write_text(
        (project / "node_modules/@native-sdk/cli/src/runtime/testdata/fonts/OFL.txt").read_text(encoding="utf-8")
        + "\n\nAdditional application typefaces\n\n"
        + (project / "src/fonts/README.md").read_text(encoding="utf-8"), encoding="utf-8")
    (output / "settings-build.json").write_text(json.dumps({
        "schema": "honk300.settings-build.v1", "version": version,
        "target": args.target, "native_sdk": "0.5.4", "zig": "0.16.0",
        "automation": args.automation, "name": name,
        "accessibility": accessibility,
        "size": installed.stat().st_size,
        "sha256": hashlib.sha256(installed.read_bytes()).hexdigest()
    }, indent=2) + "\n", encoding="utf-8")
    print(f"Built {args.target}: {installed}")


if __name__ == "__main__":
    main()
