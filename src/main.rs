//! honk300 — the binary entry point.
//!
//! M10 adds the local control plane around the current Windows runtime. The root
//! process parses CLI commands, sends stop/do/reload over IPC, or starts the one
//! allowed desktop goose instance.

mod cli;
#[cfg(not(windows))]
mod debian;
mod install;
mod runtime;
mod update;

#[cfg(any(windows, target_os = "macos", target_os = "linux"))]
mod assets;
#[cfg(any(windows, target_os = "macos", target_os = "linux"))]
mod audio;

use cli::{Cli, Command, StartOptions};
#[cfg(any(windows, target_os = "macos", target_os = "linux"))]
use honk_config::CliOverrides;
use honk_config::{reset_to_defaults, Config, ConfigError, ConfigLoadState};
#[cfg(any(windows, target_os = "macos", target_os = "linux"))]
use honk_control::CommandServer;
use honk_control::{
    send_command, wait_for_shutdown, ControlCommand, ControlResponse, LifecycleLease,
    RuntimeStatus, Singleton, SingletonStatus,
};
#[cfg(any(windows, target_os = "macos", target_os = "linux"))]
use runtime::RuntimeOptions;
use std::io::{self, Write};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(windows)]
    if install::run_windows_config_autostart_protocol()? {
        return Ok(());
    }

    #[cfg(windows)]
    if install::run_windows_slot_protocol()? {
        return Ok(());
    }

    #[cfg(windows)]
    install::reject_uncommanded_windows_installer_helper()?;

    #[cfg(windows)]
    if std::env::var_os("HONK300_INTERNAL_WINDOWS_UNINSTALL").as_deref()
        == Some(std::ffi::OsStr::new("1"))
    {
        install::run_windows_deferred_uninstall()?;
        return Ok(());
    }
    // Internal managed-installer protocol. The child retains the singleton until its stdin closes,
    // so a Unix FIFO or Windows redirected pipe gives the installer exclusive lifecycle ownership
    // across the complete swap. EOF is kernel-delivered if the parent dies, preventing an orphaned
    // lease. Keep this before clap so it stays private and exact-tag installer compatible.
    if std::env::var_os("HONK300_INTERNAL_HOLD_LIFECYCLE_LEASE").as_deref()
        == Some(std::ffi::OsStr::new("1"))
    {
        let _lease = LifecycleLease::acquire()?;
        use std::io::Write as _;
        println!("HONK300_INTERNAL_LIFECYCLE_LEASE_READY");
        std::io::stdout().flush()?;
        std::io::copy(&mut std::io::stdin().lock(), &mut std::io::sink())?;
        return Ok(());
    }
    // Internal exact-tag installer probe: unlike `status`, this acquires the real singleton and
    // therefore closes the socket-not-yet-ready startup race before an on-disk replacement.
    if std::env::var_os("HONK300_INTERNAL_WAIT_FOR_SHUTDOWN").as_deref()
        == Some(std::ffi::OsStr::new("1"))
    {
        wait_for_shutdown()?;
        return Ok(());
    }
    let cli = Cli::parse_normalized();

    if cli.is_client_command() {
        return run_client_command(cli);
    }

    match cli.command {
        Some(Command::Config { config }) => run_config(config),
        Some(Command::Install { autostart }) => install::install(autostart),
        Some(Command::Uninstall { purge }) => install::uninstall(purge),
        Some(Command::Update { json }) => update::run(json),
        Some(Command::Setup { config, reset }) => run_setup(config, reset),
        Some(Command::Start { options }) => run_start(options),
        None => run_start(StartOptions::default()),
        Some(Command::Stop { .. } | Command::Reload | Command::Status | Command::Do { .. }) => {
            unreachable!()
        }
    }
}

