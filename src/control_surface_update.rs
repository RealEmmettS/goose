//! Visible, out-of-process updater launched by the native tray/status menu.
//!
//! The runtime must stay free to service the updater's graceful Stop request, so native control
//! surfaces launch this private command in a terminal instead of updating on their event thread.

use crate::update::{self, UpdateOutcome};
use crossterm::{cursor, execute, terminal};
use honk_control::{send_command, ControlCommand, ControlResponse, SingletonStatus, UpdateGuard};
use std::error::Error;
use std::io::{self, Write};
use std::path::Path;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

type DynError = Box<dyn Error>;

const READINESS_TIMEOUT: Duration = Duration::from_secs(10);
const READINESS_INTERVAL: Duration = Duration::from_millis(100);
const CLOSE_WINDOW_PANEL: &str = "\n\
+------------------------------------------+\n\
|              HONK! ALL DONE              |\n\
|     You may now close this window.       |\n\
+------------------------------------------+\n";
const ATTENTION_CLOSE_WINDOW_PANEL: &str = "\n\
+------------------------------------------+\n\
|          HONK! NEEDS ATTENTION           |\n\
|     You may now close this window.       |\n\
+------------------------------------------+\n";

/// Exactly one hundred deliberately small bits of goose flavor. The explicit invariant lines in
/// `up_to_date_text` carry the actual state, so none of this copy can make the result ambiguous.
const NO_UPDATE_FLAVORS: [&str; 100] = [
    "The update pond is already perfectly still.",
    "The goose inspected the version sign and found it freshly honked.",
    "No new feathers were waiting in the download nest.",
    "The release pond has no newer ripples today.",
    "This goose is already wearing its newest tiny boots.",
    "The update breadcrumbs led right back to this version.",
    "Every honk is already on the latest frequency.",
    "The goose checked twice, then checked the snack drawer.",
    "Nothing new hatched from the release egg.",
    "The version feathers are already neatly preened.",
    "No upgrade migration was required across the pond.",
    "The goose found zero newer bits and one imaginary sandwich.",
    "Your installation is as current as a very punctual goose.",
    "The update nest is empty, clean, and oddly organized.",
    "All available honks are already installed.",
    "The goose consulted the manifest and nodded solemnly.",
    "No newer goose emerged from the release reeds.",
    "The updater found everything exactly where it should be.",
    "Your goose has achieved peak present-tense.",
    "The version pond reports calm conditions and no upgrades.",
    "No fresh release crumbs were found on the server path.",
    "The goose compared versions without starting any drama.",
    "This installation has already caught the latest train.",
    "The release bell had nothing newer to ring about.",
    "The goose performed an update dance, purely ceremonial.",
    "No patches were hiding beneath the lily pads.",
    "The current version won a very short footrace.",
    "The updater returned with an empty basket and good news.",
    "Your current version is already leading the newest flock.",
    "The manifest says your goose is already future-proof enough for today.",
    "No update was necessary; the goose remains smugly current.",
    "The release canal contains no boats newer than this one.",
    "Your goose and the latest tag are already best friends.",
    "The updater found no work and has begun quietly loafing.",
    "The latest version is already nesting on this computer.",
    "No newer honk package answered the roll call.",
    "The goose checked the latest tag and then its reflection.",
    "Your version is already in season.",
    "The release shelf contains nothing newer to borrow.",
    "This goose is fully current and partially mischievous.",
    "The update compass points directly at the version you have.",
    "No new build has waddled onto shore.",
    "Everything is current; the goose has filed the paperwork.",
    "The version pond and your computer are in perfect sync.",
    "No newer bundle was found behind the curtain.",
    "The goose confirmed the current version with unnecessary confidence.",
    "Your installation already knows the latest secret handshake.",
    "The updater found no missing feathers to attach.",
    "The release goose and your goose are the same goose.",
    "No update today; the bits are already behaving themselves.",
    "The latest manifest gave this installation a gold star.",
    "Your goose is current enough to correct the calendar.",
    "No new package has landed on the runway.",
    "The update expedition returned before needing trail mix.",
    "The goose found the newest version already at home.",
    "No additional honk technology was available.",
    "The release stream is quiet and your goose is current.",
    "The updater counted every version and stopped at yours.",
    "No fresh patch notes were lurking in the cattails.",
    "The goose remains exactly as updated as possible.",
    "The version check produced one satisfied little nod.",
    "No new release has crossed the tiny drawbridge.",
    "Your installation and the server agreed immediately.",
    "The updater found no newer crumbs to follow.",
    "This goose is already at the front of the version parade.",
    "No newer patch called; all feathers stayed comfortably put.",
    "The release oracle replied: already current.",
    "The goose checked for updates and discovered inner peace.",
    "No newer version has arrived at the station.",
    "The current version is still the head goose.",
    "The update basket was empty, but pleasantly so.",
    "Your goose is already fluent in the latest honks.",
    "No replacement bundle was waiting in the wings.",
    "The manifest and your current version are happily aligned.",
    "The goose found nothing to install and everything to approve.",
    "No version leap was needed across this puddle.",
    "Your installation is already perched on the latest tag.",
    "The update radar detected only ordinary goose activity.",
    "No newer package was found in the migration lane.",
    "The goose has all currently available upgrades equipped.",
    "The release gate opened onto the version already installed.",
    "No patch came knocking, so the goose checked the windows.",
    "Your version remains undefeated by newer versions.",
    "The updater found no reason to ruffle a single feather.",
    "The latest build is already waddling around your desktop.",
    "No new release artifacts were waiting to be adopted.",
    "The goose compared version numbers and declared snack time.",
    "Your installation is current and lightly honked.",
    "No update was hiding in the reeds after all.",
    "The release trail ends exactly where your goose stands.",
    "Every announced version is already accounted for.",
    "The updater found the cupboard stocked with the current version.",
    "No newer goose could be located at this time.",
    "Your installation passed the freshness sniff test.",
    "The version check is complete; no waddling was required.",
    "No update cargo arrived at the little goose harbor.",
    "The goose is already running the newest approved mischief.",
    "Your current version and latest version are wearing matching hats.",
    "The updater has nothing to add except this sentence.",
    "All systems are current. The goose remains goose.",
];

