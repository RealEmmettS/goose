TT;DR: Finish the remaining fullscreen and do-not-disturb observation gap without inferring support from saved settings.

## Why
Audit R05 remains distinct from the reliability corrections published in the first refinement stage. Manners controls must reflect the actual platform observations available to the running goose.

## Scope
Inspect supported macOS and Linux observation APIs, add observations where a native premise is proven, and retain explicit unsupported or unavailable behavior elsewhere. Coordinate Linux compositor fullscreen proof with #wlk and the subsequent adapters. Preserve saved settings, quiet hours, terminal protection, permissions and independent capability reporting. No inferred system do-not-disturb state or new permission prompts follow from a toggle alone.

Authoritative sources: the approved September 7 refinement plan, `docs/refinement-audit.md` R05, production presence adapters and current platform documentation.

## Plan
First exercise the platform's actual observation API on a disposable native desktop. Test supported, unsupported and permission-loss states before changing runtime claims. After two attempts without new evidence, stop that route and inspect its premise and observer before continuing. Keep unavailable physical-hardware acceptance explicit.

## Acceptance
The functional bar is honest independent fullscreen/DND status and correct manners behavior when observations are available or lost. The evidence bar is real native state changes and production-path failure tests; advertised support requires the normal platform/package release gates. The user-approved plan owns those gates. Unsupported APIs may be recorded as refuted with explicit fallback, never as inferred support.

## Verification
- [ ] Native fullscreen state changes affect the real engine manners path and revert on exit/loss.
- [ ] Any advertised DND support is established by an actual platform signal; unavailable paths remain clearly unsupported.
- [ ] Settings and runtime status agree after permission loss, reload and unsupported-session selection.

## Status
Queued. The first stage truthfully reports unsupported Mac/Linux observations. KDE fullscreen is under separate native qualification, while macOS and DND evidence remain open. This task preserves that original audit obligation within the refinement milestone.

## Activity
- 2026-09-08: Made unresolved R05 independently visible while reconciling the completed first-stage audit corrections. No observation or acceptance requirement was waived.