fn run_client_command(cli: Cli) -> Result<(), Box<dyn std::error::Error>> {
    let wait_for_stop = matches!(&cli.command, Some(Command::Stop { .. }));
    let is_force_stop = matches!(&cli.command, Some(Command::Stop { force: true }));
    let is_status = matches!(&cli.command, Some(Command::Status));
    let force_runtime_was_running = if is_force_stop {
        let (probe, status) = Singleton::acquire()?;
        drop(probe);
        status == SingletonStatus::AlreadyRunning
    } else {
        false
    };
    let command = match cli.command {
        Some(Command::Stop { force: false }) => ControlCommand::Stop,
        Some(Command::Stop { force: true }) => ControlCommand::ForceStop,
        Some(Command::Reload) => ControlCommand::Reload,
        Some(Command::Status) => ControlCommand::Status,
        Some(Command::Do { action }) => ControlCommand::Do(action.into_engine()),
        Some(
            Command::Start { .. }
            | Command::Config { .. }
            | Command::Install { .. }
            | Command::Uninstall { .. }
            | Command::Update { .. }
            | Command::Setup { .. },
        )
        | None => unreachable!("non-client commands are handled separately"),
    };
    let response = match send_command(command) {
        Ok(response) => response,
        // A hard stop intentionally tears down the IPC endpoint without unwinding the runtime.
        // If the response bytes lose that race, the singleton is the authoritative completion
        // signal. Only accept that fallback when a runtime demonstrably existed before dispatch.
        Err(_err) if is_force_stop && force_runtime_was_running => {
            wait_for_shutdown()?;
            ControlResponse::Ok
        }
        Err(err)
            if matches!(
                err.kind(),
                std::io::ErrorKind::NotFound | std::io::ErrorKind::ConnectionRefused
            ) =>
        {
            if is_status {
                print_status(RuntimeStatus::not_running())?;
                return Ok(());
            }
            return Err("honk300: no running goose instance.".into());
        }
        Err(err) => return Err(err.into()),
    };

    match response {
        ControlResponse::Ok => {
            if wait_for_stop {
                wait_for_shutdown()?;
            }
            println!("honk300: command accepted.");
            Ok(())
        }
        ControlResponse::Err(code) => Err(format!("honk300 command rejected: {code}").into()),
        ControlResponse::Status(status) => {
            print_status(status)?;
            Ok(())
        }
    }
}

fn print_status(status: RuntimeStatus) -> io::Result<()> {
    let stdout = io::stdout();
    let mut stdout = stdout.lock();
    ignore_broken_pipe(write_status(&mut stdout, status))
}

fn write_status(writer: &mut impl Write, status: RuntimeStatus) -> io::Result<()> {
    writeln!(
        writer,
        "honk300: {}",
        if status.running {
            "running"
        } else {
            "not running"
        }
    )?;
    writeln!(writer, "platform: {}", status.platform.label())?;
    writeln!(writer, "bundle: {}", status.bundle.label())?;
    writeln!(writer, "overlay: {}", status.overlay.label())?;
    writeln!(writer, "accessibility: {}", status.accessibility.label())?;
    writeln!(writer, "cursor: {}", status.cursor.label())?;
    writeln!(writer, "window: {}", status.window.label())?;
    writeln!(writer, "collect: {}", status.collect.label())?;
    writeln!(writer, "presence: {}", status.presence.label())?;
    writeln!(writer, "audio: {}", status.audio.label())?;
    writeln!(
        writer,
        "assets: {} notes, {} memes",
        status.notes, status.memes
    )
}

fn ignore_broken_pipe(result: io::Result<()>) -> io::Result<()> {
    match result {
        Err(err) if err.kind() == io::ErrorKind::BrokenPipe => Ok(()),
        result => result,
    }
}

