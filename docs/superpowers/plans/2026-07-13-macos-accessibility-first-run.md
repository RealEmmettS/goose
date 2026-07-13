# macOS Accessibility First-Run Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make the exact managed macOS app request Accessibility once per version, wait safely and visibly while denied, and resume FirstUX live after the grant.

**Architecture:** A dormant platform-neutral engine task owns the safe waiting behavior. A focused root runtime policy module owns installed-release eligibility and secure marker state. The macOS platform crate owns only native AppKit/ApplicationServices calls; `runtime/macos.rs` connects the pieces and polls permission once per second.

**Tech Stack:** Rust 1.95, honk-engine fixed-step tasks, objc2 AppKit and ApplicationServices bindings, serde_json receipts, owner-only Unix state, cargo test, and native Developer ID app smoke scripts.

## Global Constraints

- Automatic UI is allowed only for the exact receipted app at `~/Applications/Honk300.app`.
- Prompt state is outside TOML at `~/Library/Application Support/honk300/state/accessibility-prompt-v1/<version>`.
- The prompt opens at most once per installed version; later denied launches do not nag.
- The waiting goose permits direct honk plus status/reload/stop IPC; every prank is blocked.
- Permission changes are detected in the same process within a one-second polling interval.
- No settings window, menu bar, Dock UI, AppleScript API, config schema, or IPC schema.
- Windows and Linux behavior is unchanged; the engine primitive is dormant there.
- Terminal protection remains absolute before and after a live grant.
- The exact signed candidate must pass denied, non-nagging relaunch, granted, and revoked smoke.

---

### Task 1: Platform-neutral permission-wait task

**Files:**
- Modify: `crates/honk-engine/src/task.rs`
- Modify: `crates/honk-engine/src/world.rs`
- Modify: `crates/honk-engine/src/lib.rs`

**Interfaces:**
- Produces: `PermissionWaitTask::new(anchor: Vec2)`.
- Produces: `World::enter_permission_wait(anchor: Vec2)`, `update_permission_wait_anchor(anchor: Vec2)`, `leave_permission_wait()`, and `permission_waiting() -> bool`.
- `leave_permission_wait` installs a fresh `FirstUxTask`.

- [ ] **Step 1: Write failing task and world tests**

```rust
#[test]
fn permission_wait_walks_to_anchor_and_never_finishes() {
    let anchor = Vec2::new(700.0, 500.0);
    let mut world = World::new(bounds(), 31);
    world.enter_permission_wait(anchor);
    for _ in 0..(120 * 12) { world.tick(); }
    assert_eq!(world.current_task(), "permission_wait");
    assert!((world.goose.position - anchor).length() < 3.0);
    assert!(world.take_cursor_commands().is_empty());
    assert!(world.take_collect_window_commands().is_empty());
}

#[test]
fn permission_wait_allows_only_honk_and_grant_resumes_first_ux() {
    let mut world = World::new(bounds(), 32);
    world.enter_permission_wait(Vec2::new(700.0, 500.0));
    assert_eq!(world.poke(PokeAction::Honk), PokeOutcome::Applied);
    for action in [PokeAction::Wander, PokeAction::Mud, PokeAction::Nab,
                   PokeAction::Meme, PokeAction::Note] {
        assert_eq!(world.poke(action), PokeOutcome::Busy);
    }
    world.leave_permission_wait();
    assert_eq!(world.current_task(), "first_ux");
}
```

- [ ] **Step 2: Run the focused tests and observe failure**

Run: `cargo test -p honk-engine permission_wait -- --nocapture`
Expected: compile failure because the permission-wait APIs do not exist.

- [ ] **Step 3: Implement the minimal engine mode**

