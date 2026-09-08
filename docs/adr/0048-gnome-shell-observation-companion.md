# ADR 0048: versioned GNOME Shell observations and compatible overlay

Status: accepted; production desktop qualification passed, publication pending.

The approved refinement requires separate GNOME evidence. Native run
[34226098889](https://github.com/RealEmmettS/goose/actions/runs/34226098889)
qualifies GNOME Shell 46.0 and 48.7 on x64 and ARM64: actual window identities,
fullscreen, Mutter grab begin/end, refusal of automatic movement during a user
grab, and the complete visible goose outside overview. GNOME does not advertise
layer-shell; retain its compatible XWayland overlay and X11-owned prop controller.

The opt-in companion exports a read-only bounded snapshot on Shell's existing
unique bus connection. Rust authenticates the bus owner using kernel process/user
credentials and pins the loaded system-owned gnome-shell executable. Each connect
and snapshot has a total deadline, including bus authentication and owner checks.
The extension accepts only the recorded executable identity and private consent
nonce, and reports its exact embedded build identity. A stale loaded extension
cannot silently satisfy a newer Rust boundary. No evaluation, general command,
foreign-window movement, global pointer action or DND API is exposed.

Actual Mutter user-drag signals authorize the existing engine window ride only
for a visible normal window with complete current identity, geometry and a
nonterminal application/title. Releasing the drag, overview, stale observations,
configuration changes or removal withdraws the target. The goose moves itself;
the companion never moves another application's window. On a GNOME Wayland
session the XWayland overlay cannot supply blind global pointer or window-watch
capabilities for native windows. Fullscreen presence remains independent of
optional target identity and user action eligibility.

Explicit setup installs only honk300@emmetts.dev with private owned files, a
durable pending-to-active record and exact payload/executable identities. First
discovery or an updated loaded module may require signing out and in. Setup
never changes the global extension switch or another extension. It uses the
supported Shell enable interface, with no unsafe module reload. Removal first
records revocation, verifies owned contents, disables only the exact companion,
then removes those verified files. Interrupted install/removal resumes from the
recorded phase; edited or unknown contents are preserved and reported.

The CLI, runtime IPC and Native SDK dialogs share Rust ownership. Repeated setup
retains the same live worker and permission. Settings drafts survive setup,
removal and goose shutdown. Observations end on malformed replies, stale owners,
extension removal or lost consent; reconnecting requires an explicit action.
Credential lookup allows at most four concurrent requests. A busy response from
the same pinned Shell withdraws any old frame and may retry a bounded snapshot;
an unapproved caller cannot terminate the real worker merely by using a lookup slot.
Bounded query timeouts use the same retained connection and recheck its original
Shell owner and full live consent before publishing again. This never reconnects
to a new bus owner or restores a revoked grant. Connection establishment, identity,
permission, extension and decoding failures remain terminal. Native diagnostic
run 34256746560 records a real deadline during the final consent read; it does not
justify extending the 200 ms Shell or 250 ms transport/sample bounds.

Consent monitoring is part of that authenticated worker exchange. The rendering
thread performs no periodic user-data file reads. Shell reads private metadata
and bounded file contents through [asynchronous GIO operations](https://gjs.guide/guides/gio/file-operations.html),
checks the opened stream's identity and cancels the complete request after 200 ms
or extension removal. Admission is bounded before any file work. A distinct
authenticated revocation reply retires and joins the retained Rust worker;
other unavailable replies remain failed until explicit reconnection. No cancelled
or obsolete generation may publish a snapshot. Discard destroyed or empty native
actors before counting the 64 reportable windows, and reject the next live entry
without building an oversized report. GNOME Wayland always supplies an absent
pointer to the engine until an independent native pointer path is qualified.

The production desktop fixture must exercise the actual companion, Rust runtime,
native settings controls, ordinary and terminal user drags, fullscreen manners,
wrong-executable refusal, live revocation, extension disable and recovery, worker
identity, owned props and independent settings lifetime. Native x64/ARM64 evidence
for both exact Shell versions, visual review, all release architectures/packages,
same-source candidate/main gates and fresh immutable public bytes remain required
before advertising the integration.

Run [34243427852](https://github.com/RealEmmettS/goose/actions/runs/34243427852)
passes the complete production lifecycle and owned-prop scenarios on both Shell
versions and native architectures. Native settings, visible geese, painted note
text and uncropped images were inspected. Empty actors during creation/remapping
are omitted by the provider; every reported target still undergoes strict Rust
validation. Real held-input tests prove cancellation on configuration change,
target destruction and extension loss. Full distribution and publication remain
separate gates in the release readiness record.
