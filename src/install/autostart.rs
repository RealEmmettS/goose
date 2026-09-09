//! Receipt-owned login startup and configuration reconciliation.
use super::*;

#[derive(Debug, Clone)]
struct ManagedAutostartIdentity {
    source: InstallSource,
    program: PathBuf,
    receipt_path: Option<PathBuf>,
}

/// Apply the config's login-start preference through the one mechanism already owned by the
/// active installation family. An uninstalled/source-tree copy may save the default `false`
/// preference without mutating the machine, but enabling requires authoritative install identity.
pub fn reconcile_config_autostart(enabled: bool) -> Result<(), DynError> {
    let Some(identity) = managed_autostart_identity()? else {
        return if enabled {
            Err("login autostart requires a managed Honk300 install; run `honk300 install` or the platform installer first".into())
        } else {
            Ok(())
        };
    };

    #[cfg(windows)]
    {
        if windows_autostart_is_machine_owned(identity.source) {
            if windows_autostart_identity_matches(&identity, enabled)? {
                return Ok(());
            }
            return elevate_windows_autostart_reconcile(enabled);
        }
        reconcile_windows_autostart(&identity, enabled)?;
    }
    #[cfg(target_os = "linux")]
    reconcile_linux_autostart(&identity, enabled)?;
    #[cfg(target_os = "macos")]
    reconcile_macos_autostart(&identity, enabled)?;
    #[cfg(not(any(windows, target_os = "linux", target_os = "macos")))]
    return Err("login autostart is unsupported on this platform".into());

    Ok(())
}

/// Resolve installer intent and config intent before the runtime or TUI consumes the preference.
/// A newer receipt wins and is mirrored into config; otherwise an explicitly configured value is
/// applied to the one startup mechanism owned by the active installation.
pub fn prepare_config_autostart(
    config_path: &Path,
    config: &mut honk_config::Config,
) -> Result<(), DynError> {
    let snapshot = honk_config::ConfigSnapshot::load(config_path)?;
    if snapshot.config != *config {
        return Err(honk_config::ConfigError::Conflict.into());
    }
    let contents = match fs::read_to_string(config_path) {
        Ok(contents) => contents,
        Err(error) if error.kind() == io::ErrorKind::NotFound => String::new(),
        Err(error) => return Err(error.into()),
    };
    let explicitly_configured = contents.lines().any(|line| {
        line.split_once('#')
            .map_or(line, |(value, _)| value)
            .trim_start()
            .starts_with("autostart_on_login")
    });

    let Some(identity) = managed_autostart_identity()? else {
        return if explicitly_configured {
            snapshot.revision.with_guard(config_path, || {
                reconcile_config_autostart(config.lifecycle.autostart_on_login)
            })
        } else {
            Ok(())
        };
    };
    if let Some(receipt_path) = identity.receipt_path.as_deref() {
        let config_modified = fs::metadata(config_path)
            .and_then(|metadata| metadata.modified())
            .ok();
        let receipt_modified = fs::metadata(receipt_path)?.modified()?;
        if config_modified.is_none_or(|modified| receipt_modified > modified) {
            let actual = owned_autostart_state(&identity)?;
            if config.lifecycle.autostart_on_login != actual {
                config.lifecycle.autostart_on_login = actual;
                config.save_if_revision(config_path, &snapshot.revision)?;
            }
            return Ok(());
        }
    }
    if explicitly_configured {
        snapshot.revision.with_guard(config_path, || {
            reconcile_config_autostart(config.lifecycle.autostart_on_login)
        })?;
    }
    Ok(())
}

fn owned_autostart_state(identity: &ManagedAutostartIdentity) -> Result<bool, DynError> {
    #[cfg(windows)]
    return windows_owned_autostart_state(identity);
    #[cfg(target_os = "linux")]
    {
        let _ = identity;
        owned_text_autostart_state(&linux_autostart_path()?, OWNERSHIP_MARKER)
    }
    #[cfg(target_os = "macos")]
    {
        let _ = identity;
        owned_text_autostart_state(&macos_launch_agent_path()?, OWNERSHIP_MARKER)
    }
    #[cfg(not(any(windows, target_os = "linux", target_os = "macos")))]
    {
        let _ = identity;
        Err("login autostart is unsupported on this platform".into())
    }
}