pub(crate) fn run() -> Result<(), DynError> {
    let (guard, status) = match UpdateGuard::acquire() {
        Ok(acquired) => acquired,
        Err(error) => {
            eprintln!("{}", guard_acquisition_failure_text(&error));
            keep_terminal_alive();
        }
    };
    if status == SingletonStatus::AlreadyRunning {
        // On Windows an already-existing mutex still returns a non-owning handle. Do not retain
        // that handle while this informational window stays open: if the real helper closes, a
        // stale duplicate window must not keep the named object alive and block the next update.
        drop(guard);
        render_cleared(&duplicate_text());
        keep_terminal_alive();
    }
    let _guard = guard;
    // Preserve the pre-transaction receipt owner as a recovery candidate. Successful activation
    // always resolves again, but a cancelled elevation or interrupted install may temporarily
    // make post-failure discovery unavailable while the retained old slot is still runnable.
    let recovery_executable = update::authoritative_installed_executable().ok();

    let result = update::run_for_control_surface();
    match result {
        Ok(UpdateOutcome::Updated) => match ensure_runtime_ready(None) {
            Ok(()) => render_cleared(&updated_text()),
            Err(error) => {
                eprint!("{}", updated_restart_failure_text(&error.to_string()));
            }
        },
        Ok(UpdateOutcome::UpToDate) => match ensure_runtime_ready(recovery_executable.as_deref()) {
            Ok(()) => render_cleared(&up_to_date_text(selected_no_update_flavor())),
            Err(error) => {
                eprint!("{}", no_update_restart_failure_text(&error.to_string()));
            }
        },
        Err(error) => {
            let recovery_status = match ensure_runtime_ready(recovery_executable.as_deref()) {
                Ok(()) => "Honk300 is running from the authoritative installed release.".to_owned(),
                Err(recovery_error) => {
                    format!("Honk300 could not be restarted automatically: {recovery_error}")
                }
            };
            eprint!(
                "{}",
                update_failure_text(&error.to_string(), &recovery_status)
            );
        }
    }

    keep_terminal_alive();
}

