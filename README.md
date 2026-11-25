# Goose

<img align="right" src="settings/assets/icon.png" alt="Goose app icon" width="96">

Goose is a free desktop toy for Windows, macOS, and Linux. It waddles across your screens, honks, and leaves muddy footprints. Depending on your settings and desktop, it might drop off a note or make off with your cursor. Rub the goose for about two and a half seconds to earn a few hearts.

**[Download Goose](https://thegoose.app/#download)** · [GitHub releases and checksums](https://github.com/RealEmmettS/goose/releases/latest)

## Install

The download page walks you through choosing an installer. You can also grab one directly.

| Your computer | Download | What to do |
| --- | --- | --- |
| Windows (64-bit) | [Intel / AMD (.msi)](https://github.com/RealEmmettS/goose/releases/latest/download/honk300-x86_64-pc-windows-msvc.msi) · [ARM64 (.msi)](https://github.com/RealEmmettS/goose/releases/latest/download/honk300-aarch64-pc-windows-msvc.msi) | Open the installer, then find **Goose** in Start. |
| macOS 11 or later | [Apple Silicon and Intel (.dmg)](https://github.com/RealEmmettS/goose/releases/latest/download/honk300-universal2.dmg) | Open the disk image and double-click **Install Honk300**. It installs Goose in your home Applications folder and opens it. |
| Debian / Ubuntu | [Intel / AMD (.deb)](https://github.com/RealEmmettS/goose/releases/latest/download/honk300-amd64.deb) · [ARM64 (.deb)](https://github.com/RealEmmettS/goose/releases/latest/download/honk300-arm64.deb) | Open the download with your software installer, then find **Goose** in your applications menu. |
| Other Linux | [Terminal command](#install-from-a-terminal) | Run the command, then find **Goose** in your applications menu. |

### Install from a terminal

The official install commands pick the right download for your computer. Use one if you’re unsure which Windows or Linux package to choose.

**Windows (PowerShell)**

```powershell
irm https://github.com/RealEmmettS/goose/releases/latest/download/honk300-installer.ps1 | iex
```

**macOS and Linux (Terminal)**

```sh
curl --proto '=https' --tlsv1.2 -fsSL https://github.com/RealEmmettS/goose/releases/latest/download/honk300-installer.sh | sh
```

Fresh macOS and Linux installs from Terminal go into your account and need no `sudo`. Refreshing an existing Debian or Ubuntu package uses the system package manager.

## Let the goose out

Open **Goose** and choose **Start goose**. Try **Calm movements** or **Seasonal effects** under **Appearance**. **Start at login** is off by default. Choose **Stop goose** to send it walking offscreen. Goose leaves terminal windows, VS Code, and Codex alone.

On macOS, desktop pranks require Accessibility permission. The goose waits at the edge of the screen until permission is granted. On Linux, X11 and XWayland have the broadest support. Native Wayland is opt-in and offers fewer pranks. See the [Linux desktop guide](docs/wayland-integrations.md) or the [experimental Raspberry Pi guide](docs/raspberry-pi.md).

| Command | What it does |
| --- | --- |
| `goose` | Open the graphical controls |
| `goose start` | Start the goose directly |
| `goose do honk` | Ask the goose to honk |
| `goose stop` | Send the goose walking offscreen |
| `goose --help` | Show the full command list |

## Update or uninstall

In Goose, open **Platform & status** and choose **Check for updates**, then **Update now** when available. You can also run `goose update`. Goose updates through your existing install method. Prefer the download page? You can [install Goose again](https://thegoose.app/#download) over your current copy and keep your settings. This also repairs older Mac installations that report an unknown installer.

To remove Goose, use Windows Settings, your package manager if you installed a `.deb`, or `goose uninstall` for a managed macOS or Linux installation.

## Build from source

The Rust workspace uses Rust 1.95. Run `cargo build --locked --release` to build it and `cargo test --locked --workspace` to run its tests. See the [settings development guide](settings/README.md) for the separate graphical app.

[Changelog](HUMAN_CHANGELOG.md) · [Design decisions](docs/adr/README.md) · [Release testing](docs/readiness/)

## License

The code is licensed under the [PolyForm Noncommercial License 1.0.0](LICENSE). Bundled media has [separate terms](THIRD_PARTY_ASSETS.md).
