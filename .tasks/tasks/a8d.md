TT;DR: M19 is the final packaging and lifecycle milestone. It must turn the current placeholder lifecycle commands into real install/update/uninstall/setup behavior and produce release artifacts for every advertised OS/arch, while keeping the no-crates.io and GUI-pet constraints honest.

## Why

The repo has implemented M0-M19. At the start of this task, the CLI recognized `install`,
`uninstall`, and `update` but printed M19 placeholder messages. The Windows/Linux-first M19 pass
replaced those placeholders with real lifecycle behavior, release scaffolding, and recorded
release artifact evidence. macOS Accessibility, DMG, signing, and notarization remain explicitly
deferred outside this card.

This task is derived from the canonical plan, not a new scope decision. `honk300_plan.md` section 13 defines cargo-dist, the Windows installer matrix, macOS `.app`/`.dmg`, Linux `.desktop`, in-binary lifecycle commands, update semantics, uninstall/purge behavior, and CI release workflows. The sibling TR300/ND300 repos provide patterns to port, but honk300 must adapt them for a GUI app with assets, local IPC, Accessibility identity, Wayland degradation, and no shell-profile autorun.

## Plan

Planning canvas: `docs/thinking/2026-07-06-active-task-resolution-plan.md`.

Implementation sequence:

1. Reconcile the M19 contract across `honk300_plan.md`, ADRs, README/AGENTS/CLAUDE, current CLI placeholders, and sibling TR300/ND300 release/install files.
2. Decide whether a new ADR is needed. If implementation changes packaging shape, update the ADR record before code lands; if it only implements the existing plan, cite the existing plan/ADRs in docs.
3. Add lifecycle modules under `src/install/` and `src/update.rs`, porting reusable TR300 mechanics where they fit:
   - atomic writes and marker-block discipline,
   - install-source detection,
   - sha256 sidecar verification,
   - installer re-run and post-install `--version` verification,
   - honest handling of reboot/deferred replacement and user cancellation.
4. Adapt lifecycle semantics for honk300:
   - `install` creates config/assets, shortcuts or desktop entries, optional default-off login autostart, and install-source markers.
   - `uninstall` removes installed binaries/aliases/shortcuts/autostart/markers.
   - `--purge` removes config/state only after backing up user memes/notes and reporting the backup path.
   - `update` selects the arch-matched artifact for the current install source and never uses `cargo install`.
   - `setup` keeps config setup behavior and only expands if install-time setup requires it.
5. Add package/release metadata:
   - `[workspace.metadata.dist]` with cargo-dist 0.31.0, GitHub CI, shell/powershell installers, full target matrix, `install-updater=false`, `publish-prereleases=false`, and no crates-publish workflow.
   - fresh WiX GUIDs.
   - include required assets/scripts/installer files.
6. Add Windows installers:
   - Global MSI, Corporate MSI, Global EXE, Corporate EXE for x64 and ARM64.
   - install `honk300`, `honk`, and `goose` aliases.
   - Start Menu shortcut, optional desktop shortcut if chosen, and default-off HKCU Run autostart.
   - `InstallSource` markers and `.sha256` sidecars.
   - `windows-installers.yml` ported from sibling repos with torn-release guard and `--clobber`.
7. Defer macOS distribution:
   - keep existing universal2 `.app` staging through `script/package_macos_app.sh`,
   - do not block M19-first progress on DMG creation, signing, notarization, or Accessibility evidence,
   - when this slice resumes, default to unsigned personal-use artifacts unless signing credentials are intentionally added,
   - keep Accessibility permission tied to bundle id `dev.emmetts.honk300`.
8. Add Linux distribution:
   - install binary aliases and assets,
   - install `honk300.desktop`,
   - optional default-off autostart entry,
   - preserve X11-first default and `--wayland` reduced-mode behavior.
9. Verification ladder:
   - `cargo fmt --all -- --check`,
   - `cargo clippy --all-targets --workspace -- -D warnings`,
   - `cargo test --workspace`,
   - `cargo build --release`,
   - target checks for every advertised target,
   - cargo-dist plan/build validation,
   - lifecycle unit tests,
   - Windows installer artifact checks,
   - macOS app/DMG smoke,
   - Linux desktop/autostart smoke,
   - release workflow evidence before closure.

## Impact

Intended impact: honk300 becomes installable and updateable through first-class artifacts rather than a developer-run binary. The release process should produce a complete artifact set with the same reliability discipline as the sibling `*300` tools while preserving honk300-specific GUI behavior.

Possible unintended impact: packaging can accidentally overclaim platform parity, introduce broken autostart behavior, install stale assets, lose user-supplied memes/notes during purge, publish an incomplete release, or choose the wrong update artifact for an architecture/install source. The plan intentionally treats release evidence as part of the task, not a follow-up.

## Acceptance

- `install`, `uninstall`, `update`, and `setup` perform real lifecycle work and have tested error handling.
- `--purge` removes state/config only after backing up user memes/notes and reporting the backup location.
- The release config builds the advertised target matrix without crates.io publishing.
- Windows release produces Global/Corporate MSI and EXE variants for x64 and ARM64, plus sha256 sidecars.
- Windows installers install `honk300`, `honk`, and `goose`, shortcuts, default-off autostart, and install-source markers.
- macOS DMG/signing/notarization is explicitly deferred, with unsigned personal-use as the default
  when that slice resumes.
