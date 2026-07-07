# 0013 - M19 Lifecycle Packaging And Deferred macOS Distribution

## Status

Accepted.

## Context

The canonical M19 plan requires install, update, uninstall, setup, release metadata, Windows
installer variants, Linux desktop/autostart install support, macOS `.app`/DMG distribution, and
arch-matched self-update without crates.io. At the same time, the M16.1 macOS Accessibility card
remains open because granted Accessibility behavior still requires a pre-granted self-hosted or
manual Mac.

Continuing M19 should not be blocked by that macOS evidence gap. Windows, Linux, and advertised
non-macOS architectures can move forward now, while macOS packaging remains an unsigned
personal-use slice to resume later.

## Decision

- Keep `#m16r` open and explicitly deferred until a pre-granted macOS host or self-hosted runner
  is available. Do not spend more current implementation time on macOS Accessibility evidence.
- Treat `#a8d` as the active M19 epic, but do not close it until release/artifact evidence is
  recorded.
- Implement first-class lifecycle commands in the root binary:
  - `honk300 install [--autostart]` copies the current executable into the user install location,
    installs `honk300`, `honk`, and `goose` aliases, places `Assets/` next to the installed
    binary, creates shortcuts or desktop entries, writes install-source markers, and enables
    login autostart only when requested.
  - `honk300 uninstall [--purge]` removes installed binaries, aliases, shortcuts, autostart, and
    install markers. `--purge` backs up user memes/notes before removing config/state.
  - `honk300 update` uses GitHub Releases, detects install source, picks the matching
    arch-specific installer, verifies its `.sha256` sidecar, runs the installer, and verifies
    post-install `--version`.
  - `honk300 setup` keeps its existing config-creation behavior.
- No update strategy may invoke `cargo install`; releases remain builds, installers, and scripts
  only.
- Add cargo-dist metadata for shell/PowerShell installers and core archive artifacts across the
  Windows and Linux M19 matrix:
  `x86_64-pc-windows-msvc`, `aarch64-pc-windows-msvc`,
  `x86_64-unknown-linux-gnu`, `aarch64-unknown-linux-gnu`,
  `x86_64-unknown-linux-musl`, and `aarch64-unknown-linux-musl`.
- Add a tag-triggered `Release` workflow and a chained `Windows Installers` workflow. The Windows
  workflow is matrixed over x64 and ARM64, builds the hand-authored Global/Corporate MSI and
  Global/Corporate Inno EXE artifacts, writes `.sha256` sidecars, and refuses to attach extras to
  a torn cargo-dist release.
- Leave macOS DMG/signing/notarization out of this continuation pass. When macOS packaging resumes,
  the default artifact is unsigned personal-use unless signing and notarization credentials are
  intentionally added.

## Consequences

- Windows installer file paths, install-source marker values, and `src/update.rs` strategy
  selection are a lockstep contract:
  `msi-global`, `msi-corporate`, `exe-global`, `exe-corporate`, `manual-local`, `powershell`,
  and `shell`.
- Linux install support is implemented through the binary lifecycle command and cargo-dist shell
  installer target. The binary installs aliases under `~/.local/bin`, assets under the user
  install root, `honk300.desktop`, and optional autostart.
- cargo-dist does not produce Windows MSI artifacts for M19. The dedicated Windows installer
  workflow owns MSI/EXE generation so both installer families include aliases, assets,
  install-source markers, shortcuts, optional autostart, and matching sidecars.
- macOS users still use `script/package_macos_app.sh` for local app staging until the deferred
  distribution slice resumes.
- `#a8d` remains open after code lands if cargo-dist planning, installer artifact inspection, or
  release run evidence is missing.

## Verification

- Unit tests cover install-source markers and path classification, `--purge` user-content backup,
  update strategy selection, no-cargo update paths, release target triples, SHA-256 sidecar
  parsing/verification, checksum mismatch refusal, and version comparison.
- The local Rust gate remains: `cargo fmt --all -- --check`,
  `cargo clippy --all-targets --workspace -- -D warnings`, `cargo test --workspace`, and
  `cargo build --release`.
- Release validation requires cargo-dist plan/build evidence plus Windows x64/ARM64 installer
  artifact inspection and Linux desktop/autostart smoke where host support exists.
