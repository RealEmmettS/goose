"""Native KDE portal UI and real libei sender qualification on disposable CI."""
import json
import os
from pathlib import Path
import subprocess


def qualify(binary, evidence, wait, call, GLib, Gtk, RustBridge):
    assert os.environ.get('GITHUB_ACTIONS') == 'true'
    directory = evidence / 'portal'
    directory.mkdir()
    configuration = Path(os.environ['XDG_CONFIG_HOME']) / 'xdg-desktop-portal'
    configuration.mkdir()
    (configuration / 'portals.conf').write_text('[preferred]\ndefault=kde\n')
    environment = dict(os.environ, QT_QPA_PLATFORM='wayland', QT_ACCESSIBILITY='1',
                       QT_LINUX_ACCESSIBILITY_ALWAYS_ON='1', LANG='C.UTF-8')
    call('org.freedesktop.DBus', '/org/freedesktop/DBus', 'org.freedesktop.DBus',
         'UpdateActivationEnvironment', GLib.Variant('(a{ss})', ({key: value for key, value in environment.items()
           if key in ('WAYLAND_DISPLAY', 'XDG_RUNTIME_DIR', 'XDG_CONFIG_HOME', 'XDG_DATA_HOME',
                      'XDG_CURRENT_DESKTOP', 'XDG_SESSION_TYPE', 'QT_QPA_PLATFORM', 'QT_ACCESSIBILITY',
                      'QT_LINUX_ACCESSIBILITY_ALWAYS_ON', 'LANG')},)))
    call('org.a11y.Bus', '/org/a11y/bus', 'org.freedesktop.DBus.Properties', 'Set',
         GLib.Variant('(ssv)', ('org.a11y.Status', 'IsEnabled', GLib.Variant('b', True))))
    import gi
    gi.require_version('Atspi', '2.0')
    from gi.repository import Atspi
    Atspi.init()
    processes = []
    logs = []
    bridge = None
    window = None

    def launch(executable, name, *arguments):
        log = (directory / f'{name}.log').open('w')
        logs.append(log)
        process = subprocess.Popen([executable, *arguments], env=environment, stdout=log, stderr=log)
        processes.append(process)
        return process

    def program(package, suffix):
        files = subprocess.check_output(['dpkg-query', '-L', package], text=True).splitlines()
        return next(path for path in files if path.endswith('/' + suffix) and os.access(path, os.X_OK))

    def nodes(pid):
        desktop = Atspi.get_desktop(0)
        desktop.clear_cache()
        stack = [desktop.get_child_at_index(i) for i in range(desktop.get_child_count())]
        stack = [node for node in stack if node and node.get_process_id() == pid]
        result = []
        while stack and len(result) < 512:
            node = stack.pop()
            try:
                node.clear_cache()
                result.append(node)
                stack.extend(child for i in range(node.get_child_count()) if (child := node.get_child_at_index(i)))
            except GLib.Error:
                continue
        return result

    try:
        assert (Path(environment['XDG_RUNTIME_DIR']) / 'pipewire-0').is_socket()
        backend = launch(program('xdg-desktop-portal-kde', 'xdg-desktop-portal-kde'), 'kde')
        desktop = launch(program('xdg-desktop-portal', 'xdg-desktop-portal'), 'desktop', '--verbose', '--replace')
        wait(lambda: call('org.freedesktop.DBus', '/org/freedesktop/DBus', 'org.freedesktop.DBus',
            'NameHasOwner', GLib.Variant('(s)', ('org.freedesktop.portal.Desktop',))).unpack()[0], 'native portal service')
        wait(lambda: call('org.freedesktop.DBus', '/org/freedesktop/DBus', 'org.freedesktop.DBus',
            'NameHasOwner', GLib.Variant('(s)', ('org.freedesktop.impl.portal.desktop.kde',))).unpack()[0],
            'native KDE portal backend')

        def owns_desktop_service():
            assert desktop.poll() is None, 'Private portal frontend exited'
            try:
                return call('org.freedesktop.DBus', '/org/freedesktop/DBus', 'org.freedesktop.DBus',
                    'GetConnectionUnixProcessID', GLib.Variant('(s)', ('org.freedesktop.portal.Desktop',))).unpack()[0] == desktop.pid
            except GLib.Error:
                return False

        wait(owns_desktop_service, 'fresh private frontend owns the portal service')

        def remote_desktop_ready():
            try:
                return call('org.freedesktop.portal.Desktop', '/org/freedesktop/portal/desktop',
                    'org.freedesktop.DBus.Properties', 'Get',
                    GLib.Variant('(ss)', ('org.freedesktop.portal.RemoteDesktop', 'version'))).unpack()[0]
            except GLib.Error:
                return False

        wait(remote_desktop_ready, 'exported RemoteDesktop interface')
        owner_pids = {name: call('org.freedesktop.DBus', '/org/freedesktop/DBus', 'org.freedesktop.DBus',
            'GetConnectionUnixProcessID', GLib.Variant('(s)', (name,))).unpack()[0]
            for name in ('org.freedesktop.portal.Desktop', 'org.freedesktop.impl.portal.desktop.kde')}
        (directory / 'owners.json').write_text(json.dumps(dict(owners=owner_pids,
            launched=[dict(pid=process.pid, exit=process.poll()) for process in processes]), indent=2) + '\n')
        backend_pid = owner_pids['org.freedesktop.impl.portal.desktop.kde']
        window = Gtk.Window()
        window.set_title('Honk300 portal ordinary window')
        window.set_default_size(300, 200)
        window.set_child(Gtk.Label(label='Portal pointer test target'))
        window.present()
        wait(window.get_mapped, 'portal test window')
        bridge = RustBridge(binary, directory)
        assert bridge.request('activate')['ok']
        wait(lambda: bridge.request('snapshot')['snapshot'], 'fresh portal guard observations')
        assert bridge.request('portal_request')['ok']
        last_tree = []
        approved = False

        def granted():
            nonlocal last_tree, approved
            reply = bridge.request('portal_poll')
            if not reply['ok']:
                raise AssertionError(reply)
            if reply['ready']:
                return True
            current = nodes(backend_pid)
            last_tree = [dict(name=node.get_name(), role=node.get_role_name()) for node in current]
            (directory / 'native-consent-tree.json').write_text(json.dumps(last_tree, indent=2) + '\n')
            if not approved:
                candidates = [node for node in current if node.get_role() == Atspi.Role.PUSH_BUTTON
                              and node.get_name() in ('Share', 'Allow')]
                if len(candidates) == 1:
                    assert candidates[0].get_action_iface().do_action(0), last_tree
                    approved = True
            return False

        wait(granted, 'native permission dialog and granted absolute pointer device', timeout=45)
        assert approved, 'A pointer device appeared without the fixture accepting native consent'
        snapshot = bridge.request('snapshot')['snapshot']
        target = [snapshot['pointer'][0] + 6, snapshot['pointer'][1]]
        reply = bridge.request('portal_warp', to=target)
        assert reply['ok'], reply
        wait(lambda: (frame := bridge.request('snapshot')['snapshot']) and
             abs(frame['pointer'][0] - target[0]) < 0.1 and abs(frame['pointer'][1] - target[1]) < 0.1,
             'actual portal-authorized pointer movement')
        window.set_title('ChatGPT Codex terminal probe')
        protected = wait(lambda: next((item for item in (bridge.request('snapshot')['snapshot'] or {}).get('windows', [])
            if item['title'] == 'ChatGPT Codex terminal probe' and item['protected']), None), 'live terminal exclusion')
        x, y, width, height = protected['geometry']
        pointer = bridge.request('snapshot')['snapshot']['pointer']
        assert x <= pointer[0] < x + width and y <= pointer[1] < y + height, (protected, pointer)
        assert not bridge.request('portal_warp', to=[pointer[0] + 2, pointer[1]])['ok']
        assert not bridge.request('portal_warp', to=[pointer[0] + 200, pointer[1]])['ok']
        assert bridge.request('portal_cancel')['ok']
        assert not bridge.request('portal_poll')['ok']
        (directory / 'result.json').write_text(json.dumps(dict(ok=True, native_consent=True,
            real_libei_device=True, actual_pointer_motion=True, terminal_refused=True,
            excessive_motion_refused=True, explicit_cancel=True), indent=2) + '\n')
    finally:
        if bridge:
            bridge.close()
        if window:
            window.destroy()
        for process in reversed(processes):
            process.terminate()
            try:
                process.wait(timeout=5)
            except subprocess.TimeoutExpired:
                process.kill()
                process.wait()
        for log in logs:
            log.close()