```rust
pub struct PermissionWaitTask { anchor: Vec2 }

impl Task for PermissionWaitTask {
    fn id(&self) -> &'static str { "permission_wait" }

    fn run(&mut self, goose: &mut GooseEntity, _ctx: &mut TaskCtx) -> bool {
        goose.current_speed = goose.parameters.walk_speed;
        goose.current_acceleration = goose.parameters.acceleration_normal;
        goose.target_pos = self.anchor;
        if arrived(goose, 2.0) {
            goose.current_speed = 0.0;
            goose.velocity = Vec2::ZERO;
        }
        false
    }

    fn set_permission_wait_anchor(&mut self, anchor: Vec2) {
        self.anchor = anchor;
    }
}
```

Add a default no-op `Task::set_permission_wait_anchor`. On entry, replace current/interrupted work and clear pending cursor/collect/nab/hyper state. In `World::poke`, return `Busy` for non-honk actions while waiting. On leave, install `FirstUxTask::new()`.

- [ ] **Step 4: Run the engine gate**

Run: `cargo test -p honk-engine permission_wait -- --nocapture`
Expected: focused tests pass.

Run: `cargo test -p honk-engine`
Expected: all engine and renderer golden tests pass.

### Task 2: Native macOS consent and Settings bridge

**Files:**
- Modify: `crates/honk-platform-macos/Cargo.toml`
- Modify: `crates/honk-platform-macos/src/lib.rs`

**Interfaces:**
- Produces: `MacBundleReleaseMetadata { bundle_id, version, tag, commit }`.
- Produces: `main_bundle_release_metadata() -> Option<MacBundleReleaseMetadata>`.
- Produces: `request_accessibility_prompt() -> AccessibilityState`.
- Produces: `open_accessibility_settings() -> io::Result<()>`.

- [ ] **Step 1: Add failing native contract tests**

```rust
#[test]
fn accessibility_settings_urls_prefer_direct_then_privacy_fallback() {
    assert_eq!(accessibility_settings_urls(), [
        "x-apple.systempreferences:com.apple.preference.security?Privacy_Accessibility",
        "x-apple.systempreferences:com.apple.settings.PrivacySecurity.extension",
    ]);
}
```

Add an injected bundle-dictionary mapping test proving a missing or non-string value cannot fabricate an installed identity.

- [ ] **Step 2: Run the focused platform test and observe failure**

Run: `cargo test -p honk-platform-macos accessibility_settings -- --nocapture`
Expected: compile failure because the bridge does not exist.

- [ ] **Step 3: Enable only required objc2 features and implement native calls**

Enable `NSWorkspace`, `NSBundle`, `NSURL`, `CFDictionary`, and `CFNumber`. Implement the native prompt:

```rust
pub fn request_accessibility_prompt() -> AccessibilityState {
    let options = CFDictionary::<CFType, CFType>::from_slices(
        &[unsafe { kAXTrustedCheckOptionPrompt }.as_ref()],
        &[CFBoolean::new(true).as_ref()],
    );
    if unsafe { AXIsProcessTrustedWithOptions(Some(&options)) } {
        AccessibilityState::Trusted
    } else {
        AccessibilityState::Denied
    }
}
```

Construct each `NSURL` from the constant strings and call `NSWorkspace::sharedWorkspace().openURL(&url)` on the main thread. Return success on the first accepted URL and an `io::Error` only if both fail. Read all four release metadata keys from `NSBundle` and reject missing/non-string fields.

- [ ] **Step 4: Run the platform gate**

Run: `cargo test -p honk-platform-macos`
Expected: all platform tests pass.

Run: `cargo clippy -p honk-platform-macos --all-targets -- -D warnings`
Expected: no warnings.

### Task 3: Managed-install policy and secure marker

**Files:**
- Create: `src/runtime/macos_accessibility.rs`
- Modify: `src/runtime/mod.rs`

**Interfaces:**
- Consumes: `MacBundleReleaseMetadata`.
- Produces: `AccessibilityOnboarding::detect(home, current_exe, metadata) -> io::Result<Self>`.
- Produces: `managed()`, `should_prompt(permission)`, `mark_prompted()`, and `safe_anchor(primary_bounds)`.

