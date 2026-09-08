"""Read-only compositor observations for the disposable actual-runtime fixture."""
import json
import os
import threading
import time


class NativeObserver:
    def __init__(self, evidence, call, GLib):
        assert os.environ.get('GITHUB_ACTIONS') == 'true'
        import gi
        gi.require_version('Gio', '2.0')
        from gi.repository import Gio
        self.call, self.GLib = call, GLib
        self.evidence = evidence
        self.bus = Gio.bus_get_sync(Gio.BusType.SESSION, None)
        self.frames = []
        self.count = 0
        self.latest = []
        self.times = []
        self.owner = call('org.freedesktop.DBus', '/org/freedesktop/DBus', 'org.freedesktop.DBus',
            'GetNameOwner', GLib.Variant('(s)', ('org.kde.KWin',))).unpack()[0]
        self.context = GLib.MainContext.new()
        self.loop = GLib.MainLoop.new(self.context, False)
        self.ready = threading.Event()
        self.thread = threading.Thread(target=self.serve, daemon=True)
        self.interface = Gio.DBusNodeInfo.new_for_xml('''<node><interface name="org.emmetts.Honk300.NativeObserver">
          <method name="Capture"><arg type="s" direction="in"/></method>
        </interface></node>''').interfaces[0]
        self.thread.start()
        assert self.ready.wait(3)
        source = evidence / 'native-observer.js'
        source.write_text('''(function () {
            var timer = new QTimer(); timer.interval = 50;
            var pending = false;
            timer.timeout.connect(function () {
                if (pending) return;
                var windows = workspace.stackingOrder !== undefined ? workspace.stackingOrder : workspace.clientList();
                if (!windows || windows.length > 64) return;
                var result = [];
                for (var i = 0; i < windows.length; i++) {
                    var window = windows[i], rect = window.frameGeometry;
                    result.push({id:String(window.internalId),pid:Number(window.pid),app:String(window.resourceClass),
                        title:String(window.caption),geometry:[rect.x,rect.y,rect.width,rect.height],
                        normal:window.normalWindow,dragging:window.move,deleted:window.deleted,
                        minimized:window.minimized,fullscreen:window.fullScreen,moveable:window.moveable});
                }
                pending = true;
                callDBus(''' + json.dumps(self.bus.get_unique_name()) + ''', '/org/emmetts/Honk300/NativeObserver',
                    'org.emmetts.Honk300.NativeObserver', 'Capture', JSON.stringify(result), function () { pending = false; });
            }); timer.start();
        }());''')
        identifier = call('org.kde.KWin', '/Scripting', 'org.kde.kwin.Scripting', 'loadScript',
            GLib.Variant('(ss)', (str(source), 'honk300-fixture-observer'))).unpack()[0]
        assert identifier >= 0
        major = int((evidence.parent / 'version.txt').read_text().split()[-1].split('.')[0])
        path = f'/Scripting/Script{identifier}' if major >= 6 else f'/{identifier}'
        call('org.kde.KWin', path, 'org.kde.kwin.Script', 'run')

    def serve(self):
        self.context.push_thread_default()
        registration = self.bus.register_object('/org/emmetts/Honk300/NativeObserver',
            self.interface, self.capture, None, None)
        self.ready.set()
        try:
            self.loop.run()
        finally:
            self.bus.unregister_object(registration)
            self.context.pop_thread_default()

    def capture(self, _connection, sender, _path, _interface, _method, parameters, invocation):
        try:
            assert sender == self.owner
            raw = parameters.unpack()[0]
            assert len(raw) <= 65536
            frame = json.loads(raw)
            assert len(frame) <= 64
            self.latest = frame
            self.count += 1
            self.frames.append(frame)
            self.frames = self.frames[-256:]
            self.times.append(time.monotonic())
            self.times = self.times[-256:]
            invocation.return_value(None)
        except Exception as error:
            invocation.return_dbus_error('org.emmetts.Honk300.Invalid', str(error))

    def close(self):
        (self.evidence / 'native-observer-frames.json').write_text(json.dumps(
            dict(count=self.count, frames=self.frames, times=self.times), indent=2) + '\n')
        self.call('org.kde.KWin', '/Scripting', 'org.kde.kwin.Scripting', 'unloadScript',
            self.GLib.Variant('(s)', ('honk300-fixture-observer',)))
        self.loop.quit()
        self.thread.join(timeout=3)
        assert not self.thread.is_alive()
