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
    observer = None

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
                                       stdout=log, stderr=log,
                                       env=dict(os.environ, HONK300_TRACE_COLLECTION='1'))

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
        major = int((evidence / 'version.txt').read_text().split()[-1].split('.')[0])
        assert active['capabilities']['dnd'] == 'unsupported', active
        assert active['capabilities']['pointer_control'] == ('denied' if major >= 6 else 'unsupported'), active
        if major < 6:
            unavailable = subprocess.run([str(binary), 'integrations', 'pointer', 'request'],
                capture_output=True, text=True, timeout=10)
            assert unavailable.returncode != 0, 'KDE 5 advertised unqualified portal input'
            assert supported(), 'Unavailable pointer permission damaged window support'
        wait(lambda: status()['capabilities']['prop_positioning'] == 'supported', 'native prop placement readiness')
        active = status()
        service = subprocess.run([str(binary), '__settings-service', '--config', str(config)],
            input=json.dumps({'protocol': 1, 'request_id': 91, 'command': {'op': 'status'}}),
            capture_output=True, text=True, timeout=10, check=True)
        native_status = json.loads(service.stdout)['data']['integrations']
        assert native_status['capabilities'] == active['capabilities'], native_status
        (directory / 'settings-status.json').write_text(json.dumps(native_status, indent=2) + '\n')
        from smoke_kwin_observer import NativeObserver
        observer = NativeObserver(directory, call, GLib)
        wait(lambda: observer.frames, 'native runtime observer')
        wait(lambda: observer.count >= 220, 'continuous native observation beyond script collection', timeout=15)
        assert supported(), 'Production companion lost its long-lived observation timer'
        control('do', 'note')
        def note():
            return next((item for item in observer.latest if item['title'] == 'Honk300 note'), None)
        initial = wait(note, 'actual runtime-owned note', timeout=20)
        parent = next(line.split()[1] for line in Path(f"/proc/{initial['pid']}/status").read_text().splitlines()
                      if line.startswith('PPid:'))
        assert int(parent) == process.pid, initial
        assert initial['app'] == 'honk300.prop.1', initial
        assert initial['geometry'][2] <= 1280 * 0.48 and initial['geometry'][3] <= 900 * 0.48
        # The compositor centers new windows exactly where the engine releases
        # deliveries. Depending on approach direction, that valid trip has no
        # dragging distance. Establish an off-center fixture placement first;
        # only subsequent engine-driven movement counts toward this assertion.
        placement = directory / 'note-initial-placement.js'
        placement.write_text('''(function () {
            var windows = workspace.stackingOrder !== undefined ? workspace.stackingOrder : workspace.clientList();
            for (var i = 0; i < windows.length; i++) {
                var window = windows[i];
                if (String(window.internalId) === ''' + json.dumps(initial['id']) + ''' &&
                    Number(window.pid) === ''' + str(initial['pid']) + ''' &&
                    String(window.resourceClass) === 'honk300.prop.1') {
                    var geometry = window.frameGeometry;
                    geometry.x = 120; geometry.y = 160;
                    window.frameGeometry = geometry;
                }
            }
        }());''')
        placement_name = 'honk300-fixture-note-placement'
        identifier = call('org.kde.KWin', '/Scripting', 'org.kde.kwin.Scripting', 'loadScript',
            GLib.Variant('(ss)', (str(placement), placement_name))).unpack()[0]
        assert identifier >= 0
        major = int((evidence / 'version.txt').read_text().split()[-1].split('.')[0])
        script_path = f'/Scripting/Script{identifier}' if major >= 6 else f'/{identifier}'
        try:
            call('org.kde.KWin', script_path, 'org.kde.kwin.Script', 'run')
            initial = wait(lambda: (item if (item := note()) and
                abs(item['geometry'][0] - 120) < 0.1 and abs(item['geometry'][1] - 160) < 0.1 else None),
                'off-center initial fixture placement')
        finally:
            call('org.kde.KWin', '/Scripting', 'org.kde.kwin.Scripting', 'unloadScript',
                 GLib.Variant('(s)', (placement_name,)))
        moved = wait(lambda: (item if (item := note()) and
            sum((item['geometry'][i] - initial['geometry'][i]) ** 2 for i in (0, 1)) > 16 else None),
            'engine-driven native KDE note movement', timeout=30)
        control('integrations', 'kde', 'remove')
        wait(unsupported, 'live explicit revocation')
        assert not loaded(name) and not record.exists()
        count = observer.count
        wait(lambda: observer.count >= count + 2, 'revocation settles in compositor')
        stopped = note()
        assert stopped is not None, 'Revocation destroyed the retained note'
        count = observer.count
        wait(lambda: observer.count >= count + 10, 'movement remains stopped after revocation')
        assert note()['geometry'] == stopped['geometry'], (stopped, note())
        (directory / 'owned-movement.json').write_text(json.dumps(dict(initial=initial,
            moved=moved, stopped=stopped, after=note()), indent=2) + '\n')
        observer.close()
        observer = None
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
            graceful_cleanup=True, stopped_removal=True, unrelated_state_preserved=True,
            actual_owned_prop_movement=True, revocation_preserves_note_and_stops_motion=True), indent=2) + '\n')
    finally:
        if process is not None and process.poll() is None:
            (directory / 'final-status.json').write_text(json.dumps(status(), indent=2) + '\n')
        if observer:
            observer.close()
        if process is not None and process.poll() is None:
            process.kill()
            process.wait(timeout=5)
        if record.exists():
            control('integrations', 'kde', 'remove')
