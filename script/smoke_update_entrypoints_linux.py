#!/usr/bin/env python3
"""Disposable Debian proof of the real CLI, TUI and native GUI updater paths.

The terminal-launch adapter records the exact helper process and its retained output;
the real updater still uses the public manifest, exact-tag downloads and dpkg. No
installer or receipt is mocked. This is hosted process/AT-SPI proof, not a physical
terminal-window or administrator-prompt acceptance test.
"""
from __future__ import annotations

import argparse
import fcntl
import json
import os
from pathlib import Path
import pty
import select
import signal
import struct
import subprocess
import termios
import time


INSTALLED = Path('/usr/lib/honk300/honk300')
RECEIPT = INSTALLED.with_name('install-receipt.json')


def run(*args, **kwargs):
    return subprocess.run([str(arg) for arg in args], check=True, timeout=300, **kwargs)


def wait(check, description, timeout=40):
    deadline = time.monotonic() + timeout
    while time.monotonic() < deadline:
        result = check()
        if result:
            return result
        time.sleep(0.1)
    raise RuntimeError(f'Timed out waiting for {description}')


def text(path):
    return path.read_text(errors='replace') if path.exists() else ''


def running():
    result = subprocess.run([str(INSTALLED), 'status'], capture_output=True, text=True, timeout=10)
    return result.returncode == 0 and 'honk300: running' in result.stdout


def stop_process(process):
    if process is not None and process.poll() is None:
        process.terminate()
        try:
            process.wait(timeout=10)
        except subprocess.TimeoutExpired:
            process.kill()
            process.wait()


def stop_helper(directory):
    pid_file = directory / 'helper.pid'
    if pid_file.exists():
        pid = int(pid_file.read_text())
        # This PID is emitted immediately before exec by our private terminal
        # adapter. Check the command identity before sending it any signal.
        command = Path(f'/proc/{pid}/cmdline')
        if command.exists():
            argv = command.read_bytes().split(b'\0')
            if argv[:2] != [os.fsencode(INSTALLED), b'__control-surface-update']:
                raise RuntimeError('Retained helper PID changed identity')
            os.kill(pid, signal.SIGTERM)
            def exited():
                try:
                    return Path(f'/proc/{pid}/stat').read_text().split()[2] == 'Z'
                except FileNotFoundError:
                    return True
            wait(exited, 'owned helper exit', 10)
        pid_file.unlink()


def verify_receipt(expected_version):
    receipt = json.loads(RECEIPT.read_text())
    assert receipt['schema'] == 'honk300.install.v2'
    assert receipt['version'] == expected_version
    assert receipt['origin'] == receipt['installer_family'] == 'deb'
    assert receipt['active_release'] == str(INSTALLED.parent)
    owner = run('dpkg-query', '--search', INSTALLED, capture_output=True, text=True).stdout.strip()
    assert owner == f'honk300: {INSTALLED}'
    return receipt


def check_updates():
    response = json.loads(run(INSTALLED, 'update', '--check', '--json', capture_output=True, text=True).stdout)
    assert response['success']
    assert response['check']['available'] and response['check']['managed']
    return response['check']['latest_version']


def wait_helper(directory, success):
    expected = 'Update complete.' if success else 'HONK! NEEDS ATTENTION'
    wait(lambda: expected in text(directory / 'helper.stdout.txt') + text(directory / 'helper.stderr.txt'),
         f'helper result {expected}', 300)
    pid = int((directory / 'helper.pid').read_text())
    os.kill(pid, 0)  # The result remains held after the transaction completes.
    assert running(), 'The helper did not preserve/relaunch the runtime'
    if success:
        assert 'Honk300 has restarted.' in text(directory / 'helper.stdout.txt')


