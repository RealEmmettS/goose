"""Actual Rust owner, native settings and live Hyprland observation lifecycle."""
import json
import os
from pathlib import Path
import socket
import subprocess
import time


def qualify(binary, settings, evidence, wait, window, find_window, ipc_path, GLib):
    assert os.environ.get('GITHUB_ACTIONS') == 'true'
    import gi
    gi.require_version('Atspi', '2.0')
    from gi.repository import Atspi, Gio
    bus = Gio.bus_get_sync(Gio.BusType.SESSION, None)
    bus.call_sync('org.a11y.Bus', '/org/a11y/bus', 'org.freedesktop.DBus.Properties', 'Set',
        GLib.Variant('(ssv)', ('org.a11y.Status', 'IsEnabled', GLib.Variant('b', True))),
        None, Gio.DBusCallFlags.NONE, 2000, None)
    Atspi.init()
    directory = evidence / 'goose'
    directory.mkdir()
    config = directory / 'config.toml'
    original = '''# Retain comments and unknown settings through observation setup.
goose_config_version = 2
[behavior]
first_wander_time_seconds = 600.0
[audio]
enabled = false
[safety]
no_mouse_steal = false
no_window_ride = false
pause_on_fullscreen = true
[schedule]
quiet_hours_enabled = false
dnd_respect = false
seasonal = false
autumn = false
[future]
preserve = "untouched"
'''
    config.write_text(original)
    record = Path(os.environ['XDG_DATA_HOME']) / 'honk300/wayland/hyprland.json'
    runtime = ui = None
    logs = []
    states = []
    thread_states = []

    def launch(executable, label, *arguments):
        path = directory / f'{label}-{len(logs)}.log'
        logs.append(path)
        with path.open('w') as log:
            process = subprocess.Popen([str(executable), *arguments], stdout=log, stderr=log,
                                       env=dict(os.environ, HONK300_TRACE_PRESENCE='1', HONK300_TRACE_OBSERVER='1'))
        return process, path

    def control(*arguments, success=True):
        result = subprocess.run([str(binary), *arguments], capture_output=True, text=True, timeout=10)
        assert (result.returncode == 0) == success, (arguments, result.stdout, result.stderr)
        return result.stdout

    def status():
        if runtime is not None:
            assert runtime.poll() is None, 'Runtime exited: ' + '\n'.join(p.read_text()[-2000:] for p in logs)
        return json.loads(control('integrations', 'hyprland', 'status'))

    def expect(state, label):
        result = wait(lambda: (value if ((value := status()).get('capabilities') or {}).get('windows') == state
                              else None), label)
        caps = result['capabilities']
        assert caps['fullscreen'] == state, caps
        for key in ('movement', 'pointer_observation', 'pointer_control', 'dnd', 'prop_positioning'):
            assert caps[key] == 'unsupported', (key, caps)
        states.append(dict(label=label, state=state, capabilities=caps))
        return result

    def workers():
        threads = []
        for path in Path(f'/proc/{runtime.pid}/task').glob('*/comm'):
            try:
                name = path.read_text().strip()
                if name == 'hyprland-observer'[:15]:
                    threads.append(dict(id=path.parent.name, name=name,
                        stat=(path.parent / 'stat').read_text()))
            except FileNotFoundError:
                continue
        thread_states.append(dict(at=time.monotonic(), threads=threads))
        return sorted(thread['id'] for thread in threads)

    def joined_count():
        return runtime_log.read_text().count('honk300 observer trace: name=hyprland joined=true')

    def removed(previous_joins):
        # Require the production JoinHandle to have completed before its
        # unsupported reply. Separately observe the kernel's task retirement;
        # a directory enumeration is not the Rust join completion boundary.
        assert joined_count() == previous_joins + 1, runtime_log.read_text()[-2000:]
        def retired():
            assert status()['capabilities']['windows'] == 'unsupported'
            return not workers()
        wait(retired, 'joined native worker retired from proc', timeout=1)

    def nodes():
        assert ui.poll() is None, f'Settings exited with {ui.returncode}'
        desktop = Atspi.get_desktop(0)
        desktop.clear_cache()
        stack = [desktop.get_child_at_index(i) for i in range(desktop.get_child_count())]
        stack = [node for node in stack if node and node.get_process_id() == ui.pid]
        result = []
        while stack and len(result) < 700:
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

    def trace(fullscreen, manners, observed=True):
        wanted = f'observed={str(observed).lower()} fullscreen={str(fullscreen).lower()} manners={str(manners).lower()}'
        def current():
            entries = [line for line in runtime_log.read_text().splitlines() if 'honk300 presence trace:' in line]
            return entries and entries[-1].endswith(wanted)
        wait(current, 'actual engine presence ' + wanted)

    def close(process):
        if process and process.poll() is None:
            process.terminate()
            try:
                process.wait(timeout=5)
            except subprocess.TimeoutExpired:
                process.kill()
                process.wait(timeout=5)

    try:
        assert not record.exists() and not status()['installed']
        runtime, runtime_log = launch(binary, 'runtime', 'start', '--config', str(config), '--wayland')
        expect('unsupported', 'default-off observations')
        assert not workers(), thread_states[-1]
        ui, _ = launch(settings, 'settings', '--config', str(config))
        wait(lambda: find('First wander (seconds)'), 'actual loaded settings')
        invoke('Appearance')
        toggle = wait(lambda: find('Reduced motion', Atspi.Role.TOGGLE_BUTTON), 'appearance draft')
        assert toggle.get_action_iface().do_action(0)
        wait(lambda: find('Unsaved changes'), 'unsaved appearance draft')
        invoke('Platform & status')
        invoke('Set up Hyprland')
        wait(lambda: find('Enable Hyprland observations', Atspi.Role.DIALOG), 'native Hyprland consent dialog')
        invoke('Cancel')
        assert not record.exists(), 'Cancelled setup wrote consent'
        invoke('Set up Hyprland')
        invoke('Enable Hyprland observations')
        expect('supported', 'native settings explicitly enables the Rust observer')
        assert len(workers()) == 1
        foreign = record.parent / 'foreign.txt'
        foreign.write_text('Keep unrelated integration data')
        trace(False, False)
        invoke('Refresh status')
        wait(lambda: any('Window observation: supported | Fullscreen: supported' in node.get_name()
                         for node in nodes()), 'native Hyprland status readback')
        subprocess.run(['grim', str(directory / 'native-settings-goose.png')], check=True, timeout=8)
        control('do', 'nab', success=False)
        window.fullscreen()
        wait(lambda: find_window('Honk300 ordinary Hyprland probe')['fullscreen'] == 2, 'fixture fullscreen')
        trace(True, True)
        original_workers = workers()
        original_consent = record.read_bytes()
        control('integrations', 'hyprland', 'setup')
        expect('supported', 'repeated CLI setup retains fullscreen observations')
        assert workers() == original_workers and record.read_bytes() == original_consent
        invoke('Set up Hyprland')
        invoke('Enable Hyprland observations')
        expect('supported', 'repeated native setup retains fullscreen observations')
        assert workers() == original_workers and record.read_bytes() == original_consent
        # The actual engine honors the user's switch while the native fullscreen
        # observation stays true. No test-only schedule substitute is involved.
        config.write_text(original.replace('pause_on_fullscreen = true', 'pause_on_fullscreen = false'))
        control('reload')
        trace(True, False)
        config.write_text(original)
        control('reload')
        trace(True, True)
        window.unfullscreen()
        wait(lambda: find_window('Honk300 ordinary Hyprland probe')['fullscreen'] == 0, 'fixture leaves fullscreen')
        trace(False, False)
        invoke('Refresh status')
        invoke('Appearance')
        wait(lambda: (node := find('Reduced motion', Atspi.Role.TOGGLE_BUTTON)) and
             node.get_state_set().contains(Atspi.StateType.PRESSED), 'draft survives Hyprland setup and status')
        assert config.read_text() == original
        invoke('Platform & status')
        previous_joins = joined_count()
        invoke('Remove Hyprland observations')
        expect('unsupported', 'native settings revokes live observations')
        removed(previous_joins)
        assert not record.exists()
        control('integrations', 'hyprland', 'setup')
        expect('supported', 'explicit CLI setup')
        previous_joins = joined_count()
        record.unlink()
        expect('unsupported', 'external consent removal')
        removed(previous_joins)
        control('integrations', 'hyprland', 'setup')
        expect('supported', 'new consent after external removal')
        # Replace only this disposable compositor's IPC pathname. The overlay
        # keeps its independent Wayland connection; observations must fail closed.
        parked = ipc_path.with_name(ipc_path.name + '.parked')
        ipc_path.rename(parked)
        try:
            with socket.socket(socket.AF_UNIX, socket.SOCK_STREAM) as replacement:
                replacement.bind(str(ipc_path))
                replacement.listen(1)
                expect('failed', 'socket replacement withdraws live observations')
                trace(False, False, observed=False)
        finally:
            if ipc_path.exists():
                ipc_path.unlink()
            parked.rename(ipc_path)
        for _ in range(10):
            assert status()['capabilities']['windows'] == 'failed', 'Observer reconnected without explicit action'
            time.sleep(0.05)
        control('integrations', 'hyprland', 'setup')
        expect('supported', 'explicit reconnection to the verified compositor')
        control('stop')
        wait(lambda: runtime.poll() is not None, 'graceful runtime stop', timeout=30)
        assert runtime.returncode == 0
        runtime = None
        assert find('Unsaved changes'), 'Stopping the goose discarded the independent settings draft'
        runtime, runtime_log = launch(binary, 'runtime', 'start', '--config', str(config), '--wayland')
        expect('supported', 'saved observation consent after graceful restart')
        close(ui)
        ui = None
        expect('supported', 'closing settings preserves the runtime owner')
        for index in range(4):
            captured_after = time.monotonic() + 1
            wait(lambda: time.monotonic() >= captured_after, 'native entry animation')
            subprocess.run(['grim', str(directory / f'native-goose-{index}.png')], check=True, timeout=8)
        runtime.kill()
        runtime.wait(timeout=5)
        runtime = None
        runtime, runtime_log = launch(binary, 'runtime', 'start', '--config', str(config), '--wayland')
        expect('supported', 'observation worker after crash recovery')
        control('stop')
        wait(lambda: runtime.poll() is not None, 'second graceful stop', timeout=30)
        assert runtime.returncode == 0
        runtime = None
        control('integrations', 'hyprland', 'remove')
        assert not record.exists() and not status()['installed']
        assert foreign.read_text() == 'Keep unrelated integration data'
        assert config.read_text() == original
        (directory / 'result.json').write_text(json.dumps(dict(ok=True, default_off=True,
            native_settings_consent=True, engine_fullscreen_manners=True, live_config_toggle=True,
            idempotent_setup_preserves_worker=True,
            draft_preserved=True, unsupported_actions=True, exact_worker_removed=True,
            live_remove=True, external_revocation=True, socket_replacement=True,
            no_automatic_reconnect=True, graceful_restart=True, crash_restart=True,
            stopped_removal=True, unrelated_state_preserved=True), indent=2) + '\n')
    finally:
        (directory / 'worker-identities.json').write_text(json.dumps(thread_states, indent=2) + '\n')
        (directory / 'observed-states.json').write_text(json.dumps(states, indent=2) + '\n')
        if ui is not None and ui.poll() is None:
            (directory / 'native-settings-tree.json').write_text(json.dumps([
                dict(name=node.get_name(), role=node.get_role_name()) for node in nodes()
            ], indent=2) + '\n')
        close(ui)
        close(runtime)
        if record.exists():
            control('integrations', 'hyprland', 'remove')
