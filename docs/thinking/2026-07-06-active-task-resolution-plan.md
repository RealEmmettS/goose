# Critical Thinking Session - Active Task Resolution Plan

**Date:** 2026-07-06 local (-05:00)
**Framework:** Information Triage -> Problem-Solving
**Mode:** Recommend
**Stacked skills:** critical-thinking, logical-reasoning

> **Implementation update (2026-07-07):** The operator chose the Windows/Linux-first M19 path.
> ADR 0013 now supersedes the macOS-in-first-pass parts of this planning canvas: `#m16r` stays
> open and deferred, `#a8d` remains the active M19 epic, Windows/Linux lifecycle and release
> scaffolding are implemented first, and macOS DMG/signing/notarization default to a later
> unsigned personal-use slice.

---

## Pre-Flight: Inputs Inspected

### Inputs brought to the session

| # | Input | Source / date | One-line gist | Keep / Park / Discard |
|---|---|---|---|---|
| 1 | `.tasks/TASKS.md` | local board, launched 2026-07-06 | Active tasks are `#m16r` and `#a8d`. | Keep |
| 2 | `.tasks/tasks/m16r.md` | local task detail | M16.1 is open only for Accessibility-granted macOS evidence. | Keep |
| 3 | `docs/readiness/m16-m18-readiness.md` | local readiness record, 2026-07-01 plus 2026-07-02 evidence | Hosted macOS bundle/status passed; Accessibility smoke skipped. | Keep |
| 4 | `.github/workflows/ci.yml` | local CI workflow | Optional self-hosted macOS Accessibility job is gated by `HONK300_RUN_A11Y_SMOKE == true`. | Keep |
| 5 | GitHub Actions run list and run views | live `gh` query, 2026-07-06 | Latest successful CI run still skipped macOS Accessibility smoke. | Keep |
| 6 | `honk300_plan.md` sections 13, 14, 15 | canonical plan | M19 requires cargo-dist, all OS/arch packaging, four Windows installer variants, macOS DMG, Linux desktop/autostart, in-binary lifecycle commands, no crates.io. | Keep |
| 7 | Sibling repo files under `C:\Users\hey\git\qube-machine-report` and `C:\Users\hey\git\qube-network-diagnostics` | local clones | TR300/ND300 provide the release and windows-installer workflow shapes; TR300 provides install/update source patterns. | Keep |
| 8 | Saved memory registry | `C:\Users\hey\.codex\memories\MEMORY.md` | Prior install work emphasized reliability, prerequisite handling, and release fix-forward loops. | Keep |

### Source pass findings

- The active task set is current board state, but it was produced by live board sync during `/tasks-start`; treat it as the source of truth unless the operator reorders it.
- The M16.1 evidence question is current-data-sensitive. A live `gh run list` plus job-level `gh run view` checked the latest CI state rather than relying only on stale docs.
- The M19 packaging plan is mostly local-contract work, but implementation details should be ported from sibling repos with adaptation for honk300 being a GUI app, not a shell-profile CLI.

### What's already decided (not revisiting)

- `honk300` is the primary binary; installed aliases are `honk` and `goose`.
- No crates.io distribution.
- Runtime control remains CLI/TUI over local IPC. No tray, global quit key, native settings UI, menu-bar settings UI, or Dock control surface.
- Terminal windows remain protected targets on every platform.
- Native Wayland remains degraded mode for mischief; document unsupported capabilities rather than pretending parity.
- Windows install means shortcuts/autostart/install markers, not shell-profile autorun.

---

## Working Sections

### Facts

| Fact | Confidence | Source / surfaced at |
|---|---|---|
| The live board currently has two Active tasks: `#m16r` and `#a8d`. | High | `.tasks/TASKS.md` after board launch |
| `#m16r` cannot close on hosted macOS CI alone because its acceptance requires granted Accessibility behavior. | High | `.tasks/tasks/m16r.md`; ADR 0012; readiness doc |
| The latest successful CI run (`28569889803`, created 2026-07-02T06:21:30Z) had successful hosted macOS bundle jobs and skipped `macOS Accessibility smoke`. | High | `gh run view 28569889803` |
| `script/smoke_m16_macos_accessibility.sh` checks status for `accessibility: supported`, `cursor: supported`, and exercises `honk`, `mud`, `reload`, `nab`, `meme`, and `note`. | High | script read-back |
| M19 is broader than cargo-dist config: it includes install/update/uninstall/setup behavior, Windows MSI/EXE matrix, macOS DMG, Linux desktop/autostart, aliases, and release workflows. | High | `honk300_plan.md` section 13 and milestone table |
| Current `install`, `uninstall`, and `update` CLI commands are placeholders. `setup` creates or refreshes config. | High | `src/main.rs`; `src/cli.rs` |
| The root Cargo package has no `[workspace.metadata.dist]` yet; the file explicitly says it is deferred to M19. | High | `Cargo.toml` |
| TR300 has the richest reusable install/update source; ND300 has a newer documented windows-installer workflow shape. | Medium | sibling repo inspection |

