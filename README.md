# honk300

`honk300` is a clean-room, procedural desktop goose for Windows, macOS, and Linux. It walks
across real monitor layouts, honks, leaves muddy footprints, reacts to the pointer, and performs
bounded desktop pranks. Configuration and control stay local through a command-line interface
and terminal settings screen.

The executable is installed under three names—`honk300`, `honk`, and `goose`—so both
`honk300 start` and `goose plz` work.

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

### macOS

The signed and notarized universal DMG is the recommended install for both Apple Silicon and
Intel Macs:

- [Download Honk300 for macOS](https://github.com/RealEmmettS/goose/releases/download/v0.3.3/honk300-universal2.dmg)

Open the disk image and run **Install Honk300**. The universal Intel/Apple Silicon helper verifies
the adjacent app's Developer ID team and bundle identity, installs it without `sudo` into
`~/Applications/Honk300.app`, and opens the installed app. Receipts and user media live under
`~/Library/Application Support/honk300`; all three command aliases live under `~/.local/bin`.
Desktop pranks require Accessibility permission.

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
artifact. The exact-tag terminal bootstrap remains a supported secondary install:

```sh
curl --proto '=https' --tlsv1.2 -fsSL https://github.com/RealEmmettS/goose/releases/latest/download/honk300-installer.sh | sh
```

### Linux

Use the exact-tag terminal bootstrap:

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
claims. Terminal windows are always protected from focus, typing,
dragging, riding, and collection.

User notes and PNG memes can be added without modifying the program:

- Windows: `%LOCALAPPDATA%\honk300\media\Notes` and `...\Memes`
- macOS: `~/Library/Application Support/honk300/media/Notes` and `.../Memes`
- Linux: `${XDG_DATA_HOME:-~/.local/share}/honk300/media/Notes` and `.../Memes`

## Remove or update

On Windows, use Add/Remove Programs or rerun the Global MSI. On macOS/Linux:

```text
honk300 update
honk300 uninstall
honk300 uninstall --purge
```

A normal uninstall preserves user media. `--purge` backs up user media before removing user
state. Autostart is opt-in and off by default.

## Platform support

| Platform | Architectures | Desktop path |
| --- | --- | --- |
| Windows | x64, ARM64 | PMv2 layered overlays, one per monitor |
| macOS 11+ | Intel, Apple Silicon | Universal LSUIElement app bundle |
| Linux X11/XWayland | x64/ARM64, GNU/musl | Full overlay and supported mischief |
| Linux native Wayland | x64/ARM64, GNU/musl | Opt-in reduced overlay mode |

Linux collect-window behavior is unsupported and reported honestly. Exact signed-app macOS
Accessibility evidence for first denial, non-nagging relaunch, live grant, and live revocation
remains tracked in the readiness docs; hosted bundle and permission-adapter tests are release
gates.

## Build and verify

This is a Rust 1.95 workspace (edition 2021):

```text
cargo fmt --all -- --check
cargo clippy --all-targets --workspace -- -D warnings
cargo test --workspace
cargo build --release
dist plan --tag=v0.3.3
cargo audit --version 0.22.2
```

Architecture decisions are under [`docs/adr`](docs/adr/README.md). ADR 0018 defines distribution
and atomic publication; ADR 0019 defines the v0.3.x configuration, runtime, renderer, and platform
stabilization contracts; ADR 0020 defines Developer ID signing, notarization, and the per-user
graphical DMG; ADR 0021 defines native Wayland capability strata; ADR 0022 defines the managed
macOS Accessibility first-run, non-nagging wait, and live grant/revocation boundary.
[`docs/readiness/v0.3.3-readiness.md`](docs/readiness/v0.3.3-readiness.md) is the release checklist.

## License and assets

The code is licensed under the [PolyForm Noncommercial License 1.0.0](LICENSE). Bundled media has
separate provenance and redistribution terms in [THIRD_PARTY_ASSETS.md](THIRD_PARTY_ASSETS.md).
This repository does not contain the original developers' source/reference tree, donation pages,
or old developer branding.
