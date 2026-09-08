// Exercise the production Shell method with controlled native API responses.
// Actual GIO bindings, compositor input and revocation also run on both Shells.
import assert from 'node:assert/strict';
import {readFileSync} from 'node:fs';
import vm from 'node:vm';
import test from 'node:test';

const nonce = 'a'.repeat(32);
const source = readFileSync(new URL('../../../integrations/gnome/extension.js', import.meta.url), 'utf8')
    .replace(/^import .*;\r?\n/gm, '')
    .replace('export default class Honk300Observations', 'class Honk300Observations');

function fixture(actors = []) {
    const timers = new Map();
    let timerId = 0;
    class Cancellable {
        cancelled = false;
        listeners = [];
        cancel() { this.cancelled = true; for (const callback of this.listeners) callback(); }
        is_cancelled() { return this.cancelled; }
    }
    const attributes = {'unix::device': '1', 'unix::inode': '2', 'unix::uid': '1000', 'unix::mode': '384'};
    const info = {
        get_is_symlink: () => false,
        get_file_type: () => 1,
        get_attribute_uint32: name => Number(attributes[name]),
        get_attribute_as_string: name => attributes[name],
        get_size: () => 100,
    };
    const file = {
        query_info_async(...args) { queueMicrotask(() => args.at(-1)(file, info)); },
        query_info_finish: result => result,
    };
    const context = {
        Gio: {
            File: {new_for_path: () => file}, FileQueryInfoFlags: {NONE: 0, NOFOLLOW_SYMLINKS: 1},
            FileType: {REGULAR: 1, DIRECTORY: 2}, Cancellable,
            Credentials: class { get_unix_user() { return 1000; } get_unix_pid() { return 500; } },
        },
        GLib: {
            PRIORITY_DEFAULT: 0, SOURCE_REMOVE: false,
            file_read_link: () => '/approved/honk300',
            timeout_add(_priority, delay, callback) {
                const id = ++timerId;
                timers.set(id, setTimeout(() => { timers.delete(id); callback(); }, delay));
                return id;
            },
            source_remove(id) { clearTimeout(timers.get(id)); timers.delete(id); },
            Variant: class { constructor(_type, value) { this.value = value; } },
        },
        Extension: class {}, Meta: {WindowType: {NORMAL: 0}}, Main: {overview: {visible: false}},
        Config: {PACKAGE_VERSION: '46.0'}, TextDecoder, TextEncoder,
        global: {
            get_window_actors: () => actors,
            display: {is_grabbed: () => false},
            workspace_manager: {get_active_workspace: () => ({})},
        },
    };
    vm.runInNewContext(source + '\nglobalThis.api = {Honk300Observations, ConsentRevoked, readPrivate};', context);
    const extension = new context.api.Honk300Observations();
    Object.assign(extension, {_generation: 1, _active: true, _drag: null, _requests: new Set()});
    extension._consent = async () => ({nonce, executable: {device: '1', inode: '2', size: '100', path: '/approved/honk300'}});
    extension._credential = async (_sender, method) => method.endsWith('User') ? 1000 : 600;
    function request() {
        const result = {};
        const finished = extension.SnapshotAsync([nonce], {
            get_sender: () => ':1.2',
            return_value: value => { result.frame = JSON.parse(value.value[0]); },
            return_dbus_error: (name, message) => { result.error = name; result.message = message; },
        });
        return {result, finished};
    }
    return {extension, request, context, file, info, timers};
}

function actor(id, width = 300) {
    return {
        is_destroyed: () => false,
        meta_window: {
            get_frame_rect: () => ({x: 0, y: 50, width, height: 200}),
            get_pid: () => 600, get_gtk_application_id: () => 'org.example.Editor',
            get_title: () => 'An ordinary document', get_stable_sequence: () => id,
            get_window_type: () => 0, minimized: false,
            showing_on_its_workspace: () => true, located_on_workspace: () => true,
            is_fullscreen: () => false,
        },
    };
}

test('discard transient actors before enforcing the 64 reported-window bound', async () => {
    const actors = Array.from({length: 64}, (_, i) => actor(i + 1));
    actors.push({is_destroyed: () => true}, {is_destroyed: () => false, meta_window: null}, actor(65, 0));
    const f = fixture(actors);
    const {result, finished} = f.request();
    await finished;
    assert.equal(result.error, undefined);
    assert.equal(result.frame.windows.length, 64);
    assert.equal(f.extension._requests.size, 0);
    assert.equal(f.timers.size, 0);
});

test('65 actual reportable windows still fail closed', async () => {
    const f = fixture(Array.from({length: 65}, (_, i) => actor(i + 1)));
    const {result, finished} = f.request();
    await finished;
    assert.equal(result.frame, undefined);
    assert.equal(result.error, 'dev.emmetts.Honk300.Gnome1.Unavailable');
});

test('slow consent stays asynchronous, bounded and cancellable', async () => {
    const f = fixture();
    f.extension._consent = cancellable => new Promise((_resolve, reject) => {
        cancellable.listeners.push(() => reject(new Error('Cancelled')));
    });
    const pending = Array.from({length: 4}, () => f.request());
    const busy = f.request();
    await busy.finished;
    assert.equal(busy.result.error, 'dev.emmetts.Honk300.Gnome1.Busy');
    await new Promise(resolve => setTimeout(resolve, 10));
    assert.equal(f.extension._requests.size, 4, 'The main loop must continue while all reads wait');
    await Promise.all(pending.map(request => request.finished));
    for (const {result} of pending) assert.equal(result.error, 'dev.emmetts.Honk300.Gnome1.Unavailable');
    assert.equal(f.extension._requests.size, 0);
    assert.equal(f.timers.size, 0);
});

test('explicit revocation remains distinct from a failed observation', async () => {
    const f = fixture();
    f.extension._consent = async () => { throw new f.context.api.ConsentRevoked(); };
    const {result, finished} = f.request();
    await finished;
    assert.equal(result.error, 'dev.emmetts.Honk300.Gnome1.Revoked');
    assert(!result.message.includes(nonce));
});

test('disable cancels retained requests and prevents a late publication', async () => {
    const f = fixture();
    f.extension._consent = cancellable => new Promise((_resolve, reject) => {
        cancellable.listeners.push(() => reject(new Error('Cancelled')));
    });
    const pending = f.request();
    f.extension.disable();
    await pending.finished;
    assert.equal(pending.result.frame, undefined);
    assert.equal(f.extension._requests.size, 0);
    assert.equal(f.timers.size, 0);
});
