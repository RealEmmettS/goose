TT;DR: M19 is the final packaging and lifecycle milestone. It must turn the current placeholder lifecycle commands into real install/update/uninstall/setup behavior and produce release artifacts for every advertised OS/arch, while keeping the no-crates.io and GUI-pet constraints honest.

## Why

The repo has implemented M0-M18, but users still need release artifact evidence before honk300 can
be treated as fully packaged. At the start of this task, the CLI recognized `install`,
`uninstall`, and `update` but printed M19 placeholder messages. The Windows/Linux-first M19 pass
now replaces those placeholders with real lifecycle behavior and release scaffolding; the remaining
work is proof: full local gate, cargo-dist planning, installer artifact inspection, Linux smoke
coverage, and release evidence.

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
- [ ] Packaging contract reconciled against canonical plan, ADRs, current code, and sibling repos.
- [ ] Lifecycle command implementation has unit coverage for install-source detection, update strategy selection, sha256 sidecars, purge backup, and no-`cargo install` update behavior.
- [x] Cargo package/release metadata validates with cargo-dist planning.
- [ ] Windows installer manifests build all required variants and write expected aliases/markers.
- [x] macOS `.app`/DMG/signing/notarization work is explicitly deferred in ADR 0013 and the board.
- [ ] Linux desktop/autostart install smoke passes or has recorded host-limit waivers.
- [x] Full local Rust gate passes.
- [ ] Release workflow produces the expected artifact set, or dry-run evidence records remaining external blockers.
- [ ] Documentation and both changelogs are updated in lockstep with implemented behavior.

## Status

Open - Windows/Linux-first implementation is in progress. Lifecycle commands, update strategy
selection, cargo-dist metadata, release workflows, Windows installer manifests, and Linux
desktop/autostart install support are in-tree. The card must stay open until local gate,
cargo-dist planning, installer artifact inspection, Linux smoke coverage, and release/artifact
evidence are recorded. macOS Accessibility, DMG, signing, and notarization are deferred until a
pre-granted Mac or self-hosted runner is available; later macOS artifacts default to unsigned
personal use unless signing credentials are intentionally added.

## Activity

- 2026-07-07 - implemented the M19 Windows/Linux-first pass: `src/install.rs`, `src/update.rs`,
  CLI lifecycle flags, cargo-dist metadata, tag-triggered release workflow, matrixed Windows
  installer workflow, WiX/Inno manifests, ADR 0013, docs, and changelogs. Focused `cargo test -p
  honk300` passed; full gate and artifact evidence remain pending before closure.
- 2026-07-07 - verification checkpoint passed: `cargo fmt --all -- --check`,
  `cargo clippy --all-targets --workspace -- -D warnings`, `cargo test --workspace`,
  `cargo build --release`, `dist plan`, and `cargo check --workspace --target` for Windows
  x64/ARM64 plus Linux x64/ARM GNU/musl. Local WiX/Inno artifact build could not run because
  WiX `candle/light/heat` and Inno `iscc` are not installed on this host; release workflow
  artifact evidence remains required before closing.
- 2026-07-06 22:23 - active-task investigation completed. Created the planning canvas, confirmed current lifecycle commands are placeholders, inspected M19 plan scope and sibling TR300/ND300 packaging patterns, and added this handoff plus board-visible subtasks.
