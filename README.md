# honk300

`honk300` is a clean-room, procedural desktop goose for Windows, macOS, and Linux. It walks
across real monitor layouts, honks, leaves muddy footprints, reacts to the pointer, and performs
bounded desktop pranks. Configuration and control stay local through a command-line interface
and terminal settings screen.

The executable is installed under three names—`honk300`, `honk`, and `goose`—so both
`honk300 start` and `goose plz` work.

**Current stable release:** [v1.3.6](https://github.com/RealEmmettS/goose/releases/tag/v1.3.6),
published from exact commit `1c56bf5679d04f160d09e4765a866458cad024aa`. It makes mud an
occasional accent: natural puddle trips move to a 3–5 minute cadence, their tracking window falls
to 10–30 seconds, and fresh/direct mud defaults to 10 seconds without replacing an existing saved
preference.

The v1.3.7 fix-forward target repairs a Windows self-update PowerShell argument-transport defect
that v1.3.6 installed acceptance exposed before installer launch. v1.3.5 and its protected receipt
remained unchanged; published v1.3.6 assets remain immutable.

The immutable v1.2.3 tag failed before publication on a hosted Windows tray observation. v1.2.4
fixed that qualification boundary; v1.2.5 fixed the subsequent public Mac verifier and published
successfully. Its public Mac/Linux/Debian lanes passed, while Windows exposed that the bootstrap
still assumed `Get-FileHash` existed. v1.2.6 replaces that optional dependency with direct .NET
SHA-256 and passed the complete candidate, same-SHA main, atomic publication, and eight-lane
fresh-public-byte matrix on Windows, macOS, and Linux. v1.3.0 adds terminal-independent Windows
start, a branded pinnable launcher icon, singleton-preserving app handoff, and nonblocking tray
recovery for immediate restarts; v1.3.1 keeps current protected-receipt discovery read-only,
v1.3.2 hardens bootstrap architecture detection in app-hosted terminals, v1.3.3 detaches the
hidden runtime from breakaway-permitted integrated-terminal jobs, v1.3.4 gives tray users a
one-click verified update and restart path, v1.3.5 makes its retained terminal result
unmistakable, and v1.3.6 calms the default mud cadence. Each published patch completed the required
cross-platform release gates; the v1.3.6 installed-updater finding is retained honestly and fixed
forward in v1.3.7.

## Install

### Windows

The recommended Windows install is the official versionless PowerShell bootstrap:

```powershell
irm https://github.com/RealEmmettS/goose/releases/latest/download/honk300-installer.ps1 | iex
```

It discovers the current stable release, downloads and verifies its exact Global-MSI and
lifecycle bytes, installs the managed PowerShell channel under Program Files, and verifies the
three public commands. It may request the normal Windows administrator approval; its app,
shortcut, and login-start paths use the windowless launcher and do not open a background console.

For native package deployment, use the Global MSI directly:

- [Global MSI for x64 (most Windows PCs)](https://github.com/RealEmmettS/goose/releases/latest/download/honk300-x86_64-pc-windows-msvc.msi)
- [Global MSI for ARM64 (Windows on ARM)](https://github.com/RealEmmettS/goose/releases/latest/download/honk300-aarch64-pc-windows-msvc.msi)

The MSI owns repair, upgrade, rollback, and uninstall. Corporate per-user packages, EXE
installers, portable archives, and checksums remain under each
[GitHub release](https://github.com/RealEmmettS/goose/releases/latest) for administrators and
compatibility.

Slot-aware releases keep each verified Windows payload in an immutable version/target directory
and move the stable `honk300`, `honk`, and `goose` command paths through a neutral `current`
selector. `honk300 update` reads the protected installation receipt and delegates to that exact
origin—Global/Corporate MSI, Global/Corporate EXE, or PowerShell—while the invoking old executable
remains untouched and alive. Unknown or conflicting ownership stops with an assisted reinstall
link instead of guessing Global MSI. A newly run installer, including an intentional downgrade,
is the user's latest intent and becomes authoritative only after staged verification.
If that change crosses from a machine-wide install to a per-user Corporate install, Windows may
require one administrator grant to retire the old machine owner. Cancelling that grant preserves
the new staged slot but reports `cleanup_pending`; Honk300 does not pretend the older machine PATH
has stopped winning. Running the new slot's `honk300 update` retries only the validated Honk300
registration. The same hidden elevated coordinator removes only that retired root's exact PATH
and login-start entries, verifies the active PATH, and clears the pending state without a second
elevation prompt.

Windows users on v1.0.2 or earlier must rerun the current installer once. Those older immutable
executables cannot self-repair their update-discovery bug; v1.0.3 and later perform subsequent
update checks correctly. Supported installer upgrades preserve settings and user content.

While Honk300 is running, an accessible **Honk300 controls** notification-area icon offers
**Configure Honk300…**, which launches the exact running copy's existing terminal settings
screen; **Update Honk300…**, which opens a new terminal and completes the verified update without
manual typing; and **Quit Honk300**, which sends the goose through its normal full walk-off before
exit. Opening the menu performs no update check. The update terminal stays open with an explicit
success, no-op, or failure/recovery result until the user closes it.
The fixed product icon returns after Explorer recreates the taskbar. If an unusual interactive
session cannot host notification icons, Honk300 reports the limitation and keeps CLI/TUI/IPC
control available.

On Windows, typing `honk300 start` (or the equivalent `honk`/`goose`, bare, or `plz` spelling)
uses the branded app launcher and returns the prompt only after the hidden runtime answers its
readiness check. The PowerShell or terminal window is then only a controller and may be closed;
the runtime and notification-area icon continue independently. A developer build must include
both `honk300.exe` and its exact sibling `honk300-app.exe`.

### macOS

The recommended Apple Silicon/Intel install is the official versionless terminal bootstrap:

```sh
curl --proto '=https' --tlsv1.2 -fsSL https://github.com/RealEmmettS/goose/releases/latest/download/honk300-installer.sh | sh
```

It installs the real signed universal `Honk300.app` without `sudo` at
`~/Applications/Honk300.app`, creates the three command aliases, and retains normal managed
Accessibility onboarding. For a graphical install, use the signed and notarized universal DMG:

- [Download the latest Honk300 DMG](https://github.com/RealEmmettS/goose/releases/latest/download/honk300-universal2.dmg)

The DMG remains the fresh graphical alternative. A receipted app at
`~/Applications/Honk300.app` updates synchronously with the exact-tag signed app ZIP while
preserving the same bundle and Accessibility identity. Mounted-DMG, foreign, read-only, and
unreceipted launches use the latest DMG as an assisted reinstall and do not claim update success.

Open the disk image and run **Install Honk300**. The universal Intel/Apple Silicon helper verifies
the adjacent app's Developer ID team and bundle identity, installs it without `sudo` into
`~/Applications/Honk300.app`, and opens the installed app. Receipts and user media live under
`~/Library/Application Support/honk300`; all three command aliases live under `~/.local/bin`.
Desktop pranks require Accessibility permission.

Launch the installed app by double-clicking `~/Applications/Honk300.app`, or keep that app in
the Dock as a launcher. While Honk300 is running, an accessible goose menu-bar icon offers
**Configure Honk300…**, which opens the existing terminal settings screen; **Update Honk300…**,
which opens the signed bundle's exact update command in Terminal; and **Quit Honk300**, which sends
the goose walking fully offscreen before the app exits. The update helper relaunches the verified
installed app and leaves its final terminal result visible until the user closes it. The item
exists only while the Mac app is running. Honk300 remains an agent app with no native settings
window or running Dock control surface.

When the exact managed app starts without Accessibility permission, it records a secure
per-update prompt marker before asking macOS for consent and opening Privacy & Security >
Accessibility. The goose walks to a calm lower-right screen-edge perch while permission is
denied. Status, reload, honk, and stop continue to work; other direct actions report busy. A
second denied launch of the same update waits without reopening Settings. If permission is
granted or revoked while Honk300 is running, the same process notices within about a second and
either starts the normal introduction or returns to the safe wait. Development binaries, bare
copies, source-tree bundles, and an app launched directly from a mounted DMG do not open
permission UI automatically.

The app and graphical helper use hardened Developer ID signatures. The app sealed inside the ZIP
and the DMG are Apple-notarized and stapled, and release checks publish SHA-256 values for every
artifact. Every general release uses GitHub's macOS runners to produce a fresh signed/notarized
DMG even when the release was initiated from Windows or Linux. The stable link above advances to
the newest complete release; each tag and its DMG bytes remain immutable.

### Linux

The recommended Linux install is the no-sudo official versionless bootstrap:

```sh
curl --proto '=https' --tlsv1.2 -fsSL https://github.com/RealEmmettS/goose/releases/latest/download/honk300-installer.sh | sh
```

The bootstrap is stamped for one exact release. It detects the architecture and libc, downloads
that exact-tag payload, verifies its embedded SHA-256, stages on the destination filesystem, and
rolls back the payload and owned integrations if installation fails. It does not use `sudo`.

On Debian or Ubuntu, the architecture-matched native package remains an alternative:

- [Debian/Ubuntu x64 package](https://github.com/RealEmmettS/goose/releases/latest/download/honk300-amd64.deb)
- [Debian/Ubuntu ARM64 package](https://github.com/RealEmmettS/goose/releases/latest/download/honk300-arm64.deb)

The package installs machine-wide under `/usr/lib/honk300`, provides all three commands in
`/usr/bin`, and can request `sudo` or a graphical administrator prompt for install, update, and
removal. Configuration and personal memes/notes remain in the current user's XDG directories.

Shell-managed Linux installs use immutable release directories and an atomic `current` symlink.
Debian installs remain dpkg-owned and update only with the matching architecture package; the two
origins never convert into one another during `update`. Because changing between a per-user shell
install and a machine-owned Debian package crosses an authority boundary, a fresh installer stops
before mutation and gives the exact uninstall-and-retry command instead of leaving two competing
owners.

- Linux: managed payload under `${XDG_DATA_HOME:-~/.local/share}/honk300/install`; receipt,
  desktop entry, and user media stay in the corresponding XDG user directories; aliases under
  `~/.local/bin`.

Each general release contains both Debian packages plus x64/ARM64 GNU and musl archives. Stable
`latest` links point to the newest complete release, while the updater resolves the manifest's
exact immutable tag and verifies the selected platform artifact's kind, target, size, and SHA-256
before changing an owned installation.

While Honk300 is running, desktops with a StatusNotifier watcher and host show the shared
**Honk300 controls** item with Configure, Update, and graceful-Quit actions. Configure and Update
prefer `xdg-terminal-exec` and then known terminal argument-vector interfaces; they pass the exact
executable and command as literal arguments and never interpolate a shell command. Update runs in
a terminal, restarts the receipt-owned app after activation, and retains its final result until the
window is closed. The icon is embedded in portable binaries, and Debian packages also own its
hicolor-theme copy. Sessions without a compatible host or session bus log the explicit non-fatal
reason while overlays, CLI/TUI/IPC, and supported mischief continue independently.

The commands above are product-owned managed installers. Raw `cargo install`, a source-tree
binary, or a portable copy is an advanced unmanaged path: it cannot retire an MSI/EXE/DMG/DEB
owner and is not treated as an installation channel by `honk300 update`. Honk300 is intentionally
not published to crates.io.

## Use

```text
honk300 start                 Start the goose (returns after readiness on Windows)
honk300 status                Show runtime and platform capabilities
honk300 config                Open the terminal settings editor
honk300 reload                Apply reloadable saved settings
honk300 do honk               Request an action
honk300 stop                  Stop the running goose
honk300 stop --force          Stop immediately without the walk-off
```

Friendly aliases include `goose plz`, `honk bad`, `goose no honk`, `goose quit`, and
`goose do mud`. Run `honk300 help` for the complete grammar.

Honk300 enters by walking in from a real exposed screen edge and leaves the same way on a normal
stop. During ordinary roaming, touching monitors remain one continuous desktop; occasionally the
goose can walk completely beyond an exposed edge and, only while fully hidden, return from the
opposite exposed edge. Its deliberate puddle and prank errands always return through their own
departure edge instead. If you personally close a note or meme it opened, there is a roughly 30%
chance it gets visibly annoyed and then tries its existing bounded mouse-steal prank. That second
step still obeys your settings, quiet/fullscreen manners, live permission, and platform support;
program cleanup never triggers it, and Linux currently has no collect windows to close.

Collected notes and pictures are fitted to the monitor receiving the goose. Notes remain readable
without exceeding 48% of either monitor dimension. Pictures preserve their complete source and
aspect ratio, downscale only when needed, and are never cropped or enlarged beyond their natural
size. Windows notes use a Honk300-owned native editable window rather than Notepad, so the prank
cannot restore, focus, type into, or expose a user's editor tabs.

First run materializes schema-current configuration. Existing malformed files and files from a
newer schema are never replaced automatically. To intentionally reset one, use
`honk300 setup --reset`; the previous bytes are backed up first.

Default config locations:

- Windows: `%LOCALAPPDATA%\honk300\config.toml`
- macOS: `~/Library/Application Support/honk300/config.toml`
- Linux: `${XDG_DATA_HOME:-~/.local/share}/honk300/config.toml`

Settings that affect backend selection, including native Wayland mode, require a restart and are
reported as such. Native Wayland remains an explicit reduced mode (`honk300 start --wayland`):
the overlay works, while cursor and foreign-window mischief report unsupported. X11/XWayland is
the full-mischief Linux default. ADR 0021 records why universal native parity is not a single
portable protocol feature and defines future opt-in portal/KDE/GNOME/wlroots adapters as separate
claims. Terminal windows are always protected from focus, typing, dragging, riding, and
collection. Codex and Visual Studio Code surfaces receive the same conservative protection across
platforms, including the ChatGPT-titled Codex desktop surface on Windows.

The General settings page also exposes default-off **Start on login**, stored as
`[lifecycle].autostart_on_login`. It reconciles only the startup integration owned by the active
installer: the matching Windows machine/user Run value, the managed Mac LaunchAgent, or the
current Linux user's XDG autostart entry. A fresh installer's selection is latest intent; later
config saves update that same integration. Unknown ownership and foreign startup entries are
refused instead of duplicated. On Windows, shortcuts and login startup target the internal
GUI-subsystem `honk300-app.exe`. Typed starts remain ordinary console commands for diagnostics,
but they now hand their start options to that same exact sibling launcher, wait for runtime
readiness, and return; its console-free child owns the overlay, IPC, and tray after the terminal
closes. Other commands and the configuration TUI continue to use the user's intentional terminal.

User notes and PNG memes can be added without modifying the program:

- Windows: `%LOCALAPPDATA%\honk300\media\Notes` and `...\Memes`
- macOS: `~/Library/Application Support/honk300/media/Notes` and `.../Memes`
- Linux: `${XDG_DATA_HOME:-~/.local/share}/honk300/media/Notes` and `.../Memes`

## Remove or update

On every platform, all three command names share the provenance-preserving update command.
Windows still uses Add/Remove Programs for graphical removal; update delegates to the exact
installer family named by the active receipt:

```text
honk300 update
honk300 update --json
honk300 uninstall
honk300 uninstall --purge
```

A normal uninstall preserves user media. `--purge` backs up user media before removing user
state. Debian package installs update and uninstall through `dpkg` and may request administrator
approval; per-user macOS/Linux installs do not use `sudo`. Login autostart is opt-in and off by
default; use **Start on login** in `honk300 config` or set `autostart_on_login = true` under
`[lifecycle]`.
Human update progress is stderr-only. `--json` writes exactly one final stdout object, and exit
zero means the selected release, receipt, selector, and aliases were activated and verified.

## Platform support

| Platform | Architectures | Desktop path |
| --- | --- | --- |
| Windows | x64, ARM64 | PMv2 layered overlays, one per monitor; Configure/Update/Quit notification-area item |
| macOS 11+ | Intel, Apple Silicon | Universal LSUIElement app; capture-safe AppKit RGBA overlays, one per display; Configure/Update/Quit menu-bar item |
| Linux X11/XWayland | x64/ARM64, GNU/musl | Full overlay and supported mischief; Configure/Update/Quit when a StatusNotifier host exists |
| Linux native Wayland | x64/ARM64, GNU/musl | Opt-in reduced overlay mode; independent StatusNotifier control when hosted |

Linux collect-window behavior is unsupported and reported honestly. Exact signed-app macOS
Accessibility evidence for first denial, non-nagging relaunch, live grant, and live revocation
remains tracked in the readiness docs; hosted bundle and permission-adapter tests are release
gates. Native release gates hold exact Windows/Linux binaries unchanged while checking
body/wing/beak/legs, asymmetric color channels, and transparency after each platform's real
presentation bridge rather than only in renderer goldens. Windows x64 and any host whose capture
API exposes ordinary windows must pass paired composed-desktop captures. GitHub's hosted ARM64
runner may use the narrowly identified ADR 0026 path: a real visible layered HWND plus the exact
cropped premultiplied-BGRA DIB recorded only after a successful native present. That runner-limited
evidence is not described as an ARM64 DWM screenshot; local and self-hosted ARM64 remain strict.

On macOS, tiny-skia's premultiplied RGBA bytes are copied into an alpha-last Device-RGB bitmap;
the transparent overlay window uses a stable standard-sRGB destination, and WindowServer performs
the final conversion for each physical display. This keeps screen capture, wide-gamut monitors,
and the on-screen palette on one explicit, regression-tested path.

## Build and verify

This is a Rust 1.95 workspace (edition 2021):

```text
cargo fmt --all -- --check
cargo clippy --all-targets --workspace -- -D warnings
cargo test --workspace
cargo build --release
dist plan --tag=v1.3.7
cargo audit --version 0.22.2
```

Architecture decisions are under [`docs/adr`](docs/adr/README.md). ADR 0018 defines distribution
and atomic publication; ADR 0019 defines the v0.3.x configuration, runtime, renderer, and platform
stabilization contracts; ADR 0020 defines Developer ID signing, notarization, and the per-user
graphical DMG; ADR 0021 defines native Wayland capability strata; ADR 0022 defines the managed
macOS Accessibility first-run, non-nagging wait, and live grant/revocation boundary; ADR 0023
defines stable latest links, exact-tag platform-isolated updates, every-release Mac packaging,
and native Debian lifecycle ownership; ADR 0024 defines the macOS-only menu-bar bridge to the
existing terminal TUI and graceful shutdown; ADR 0025 records the first stable v1 release and
post-release Alienware verification boundary; ADR 0026 defines the narrow GitHub-hosted Windows
ARM64 compositor-evidence exception without weakening normal paired-DWM proof; and ADR 0027
records the immutable-tag fix-forward to the public v1.0.1 identity. ADR 0028 defines the shared
goose control-surface icon and the Configure/TUI plus graceful-Quit parity contract. ADR 0029
records the Alienware-derived Windows update/lifecycle, Corporate retry, and integrated-terminal
hardening contract; ADR 0030 implements that contract with native Windows and Linux surfaces and
explicit unavailable-host boundaries. ADR 0031 supersedes ADR 0029's flat Windows update
transaction with authoritative provenance receipts, immutable slots, synchronous verification,
and matching Mac/Linux update ownership. ADR 0032 replaces external Windows Notepad props with
owned bounded windows and defines monitor-relative image fitting, graceful versus forced stop,
and the one provenance-owned login-start setting. ADR 0033 makes Windows app/login/background
launch windowless while preserving normal intentional CLI behavior, and moves every full-desktop
compositor surface to disposable CI; product startup performs no calibration. ADR 0034 makes the
official managed commands the recommended fresh install while retaining native-package ownership,
same-origin updates, authoritative fresh intent, and raw Cargo's unmanaged boundary.
ADR 0035 permits one narrow hosted x64 observation fallback only when an independent fixed-GUID
tray probe fails identically; ordinary Windows CI still proves actual registration and recovery.
ADR 0036 routes typed Windows starts through the branded GUI launcher and a hidden console-free
runtime, while retaining bounded readiness, exact-sibling ownership, and terminal TUI behavior.
ADR 0037 adds integrated-terminal Windows job breakaway with an access-denied-only fallback. ADR
0038 extends the shared control surface with the out-of-process, serialized, terminal-backed
update helper, receipt-owned relaunch/recovery, and explicit retained result screens. ADR 0039
supersedes only the original mud timing defaults with a calmer cadence while preserving the
puddle-hop story, saved user duration, footprint fade, and stable config shape. ADR 0040 makes
the Windows updater's generated archive-path check safe across PowerShell argument transport
without relaxing traversal rejection or lifecycle ownership.
[`docs/readiness/v1.2.2-readiness.md`](docs/readiness/v1.2.2-readiness.md) records the completed
semantic receipt-verifier fix-forward, candidate, same-SHA main, publication, and public-byte
qualification. The v1.2.0/v1.2.1 reports preserve their immutable release evidence.
[`docs/readiness/v1.2.3-readiness.md`](docs/readiness/v1.2.3-readiness.md) preserves the immutable
failed-before-publication evidence. [`docs/readiness/v1.2.4-readiness.md`](docs/readiness/v1.2.4-readiness.md)
records the public release and failed Mac verifier assertion.
[`docs/readiness/v1.2.6-readiness.md`](docs/readiness/v1.2.6-readiness.md) records the completed
Windows compatibility fix-forward, public-byte matrix, latest-alias audit, and production-site
qualification. [`docs/readiness/v1.3.0-readiness.md`](docs/readiness/v1.3.0-readiness.md) records
the detached Windows app-launch handoff, tray-recovery fix-forwards, native x64/ARM64 gates,
atomic publication, and fresh-public-byte closure. [`docs/readiness/v1.3.3-readiness.md`](docs/readiness/v1.3.3-readiness.md)
records the integrated-terminal job-breakaway release and installed proof.
[`docs/readiness/v1.3.4-readiness.md`](docs/readiness/v1.3.4-readiness.md) records the completed
cross-platform tray updater, immutable publication, public-byte matrix, production site, and
physical Windows acceptance. [`docs/readiness/v1.3.6-readiness.md`](docs/readiness/v1.3.6-readiness.md)
tracks the calmer-mud patch through exact candidate, publication, public bytes, and installed
acceptance. [`docs/readiness/v1.3.7-readiness.md`](docs/readiness/v1.3.7-readiness.md) tracks the
immutable Windows updater fix-forward and final installed acceptance.

## License and assets

The code is licensed under the [PolyForm Noncommercial License 1.0.0](LICENSE). Bundled media has
separate provenance and redistribution terms in [THIRD_PARTY_ASSETS.md](THIRD_PARTY_ASSETS.md).
This repository does not contain the original developers' source/reference tree, donation pages,
or old developer branding.
