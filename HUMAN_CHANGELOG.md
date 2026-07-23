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
> CI smoke proof for hosted Mac bundle checks and Linux desktop behavior. The installed Mac app
> now has a calm, non-nagging permission handoff. One unchanged signed copy passed denied,
> repeat-denied, granted, and revoked behavior on the physical Mac; a later fresh-release repeat
> remains visible follow-up work. While running, the Mac app also has a small goose menu that
> opens the same terminal settings screen or starts its animated goodbye.
> Windows and compatible Linux desktops now show the same small goose control as the Mac: it
> opens the one terminal settings screen or starts the Goose's animated goodbye.
> Every desktop now stages the
> Goose's arrival and departure beyond a real screen edge, and a person closing its note or meme
> can provoke a safely bounded annoyed reaction. The Windows/Linux installer
> and update
> work now has release artifact proof, including Windows installers for both regular x64 and ARM64
> machines.

---

## Latest — July 2026

### Fixed
- **Windows updates no longer stumble over a backslash while preparing their safety helper.**
  The failed update stopped before changing the installed Goose, and the repair keeps the same
  checks against unsafe archive paths. This is a new patch rather than a rewrite of already
  published downloads.

### Improved
- **Mud is now an occasional mess instead of a near-constant one.** The Goose waits roughly
  three to five minutes between natural puddle trips and tracks new footprints for only 10–30
  seconds after returning. Asking for mud directly also uses a shorter fresh default, while a
  duration someone already chose remains theirs. Existing prints still fade naturally, so the
  trail ends gently instead of disappearing.

### Changed
- **The updater's final instruction now looks like a final instruction.** A blank line and a
  prominent text box separate the result from **You may now close this window.** Successful and
  no-change checks say **HONK! ALL DONE**; a problem says **HONK! NEEDS ATTENTION** and keeps the
  useful error details above it. The box stays readable even when terminal colors are unavailable.

### Added
- **The Goose can now update itself from its tray menu.** Windows, Mac, and compatible Linux
  desktops show an **Update Honk300…** choice beside settings and quit. Clicking it opens a
  terminal and does the complete verified update without asking the person to type a command.
  Opening the menu alone does not contact the internet.
- **Updates finish with a clear, friendly result.** After installing a newer copy, Honk300 starts
  again and the terminal clears to say the update is complete. If nothing changed, it picks one of
  100 small goose messages but always plainly says there was nothing to update, the Goose is
  current and running, and the window can be closed. Failures leave the useful details on screen
  and say whether the installed Goose was successfully brought back. The window stays open until
  the person closes it, so terminal settings cannot make the result vanish unexpectedly.

### Improved
- **The convenient button keeps the same update safety as the command.** It remembers whether the
  Goose came from a Windows installer, Mac app, Linux package, or terminal install; checks the
  exact download and owner; prevents two update windows from racing; and restarts only the copy
  named by the verified installation record. Script-friendly update output is unchanged.

### Added
- **The easiest install is now one official command on every platform.** Windows shows the
  PowerShell command first, while Mac and Linux show the terminal command first. These are real
  managed installers that check the exact download, remember their ownership, and verify every
  Goose command name, so the convenient path keeps the same safety as the downloadable packages.
- **Release checks now prove that choosing another official installer really changes ownership.**
  Windows tests move through the PowerShell, all-users, workplace, and executable installers while
  an older Goose is still running. Mac tests move between the terminal and disk-image choices,
  and Linux checks refuse unsafe package collisions before touching the active copy.

### Improved
- **The Windows start command now returns from app-owned command runners too.** Some integrated
  terminals keep every descendant inside one Windows job even after the terminal command itself
  finishes. The quiet Goose launcher now steps the long-running Goose out of that job when Windows
  permits it, so the command runner becomes available again while the Goose and tray controls stay
  alive. Managed workplace jobs that forbid this still use the original safe background launch.
- **The official Windows install command now works from app-hosted terminals too.** Some terminal
  apps intentionally leave out the usual processor label. The installer now asks Windows itself
  which kind of computer it is before choosing the x64 or ARM64 download, so the same documented
  command works without setting a hidden environment value by hand.