#[cfg(any(test, target_os = "linux", target_os = "macos"))]
pub(super) fn owned_text_autostart_state(path: &Path, marker: &str) -> Result<bool, DynError> {
    let metadata = match fs::symlink_metadata(path) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(false),
        Err(error) => return Err(error.into()),
    };
    if !metadata.is_file() || metadata.file_type().is_symlink() {
        // A directory, device, or symlink at the conventional location is foreign state. It is
        // never considered enabled and is never mutated by the disable path.
        return Ok(false);
    }
    let contents = fs::read_to_string(path)?;
    if !contents.contains(marker) {
        // A foreign file at our conventional path is not evidence that Honk300 owns or enabled
        // login start. Preserve it and report our preference as disabled; mutation paths retain
        // their stricter ownership preflight before they create or replace anything.
        return Ok(false);
    }
    Ok(true)
}

fn managed_autostart_identity() -> Result<Option<ManagedAutostartIdentity>, DynError> {
    let current_exe = std::env::current_exe()?;
    let source = detect_install_source();
    if source == InstallSource::Unknown {
        return Ok(None);
    }

    for receipt_path in current_owned_receipt_candidates(&current_exe)
        .into_iter()
        .chain(external_receipt_candidates())
    {
        let metadata = match fs::symlink_metadata(&receipt_path) {
            Ok(metadata) => metadata,
            Err(error) if error.kind() == io::ErrorKind::NotFound => continue,
            Err(error) => return Err(error.into()),
        };
        if !metadata.is_file() || metadata.file_type().is_symlink() {
            return Err(format!(
                "install receipt is not a regular owned file: {}",
                receipt_path.display()
            )
            .into());
        }
        let receipt: serde_json::Value = serde_json::from_slice(&fs::read(&receipt_path)?)?;
        if validated_receipt_source(&receipt, &current_exe) != Some(source) {
            continue;
        }
        let root = PathBuf::from(
            receipt
                .get("install_root")
                .and_then(serde_json::Value::as_str)
                .ok_or("managed install receipt has no install_root")?,
        );
        let program = managed_autostart_program(source, &root, &receipt)?;
        if !program.exists() {
            return Err(format!(
                "managed login-autostart program is missing: {}",
                program.display()
            )
            .into());
        }
        return Ok(Some(ManagedAutostartIdentity {
            source,
            program,
            receipt_path: Some(receipt_path),
        }));
    }

    if source == InstallSource::ManualLocal {
        let program = manual_autostart_program(current_exe);
        if !program.exists() {
            return Err(format!(
                "manual login-autostart program is missing: {}",
                program.display()
            )
            .into());
        }
        return Ok(Some(ManagedAutostartIdentity {
            source,
            program,
            receipt_path: None,
        }));
    }
    Err("the detected install has no authoritative receipt for login autostart".into())
}

pub(super) fn manual_autostart_program(current_exe: PathBuf) -> PathBuf {
    #[cfg(windows)]
    {
        current_exe.with_file_name(WINDOWS_APP_LAUNCHER_NAME)
    }
    #[cfg(not(windows))]
    {
        current_exe
    }
}

fn managed_autostart_program(
    source: InstallSource,
    root: &Path,
    receipt: &serde_json::Value,
) -> Result<PathBuf, DynError> {
    #[cfg(windows)]
    {
        let _ = (source, receipt);
        Ok(root.join("bin").join(WINDOWS_APP_LAUNCHER_NAME))
    }
    #[cfg(target_os = "macos")]
    {
        let _ = (source, receipt);
        Ok(root.join("Contents").join("MacOS").join("honk300"))
    }
    #[cfg(target_os = "linux")]
    {
        let _ = (source, root);
        receipt
            .get("aliases")
            .and_then(serde_json::Value::as_array)
            .and_then(|aliases| aliases.first())
            .and_then(serde_json::Value::as_str)
            .map(PathBuf::from)
            .ok_or_else(|| "managed Linux receipt has no stable honk300 alias".into())
    }
    #[cfg(not(any(windows, target_os = "linux", target_os = "macos")))]
    {
        let _ = (source, root, receipt);
        Err("login autostart is unsupported on this platform".into())
    }
}

#[cfg(windows)]
fn receipt_autostart_enabled(path: Option<&Path>) -> Result<Option<bool>, DynError> {
    let Some(path) = path else {
        return Ok(None);
    };
    let value: serde_json::Value = serde_json::from_slice(&fs::read(path)?)?;
    Ok(value
        .get("autostart")
        .and_then(|autostart| autostart.get("enabled"))
        .and_then(serde_json::Value::as_bool))
}

