#!/usr/bin/env python3
"""Prove actual denied presence from a private LaunchServices identity in CI."""
import argparse
import json
import os
from pathlib import Path
import plistlib
import shutil
import subprocess
import time


def main():
    assert os.environ.get('GITHUB_ACTIONS') == 'true'
    assert os.uname().sysname == 'Darwin'
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--binary', type=Path, required=True)
    parser.add_argument('--evidence', type=Path, required=True)
    args = parser.parse_args()
    binary, root = args.binary.resolve(), args.evidence.resolve()
    root.mkdir(parents=True, exist_ok=True)
    app = root / 'HonkPresenceDenied.app'
    executable = app / 'Contents/MacOS/honk300'
    executable.parent.mkdir(parents=True)
    shutil.copy2(binary, executable)
    identifier = 'dev.emmetts.honk300.presence-denial-probe'
    with (app / 'Contents/Info.plist').open('wb') as output:
        plistlib.dump(dict(CFBundleIdentifier=identifier,
            CFBundleName='Honk300 private permission check', CFBundleExecutable='honk300',
            CFBundlePackageType='APPL', CFBundleVersion='1', LSUIElement=True), output)
    # This ad-hoc signature belongs only to an uninstalled private test bundle.
    # Stable publication continues to require Developer ID and notarization.
    subprocess.run(['codesign', '--force', '--sign', '-', str(app)], check=True, timeout=10)
    config = root / 'config.toml'
    config.write_text('goose_config_version = 2\n[behavior]\nfirst_wander_time_seconds = 600.0\n'
        '[audio]\nenabled = false\n[safety]\nno_mouse_steal = true\nno_window_ride = true\n'
        'pause_on_fullscreen = true\n[schedule]\nquiet_hours_enabled = false\n'
        'autumn = false\nseasonal = false\ndnd_respect = true\n')
    log = root / 'runtime.log'
    def cli(*words):
        return subprocess.run([str(binary), *words], capture_output=True, text=True, timeout=12)
    assert 'honk300: not running' in cli('status').stdout, 'Another runtime owns IPC'
    launcher = None
    try:
        launcher = subprocess.Popen(['/usr/bin/open', '-n', '-W',
            '--stdout', str(log), '--stderr', str(log),
            '--env', 'GITHUB_ACTIONS=true', '--env', 'HONK300_TRACE_PRESENCE=1',
            str(app), '--args', 'start', '--config', str(config), '--no-sound', '--no-window-ride'])
        deadline = time.monotonic() + 15
        value = ''
        while time.monotonic() < deadline:
            assert launcher.poll() is None, 'Private LaunchServices runtime exited before readiness'
            value = cli('status').stdout
            if 'honk300: running' in value:
                break
            time.sleep(.1)
        assert 'honk300: running' in value, value
        assert 'accessibility: denied' in value, value
        assert 'fullscreen observation: denied' in value, value
        assert 'do not disturb observation: unsupported' in value, value
        (root / 'status.txt').write_text(value)
        entries = [json.loads(line.split('honk300 macos presence trace: ', 1)[1])
                   for line in log.read_text().splitlines()
                   if 'honk300 macos presence trace: ' in line]
        assert entries and entries[-1] == dict(fullscreen_status='denied', fullscreen=False, manners=False), entries
        assert cli('reload').returncode == 0
        assert 'fullscreen observation: denied' in cli('status').stdout
        started = time.monotonic()
        assert cli('stop').returncode == 0
        launcher.wait(timeout=12)
        shutdown = time.monotonic() - started
        assert launcher.returncode == 0 and 'honk300: not running' in cli('status').stdout
        (root / 'result.json').write_text(json.dumps(dict(ok=True, bundle_identifier=identifier,
            fullscreen='denied', dnd='unsupported', reload=True, graceful_stop_seconds=shutdown,
            permission_granted=False, installed=False), indent=2))
    finally:
        if launcher is not None and launcher.poll() is None:
            cli('stop')
            try:
                launcher.wait(timeout=12)
            except subprocess.TimeoutExpired:
                # This disposable job contains only the runtime launched above.
                cli('stop', '--force')
                launcher.wait(timeout=5)


if __name__ == '__main__':
    main()
