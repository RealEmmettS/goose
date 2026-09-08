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

class ConsentRevoked extends Error {}

function ioAsync(object, method, finish, ...args) {
    return new Promise((resolve, reject) => {
        object[method](...args, (source, result) => {
            try { resolve(source[finish](result)); }
            catch (error) { reject(error); }
        });
    });
}

async function together(cancellable, tasks) {
    // Join every operation, including cancelled stream cleanup, before the
    // request releases its bounded admission slot.
    const results = await Promise.allSettled(tasks.map(async task => {
        try { return await task(); }
        catch (error) { cancellable.cancel(); throw error; }
    }));
    const failed = results.find(result => result.status === 'rejected');
    if (failed) throw failed.reason;
    return results.map(result => result.value);
}

async function privateInfo(path, cancellable, directory = false) {
    const file = Gio.File.new_for_path(path);
    const info = await ioAsync(file, 'query_info_async', 'query_info_finish',
        INFO, Gio.FileQueryInfoFlags.NOFOLLOW_SYMLINKS, GLib.PRIORITY_DEFAULT, cancellable);
    if (info.get_is_symlink() || info.get_file_type() !==
        (directory ? Gio.FileType.DIRECTORY : Gio.FileType.REGULAR) ||
        info.get_attribute_uint32('unix::uid') !== new Gio.Credentials().get_unix_user() ||
        (info.get_attribute_uint32('unix::mode') & 0o077) !== 0)
        throw new Error('GNOME consent is not privately owned');
    return {file, info};
}

async function readPrivate(path, limit, cancellable) {
    const {file, info} = await privateInfo(path, cancellable);
    if (info.get_size() > limit)
        throw new Error('GNOME consent exceeds its bound');
    const stream = await ioAsync(file, 'read_async', 'read_finish', GLib.PRIORITY_DEFAULT, cancellable);
    try {
        const opened = await ioAsync(stream, 'query_info_async', 'query_info_finish',
            INFO, GLib.PRIORITY_DEFAULT, cancellable);
        for (const attribute of ['unix::device', 'unix::inode', 'unix::uid', 'unix::mode']) {
            if (opened.get_attribute_as_string(attribute) !== info.get_attribute_as_string(attribute))
                throw new Error('GNOME consent changed while opening');
        }
        if (opened.get_file_type() !== Gio.FileType.REGULAR || opened.get_size() > limit)
            throw new Error('GNOME consent changed while opening');
        const chunks = [];
        let length = 0;
        while (true) {
            const bytes = (await ioAsync(stream, 'read_bytes_async', 'read_bytes_finish',
                Math.min(8192, limit + 1 - length), GLib.PRIORITY_DEFAULT, cancellable)).get_data();
            if (bytes.length === 0)
                break;
            length += bytes.length;
            if (length > limit)
                throw new Error('GNOME consent exceeds its bound');
            chunks.push(bytes);
        }
        const contents = new Uint8Array(length);
        let offset = 0;
        for (const chunk of chunks) {
            contents.set(chunk, offset);
            offset += chunk.length;
        }
        return decoder.decode(contents);
    } finally {
        // Cleanup remains asynchronous, even after the request was cancelled.
        await ioAsync(stream, 'close_async', 'close_finish', GLib.PRIORITY_DEFAULT, null);
    }
}