class TerminalEditor:
    def __init__(self, config, directory, environment):
        master, slave = pty.openpty()
        fcntl.ioctl(slave, termios.TIOCSWINSZ, struct.pack('HHHH', 35, 120, 0, 0))
        self.master = master
        self.output = ''
        self.directory = directory
        self.process = subprocess.Popen([str(INSTALLED), 'config', '--config', str(config)],
                                        stdin=slave, stdout=slave, stderr=slave,
                                        env=environment, start_new_session=True)
        os.close(slave)
        self.until('honk300 config')

    def until(self, needle):
        def observed():
            if select.select([self.master], [], [], 0)[0]:
                self.output += os.read(self.master, 65536).decode(errors='replace')
            if self.process.poll() is not None:
                raise RuntimeError('Terminal editor exited before the update handoff')
            return needle in self.output
        wait(observed, f'terminal text {needle}')
        (self.directory / 'terminal.txt').write_text(self.output)

    def key(self, key):
        os.write(self.master, key.encode())

    def close(self):
        stop_process(self.process)
        os.close(self.master)


class NativeSettings:
    def __init__(self, config, directory, environment):
        import gi
        gi.require_version('Atspi', '2.0')
        from gi.repository import Atspi, GLib
        self.Atspi, self.GLib = Atspi, GLib
        run('gdbus', 'call', '--session', '--dest', 'org.a11y.Bus', '--object-path', '/org/a11y/bus',
            '--method', 'org.freedesktop.DBus.Properties.Set', 'org.a11y.Status', 'IsEnabled', '<true>',
            capture_output=True)
        Atspi.init()
        self.log = (directory / 'settings.log').open('w')
        self.process = subprocess.Popen([str(INSTALLED.with_name('honk300-settings')), '--config', str(config)],
                                        stdout=self.log, stderr=self.log, env=environment)
        self.find('Platform & status', actionable=True)

    def nodes(self):
        context = self.GLib.MainContext.default()
        for _ in range(100):
            if not context.pending():
                break
            context.iteration(False)
        desktop = self.Atspi.get_desktop(0)
        desktop.clear_cache()
        stack = [desktop.get_child_at_index(i) for i in range(desktop.get_child_count())]
        stack = [node for node in stack if node and node.get_process_id() == self.process.pid]
        found = []
        while stack and len(found) < 640:
            node = stack.pop()
            node.clear_cache()
            found.append(node)
            stack.extend(child for i in range(node.get_child_count())
                         if (child := node.get_child_at_index(i)) is not None)
        return found

    def find(self, name, actionable=False):
        def lookup():
            if self.process.poll() is not None:
                raise RuntimeError('Native settings exited before update handoff')
            return next((node for node in self.nodes() if node.get_name() == name and
                         (not actionable or (node.get_role() == self.Atspi.Role.PUSH_BUTTON and
                          node.get_state_set().contains(self.Atspi.StateType.ENABLED)))), None)
        return wait(lookup, f'native settings control {name}')

    def invoke(self, name):
        assert self.find(name, actionable=True).get_action_iface().do_action(0)

    def close(self):
        stop_process(self.process)
        self.log.close()
        self.Atspi.exit()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--package', type=Path, required=True)
    parser.add_argument('--fixture-version', required=True)
    parser.add_argument('--evidence', type=Path, required=True)
    args = parser.parse_args()
    if os.environ.get('GITHUB_ACTIONS') != 'true' or os.geteuid() == 0:
        raise RuntimeError('Run only as the ordinary user on a disposable GitHub Linux runner')
    evidence = args.evidence.resolve()
    evidence.mkdir(parents=True, exist_ok=True)
    launcher_dir = evidence / 'terminal-adapter'
    launcher_dir.mkdir()
    launcher = launcher_dir / 'xdg-terminal-exec'
    launcher.write_text('''#!/bin/sh
set -eu
test "$#" = 2
test "$1" = /usr/lib/honk300/honk300
test "$2" = __control-surface-update
printf '%s\\n' "$$" > "$HONK300_ENTRYPOINT_EVIDENCE/helper.pid"
if test -e "$HONK300_ENTRYPOINT_EVIDENCE/offline"; then
  export https_proxy=http://127.0.0.1:1 http_proxy=http://127.0.0.1:1
  export HTTPS_PROXY="$https_proxy" HTTP_PROXY="$http_proxy" ALL_PROXY="$http_proxy" NO_PROXY= no_proxy=
fi
exec "$@" > "$HONK300_ENTRYPOINT_EVIDENCE/helper.stdout.txt" 2> "$HONK300_ENTRYPOINT_EVIDENCE/helper.stderr.txt"
''')
    launcher.chmod(0o700)
    results = []
    for mode in ('cli', 'tui', 'gui'):
        directory = evidence / mode
        directory.mkdir()
        config = directory / 'config.toml'
        config.write_text('goose_config_version = 2\n[audio]\nenabled = false\n[safety]\nno_mouse_steal = true\nno_window_ride = true\n')
        environment = dict(os.environ, PATH=str(launcher_dir) + os.pathsep + os.environ['PATH'],
                           HONK300_ENTRYPOINT_EVIDENCE=str(directory), TERM='xterm-256color')
        runtime = editor = None
        runtime_log = (directory / 'runtime.log').open('w')
        try:
            run('sudo', 'dpkg', '--remove', 'honk300', capture_output=True)
            run('sudo', 'apt-get', 'install', '--yes', args.package)
            verify_receipt(args.fixture_version)
            before = RECEIPT.read_bytes()
            latest = check_updates()
            runtime = subprocess.Popen([str(INSTALLED), 'start', '--config', str(config)],
                                       stdout=runtime_log, stderr=runtime_log, env=environment)
            wait(running, 'fixture runtime readiness')
            if mode == 'cli':
                result = run(INSTALLED, 'update', '--json', capture_output=True, text=True)
                (directory / 'update.stdout.json').write_text(result.stdout)
                (directory / 'update.stderr.txt').write_text(result.stderr)
                assert json.loads(result.stdout)['result'] == 'updated'
            elif mode == 'tui':
                editor = TerminalEditor(config, directory, environment)
                editor.key('c')
                editor.until('Update available:')
                editor.key('i')
                wait(lambda: (directory / 'helper.pid').exists(), 'TUI helper process')
                wait_helper(directory, success=True)
            else:
                editor = NativeSettings(config, directory, environment)
                editor.invoke('Platform & status')
                editor.invoke('Check for updates')
                editor.find('Update now', actionable=True)
                (directory / 'offline').touch()
                editor.invoke('Update now')
                wait_helper(directory, success=False)
                assert RECEIPT.read_bytes() == before, 'Failed GUI update changed the receipt'
                (directory / 'failure.stderr.txt').write_text(text(directory / 'helper.stderr.txt'))
                stop_helper(directory)
                (directory / 'offline').unlink()
                editor.invoke('Check for updates')
                editor.invoke('Update now')
                wait_helper(directory, success=True)
            verify_receipt(latest)
            no_op = run(INSTALLED, 'update', '--json', capture_output=True, text=True)
            assert json.loads(no_op.stdout)['result'] == 'up_to_date'
            (directory / 'no-op.json').write_text(no_op.stdout)
            results.append({'entrypoint': mode, 'fixture': args.fixture_version, 'updated': latest,
                            'receipt': 'verified', 'result': 'updated', 'public_no_op': True,
                            'helper_relaunch': mode != 'cli', 'failed_update_preserved_state': mode == 'gui'})
        finally:
            if editor is not None:
                editor.close()
            stop_helper(directory)
            if INSTALLED.exists():
                subprocess.run([str(INSTALLED), 'stop', '--force'], capture_output=True, timeout=15)
            stop_process(runtime)
            runtime_log.close()
    (evidence / 'result.json').write_text(json.dumps({'ok': True, 'checks': results}, indent=2) + '\n')
    print('CLI, TUI and native GUI updater entrypoints passed with real Debian transactions.')


if __name__ == '__main__':
    main()
