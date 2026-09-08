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

function privateFiles(f, {changedIdentity = false, holdCleanup = false} = {}) {
    const root = '/private/honk300/wayland';
    const extensionPath = '/private/extension';
    const metadata = '{"uuid":"honk300@emmetts.dev"}';
    const script = 'the explicitly approved companion';
    const consent = {phase: 'active', previous: null, boundary: 'gnome-observe-1', nonce,
        script, metadata, executable: {device: '1', inode: '2', size: '100', path: '/approved/honk300'}};
    const entries = new Map([
        [root, null], [extensionPath, null],
        [`${root}/gnome.json`, JSON.stringify(consent)],
        [`${extensionPath}/extension.js`, script], [`${extensionPath}/metadata.json`, metadata],
    ]);
    f.context.GLib.get_user_data_dir = () => '/private';
    f.context.GLib.build_filenamev = parts => parts.join('/');
    f.extension.path = extensionPath;
    f.extension._consent = f.context.api.Honk300Observations.prototype._consent.bind(f.extension);
    const state = {open: 0, peak: 0, batches: 0, releaseCleanup: null};
    let cleanupStarted;
    state.cleanupStarted = new Promise(resolve => { cleanupStarted = resolve; });
    let pending = [];
    const finish = result => { if (result instanceof Error) throw result; return result; };
    f.context.Gio.File.new_for_path = path => {
        if (path.startsWith('/proc/')) return f.file;
        assert(entries.has(path), path);
        const content = entries.get(path);
        const bytes = new TextEncoder().encode(content ?? '');
        const directory = content === null;
        const attributes = {'unix::device': '1', 'unix::inode': String([...entries.keys()].indexOf(path) + 10),
            'unix::uid': '1000', 'unix::mode': directory ? '448' : '384'};
        const info = {...f.info, get_file_type: () => directory ? 2 : 1, get_size: () => bytes.length,
            get_attribute_uint32: name => Number(attributes[name]),
            get_attribute_as_string: name => attributes[name]};
        const callback = (source, result, cancellable, done) => queueMicrotask(() =>
            done(source, cancellable?.is_cancelled() ? new Error('Cancelled') : result));
        const file = {
            query_info_async(_attrs, _flags, _priority, cancellable, done) { callback(file, info, cancellable, done); },
            query_info_finish: finish,
            read_async(_priority, cancellable, done) {
                let offset = 0;
                const openedInfo = changedIdentity && path.endsWith('/extension.js') ? {...info,
                    get_attribute_as_string: name => name === 'unix::inode' ? 'unrelated' : attributes[name]} : info;
                const stream = {
                    query_info_async(_attrs, _priority, cancel, complete) { callback(stream, openedInfo, cancel, complete); },
                    query_info_finish: finish,
                    read_bytes_async(size, _priority, cancel, complete) {
                        const chunk = bytes.slice(offset, offset + size);
                        offset += chunk.length;
                        callback(stream, {get_data: () => chunk}, cancel, complete);
                    },
                    read_bytes_finish: finish,
                    close_async(_priority, _cancel, complete) {
                        const close = () => queueMicrotask(() => complete(stream, true));
                        if (holdCleanup && path.endsWith('/metadata.json')) {
                            state.releaseCleanup = close;
                            cleanupStarted();
                        } else close();
                    },
                    close_finish(result) { state.open--; return result; },
                };
                let completed = false;
                const deliver = () => {
                    if (completed) return;
                    completed = true;
                    callback(file, stream, cancellable, done);
                };
                cancellable.listeners.push(deliver);
                pending.push(deliver);
                // The real _consent must issue all three independent reads;
                // a serial implementation cannot pass this native-I/O barrier.
                if (pending.length === 3) {
                    state.batches++;
                    const batch = pending;
                    pending = [];
                    batch.forEach(deliver => deliver());
                }
            },
            read_finish(result) {
                const stream = finish(result);
                state.peak = Math.max(state.peak, ++state.open);
                return stream;
            },
        };
        return file;
    };
    return state;
}

test('production consent reads independent files together and verifies them twice', async () => {
    const f = fixture();
    const files = privateFiles(f);
    const request = f.request();
    await request.finished;
    assert.equal(request.result.error, undefined);
    assert.equal(request.result.frame.windows.length, 0);
    assert.equal(files.batches, 2);
    assert.equal(files.peak, 3);
    assert.equal(files.open, 0);
    assert.equal(f.extension._requests.size, 0);
});

test('changed opened identity cancels and joins other consent reads before releasing admission', async () => {
    const f = fixture();
    const files = privateFiles(f, {changedIdentity: true, holdCleanup: true});
    const request = f.request();
    await files.cleanupStarted;
    assert.equal(f.extension._requests.size, 1, 'Pending cleanup must retain the request slot');
    assert.equal(request.result.frame, undefined);
    files.releaseCleanup();
    await request.finished;
    assert.equal(request.result.error, 'dev.emmetts.Honk300.Gnome1.Unavailable');
    assert.equal(files.open, 0);
    assert.equal(f.extension._requests.size, 0);
    assert(!request.result.message.includes(nonce));
});

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
    const healthyConsent = f.extension._consent;
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
    for (const {result} of pending) {
        assert.equal(result.error, 'dev.emmetts.Honk300.Gnome1.Deadline');
        assert.equal(result.message, 'GNOME observation failed during consent-before');
    }
    assert.equal(f.extension._requests.size, 0);
    assert.equal(f.timers.size, 0);
    // A timed-out request publishes nothing and releases its slot. A new
    // request must perform live consent again before it can report a frame.
    f.extension._consent = healthyConsent;
    const recovered = f.request();
    await recovered.finished;
    assert.equal(recovered.result.error, undefined);
    assert.equal(recovered.result.frame.windows.length, 0);
    assert.equal(f.extension._requests.size, 0);
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
