#!/usr/bin/env python3
"""Exercise the real AT-SPI provider on a disposable Xvfb/D-Bus desktop."""
from __future__ import annotations

import argparse
import json
from pathlib import Path
import subprocess
import time


def main() -> None:
    import gi
    gi.require_version("Atspi", "2.0")
    from gi.repository import Atspi, GLib

    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--binary", type=Path, required=True)
    parser.add_argument("--evidence", type=Path, required=True)
    args = parser.parse_args()
    evidence = args.evidence.resolve()
    evidence.mkdir(parents=True, exist_ok=True)
    config = evidence / "config.toml"
    if config.exists():
        raise RuntimeError("Accessibility fixture requires a fresh isolated config")
    config.write_text("# Native AT-SPI fixture\ngoose_config_version = 2\n", encoding="utf-8")
    subprocess.run([
        "gdbus", "call", "--session", "--dest", "org.a11y.Bus",
        "--object-path", "/org/a11y/bus", "--method", "org.freedesktop.DBus.Properties.Set",
        "org.a11y.Status", "IsEnabled", "<true>",
    ], check=True)
    Atspi.init()
    context = GLib.MainContext.default()
    log = (evidence / "process.log").open("w", encoding="utf-8")
    process = subprocess.Popen([str(args.binary.resolve()), "--config", str(config)], stdout=log, stderr=log)

    def tree():
        desktop = Atspi.get_desktop(0)
        desktop.clear_cache()
        stack = []
        for index in range(desktop.get_child_count()):
            app = desktop.get_child_at_index(index)
            if app and app.get_process_id() == process.pid:
                stack.append(app)
        found = []
        while stack and len(found) < 640:
            node = stack.pop()
            try:
                node.clear_cache()
                found.append(node)
                stack.extend(node.get_child_at_index(i) for i in range(node.get_child_count()))
                stack = [child for child in stack if child is not None]
            except GLib.Error:
                continue  # A replaced node is retried through the next root snapshot.
        return found

    def find(name, role=None):
        return next((node for node in tree() if node.get_name() == name
                     and (role is None or node.get_role() == role)), None)

    def wait(check, description):
        deadline = time.monotonic() + 20
        while time.monotonic() < deadline:
            if process.poll() is not None:
                raise RuntimeError(f"Settings exited while waiting for {description}: {process.returncode}")
            for _ in range(100):
                if not context.pending():
                    break
                context.iteration(False)
            result = check()
            if result:
                return result
            time.sleep(0.1)
        raise RuntimeError(f"AT-SPI did not provide {description}")

    def invoke(name):
        node = wait(lambda: find(name, Atspi.Role.PUSH_BUTTON), name)
        action = node.get_action_iface()
        if action is None or action.get_n_actions() == 0 or not action.do_action(0):
            raise RuntimeError(f"AT-SPI action failed: {name}")

    try:
        wait(lambda: find("First wander (seconds)"), "loaded named controls")
        invoke("Appearance")
        toggle = wait(lambda: find("Reduced motion", Atspi.Role.TOGGLE_BUTTON), "reduced motion switch")
        if toggle.get_state_set().contains(Atspi.StateType.PRESSED):
            raise RuntimeError("Fixture switch did not start off")
        if not toggle.get_action_iface().do_action(0):
            raise RuntimeError("AT-SPI could not toggle the switch")
        wait(lambda: find("Reduced motion", Atspi.Role.TOGGLE_BUTTON).get_state_set().contains(Atspi.StateType.PRESSED), "changed switch state")
        invoke("Save & apply")
        wait(lambda: "reduced_motion = true" in config.read_text(), "persisted switch")
        invoke("General")
        invoke("First wander (seconds)")
        wait(lambda: find("Edit setting", Atspi.Role.DIALOG), "editor dialog")
        if find("General", Atspi.Role.PUSH_BUTTON):
            raise RuntimeError("Modal exposed background page controls")
        field = wait(lambda: find("First wander (seconds)", Atspi.Role.ENTRY), "named entry")
        if field.get_text_iface().get_text(0, -1) != "20":
            raise RuntimeError("AT-SPI cannot read the initial field value")
        if not field.get_component_iface().grab_focus():
            raise RuntimeError("AT-SPI cannot focus the field")
        # The Unix AccessKit adapter exposes readable text and normal keyboard
        # editing; it does not implement the optional EditableText interface.
        windows = subprocess.check_output([
            "xdotool", "search", "--onlyvisible", "--pid", str(process.pid),
            "--name", "Honk300 settings",
        ], text=True).splitlines()
        if len(windows) != 1:
            raise RuntimeError("Could not identify the single fixture window")
        subprocess.run(["xdotool", "windowfocus", "--sync", windows[0]], check=True)
        focused_pid = subprocess.check_output(["xdotool", "getwindowfocus", "getwindowpid"], text=True).strip()
        if focused_pid != str(process.pid):
            raise RuntimeError("Keyboard focus left the fixture process")
        subprocess.run(["xdotool", "key", "--clearmodifiers", "ctrl+a"], check=True)
        subprocess.run(["xdotool", "type", "--clearmodifiers", "25"], check=True)
        wait(lambda: find("First wander (seconds)", Atspi.Role.ENTRY).get_text_iface().get_text(0, -1) == "25", "changed readable field text")
        invoke("Apply to draft")
        wait(lambda: find("General", Atspi.Role.PUSH_BUTTON), "dismissed modal")
        invoke("Save & apply")
        wait(lambda: "first_wander_time_seconds = 25" in config.read_text(), "saved editor value")
        wait(lambda: find("Saved. The goose will use these settings when it starts."), "readable saved status")
        # The largest page grows beyond the former 128-node ceiling as dirty
        # badges appear. Exercise its complete native action tree before Save.
        invoke("Behavior")
        for name in ("Honk on the hour", "Travel across monitors", "Allow cursor nabs",
                     "Random cursor nabs", "Prevent all cursor nabs", "Prevent window rides",
                     "Ride supported windows", "Bring notes and memes"):
            node = wait(lambda: find(name, Atspi.Role.TOGGLE_BUTTON), name)
            was_on = node.get_state_set().contains(Atspi.StateType.PRESSED)
            if not node.get_action_iface().do_action(0):
                raise RuntimeError(f"AT-SPI could not change {name}")
            wait(lambda: find(name, Atspi.Role.TOGGLE_BUTTON).get_state_set().contains(Atspi.StateType.PRESSED) != was_on, f"changed {name}")
        invoke("Save & apply")
        wait(lambda: "no_mouse_steal = true" in config.read_text() and "can_attack_mouse = false" in config.read_text(), "saved full-page changes")
        invoke("Platform & status")
        wait(lambda: any("Goose: stopped" in node.get_name()
                         and "Fullscreen observation: unprobed" in node.get_name()
                         and "Do not disturb: unprobed" in node.get_name() for node in tree()),
             "independent native fullscreen and do-not-disturb status")
        disabled = wait(lambda: find("Update now", Atspi.Role.PUSH_BUTTON), "disabled update button")
        if disabled.get_state_set().contains(Atspi.StateType.ENABLED) or disabled.get_state_set().contains(Atspi.StateType.SENSITIVE):
            raise RuntimeError("Disabled Update now was reported as enabled or sensitive")
        action = disabled.get_action_iface()
        if action is not None and action.get_n_actions() != 0:
            raise RuntimeError("Disabled Update now exposed an action")
        (evidence / "result.json").write_text(json.dumps({
            "schema": "honk300.settings-atspi-smoke.v1", "ok": True,
            "checks": ["native-names", "action", "toggle-state", "text-interface", "focus", "keyboard-edit", "modal-isolation", "save-readback", "largest-dirty-page", "disabled-button-state", "independent-presence-status"],
        }, indent=2) + "\n", encoding="utf-8")
    except Exception:
        (evidence / "failed-tree.json").write_text(json.dumps([
            {"name": node.get_name(), "role": node.get_role_name(),
             "interfaces": list(node.get_interfaces())} for node in tree()
        ], indent=2) + "\n", encoding="utf-8")
        raise
    finally:
        if process.poll() is None:
            process.terminate()
            try:
                process.wait(timeout=10)
            except subprocess.TimeoutExpired:
                process.kill()
                process.wait()
        log.close()
        Atspi.exit()


if __name__ == "__main__":
    main()
