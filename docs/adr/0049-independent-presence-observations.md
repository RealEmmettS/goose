# ADR 0049: independent fullscreen and notification observations

Status: accepted; native Intel/Apple Silicon runtime and denied-identity qualification passed, publication pending.

Audit R05 requires effective observations independently from saved manners toggles.
Add a bounded additive `PRESENCE` control frame with separate fullscreen and DND
capabilities. Keep the legacy `STATUS` wire layout unchanged. CLI, Native SDK and
TUI read the separate frame; older runtimes that do not implement it produce an
unprobed result, never support inferred from the aggregate field.

Native macOS premise run
[34243557287](https://github.com/RealEmmettS/goose/actions/runs/34243557287)
observes real AppKit fullscreen transitions on Intel and Apple Silicon. The
accessory app's `currentSystemPresentationOptions` remains zero and is unsuitable
for this observation. The focused application's AX window reports `AXFullScreen`
through the existing permission-gated Accessibility API. A missing attribute or
window remains unavailable; screen-sized geometry is not evidence of fullscreen.

Keep AppKit identity lookup on the main thread using
[NSWorkspace.frontmostApplication](https://developer.apple.com/documentation/appkit/nsworkspace/frontmostapplication).
One retained worker receives only its PID and a generation. It uses the remaining
time from a single 100 ms query deadline for each AX operation. No native query
runs under the shared result lock. Samples age from before the query, expire
after 250 ms and are discarded on target change or permission loss. An expired
completed sample reports failed; unprobed is reserved for a new target. A fresh
result recovers the capability without extending the observation lifetime. The worker
is joined on shutdown; unresponsive native target, live setting changes, stale
results and graceful stop must be exercised through the actual runtime.

This adds no production permission prompt or persistence mechanism. Existing
managed Accessibility onboarding retains its receipt and update boundaries.
Native denied behavior, in-flight withdrawal and physical permission acceptance
remain distinct evidence. Native run 34253150265 passes fullscreen transitions,
stopped-target expiry/recovery, live preferences, independent commands, shutdown
and the separate denied app identity on Intel and Apple Silicon.

Apple documents
[INFocusStatusCenter authorization](https://developer.apple.com/documentation/intents/infocusstatuscenter/requestauthorization(completionhandler:))
and a purpose string before requesting Focus status. A false value returned while
authorization is not determined is not a usable DND observation. Private probes
34244111946 and 34244463220 reached an unanswered prompt attributed to the hosted
command runner. LaunchServices probes 34244921263 and 34245719504 returned neither
an authorization callback nor an app-specific prompt within the bound. This does
not refute the API: app-specific authorization and actual on/off signal remain
unqualified. Leave Mac DND unsupported in the product. Keep the experiment an
explicit optional CI dispatch, without granting the broader runner.

Linux reports fullscreen from its actual enabled KDE, Sway, Hyprland or GNOME
observation provider, including failed, withdrawn and unsupported states. No
portable or per-compositor DND observation has been qualified, so that capability
remains unsupported. Windows retains its existing notification-state mapping
and recovers its reported capability after a successful poll following failure;
its notification availability categories do not identify a specific Focus mode.

The complete staged-release gates still apply to the resulting change. Hosted
native evidence does not establish physical Mac permission or Pi acceptance.