fn runtime_ready() -> bool {
    matches!(
        send_command(ControlCommand::Status),
        Ok(ControlResponse::Status(status)) if status.running
    )
}

fn ensure_runtime_ready(recovery_executable: Option<&Path>) -> Result<(), DynError> {
    if runtime_ready() {
        return Ok(());
    }
    let authoritative = update::authoritative_installed_executable();
    let discovery_error = authoritative.as_ref().err().map(ToString::to_string);
    let candidates = restart_candidates(authoritative.ok(), recovery_executable);
    if candidates.is_empty() {
        return Err(discovery_error
            .unwrap_or_else(|| "no receipt-owned restart candidate exists".into())
            .into());
    }

    start_first_ready_candidate(
        candidates,
        discovery_error.as_deref(),
        READINESS_TIMEOUT,
        READINESS_INTERVAL,
        launch_authoritative_app,
        runtime_ready,
    )
}

fn start_first_ready_candidate(
    candidates: Vec<std::path::PathBuf>,
    discovery_error: Option<&str>,
    timeout: Duration,
    interval: Duration,
    mut launch: impl FnMut(&Path) -> Result<(), DynError>,
    mut ready: impl FnMut() -> bool,
) -> Result<(), DynError> {
    let mut errors = Vec::new();
    for candidate in candidates {
        match launch(&candidate) {
            Ok(()) => match wait_for_readiness(timeout, interval, &mut ready) {
                Ok(()) => return Ok(()),
                Err(error) => errors.push(format!(
                    "{} launched but did not become ready: {error}",
                    candidate.display()
                )),
            },
            Err(error) => errors.push(format!("{}: {error}", candidate.display())),
        }
    }
    let mut detail = errors.join("; ");
    if let Some(error) = discovery_error {
        detail = format!("authoritative release discovery failed ({error}); {detail}");
    }
    Err(format!("Honk300 restart candidates failed: {detail}").into())
}

fn restart_candidates(
    authoritative: Option<std::path::PathBuf>,
    recovery_executable: Option<&Path>,
) -> Vec<std::path::PathBuf> {
    let mut candidates = Vec::with_capacity(2);
    if let Some(authoritative) = authoritative {
        candidates.push(authoritative);
    }
    if let Some(recovery) = recovery_executable {
        if !candidates.iter().any(|candidate| candidate == recovery) {
            candidates.push(recovery.to_path_buf());
        }
    }
    candidates
}

fn wait_for_readiness(
    timeout: Duration,
    interval: Duration,
    mut ready: impl FnMut() -> bool,
) -> Result<(), DynError> {
    let deadline = Instant::now() + timeout;
    loop {
        if ready() {
            return Ok(());
        }
        if Instant::now() >= deadline {
            return Err("Honk300 did not become ready before the restart deadline".into());
        }
        std::thread::sleep(interval);
    }
}

#[cfg(windows)]
fn launch_authoritative_app(executable: &Path) -> Result<(), DynError> {
    use std::os::windows::process::CommandExt;

    let status = Command::new(executable)
        .arg("start")
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .creation_flags(windows_restart_creation_flags())
        .status()?;
    if status.success() {
        Ok(())
    } else {
        Err(format!("the installed Windows launcher returned {status}").into())
    }
}

#[cfg(any(test, windows))]
fn windows_restart_creation_flags() -> u32 {
    const CREATE_NEW_PROCESS_GROUP: u32 = 0x0000_0200;
    const CREATE_NO_WINDOW: u32 = 0x0800_0000;
    CREATE_NEW_PROCESS_GROUP | CREATE_NO_WINDOW
}

#[cfg(target_os = "macos")]
fn launch_authoritative_app(executable: &Path) -> Result<(), DynError> {
    let bundle = macos_bundle_root_from_executable(executable)?;
    let status = Command::new("/usr/bin/open")
        .arg("-n")
        .arg(bundle)
        .arg("--args")
        .arg("start")
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()?;
    if status.success() {
        Ok(())
    } else {
        Err(format!("macOS open returned {status}").into())
    }
}

