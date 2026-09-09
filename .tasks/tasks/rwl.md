TT;DR: Additional Wayland adapters.

## Why
Deliver the user-approved refinement while preserving healthy installations and explicit platform claims.

## Scope
Separate GNOME, Sway, Hyprland implementations and exact supported-desktop evidence. Never borrow KDE claims.

Authoritative sources: approved September 7 implementation plan, docs/adr/0041-refinement-native-settings-and-continuous-goose.md, current source/tests, docs/refinement-audit.md.

Preserve saved user settings/media, verified installer ownership, immutable releases, terminal protection, and graceful stop. Native hardware beyond this Windows PC is unavailable; hosted proof is identified separately.

## Plan
The independently scoped implementation and publication tasks are #rsw (Sway), #rhy
(Hyprland) and #rgn (GNOME). Each starts from its real native desktop premise, then
connects Rust and native settings, and earns its own complete release evidence. The
historical #wlg prototype scope is covered by these tasks; do not run a duplicate effort.
At each failed check record the exact failure and fix or leave the gate open.

## Impact
Separate GNOME, Sway, Hyprland implementations and exact supported-desktop evidence. Never borrow KDE claims.

## Acceptance
The requested behavior runs through its production path and the supporting evidence matches the claimed platform and installation.

## Verification
- [x] Relevant production-path regression tests pass.
- [x] Actual behavior or rendered output is inspected in its supported environment.
- [x] Integration and required package/release checks pass before publication.

## Status
Done. Sway v1.7.0, Hyprland v1.8.0 and GNOME v1.9.0 are independently
qualified, published and verified against freshly downloaded artifacts and
deployed website guidance. Each desktop retains its exact supported versions
and separate capability claims. The historical #wlg prototype scope is complete
through these production implementations; no duplicate prototype effort remains.

## Activity
- 2026-09-08 — Complete the ordered publication, fresh-public and website gates. See the final records in docs/readiness/v1.9.0-readiness.md and docs/readiness/v1.10.1-readiness.md. Preserve prior failures below as qualification history and keep unavailable physical acceptance separate.
- 2026-09-08 — Close the separately qualified Sway and Hyprland public stages; retain GNOME as the remaining compositor gate without extending either published version's claims.
- 2026-09-08 — Add individual adapter tasks, native premise checks, version boundaries and publication gates after current upstream review; all implementations remain queued behind the KDE stage.
- 2026-09-07 — created from the user-approved implementation plan (agent: codex).
