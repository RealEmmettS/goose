TT;DR: Verify the actual Mac settings Accessibility provider independently of the SDK UI harness.

## Why
Final acceptance reconciliation confirms native Windows UIA and Linux AT-SPI actions, while Mac interaction records use the SDK harness. The SDK implements an AppKit bridge, but source presence alone is not an OS-provider acceptance result.

## Scope
Begin with the exact already-built production companion and Rust sibling from
source b993b89dc0319ece8b790dabd1fc9881b066765f, then qualify the corrected production
companion on native Intel and Apple Silicon. Query and operate only its own
Accessibility application with a fresh isolated configuration. No runtime start,
installation, permission grant or global input is part of this provider probe.

## Plan
Exercise native names, button actions, switch state, focused text editing, modal isolation, full dirty-page save, disabled actions and separate manners status. Preserve failed trees. Fix a demonstrated provider defect before either pending release; keep the passing SDK checks distinct.

## Verification
- [x] Native Intel and Apple Silicon readers operate the actual provider and verify saved configuration.
- [x] Preserve real failures and correct any demonstrated provider defect without weakening the oracle.
- [x] Record final-source evidence and retain native reader coverage in future settings qualification.

## Status
Done. Independent native Intel and Apple Silicon readers reproduce and
correct dialog leakage and unapplied native text edits in the Mac settings
provider. The final production companion passes actual names, actions, switch
state, focus transfer, saved text, modal isolation and cached-action refusal.
Both releases retain the native reader in their final-source settings matrix;
v1.10.1 also verifies its separate presence labels. The correction is published
and fresh-public verified in v1.9.0 and v1.10.1. Exact-hash preparation, original
failures and physical screen-reader acceptance remain documented separately.

## Activity
- 2026-09-08 — Complete the ordered publication, fresh-public and website gates. See the final records in docs/readiness/v1.9.0-readiness.md and docs/readiness/v1.10.1-readiness.md. Preserve prior failures below as qualification history and keep unavailable physical acceptance separate.
- 2026-09-08 — Independent run 34292293311 passes all twelve native provider
  checks on Intel and Apple Silicon with automation disabled. Saved text, switch
  values, modal isolation and cached-action refusal are confirmed. Integrate
  both pending releases and repeat their complete source/publication gates.
- 2026-09-08 — Native run 34292132561's repeat preparation rejects an ambiguous
  reverse-patch match before compilation. Qualify the text-attribute replacement
  with its complete surrounding condition; repeated local preparation now
  recovers the exact original hash. Retain the guard and rerun both native lanes.
- 2026-09-08 — Run 34291576968 passes modal isolation and cached-action refusal
  on Intel and Apple Silicon, then fails edited-value persistence: the inherited
  modern AXValue setter only changes the native snapshot. Route that setter into
  the real text action and separate value/focus publication from input setters.
  Preserve both failure trees and the existing save-readback assertion.
- 2026-09-08 — Preserve run 34290279357's actual modal failure and both captured
  native trees; its initial Intel artifact transfer failure is separate. Cancel
  candidates 34287468666 and 34287505007 before publication. Add ADR 0051,
  bounded modal publication, current-action checks, an actual retained-reference
  regression and permanent production-Mac reader qualification. Local SDK hash
  checks pass; both native corrected-companion runs are next.
- 2026-09-08 — Identify the missing independent Mac reader result during final acceptance reconciliation; add a bounded actual Accessibility API fixture using the built universal app and isolated settings.