- **Checking for updates on Windows no longer asks for protected-folder access too early.** A
  normal terminal now leaves an already-correct install record alone while it checks for a newer
  Goose. Windows can still request approval when the verified installer actually needs to make
  system changes.
- **Starting the Goose on Windows no longer ties it to that terminal window.** The command now
  returns after the Goose is genuinely ready, and closing the PowerShell or terminal that started
  it leaves both the Goose and its tray controls running. Shortcuts, start-at-login, and all three
  friendly command names use the same quiet app launch path; the terminal settings screen still
  opens as a normal terminal when requested. Rapid restarts also wait briefly for Windows to
  finish retiring the old tray item instead of coming back without controls.
- **The Windows app launcher now has its own Goose icon.** Its shortcut is easier to recognize and
  can be pinned by the user, while the running Goose stays unobtrusively in the notification area
  instead of holding a taskbar window.
- **A fresh Mac terminal install now means what the user just asked for.** Running the public
  command deliberately makes the terminal installer the current choice. An ordinary update still
  remembers whether the app originally came from the graphical disk image or the terminal, and
  both managed choices keep the same calm permission onboarding. Macs updating from the previous
  release also retain that choice even though their older updater predates the new signal.
- **Downloadable installers remain first-class options.** The graphical Mac download, Windows
  installer choices, and Debian packages still handle their own future updates and removal. A
  raw developer build cannot silently take ownership away from one of them, which prevents an
  advanced command from leaving two competing copies.

### Fixed
- **The official Windows command no longer depends on an optional PowerShell hashing command.**
  It now checks downloaded and installed files with Windows' built-in cryptography libraries,
  keeping the same tamper protection on machines where that PowerShell command is missing. This
  matters because the convenient one-line installer should work anywhere the supported Windows
  PowerShell runtime works, not only on systems with one extra command available.
- **The Mac public-release checker now recognizes both legitimate installation writers.** The
  terminal installer and graphical app use different internal owner labels for the same protected
  start-at-login setting. The checker still requires the setting to stay off around an unrelated
  startup file, but no longer reports a successful signed Mac takeover as a failure just because
  the trusted writer changed. This lets the full public-byte gate describe the real install state.
- **The Windows public-release checker no longer asks two update coordinators to hold the same
  lock.** The official PowerShell installer now gets to own its safety lock during the test, while
  a separate check still proves an older running Goose survives release-slot changes on both PC
  architectures. Installer output is also saved so any future failure explains itself instead of
  ending with only a generic exit code.
- **A flaky hosted Windows desktop check no longer blocks an otherwise verified release.** The
  exception is allowed only when the test can still find the Goose's real control owner and a
  separate ordinary tray icon fails the same Windows check. The suite continues testing the real
  tray menu, graceful goodbye, immediate force quit, rendering, and download identity, while the
  normal Windows lane still requires the complete visible tray proof. This keeps the release gate
  honest without treating a broken hosted desktop shell as a product regression.
- **Opening the all-users Windows installer now records the choice the user actually made.** It
  no longer carries forward an older PowerShell-install label, so the next update stays with the
  most recently chosen official installer.
- **Linux stops before mixing a personal terminal install with a system package.** Either path
  now identifies the existing owner and gives the exact removal-and-retry instruction. The
  system package checks every Goose command name and never deletes another user's files, which
  keeps the current working copy safe until the user completes the channel change.
- **Release checks now recognize the same installation record in either valid layout.** The Mac
  and Linux public-install test reads the record's actual values instead of depending on spaces
  and line breaks, which lets the full update and rollback safety check finish reliably.
- **An unrelated startup file no longer prevents the Goose from launching.** Mac and Linux
  terminal installs now turn on start-at-login only when the existing startup entry genuinely
  belongs to the Goose. Unfamiliar files remain untouched and count as the Goose's setting being
  off, which keeps another program's setup safe without blocking normal launch.

### Added
- **Updates now remember exactly how the Goose was installed.** A protected installation record
  keeps the all-users, workplace, Mac, Linux-package, and terminal-install choices separate, while
  saved release copies let the command switch safely without overwriting the copy that is still
  running. This makes future updates predictable and preserves the user's chosen installation.
