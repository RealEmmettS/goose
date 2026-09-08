TT;DR: Verify the actual Mac settings Accessibility provider independently of the SDK UI harness.

## Why
Final acceptance reconciliation confirms native Windows UIA and Linux AT-SPI actions, while Mac interaction records use the SDK harness. The SDK implements an AppKit bridge, but source presence alone is not an OS-provider acceptance result.

## Scope
Use the exact already-built production companion and Rust sibling from source b993b89dc0319ece8b790dabd1fc9881b066765f on native Intel and Apple Silicon. Query and operate only its own Accessibility application with a fresh isolated configuration. No runtime start, installation, permission grant or global input is part of this probe.

## Plan
Exercise native names, button actions, switch state, focused text editing, modal isolation, full dirty-page save, disabled actions and separate manners status. Preserve failed trees. Fix a demonstrated provider defect before either pending release; keep the passing SDK checks distinct.

## Verification
- [ ] Native Intel and Apple Silicon readers operate the actual provider and verify saved configuration.
- [ ] Preserve real failures and correct any demonstrated provider defect without weakening the oracle.
- [ ] Record final-source evidence and retain native reader coverage in future settings qualification.

## Status
Active. A separate two-host diagnostic consumes the completed source-CI app artifacts without rebuilding or modifying either pending release source.

## Activity
- 2026-09-08 — Identify the missing independent Mac reader result during final acceptance reconciliation; add a bounded actual Accessibility API fixture using the built universal app and isolated settings.
