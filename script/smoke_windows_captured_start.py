#!/usr/bin/env python3
"""Prove captured Windows start commands reach EOF while their runtime stays alive.

Uses one isolated config, disables sound and external input/window manipulation,
and refuses to replace an existing runtime. Does not install or capture the desktop.
"""
from __future__ import annotations

import argparse
import json
import os
from pathlib import Path
import subprocess
import tempfile
import time


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--binary', type=Path, required=True)
    parser.add_argument('--evidence', type=Path, required=True)
    args = parser.parse_args()
    assert os.name == 'nt', 'Windows fixture only'
    binary = args.binary.resolve(strict=True)
    launcher = binary.with_name('honk300-app.exe').resolve(strict=True)
    evidence = args.evidence.resolve()
    evidence.mkdir(parents=True, exist_ok=True)
    hidden = subprocess.CREATE_NO_WINDOW

    def control(*arguments):
        return subprocess.run([str(binary), *arguments], capture_output=True, text=True,
                              timeout=20, check=True, creationflags=hidden).stdout

    assert 'honk300: not running' in control('status'), 'preserve the existing runtime'
    results = []
    with tempfile.TemporaryDirectory(prefix='honk300-captured-start-') as temporary:
        config = Path(temporary) / 'config.toml'
        config.write_text('goose_config_version = 2\n[behavior]\nfirst_wander_time_seconds = 180.0\n'
                          '[mischief]\ncollect_windows = false\n', encoding='utf-8')
        options = ['--config', str(config), '--no-sound', '--no-mouse-steal', '--no-window-ride']
        for executable, arguments in ((binary, ['start', *options]), (launcher, options)):
            process = None
            try:
                started = time.monotonic()
                process = subprocess.Popen([str(executable), *arguments], stdin=subprocess.DEVNULL,
                                           stdout=subprocess.PIPE, stderr=subprocess.PIPE,
                                           text=True, creationflags=hidden)
                stdout, stderr = process.communicate(timeout=15)
                elapsed = time.monotonic() - started
                assert process.returncode == 0, f'start failed: {stdout} {stderr}'
                assert 'honk300: running' in control('status'), 'EOF must arrive before runtime exit'
                results.append({'entrypoint': executable.name, 'seconds_to_exit_and_eof': elapsed,
                                'stdout': stdout, 'stderr': stderr, 'runtime_alive_after_eof': True})
                control('stop')
                deadline = time.monotonic() + 45
                while time.monotonic() < deadline:
                    if 'honk300: not running' in control('status'):
                        break
                    time.sleep(0.1)
                else:
                    raise AssertionError('graceful stop did not finish')
            finally:
                # The initial singleton check passed and this fixture is the only
                # process owner on its disposable host (or explicitly isolated run).
                if process is not None:
                    subprocess.run([str(binary), 'stop', '--force'], capture_output=True,
                                   timeout=20, creationflags=hidden, check=False)
                    if process.poll() is None:
                        process.kill()
                    process.communicate(timeout=10)
        (evidence / 'result.json').write_text(json.dumps({'ok': True, 'checks': results}, indent=2) + '\n',
                                            encoding='utf-8')
        print(f'Captured start EOF passed for CLI and app launcher: {evidence}')


if __name__ == '__main__':
    main()