### Assumptions

| Assumption | Status | Surfaced at | Notes |
|---|---|---|---|
| The operator wants planning artifacts and board details updated, not code implementation yet. | tested | User wording | User asked to investigate, explore, analyze, and plan. |
| A self-hosted/pre-granted macOS machine is not available from this Windows session. | open | M16.1 planning | If it exists, the next action is to run the existing script there. |
| M19 should be implemented as one large epic with board-visible subtasks, but may need child tasks if the implementation becomes too large. | tested | task-system guidance | Keep visible subtasks now; split only if work needs separate owner/status. |
| cargo-dist 0.31.0 remains the intended release tool version. | tested | canonical plan and sibling repos | Verify against official cargo-dist docs during implementation if behavior diverges. |

### Constraints

- M16.1 evidence must come from real macOS, not Windows-host claims.
- The macOS Accessibility grant must attach to the `.app` bundle identity `dev.emmetts.honk300`.
- M19 must not introduce crates.io publish workflow.
- M19 must preserve all three installed names and arch-matched update behavior.
- GUI install must not autorun from shell profile snippets.
- Changelog updates must be lockstep if implementation changes user-facing behavior.

### Open questions

- Is there a self-hosted macOS runner with labels `[self-hosted, macOS, ARM64, honk300-a11y]` and a pre-granted Accessibility permission for Honk300?
- If no self-hosted runner exists, who will run the manual macOS Accessibility smoke and capture evidence?
- For M19, will the release be unsigned/personal-use only, or does the operator want Developer ID signing/notarization wired when secrets are available?
- For Windows ARM64 installers, should CI build on native ARM runners when available, or cross-build and record test limitations?

---

## Framework Steps

### Step 1: Forage

**Sub-questions asked:** What exists? Which docs are authoritative? Which inputs expire? Which state is current?

**Insights:**
- The M16.1 task is not mainly an implementation task anymore. It is an evidence acquisition task.
- The M19 task lacks a detail file, so planning should create a durable handoff before implementation starts.
- The active board state should be made self-explanatory with subtasks, because both active cards are currently broad.

**Mode:** Convergent

### Step 2: Frame

**Framing question:** What concrete path resolves the two active tasks without making unsupported readiness or packaging claims?

**Exit ramp:** Problem-Solving.

### Step 3: Problem Definitions

#### Problem statement A: M16.1 macOS readiness

What is happening: hosted macOS bundle/status smoke passes, but Accessibility-granted behavior has not been proven.

What should be happening: either a pre-granted self-hosted runner or a manual macOS run produces evidence for the existing Accessibility smoke and the readiness doc records it.

Gap: missing granted Accessibility evidence, not missing hosted bundle/status evidence.

#### Problem statement B: M19 packaging

What is happening: the app has CLI placeholders and an implementation plan, but no cargo-dist release config, installer sources, lifecycle implementation, or release workflows.

What should be happening: M19 should ship working install/update/uninstall/setup and release artifacts for every advertised OS/arch, with three names and no crates.io.

Gap: packaging implementation and proof matrix are absent.

**Mode:** Divergent then convergent

### Step 4: Five Whys

#### M16.1

1. Why is `#m16r` open? Because Accessibility-granted behavior is missing.
2. Why is that missing? Because hosted macOS runners cannot hold a durable grant for this app identity.
3. Why not use current CI? The optional job is gated and skipped without the pre-granted self-hosted runner/variable.
4. Why is manual evidence acceptable? The task explicitly allows pre-granted self-hosted or manual macOS evidence.
5. Why not close with denied/degraded behavior? The acceptance includes granted cursor/window/collect behavior; closing without it would overclaim readiness.

#### M19

1. Why is `#a8d` active but unresolved? Because M19 is the final packaging/lifecycle milestone and only placeholders exist.
2. Why are placeholders insufficient? Users need installers, autostart/shortcuts, updates, and uninstall/purge behavior, not just recognized words.
3. Why is this more than copying TR300? honk300 is a GUI pet with assets, IPC, `.app` identity, Wayland degradation, and no shell-profile autorun.
4. Why is CI/release part of the task? The acceptance is artifact evidence across OS/arch, not local code only.
5. Why plan before coding? The blast radius spans Cargo metadata, workflows, installer manifests, platform install code, docs, and release process.

