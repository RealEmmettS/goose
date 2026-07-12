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
> controls. Mac support and the Linux desktop paths are now in the codebase, with repeatable
> CI smoke proof for hosted Mac bundle checks and Linux desktop behavior. Mac Accessibility-granted
> desktop tricks still need a pre-approved Mac smoke run. The Windows/Linux installer and update
> work now has release artifact proof, including Windows installers for both regular x64 and ARM64
> machines.

---

## Latest — July 2026

### Fixed
- **Windows setup replaced its filler agreement with something worth scrolling.** The installer
  now shows the real noncommercial software terms first, then a long and deliberately ridiculous
  personal pact with the Goose that is clearly just a joke. This keeps the important terms honest
  while letting the Goose negotiate cursor custody, imaginary bread, pond arbitration, and the
  serious allegation that it might be a duck.

### Behind the scenes
- **Published downloads now get a full cross-platform rehearsal.** A separate release check can
  install the exact public downloads on both kinds of Mac, both Linux architectures, and Windows,
  then deliberately interrupt replacement work to prove the previous install returns intact. It
  also rehearses Windows upgrade, repair, downgrade protection, and removal, which helps catch
  packaging failures that source tests cannot see.

### Added
- **One dependable install story.** Windows now leads with the all-users installer, while Mac and
  Linux share one verified terminal command that chooses the right download without asking for
  administrator access. Each managed install leaves a receipt so updates and removal know exactly
  what they own.
- **Safer settings recovery.** The app can now tell the difference between a missing settings
  file, a healthy one, a broken one, and one written by a newer app. Resetting is always explicit
  and backs up the previous file first, so recovery never silently destroys a user's choices.
- **Clear licensing and media provenance.** Releases now say which terms cover the app, which
  bundled media still belongs to other creators, and where the temporary Wayland build fix came
  from. This makes downstream use less ambiguous.

### Improved
- **A more natural long-necked goose.** The side view is now one coherent silhouette, with a broad
  neck base, separate back and throat curves, and a simpler oval head. It keeps the familiar wing,
  beak, legs, colors, directions, and poses while looking less pinched or assembled from parts.
- **Steadier long-running and multi-monitor behavior.** The goose's clock keeps its precision over
  long sessions, monitor gaps are no longer treated as places it can walk, hot-plugging screens
  safely rebases what it is doing, and drawing work stays limited to the small area that changed.
  Shared runtime ordering also makes the different operating-system versions behave alike.
- **Fewer install choices to second-guess.** Windows recommends the machine-wide installer. Mac
  and Linux recommend the same no-sudo terminal flow. The old Mac disk image remains only for
  older updater compatibility, and the site/docs are candid that the Mac build is not Apple
  notarized.

### Fixed
- **Broken settings are no longer replaced with defaults.** Saves are safer, linked settings
  files are handled deliberately, display-backend changes clearly ask for a restart, and a
  stopped goose no longer claims every capability is unsupported. The settings screen stays
  responsive during slow commands, saves before starting, reports the real launch error, and
  remains usable in small terminals.
- **Animations and delayed behavior now finish cleanly.** View transitions no longer briefly
  over-brighten, puddle hops return smoothly, quiet-time manners cancel queued pranks, note and
  meme switches act independently, scheduled honks do not duplicate, and missing reference
  images fail tests instead of quietly changing the expected picture.
- **Desktop backends recover instead of surprising the user.** Windows adapts when one monitor is
  removed and never types through global keyboard focus. Mac screen coordinates, real window
  drags, display changes, and temporary audio failures are handled correctly. Linux refuses an
  unsafe opaque/click-catching overlay, while reduced Wayland mode now handles multiple scaled
  outputs, unplugging, and compositor buffers without growing memory forever.
- **Install, update, and release changes are transactional.** Unsafe archives are refused, files
  owned by somebody else are left alone, interrupted replacements restore the prior install, and
  privileged Windows updates wait until the running app exits. A release becomes public only
  after every required download has been assembled and verified together.
- **Linux release checks no longer race desktop startup.** The automated visible-desktop check
  waits until its compositor is genuinely ready before launching the goose, and shows the real
  startup error if readiness still fails. This makes Linux release evidence repeatable and easier
  to diagnose.
- **Linux click-through setup now completes its required handshake.** X11 desktops can reject an
  input-shape request if an app skips the extension handshake, even when support is installed.
  The goose now performs that handshake before creating any overlay, so it can remain safely
  click-through instead of refusing an otherwise healthy desktop.
