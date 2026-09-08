"""Real Native SDK controls and AT-SPI consent/removal on private KDE desktops."""
import json
import os
from pathlib import Path
import subprocess


def qualify(binary, evidence, wait, call, GLib):
    assert os.environ.get('GITHUB_ACTIONS') == 'true'
    import gi
    gi.require_version('Atspi', '2.0')
    from gi.repository import Atspi
    directory = evidence / 'settings'
    directory.mkdir()
    config = directory / 'config.toml'
    original = '# Keep this native consent fixture\ngoose_config_version = 2\nfuture_key = "keep"\n'
    config.write_text(original)
    record = Path(os.environ['XDG_DATA_HOME']) / 'honk300/wayland/kwin.json'
    assert not record.exists()
    call('org.a11y.Bus', '/org/a11y/bus', 'org.freedesktop.DBus.Properties', 'Set',
         GLib.Variant('(ssv)', ('org.a11y.Status', 'IsEnabled', GLib.Variant('b', True))))
    Atspi.init()
    log = (directory / 'process.log').open('w')
    process = subprocess.Popen([str(binary), '--config', str(config)], stdout=log, stderr=log)

    def tree():
        assert process.poll() is None, f'Settings exited: {process.returncode}'
        desktop = Atspi.get_desktop(0)
        desktop.clear_cache()
        stack = [desktop.get_child_at_index(i) for i in range(desktop.get_child_count())]
        stack = [node for node in stack if node and node.get_process_id() == process.pid]
        result = []
        while stack and len(result) < 640:
            node = stack.pop()
            try:
                node.clear_cache()
                result.append(node)
                stack.extend(child for i in range(node.get_child_count())
                             if (child := node.get_child_at_index(i)))
            except GLib.Error:
                continue
        return result

    def find(name, role=None):
        return next((node for node in tree() if node.get_name() == name
                     and (role is None or node.get_role() == role)), None)

    def invoke(name):
        node = wait(lambda: find(name, Atspi.Role.PUSH_BUTTON), name)
        assert node.get_state_set().contains(Atspi.StateType.ENABLED), name
        assert node.get_action_iface().do_action(0), name

    try:
        wait(lambda: find('First wander (seconds)'), 'loaded KDE settings')
        invoke('Appearance')
        toggle = wait(lambda: find('Reduced motion', Atspi.Role.TOGGLE_BUTTON), 'appearance draft')
        assert not toggle.get_state_set().contains(Atspi.StateType.PRESSED)
        assert toggle.get_action_iface().do_action(0)
        wait(lambda: find('Unsaved changes'), 'retained unsaved draft')
        invoke('Platform & status')
        invoke('Set up KDE')
        wait(lambda: find('Enable KDE integration', Atspi.Role.DIALOG), 'native KDE consent dialog')
        assert not find('Appearance', Atspi.Role.PUSH_BUTTON), 'Consent exposed background controls'
        invoke('Cancel')
        wait(lambda: find('Set up KDE', Atspi.Role.PUSH_BUTTON), 'cancelled consent')
        assert not record.exists()
        assert config.read_text() == original
        invoke('Set up KDE')
        invoke('Enable KDE integration')
        wait(record.exists, 'Rust-owned KDE consent record')
        invoke('Remove KDE integration')
        wait(lambda: not record.exists(), 'native removal of KDE integration')
        assert config.read_text() == original
        invoke('Appearance')
        wait(lambda: (node := find('Reduced motion', Atspi.Role.TOGGLE_BUTTON))
             and node.get_state_set().contains(Atspi.StateType.PRESSED), 'draft survived setup and removal')
        invoke('Save & apply')
        wait(lambda: 'reduced_motion = true' in config.read_text(), 'native save after integration removal')
        saved = config.read_text()
        assert '# Keep this native consent fixture' in saved and 'future_key = "keep"' in saved
        assert not record.exists()
        (directory / 'result.json').write_text(json.dumps(dict(ok=True, native_atspi=True,
            explicit_consent=True, cancel_preserved_state=True, setup_and_removal=True,
            modal_isolation=True, draft_preserved=True, saved_comment_and_unknown_key=True), indent=2) + '\n')
    except Exception:
        if process.poll() is None:
            (directory / 'failed-tree.json').write_text(json.dumps([
                dict(name=node.get_name(), role=node.get_role_name()) for node in tree()
            ], indent=2) + '\n')
        raise
    finally:
        if process.poll() is None:
            process.terminate()
            try:
                process.wait(timeout=5)
            except subprocess.TimeoutExpired:
                process.kill()
                process.wait()
        log.close()