### Step 5: Logical Reasoning Check

#### Argument: `#m16r` cannot close yet

P1. If a task's acceptance requires granted Accessibility evidence, then the task cannot honestly close without that evidence.

P2. `#m16r` acceptance requires granted Accessibility evidence for cursor/window/collect behavior.

P3. The latest checked successful CI run skipped `macOS Accessibility smoke`, and the readiness doc records no manual granted evidence.

Therefore, `#m16r` cannot honestly close yet.

Support type: deductive with a current-state factual premise.

Verdict: valid if P1-P3 hold; P2 and P3 are directly supported. P1 is the task-system completion rule plus project readiness contract. Confidence: High.

#### Argument: M19 should be treated as a packaging epic, not a single quick config change

P1. Work that affects multiple release artifacts, OS install state, lifecycle commands, and update behavior has cross-module and user-facing blast radius.

P2. M19 affects all of those surfaces.

Therefore, M19 requires staged implementation, tests, and release evidence rather than a narrow cargo-dist edit.

Support type: practical inductive argument. Confidence: High.

---

## Visual Models In Play

### Active Task Issue Tree

```text
Resolve Active Tasks
|- #m16r macOS host readiness
|  |- Already proven: hosted bundle/status on arm64 + Intel macOS
|  |- Missing: Accessibility-granted behavior evidence
|  |- Path A: self-hosted runner with HONK300_RUN_A11Y_SMOKE=true
|  |- Path B: manual macOS run, evidence captured in readiness doc
|  `- Closure: update readiness doc, task detail, board, changelogs only if user-facing docs change
`- #a8d M19 packaging/lifecycle
   |- Cargo/release metadata
   |- In-binary install/update/uninstall/setup
   |- Windows MSI/EXE matrix
   |- macOS app/DMG/signing-or-unsigned docs
   |- Linux desktop/autostart/installers
   |- CI release workflows and artifact evidence
   `- Docs/changelogs/readiness proof