- **Windows update tests now keep the old Goose running during the switch.** The test also forces
  a failure halfway through, confirms the previous copy returns completely, and then checks every
  command name after a successful retry. This catches the lock and partial-update failures people
  actually experience.
- **Starting the Goose when you sign in is now a real setting.** The terminal settings screen has
  one default-off switch that uses the installation's existing Windows, Mac, or Linux login
  mechanism. It refuses unfamiliar startup entries instead of creating duplicates, which keeps
  the choice predictable and easy to undo.
- **There is now an explicit emergency stop.** Adding `--force` to stop, quit, or exit under any
  Goose command name closes it immediately. Leaving that flag off still lets it walk fully out of
  frame, so everyday exits remain friendly while automation and recovery have a clear escape.
- **Windows and compatible Linux desktops now have a Goose control icon.** While the Goose is
  running, the small icon can open the existing settings screen or ask it to walk away and quit.
  It uses the same accessible name and familiar two choices as the Mac, which makes control easier
  to discover without creating another settings app.
- **The control comes back when the desktop shell does.** Restarting Windows Explorer or a
  compatible Linux panel no longer permanently loses the running Goose's shortcut, which keeps
  the control useful through ordinary desktop restarts.
- **The Mac menu now looks like the Goose.** A compact goose silhouette replaces the temporary
  word in the menu bar, automatically follows the Mac's light, dark, and highlighted appearances,
  and still has a clear spoken label for people using accessibility tools. The original artwork
  is saved as one shared source so later Windows and Linux tray work can look and behave the same.
  This update does not pretend those other trays already exist.

### Improved
- **The update command now waits until the result is real.** It uses the same kind of installer as
  last time, checks the final copy before saying it worked, and can return one clean result for
  scripts. If ownership is unclear, it gives a safe reinstall instruction instead of making a
  potentially wrong system-wide choice.
- **The most recent successful install represents the user's current choice.** Running another
  official installer can deliberately move to an older release or another edition after its files
  are checked. Mac, supported Linux packages, and terminal installs keep their own familiar
  update experiences instead of silently changing installation methods.
- **Windows no longer hides a half-finished switch between all-users and workplace installs.** The
  new checked copy stays available, but removing an older all-users owner can still need an
  administrator's approval. If that approval is cancelled, the update reports that cleanup is
  pending and gives a safe retry instead of pretending the command now opens the new copy.
- **Notes and pictures stay noticeable without taking over the monitor.** Notes use a readable
  fraction of the screen. Pictures keep their full shape and content, shrink only when needed,
  and are never cropped, stretched, or blown up from a naturally small image. This keeps the
  prank mildly disruptive instead of blocking the desktop.
- **Login-start choices survive installs and updates honestly.** A fresh installer choice becomes
  the current preference, while a later settings change updates that same owned startup entry.
  Updates keep the chosen installer and startup behavior, which prevents stale defaults from
  quietly reversing what the user asked for.
- **Every desktop control follows one behavior.** Configure opens the exact running copy's one
  terminal settings screen, and Quit lets the Goose finish walking completely offscreen before it
  closes. This keeps platform menus from drifting into separate settings or abrupt shutdowns.
- **Linux packages cleanly own their menu artwork.** Supported package installs add the shared
  Goose icon and remove only that owned copy during uninstall, so personal notes, pictures, and
  settings remain untouched.
- **Per-user Windows upgrades recover cleanly from a failed attempt.** The previous working copy
  stays intact until the new copy is fully installed, so retrying or removing the app does not
  inherit broken ownership records.
- **Development terminals receive the same safety treatment across desktops.** The Goose now
  recognizes Codex and Visual Studio Code surfaces on Windows and Linux and leaves them alone,
  which protects integrated terminal work from window pranks.
- **The menu remains the same simple control, just more polished.** Configure still opens the one
  real terminal settings screen, closing that screen leaves the Goose running, and Quit still
  sends it walking completely offscreen before the process ends. Those learned reactions are now
  written down as requirements for every future tray, which prevents another operating system
  from gaining a separate settings model or a jarring instant disappearance.
- **The icon is safer on older Macs.** The app includes a small transparent image format that Macs
  have supported for years while keeping the reusable vector original for future platforms. If a
  developer copy somehow lacks the artwork, the old word appears instead and the app remains
  controllable. This keeps a cosmetic problem from blocking startup.