fn update_receipt_autostart(
    identity: &ManagedAutostartIdentity,
    enabled: bool,
) -> Result<(), DynError> {
    let Some(path) = identity.receipt_path.as_deref() else {
        return Ok(());
    };
    // A Debian receipt is machine-owned while this preference is intentionally per-user XDG
    // state. Do not claim one user's choice as package-global receipt state.
    if identity.source == InstallSource::Deb {
        return Ok(());
    }
    let metadata = fs::symlink_metadata(path)?;
    if !metadata.is_file() || metadata.file_type().is_symlink() {
        return Err("install receipt is not a regular owned file".into());
    }
    let mut receipt: serde_json::Value = serde_json::from_slice(&fs::read(path)?)?;
    let owner = receipt
        .get("autostart")
        .and_then(|autostart| autostart.get("owner"))
        .and_then(serde_json::Value::as_str)
        .unwrap_or("honk300-install")
        .to_owned();
    receipt["autostart"] = serde_json::json!({ "enabled": enabled, "owner": owner });
    let parent = path
        .parent()
        .ok_or("install receipt has no parent directory")?;
    let temporary = parent.join(format!(
        ".install-receipt.autostart.{}.tmp",
        std::process::id()
    ));
    if temporary.exists() {
        return Err(format!(
            "stale autostart receipt transaction exists: {}",
            temporary.display()
        )
        .into());
    }
    fs::write(&temporary, serde_json::to_vec_pretty(&receipt)?)?;
    fs::set_permissions(&temporary, metadata.permissions())?;
    if let Err(error) = fs::rename(&temporary, path) {
        let _ = fs::remove_file(&temporary);
        return Err(error.into());
    }
    Ok(())
}

#[cfg(target_os = "linux")]
fn reconcile_linux_autostart(
    identity: &ManagedAutostartIdentity,
    enabled: bool,
) -> Result<(), DynError> {
    let path = linux_autostart_path()?;
    reconcile_owned_text_autostart_file(
        &path,
        enabled,
        &linux_desktop_entry(&identity.program, true),
        OWNERSHIP_MARKER,
    )?;
    update_receipt_autostart(identity, enabled)
}

#[cfg(target_os = "macos")]
fn reconcile_macos_autostart(
    identity: &ManagedAutostartIdentity,
    enabled: bool,
) -> Result<(), DynError> {
    let path = macos_launch_agent_path()?;
    reconcile_owned_text_autostart_file(
        &path,
        enabled,
        &macos_launch_agent_plist(&identity.program),
        OWNERSHIP_MARKER,
    )?;
    update_receipt_autostart(identity, enabled)
}

#[cfg(any(test, target_os = "linux", target_os = "macos"))]
pub(super) fn reconcile_owned_text_autostart_file(
    path: &Path,
    enabled: bool,
    contents: &str,
    marker: &str,
) -> io::Result<()> {
    if enabled {
        preflight_owned_text_file(path, marker)?;
        write_owned_text_file(path, contents, marker)?;
    } else {
        remove_owned_text_file(path, marker)?;
    }
    Ok(())
}

#[cfg(windows)]
pub(super) fn set_windows_autostart(exe: Option<&Path>) -> io::Result<()> {
    use winreg::enums::HKEY_CURRENT_USER;
    use winreg::RegKey;

    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let (run, _) = hkcu.create_subkey("Software\\Microsoft\\Windows\\CurrentVersion\\Run")?;
    if let Some(exe) = exe {
        run.set_value("Honk300", &windows_autostart_command(exe))
    } else {
        let _ = run.delete_value("Honk300");
        Ok(())
    }
}

#[cfg(windows)]
fn windows_autostart_is_machine_owned(source: InstallSource) -> bool {
    matches!(
        source,
        InstallSource::MsiGlobal | InstallSource::ExeGlobal | InstallSource::PowerShell
    )
}

#[cfg(windows)]
pub(super) fn windows_autostart_command(program: &Path) -> String {
    format!("\"{}\"", program.display())
}

#[cfg(windows)]
pub(super) fn legacy_windows_autostart_command(program: &Path) -> Option<String> {
    (program.file_name()?.to_str()? == WINDOWS_APP_LAUNCHER_NAME).then(|| {
        format!(
            "\"{}\" start",
            program.with_file_name("honk300.exe").display()
        )
    })
}

#[cfg(windows)]
fn windows_autostart_value(source: InstallSource) -> io::Result<Option<String>> {
    use winreg::enums::{HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE, KEY_READ};
    use winreg::RegKey;

    let hive = if windows_autostart_is_machine_owned(source) {
        HKEY_LOCAL_MACHINE
    } else {
        HKEY_CURRENT_USER
    };
    let root = RegKey::predef(hive);
    let run = match root.open_subkey_with_flags(
        "Software\\Microsoft\\Windows\\CurrentVersion\\Run",
        KEY_READ,
    ) {
        Ok(run) => run,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(error),
    };
    match run.get_value::<String, _>("Honk300") {
        Ok(value) => Ok(Some(value)),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(error),
    }
}

