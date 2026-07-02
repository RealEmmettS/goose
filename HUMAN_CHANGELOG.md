# Human Changelog

A plain-English companion to [CHANGELOG.md](./CHANGELOG.md). Every change in the technical
changelog has a layman's-terms version here. No version numbers, no code references — just
what changed and why.

For the technical version with file paths and exact details, see CHANGELOG.md.

> **Where the project is:** the goose is alive on screen. It appears on your desktop, walks
> around, reacts to your mouse, makes sounds, can steal the cursor in a short, bounded prank,
> can hop onto a window while you drag it around, and can now bring in note and meme windows.
> It can now be controlled through a local command channel for starting, stopping, reloading, and
> simple poke commands. It now understands the friendly three-name command grammar and has a
> terminal settings screen backed by a saved config file. It now has dynamic moods and a double
> honk at the top of each hour. It now also respects quiet times, fullscreen/DND manners, and
> seasonal Autumn leaves. It now supports Windows multi-monitor chasing and fuller appearance
> controls. Mac support is now in the codebase and still needs real Mac smoke testing before the
> milestone is fully closed. There's no installer yet — that comes later.

---

## Latest — July 2026

**Added**
- Mac support is now in the app's codebase. It has a real Mac app identity for permissions,
  starts through the same command system as Windows, can show the goose through Mac desktop
  windows, can play sounds, can use Mac-owned note and meme windows, and reports permission
  problems clearly. Window-riding tricks stay gated behind Mac Accessibility permission. It
  still needs a real Mac smoke test before the Mac milestone is fully closed.
- There is a new status command and a Status page in the terminal settings screen. They show
  whether the goose is running, what platform and bundle mode it is using, whether Mac
  Accessibility is allowed or denied, which desktop tricks are available, and how many note and
  meme assets were loaded.
- The Mac app bundle can now be staged as a local personal-use app. The staging script builds
  Intel and Apple Silicon slices, combines them into one app, copies the assets, gives the app
  the stable permission identity, and signs it for local testing. Final disk images, notarized
  signing, and installers still come later.
- The goose can now roam across multiple Windows monitors when multi-monitor chasing is on. It
  treats the whole signed desktop as one space, so monitors to the left or above the main screen
  work too. If you turn multi-monitor chasing off, it stays on the primary screen.
- Drawing is now lighter. Instead of repainting a whole monitor-sized layer every frame, the app
  redraws the small part of the desktop where the goose and its active effects can appear, then
  clips that drawing to each monitor.
- Calm Goose is now a real setting. When it is on, the goose stops doing surprise disruptions and
  random honks, but direct clicks and commands still work.
- Appearance controls are now more complete. The settings screen lets you adjust red, green, and
  blue channels for the goose body, orange parts, and outline, so custom colors can change hue
  instead of only getting lighter or darker.
- The goose now has manners for quiet time, Do Not Disturb, and fullscreen use. During those
  periods it calms down: no random honks, no hourly double honk, and no autonomous pranks like
  cursor grabbing or dragging windows around. You can still click it or use direct commands, so
  the goose stays controllable instead of freezing completely.
- Autumn is now built in. From September through November, the goose can find little procedural
  leaf piles, run through them, and kick leaves around. The leaves are drawn and simulated inside
  the app rather than loading the original Autumn add-on.
- The settings screen now treats quiet hours, Do Not Disturb, fullscreen respect, seasonal mode,
  and Autumn as real live settings instead of future placeholders. There is also a separate row
  for whether fullscreen should make the goose calm down.
- The goose now has little moods. Most of the time it stays content, but it can occasionally get
  sleepy, sad, hyper, or mischievous. These moods change how it moves and carries itself without
  replacing its normal behavior system: sleepy and sad slow it down, sleepy makes little Zs, hyper
  can kick off the existing zoomy burst, and mischievous only leans into tricks that are already
  enabled and supported.
- The goose now does the on-hour double honk. At the top of each local hour it makes one high honk,
  then a second one a moment later, and it will not keep repeating during the same hour.
- The goose now has a real saved settings file and a terminal settings screen. You can open it
  with the config command, change current settings such as sound, mouse stealing, window riding,
  note/meme behavior, petting behavior, and timing, and save them without mixing settings code
  into the goose's core brain.