```

### M19 Implementation Slice Matrix

| Slice | Primary files | Proof needed | Notes |
|---|---|---|---|
| Cargo/package metadata | `Cargo.toml`, generated `release.yml` | `dist plan`, target matrix check | No crates-publish workflow. |
| Lifecycle commands | `src/install/*`, `src/update.rs`, `src/main.rs`, `src/cli.rs` | unit tests plus platform smoke where possible | Port TR300 mechanics, adapt semantics. |
| Windows installers | `wix/`, `wix-corporate/`, `inno/`, `.github/workflows/windows-installers.yml` | MSI/EXE artifacts, sha256, silent update path | Matrix x64 + ARM64. |
| macOS distribution | `script/package_macos_app.sh`, new DMG script/workflow pieces | universal2 app, DMG, launch/status, quarantine docs | Signing/notarization depends on cert availability. |
| Linux distribution | `.desktop` templates, shell installer integration | x64/ARM gnu+musl artifacts, desktop/autostart install smoke | X11 default, Wayland reduced flag preserved. |
| Documentation and board closure | `README.md`, `AGENTS.md`, `CLAUDE.md`, changelogs, `.tasks/` | docs reflect actual artifacts and limits | Keep claims honest by platform. |

---

## Steel-Manned Dissent

- **The case against:** M16.1 could be closed because hosted macOS already proves the app bundle, status, IPC, and denied/degraded behavior on both arm64 and Intel.
- **What would have to be true for it to be correct:** The task acceptance would need to exclude granted Accessibility behavior, or a policy decision would need to declare granted behavior out of scope.
- **How it was handled:** Rejected. ADR 0012 and the task explicitly require granted Accessibility evidence.
- **Confidence in rejection:** High.

- **The case against:** M19 could be completed by adding cargo-dist metadata and release.yml, leaving lifecycle commands for later.
- **What would have to be true for it to be correct:** The milestone acceptance would need to define packaging as release artifact generation only.
- **How it was handled:** Rejected. The canonical plan defines M19 as install/update/uninstall/setup plus packaging all targets.
- **Confidence in rejection:** High.

---

## Resolution Plan

### `#m16r` plan

1. Reconfirm no newer CI run has passed the optional Accessibility job.
2. If a self-hosted macOS runner exists, enable `HONK300_RUN_A11Y_SMOKE=true` and run CI on `main`; otherwise run `script/smoke_m16_macos_accessibility.sh` manually on a macOS machine where `Honk300.app` already has Accessibility permission.
3. Capture evidence: run URL or terminal transcript, status output showing `accessibility: supported` and `cursor: supported`, successful `honk/mud/reload/nab/meme/note`, plus notes on terminal non-targeting and any hardware-limited multi-monitor checks.
4. Append the evidence to `docs/readiness/m16-m18-readiness.md`.
5. Update `.tasks/tasks/m16r.md` status/activity and move `#m16r` to Done only after all verification items are checked or explicitly waived with a reason.

### `#a8d` plan

1. Create the M19 handoff and board-visible subtasks before implementation.
2. Reconcile the M19 contract across `honk300_plan.md`, ADRs, current CLI placeholders, and sibling TR300/ND300 packaging files.
3. Implement lifecycle command architecture:
   - `install`: config/assets setup, shortcuts/desktop entries/LaunchAgent, optional autostart, install-source marker.
   - `uninstall`: remove installed state, aliases, shortcuts/autostart, and optionally purge config/state after backing up user memes/notes.
   - `update`: detect install source and arch, download matching artifact, verify sha256, run installer, handle reboot/deferred replacement honestly, post-install `--version` verify.
   - `setup`: keep current config behavior and extend only if packaging needs asset/shortcut setup.
4. Add package metadata and generated/adapted release workflow:
   - `[workspace.metadata.dist]` with cargo-dist 0.31.0, full target matrix, shell/powershell/msi installers, `install-updater=false`, `publish-prereleases=false`, and no crates-publish workflow.
   - Fresh WiX GUIDs.
   - Include assets, scripts, installer manifests, and docs needed by release artifacts.
5. Add Windows installer manifests and workflow:
   - Global MSI, Corporate MSI, Global EXE, Corporate EXE.
   - All three aliases.
   - Start Menu/desktop shortcut behavior and optional default-off autostart.
   - `.sha256` sidecars and torn-release guard.
   - Matrix x64 and ARM64, with explicit test-gap notes if ARM64 cannot be runtime-smoked.
6. Add macOS distribution:
   - Keep universal2 `.app` staging.
   - Add DMG creation and personal-use quarantine documentation.
   - Wire optional Developer ID signing/notarization only if credentials are available; otherwise document unsigned limits.
7. Add Linux distribution:
   - Install binaries/aliases/assets.
   - Install `.desktop` and optional autostart entry.
   - Preserve X11-first default and `--wayland` reduced mode.
8. Verification ladder:
   - local Rust gate,
   - `cargo dist plan` or generated workflow validation,
   - unit tests for lifecycle/update detection,
   - platform-specific smoke scripts,
   - release workflow dry-run where possible,
   - tag/release artifact proof before closure.
9. Update README/AGENTS/CLAUDE/changelogs and task details only after the implemented behavior matches the claims.

---

## Closing

### Sanity check

- Does the result make intuitive sense? Yes. One active task is blocked on external macOS evidence; the other is a large implementation epic.
- Does the conclusion follow from the evidence? Yes. The task acceptance and current CI job state directly support the M16.1 conclusion; the canonical M19 plan directly supports the packaging epic scope.
- What would I expect to be true if this conclusion is right? `#m16r` should remain open until a granted Accessibility run exists, and `#a8d` should need changes across Cargo metadata, workflows, installer manifests, lifecycle code, docs, and tests. That matches the repo.

### Decision / Conclusion

Exit state: Directed. Resolve `#m16r` by obtaining and recording Accessibility-granted macOS evidence. Resolve `#a8d` through a staged M19 packaging/lifecycle implementation, not a narrow cargo-dist-only change.

### Confidence band on the conclusion

High for the M16.1 blocker and M19 scope. Medium for the exact implementation order because release tooling and signing details may shift during implementation.

### Next steps

- Update task details and board subtasks with this plan.
- For `#m16r`, ask the operator whether a pre-granted macOS runner/manual host is available, then run the existing smoke there.
- For `#a8d`, start implementation with the lifecycle command architecture and package metadata, then add platform installers and release workflows.

### Open questions

- Is Developer ID signing/notarization available or intentionally out of scope for the first M19 pass?
- Is Windows ARM64 runtime smoke required, or is build artifact proof plus documented gap acceptable?
- Should M19 be split into child tasks before implementation, or kept as one active epic with subtasks until it becomes unwieldy?

### Spaced revisit

- **Revisit on:** first implementation checkpoint or first macOS Accessibility evidence run.
- **Why:** Those events decide whether the plan needs to split into child tasks or can proceed as written.
- **Trigger:** A CI runner becomes available, manual macOS evidence lands, or cargo-dist workflow generation exposes a tool-version mismatch.