- **Every release builder now starts with the tools and package metadata it actually needs.** An
  unpublished build caught missing Linux audio headers, platform-specific checksum and archive
  assumptions, an incomplete Mac target setup, and invalid or auto-corrected Windows installer
  metadata. The complete download set can now be assembled and tested before reserving a public
  release name, and Windows proves that the all-users installer upgrades the prior release,
  repairs cleanly, refuses a downgrade, and uninstalls fully. These checks keep a failed release
  identity immutable while making the corrected one safer.

### Security
- **Local controls and downloaded installers are more tightly contained.** Only the signed-in
  user and the operating system can reach the Windows command channel; Mac and Linux verify the
  identity and permissions of local peers, even when a managed Mac supplies an unusually long
  temporary folder. Windows install/update helpers and note pranks launch only the operating
  system's own tools, never a lookalike from the current folder. Network reads have deadlines,
  dependency auditing is mandatory, and the Wayland build uses the upstream parser security
  correction without pulling in incompatible unfinished changes.

### Removed
- **Old reference/source baggage and dead settings are gone.** The original developer reference
  folder, duplicate mute switch, setting that never did anything, and old donation/developer
  material are no longer in the active project. Historical planning notes remain for context.

### Improved
- **A better goose neck.** Seen from the side, the goose's neck used to curve slightly backward
  into its body and showed a faint line where the neck met the shoulders. It now rises in a clean,
  natural sweep that leans gently forward and blends smoothly into the body, with no seam. The
  walking goose on the website was refreshed to match.

### Added
- **A brand-new goose.** The goose has been completely redrawn in a clean, modern flat style —
  layered dark wing with feathered tips, two-tone beak, proper little legs with webbed feet, a
  graceful curved neck — and it finally animates like a real bird: feet plant on the ground and
  step (no more ice-skating), the neck raises and settles smoothly, it blinks, breathes when
  idle, and flicks its tail when it honks. When it walks steeply up or down the screen you now
  see it from above instead of a sideways-rotated bird.
- **Recolor everything.** Six goose colors (body, shading, wing, beak, dark accents, outline) are
  now adjustable from the settings screen, and old saved settings keep working.
- **A goose with a life.** It wanders in curvy, gooselike paths instead of straight lines; every
  few minutes it waddles off the edge of the screen, disappears for a bit, and comes back —
  sometimes with a prank in tow. Muddy footprints now have a story: it only tracks mud after
  nipping off-screen to (apparently) find a puddle, instead of being muddy all the time.
- **More natural ways to say stop.** `goose exit` and `goose quit` now work just like
  `goose stop`.
- **Two more sliders in settings** for how close the goose must get to grab the cursor and how
  far it drags it.
- **A dormant setting now works.** "Attack randomly" — the goose occasionally deciding on its own
  to grab your cursor — was saved in settings but never actually did anything. It works now, off
  by default, with its own switch in the settings screen.
- **The goose can now be installed on a Mac.** There's a proper Mac app now: a single download
  (a disk image) that works on both Intel and Apple Silicon Macs, which you drag into your
  Applications folder like any other app. Because it isn't signed by Apple (it's a personal build),
  the very first time you open it you right-click the app and choose "Open" once to approve it —
  after that it launches normally. You can also have it start automatically when you log in.
- **Installing, removing, and updating all work on the Mac too.** The goose sets itself up under
  three names, can start at login, cleans itself up when removed (keeping any memes or notes you
  added), and can update itself in place — the same convenience the Windows and Linux versions
  already had.

### Changed
- The goose's default colors are a touch softer and more designed; if you liked the old stark
  white look you can dial it back in the color settings.

### Fixed
- **Sharper, correctly-placed goose on high-resolution screens.** The app now tells Windows it
  understands modern display scaling, so the goose is crisp and stands exactly where it should on
  mixed-resolution monitor setups — and it adapts on the fly if you change scaling or plug in a
  screen, no restart needed.
- **No more freezes while the goose fetches a note.** Bringing in a Notepad window used to stall
  the whole goose (and its remote controls) for up to a few seconds; it all happens in the
  background now, so the goose keeps moving and commands keep answering.
- **No leftover Notepad windows.** Notepads the goose opened are properly closed when it leaves —
  quitting the goose no longer strands stray windows or background processes.