#[cfg(target_os = "macos")]
fn macos_bundle_root_from_executable(executable: &Path) -> Result<&Path, DynError> {
    let macos = executable
        .parent()
        .filter(|path| path.file_name().and_then(|name| name.to_str()) == Some("MacOS"))
        .ok_or("receipt-owned macOS executable is outside a MacOS directory")?;
    let contents = macos
        .parent()
        .filter(|path| path.file_name().and_then(|name| name.to_str()) == Some("Contents"))
        .ok_or("receipt-owned macOS executable is outside a Contents directory")?;
    let bundle = contents
        .parent()
        .filter(|path| path.extension().and_then(|extension| extension.to_str()) == Some("app"))
        .ok_or("receipt-owned macOS executable is outside an app bundle")?;
    Ok(bundle)
}

#[cfg(target_os = "linux")]
fn launch_authoritative_app(executable: &Path) -> Result<(), DynError> {
    use std::os::unix::process::CommandExt;

    let mut command = Command::new(executable);
    command
        .arg("start")
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    // A direct Linux start owns the runtime for its process lifetime. A new session makes that
    // runtime independent from the updater terminal and its eventual close/SIGHUP.
    unsafe {
        command.pre_exec(|| {
            if libc::setsid() == -1 {
                Err(io::Error::last_os_error())
            } else {
                Ok(())
            }
        });
    }
    command.spawn()?;
    Ok(())
}

#[cfg(not(any(windows, target_os = "macos", target_os = "linux")))]
fn launch_authoritative_app(_executable: &Path) -> Result<(), DynError> {
    Err("Honk300 restart is unsupported on this platform".into())
}

fn selected_no_update_flavor() -> &'static str {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or_default();
    let mixed = nanos ^ u128::from(std::process::id()).wrapping_mul(0x9e37_79b9);
    NO_UPDATE_FLAVORS[(mixed % NO_UPDATE_FLAVORS.len() as u128) as usize]
}

fn guard_acquisition_failure_text(error: &dyn std::fmt::Display) -> String {
    with_attention_panel(&format!(
        "Honk300 could not start the update helper: {error}\nNo update was started.\n"
    ))
}

fn up_to_date_text(flavor: &str) -> String {
    format!(
        "{flavor}\n\
         There was nothing to update.\n\
         Honk300 is already up to date and running.\n{CLOSE_WINDOW_PANEL}"
    )
}

fn updated_text() -> String {
    format!("Update complete.\nHonk300 has restarted.\n{CLOSE_WINDOW_PANEL}")
}

fn duplicate_text() -> String {
    format!(
        "Another Honk300 update window is already open.\n\
         No second update was started.\n{CLOSE_WINDOW_PANEL}"
    )
}

fn updated_restart_failure_text(error: &str) -> String {
    with_attention_panel(&format!(
        "\nUpdate was activated, but Honk300 could not restart: {error}\n\
         Recovery status: the verified update remains installed, but the app is not ready.\n"
    ))
}

fn no_update_restart_failure_text(error: &str) -> String {
    with_attention_panel(&format!(
        "\nNothing required updating, but Honk300 could not be confirmed running: {error}\n\
         Recovery status: the installed version was left unchanged.\n"
    ))
}

fn update_failure_text(error: &str, recovery_status: &str) -> String {
    with_attention_panel(&format!(
        "\nUpdate failed: {error}\nRecovery status: {recovery_status}\n"
    ))
}

fn with_attention_panel(message: &str) -> String {
    format!("{message}{ATTENTION_CLOSE_WINDOW_PANEL}")
}

fn render_cleared(message: &str) {
    let stdout = io::stdout();
    let mut stdout = stdout.lock();
    let clear_result = execute!(
        stdout,
        terminal::Clear(terminal::ClearType::All),
        cursor::MoveTo(0, 0)
    );
    let _ = render_after_clear(&mut stdout, clear_result, message);
    let _ = stdout.flush();
}

fn render_after_clear(
    writer: &mut impl Write,
    clear_result: io::Result<()>,
    message: &str,
) -> io::Result<()> {
    if clear_result.is_err() {
        writeln!(writer, "\n--- Honk300 update result ---")?;
    }
    write!(writer, "{message}")
}

