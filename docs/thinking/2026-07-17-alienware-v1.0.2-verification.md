# Critical Thinking Session — Alienware v1.0.2 Windows Verification

**Date:** 2026-07-17
**Framework:** Scientific Inquiry
**Mode:** Self-check
**Stacked skills:** shaughv-tasks:tasks-start

---

## Pre-Flight: Inputs Inspected

### Inputs brought to the session

- .tasks/tasks/v1a.md: current internal task and verification contract.
- docs/agents/handoff/2026-07-17-001-alienware-post-v1.0.2-verification.md: exact operator handoff.
- docs/readiness/v1.0.2-readiness.md: release evidence and explicit next-machine boundary.
- Published GitHub release and exact downloaded artifacts: external system of record, not yet inspected in this session.
- Current Alienware host, installed state, displays, terminals, audio, and compositor: empirical environment, not yet measured.

### Source pass findings

- The internal records are current as of 2026-07-17 and distinguish completed hosted/Mac evidence from open Alienware evidence.
- Hosted Windows x64/ARM64 success is strong evidence for release construction, but it cannot prove this physical machine's installer, driver, display, terminal, and audio paths.
- The GitHub release, immutable-tag manifest, stable-latest manifest, downloaded bytes, Windows trust APIs, and real installed state are the sources capable of settling the open claims.

### What's already decided (not revisiting)

- v1.0.0, v1.0.1, and v1.0.2 tags/assets are immutable.
- Any defect fixes forward as v1.0.3 or newer after a failing regression.
- Accepted macOS security, presentation, lifecycle, and control contracts remain intact.
- Tests use fresh published bytes; a local rebuild cannot substitute for product evidence.

---

## Working Sections

### Facts

| Fact | Confidence | Source / surfaced at |
|---|---|---|
| The repository was on clean main at 6c469f8ab588761233b4da1a0d94a1b181ab1349 before board activation. | High | Git baseline before Step 1 |
| v1.0.2 release records identify release commit 964305869e9ec28768c789465db1b6317dfa3f6f. | High | Handoff and readiness inputs |
| Hosted replacement candidate, same-SHA CI, release, and post-release jobs passed, including Windows x64/ARM64. | High | Handoff and readiness inputs |
| Alienware published-byte lifecycle/compositor evidence was explicitly open when #v1a began. | High | .tasks/tasks/v1a.md |

### Assumptions

| Assumption | Status | Surfaced at | Notes |
|---|---|---|---|
| The current host is the intended Alienware and is suitable for native Windows x64 evidence. | open | Pre-flight | Confirm model, architecture, OS, elevation, displays, audio, and session type. |
| No existing Honk300 install or runtime will contaminate the baseline. | open | Pre-flight | Inventory processes, registry, paths, receipts, services/startup, and user data before mutation. |
| Stable-latest and immutable-tag manifests still resolve to identical v1.0.2 payload identities. | open | Pre-flight | Verify live, do not infer from the Mac record. |
| A passing scripted overlay smoke represents the intended DWM surface rather than a partial entrance frame or wrong process. | open | Pre-flight | Retain raw/paired evidence and inspect it visually. |
| Any stale bootstrap receipt fields after direct MSI update are informational only. | open | Pre-flight | Test authoritative source markers, update selection, and uninstall ownership before deciding. |

### Constraints

- Published releases are immutable; no force tags or asset replacement.
- Machine mutations must be reversible and use the product's real supported paths.
- Installer identity and hashes are checked before execution.
- Terminal windows must never be mischief targets.
- Windows observations cannot be generalized into Linux or macOS claims.
- Task board and this canvas remain current as evidence accumulates.

### Open questions

- Does the exact published v1.0.2 Windows x64 release satisfy identity, lifecycle, compositor, IPC, TUI, audio, terminal-protection, and multi-monitor contracts on this Alienware?
- Does any observed failure reproduce from clean published bytes and distinguish product behavior from local configuration or test-fixture behavior?
- Are bootstrap receipt fields operationally consumed after direct MSI update, or safely informational?

### Tensions

