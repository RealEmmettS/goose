# ADR 0051: Native Mac settings modal accessibility

Status: accepted; published and fresh-public verified in v1.9.0 and v1.10.1.

## Evidence

Final acceptance reconciliation found that Mac settings interaction records used
the SDK harness without an independent native reader result. Diagnostic
34290279357 queries the already-built production app from b993b89 through the
actual macOS Accessibility API. Switch actions, checked state and isolated save
work, but the edit dialog leaves background General/Appearance buttons exposed
as enabled AXPress actions. The native tree and failure remain evidence. An
Intel artifact-download failure is retained separately from application behavior.

## Decision

Keep Native SDK 0.5.4 and its AppKit accessibility implementation. Extend only the
existing exact-source-hash local patch mechanism. As in the Windows/Linux bridge,
publish the active dialog and its actual semantic descendants while it is open.
Traverse parents within the existing snapshot capacity; missing or cyclic
ancestry does not expose background controls. When no dialog exists, retain the
whole existing native widget snapshot.

Native selectors and action dispatch consult the currently published widget and
its enabled/action flags. A cached native element cannot retain action authority
after its page, dialog scope or allowed actions change. Keep the original
widget identities, shared Rust validation, bounded snapshots and configuration
ownership. This changes no installation or permission policy.

The ordinary native settings matrix now exercises the production Mac companion
with automation disabled through an independent Accessibility client. Native
Intel and Apple Silicon must prove names, switch state, focused text editing,
modal isolation, stale-reference refusal, full dirty-page save, disabled actions
and the status labels present in that release. The client uses only its own
settings process and a fresh isolated config, without a runtime start or grant.

## Qualification and publication

Retain the actual failing binary/tree, then repeat the same native assertions
against the corrected companion. Carry this shared settings correction into
both pending releases before tags. Repeat their final-source settings, candidate,
main and public gates; earlier green SDK harness results are separate evidence.
Physical VoiceOver user acceptance remains distinct from hosted AX-provider proof.

The first corrected native run, 34291576968, passes modal isolation and retained
background-reference refusal on both architectures. It then demonstrates a second
provider defect: setting AXValue changes the inherited native element to `25`,
but applying that edit leaves the draft unchanged and Save disabled. Route the
modern AppKit value setter through the existing runtime text action, alongside
the legacy attribute API. Publish snapshot value and focus through explicit
superclass setters so readback cannot synthesize input or claim an unapplied edit.
The original saved-value assertion remains required.

Independent native run [34292293311](https://github.com/RealEmmettS/goose/actions/runs/34292293311)
passes all twelve actual provider checks on Intel and Apple Silicon using the
production companion at `8d27d3e` and the identical previously qualified Rust
sibling. Both saved configurations confirm the native text edit. Original modal
and text-persistence failures remain evidence. Run 34292132561 was stopped by
the exact-source guard during repeated patch preparation; contextual matching
fixes its reversible recipe without relaxing that hash. This focused native
result permits integration, while both releases still require complete final-source
settings, candidate, unchanged-main and fresh-public qualification.

## Publication evidence

The final unchanged-source candidate, main, immutable publication, fresh-download and website results are recorded in [the GNOME release](../readiness/v1.9.0-readiness.md) and [the final manners release](../readiness/v1.10.1-readiness.md). Earlier pending statements and cancelled candidates above describe the original qualification sequence. Physical acceptance retains its separate limits.
