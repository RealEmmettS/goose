# ADR 0034 — Command-First Managed Installation

- Status: Accepted (2026-07-18)
- Relates to: ADR 0020 (Developer ID DMG), ADR 0022 (managed Accessibility identity), ADR 0023
  (rolling latest and Debian ownership), and ADR 0031 (provenance-preserving self-update).
- Supersedes: only the package-first recommendation in ADRs 0020, 0023, and 0031. Their artifact,
  signing, package ownership, update provenance, rollback, and release requirements remain
  accepted.

## Context

Honk300 already publishes stable, versionless PowerShell and POSIX bootstrap names, but its public
guidance recommends the Global MSI on Windows, the DMG on macOS, and `.deb` packages on Debian.
The sibling `*300` products independently converged on an easier default: one copyable command per
platform, with native packages kept as deliberate alternatives.

Changing documentation alone would be misleading. A supported command-first installer must own a
durable receipt, install the actual GUI-capable product shape, preserve that owner during
`honk300 update`, and either take over a recognized prior official channel or fail without
claiming success. Raw `cargo install` cannot run a product post-install lifecycle hook and this
project is not distributed through crates.io, so Cargo cannot satisfy that contract.

## Decision

### Public recommendation

The preferred fresh install is the stable versionless official bootstrap:

- Windows: `irm https://github.com/RealEmmettS/goose/releases/latest/download/honk300-installer.ps1 | iex`
- macOS/Linux: `curl --proto '=https' --tlsv1.2 -fsSL https://github.com/RealEmmettS/goose/releases/latest/download/honk300-installer.sh | sh`

The bootstrap itself is immutable-release-stamped after discovery. It downloads exact-tag bytes,
verifies embedded target/hash/size identity, performs the platform transaction, writes a protected
v2 receipt, and verifies the public command. Mutable `latest` is never used for payload mutation.

The signed/notarized/stapled universal DMG remains the graphical Mac alternative and installs the
same real app at `~/Applications/Honk300.app`. Global/Corporate MSI and EXE packages remain native
Windows alternatives. Architecture-matched Debian packages remain the native Debian/Ubuntu
alternative. Portable/source builds and raw Cargo commands are development or unmanaged paths;
they do not silently acquire installer ownership.

### Fresh intent versus update transport

`honk300 update` always preserves the proven origin. It passes an internal, allowlisted origin hint
only when invoking the Unix bootstrap as an update transport. The script validates that hint
against the existing regular receipt and exact owned root before retaining `mac-app`/DMG or
`shell` identity.

Already-published v1.2.2-and-earlier Mac updaters cannot supply that hint. They invoke the
downloaded script under the established `honk300-update-*-honk300-installer.sh` temporary name.
The new script retains a narrow compatibility bridge for that name only, and still requires the
existing receipt/root to prove DMG or shell ownership before preserving it. A public pipe or a
normally downloaded `honk300-installer.sh` does not match the bridge and remains fresh intent.

A public bootstrap invocation has no update hint and is therefore a deliberate fresh install. On
macOS it writes shell ownership even when replacing a previously DMG-owned app. A later graphical
DMG install runs the sealed app's shared installer and writes DMG ownership again. Both paths
install the same signed app bundle and both exact managed receipts remain eligible for automatic
Accessibility onboarding; mounted-DMG and unreceipted launches remain ineligible.

On Windows the PowerShell bootstrap deliberately delegates to the Global MSI transaction with
`HONK300ORIGIN=powershell`. A directly launched Global MSI always defaults to `msi-global`; it no
longer carries a prior PowerShell marker across a repair. Thus MSI → bootstrap → MSI changes
ownership in both directions, while updates from either origin retain it. The existing protected
slot, registration, cross-scope cleanup, and nonzero `cleanup_pending` rules remain unchanged.

### Scope boundaries

An official fresh installer is authoritative only when the transaction can safely converge the
recognized owners. Windows may commit the new verified slot and then report nonzero
`cleanup_pending` if an administrator grant is needed to retire the prior scope.

The no-sudo Linux shell bootstrap cannot remove a machine-owned Debian package. It proves the
dpkg registration, marker, receipt, and owned executable before refusing pre-commit with the exact
`sudo dpkg --remove honk300` recovery. Conversely, the Debian `preinst` refuses before package
commit when a standard user-scoped shell alias is active and instructs that user to run the
owned shell uninstall first. It does not delete user files as root. This explicit assisted
two-step is preferable to two competing command paths or an unsafe cross-scope mutation.

## Consequences

- A GUI does not require package-first advertising. The official macOS command installs the real
  signed app bundle, and Windows/Linux desktop shortcuts launch their existing windowless GUI
  entrypoints.
- Native packages remain fully supported and keep their own update/uninstall semantics; they are
  alternatives rather than prerequisites.
- A fresh official install can intentionally change channel or downgrade after verification.
  Updates never change channel.
- Raw Cargo cannot be advertised as equivalent to the managed bootstrap and never removes an
  MSI, EXE, DMG, shell, or Debian owner.
- Cross-scope Linux conversion is visible and assisted instead of falsely seamless.

## Verification contract

- Windows x64 and native ARM64 qualification covers Global MSI → PowerShell bootstrap → fresh
  Global MSI plus every Global/Corporate MSI/EXE transition while an old process remains mapped.
  Public-byte qualification runs the actual downloaded PowerShell script.
- Signed Mac candidate qualification covers fresh shell install, fresh DMG takeover,
  legacy-filename and current hinted DMG preservation, and fresh shell retake on the same app
  path. Public smoke repeats the exact-tag DMG/script sequence and verifies signatures/receipts.
- Mac Accessibility tests accept only exact-path, release-matching v2 DMG or shell receipts.
- Debian package tests require a non-mutating pre-install collision refusal. Shell tests prove
  Debian ownership is validated and refused before staging, while ordinary shell updates retain
  shell provenance.
- README, website, project instructions, canonical plan, both changelogs, and release readiness
  use the same recommendation and unmanaged-Cargo wording.