- Hosted gates are comprehensive vs. physical hardware can still expose driver, timing, DPI, terminal, and installer integration defects.
- Thorough lifecycle coverage requires machine-wide mutations vs. preserving the operator's pre-existing state.
- Scripted semantic acceptance is repeatable vs. visual/manual behavior still needs honest observation.

### Deferred items

- macOS and Linux supplemental evidence: unavailable on this host and remains separately waived/open.
- Windows/Linux tray implementation: separate future ADR/task; not part of this verification pass.

---

## Framework Steps

### Step 1: Observe / Question

**Sub-questions asked:** What exactly is open? Where was it measured? What was expected instead? What falsifiable question settles it?

**Responses:**

- Observation: hosted Windows jobs passed, but the board still has no Alienware evidence from exact published bytes.
- Measurement boundary: the release workflows measured GitHub runners; this session measures the physical Windows x64 operator machine.
- Expected result: exact public artifacts agree with the immutable release record, install/update/uninstall cleanly, and the real DWM/runtime behavior satisfies the documented contracts.
- Falsifiable question: **Do exact published v1.0.2 Windows x64 artifacts pass identity, supported lifecycle, and real-hardware runtime/compositor/terminal checks on this Alienware without a contract violation?**

**Insights:** One failure from a contaminated baseline is not enough to indict the product; one passing automation run is not enough to claim the manual hardware matrix.

**Mode:** Convergent

### Step 2: Research

**Sub-questions asked:** What source would settle identity? What does the system of record say? Could the evidence be stale?

**Responses:**

- Internal source: current task, readiness, and handoff define exact expected SHA, workflows, and open checks.
- External source: live GitHub tag/release/manifests/assets settles current public identity.
- Host source: Windows registry, filesystem, process list, Authenticode, display/audio inventory, and retained smoke artifacts settle local behavior.
- The Mac evidence is same-day but still not a substitute for an independent live Alienware preflight.

**Insights:** Verification must bind four identities in sequence: source commit → immutable release manifest → downloaded bytes → executed/installed bytes.

**Mode:** Convergent

### Step 3: Hypothesize

| Hypothesis | Prediction | Falsifier | Prior |
|---|---|---|---|
| H1: Published v1.0.2 is fully healthy on this Alienware. | Identities agree; supported lifecycle and runtime checks pass without unexplained residue. | Any reproducible contract failure from verified clean public bytes. | Medium-high after hosted x64/ARM64 success. |
| H2: Local state or artifact/provenance drift creates an apparent failure. | Baseline reveals an older install, running process, stale PATH/receipt, mixed latest/tag bytes, or wrong installer provenance. | Clean inventory plus exact hashes and isolated paths still reproduce the failure. | Medium; common in installer testing. |
| H3: Real Alienware hardware exposes a Windows product defect missed by hosted gates. | Failure correlates with DWM, display topology/DPI, audio, terminal classification, IPC timing, or lifecycle behavior and reproduces from exact public bytes. | Same exact path passes repeated clean runs and retained evidence. | Low-medium but the reason this task exists. |
| H4: Direct-MSI stale receipt fields are operationally unsafe. | A stale receipt changes later update selection, exact-path proof, or uninstall ownership. | Authoritative registry/source markers drive all decisions and later operations succeed despite stale informational fields. | Low based on code review, not yet hardware-tested. |

**Mode:** Divergent

### Step 4: Experiment — committed predictions before execution

**Cheapest decisive sequence:**

1. Read-only host/install inventory.
2. Live release/tag/manifest identity and fresh download with independent hash/signature checks.
3. Clean published executable smoke before installation where supported.
4. Reversible machine-wide lifecycle matrix using real installers and preserved evidence.
5. Manual runtime/terminal/audio/display observations.

**Predictions committed in advance:**

- If H1 is true, the immutable and latest manifests agree on v1.0.2 and every downloaded artifact matches exact size/SHA before valid signature/architecture checks.
- If H2 is true, inventory or identity binding will expose contamination before a product test is interpreted.
- If H3 is true, at least one exact-byte real-hardware check will fail consistently after contamination is excluded.
- If H4 is false, stale informational receipt fields will not alter authoritative update selection, installed-path proof, or uninstall ownership.
- If none fit, stop interpretation and add a new hypothesis before modifying source.

