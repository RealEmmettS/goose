# ADR 0055: Opt-in seasons and intentional petting

Date: 2026-09-09

Status: Accepted; implementation and release qualification in task #g12.

## Context

The user wants an easy way to turn autumn leaves off and seasonal effects disabled
by default. They also want petting to require a deliberate few seconds of rubbing
before any hearts or pleased animation, so passing the cursor over the goose does
not immediately trigger an affectionate reaction.

## Decision

- Move the existing `schedule.seasonal` and `schedule.autumn` switches to Appearance.
  Seasonal effects default to false in both configuration and engine options. An
  explicitly saved value keeps its meaning; loading or updating never silently
  replaces it. Autumn leaves retain their dependent switch, and disabling either
  switch clears active piles through the existing live configuration path.
- Reuse the existing 2.5-second active rubbing threshold for every petting response:
  the pleased animation, hearts, pat sound, calming and the friendly follow invitation.
  Stationary hover and quick sweeps do not earn time. Leaving the goose, pressing a
  button or pausing motion for more than 0.3 seconds resets unearned progress.
  Movement before qualification is discarded, so it cannot produce queued hearts.
- Retain one follow invitation per sustained rubbing sequence, the existing follow
  duration and gap, configuration gate, cancellation conditions and input boundaries.

This supersedes the immediate-pat behavior in the original interaction plan and
extends ADR 0052's timed follow threshold to the first visible petting response.
It changes no renderer, overlay ownership, installation or permission contract.

## Verification

Verify default-off behavior for absent settings, round-trip preservation of an
explicit seasonal choice, live clearing and suppression of piles with either switch,
and resumption after enabling both. Inspect the actual Appearance page.

Exercise quick swipes, stationary hover, interrupted rubbing and sustained movement
at several polling rates. Through the real world input path, assert that hearts,
the pleased pose, pat sound, calming and following are absent before the threshold
and that a deliberate rub earns the existing bounded response afterward.

The user also reported intermittent black rectangles around Windows overlay content.
The installed binary and the preceding candidate share identical overlay code. The
user saw no rectangles during a five-minute isolated installed-binary rerun; native
presented pixels were valid premultiplied alpha. Those observations do not identify
the original cause or establish a renderer fix. Retain that distinction in release
qualification and do not add a speculative rendering change.