#[cfg(windows)]
fn windows_owned_autostart_state(identity: &ManagedAutostartIdentity) -> Result<bool, DynError> {
    let expected = windows_autostart_command(&identity.program);
    let legacy = legacy_windows_autostart_command(&identity.program);
    match windows_autostart_value(identity.source)? {
        None => Ok(false),
        Some(actual) if actual.eq_ignore_ascii_case(&expected) => Ok(true),
        Some(actual)
            if legacy
                .as_deref()
                .is_some_and(|legacy| actual.eq_ignore_ascii_case(legacy)) =>
        {
            Ok(true)
        }
        Some(actual) => Err(format!(
            "refusing to replace foreign Honk300 login-start value `{actual}`; expected `{expected}`"
        )
        .into()),
    }
}

#[cfg(windows)]
fn windows_autostart_identity_matches(
    identity: &ManagedAutostartIdentity,
    enabled: bool,
) -> Result<bool, DynError> {
    let expected = windows_autostart_command(&identity.program);
    let actual = windows_autostart_value(identity.source)?;
    let mechanism_matches = if enabled {
        actual
            .as_deref()
            .is_some_and(|actual| actual.eq_ignore_ascii_case(&expected))
    } else {
        actual.is_none()
    };
    Ok(mechanism_matches
        && receipt_autostart_enabled(identity.receipt_path.as_deref())?
            .is_none_or(|v| v == enabled))
}

#[cfg(windows)]
fn reconcile_windows_autostart(
    identity: &ManagedAutostartIdentity,
    enabled: bool,
) -> Result<(), DynError> {
    use winreg::enums::{HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE};
    use winreg::RegKey;

    let expected = windows_autostart_command(&identity.program);
    let legacy = legacy_windows_autostart_command(&identity.program);
    let existing = windows_autostart_value(identity.source)?;
    if existing.as_deref().is_some_and(|actual| {
        !actual.eq_ignore_ascii_case(&expected)
            && !legacy
                .as_deref()
                .is_some_and(|legacy| actual.eq_ignore_ascii_case(legacy))
    }) {
        return Err(format!(
            "refusing to replace foreign Honk300 login-start value `{}`",
            existing.expect("checked as present")
        )
        .into());
    }
    let hive = if windows_autostart_is_machine_owned(identity.source) {
        HKEY_LOCAL_MACHINE
    } else {
        HKEY_CURRENT_USER
    };
    let root = RegKey::predef(hive);
    let (run, _) = root.create_subkey("Software\\Microsoft\\Windows\\CurrentVersion\\Run")?;
    if enabled {
        run.set_value("Honk300", &expected)?;
    } else if existing.is_some() {
        run.delete_value("Honk300")?;
    }
    update_receipt_autostart(identity, enabled)
}

#[cfg(windows)]
fn elevate_windows_autostart_reconcile(enabled: bool) -> Result<(), DynError> {
    use std::os::windows::process::CommandExt;

    const CREATE_NO_WINDOW: u32 = 0x0800_0000;
    let current = std::env::current_exe()?;
    let script = format!(
        "$p=Start-Process -FilePath '{}' -ArgumentList @('__windows-config-autostart','{}') -Verb RunAs -WindowStyle Hidden -Wait -PassThru; exit $p.ExitCode",
        powershell_literal(&current.to_string_lossy()),
        enabled
    );
    let status = std::process::Command::new(system_windows_powershell_path()?)
        .args(["-NoProfile", "-NonInteractive", "-Command", &script])
        .creation_flags(CREATE_NO_WINDOW)
        .status()?;
    if status.success() {
        Ok(())
    } else {
        Err(format!(
            "elevated Global login-autostart reconciliation exited with {}",
            status.code().unwrap_or(-1)
        )
        .into())
    }
}

#[cfg(windows)]
pub fn run_windows_config_autostart_protocol() -> Result<bool, DynError> {
    let mut args = std::env::args_os();
    let _program = args.next();
    if args.next().as_deref() != Some(std::ffi::OsStr::new("__windows-config-autostart")) {
        return Ok(false);
    }
    let enabled = match args.next().as_deref().and_then(std::ffi::OsStr::to_str) {
        Some("true") => true,
        Some("false") => false,
        _ => return Err("invalid internal Windows autostart preference".into()),
    };
    if args.next().is_some() {
        return Err("unexpected internal Windows autostart argument".into());
    }
    let identity = managed_autostart_identity()?
        .ok_or("elevated Windows autostart helper has no managed install identity")?;
    if !windows_autostart_is_machine_owned(identity.source) {
        return Err("elevated Windows autostart helper refused a non-machine install".into());
    }
    reconcile_windows_autostart(&identity, enabled)?;
    Ok(true)
}
