"""Exercise the real vendored board server against disposable task files."""
import json
from pathlib import Path
import shutil
import socket
import subprocess
import tempfile
import time
import unittest
from urllib.error import HTTPError
from urllib.request import Request, urlopen


class TaskBoardTransactions(unittest.TestCase):
    def setUp(self):
        if not shutil.which('node'):
            self.skipTest('Node is required for native board transactions')
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.directory = Path(self.temp.name)
        shutil.copy2(Path(__file__).resolve().parents[2] / '.tasks/board-server.mjs', self.directory)
        (self.directory / 'tasks').mkdir()
        self.board = '# Tasks\n\n## Active\n- [ ] **Fixture task** #ab\n'
        (self.directory / 'TASKS.md').write_text(self.board)
        self.detail = self.directory / 'tasks/ab.md'
        self.detail.write_text('## Status\nActive\n\n## Activity\nCreated\n')
        with socket.socket() as listener:
            listener.bind(('127.0.0.1', 0))
            port = listener.getsockname()[1]
        self.process = subprocess.Popen(['node', str(self.directory / 'board-server.mjs'), 'serve', '--port', str(port)],
            stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL,
            creationflags=getattr(subprocess, 'CREATE_NO_WINDOW', 0))
        self.addCleanup(self.stop)
        state = self.directory / '.board-server.json'
        deadline = time.monotonic() + 10
        while time.monotonic() < deadline and not state.exists():
            if self.process.poll() is not None:
                self.fail('Board server exited before readiness')
            time.sleep(0.02)
        self.url = 'http://127.0.0.1:' + str(json.loads(state.read_text())['port'])

    def stop(self):
        self.process.terminate()
        self.process.wait(timeout=10)

    def request(self, path, body=None, headers=None):
        request = Request(self.url + path, data=body.encode() if body is not None else None, headers=headers or {})
        try:
            response = urlopen(request, timeout=5)
        except HTTPError as error:
            response = error
        with response:
            return response.status, response.read().decode(), response.headers

    def test_stale_detail_deletion_preserves_new_activity(self):
        _, _, board_headers = self.request('/api/tasks')
        _, _, detail_headers = self.request('/api/task?id=ab')
        newer = self.detail.read_text() + '\nNew concurrent activity\n'
        self.detail.write_text(newer)
        status, _, _ = self.request('/api/tasks', '# Tasks\n', {
            'X-Base-Board-Revision': board_headers['X-Board-Revision'],
            'X-Delete-Task-Id': 'ab', 'X-Delete-Detail-Revision': detail_headers['X-Detail-Revision']})
        self.assertEqual(status, 412)
        self.assertEqual(self.detail.read_text(), newer)
        self.assertEqual((self.directory / 'TASKS.md').read_text(), self.board)

    def test_missing_detail_cannot_complete(self):
        self.detail.unlink()
        _, _, board_headers = self.request('/api/tasks')
        _, _, detail_headers = self.request('/api/task?id=ab')
        status, _, _ = self.request('/api/tasks', self.board.replace('[ ]', '[x]'), {
            'X-Base-Board-Revision': board_headers['X-Board-Revision'],
            'X-Completion-Task-Id': 'ab', 'X-Completion-Detail-Revision': detail_headers['X-Detail-Revision']})
        self.assertEqual(status, 412)
        self.assertEqual((self.directory / 'TASKS.md').read_text(), self.board)

    def test_current_revision_deletion_removes_only_the_fixture(self):
        _, _, board_headers = self.request('/api/tasks')
        _, _, detail_headers = self.request('/api/task?id=ab')
        status, _, _ = self.request('/api/tasks', '# Tasks\n', {
            'X-Base-Board-Revision': board_headers['X-Board-Revision'],
            'X-Delete-Task-Id': 'ab', 'X-Delete-Detail-Revision': detail_headers['X-Detail-Revision']})
        self.assertEqual(status, 200)
        self.assertFalse(self.detail.exists())
        self.assertEqual((self.directory / 'TASKS.md').read_text(), '# Tasks\n')
