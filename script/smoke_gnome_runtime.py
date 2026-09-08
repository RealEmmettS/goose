"""Actual Rust owner, native settings and live GNOME observation lifecycle."""
import json
import os
from pathlib import Path
import socket
import subprocess
import time


def qualify(binary, settings, evidence, wait, window, protected_window, find_window,
            snapshot, shell_owner, environment, capture_environment, GLib):
    assert os.environ.get('GITHUB_ACTIONS') == 'true'
    os.environ.update(environment)
    import gi
    gi.require_version('Atspi', '2.0')
    from gi.repository import Atspi, Gio, Gtk
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
    record = Path(os.environ['XDG_DATA_HOME']) / 'honk300/wayland/gnome.json'
    runtime = ui = None
    logs = []
    states = []
    thread_states = []

    def launch(executable, label, *arguments):
        path = directory / f'{label}-{len(logs)}.log'
        logs.append(path)
        with path.open('w') as log:
            process = subprocess.Popen([str(executable), *arguments], stdout=log, stderr=log,
                                       env=dict(os.environ, HONK300_TRACE_PRESENCE='1', HONK300_TRACE_GNOME='1', HONK300_TRACE_OBSERVER='1'))
        return process, path

    def control(*arguments, success=True):
        result = subprocess.run([str(binary), *arguments], capture_output=True, text=True, timeout=10)
        assert (result.returncode == 0) == success, (arguments, result.stdout, result.stderr)
        return result.stdout

    def status():
        if runtime is not None:
            assert runtime.poll() is None, 'Runtime exited: ' + '\n'.join(p.read_text()[-2000:] for p in logs)
        return json.loads(control('integrations', 'gnome', 'status'))

    def expect(state, label):
        result = wait(lambda: (value if ((value := status()).get('capabilities') or {}).get('windows') == state
                              and value['capabilities'].get('prop_positioning') == 'supported' else None), label)
        caps = result['capabilities']
        assert caps['fullscreen'] == state, caps
        assert caps['prop_positioning'] == 'supported', caps
        for key in ('movement', 'pointer_observation', 'pointer_control', 'dnd'):
            assert caps[key] == 'unsupported', (key, caps)
        states.append(dict(label=label, state=state, capabilities=caps))
        return result

    def workers():
        threads = []
        for p in Path(f'/proc/{runtime.pid}/task').glob('*/comm'):
            try:
                if p.read_text().strip() == 'gnome-observer'[:15]:
                    threads.append(dict(id=p.parent.name, stat=(p.parent / 'stat').read_text()))
            except FileNotFoundError:
                continue
        thread_states.append(dict(at=time.monotonic(), threads=threads))
        return sorted(thread['id'] for thread in threads)

    def joined_count():
        return runtime_log.read_text().count('honk300 observer trace: name=gnome joined=true')

    def removed(previous_joins):
        assert joined_count() == previous_joins + 1, runtime_log.read_text()[-2000:]
        def retired():
            assert status()['capabilities']['windows'] == 'unsupported'
            return not workers()
        wait(retired, 'joined GNOME worker retired from proc', timeout=1)

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

    def observed(check, label):
        def current():
            entries = [json.loads(line.split('honk300 gnome trace: ', 1)[1])
                       for line in runtime_log.read_text().splitlines(keepends=True)
                       if line.endswith('\n') and 'honk300 gnome trace: ' in line]
            return entries and entries[-1] if entries and check(entries[-1]) else None
        return wait(current, label)

    def drag(target, protected=False, interrupt=None, label='ordinary'):
        target.present()
        title = target.get_title()
        previous = None
        stable_since = time.monotonic()
        def settled(focused=False):
            nonlocal previous, stable_since
            node = find_window(title)
            state = (node, target.get_width(), target.get_height())
            if state != previous:
                previous, stable_since = state, time.monotonic()
            return node if (node and node['pid'] == os.getpid() and not node['fullscreen']
                and node['rect'][2] >= 300 and node['rect'][3] >= 200
                and target.get_width() > 0 and target.get_height() > 0
                and (not focused or node['focused']) and not snapshot()['overview']
                and time.monotonic() - stable_since >= 0.25) else None
        arranged = wait(settled, 'settled native drag fixture: ' + label)
        bus.call_sync(shell_owner, '/dev/emmetts/Honk300/GnomeProbe1',
            'dev.emmetts.Honk300.GnomeProbe1', 'FocusFixture',
            GLib.Variant('(tus)', (arranged['id'], os.getpid(),
                json.dumps(arranged['rect'], separators=(',', ':')))),
            GLib.VariantType.new('(s)'), Gio.DBusCallFlags.NONE, 1000, None)
        native = wait(lambda: settled(focused=True), 'settled native drag focus: ' + label)
        (directory / f'arranged-{label}.json').write_text(json.dumps(native, indent=2))
        x, y, width, height = native['rect']
        # A previous ride can leave the naturally interactive goose above the
        # middle of the title bar. Use its clear left portion; GTK must still
        # prove receipt before any held gesture can qualify.
        pointer_x, pointer_y = x + 36, y + 18
        # A Shell coordinate is not proof that the native client has received
        # pointer entry. Record the actual GTK capture events without consuming
        # them, and await the target surface before beginning the gesture.
        native_events = []
        controller = Gtk.EventControllerMotion.new()
        controller.set_propagation_phase(Gtk.PropagationPhase.CAPTURE)
        def record_event(kind, *coordinates):
            if len(native_events) < 100:
                native_events.append(dict(type=kind, coordinates=coordinates, at=time.monotonic()))
        controller.connect('enter', lambda _controller, x, y: record_event('enter', x, y))
        controller.connect('motion', lambda _controller, x, y: record_event('motion', x, y))
        controller.connect('leave', lambda _controller: record_event('leave'))
        target.add_controller(controller)
        surface = target.get_surface()
        device = surface.get_display().get_default_seat().get_pointer()
        def event(*arguments):
            subprocess.run(['xdotool', *map(str, arguments)], env=capture_environment,
                           check=True, capture_output=True, timeout=5)
        try:
            event('mousemove', pointer_x, pointer_y)
            wait(lambda: snapshot()['pointer'] == [pointer_x, pointer_y],
                 'native pointer reaches the private fixture: ' + label)
            wait(lambda: surface.get_device_position(device)[0],
                 'GTK receives pointer entry on the exact native surface: ' + label)
            # Drag the actual native title bar. A modifier mask alone does not
            # prove Mutter accepted a move gesture on a Wayland client surface.
            event('mousedown', 1)
            wait(lambda: snapshot()['button_pressed'], 'native held button: ' + label)
            event('mousemove', pointer_x + 12, pointer_y + 12)
            wait(lambda: (value if (value := snapshot())['drag'] and
                          value['drag']['id'] == native['id'] else None), 'actual native held drag: ' + label)
            if protected:
                state = observed(lambda value: value['observed'] and value['drag_id'] is None
                                 and value['task'] != 'perch_ride', 'protected terminal is never a ride target')
            else:
                observed(lambda value: value['drag_id'] == native['id'] and value['drag_pid'] == os.getpid()
                         and value['task'] == 'perch_ride', 'actual goose starts native window ride')
                def perched(value):
                    return value['anchor'] and sum((a-b)**2 for a,b in
                        zip(value['position'], value['anchor'])) < 1
                state = observed(perched, 'goose reaches the actual window anchor')
                event('mousemove', pointer_x + 36, pointer_y + 24)
                moved = observed(lambda value: perched(value) and value['anchor'] != state['anchor'],
                                 'perched goose follows the actual user drag')
                (directory / f'native-ride-{label}.json').write_text(json.dumps(dict(before=state, moved=moved), indent=2))
                if interrupt:
                    interrupt(native)
                    cancelled = observed(lambda value: value['task'] != 'perch_ride',
                                         'actual held ride cancellation: ' + label)
                    (directory / f'interrupted-{label}.json').write_text(json.dumps(cancelled, indent=2))
            (directory / ('protected-drag.json' if protected else f'{label}-drag.json')).write_text(json.dumps(state, indent=2))
        except BaseException:
            failed = snapshot()
            failed['gtk_surface_pointer'] = list(surface.get_device_position(device))
            (directory / f'failed-grab-{label}.json').write_text(json.dumps(failed, indent=2))
            subprocess.run(['import', '-display', capture_environment['DISPLAY'], '-window', 'root',
                str(directory / f'failed-grab-{label}.png')], env=capture_environment, check=True, timeout=8)
            raise
        finally:
            event('mouseup', 1, 'keyup', 'Alt_L')
            (directory / f'input-events-{label}.json').write_text(json.dumps(native_events, indent=2))
            target.remove_controller(controller)
        wait(lambda: not (value := snapshot())['grabbed'] and not value['alt_pressed']
             and not value['button_pressed'], 'native drag and modifier release')
        observed(lambda value: value['drag_id'] is None and value['task'] != 'perch_ride',
                 'actual goose leaves the released window')

    def close(process):
        if process and process.poll() is None:
            process.terminate()
            try:
                process.wait(timeout=5)
            except subprocess.TimeoutExpired:
                process.kill()
                process.wait(timeout=5)

    try:
        control('integrations', 'gnome', 'remove')
        assert not record.exists() and not status()['installed']
        runtime, runtime_log = launch(binary, 'runtime', 'start', '--config', str(config))
        expect('unsupported', 'default-off observations')
        assert not workers()
        ui, _ = launch(settings, 'settings', '--config', str(config))
        wait(lambda: find('First wander (seconds)'), 'actual loaded settings')
        invoke('Appearance')
        toggle = wait(lambda: find('Reduced motion', Atspi.Role.TOGGLE_BUTTON), 'appearance draft')
        assert toggle.get_action_iface().do_action(0)
        wait(lambda: find('Unsaved changes'), 'unsaved appearance draft')
        invoke('Platform & status')
        invoke('Set up GNOME')
        wait(lambda: find('Enable GNOME observations', Atspi.Role.DIALOG), 'native GNOME consent dialog')
        invoke('Cancel')
        assert not record.exists(), 'Cancelled setup wrote consent'
        invoke('Set up GNOME')
        invoke('Enable GNOME observations')
        expect('supported', 'native settings explicitly enables the Rust observer')
        assert len(workers()) == 1
        foreign = record.parent / 'foreign.txt'
        foreign.write_text('Keep unrelated integration data')
        trace(False, False)
        # Even a client that knows this private fixture's consent cannot query the
        # production API without the kernel-authenticated approved executable.
        consent = json.loads(record.read_bytes())
        for _ in range(12):
            try:
                bus.call_sync(shell_owner, '/dev/emmetts/Honk300/Gnome1',
                    'dev.emmetts.Honk300.Gnome1', 'Snapshot', GLib.Variant('(s)', (consent['nonce'],)),
                    GLib.VariantType.new('(s)'), Gio.DBusCallFlags.NONE, 1000, None)
            except GLib.Error:
                pass
            else:
                raise AssertionError('Unapproved executable queried production observations')
        expect('supported', 'rejected foreign callers preserve the authenticated runtime')
        drag(window)
        drag(protected_window, protected=True, label='protected')
        invoke('Refresh status')
        window.set_visible(False)
        protected_window.set_visible(False)
        wait(lambda: not find_window(window.get_title()) and not find_window(protected_window.get_title()),
             'unobscured native settings capture')
        subprocess.run(['import', '-display', capture_environment['DISPLAY'], '-window', 'root',
                        str(directory / 'native-settings-goose.png')], env=capture_environment,
                       check=True, timeout=8)
        protected_window.present()
        window.present()
        wait(lambda: find_window(window.get_title()) and find_window(protected_window.get_title()),
             'restored private desktop fixtures')
        control('do', 'nab', success=False)
        window.fullscreen()
        wait(lambda: find_window('Honk300 ordinary GNOME probe')['fullscreen'], 'fixture fullscreen')
        trace(True, True)
        original_workers = workers()
        original_consent = record.read_bytes()
        control('integrations', 'gnome', 'setup')
        expect('supported', 'repeated CLI setup retains fullscreen observations')
        assert workers() == original_workers and record.read_bytes() == original_consent
        invoke('Set up GNOME')
        invoke('Enable GNOME observations')
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
        wait(lambda: not find_window('Honk300 ordinary GNOME probe')['fullscreen'], 'fixture leaves fullscreen')
        trace(False, False)
        def disable_rides(native):
            config.write_text(original.replace('no_window_ride = false', 'no_window_ride = true'))
            control('reload')
            observed(lambda value: value['observed'] and value['drag_id'] == native['id']
                     and value['task'] != 'perch_ride', 'live config ends a still-held ride')
            assert snapshot()['drag']['id'] == native['id'], 'Fixture released before config cancellation'
        drag(window, interrupt=disable_rides, label='config-disabled')
        config.write_text(original)
        control('reload')
        transient = Gtk.Window(title='Honk300 transient GNOME probe')
        transient.set_default_size(300, 200)
        transient.set_child(Gtk.Label(label='Close during an actual held ride'))
        transient.present()
        def destroy_target(native):
            assert snapshot()['drag']['id'] == native['id']
            transient.destroy()
            wait(lambda: find_window('Honk300 transient GNOME probe') is None, 'destroyed held native target')
            observed(lambda value: value['observed'] and value['drag_id'] is None
                     and value['task'] != 'perch_ride', 'destroyed target withdraws the live ride')
        try:
            drag(transient, interrupt=destroy_target, label='destroyed-target')
        finally:
            transient.destroy()
        invoke('Refresh status')
        invoke('Appearance')
        wait(lambda: (node := find('Reduced motion', Atspi.Role.TOGGLE_BUTTON)) and
             node.get_state_set().contains(Atspi.StateType.PRESSED), 'draft survives GNOME setup and status')
        assert config.read_text() == original
        invoke('Platform & status')
        previous_joins = joined_count()
        invoke('Remove GNOME observations')
        expect('unsupported', 'native settings revokes live observations')
        removed(previous_joins)
        # The runtime loses authority before the asynchronous native settings
        # action finishes disabling Shell and removing the recorded files.
        wait(lambda: find('GNOME companion and observations removed.'), 'native removal completion')
        assert not record.exists(), 'Completed native removal retained consent'
        control('integrations', 'gnome', 'setup')
        expect('supported', 'explicit CLI setup')
        saved_record = record.read_bytes()
        revoked = json.loads(saved_record)
        revoked['phase'] = 'revoking'
        previous_joins = joined_count()
        record.write_text(json.dumps(revoked))
        expect('unsupported', 'external durable consent revocation')
        removed(previous_joins)
        control('integrations', 'gnome', 'remove')
        control('integrations', 'gnome', 'setup')
        expect('supported', 'new consent after external revocation')
        def disable_extension(native):
            assert snapshot()['drag']['id'] == native['id']
            subprocess.run(['gnome-extensions', 'disable', 'honk300@emmetts.dev'],
                           check=True, capture_output=True, timeout=5)
            expect('failed', 'native extension disable withdraws observations during a held ride')
            trace(False, False, observed=False)
            observed(lambda value: not value['observed'] and value['drag_id'] is None
                     and value['task'] != 'perch_ride', 'extension loss cancels the still-held ride')
        drag(window, interrupt=disable_extension, label='extension-disabled')
        subprocess.run(['gnome-extensions', 'enable', 'honk300@emmetts.dev'],
                       check=True, capture_output=True, timeout=5)
        for _ in range(10):
            assert status()['capabilities']['windows'] == 'failed', 'Observer reconnected without explicit action'
            time.sleep(0.05)
        control('integrations', 'gnome', 'setup')
        expect('supported', 'explicit reconnection to the verified Shell')
        control('stop')
        wait(lambda: runtime.poll() is not None, 'graceful runtime stop', timeout=30)
        assert runtime.returncode == 0
        runtime = None
        assert find('Unsaved changes'), 'Stopping the goose discarded the independent settings draft'
        runtime, runtime_log = launch(binary, 'runtime', 'start', '--config', str(config))
        expect('supported', 'saved observation consent after graceful restart')
        close(ui)
        ui = None
        expect('supported', 'closing settings preserves the runtime owner')
        runtime.kill()
        runtime.wait(timeout=5)
        runtime = None
        runtime, runtime_log = launch(binary, 'runtime', 'start', '--config', str(config))
        expect('supported', 'observation worker after crash recovery')
        control('stop')
        wait(lambda: runtime.poll() is not None, 'second graceful stop', timeout=30)
        assert runtime.returncode == 0
        runtime = None
        control('integrations', 'gnome', 'remove')
        assert not record.exists() and not status()['installed']
        assert foreign.read_text() == 'Keep unrelated integration data'
        assert config.read_text() == original
        (directory / 'result.json').write_text(json.dumps(dict(ok=True, default_off=True,
            native_settings_consent=True, native_user_ride=True, protected_drag_excluded=True, foreign_caller_refused=True, engine_fullscreen_manners=True, live_config_toggle=True,
            ride_config_cancellation=True, destroyed_ride_target=True, extension_loss_during_ride=True,
            idempotent_setup_preserves_worker=True,
            draft_preserved=True, unsupported_actions=True, exact_worker_removed=True,
            live_remove=True, external_revocation=True, native_extension_disable=True,
            no_automatic_reconnect=True, graceful_restart=True, crash_restart=True,
            stopped_removal=True, unrelated_state_preserved=True), indent=2) + '\n')
    finally:
        (directory / 'observed-states.json').write_text(json.dumps(states, indent=2) + '\n')
        (directory / 'worker-identities.json').write_text(json.dumps(thread_states, indent=2) + '\n')
        try:
            if ui is not None and ui.poll() is None:
                diagnostic = []
                for node in nodes():
                    try:
                        diagnostic.append(dict(name=node.get_name(), role=node.get_role_name()))
                    except GLib.Error:
                        continue
                (directory / 'native-settings-tree.json').write_text(json.dumps(diagnostic, indent=2) + '\n')
        finally:
            close(ui)
            close(runtime)
            if record.exists():
                control('integrations', 'gnome', 'remove')