**Safety and reversibility:** Capture baseline first; preserve operator data; use supported install/update/uninstall paths; never overwrite a running binary; retain logs and rollback evidence.

**Mode:** Convergent

---

## Visual Models In Play

### Identity and evidence chain

source SHA → immutable manifest → downloaded SHA/size/signature → executed binary → installed registry/path/source marker → observed runtime

Every arrow must be demonstrated; downstream success cannot repair an upstream identity gap.

---

## Steel-Manned Dissent

- **The case against doing this work:** Hosted Windows x64/ARM64 candidate and post-release smokes already passed, so repeating them on one consumer laptop may add little evidence while risking local-state contamination.
- **What would have to be true:** GitHub environments would need to represent the Alienware's DWM, DPI, display topology, audio, terminal set, and installer history closely enough that local differences cannot reveal defects.
- **How it was handled:** Rejected as too strong. The task remains verification-first and starts with a contamination inventory; automation is repeated only against published bytes, then supplemented with hardware-specific observations.
- **Confidence in the rejection:** High.

---

## Closing

Session remains open. Findings, sanity check, conclusion, confidence, next steps, and revisit date will be appended after experiments.

---

## Operator Decision — two-release sequence

- 2026-07-17 04:50: the operator explicitly required two separate releases in this session.
- First release: v1.0.3 for the original Alienware verification and evidence-backed compatible hardening.
- Second release: v1.1.0 for shared Windows/Linux tray parity, starting only after v1.0.3 is public and independently verified.
- Status: confirmed constraint; the board now represents two milestones and dependency-blocks tray design on #r103.

---

## Experiment 1 — Alienware baseline inventory

**Predictions checked:** H2 predicted that pre-existing local state might contaminate a naive fresh-install interpretation. H1 and H3 were not yet tested because no current public bytes were executed.

### Observations

- Host: Alienware m16 R2, Windows 11 Home x64 build 26200, Intel Core Ultra 7 155H.
- Session shell is not elevated; elevation requirements must be exercised through supported installer behavior.
- Display: one primary 1920×1080 32-bit screen, NVIDIA RTX 4070 Laptop plus Intel Arc graphics. Live multi-monitor and hot-plug are unavailable in the present topology.
- Audio: real Realtek speakers plus Shure virtual endpoints are present.
- Terminals/classifier targets present: Windows Terminal, PowerShell 7, Windows PowerShell, cmd, VS Code, and Codex. Ghostty is absent.
- No honk300/goose process was running.
- Existing installation: machine-wide Global MSI honk300 v0.3.1 at C:\Program Files\honk300, with HKLM InstallSource=msi-global and three byte-identical aliases.
- Existing binaries report honk300 0.3.1 and are unsigned. That is historical fixture evidence, not a claim about v1.0.2.
- No LocalAppData honk300 state or install receipt, no autostart value, and no discovered Honk300 shortcut were present.

### Analysis

- H2 is **diagnostic and confirmed for baseline contamination**: the machine was not clean, but the old supported MSI is a valuable real update fixture.
- The assumption that no existing install would contaminate the baseline is dismissed.
- The host-identity assumption is confirmed.
- The operator explicitly authorized treating the local install as a very old update source.
- No product defect is inferred from v0.3.1 signature/shortcut history.

**Confidence:** High; all observations came from Windows process, registry, filesystem, version, hash, display, audio, and command inventory.

**Next experiment:** Resolve the live immutable v1.0.2 release identity, compare immutable/latest manifests, and fresh-download exact Windows x64 assets before executing them.

---

## Experiment 2 — immutable release identity and exact Windows x64 bytes

**Predictions checked:** H1 predicted agreement from source/tag through downloaded bytes. H2
predicted that any local or rolling-alias drift would be exposed before execution.

### Observations

- GitHub authentication is active as `RealEmmettS`; a fresh tag fetch pruned the already-removed
  work branches and fetched `v1.0.2` locally.
- The annotated tag, immutable GitHub Release target, and manifest commit all resolve to
  `964305869e9ec28768c789465db1b6317dfa3f6f`.
