#!/usr/bin/env python3
"""Qualify real GTK prop windows on a disposable X11 desktop, never the user's desktop."""
from __future__ import annotations
import argparse
import base64
import json
import os
from pathlib import Path
import selectors
import subprocess
import time


class Host:
    def __init__(self, binary, directory, positioning=True):
        self.directory = directory
        directory.mkdir()
        self.log = (directory / 'process.log').open('w')
        self.process = subprocess.Popen([str(binary), '--owned-props'], stdin=subprocess.PIPE,
                                        stdout=subprocess.PIPE, stderr=self.log)
        self.selector = selectors.DefaultSelector()
        self.selector.register(self.process.stdout, selectors.EVENT_READ)
        self.buffer = b''
        self.events = []
        try:
            self.wait(lambda event: event.get('event') == 'ready' and event.get('positioning') is positioning)
        except BaseException:
            self.close()
            raise

    def send(self, op, **fields):
        self.process.stdin.write((json.dumps({'v': 1, 'op': op, **fields}) + '\n').encode())
        self.process.stdin.flush()

    def wait(self, predicate, timeout=20):
        deadline = time.monotonic() + timeout
        while time.monotonic() < deadline:
            if b'\n' in self.buffer:
                line, self.buffer = self.buffer.split(b'\n', 1)
                try:
                    event = json.loads(line)
                except json.JSONDecodeError:
                    (self.directory / 'invalid-protocol.txt').write_text(repr(line) + '\n')
                    raise
                self.events.append(event)
                (self.directory / 'events.json').write_text(json.dumps(self.events, indent=2) + '\n')
                if predicate(event):
                    return event
            elif self.selector.select(timeout=0.1):
                chunk = os.read(self.process.stdout.fileno(), 65536)
                if not chunk:
                    raise RuntimeError(f'Prop host exited: {self.process.poll()}; {self.directory / "process.log"}')
                self.buffer += chunk
        raise RuntimeError(f'Missing expected native event; received {self.events[-8:]}')

    def window(self, identity):
        return self.wait(lambda event: event.get('event') == 'window' and event.get('id') == identity and event['alive'])

    def close(self):
        try:
            self.process.stdin.close()
        except BrokenPipeError:
            pass
        if self.process.poll() is None:
            try:
                self.process.wait(timeout=10)
            except subprocess.TimeoutExpired:
                self.process.kill()
                self.process.wait()
        self.selector.close()
        self.process.stdout.close()
        self.log.close()


def capture_complete_picture(host, capture_prefix=('import', '-window', 'root')):
    from PIL import Image
    colors = [(255, 255, 0), (0, 255, 0), (255, 0, 0), (0, 0, 255)]
    deadline = time.monotonic() + 15
    while time.monotonic() < deadline:
        path = host.directory / 'desktop.png'
        subprocess.run([*capture_prefix, str(path)], check=True)
        with Image.open(path).convert('RGB') as image:
            points = {color: [] for color in colors}
            for y in range(image.height):
                for x in range(image.width):
                    color = image.getpixel((x, y))
                    if color in points:
                        points[color].append((x, y))
        if all(len(points[color]) > 100 for color in colors):
            boxes = [(min(x for x, y in points[color]), min(y for x, y in points[color]),
                      max(x for x, y in points[color]), max(y for x, y in points[color])) for color in colors]
            yellow, green, red, blue = boxes
            assert abs(yellow[0] - red[0]) <= 1 and abs(green[0] - blue[0]) <= 1, boxes
            assert abs(yellow[1] - green[1]) <= 1 and abs(red[1] - blue[1]) <= 1, boxes
            widths = [box[2] - box[0] + 1 for box in boxes]
            heights = [box[3] - box[1] + 1 for box in boxes]
            assert max(widths) - min(widths) <= 2 and max(heights) - min(heights) <= 2, boxes
            width, height = blue[2] - yellow[0] + 1, blue[3] - yellow[1] + 1
            assert abs(width / height - 1.5) < 0.04 and width <= 180 and height <= 120, boxes
            (host.directory / 'picture-pixels.json').write_text(json.dumps({'quadrants': boxes, 'width': width, 'height': height}) + '\n')
            return
        time.sleep(0.1)
    raise RuntimeError('The complete picture never appeared in the native capture')