- The settings screen also shows future options honestly. Wayland/backend mode and extra prank
  behavior can be saved for later, but they are marked as planned or restart-required until those
  milestones actually exist.
- The command grammar now works under all three intended names: `honk300`, `honk`, and `goose`.
  `honk plz`, `goose plz`, and `honk300 plz` start it; `honk bad`, `goose no`, and
  `goose no honk` stop it; and pokes like honk, wander, mud, note, meme, or nab stay explicit
  through the `do` command.
- Installer, uninstaller, updater, and setup words are now recognized so help can list them, but
  the real installer/update work still waits for the packaging milestone.
- You can now control the running goose from commands. Starting a second goose is blocked, and
  commands can tell the current goose to stop, reload its options, honk, wander, track mud, or
  bring in a note or meme. This is also the foundation for the future settings screen.
- Terminal windows are now protected. The goose can wander over them visually like any other part
  of the desktop, but it is not allowed to move them, focus them, type into them, drag them, ride
  them, collect them, or target them for future prank behavior.
- The goose can now bring things onto your desktop: a real Notepad window for little goose notes,
  and separate meme image windows that it drags around. This is built so the goose's core logic
  still stays separate from Windows-specific window handles and typing tricks, which keeps future
  Mac and Linux support honest.
- The note and meme assets now have a clear personal-use rule. Screened original notes and memes
  are included for this owner's machines only when they pass the no-old-links/no-handles check,
  each copied one gets a custom in-house counterpart in the clumsy paint style, and the
  user-supplied goose drawing is included as an extra meme. One original meme with a visible
  handle watermark is left out. Old donate pages and old developer references are left out too.
- The goose can now run toward a window while you are dragging it and ride along if it gets
  there before you let go. When the drag ends, or if the computer says window watching is not
  available, it drops the trick and goes back to what it was doing. There is also a temporary
  no-window-riding option for running without that prank. Behind the scenes, this keeps Windows
  support separate from the goose's core logic so Mac, Linux, and limited Wayland support can
  report what they can honestly do later.
- The goose can steal your mouse cursor now, and that milestone is complete. When mouse stealing
  is available, clicking the goose makes it charge toward the pointer, bite when it catches it,
  and run around in its startled zooming mode for a short, bounded moment while holding the cursor
  before letting go. There is also a no-mouse-stealing option for running the goose without that
  prank. Behind the scenes, this was built so Windows works first while Mac and Linux can plug in
  their own cursor support later without changing the goose's brain.
- The project now has a place to record important architecture decisions. The first record
  captures how cursor stealing works, how unsupported systems should gracefully say "not
  available," why the renderer should move toward a lightweight sprite-sheet approach later, and
  what follow-up work should happen next. This keeps big decisions from getting buried in chat or
  task notes.
- The goose notices your mouse now. If you sweep the cursor back and forth over it — petting
  it — little hearts puff up from its head and it settles down happily and goes quiet for a
  moment. And if you *click* it, it gets startled and zooms around the screen for a couple of
  seconds before going back to whatever it was doing. (Clicks and pets only land on the goose
  itself; everywhere else your mouse works as normal.)
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

**Fixed**
- Settings for speed, muddy-footprint timing, colors, moods, and the hourly honk now actually
  affect the running goose instead of only being written to the settings file.
- If the settings file has extra unknown fields, the loader now warns once while still preserving
  those fields when it saves again. That keeps the config friendly to hand edits and future
  versions without silently hiding mistakes.
- When you tell the goose to do something from a command — like honk or grab the cursor — it now
  tells you the truth about whether it actually did it. Before, it always answered "okay!" even
  when it ignored you because it was busy or because that trick was switched off. Now, if it can't
  do the thing, it says so.
- If you switch mouse-stealing off and later switch it back on in your settings and reload, it
  actually comes back on now. Before, turning it off once quietly jammed it off until you fully
  restarted the goose.
- If the goose ever loses the ability to bring note and meme windows onto your computer, it now
  stays switched off instead of forgetting and pointlessly trying again every time you reload your
  settings.