- [ ] **Step 1: Write failing filesystem-policy tests**

```rust
#[test]
fn exact_receipted_release_is_managed_and_prompts_once_per_version() {
    let fixture = managed_fixture("0.3.3", "v0.3.3", SHA);
    let mut policy = AccessibilityOnboarding::detect(
        fixture.home(), fixture.executable(), fixture.metadata()).unwrap();
    assert!(policy.managed());
    assert!(policy.should_prompt(AccessibilityState::Denied));
    policy.mark_prompted().unwrap();
    assert!(!policy.should_prompt(AccessibilityState::Denied));
    assert_eq!(fs::metadata(policy.marker_path()).unwrap().permissions().mode() & 0o777, 0o600);
}
```

Add mismatched app path, bundle id, version, tag, commit, receipt schema, symlinked state directory, marker-present, and granted tests.

- [ ] **Step 2: Run policy tests and observe failure**

Run: `cargo test --bin honk300 macos_accessibility -- --nocapture`
Expected: compile failure because the module and policy do not exist.

- [ ] **Step 3: Implement eligibility and marker security**

Parse `install-receipt.json` with `serde_json::Value`; require schema `honk300.install.v1`, exact install root, and matching version/tag/commit. Require exact non-symlinked executable and app directories. Build the marker from the validated version. Securely create state directories at mode `0700`, reject symlinks/foreign ownership, and create the marker with `OpenOptions::create_new(true).mode(0o600)`.

```rust
pub(crate) fn safe_anchor(bounds: Rect) -> Vec2 {
    Vec2::new(
        (bounds.max.x - 120.0).max(bounds.min.x + 40.0),
        (bounds.max.y - 110.0).max(bounds.min.y + 40.0),
    )
}
```

- [ ] **Step 4: Pass focused policy tests**

Run: `cargo test --bin honk300 macos_accessibility -- --nocapture`
Expected: all policy tests pass.

### Task 4: Runtime state-machine integration

**Files:**
- Modify: `src/runtime/macos.rs`
- Modify: `script/smoke_m16_macos_accessibility.sh`
- Modify: `script/tests/test_macos_smoke_contract.py`

**Interfaces:**
- Consumes: Tasks 1-3.
- Produces: startup prompt/wait behavior and one-second live permission transitions.

- [ ] **Step 1: Add failing transition and smoke-contract tests**

```rust
#[test]
fn denied_granted_and_revoked_transitions_are_deterministic() {
    assert_eq!(transition(Denied, Granted, true), ResumeFirstUx);
    assert_eq!(transition(Granted, Denied, true), EnterWait);
    assert_eq!(transition(Denied, Denied, true), Stable);
}
```

```python
def test_accessibility_smoke_exercises_non_nagging_live_transition():
    source = SCRIPT.read_text()
    assert "accessibility-prompt-v1" in source
    assert "BUSY" in source
    assert "same signed app" in source
```

- [ ] **Step 2: Run focused tests and observe failure**

Run: `cargo test --bin honk300 macos_accessibility -- --nocapture`
Run: `python3 -m unittest script.tests.test_macos_smoke_contract -v`
Expected: new transition and script-contract tests fail.

- [ ] **Step 3: Integrate startup and polling**

After AppKit initialization, detect policy. If managed and denied, create the marker before calling the native prompt/settings functions, then call `world.enter_permission_wait(safe_anchor(primary_bounds))`. At each one-second deadline, apply:

