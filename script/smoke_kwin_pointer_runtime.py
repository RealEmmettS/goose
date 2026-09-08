"""Real settings consent, engine cursor motion, and revocation in private KDE 6."""
import json
import os
from pathlib import Path
import subprocess


def qualify(binary, settings, evidence, wait, call, GLib, Atspi, portal_nodes, backend):
    assert os.environ.get('GITHUB_ACTIONS') == 'true'
    from smoke_kwin_observer import NativeObserver
    directory = evidence / 'pointer-runtime'
    directory.mkdir()
    config = directory / 'config.toml'
    original = '''# Keep this pointer permission fixture unchanged.
goose_config_version = 2
[behavior]
first_wander_time_seconds = 600.0
[audio]
enabled = false
[safety]
no_mouse_steal = false
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
    runtime = ui = observer = None
    logs = []
    states = []

    def launch(executable, name, *arguments):
        log = (directory / f'{name}-{len(logs)}.log').open('w')
        logs.append(log)
        return subprocess.Popen([str(executable), *arguments], stdout=log, stderr=log)

    def control(*arguments, success=True):
        result = subprocess.run([str(binary), *arguments], capture_output=True, text=True, timeout=10)
        assert (result.returncode == 0) == success, (arguments, result.stdout, result.stderr)
        return result.stdout

    def status():
        assert runtime.poll() is None, f'Goose exited with {runtime.returncode}'
        return json.loads(control('integrations', 'pointer', 'status'))

    def expect(state, label):
        result = wait(lambda: (value if ((value := status()).get('capabilities') or {}).get('pointer_control') == state
                              else None), label, timeout=30)
        states.append(dict(label=label, state=state, capabilities=result['capabilities']))
        return result

    def nodes():
        assert ui.poll() is None, f'Settings exited with {ui.returncode}'
        desktop = Atspi.get_desktop(0)
        desktop.clear_cache()
        stack = [desktop.get_child_at_index(i) for i in range(desktop.get_child_count())]
        stack = [node for node in stack if node and node.get_process_id() == ui.pid]
        result = []
        while stack and len(result) < 640:
            node = stack.pop()
            try:
                node.clear_cache()
                result.append(node)
                stack.extend(child for i in range(node.get_child_count()) if (child := node.get_child_at_index(i)))
            except GLib.Error:
                continue
        return result

    def find(name, role=None):
        return next((node for node in nodes() if node.get_name() == name
                     and (role is None or node.get_role() == role)), None)

    def invoke(name):
        node = wait(lambda: (value if (value := find(name, Atspi.Role.PUSH_BUTTON)) and
                            value.get_state_set().contains(Atspi.StateType.ENABLED) else None), name)
        assert node.get_action_iface().do_action(0), name

    def desktop_action(names, label):
        def button():
            current = portal_nodes()
            (directory / f'{label}-portal-tree.json').write_text(json.dumps([
                dict(name=node.get_name(), role=node.get_role_name()) for node in current], indent=2) + '\n')
            candidates = [node for node in current if node.get_role() == Atspi.Role.PUSH_BUTTON
                          and node.get_name() in names and node.get_state_set().contains(Atspi.StateType.ENABLED)]
            return candidates[0] if len(candidates) == 1 else None
        assert wait(button, label, timeout=30).get_action_iface().do_action(0)

    def request_and_grant(label):
        control('integrations', 'pointer', 'request')
        expect('unprobed', label + ' pending')
        desktop_action(('Share', 'Allow'), label)
        expect('supported', label + ' granted')

    def close(process):
        if process and process.poll() is None:
            process.terminate()
            try:
                process.wait(timeout=5)
            except subprocess.TimeoutExpired:
                process.kill()
                process.wait()

    try:
        assert not record.exists()
        runtime = launch(binary, 'runtime', 'start', '--config', str(config), '--wayland')
        expect('unsupported', 'default-off pointer support')
        control('integrations', 'kde', 'setup')
        expect('denied', 'window setup does not grant pointer permission')
        ui = launch(settings, 'settings', '--config', str(config))
        wait(lambda: find('First wander (seconds)'), 'loaded actual settings')
        invoke('Appearance')
        toggle = wait(lambda: find('Reduced motion', Atspi.Role.TOGGLE_BUTTON), 'appearance draft')
        assert not toggle.get_state_set().contains(Atspi.StateType.PRESSED)
        assert toggle.get_action_iface().do_action(0)
        wait(lambda: find('Unsaved changes'), 'unsaved draft before pointer request')
        invoke('Platform & status')
        invoke('Request pointer access')
        expect('unprobed', 'native settings requested desktop consent')
        desktop_action(('Cancel',), 'deny-first-request')
        wait(lambda: status()['capabilities']['pointer_control'] in ('failed', 'denied'), 'desktop denial')
        assert status()['capabilities']['windows'] == 'supported'
        invoke('Refresh status')
        invoke('Request pointer access')
        expect('unprobed', 'retry after denied consent')
        invoke('Cancel pointer access')
        expect('denied', 'settings cancels pending portal request')
        invoke('Request pointer access')
        expect('unprobed', 'settings requests fresh consent')
        desktop_action(('Share', 'Allow'), 'settings-grant')
        expect('supported', 'settings granted real pointer device')
        original_consent = record.read_bytes()
        control('integrations', 'kde', 'setup')
        expect('supported', 'repeated CLI setup preserves the active pointer grant')
        invoke('Set up KDE')
        invoke('Enable KDE integration')
        expect('supported', 'repeated native settings setup preserves the active pointer grant')
        assert record.read_bytes() == original_consent, 'Idempotent setup replaced the consent identity'
        invoke('Refresh status')
        wait(lambda: find('Cancel pointer access').get_state_set().contains(Atspi.StateType.ENABLED),
             'native cancellation is available')
        invoke('Appearance')
        wait(lambda: (node := find('Reduced motion', Atspi.Role.TOGGLE_BUTTON)) and
             node.get_state_set().contains(Atspi.StateType.PRESSED), 'draft survives pointer lifecycle')
        assert config.read_text() == original
        close(ui)
        ui = None
        expect('supported', 'closing settings preserves the running permission owner')
        observer = NativeObserver(directory, call, GLib)
        wait(lambda: observer.pointer, 'independent native pointer observation')
        wait(lambda: observer.count >= 220, 'continuous pointer observations beyond script collection', timeout=15)
        expect('supported', 'pointer grant remains live during the observation soak')
        initial = observer.pointer[:]
        control('do', 'nab')
        moved = wait(lambda: observer.pointer[:] if sum((observer.pointer[i] - initial[i]) ** 2
                     for i in (0, 1)) > 4 else None, 'actual engine-driven portal cursor movement', timeout=35)
        control('integrations', 'pointer', 'cancel')
        expect('denied', 'explicit CLI cancellation')
        count = observer.count
        wait(lambda: observer.count >= count + 2, 'pending cursor delivery settles')
        stopped = observer.pointer[:]
        control('do', 'nab', success=False)
        count = observer.count
        wait(lambda: observer.count >= count + 10, 'revoked pointer remains stationary')
        assert observer.pointer == stopped, (stopped, observer.pointer)
        (directory / 'actual-pointer-movement.json').write_text(json.dumps(
            dict(initial=initial, moved=moved, cancelled=stopped, after=observer.pointer), indent=2) + '\n')
        observer.close()
        observer = None
        request_and_grant('external-revocation')
        record.unlink()
        expect('unsupported', 'external companion removal revokes pointer session')
        control('integrations', 'kde', 'setup')
        expect('denied', 'new companion setup does not restore pointer grant')
        request_and_grant('graceful-stop')
        control('stop')
        wait(lambda: runtime.poll() is not None, 'graceful stop with active pointer permission', timeout=30)
        assert runtime.returncode == 0
        runtime = launch(binary, 'runtime', 'start', '--config', str(config), '--wayland')
        expect('denied', 'graceful restart requires fresh pointer consent')
        request_and_grant('crash-recovery')
        runtime.kill()
        runtime.wait(timeout=5)
        runtime = launch(binary, 'runtime', 'start', '--config', str(config), '--wayland')
        expect('denied', 'crash recovery does not restore pointer grant')
        request_and_grant('service-loss')
        (directory / 'native-observer-frames.json').rename(directory / 'initial-pointer-frames.json')
        observer = NativeObserver(directory, call, GLib)
        wait(lambda: observer.pointer, 'native pointer before active backend loss')
        initial = observer.pointer[:]
        control('do', 'nab')
        wait(lambda: sum((observer.pointer[i] - initial[i]) ** 2 for i in (0, 1)) > 4,
             'active pointer motion before backend loss', timeout=35)
        close(backend)
        # KDE sends DEVICE_REMOVED before the EIS disconnect. Production treats
        # removal as permission ending, even when backend shutdown caused it.
        expect('denied', 'backend loss removes the native granted pointer device')
        assert status()['capabilities']['windows'] == 'supported'
        count = observer.count
        wait(lambda: observer.count >= count + 2, 'removed-device delivery settles')
        stopped = observer.pointer[:]
        control('do', 'nab', success=False)
        count = observer.count
        wait(lambda: observer.count >= count + 10, 'removed pointer stays stationary')
        assert observer.pointer == stopped, (stopped, observer.pointer)
        observer.close()
        observer = None
        control('integrations', 'kde', 'remove')
        expect('unsupported', 'explicit removal after backend failure')
        assert config.read_text() == original and not record.exists()
        (directory / 'result.json').write_text(json.dumps(dict(ok=True, native_settings=True,
            actual_engine_pointer_motion=True, actual_desktop_denial=True, pending_cancel=True,
            idempotent_setup_preserves_pointer_grant=True,
            draft_preserved=True, closing_settings_preserves_runtime=True, explicit_cancel=True,
            external_revocation=True, graceful_stop=True, crash_recovery=True,
            no_restored_pointer_grant=True, backend_loss=True, window_support_preserved=True,
            states=states), indent=2) + '\n')
    except Exception:
        if ui and ui.poll() is None:
            (directory / 'failed-settings-tree.json').write_text(json.dumps([
                dict(name=node.get_name(), role=node.get_role_name()) for node in nodes()], indent=2) + '\n')
        raise
    finally:
        (directory / 'observed-states.json').write_text(json.dumps(states, indent=2) + '\n')
        if observer:
            observer.close()
        close(ui)
        close(runtime)
        if record.exists():
            control('integrations', 'kde', 'remove')
        for log in logs:
            log.close()