- Turning off petting no longer accidentally turns off clicking. Before, switching off the
  hearts-and-calm petting also stopped the goose from reacting to a click. Now a click still makes
  it zoom around (or grab your cursor when that's allowed), even with petting turned off.

**Improved**
- The settings screen is cleaner internally and more complete to use. It now scrolls through real
  rows, edits quiet-hour times in 15-minute steps, cycles mood intensity through calm, normal, and
  spicy, asks before throwing away unsaved edits, and starts the goose without letting the child
  process mess up the terminal screen.
- The goose has been pulled back toward the original Desktop Goose look. The drawing is still
  fully procedural, but it now uses one cleaner, thinner oval body instead of several obvious
  pieces stuck together. The head stays tucked in, the beak is short, the eye is simpler, the
  feet are a little clearer, and the shadow is softer. A taller sprite-like version was tried and
  saved as a local comparison, then replaced because it did not feel as much like the original.

**Behind the scenes**
- The multi-monitor and appearance milestone now has its own architecture record. It says the
  shared goose logic only receives desktop bounds, while Windows owns monitor discovery and
  per-monitor transparent windows. It also records that recoloring means the original three-color
  goose palette, not a new renderer or new art.
- The quiet-hours, fullscreen/DND, and Autumn milestone now has its own architecture record. It
  keeps local date/time and computer-presence checks outside the goose's shared brain, while the
  shared brain decides what "be polite right now" means.
- The control milestone now has its own architecture record. It says control is handled by the
  command line and future terminal settings screen only, with no tray menu and no separate stop
  shortcut. It also records the permanent terminal-window protection rule.
- The window-riding milestone now has its own architecture record. It says the goose's brain
  only sees an anonymous window target and a place to ride, while Windows-specific hooks and
  window handles stay in the Windows layer. That keeps the next window tricks from leaking
  operating-system details into the shared engine.
- The earlier milestones were reviewed before closing the cursor-stealing milestone. That review
  fixed stale status notes, confirmed the goose's core logic still stays separate from
  Windows-only behavior, and created follow-up work for improving the fullscreen overlay's
  performance before packaging.
- The task board now shows the command grammar, settings-screen, mood, schedule/Autumn, and
  multi-monitor/appearance milestones as done. The future sprite-sheet renderer remains tracked as its own
  follow-up task instead of being treated as unfinished cursor-stealing work.
- The project guidance for future agents now says when to add or update architecture decision
  records, and it repeats the rule that the technical and human changelogs must stay in sync.
- The command-grammar and settings-screen milestones now have their own architecture records,
  written after a careful second look at how they were built. Those records also capture the four
  fixes above so the reasoning behind them isn't lost: commands should report what really
  happened, and the difference between "the user turned this off" and "your computer can't do
  this" must be kept straight so a setting can always be turned back on.
- The mood and hourly-honk milestone now has its own architecture record. It keeps the goose's
  mood logic inside the shared engine, while the platform-specific app simply tells the engine
  what the current local time is.

**Decided**
- The next major renderer should be a small, custom sprite-sheet system rather than a full game
  engine or heavy graphics framework. That should make the goose easier to customize while still
  fitting the transparent desktop overlay used on Windows and the future Mac and Linux versions.
- Starting, stopping, and changing settings will be done through commands and the future terminal
  settings screen. There is no tray menu and no separate stop shortcut.
- Terminal windows are off-limits for goose mischief, even for optional prank modes.

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
- The goose itself will be drawn from scratch rather than copied from the original. The original
  honk sound effects, screened meme pictures, and screened little notes are reused for personal
  use, and each copied meme or note also gets a custom in-house counterpart. Old donate pages and
  old developer references are not carried forward.
- On Linux it will target the older, more capable display system by default, with the
  newer one available as an opt-in — where the goose can do far less, because that newer
  system deliberately blocks most of the pranks.
- It will ship with proper Windows installers (four flavors), simple installers for Mac and
  Linux, and a real Mac app — but it won't be published to the Rust software registry.

**Behind the scenes**
- No actual program code yet — this round was entirely research and planning. The folder
  holding the original Desktop Goose files is kept only as a reference and won't be handed
  out or shipped.