fn run_config(config: Option<std::path::PathBuf>) -> Result<(), Box<dyn std::error::Error>> {
    let path = honk_config::resolve_path(config)?;
    let mut loaded = Config::load_or_default(Some(path.clone()))?;
    install::prepare_config_autostart(&path, &mut loaded.config)?;
    honk_config_tui::run_with_save_hook(path, |config| {
        install::reconcile_config_autostart(config.lifecycle.autostart_on_login)
            .map_err(|error| error.to_string())
    })?;
    Ok(())
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum SetupResult {
    Created,
    AlreadyExists,
    Reset { backup: Option<std::path::PathBuf> },
}

fn run_setup_at(path: &std::path::Path, reset: bool) -> Result<SetupResult, ConfigError> {
    if reset {
        return reset_to_defaults(path).map(|backup| SetupResult::Reset { backup });
    }
    match Config::load(Some(path.to_path_buf()))? {
        ConfigLoadState::Missing { .. } => {
            Config::default().save_atomic(path)?;
            Ok(SetupResult::Created)
        }
        ConfigLoadState::Loaded(_) => Ok(SetupResult::AlreadyExists),
        ConfigLoadState::Malformed { error, .. } => Err(ConfigError::MalformedDocument(error)),
        ConfigLoadState::UnsupportedVersion { found, .. } => Err(ConfigError::WrongVersion(found)),
    }
}

fn run_setup(
    config: Option<std::path::PathBuf>,
    reset: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let path = honk_config::resolve_path(config)?;
    match run_setup_at(&path, reset)? {
        SetupResult::Created => println!("honk300: config created at {}.", path.display()),
        SetupResult::AlreadyExists => println!(
            "honk300: config already exists at {}; left unchanged (use --reset to replace it).",
            path.display()
        ),
        SetupResult::Reset {
            backup: Some(backup),
        } => println!(
            "honk300: config reset at {}; backup saved to {}.",
            path.display(),
            backup.display()
        ),
        SetupResult::Reset { backup: None } => {
            println!("honk300: config created at {}.", path.display())
        }
    }
    Ok(())
}

#[cfg(windows)]
fn run_start(options: StartOptions) -> Result<(), Box<dyn std::error::Error>> {
    let (_singleton, status) = Singleton::acquire()?;
    if status == honk_control::SingletonStatus::AlreadyRunning {
        println!("honk300: a goose is already running. Use `honk300 stop` to stop it.");
        return Ok(());
    }

    let mut loaded = Config::load_or_default(options.config.clone())?;
    if let Some(warning) = &loaded.warning {
        eprintln!("honk300 config: {warning}");
    }
    install::prepare_config_autostart(&loaded.path, &mut loaded.config)?;

    let server = CommandServer::start()?;
    runtime::windows::run(
        RuntimeOptions {
            config_path: loaded.path,
            config: loaded.config,
            cli_overrides: CliOverrides {
                no_sound: options.no_sound,
                no_mouse_steal: options.no_mouse_steal,
                no_window_ride: options.no_window_ride,
                wayland: options.wayland,
            },
        },
        &server,
    )
}

#[cfg(target_os = "macos")]
fn run_start(options: StartOptions) -> Result<(), Box<dyn std::error::Error>> {
    let (_singleton, status) = Singleton::acquire()?;
    if status == honk_control::SingletonStatus::AlreadyRunning {
        println!("honk300: a goose is already running. Use `honk300 stop` to stop it.");
        return Ok(());
    }

    let mut loaded = Config::load_or_default(options.config.clone())?;
    if let Some(warning) = &loaded.warning {
        eprintln!("honk300 config: {warning}");
    }
    install::prepare_config_autostart(&loaded.path, &mut loaded.config)?;

    let server = CommandServer::start()?;
    runtime::macos::run(
        RuntimeOptions {
            config_path: loaded.path,
            config: loaded.config,
            cli_overrides: CliOverrides {
                no_sound: options.no_sound,
                no_mouse_steal: options.no_mouse_steal,
                no_window_ride: options.no_window_ride,
                wayland: options.wayland,
            },
        },
        &server,
    )
}

#[cfg(target_os = "linux")]
fn run_start(options: StartOptions) -> Result<(), Box<dyn std::error::Error>> {
    let (_singleton, status) = Singleton::acquire()?;
    if status == honk_control::SingletonStatus::AlreadyRunning {
        println!("honk300: a goose is already running. Use `honk300 stop` to stop it.");
        return Ok(());
    }
    let mut loaded = Config::load_or_default(options.config.clone())?;
    if let Some(warning) = &loaded.warning {
        eprintln!("honk300 config: {warning}");
    }
    install::prepare_config_autostart(&loaded.path, &mut loaded.config)?;

    let server = CommandServer::start()?;
    runtime::linux::run(
        RuntimeOptions {
            config_path: loaded.path,
            config: loaded.config,
            cli_overrides: CliOverrides {
                no_sound: options.no_sound,
                no_mouse_steal: options.no_mouse_steal,
                no_window_ride: options.no_window_ride,
                wayland: options.wayland,
            },
        },
        &server,
    )
}

#[cfg(not(any(windows, target_os = "macos", target_os = "linux")))]
fn run_start(options: StartOptions) -> Result<(), Box<dyn std::error::Error>> {
    let (_singleton, status) = Singleton::acquire()?;
    if status == honk_control::SingletonStatus::AlreadyRunning {
        println!("honk300: a goose is already running. Use `honk300 stop` to stop it.");
        return Ok(());
    }
    let loaded = Config::load_or_default(options.config)?;
    if let Some(warning) = loaded.warning {
        eprintln!("honk300 config: {warning}");
    }
    eprintln!("honk300: this OS does not have a desktop backend yet.");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    struct FailingWriter(io::ErrorKind);

    impl Write for FailingWriter {
        fn write(&mut self, _buf: &[u8]) -> io::Result<usize> {
            Err(io::Error::from(self.0))
        }

        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }

    #[test]
    fn status_output_treats_a_closed_downstream_pipe_as_success() {
        let result = write_status(
            &mut FailingWriter(io::ErrorKind::BrokenPipe),
            RuntimeStatus::not_running(),
        );

        assert!(ignore_broken_pipe(result).is_ok());
    }

    #[test]
    fn status_output_preserves_other_write_errors() {
        let result = write_status(
            &mut FailingWriter(io::ErrorKind::PermissionDenied),
            RuntimeStatus::not_running(),
        );

        assert_eq!(
            ignore_broken_pipe(result).unwrap_err().kind(),
            io::ErrorKind::PermissionDenied
        );
    }

    #[test]
    fn setup_creates_a_valid_v2_config_only_when_missing() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config.toml");
        assert_eq!(run_setup_at(&path, false).unwrap(), SetupResult::Created);
        assert_eq!(Config::load_existing(&path).unwrap(), Config::default());

        fs::write(
            &path,
            "goose_config_version = 2\nfuture_root = 'preserve me'\n",
        )
        .unwrap();
        let original = fs::read(&path).unwrap();
        assert_eq!(
            run_setup_at(&path, false).unwrap(),
            SetupResult::AlreadyExists
        );
        assert_eq!(fs::read(&path).unwrap(), original);
    }

    #[test]
    fn setup_refuses_malformed_existing_config_without_reset() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config.toml");
        let original = b"[broken\nvalue = true\n";
        fs::write(&path, original).unwrap();
        assert!(run_setup_at(&path, false).is_err());
        assert_eq!(fs::read(&path).unwrap(), original);
    }

    #[test]
    fn setup_reset_backs_up_then_replaces_existing_config() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config.toml");
        let original = b"goose_config_version = 99\nfuture = true\n";
        fs::write(&path, original).unwrap();
        let SetupResult::Reset {
            backup: Some(backup),
        } = run_setup_at(&path, true).unwrap()
        else {
            panic!("reset should report its backup");
        };
        assert_eq!(fs::read(backup).unwrap(), original);
        assert_eq!(Config::load_existing(&path).unwrap(), Config::default());
    }
}
