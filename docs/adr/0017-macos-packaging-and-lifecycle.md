# ADR 0017 — macOS Packaging and Lifecycle (R3)

- Status: Accepted (2026-07-08)
- Supersedes: the deferred-macOS-distribution slice of ADR 0013 (§Decision "Leave macOS
  DMG/signing/notarization out of this continuation pass") and amends its "unsigned
  personal-use" default into a concrete pipeline. ADR 0013's Windows/Linux lifecycle contract
  is otherwise unchanged.
- Relates to: ADR 0010 (macOS agent bundle, LSUIElement identity), ADR 0012 (CI-proven backend
  readiness), ADR 0015 §7 (user-content-preserving uninstall).
- Owner: Fable (orchestrator), implemented in the R3 worktree.

## Context

ADR 0013 shipped Windows and Linux packaging first and explicitly deferred the macOS
`.app`/DMG, signing/notarization, and the arch-matched macOS update path until a later slice.
That slice is R3. The scope is locked: **x64 + ARM64 only**, **unsigned personal-use**
artifacts (no Developer ID signing, no notarization), and **CI does all macOS verification** —
there is no Mac on the build host, so hands-on Mac steps become a documented checklist rather
than a code gate. The goose already had an in-tree macOS backend (M16), a bundle-aware asset
loader, and `script/package_macos_app.sh` staging; what was missing was the release matrix, a
DMG, and the `install`/`uninstall`/`update` lifecycle for macOS.

## Decisions

### 1. cargo-dist gains the two Apple targets

`[workspace.metadata.dist].targets` adds `x86_64-apple-darwin` and `aarch64-apple-darwin`. The
Release workflow (`release.yml`) is data-driven — its build matrix comes from
`fromJson(...ci.github.artifacts_matrix)` — so adding targets needs no workflow edit
(`dist generate --check` stays green). cargo-dist now emits `honk300-<arch>-apple-darwin.tar.xz`
per-arch archives (bare binary + README/CHANGELOG) with `.sha256` sidecars, and its `shell`
installer auto-detects Darwin and pulls the right archive. These per-arch tarballs are the macOS
parity path for the shell installer and for bare/symlink `update` — deliberately **not**
duplicated with hand-rolled app tarballs (one source of truth per artifact, already checksummed
by cargo-dist).

### 2. A hand-authored `macos-packaging.yml` builds the universal2 `.app` + DMG

Mirroring `windows-installers.yml`'s shape: `workflow_run` on the Release workflow's completion
(gated on a `v`-prefixed head branch) plus `workflow_dispatch` with a `tag` input; resolve the
tag → version, verify the upstream cargo-dist release is complete (`dist-manifest.json` and both
darwin tar.xz present), then check out the tag. On `macos-15`:

- install both Rust targets and run `script/package_macos_app.sh`, which builds both arches,
  `lipo -create`s a universal2 binary into `Honk300.app/Contents/MacOS/honk300`, copies `Assets`
  into `Contents/Resources`, writes the LSUIElement `Info.plist`, and ad-hoc codesigns the
  bundle (`codesign --force --deep --sign -`);
- `hdiutil create -format UDZO` a compressed `honk300-universal2.dmg` containing the `.app` and a
  `/Applications` symlink;
- `shasum -a 256` sidecar for the DMG;
- `gh release upload --clobber` the DMG + sidecar onto the tag.

Per-arch bare binaries stay with cargo-dist's tar.xz; the DMG is the universal2 app deliverable.

### 3. `package_macos_app.sh` stamps the real version

The script previously hardcoded `CFBundleShortVersionString 0.0.0` / `CFBundleVersion 0`. It now
reads `HONK300_VERSION` (the workflow passes the resolved tag without its leading `v`; default
`0.0.0` for local staging) and stamps both keys.

### 4. Ad-hoc signing only (no Developer ID / notarization)

The bundle is ad-hoc signed (`-s -`) so Gatekeeper's quarantine behavior is at least consistent
across machines; the DMG itself is left unsigned (UDZO is not a signable Mach-O and ad-hoc DMG
signing buys nothing). First launch is therefore quarantined: the documented first-run
instruction is **right-click the app in Finder → Open** once to approve it. Developer ID signing
and notarization remain out of scope and can be added later without changing this contract.

### 5. macOS `install` / `uninstall` mirror the Linux lifecycle

`install` stages `~/Applications/Honk300.app` — copying the whole bundle when run from a staged
`.app` (DMG/app), or synthesizing the same layout (Info.plist + `Contents/MacOS/honk300` +
`Contents/Resources/Assets`, version from the running build) when run as a bare binary (shared
`copy_assets_into` staging). It symlinks `honk300`/`honk`/`goose` into `~/.local/bin` pointing at
`Contents/MacOS/honk300`, writes a `mac-app` install-source marker into `Contents/Resources`, and
— only with `--autostart` — writes a `RunAtLoad` LaunchAgent at
`~/Library/LaunchAgents/dev.emmetts.honk300.plist` whose program is the bundle binary with the
`start` argument. `uninstall` removes the symlinks, the LaunchAgent, and the `.app`, and
**preserves user memes/notes** exactly like ADR 0015 §7 (plain uninstall relocates the
user-provenance dirs to a timestamped `preserved-<ts>`; `--purge` backs them up then also removes
config/state at `~/Library/Application Support/honk300`, matching honk-config's macOS root). A new
`InstallSource::MacApp` variant carries this provenance.

### 6. macOS `update` splits on install provenance

`select_update_plan` for a macOS target: `MacApp` → download+verify `honk300-universal2.dmg`,
`hdiutil attach` read-only, `ditto` the bundle over `~/Applications/Honk300.app`, detach;
everything else (shell/cargo-home/bare, detected as non-`MacApp`) → re-run the cargo-dist
`honk300-installer.sh`, exactly like Linux. The DMG download reuses the existing
`.sha256`-sidecar verification and checksum-mismatch refusal; there is no `cargo install` path.

### 7. Hands-on Mac steps are a documented checklist, not a code gate

`docs/readiness/macos-handson-checklist.md` records the manual Mac procedure (install from DMG,
first-run right-click-Open, grant Accessibility, run `script/smoke_m16_macos_accessibility.sh`,
visually verify the V2 goose across all directions/nab/collect, record evidence in
`docs/readiness/m16-m18-readiness.md`). It feeds board task `#m16r`; that card stays open until a
pre-granted Mac or self-hosted runner supplies the Accessibility-granted evidence.

## Consequences

- macOS joins the release matrix: every tagged release carries darwin per-arch tar.xz (+ sidecars)
  from cargo-dist and, after the chained macOS workflow, a universal2 `honk300-universal2.dmg`
  (+ sidecar).
- Install-source provenance now has a macOS value; the `update` strategy selection is a lockstep
  contract: `mac-app` → DMG replacement, any other macOS provenance → shell installer.
- First run is quarantined until the user right-click-Opens the unsigned app once; this is
  expected and documented, not a bug.
- The macOS lifecycle can be exercised for structure/compile in CI cross-checks, but real install
  → Accessibility → visual verification still requires the hands-on checklist; `#m16r` remains the
  open evidence gate.

## Verification

- Local Rust gate green on the R3 worktree: `cargo fmt --all -- --check`,
  `cargo clippy --all-targets --workspace -- -D warnings`, `cargo test --workspace` (macOS
  install-source marker, `.app`-ancestor detection, and DMG-vs-shell update-plan tests included),
  and `cargo build --release`.
- Cross-target compile checks: `cargo check --workspace --target x86_64-apple-darwin` and
  `--target aarch64-apple-darwin` (the new macOS `install`/`uninstall`/`update` code compiles
  clean).
- `dist plan` lists both `honk300-{x86_64,aarch64}-apple-darwin.tar.xz` (+ sidecars) and the
  Darwin-capable shell installer; `dist generate --check` confirms `release.yml` needs no edit.
- Both workflow YAMLs parse; `package_macos_app.sh` passes `bash -n`.
- CI-only (cannot run on the Windows build host): the `macos-packaging.yml` dry-run
  (`workflow_dispatch` against an existing tag) must show `lipo -verify_arch x86_64 arm64`,
  version-stamped `CFBundleShortVersionString`, the compressed DMG, its `.sha256`, and the
  `gh release upload`; and a hands-on Mac must complete the checklist for `#m16r`.