### Fixed
- **Windows updates no longer need the command that started them to disappear.** The installer
  moves a stable pointer to the checked new copy while the old command finishes normally, then
  confirms all three Goose command names work. This avoids the locked-running-file problem rather
  than trying to force the file closed.
- **Switching between all-users and workplace installs no longer leaves the old command ahead.**
  One quiet administrator-approved cleanup removes only the retired installation's startup and
  command entries, then confirms the new location wins. It recognizes the supported Windows
  Installer record even when Windows stores that per-user package in its protected system list,
  while still refusing user-made copies of all-users records. Folder names passed through the
  administrator prompt also stay intact when Windows records a final slash, and one supported
  record shown through two system views is not mistaken for two installed copies. This avoids a
  second prompt and makes the user's newest successful install the one their commands actually
  open.
- **Goose notes no longer involve Notepad at all.** The Goose owns its little editable note
  window, so Windows cannot restore an unrelated user tab and the program cannot accidentally
  surface or run a script. Closing the Goose's own note still behaves like closing the prank.
- **Windows app and login startup no longer flash or focus a terminal.** Shortcuts and automatic
  login startup use a tiny native background launcher instead of opening the command program or
  PowerShell visibly. Commands typed in a terminal still behave normally in that same terminal,
  and choosing Configure still intentionally opens settings; everything else stays out of the
  user's way.
- **A failed update no longer loses the previous installation record.** The rollback keeps the
  earlier record and command selection even when failure happens at an awkward instant, and a
  repeated Linux installation keeps its already-checked saved copy. This makes retries safe and
  truthful.
- **The Windows tray now opens a usable settings screen.** It starts a normal terminal with clean
  input and output instead of inheriting background redirection, which prevents an empty-looking
  settings window.
- **Desktops without a compatible tray remain fully usable.** The Goose explains that the shortcut
  is unavailable while command-line controls, settings, and supported desktop behavior continue
  normally. This avoids pretending every Linux panel or unusual Windows session provides a tray.
- **Checking for updates now works in built-in Windows shells.** Download addresses and file
  locations are passed safely, and response text is decoded consistently, so update discovery no
  longer stops at a command-parsing error. Windows users with an older copy need to rerun the
  current installer once because the already-published old copy cannot repair itself; settings
  and personal content remain in place.
- **Two supported Windows installs no longer confuse each other's maintenance commands.** The
  copy whose command you invoked keeps its own update and removal identity, even when both the
  all-users and per-user editions exist.
- **Background removal helpers no longer fail just because the original command already closed.**
  A normal timing race is treated as completion while real failures still stop safely.
- **Windows update records stay current without taking ownership of unfamiliar files.** A record
  created by the official all-users installer is refreshed only after the new app is verified;
  missing, damaged, unrelated, or redirected records are left untouched.
- **Windows command startup no longer mistakes a brief pause for an empty request.** If the Goose
  and a command connect just before the command bytes arrive, it now waits for the rest of the
  existing short deadline instead of rejecting an action that it actually received. It still
  gives up safely when a broken peer sends nothing, which keeps startup responsive and reliable.

### Behind the scenes
- **Screen-covering graphics calibration no longer runs on a person's Windows desktop.** The app
  never needs it, and local checks now refuse to create the old dark and near-white full-screen
  surfaces. Strict transparency proof runs only on disposable release machines, so development
  cannot blank the user's display or interrupt what they are doing.
- **Release checks now reproduce Windows shell restart ordering.** The test first proves the old
  icon stays gone before asking the Goose to restore it, then allows a bounded settling period and
  saves clearer timing details. One older hosted Windows shell may call the recovery unobservable
  only if a completely separate stock icon fails the same sequence; ordinary Windows testing and
  the Alienware still require the Goose icon itself to return. This prevents a runner limitation
  from hiding a real product failure.
- **Every finished Mac download must contain the menu artwork.** Release checks now look for both
  the reusable source and the Mac-ready image before signing, after packing the app, and in both
  inspections of the finished disk image. A missing icon stops publication instead of quietly
  shipping the developer fallback. Follow-up checks on the physical Mac also confirmed the real
  settings screen, permission transition, readable dark note, animated goodbye, quick restart,
  and low process-specific memory and processor use.