- GitHub reports `v1.0.2` as the latest, immutable, non-draft, non-prerelease release published at
  2026-07-17T08:39:59Z. Current `main`/`origin/main` is the later clean-up commit
  `6c469f8ab588761233b4da1a0d94a1b181ab1349`, so the release was not falsely rebound to branch
  head.
- The immutable-tag and rolling-latest `release-manifest.json` downloads are byte-identical at
  SHA-256 `13df4b79bc73e0b68051169661ad14957a7c61f123da2c497f8858e562d0e0b2`.
- The manifest declares schema `honk300.release.v1`, tag `v1.0.2`, version `1.0.2`, and the exact
  release commit above.
- Fresh downloads covered the PowerShell bootstrap, x64 portable ZIP, Global MSI/EXE, and
  Corporate MSI/EXE. All six payloads agree independently with manifest size/hash, their named
  sidecars, `sha256.sum`, and GitHub's own release-asset size/digest; all nine accompanying
  publication files also agree with GitHub metadata.
- The portable executable is PE x86_64 and reports `honk300 1.0.2`; both setup launchers are the
  expected x86 Inno bootstrap. Global and Corporate MSIs both declare version 1.0.2 with distinct
  stable UpgradeCodes and expected product names/scope properties.
- Windows payloads are Authenticode `NotSigned`. The repository's fail-closed public-signing
  contract is macOS-specific; this is therefore a current Windows distribution characteristic,
  not an identity mismatch. It remains relevant to user trust/SmartScreen but is outside this
  verification card unless a board-scoped test exposes a functional defect.

### Analysis

- H1 is **confirmed through the downloaded-byte boundary** with high confidence.
- H2 is **rejected for public artifact drift**: the rolling alias, immutable tag, sidecars,
  checksum inventory, manifest, and GitHub digests agree. H2 remains active only for the known
  local v0.3.1 install fixture.
- H3 and H4 remain untested by this experiment.
- No source modification is justified by release identity or packaging metadata.

**Confidence:** High; the chain was independently checked against Git object identity, GitHub
release metadata, two download routes, manifest metadata, sidecars, aggregate checksums, file
bytes, PE headers, MSI tables, and executable-reported version.

**Next experiment:** Run the repository compositor smoke against the exact verified portable
v1.0.2 executable, retain and visually inspect its DWM evidence, then begin supported lifecycle
mutation from the preserved v0.3.1 Global MSI fixture.

---

## Experiment 3 — exact published x64 compositor and IPC lifecycle smoke

**Predictions checked:** H1 predicted that the verified portable payload would pass the repository's
paired-DWM and lifecycle contract on the Alienware. H3 predicted a repeatable real-hardware failure
if this host exposed a compositor, IPC, or immediate-restart defect.

### Observations

- `script/smoke_windows_overlay.ps1` passed against the executable extracted from the freshly
  verified v1.0.2 x64 portable ZIP; executed SHA-256 was
  `c22a5c488765b330a4ed215041fff6c860ec937427532b77933cd43a922ddf8f`.
- Paired controlled dark/light DWM captures passed every semantic threshold: visible body,
  outline, wing, shading, two-tone orange articulation, semi-transparent edges/shadow,
  transparent background, complete content margin, and no opaque black surface.
- All six PNG evidence files were inspected directly. The controlled backgrounds are uniform;
  both retained attempt-3 and canonical pairs show the complete side-view goose, beak, legs,
  wing, outline, shading, and appropriate light/dark composition without stale pixels or a black
  rectangle.
- Runtime status reported Windows overlay/cursor/window/collect/presence/audio supported, twelve
  note and fifteen meme assets, and expected bare-binary identity.
- Single-instance rejection, reload, graceful stop, stopped status, immediate restart, restarted
  status, and the second graceful stop all passed. Both runtime logs ended with `goose walked
  home; stopping`; stderr logs were empty.

### Analysis

- H1 is **confirmed for the published portable compositor and automated IPC lifecycle slice**.
- H3 is **not supported by this experiment**: the Alienware's current hybrid NVIDIA/Intel DWM
  path produced valid transparency and semantic renderer evidence on the first completed smoke.
- This does not yet cover installed provenance, update selection, repair/uninstall, interactive
  terminal/TUI behavior, audio playback, reactions, alternate display topology, or long-run
  resource behavior.

