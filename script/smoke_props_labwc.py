#!/usr/bin/env python3
"""Native labwc qualification on a private software-rendered CI desktop."""
import argparse
import json
import os
from pathlib import Path
import subprocess
import time


def main():
    if os.environ.get('GITHUB_ACTIONS') != 'true':
        raise RuntimeError('Use only a disposable GitHub runner')
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--directory', type=Path, required=True)
    parser.add_argument('--evidence', type=Path, required=True)
    args = parser.parse_args()
    directory, evidence = args.directory.resolve(), args.evidence.resolve()
    evidence.mkdir(parents=True)
    runtime_dir, config_dir = evidence / 'runtime', evidence / 'labwc'
    runtime_dir.mkdir(mode=0o700)
    config_dir.mkdir()
    (config_dir / 'rc.xml').write_text('<labwc_config/>\n')
    (config_dir / 'autostart').write_text('#!/bin/sh\n')
    environment = dict(os.environ, XDG_RUNTIME_DIR=str(runtime_dir), XDG_SESSION_TYPE='wayland',
                       XDG_CURRENT_DESKTOP='labwc', WLR_BACKENDS='headless', WLR_HEADLESS_OUTPUTS='1',
                       WLR_RENDERER='pixman', GDK_BACKEND='wayland', LABWC_UPDATE_ACTIVATION_ENV='0')
    environment.pop('DISPLAY', None)
    environment.pop('WAYLAND_DISPLAY', None)
    with (evidence / 'compositor.log').open('w') as log:
        compositor = subprocess.Popen(['labwc', '--config-dir', str(config_dir), '--debug'],
                                      stdout=log, stderr=log, env=environment)
        try:
            deadline = time.monotonic() + 20
            while time.monotonic() < deadline:
                if compositor.poll() is not None:
                    raise RuntimeError('Private labwc compositor exited before readiness')
                sockets = [path for path in runtime_dir.glob('wayland-*') if path.is_socket()]
                if len(sockets) == 1:
                    environment['WAYLAND_DISPLAY'] = sockets[0].name
                    break
                time.sleep(0.1)
            else:
                raise RuntimeError('Private labwc did not create its socket')
            outputs = json.loads(subprocess.check_output(['wlr-randr', '--json'], env=environment))
            assert len(outputs) == 1, outputs
            output = outputs[0]['name']
            subprocess.run(['wlr-randr', '--output', output, '--custom-mode', '1280x900@60Hz'], env=environment, check=True)
            for scale in (1, 2):
                subprocess.run(['wlr-randr', '--output', output, '--scale', str(scale)], env=environment, check=True)
                subprocess.run(['python3', 'script/smoke_owned_props_linux.py', '--binary', str(directory / 'honk300-settings'),
                                '--evidence', str(evidence / f'native-scale-{scale}')], env=environment, check=True, timeout=240)
            subprocess.run(['wlr-randr', '--output', output, '--scale', '1'], env=environment, check=True)
            subprocess.run(['python3', 'script/smoke_runtime_props_linux.py', '--binary', str(directory / 'honk300'),
                            '--evidence', str(evidence / 'engine')], env=environment, check=True, timeout=300)
            (evidence / 'result.json').write_text(json.dumps({'ok': True, 'scales': [1, 2],
                'labwc': subprocess.check_output(['labwc', '--version'], text=True).strip(),
                'architecture': os.uname().machine, 'rendering': 'headless pixman',
                'physical_pi_acceptance': False}, indent=2) + '\n')
        finally:
            compositor.terminate()
            try:
                compositor.wait(timeout=10)
            except subprocess.TimeoutExpired:
                compositor.kill()
                compositor.wait()


if __name__ == '__main__':
    main()
