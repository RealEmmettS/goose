# ADR 0023 — Rolling Latest Artifacts And Debian Package Lifecycle

- Status: Accepted (2026-07-14)
- Extends: ADR 0018 (atomic immutable releases), ADR 0020 (Developer ID macOS distribution),
  and ADR 0013 (shared lifecycle ownership).
- Relates to: ADR 0015 (fail-closed lifecycle safety) and ADR 0022 (stable macOS permission
  identity).

## Context

Honk300 is released from GitHub Actions, not from the operator's current workstation. A normal
release may be initiated from Windows, macOS, or Linux, but users still need one complete
cross-platform release: Windows installers, Linux archives, native Debian packages, and a newly
signed/notarized/stapled macOS app and DMG. The machine that pushes the tag must not determine
which platforms receive current artifacts.

Public install links should remain useful across future releases. At the same time, an installed
copy must never trust a mutable URL as proof of what it is about to execute, and publishing a new
release must never rewrite an older tag's bytes. Debian/Ubuntu users also need a conventional
package that owns its system paths rather than a renamed archive that bypasses `dpkg`.

## Decision

### Complete releases and stable public names

- Every stable `vMAJOR.MINOR.PATCH` release runs the complete GitHub matrix. GitHub-hosted macOS
  runners build the universal app and graphical helper, Developer ID-sign them, notarize and
  staple the app and DMG, and return those artifacts to the one atomic release assembly job.
  This is required even when the operator initiates the release from Windows or Linux.
- Public installer filenames are stable and unversioned. In particular:
  `honk300-universal2.dmg`, `honk300-amd64.deb`, `honk300-arm64.deb`, the two Global MSI names,
  and both bootstrap names are reachable through `/releases/latest/download/...`.
- A published tag and every asset attached to it remain immutable. `latest` is only GitHub's
  moving pointer to the newest complete stable release; it is not a mutable package stored
  outside the tagged release.
- The release is promoted to `latest` only after every platform producer, native smoke,
  package identity check, checksum, manifest, atomic assembly, remote byte comparison, and
  publication gate succeeds. Candidate mode performs the same production and assembly gates
  without creating a tag or release.

### Exact update identity and platform isolation

- `honk300 update`, `honk update`, and `goose update` are the same command grammar and use the
  same updater on every platform.
- The updater may fetch `release-manifest.json` from the stable `latest` URL only to discover the
  newest version and its immutable identity. The manifest must bind a stable tag, full commit,
  artifact name, target, kind, byte size, and SHA-256.
- Payload downloads use the manifest's exact `/releases/download/<tag>/<artifact>` URL, never
  the moving `latest` payload URL. Size, hash, artifact kind, target triple, install provenance,
  and owned install path must all match before mutation.
- Windows installer provenance stays within the matching architecture and installer family.
  Managed macOS apps update through the exact-tag universal app ZIP selected by the pinned shell
  bootstrap; the DMG remains the graphical fresh-install artifact, not a self-update transport.
  Shell-managed GNU/musl Linux installs remain on the matching archive selected by the pinned
  bootstrap. A release on one platform cannot replace another platform's installation.

### Debian package contract

- Each general release includes `honk300-amd64.deb` and `honk300-arm64.deb`. They reuse the
  byte-exact, native-compositor-qualified GNU binaries from the corresponding cargo-dist
  archives; the packaging job does not rebuild or substitute a second executable.
- The package owns `/usr/lib/honk300/honk300`, a regular `install-source.txt` marker containing
  `deb`, `/usr/bin/{honk300,honk,goose}` symlinks, the desktop entry, license, and exact release
  metadata. Mutable configuration and memes/notes remain in the invoking user's XDG directories
  and are never package-owned.
- A Debian-sourced update accepts only the architecture-matched `.deb` whose manifest kind is
  `deb` and target is the current GNU triple. Before downloading or invoking `dpkg`, the updater
  proves the current executable's exact path, regular-file marker, and `dpkg-query` ownership.
- Install, upgrade, and removal use `dpkg`; when the command is not already root, Honk300 may
  request elevation through `sudo`, falling back to `pkexec`. Failure to prove ownership or
  elevation fails closed before package mutation.
- Normal CLI uninstall removes the package but preserves user media. `uninstall --purge` first
  backs up user media, removes user state, and reports the backup. Package removal never treats
  XDG user content as a `dpkg` conffile.

## Consequences

- A developer can start a global release from any supported workstation and still produce a
  current, fully trusted Mac DMG and native Debian packages through GitHub's platform runners.
- Website and README links can remain stable while exact tag pages preserve reproducible history.
- Installed apps follow a moving stable channel without executing mutable or cross-platform
  bytes. The DMG is rebuilt for every release but is not required for in-place Mac updates.
- Debian packages are machine-wide and can require an administrator prompt; the no-sudo shell
  bootstrap remains available for per-user Linux installation.

## Verification

- Contract tests require both Debian package names and the DMG in every release manifest, and
  reject missing, unsafe, misclassified, wrong-target, wrong-size, or wrong-hash entries.
- The Debian producer extracts both `.deb` files, compares their installed executable byte-for-
  byte with the qualified GNU archive, and checks ELF architecture and package metadata. Native
  amd64 and arm64 candidate jobs then install the exact producer bytes and exercise aliases,
  compositor output, update grammar, uninstall preservation, and purge before atomic assembly.
- Post-release jobs run on native amd64 and arm64 Ubuntu hosts. They compare exact-tag and latest
  package bytes, verify sidecars and metadata, install the package, exercise all three update
  aliases, run X11 and Wayland compositor capture against the installed executable, and prove
  preserve-on-uninstall plus backup-on-purge behavior.
- macOS candidate/release jobs retain signing, notarization, stapling, Gatekeeper, app-ZIP, and
  DMG evidence on every release invocation; independent fresh-download verification checks the
  published bytes before the website is promoted.
