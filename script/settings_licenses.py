"""Preserve the license texts of the accessibility bridge's locked dependencies."""
from __future__ import annotations
import json
from pathlib import Path
import subprocess


def accessibility_notices(project: Path, target: str) -> str:
    if "windows" not in target and "linux" not in target:
        return ""
    metadata = json.loads(subprocess.check_output([
        "cargo", "metadata", "--locked", "--format-version", "1",
        "--manifest-path", str(project / "accessibility/Cargo.toml"),
        "--filter-platform", target,
    ], text=True))
    included = {node["id"] for node in metadata["resolve"]["nodes"]}
    notices = ["\n\nNative accessibility dependencies\n"]
    for package in sorted(metadata["packages"], key=lambda item: (item["name"], item["version"])):
        if package["id"] not in included:
            continue
        directory = Path(package["manifest_path"]).parent
        if package["source"] is None and directory.resolve() == (project / "accessibility").resolve():
            continue  # Only our bridge is first-party; vendored dependencies retain notices.
        files = sorted({p for pattern in ("LICENSE*", "LICENCE*", "COPYING*")
                        for p in directory.glob(pattern) if p.is_file()})
        if package.get("license_file"):
            files = sorted(set(files + [directory / package["license_file"]]))
        if not files and package["repository"] == "https://github.com/AccessKit/accesskit":
            files = sorted((project / "accessibility/licenses").glob("LICENSE*"))
        if not files:
            raise RuntimeError(f"Missing license text for {package['name']} {package['version']}")
        notices.append(f"\n{package['name']} {package['version']} ({package['license']})\n")
        for file in files:
            notices.append(f"\n{file.name}\n\n{file.read_text(encoding='utf-8')}\n")
    return "".join(notices)