- **The newest goose-menu update is live across every supported desktop.** The complete release
  rehearsal, ordinary follow-up checks, Mac trust and graphical install, real in-place update
  through all three command names, repeat update, and public download hashes passed. The download
  page now shows one recommended option for the visitor's computer and reveals other systems or
  terminal choices only when requested. This makes the release easier to choose while preserving
  independently checked Windows, Mac, and Linux downloads.
- **The first stable release and its download page are live and independently checked.** The
  complete cross-platform rehearsal, ordinary follow-up checks, Apple trust checks, fresh Mac
  install and update, and the public download page all passed before the release board closed.
  Extra hands-on computer checks remain useful, but any issue they find will become a new update
  instead of silently changing the already published downloads. This keeps the release
  reproducible while still leaving a clear path for hardware-specific refinements.

### Added
- **Mac users now get a simple Honk menu while the Goose is running.** Configure opens the same
  terminal settings screen that already owns every setting, and Quit sends the Goose walking
  fully offscreen before the app closes. The menu disappears with the app and does not add a
  second preferences window, a running Dock control, or a tray to Windows and Linux. This makes a
  graphical Mac install easier to control without splitting settings or bypassing the animated
  goodbye. A local packaged app opened and restored the full settings screen and walked fully out
  when Quit was chosen; a later fresh-release repeat remains visible follow-up work.
- **Mac permission setup is now a calm, one-time handoff.** The officially installed Goose asks
  once for each installed update, opens the right Mac privacy page, and waits visibly near a safe
  screen edge instead of wandering off or attempting pranks. Honk, status, reload, and stop still
  work while it waits, and another denied launch does not reopen the page. Granting permission
  lets the same running Goose begin its normal introduction; taking permission away sends it
  safely back to the wait. Developer and test copies do not open permission pages on their own,
  which avoids surprise prompts and repeated nagging. The behind-the-scenes checks pass, and one
  unchanged signed copy also proved denial, a second denial, permission granted, and permission
  removed on the physical Mac. A fresh-release repeat is still tracked without pretending it has
  already happened.
- **Mac now has a real graphical installer path.** The disk image includes the Goose, a small
  “Install Honk300” app, and short instructions. The helper confirms both apps came from the same
  fixed Developer ID application certificate, installs only for the signed-in user without an administrator password,
  explains failures in a normal Mac dialog, and opens the installed Goose. This makes the Mac
  download approachable without creating a second, inconsistent install system.
- **Debian and Ubuntu now have native packages for regular and ARM computers.** Each package
  installs all three Goose command names through the operating system's package manager while
  leaving personal notes, memes, and settings in the user's own folders. Updating and removing a
  package checks that it really belongs to Honk300 first, preserves personal media by default,
  and backs it up before a full cleanup. Real release checks install and run both architectures,
  which makes the new download more than a renamed archive.
- **Mac release builds now require Apple's trust checks.** Automated release work signs the app
  and installer with hardened settings, sends both the app and disk image to Apple, attaches the
  approval tickets, and checks that Gatekeeper accepts them. Missing credentials stop the build
  instead of quietly publishing a weaker download, which keeps the promised trust level honest.
  The finished disk image is opened once more and both contained apps are checked exactly where
  people will launch them, so a trusted container cannot hide an untrusted installer. Each app's
  actual executable is sealed before its outer app bundle, making the trust chain explicit.
- **Mac drawing now has native color and visibility checks.** Tests send deliberately different
  red, green, blue, and transparency values through the real Mac drawing stack and check that all
  recognizable goose parts remain visible on light and dark desktops. This catches the exact
  class of bug that produced the washed-out blob.
- **Every desktop now guards its final color handoff.** Windows and both Linux display paths now
  test deliberately different color and transparency values too. That makes the same washed-out,
  transparent, or color-swapped failure much harder to reintroduce anywhere.
- **Linux checks now inspect what the desktop really shows.** The test keeps one exact Goose
  build unchanged and captures both Linux display paths after they have finished drawing. It
  requires the pale body, dark wing, and distinctly orange beak and feet to remain recognizable,
  and refuses an unfamiliar screen color layout instead of risking swapped or transparent
  colors. This makes Linux release proof match the real view people will see.
