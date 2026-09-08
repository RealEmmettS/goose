#!/usr/bin/env python3
"""Earliest real KWin script premise test, confined to a disposable CI compositor."""
from __future__ import annotations
import argparse
from contextlib import contextmanager
import json
import os
from pathlib import Path
import selectors
import subprocess
import threading
import time
import traceback


class RustBridge:
    def __init__(self, binary, evidence):
        self.log = (evidence / 'rust-bridge.log').open('a')
        self.process = subprocess.Popen([str(binary)], stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=self.log)
        self.selector = selectors.DefaultSelector()
        self.selector.register(self.process.stdout, selectors.EVENT_READ)
        self.buffer = b''
        assert self.receive() == {'ready': True}

    def receive(self):
        deadline = time.monotonic() + 5
        while time.monotonic() < deadline:
            if b'\n' in self.buffer:
                line, self.buffer = self.buffer.split(b'\n', 1)
                return json.loads(line)
            if self.selector.select(0.1):
                chunk = os.read(self.process.stdout.fileno(), 65536)
                if not chunk:
                    raise RuntimeError(f'Rust bridge exited: {self.process.poll()}')
                self.buffer += chunk
        raise RuntimeError('Rust bridge response deadline exceeded')

    def request(self, op, **fields):
        self.process.stdin.write((json.dumps({'op': op, **fields}) + '\n').encode())
        self.process.stdin.flush()
        return self.receive()

    def close(self):
        self.process.stdin.close()
        try:
            self.process.wait(timeout=5)
        except subprocess.TimeoutExpired:
            self.process.kill()
            self.process.wait()
            raise
        finally:
            self.selector.close()
            self.process.stdout.close()
            self.log.close()
        assert self.process.returncode == 0


@contextmanager
def private_pipewire(environment, evidence, enabled):
    if not enabled:
        yield
        return
    with (evidence / 'pipewire.log').open('w') as log:
        process = subprocess.Popen(['pipewire'], env=environment, stdout=log, stderr=log)
        try:
            deadline = time.monotonic() + 10
            while not (Path(environment['XDG_RUNTIME_DIR']) / 'pipewire-0').is_socket():
                assert process.poll() is None, 'Private PipeWire exited before readiness'
                assert time.monotonic() < deadline, 'Private PipeWire socket did not become ready'
                time.sleep(0.02)
            yield
        finally:
            process.terminate()
            try:
                process.wait(timeout=5)
            except subprocess.TimeoutExpired:
                process.kill()
                process.wait()