- Linux release installs aliases, assets, `.desktop`, and optional autostart across advertised targets.
- Update selects the arch-matched artifact for the detected install source, verifies sha256, handles reboot/deferred replacement honestly, and verifies `--version`.
- README, AGENTS, CLAUDE, CHANGELOG, HUMAN_CHANGELOG, and task details reflect the actual implemented behavior and remaining limits.
- Release/artifact evidence is recorded before the card moves to Done.

## Verification

- [x] Investigation plan written in `docs/thinking/2026-07-06-active-task-resolution-plan.md`.
- [x] Packaging contract reconciled against canonical plan, ADRs, current code, and sibling repos.
- [x] Lifecycle command implementation has unit coverage for install-source detection, update strategy selection, sha256 sidecars, purge backup, and no-`cargo install` update behavior.
- [x] Cargo package/release metadata validates with cargo-dist planning.
- [x] Windows installer manifests build all required variants and write expected aliases/markers.
- [x] macOS `.app`/DMG/signing/notarization work is explicitly deferred in ADR 0013 and the board.
- [x] Linux desktop/autostart install smoke passes or has recorded host-limit waivers.
  - Waived on this Windows host: local Linux desktop/autostart execution is not available here; lifecycle unit coverage validates file generation paths, and the `v0.0.0` release built Linux x64/ARM GNU/musl artifacts.
- [x] Full local Rust gate passes.
- [x] Release workflow produces the expected artifact set, or dry-run evidence records remaining external blockers.
- [x] Documentation and both changelogs are updated in lockstep with implemented behavior.

## Status

Done - closed 2026-07-07. Lifecycle commands, update strategy selection, cargo-dist metadata,
release workflows, Windows installer manifests, and Linux desktop/autostart install support are
in-tree. Local gate, cargo-dist planning, local x64 installer inspection, GitHub Release evidence,
and Windows x64/ARM64 installer workflow evidence are recorded. macOS Accessibility, DMG, signing,
and notarization remain deferred until a pre-granted Mac or self-hosted runner is available; later
macOS artifacts default to unsigned personal use unless signing credentials are intentionally
added.

## Activity

- 2026-07-07 - closed M19 after release evidence. Installed/located required packaging tools on
  the Windows host: Inno Setup 6.7.3 through winget, WiX 3.14.1 classic binaries under
  `%LOCALAPPDATA%\honk300-tools\wix314-home`, and actionlint 1.7.12 through winget. Local x64
  WiX/Inno artifact builds and MSI inspection passed.
- 2026-07-07 - release fix-forward completed. The first tag run exposed cargo-dist MSI generation
  inside the cargo-xwin ARM64 Windows container; M19 now lets cargo-dist own core archives and
  shell/PowerShell installers while the hand-authored Windows workflow owns Global/Corporate
  MSI/EXE artifacts. A second workflow failure exposed missing ARM64 Rust target installation for
  the pinned 1.95 toolchain; the Windows installer workflow now runs `rustup target add` for the
  matrix target before building.
- 2026-07-07 - GitHub evidence: Release run
  `https://github.com/RealEmmettS/goose/actions/runs/28842068256` passed for Windows x64/ARM64
  zips, Linux x64/ARM GNU/musl archives, source tarball, shell/PowerShell installers, and cargo-dist
  manifest/checksums. Windows installer run
  `https://github.com/RealEmmettS/goose/actions/runs/28842489497` passed for x64 and ARM64 Global
  MSI, Corporate MSI, Global EXE, Corporate EXE, and sha256 sidecars.
- 2026-07-07 - release asset evidence:
  `https://github.com/RealEmmettS/goose/releases/tag/v0.0.0` contains 34 assets, including
  Windows x64/ARM64 zip artifacts, Linux x64/ARM GNU/musl tarballs, shell/PowerShell installers,
  Windows x64/ARM64 Global/Corporate MSI and EXE installers, and sha256 sidecars. `#m16r` remains
  open and deferred for macOS Accessibility-granted host evidence.
- 2026-07-07 - prepared the real first release version `v0.1.0` from current `main` so downloads
  are tied to the final M19 workflow/docs state rather than the internal `v0.0.0` evidence tag.
- 2026-07-07 - verified the `v0.1.0` release prep locally with `cargo fmt --all -- --check`,
  `git diff --check`, clippy with warnings denied, workspace tests, release build,
  `dist plan --tag v0.1.0`, and a `honk300 0.1.0` binary version smoke.
- 2026-07-07 - implemented the M19 Windows/Linux-first pass: `src/install.rs`, `src/update.rs`,
  CLI lifecycle flags, cargo-dist metadata, tag-triggered release workflow, matrixed Windows
  installer workflow, WiX/Inno manifests, ADR 0013, docs, and changelogs. Focused `cargo test -p
  honk300` passed; full gate and artifact evidence were still pending at that checkpoint.
- 2026-07-07 - verification checkpoint passed: `cargo fmt --all -- --check`,
  `cargo clippy --all-targets --workspace -- -D warnings`, `cargo test --workspace`,
  `cargo build --release`, `dist plan`, and `cargo check --workspace --target` for Windows
  x64/ARM64 plus Linux x64/ARM GNU/musl. Before the packaging tools were installed locally,
  WiX/Inno artifact builds were still pending and release workflow evidence remained required.
- 2026-07-06 22:23 - active-task investigation completed. Created the planning canvas, confirmed current lifecycle commands are placeholders, inspected M19 plan scope and sibling TR300/ND300 packaging patterns, and added this handoff plus board-visible subtasks.
