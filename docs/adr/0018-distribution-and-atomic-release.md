# ADR 0018 — Distribution and Atomic Release

- Status: Accepted (2026-07-10)
- Superseded in part: ADR 0020 replaces only this ADR's macOS primary-install, ad-hoc-signing,
  hidden-DMG, and macOS artifact-verification decisions. Atomic publication and all non-macOS
  decisions below remain accepted.
- Supersedes: ADR 0013's release mutation workflow and ADR 0017's advertised DMG,
  bundle-synthesis, and DMG-update decisions. ADR 0017 remains the historical record for the
  first macOS packaging slice.
- Relates to: ADR 0010 (stable macOS bundle identity), ADR 0015 (lifecycle safety), ADR 0019
  (runtime stabilization contracts).

## Context

The v0.2.x release was assembled by independent workflows that published first and appended
platform installers later. That left a race between `latest`, the updater, the website, and an
incomplete asset set. It also advertised multiple equally prominent install paths even though
only one path per platform is supportable as the default. macOS additionally treated a DMG as
both delivery and update machinery, which encouraged mutation or synthesis of an app bundle
whose Accessibility identity depends on a stable, sealed bundle.

v0.3.x needs one distribution contract that is simple for users, verifiable by machines, and
transactional when an existing installation is replaced.

Implementation note (2026-07-10): the immutable `v0.3.0` tag failed during artifact production
before a draft existed. Applying this ADR's fix-forward rule, the corrected stable target is
`v0.3.1`; the failed tag is not moved or rebuilt.

## Decision

> Current macOS distribution policy lives in ADR 0020. The macOS bullets below are retained as
> the v0.3.1 historical decision; the atomic release contract remains current.

### Primary install paths

- Windows uses the x64 or ARM64 **Global MSI**. It is a per-machine package under Program Files
  with machine PATH, all-users Start Menu, HKLM, Add/Remove Programs, repair, rollback, upgrade,
  and uninstall ownership. Corporate MSI/EXE and portable archives remain compatibility or
  administrator choices, not recommendations.
- macOS and Linux use the version-stamped `honk300-installer.sh` bootstrap. The public command
  downloads only that bootstrap through `latest/download`; the rendered script then downloads
  its payload from its embedded exact stable tag and verifies an embedded SHA-256.
- macOS installs a prebuilt universal2 `~/Applications/Honk300.app`. The installer verifies
  x86_64 and arm64 slices, bundle id, version, and strict deep code-signing validity and never
  mutates the sealed bundle.
- Linux installs a managed payload under `${XDG_DATA_HOME:-~/.local/share}/honk300/install`.
  Both Unix platforms create only owned aliases/integrations under the current user's home and
  require no `sudo`. Autostart is off by default.
- The universal DMG remains in stable releases only because v0.2.1 updaters hard-coded its
  filename. It is not advertised. The artifact is ad-hoc signed and not notarized; terminal
  delivery does not replace Developer ID/notarization or guarantee Accessibility-grant
  persistence.

### Release and receipt contracts

`release-manifest.json` has schema `honk300.release.v1`. It records the exact stable version,
tag, full commit SHA, layouts, and every artifact's safe filename, target, kind, size, and
SHA-256. Install receipts use `honk300.install.v1` and record the same identity plus install
root, aliases, layout, channel, and autostart ownership. Consumers reject unknown schemas,
unstable tags, malformed commit hashes, unsafe/duplicate names, missing artifacts, or hash/size
mismatches.

The updater reads `latest/download/release-manifest.json` only to discover the immutable stable
tag, then fetches the manifest and payload from that exact tag. There is no tags-API fallback
and no mixture of `latest` payload URLs with a separately discovered version.

### Transactional Unix installation

The rendered bootstrap detects Darwin/Linux, x64/ARM64, and GNU/musl (with an explicit
`HONK300_LIBC` escape hatch for ambiguous systems). Before extraction it rejects absolute and
traversing paths, duplicates, links, and malformed archives. It stages on the destination
filesystem, verifies the exact executable, stops the current runtime with a bound, swaps the
managed root atomically, updates only owned aliases/profile markers/desktop entries/receipts,
then verifies again. Any failure restores the previous payload and every integration changed by
the attempt. Foreign files, aliases, app bundles, receipts, and desktop entries are not
overwritten.

Mutable notes and memes live in the platform user-media directory, never in Program Files,
installer-owned binary trees, or `Honk300.app`. Legacy user content is migrated conservatively.

### Atomic publication

Before a tag is created, `release.yml` candidate mode builds every portable target and both
platform packaging matrices from an explicit full commit SHA. It requires the proposed tag to be
absent, assembles the complete artifact set, validates installers/checksums/rollback, and has no
publication steps. This proves the producer graph without consuming an immutable release identity.

After that gate passes, one release-mode invocation rebuilds the same graph from the tagged SHA.
It flattens producer outputs while rejecting duplicate filenames, renders the bootstraps, validates
the complete stable plus v0.2.1-compatibility set, generates the public manifest and per-file
checksums, runs installer smoke and rollback injection, and then:

1. refuses a pre-existing release;
2. creates one unpublished draft;
3. uploads the complete asset set once, without `--clobber`;
4. compares remote filenames and bytes with the local verified set;
5. publishes it as stable/latest.

A failed unpublished attempt removes only the draft created by that run. Published tags and
releases are immutable; corrections use a new patch version.

## Consequences

- Users see one recommended install path per platform while compatibility artifacts remain
  available.
- The updater and website cannot observe a partially assembled public release.
- MSI owns privileged Windows lifecycle changes; the application does not delete running
  MSI-owned files itself.
- A universal ad-hoc-signed macOS app is a real application bundle but still requires honest
  Gatekeeper/Privacy & Security guidance. Notarization remains future operational work, not an
  implied property of the shell command.
- The project owns its bootstrap templates; cargo-dist produces portable archives and its plan,
  with `installers = []` preventing competing generated public installers.

## Verification

- Python contract suites validate metadata classification, required assets, exact-tag template
  rendering, archive defenses, installer ownership, producer prerequisites, and workflow atomicity.
- The release workflow installs the Linux payload twice and injects a post-swap failure to prove
  rollback before any publication step. Candidate mode runs this complete gate before the tag
  exists and refuses to publish.
- Windows CI installs v0.2.1, upgrades to the candidate x64 Global MSI, repairs it, refuses a
  downgrade, and uninstalls it; it also administratively extracts the ARM64 MSI.
- macOS CI validates the sealed app's slices, bundle id, version, `ditto` archive, and
  `codesign --verify --deep --strict` result.
- `dist plan --tag=v0.3.1`, checksum validation, and remote byte comparison are release gates.
