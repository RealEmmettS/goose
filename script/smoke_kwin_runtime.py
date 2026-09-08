"""Production runtime/setup qualification inside the private KWin fixture."""
import json
import os
from pathlib import Path
import subprocess


def qualify(binary, evidence, wait, call, GLib):
    assert os.environ.get('GITHUB_ACTIONS') == 'true'
    directory = evidence / 'goose'
    directory.mkdir()
    config = directory / 'config.toml'
    original = '''# Preserve this user's existing settings.
goose_config_version = 2
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
[future]
preserve = "untouched"
'''
    config.write_text(original)
    record = Path(os.environ['XDG_DATA_HOME']) / 'honk300/wayland/kwin.json'
    process = None
    logs = []

    def control(*args):
        result = subprocess.run([str(binary), *args], capture_output=True, text=True, timeout=10)
        assert result.returncode == 0, (args, result.stdout, result.stderr)
        return result.stdout

    def setup():
        return json.loads(control('integrations', 'kde', 'setup'))

    def status():
        if process is not None and process.poll() is not None:
            raise AssertionError('Goose exited: ' + '\n'.join(path.read_text()[-5000:] for path in logs))
        return json.loads(control('integrations', 'kde', 'status'))

    def supported():
        details = status()
        return details if (details.get('capabilities') or {}).get('windows') == 'supported' else None

    def unsupported():
        return (status().get('capabilities') or {}).get('windows') == 'unsupported'

    def loaded(name):
        return call('org.kde.KWin', '/Scripting', 'org.kde.kwin.Scripting', 'isScriptLoaded',
                    GLib.Variant('(s)', (name,))).unpack()[0]

    def start():
        nonlocal process
        path = directory / f'runtime-{len(logs)}.log'
        logs.append(path)
        with path.open('w') as log:
            process = subprocess.Popen([str(binary), 'start', '--config', str(config), '--wayland'],
                                       stdout=log, stderr=log)

    def kill():
        nonlocal process
        process.kill()
        process.wait(timeout=5)
        process = None

    try:
        assert not status()['installed']
        assert not record.exists()
        start()
        wait(unsupported, 'default-off real runtime capabilities')
        setup()
        name = 'honk300-' + json.loads(record.read_text())['nonce']
        foreign = record.parent / 'foreign.txt'
        foreign.write_text('Keep unrelated user data')
        active = wait(supported, 'explicitly enabled runtime KWin capabilities')
        for capability in ('windows', 'movement', 'pointer_observation', 'fullscreen'):
            assert active['capabilities'][capability] == 'supported', active
        for capability in ('pointer_control', 'dnd', 'prop_positioning'):
            assert active['capabilities'][capability] == 'unsupported', active
        service = subprocess.run([str(binary), '__settings-service', '--config', str(config)],
            input=json.dumps({'protocol': 1, 'request_id': 91, 'command': {'op': 'status'}}),
            capture_output=True, text=True, timeout=10, check=True)
        native_status = json.loads(service.stdout)['data']['integrations']
        assert native_status['capabilities'] == active['capabilities'], native_status
        (directory / 'settings-status.json').write_text(json.dumps(native_status, indent=2) + '\n')
        control('integrations', 'kde', 'remove')
        wait(unsupported, 'live explicit revocation')
        assert not loaded(name) and not record.exists()
        setup()
        wait(supported, 'live explicit setup after removal')
        name = 'honk300-' + json.loads(record.read_text())['nonce']
        record.unlink()
        wait(unsupported, 'external removal of the saved grant')
        assert not loaded(name)
        setup()
        wait(supported, 'new explicit setup before crash')
        name = 'honk300-' + json.loads(record.read_text())['nonce']
        kill()
        assert loaded(name), 'Crash fixture did not retain the stopped KWin registration'
        start()
        wait(supported, 'real runtime restart reclaims its exact stopped registration')
        assert loaded(name)
        control('stop')
        wait(lambda: process.poll() is not None, 'graceful runtime shutdown', timeout=30)
        assert process.returncode == 0
        process = None
        assert not loaded(name), 'Graceful shutdown leaked its KWin script'
        start()
        wait(supported, 'saved consent remains usable after graceful restart')
        kill()
        control('integrations', 'kde', 'remove')
        assert not loaded(name) and not record.exists(), 'Stopped runtime removal leaked a registration'
        assert foreign.read_text() == 'Keep unrelated user data'
        assert config.read_text() == original
        (directory / 'result.json').write_text(json.dumps(dict(ok=True, goose_runtime_connected=True,
            explicit_setup=True, distinct_capabilities=True, settings_service=True, default_off=True,
            live_remove=True, external_revocation=True, crash_recovery=True,
            graceful_cleanup=True, stopped_removal=True, unrelated_state_preserved=True), indent=2) + '\n')
    finally:
        if process is not None and process.poll() is None:
            process.kill()
            process.wait(timeout=5)
