#!/usr/bin/env python3
"""First real GNOME Shell companion and compatible overlay premise, CI only."""
import argparse
import json
import os
from pathlib import Path
import shutil
import subprocess
import time


def main():
    assert os.environ.get('GITHUB_ACTIONS') == 'true', 'Disposable CI only'
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--evidence', type=Path, required=True)
    parser.add_argument('--goose', type=Path, required=True)
    args = parser.parse_args()
    evidence = args.evidence.resolve()
    evidence.mkdir(parents=True, exist_ok=True)
    runtime = Path('/tmp') / f'gn-{os.getpid()}'
    runtime.mkdir(mode=0o700)
    environment = dict(os.environ, XDG_RUNTIME_DIR=str(runtime),
        XDG_CONFIG_HOME=str(evidence / 'config'), XDG_DATA_HOME=str(evidence / 'data'),
        XDG_CACHE_HOME=str(evidence / 'cache'), XDG_CURRENT_DESKTOP='GNOME',
        XDG_SESSION_TYPE='wayland', LIBGL_ALWAYS_SOFTWARE='1', GDK_BACKEND='wayland',
        MUTTER_DEBUG_DUMMY_MODE_SPECS='1280x900', HONK300_GNOME_PROBE_PID=str(os.getpid()))
    for key in ('XDG_CONFIG_HOME', 'XDG_DATA_HOME', 'XDG_CACHE_HOME'):
        Path(environment[key]).mkdir(mode=0o700)
    outer_display = environment['DISPLAY']
    for key in ('WAYLAND_DISPLAY', 'SWAYSOCK', 'HYPRLAND_INSTANCE_SIGNATURE'):
        environment.pop(key, None)
    uuid = 'honk300-probe@emmetts.dev'
    extension = Path(environment['XDG_DATA_HOME']) / 'gnome-shell' / 'extensions' / uuid
    shutil.copytree(Path(__file__).parent / 'fixtures' / 'gnome-probe', extension)
    subprocess.run(['gsettings', 'set', 'org.gnome.shell', 'enabled-extensions', f"['{uuid}']"],
                   env=environment, check=True, timeout=5)
    subprocess.run(['gsettings', 'set', 'org.gnome.shell', 'disable-user-extensions', 'false'],
                   env=environment, check=True, timeout=5)
    os.environ.update(environment)
    import gi
    gi.require_version('Gtk', '4.0')
    from gi.repository import Gio, GLib, Gtk
    bus = Gio.bus_get_sync(Gio.BusType.SESSION, None)
    windows = []
    goose = None
    with (evidence / 'shell.log').open('w') as shell_log:
        shell = subprocess.Popen(['gnome-shell', '--nested', '--wayland'],
                                  env=environment, stdout=shell_log, stderr=shell_log)
        try:
            def pump():
                context = GLib.MainContext.default()
                for _ in range(100):
                    if not context.pending():
                        break
                    context.iteration(False)

            def wait(check, description, timeout=30):
                deadline = time.monotonic() + timeout
                while time.monotonic() < deadline:
                    assert shell.poll() is None, 'GNOME Shell exited during ' + description
                    pump()
                    if value := check():
                        return value
                    time.sleep(0.03)
                raise AssertionError('Timed out waiting for ' + description)

            def call(method, parameters=None):
                reply = bus.call_sync('dev.emmetts.Honk300.GnomeProbe1',
                    '/dev/emmetts/Honk300/GnomeProbe1', 'dev.emmetts.Honk300.GnomeProbe1',
                    method, parameters, GLib.VariantType.new('(s)'), Gio.DBusCallFlags.NONE, 500, None)
                return reply.unpack()[0]

            def snapshot():
                return json.loads(call('Snapshot'))

            def owner():
                return bus.call_sync('org.freedesktop.DBus', '/org/freedesktop/DBus',
                    'org.freedesktop.DBus', 'NameHasOwner',
                    GLib.Variant('(s)', ('dev.emmetts.Honk300.GnomeProbe1',)),
                    GLib.VariantType.new('(b)'), Gio.DBusCallFlags.NONE, 500, None).unpack()[0]

            wait(owner, 'actual extension enable')
            initial = snapshot()
            assert initial['pid'] == shell.pid and initial['session_wayland'], initial
            assert initial['version'].split('.')[0] in ('46', '48'), initial
            (evidence / 'shell-initial.json').write_text(json.dumps(initial, indent=2) + '\n')
            environment['WAYLAND_DISPLAY'] = initial['wayland']
            os.environ['WAYLAND_DISPLAY'] = initial['wayland']
            # Record actual advertised globals; never assume the wlroots overlay
            # protocol exists on GNOME simply because another compositor has it.
            protocol = subprocess.run(['wayland-info'], env=environment, capture_output=True,
                                      text=True, timeout=8)
            assert protocol.returncode == 0, protocol.stderr
            (evidence / 'wayland-globals.txt').write_text(protocol.stdout)
            layer_shell = 'zwlr_layer_shell_v1' in protocol.stdout
            GLib.set_prgname('honk300-gnome-probe')
            Gtk.init()
            def find(title):
                return next((node for node in snapshot()['windows'] if node['title'] == title), None)

            for title in ('Honk300 ordinary GNOME probe', 'ChatGPT Codex terminal probe'):
                window = Gtk.Window(title=title)
                window.set_default_size(300, 200)
                window.set_child(Gtk.Label(label=title))
                window.present()
                windows.append(window)
            ordinary = wait(lambda: find(windows[0].get_title()), 'native ordinary window')
            protected = wait(lambda: find(windows[1].get_title()), 'native protected identity')
            for node in (ordinary, protected):
                assert node['pid'] == os.getpid() and node['app'] == 'honk300-gnome-probe', node
                assert node['showing'] and not node['minimized'], node
            assert call('MoveFixture', GLib.Variant('(tus)', (ordinary['id'], os.getpid(),
                json.dumps(ordinary['rect'], separators=(',', ':'))))) == 'ok'
            moved = wait(lambda: (node if (node := find(ordinary['title'])) and
                node['rect'][0] == ordinary['rect'][0] + 6 else None), 'actual bounded fixture move')
            assert find(protected['title'])['rect'] == protected['rect']
            try:
                call('MoveFixture', GLib.Variant('(tus)', (ordinary['id'], os.getpid(),
                    json.dumps(ordinary['rect'], separators=(',', ':')))))
            except GLib.Error:
                pass
            else:
                raise AssertionError('Stale geometry was accepted')
            windows[0].fullscreen()
            wait(lambda: find(ordinary['title'])['fullscreen'], 'native fullscreen observation')
            windows[0].unfullscreen()
            wait(lambda: not find(ordinary['title'])['fullscreen'], 'native fullscreen removal')
            environment['DISPLAY'] = snapshot()['display']
            assert environment['DISPLAY'] and environment['DISPLAY'] != outer_display, environment['DISPLAY']
            config = evidence / 'goose.toml'
            config.write_text('goose_config_version = 2\n[audio]\nenabled = false\n'
                              '[behavior]\nfirst_wander_time_seconds = 600.0\n'
                              '[safety]\nno_mouse_steal = true\nno_window_ride = true\n'
                              '[schedule]\nquiet_hours_enabled = false\nseasonal = false\n'
                              'autumn = false\n')
            with (evidence / 'goose.log').open('w') as log:
                goose = subprocess.Popen([str(args.goose.resolve()), 'start', '--config', str(config)],
                    env=environment, stdout=log, stderr=log)
                def ready():
                    assert goose.poll() is None, 'Actual goose exited before readiness'
                    result = subprocess.run([str(args.goose.resolve()), 'status'], env=environment,
                        capture_output=True, text=True, timeout=5)
                    if result.returncode == 0 and 'honk300: running' in result.stdout:
                        (evidence / 'goose-status.txt').write_text(result.stdout)
                        return True
                    return False
                wait(ready, 'actual XWayland goose overlay')
                for index in range(80):
                    pump()
                    time.sleep(0.05)
                subprocess.run(['import', '-display', outer_display, '-window', 'root',
                    str(evidence / 'native-goose.png')], env=environment, check=True, timeout=8)
                subprocess.run([str(args.goose.resolve()), 'stop'], env=environment,
                    check=True, capture_output=True, timeout=5)
                goose.wait(timeout=30)
                assert goose.returncode == 0
                goose = None
            windows[0].destroy()
            wait(lambda: find(ordinary['title']) is None, 'vanished native target')
            subprocess.run(['gnome-extensions', 'disable', uuid], env=environment, check=True, timeout=5)
            wait(lambda: not owner(), 'actual extension disable')
            assert windows[1].get_mapped(), 'Disabling the extension closed the unrelated window'
            result = dict(ok=True, version=initial['version'], architecture=os.uname().machine,
                shell_pid=shell.pid, peer_credentials=True, layer_shell=layer_shell,
                xwayland_overlay_ready=True, actual_fixture_move=True, stale_geometry_refused=True,
                native_fullscreen=True, extension_disable=True, initial=ordinary, moved=moved,
                protected=protected, user_drag_qualified=False, pointer_control_qualified=False)
            (evidence / 'result.json').write_text(json.dumps(result, indent=2) + '\n')
            print(json.dumps(result))
        finally:
            for window in windows:
                window.destroy()
            if goose is not None and goose.poll() is None:
                goose.kill()
                goose.wait(timeout=5)
            shell.terminate()
            try:
                shell.wait(timeout=5)
            except subprocess.TimeoutExpired:
                shell.kill()
                shell.wait(timeout=5)


if __name__ == '__main__':
    main()
