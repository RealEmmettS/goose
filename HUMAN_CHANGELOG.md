# Human Changelog

A plain-English companion to [CHANGELOG.md](./CHANGELOG.md). Every change in the technical
changelog has a layman's-terms version here. No version numbers, no code references — just
what changed and why.

For the technical version with file paths and exact details, see CHANGELOG.md.

> **Where the project is:** the goose is alive on screen. It now appears on your desktop and
> walks around. There's no installer yet — that comes later.

---

## Latest — June 2026

**Added**
- The goose makes noise now! It honks as it wanders and squelches when it tromps through
  mud, using the original goose's own sounds. You can silence it with a "no sound" option,
  and if your computer has no speakers it just stays quiet. (Since this is a personal version
  you run on your own machines, the original sounds — and later the memes and notes — are
  bundled right in.)
- The goose now has a proper "mind" instead of the temporary wandering placeholder. When it
  first shows up it makes a little entrance — it walks on from the bottom of the screen and
  pauses a moment to introduce itself — then settles into roaming on its own, choosing where
  to go and occasionally tracking mud. This is built so new tricks (grabbing your cursor,
  opening windows, and so on) can be added cleanly later.
- The goose now leaves muddy footprints! When it "steps in mud," a trail of little brown
  prints follows it as it waddles, and they slowly fade and shrink away on their own. (To show
  the trail across your screen, the goose's see-through layer now covers the whole monitor.)

**Improved**
- The goose actually looks like the real Desktop Goose now. To get it right, the real goose was
  run and watched directly: it's a soft, rounded white blob with its head tucked into the front
  of its body, a little rounded orange beak, a small dark eye, orange webbed feet, and a soft
  dotted shadow underneath. An earlier version had given it too long a neck (copied from a
  drawing rather than the real thing) — that's fixed.

**Added (earlier this session)**
- The goose now actually appears on your screen and walks around! It floats on top of
  everything as a see-through window, so you can still click the things behind it — only the
  goose itself is solid. It wanders to a random spot, waddling on two little orange feet, then
  pauses and picks a new spot. We rebuilt how it's drawn so it looks like the real Desktop
  Goose: a plump white body, a neck up to a small head with an orange beak and an eye, and a
  soft shadow underneath. (The wandering is a simple placeholder for now — the goose's real
  personality and mischief come later.) It's smooth and light on your computer because it only
  redraws the little patch around the goose, not the whole screen.
- The first real piece of the goose: its "brain." This is the part that knows how the goose
  moves (its walk, run, and charge speeds), how it's shaped, how it leaves muddy footprints that
  fade away, and how it randomly decides what to do next — all rebuilt from scratch and matched
  exactly to the original goose's own numbers. It can even draw a little picture of the goose to
  a file (used behind the scenes to catch accidental changes later), even though it isn't running
  on your desktop yet. None of the screen, window, or computer-specific parts are here yet — on
  purpose — so this piece is simple to test thoroughly, and every automated check passes. The
  drawn goose's exact proportions are a rough first pass; making it look just right happens once
  it's actually on screen.
- One master build plan that combines the two earlier plans into a single source of truth. Both
  earlier plans were fact-checked against the original goose's own files; the more accurate one
  was used as the foundation and the best ideas from the other were merged in. (For example, one
  plan had the goose's exact speed and size numbers right, while the other had guessed them wrong
  — so the correct numbers won.)

**Decided**
- A batch of new, optional things the goose can do on its own — all switch-on/switch-off — that
  build on the original's spirit: little moods (it might get hyper, sad, or sleepy and act
  differently), gentle season changes through the year (the autumn leaves become a year-round
  idea), chasing across multiple monitors, a double honk at the top of each hour, hopping up to
  ride a window while you drag it, happy hearts when you pet it by sweeping your cursor over it,
  and quiet manners (it calms down at night, during Do-Not-Disturb, and while you're in a
  full-screen game or call). Out of the box it still behaves like the original prank, always on.
- A built-in settings screen you open in the terminal to flip any of these on or off (including
  the autumn leaves), with changes that mostly take effect instantly on the running goose.
- Three names to launch and control it (you can type "honk300," "honk," or "goose"), with playful
  commands like "goose plz" to start it, "honk bad" to stop it, and "goose do honk" to make it
  honk on demand. A help screen and the settings screen both list everything.
- It will be built and packaged for every system and chip type it's advertised on — Windows, Mac,
  and Linux, on both the standard and the newer ARM processors.

**Changed**
- The two earlier plans are now kept only as background reference; the new combined plan is the
  one to follow. The project's front-page notes now say this too, with a short summary of what was
  decided.

---

## Earlier — June 2026

**Added**
- A detailed build plan for "honk300," a brand-new version of Desktop Goose (the prank app
  where a goose wanders around your screen and causes mischief), rebuilt to run on Windows,
  Mac, and Linux. The plan works out how the original goose actually behaves and lays out
  exactly how to recreate it — how the goose is drawn, how it decides what to do, how it'll
  be packaged into proper installers, and the tricky cross-platform problems to watch for.
- A second, separate plan written by a different AI assistant (Codex), kept alongside the
  first for comparison.
- Two changelogs (this human-readable one and a technical one) and a guidance file for
  future AI sessions working in this project.

**Decided**
- The new app will be called "honk300," matching the family of similarly-named tools on
  this machine.
- The goose itself will be drawn from scratch rather than copied from the original. The
  original honk sound effects are reused for personal use; the meme pictures will be
  re-created as original art instead of copied; and the little notes the goose types will
  be written fresh.
- On Linux it will target the older, more capable display system by default, with the
  newer one available as an opt-in — where the goose can do far less, because that newer
  system deliberately blocks most of the pranks.
- It will ship with proper Windows installers (four flavors), simple installers for Mac and
  Linux, and a real Mac app — but it won't be published to the Rust software registry.

**Behind the scenes**
- No actual program code yet — this round was entirely research and planning. The folder
  holding the original Desktop Goose files is kept only as a reference and won't be handed
  out or shipped.
