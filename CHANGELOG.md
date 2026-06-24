# Changelog

All notable changes to this project are documented here. Format based on
[Keep a Changelog](https://keepachangelog.com/); the project will adopt
[Semantic Versioning](https://semver.org/) once it produces releasable artifacts.

> **Project stage: planning.** There is no application code or release yet. The
> entries below track planning and scaffolding work. A plain-English companion
> lives in [HUMAN_CHANGELOG.md](./HUMAN_CHANGELOG.md) and must stay in lockstep
> (see `CLAUDE.md` → "Changelog rule").

## [Unreleased]

### Added
- `claude_plan.md` — comprehensive, adversarially-reviewed plan for **honk300**, a
  cross-platform (Windows/macOS/Linux) Rust reimplementation of Desktop Goose. Derived
  from analysis of `DESKTOP-GOOSE/` (the original v0.31 Windows + v0.22 macOS builds) and
  the `*300` sibling repos (TR300/ND300/WB300). Captures the reverse-engineered engine
  (rig geometry + physics constants, 120 Hz fixed tick, the biased `Deck` shuffle-bag, the
  Task/`InjectionPoints` model from `…/GooseModdingAPI/{SamEngine,Exports}.cs`), a
  Cargo-workspace architecture (`honk-engine` + capability-trait platform backends), build
  milestones M0–M17, the packaging pipeline (cargo-dist + hand-authored
  `windows-installers.yml`), a per-platform capability matrix, and a ranked risk table.
- `codex_plan.md` — a parallel planning document produced by Codex.
- `CHANGELOG.md` / `HUMAN_CHANGELOG.md` — dual changelogs, mirroring the `*300` family
  convention.
- `CLAUDE.md` — repository guidance for future Claude Code sessions.

### Decided
- App name **honk300** (binary `honk300`, optional `honk` alias); fresh permanent WiX/Inno
  GUIDs (never reuse the sibling repos').
- Clean-room **procedural** goose renderer — no sprite extraction. Original sound effects
  bundled 1:1 (personal use); meme images **regenerated originally** via an
  `Assets/Images/Memes/codex.md` brief (not copied); notepad messages **authored fresh**
  (not paraphrased).
- Linux: **X11-first** (runs under XWayland on Wayland sessions); native Wayland behind an
  opt-in `--wayland` flag with reduced mischief.
- Distribution: Windows-first installer matrix (Global/Corporate × MSI/EXE) + shell/
  PowerShell installers + macOS `.app`/`.dmg` + Linux `.desktop`. **No crates.io** —
  `crates-publish.yml` intentionally dropped from the family pipeline.

### Notes
- No `Cargo.toml` / `src/` exists yet — implementation is a later round (starts at plan
  milestone M0). `DESKTOP-GOOSE/` is **reference-only** and contains third-party
  copyrighted assets that are not for redistribution.