def main():
    if os.environ.get('GITHUB_ACTIONS') != 'true':
        raise RuntimeError('Use only a disposable GitHub runner')
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--evidence', type=Path, required=True)
    parser.add_argument('--bridge', type=Path)
    parser.add_argument('--goose', type=Path)
    parser.add_argument('--settings', type=Path)
    parser.add_argument('--portal', action='store_true')
    parser.add_argument('--runtime-repetitions', type=int, choices=range(1, 9), default=1)
    args = parser.parse_args()
    evidence = args.evidence.resolve()
    evidence.mkdir(parents=True, exist_ok=True)
    runtime = evidence / 'runtime'
    runtime.mkdir(mode=0o700)
    config = evidence / 'config'
    config.mkdir()
    environment = dict(os.environ, XDG_RUNTIME_DIR=str(runtime), XDG_CONFIG_HOME=str(config),
        XDG_DATA_HOME=str(evidence / 'data'),
        XDG_CURRENT_DESKTOP='KDE', XDG_SESSION_TYPE='wayland', XDG_MENU_PREFIX='plasma-',
        QT_QPA_PLATFORM='offscreen',
        KWIN_COMPOSE='Q', LIBGL_ALWAYS_SOFTWARE='true', GDK_BACKEND='wayland',
        QT_LOGGING_RULES='kwin_core.debug=true')
    environment.pop('DISPLAY', None)
    environment.pop('WAYLAND_DISPLAY', None)
    version = subprocess.check_output(['kwin_wayland', '--version'], env=environment, text=True).strip()
    (evidence / 'version.txt').write_text(version)
    if args.portal and 'kwin 6.' in version.lower():
        backend_desktop = Path('/usr/share/applications/org.freedesktop.impl.portal.desktop.kde.desktop')
        (evidence / 'backend.desktop').write_bytes(backend_desktop.read_bytes())
        menu = Path('/etc/xdg/menus/plasma-applications.menu')
        assert menu.is_file(), 'The installed Plasma service menu is required for KService discovery'
        # A bare container lacks the desktop login's service-cache refresh.
        # Discover the installed backend's own desktop-file permissions through
        # KService; retain the normal compositor interface authorization checks.
        with (evidence / 'service-cache.log').open('w') as service_log:
            subprocess.run(['kbuildsycoca6', '--noincremental'], env=environment,
                           stdout=service_log, stderr=service_log, check=True, timeout=30)
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
    with private_pipewire(environment, evidence, args.portal and 'kwin 6.' in version.lower()), \
            (evidence / 'compositor.log').open('w') as log:
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
            if args.portal and 'kwin 6.' in version.lower():
                for property_name in ('AvailablePlugins', 'LoadedPlugins'):
                    value = call('org.kde.KWin', '/Plugins', 'org.freedesktop.DBus.Properties',
                                 'Get', GLib.Variant('(ss)', ('org.kde.KWin.Plugins', property_name))).unpack()[0]
                    (evidence / f'{property_name}.json').write_text(json.dumps(value, indent=2) + '\n')
            # GTK and the settings service may activate portals before the input
            # qualifier. Give those services the actual private compositor first.
            activation = {key: value for key, value in os.environ.items()
                if key in ('WAYLAND_DISPLAY', 'XDG_RUNTIME_DIR', 'XDG_CONFIG_HOME', 'XDG_DATA_HOME',
                           'XDG_CURRENT_DESKTOP', 'XDG_SESSION_TYPE', 'XDG_MENU_PREFIX', 'LANG')}
            activation.update(QT_QPA_PLATFORM='wayland', QT_ACCESSIBILITY='1',
                              QT_LINUX_ACCESSIBILITY_ALWAYS_ON='1')
            call('org.freedesktop.DBus', '/org/freedesktop/DBus', 'org.freedesktop.DBus',
                 'UpdateActivationEnvironment', GLib.Variant('(a{ss})', (activation,)))
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
            wait(lambda: normal.get_mapped() and protected.get_mapped(), 'mapped native fixture windows')
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

            # Export the fixture responder in its own GLib context. Native GTK paint
            # and synchronous test control calls must not starve protocol replies.
            service_context = GLib.MainContext.new()
            service_loop = GLib.MainLoop.new(service_context, False)
            service_ready = threading.Event()
            service_registration = []

            def run_service():
                service_context.push_thread_default()
                try:
                    service_registration.append(bus.register_object('/org/emmetts/Honk300/KWin',
                        interface, exchange, None, None))
                    service_ready.set()
                    service_loop.run()
                finally:
                    if service_registration:
                        bus.unregister_object(service_registration[0])
                    service_context.pop_thread_default()

            service_thread = threading.Thread(target=run_service, daemon=True)
            service_thread.start()
            assert service_ready.wait(3), 'Fixture D-Bus responder did not start'
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
            assert ordinary['on_desktop'] and ordinary['on_activity'] and ordinary['drag_known'], ordinary
            assert latest['stacking_order'] == (major >= 6), latest

            fixture_name = None

            def unload_fixture():
                nonlocal fixture_name
                if fixture_name:
                    assert call('org.kde.KWin', '/Scripting', 'org.kde.kwin.Scripting', 'unloadScript',
                        GLib.Variant('(s)', (fixture_name,))).unpack()[0]
                    fixture_name = None

            def fixture_action(name, action):
                nonlocal fixture_name
                unload_fixture()
                # This script is fixture-only and never packaged. The test owns the
                # private compositor and selects only its exact native window/PID.
                helper = evidence / f'fixture-{name}.js'
                helper.write_text('''(function () {
                    var windows = workspace.stackingOrder !== undefined ? workspace.stackingOrder : workspace.clientList();
                    for (var i = 0; i < windows.length; i++) {
                        var window = windows[i];
                        if (Number(window.pid) === ''' + str(os.getpid()) + ''' &&
                            String(window.caption) === "Honk300 ordinary probe") {
                            ''' + action + '''
                            return;
                        }
                    }
                    throw new Error("Fixture-owned window unavailable");
                }());''')
                identifier = call('org.kde.KWin', '/Scripting', 'org.kde.kwin.Scripting', 'loadScript',
                    GLib.Variant('(ss)', (str(helper), f'honk300-fixture-{name}'))).unpack()[0]
                assert identifier >= 0
                fixture_name = f'honk300-fixture-{name}'
                address = f'/Scripting/Script{identifier}' if major >= 6 else f'/{identifier}'
                call('org.kde.KWin', address, 'org.kde.kwin.Script', 'run')

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
            service_loop.quit()
            service_thread.join(timeout=3)
            assert not service_thread.is_alive(), 'Fixture D-Bus responder did not stop'
            call('org.freedesktop.DBus', '/org/freedesktop/DBus', 'org.freedesktop.DBus',
                 'ReleaseName', GLib.Variant('(s)', ('org.emmetts.Honk300.Wayland',)))
            if args.bridge:
                def load_rust_script():
                    identity = call('org.kde.KWin', '/Scripting', 'org.kde.kwin.Scripting', 'loadScript',
                        GLib.Variant('(ss)', (script_path, 'honk300-rust-probe'))).unpack()[0]
                    assert identity >= 0
                    address = f'/Scripting/Script{identity}' if major >= 6 else f'/{identity}'
                    call('org.kde.KWin', address, 'org.kde.kwin.Script', 'run')

                def unload_rust_script():
                    # KWin allocates ids from the current script list length. Remove
                    # the last fixture script first so a later reconnect cannot reuse
                    # an id whose D-Bus object is still occupied by our test helper.
                    unload_fixture()
                    call('org.kde.KWin', '/Scripting', 'org.kde.kwin.Scripting', 'unloadScript',
                         GLib.Variant('(s)', ('honk300-rust-probe',)))

                bridge = RustBridge(args.bridge.resolve(), evidence)
                try:
                    load_rust_script()

                    def rust_window(title):
                        current = bridge.request('snapshot')['snapshot']
                        return next((item for item in (current or {}).get('windows', []) if item['title'] == title), None)

                    ordinary = wait(lambda: rust_window('Honk300 ordinary probe'), 'authenticated Rust snapshot')
                    denied = wait(lambda: rust_window('ChatGPT Codex terminal probe'), 'Rust protected snapshot')
                    response = bridge.request('move', window=ordinary,
                                              to=[ordinary['geometry'][0] + 10, ordinary['geometry'][1]])
                    assert response['ok'], response
                    moved = wait(lambda: (item if (item := rust_window('Honk300 ordinary probe')) and
                        item['geometry'][0] == ordinary['geometry'][0] + 10 else None), 'Rust-authorized native move')
                    assert not bridge.request('move', window=denied,
                        to=[denied['geometry'][0] + 10, denied['geometry'][1]])['ok']
                    assert not bridge.request('move', window=moved,
                        to=[moved['geometry'][0] + 200, moved['geometry'][1]])['ok']
                    assert not bridge.request('move', window=ordinary,
                        to=[ordinary['geometry'][0] + 5, ordinary['geometry'][1]])['ok']
                    try:
                        call('org.emmetts.Honk300.Wayland', '/org/emmetts/Honk300/KWin',
                             'org.emmetts.Honk300.KWin1', 'Exchange', GLib.Variant('(s)', (json.dumps(latest),)))
                    except GLib.Error as error:
                        assert 'AccessDenied' in str(error), error
                    else:
                        raise AssertionError('Untrusted D-Bus peer injected a compositor frame')
                    assert rust_window('Honk300 ordinary probe'), 'Untrusted peer revoked the valid observer'
                    normal.fullscreen()
                    fullscreen = wait(lambda: (item if (item := rust_window('Honk300 ordinary probe')) and
                        item['fullscreen'] else None), 'native fullscreen observation')
                    assert not bridge.request('move', window=fullscreen,
                        to=[fullscreen['geometry'][0] + 5, fullscreen['geometry'][1]])['ok']
                    normal.unfullscreen()
                    wait(lambda: (item if (item := rust_window('Honk300 ordinary probe')) and
                        not item['fullscreen'] else None), 'fullscreen exit')
                    fixture_action('desktop', 'workspace.createDesktop(1, "Honk300 private desktop"); workspace.slotSwitchDesktopNext();')
                    hidden = wait(lambda: (item if (item := rust_window('Honk300 ordinary probe')) and
                        not item['on_desktop'] else None), 'window on another native desktop')
                    assert not bridge.request('move', window=hidden,
                        to=[hidden['geometry'][0] + 5, hidden['geometry'][1]])['ok']
                    fixture_action('return', 'workspace.slotSwitchDesktopPrevious();')
                    wait(lambda: (item if (item := rust_window('Honk300 ordinary probe')) and
                        item['on_desktop'] else None), 'return to native desktop')
                    (evidence / 'rust-native-snapshot.json').write_text(json.dumps(bridge.request('snapshot'), indent=2) + '\n')
                    unload_rust_script()
                    wait(lambda: bridge.request('snapshot')['snapshot'] is None, 'Rust expiry after companion loss')
                    assert not bridge.request('move', window=moved,
                        to=[moved['geometry'][0] + 5, moved['geometry'][1]])['ok']
                finally:
                    bridge.close()
                # A new explicitly established connection can recover; old queues do not.
                bridge = RustBridge(args.bridge.resolve(), evidence)
                try:
                    load_rust_script()
                    recovered = wait(lambda: rust_window('Honk300 ordinary probe'), 'new Rust connection')
                    fixture_action('drag', '''if (workspace.activeWindow !== undefined) workspace.activeWindow = window;
                        else workspace.activeClient = window; workspace.slotWindowMove();''')
                    dragging = wait(lambda: (item if (item := rust_window('Honk300 ordinary probe')) and
                        item['drag_known'] and item['dragging'] else None), 'actual native interactive move')
                    assert not bridge.request('move', window=dragging,
                        to=[dragging['geometry'][0] + 5, dragging['geometry'][1]])['ok']
                    assert bridge.request('stop')['ok']
                    assert bridge.request('snapshot')['snapshot'] is None
                    assert not bridge.request('move', window=recovered,
                        to=[recovered['geometry'][0] + 5, recovered['geometry'][1]])['ok']
                    unload_rust_script()
                finally:
                    bridge.close()
                # The production owner loads sealed script bytes itself and retires
                # only that registration when its retained connection is dropped.
                bridge = RustBridge(args.bridge.resolve(), evidence)
                try:
                    activated = bridge.request('activate')
                    assert activated['ok'], activated
                    wait(lambda: bridge.request('snapshot')['snapshot'], 'Rust-owned sealed companion')
                    assert not bridge.request('activate')['ok'], 'Duplicate activation was accepted'
                finally:
                    bridge.close()
                assert not call('org.kde.KWin', '/Scripting', 'org.kde.kwin.Scripting', 'isScriptLoaded',
                    GLib.Variant('(s)', ('honk300-native-owned-probe',))).unpack()[0], 'Owned companion leaked after drop'
                (evidence / 'rust-result.json').write_text(json.dumps(dict(ok=True,
                    native_identity=True, bounded_move=True, untrusted_peer_refused=True,
                    protected_stale_excessive_refused=True, connection_loss_expires=True,
                    explicit_reconnect=True, stop=True, fullscreen_refused=True,
                    other_desktop_refused=True, actual_user_drag_observed=True,
                    sealed_rust_activation=True, owned_script_cleanup=True,
                    goose_runtime_connected=False), indent=2) + '\n')
            normal.destroy()
            protected.destroy()
            pump()
            failures = []

            def qualify_independently(name, action):
                try:
                    action()
                except Exception:
                    failure = traceback.format_exc()
                    (evidence / f'{name}-failure.txt').write_text(failure)
                    failures.append(name)
                    print(failure, flush=True)

            if args.goose:
                from smoke_kwin_runtime import qualify
                for iteration in range(1, args.runtime_repetitions + 1):
                    qualify_independently(f'runtime-{iteration}', lambda: qualify(
                        args.goose.resolve(), evidence, wait, call, GLib, iteration=iteration))
            if args.settings:
                from smoke_kwin_settings import qualify
                qualify_independently('settings', lambda: qualify(args.settings.resolve(), evidence, wait, call, GLib))
            if args.portal and major >= 6:
                from smoke_kwin_portal import qualify
                qualify_independently('portal', lambda: qualify(args.bridge.resolve(), evidence, wait, call, GLib, Gtk, RustBridge,
                    args.goose.resolve() if args.goose else None, args.settings.resolve() if args.settings else None))
            assert not failures, f'Native qualifiers failed: {failures}'
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
