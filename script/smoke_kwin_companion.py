#!/usr/bin/env python3
"""Earliest real KWin script premise test, confined to a disposable CI compositor."""
from __future__ import annotations
import argparse
import json
import os
from pathlib import Path
import subprocess
import time


def main():
    if os.environ.get('GITHUB_ACTIONS') != 'true':
        raise RuntimeError('Use only a disposable GitHub runner')
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--evidence', type=Path, required=True)
    args = parser.parse_args()
    evidence = args.evidence.resolve()
    evidence.mkdir(parents=True, exist_ok=True)
    runtime = evidence / 'runtime'
    runtime.mkdir(mode=0o700)
    config = evidence / 'config'
    config.mkdir()
    environment = dict(os.environ, XDG_RUNTIME_DIR=str(runtime), XDG_CONFIG_HOME=str(config),
        XDG_CURRENT_DESKTOP='KDE', XDG_SESSION_TYPE='wayland', QT_QPA_PLATFORM='offscreen',
        KWIN_COMPOSE='Q', LIBGL_ALWAYS_SOFTWARE='true', GDK_BACKEND='wayland')
    environment.pop('DISPLAY', None)
    environment.pop('WAYLAND_DISPLAY', None)
    version = subprocess.check_output(['kwin_wayland', '--version'], env=environment, text=True).strip()
    (evidence / 'version.txt').write_text(version)
    import gi
    gi.require_version('Gio', '2.0')
    from gi.repository import Gio, GLib
    bus = Gio.bus_get_sync(Gio.BusType.SESSION, None)

    def call(destination, path, interface, method, parameters=None):
        return bus.call_sync(destination, path, interface, method, parameters, None,
                             Gio.DBusCallFlags.NONE, 3000, None)

    def pump():
        context = GLib.MainContext.default()
        for _ in range(100):
            if not context.pending():
                break
            context.iteration(False)

    def wait(check, description, timeout=20):
        deadline = time.monotonic() + timeout
        while time.monotonic() < deadline:
            if compositor.poll() is not None:
                raise RuntimeError(f'KWin exited during {description}: {(evidence / "compositor.log").read_text()[-5000:]}')
            pump()
            if result := check():
                return result
            time.sleep(0.02)
        names = call('org.freedesktop.DBus', '/org/freedesktop/DBus',
                     'org.freedesktop.DBus', 'ListNames').unpack()[0]
        (evidence / 'timeout-bus-names.json').write_text(json.dumps(names, indent=2) + '\n')
        raise RuntimeError(f'Timed out waiting for {description}; latest snapshot: {latest}; '
                           f'compositor: {(evidence / "compositor.log").read_text()[-5000:]}')

    latest = None
    with (evidence / 'compositor.log').open('w') as log:
        compositor = subprocess.Popen(['kwin_wayland', '--virtual', '--width', '1280', '--height', '900',
            '--no-lockscreen', '--socket', 'wayland-honk-kwin'], env=environment, stdout=log, stderr=log)
        try:
            wait(lambda: (runtime / 'wayland-honk-kwin').is_socket(), 'private Wayland socket')
            # KWin creates its socket before registering its scripting service.
            # Readiness requires both; a native socket is not a D-Bus owner.
            wait(lambda: call('org.freedesktop.DBus', '/org/freedesktop/DBus',
                              'org.freedesktop.DBus', 'NameHasOwner',
                              GLib.Variant('(s)', ('org.kde.KWin',))).unpack()[0],
                 'KWin D-Bus registration')
            owner = call('org.freedesktop.DBus', '/org/freedesktop/DBus', 'org.freedesktop.DBus',
                         'GetNameOwner', GLib.Variant('(s)', ('org.kde.KWin',))).unpack()[0]
            assert owner.startswith(':'), owner
            os.environ.update(environment)
            os.environ['WAYLAND_DISPLAY'] = 'wayland-honk-kwin'
            gi.require_version('Gtk', '4.0')
            from gi.repository import Gtk
            GLib.set_prgname('honk300-native-probe')
            Gtk.init()
            normal, protected = Gtk.Window(), Gtk.Window()
            normal.set_title('Honk300 ordinary probe')
            protected.set_title('ChatGPT Codex terminal probe')
            for window in (normal, protected):
                window.set_default_size(300, 200)
                window.set_child(Gtk.Label(label='Fixture-owned native KWin window'))
                window.present()
            state = {'command': None, 'stop': False, 'count': 0}
            interface = Gio.DBusNodeInfo.new_for_xml('''<node><interface name="org.emmetts.Honk300.KWin1">
              <method name="Exchange"><arg type="s" direction="in"/><arg type="s" direction="out"/></method>
            </interface></node>''').interfaces[0]

            def exchange(_connection, sender, _path, _interface, method, parameters, invocation):
                nonlocal latest
                try:
                    assert sender == owner and method == 'Exchange', 'Unexpected compositor sender'
                    raw = parameters.unpack()[0]
                    assert len(raw) <= 65536
                    latest = json.loads(raw)
                    assert latest['protocol'] == 1 and len(latest['windows']) <= 64
                    state['count'] += 1
                    response = {'protocol': 1, 'sequence': latest['sequence'], 'commands': [], 'stop': state['stop']}
                    if state['command']:
                        response['commands'] = [state.pop('command')]
                        state['command'] = None
                    invocation.return_value(GLib.Variant('(s)', (json.dumps(response),)))
                except Exception as error:
                    invocation.return_dbus_error('org.emmetts.Honk300.Invalid', str(error))

            registration = bus.register_object('/org/emmetts/Honk300/KWin', interface, exchange, None, None)
            request = call('org.freedesktop.DBus', '/org/freedesktop/DBus', 'org.freedesktop.DBus',
                           'RequestName', GLib.Variant('(su)', ('org.emmetts.Honk300.Wayland', 4))).unpack()[0]
            assert request == 1, request
            script_path = str(Path('integrations/kwin/contents/code/main.js').resolve())
            script_id = call('org.kde.KWin', '/Scripting', 'org.kde.kwin.Scripting', 'loadScript',
                             GLib.Variant('(ss)', (script_path, 'honk300-native-probe'))).unpack()[0]
            assert script_id >= 0, script_id
            major = int(version.split()[-1].split('.')[0])
            object_path = f'/Scripting/Script{script_id}' if major >= 6 else f'/{script_id}'
            call('org.kde.KWin', object_path, 'org.kde.kwin.Script', 'run')

            def find(title):
                return next((item for item in (latest or {}).get('windows', []) if item['title'] == title), None)

            ordinary = wait(lambda: find('Honk300 ordinary probe'), 'native ordinary identity')
            denied = wait(lambda: find('ChatGPT Codex terminal probe'), 'native protected identity')
            (evidence / 'initial.json').write_text(json.dumps(latest, indent=2) + '\n')
            assert not ordinary['protected'] and denied['protected'], (ordinary, denied)
            assert ordinary['pid'] == os.getpid() and denied['pid'] == os.getpid(), latest

            def move(item, dx, dy):
                return dict(kind='move', id=item['id'], pid=item['pid'], app=item['app'],
                            **{'from': item['geometry'], 'to': [item['geometry'][0] + dx, item['geometry'][1] + dy]})

            start = ordinary['geometry']
            state['command'] = move(ordinary, 10, 0)
            moved = wait(lambda: (item if (item := find('Honk300 ordinary probe')) and item['geometry'][0] == start[0] + 10 else None), 'bounded real native movement')
            assert moved['geometry'][1:] == start[1:]
            state['command'] = move(denied, 10, 0)
            wait(lambda: latest.get('result') == 'protected', 'terminal refusal')
            assert find('ChatGPT Codex terminal probe')['geometry'] == denied['geometry']
            state['command'] = move(moved, 200, 0)
            wait(lambda: latest.get('result') == 'bounded', 'excess movement refusal')
            state['command'] = move(ordinary, 5, 0)
            wait(lambda: latest.get('result') == 'stale', 'stale geometry refusal')
            state['stop'] = True
            wait(lambda: not state['command'], 'final exchange')
            before = state['count']
            deadline = time.monotonic() + 0.7
            while time.monotonic() < deadline:
                pump()
                time.sleep(0.02)
            after = state['count']
            assert after <= before + 1, (before, after)
            call('org.kde.KWin', '/Scripting', 'org.kde.kwin.Scripting', 'unloadScript',
                 GLib.Variant('(s)', ('honk300-native-probe',)))
            normal.destroy()
            protected.destroy()
            bus.unregister_object(registration)
            (evidence / 'result.json').write_text(json.dumps(dict(ok=True, kwin=version,
                architecture=os.uname().machine, native_identity=True, bounded_move=True,
                terminal_refused=True, excessive_move_refused=True, stale_refused=True,
                stop=True, rust_runtime_connected=False), indent=2) + '\n')
        finally:
            if latest:
                (evidence / 'last-snapshot.json').write_text(json.dumps(latest, indent=2) + '\n')
            compositor.terminate()
            try:
                compositor.wait(timeout=10)
            except subprocess.TimeoutExpired:
                compositor.kill()
                compositor.wait()


if __name__ == '__main__':
    main()