- **Windows checks now inspect the real transparent desktop Goose.** The test keeps one exact
  Windows build unchanged, holds the same pose over dark and light desktops, and checks the body,
  shading, outline, wing, transparency, correct orange colors, and softly blended edges. A side
  pose must also show its beak, two legs, and soft shadow; a top-down pose must show its compact
  beak and complete overhead body-and-wing shape without pretending it has visible legs. It also
  proves start, status, reload, stop, and an immediate restart, then keeps the
  pictures and logs when anything fails. The release gate runs this on both regular Windows PCs
  and native Windows-on-ARM hardware, using the same ARM file headed into its installer, then
  repeats it for the published regular-PC installer. This catches visual handoff bugs and everyday
  control problems before release.
- **The Goose can now take the scenic route around real screen edges.** Screens that touch still
  behave like one continuous desktop. Every so often the Goose may walk completely beyond a truly
  exposed edge and come back from the other side, but the switch happens only while every part of
  it is hidden. Trips to fetch mud or return with a prank still come back through the same edge.
  Starting and stopping also happen as visible walks from or into an edge, so the Goose does not
  simply pop into or out of existence. This behavior is shared by every desktop version.
- **Closing one of the Goose's notes or memes can offend it.** When you personally close a window
  it opened, there is about a three-in-ten chance it gets visibly annoyed and then tries the same
  short, bounded mouse-stealing prank it already knows. Cleanup does not provoke it, and the mouse
  part still respects your settings, quiet/fullscreen manners, permission, and platform support.
  Linux does not currently open those windows, so it has no close reaction there.

### Improved
- **This is now the first stable Honk300 release.** Every download, installer, update record, and
  website check moves to the same stable baseline, including upgrades from the previous public
  build. The first publication attempt stopped safely before making any download public, so the
  corrected release moves forward as a new patch instead of reusing that frozen tag. Extra
  hands-on Windows checks will continue afterward, and anything they find will be repaired in a
  new update instead of silently replacing already published files. This makes the release
  identity clear and keeps downloads reproducible.
- **One stable download address now follows every complete release.** The familiar Mac disk
  image, Windows installers, Linux packages, and terminal installers keep the same latest links,
  while every older release and its files stay frozen. A release started from any computer still
  asks GitHub's Mac machines to make and pass Apple's checks on a fresh Mac download. Installed
  copies discover the newest release, then fetch and verify the exact matching file for their
  operating system, processor, and install type before changing anything. The Mac disk image is
  for graphical installs; command-line updates use the signed app package behind the same release.
- **Walking looks less stretchy everywhere without becoming frantic.** Planted feet release a
  little sooner, while only the fastest running and charging strides shorten further. Normal
  walking keeps its weighted rhythm, and checks prevent rapid bicycle-like stepping. The same
  shared animation runs on Windows, Mac, X11, and Wayland.
- **Mac drawing does less repeated work.** The renderer reuses drawing space and shadows, while
  the Mac app reuses native images, remembers screen geometry, and limits display updates without
  slowing the simulation. After a temporarily large note or meme interaction, its drawing space
  returns to a sensible size instead of making every later walking frame process a mostly empty
  screen-sized image. The incoming pixels keep their explicit drawing color format, while the Mac
  window uses a stable standard screen-color destination and lets the operating system handle the
  final physical display profile. This avoids a repeated conversion on every frame and passed a
  clean low-processor diagnostic; a fresh-release repeat remains follow-up verification. It saves
  processor and battery work without changing the Goose's colors, motion, or update rate.
- **A mounted disk image can use the normal safe installer.** The Goose can now copy its own
  verified app bundle into the user's Applications folder while preserving aliases, autostart,
  user media, updates, removal, and rollback. Automated release checks can interrupt the swap on
  purpose and prove the previous app returns. A failure late in setup also puts command aliases,
  login startup, install records, and newly copied starter media back exactly as they were
  without touching existing personal media. This keeps graphical and terminal installs under
  the same ownership rules.