**Confidence:** High for the automated contract and retained images; medium for broader visual
behavior because this gate intentionally captures one qualifying side pose rather than every
autonomous task.

**Next experiment:** Review the supported updater/installer commands and then mutate the retained
v0.3.1 Global MSI only through published product paths, retaining registry, receipt, file, alias,
and rollback evidence at each boundary.

---

## Experiment 4 — Windows latest-manifest discovery from released binaries

**Predictions checked:** H1 predicted that current v1.0.2 would discover its own rolling manifest
and return an update no-op. H2 predicted that a failure might belong only to stale v0.3.1 state.

### Observations

- The installed v0.3.1 `honk300 update` failed before download/elevation with Windows PowerShell
  `Unexpected token 'https://github.com/.../release-manifest.json'`.
- The exact verified portable v1.0.2 executable reproduced the identical parser error and exit 1.
- Source inspection isolates the defect to `fetch_text_with_system_tool`: it passes a PowerShell
  `param($uri) ...` script through `-Command`, then appends the URL as a process argument. Windows
  PowerShell concatenates that argument into command text instead of binding it as `$uri`.
- No installer, updater helper, or UAC boundary was reached; the v0.3.1 install remains unchanged.

### Analysis

- H1 is **falsified for Windows CLI update discovery**. This is a current v1.0.2 defect, not merely
  an old-install limitation.
- H2 is **rejected as the primary explanation** because a clean exact v1.0.2 portable binary
  reproduces without consuming installed receipt or registry state.
- H3 is **refined**: this is a Windows command-construction defect exposed by native execution,
  not Alienware hardware or DWM behavior.
- The smallest justified patch is to place user-controlled URL/output values in environment
  variables (or otherwise outside PowerShell command text), retain System32 PowerShell selection
  and hidden execution, and add Windows-focused tests proving arguments are not appended to
  `-Command`. Both text fetch and file download paths share the risk and must be covered.

**Confidence:** High; the failure is deterministic in two released versions and the error text
matches the inspected invocation exactly.

**Next experiment:** Use the independently verified v1.0.2 Global MSI to cross the broken updater
boundary, then continue installed-provenance, repair, rollback, uninstall, and runtime checks. The
regression and its fix belong to the planned v1.0.3 patch.

### Experiment 4 continuation — red/green updater hardening

- A red regression demonstrated that both the URI and output path were appended after PowerShell
  `-Command` and that neither was present in a child-only data channel.
- Moving those values to two scoped child environment variables fixed command parsing without
  interpolating user-controlled strings into PowerShell source.
- The first green attempt exposed a second native detail: GitHub serves the manifest such that
  Windows PowerShell 5.1 returns `Content` as `System.Byte[]`. Explicit UTF-8 byte decoding plus
  UTF-8 console output fixed the real parser boundary.
- The focused regression is green. A rebuilt binary successfully fetched the live rolling
  manifest and reported the expected v1.0.2 no-op. The file-download half independently wrote to
  a path containing spaces and an apostrophe and matched the immutable manifest SHA-256 exactly.

**Updated confidence:** High. Both pure command construction and the two real Windows PowerShell
network paths are proven.

---

## Experiment 5 — non-elevated Corporate MSI lifecycle and installed runtime

**Reason for the branch:** The operator canceled the Global-MSI UAC prompt before `msiexec`
started; no log or state change occurred. That gate remains open. The verified Corporate MSI is a
supported per-user path and allowed useful isolation without misreporting Global coverage.

### Observations

- Published v1.0.2 Corporate MSI installed beside the untouched Global v0.3.1 fixture. It produced
  three byte-identical x86_64 aliases reporting 1.0.2, adjacent/HKCU `msi-corporate` provenance,
  a correct Start Menu shortcut and user PATH entry, no surprise autostart, and no bootstrap
  receipt.
- Repair returned 0 and preserved installed SHA-256 exactly. A v0.2.1 downgrade attempt returned
  1603 and left version 1.0.2 intact.
- A forced post-InstallFiles v1.0.2 failure restored the prior v0.2.1 binary byte-for-byte; the
  subsequent clean upgrade returned to 1.0.2.
