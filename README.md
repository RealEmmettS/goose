# honk300

`honk300` is a clean-room, procedural desktop goose for Windows, macOS, and Linux. It walks
across real monitor layouts, honks, leaves muddy footprints, reacts to the pointer, and performs
bounded desktop pranks. Configuration and control stay local through a command-line interface
and terminal settings screen.

The executable is installed under three names—`honk300`, `honk`, and `goose`—so both
`honk300 start` and `goose plz` work.

**Current stable release:** [v1.0.3](https://github.com/RealEmmettS/goose/releases/tag/v1.0.3),
published from exact commit `5192fab9690ff8b6777366a5918c12bbe1ee247a`. Its complete Windows,
macOS, and Linux matrix, ordinary same-SHA CI, atomic publication, and post-release installer
smokes passed. The Mac app and DMG are Developer ID-signed, notarized, stapled, and independently
Gatekeeper-verified; the menu bar now uses the shared accessible goose icon while preserving the
existing Configure and animated-Quit behavior.

**Next release candidate:** v1.1.0 adds the same accessible goose control to the Windows
notification area and compatible Linux StatusNotifier hosts. It remains gated on one exact-SHA
all-platform candidate, ordinary main CI, atomic publication, and fresh public verification.

## Install

### Windows

The machine-wide Global MSI is the recommended Windows install. It adds Honk300 to Program
Files, the machine PATH, the all-users Start Menu, and Add/Remove Programs.

- [Global MSI for x64 (most Windows PCs)](https://github.com/RealEmmettS/goose/releases/latest/download/honk300-x86_64-pc-windows-msvc.msi)
- [Global MSI for ARM64 (Windows on ARM)](https://github.com/RealEmmettS/goose/releases/latest/download/honk300-aarch64-pc-windows-msvc.msi)

The MSI owns repair, upgrade, rollback, and uninstall. Corporate per-user packages, EXE
installers, portable archives, checksums, and the PowerShell bootstrap remain under each
[GitHub release](https://github.com/RealEmmettS/goose/releases/latest) for administrators and
compatibility.

Windows users on v1.0.2 or earlier must rerun the current installer once to reach v1.0.3. The
older immutable Windows executable cannot self-repair its update-discovery bug; v1.0.3 fixes
subsequent update checks. Supported installer upgrades preserve settings and user content.

While Honk300 is running, an accessible **Honk300 controls** notification-area icon offers
**Configure Honk300…**, which launches the exact running copy's existing terminal settings
screen, and **Quit Honk300**, which sends the goose through its normal full walk-off before exit.
The fixed product icon returns after Explorer recreates the taskbar. If an unusual interactive
session cannot host notification icons, Honk300 reports the limitation and keeps CLI/TUI/IPC
control available.

### macOS

The signed and notarized universal DMG is the recommended install for both Apple Silicon and
Intel Macs:

- [Download the latest Honk300 DMG](https://github.com/RealEmmettS/goose/releases/latest/download/honk300-universal2.dmg)

Open the disk image and run **Install Honk300**. The universal Intel/Apple Silicon helper verifies
the adjacent app's Developer ID team and bundle identity, installs it without `sudo` into
`~/Applications/Honk300.app`, and opens the installed app. Receipts and user media live under
`~/Library/Application Support/honk300`; all three command aliases live under `~/.local/bin`.
Desktop pranks require Accessibility permission.

Launch the installed app by double-clicking `~/Applications/Honk300.app`, or keep that app in
the Dock as a launcher. While Honk300 is running, an accessible goose menu-bar icon offers
**Configure Honk300…**, which opens the existing terminal settings screen, and **Quit Honk300**,
which sends the goose walking fully offscreen before the app exits. The item exists only while
the Mac app is running. Honk300 remains an agent app with no native settings window or running
Dock control surface. The shared icon and these exact behaviors are the contract for later
platform controls; v1.1.0 applies that contract to Windows and compatible Linux desktops.

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
the newest complete release; each tag and its DMG bytes remain immutable. The terminal bootstrap
remains a supported secondary install:

```sh
curl --proto '=https' --tlsv1.2 -fsSL https://github.com/RealEmmettS/goose/releases/latest/download/honk300-installer.sh | sh
```

### Linux

On Debian or Ubuntu, install the native package for the machine architecture:

- [Debian/Ubuntu x64 package](https://github.com/RealEmmettS/goose/releases/latest/download/honk300-amd64.deb)
- [Debian/Ubuntu ARM64 package](https://github.com/RealEmmettS/goose/releases/latest/download/honk300-arm64.deb)

The package installs machine-wide under `/usr/lib/honk300`, provides all three commands in
`/usr/bin`, and can request `sudo` or a graphical administrator prompt for install, update, and
removal. Configuration and personal memes/notes remain in the current user's XDG directories.

For other Linux distributions, or for a no-sudo per-user install, use the terminal bootstrap:

```sh
curl --proto '=https' --tlsv1.2 -fsSL https://github.com/RealEmmettS/goose/releases/latest/download/honk300-installer.sh | sh
```

The bootstrap is stamped for one exact release. It detects the OS, architecture, and Linux libc,
downloads that exact-tag payload, verifies its embedded SHA-256, stages on the destination
filesystem, and rolls back the payload and owned integrations if installation fails. It does not
use `sudo`.

- Linux: managed payload under `${XDG_DATA_HOME:-~/.local/share}/honk300/install`; receipt,
  desktop entry, and user media stay in the corresponding XDG user directories; aliases under
  `~/.local/bin`.

Each general release contains both Debian packages plus x64/ARM64 GNU and musl archives. Stable
`latest` links point to the newest complete release, while the updater resolves the manifest's
exact immutable tag and verifies the selected platform artifact's kind, target, size, and SHA-256
before changing an owned installation.

While Honk300 is running, desktops with a StatusNotifier watcher and host show the shared
**Honk300 controls** item with the same Configure and graceful-Quit actions. Configure prefers
`xdg-terminal-exec` and then known terminal argument-vector interfaces; it never interpolates a
shell command. The icon is embedded in portable binaries, and Debian packages also own its
hicolor-theme copy. Sessions without a compatible host or session bus log the explicit non-fatal
reason while overlays, CLI/TUI/IPC, and supported mischief continue independently.

## Use

```text
honk300 start                 Start the goose
honk300 status                Show runtime and platform capabilities
honk300 config                Open the terminal settings editor
honk300 reload                Apply reloadable saved settings
honk300 do honk               Request an action
honk300 stop                  Stop the running goose
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

User notes and PNG memes can be added without modifying the program:

- Windows: `%LOCALAPPDATA%\honk300\media\Notes` and `...\Memes`
- macOS: `~/Library/Application Support/honk300/media/Notes` and `.../Memes`
- Linux: `${XDG_DATA_HOME:-~/.local/share}/honk300/media/Notes` and `.../Memes`

## Remove or update

On Windows, use Add/Remove Programs or rerun the Global MSI. On macOS and Linux, all three command
names share the same update and removal paths:

```text
honk300 update
honk300 uninstall
honk300 uninstall --purge
```

A normal uninstall preserves user media. `--purge` backs up user media before removing user
state. Debian package installs update and uninstall through `dpkg` and may request administrator
approval; per-user macOS/Linux installs do not use `sudo`. Autostart is opt-in and off by default.

## Platform support

| Platform | Architectures | Desktop path |
| --- | --- | --- |
| Windows | x64, ARM64 | PMv2 layered overlays, one per monitor; Configure/Quit notification-area item |
| macOS 11+ | Intel, Apple Silicon | Universal LSUIElement app; capture-safe AppKit RGBA overlays, one per display; Configure/Quit menu-bar item |
| Linux X11/XWayland | x64/ARM64, GNU/musl | Full overlay and supported mischief; Configure/Quit when a StatusNotifier host exists |
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
dist plan --tag=v1.1.0
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
explicit unavailable-host boundaries.
[`docs/readiness/v1.1.0-readiness.md`](docs/readiness/v1.1.0-readiness.md) is the active release
gate; the v1.0.3 readiness report remains completed immutable-release history.

## License and assets

The code is licensed under the [PolyForm Noncommercial License 1.0.0](LICENSE). Bundled media has
separate provenance and redistribution terms in [THIRD_PARTY_ASSETS.md](THIRD_PARTY_ASSETS.md).
This repository does not contain the original developers' source/reference tree, donation pages,
or old developer branding.
