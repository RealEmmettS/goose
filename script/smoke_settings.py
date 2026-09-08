#!/usr/bin/env python3
"""Exercise the real native settings window against an isolated TOML file.

Requires an automation-enabled settings binary beside the exact Rust service.
Screenshots are retained-scene evidence, not an OS compositor qualification.
"""
from __future__ import annotations

import argparse
import json
import os
from pathlib import Path
import re
import shutil
import subprocess
import tempfile
import time
try:
    import tomllib
except ModuleNotFoundError:
    import tomli as tomllib


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--binary", type=Path, required=True)
    parser.add_argument("--evidence", type=Path, required=True)
    parser.add_argument("--network", action="store_true")
    parser.add_argument("--lifecycle", action="store_true", help="start/stop on a disposable CI desktop")
    args = parser.parse_args()
    root = Path(__file__).resolve().parents[1]
    binary = args.binary.resolve(strict=True)
    service = binary.with_name("honk300.exe" if os.name == "nt" else "honk300")
    assert service.is_file(), "settings must use its exact Rust sibling"
    cli = root / "settings/node_modules/@native-sdk/cli/bin/native.js"
    evidence = args.evidence.resolve()
    evidence.mkdir(parents=True, exist_ok=True)
    hidden = subprocess.CREATE_NO_WINDOW if os.name == "nt" else 0
    if args.lifecycle and os.environ.get("GITHUB_ACTIONS") != "true":
        raise RuntimeError("Lifecycle fixture runs only on a disposable GitHub desktop")
    environment = dict(os.environ)
    if args.lifecycle and os.name != "nt" and os.uname().sysname == "Linux":
        # These GUI fixtures use Xvfb without a compositing manager. The separate
        # Linux overlay gate owns pixels; this probe owns the GUI/control handoff.
        environment["HONK300_ALLOW_HEADLESS"] = "1"
    # Unique cwd isolates the SDK's file protocol from other development windows.
    with tempfile.TemporaryDirectory(prefix="honk300-settings-") as temporary:
        work = Path(temporary)
        config = work / "config.toml"
        config.write_text("# Preserved by the real shared editor\ngoose_config_version = 2\n", encoding="utf-8")
        process = subprocess.Popen([str(binary), "--config", str(config)], cwd=work,
                                   stdin=subprocess.DEVNULL, stdout=subprocess.DEVNULL,
                                   stderr=subprocess.DEVNULL, env=environment)
        snapshot_path = work / ".zig-cache/native-sdk-automation/snapshot.txt"
        started_runtime = False
        lifecycle_cycles = 0

        def control(*arguments: str):
            return subprocess.run([str(service), *arguments], capture_output=True, text=True,
                                  timeout=20, creationflags=hidden)

        def snapshot() -> str:
            # The pinned SDK truncates and rewrites this file each frame. Its
            # command-file deletion acknowledges consumption, before dispatch
            # and the next layout snapshot. Reject in-flight file reads; callers
            # below wait for the actual semantic result of each input.
            try:
                before = snapshot_path.stat()
                current = snapshot_path.read_text(encoding="utf-8")
                after = snapshot_path.stat()
            except (FileNotFoundError, PermissionError):
                return ""
            if (before.st_mtime_ns, before.st_size) != (after.st_mtime_ns, after.st_size):
                return ""
            if not re.match(r'ready=true [^\n]+\n', current) or not current.endswith("\n"):
                return ""
            return current

        def wait(pattern: str, timeout: int = 30) -> str:
            deadline = time.monotonic() + timeout
            while time.monotonic() < deadline:
                assert process.poll() is None, "settings process exited"
                current = snapshot()
                errors = re.match(r'ready=true .*? dispatch_errors=(\d+) ', current)
                assert not errors or errors[1] == "0", current[-6000:]
                if re.search(pattern, current):
                    return current
                time.sleep(0.1)
            raise AssertionError(f"settings did not show {pattern!r}\n{snapshot()[-6000:]}")

        def automate(*arguments: str) -> None:
            subprocess.run(["node", str(cli), "automate", *arguments], cwd=work,
                           check=True, capture_output=True, text=True, timeout=40,
                           creationflags=hidden)

        def widget(label: str, role: str = "button") -> str:
            pattern = (r'widget @w1/main-canvas#(\d+) role=' + role +
                       ' name="' + re.escape(label) + r'" .*? enabled=true')
            match = re.search(pattern, wait(pattern))
            assert match
            return match[1]

        def click(label: str, role: str = "button") -> None:
            automate("widget-click", "main-canvas", widget(label, role))

        def capture(name: str, scale: str = "1") -> None:
            automate("screenshot", "main-canvas", scale)
            shutil.copyfile(work / ".zig-cache/native-sdk-automation/screenshot-main-canvas.png",
                            evidence / f"{name}.png")
            (evidence / f"{name}.txt").write_text(snapshot(), encoding="utf-8")

        try:
            wait('name="Ready.')
            wait(f"publisher_pid={process.pid} ")
            for page in ("General", "Appearance", "Behavior", "Sound & manners", "Platform & status"):
                click(page)
                wait('role=text name="' + re.escape(page) + '"')
                capture(page.split()[0].lower())
            click("Appearance")
            click("Expressive reactions", "switch")
            click("Save & apply")
            wait(r'role=text name="Saved\.')
            assert tomllib.loads(config.read_text(encoding="utf-8"))["appearance"]["expressions"] is False
            assert config.read_text(encoding="utf-8").startswith("# Preserved")
            click("Reduced motion", "switch")
            with config.open("a", encoding="utf-8") as file:
                file.write("\n# A concurrent editor changed this file\n")
            concurrent = config.read_bytes()
            click("Save & apply")
            wait("changed in another editor")
            assert config.read_bytes() == concurrent
            wait('name="Unsaved changes"')
            capture("conflict")
            click("Reload saved settings")
            click("Keep editing")
            wait('name="Unsaved changes"')
            click("Reload saved settings")
            capture("discard-dialog")
            click("Discard and reload")
            wait('name="Ready.')
            click("General")
            click("First wander (seconds)")
            automate("widget-action", "main-canvas", widget("First wander (seconds)", "textbox"), "set_text", "-1")
            capture("edit-dialog", "2")
            # Keyboard confirmation exercises the same focused text-input path.
            automate("widget-key", "main-canvas", "enter")
            wait('name="Changed"')
            click("Save & apply")
            wait("must be")
            assert config.read_bytes() == concurrent
            wait('name="Unsaved changes"')
            click("Reload saved settings")
            click("Discard and reload")
            wait('name="Ready.')
            click("Behavior")
            for label in ("Honk on the hour", "Travel across monitors", "Allow cursor nabs",
                          "Random cursor nabs", "Prevent all cursor nabs", "Prevent window rides",
                          "Ride supported windows", "Bring notes and memes"):
                if label == "Bring notes and memes":
                    # macOS ARM runners expose a shorter viewport. Native actions
                    # still reject fully clipped controls, so scroll the real pane.
                    scroll_pattern = r'widget @w1/main-canvas#(\d+) .*? scroll=\[offset=([^,]+),'
                    scroll = re.search(scroll_pattern, wait(scroll_pattern))
                    assert scroll, "missing settings scroll area"
                    automate("widget-action", "main-canvas", scroll[1], "increment")
                    wait(r'#' + scroll[1] + r' .*? scroll=\[offset=(?!' + re.escape(scroll[2]) + r',)[^,]+,')
                identifier = widget(label, "switch")
                value_pattern = r'#' + identifier + r' role=switch .*? value=([01]) '
                before = re.search(value_pattern, wait(value_pattern))
                assert before, f"missing switch value: {label}"
                automate("widget-action", "main-canvas", identifier, "toggle")
                wait(r'#' + identifier + r' role=switch .*? value=' + str(1 - int(before[1])) + ' ')
            capture("largest-dirty-page")
            click("Save & apply")
            wait(r'role=text name="Saved\.')
            saved = tomllib.loads(config.read_text(encoding="utf-8"))
            for section, key, expected in (
                ("behaviors", "on_hour_double_honk", False),
                ("behaviors", "multi_monitor_chase", False),
                ("behavior", "can_attack_mouse", False),
                ("behavior", "attack_randomly", True),
                ("safety", "no_mouse_steal", True),
                ("safety", "no_window_ride", True),
                ("mischief", "perch_and_ride", False),
                ("mischief", "collect_windows", False),
            ):
                assert saved[section][key] is expected, f"saved toggle mismatch: {section}.{key}"
            if args.lifecycle:
                assert "honk300: not running" in control("status").stdout, "fixture found an existing runtime"
                click("General")
                for _ in range(3):
                    started_runtime = True
                    click("Start goose")
                    wait('role=text name="start ready"')
                    assert "honk300: running" in control("status").stdout
                    click("Stop goose")
                    wait('role=text name="Goose stopped.')
                    assert "honk300: not running" in control("status").stdout
                    started_runtime = False
                    lifecycle_cycles += 1
            if args.network:
                click("Platform & status")
                click("Check for updates")
                wait("Latest release:", 40)
                capture("update-discovery")
            wait(r'^ready=true .*? dispatch_errors=0 ')
            (evidence / "result.json").write_text(json.dumps({
                "ok": True, "binary": str(binary), "network": args.network,
                "lifecycle": args.lifecycle,
                "lifecycle_cycles": lifecycle_cycles,
                "checks": ["five-pages", "save-readback", "comment-preservation", "revision-conflict",
                           "keep-draft", "discard-reload", "numeric-validation", "keyboard-input",
                           "modal-layout", "scale-two", "largest-dirty-page", "no-dispatch-errors"],
                "pixels": "Native SDK retained-scene renderer"
            }, indent=2) + "\n", encoding="utf-8")
            print("Native settings smoke passed; evidence:", evidence)
        finally:
            if started_runtime:
                try:
                    status = control("status")
                    observed = dict(returncode=status.returncode,
                                    stdout=status.stdout, stderr=status.stderr)
                except (OSError, subprocess.TimeoutExpired) as error:
                    observed = dict(error=str(error))
                (evidence / "status-before-cleanup.json").write_text(
                    json.dumps(observed, indent=2) + "\n", encoding="utf-8")
                control("stop", "--force")
            if snapshot_path.exists():
                (evidence / "final-snapshot.txt").write_text(snapshot(), encoding="utf-8")
            if process.poll() is None:
                process.terminate()
                process.wait(timeout=10)


if __name__ == "__main__":
    main()