```rust
match transition(previous, current, policy.managed()) {
    PermissionTransition::ResumeFirstUx => {
        cursor_warp = BackendCapability::Supported;
        window_watch = BackendCapability::Supported;
        effective = effective_options(/* refreshed backend */);
        world.apply_options(effective.world);
        world.leave_permission_wait();
        window_watcher = ForeignWindowWatcher::new(&overlay).ok();
    }
    PermissionTransition::EnterWait => {
        window_watcher = None;
        cursor_warp = BackendCapability::Denied;
        window_watch = BackendCapability::Denied;
        world.set_cursor_warp_supported(false);
        world.set_foreign_window_watch_supported(false);
        world.enter_permission_wait(safe_anchor(overlay.primary_monitor_bounds()));
    }
    PermissionTransition::Stable => {}
}
```

Update the anchor on topology change. Existing `ControlCommand::Do` delegates to `World::poke`, which owns Busy responses. Keep status/reload/stop behavior unchanged.

- [ ] **Step 4: Pass focused integration tests**

Run: `cargo test --bin honk300 macos_accessibility -- --nocapture`
Run: `python3 -m unittest script.tests.test_macos_smoke_contract -v`
Expected: focused tests pass.

- [ ] **Step 5: Run exact signed-app native smoke**

Build and sign one universal app once, install it through the shared lifecycle, reset only Honk300 Accessibility where permitted, and test denied, second denied, granted, and revoked without rebuilding. Capture safe-edge wait and resumed FirstUX. Stop and purge all fixtures.

### Task 5: Decision record, release evidence, and publication

**Files:**
- Create: `docs/adr/0022-macos-accessibility-first-run-onboarding.md`
- Modify: `docs/adr/README.md`
- Modify: `README.md`
- Modify: `AGENTS.md`
- Modify: `CLAUDE.md`
- Modify: `honk300_plan.md`
- Modify: `CHANGELOG.md`
- Modify: `HUMAN_CHANGELOG.md`
- Modify: `CODEX_PROJECT.md`
- Modify: `docs/readiness/m16-m18-readiness.md`
- Modify: `docs/readiness/v0.3.3-readiness.md`
- Modify: `.tasks/TASKS.md`
- Modify: `.tasks/tasks/m16r.md`
- Modify: `.tasks/tasks/m20q.md`

**Interfaces:**
- Consumes: exact native evidence and final gates.
- Produces: candidate-proven v0.3.3 publication plus a complete handoff. Website rollout remains parked.

- [ ] **Step 1: Document only verified behavior**

Create ADR 0022 with context, decision, consequences, marker path, eligibility boundary, and wait rules. Add matched technical/plain-English changelog entries, update the README Mac first-run section, and record exact evidence in both readiness reports and board task files. Regenerate the complete CODEX_PROJECT tree.

- [ ] **Step 2: Run the complete local release gate**

```text
cargo fmt --all -- --check
cargo clippy --all-targets --workspace -- -D warnings
cargo test --workspace
cargo build --release
dist plan --tag=v0.3.3
python3 -m unittest discover -s script/tests -p 'test_*.py' -v
cargo audit
target/tools/bin/actionlint
git diff --check
```

Expected: every gate passes; cross-target checks and the one-display hardware waiver are recorded.

- [ ] **Step 3: Commit and push the complete branch**

Stage code, docs, board assets, task handoffs, and the final handoff document. Verify no secret or temporary evidence is staged. Commit with conventional release-scoped messages and push `codex/macos-v0.3.3`.

- [ ] **Step 4: Candidate, default branch, tag, and release**

Run candidate mode against the exact SHA. When green, fast-forward `main`, wait for ordinary CI, create the single immutable `v0.3.3` tag, and wait for atomic publication plus post-release smoke. Missing notarization credentials fail closed and must be recorded as the only publication blocker.

- [ ] **Step 5: Fresh-download verification and cleanup**

Download the published app ZIP and DMG; verify hashes, Developer ID team, hardened runtime, notarization, stapling, Gatekeeper, install, and v0.3.2-to-v0.3.3 update. Return this Mac to no installed app, aliases, LaunchAgent, socket, receipt, or test media. Push final board/readiness evidence and stop; do not deploy the website in this session.
