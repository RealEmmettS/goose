import Gio from 'gi://Gio';
import GLib from 'gi://GLib';
import Meta from 'gi://Meta';
import Clutter from 'gi://Clutter';
import {Extension} from 'resource:///org/gnome/shell/extensions/extension.js';
import * as Main from 'resource:///org/gnome/shell/ui/main.js';
import * as Config from 'resource:///org/gnome/shell/misc/config.js';

const XML = `<node><interface name="dev.emmetts.Honk300.GnomeProbe1">
<method name="Snapshot"><arg type="s" direction="out"/></method>
<method name="ShowDesktop"><arg type="s" direction="out"/></method>
<method name="MoveFixture"><arg type="t" direction="in"/><arg type="u" direction="in"/>
<arg type="s" direction="in"/><arg type="s" direction="out"/></method>
<method name="FocusFixture"><arg type="t" direction="in"/><arg type="u" direction="in"/>
<arg type="s" direction="in"/><arg type="s" direction="out"/></method>
</interface></node>`;

export default class Probe extends Extension {
    enable() {
        if (GLib.getenv('GITHUB_ACTIONS') !== 'true')
            throw new Error('This probe only runs in disposable CI');
        this._drag = null;
        this._begin = global.display.connect('grab-op-begin', (_display, window, op) => {
            this._drag = {id: window.get_stable_sequence(), op};
        });
        this._end = global.display.connect('grab-op-end', () => { this._drag = null; });
        this._object = Gio.DBusExportedObject.wrapJSObject(XML, this);
        this._object.export(Gio.DBus.session, '/dev/emmetts/Honk300/GnomeProbe1');
        this._bus = Gio.bus_own_name_on_connection(Gio.DBus.session,
            'dev.emmetts.Honk300.GnomeProbe1', Gio.BusNameOwnerFlags.NONE, null, null);
        Main.overview.hide();
    }

    _windows() {
        return global.get_window_actors().map(actor => actor.meta_window);
    }

    _window(window) {
        const rect = window.get_frame_rect();
        return {id: window.get_stable_sequence(), pid: window.get_pid(),
            title: window.get_title(), app: window.get_wm_class(),
            gtk_app: window.get_gtk_application_id(), type: window.get_window_type(),
            client: window.get_client_type(), rect: [rect.x, rect.y, rect.width, rect.height],
            fullscreen: window.is_fullscreen(), minimized: window.minimized,
            focused: window.has_focus(),
            showing: window.showing_on_its_workspace(), monitor: window.get_monitor()};
    }

    Snapshot() {
        const windows = this._windows();
        const [x, y, modifiers] = global.get_pointer();
        if (windows.length > 64)
            throw new Error('Too many probe windows');
        return JSON.stringify({version: Config.PACKAGE_VERSION, pid: new Gio.Credentials().get_unix_pid(),
            display: GLib.getenv('DISPLAY'), wayland: GLib.getenv('WAYLAND_DISPLAY'),
            session_wayland: Meta.is_wayland_compositor(), drag: this._drag,
            grabbed: global.display.is_grabbed(),
            pointer: [x, y],
            alt_pressed: (modifiers & Clutter.ModifierType.MOD1_MASK) !== 0,
            button_pressed: (modifiers & Clutter.ModifierType.BUTTON1_MASK) !== 0,
            overview: Main.overview.visible, stage: [global.stage.width, global.stage.height],
            windows: windows.map(window => this._window(window))});
    }

    ShowDesktop() {
        Main.overview.hide();
        return 'ok';
    }

    MoveFixture(id, pid, expected) {
        const window = this._windows().find(value => value.get_stable_sequence() === id);
        const failures = [];
        if (!window) failures.push('window identity');
        if (pid !== Number(GLib.getenv('HONK300_GNOME_PROBE_PID'))) failures.push('fixture process');
        if (window && window.get_pid() !== pid) failures.push('window process');
        if (window && window.get_title() !== 'Honk300 ordinary GNOME probe') failures.push('title');
        if (window && window.get_wm_class() !== 'honk300-gnome-probe') failures.push('application');
        if (window && JSON.stringify(this._window(window).rect) !== expected) failures.push('stale geometry');
        if (this._drag || global.display.is_grabbed()) failures.push('active grab');
        if (failures.length) throw new Error('Stale or unrelated probe target: ' + failures.join(', '));
        const rect = window.get_frame_rect();
        window.move_frame(false, rect.x + 6, rect.y);
        return 'ok';
    }

    FocusFixture(id, pid, expected) {
        // Arrange only our private test windows. This API never ships in the
        // production companion and provides no production focus capability.
        const window = this._windows().find(value => value.get_stable_sequence() === id);
        if (!window || pid !== Number(GLib.getenv('HONK300_GNOME_PROBE_PID')) ||
            window.get_pid() !== pid || window.get_wm_class() !== 'honk300-gnome-probe' ||
            !['Honk300 ordinary GNOME probe', 'ChatGPT Codex terminal probe',
                'Honk300 transient GNOME probe'].includes(window.get_title()) ||
            JSON.stringify(this._window(window).rect) !== expected ||
            this._drag || global.display.is_grabbed())
            throw new Error('Stale or unrelated focus fixture');
        Main.overview.hide();
        // A restored nested Wayland fixture may land at (0, 0), underneath
        // Shell's top panel. Arrange only these authenticated private windows
        // inside the usable stage before sending a real title-bar gesture.
        const rect = window.get_frame_rect();
        if (rect.width > global.stage.width - 48 || rect.height > global.stage.height - 96)
            throw new Error('Private fixture does not fit the usable stage');
        window.move_frame(false, Math.round((global.stage.width - rect.width) / 2),
            Math.max(48, Math.round((global.stage.height - rect.height) / 2)));
        window.activate(global.get_current_time());
        return 'ok';
    }

    disable() {
        if (this._begin)
            global.display.disconnect(this._begin);
        if (this._end)
            global.display.disconnect(this._end);
        this._begin = this._end = null;
        this._drag = null;
        this._object?.unexport();
        this._object = null;
        if (this._bus)
            Gio.bus_unown_name(this._bus);
        this._bus = null;
    }
}
