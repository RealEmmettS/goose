#!/usr/bin/env python3
"""Exercise the real Rust engine/GTK child connection on a disposable Linux desktop."""
from __future__ import annotations
import argparse
import ctypes
import json
import os
from pathlib import Path
import signal
import shutil
import subprocess
import time


def main():
    if os.environ.get('GITHUB_ACTIONS') != 'true' or os.environ.get('GDK_BACKEND') not in ('x11', 'wayland'):
        raise RuntimeError('This probe requires the disposable GitHub desktop')
    positioning = os.environ['GDK_BACKEND'] == 'x11'
    import gi
    gi.require_version('Atspi', '2.0')
    from gi.repository import Atspi, GLib
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--binary', type=Path, required=True)
    parser.add_argument('--evidence', type=Path, required=True)
    args = parser.parse_args()
    binary, evidence = args.binary.resolve(), args.evidence.resolve()
    evidence.mkdir(parents=True, exist_ok=True)

    def control(*arguments, check=True):
        return subprocess.run([str(binary), *arguments], capture_output=True, text=True, timeout=10, check=check)

    assert 'honk300: running' not in control('status', check=False).stdout, 'Another runtime is already active'
    subprocess.run(['gdbus', 'call', '--session', '--dest', 'org.a11y.Bus', '--object-path', '/org/a11y/bus',
                    '--method', 'org.freedesktop.DBus.Properties.Set', 'org.a11y.Status', 'IsEnabled', '<true>'], check=True)
    Atspi.init()

    def wait(check, description, timeout=60):
        deadline = time.monotonic() + timeout
        while time.monotonic() < deadline:
            if result := check():
                return result
            time.sleep(0.1)
        raise RuntimeError(f'Timed out waiting for {description}')

    if positioning:
        # The production transparent overlay requires a live compositing manager.
        # A window manager alone is sufficient for GTK props, but not the goose.
        x11 = ctypes.CDLL('libX11.so.6')
        x11.XOpenDisplay.argtypes = [ctypes.c_char_p]
        x11.XOpenDisplay.restype = ctypes.c_void_p
        x11.XDefaultScreen.argtypes = [ctypes.c_void_p]
        x11.XDefaultScreen.restype = ctypes.c_int
        x11.XInternAtom.argtypes = [ctypes.c_void_p, ctypes.c_char_p, ctypes.c_int]
        x11.XInternAtom.restype = ctypes.c_ulong
        x11.XGetSelectionOwner.argtypes = [ctypes.c_void_p, ctypes.c_ulong]
        x11.XGetSelectionOwner.restype = ctypes.c_ulong
        x11.XCloseDisplay.argtypes = [ctypes.c_void_p]
        display = x11.XOpenDisplay(None)
        assert display, 'Private X11 display is unavailable'
        try:
            screen = x11.XDefaultScreen(display)
            atom = x11.XInternAtom(display, f'_NET_WM_CM_S{screen}'.encode(), 0)
            wait(lambda: x11.XGetSelectionOwner(display, atom), 'private X11 compositor selection', 15)
        finally:
            x11.XCloseDisplay(display)

    def nodes(pid):
        context = GLib.MainContext.default()
        for _ in range(100):
            if not context.pending():
                break
            context.iteration(False)
        desktop = Atspi.get_desktop(0)
        desktop.clear_cache()
        stack = [desktop.get_child_at_index(i) for i in range(desktop.get_child_count())]
        stack = [node for node in stack if node and node.get_process_id() == pid]
        result = []
        while stack and len(result) < 512:
            node = stack.pop()
            node.clear_cache()
            result.append(node)
            stack.extend(child for i in range(node.get_child_count()) if (child := node.get_child_at_index(i)))
        return result

    def delivered_text(pid):
        for node in nodes(pid):
            if node.get_name() == 'Note' and (interface := node.get_text_iface()):
                value = Atspi.Text.get_text(interface, 0, -1)
                if value.strip():
                    return value
        return None

    def owned_windows(pid):
        if not Path(f'/proc/{pid}').exists():
            return []
        if not positioning:
            return [node for node in nodes(pid) if node.get_role() == Atspi.Role.FRAME]
        result = subprocess.run(['xdotool', 'search', '--onlyvisible', '--pid', str(pid), '--name', '^Honk300 (note|picture)$'], capture_output=True, text=True)
        return result.stdout.splitlines() if result.returncode == 0 else []

    def capture_visible_note(pid, directory):
        from PIL import Image
        path = directory / 'delivered-note.png'
        prefix = ['import', '-window', 'root'] if positioning else ['grim']

        def visible():
            subprocess.run([*prefix, str(path)], check=True, timeout=10)
            if positioning:
                windows = owned_windows(pid)
                assert len(windows) == 1, windows
                output = subprocess.check_output(['xdotool', 'getwindowgeometry', '--shell', windows[0]], text=True)
                geometry = dict(line.split('=', 1) for line in output.splitlines())
                x, y, width, height = (int(geometry[key]) for key in ('X', 'Y', 'WIDTH', 'HEIGHT'))
                (directory / 'note-geometry.json').write_text(json.dumps(geometry, indent=2) + '\n')
                with Image.open(path).convert('RGB') as picture:
                    assert 0 <= x <= picture.width - width and 0 <= y <= picture.height - height, geometry
                    assert width <= picture.width * 0.48 and height <= picture.height * 0.48, geometry
                    # Native text readback is necessary but cannot prove a
                    # transparent overlay has not obscured the entire prop.
                    pixels = picture.crop((x, y, x + width, y + height))
                    painted = sum(max(pixel) > 12 for pixel in pixels.getdata())
                    return painted > width * height * 0.3
            # Wayland does not expose global prop coordinates. On this empty,
            # black test desktop, a real painted note is much larger than the
            # goose, while AT-SPI separately confirms its owned native text.
            with Image.open(path).convert('RGB') as picture:
                return sum(max(pixel) > 12 for pixel in picture.getdata()) > 25_000

        wait(visible, 'visible native note in the composited desktop', 15)

    config = evidence / 'config.toml'
    config.write_text('''goose_config_version = 2
[behavior]
first_wander_time_seconds = 600.0
[audio]
enabled = false
[safety]
no_mouse_steal = true
no_window_ride = true
pause_on_fullscreen = false
[schedule]
quiet_hours_enabled = false
dnd_respect = false
seasonal = false
autumn = false
''')
    results = []
    try:
        for cycle in range(2):
            directory = evidence / f'cycle-{cycle}'
            directory.mkdir()
            with (directory / 'runtime.log').open('w') as log:
                runtime = subprocess.Popen([str(binary), 'start', '--config', str(config),
                                            *([] if positioning else ['--wayland'])], stdout=log, stderr=log)
                host_pid = None
                try:
                    def ready():
                        if runtime.poll() is not None:
                            raise RuntimeError(f'Goose exited during startup: {(directory / "runtime.log").read_text()[-4000:]}')
                        status = control('status', check=False).stdout
                        if 'collect: failed' in status:
                            raise RuntimeError(f'Owned companion failed: {(directory / "runtime.log").read_text()[-4000:]}')
                        return 'collect: supported' in status
                    wait(ready, 'real prop readiness')
                    children = Path(f'/proc/{runtime.pid}/task/{runtime.pid}/children').read_text().split()
                    candidates = [int(pid) for pid in children if Path(f'/proc/{pid}/cmdline').read_bytes().split(b'\0')[:2] ==
                                  [os.fsencode(binary.with_name('honk300-settings')), b'--owned-props']]
                    assert len(candidates) == 1, candidates
                    host_pid = candidates[0]
                    control('do', 'note')
                    value = wait(lambda: delivered_text(host_pid), 'engine delivery and native note text', 100)
                    (directory / 'note.txt').write_text(value)
                    capture_visible_note(host_pid, directory)
                    assert owned_windows(host_pid), 'Delivered note has no native window'
                    status = control('status').stdout
                    assert 'collect: supported' in status
                    assert f'display backend: {"X11" if positioning else "native Wayland"}' in status, status
                    assert f'note/picture positioning: {"supported" if positioning else "unsupported"}' in status, status
                    if not positioning:
                        assert 'desktop (session hint): labwc' in status, status
                    service = subprocess.run([str(binary), '__settings-service', '--config', str(config)],
                        input=json.dumps({'protocol': 1, 'request_id': 78, 'command': {'op': 'status'}}),
                        capture_output=True, text=True, check=True, timeout=15)
                    details = json.loads(service.stdout)['data']['runtime']['session']
                    assert details['prop_positioning'] == ('supported' if positioning else 'unsupported'), details
                    caller = subprocess.run([str(binary), 'status'], capture_output=True, text=True, check=True, timeout=10,
                        env=dict(os.environ, XDG_CURRENT_DESKTOP='untrusted-caller', XDG_SESSION_TYPE='tty'))
                    assert f'desktop (session hint): {details["desktop_hint"]}' in caller.stdout, caller.stdout
                    (directory / 'session-status.json').write_text(json.dumps(details, indent=2) + '\n')
                    if not positioning:
                        status = control('status').stdout
                        assert 'cursor: unsupported' in status and 'window: unsupported' in status, status
                    if cycle == 0:
                        # Simulate the actual owned child disappearing, using a
                        # descriptor and fresh command identity to avoid PID reuse.
                        descriptor = os.pidfd_open(host_pid)
                        try:
                            assert Path(f'/proc/{host_pid}/cmdline').read_bytes().split(b'\0')[:2] == candidates_argv(binary)
                            signal.pidfd_send_signal(descriptor, signal.SIGTERM)
                        finally:
                            os.close(descriptor)
                        status = wait(lambda: (value if 'collect: failed' in (value := control('status').stdout) else None), 'failed child capability')
                        (directory / 'failed-child-status.txt').write_text(status)
                        assert 'running' in status
                        refused = control('do', 'note', check=False)
                        assert 'UNSUPPORTED' in (refused.stdout + refused.stderr).upper()
                        control('reload')
                        assert 'collect: failed' in control('status').stdout, 'Reload resurrected the failed child'
                    control('stop')
                    assert runtime.wait(timeout=60) == 0
                    if host_pid:
                        assert not owned_windows(host_pid)
                    results.append({'cycle': cycle, 'native_note_text': True, 'owned_child': True,
                                    'graceful_cleanup': True, 'failed_child_stays_failed': cycle == 0})
                finally:
                    if runtime.poll() is None:
                        control('stop', '--force', check=False)
                        try:
                            runtime.wait(timeout=10)
                        except subprocess.TimeoutExpired:
                            runtime.kill()
                            runtime.wait()
        standalone = evidence / 'without-companion'
        standalone.mkdir()
        shutil.copy2(binary, standalone / 'honk300')
        with (standalone / 'runtime.log').open('w') as log:
            runtime = subprocess.Popen([str(standalone / 'honk300'), 'start', '--config', str(config),
                                        *([] if positioning else ['--wayland'])], stdout=log, stderr=log)
            try:
                status = wait(lambda: (value if 'collect: unsupported' in (value := control('status', check=False).stdout) else None), 'standalone runtime without GTK companion')
                assert 'running' in status
                service = subprocess.run([str(standalone / 'honk300'), '__settings-service', '--config', str(config)],
                                         input=json.dumps({'protocol': 1, 'request_id': 77, 'command': {'op': 'read'}}),
                                         capture_output=True, text=True, check=True, timeout=15)
                response = json.loads(service.stdout)
                assert response['ok'] and len(response['data']['fields']) >= 50, response
                control('stop')
                assert runtime.wait(timeout=60) == 0
            finally:
                if runtime.poll() is None:
                    control('stop', '--force', check=False)
                    try:
                        runtime.wait(timeout=10)
                    except subprocess.TimeoutExpired:
                        runtime.kill()
                        runtime.wait()
        (evidence / 'result.json').write_text(json.dumps({'ok': True, 'cycles': results,
            'without_companion': {'runtime': True, 'shared_settings_service': True, 'props': 'unsupported'}}, indent=2) + '\n')
    finally:
        Atspi.exit()


def candidates_argv(binary):
    return [os.fsencode(binary.with_name('honk300-settings')), b'--owned-props']


if __name__ == '__main__':
    main()
