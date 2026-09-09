#!/usr/bin/env python3
"""Resolve the installed Goose launcher and running-window icon through native GIO/GTK."""

import argparse
import hashlib
import json
from pathlib import Path

import gi

gi.require_version("Gtk", "4.0")
from gi.repository import Gdk, Gio, Gtk  # noqa: E402


APPLICATION_ID = "dev.emmetts.honk300.settings"


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--expected-icon", type=Path, required=True)
    parser.add_argument("--expected-binary", type=Path, required=True)
    parser.add_argument("--evidence", type=Path, required=True)
    args = parser.parse_args()

    Gtk.init()
    display = Gdk.Display.get_default()
    assert display is not None, "native GTK display is required"
    launcher = Gio.DesktopAppInfo.new("honk300.desktop")
    identity = Gio.DesktopAppInfo.new(f"{APPLICATION_ID}.desktop")
    assert launcher is not None, "installed launcher is not discoverable through GIO"
    assert identity is not None, "running-window identity is not discoverable through GIO"
    assert launcher.should_show(), "Goose launcher must remain visible"
    assert identity.get_nodisplay() and not identity.should_show(), "identity must not duplicate the menu entry"
    for entry in (launcher, identity):
        assert entry.get_name() == "Goose", entry.get_name()
        assert entry.get_startup_wm_class() == APPLICATION_ID
        assert Path(entry.get_executable()).resolve() == args.expected_binary.resolve()
        assert entry.get_commandline().endswith(" settings"), entry.get_commandline()
    assert launcher.get_icon().equal(identity.get_icon())

    theme = Gtk.IconTheme.get_for_display(display)
    assert theme.has_icon(APPLICATION_ID), "GTK cannot find the running-window icon"
    paintable = theme.lookup_icon(APPLICATION_ID, [], 32, 1, Gtk.TextDirection.NONE, 0)
    icon_file = paintable.get_file()
    assert icon_file is not None and icon_file.get_path(), "GTK returned a missing or non-file icon"
    resolved_icon = Path(icon_file.get_path())
    expected_hash = hashlib.sha256(args.expected_icon.read_bytes()).hexdigest()
    actual_hash = hashlib.sha256(resolved_icon.read_bytes()).hexdigest()
    assert actual_hash == expected_hash, f"GTK resolved unexpected artwork: {resolved_icon}"
    evidence = {
        "application_id": APPLICATION_ID,
        "launcher": launcher.get_filename(),
        "identity": identity.get_filename(),
        "command": launcher.get_commandline(),
        "one_visible_entry": True,
        "gtk_icon": str(resolved_icon),
        "icon_sha256": actual_hash,
        "gtk_version": f"{Gtk.get_major_version()}.{Gtk.get_minor_version()}.{Gtk.get_micro_version()}",
    }
    args.evidence.parent.mkdir(parents=True, exist_ok=True)
    args.evidence.write_text(json.dumps(evidence, indent=2) + "\n", encoding="utf-8")
    print(json.dumps(evidence, indent=2))


if __name__ == "__main__":
    main()
