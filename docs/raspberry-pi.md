# Experimental Raspberry Pi desktop support

The Linux expansion targets Raspberry Pi 4 and 5 running **64-bit Raspberry Pi OS
Desktop**. This is an experimental target: native ARM64 automation exercises the
real application under labwc, but a physical Pi has not yet been available for
performance, graphics, audio, suspend or multi-monitor acceptance.

This guide describes the expansion under development. Its owned Linux notes and
pictures are not yet in the public stable release. The commands below always
install the currently published stable version; check its release notes before
expecting those features.

## Desktop and installation

Use the desktop edition of Raspberry Pi OS with a 64-bit userspace. Raspberry Pi's
[OS documentation](https://www.raspberrypi.com/documentation/computers/os.html)
identifies Trixie as the current release and distinguishes the graphical Desktop
edition from command-line-only Lite. Honk300 does not add a desktop to Lite and
does not provide a 32-bit ARM package. An existing compatible desktop does not
need to be reinstalled to try Honk300.

Open a terminal on the Pi's graphical desktop and check its userspace:

```sh
dpkg --print-architecture
getconf LONG_BIT
```

The expected results are `arm64` and `64`. Raspberry Pi 4/5 hardware alone does not
establish that the installed OS has a 64-bit userspace.

Use the official per-user bootstrap, which selects and verifies the existing
ARM64 GNU package identity:

```sh
curl --proto '=https' --tlsv1.2 -fsSL https://github.com/RealEmmettS/goose/releases/latest/download/honk300-installer.sh | sh
```

Honk300's shell installation does not use sudo. Graphical settings and owned
props require the distribution's GTK4 runtime. If it is missing, install the
`libgtk-4-1` package using Raspberry Pi's Add / Remove Software application or
`sudo apt install libgtk-4-1`. The architecture-matched
[Debian package](https://github.com/RealEmmettS/goose/releases/latest/download/honk300-arm64.deb)
is the supported machine-wide alternative and declares its runtime dependencies.
Keep the installer family reported by `honk300 update --check`; a raw source
binary does not become a managed installation by being copied over it.

## Start on labwc

Raspberry Pi's [desktop configuration documentation](https://www.raspberrypi.com/documentation/computers/configuration.html)
describes the Wayland/labwc desktop. Start its native reduced mode explicitly:

```sh
honk300 start --wayland
honk300 status
honk300 settings
```

Use the settings window's platform option to retain native Wayland selection for
future starts. Backend changes require a restart. Login startup stays off until
explicitly enabled. `honk300 config` provides the terminal settings interface.

On native Wayland the expansion reports `native Wayland`, the known desktop hint
when available, `collect: supported` after its companion becomes ready, and
unsupported note/picture positioning. Notes and pictures appear where the
compositor places them. The goose cannot animate their global positions, move
other applications or grab the global pointer through this mode. Desktop hints
do not enable privileges. X11/XWayland remains available where that desktop
supports Honk300's transparent overlay requirements.

Try `honk300 do note` or `honk300 do meme`. The goose retains at most eight open
owned props; closing one makes room for another without deleting existing notes.
If the companion is missing or fails, status reports that limitation while the
Rust controls remain usable. Use `honk300 stop` for its normal walk-off and owned
window cleanup.

## Evidence and remaining acceptance

[Native run 34186038608](https://github.com/RealEmmettS/goose/actions/runs/34186038608)
passed on x64 and ARM64 Ubuntu desktops with GTK4 and labwc 0.7.1. Each used a
private headless compositor with pixman, scale factors one and two, real owned
GTK windows and the production Rust engine. Checks cover native Unicode text,
complete-image pixels, bounded capacity, user/program closes, invalid messages,
child loss, standalone controls, session readback and graceful cleanup. ARM64
captures were inspected. This is Linux architecture/compositor evidence; it does
not certify Raspberry Pi OS's current labwc version or the Pi's graphics driver.

Official GNU companions also have an Ubuntu 22.04/GTK 4.6 baseline gate. Complete
package and fresh-public-byte qualification is recorded in the release readiness
record before publication.

Physical acceptance should record the Pi model, OS and labwc versions, display
layout/scales, GTK version, package identity and saved configuration. Exercise
entry/walk-off, idle/walking/mud CPU and retained memory, audio output, settings,
complete notes/pictures, display unplug/replug, suspend/resume and owned updates.
Keep failures and exact conditions in the readiness record. No Pi frame-rate,
power, temperature or memory-performance promise is made from the hosted run.
