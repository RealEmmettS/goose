#!/usr/bin/env python3
"""Earliest real Sway adapter premise, on an isolated disposable CI desktop."""
import argparse
import json
import os
from pathlib import Path
import socket
import struct
import subprocess
import time


def main():
    assert os.environ.get('GITHUB_ACTIONS') == 'true', 'Disposable CI only'
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--evidence', type=Path, required=True)
    parser.add_argument('--bridge', type=Path)
    parser.add_argument('--goose', type=Path)
    parser.add_argument('--settings', type=Path)
    args = parser.parse_args()
    evidence = args.evidence.resolve()
    evidence.mkdir(parents=True, exist_ok=True)
    runtime = evidence / 'runtime'
    runtime.mkdir(mode=0o700)
    config = evidence / 'sway.conf'
    config.write_text('output * mode 1280x900\ninput * xkb_layout us\n'
                      'default_border none\nfocus_follows_mouse no\n'
                      'for_window [app_id="honk300-sway-probe"] floating enable\n')
    environment = dict(os.environ, XDG_RUNTIME_DIR=str(runtime),
        XDG_DATA_HOME=str(evidence / 'user-data'), XDG_CONFIG_HOME=str(evidence / 'user-config'),
        XDG_CURRENT_DESKTOP='sway', XDG_SESSION_TYPE='wayland',
        WLR_BACKENDS='headless', WLR_HEADLESS_OUTPUTS='1',
        WLR_LIBINPUT_NO_DEVICES='1', WLR_RENDERER='pixman', GDK_BACKEND='wayland')
    for key in ('DISPLAY', 'WAYLAND_DISPLAY', 'SWAYSOCK', 'I3SOCK'):
        environment.pop(key, None)
    windows = []
    ipc_path = None
    with (evidence / 'compositor.log').open('w') as log:
        compositor = subprocess.Popen(['sway', '-c', str(config), '-d'],
                                      env=environment, stdout=log, stderr=log)
        try:
            def wait(check, description, timeout=15):
                deadline = time.monotonic() + timeout
                while time.monotonic() < deadline:
                    assert compositor.poll() is None, 'Sway exited during ' + description
                    if 'GLib' in locals_for_pump:
                        context = locals_for_pump['GLib'].MainContext.default()
                        for _ in range(100):
                            if not context.pending():
                                break
                            context.iteration(False)
                    if result := check():
                        return result
                    time.sleep(0.01)
                raise AssertionError('Timed out waiting for ' + description)

            locals_for_pump = {}
            ipc_path = wait(lambda: next(iter(runtime.glob('sway-ipc.*.sock')), None), 'IPC socket')
            display = wait(lambda: next((p for p in runtime.glob('wayland-*') if p.is_socket()), None),
                           'Wayland socket')
            environment['WAYLAND_DISPLAY'] = display.name
            environment['SWAYSOCK'] = str(ipc_path)
            os.environ.update(environment)

            def ipc(kind, payload=''):
                with socket.socket(socket.AF_UNIX, socket.SOCK_STREAM) as connection:
                    connection.settimeout(0.25)
                    connection.connect(str(ipc_path))
                    peer = struct.unpack('3i', connection.getsockopt(socket.SOL_SOCKET,
                                         socket.SO_PEERCRED, struct.calcsize('3i')))
                    assert peer[:2] == (compositor.pid, os.getuid()), peer
                    body = payload.encode()
                    connection.sendall(b'i3-ipc' + struct.pack('=II', len(body), kind) + body)
                    def receive(size):
                        chunks = bytearray()
                        while len(chunks) < size:
                            chunk = connection.recv(size - len(chunks))
                            assert chunk, 'IPC closed before the complete reply'
                            chunks.extend(chunk)
                        return bytes(chunks)
                    header = receive(14)
                    assert header[:6] == b'i3-ipc'
                    size, reply = struct.unpack('=II', header[6:])
                    assert reply == kind and size <= 256 * 1024, (size, reply)
                    return json.loads(receive(size))

            def command(text):
                response = ipc(0, text)
                assert response and all(item.get('success') is True for item in response), response

            def nodes(root):
                stack = [root]
                count = 0
                while stack:
                    node = stack.pop()
                    count += 1
                    assert count <= 512
                    yield node
                    stack.extend(node.get('nodes', []))
                    stack.extend(node.get('floating_nodes', []))

            def find(title):
                return next((node for node in nodes(ipc(4)) if node.get('name') == title), None)

            version = ipc(7)
            (evidence / 'version.json').write_text(json.dumps(version, indent=2) + '\n')
            (evidence / 'seats.json').write_text(json.dumps(ipc(101), indent=2) + '\n')
            import gi
            gi.require_version('Gtk', '4.0')
            from gi.repository import Gtk, GLib
            locals_for_pump['GLib'] = GLib
            GLib.set_prgname('honk300-sway-probe')
            Gtk.init()
            for title in ('Honk300 ordinary Sway probe', 'ChatGPT Codex terminal probe'):
                window = Gtk.Window(title=title)
                window.set_default_size(300, 200)
                window.set_child(Gtk.Label(label=title))
                window.present()
                windows.append(window)
            ordinary = wait(lambda: find('Honk300 ordinary Sway probe'), 'ordinary native window')
            protected = wait(lambda: find('ChatGPT Codex terminal probe'), 'protected native identity')
            for node in (ordinary, protected):
                assert node['pid'] == os.getpid(), node
                assert node['app_id'] == 'honk300-sway-probe', node
                assert node['visible'] is True, node
                assert isinstance(node['id'], int) and node['id'] > 0
            assert ordinary['type'] == 'floating_con', ordinary
            def rust_snapshot(label):
                if args.bridge is None:
                    return None
                result = subprocess.run([str(args.bridge.resolve())], env=environment,
                    capture_output=True, text=True, timeout=5)
                assert result.returncode == 0, result.stderr
                details = json.loads(result.stdout)
                assert details['peer_pid'] == compositor.pid and details['peer_uid'] == os.getuid()
                assert not details['movement'] and not details['pointer_control']
                assert not details['pointer_observation']
                (evidence / f'rust-{label}.json').write_text(json.dumps(details, indent=2) + '\n')
                return details

            rust = rust_snapshot('initial')
            if rust:
                observed = next(item for item in rust['windows'] if item['id'] == ordinary['id'])
                assert observed['pid'] == os.getpid() and observed['app'] == ordinary['app_id']
                assert observed['geometry'] == [ordinary['rect'][key] for key in ('x', 'y', 'width', 'height')]
                assert not rust['fullscreen']
                # An unrelated same-user listener cannot impersonate the
                # system compositor, even with a private socket and valid UID.
                fake = runtime / 'fake-sway.sock'
                with socket.socket(socket.AF_UNIX, socket.SOCK_STREAM) as listener:
                    listener.bind(str(fake))
                    listener.listen(1)
                    listener.settimeout(2)
                    rejected = subprocess.run([str(args.bridge.resolve())],
                        env=dict(environment, SWAYSOCK=str(fake)), capture_output=True,
                        text=True, timeout=5)
                    assert rejected.returncode != 0
                    assert 'system-owned compositor executable' in rejected.stderr, rejected.stderr
                    client, _ = listener.accept()
                    with client:
                        assert client.recv(32) == b'', 'Untrusted peer received a query before rejection'
                fake.unlink()
            # Only this fixture-owned node receives commands. Native criteria bind
            # its numeric id, process id and exact application identity together.
            criteria = f'[con_id={ordinary["id"]} pid={os.getpid()} app_id="^honk300-sway-probe$"]'
            initial = ordinary['rect']
            command(f'{criteria} move absolute position {initial["x"] + 6} px {initial["y"]} px')
            moved = wait(lambda: (node if (node := find(ordinary['name'])) and
                node['rect']['x'] == initial['x'] + 6 else None), 'actual bounded native movement')
            assert find(protected['name'])['rect'] == protected['rect'], 'Protected fixture was moved'
            windows[0].fullscreen()
            fullscreen = wait(lambda: (node if (node := find(ordinary['name'])) and
                node['fullscreen_mode'] > 0 else None), 'native fullscreen observation')
            rust = rust_snapshot('fullscreen')
            if rust:
                assert rust['fullscreen']
            windows[0].unfullscreen()
            wait(lambda: find(ordinary['name'])['fullscreen_mode'] == 0, 'fullscreen revocation')
            if args.goose is not None:
                assert args.settings is not None, 'Native settings are required with the real goose'
                from smoke_sway_runtime import qualify
                qualify(args.goose.resolve(), args.settings.resolve(), evidence, wait,
                        windows[0], find, ipc_path, GLib)
            windows[0].destroy()
            windows.pop(0)
            wait(lambda: find(ordinary['name']) is None, 'native disappearance')
            (evidence / 'result.json').write_text(json.dumps(dict(ok=True,
                compositor_pid=compositor.pid, unix_uid=os.getuid(), peer_credentials=True,
                version=version, architecture=os.uname().machine, initial=ordinary,
                moved=moved, protected=protected, fullscreen=fullscreen,
                vanished=True, pointer_control_qualified=False,
                user_drag_observation_qualified=False,
                production_rust_observation=args.bridge is not None,
                production_runtime=args.goose is not None,
                untrusted_peer_refused=args.bridge is not None), indent=2) + '\n')
        finally:
            for window in windows:
                window.destroy()
            compositor.terminate()
            try:
                compositor.wait(timeout=5)
            except subprocess.TimeoutExpired:
                compositor.kill()
                compositor.wait(timeout=5)


if __name__ == '__main__':
    main()
