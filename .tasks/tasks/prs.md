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
Active. Linux compositor fullscreen now has separate native evidence. Probe actual
macOS system presentation, window accessibility and Focus authorization from an
independent accessory process during a private native app's fullscreen transitions.
Do not request permissions or infer availability from window size.

## Activity
- 2026-09-08 — Both 34244463220 screenshots show the real Focus permission dialog attributed to the hosted command runner. Launch the private bundle through LaunchServices before qualifying its own permission identity; do not grant the broader runner or classify an unanswered native prompt as an unsupported API.
- 2026-09-08 — Both signed private Focus requests in 34244111946 reach authorization not-determined and do not call back within the bound. Capture the actual disposable native permission surface before classifying this as an unavailable API or awaiting user consent.
- 2026-09-08 — Run 34243557287 completes on native Intel and Apple Silicon. Accessibility reports actual fullscreen transitions; currentSystemPresentationOptions stays zero throughout and is refuted for this accessory path. Focus reports not-determined authorization despite a false value, so that value is not a DND signal. Probe the documented explicit authorization request in a separately signed private fixture bundle next.
- 2026-09-08 — Both native architectures compile the actual API premise, but the first observer does not exit. Bound the independent process with a watchdog and record its last API stage; use explicit process completion after writing evidence instead of waiting for another accessory-app event after stopping its run loop.
- 2026-09-08 — Start a bounded native x64/Apple Silicon API premise using documented system presentation and Focus APIs. Observe actual normal/fullscreen/restored phases from an independent process, record existing authorization without prompting, and retain unavailable APIs honestly before production wiring.
- 2026-09-08: Made unresolved R05 independently visible while reconciling the completed first-stage audit corrections. No observation or acceptance requirement was waived.