- The installed binary passed the full paired-DWM/lifecycle smoke. Its qualifying top-down pose
  complements Experiment 3's side pose; all six new PNGs were visually inspected and valid.
- Three cycles proved `honk300`, `honk`, and `goose` start/status/reload/action/stop interoperability.
  `bad`, `exit`, and `no honk` each produced graceful walk-off with zero stderr and roughly 2.0
  second shutdown; immediate restarts succeeded.
- The audio-enabled path reported supported, applied two honks, logged no errors, and walked off.
  Audible quality is not claimed by automation. The muted path already passed both smokes.
- A real 80×24 terminal showed the complete nine-category TUI. Direct screenshot inspection found
  no clipping; `q` exited and restored the same prompt, which then closed normally.
- Published v1.0.2 Corporate `uninstall` failed closed because source detection preferred the
  coexisting HKLM Global marker and the Corporate ARP record was in HKLM on this host while the
  lookup expected HKCU. No managed file was touched.
- Red regressions now require the adjacent running-install marker to win over a coexisting registry
  marker and require Corporate uninstall discovery to search HKCU then HKLM while retaining exact
  product/publisher/product-code/install-root validation. Both are green.
- The deferred helper then exposed a second failure: an already-exited parent makes Windows
  PowerShell's silent `Get-Process` return process exit 1. A red/green regression now makes this
  explicit success while retaining stop-on-error behavior for genuine wait failures.
- With those patches, a clean Corporate CLI uninstall removed MSI ownership and preserved config
  plus both user-media sentinels. A clean `--purge` removed state only after backing up both files;
  normal v0.2.1→v1.0.2 upgrade followed by patched CLI purge also passed. Global v0.3.1 remained
  untouched throughout.

### New hypothesis H5 — Corporate rollback/retry leaves an orphan client

- In the forced-failure/retry sequence only, final v1.0.2 uninstall reported success but retained
  files. Verbose MSI evidence showed each current component was protected because “another client
  exists.” `MsiEnumClients` identified the removed v0.2.1 ProductCode even though both products
  were unregistered.
- Reinstalling then uninstalling the exact v0.2.1 MSI cleared those component clients without
  manual Installer-registry edits. A clean old→new upgrade/uninstall then passed.
- H5 prediction: Corporate `RemoveExistingProducts` at `afterInstallInitialize` involves the old
  product in rollback and can strand client registrations after a mid-upgrade failure; a schedule
  that commits the new product before removing the old should leave v0.2.1 untouched on injected
  failure and allow retry/uninstall without orphan clients.
- H5 falsifier: the alternative schedule reproduces the same orphan, or component/rule analysis
  shows the observed client belongs to a different cause.

### Analysis

- H1 is confirmed for Corporate install, repair, ordinary upgrade, runtime, command aliases,
  TUI, audio initialization, non-purge uninstall, and purge.
- H3 is confirmed for three Windows lifecycle defects in current v1.0.2: web-request construction,
  side-by-side install-source/ARP discovery, and already-exited deferred-parent handling.
- H5 remains active and must be resolved or explicitly bounded before v1.0.3.
- The Global MSI lifecycle remains unexecuted because elevation was canceled, not failed.

**Confidence:** High for every recorded native result; medium for H5's causal schedule explanation
until the alternative is built and run.

**Next experiment:** Research the official Windows Installer scheduling contract, patch/test the
smallest Corporate rollback/retry correction if supported, then retry the physical Global gate
when UAC consent is available.

### Experiment 5 continuation — H5 schedule falsification test

- Microsoft documents `RemoveExistingProducts` after `InstallFinalize` as committing the new
  product first and removing the old product in a separate transaction; if old-product removal
  fails, only that removal rolls back. WiX exposes this placement as `afterInstallFinalize`.
- A locally built Corporate 1.0.3 test MSI using that schedule was installed over a clean v0.2.1
  fixture. An injected failure after file installation returned 1603 and left the complete old
  executable byte-for-byte unchanged.
- Retrying the unmodified test MSI succeeded. Its uninstall returned 0, removed the binary, left
  no clients for the four sampled shared components, and left the v0.2.1 ProductCode absent.