def main():
    if os.environ.get('GITHUB_ACTIONS') != 'true' or os.environ.get('GDK_BACKEND') not in ('x11', 'wayland'):
        raise RuntimeError('Use the disposable GitHub X11 or Wayland fixture')
    positioning = os.environ['GDK_BACKEND'] == 'x11'
    import gi
    gi.require_version('Atspi', '2.0')
    from gi.repository import Atspi, GLib
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--binary', required=True, type=Path)
    parser.add_argument('--evidence', required=True, type=Path)
    args = parser.parse_args()
    evidence = args.evidence.resolve()
    evidence.mkdir(parents=True, exist_ok=True)
    subprocess.run(['gdbus', 'call', '--session', '--dest', 'org.a11y.Bus', '--object-path', '/org/a11y/bus',
                    '--method', 'org.freedesktop.DBus.Properties.Set', 'org.a11y.Status', 'IsEnabled', '<true>'], check=True)
    Atspi.init()

    def read_note(pid, expected):
        deadline = time.monotonic() + 15
        while time.monotonic() < deadline:
            context = GLib.MainContext.default()
            for _ in range(100):
                if not context.pending():
                    break
                context.iteration(False)
            root = Atspi.get_desktop(0)
            root.clear_cache()
            stack = [root.get_child_at_index(i) for i in range(root.get_child_count())]
            stack = [node for node in stack if node and node.get_process_id() == pid]
            inspected = 0
            while stack and inspected < 512:
                node = stack.pop()
                node.clear_cache()
                inspected += 1
                if node.get_name() == 'Note':
                    text = node.get_text_iface()
                    # Noble's GI object also has Accessible.get_text(), whose
                    # deprecated no-argument method shadows Text.get_text().
                    if text and Atspi.Text.get_text(text, 0, -1) == expected:
                        return
                stack.extend(child for i in range(node.get_child_count())
                             if (child := node.get_child_at_index(i)) is not None)
            time.sleep(0.1)
        raise RuntimeError('The native GTK text interface did not retain the note')

    def native_windows(pid):
        if not Path(f'/proc/{pid}').exists():
            return []
        if not positioning:
            context = GLib.MainContext.default()
            for _ in range(100):
                if not context.pending():
                    break
                context.iteration(False)
            root = Atspi.get_desktop(0)
            root.clear_cache()
            stack = [root.get_child_at_index(i) for i in range(root.get_child_count())]
            stack = [node for node in stack if node and node.get_process_id() == pid]
            frames = []
            while stack:
                node = stack.pop()
                node.clear_cache()
                if node.get_role() == Atspi.Role.FRAME:
                    frames.append(node)
                stack.extend(child for i in range(node.get_child_count())
                             if (child := node.get_child_at_index(i)) is not None)
            return frames
        result = subprocess.run(['xdotool', 'search', '--onlyvisible', '--pid', str(pid), '--name', '^Honk300 (note|picture)$'],
                                capture_output=True, text=True)
        return result.stdout.splitlines() if result.returncode == 0 else []

    def require_window_count(pid, count):
        deadline = time.monotonic() + 15
        while time.monotonic() < deadline:
            if len(windows := native_windows(pid)) == count:
                return windows
            time.sleep(0.1)
        raise RuntimeError(f'Expected {count} native owned windows; observed {len(windows)}')

    def user_close_note(host):
        # xdotool windowclose uses XDestroyWindow, which bypasses GTK's close
        # handling entirely. Activate the real, process-owned native Close
        # button so this tests the user-close signal and preserves other notes.
        deadline = time.monotonic() + 10
        while time.monotonic() < deadline:
            context = GLib.MainContext.default()
            for _ in range(100):
                if not context.pending():
                    break
                context.iteration(False)
            root = Atspi.get_desktop(0)
            root.clear_cache()
            stack = [root.get_child_at_index(i) for i in range(root.get_child_count())]
            stack = [node for node in stack if node and node.get_process_id() == host.process.pid]
            records = []
            while stack and len(records) < 512:
                node = stack.pop()
                node.clear_cache()
                action = node.get_action_iface()
                count = Atspi.Action.get_n_actions(action) if action else 0
                record = {'name': node.get_name(), 'role': node.get_role_name(), 'interfaces': list(node.get_interfaces()),
                          'actions': [Atspi.Action.get_action_name(action, i) for i in range(count)]}
                records.append(record)
                if node.get_name() == 'Close' and node.get_role() == Atspi.Role.PUSH_BUTTON:
                    if action and Atspi.Action.do_action(action, 0):
                        (host.directory / 'close-tree.json').write_text(json.dumps(records, indent=2) + '\n')
                        return
                stack.extend(child for i in range(node.get_child_count())
                             if (child := node.get_child_at_index(i)) is not None)
            (host.directory / 'close-tree.json').write_text(json.dumps(records, indent=2) + '\n')
            time.sleep(0.1)
        subprocess.run([*capture_prefix, str(host.directory / 'failed-close.png')], check=True)
        raise RuntimeError('Native owned Close action is unavailable')


    capture_prefix = ('import', '-window', 'root') if positioning else ('grim',)
    results = []
    try:
        for cycle in range(2):
            host = Host(args.binary.resolve(), evidence / f'cycle-{cycle}', positioning)
            owned = []
            try:
                for identity in range(1, 9):
                    host.send('note', id=identity, x=50 + identity * 12, y=50 + identity * 8,
                              width=400, height=250, title='A note from your goose')
                    event = host.window(identity)
                    assert event['width'] <= 400 and event['height'] <= 250, event
                    if not positioning:
                        assert event['x'] == event['y'] == 0, 'Wayland invented global coordinates'
                require_window_count(host.process.pid, 8)
                host.send('text', id=1, text='Café 🦆\nKeep this note.')
                read_note(host.process.pid, 'Café 🦆\nKeep this note.')
                host.send('note', id=9, width=400, height=250, title='No room yet')
                host.wait(lambda event: event.get('event') == 'busy' and event.get('id') == 9)
                require_window_count(host.process.pid, 8)
                read_note(host.process.pid, 'Café 🦆\nKeep this note.')
                if positioning:
                    host.send('move', id=1, x=500, y=300)
                    host.wait(lambda event: event.get('id') == 1 and event.get('x') == 500 and event.get('y') == 300)
                owned = require_window_count(host.process.pid, 8)
                user_close_note(host)
                closed = host.wait(lambda event: event.get('event') == 'window' and not event['alive'] and event['origin'] == 'user')
                assert closed['id'] in range(1, 9)
                host.send('note', id=10, x=40, y=40, width=400, height=250, title='Room again')
                host.window(10)
                require_window_count(host.process.pid, 8)
                host.send('close', id=10)
                host.wait(lambda event: event.get('id') == 10 and not event['alive'] and event['origin'] == 'program')
                # Distinct colored corners expose stretching/cropping in the native capture.
                pixels = bytearray()
                for y in range(120):
                    for x in range(180):
                        pixels.extend((255 if x < 90 else 0, 255 if y < 60 else 0, 255 if x >= 90 and y >= 60 else 0, 255))
                host.send('image', id=11, x=700, y=100, width=180, height=154,
                          pixel_width=180, pixel_height=120, title='All four corners', pixels=base64.b64encode(pixels).decode())
                picture = host.window(11)
                assert picture['width'] <= 180 and picture['height'] <= 154
                capture_complete_picture(host, capture_prefix)
                host.send('shutdown')
                assert host.process.wait(timeout=10) == 0
                assert not native_windows(host.process.pid)
                results.append({'cycle': cycle, 'capacity': 8, 'native_text': True, 'owned_move': positioning,
                                'user_close': True, 'program_close': True, 'recovery': True,
                                'complete_picture_capture': True, 'shutdown_removed_windows': True})
            finally:
                host.close()
        disconnected = Host(args.binary.resolve(), evidence / 'disconnect', positioning)
        try:
            disconnected.send('note', id=1, x=80, y=80, width=400, height=250, title='Connection-owned note')
            disconnected.window(1)
            disconnected.process.stdin.close()
            assert disconnected.process.wait(timeout=10) == 0
            assert not native_windows(disconnected.process.pid)
        finally:
            disconnected.close()
        rejected = []
        for failure in ('unknown-field', 'oversized-frame'):
            broken = Host(args.binary.resolve(), evidence / failure, positioning)
            try:
                broken.send('note', id=1, x=80, y=80, width=400, height=250, title='Connection-owned note')
                broken.window(1)
                if failure == 'unknown-field':
                    broken.send('close', id=1, foreign_window_id=123456)
                else:
                    try:
                        broken.process.stdin.write(b'x' * (4 * 1024 * 1024 + 1))
                        broken.process.stdin.flush()
                    except BrokenPipeError:
                        pass
                assert broken.process.wait(timeout=15) != 0, 'Malformed input did not fail the connection'
                assert not native_windows(broken.process.pid)
                rejected.append(failure)
            finally:
                broken.close()
        (evidence / 'result.json').write_text(json.dumps({'schema': 'honk300.owned-props.v1', 'ok': True,
            'cycles': results, 'stdin_eof_removed_windows': True,
            'malformed_input_closed_owned_windows': rejected,
            'proof': f'native GTK on disposable {os.environ["GDK_BACKEND"]}'}, indent=2) + '\n')
    finally:
        Atspi.exit()


if __name__ == '__main__':
    main()
