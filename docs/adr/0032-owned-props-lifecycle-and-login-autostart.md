# ADR 0032 — Owned Props, Explicit Lifecycle, And Login Autostart

- Status: Accepted (2026-07-18)
- Relates to: ADR 0003 (collect-window behavior), ADR 0006 (durable config), ADR 0015
  (platform safety), ADR 0016 (off-screen locomotion), ADR 0028 (shared graceful Quit), ADR 0030
  (tray parity), and ADR 0031 (installation provenance and receipts).
- Supersedes: ADR 0003's Windows external-Notepad process and synthetic typing implementation,
  plus ADR 0015's corresponding Notepad-child cleanup rules. The collect task, note content,
  native user-close provenance, bounded annoyed reaction, external user media, and explicit Linux
  unsupported boundary remain accepted.
- Partially superseded by: ADR 0033 replaces only the Windows login-start command and the
  interactive-local calibration-notice decision.

## Context

Windows 11 can restore an existing Notepad session when a new Notepad process is launched. During
qualification this surfaced a user-owned `npm.ps1` tab even though Honk300 neither selected nor
executed it. A goose prank must not touch, focus, restore, type into, or become responsible for a
user's editor state. The same qualification exposed an image window that used raw source pixels
and dominated a 1920×1080 monitor. Props should be noticeable and readable, not full-screen, and
images must never be cropped or distorted.

The paired Windows transparency smoke intentionally covers the virtual desktop with dark and
near-white calibration surfaces. That diagnostic surface was mistaken for a product regression
because it appeared before a clear product-owned cue. Separately, ordinary stop and native Quit
already use an engine locomotion state, but there was no explicit immediate-stop escape hatch.

Installers already offered optional login start, but durable configuration did not. Creating a
second startup task from config would introduce competing owners and break ADR 0031 provenance.

## Decision

### Owned, monitor-relative collect props

Windows note collection uses a Honk300-owned native top-level window containing a child edit
control. It never launches Notepad, sends global input, or enumerates/restores editor sessions.
The window is tracked by its own process and exact class, and only its native close event may
count as a user closing a collected note. macOS retains its owned AppKit note window. Linux
continues to report collect windows unsupported.

The platform-free engine owns the sizing policy. Each prop is fitted against the physical bounds
of the monitor currently receiving the goose:

- a note targets 32% of monitor width and height, with readable lower targets but an absolute
  48% ceiling in either dimension;
- an image is uniformly downscaled to at most 48% of monitor width and height and at most
  900×700 logical pixels;
- image aspect ratio is preserved, the complete source rectangle is resampled, no edge is
  cropped, and an image is never enlarged above its natural dimensions.

Windows and macOS consume the same fit result. Positioning may remain playful, but window bounds
must remain within the selected monitor. These ceilings are hard safety boundaries, not target
sizes for already-small content.

### Arrival, graceful exit, force, and calibration notice

Every normal launch stages the complete pose beyond a genuinely exposed edge of the current
monitor topology and walks it into frame. Touching monitor seams are continuous and cannot be
used as fake outside edges. Ordinary `stop`, `quit`, and `exit`, including tray/menu Quit, enter
the shared engine-owned walk-off and wait until the complete pose is outside the desktop.

`honk300`, `honk`, and `goose` accept `stop|quit|exit --force`. That command uses a separate local
IPC message and terminates the runtime immediately without the walk-off. Native tray/menu Quit
never implies force.

Any interactive calibration or compositor diagnostic that will obscure a user's desktop must
first start the exact product binary, wait until the goose is visible, and have it deliver an
owned note explaining the imminent calibration. The diagnostic still requires explicit local
consent. Disposable CI desktops may run automatically, but production startup never creates a
calibration background.

### One provenance-owned login-start preference

Schema-current TOML adds `[lifecycle].autostart_on_login`, default `false`, and the General TUI
exposes **Start on login**. Saving applies the choice synchronously and truthfully through the
active install's existing owner:

- Windows Global MSI/EXE and PowerShell origins use the machine `Run` value and request elevation;
  Corporate MSI/EXE and manual per-user installs use the user `Run` value.
- The managed macOS app uses its owned LaunchAgent.
- shell-managed and Debian Linux use the current user's owned XDG autostart desktop entry.

The value always invokes the stable unversioned `honk300` path with `start`. Foreign values,
symlinks, unexpected file types, ambiguous provenance, or missing managed identity are refused;
config never creates a second persistence mechanism. Receipts record the installer-selected
state. When an installer receipt is newer than config, that fresh installer choice is the user's
latest intent and is mirrored into config. Otherwise an explicit config edit wins and updates
the owned mechanism and receipt. Package updates preserve their selected feature/task state.

## Consequences

- A note prank cannot expose or alter a user's Notepad tabs and cannot accidentally treat a
  script as something to open or execute.
- Large and unusual images remain completely visible and aspect-correct; small images are not
  made comically large just to fill a quota.
- Normal lifecycle remains expressive and observable, while automation and recovery retain one
  explicit immediate-stop command.
- Login start is one real cross-platform setting without registry/task/LaunchAgent duplication.
  Machine-owned Windows changes may legitimately show one UAC consent prompt.
- Installer intent and configuration no longer silently overwrite one another based only on
  semantic version or stale defaults.

## Verification contract

- Shared geometry tests cover landscape, portrait, panoramic, tiny-display, small-natural-image,
  ceiling, aspect-ratio, and no-upscale cases. Native Windows evidence records source, fitted,
  monitor, and HWND dimensions and captures the complete visible image.
- Windows note qualification proves the exact owned class/PID, readable monitor-relative bounds,
  expected text hash, movement, user-close tracking, and absence of a Notepad child.
- Lifecycle qualification visibly proves offscreen entry, ordinary CLI and native graceful exit,
  and every command-name/stop-synonym force combination. The product process must be absent at
  completion.
- Interactive calibration proves the explanatory note precedes the first obscuring background;
  ordinary launch separately proves transparent presentation with zero calibration surfaces.
- Config tests prove default-off parsing, save/load round-trip, TUI toggling, failed-reconcile
  truthfulness, installer-newer precedence, foreign-owner refusal, and the platform owner selected
  for each receipt family. Candidate installers verify startup selection and update preservation
  across all Windows families and both architectures; hosted macOS/Linux gates exercise their
  owned integration paths.