- **Restarting after a crash works on Mac and Linux.** A crash used to leave behind a "still
  running" marker that blocked the next start; the marker is now managed by the operating system
  and clears itself no matter how the app ended.
- **Smoother window handling on the Mac.** The Mac overlay now processes its window events
  properly (close buttons on goose-made windows work), and screen drawing uses safer memory
  handling.
- **The goose won't pretend to run invisibly on Linux.** If the screen overlay can't be created,
  starting now fails with a clear message instead of a silent, invisible "running" state — and a
  new "overlay" line in the status output tells you exactly what the display is doing.
- **Uninstalling no longer deletes your own memes and notes.** A plain uninstall now sets your
  user-added content aside in a clearly-named folder and tells you where it is.

**Added**
- The installer words are no longer just placeholders. The app can now install itself for the
  current user, add the three command names, copy its assets, create Windows shortcuts or Linux
  desktop entries, optionally start at login, uninstall itself, and back up user-added memes and
  notes before a full purge.
- The updater now follows the release installer path instead of trying to use Cargo. It detects
  how the app was installed, chooses the matching download for the computer's architecture, checks
  the published checksum before running it, and verifies the installed version afterward.
- The release setup now has cargo-dist metadata, a release workflow, Windows x64 and ARM64
  installer workflows, WiX and Inno installer manifests, install-source markers, and checksum
  sidecars. The first personal-use release now contains the Windows and Linux downloads, shell and
  PowerShell installers, and both Global and Corporate Windows installers for x64 and ARM64. Mac
  disk images, signing, notarization, and Accessibility-granted proof are still deliberately
  deferred; later Mac packaging defaults to unsigned personal-use builds unless signing
  credentials are added on purpose.
- Mac support is now in the app's codebase. It has a real Mac app identity for permissions,
  starts through the same command system as Windows, can show the goose through Mac desktop
  windows, can play sounds, can use Mac-owned note and meme windows, and reports permission
  problems clearly. Window-riding tricks stay gated behind Mac Accessibility permission. It
  has now passed hosted Mac app bundle and status smoke tests on Intel and Apple Silicon runners,
  but still needs a pre-approved Accessibility smoke test before the Mac readiness card is fully
  closed.
- There is a new status command and a Status page in the terminal settings screen. They show
  whether the goose is running, what platform and bundle mode it is using, whether Mac
  Accessibility is allowed or denied, which desktop tricks are available, and how many note and
  meme assets were loaded.
- The Mac app bundle can now be staged as a local personal-use app. The staging script builds
  Intel and Apple Silicon slices, combines them into one app, copies the assets, gives the app
  the stable permission identity, and signs it for local testing. Final disk images, notarized
  signing, and installers still come later.
- Linux no longer only says "not supported" when you start it. It now has the same local command
  channel foundation as Mac and Windows, and it can render through the older X11 desktop path or
  a reduced native Wayland path. X11 can sample and move the cursor and notice dragged windows
  while protecting common terminal apps. Native Wayland stays intentionally limited: it can show
  the goose and take commands, but cursor stealing and window tricks report unavailable instead
  of pretending they work.
- There is now a real CI smoke-test setup for Mac and Linux backend readiness. It builds the Mac
  app, checks the app identity/signing/universal build, launches it, asks it for status, pokes it,
  reloads it, and stops it. On Linux it runs visible X11 under a virtual display and reduced
  Wayland under headless sway, then checks both the app's frame output and the actual X11 screen
  capture for visible pixels. The hosted Linux X11 and reduced Wayland readiness cards are now
  closed from that CI proof; the Mac card stays open only for the Accessibility-granted behavior
  that hosted runners cannot provide.
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
- Installer, uninstaller, updater, and setup words are now recognized by help, and the
  Windows/Linux installer and updater behavior has started landing as real behavior.
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
- The project guidance and task board now show the Windows/Linux installer milestone as done, with
  the remaining Mac Accessibility proof kept as its own separate open task.
- The Linux backend foundation now has its own architecture record. It says Linux should use the
  more capable X11 path by default, only use native Wayland when asked or when X11 is not
  available, and keep unsupported tricks disabled instead of pretending they work.
- The Mac/Linux readiness work now has its own architecture record. It says the remaining backend
  cards are closed by CI evidence, not local guesses from Windows, and records the optional
  self-hosted Mac Accessibility check for permission-granted behavior.
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
