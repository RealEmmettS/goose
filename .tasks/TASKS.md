# Tasks

## Backlog
- [ ] **Default-OFF spicy behaviors** - clipboard honk, fake-photo flash, gaggle cameo, easter eggs, goose gifts, speech bubbles (plan §5.12); generate any needed image assets with the image-gen tool using the project's clumsy MS-Paint base prompt (see `b9e.md`); preserve terminal-window protection absolutely #b9e
- [ ] **Portable Wayland observation and portal spike** - probe staging toplevel identity plus explicit XDG Remote Desktop/libei cursor capability without claiming geometry/move parity; terminal protection and revocation are hard gates (needs #a6e) #wlp
- [ ] **KDE native Wayland companion** - prototype a user-enabled KWin script and authenticated same-user bridge for exact window geometry, move-state, bounded movement, topology, and terminal-negative enforcement (needs #a6e) #wlk
- [ ] **GNOME and wlroots adapter prototypes** - separately test a versioned GNOME Shell `Meta.Window` companion and capability-probed Sway/Hyprland adapters; publish support only per proven compositor/version (needs #a6e) #wlg

## To-Do

## Active

## Done
- [x] **Close the v1.2.2 provenance-preserving updater fix-forward** - preserved immutable v1.2.0/v1.2.1, qualified the semantic receipt verifier, verified every public byte, and cleaned every branch (done 2026-07-18) (needs #u31) #r120
  - [x] Synchronize v1.2.2 version, readiness, board, public guidance, and both changelogs.
  - [x] Pass the complete v1.2.0 local, cross-target, packaging, audit, and exact-candidate gate.
    > Candidate 29649872307 attempt 2 passed at fcbb6337830a14cb187c93cfe3e499a3f2f73945.
  - [x] Diagnose the reported Windows white-screen event and retire full-desktop calibration from local/product execution.
    > The white/blank display was the developer compositor fixture, not product rendering. Product startup has no calibration; the former local obscuration opt-in now refuses, and strict paired-color surfaces run only on disposable CI.
  - [x] Make Windows shortcuts, login startup, and background helpers windowless and non-activating while preserving intentional CLI terminals.
    > Console-subsystem public aliases stay scriptable; independently hashed GUI-subsystem `honk300-app.exe` starts the exact sibling with `CREATE_NO_WINDOW`, no shell/PowerShell intermediary, and is wired through all MSI/EXE/manual shortcut and Run owners.
  - [x] Prove disposable-CI offscreen walk-in, graceful stop/quit/tray walk-off, and immediate every-alias `--force` shutdown without global input.
    > Focused exact-SHA Windows run 29649183286 passed x64 and native ARM64 lifecycle/compositor proof plus every installer-origin takeover while each old process stayed mapped.
  - [x] Prove owned note and image windows stay readable, aspect-correct, uncropped, and bounded to the active monitor without taking over the screen.
    > Shared Windows/macOS sizing tests pass; live Windows proof measured a 614x346 note and a full-source 648x518 image on a 1920x1080 monitor, captured the image, and completed graceful cleanup.
  - [x] Add a cross-platform login-autostart config option that reconciles through each install family's existing owned startup mechanism.
    > Default-off TOML/TUI state now uses only receipt-owned HKLM/HKCU Run, LaunchAgent, or XDG autostart; fresh installer intent wins over stale config, foreign owners fail closed, and all four x64 Windows packages compile with exact selected state in the receipt.
  - [x] Merge the v1.2.0 candidate-proven SHA to main and pass same-SHA ordinary CI.
    > Main CI 29650347708 passed the exact candidate SHA.
  - [~] Publish v1.2.0 atomically and preserve its public-byte failure as immutable evidence.
    > Atomic run 29650650643 published 47 assets. Public run 29651013524 passed Mac, Debian, and Windows but both Linux shell jobs caught the same foreign-autostart receipt defect.
  - [~] Fix forward as v1.2.1 and repeat candidate, same-SHA main, publication, public-byte, latest, and site gates.
    > v1.2.1 candidate/main/publication passed. Public Linux install/start/X11/Wayland passed, then a compact-JSON harness assertion stopped rollback closure after runtime pretty-printing.
  - [x] Pass the v1.2.1 noninteractive local, cross-target, audit, parser, and package preflight.
    > 434 Rust tests, 83 focused contracts with two POSIX skips, strict native/musl Clippy, ARM64 Windows check, RustSec, actionlint, cargo-dist, shell/PowerShell parsing, and package-list preflight pass without launching the app.
  - [x] Ship v1.2.2's semantic receipt verifier and repeat the exact candidate/main/public/latest/site closure.
    > Candidate 29653156436, main CI 29653528229, atomic release 29653833434, and all-eight-lane public run 29654151526 passed at bbbe86367d185fe8910af6326a7d51c04b282aec.
  - [x] Pass the v1.2.2 noninteractive local, cross-target, audit, parser, and package preflight.
    > Semantic verifier contracts, 434 Rust tests, strict native/musl Clippy, locked build, ARM64 Windows check, RustSec, actionlint, cargo-dist, and parser gates pass without launching the app.
  - [x] Return to clean main and remove every non-main local and remote branch after preserving merged work.
    > Every deleted tip was an ancestor of main; the stale worktree, two local branches, three release branches, and protected starting archive were removed. Only main remains.
- [x] **Provenance-preserving slot-based self-updater** - implemented synchronous same-origin updates, atomic Windows release slots, DMG-managed Mac updates, and complete release qualification (done 2026-07-18) (owner codex) #u31
  - [x] Record ADR 0031 and the versioned receipt/provenance contract.
  - [x] Implement synchronous CLI/JSON results and fail-closed origin resolution.
  - [x] Implement Windows immutable release slots, stable junctions, and safe legacy migration/takeover.
  - [x] Preserve DMG, Debian, shell, and PowerShell update origins without cross-channel conversion.
  - [x] Update installers, unversioned release artifacts, workflows, and public guidance.
  - [x] Pass focused fault tests and the complete local/release qualification gate.
    > v1.2.2 completed candidate, same-SHA main, atomic publication, all platform-native packaging, exact/latest identity, production-site, and eight-lane fresh-public-byte proof.
- [x] **Publish v1.1.0 cross-platform tray parity** - published and independently verified the immutable Windows/Linux tray-parity release across the complete Windows, macOS, Linux, Debian, stable/latest, and production-site matrix (done 2026-07-17) #r110
  - [x] Synchronize version, ADRs, README, guidance, task records, and both changelogs.
  - [x] Pass local, cross-target, packaging, security, workflow, and platform-native gates.
  - [x] Pass candidate and same-SHA main CI before creating the immutable tag.
    > Candidate 29588487072 and main CI 29589048598 passed at exact commit e58b5ec09ea140e22927e3f8e8cf339b5a7d5bea.
  - [x] Publish atomically, pass post-release smoke, fresh-download artifacts, verify updates, and finish on clean main.
    > Atomic release 29589698302 and post-release smoke 29590274819 passed; 47 public assets, recommended payload hashes, fresh Windows tray behavior, stable/latest, and the manifest-driven production site all agree on v1.1.0.
- [x] **Linux tray implementation and hosted qualification** - added accessible Configure/graceful-Quit StatusNotifier controls, explicit unavailable degradation, and package ownership without changing X11 or native Wayland capability claims (done 2026-07-17) #ltray
  - [x] Implement the supported Linux status-notifier path with explicit unavailable degradation.
  - [x] Preserve one TUI schema, graceful IPC stop, terminal protection, and X11/Wayland capability boundaries.
  - [x] Integrate archive, Debian, install, update, and uninstall ownership.
  - [x] Qualify X11 and native Wayland reduced mode in hosted/native Linux jobs and document unavailable desktop-shell cases.
    > CI 29584610137 passed the private StatusNotifier watcher/host protocol, menu actions, recovery, deterministic no-bus degradation, X11 and native Wayland reduced-mode smokes, Debian contracts, and GNU/musl target checks on native x64 and ARM64 Linux.
- [x] **Windows tray implementation and native qualification** - added one accessible fixed-GUID goose icon with Configure and graceful Quit, kept packaging ownership embedded, and proved recovery/lifecycle behavior on the Alienware (done 2026-07-17) #wtray
  - [x] Implement the Windows notification-area icon and native action routing.
  - [x] Preserve terminal restoration, local IPC, permissions, singleton, and animated graceful exit.
  - [x] Integrate installer ownership, upgrade, repair, and uninstall behavior.
  - [x] Qualify keyboard/accessibility, Explorer restart, immediate restart, DPI, and multi-monitor behavior.
    > Alienware proof covered UI Automation/keyboard access, exact-executable TUI launch and restoration, 3.1-second animated Quit, immediate restart, fixed-GUID TaskbarCreated recovery, and 100% DPI. The host exposes one 1920x1080 display, so live multi-monitor proof is explicitly unavailable; signed-coordinate/topology tests and the native matrix remain mandatory.
- [x] **Shared tray contract and asset pipeline** - defined the finite native action contract, accepted platform event-loop/dependency boundaries, shared one embedded goose representation, and prevented abrupt Quit or a second settings model (done 2026-07-17) #trayc
  - [x] Research native tray dependencies and event-loop constraints for supported Windows and Linux targets.
    > Native Win32 keeps the existing pump; pinned pure-Rust ksni compiles for musl without GTK/AppIndicator dependencies, while absent Linux watcher/host remains explicit.
  - [x] Define the shared tray command/lifetime contract and terminal-launch behavior in a new ADR.
    > ADR 0030 makes honk-control::ControlSurfaceCommand finite and routes Quit only through RuntimeCore::begin_graceful_stop.
  - [x] Build reusable icon derivation and packaging checks from the canonical goose SVG.
    > Both binaries embed the tracked 36x36 representation; native BGRA/ARGB contrast tests pass and Debian package/icon ownership assertions are wired for Linux CI.
  - [x] Add platform-neutral tests that prevent abrupt Quit or a second settings model.
    > Injected Configure/launcher tests and Quit isolation pass; Linux launchers use literal argument vectors without shell interpolation.
- [x] **Publish v1.0.3 Alienware hardening** - published and independently verified the exact-SHA immutable patch with native/public Windows lifecycle, update-discovery, terminal-safety, signed Mac, Linux, Debian, stable/latest, and site evidence (done 2026-07-17) #r103
  - [x] Synchronize the patch version, ADR/readiness guidance, board evidence, and both changelogs.
  - [x] Pass the complete local, cross-target, packaging, workflow, audit, and native Windows gate.
  - [x] Pass candidate and same-SHA main CI before creating the immutable tag.
    > Candidate 29577145711 attempt 2 and main CI 29577774029 passed at exact commit 5192fab9690ff8b6777366a5918c12bbe1ee247a.
  - [x] Publish atomically, fresh-download and verify artifacts/updates, validate stable/latest/site, and return to clean main.
    > Atomic release 29578238463 and post-release smoke 29578671930 passed; fresh public Windows/site checks and the supported v1.0.2→v1.0.3 installer migration passed with the physical Global elevation waiver preserved.
- [x] **Post-v1 Alienware and supplemental native verification** - fresh-downloaded and bound public v1.0.2, completed physical Windows compositor/Corporate/runtime qualification, and converted every reproduced finding into reviewed v1.0.3 regressions without mutating prior releases (done 2026-07-17) #v1a
  - [x] Verify published SHA, manifest, hashes, stable latest aliases, and Windows x64 lifecycle paths on the Alienware.
    > Portable and Corporate paths passed. Three Global elevation attempts ended before `msiexec`, logging, or byte mutation—the final attempt was explicitly canceled by the operator—so physical privileged mutation is waived, not inferred; candidate and post-release hosted Global MSI smoke remain mandatory.
  - [x] Run the exact Windows layered-overlay lifecycle smoke and visually inspect side/top-down, movement, reactions, notes/memes, audio, terminal protection, and multi-monitor behavior.
    > Published and optimized v1.0.3 source smokes passed with directly inspected side/top-down evidence, three-alias control, graceful restart, audio initialization, and TUI restoration. The one-display host cannot provide live seam/hot-plug proof; Ghostty is absent.
  - [x] Record Linux real-hardware observations where available without overstating native Wayland reduced-mode capabilities.
    > This host has no Linux desktop or WSL distribution. All four GNU/musl platform targets cross-check; native X11/Wayland and POSIX filesystem proof remain exact candidate gates.
  - [x] Ship findings only as reviewed forward patch releases; never move or replace v1.0.0/v1.0.1/v1.0.2 tags or assets.
    > ADR 0029 and task #r103 bind all fixes to v1.0.3; prior tags/releases remain unchanged.
- [x] **v1.0.2 macOS production patch — shared goose menu icon and exact release** - replaced the text status item with a reusable accessible goose template icon, repeated physical-Mac/package verification, published the immutable patch, and advanced the DMG-first latest channel without implementing Windows/Linux trays (done 2026-07-17) #v102
  - [x] Generate and normalize the shared Quiver goose SVG with a deterministic AppKit raster representation and recorded provenance.
  - [x] Integrate the image-only macOS status item with explicit accessibility naming, retained lifetime, proportional scaling, and a safe text fallback.
  - [x] Seal and require both icon resources in the staged app, final app ZIP, and both DMG verification mounts.
  - [x] Prove the packaged icon, Configure-to-TUI path, permission-wait availability, and animated graceful Quit on this physical Mac.
    > Exact public v1.0.2 exposed the image-only goose item and accessible controls; the preflight opened/restored the complete TUI, and the public Quit walked off in five seconds before immediate restart.
  - [x] Complete focused Rust, universal-app, packaging, signing/notarization, and release gates on one exact v1.0.2 SHA.
    > Replacement candidate 29565557915 and same-SHA CI 29566294408 passed at 9643058 after the Windows PIPE_NOWAIT transient-zero fix.
  - [x] Publish v1.0.2 as stable/latest, fresh-download and verify its DMG/app, update from the installed v1.0.1 app, and confirm the website resolves the new DMG.
    > Atomic release 29566759574 and post-release smoke 29567257622 passed. Fresh public trust/hash, the real managed update, all three repeat no-ops, and thegoose.app's exact DMG path passed.
  - [x] Synchronize ADR 0028, readiness, README, guides, both changelogs, project memory, board, and Alienware handoff; push the final closure state.
- [x] **v1.0.1 first public stable release — native qualification, signed distribution, and DMG-first site** - preserved the immutable failed-before-draft v1.0.0 tag, fixed its pose-only Windows qualifier, published the first public stable major Developer ID-signed and notarized release, and promoted its DMG on the website (done 2026-07-15) #m20q
  - [x] Repair and visually verify the macOS renderer bridge.
  - [x] Fix dark-mode Mac note contrast and audit equivalent Windows/Linux presentation paths.
  - [x] Refine cross-platform planted-foot timing and cap visible leg stretch.
  - [x] Finish shared exposed-edge movement: natural monitor seams, occasional hidden wrap, non-wrapping errands, fully offscreen entry, and graceful animated exit.
    > Final-clear, gapped-monitor staging, permission-wait hot-plug, stop-transition, and bounded-deadline regressions pass independent review.
  - [x] Finish the user-only 30% note/meme-close annoyed reaction with separately gated bounded mouse nab.
    > Typed request correlation, lingering-window routing, Windows close provenance, onscreen deferral, and exact-once behavior pass independent review.
    > The per-tick beak-offset target and realistic 120 Hz grab/type regression pass. A product-equivalent signed app spawned and typed a readable native note; exact visual beak-contact capture is deferred verification, not claimed (waived 2026-07-15 — agent: codex).
  - [x] Pin an exact Linux binary and add compositor-visible X11/Wayland color evidence.
  - [x] Add exact-binary Windows layered-compositor and lifecycle evidence to CI, candidate, and published-MSI gates.
  - [x] Run the new exact-candidate compositor gates natively on Windows x64/ARM64 and Linux x64/ARM64.
    > Candidate 29401457634 and same-SHA CI 29401961540 attempt 2 passed at de8da8a9, including paired-DWM x64, hosted native ARM64 presenter proof, and all x64/ARM64 GNU/musl X11/Wayland paths.
    > v1.0.0 candidate 29392439475 and same-SHA CI 29392827146 passed every native compositor/release gate at 9c5692b after the private Sway background repair. Its later release failed only the side-only Windows pose oracle; v1.0.1 then repeated the full matrix with the strict side/top-down analyzer. No product background or ARM64 DWM claim is made.
    > Active diagnostic passed at 5.55% median CPU, 29.52 MiB max RSS, -9.89 MiB growth, zero leaks, and 20 clean captures. Exact-final-SHA repeat is source-equivalent-waived for stable publication and remains Alienware/Mac forward verification (waived 2026-07-15 — agent: codex).
  - [x] Implement the approved first-run Accessibility prompt and calm permission-wait behavior.
  - [x] Add a macOS-only menu-bar control with accessible Configure and graceful Quit actions.
    > Local packaged-app smoke exposed the Honk/Configure/Quit controls, launched the full existing TUI in Terminal, restored it cleanly, and completed animated Quit in four seconds. Exact-final-SHA repetition moves to #v1a; Windows and Linux remain unchanged.
  - [x] Complete denied and granted Accessibility qualification.
    > One unchanged signed executable passed first denied, non-nag denied relaunch, 102 ms same-process grant, and 127 ms same-process revoke. Exact-SHA/terminal-driver/Ghostty/one-display limitations are recorded `[~]` under closed #m16r.
  - [x] Verify CLI, TUI, IPC, and preliminary lifecycle behavior; repeat terminal/lifecycle checks on the exact candidate.
  - [x] Add the per-user graphical DMG installer.
  - [x] Quiesce the running managed app before shell update, mounted-DMG replacement, or uninstall; fail closed before mutation and verify release metadata before writing a receipt.
  - [x] Retain the lifecycle singleton through each mutation; remove ambient lease bypasses.
  - [x] Pin Windows payload identity from same-stream size/hash verification through execution.
  - [x] Roll Unix HUP/INT/TERM back with explicit nonzero status, including after-swap interruption.
  - [x] Check machine-wide Windows paths across sessions and reject reboot-deferred MSI state.
  - [x] Kill and reap every deferred-uninstall helper that fails before the exact READY handoff.
  - [x] Pin the exact G2 Developer ID leaf in CI and revalidate the extracted final app ZIP, stapling, Gatekeeper result, and required notarization evidence.
  - [x] Prove the rolling latest-update channel and immutable per-tag DMG behavior from every release trigger host.
    > Stable asset names and `latest/download/release-manifest.json` drive exact-tag, size/hash-pinned in-place updates. A Windows-triggered global release still uses GitHub's macOS runner for a fresh signed/notarized DMG; older tagged DMGs are never mutated.
  - [x] Add deterministic native amd64/arm64 Debian packages with real package-manager-owned aliases, platform-isolated CLI updates, preserve-on-uninstall, and backup-on-purge.
  - [x] Prove both Debian packages on native amd64/arm64 candidate and published-release hosts, including exact-vs-latest bytes, compositor output, all three update aliases, uninstall, and purge.
  - [x] Run the complete integrated local Rust, universal-Apple, packaging, workflow, audit, shell, cross-target, and diff gate on the Mac stopping commit.
    > Repeated after the menu, X11/Windows capture, status-pipe, and collect fixes: 432 Rust tests, 95 Python contracts, all seven goldens, strict native/cross clippy, release/Apple builds, dist plan, audit, actionlint, syntax, and diff checks pass.
    > Product-equivalent light/dark/note evidence plus automated eight-heading/secondary-motion/mud/prank/edge/reaction contracts pass. The complete exact live capture sheet moves to #v1a without a stronger claim (waived 2026-07-15 — agent: codex).
    > Exact final source passed aliases, grammar, status/help/version/broken-pipe and 80×24 TUI; product-equivalent native actions/audio passed. The manipulation matrix is tooling-waived under #m16r and moves to #v1a (waived 2026-07-15 — agent: codex).
    > Candidate-equivalent install/rollback/foreign-file/purge paths, the fresh published v0.3.2→v1.0.1 update, and final real-host cleanup pass. Only broader exact-release interaction/fault-injection repetition moves to verification card #v1a (waived 2026-07-15 — agent: codex).
  - [x] Add Developer ID signing, notarization, and packaging evidence.
    > Exact v1.0.0 candidate/release jobs produced and independently validated the G2-signed/notarized/stapled app and DMG before atomic publication failed elsewhere; v1.0.1 candidate/release freshly repeated and published the same fail-closed producer.
  - [x] Pass candidate and default-branch CI on the same exact SHA before creating the immutable tag.
  - [x] Cut the immutable v1.0.1 release and pass atomic publication plus post-release smoke tests.
    > Immutable v1.0.0 failed before draft creation on a top-down/side-only test mismatch and remains untouched under ADR 0027.
  - [x] Fresh-download the published app ZIP and DMG and independently recheck hashes, trust, install, and v0.3.2→v1.0.1 update.
  - [x] Finish and push the accessible progressive-disclosure site with stable latest DMG/Debian links, strict manifest checks, local/browser tests, hosted Windows/Linux CI, and a protected preview.
  - [x] Validate the website preview against the live v1.0.1 manifest, then promote and verify the DMG-first production deployment.
  - [x] Synchronize ADRs, README, both changelogs, readiness evidence, CODEX_PROJECT, guidance, task activity, and the Alienware handoff; commit and push the completed board.
- [x] **M16.1 — exact-candidate macOS host readiness** - one unchanged signed candidate completed first-denied, non-nagging relaunch, live grant/revocation, with explicit exact-final-SHA, terminal-driver, absent-Ghostty, and one-display waivers recorded in readiness evidence (done 2026-07-15) #m16r
  - [x] Re-check latest GitHub Actions state for newer macOS Accessibility evidence.
  - [x] Run the preliminary denied/granted smoke on one unchanged Developer ID-signed app on the physical M2.
  - [x] Record the preliminary Accessibility-granted evidence in `docs/readiness/m16-m18-readiness.md`.
  - [x] Verify or waive every `#m16r` verification item before moving the card to Done.
- [x] **Name interchangeability + site usage clarity** - verified `honk300`/`honk`/`goose` are fully interchangeable across every command (install aliases all three → same binary; `normalize_args` never branches on arg0; only internal `honk300 --version` self-call in update.rs). Locked in with `all_three_names_are_fully_interchangeable` test (13 CLI tests green). Site usage section reworked: consistent `honk300` display (no more honk300↔goose switching), natural order (start/stop → configure → install/autostart → pokes → flags), goose-speak kept as "Also:" notes, interchangeability banner (done 2026-07-08) #names
- [x] **v0.2.1 release — side-view neck refinement** - version bump + changelog cut (commit eb40260); tag `v0.2.1` drove cargo-dist Release → chained macOS Packaging + Windows Installers; wrap-up email sent (done 2026-07-08) #v021
  - [x] Release + macOS Packaging + Windows Installers workflows green; full 40-asset set verified (universal2 DMG + darwin tarballs + Windows MSI/EXE ×2 arch); v0.2.1 is latest, latest/download serves real bytes.
  - [x] Site `releaseFallback` → v0.2.1 (commit 3015992, site pushed).
  - [x] Wrap-up email sent to hey@emmetts.dev via Pipedream/SES us-east-1 (Message ID 0100019f426158d8-…); saved SES region to memory.
- [x] **R3 — macOS packaging + lifecycle** - apple triples in cargo-dist, universal2 .app + DMG CI job (version-stamped, unsigned personal-use), macOS install/uninstall/update, hands-on Mac checklist; ADR 0017 supersedes 0013 deferral (done 2026-07-08; commit 2980cbc; tag v0.2.0 cut) #r3m
  - [x] Apple triples added to `[workspace.metadata.dist].targets`; `dist generate --check` clean (matrix is data-driven, no release.yml edit).
  - [x] `macos-packaging.yml`: workflow_run-chained universal2 .app (lipo + ad-hoc codesign) + UDZO DMG + sha256 + `gh release upload`; `package_macos_app.sh` version-stamped via HONK300_VERSION.
  - [x] macOS `install`/`uninstall`: ~/Applications/Honk300.app, 3 CLI symlinks, LaunchAgent autostart, ADR 0015-style user-content preservation; `InstallSource::MacApp`.
  - [x] macOS `update`: DMG swap (hdiutil + ditto) for app installs; shell installer otherwise; sha256-verified.
  - [x] `docs/readiness/macos-handson-checklist.md` (feeds #m16r); ADR 0017; ADR indexes backfilled; changelogs lockstep.
  - [x] Gate green (16 suites, clippy, both apple cargo checks); v0.2.0 tagged — Release + installer workflows running.
  - [x] v0.2.0 workflow chain verified: Release + macOS Packaging + Windows Installers all green; universal2 DMG + sha256 + darwin tar.xz live on the release.
- [x] **R4 — website evaluation + improvements** - `C:\Users\hey\git\desktop-goose-site`; committed the live-release layer first, then full elevation; deployed live to https://thegoose.app (done 2026-07-08; commit d6ebbc1) #r4w
  - [x] Real-renderer site assets: `render_rig_scaled` + preview `frames` mode (commit d4d561a); 12-frame walk spritesheet + 4 large poses copied to site `public/assets/goose/`.
  - [x] R4.0 commit uncommitted `src/main.jsx` + `src/styles.css` live-release layer as-is (commit 6592a16).
  - [x] Split `createRoot` into entry module (restore Fast Refresh) (commit fdb5251).
  - [x] Usage table: install/uninstall/setup/reload + global flags; OG/SEO meta; honk300 favicon + OG image (commit 9c1be9c).
  - [x] Design refinement pass: typographic scale, spacing system, alignment, breakpoints, state consistency (commit 63acc51).
  - [x] R4b elevation (Fable hands-on): accurate hero from real walk frames, OS-aware download surfacing, Corporate-MSI-first Windows ordering, macOS column live (v0.2.0), micro-interactions, thegoose.app domain meta (commit d6ebbc1).
    > Done 2026-07-08 hands-on (spend limit blocked the delegated subagent). Hero now animates the real 12-frame sprite sheet; OS detect reorders columns + adapts hero CTA; Windows leads with per-user Corporate MSI then global; macOS live with universal2 DMG + apple-darwin tarballs; honk easter egg, scroll bar, magnetic CTA, typewriter terminal. `npm run build` clean, no console errors, verified in Chrome at desktop + 390px mobile.
  - [x] Vercel deploy verify: d6ebbc1 built and live at https://thegoose.app (walking-goose sprite + OS badge + v0.2.0 confirmed serving in prod).
- [x] **Native Wayland full-support research + integration path** - current upstream review found no portable full-parity client path; ADR 0021 defines a reduced portable base plus explicit portal/KDE/GNOME/wlroots capability adapters, with implementation split into follow-up cards (done 2026-07-12) #a6e
  - [x] Reconcile current plan and architecture assumptions.
  - [x] Research overlay, pointer, input, toplevel, move, portal, and compositor-extension paths.
  - [x] Publish the portable/wlroots/KDE/GNOME/portal/XWayland/privileged capability matrix.
  - [x] Define honest native support as capability strata rather than universal parity.
  - [x] Record ADR 0021 and the concrete adapter/testing/release path.
  - [x] Split portable/portal, KDE, and GNOME/wlroots implementations into follow-up cards.
- [x] **Runtime-loop dedup** - verified the shared `RuntimeCore` owns clock sampling, 120 Hz fixed-step sequencing, 60 Hz damage cadence, restart-required detection, and frame-order enforcement for all three platform runtimes; focused tests and both Windows cross-target checks pass (done 2026-07-12) #r5d
- [x] **Measure and optimize fullscreen overlay present cost** - M15/R5 already replaced the old fullscreen-redraw model with current/previous visual bounds, bounded dirty canvases, one layered window per monitor, and per-monitor clipping; 4K non-accumulation tests plus Windows x64/ARM64 checks pass (done 2026-07-12) #p4d
- [x] **Replace MSI placeholder with PolyForm + Great Honk Accord** - shipped the real license and non-binding goose appendix in every Windows MSI as stable/latest v0.3.2 (done 2026-07-11) #gla
  - [x] Add the combined RTF and regression contract.
  - [x] Wire Global and Corporate MSI manifests to the shared agreement.
  - [x] Update paired changelogs and v0.3.2 release/readiness metadata.
  - [x] Verify the repository gate plus x64/ARM64 MSI builds and UI rendering.
  - [x] Merge and push main, publish and verify v0.3.2, then clean temporary branches.
- [x] **R5 — v0.3.1 distribution-readiness stabilization** - fixed audited config, renderer, platform, lifecycle, release, and site defects; published only after the complete gate (done 2026-07-10) #r5s
  - [x] Remove the obsolete `DESKTOP-GOOSE/` tree and refresh active licensing/provenance/docs.
  - [x] Land config v2, safe setup/save/reload behavior, scoped CLI flags, and responsive nonblocking TUI control.
  - [x] Fix long-runtime timekeeping, monitor-region layout, damage tracking, renderer concept C, behavior regressions, and built-in asset loading.
  - [x] Harden Windows, macOS, X11, native Wayland reduced mode, audio, and local IPC behavior.
  - [x] Replace lifecycle/install/update paths and make release publication atomic with compatibility artifacts.
  - [x] Refresh `desktop-goose-site` install hierarchy, renderer assets, motion, responsiveness, accessibility, and release-data handling.
  - [x] Run the complete Rust, platform, lifecycle, site, artifact, and live-release verification gates.
- [x] **R1 — reliability + correctness fixes (all platforms)** - the ranked defect list from the 2026-07-07 Fable evaluation; gates R2's visual verification (DPI first). Plan: `~/.claude/plans/examine-this-goose-program-enchanted-charm.md` (done 2026-07-08; commits 90ad936 + b65424f) #r1f
  - [x] Windows PMv2 DPI awareness + `WM_DPICHANGED`/`WM_DISPLAYCHANGE` monitor rebuild.
  - [x] Non-blocking collect-window state machine + Notepad child kill on close/stop (fixes sim freeze, IPC TIMEOUT, zombie leak).
    > Typing is now best-effort (focus steal skips a note instead of latching collect off) — deliberate, in ADR 0015.
  - [x] Unix singleton flock liveness (replaces stale `create_new` lockfile).
    > rustix flock, kernel-released on crash; lock file intentionally never unlinked.
  - [x] macOS NSApp event-drain pump + rep-owned bitmap present (no external Vec aliasing).
  - [x] Linux degraded-overlay honesty: loud failure unless `HONK300_ALLOW_HEADLESS=1`; `overlay` capability in status protocol.
    > + X11 XFixes region caching, event mask set once. CI smoke uses real Xvfb/sway — env var NOT added there.
  - [x] Wire `behavior.attack_randomly` into the roam deck (default off, capability-gated; TUI toggle; commit b65424f).
  - [x] TUI rows for `mouse.grab_distance`/`drop_distance` (landed with R2 TUI work on r2-renderer).
  - [x] Non-purge `uninstall` relocates user memes/notes to `preserved-<ts>` with printed location.
  - [x] ADR 0015 written; CHANGELOG + HUMAN_CHANGELOG lockstep entries staged on r1-reliability.
  - [x] Local gate green on integrated branch (223 tests) + cross-target checks; Windows live smoke on r2 passed.
  - [x] Committed `90ad936` on r1-reliability with Emmett's go-ahead (2026-07-08); merging into r2 then main.
- [x] **R2 — Renderer V2 "Procedural Vector V2"** - supersedes the sprite-atlas direction (ADR 0001) per Emmett's 2026-07-07 approval; folds in old #c0d polish scope. Flat-illustration goose per Emmett's reference art (`docs/art-reference/`), dual-view rig (side + top-down, crossfade), stateful plant-and-swing feet, S-curve neck, 6-tone palette, secondary motion. Fable hands-on (done 2026-07-08; commits 6bd6c8e + 30aeb79) #r2v
  - [x] Copy reference SVGs into `docs/art-reference/` and extract part geometry.
    > Offline absolutizer transcribed the paths; no SVG dep in honk-engine.
  - [x] Stateful `FeetState` plant-and-swing gait (no foot slide) + invariant test.
  - [x] Dual-view rig state (side mirrored / top-down rotating) + crossfade with hysteresis.
    > 55°/45° hysteresis, 125ms fade, GoosePose union dirty rects.
  - [x] Part-builder renderer: wing layers, two-tone beak, shading, legs/webbed feet, tail.
    > 2x supersampled per-view layers composited with opacity.
  - [x] Damped mood posture, eased neck lerp, blink/breath/head-bob secondary motion.
    > Neck eases at 10/s; idle posture raised to 0.45 baseline; honk tail-flick.
  - [x] Palette 3→6 tones: config keys + `RenderPalette` + TUI RGB rows (back-compat).
    > New optional keys derive from legacy three; TUI materializes on first edit.
  - [x] Re-bless goldens + new top-down/walk-cycle frames; preview PNG review.
    > 6 goldens (side rest/reach/left/stride, top-down ×2); preview harness added.
  - [x] ADR 0014 written (supersedes ADR 0001 renderer direction; full decision log).
  - [x] Neck seam fix per Emmett's live review: body+neck+head as one outline-then-fill mass; goldens re-blessed.
  - [x] Neck refinement pass 2 (Emmett 2026-07-08): removed the backward bow at `neck_c1` (near-straight forward lean per reference) and buried `neck_base` (ref y80→y92, width 30→32) so no seam at the shoulders; goldens re-blessed; site frames regenerated.
  - [x] Meander walking (ADR 0016): rng-driven lateral wobble on wander/excursion paths, fades near targets.
  - [x] Story-driven mud (ADR 0016): wander no longer muds; off-screen puddle hops (8-15s away) return tracking mud 30-90s.
  - [x] Off-screen errands (ADR 0016): every 4-7min, away 90-120s, horizontal-edge preference, 40% return with a collect prank.
  - [x] `exit`/`quit` stop synonyms in goose-speak grammar; all stop commands verified live.
  - [x] Live Windows smoke ×2 with screenshots: dual-view transition, serpentine mud trail, clean stop, no notepad zombies.
  - [x] Committed `6bd6c8e` on r2-renderer with Emmett's go-ahead (2026-07-08).
  - [x] Merged r1-reliability in; changelogs + honk300_plan §5.2 + README/AGENTS/CLAUDE sync; final gate; merged + pushed to main (30aeb79, b65424f).
- [x] **M19 — packaging (all OS/arch) + install/update/uninstall** - cargo-dist + windows-installers.yml; 3 name aliases; autostart (done 2026-07-07) #a8d
  - [x] Investigate and write the M19 resolution plan.
    > See `docs/thinking/2026-07-06-active-task-resolution-plan.md` and `.tasks/tasks/a8d.md`.
  - [x] Reconcile packaging contract across `honk300_plan.md`, ADRs, current CLI placeholders, and TR300/ND300 sibling patterns.
    > ADR 0013 records the Windows/Linux-first M19 continuation and deferred macOS distribution slice.
  - [x] Implement lifecycle commands: install, uninstall, update, setup, `--purge`, and install-source detection.
  - [x] Add cargo-dist metadata, release workflow, and fresh WiX GUIDs without crates.io publishing.
  - [x] Add Windows Global/Corporate MSI and EXE installers for x64 and ARM64, with aliases, shortcuts, autostart, and sha256 sidecars.
  - [x] Defer macOS universal2 `.app`/DMG/signing/notarization work until the macOS slice resumes, defaulting later artifacts to unsigned personal use.
  - [x] Add Linux desktop/autostart installation across GNU/musl x64/ARM targets.
  - [x] Run local gate, package/release dry-runs, platform smoke checks, and record artifact evidence before closing.
    > Local Rust gate, `dist plan`, local x64 WiX/Inno artifact inspection, release workflow evidence, Windows installer workflow evidence, and release asset lists are recorded in `.tasks/tasks/a8d.md`.
- [x] **M18.1 — native Wayland reduced-mode readiness** - CI-smoked visible layer-shell reduced mode with IPC stop/poke/reload/status and explicit unsupported mischief evidence on Linux x64/ARM (done 2026-07-02) #m18r
- [x] **M17.1 — Linux X11 visible backend readiness** - CI-smoked the visible transparent X11/XWayland overlay, input shaping/click-through, pointer/window support, terminal filtering, and Linux x64/ARM evidence (done 2026-07-02) #m17r
- [x] **M16–M18 — macOS / Linux X11 / Wayland backends** - in-tree macOS backend plus Linux control-runtime foundation, status/TUI capability reporting, scripts/readiness handoff, ADRs, docs/changelogs, and full Windows-host gate/cross-target checks complete; host GUI smoke split to #m16r/#m17r/#m18r (done 2026-07-01) #f7c
  - [x] Land M16 macOS backend/runtime/status/.app-staging implementation in-tree.
  - [x] Split M16.1 macOS-host readiness smoke to follow-up #m16r with repeatable `script/smoke_m16_macos.sh`.
  - [x] Land M17/M18 Linux control-runtime foundation: Linux `start`, Unix IPC status/reload/stop/poke, X11-first/`--wayland` detection, terminal classifier, local-time sampling, command-player audio, and honest unsupported/failed capability reporting.
  - [x] Split M17.1 Linux X11 visible backend readiness to follow-up #m17r with host-smoke requirements.
  - [x] Split M18.1 native Wayland reduced-mode readiness to follow-up #m18r with repeatable `script/smoke_m17_m18_linux.sh`.
  - [x] Added readiness pass record in `docs/readiness/m16-m18-readiness.md`.
  - [x] Confirmed `honk-engine` stays OS-free and architecture-neutral after each backend is added.
  - [x] Confirmed backend support is reported through capabilities, not compile-time assumptions inside engine tasks.
  - [x] Confirmed each implemented backend has terminal-window detection/filtering before emitting foreign-window, collect-window, or spicy-behavior targets.
  - [x] Confirmed degraded paths are explicit and user-visible for macOS permission denial, X11 unavailability, and native Wayland unsupported mischief.
  - [x] Confirmed install/package target implications are captured for Intel/Apple Silicon, x64/ARM GNU, and x64/ARM musl where applicable.
- [x] **M15 — multi-monitor chase + full recolor/appearance** - Windows signed virtual-desktop world bounds, one layered overlay HWND per monitor, dirty-region present/crop fan-out, live Calm Goose, RGB channel palette editing, ADR 0009, docs/changelogs, and local gate complete (done 2026-07-01) #e6b
  - [x] Added M15 monitor/appearance ADR and startup bounds contract.
  - [x] Implemented per-monitor Windows overlay windows and dirty-region render/present.
  - [x] Wired multi-monitor chase and Calm Goose config into runtime/world options.
  - [x] Replaced brightness-only TUI color rows with RGB channel editing.
  - [x] Updated README, AGENTS, CLAUDE, changelogs, and task details.
- [x] **M14 — schedule (quiet hours / DND-fullscreen) + seasonal (Autumn)** - platform-free schedule/presence gates, Calm Suppression manners, Windows DND/fullscreen polling, built-in procedural Autumn piles/leaves, config/TUI plumbing, ADR 0008, and local gate complete (done 2026-06-29) #d5a
  - [x] Enforced quiet-hours config in the engine/runtime without making `honk-engine` sample host time directly.
  - [x] Added Windows DND/fullscreen presence detection with an explicit platform-neutral input to the engine.
  - [x] Added built-in seasonal Autumn behavior/assets within the existing asset/IP guardrails.
  - [x] Updated ADRs, README, AGENTS, CLAUDE, changelogs, and task details when the schedule/season contract was accepted.
- [x] **M13 — dynamic moods + on-hour double honk** - platform-free mood machine, conservative default intensity, sleepy Z particles, mood-biased enabled tasks only, runtime-injected local time, honk tone mapping, config/TUI plumbing, ADR 0007, full local gate, and Windows smoke complete (done 2026-06-28) #c4f
  - [x] Added `honk-engine::mood` with deterministic weighted transitions and `MoodIntensity::{calm,normal,spicy}` mapping.
  - [x] Implemented mood effects as post-task modulation: sad/sleepy slow movement and lower posture, sleepy renders Z particles, hyper can request `HyperTask`, mischievous only duplicates already-enabled nab/collect factories.
  - [x] Added `World::set_local_time(LocalTime)` and on-hour double honk emission once per local hour.
  - [x] Added `HonkTone` to sound requests and mapped normal/high/low honks in the audio backend.
  - [x] Threaded Windows local-time sampling through the runtime without adding OS dependencies to `honk-engine`.
  - [x] Added ADR 0007 plus README/AGENTS/CLAUDE/changelog/task updates.
- [x] **M11/M12 hot-apply review fixes and M12R polish** - reviewer pass found the reload path collapsed backend-capability vs user-preference; the four contract bugs landed on `main` (`c8b63e1`, pushed 2026-06-28), and the remaining config/TUI polish is now complete (done 2026-06-28). See `m12r.md` #m12r
  - [x] **MAJOR — FIXED** Re-enabling mouse-steal via reload was a silent no-op: `cursor_warp_supported` was seeded from the `no_mouse_steal` preference and never reset true. Now a pure platform capability (`initial_cursor_warp_supported()` → `true` on Windows), degrading only on a real warp failure; the preference rides `MouseStealOptions::enabled`. Verified end-to-end (start steal-off → `do nab` UNSUPPORTED; flip config + reload → `do nab` accepted). ADR 0006.
  - [x] **MAJOR — FIXED** Collect-window backend-failure was resurrected on reload. `BackendState` gained `collect_window_supported`, threaded through `effective_options`, latched false in the collect-failure branch, so the loss is durable across reloads. Engine/config test `backend_collect_loss_disables_collect_window`. ADR 0006.
  - [x] **MINOR — FIXED** Poke `Busy`/`Unsupported` outcomes were dropped (IPC answered `Ok` at enqueue). `honk-control` now does a bounded request/response round-trip (`ControlRequest` + 2 s wait + `PokeOutcome`→`ControlResponse`); `reload` reports `RELOAD_REJECTED` on failure. CLI/TUI "rejected: {code}" now fires. ADR 0005.
  - [x] **EXTRA — FIXED** (found in this pass) Disabling `interaction.pat_streak` also disabled the click reaction; decoupled so clicking still triggers hyper/nab with pats off. Engine test `clicking_the_goose_triggers_hyper_even_with_pat_streak_off`. ADR 0006.
  - [x] Added ADR 0005 + ADR 0006 (closing the missing-ADR process gap) and updated `CHANGELOG.md` + `HUMAN_CHANGELOG.md` in lockstep.
  - [x] Corrected by construction: capability states now genuinely stay distinguishable after reload, so the M12 Done readiness note holds.
  - [x] **MINOR — FIXED** `[speeds]`/`[mud]`/`[colors]` now validate, map into `WorldOptions`, hot-apply to movement/footmark/render behavior, and surface in the TUI.
  - [x] **MINOR — FIXED** Unknown TOML keys now warn once while preserving unknowns on save; quiet start/end rows edit in 15-minute increments.
  - [x] **MINOR — FIXED** TUI `Start` now launches `start --config <path>` with null stdio and Windows detached process flags.
  - [x] **NIT — FIXED** TUI now uses a current-thread Tokio `select!` loop, reducer-owned command results, row-model counts, scroll support, and two-step dirty quit confirmation.
- [x] **M12 — config TUI (ratatui reducer; start/stop/config UI; Poke panel; TOML I/O; hot-apply via M10 IPC)** - config/reload readiness check complete (done 2026-06-28) #b3e
  - [x] Added an M12.1-style readiness pass before moving M12 to Done.
  - [x] Confirmed config schema is versioned, tolerant of unknown keys, and has stable defaults for all M0-M12 behavior.
  - [x] Confirmed all hot-applied fields map to engine options or command enums without leaking platform file paths, handles, or OS APIs into `honk-engine`.
  - [x] Confirmed reload is atomic: parse and validate outside the running engine, then apply a complete option set through the existing reload path.
  - [x] Confirmed platform capability display distinguishes user-enabled settings from backend unsupported or permission-denied states.
  - [x] Persisted future settings as planned/restart-required rather than implementing later milestones early.
- [x] **M11 — CLI grammar (3 names + goose-speak) + help** - normalized `honk300`/`honk`/`goose` grammar, explicit `do <action>` pokes, config/lifecycle/help coverage (done 2026-06-28) #a2d
  - [x] Added deterministic pre-clap normalization for executable stems and fixed goose-speak phrases.
  - [x] Mapped `plz` to start for all three names; mapped `bad`, `no`, and `no honk` to stop.
  - [x] Kept pokes explicit through `do <honk|wander|mud|meme|note|nab>`.
  - [x] Added `config`, `--config <path>`, `--wayland`, lifecycle placeholders, help, version, and coverage for invalid phrases.
- [x] **M10 — single-instance + IPC command channel (start/stop/do/reload), no tray, no global quit key** - local CLI/TUI-only control plane plus terminal-window protection readiness (done 2026-06-27) #f1c
  - [x] Added an M10.1-style readiness pass before moving M10 to Done.
  - [x] Confirmed IPC transport is local-only, same-user scoped, and not exposed on the network.
  - [x] Confirmed command payloads use structured enums rather than free-form strings after CLI normalization.
  - [x] Confirmed Windows named-pipe, macOS Unix-socket, and Linux Unix-socket readiness share the same engine-facing command model.
  - [x] Removed non-IPC stop semantics from the roadmap: no system tray, no global quit key, CLI/TUI control only.
  - [x] Added terminal-window protected filtering and regression coverage readiness for foreign-window ride, collect-window, and future spicy behavior paths.
  - [x] Added ADR 0004 and updated README, AGENTS, CLAUDE, canonical plan, task details, and both changelogs.
- [x] **M9 — collect-window dispatcher (notepad / meme)** - real Notepad typing plus meme windows; asset policy resolved as screened originals plus one complete custom counterpart each; user-supplied Meme8 included; donate removed as an old-developer artifact (done 2026-06-27) #e9b
  - [x] Added ADR 0003 and reconciled docs/changelogs/task memory around the M9 asset and no-donate decisions.
  - [x] Added note and meme assets under provenance-separated originals/custom/user directories.
  - [x] Added platform-free collect-window command/snapshot contract to `honk-engine`.
  - [x] Implemented `CollectWindowTask` with spawn, pickup, drag-back, release/type, dwell, and cleanup states.
  - [x] Added Windows controlled-window movement, pass-through toggling, owned image windows, Notepad spawn, focus verification, and Unicode keystroke synthesis.
  - [x] Wired runtime asset loading, command draining, snapshot feeding, and `HONK300_SMOKE_COLLECT=note|meme`.
  - [x] Added engine regression coverage for note/meme collection, missing/unsupported/capability-loss paths, ordered commands, and suppression.
  - [x] Full gate, installed target checks, and visual smoke evidence recorded before final closure.
- [x] **M7/M8 audit + M9 enrichment (reviewer pass)** - audited M7/M8 against ADR 0001/0002 (both PASS; M8's clean visual-ride proof is the one open caveat, folded into M9 smoke) and enriched the M9 plan: stripped all donate references, appended the Reviewer Addendum (verdict + 10 code-grounded enrichments + resolved copy-1:1-plus-one-complete-custom-counterpart asset policy + base prompt + no-old-dev guardrail), modified #b9e for house image-gen style, refreshed stale `.tasks/CLAUDE.md`, and approved user-supplied `Meme8.png` for the `Assets/Images/Memes/user/` provenance bucket (done 2026-06-27) #r7a
- [x] **M8 — foreign-window drag + perch & ride** - ride a dragged window; includes engine/platform readiness check for window capabilities (done 2026-06-26) #c8a
  - [x] Add an M8.1-style readiness pass before moving M8 to Done.
  - [x] Confirm the engine uses neutral concepts such as `ForeignWindowId`, capability tokens, or intent commands, not raw platform handles.
  - [x] Confirm window movement/watch support is capability-gated across Windows, macOS, X11, and native Wayland no-op/degraded mode.
  - [x] Confirm negative monitor coordinates, mixed DPI, and future per-monitor overlays are not ruled out by the engine API.
  - [x] Add follow-up tasks for any platform or performance gaps discovered during the M8 readiness pass.
- [x] **M7 — cursor mischief (warp + nab sub-states)** - clicking the goose now starts NAB when cursor warp is supported; the goose bites the cursor, runs a bounded HYPR-style burst while holding it, and releases safely; M7.0/M7.1/M7.2 review work and visual acceptance completed (done 2026-06-26) #b7f
  - [x] Inspect the current task and world APIs, especially how `HyperTask` interrupts and restores work from M6.
  - [x] Add a platform-free cursor-mischief task/state model in `honk-engine`.
  - [x] Add the Windows cursor-warp capability while keeping unsupported-platform denial explicit.
  - [x] Wire cursor nabbing into click/roam behavior only when mouse stealing is enabled and cursor warp is supported.
  - [x] Add focused engine/world regression coverage for cursor nab transitions, disabled/unsupported paths, retargeting, timeout/drop, command drain, and M6 suppression during nab.
  - [x] Run the full local gate and a non-invasive startup smoke.
  - [x] Update `CHANGELOG.md` and `HUMAN_CHANGELOG.md` together for M7.
  - [x] M7.0 completed-milestone audit — re-checked M0-M6 against `honk300_plan.md`, fixed stale repo/module status docs, and added follow-up `#p4d` for fullscreen overlay present-cost measurement/optimization.
  - [x] M7.1 honk-engine cross-platform readiness pass — confirmed `honk-engine` remains platform-free and checked Windows, Linux, and macOS target coverage through rustup.
  - [x] M7.2 renderer/runtime architecture spike — compared sprite atlas, `tiny-skia`/`resvg`, Vello/wgpu, Skia, Macroquad/ggez/Bevy; selected a custom CPU sprite/atlas blitter as the future efficient/customizable cross-platform renderer while keeping platform backends on plain premultiplied pixels.
  - [x] Visually confirm default cursor-drag behavior and goose visual polish in the running app before moving M7 to Done.
  - [x] Split the accepted Renderer V2 sprite-atlas implementation into follow-up backlog task `#r2v`, including atlas metadata, anchor/hit-mask format, premultiplied blit path, and platform-present integration.
- [x] **M6 — hit-testing: pat (hover-streak + hearts) + click→hyper** - cursor hover-sweeps over the goose build a streak → rising heart particles + brief calm (suppresses spontaneous honks); a left-click sends it into a charge-speed hyper burst that resumes the interrupted task; new engine modules `hearts` + `interaction`, `HyperTask` save/restore interrupt, Windows cursor polling; visually verified (done 2026-06-25) #d6e
- [x] **M5 — audio** - rodio backend; honks / bite / mud-squish / pat; SilenceSounds (`--no-sound`); original sounds bundled 1:1 (committed); silent no-op with no audio device (done 2026-06-25) #a5c
- [x] **M4 — task state machine + wander + FirstUX** - Task trait (extension seam) + Deck-picked roaming + WanderTask + scripted FirstUxTask entrance; replaced the M2 roam stand-in (done 2026-06-25) #f4a
- [x] **M3 — footmarks + mud trail** - fullscreen primary-monitor overlay; fading prints (8.5 s / 1 s); procedural goose renderer established with golden-frame coverage (done 2026-06-25) #e3f
- [x] **M1+M2 — Windows overlay + walking goose** - transparent click-through layered overlay (UpdateLayeredWindow) + 120 Hz loop + clean-room locomotion + procedural-feet gait (done 2026-06-24) #c1d
- [x] **M0 — platform-free engine core** - honk-engine (math/time/Deck/entity/rig/feet/footmarks/render) + 29 tests + golden frames; committed 21a95b9 (done 2026-06-25) #b0a