- The packaging contract now pins Global to its existing transactional
  `afterInstallInitialize` schedule and Corporate to `afterInstallFinalize`; its red regression is
  green.

### H5 conclusion

H5 is **confirmed** for the reproduced failure path. Moving only the per-user Corporate package
to `afterInstallFinalize` prevents the failed new transaction from involving the old product and
prevents the retry/uninstall orphan observed under `afterInstallInitialize`. This changes failure
semantics intentionally: a failure while removing the old product can leave the newly committed
Corporate product installed and the old removal rolled back, which is safer for a non-elevated
retry than corrupting component ownership. Global remains unchanged.

**Confidence:** High for the tested x64 Corporate path; the causal prediction, injected failure,
retry, uninstall, component-client enumeration, and official sequencing model agree.

**Next experiment:** Implement and test the already-planned guarded refresh of an owned
PowerShell-bootstrap receipt after a direct MSI update. Foreign, malformed, or reparse-point
receipts must remain untouched.

---

## Experiment 6 — direct-MSI bootstrap receipt freshness

### Prediction

Only an existing receipt with the exact Honk300 ownership schema, PowerShell Global-MSI channel,
Windows Global-MSI layout, complete bootstrap shape, managed install root, and regular non-reparse
file/directory chain should advance after the installed executable reports the manifest version.
All other paths should remain unchanged and should not make a valid MSI update fail.

### Observations

- A red handoff regression proved no receipt refresh existed after exact-path version verification.
- The helper now gates the refresh to direct Global MSI only. Delegated PowerShell bootstrap
  updates continue to own their complete receipt transaction.
- A native Windows PowerShell test used a receipt path containing an apostrophe. Exact owned
  version/tag/commit/target/artifact metadata advanced through a create-new sibling and atomic
  replacement while aliases and autostart data were retained.
- The same native test proved malformed JSON and a foreign channel remain byte-for-byte unchanged.
  Generated-helper assertions pin missing/foreign return paths, directory/reparse guards,
  create-new staging, atomic replacement, and version-verification-before-refresh ordering.

### Analysis

The v1.0.2 readiness limitation is resolved in v1.0.3 source without redefining receipt fields as
merely informational. Ownership does not broaden: absence and non-matching state are no-ops, while
write failure after exact ownership proof remains a real update-helper failure rather than silently
leaving trusted metadata stale.

**Confidence:** High for native Windows PowerShell behavior and ownership guards; final confidence
still requires the public v1.0.2→v1.0.3 Global CLI update after release.

---

## Experiment 7 — live integrated-terminal identity audit

### Observation and prediction

The current Codex desktop window enumerated as process `ChatGPT`, class
`Chrome_WidgetWin_1`, title `ChatGPT`. VS Code also uses a generic Chromium class. The pre-patch
Windows classifier did not protect either title unless some other word happened to match a
terminal token. Linux explicitly treated `code / Visual Studio Code` as a regular application,
while macOS already protected Codex and Visual Studio Code by bundle/application identity.

### Result

Red tests reproduced the Windows and Linux misses. Windows now protects Visual Studio Code,
Codex, and the observed ChatGPT-titled Codex surface; Linux protects `code` and `codex` tokens.
Focused regressions are green and ordinary Notepad, Firefox, and file-manager controls remain
eligible.

**Conclusion:** H3 extends to a cross-platform terminal-safety inconsistency. Conservative full-
tool protection matches the existing macOS contract and is safer than guessing whether an
integrated terminal is visible inside a generic application HWND/surface.

**Confidence:** High for classification. A live destructive manipulation attempt is unnecessary
and would be weaker than proving the surface is removed before target selection.

---

## Experiment 8 — second Global elevation attempt

The verified v1.0.2 Global MSI was launched with an explicit verbose-log path and UAC elevation.
Consent was not received before the bounded launcher timeout. No `msiexec` remained, no MSI log
was created, and the preserved Global executable still reported v0.3.1 with SHA-256
`50ffd6e670b334d2887c515cd5f017a488fee421b251ebccb1d780dfc8885af4`. This is a non-event,
not a failed lifecycle result. The physical Global gate remains open for the final candidate/public
update rather than being inferred from Corporate or hosted evidence.

---

## Experiment 9 — optimized v1.0.3 source/package qualification and final elevation boundary