- **Mac permission tests can hold one identity still.** The smoke checks can reuse an exact built
  app and no longer create the settings file prematurely. That lets denied and granted
  Accessibility checks happen without rebuilding the app between them.

### Fixed
- **A valid overhead Goose no longer blocks an otherwise healthy release.** The first publication
  attempt happened to photograph only complete top-down poses, while the Windows checker demanded
  side-view legs and a ground shadow. The checker now recognizes either full drawing view and
  still reconstructs softly blended edge colors so washed-out, double-faded, swapped, clipped, or
  black-background pictures fail. Incomplete entrance frames still do not count. This removes
  random pose timing from publication without changing how the Goose looks or moves.
- **The Linux screen test no longer inherits a test machine's wallpaper.** The complete release
  rehearsal passed, but the ordinary follow-up check installed a fuller desktop package whose
  default wallpaper and top bar stayed on one virtual monitor. The Goose was correctly
  transparent and revealed that system picture; it was not adding a background of its own. The
  check now starts a tiny private desktop configuration with no catch-all wallpaper, paints each
  virtual monitor with a small fully opaque color tile that cannot fade under fractional scaling,
  and photographs both colors before the Goose launches. This prevents either a runner decoration
  or a distorted test color from being mistaken for app output while keeping every transparency
  and body-part requirement.
- **Release checks now recognize a real top-down Goose without excusing broken pictures.** One
  Linux rehearsal showed a clearly visible body and wing with a smaller orange beak-and-feet
  footprint than a side view, so the check now accepts that genuine pose while keeping every
  background, transparency, and solid-rectangle safeguard. A separate hosted Windows-on-ARM
  machine acknowledged two colored test windows but returned the same wallpaper for both
  screenshots. Only that exact hosted limitation can use the pixels the real Goose successfully
  handed to its visible window, and those pixels must still prove correct transparency, color,
  body parts, shadow, and the matching live window. Regular Windows and real ARM machines still
  need the full two-background desktop pictures. This keeps the release gate honest without
  confusing a test-machine limitation for a product failure or calling it proof that was not
  seen. The next rehearsal produced several completely valid Goose surfaces there, but the
  moving window shifted slightly between the app recording its successful draw and the test
  freezing it. The check now accepts only that tightly bounded single-frame shift on the exact
  same window; all picture, transparency, body-part, machine-identity, and regular desktop-picture
  requirements remain unchanged. This fixes a timing race without making a broken picture pass.
- **The hosted desktop checks now use safe temporary paths on Linux and read Windows line endings
  correctly.** The first major-release rehearsal successfully photographed the new Linux test
  background and proved matching Windows screen geometry, then stopped because Linux's socket
  filename was too long and Windows read its multi-line report as the wrong shape. Linux now uses
  a short private socket folder and removes it afterward; Windows accepts both standard line-
  ending styles while still demanding an exact geometry line. The Goose's drawing standards were
  not loosened, so these fixes let the real checks run instead of changing what counts as a pass.
- **Windows and Linux release screenshots now prove their test desktop before judging the Goose.**
  A later release rehearsal showed that Linux was photographing a compositor's remembered gray
  tile and Windows on ARM could miss both its colored test surface and the Goose because the two
  helpers disagreed about screen scaling and briefly fought over one file. Linux now keeps a
  temporary colored window alive behind the Goose only inside its fake test desktop. Both Windows
  helpers agree on physical screen coordinates, exchange each color safely, and prove real dark
  and light pictures before the Goose starts. Piping status into a command that stops reading is
  also treated as a normal finish, while genuine output errors still fail. None of the required
  body-part, color, shadow, or transparency standards were loosened, and the repaired source must
  still pass one final hosted run.
- **The Goose's beak is being aligned with windows it fetches.** A live Mac run found that its
  body could reach the center of a note while its beak—the part that actually has to grab it—was
  still too far away, leaving the behavior stuck. The Goose now adjusts where its body is headed
  as its beak moves, and a lifelike timed check walks all the way through grabbing and typing
  without moving the beak by hand. The full local test set passes, and a signed copy created and
  typed a readable note. A fresh-release picture of exact beak contact remains follow-up evidence.
