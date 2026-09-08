# Linux owned windows

The internal `honk300-settings --owned-props` mode supplies native GTK notes and
pictures to the Rust runtime. It is a separate child process of the same immutable
companion executable. It neither loads configuration nor owns installation, startup,
updates or the tray. The regular settings invocation still uses Native SDK.

Rust selects approved assets, fits them to the receiving monitor, verifies the
companion against the existing receipt boundary, and retains the exact child. It
forces the child's GDK backend to the actual overlay mode. A missing companion
leaves the runtime and shared settings service available with props unsupported.
Other startup/protocol/process errors latch prop capability to failed until restart.

The private input consists of newline-delimited JSON envelopes with `v: 1`, an `op`,
and only the fields defined by the decoder in `../src/owned_props.zig`. Operations
are `note`, `image`, `move`, `passthrough`, `focus`, `text`, `close`, and `shutdown`.
IDs are monotonically assigned by Rust and are never XIDs, HWNDs or process IDs.
The C adapter obtains an XID only from the GTK object in its own registry.

Bounds are fixed: eight pending/live windows, 64 queued commands and 4 MiB total
outbound storage, a 4 MiB input frame, 16 KiB of UTF-8 note text, 256 title bytes,
and at most 900 by 700 premultiplied RGBA pixels encoded as base64. Embedded NUL,
unknown properties/operations, invalid dimensions and malformed JSON fail the
connection. Unsent adjacent moves for the same window may coalesce; partially
written commands and other operations retain order. Native dispatch and Rust I/O
have per-iteration limits. A full registry returns busy without discarding notes.

Output consists of a readiness message with its positioning capability, window
snapshots, and busy responses. Every output frame is smaller than 512 bytes.
Snapshots retain opaque identity, type, dimensions and an explicit user/program
close origin. Rust rejects unknown identities, unexpected capability changes,
out-of-budget dimensions and invalid close state before updating the engine.

X11 uses physical coordinates and only the owned surface's move/input-region APIs.
Wayland reports logical dimensions and zero global coordinates. Its ordinary
compositor placement completes the engine's placement-only path; it never claims
window dragging, pointer injection or foreign-window control. Whole images are
uniformly downscaled inside the content area and never enlarged. The header and
Close button share the outer monitor-relative ceiling. GTK handles editable notes,
native appearance and accessibility; Makira faces are registered from private
anonymous font files, without changing the user's font installation.

The Rust controller has a bounded readiness deadline and write-stall deadline,
retains bounded diagnostics, and owns child termination/reaping. Native EOF,
malformed input, shutdown and process loss destroy only its own windows. Closing
the visible Close control emits the user origin; program cleanup does not.

The dedicated workflow exercises native x64/ARM64 GTK, complete image pixels,
capacity/close/EOF and invalid input, the real engine delivery path, child loss,
standalone operation, display scaling and labwc. These are disposable hosted
desktop checks. Physical Raspberry Pi performance and device acceptance are separate.