- Workspace format, warnings-as-errors Clippy, 407 Rust unit/integration tests plus seven renderer
  goldens, optimized build, cargo audit, cargo-dist plan/generation, actionlint, script parsing,
  Windows packaging, and release-orchestration contracts pass.
- Windows ARM64, both Apple platform/core targets, and all four Linux GNU/musl platform/core
  targets cross-check. Full CoreAudio/app signing requires the native Mac producer; Linux POSIX
  filesystem/compositor execution requires the candidate Linux runners.
- The optimized x64 source binary reports 1.0.3 and SHA-256
  `ab0e1f57b12583fd36c75b169e7bf4a0b89b548645ac109d9427f274766a6332`. It passed the
  paired-DWM lifecycle smoke; direct inspection showed uniform baselines and a complete side pose
  with transparent margins, correct palette/channels, wing, shade, articulated legs, and no black
  surface.
- Locally assembled Global/Corporate MSIs report version 1.0.3. Global retains sequence 1501;
  Corporate uses sequence 6601. Global administrative extraction contains the exact x64 PE.
  Corporate installed beside old Global, reported supported capabilities, walked off, and removed
  itself through the patched CLI; product/binary cleanup passed and old Global stayed v0.3.1.
- The final verified-public-Global launch returned Windows' explicit “operation was canceled by
  the user” before `msiexec`. As with the two earlier attempts, no log or byte change occurred.

## Final sanity check

- **Identity:** public v1.0.2 tag/latest/source/download hashes agree; every mutated Corporate or
  locally assembled artifact was separately identified and never represented as public bytes.
- **Alternative explanations:** stale local v0.3.1 state does not explain the updater parser bug
  because exact portable v1.0.2 reproduced it. The Corporate orphan appears only after the forced-
  failure/retry sequence and disappears under the prediction-derived schedule. The Codex miss is
  demonstrated by the live class/title plus red classifier tests.
- **Counterexamples:** ordinary old→new Corporate upgrade/uninstall passed even under the old
  schedule; therefore the schedule finding is intentionally limited to rollback/retry. Notepad,
  Firefox, and file-manager classifier negatives remain green. Missing/foreign receipts remain
  no-ops.
- **Claims not made:** no live second-display/hot-plug, Ghostty, Linux desktop, CoreAudio, Mac
  signing, or physical privileged Global result is inferred. Candidate workflows own the relevant
  native/privileged gates.
- **Scope:** every code change maps to a reproduced Active-card finding. Tray implementation and
  Backlog work remain untouched.

## Conclusion

- H1 is **partially falsified**: public v1.0.2 rendering, runtime, portable, and ordinary Corporate
  behavior are healthy, but its Windows updater invocation and three lifecycle/safety edges are
  not fully healthy on this Alienware.
- H2 is **confirmed only as baseline context** (the old Global fixture and coexisting installs)
  and rejected as the primary cause of the reproduced current-version defects.
- H3 is **confirmed** for updater command construction/byte decoding, coexisting lifecycle
  identity, deferred-parent timing, Corporate forced-failure retry, and integrated-terminal
  classification. Each has a red/green regression and native or prediction-matched evidence.
- H4 is **refined rather than confirmed**: stale receipt identity did not control authoritative
  lifecycle selection, but leaving exact owned metadata stale is avoidable. v1.0.3 refreshes only
  proven bootstrap ownership after installed-version verification.
- H5 is **confirmed** for the forced-failure/retry path and resolved by Corporate-only late old-
  product removal.

**Overall confidence:** High that the accepted v1.0.3 patch is causally tied to the findings and
does not weaken other platform contracts. Medium-high for full release readiness until candidate,
same-SHA main CI, hosted Global/public installer smoke, and fresh stable/latest checks pass.

**Exit condition:** The investigation itself is complete. All findings are reproduced, bounded,
regression-tested, documented under ADR 0029, and handed to active release task `#r103`. The
canvas should reopen only if candidate/public evidence contradicts these predictions.

**Revisit:** During v1.0.3 candidate/public verification in this session; separately when real
Linux hardware, a second display, Ghostty, or operator-approved physical Global elevation becomes
available.
