import Gio from 'gi://Gio';
import GLib from 'gi://GLib';
import Meta from 'gi://Meta';
import {Extension} from 'resource:///org/gnome/shell/extensions/extension.js';
import * as Main from 'resource:///org/gnome/shell/ui/main.js';
import * as Config from 'resource:///org/gnome/shell/misc/config.js';

const BOUNDARY = 'gnome-observe-1';
const BUILD = '@@HONK300_GNOME_BUILD@@';
const IFACE = 'dev.emmetts.Honk300.Gnome1';
const XML = `<node><interface name="${IFACE}"><method name="Snapshot">
<arg type="s" direction="in"/><arg type="s" direction="out"/>
</method></interface></node>`;
const INFO = 'standard::type,standard::is-symlink,standard::size,unix::uid,unix::mode,unix::device,unix::inode';
const decoder = new TextDecoder();

function privateInfo(path, directory = false) {
    const file = Gio.File.new_for_path(path);
    const info = file.query_info(INFO, Gio.FileQueryInfoFlags.NOFOLLOW_SYMLINKS, null);
    if (info.get_is_symlink() || info.get_file_type() !==
        (directory ? Gio.FileType.DIRECTORY : Gio.FileType.REGULAR) ||
        info.get_attribute_uint32('unix::uid') !== new Gio.Credentials().get_unix_user() ||
        (info.get_attribute_uint32('unix::mode') & 0o077) !== 0)
        throw new Error('GNOME consent is not privately owned');
    return {file, info};
}

function readPrivate(path, limit) {
    const {file, info} = privateInfo(path);
    if (info.get_size() > limit)
        throw new Error('GNOME consent exceeds its bound');
    const stream = file.read(null);
    try {
        const bytes = stream.read_bytes(limit + 1, null).get_data();
        if (bytes.length > limit)
            throw new Error('GNOME consent exceeds its bound');
        return decoder.decode(bytes);
    } finally {
        stream.close(null);
    }
}

export default class Honk300Observations extends Extension {
    enable() {
        if (!['46.0', '48.7'].includes(Config.PACKAGE_VERSION) || !Meta.is_wayland_compositor())
            throw new Error('This GNOME version has not passed Honk300 qualification');
        this._generation = (this._generation ?? 0) + 1;
        this._active = true;
        this._drag = null;
        this._pending = false;
        this._begin = global.display.connect('grab-op-begin', (_display, window, op) => {
            this._drag = window && [Meta.GrabOp.MOVING, Meta.GrabOp.MOVING_UNCONSTRAINED,
                Meta.GrabOp.KEYBOARD_MOVING].includes(op)
                ? window.get_stable_sequence() : null;
        });
        this._end = global.display.connect('grab-op-end', () => { this._drag = null; });
        this._object = Gio.DBusExportedObject.wrapJSObject(XML, this);
        // Export only on Shell's unique connection. Rust authenticates org.gnome.Shell
        // through the bus and pins its process/executable before any private query.
        this._object.export(Gio.DBus.session, '/dev/emmetts/Honk300/Gnome1');
    }

    _consent() {
        const root = GLib.build_filenamev([GLib.get_user_data_dir(), 'honk300', 'wayland']);
        privateInfo(root, true);
        privateInfo(this.path, true);
        const consent = JSON.parse(readPrivate(`${root}/gnome.json`, 262144));
        if (consent.phase !== 'active' || consent.previous !== null ||
            consent.boundary !== BOUNDARY || !/^[0-9a-f]{32}$/.test(consent.nonce) ||
            consent.script !== readPrivate(`${this.path}/extension.js`, 65536) ||
            consent.metadata !== readPrivate(`${this.path}/metadata.json`, 4096))
            throw new Error('GNOME companion needs explicit setup for this update');
        return consent;
    }

    _credential(sender, method) {
        return new Promise((resolve, reject) => {
            Gio.DBus.session.call('org.freedesktop.DBus', '/org/freedesktop/DBus',
                'org.freedesktop.DBus', method, new GLib.Variant('(s)', [sender]),
                new GLib.VariantType('(u)'), Gio.DBusCallFlags.NO_AUTO_START, 75, null,
                (bus, result) => {
                    try { resolve(bus.call_finish(result).deep_unpack()[0]); }
                    catch (error) { reject(error); }
                });
        });
    }

