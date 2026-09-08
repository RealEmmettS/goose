#!/usr/bin/env python3
"""Prove Hyprland's native interface in a private disposable CI desktop."""
import argparse
import json
import os
from pathlib import Path
import re
import select
import shutil
import socket
import struct
import subprocess
import time


def main():
    assert os.environ.get('GITHUB_ACTIONS') == 'true', 'Disposable CI only'
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--evidence', type=Path, required=True)
    parser.add_argument('--generation', choices=('legacy', 'lua'), required=True)
    parser.add_argument('--bridge', type=Path)
    parser.add_argument('--goose', type=Path)
    parser.add_argument('--settings', type=Path)
    args = parser.parse_args()
    evidence = args.evidence.resolve()
    evidence.mkdir(parents=True, exist_ok=True)
    # The compositor adds its long build signature below this path. Keep the
    # private runtime short enough for Linux's 108-byte Unix socket pathname.
    runtime = Path('/tmp') / f'hy-{os.getpid()}'
    runtime.mkdir(mode=0o700)
    lua = args.generation == 'lua'
    config = evidence / ('hyprland.lua' if lua else 'hyprland.conf')
    config.write_text(
        'hl.monitor({output="HONK-PROBE",mode="1280x900@60",position="0x0",scale="1"})\n'
        'hl.monitor({output="",disabled=true})\n'
        'hl.config({animations={enabled=false},input={follow_mouse=0},'
        'misc={disable_hyprland_logo=true,disable_splash_rendering=true},'
        'debug={disable_logs=false,enable_stdout_logs=true}})\n' if lua else
        'monitor = HONK-PROBE,1280x900@60,0x0,1\nmonitor = ,disable\n'
        'animations {\n enabled = false\n}\ninput {\n follow_mouse = 0\n}\n'
        'misc {\n disable_hyprland_logo = true\n disable_splash_rendering = true\n}\n'
        'debug {\n disable_logs = false\n enable_stdout_logs = true\n}\n')
    environment = dict(os.environ, XDG_RUNTIME_DIR=str(runtime),
        XDG_DATA_HOME=str(evidence / 'user-data'), XDG_CONFIG_HOME=str(evidence / 'user-config'),
        XDG_CURRENT_DESKTOP='Hyprland', XDG_SESSION_TYPE='wayland',
        LIBGL_ALWAYS_SOFTWARE='1', GDK_BACKEND='wayland', LIBSEAT_BACKEND='seatd')
    for key in ('XDG_DATA_HOME', 'XDG_CONFIG_HOME'):
        Path(environment[key]).mkdir(mode=0o700)
    for key in ('DISPLAY', 'WAYLAND_DISPLAY', 'HYPRLAND_INSTANCE_SIGNATURE'):
        environment.pop(key, None)
    windows = []
    modules = {}
    ipc_path = None
    transactions = []
    watcher = None
    watch_events = []
    with (evidence / 'compositor.log').open('w') as log:
        compositor = subprocess.Popen(['Hyprland', '--config', str(config)],
                                      env=environment, stdout=log, stderr=log)
        try:
            def wait(check, description, timeout=20):
                deadline = time.monotonic() + timeout
                while time.monotonic() < deadline:
                    assert compositor.poll() is None, 'Hyprland exited during ' + description
                    if 'GLib' in modules:
                        context = modules['GLib'].MainContext.default()
                        for _ in range(100):
                            if not context.pending():
                                break
                            context.iteration(False)
                    try:
                        if result := check():
                            return result
                    except TimeoutError:
                        # Output creation can occupy the compositor before the
                        # first observation. Retry only this read-side wait; each
                        # transaction still has the original single deadline.
                        pass
                    time.sleep(0.01)
                raise AssertionError('Timed out waiting for ' + description)

            ipc_path = wait(lambda: next(iter(runtime.glob('hypr/*/.socket.sock')), None), 'IPC socket')
            display = wait(lambda: next((p for p in runtime.glob('wayland-*') if p.is_socket()), None),
                           'Wayland socket')
            environment['WAYLAND_DISPLAY'] = display.name
            environment['HYPRLAND_INSTANCE_SIGNATURE'] = ipc_path.parent.name
            os.environ.update(environment)

            def ipc(payload, decode=True):
                # Hyprland handles each request synchronously. Never keep the
                # connection open between transactions or reset its total deadline.
                started = time.monotonic()
                deadline = started + 0.25
                transaction = dict(request=payload, bytes=0, complete=False)
                transactions.append(transaction)
                with socket.socket(socket.AF_UNIX, socket.SOCK_STREAM) as connection:
                    connection.settimeout(max(0.001, deadline - time.monotonic()))
                    connection.connect(str(ipc_path))
                    peer = struct.unpack('3i', connection.getsockopt(socket.SOL_SOCKET,
                                         socket.SO_PEERCRED, struct.calcsize('3i')))
                    assert peer[:2] == (compositor.pid, os.getuid()), peer
                    connection.sendall(payload.encode())
                    chunks = bytearray()
                    while True:
                        remaining = deadline - time.monotonic()
                        if remaining <= 0:
                            raise TimeoutError('Hyprland response exceeded the total deadline')
                        connection.settimeout(remaining)
                        # GTK fixture windows live in this process. Keep their
                        # configure acknowledgements flowing while the real
                        # compositor handles the request; a blocking recv would
                        # stall the very client whose placement is under test.
                        if 'GLib' in modules:
                            context = modules['GLib'].MainContext.default()
                            for _ in range(100):
                                if not context.pending():
                                    break
                                context.iteration(False)
                        if not select.select([connection], [], [], min(0.01, remaining))[0]:
                            if time.monotonic() >= deadline:
                                raise TimeoutError('Hyprland response exceeded the total deadline')
                            continue
                        chunk = connection.recv(min(8192, 256 * 1024 + 1 - len(chunks)))
                        if not chunk:
                            break
                        chunks.extend(chunk)
                        transaction['bytes'] = len(chunks)
                        assert len(chunks) <= 256 * 1024, 'Oversized native response'
                    text = chunks.decode()
                    transaction.update(complete=True, elapsed_ms=(time.monotonic() - started) * 1000)
                    return json.loads(text) if decode else text

            def command(text):
                response = ipc(text, False)
                assert response.strip() == 'ok', (text, response)

            startup_began = time.monotonic()
            version = wait(lambda: ipc('j/version'), 'initial version response')
            (evidence / 'startup-readiness.json').write_text(json.dumps(dict(
                seconds=time.monotonic() - startup_began,
                attempts=len(transactions), transaction_deadline_seconds=.25), indent=2))
            (evidence / 'version.json').write_text(json.dumps(version, indent=2) + '\n')
            command('/output create headless HONK-PROBE')
            monitors = wait(lambda: ipc('j/monitors'), 'headless output')
            assert len(monitors) == 1 and monitors[0]['name'] == 'HONK-PROBE', monitors
            (evidence / 'monitors.json').write_text(json.dumps(monitors, indent=2) + '\n')
            (evidence / 'devices.json').write_text(json.dumps(ipc('j/devices'), indent=2) + '\n')
            assert ipc('/configerrors', False).strip() == '', 'Private compositor config has errors'

            import gi
            gi.require_version('Gtk', '4.0')
            from gi.repository import Gtk, GLib
            modules['GLib'] = GLib
            GLib.set_prgname('honk300-hyprland-probe')
            Gtk.init()
            def find(title):
                return next((node for node in ipc('j/clients') if node.get('title') == title), None)

            # Fixed-size clients are natively floated by Hyprland before mapping.
            # Avoid a layout-mode transition (which makes a software-rendered
            # snapshot) while qualifying observations and ordinary placement.
            for title in ('Honk300 ordinary Hyprland probe', 'ChatGPT Codex terminal probe'):
                window = Gtk.Window(title=title)
                window.set_default_size(300, 200)
                window.set_resizable(False)
                window.set_child(Gtk.Label(label=title))
                window.present()
                windows.append(window)
                node = wait(lambda: find(title), 'native fixture identity')
                assert node['pid'] == os.getpid() and node['class'] == 'honk300-hyprland-probe', node
                assert re.fullmatch(r'0x[0-9a-fA-F]+', node['address']), node
                wait(lambda: find(title)['floating'], 'floating fixture')
            ordinary, protected = (find(window.get_title()) for window in windows)
            for node in (ordinary, protected):
                assert node['mapped'] and not node['hidden'] and not node['xwayland'], node
            def run_bridge(probe_environment):
                process = subprocess.Popen([str(args.bridge.resolve())], env=probe_environment,
                    stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True)
                deadline = time.monotonic() + 5
                try:
                    while time.monotonic() < deadline:
                        try:
                            stdout, stderr = process.communicate(timeout=0.01)
                            return subprocess.CompletedProcess(process.args, process.returncode, stdout, stderr)
                        except subprocess.TimeoutExpired:
                            # A separate observer must not freeze its GTK target.
                            # Keep native fullscreen configure acknowledgements
                            # flowing while Rust performs its bounded requests.
                            context = GLib.MainContext.default()
                            for _ in range(100):
                                if not context.pending():
                                    break
                                context.iteration(False)
                    raise TimeoutError('Native Rust probe exceeded its process deadline')
                finally:
                    if process.poll() is None:
                        process.kill()
                        process.communicate(timeout=2)
            def rust_snapshot(label):
                if args.bridge is None:
                    return None
                result = run_bridge(environment)
                if result.returncode != 0:
                    diagnostics = dict(error=result.stderr, phase=label, queries=[])
                    for query in ('j/clients', 'j/monitors', 'j/version',
                                  '[[BATCH]]j/monitors;j/clients;j/monitors'):
                        started = time.monotonic()
                        try:
                            reply = ipc(query, decode=False)
                            diagnostics['queries'].append(dict(query=query, reply=reply,
                                elapsed_ms=(time.monotonic() - started) * 1000))
                        except TimeoutError as error:
                            diagnostics['queries'].append(dict(query=query, error=str(error),
                                elapsed_ms=(time.monotonic() - started) * 1000))
                    (evidence / f'rust-{label}-failure.json').write_text(
                        json.dumps(diagnostics, indent=2) + '\n')
                assert result.returncode == 0, result.stderr
                observed = json.loads(result.stdout)
                assert observed['peer_pid'] == compositor.pid and observed['peer_uid'] == os.getuid()
                assert not observed['movement'] and not observed['pointer_control']
                assert not observed['pointer_observation']
                (evidence / f'rust-{label}.json').write_text(json.dumps(observed, indent=2) + '\n')
                return observed
            rust = rust_snapshot('initial')
            if rust:
                actual = next(window for window in rust['windows'] if window['id'] == ordinary['address'])
                assert actual['pid'] == os.getpid() and actual['geometry'] == ordinary['at'] + ordinary['size']
                assert actual['visible'] and not rust['fullscreen']
                fake_signature = 'f' * 40 + '_1_1'
                fake_directory = runtime / 'hypr' / fake_signature
                fake_directory.mkdir(mode=0o700)
                fake_socket = fake_directory / '.socket.sock'
                with socket.socket(socket.AF_UNIX, socket.SOCK_STREAM) as listener:
                    listener.bind(str(fake_socket))
                    listener.listen(1)
                    listener.settimeout(2)
                    denied = run_bridge(dict(environment, HYPRLAND_INSTANCE_SIGNATURE=fake_signature))
                    assert denied.returncode != 0 and 'system-owned compositor executable' in denied.stderr, denied.stderr
                    client, _ = listener.accept()
                    with client:
                        assert client.recv(32) == b'', 'An impostor received a query before rejection'
                fake_socket.unlink()
            watch_buffer = bytearray()
            def watch_current():
                if watcher is None:
                    return None
                assert watcher.poll() is None, 'Native observation worker exited'
                while True:
                    try:
                        chunk = os.read(watcher.stdout.fileno(), 8192)
                    except BlockingIOError:
                        break
                    if not chunk:
                        break
                    watch_buffer.extend(chunk)
                    assert len(watch_buffer) < 65536
                while b'\n' in watch_buffer:
                    line, _, tail = watch_buffer.partition(b'\n')
                    watch_buffer[:] = tail
                    assert len(line) < 512 and len(watch_events) < 400
                    watch_events.append(json.loads(line))
                return watch_events[-1] if watch_events else None
            if args.bridge:
                watcher = subprocess.Popen([str(args.bridge.resolve()), '--watch'],
                    env=environment, stdout=subprocess.PIPE, stderr=subprocess.DEVNULL)
                os.set_blocking(watcher.stdout.fileno(), False)
                wait(lambda: (state if (state := watch_current()) and state['observed']
                    and not state['fullscreen'] else None), 'retained observation worker')
            initial = ordinary['at']
            selector = 'address:' + ordinary['address']
            command("/dispatch hl.dsp.window.move({x=" + str(initial[0] + 6) + ',y=' +
                    str(initial[1]) + ",relative=false,window='" + selector + "'})" if lua else
                    f'/dispatch movewindowpixel exact {initial[0] + 6} {initial[1]},{selector}')
            moved = wait(lambda: (node if (node := find(ordinary['title'])) and
                node['at'] == [initial[0] + 6, initial[1]] else None), 'six-pixel owned movement')
            assert find(protected['title'])['at'] == protected['at'], 'Protected fixture moved'
            windows[0].fullscreen()
            fullscreen = wait(lambda: (node if (node := find(ordinary['title'])) and
                node['fullscreen'] == 2 else None), 'actual fullscreen observation')
            (evidence / 'native-fullscreen.json').write_text(json.dumps(dict(
                native=fullscreen, gtk_size=[windows[0].get_width(), windows[0].get_height()]),
                indent=2) + '\n')
            if watcher:
                observed = wait(lambda: (state if (state := watch_current()) and state['observed']
                    and state['fullscreen'] and state['fullscreen_window'] is not None else None),
                    'fresh fullscreen from the retained production observer')
                actual = observed['fullscreen_window']
                assert actual['id'] == ordinary['address'] and actual['pid'] == os.getpid(), actual
                assert actual['geometry'] == fullscreen['at'] + fullscreen['size'], actual
                (evidence / 'rust-fullscreen.json').write_text(json.dumps(observed, indent=2) + '\n')
            windows[0].unfullscreen()
            wait(lambda: find(ordinary['title'])['fullscreen'] == 0, 'fullscreen removal')
            if watcher:
                wait(lambda: (state if (state := watch_current()) and state['observed']
                    and not state['fullscreen'] else None), 'fresh fullscreen removal')
            if watcher is not None:
                watcher.terminate()
                watcher.wait(timeout=3)
                watcher = None
            if args.goose is not None:
                assert args.settings is not None, 'Native settings are required with the real goose'
                from smoke_hyprland_runtime import qualify
                qualify(args.goose.resolve(), args.settings.resolve(), evidence, wait,
                        windows[0], find, ipc_path, GLib)
            windows[0].destroy()
            wait(lambda: find(ordinary['title']) is None, 'vanished target')
            assert find(protected['title'])['at'] == protected['at'], 'Protected fixture changed'
            if args.goose is not None:
                windows[1].destroy()
                wait(lambda: find(protected['title']) is None, 'native fixture cleanup before prop qualification')
                for label, script, executable, extra in (
                    ('owned-props', 'script/smoke_owned_props_linux.py', args.settings, []),
                    ('runtime-props', 'script/smoke_runtime_props_linux.py', args.goose,
                     ['--expected-desktop', 'Hyprland']),
                ):
                    with (evidence / f'{label}.log').open('w') as log:
                        subprocess.run(['python3', script, '--binary', str(executable.resolve()),
                            '--evidence', str(evidence / label), *extra],
                            stdout=log, stderr=log, check=True, timeout=300)
            result = dict(ok=True, compositor_pid=compositor.pid, unix_uid=os.getuid(),
                peer_credentials=True, version=version, architecture=os.uname().machine,
                initial=ordinary, moved=moved, protected=protected, fullscreen=fullscreen,
                vanished=True, pointer_control_qualified=False, user_drag_observation_qualified=False,
                production_rust_observation=args.bridge is not None, untrusted_peer_refused=args.bridge is not None,
                retained_worker_fullscreen_recovery=args.bridge is not None,
                production_runtime=args.goose is not None, native_owned_props=args.goose is not None,
                expired_observation_samples=sum(not state['observed'] for state in watch_events))
            (evidence / 'result.json').write_text(json.dumps(result, indent=2) + '\n')
            print(json.dumps(result))
        finally:
            if watcher is not None:
                watcher.terminate()
                watcher.wait(timeout=3)
            (evidence / 'worker-observations.json').write_text(json.dumps(watch_events, indent=2) + '\n')
            (evidence / 'transactions.json').write_text(json.dumps(transactions, indent=2) + '\n')
            for window in windows:
                window.destroy()
            compositor.terminate()
            try:
                compositor.wait(timeout=5)
            except subprocess.TimeoutExpired:
                compositor.kill()
                compositor.wait(timeout=5)
            # Preserve flushed compositor diagnostics, never sockets or other
            # user state from outside this fixture-owned runtime.
            for source in runtime.glob('hypr/*/hyprland.log'):
                shutil.copyfile(source, evidence / 'backend.log')


if __name__ == '__main__':
    main()