export default class Honk300Observations extends Extension {
    enable() {
        if (!['46.0', '48.7'].includes(Config.PACKAGE_VERSION) || !Meta.is_wayland_compositor())
            throw new Error('This GNOME version has not passed Honk300 qualification');
        this._generation = (this._generation ?? 0) + 1;
        this._active = true;
        this._drag = null;
        this._requests ??= new Set();
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

    async _consent(cancellable) {
        const root = GLib.build_filenamev([GLib.get_user_data_dir(), 'honk300', 'wayland']);
        await together(cancellable, [
            () => privateInfo(root, cancellable, true),
            () => privateInfo(this.path, cancellable, true),
        ]);
        const [record, script, metadata] = await together(cancellable, [
            () => readPrivate(`${root}/gnome.json`, 262144, cancellable),
            () => readPrivate(`${this.path}/extension.js`, 65536, cancellable),
            () => readPrivate(`${this.path}/metadata.json`, 4096, cancellable),
        ]);
        const consent = JSON.parse(record);
        if (consent.phase !== 'active' || consent.previous !== null)
            throw new ConsentRevoked();
        if (consent.boundary !== BOUNDARY || !/^[0-9a-f]{32}$/.test(consent.nonce) ||
            consent.script !== script || consent.metadata !== metadata)
            throw new Error('GNOME companion needs explicit setup for this update');
        return consent;
    }

    _credential(sender, method, cancellable) {
        return new Promise((resolve, reject) => {
            Gio.DBus.session.call('org.freedesktop.DBus', '/org/freedesktop/DBus',
                'org.freedesktop.DBus', method, new GLib.Variant('(s)', [sender]),
                new GLib.VariantType('(u)'), Gio.DBusCallFlags.NO_AUTO_START, 75, cancellable,
                (bus, result) => {
                    try { resolve(bus.call_finish(result).deep_unpack()[0]); }
                    catch (error) { reject(error); }
                });
        });
    }

    async SnapshotAsync([nonce], invocation) {
        const generation = this._generation;
        let cancellable = null;
        let timeout = 0;
        let expired = false;
        let stage = 'admission';
        try {
            if (!this._active || nonce.length !== 32)
                throw new Error('GNOME observations are unavailable');
            // An unapproved caller must not evict the real runtime by occupying
            // a single in-flight slot while its bus credentials are checked.
            // Bound the authentication work independently from snapshot validity.
            if (this._requests.size >= 4) {
                invocation.return_dbus_error(`${IFACE}.Busy`, 'GNOME observations are busy');
                return;
            }
            cancellable = new Gio.Cancellable();
            this._requests.add(cancellable);
            timeout = GLib.timeout_add(GLib.PRIORITY_DEFAULT, 200, () => {
                timeout = 0;
                expired = true;
                cancellable.cancel();
                return GLib.SOURCE_REMOVE;
            });
            stage = 'consent-before';
            const consent = await this._consent(cancellable);
            if (nonce !== consent.nonce)
                throw new ConsentRevoked();
            const sender = invocation.get_sender();
            stage = 'credentials';
            const [pid, uid] = await Promise.all([
                this._credential(sender, 'GetConnectionUnixProcessID', cancellable),
                this._credential(sender, 'GetConnectionUnixUser', cancellable),
            ]);
            if (uid !== new Gio.Credentials().get_unix_user())
                throw new Error('GNOME caller belongs to another user');
            stage = 'caller';
            const executable = Gio.File.new_for_path(`/proc/${pid}/exe`);
            const info = await ioAsync(executable, 'query_info_async', 'query_info_finish',
                INFO, Gio.FileQueryInfoFlags.NONE, GLib.PRIORITY_DEFAULT, cancellable);
            if (info.get_attribute_uint32('unix::uid') !== new Gio.Credentials().get_unix_user() &&
                info.get_attribute_uint32('unix::uid') !== 0)
                throw new Error('GNOME caller executable has an unrelated owner');
            if (info.get_file_type() !== Gio.FileType.REGULAR ||
                info.get_attribute_as_string('unix::device') !== consent.executable.device ||
                info.get_attribute_as_string('unix::inode') !== consent.executable.inode ||
                info.get_size().toString() !== consent.executable.size ||
                GLib.file_read_link(`/proc/${pid}/exe`) !== consent.executable.path)
                throw new Error('GNOME caller is not the explicitly approved executable');
            stage = 'consent-after';
            // Losing this extension instance does not revoke stored consent.
            // Native I/O may finish successfully after disable cancels it.
            if (!this._active || generation !== this._generation)
                throw new Error('GNOME companion is inactive');
            if ((await this._consent(cancellable)).nonce !== nonce)
                throw new ConsentRevoked();
            if (cancellable.is_cancelled())
                throw new Error('GNOME observation deadline exceeded');
            stage = 'windows';
            const actors = global.get_window_actors();
            const workspace = global.workspace_manager.get_active_workspace();
            const grabbed = global.display.is_grabbed();
            const windows = [];
            for (const actor of actors) {
                if (actor.is_destroyed() || !actor.meta_window)
                    continue;
                const window = actor.meta_window;
                const rect = window.get_frame_rect();
                // Empty actors have no live target. Apply the observation bound
                // after omitting them, without allocating an oversized report.
                if (rect.width === 0 || rect.height === 0)
                    continue;
                if (windows.length === 64)
                    throw new Error('Too many GNOME windows');
                const pid = window.get_pid();
                const app = window.get_gtk_application_id() || window.get_wm_class();
                const title = window.get_title();
                if ((app && app.length > 1024) || (title && title.length > 1024))
                    throw new Error('GNOME window identity exceeds its bound');
                windows.push({
                    id: window.get_stable_sequence(), pid: pid > 0 ? pid : null,
                    app, title, geometry: [rect.x, rect.y, rect.width, rect.height],
                    normal: window.get_window_type() === Meta.WindowType.NORMAL,
                    visible: !window.minimized && window.showing_on_its_workspace() &&
                        window.located_on_workspace(workspace),
                    fullscreen: window.is_fullscreen(),
                    dragging: grabbed && this._drag === window.get_stable_sequence(),
                });
            }
            const data = {
                boundary: BOUNDARY, build: BUILD, version: Config.PACKAGE_VERSION,
                pid: new Gio.Credentials().get_unix_pid(), session_wayland: true,
                grabbed, overview: Main.overview.visible, windows,
            };
            const raw = JSON.stringify(data);
            if (new TextEncoder().encode(raw).length > 65536)
                throw new Error('GNOME observations exceed their byte bound');
            invocation.return_value(new GLib.Variant('(s)', [raw]));
        } catch (error) {
            // No nonce, private record, window content or executable identity is
            // included in error replies or Shell logs.
            const reason = error instanceof ConsentRevoked ? 'Revoked' : expired ? 'Deadline' : 'Unavailable';
            invocation.return_dbus_error(`${IFACE}.${reason}`, `GNOME observation failed during ${stage}`);
        } finally {
            if (timeout) GLib.source_remove(timeout);
            if (cancellable) this._requests.delete(cancellable);
        }
    }

    disable() {
        this._active = false;
        this._generation = (this._generation ?? 0) + 1;
        for (const request of this._requests ?? []) request.cancel();
        if (this._begin) global.display.disconnect(this._begin);
        if (this._end) global.display.disconnect(this._end);
        this._begin = this._end = null;
        this._drag = null;
        this._object?.unexport();
        this._object = null;
    }
}