    async SnapshotAsync([nonce], invocation) {
        const generation = this._generation;
        let ownsPending = false;
        try {
            if (!this._active || this._pending || nonce.length !== 32)
                throw new Error('GNOME observations are unavailable');
            const consent = this._consent();
            if (nonce !== consent.nonce)
                throw new Error('GNOME observation permission does not match');
            this._pending = ownsPending = true;
            const sender = invocation.get_sender();
            const [pid, uid] = await Promise.all([
                this._credential(sender, 'GetConnectionUnixProcessID'),
                this._credential(sender, 'GetConnectionUnixUser'),
            ]);
            if (uid !== new Gio.Credentials().get_unix_user())
                throw new Error('GNOME caller belongs to another user');
            const executable = Gio.File.new_for_path(`/proc/${pid}/exe`);
            const info = executable.query_info(INFO, Gio.FileQueryInfoFlags.NONE, null);
            if (info.get_attribute_uint32('unix::uid') !== new Gio.Credentials().get_unix_user() &&
                info.get_attribute_uint32('unix::uid') !== 0)
                throw new Error('GNOME caller executable has an unrelated owner');
            if (info.get_file_type() !== Gio.FileType.REGULAR ||
                info.get_attribute_as_string('unix::device') !== consent.executable.device ||
                info.get_attribute_as_string('unix::inode') !== consent.executable.inode ||
                info.get_size().toString() !== consent.executable.size ||
                GLib.file_read_link(`/proc/${pid}/exe`) !== consent.executable.path)
                throw new Error('GNOME caller is not the explicitly approved executable');
            if (!this._active || generation !== this._generation ||
                this._consent().nonce !== nonce)
                throw new Error('GNOME observations were revoked');
            const windows = global.get_window_actors().map(actor => actor.meta_window);
            if (windows.length > 64)
                throw new Error('Too many GNOME windows');
            const workspace = global.workspace_manager.get_active_workspace();
            const grabbed = global.display.is_grabbed();
            const data = {
                boundary: BOUNDARY, build: BUILD, version: Config.PACKAGE_VERSION,
                pid: new Gio.Credentials().get_unix_pid(), session_wayland: true,
                grabbed, overview: Main.overview.visible,
                windows: windows.map(window => {
                    const rect = window.get_frame_rect();
                    const pid = window.get_pid();
                    const app = window.get_gtk_application_id() || window.get_wm_class();
                    const title = window.get_title();
                    if ((app && app.length > 1024) || (title && title.length > 1024))
                        throw new Error('GNOME window identity exceeds its bound');
                    return {
                        id: window.get_stable_sequence(), pid: pid > 0 ? pid : null,
                        app, title, geometry: [rect.x, rect.y, rect.width, rect.height],
                        normal: window.get_window_type() === Meta.WindowType.NORMAL,
                        visible: !window.minimized && window.showing_on_its_workspace() &&
                            window.located_on_workspace(workspace),
                        fullscreen: window.is_fullscreen(),
                        dragging: grabbed && this._drag === window.get_stable_sequence(),
                    };
                }),
            };
            const raw = JSON.stringify(data);
            if (new TextEncoder().encode(raw).length > 65536)
                throw new Error('GNOME observations exceed their byte bound');
            invocation.return_value(new GLib.Variant('(s)', [raw]));
        } catch (_error) {
            // No nonce, private record, window content or executable identity is
            // included in error replies or Shell logs.
            invocation.return_dbus_error(`${IFACE}.Unavailable`, 'GNOME observations unavailable; check explicit setup and extension status');
        } finally {
            if (ownsPending && generation === this._generation)
                this._pending = false;
        }
    }

    disable() {
        this._active = false;
        this._generation = (this._generation ?? 0) + 1;
        if (this._begin) global.display.disconnect(this._begin);
        if (this._end) global.display.disconnect(this._end);
        this._begin = this._end = null;
        this._drag = null;
        this._object?.unexport();
        this._object = null;
    }
}
