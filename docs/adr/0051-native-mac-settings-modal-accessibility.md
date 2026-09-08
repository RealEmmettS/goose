# ADR 0051: Native Mac settings modal accessibility

Status: accepted correction; native and release qualification in progress.

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