- **An older Goose note or meme no longer blocks the next one.** Mac and Windows now keep track of
  which new window the Goose is actively fetching, even when an earlier note stays open. Closing
  an older window is still noticed once, but it cannot hide the current window and make the new
  behavior give up.
- **The Mac release checklist now matches the download people will actually use.** It starts with
  the signed and Apple-approved disk image and keeps one unchanged Goose through permission,
  window-safety, appearance, performance, update, cleanup, and fresh-download checks. It also
  separates the before-release and after-release gates, which helps prevent an older or weaker
  Mac download from being promoted by mistake.
- **Goose notes stay readable in Mac dark mode.** Note lettering now follows the Mac's chosen
  appearance and accessibility contrast instead of always being black. Windows notes already use
  the system Notepad's colors, and Linux does not currently offer the note-window prank, so the
  same hidden contrast problem is not present there.
- **The Goose is no longer a transparent white-and-purple blur on Mac.** The Mac window now reads
  the renderer's color and transparency bytes in the format they were actually produced, so the
  body, wing, beak, legs, outline, and shadow appear as intended. The underlying shared Goose art
  did not need to be replaced.
- **Mac screenshots and screen sharing use the real transparent Goose window.** The overlay now
  goes through the normal Mac window drawing path, so capture tools no longer lose the Goose or
  replace unused transparent space with large black blocks. Returning the drawing space to normal
  after a big interaction also keeps ordinary movement from wasting processor time. The walking
  rhythm and leg animation are unchanged by this Mac-only presentation fix.
- **Mac-only build warnings are cleared.** Old and unnecessary platform paths were removed or
  corrected, which keeps strict build checks useful instead of hiding new mistakes among known
  warnings.
- **Stop now means fully stopped.** Starting again immediately after a stop could occasionally
  catch the old Goose halfway out, refuse the new launch, and then leave no Goose running. Stop,
  exit, and quit now let it run to a real screen edge, walk fully out of view, and only then finish
  a bounded shutdown on every desktop. Quick restarts stay dependable while a genuinely stuck
  exit is still reported.
- **Installing, updating, or removing the Goose can no longer race a new launch.** Every desktop
  now keeps exclusive ownership from the moment the running Goose finishes walking out until all
  owned files are safely changed. The Windows updater uses the exact verified portable download
  to hold that ownership and keeps downloaded files locked against replacement while they run.
  Windows also checks for other signed-in sessions using the shared installation, rejects updates
  that Windows wants to postpone until a reboot, and stops any temporary removal helper that has
  not safely taken ownership. Mac and Linux installers now roll back on interruption instead of
  accepting a half-finished change. If shutdown or ownership cannot be proven, nothing is changed.
- **Two old performance follow-ups are now formally closed.** The three desktop versions already
  share the same timing rules, and Windows redraws only the small changed area on each affected
  monitor. Fresh checks confirmed both designs, which reduces release risk and keeps the board
  honest about work that is truly finished.
- **The Wayland roadmap now says exactly what can work where.** A normal Wayland app still cannot
  safely control every other window or the system pointer, but future user-approved integrations
  can add selected abilities on KDE, GNOME, and some other desktops. This keeps today's reduced
  mode truthful while turning vague “full support” into clear, testable choices.

### Security
- **Private release credentials cannot be added to Git by accident.** The local folder used for
  one-time Apple signing and approval files is ignored and kept owner-only, while automation gets
  its copy through encrypted repository secrets. This keeps release keys out of source history.
- **The graphical installer is tightly bound to the official app beside it.** It refuses a
  different app, mismatched developer, unsafe link, foreign command alias, or unowned receipt
  before changing anything. The install record also remembers the exact release and source
  revision, which helps future updates replace only what Honk300 owns. Graphical and terminal
  installs will not replace an existing app unless that app has Honk300's matching install record.
  Even a broken link left by another tool is treated as somebody else's file and kept untouched,
  which prevents a failed destination from becoming accidental permission to overwrite it.
- **Mac editor terminals stay out of reach.** Codex and Visual Studio Code are now protected as
  whole apps alongside Terminal, Ghostty, iTerm, Warp, and other terminals. This conservative
  boundary matters because it guarantees the Goose cannot mistake an embedded terminal panel for
  an ordinary draggable window.

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