fn keep_terminal_alive() -> ! {
    loop {
        std::thread::park_timeout(Duration::from_secs(60));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn no_update_pool_is_exactly_one_hundred_unique_messages() {
        assert_eq!(NO_UPDATE_FLAVORS.len(), 100);
        let unique = NO_UPDATE_FLAVORS.iter().copied().collect::<HashSet<_>>();
        assert_eq!(unique.len(), NO_UPDATE_FLAVORS.len());
        assert!(NO_UPDATE_FLAVORS
            .iter()
            .all(|message| !message.trim().is_empty()));
    }

    #[test]
    fn every_flavor_keeps_the_explicit_no_update_and_close_contract() {
        for flavor in NO_UPDATE_FLAVORS {
            let text = up_to_date_text(flavor);
            assert!(text.contains("There was nothing to update."));
            assert!(text.contains("Honk300 is already up to date and running."));
            assert!(text.contains("You may now close this window."));
        }
    }

    #[test]
    fn completion_panels_are_aligned_and_visually_separated() {
        for panel in [CLOSE_WINDOW_PANEL, ATTENTION_CLOSE_WINDOW_PANEL] {
            assert!(panel.starts_with('\n'));
            let lines = panel.trim().lines().collect::<Vec<_>>();
            assert_eq!(lines.len(), 4);
            assert!(lines.iter().all(|line| line.chars().count() == 44));
            assert_eq!(lines.first(), lines.last());
            assert!(panel.contains("You may now close this window."));
        }

        let updated = updated_text();
        assert!(updated.starts_with("Update complete.\nHonk300 has restarted.\n\n"));
        assert!(updated.contains("HONK! ALL DONE"));

        let no_update = up_to_date_text("The goose checked.");
        assert!(no_update.contains("up to date and running.\n\n+---"));
        assert!(no_update.contains("HONK! ALL DONE"));
    }

    #[test]
    fn every_terminal_outcome_routes_to_the_expected_panel() {
        let done = [
            updated_text(),
            duplicate_text(),
            up_to_date_text("All current."),
        ];
        for text in done {
            assert!(text.contains("HONK! ALL DONE"));
            assert!(!text.contains("HONK! NEEDS ATTENTION"));
            assert!(text.contains("\n\n+------------------------------------------+"));
        }

        let lock_error = io::Error::new(io::ErrorKind::PermissionDenied, "lock denied");
        let attention = [
            guard_acquisition_failure_text(&lock_error),
            updated_restart_failure_text("restart deadline expired"),
            no_update_restart_failure_text("runtime unavailable"),
            update_failure_text("hash mismatch", "the prior release is running."),
        ];
        for text in attention {
            assert!(text.contains("HONK! NEEDS ATTENTION"));
            assert!(!text.contains("HONK! ALL DONE"));
            assert!(text.contains("\n\n+------------------------------------------+"));
            assert!(text.contains("You may now close this window."));
        }
    }

    #[test]
    fn readiness_tolerates_transient_failure_and_stops_when_ready() {
        let mut attempts = 0;
        wait_for_readiness(Duration::from_secs(1), Duration::ZERO, || {
            attempts += 1;
            attempts == 3
        })
        .unwrap();
        assert_eq!(attempts, 3);
    }

    #[test]
    fn readiness_timeout_is_bounded() {
        let error = wait_for_readiness(Duration::ZERO, Duration::ZERO, || false).unwrap_err();
        assert!(error.to_string().contains("restart deadline"));
    }

    #[test]
    fn recovery_candidates_prefer_the_post_transaction_owner_and_deduplicate() {
        let active = std::path::PathBuf::from("active/honk300");
        let old = Path::new("old/honk300");
        assert_eq!(
            restart_candidates(Some(active.clone()), Some(old)),
            [active.clone(), old.to_path_buf()]
        );
        assert_eq!(
            restart_candidates(Some(active.clone()), Some(&active)),
            [active]
        );
        assert_eq!(restart_candidates(None, Some(old)), [old.to_path_buf()]);
    }

    #[test]
    fn recovery_tries_the_retained_owner_when_the_new_owner_never_becomes_ready() {
        use std::cell::Cell;

        let active = std::path::PathBuf::from("active/honk300");
        let retained = std::path::PathBuf::from("retained/honk300");
        let launched = Cell::new(0_u8);
        let mut attempts = Vec::new();
        start_first_ready_candidate(
            vec![active.clone(), retained.clone()],
            None,
            Duration::ZERO,
            Duration::ZERO,
            |candidate| {
                attempts.push(candidate.to_path_buf());
                launched.set(if candidate == active { 1 } else { 2 });
                Ok(())
            },
            || launched.get() == 2,
        )
        .unwrap();
        assert_eq!(attempts, [active, retained]);
    }

    #[test]
    fn guard_acquisition_failure_remains_explicit_before_the_terminal_hold() {
        let error = io::Error::new(io::ErrorKind::PermissionDenied, "lock denied");
        let text = guard_acquisition_failure_text(&error);
        assert!(text.contains("could not start the update helper"));
        assert!(text.contains("lock denied"));
        assert!(text.contains("No update was started."));
        assert!(text.contains("HONK! NEEDS ATTENTION"));
        assert!(text.contains("You may now close this window."));
    }

    #[test]
    fn terminal_clear_failure_keeps_a_visible_fallback_and_result() {
        let message = updated_text();
        let mut output = Vec::new();
        render_after_clear(
            &mut output,
            Err(io::Error::other("unsupported clear")),
            &message,
        )
        .unwrap();
        let output = String::from_utf8(output).unwrap();
        assert!(output.contains("--- Honk300 update result ---"));
        assert!(output.ends_with(&message));
        assert!(output.contains("HONK! ALL DONE"));

        let mut cleared = Vec::new();
        render_after_clear(&mut cleared, Ok(()), &message).unwrap();
        assert_eq!(String::from_utf8(cleared).unwrap(), message);
    }

    #[test]
    fn windows_restart_controller_is_windowless_and_in_a_new_process_group() {
        const CREATE_NEW_CONSOLE: u32 = 0x0000_0010;
        const CREATE_NEW_PROCESS_GROUP: u32 = 0x0000_0200;
        const CREATE_NO_WINDOW: u32 = 0x0800_0000;
        let flags = windows_restart_creation_flags();
        assert_ne!(flags & CREATE_NEW_PROCESS_GROUP, 0);
        assert_ne!(flags & CREATE_NO_WINDOW, 0);
        assert_eq!(flags & CREATE_NEW_CONSOLE, 0);
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn linux_restart_is_a_new_session_with_the_literal_start_argument() {
        use std::os::unix::fs::PermissionsExt;

        let directory = tempfile::tempdir().unwrap();
        let output = directory.path().join("session.txt");
        let launcher = directory.path().join("fake-honk300");
        std::fs::write(
            &launcher,
            format!(
                "#!/bin/sh\nprintf '%s %s %s\\n' \"$$\" \"$(ps -o sid= -p $$ | tr -d ' ')\" \"$1\" > '{}'\n",
                output.display()
            ),
        )
        .unwrap();
        std::fs::set_permissions(&launcher, std::fs::Permissions::from_mode(0o700)).unwrap();

        launch_authoritative_app(&launcher).unwrap();
        let deadline = Instant::now() + Duration::from_secs(2);
        while !output.is_file() && Instant::now() < deadline {
            std::thread::sleep(Duration::from_millis(10));
        }
        let fields = std::fs::read_to_string(output).unwrap();
        let fields = fields.split_whitespace().collect::<Vec<_>>();
        assert_eq!(fields.len(), 3);
        assert_eq!(
            fields[0], fields[1],
            "setsid must make child PID equal its session ID"
        );
        assert_eq!(fields[2], "start");
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn macos_restart_resolves_only_a_real_app_bundle_shape() {
        let executable = Path::new("/Users/test/Applications/Honk300.app/Contents/MacOS/honk300");
        assert_eq!(
            macos_bundle_root_from_executable(executable).unwrap(),
            Path::new("/Users/test/Applications/Honk300.app")
        );
        assert!(macos_bundle_root_from_executable(Path::new("/tmp/honk300")).is_err());
    }
}
