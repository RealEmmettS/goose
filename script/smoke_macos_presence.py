#!/usr/bin/env python3
"""Actual Mac fullscreen/IPC/manners qualification on a disposable CI desktop."""
import argparse
import json
import os
from pathlib import Path
import signal
import subprocess
import time


def main():
    assert os.environ.get('GITHUB_ACTIONS') == 'true'
    assert os.uname().sysname == 'Darwin'
    parser = argparse.ArgumentParser()
    parser.add_argument('--binary', type=Path, required=True)
    parser.add_argument('--fixture', type=Path, required=True)
    parser.add_argument('--evidence', type=Path, required=True)
    args = parser.parse_args()
    root = args.evidence.resolve()
    root.mkdir(parents=True, exist_ok=True)
    binary, fixture = args.binary.resolve(), args.fixture.resolve()
    environment = dict(os.environ, HONK300_TRACE_PRESENCE='1')
    config = root / 'config.toml'
    log_path = root / 'runtime.log'
    def write_config(enabled):
        config.write_text('goose_config_version = 2\n[behavior]\nfirst_wander_time_seconds = 600.0\n'
            '[audio]\nenabled = false\n[safety]\nno_mouse_steal = true\nno_window_ride = true\n'
            f'pause_on_fullscreen = {str(enabled).lower()}\n'
            '[schedule]\nquiet_hours_enabled = false\nautumn = false\nseasonal = false\ndnd_respect = true\n')
    def cli(*words):
        return subprocess.run([str(binary), *words], env=environment, text=True,
                              capture_output=True, timeout=12)
    def wait(check, description, timeout=8):
        until = time.monotonic() + timeout
        while time.monotonic() < until:
            value = check()
            if value:
                return value
            time.sleep(.05)
        raise AssertionError('Timed out waiting for ' + description)
    def trace(check):
        lines = log_path.read_text(errors='replace').splitlines(keepends=True)
        values = [json.loads(line.split('honk300 macos presence trace: ', 1)[1])
                  for line in lines if line.endswith('\n') and 'honk300 macos presence trace: ' in line]
        return values and values[-1] if values and check(values[-1]) else None
    sequence = 0
    def command(op, phase):
        nonlocal sequence
        sequence += 1
        temporary = root / 'command.pending'
        temporary.write_text(json.dumps(dict(sequence=sequence, op=op)))
        temporary.replace(root / 'command.json')
        def completed():
            state = json.loads((root / 'fixture.json').read_text())
            return state if state['phase'] == phase and state['command'] == sequence else None
        return wait(completed, 'actual native fixture ' + phase)
    def status():
        value = cli('status')
        assert value.returncode == 0, value.stderr
        assert 'do not disturb observation: unsupported' in value.stdout, value.stdout
        return value.stdout
    stopped_fixture = False
    goose = native = None
    write_config(True)
    try:
        with log_path.open('w') as log:
            goose = subprocess.Popen([str(binary), 'start', '--config', str(config),
                '--no-sound', '--no-window-ride'], env=environment, stdout=log, stderr=log)
            wait(lambda: 'honk300: running' in cli('status').stdout, 'real runtime IPC')
            native = subprocess.Popen([str(fixture), 'controlled', str(root)], env=environment,
                                      stdout=log, stderr=log)
            wait(lambda: (root / 'fixture.json').exists(), 'native private window')
            wait(lambda: trace(lambda v: v['fullscreen_status'] == 'supported' and not v['fullscreen'] and not v['manners']), 'normal native state')
            assert 'fullscreen observation: supported' in status()
            command('fullscreen', 'fullscreen')
            wait(lambda: trace(lambda v: v['fullscreen'] and v['manners']), 'real fullscreen pauses manners')
            subprocess.run(['/usr/sbin/screencapture', '-x', str(root / 'native-fullscreen.png')], check=True, timeout=5)

            # A stopped native target must not block the actual AppKit loop, IPC,
            # or graceful stop, and a previous fullscreen result must expire.
            os.kill(native.pid, signal.SIGSTOP)
            stopped_fixture = True
            wait(lambda: trace(lambda v: v['fullscreen_status'] == 'failed' and not v['fullscreen'] and not v['manners']), 'stopped target withdraws fullscreen', 3)
            started = time.monotonic()
            failed_status = status()
            responsive = time.monotonic() - started
            assert responsive < 1, responsive
            assert 'fullscreen observation: failed' in failed_status, failed_status
            os.kill(native.pid, signal.SIGCONT)
            stopped_fixture = False
            wait(lambda: trace(lambda v: v['fullscreen'] and v['manners']), 'native target recovery')
            write_config(False)
            assert cli('reload').returncode == 0
            wait(lambda: trace(lambda v: v['fullscreen'] and not v['manners']), 'live fullscreen opt-out')
            write_config(True)
            assert cli('reload').returncode == 0
            wait(lambda: trace(lambda v: v['fullscreen'] and v['manners']), 'live fullscreen opt-in')
            command('restore', 'restored')
            wait(lambda: trace(lambda v: v['fullscreen_status'] == 'supported' and not v['fullscreen'] and not v['manners']), 'actual fullscreen exit')
            command('hide', 'hidden')
            wait(lambda: trace(lambda v: v['fullscreen_status'] != 'supported' and not v['fullscreen'] and not v['manners']), 'missing focused window fallback')
            command('show', 'normal')
            wait(lambda: trace(lambda v: v['fullscreen_status'] == 'supported' and not v['fullscreen']), 'focused window return')
            command('fullscreen', 'fullscreen')
            wait(lambda: trace(lambda v: v['fullscreen'] and v['manners']), 'second fullscreen')
            os.kill(native.pid, signal.SIGSTOP)
            stopped_fixture = True
            wait(lambda: trace(lambda v: v['fullscreen_status'] == 'failed'), 'second target interruption', 3)
            started = time.monotonic()
            assert cli('stop').returncode == 0
            goose.wait(timeout=12)
            shutdown = time.monotonic() - started
            assert goose.returncode == 0 and shutdown < 12, (goose.returncode, shutdown)
            os.kill(native.pid, signal.SIGCONT)
            stopped_fixture = False
            command('close', 'done')
            native.wait(timeout=4)
            (root / 'result.json').write_text(json.dumps(dict(ok=True,
                status_during_stopped_target_seconds=responsive, graceful_stop_seconds=shutdown,
                fullscreen_transition=True, live_settings=True, unavailable_window=True,
                stopped_target=True, dnd='unsupported'), indent=2))
    finally:
        if native is not None and native.poll() is None:
            if stopped_fixture: os.kill(native.pid, signal.SIGCONT)
            native.terminate()
            try: native.wait(timeout=4)
            except subprocess.TimeoutExpired: native.kill(); native.wait(timeout=4)
        if goose is not None and goose.poll() is None:
            cli('stop')
            try: goose.wait(timeout=12)
            except subprocess.TimeoutExpired: goose.kill(); goose.wait(timeout=4)


if __name__ == '__main__':
    main()
