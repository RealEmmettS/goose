mod autostart;
mod media;
mod provenance;
#[cfg(windows)]
pub use autostart::run_windows_config_autostart_protocol;
#[cfg(any(test, windows, target_os = "linux"))]
use autostart::*;
pub use autostart::{prepare_config_autostart, reconcile_config_autostart};
use media::*;
#[cfg(windows)]
pub(crate) use provenance::detected_windows_install_root;
use provenance::*;
pub use provenance::{detect_install_source, InstallSource};
pub(crate) mod companions;
pub(crate) use companions::verify_settings_companion;
use std::fs;
use std::io;
#[cfg(target_os = "macos")]
use std::io::Read;
use std::path::{Path, PathBuf};

#[cfg(target_os = "macos")]
use sha2::{Digest, Sha256};

#[cfg(any(windows, target_os = "linux", target_os = "macos"))]
use honk_control::LifecycleLease;

const APP_NAME: &str = "honk300";
#[cfg(windows)]
const DISPLAY_NAME: &str = "Honk300";
#[cfg(windows)]
const WINDOWS_APP_LAUNCHER_NAME: &str = "honk300-app.exe";
const MARKER_FILE: &str = "install-source.txt";
const COMMAND_NAMES: &[&str] = &["honk300", "honk", "goose"];
const OWNERSHIP_MARKER: &str = "honk300.install.v1";
const INSTALL_RECEIPT_V2: &str = "honk300.install.v2";
#[cfg(any(test, target_os = "linux", target_os = "macos"))]
const PATH_MARKER_START: &str = "# >>> honk300 managed PATH >>>";
#[cfg(any(test, target_os = "linux", target_os = "macos"))]
const PATH_MARKER_END: &str = "# <<< honk300 managed PATH <<<";

type DynError = Box<dyn std::error::Error>;

#[cfg(test)]
fn mutate_with_lifecycle_lease<L, T>(
    acquire: impl FnOnce() -> io::Result<L>,
    mutation: impl FnOnce() -> io::Result<T>,
) -> io::Result<T> {
    let _lease = acquire()?;
    mutation()
}

#[cfg(windows)]
pub fn install(autostart: bool) -> Result<(), DynError> {
    let root = windows_user_install_root()?;
    ensure_owned_install_root(&root, &[InstallSource::ManualLocal])?;
    let _lifecycle_lease = LifecycleLease::acquire()?;
    let bin_dir = root.join("bin");
    fs::create_dir_all(&bin_dir)?;

    let media = windows_media_root()?;
    ensure_external_media_root(&media)?;
    migrate_legacy_user_media(&bin_dir.join("Assets"), &media, LegacyMigrationMode::Move)?;
    copy_current_exe_to_aliases(&bin_dir)?;
    copy_windows_app_launcher(&bin_dir)?;
    companions::copy_settings_if_present(&bin_dir)?;
    write_install_marker(&root, InstallSource::ManualLocal)?;
    write_windows_install_source_marker(InstallSource::ManualLocal)?;
    add_windows_user_path(&bin_dir)?;
    create_windows_start_menu_shortcut(&bin_dir.join(WINDOWS_APP_LAUNCHER_NAME))?;

    if autostart {
        set_windows_autostart(Some(&bin_dir.join(WINDOWS_APP_LAUNCHER_NAME)))?;
    } else {
        set_windows_autostart(None)?;
    }

    println!("honk300: installed to {}.", root.display());
    println!("honk300: command aliases available after opening a new terminal.");
    if autostart {
        println!("honk300: login autostart enabled.");
    }
    Ok(())
}

#[cfg(target_os = "linux")]
pub fn install(autostart: bool) -> Result<(), DynError> {
    let root = linux_user_install_root()?;
    ensure_owned_install_root(&root, &[InstallSource::ManualLocal, InstallSource::Shell])?;
    let _lifecycle_lease = LifecycleLease::acquire()?;
    let bin_dir = root.join("bin");
    fs::create_dir_all(&bin_dir)?;

    let media = linux_media_root()?;
    ensure_external_media_root(&media)?;
    migrate_legacy_user_media(&bin_dir.join("Assets"), &media, LegacyMigrationMode::Move)?;
    let installed = bin_dir.join("honk300");
    copy_current_exe(&installed)?;
    companions::copy_settings_if_present(&bin_dir)?;
    make_executable(&installed)?;
    write_install_marker(&root, InstallSource::ManualLocal)?;

    let aliases_dir = linux_user_alias_dir()?;
    fs::create_dir_all(&aliases_dir)?;
    let owned_targets = [&installed as &Path];
    for name in COMMAND_NAMES {
        install_owned_unix_alias(&aliases_dir.join(name), &installed, &owned_targets)?;
    }

    let desktop = linux_desktop_entry(&installed);
    let desktop_path = linux_applications_dir()?.join("honk300.desktop");
    write_owned_text_file(&desktop_path, &desktop, OWNERSHIP_MARKER)?;
    if autostart {
        write_owned_text_file(&linux_autostart_path()?, &desktop, OWNERSHIP_MARKER)?;
    } else {
        remove_owned_text_file(&linux_autostart_path()?, OWNERSHIP_MARKER)?;
    }

    println!("honk300: installed to {}.", root.display());
    println!("honk300: aliases linked in {}.", aliases_dir.display());
    if !path_contains(&aliases_dir) {
        println!(
            "honk300: {} is not currently on PATH; add it or open a shell that loads it.",
            aliases_dir.display()
        );
    }
    if autostart {
        println!("honk300: login autostart enabled.");
    }
    Ok(())
}

#[cfg(target_os = "macos")]
pub fn install(autostart: bool) -> Result<(), DynError> {
    let app_dir = macos_app_install_path()?;
    let installed_bin = app_dir.join("Contents").join("MacOS").join("honk300");
    let current_exe = std::env::current_exe()?;
    let disposition = macos_install_disposition(&current_exe, &app_dir)?;
    let media = macos_media_root()?;
    let aliases_dir = macos_user_alias_dir()?;
    let owned_targets = [&installed_bin as &Path];
    let alias_paths = COMMAND_NAMES
        .iter()
        .map(|name| aliases_dir.join(name))
        .collect::<Vec<_>>();
    for alias in &alias_paths {
        if unix_alias_install_decision(alias, &installed_bin, &owned_targets)?
            == AliasInstallDecision::PreserveForeign
        {
            return Err(io::Error::new(
                io::ErrorKind::AlreadyExists,
                format!(
                    "refusing to replace foreign command alias {}; move it aside and retry",
                    alias.display()
                ),
            )
            .into());
        }
    }
    let plist_path = macos_launch_agent_path()?;
    if autostart {
        preflight_owned_text_file(&plist_path, OWNERSHIP_MARKER)?;
    }
    let receipt_path = macos_receipt_path()?;
    preflight_owned_macos_receipt(&receipt_path, &app_dir)?;
    let destination_exists = fs::symlink_metadata(&app_dir).is_ok();
    let receipt_owned = receipt_is_owned(&receipt_path, &app_dir)?;
    if matches!(disposition, MacosInstallDisposition::CopyBundle(_))
        && !macos_bundle_replacement_is_owned(destination_exists, receipt_owned)
    {
        return Err(io::Error::other(format!(
            "refusing to replace an unreceipted macOS app bundle {}; move it aside or uninstall it first",
            app_dir.display()
        ))
            .into());
    }
    let _lifecycle_lease = LifecycleLease::acquire()?;
    let mut integrations =
        MacosIntegrationTransaction::capture(&alias_paths, &plist_path, &receipt_path)?;

    let (mut swap, bundle_metadata) = match disposition {
        MacosInstallDisposition::ConfigureExisting => (None, validate_macos_bundle(&app_dir)?),
        MacosInstallDisposition::CopyBundle(source) => {
            let (swap, metadata) = MacosBundleSwap::begin(&source, &app_dir)?;
            (Some(swap), metadata)
        }
    };
    integrations.begin();

    let configure = (|| -> Result<(), DynError> {
        if std::env::var_os("HONK300_TEST_FAIL_AFTER_BUNDLE_SWAP").as_deref()
            == Some(std::ffi::OsStr::new("1"))
        {
            return Err("honk300 install: injected failure after macOS bundle activation".into());
        }
        ensure_real_directory(&aliases_dir)?;
        let media_changes = migrate_legacy_user_media(
            &app_dir.join("Contents").join("Resources").join("Assets"),
            &media,
            LegacyMigrationMode::Copy,
        )?;
        integrations.record_media_migration(media_changes);
        for alias in &alias_paths {
            install_owned_unix_alias(alias, &installed_bin, &owned_targets)?;
        }

        if autostart {
            write_owned_text_file(
                &plist_path,
                &macos_launch_agent_plist(&installed_bin),
                OWNERSHIP_MARKER,
            )?;
        } else {
            remove_owned_text_file(&plist_path, OWNERSHIP_MARKER)?;
        }
        write_macos_receipt(&receipt_path, &app_dir, &bundle_metadata, autostart)?;
        Ok(())
    })();
    if let Err(error) = configure {
        let mut rollback_errors = Vec::new();
        if let Err(rollback) = integrations.rollback() {
            rollback_errors.push(format!("integration rollback failed: {rollback}"));
        }
        if let Some(swap) = swap.as_mut() {
            if let Err(rollback) = swap.rollback() {
                rollback_errors.push(format!("app bundle rollback failed: {rollback}"));
            }
        }
        if !rollback_errors.is_empty() {
            return Err(format!("{error}; additionally {}", rollback_errors.join("; ")).into());
        }
        return Err(error);
    }
    if let Some(swap) = swap.as_mut() {
        if let Err(error) = swap.commit() {
            let mut rollback_errors = Vec::new();
            if let Err(rollback) = integrations.rollback() {
                rollback_errors.push(format!("integration rollback failed: {rollback}"));
            }
            if let Err(rollback) = swap.rollback() {
                rollback_errors.push(format!("app bundle rollback failed: {rollback}"));
            }
            if rollback_errors.is_empty() {
                return Err(error.into());
            }
            return Err(format!("{error}; additionally {}", rollback_errors.join("; ")).into());
        }
    }
    integrations.commit();

    println!("honk300: installed {}.", app_dir.display());
    println!("honk300: aliases linked in {}.", aliases_dir.display());
    if !path_contains(&aliases_dir) {
        println!(
            "honk300: {} is not currently on PATH; add it or open a shell that loads it.",
            aliases_dir.display()
        );
    }
    println!("honk300: installed app signature and release metadata verified.");
    if autostart {
        println!("honk300: login autostart enabled via LaunchAgent.");
    }
    Ok(())
}

#[cfg(not(any(windows, target_os = "linux", target_os = "macos")))]
pub fn install(_autostart: bool) -> Result<(), DynError> {
    Err("honk300 install: this OS is not supported by M19 lifecycle installers.".into())
}

#[cfg(windows)]
pub fn uninstall(purge: bool) -> Result<(), DynError> {
    let source = detect_install_source();
    let current_exe = std::env::current_exe()?;
    let deferred_source = if matches!(
        source,
        InstallSource::MsiGlobal
            | InstallSource::MsiCorporate
            | InstallSource::ExeGlobal
            | InstallSource::ExeCorporate
    ) {
        if find_windows_managed_uninstall(source, &current_exe)?.is_none() {
            return Err("honk300 uninstall: the Windows installer identity could not be proven, so no installed files were touched. Uninstall Honk300 from Windows Installed Apps instead.".into());
        }
        source
    } else {
        ensure_owned_install_root(&windows_user_install_root()?, &[InstallSource::ManualLocal])?;
        // A portable copy can legitimately remove an existing receipt-owned manual install. Once
        // that ownership proof succeeds, normalize the helper request instead of trusting the
        // portable binary's otherwise-unknown provenance marker.
        InstallSource::ManualLocal
    };
    schedule_windows_deferred_uninstall(deferred_source, purge, &current_exe)
}

#[cfg(target_os = "linux")]
pub fn uninstall(purge: bool) -> Result<(), DynError> {
    if detect_install_source() == InstallSource::Deb {
        return uninstall_debian_package(purge);
    }
    let root = linux_user_install_root()?;
    let receipt = linux_receipt_path()?;
    ensure_owned_install_root_or_receipt(
        &root,
        &[InstallSource::ManualLocal, InstallSource::Shell],
        &receipt,
    )?;
    let _lifecycle_lease = LifecycleLease::acquire()?;
    let media = linux_media_root()?;
    ensure_external_media_root(&media)?;
    migrate_legacy_user_media(
        &root.join("bin").join("Assets"),
        &media,
        LegacyMigrationMode::Move,
    )?;
    let backup_root = linux_backup_root()?;
    let backup = if purge {
        backup_user_content(&media, &backup_root)?
    } else {
        None
    };

    let installed = root.join("bin").join("honk300");
    let owned_targets = [installed.as_path()];
    for name in COMMAND_NAMES {
        remove_owned_unix_alias(&linux_user_alias_dir()?.join(name), &owned_targets)?;
    }
    remove_owned_text_file(
        &linux_applications_dir()?.join("honk300.desktop"),
        OWNERSHIP_MARKER,
    )?;
    remove_owned_text_file(&linux_autostart_path()?, OWNERSHIP_MARKER)?;
    remove_managed_path_blocks_from_profiles(&home_dir()?)?;
    remove_owned_receipt(&receipt, &root)?;
    remove_dir_if_exists(&root)?;

    if purge {
        purge_config_state_preserving_foreign_receipt(&linux_config_state_root()?, &root)?;
        report_backup(backup);
    } else {
        report_preserved(media_has_user_content(&media)?.then_some(media));
    }

    println!("honk300: uninstalled.");
    Ok(())
}

#[cfg(target_os = "linux")]
fn uninstall_debian_package(purge: bool) -> Result<(), DynError> {
    let current_exe = std::env::current_exe()?;
    crate::debian::prove_current_executable(&current_exe)?;
    let _lifecycle_lease = LifecycleLease::acquire()?;
    let media = linux_media_root()?;
    ensure_external_media_root(&media)?;
    let backup = if purge {
        backup_user_content(&media, &linux_backup_root()?)?
    } else {
        None
    };

    crate::debian::remove_package()?;
    if purge {
        purge_config_state_preserving_foreign_receipt(
            &linux_config_state_root()?,
            Path::new(crate::debian::INSTALL_ROOT),
        )?;
        report_backup(backup);
    } else {
        report_preserved(media_has_user_content(&media)?.then_some(media));
    }
    println!("honk300: Debian package uninstalled.");
    Ok(())
}

#[cfg(target_os = "macos")]
pub fn uninstall(purge: bool) -> Result<(), DynError> {
    let app_dir = macos_app_install_path()?;
    let current_exe = std::env::current_exe()?;
    if !is_exact_macos_managed_executable(&current_exe, &app_dir) {
        return Err(format!(
            "honk300 uninstall: refusing to remove anything because this is not the managed app at {}. Use the official shell installer or remove it from that exact app.",
            app_dir.display()
        )
        .into());
    }
    let receipt = macos_receipt_path()?;
    preflight_owned_macos_receipt(&receipt, &app_dir)?;
    let _lifecycle_lease = LifecycleLease::acquire()?;
    let media = macos_media_root()?;
    ensure_external_media_root(&media)?;
    migrate_legacy_user_media(
        &app_dir.join("Contents").join("Resources").join("Assets"),
        &media,
        LegacyMigrationMode::Copy,
    )?;
    let backup_root = macos_backup_root()?;
    let backup = if purge {
        backup_user_content(&media, &backup_root)?
    } else {
        None
    };

    let installed = app_dir.join("Contents").join("MacOS").join("honk300");
    let owned_targets = [installed.as_path()];
    for name in COMMAND_NAMES {
        remove_owned_unix_alias(&macos_user_alias_dir()?.join(name), &owned_targets)?;
    }
    remove_owned_text_file(&macos_launch_agent_path()?, OWNERSHIP_MARKER)?;
    remove_managed_path_blocks_from_profiles(&home_dir()?)?;
    remove_owned_receipt(&receipt, &app_dir)?;
    remove_dir_if_exists(&app_dir)?;

    if purge {
        purge_config_state_preserving_foreign_receipt(&macos_config_state_root()?, &app_dir)?;
        report_backup(backup);
    } else {
        report_preserved(media_has_user_content(&media)?.then_some(media));
    }

    println!("honk300: uninstalled.");
    Ok(())
}

#[cfg(not(any(windows, target_os = "linux", target_os = "macos")))]
pub fn uninstall(_purge: bool) -> Result<(), DynError> {
    Err("honk300 uninstall: this OS is not supported by M19 lifecycle installers.".into())
}

#[cfg(windows)]
fn copy_current_exe_to_aliases(bin_dir: &Path) -> io::Result<()> {
    let ext = std::env::consts::EXE_SUFFIX;
    for name in COMMAND_NAMES {
        copy_current_exe(&bin_dir.join(format!("{name}{ext}")))?;
    }
    Ok(())
}

#[cfg(windows)]
fn copy_windows_app_launcher(bin_dir: &Path) -> io::Result<()> {
    let current = std::env::current_exe()?;
    let source = current
        .parent()
        .ok_or_else(|| io::Error::other("current executable has no parent directory"))?
        .join(WINDOWS_APP_LAUNCHER_NAME);
    let metadata = fs::symlink_metadata(&source).map_err(|error| {
        io::Error::new(
            error.kind(),
            format!(
                "windowless Windows app launcher is missing at {}; reinstall from the current Windows package: {error}",
                source.display()
            ),
        )
    })?;
    if !metadata.is_file() || metadata.file_type().is_symlink() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!(
                "windowless Windows app launcher is not a regular file: {}",
                source.display()
            ),
        ));
    }
    let destination = bin_dir.join(WINDOWS_APP_LAUNCHER_NAME);
    if !same_file_best_effort(&source, &destination) {
        fs::copy(source, destination)?;
    }
    Ok(())
}

#[cfg(any(windows, target_os = "linux"))]
fn copy_current_exe(dest: &Path) -> io::Result<()> {
    let source = std::env::current_exe()?;
    if same_file_best_effort(&source, dest) {
        return Ok(());
    }
    if let Some(parent) = dest.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::copy(source, dest)?;
    Ok(())
}

#[cfg(any(windows, target_os = "linux"))]
fn same_file_best_effort(left: &Path, right: &Path) -> bool {
    match (left.canonicalize(), right.canonicalize()) {
        (Ok(left), Ok(right)) => left == right,
        _ => false,
    }
}

fn copy_dir_recursive(source: &Path, dest: &Path) -> io::Result<()> {
    validate_real_directory(source)?;
    ensure_real_directory(dest)?;
    for entry in fs::read_dir(source)? {
        let entry = entry?;
        let source_path = entry.path();
        let dest_path = dest.join(entry.file_name());
        let file_type = entry.file_type()?;
        if file_type.is_dir() {
            copy_dir_recursive(&source_path, &dest_path)?;
        } else if file_type.is_file() {
            if let Some(parent) = dest_path.parent() {
                ensure_real_directory(parent)?;
            }
            fs::copy(&source_path, &dest_path)?;
        } else {
            return Err(io::Error::other(format!(
                "refusing to copy symlink or special file {}",
                source_path.display()
            )));
        }
    }
    Ok(())
}

fn paths_match(left: &Path, right: &Path) -> bool {
    if let (Ok(left), Ok(right)) = (left.canonicalize(), right.canonicalize()) {
        return left == right;
    }
    let normalize = |path: &Path| {
        let value = path.to_string_lossy().replace('\\', "/");
        if cfg!(windows) {
            value.to_ascii_lowercase()
        } else {
            value
        }
        .trim_end_matches('/')
        .to_owned()
    };
    normalize(left) == normalize(right)
}

fn path_is_within(path: &Path, root: &Path) -> bool {
    let normalize = |value: &Path| {
        let value = value.to_string_lossy().replace('\\', "/");
        if cfg!(windows) {
            value.to_ascii_lowercase()
        } else {
            value
        }
        .trim_end_matches('/')
        .to_owned()
    };
    let path = normalize(path);
    let root = normalize(root);
    path == root
        || path
            .strip_prefix(&root)
            .is_some_and(|rest| rest.starts_with('/'))
}

#[cfg(any(windows, target_os = "linux"))]
fn ensure_owned_install_root(root: &Path, accepted: &[InstallSource]) -> io::Result<()> {
    match fs::symlink_metadata(root) {
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(()),
        Err(error) => return Err(error),
        Ok(metadata) if !metadata.is_dir() || metadata.file_type().is_symlink() => {
            return Err(io::Error::other(format!(
                "refusing to use non-directory or symlinked install root {}",
                root.display()
            )));
        }
        Ok(_) => {}
    }
    let marker = fs::read_to_string(root.join(MARKER_FILE)).unwrap_or_default();
    let source = InstallSource::from_marker(&marker);
    if accepted.contains(&source) {
        Ok(())
    } else {
        Err(io::Error::other(format!(
            "refusing to modify unrecognized install root {}",
            root.display()
        )))
    }
}

#[cfg(target_os = "linux")]
fn ensure_owned_install_root_or_receipt(
    root: &Path,
    accepted: &[InstallSource],
    receipt: &Path,
) -> io::Result<()> {
    if receipt_is_owned(receipt, root)? {
        return Ok(());
    }
    ensure_owned_install_root(root, accepted)
}

fn ensure_real_directory(path: &Path) -> io::Result<()> {
    match fs::symlink_metadata(path) {
        Ok(metadata) => {
            if metadata.is_dir() && !metadata.file_type().is_symlink() {
                Ok(())
            } else {
                Err(io::Error::other(format!(
                    "refusing to use non-directory or symlinked path {}",
                    path.display()
                )))
            }
        }
        Err(error) if error.kind() == io::ErrorKind::NotFound => {
            fs::create_dir_all(path)?;
            validate_real_directory(path)
        }
        Err(error) => Err(error),
    }
}

fn validate_real_directory(path: &Path) -> io::Result<()> {
    let metadata = fs::symlink_metadata(path)?;
    if metadata.is_dir() && !metadata.file_type().is_symlink() {
        Ok(())
    } else {
        Err(io::Error::other(format!(
            "refusing to use non-directory or symlinked path {}",
            path.display()
        )))
    }
}

fn unix_timestamp() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

fn purge_config_state_preserving_foreign_receipt(
    root: &Path,
    owned_install_root: &Path,
) -> io::Result<()> {
    let receipt = root.join("install-receipt.json");
    if receipt.exists() && !receipt_is_owned(&receipt, owned_install_root)? {
        let metadata = fs::symlink_metadata(root)?;
        if !metadata.is_dir() || metadata.file_type().is_symlink() {
            return Err(io::Error::other(format!(
                "refusing to purge symlinked state root {}",
                root.display()
            )));
        }
        for entry in fs::read_dir(root)? {
            let entry = entry?;
            if entry.path() == receipt {
                continue;
            }
            remove_path_no_follow(&entry.path())?;
        }
        return Ok(());
    }
    remove_dir_if_exists(root)
}

fn report_backup(backup: Option<PathBuf>) {
    if let Some(path) = backup {
        println!("honk300: backed up user memes/notes to {}.", path.display());
    } else {
        println!("honk300: no user memes/notes were present to back up.");
    }
}

fn report_preserved(preserved: Option<PathBuf>) {
    if let Some(path) = preserved {
        println!(
            "honk300: kept your memes/notes at {} (uninstall did not delete them).",
            path.display()
        );
        println!("honk300: re-run with `uninstall --purge` to remove them too.");
    }
}

fn receipt_is_owned(receipt: &Path, install_root: &Path) -> io::Result<bool> {
    let metadata = match fs::symlink_metadata(receipt) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(false),
        Err(error) => return Err(error),
    };
    if !metadata.is_file() || metadata.file_type().is_symlink() {
        return Ok(false);
    }
    let value: serde_json::Value = match serde_json::from_slice(&fs::read(receipt)?) {
        Ok(value) => value,
        Err(_) => return Ok(false),
    };
    let schema = value.get("schema").and_then(serde_json::Value::as_str);
    let root = value
        .get("install_root")
        .and_then(serde_json::Value::as_str)
        .map(Path::new);
    Ok(
        matches!(schema, Some(OWNERSHIP_MARKER) | Some(INSTALL_RECEIPT_V2))
            && root.is_some_and(|recorded| paths_match(recorded, install_root)),
    )
}

fn remove_owned_receipt(receipt: &Path, install_root: &Path) -> io::Result<bool> {
    if !receipt_is_owned(receipt, install_root)? {
        return Ok(false);
    }
    fs::remove_file(receipt)?;
    Ok(true)
}

#[cfg(any(test, target_os = "linux", target_os = "macos"))]
fn write_owned_text_file(path: &Path, text: &str, marker: &str) -> io::Result<()> {
    if !text.contains(marker) {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "owned integration content is missing its ownership marker",
        ));
    }
    match fs::symlink_metadata(path) {
        Ok(metadata) => {
            if !metadata.is_file() || metadata.file_type().is_symlink() {
                return Err(io::Error::other(format!(
                    "refusing to replace non-regular integration {}",
                    path.display()
                )));
            }
            if !fs::read_to_string(path)?.contains(marker) {
                return Err(io::Error::other(format!(
                    "refusing to replace foreign integration {}",
                    path.display()
                )));
            }
        }
        Err(error) if error.kind() == io::ErrorKind::NotFound => {}
        Err(error) => return Err(error),
    }
    write_text_file(path, text)
}

#[cfg(any(test, target_os = "linux", target_os = "macos"))]
fn remove_owned_text_file(path: &Path, marker: &str) -> io::Result<bool> {
    let metadata = match fs::symlink_metadata(path) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(false),
        Err(error) => return Err(error),
    };
    if !metadata.is_file()
        || metadata.file_type().is_symlink()
        || !fs::read_to_string(path)?.contains(marker)
    {
        return Ok(false);
    }
    fs::remove_file(path)?;
    Ok(true)
}

#[cfg(any(test, target_os = "linux", target_os = "macos"))]
fn strip_managed_path_blocks(profile: &str) -> (String, bool) {
    let mut updated = profile.to_owned();
    let mut changed = false;
    while let Some(start) = updated.find(PATH_MARKER_START) {
        let after_start = start + PATH_MARKER_START.len();
        let Some(relative_end) = updated[after_start..].find(PATH_MARKER_END) else {
            break;
        };
        let mut end = after_start + relative_end + PATH_MARKER_END.len();
        if updated[end..].starts_with("\r\n") {
            end += 2;
        } else if updated[end..].starts_with('\n') {
            end += 1;
        }
        updated.replace_range(start..end, "");
        changed = true;
    }
    (updated, changed)
}

#[cfg(any(target_os = "linux", target_os = "macos"))]
fn remove_managed_path_blocks_from_profiles(home: &Path) -> io::Result<()> {
    let mut profiles = vec![home.join(".profile"), home.join(".zprofile")];
    if let Some(zdotdir) = std::env::var_os("ZDOTDIR") {
        profiles.push(PathBuf::from(zdotdir).join(".zprofile"));
    }
    profiles.sort();
    profiles.dedup();
    for profile in profiles {
        let metadata = match fs::symlink_metadata(&profile) {
            Ok(metadata) => metadata,
            Err(error) if error.kind() == io::ErrorKind::NotFound => continue,
            Err(error) => return Err(error),
        };
        if !metadata.is_file() || metadata.file_type().is_symlink() {
            continue;
        }
        let original = fs::read_to_string(&profile)?;
        let (updated, changed) = strip_managed_path_blocks(&original);
        if changed {
            fs::write(profile, updated)?;
        }
    }
    Ok(())
}

fn write_text_file(path: &Path, text: &str) -> io::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, text)
}

#[cfg(target_os = "macos")]
fn sha256_file(path: &Path) -> io::Result<String> {
    let mut file = fs::File::open(path)?;
    let mut hasher = Sha256::new();
    let mut buffer = [0u8; 64 * 1024];
    loop {
        let read = file.read(&mut buffer)?;
        if read == 0 {
            break;
        }
        hasher.update(&buffer[..read]);
    }
    Ok(format!("{:x}", hasher.finalize()))
}

#[cfg(windows)]
fn remove_file_if_exists(path: &Path) -> io::Result<()> {
    match fs::remove_file(path) {
        Ok(()) => Ok(()),
        Err(err) if err.kind() == io::ErrorKind::NotFound => Ok(()),
        Err(err) => Err(err),
    }
}

fn remove_dir_if_exists(path: &Path) -> io::Result<()> {
    match fs::remove_dir_all(path) {
        Ok(()) => Ok(()),
        Err(err) if err.kind() == io::ErrorKind::NotFound => Ok(()),
        Err(err) => Err(err),
    }
}

#[cfg(windows)]
pub fn run_windows_slot_protocol() -> Result<bool, DynError> {
    let args = std::env::args_os().skip(1).collect::<Vec<_>>();
    let Some(action) = args.first().and_then(|arg| arg.to_str()) else {
        return Ok(false);
    };
    match action {
        "__wsa" => {
            let values = parse_internal_short_args(&args[1..])?;
            let origin = InstallSource::from_marker(required_internal_short_arg(&values, "o")?);
            let target = current_windows_target_triple();
            let version = env!("CARGO_PKG_VERSION").to_owned();
            let artifact_name = canonical_windows_artifact_name(origin, target)?;
            let payload_sha256 =
                crate::update::compute_sha256_for_install(&std::env::current_exe()?)?;
            windows_slot_activate(WindowsSlotActivation {
                root: required_internal_short_path(&values, "r")?,
                origin,
                tag: format!("v{version}"),
                version,
                commit: required_internal_short_arg(&values, "c")?.to_owned(),
                target: target.to_owned(),
                artifact_name,
                artifact_path: required_internal_short_path(&values, "a")?,
                payload_sha256,
                // WiX executes Binary-table custom actions from an isolated temporary path, so
                // there cannot be a sibling honk300-app.exe to inspect here. The exact launcher
                // hash is compiled into the MSI and delivered in hidden CustomActionData; the
                // activation transaction verifies that identity against the staged slot.
                launcher_sha256: required_internal_short_arg(&values, "l")?.to_owned(),
                settings_sha256: required_internal_short_arg(&values, "s")?.to_owned(),
                accessibility_sha256: required_internal_short_arg(&values, "b")?.to_owned(),
                autostart: required_internal_bool_short(&values, "u")?,
            })?;
            Ok(true)
        }
        "__windows-slot-activate" => {
            let values = parse_internal_named_args(&args[1..])?;
            windows_slot_activate(WindowsSlotActivation {
                root: required_internal_path(&values, "root")?,
                origin: InstallSource::from_marker(required_internal_arg(&values, "origin")?),
                version: required_internal_arg(&values, "version")?.to_owned(),
                tag: required_internal_arg(&values, "tag")?.to_owned(),
                commit: required_internal_arg(&values, "commit")?.to_owned(),
                target: required_internal_arg(&values, "target")?.to_owned(),
                artifact_name: required_internal_arg(&values, "artifact-name")?.to_owned(),
                artifact_path: required_internal_path(&values, "artifact-path")?,
                payload_sha256: required_internal_arg(&values, "payload-sha256")?.to_owned(),
                launcher_sha256: current_windows_app_launcher_hash()?,
                settings_sha256: companions::current_settings_hash()?,
                accessibility_sha256: companions::current_accessibility_hash()?,
                autostart: required_internal_bool(&values, "autostart")?,
            })?;
            Ok(true)
        }
        "__windows-slot-rollback" => {
            let values = parse_internal_named_args(&args[1..])?;
            windows_slot_rollback(&required_internal_path(&values, "root")?)?;
            Ok(true)
        }
        "__windows-slot-commit" => {
            let values = parse_internal_named_args(&args[1..])?;
            windows_slot_commit(&required_internal_path(&values, "root")?)?;
            Ok(true)
        }
        "__windows-slot-uninstall" => {
            let values = parse_internal_named_args(&args[1..])?;
            let root = required_internal_path(&values, "root")?;
            let origin = InstallSource::from_marker(required_internal_arg(&values, "origin")?);
            windows_slot_uninstall(&root, origin)?;
            Ok(true)
        }
        "__windows-retire-owner" => {
            let values = parse_internal_named_args(&args[1..])?;
            let root = required_internal_path(&values, "root")?;
            let origin = InstallSource::from_marker(required_internal_arg(&values, "origin")?);
            retire_windows_registered_owner(
                &root,
                origin,
                required_internal_arg(&values, "registration")?,
            )?;
            Ok(true)
        }
        _ => Ok(false),
    }
}

#[cfg(windows)]
pub fn reject_uncommanded_windows_installer_helper() -> Result<(), DynError> {
    let executable = std::env::current_exe()?;
    if is_windows_installer_custom_action_path(&executable) {
        return Err(
            "Windows Installer extracted the Honk300 slot helper without a recognized internal command; refusing to start the desktop app"
                .into(),
        );
    }
    Ok(())
}

#[cfg(windows)]
fn is_windows_installer_custom_action_path(path: &Path) -> bool {
    path.extension()
        .and_then(|value| value.to_str())
        .is_some_and(|value| value.eq_ignore_ascii_case("tmp"))
        && path
            .file_name()
            .and_then(|value| value.to_str())
            .and_then(|value| value.get(..3))
            .is_some_and(|value| value.eq_ignore_ascii_case("msi"))
        && path
            .parent()
            .and_then(Path::file_name)
            .and_then(|value| value.to_str())
            .is_some_and(|value| value.eq_ignore_ascii_case("installer"))
}

#[cfg(windows)]
fn parse_internal_short_args(
    args: &[std::ffi::OsString],
) -> Result<std::collections::HashMap<String, std::ffi::OsString>, DynError> {
    let mut values = std::collections::HashMap::new();
    let mut index = 0;
    while index < args.len() {
        let key = args[index]
            .to_str()
            .filter(|key| key.starts_with('-') && key.len() == 2)
            .ok_or("invalid compact Windows slot argument")?[1..]
            .to_owned();
        let value = args
            .get(index + 1)
            .ok_or_else(|| format!("missing value for -{key}"))?
            .clone();
        if values.insert(key.clone(), value).is_some() {
            return Err(format!("duplicate compact argument -{key}").into());
        }
        index += 2;
    }
    Ok(values)
}

#[cfg(windows)]
fn required_internal_short_arg<'a>(
    values: &'a std::collections::HashMap<String, std::ffi::OsString>,
    key: &str,
) -> Result<&'a str, DynError> {
    values
        .get(key)
        .and_then(|value| value.to_str())
        .ok_or_else(|| format!("missing or non-Unicode compact argument -{key}").into())
}

#[cfg(windows)]
fn required_internal_short_path(
    values: &std::collections::HashMap<String, std::ffi::OsString>,
    key: &str,
) -> Result<PathBuf, DynError> {
    values
        .get(key)
        .map(PathBuf::from)
        .ok_or_else(|| format!("missing compact path -{key}").into())
}

#[cfg(windows)]
fn required_internal_bool_short(
    values: &std::collections::HashMap<String, std::ffi::OsString>,
    key: &str,
) -> Result<bool, DynError> {
    parse_internal_bool(required_internal_short_arg(values, key)?)
}

#[cfg(all(windows, target_arch = "x86_64"))]
fn current_windows_target_triple() -> &'static str {
    "x86_64-pc-windows-msvc"
}

#[cfg(all(windows, target_arch = "aarch64"))]
fn current_windows_target_triple() -> &'static str {
    "aarch64-pc-windows-msvc"
}

#[cfg(windows)]
fn canonical_windows_artifact_name(
    origin: InstallSource,
    target: &str,
) -> Result<String, DynError> {
    let name = match origin {
        InstallSource::MsiGlobal | InstallSource::PowerShell => format!("honk300-{target}.msi"),
        InstallSource::MsiCorporate => format!("honk300-{target}-corporate.msi"),
        InstallSource::ExeGlobal => format!("honk300-{target}-setup.exe"),
        InstallSource::ExeCorporate => format!("honk300-{target}-corporate-setup.exe"),
        _ => return Err("unsupported compact Windows slot origin".into()),
    };
    Ok(name)
}

#[cfg(windows)]
fn parse_internal_named_args(
    args: &[std::ffi::OsString],
) -> Result<std::collections::HashMap<String, std::ffi::OsString>, DynError> {
    let mut values = std::collections::HashMap::new();
    let mut index = 0;
    while index < args.len() {
        let key = args[index]
            .to_str()
            .filter(|key| key.starts_with("--") && key.len() > 2)
            .ok_or("invalid internal Windows slot argument")?[2..]
            .to_owned();
        let value = args
            .get(index + 1)
            .ok_or_else(|| format!("missing value for --{key}"))?
            .clone();
        if values.insert(key.clone(), value).is_some() {
            return Err(format!("duplicate internal argument --{key}").into());
        }
        index += 2;
    }
    Ok(values)
}

#[cfg(windows)]
fn required_internal_arg<'a>(
    values: &'a std::collections::HashMap<String, std::ffi::OsString>,
    key: &str,
) -> Result<&'a str, DynError> {
    values
        .get(key)
        .and_then(|value| value.to_str())
        .ok_or_else(|| format!("missing or non-Unicode internal argument --{key}").into())
}

#[cfg(windows)]
fn required_internal_path(
    values: &std::collections::HashMap<String, std::ffi::OsString>,
    key: &str,
) -> Result<PathBuf, DynError> {
    values
        .get(key)
        .map(PathBuf::from)
        .ok_or_else(|| format!("missing internal path --{key}").into())
}

#[cfg(windows)]
fn required_internal_bool(
    values: &std::collections::HashMap<String, std::ffi::OsString>,
    key: &str,
) -> Result<bool, DynError> {
    parse_internal_bool(required_internal_arg(values, key)?)
}

#[cfg(windows)]
fn parse_internal_bool(value: &str) -> Result<bool, DynError> {
    match value {
        "true" => Ok(true),
        "false" => Ok(false),
        _ => Err("invalid internal boolean argument".into()),
    }
}

#[cfg(windows)]
#[derive(Debug, Clone)]
struct WindowsSlotActivation {
    root: PathBuf,
    origin: InstallSource,
    version: String,
    tag: String,
    commit: String,
    target: String,
    artifact_name: String,
    artifact_path: PathBuf,
    payload_sha256: String,
    launcher_sha256: String,
    settings_sha256: String,
    accessibility_sha256: String,
    autostart: bool,
}

#[cfg(windows)]
fn current_windows_app_launcher_hash() -> Result<String, DynError> {
    let current = std::env::current_exe()?;
    let launcher = current
        .parent()
        .ok_or("Windows slot helper has no parent directory")?
        .join(WINDOWS_APP_LAUNCHER_NAME);
    Ok(crate::update::compute_sha256_for_install(&launcher)?)
}

#[cfg(windows)]
#[derive(Debug, Clone)]
struct WindowsSlotState {
    root: PathBuf,
    new_release: PathBuf,
    previous_current: Option<PathBuf>,
    previous_bin: Option<PathBuf>,
    legacy_bin: Option<PathBuf>,
    receipt_backup: Option<PathBuf>,
    previous_marker: Option<String>,
}

#[cfg(windows)]
#[derive(Debug, Clone)]
struct WindowsRegisteredOwner {
    source: InstallSource,
    install_root: PathBuf,
    uninstall: WindowsManagedUninstall,
    registration: String,
    logical_registration: String,
}

#[cfg(windows)]
const WINDOWS_OWNER_CLEANUP_JOURNAL: &str = ".owner-cleanup-pending.json";

#[cfg(any(test, windows))]
fn windows_origins_share_registration(left: InstallSource, right: InstallSource) -> bool {
    left == right
        || matches!(
            (left, right),
            (InstallSource::PowerShell, InstallSource::MsiGlobal)
                | (InstallSource::MsiGlobal, InstallSource::PowerShell)
        )
}

#[cfg(any(test, windows))]
fn windows_package_owns_active_slot(
    package_origin: InstallSource,
    active_origin: InstallSource,
) -> bool {
    windows_origins_share_registration(package_origin, active_origin)
}

#[cfg(any(test, windows))]
fn windows_owner_conflicts(
    active_origin: InstallSource,
    active_root: &Path,
    owner_origin: InstallSource,
    owner_root: &Path,
) -> bool {
    !paths_match(active_root, owner_root)
        || !windows_origins_share_registration(active_origin, owner_origin)
}

#[cfg(windows)]
fn windows_slot_family(origin: InstallSource) -> Option<&'static str> {
    match origin {
        InstallSource::MsiGlobal | InstallSource::PowerShell => Some("msi-global"),
        InstallSource::MsiCorporate => Some("msi-corporate"),
        InstallSource::ExeGlobal => Some("exe-global"),
        InstallSource::ExeCorporate => Some("exe-corporate"),
        _ => None,
    }
}

#[cfg(windows)]
fn windows_slot_activate(request: WindowsSlotActivation) -> Result<(), DynError> {
    validate_windows_slot_activation(&request)?;
    let root = &request.root;
    finish_windows_committed_slot_cleanup(root)?;
    let state_path = root.join(".slot-transaction.json");
    if state_path.exists() {
        windows_slot_rollback(root)?;
    }
    let slot_family = windows_slot_family(request.origin)
        .ok_or("internal Windows slot activation received a non-Windows origin")?;
    let release = root
        .join("channels")
        .join(slot_family)
        .join("releases")
        .join(format!("{}-{}", request.version, request.target));
    let release_bin = release.join("bin");
    validate_real_directory(&release_bin)?;
    for name in ["honk300.exe", "honk.exe", "goose.exe"] {
        let executable = release_bin.join(name);
        validate_regular_file_hash(&executable, &request.payload_sha256)?;
        verify_installed_version(&executable, &request.version)?;
    }
    validate_regular_file_hash(
        &release_bin.join(WINDOWS_APP_LAUNCHER_NAME),
        &request.launcher_sha256,
    )?;
    validate_regular_file_hash(
        &release_bin.join(companions::SETTINGS_NAME),
        &request.settings_sha256,
    )?;
    validate_regular_file_hash(
        &release_bin.join(companions::ACCESSIBILITY_NAME),
        &request.accessibility_sha256,
    )?;
    let current = root.join("current");
    let bin = root.join("bin");
    let previous_current = read_owned_junction_target(&current)?;
    let mut previous_bin = read_owned_junction_target(&bin)?;
    let mut legacy_bin = None;
    if bin.exists() && previous_bin.is_none() {
        let metadata = fs::symlink_metadata(&bin)?;
        if !metadata.is_dir() || metadata.file_type().is_symlink() {
            return Err(format!(
                "legacy command path is not a real directory: {}",
                bin.display()
            )
            .into());
        }
        let retired = root
            .join("channels")
            .join("legacy-flat")
            .join("releases")
            .join(format!("retired-{}", std::process::id()));
        if retired.exists() {
            return Err(format!(
                "legacy retirement path already exists: {}",
                retired.display()
            )
            .into());
        }
        fs::create_dir_all(retired.parent().expect("retired release has a parent"))?;
        legacy_bin = Some(retired);
        previous_bin = None;
    }
    let receipt = root.join("install-receipt.json");
    preflight_windows_slot_receipt(&receipt, root)?;
    let receipt_backup = receipt.exists().then(|| {
        root.join(format!(
            ".install-receipt.rollback.{}.json",
            std::process::id()
        ))
    });
    let state = WindowsSlotState {
        root: root.clone(),
        new_release: release.clone(),
        previous_current,
        previous_bin,
        legacy_bin: legacy_bin.clone(),
        receipt_backup: receipt_backup.clone(),
        previous_marker: fs::read_to_string(root.join(MARKER_FILE)).ok(),
    };
    write_windows_slot_state(&state_path, &state)?;

    let activation = (|| -> Result<(), DynError> {
        if let Some(retired) = &legacy_bin {
            fs::rename(&bin, retired)?;
        }
        windows_slot_fault("after_legacy_retirement")?;
        retarget_windows_junction(&current, &release)?;
        windows_slot_fault("after_current_junction")?;
        retarget_windows_junction(&bin, &current.join("bin"))?;
        windows_slot_fault("after_bin_junction")?;
        if let Some(backup) = &receipt_backup {
            fs::rename(&receipt, backup)?;
        }
        windows_slot_fault("before_receipt_commit")?;
        write_windows_slot_receipt(&receipt, &request, &release)?;
        windows_slot_fault("after_receipt_commit")?;
        write_text_file(&root.join(MARKER_FILE), request.origin.marker_value())?;
        windows_slot_fault("before_alias_verification")?;
        verify_windows_slot_activation(root, &request, &release)?;
        Ok(())
    })();
    if let Err(error) = activation {
        let rollback = windows_slot_rollback(root);
        if let Err(rollback_error) = rollback {
            return Err(format!(
                "Windows slot activation failed: {error}; rollback also failed: {rollback_error}"
            )
            .into());
        }
        return Err(error);
    }
    Ok(())
}

#[cfg(windows)]
fn windows_slot_fault(point: &str) -> Result<(), DynError> {
    if std::env::var("HONK300_TEST_WINDOWS_SLOT_FAIL_AT").as_deref() == Ok(point) {
        Err(format!("injected Windows slot failure at {point}").into())
    } else {
        Ok(())
    }
}

#[cfg(windows)]
fn validate_windows_slot_activation(request: &WindowsSlotActivation) -> Result<(), DynError> {
    if !request.root.is_absolute()
        || request.root.file_name().and_then(|name| name.to_str()) != Some(APP_NAME)
    {
        return Err("Windows slot root must be an absolute honk300 directory".into());
    }
    if request.tag != format!("v{}", request.version)
        || request.commit.len() != 40
        || !request.commit.chars().all(|c| c.is_ascii_hexdigit())
        || !matches!(
            request.target.as_str(),
            "x86_64-pc-windows-msvc" | "aarch64-pc-windows-msvc"
        )
        || request.artifact_name.is_empty()
        || request.payload_sha256.len() != 64
        || !request
            .payload_sha256
            .chars()
            .all(|c| c.is_ascii_hexdigit())
        || request.settings_sha256.len() != 64
        || request.accessibility_sha256.len() != 64
        || !request
            .accessibility_sha256
            .chars()
            .all(|c| c.is_ascii_hexdigit())
        || !request
            .settings_sha256
            .chars()
            .all(|c| c.is_ascii_hexdigit())
        || request.launcher_sha256.len() != 64
        || !request
            .launcher_sha256
            .chars()
            .all(|c| c.is_ascii_hexdigit())
    {
        return Err("Windows slot release identity is invalid".into());
    }
    let artifact = fs::symlink_metadata(&request.artifact_path)?;
    if !artifact.is_file() || artifact.file_type().is_symlink() {
        return Err("Windows installer artifact is not a regular pinned file".into());
    }
    Ok(())
}

#[cfg(windows)]
fn validate_regular_file_hash(path: &Path, expected: &str) -> Result<(), DynError> {
    let metadata = fs::symlink_metadata(path)?;
    if !metadata.is_file() || metadata.file_type().is_symlink() {
        return Err(format!("slot payload is not a regular file: {}", path.display()).into());
    }
    let actual = crate::update::compute_sha256_for_install(path)?;
    if !actual.eq_ignore_ascii_case(expected) {
        return Err(format!("slot payload hash mismatch: {}", path.display()).into());
    }
    Ok(())
}

#[cfg(windows)]
fn verify_installed_version(executable: &Path, expected: &str) -> Result<(), DynError> {
    use std::os::windows::process::CommandExt;

    const CREATE_NO_WINDOW: u32 = 0x0800_0000;
    let output = std::process::Command::new(executable)
        .arg("--version")
        .creation_flags(CREATE_NO_WINDOW)
        .output()?;
    let reported = String::from_utf8_lossy(&output.stdout)
        .split_whitespace()
        .last()
        .unwrap_or("")
        .split(['+', '-'])
        .next()
        .unwrap_or("")
        .to_owned();
    if !output.status.success() || reported != expected {
        return Err(format!(
            "{} reports {reported}, expected {expected}",
            executable.display()
        )
        .into());
    }
    Ok(())
}

#[cfg(windows)]
fn preflight_windows_slot_receipt(path: &Path, root: &Path) -> Result<(), DynError> {
    let metadata = match fs::symlink_metadata(path) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(()),
        Err(error) => return Err(error.into()),
    };
    if !metadata.is_file() || metadata.file_type().is_symlink() || !receipt_is_owned(path, root)? {
        return Err(format!(
            "refusing to replace foreign Windows slot receipt: {}",
            path.display()
        )
        .into());
    }
    Ok(())
}

#[cfg(windows)]
fn write_windows_slot_receipt(
    path: &Path,
    request: &WindowsSlotActivation,
    active_release: &Path,
) -> Result<(), DynError> {
    let (family, edition, scope) = match request.origin {
        InstallSource::MsiGlobal => ("msi", "global", "machine"),
        InstallSource::MsiCorporate => ("msi", "corporate", "user"),
        InstallSource::ExeGlobal => ("exe", "global", "machine"),
        InstallSource::ExeCorporate => ("exe", "corporate", "user"),
        InstallSource::PowerShell => ("powershell", "global", "machine"),
        _ => return Err("unsupported Windows receipt origin".into()),
    };
    let artifact_size = fs::metadata(&request.artifact_path)?.len();
    let artifact_hash = crate::update::compute_sha256_for_install(&request.artifact_path)?;
    let bin = request.root.join("bin");
    let receipt = serde_json::json!({
        "schema": INSTALL_RECEIPT_V2,
        "version": request.version,
        "tag": request.tag,
        "commit": request.commit,
        "channel": request.origin.marker_value(),
        "origin": request.origin.marker_value(),
        "installer_family": family,
        "edition": edition,
        "scope": scope,
        "release_track": "stable",
        "layout": "windows-slots-v1",
        "target": request.target,
        "artifact": {
            "name": request.artifact_name,
            "sha256": artifact_hash,
            "size": artifact_size
        },
        "install_root": request.root.to_string_lossy(),
        "owned_root": request.root.to_string_lossy(),
        "active_release": active_release.to_string_lossy(),
        "aliases": [
            bin.join("honk300.exe").to_string_lossy(),
            bin.join("honk.exe").to_string_lossy(),
            bin.join("goose.exe").to_string_lossy()
        ],
        "settings_app": {
            "name": companions::SETTINGS_NAME,
            "sha256": request.settings_sha256,
            "size": fs::metadata(bin.join(companions::SETTINGS_NAME))?.len(),
            "accessibility": {
                "name": companions::ACCESSIBILITY_NAME,
                "size": fs::metadata(bin.join(companions::ACCESSIBILITY_NAME))?.len(),
                "sha256": request.accessibility_sha256
            }
        },
        "app_launcher": {
            "path": bin.join(WINDOWS_APP_LAUNCHER_NAME).to_string_lossy(),
            "sha256": request.launcher_sha256
        },
        "autostart": { "enabled": request.autostart, "owner": family },
        "cleanup": { "state": "inactive_releases_retained" }
    });
    let temp = request
        .root
        .join(format!(".install-receipt.{}.tmp", std::process::id()));
    if temp.exists() {
        return Err(format!("stale receipt transaction exists: {}", temp.display()).into());
    }
    fs::write(&temp, serde_json::to_vec_pretty(&receipt)?)?;
    fs::rename(&temp, path)?;
    Ok(())
}

#[cfg(windows)]
fn verify_windows_slot_activation(
    root: &Path,
    request: &WindowsSlotActivation,
    release: &Path,
) -> Result<(), DynError> {
    if !read_owned_junction_target(&root.join("current"))?
        .is_some_and(|target| paths_match(&target, release))
    {
        return Err("stable current junction does not select the staged release".into());
    }
    let bin_target = root.join("current").join("bin");
    if !read_owned_junction_target(&root.join("bin"))?
        .is_some_and(|target| paths_match(&target, &bin_target))
    {
        return Err("stable bin junction does not follow current/bin".into());
    }
    for name in ["honk300.exe", "honk.exe", "goose.exe"] {
        let alias = root.join("bin").join(name);
        validate_regular_file_hash(&alias, &request.payload_sha256)?;
        verify_installed_version(&alias, &request.version)?;
    }
    validate_regular_file_hash(
        &root.join("bin").join(WINDOWS_APP_LAUNCHER_NAME),
        &request.launcher_sha256,
    )?;
    validate_regular_file_hash(
        &root.join("bin").join(companions::SETTINGS_NAME),
        &request.settings_sha256,
    )?;
    validate_regular_file_hash(
        &root.join("bin").join(companions::ACCESSIBILITY_NAME),
        &request.accessibility_sha256,
    )?;
    let value: serde_json::Value =
        serde_json::from_slice(&fs::read(root.join("install-receipt.json"))?)?;
    if validated_receipt_source(&value, &root.join("bin").join("honk300.exe"))
        != Some(request.origin)
    {
        return Err("activated receipt does not preserve the requested provenance".into());
    }
    Ok(())
}

#[cfg(windows)]
fn windows_slot_rollback(root: &Path) -> Result<(), DynError> {
    let state_path = root.join(".slot-transaction.json");
    if !state_path.exists() {
        return Ok(());
    }
    let state = read_windows_slot_state(&state_path)?;
    if !paths_match(&state.root, root) {
        return Err("Windows slot rollback state names a different root".into());
    }
    restore_windows_junction(&root.join("current"), state.previous_current.as_deref())?;
    if let Some(legacy) = &state.legacy_bin {
        remove_windows_junction(&root.join("bin"))?;
        if legacy.exists() {
            fs::rename(legacy, root.join("bin"))?;
        }
    } else {
        restore_windows_junction(&root.join("bin"), state.previous_bin.as_deref())?;
    }
    let receipt = root.join("install-receipt.json");
    match &state.receipt_backup {
        Some(backup) if backup.exists() => {
            if receipt.exists() {
                fs::remove_file(&receipt)?;
            }
            fs::rename(backup, &receipt)?;
        }
        Some(_) => {
            // Activation failed before moving the pre-existing receipt; leave it untouched.
        }
        None if receipt.exists() => fs::remove_file(&receipt)?,
        None => {}
    }
    let marker = root.join(MARKER_FILE);
    if let Some(previous) = state.previous_marker {
        write_text_file(&marker, &previous)?;
    } else if marker.exists() {
        fs::remove_file(marker)?;
    }
    fs::remove_file(state_path)?;
    Ok(())
}

#[cfg(windows)]
fn windows_slot_commit(root: &Path) -> Result<(), DynError> {
    let state_path = root.join(".slot-transaction.json");
    let committed_path = root.join(".slot-committed.json");
    if state_path.exists() {
        if committed_path.exists() {
            return Err("Windows slot committed-cleanup journal already exists".into());
        }
        fs::rename(&state_path, &committed_path)?;
    } else if !committed_path.exists() {
        return Ok(());
    }
    finish_windows_committed_slot_cleanup(root)?;
    // Conflicting-owner retirement is deliberately post-commit. The native installer must keep
    // its own registration and staged slot intact; the initiating `honk300 update` observes the
    // journal during final verification and returns nonzero cleanup_pending. A direct graphical
    // install retains the same state for the next explicit update/cleanup retry.
    refresh_windows_owner_cleanup_journal(root)?;
    Ok(())
}

#[cfg(windows)]
fn finish_windows_committed_slot_cleanup(root: &Path) -> Result<(), DynError> {
    let committed_path = root.join(".slot-committed.json");
    if !committed_path.exists() {
        return Ok(());
    }
    let state = read_windows_slot_state(&committed_path)?;
    if !paths_match(&state.root, root) {
        return Err("Windows committed slot journal names a different root".into());
    }
    windows_slot_fault("commit_cleanup")?;
    if let Some(backup) = state.receipt_backup {
        if backup.exists() {
            fs::remove_file(backup)?;
        }
    }
    fs::remove_file(committed_path)?;
    Ok(())
}

#[cfg(windows)]
fn windows_registered_owners() -> Result<Vec<WindowsRegisteredOwner>, DynError> {
    use winreg::enums::{
        HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE, KEY_READ, KEY_WOW64_32KEY, KEY_WOW64_64KEY,
    };
    use winreg::RegKey;

    let mut owners = Vec::new();
    for (hive_name, hive) in [("HKCU", HKEY_CURRENT_USER), ("HKLM", HKEY_LOCAL_MACHINE)] {
        let root = RegKey::predef(hive);
        for (view_name, view) in [("64", KEY_WOW64_64KEY), ("32", KEY_WOW64_32KEY)] {
            let uninstall = match root.open_subkey_with_flags(
                "Software\\Microsoft\\Windows\\CurrentVersion\\Uninstall",
                KEY_READ | view,
            ) {
                Ok(key) => key,
                Err(error) if error.kind() == io::ErrorKind::NotFound => continue,
                Err(error) => return Err(error.into()),
            };
            for key_name in uninstall.enum_keys().flatten() {
                let Ok(key) = uninstall.open_subkey_with_flags(&key_name, KEY_READ) else {
                    continue;
                };
                let identity = WindowsUninstallIdentity {
                    key_name: key_name.clone(),
                    display_name: key.get_value("DisplayName").unwrap_or_default(),
                    publisher: key.get_value("Publisher").unwrap_or_default(),
                    install_location: key
                        .get_value::<String, _>("InstallLocation")
                        .map(PathBuf::from)
                        .unwrap_or_default(),
                    uninstall_command: key.get_value("UninstallString").unwrap_or_default(),
                    windows_installer: key
                        .get_value::<u32, _>("WindowsInstaller")
                        .unwrap_or_default()
                        != 0,
                };
                if !identity.install_location.is_absolute()
                    || identity
                        .install_location
                        .file_name()
                        .and_then(|name| name.to_str())
                        != Some(APP_NAME)
                {
                    continue;
                }
                let candidate = identity.install_location.join("bin").join("honk300.exe");
                let source = if identity.windows_installer {
                    if identity.display_name.eq_ignore_ascii_case(DISPLAY_NAME) {
                        InstallSource::MsiGlobal
                    } else if identity
                        .display_name
                        .eq_ignore_ascii_case("honk300 (Corporate Edition)")
                    {
                        InstallSource::MsiCorporate
                    } else {
                        continue;
                    }
                } else if identity
                    .key_name
                    .eq_ignore_ascii_case("{5A94FBD0-DA02-4F63-9363-7D9CE0E280F5}_is1")
                {
                    InstallSource::ExeGlobal
                } else if identity
                    .key_name
                    .eq_ignore_ascii_case("{A072F01B-0AE8-4ED9-B67F-845ADF7831F9}_is1")
                {
                    InstallSource::ExeCorporate
                } else {
                    continue;
                };
                if !windows_registration_hive_is_valid(source, hive == HKEY_LOCAL_MACHINE) {
                    // Never elevate an uninstall command discovered in a user-writable fake
                    // Global registration. Corporate EXE is per-user. Windows Installer can
                    // register the per-user Corporate MSI in either HKCU or its protected HKLM
                    // product inventory while retaining a LocalAppData payload and user scope.
                    continue;
                }
                let Some(uninstall) =
                    validate_windows_uninstall_identity(source, &candidate, &identity)
                else {
                    continue;
                };
                owners.push(WindowsRegisteredOwner {
                    source,
                    install_root: identity.install_location,
                    uninstall,
                    registration: format!("{hive_name}:{view_name}:{key_name}"),
                    // HKCU's uninstall key is shared between registry views on supported
                    // Windows versions. Preserve the observed view for the journal, but treat
                    // byte-for-byte equivalent owner evidence as one logical registration.
                    logical_registration: format!("{hive_name}:{}", key_name.to_ascii_lowercase()),
                });
            }
        }
    }
    deduplicate_windows_registered_owners(&mut owners);
    Ok(owners)
}

#[cfg(windows)]
fn deduplicate_windows_registered_owners(owners: &mut Vec<WindowsRegisteredOwner>) {
    owners.sort_by(|left, right| {
        left.logical_registration
            .cmp(&right.logical_registration)
            .then_with(|| left.registration.cmp(&right.registration))
    });
    owners.dedup_by(|left, right| {
        left.logical_registration == right.logical_registration
            && left.source == right.source
            && paths_match(&left.install_root, &right.install_root)
            && left.uninstall == right.uninstall
    });
}

#[cfg(any(test, windows))]
fn windows_registration_hive_is_valid(source: InstallSource, machine_hive: bool) -> bool {
    match source {
        InstallSource::MsiGlobal | InstallSource::ExeGlobal => machine_hive,
        InstallSource::MsiCorporate => true,
        InstallSource::ExeCorporate => !machine_hive,
        _ => false,
    }
}

#[cfg(windows)]
fn active_windows_slot_identity(
    root: &Path,
) -> Result<(InstallSource, serde_json::Value), DynError> {
    let receipt_path = root.join("install-receipt.json");
    let metadata = fs::symlink_metadata(&receipt_path)?;
    if !metadata.is_file() || metadata.file_type().is_symlink() {
        return Err("active Windows slot receipt is not a protected regular file".into());
    }
    let receipt: serde_json::Value = serde_json::from_slice(&fs::read(&receipt_path)?)?;
    let executable = root.join("bin").join("honk300.exe");
    let source = validated_receipt_source(&receipt, &executable)
        .ok_or("active Windows slot receipt is not authoritative")?;
    Ok((source, receipt))
}

#[cfg(windows)]
fn pending_windows_registered_owners(
    active_origin: InstallSource,
    active_root: &Path,
) -> Result<Vec<WindowsRegisteredOwner>, DynError> {
    let owners = windows_registered_owners()?;
    if !owners.iter().any(|owner| {
        !windows_owner_conflicts(
            active_origin,
            active_root,
            owner.source,
            &owner.install_root,
        )
    }) {
        // Conflict cleanup is a package-owner operation. An internal slot harness, portable copy,
        // or forged receipt without its matching native registration must never gain the ability
        // to retire an unrelated installed product.
        return Ok(Vec::new());
    }
    Ok(owners
        .into_iter()
        .filter(|owner| {
            windows_owner_conflicts(
                active_origin,
                active_root,
                owner.source,
                &owner.install_root,
            )
        })
        .collect())
}

#[cfg(windows)]
fn windows_owner_assisted_command(owner: &WindowsRegisteredOwner) -> String {
    match &owner.uninstall {
        WindowsManagedUninstall::Msi { product_code, .. } => {
            format!("msiexec.exe /x {product_code} /passive /norestart")
        }
        WindowsManagedUninstall::Exe { uninstaller, .. } => format!(
            "\"{}\" /VERYSILENT /SUPPRESSMSGBOXES /NORESTART",
            uninstaller.display()
        ),
    }
}

#[cfg(windows)]
fn set_windows_receipt_cleanup_state(
    root: &Path,
    mut receipt: serde_json::Value,
    state: &str,
) -> Result<(), DynError> {
    if receipt
        .pointer("/cleanup/state")
        .and_then(serde_json::Value::as_str)
        == Some(state)
    {
        return Ok(());
    }
    receipt["cleanup"] = serde_json::json!({ "state": state });
    let path = root.join("install-receipt.json");
    let temp = root.join(format!(
        ".install-receipt.cleanup.{}.tmp",
        std::process::id()
    ));
    if temp.exists() {
        return Err(format!(
            "stale receipt cleanup transaction exists: {}",
            temp.display()
        )
        .into());
    }
    fs::write(&temp, serde_json::to_vec_pretty(&receipt)?)?;
    fs::rename(temp, path)?;
    Ok(())
}

#[cfg(windows)]
fn refresh_windows_owner_cleanup_journal(root: &Path) -> Result<usize, DynError> {
    let (active_origin, receipt) = active_windows_slot_identity(root)?;
    let conflicts = pending_windows_registered_owners(active_origin, root)?;
    let journal = root.join(WINDOWS_OWNER_CLEANUP_JOURNAL);
    if conflicts.is_empty() {
        remove_file_if_exists(&journal)?;
        set_windows_receipt_cleanup_state(root, receipt, "inactive_releases_retained")?;
        return Ok(0);
    }
    if let Ok(metadata) = fs::symlink_metadata(&journal) {
        if !metadata.is_file() || metadata.file_type().is_symlink() {
            return Err("Windows owner cleanup journal is not a protected regular file".into());
        }
    }
    let value = serde_json::json!({
        "schema": "honk300.windows-owner-cleanup.v1",
        "state": "cleanup_pending",
        "active_origin": active_origin.marker_value(),
        "active_root": root.to_string_lossy(),
        "conflicts": conflicts.iter().map(|owner| serde_json::json!({
            "origin": owner.source.marker_value(),
            "install_root": owner.install_root.to_string_lossy(),
            "registration": owner.registration,
            "assisted_command": windows_owner_assisted_command(owner),
        })).collect::<Vec<_>>(),
    });
    let temp = root.join(format!(
        ".{WINDOWS_OWNER_CLEANUP_JOURNAL}.{}.tmp",
        std::process::id()
    ));
    if temp.exists() {
        return Err(format!("stale owner cleanup transaction exists: {}", temp.display()).into());
    }
    fs::write(&temp, serde_json::to_vec_pretty(&value)?)?;
    fs::rename(temp, journal)?;
    set_windows_receipt_cleanup_state(root, receipt, "cleanup_pending")?;
    Ok(conflicts.len())
}

#[cfg(windows)]
fn windows_owner_retirement_invocation(
    active_executable: &Path,
    active_root: &Path,
    active_origin: InstallSource,
    owner: &WindowsRegisteredOwner,
) -> WindowsPostExitInvocation {
    // Start-Process joins ArgumentList into one Windows command line. Preserve explicit quotes
    // around the root so Program Files remains one argv value in the elevated coordinator. MSI
    // directory properties retain a trailing backslash; remove only that redundant separator so
    // it cannot escape the closing quote under CommandLineToArgvW parsing.
    let root = active_root.to_string_lossy();
    let quoted_root = format!("\"{}\"", root.trim_end_matches(['\\', '/']));
    let elevation = if owner.uninstall.requires_elevation() {
        " -Verb RunAs"
    } else {
        ""
    };
    let script = format!(
        "$ErrorActionPreference='Stop'; $process=Start-Process -FilePath '{}' -ArgumentList @('__windows-retire-owner','--root','{}','--origin','{}','--registration','{}') -WindowStyle Hidden -Wait -PassThru{elevation}; exit $process.ExitCode",
        powershell_literal(&active_executable.to_string_lossy()),
        powershell_literal(&quoted_root),
        active_origin.marker_value(),
        powershell_literal(&owner.registration),
    );
    WindowsPostExitInvocation {
        args: vec![
            "-NoProfile".to_owned(),
            "-WindowStyle".to_owned(),
            "Hidden".to_owned(),
            "-ExecutionPolicy".to_owned(),
            "Bypass".to_owned(),
            "-Command".to_owned(),
            script.clone(),
        ],
        script,
    }
}

#[cfg(windows)]
fn retire_windows_registered_owner(
    root: &Path,
    expected_origin: InstallSource,
    registration: &str,
) -> Result<(), DynError> {
    use std::os::windows::process::CommandExt;

    const CREATE_NO_WINDOW: u32 = 0x0800_0000;
    let journal = root.join(WINDOWS_OWNER_CLEANUP_JOURNAL);
    let metadata = fs::symlink_metadata(&journal)
        .map_err(|error| format!("conflicting-owner cleanup journal is unavailable: {error}"))?;
    if !metadata.is_file() || metadata.file_type().is_symlink() {
        return Err("conflicting-owner cleanup journal is not a protected regular file".into());
    }
    let (active_origin, _) = active_windows_slot_identity(root)?;
    if !windows_origins_share_registration(active_origin, expected_origin) {
        return Err("conflicting-owner coordinator belongs to a different active origin".into());
    }
    let mut matching = pending_windows_registered_owners(active_origin, root)?
        .into_iter()
        .filter(|owner| owner.registration == registration)
        .collect::<Vec<_>>();
    if matching.len() != 1 {
        return Err(format!(
            "conflicting-owner coordinator expected exactly one protected registration `{registration}`, found {}",
            matching.len()
        )
        .into());
    }
    let owner = matching.pop().expect("length checked");
    let mut command = match &owner.uninstall {
        WindowsManagedUninstall::Msi { product_code, .. } => {
            let mut command = std::process::Command::new(system_windows_msiexec_path()?);
            command.args(["/x", product_code, "/qn", "/norestart"]);
            command
        }
        WindowsManagedUninstall::Exe { uninstaller, .. } => {
            let mut command = std::process::Command::new(uninstaller);
            command.args(["/VERYSILENT", "/SUPPRESSMSGBOXES", "/NORESTART"]);
            command
        }
    };
    let status = command
        .creation_flags(CREATE_NO_WINDOW)
        .status()
        .map_err(|error| {
            format!(
                "could not start protected conflicting owner {}: {error}",
                owner.registration
            )
        })?;
    if !status.success() && status.code() != Some(1605) {
        return Err(format!(
            "protected conflicting owner {} exited {}; reboot-deferred results are not accepted",
            owner.registration,
            status.code().unwrap_or(-1)
        )
        .into());
    }
    if windows_registered_owners()?
        .iter()
        .any(|candidate| candidate.registration == owner.registration)
    {
        return Err(format!(
            "protected conflicting owner {} remained registered after its uninstaller returned",
            owner.registration
        )
        .into());
    }

    // Native installer tables can intentionally retain shared PATH components across an
    // in-place family takeover. Once a differently rooted owner is gone, explicitly retire only
    // that owner's exact stable bin and Run value. This executes in the same elevated coordinator
    // as a machine-wide uninstall, so it cannot leave a stale higher-precedence public command.
    if !paths_match(root, &owner.install_root) {
        remove_windows_retired_owner_integrations(&owner.install_root, owner.source)?;
    }
    let (verified_origin, _) = active_windows_slot_identity(root)?;
    if !windows_origins_share_registration(verified_origin, expected_origin) {
        return Err("active Windows owner changed during conflicting-owner retirement".into());
    }
    if !windows_public_path_contains(root, verified_origin)? {
        return Err(format!(
            "active Windows owner {} is missing its exact persisted public PATH entry",
            verified_origin.marker_value()
        )
        .into());
    }
    if !paths_match(root, &owner.install_root)
        && windows_public_path_contains(&owner.install_root, owner.source)?
    {
        return Err(format!(
            "retired Windows owner {} still owns its persisted public PATH entry",
            owner.registration
        )
        .into());
    }
    Ok(())
}

#[cfg(windows)]
pub(crate) fn windows_owner_cleanup_is_pending(root: &Path) -> bool {
    root.join(WINDOWS_OWNER_CLEANUP_JOURNAL).exists()
}

#[cfg(windows)]
pub(crate) fn discover_windows_owner_cleanup(root: &Path) -> Result<bool, DynError> {
    Ok(refresh_windows_owner_cleanup_journal(root)? != 0)
}

#[cfg(windows)]
pub(crate) fn retry_windows_owner_cleanup(
    root: &Path,
    expected_origin: InstallSource,
) -> Result<bool, DynError> {
    use std::os::windows::process::CommandExt;

    const CREATE_NO_WINDOW: u32 = 0x0800_0000;
    let journal = root.join(WINDOWS_OWNER_CLEANUP_JOURNAL);
    if !journal.exists() {
        return Ok(false);
    }
    let (active_origin, _) = active_windows_slot_identity(root)
        .map_err(|error| format!("could not validate the active Windows owner: {error}"))?;
    if !windows_origins_share_registration(active_origin, expected_origin) {
        return Err("pending Windows owner cleanup belongs to a different active origin".into());
    }
    let powershell = system_windows_powershell_path()
        .map_err(|error| format!("could not resolve system Windows PowerShell: {error}"))?;
    let active_executable = root.join("bin").join("honk300.exe");
    let owners = pending_windows_registered_owners(active_origin, root)
        .map_err(|error| format!("could not discover conflicting Windows owners: {error}"))?;
    for owner in owners {
        let invocation =
            windows_owner_retirement_invocation(&active_executable, root, active_origin, &owner);
        let status = std::process::Command::new(&powershell)
            .args(&invocation.args)
            .creation_flags(CREATE_NO_WINDOW)
            .status()
            .map_err(|error| {
                format!(
                    "could not launch the protected conflicting-owner cleanup helper {}: {error}",
                    powershell.display()
                )
            })?;
        if !status.success() {
            let remaining = refresh_windows_owner_cleanup_journal(root).map_err(|error| {
                format!("conflicting-owner cleanup failed and its journal could not be refreshed: {error}")
            })?;
            return Err(format!(
                "the selected Windows slot remains active, but conflicting owner {} could not be retired (exit {}); cleanup_pending with {remaining} owner(s). Assisted command: {}",
                owner.registration,
                status.code().unwrap_or(-1),
                windows_owner_assisted_command(&owner)
            )
            .into());
        }
    }
    let remaining = refresh_windows_owner_cleanup_journal(root)
        .map_err(|error| format!("could not verify conflicting-owner cleanup: {error}"))?;
    if remaining != 0 {
        return Err(format!(
            "the selected Windows slot remains active, but {remaining} conflicting owner(s) are still registered; cleanup_pending"
        )
        .into());
    }
    Ok(true)
}

#[cfg(windows)]
fn windows_slot_uninstall(root: &Path, origin: InstallSource) -> Result<(), DynError> {
    if windows_slot_family(origin).is_none() {
        return Err("unsupported Windows slot uninstall origin".into());
    }
    finish_windows_committed_slot_cleanup(root)?;
    let receipt = root.join("install-receipt.json");
    if !receipt.exists() {
        return Ok(());
    }
    let value: serde_json::Value = serde_json::from_slice(&fs::read(&receipt)?)?;
    let executable = root.join("bin").join("honk300.exe");
    if !validated_receipt_source(&value, &executable)
        .is_some_and(|active| windows_package_owns_active_slot(origin, active))
    {
        // A newer cross-channel install owns the neutral selectors. This uninstaller may retire
        // only its registration and inactive payloads; it must not deactivate the latest intent.
        return Ok(());
    }
    remove_windows_slot_integrations(root, origin)?;
    remove_windows_junction(&root.join("bin"))?;
    remove_windows_junction(&root.join("current"))?;
    remove_file_if_exists(&root.join(WINDOWS_OWNER_CLEANUP_JOURNAL))?;
    fs::remove_file(receipt)?;
    let marker = root.join(MARKER_FILE);
    if fs::read_to_string(&marker).is_ok_and(|value| InstallSource::from_marker(&value) == origin) {
        fs::remove_file(marker)?;
    }
    Ok(())
}

#[cfg(windows)]
fn remove_windows_slot_integrations(root: &Path, origin: InstallSource) -> Result<(), DynError> {
    use winreg::enums::{HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE, KEY_QUERY_VALUE, KEY_SET_VALUE};
    use winreg::RegKey;

    let (hive, environment_key, start_menu_root, desktop_root) = match origin {
        InstallSource::MsiGlobal | InstallSource::ExeGlobal | InstallSource::PowerShell => (
            HKEY_LOCAL_MACHINE,
            "SYSTEM\\CurrentControlSet\\Control\\Session Manager\\Environment",
            std::env::var_os("ProgramData").map(PathBuf::from),
            std::env::var_os("PUBLIC").map(|path| PathBuf::from(path).join("Desktop")),
        ),
        InstallSource::MsiCorporate | InstallSource::ExeCorporate => (
            HKEY_CURRENT_USER,
            "Environment",
            std::env::var_os("APPDATA")
                .map(PathBuf::from)
                .map(|path| path.join("Microsoft/Windows/Start Menu/Programs")),
            std::env::var_os("USERPROFILE").map(|path| PathBuf::from(path).join("Desktop")),
        ),
        _ => return Err("unsupported Windows integration owner".into()),
    };
    let registry = RegKey::predef(hive);
    if let Ok(environment) =
        registry.open_subkey_with_flags(environment_key, KEY_QUERY_VALUE | KEY_SET_VALUE)
    {
        remove_windows_registry_path(&environment, &root.join("bin"))?;
    }
    if let Ok(honk300) =
        registry.open_subkey_with_flags("Software\\Honk300", KEY_QUERY_VALUE | KEY_SET_VALUE)
    {
        let recorded: String = honk300.get_value("InstallSource").unwrap_or_default();
        if InstallSource::from_marker(&recorded) == origin {
            let _ = honk300.delete_value("InstallSource");
        }
    }
    if let Ok(run) = registry.open_subkey_with_flags(
        "Software\\Microsoft\\Windows\\CurrentVersion\\Run",
        KEY_QUERY_VALUE | KEY_SET_VALUE,
    ) {
        let command: String = run.get_value("Honk300").unwrap_or_default();
        if command_executable_path(&command)
            .is_some_and(|executable| path_is_within(&executable, root))
        {
            let _ = run.delete_value("Honk300");
        }
    }

    if let Some(programs) = start_menu_root {
        let programs = if hive == HKEY_LOCAL_MACHINE {
            programs.join("Microsoft/Windows/Start Menu/Programs")
        } else {
            programs
        };
        let group = programs.join("honk300");
        remove_file_if_exists(&group.join("Honk300.lnk"))?;
        let _ = fs::remove_dir(group);
    }
    if let Some(desktop) = desktop_root {
        remove_file_if_exists(&desktop.join("Honk300.lnk"))?;
    }
    Ok(())
}

#[cfg(windows)]
fn remove_windows_registry_path(key: &winreg::RegKey, removed: &Path) -> io::Result<()> {
    use std::os::windows::ffi::OsStrExt;

    let current: String = match key.get_value("Path") {
        Ok(value) => value,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(()),
        Err(error) => return Err(error),
    };
    let Some(updated) = windows_path_without_entry(&current, removed) else {
        return Ok(());
    };
    let mut raw = key.get_raw_value("Path")?;
    let mut bytes = std::ffi::OsStr::new(&updated)
        .encode_wide()
        .flat_map(u16::to_le_bytes)
        .collect::<Vec<_>>();
    bytes.extend_from_slice(&0_u16.to_le_bytes());
    raw.bytes = bytes;
    key.set_raw_value("Path", &raw)
}

#[cfg(windows)]
fn remove_windows_retired_owner_integrations(
    root: &Path,
    origin: InstallSource,
) -> Result<(), DynError> {
    use winreg::enums::{HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE, KEY_QUERY_VALUE, KEY_SET_VALUE};
    use winreg::RegKey;

    let (hive, environment_key) = match origin {
        InstallSource::MsiGlobal | InstallSource::ExeGlobal | InstallSource::PowerShell => (
            HKEY_LOCAL_MACHINE,
            "SYSTEM\\CurrentControlSet\\Control\\Session Manager\\Environment",
        ),
        InstallSource::MsiCorporate | InstallSource::ExeCorporate => {
            (HKEY_CURRENT_USER, "Environment")
        }
        _ => return Err("unsupported retired Windows integration owner".into()),
    };
    let registry = RegKey::predef(hive);
    match registry.open_subkey_with_flags(environment_key, KEY_QUERY_VALUE | KEY_SET_VALUE) {
        Ok(environment) => remove_windows_registry_path(&environment, &root.join("bin"))?,
        Err(error) if error.kind() == io::ErrorKind::NotFound => {}
        Err(error) => return Err(error.into()),
    }
    match registry.open_subkey_with_flags(
        "Software\\Microsoft\\Windows\\CurrentVersion\\Run",
        KEY_QUERY_VALUE | KEY_SET_VALUE,
    ) {
        Ok(run) => {
            let command: String = run.get_value("Honk300").unwrap_or_default();
            if command_executable_path(&command)
                .is_some_and(|executable| path_is_within(&executable, root))
            {
                run.delete_value("Honk300")?;
            }
        }
        Err(error) if error.kind() == io::ErrorKind::NotFound => {}
        Err(error) => return Err(error.into()),
    }
    Ok(())
}

#[cfg(windows)]
fn windows_public_path_contains(root: &Path, origin: InstallSource) -> Result<bool, DynError> {
    use winreg::enums::{HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE, KEY_QUERY_VALUE};
    use winreg::RegKey;

    let (hive, environment_key) = match origin {
        InstallSource::MsiGlobal | InstallSource::ExeGlobal | InstallSource::PowerShell => (
            HKEY_LOCAL_MACHINE,
            "SYSTEM\\CurrentControlSet\\Control\\Session Manager\\Environment",
        ),
        InstallSource::MsiCorporate | InstallSource::ExeCorporate => {
            (HKEY_CURRENT_USER, "Environment")
        }
        _ => return Err("unsupported active Windows PATH owner".into()),
    };
    let registry = RegKey::predef(hive);
    let environment = match registry.open_subkey_with_flags(environment_key, KEY_QUERY_VALUE) {
        Ok(environment) => environment,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(false),
        Err(error) => return Err(error.into()),
    };
    let current: String = match environment.get_value("Path") {
        Ok(current) => current,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(false),
        Err(error) => return Err(error.into()),
    };
    Ok(current
        .split(';')
        .any(|entry| windows_path_entry_matches(entry, &root.join("bin"))))
}

#[cfg(any(test, windows))]
fn windows_path_entry_matches(entry: &str, expected: &Path) -> bool {
    entry
        .trim()
        .trim_end_matches(['\\', '/'])
        .eq_ignore_ascii_case(expected.to_string_lossy().trim_end_matches(['\\', '/']))
}

#[cfg(any(test, windows))]
fn windows_path_without_entry(current: &str, removed: &Path) -> Option<String> {
    let parts = current.split(';').collect::<Vec<_>>();
    let kept = parts
        .iter()
        .copied()
        .filter(|part| !windows_path_entry_matches(part, removed))
        .collect::<Vec<_>>();
    (kept.len() != parts.len()).then(|| kept.join(";"))
}

#[cfg(windows)]
fn write_windows_slot_state(path: &Path, state: &WindowsSlotState) -> Result<(), DynError> {
    let value = serde_json::json!({
        "schema": "honk300.windows-slot-transaction.v1",
        "root": state.root.to_string_lossy(),
        "new_release": state.new_release.to_string_lossy(),
        "previous_current": state.previous_current.as_ref().map(|path| path.to_string_lossy()),
        "previous_bin": state.previous_bin.as_ref().map(|path| path.to_string_lossy()),
        "legacy_bin": state.legacy_bin.as_ref().map(|path| path.to_string_lossy()),
        "receipt_backup": state.receipt_backup.as_ref().map(|path| path.to_string_lossy()),
        "previous_marker": state.previous_marker.as_deref(),
    });
    let temp = path.with_extension(format!("{}.tmp", std::process::id()));
    fs::write(&temp, serde_json::to_vec_pretty(&value)?)?;
    fs::rename(temp, path)?;
    Ok(())
}

#[cfg(windows)]
fn read_windows_slot_state(path: &Path) -> Result<WindowsSlotState, DynError> {
    let metadata = fs::symlink_metadata(path)?;
    if !metadata.is_file() || metadata.file_type().is_symlink() {
        return Err("Windows slot state is not a regular file".into());
    }
    let value: serde_json::Value = serde_json::from_slice(&fs::read(path)?)?;
    if value.get("schema").and_then(serde_json::Value::as_str)
        != Some("honk300.windows-slot-transaction.v1")
    {
        return Err("Windows slot state schema is invalid".into());
    }
    let path_value = |key: &str| {
        value
            .get(key)
            .and_then(serde_json::Value::as_str)
            .map(PathBuf::from)
    };
    Ok(WindowsSlotState {
        root: path_value("root").ok_or("Windows slot state has no root")?,
        new_release: path_value("new_release").ok_or("Windows slot state has no new release")?,
        previous_current: path_value("previous_current"),
        previous_bin: path_value("previous_bin"),
        legacy_bin: path_value("legacy_bin"),
        receipt_backup: path_value("receipt_backup"),
        previous_marker: value
            .get("previous_marker")
            .and_then(serde_json::Value::as_str)
            .map(str::to_owned),
    })
}

#[cfg(windows)]
fn read_owned_junction_target(path: &Path) -> Result<Option<PathBuf>, DynError> {
    match fs::symlink_metadata(path) {
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(error.into()),
        Ok(metadata) if metadata.file_type().is_symlink() => Ok(Some(fs::read_link(path)?)),
        Ok(_) => Ok(None),
    }
}

#[cfg(windows)]
fn restore_windows_junction(path: &Path, target: Option<&Path>) -> Result<(), DynError> {
    match target {
        Some(target) => retarget_windows_junction(path, target),
        None => remove_windows_junction(path),
    }
}

#[cfg(windows)]
fn remove_windows_junction(path: &Path) -> Result<(), DynError> {
    match fs::remove_dir(path) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(error.into()),
    }
}

#[cfg(windows)]
fn retarget_windows_junction(junction: &Path, target: &Path) -> Result<(), DynError> {
    use std::os::windows::ffi::OsStrExt;

    const GENERIC_WRITE: u32 = 0x4000_0000;
    const FILE_SHARE_ALL: u32 = 0x0000_0007;
    const OPEN_EXISTING: u32 = 3;
    const FILE_FLAG_OPEN_REPARSE_POINT: u32 = 0x0020_0000;
    const FILE_FLAG_BACKUP_SEMANTICS: u32 = 0x0200_0000;
    const FSCTL_SET_REPARSE_POINT: u32 = 0x0009_00A4;
    const IO_REPARSE_TAG_MOUNT_POINT: u32 = 0xA000_0003;
    const INVALID_HANDLE_VALUE: isize = -1;

    #[link(name = "Kernel32")]
    unsafe extern "system" {
        fn CreateFileW(
            name: *const u16,
            access: u32,
            share: u32,
            security: *const std::ffi::c_void,
            creation: u32,
            flags: u32,
            template: isize,
        ) -> isize;
        fn DeviceIoControl(
            handle: isize,
            code: u32,
            input: *const std::ffi::c_void,
            input_size: u32,
            output: *mut std::ffi::c_void,
            output_size: u32,
            returned: *mut u32,
            overlapped: *mut std::ffi::c_void,
        ) -> i32;
        fn CloseHandle(handle: isize) -> i32;
    }

    let absolute_target = if target.is_absolute() {
        target.to_path_buf()
    } else {
        std::env::current_dir()?.join(target)
    };
    // Prove that the lexical target currently resolves before storing that lexical path. Keeping
    // `root\current\bin` lexical is what makes the public bin junction follow later activations.
    absolute_target.canonicalize()?;
    let created = match fs::symlink_metadata(junction) {
        Err(error) if error.kind() == io::ErrorKind::NotFound => {
            fs::create_dir(junction)?;
            true
        }
        Err(error) => return Err(error.into()),
        Ok(metadata) if metadata.file_type().is_symlink() => false,
        Ok(_) => {
            return Err(format!("refusing to retarget non-junction {}", junction.display()).into())
        }
    };
    let result = (|| -> Result<(), DynError> {
        let mut junction_wide = junction.as_os_str().encode_wide().collect::<Vec<_>>();
        junction_wide.push(0);
        let canonical_text = absolute_target.to_string_lossy();
        let print_text = canonical_text
            .strip_prefix(r"\\?\UNC\")
            .map(|path| format!(r"\\{path}"))
            .or_else(|| canonical_text.strip_prefix(r"\\?\").map(str::to_owned))
            .unwrap_or_else(|| canonical_text.into_owned());
        let print = std::ffi::OsStr::new(&print_text)
            .encode_wide()
            .collect::<Vec<_>>();
        let substitute = format!(r"\??\{print_text}")
            .encode_utf16()
            .collect::<Vec<_>>();
        let substitute_bytes = substitute.len() * 2;
        let print_bytes = print.len() * 2;
        let path_bytes = substitute_bytes + 2 + print_bytes + 2;
        let data_length = 8 + path_bytes;
        let mut buffer = vec![0u8; 8 + data_length];
        buffer[0..4].copy_from_slice(&IO_REPARSE_TAG_MOUNT_POINT.to_le_bytes());
        buffer[4..6].copy_from_slice(&(data_length as u16).to_le_bytes());
        buffer[8..10].copy_from_slice(&0u16.to_le_bytes());
        buffer[10..12].copy_from_slice(&(substitute_bytes as u16).to_le_bytes());
        buffer[12..14].copy_from_slice(&((substitute_bytes + 2) as u16).to_le_bytes());
        buffer[14..16].copy_from_slice(&(print_bytes as u16).to_le_bytes());
        let mut offset = 16;
        for unit in substitute
            .iter()
            .chain(std::iter::once(&0))
            .chain(print.iter())
            .chain(std::iter::once(&0))
        {
            buffer[offset..offset + 2].copy_from_slice(&unit.to_le_bytes());
            offset += 2;
        }
        // SAFETY: the UTF-16 path and reparse buffer remain live for each synchronous Win32 call;
        // the handle is checked and closed exactly once.
        let handle = unsafe {
            CreateFileW(
                junction_wide.as_ptr(),
                GENERIC_WRITE,
                FILE_SHARE_ALL,
                std::ptr::null(),
                OPEN_EXISTING,
                FILE_FLAG_OPEN_REPARSE_POINT | FILE_FLAG_BACKUP_SEMANTICS,
                0,
            )
        };
        if handle == INVALID_HANDLE_VALUE {
            return Err(io::Error::last_os_error().into());
        }
        let mut returned = 0;
        // SAFETY: `handle` is a valid reparse-point handle and `buffer` describes a mount-point
        // reparse buffer of exactly the supplied length. No output buffer or OVERLAPPED is used.
        let success = unsafe {
            DeviceIoControl(
                handle,
                FSCTL_SET_REPARSE_POINT,
                buffer.as_ptr().cast(),
                buffer.len() as u32,
                std::ptr::null_mut(),
                0,
                &mut returned,
                std::ptr::null_mut(),
            )
        };
        let call_error = (success == 0).then(io::Error::last_os_error);
        // SAFETY: the handle was returned by CreateFileW and has not been closed yet.
        unsafe { CloseHandle(handle) };
        if let Some(error) = call_error {
            return Err(error.into());
        }
        Ok(())
    })();
    if result.is_err() && created {
        let _ = fs::remove_dir(junction);
    }
    result
}

#[cfg(windows)]
fn windows_user_install_root() -> Result<PathBuf, DynError> {
    Ok(
        PathBuf::from(std::env::var_os("LOCALAPPDATA").ok_or("LOCALAPPDATA is not set")?)
            .join("Programs")
            .join(APP_NAME),
    )
}

#[cfg(windows)]
fn windows_config_state_root() -> Result<PathBuf, DynError> {
    Ok(
        PathBuf::from(std::env::var_os("LOCALAPPDATA").ok_or("LOCALAPPDATA is not set")?)
            .join(APP_NAME),
    )
}

#[cfg(windows)]
fn windows_backup_root() -> Result<PathBuf, DynError> {
    Ok(
        PathBuf::from(std::env::var_os("LOCALAPPDATA").ok_or("LOCALAPPDATA is not set")?)
            .join("honk300-backups"),
    )
}

#[cfg(windows)]
fn windows_media_root() -> Result<PathBuf, DynError> {
    let local_app_data =
        PathBuf::from(std::env::var_os("LOCALAPPDATA").ok_or("LOCALAPPDATA is not set")?);
    Ok(windows_media_root_from(&local_app_data))
}

#[cfg(windows)]
fn windows_receipt_path() -> Result<PathBuf, DynError> {
    Ok(windows_config_state_root()?.join("install-receipt.json"))
}

#[cfg(any(test, target_os = "linux", target_os = "macos"))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AliasState<'a> {
    Missing,
    Symlink(&'a Path),
    Other,
}

#[cfg(any(test, target_os = "linux", target_os = "macos"))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AliasInstallDecision {
    Create,
    Keep,
    ReplaceOwned,
    PreserveForeign,
}

#[cfg(any(test, target_os = "linux", target_os = "macos"))]
fn alias_install_decision(
    state: AliasState<'_>,
    desired: &Path,
    owned_targets: &[&Path],
) -> AliasInstallDecision {
    match state {
        AliasState::Missing => AliasInstallDecision::Create,
        AliasState::Symlink(target) if paths_match(target, desired) => AliasInstallDecision::Keep,
        AliasState::Symlink(target)
            if owned_targets.iter().any(|owned| paths_match(target, owned)) =>
        {
            AliasInstallDecision::ReplaceOwned
        }
        AliasState::Symlink(_) | AliasState::Other => AliasInstallDecision::PreserveForeign,
    }
}

fn remove_path_no_follow(path: &Path) -> io::Result<()> {
    let metadata = match fs::symlink_metadata(path) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(()),
        Err(error) => return Err(error),
    };
    if metadata.is_dir() && !metadata.file_type().is_symlink() {
        fs::remove_dir_all(path)
    } else {
        fs::remove_file(path)
    }
}

#[cfg(any(test, windows))]
#[derive(Debug, Clone)]
struct WindowsUninstallIdentity {
    key_name: String,
    display_name: String,
    publisher: String,
    install_location: PathBuf,
    uninstall_command: String,
    windows_installer: bool,
}

#[cfg(any(test, windows))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum WindowsUninstallHive {
    CurrentUser,
    LocalMachine,
}

#[cfg(any(test, windows))]
fn windows_uninstall_hive_order(source: InstallSource) -> &'static [WindowsUninstallHive] {
    match source {
        InstallSource::MsiGlobal | InstallSource::ExeGlobal => {
            &[WindowsUninstallHive::LocalMachine]
        }
        InstallSource::MsiCorporate | InstallSource::ExeCorporate => &[
            WindowsUninstallHive::CurrentUser,
            WindowsUninstallHive::LocalMachine,
        ],
        _ => &[],
    }
}

#[cfg(any(test, windows))]
#[derive(Debug, Clone, PartialEq, Eq)]
enum WindowsManagedUninstall {
    Msi {
        product_code: String,
        elevated: bool,
    },
    Exe {
        uninstaller: PathBuf,
        elevated: bool,
    },
}

#[cfg(any(test, windows))]
impl WindowsManagedUninstall {
    #[cfg(windows)]
    fn requires_elevation(&self) -> bool {
        match self {
            Self::Msi { elevated, .. } | Self::Exe { elevated, .. } => *elevated,
        }
    }
}

#[cfg(any(test, windows))]
#[derive(Debug, Clone, PartialEq, Eq)]
struct WindowsPostExitInvocation {
    args: Vec<String>,
    script: String,
}

#[cfg(windows)]
fn remove_windows_install_source_marker() -> io::Result<()> {
    use winreg::enums::HKEY_CURRENT_USER;
    use winreg::RegKey;

    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    if let Ok(key) = hkcu.open_subkey_with_flags(
        "Software\\Honk300",
        winreg::enums::KEY_SET_VALUE | winreg::enums::KEY_QUERY_VALUE,
    ) {
        let _ = key.delete_value("InstallSource");
    }
    Ok(())
}

#[cfg(windows)]
fn add_windows_user_path(bin_dir: &Path) -> io::Result<()> {
    use winreg::enums::HKEY_CURRENT_USER;
    use winreg::RegKey;

    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let (env, _) = hkcu.create_subkey("Environment")?;
    let current: String = env.get_value("Path").unwrap_or_default();
    let bin = bin_dir.to_string_lossy();
    if current
        .split(';')
        .any(|part| part.trim().eq_ignore_ascii_case(&bin))
    {
        return Ok(());
    }
    let updated = if current.trim().is_empty() {
        bin.to_string()
    } else {
        format!("{};{}", current.trim_end_matches(';'), bin)
    };
    env.set_value("Path", &updated)
}

#[cfg(windows)]
fn remove_windows_user_path(bin_dir: &Path) -> io::Result<()> {
    use winreg::enums::HKEY_CURRENT_USER;
    use winreg::RegKey;

    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let (env, _) = hkcu.create_subkey("Environment")?;
    let current: String = env.get_value("Path").unwrap_or_default();
    let bin = bin_dir.to_string_lossy();
    let parts: Vec<_> = current
        .split(';')
        .filter(|part| !part.trim().eq_ignore_ascii_case(&bin))
        .collect();
    env.set_value("Path", &parts.join(";"))
}

#[cfg(windows)]
fn create_windows_start_menu_shortcut(exe: &Path) -> io::Result<()> {
    use std::os::windows::process::CommandExt;

    const CREATE_NO_WINDOW: u32 = 0x0800_0000;
    let shortcut = windows_start_menu_shortcut_path()?;
    let working_dir = exe.parent().unwrap_or_else(|| Path::new(""));
    let script = format!(
        "$w = New-Object -ComObject WScript.Shell; $s = $w.CreateShortcut('{}'); $s.TargetPath = '{}'; $s.Arguments = ''; $s.WorkingDirectory = '{}'; $s.WindowStyle = 7; $s.Save()",
        ps_quote(&shortcut.to_string_lossy()),
        ps_quote(&exe.to_string_lossy()),
        ps_quote(&working_dir.to_string_lossy())
    );
    let status = std::process::Command::new(system_windows_powershell_path()?)
        .args([
            "-NoProfile",
            "-ExecutionPolicy",
            "Bypass",
            "-Command",
            &script,
        ])
        .creation_flags(CREATE_NO_WINDOW)
        .status()?;
    if status.success() {
        Ok(())
    } else {
        Err(io::Error::other(
            "PowerShell failed to create Start Menu shortcut",
        ))
    }
}

#[cfg(windows)]
fn remove_windows_start_menu_shortcut() -> io::Result<()> {
    remove_file_if_exists(&windows_start_menu_shortcut_path()?)
}

#[cfg(windows)]
fn windows_start_menu_shortcut_path() -> io::Result<PathBuf> {
    let appdata = std::env::var_os("APPDATA")
        .map(PathBuf::from)
        .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "APPDATA is not set"))?;
    Ok(appdata
        .join("Microsoft")
        .join("Windows")
        .join("Start Menu")
        .join("Programs")
        .join(format!("{DISPLAY_NAME}.lnk")))
}

#[cfg(windows)]
fn ps_quote(value: &str) -> String {
    value.replace('\'', "''")
}

#[cfg(windows)]
pub(crate) fn system_windows_powershell_path() -> io::Result<PathBuf> {
    let powershell = windows_system_directory()?
        .join("WindowsPowerShell")
        .join("v1.0")
        .join("powershell.exe");
    validate_windows_system_executable(powershell, "Windows PowerShell")
}

#[cfg(windows)]
pub(crate) fn system_windows_msiexec_path() -> io::Result<PathBuf> {
    let msiexec = windows_system_directory()?.join("msiexec.exe");
    validate_windows_system_executable(msiexec, "Windows Installer")
}

#[cfg(windows)]
fn windows_system_directory() -> io::Result<PathBuf> {
    use std::ffi::OsString;
    use std::os::windows::ffi::OsStringExt;
    use windows::Win32::System::SystemInformation::GetSystemDirectoryW;

    // SAFETY: GetSystemDirectoryW writes at most the supplied slice length and does not retain
    // the buffer. The first call requests the required length; the second owns a suitably sized
    // UTF-16 buffer for the duration of the call.
    let required = unsafe { GetSystemDirectoryW(None) };
    if required == 0 {
        return Err(io::Error::last_os_error());
    }
    let mut buffer = vec![0_u16; required as usize + 1];
    let written = unsafe { GetSystemDirectoryW(Some(&mut buffer)) };
    if written == 0 {
        return Err(io::Error::last_os_error());
    }
    if written as usize >= buffer.len() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "Windows system directory changed while it was queried",
        ));
    }
    Ok(PathBuf::from(OsString::from_wide(
        &buffer[..written as usize],
    )))
}

#[cfg(windows)]
fn validate_windows_system_executable(path: PathBuf, label: &str) -> io::Result<PathBuf> {
    let metadata = fs::symlink_metadata(&path)?;
    if !metadata.is_file() || metadata.file_type().is_symlink() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!(
                "{label} is not a regular system executable: {}",
                path.display()
            ),
        ));
    }
    Ok(path)
}

#[cfg(any(test, windows))]
fn validate_windows_uninstall_identity(
    source: InstallSource,
    current_exe: &Path,
    identity: &WindowsUninstallIdentity,
) -> Option<WindowsManagedUninstall> {
    let (expected_name, elevated) = match source {
        InstallSource::MsiGlobal | InstallSource::ExeGlobal => ("honk300", true),
        InstallSource::MsiCorporate | InstallSource::ExeCorporate => {
            ("honk300 (Corporate Edition)", false)
        }
        _ => return None,
    };
    if !identity.display_name.eq_ignore_ascii_case(expected_name)
        || identity.publisher != "Emmett S"
        || identity.install_location.as_os_str().is_empty()
        || !path_is_within(current_exe, &identity.install_location)
    {
        return None;
    }

    match source {
        InstallSource::MsiGlobal | InstallSource::MsiCorporate => {
            if !identity.windows_installer || !is_windows_product_code(&identity.key_name) {
                return None;
            }
            let command = identity.uninstall_command.to_ascii_lowercase();
            if !command.contains("msiexec")
                || !command.contains(&identity.key_name.to_ascii_lowercase())
            {
                return None;
            }
            Some(WindowsManagedUninstall::Msi {
                product_code: identity.key_name.clone(),
                elevated,
            })
        }
        InstallSource::ExeGlobal | InstallSource::ExeCorporate => {
            let expected_key = match source {
                InstallSource::ExeGlobal => "{5A94FBD0-DA02-4F63-9363-7D9CE0E280F5}_is1",
                InstallSource::ExeCorporate => "{A072F01B-0AE8-4ED9-B67F-845ADF7831F9}_is1",
                _ => unreachable!(),
            };
            if identity.windows_installer || !identity.key_name.eq_ignore_ascii_case(expected_key) {
                return None;
            }
            let uninstaller = command_executable_path(&identity.uninstall_command)?;
            let file_name = uninstaller.file_name()?.to_str()?.to_ascii_lowercase();
            if !path_is_within(&uninstaller, &identity.install_location)
                || !file_name.starts_with("unins")
                || !file_name.ends_with(".exe")
            {
                return None;
            }
            Some(WindowsManagedUninstall::Exe {
                uninstaller,
                elevated,
            })
        }
        _ => None,
    }
}

#[cfg(any(test, windows))]
fn is_windows_product_code(value: &str) -> bool {
    let bytes = value.as_bytes();
    if bytes.len() != 38 || bytes[0] != b'{' || bytes[37] != b'}' {
        return false;
    }
    bytes[1..37].iter().enumerate().all(|(index, byte)| {
        matches!(index, 8 | 13 | 18 | 23) && *byte == b'-'
            || !matches!(index, 8 | 13 | 18 | 23) && byte.is_ascii_hexdigit()
    })
}

#[cfg(any(test, windows))]
fn command_executable_path(command: &str) -> Option<PathBuf> {
    let command = command.trim();
    if let Some(rest) = command.strip_prefix('"') {
        let end = rest.find('"')?;
        return Some(PathBuf::from(&rest[..end]));
    }
    command.split_whitespace().next().map(PathBuf::from)
}

#[cfg(any(test, windows))]
const WINDOWS_RESTART_MANAGER_PROBE: &str = r#"
if (-not ('Honk300RestartManagerProbe' -as [type])) {
  Add-Type -TypeDefinition @'
using System;
using System.ComponentModel;
using System.IO;
using System.Runtime.InteropServices;
using System.Text;
public static class Honk300RestartManagerProbe {
  private const int ErrorMoreData = 234;
  [DllImport("rstrtmgr.dll", CharSet=CharSet.Unicode)] private static extern int RmStartSession(out uint handle, int flags, StringBuilder key);
  [DllImport("rstrtmgr.dll", CharSet=CharSet.Unicode)] private static extern int RmRegisterResources(uint handle, uint fileCount, string[] files, uint applicationCount, IntPtr applications, uint serviceCount, string[] services);
  [DllImport("rstrtmgr.dll")] private static extern int RmGetList(uint handle, out uint needed, ref uint count, IntPtr info, ref uint reasons);
  [DllImport("rstrtmgr.dll")] private static extern int RmEndSession(uint handle);
  public static void AssertUnlocked(string path) {
    if (!File.Exists(path)) return;
    uint handle; var key=new StringBuilder(64); int result=RmStartSession(out handle,0,key);
    if (result != 0) throw new Win32Exception(result,"Restart Manager session failed");
    try {
      result=RmRegisterResources(handle,1,new[] { path },0,IntPtr.Zero,0,null);
      if (result != 0) throw new Win32Exception(result,"Restart Manager registration failed");
      uint needed=0,count=0,reasons=0; result=RmGetList(handle,out needed,ref count,IntPtr.Zero,ref reasons);
      if (result != 0 && result != ErrorMoreData) throw new Win32Exception(result,"Restart Manager lock query failed");
      if (needed != 0) throw new InvalidOperationException("another Windows session is using " + path);
    } finally { RmEndSession(handle); }
  }
}
'@
}
"#;

#[cfg(any(test, windows))]
fn windows_managed_uninstall_invocation(
    plan: &WindowsManagedUninstall,
    installed_executable: &Path,
    system_msiexec: &Path,
) -> WindowsPostExitInvocation {
    let (file, arguments, elevated) = match plan {
        WindowsManagedUninstall::Msi {
            product_code,
            elevated,
        } => (
            system_msiexec.to_string_lossy().into_owned(),
            format!(
                "@('/x','{}','/passive','/norestart')",
                powershell_literal(product_code)
            ),
            *elevated,
        ),
        WindowsManagedUninstall::Exe {
            uninstaller,
            elevated,
        } => (
            uninstaller.to_string_lossy().into_owned(),
            "@('/VERYSILENT','/SUPPRESSMSGBOXES','/NORESTART')".to_owned(),
            *elevated,
        ),
    };
    let elevation = if elevated { " -Verb RunAs" } else { "" };
    let installed = powershell_literal(&installed_executable.to_string_lossy());
    let restart_manager_probe = WINDOWS_RESTART_MANAGER_PROBE;
    let script = format!(
        "$ErrorActionPreference='Stop'; {restart_manager_probe}; $installedBin=[IO.Path]::GetDirectoryName('{installed}'); foreach ($candidate in @((Join-Path $installedBin 'honk300.exe'),(Join-Path $installedBin 'honk.exe'),(Join-Path $installedBin 'goose.exe'))) {{ [Honk300RestartManagerProbe]::AssertUnlocked($candidate) }}; $process = Start-Process -FilePath '{}' -ArgumentList {arguments} -WindowStyle Hidden -Wait -PassThru{elevation}; if ($process.ExitCode -notin @(0,1605)) {{ exit $process.ExitCode }}",
        powershell_literal(&file)
    );
    WindowsPostExitInvocation {
        args: vec![
            "-NoProfile".to_owned(),
            "-WindowStyle".to_owned(),
            "Hidden".to_owned(),
            "-ExecutionPolicy".to_owned(),
            "Bypass".to_owned(),
            "-Command".to_owned(),
            script.clone(),
        ],
        script,
    }
}

#[cfg(any(test, windows))]
fn windows_wait_for_parent_script(parent_pid: u32) -> String {
    format!(
        "$ErrorActionPreference='Stop'; $process = Get-Process -Id {parent_pid} -ErrorAction SilentlyContinue; if ($null -ne $process) {{ $process.WaitForExit() }}; exit 0"
    )
}

#[cfg(any(test, windows))]
fn powershell_literal(value: &str) -> String {
    value.replace('\'', "''")
}

#[cfg(windows)]
const WINDOWS_DEFERRED_UNINSTALL_READY: &str = "HONK300_INTERNAL_WINDOWS_UNINSTALL_READY";

#[cfg(windows)]
struct DeferredUninstallChild {
    child: Option<std::process::Child>,
}

#[cfg(windows)]
impl DeferredUninstallChild {
    fn new(child: std::process::Child) -> Self {
        Self { child: Some(child) }
    }

    fn try_wait(&mut self) -> io::Result<Option<std::process::ExitStatus>> {
        self.child
            .as_mut()
            .expect("deferred uninstall helper is armed")
            .try_wait()
    }

    fn disarm(mut self) {
        let _ = self.child.take();
    }
}

#[cfg(windows)]
impl Drop for DeferredUninstallChild {
    fn drop(&mut self) {
        let Some(child) = self.child.as_mut() else {
            return;
        };
        if !matches!(child.try_wait(), Ok(Some(_))) {
            let _ = child.kill();
        }
        let _ = child.wait();
    }
}

#[cfg(windows)]
fn schedule_windows_deferred_uninstall(
    source: InstallSource,
    purge: bool,
    current_exe: &Path,
) -> Result<(), DynError> {
    use std::os::windows::process::CommandExt;
    use std::process::Stdio;
    use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

    const CREATE_NO_WINDOW: u32 = 0x0800_0000;
    let metadata = fs::symlink_metadata(current_exe)?;
    if !metadata.is_file() || metadata.file_type().is_symlink() {
        return Err("honk300 uninstall: the running executable is not a regular owned file".into());
    }
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    let helper_root =
        std::env::temp_dir().join(format!("honk300-uninstall-{}-{nonce}", std::process::id()));
    fs::create_dir(&helper_root)?;
    let helper = helper_root.join("honk300-uninstall-helper.exe");
    let ready = helper_root.join("ready");
    let log = helper_root.join("uninstall.log");
    let scheduled = (|| -> Result<(), DynError> {
        fs::copy(current_exe, &helper)?;
        let stdout = fs::OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(&log)?;
        let stderr = stdout.try_clone()?;
        let child = std::process::Command::new(&helper)
            .env("HONK300_INTERNAL_WINDOWS_UNINSTALL", "1")
            .env(
                "HONK300_INTERNAL_WINDOWS_UNINSTALL_SOURCE",
                source.marker_value(),
            )
            .env(
                "HONK300_INTERNAL_WINDOWS_UNINSTALL_PURGE",
                if purge { "1" } else { "0" },
            )
            .env(
                "HONK300_INTERNAL_WINDOWS_UNINSTALL_PARENT_PID",
                std::process::id().to_string(),
            )
            .env(
                "HONK300_INTERNAL_WINDOWS_UNINSTALL_ORIGINAL_EXE",
                current_exe,
            )
            .env("HONK300_INTERNAL_WINDOWS_UNINSTALL_ROOT", &helper_root)
            .env("HONK300_INTERNAL_WINDOWS_UNINSTALL_READY_PATH", &ready)
            .current_dir(&helper_root)
            .stdin(Stdio::null())
            .stdout(Stdio::from(stdout))
            .stderr(Stdio::from(stderr))
            .creation_flags(CREATE_NO_WINDOW)
            .spawn()?;
        let mut child = DeferredUninstallChild::new(child);
        let deadline = Instant::now() + Duration::from_secs(35);
        loop {
            match fs::read_to_string(&ready) {
                Ok(value) if value == WINDOWS_DEFERRED_UNINSTALL_READY => {
                    if child.try_wait()?.is_some() {
                        return Err("honk300 uninstall: deferred helper exited after its ownership handshake".into());
                    }
                    child.disarm();
                    break;
                }
                Ok(value) if !value.is_empty() => {
                    return Err(
                        "honk300 uninstall: deferred helper wrote an invalid ownership handshake"
                            .into(),
                    );
                }
                Ok(_) => {}
                Err(error) if error.kind() == io::ErrorKind::NotFound => {}
                Err(error) => return Err(error.into()),
            }
            if let Some(status) = child.try_wait()? {
                let detail = fs::read_to_string(&log).unwrap_or_default();
                return Err(format!(
                    "honk300 uninstall: deferred helper could not acquire lifecycle ownership (exit {}): {}",
                    status.code().unwrap_or(-1),
                    detail.trim()
                )
                .into());
            }
            if Instant::now() >= deadline {
                return Err("honk300 uninstall: timed out acquiring exclusive lifecycle ownership; no managed files were touched".into());
            }
            std::thread::sleep(Duration::from_millis(50));
        }
        Ok(())
    })();
    if let Err(error) = scheduled {
        let _ = fs::remove_dir_all(&helper_root);
        return Err(error);
    }
    println!(
        "honk300: exclusive Windows uninstall handoff acquired; removal begins after this process exits."
    );
    Ok(())
}

#[cfg(windows)]
pub(crate) fn run_windows_deferred_uninstall() -> Result<(), DynError> {
    use std::io::Write as _;

    let source =
        InstallSource::from_marker(&std::env::var("HONK300_INTERNAL_WINDOWS_UNINSTALL_SOURCE")?);
    let purge = match std::env::var("HONK300_INTERNAL_WINDOWS_UNINSTALL_PURGE")?.as_str() {
        "0" => false,
        "1" => true,
        _ => return Err("invalid deferred Windows uninstall purge marker".into()),
    };
    let parent_pid =
        std::env::var("HONK300_INTERNAL_WINDOWS_UNINSTALL_PARENT_PID")?.parse::<u32>()?;
    let original_exe = PathBuf::from(
        std::env::var_os("HONK300_INTERNAL_WINDOWS_UNINSTALL_ORIGINAL_EXE")
            .ok_or("missing deferred Windows uninstall original executable")?,
    );
    let helper_root = PathBuf::from(
        std::env::var_os("HONK300_INTERNAL_WINDOWS_UNINSTALL_ROOT")
            .ok_or("missing deferred Windows uninstall helper root")?,
    );
    let ready = PathBuf::from(
        std::env::var_os("HONK300_INTERNAL_WINDOWS_UNINSTALL_READY_PATH")
            .ok_or("missing deferred Windows uninstall ready path")?,
    );
    let current_exe = std::env::current_exe()?;
    if current_exe.parent() != Some(helper_root.as_path())
        || ready.parent() != Some(helper_root.as_path())
        || !current_exe
            .file_name()
            .is_some_and(|name| name.eq_ignore_ascii_case("honk300-uninstall-helper.exe"))
    {
        return Err("invalid deferred Windows uninstall helper identity".into());
    }
    let _lease = LifecycleLease::acquire()?;
    fs::write(&ready, WINDOWS_DEFERRED_UNINSTALL_READY)?;
    std::io::stdout().flush()?;
    wait_for_windows_parent_exit(parent_pid)?;

    let result = if matches!(
        source,
        InstallSource::MsiGlobal
            | InstallSource::MsiCorporate
            | InstallSource::ExeGlobal
            | InstallSource::ExeCorporate
    ) {
        uninstall_windows_managed_under_lease(source, purge, &original_exe)
    } else if source == InstallSource::ManualLocal {
        uninstall_windows_manual_under_lease(purge)
    } else {
        Err("deferred Windows uninstall source is not managed by honk300".into())
    };
    if result.is_ok() {
        schedule_windows_helper_cleanup(&helper_root)?;
    }
    result
}

#[cfg(windows)]
fn wait_for_windows_parent_exit(parent_pid: u32) -> io::Result<()> {
    use std::os::windows::process::CommandExt;

    const CREATE_NO_WINDOW: u32 = 0x0800_0000;
    let script = windows_wait_for_parent_script(parent_pid);
    let status = std::process::Command::new(system_windows_powershell_path()?)
        .args([
            "-NoProfile",
            "-ExecutionPolicy",
            "Bypass",
            "-Command",
            &script,
        ])
        .creation_flags(CREATE_NO_WINDOW)
        .status()?;
    if status.success() {
        Ok(())
    } else {
        Err(io::Error::other(
            "Windows deferred uninstall could not wait for the invoking process to exit",
        ))
    }
}

#[cfg(windows)]
fn schedule_windows_helper_cleanup(helper_root: &Path) -> io::Result<()> {
    use std::os::windows::process::CommandExt;
    use std::process::Stdio;

    const CREATE_NO_WINDOW: u32 = 0x0800_0000;
    let root = powershell_literal(&helper_root.to_string_lossy());
    let script = format!(
        "$process = Get-Process -Id {} -ErrorAction SilentlyContinue; if ($null -ne $process) {{ $process.WaitForExit() }}; $item = Get-Item -LiteralPath '{root}' -Force -ErrorAction SilentlyContinue; if ($null -ne $item -and $item.PSIsContainer -and (($item.Attributes -band [IO.FileAttributes]::ReparsePoint) -eq 0)) {{ Remove-Item -LiteralPath '{root}' -Recurse -Force }}",
        std::process::id()
    );
    std::process::Command::new(system_windows_powershell_path()?)
        .args([
            "-NoProfile",
            "-WindowStyle",
            "Hidden",
            "-ExecutionPolicy",
            "Bypass",
            "-Command",
            &script,
        ])
        .creation_flags(CREATE_NO_WINDOW)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()?;
    Ok(())
}

#[cfg(windows)]
fn uninstall_windows_manual_under_lease(purge: bool) -> Result<(), DynError> {
    let root = windows_user_install_root()?;
    let bin_dir = root.join("bin");
    ensure_owned_install_root(&root, &[InstallSource::ManualLocal])?;
    let media = windows_media_root()?;
    ensure_external_media_root(&media)?;
    migrate_legacy_user_media(&bin_dir.join("Assets"), &media, LegacyMigrationMode::Move)?;
    let backup = if purge {
        backup_user_content(&media, &windows_backup_root()?)?
    } else {
        None
    };

    set_windows_autostart(None)?;
    remove_windows_start_menu_shortcut()?;
    remove_windows_user_path(&bin_dir)?;
    remove_windows_install_source_marker()?;
    remove_dir_if_exists(&root)?;

    if purge {
        purge_config_state_preserving_foreign_receipt(&windows_config_state_root()?, &root)?;
        report_backup(backup);
    } else {
        remove_owned_receipt(&windows_receipt_path()?, &root)?;
        report_preserved(media_has_user_content(&media)?.then_some(media));
    }

    println!("honk300: uninstalled.");
    Ok(())
}

#[cfg(windows)]
fn uninstall_windows_managed_under_lease(
    source: InstallSource,
    purge: bool,
    original_exe: &Path,
) -> Result<(), DynError> {
    use std::os::windows::process::CommandExt;

    const CREATE_NO_WINDOW: u32 = 0x0800_0000;
    let Some((plan, install_root)) = find_windows_managed_uninstall(source, original_exe)? else {
        return Err("honk300 uninstall: the Windows installer identity could not be proven, so no installed files were touched. Uninstall Honk300 from Windows Installed Apps instead.".into());
    };

    let media = windows_media_root()?;
    ensure_external_media_root(&media)?;
    if let Some(bin_dir) = original_exe.parent() {
        migrate_legacy_user_media(&bin_dir.join("Assets"), &media, LegacyMigrationMode::Copy)?;
    }
    let backup = if purge {
        backup_user_content(&media, &windows_backup_root()?)?
    } else {
        None
    };
    let receipt = windows_receipt_path()?;
    let owned_receipt = receipt_is_owned(&receipt, &install_root)?;
    let system_msiexec = system_windows_msiexec_path()?;
    let invocation = windows_managed_uninstall_invocation(&plan, original_exe, &system_msiexec);
    let status = std::process::Command::new(system_windows_powershell_path()?)
        .args(&invocation.args)
        .creation_flags(CREATE_NO_WINDOW)
        .status()?;
    if !status.success() {
        return Err(format!(
            "honk300 uninstall: verified Windows installer exited with code {}",
            status.code().unwrap_or(-1)
        )
        .into());
    }
    let installed_bin = original_exe
        .parent()
        .ok_or("honk300 uninstall: installed executable has no parent directory")?;
    for name in [
        "honk300.exe",
        "honk.exe",
        "goose.exe",
        WINDOWS_APP_LAUNCHER_NAME,
        companions::SETTINGS_NAME,
    ] {
        match fs::symlink_metadata(installed_bin.join(name)) {
            Err(error) if error.kind() == io::ErrorKind::NotFound => {}
            Err(error) => return Err(error.into()),
            Ok(_) => {
                return Err(format!(
                    "honk300 uninstall: Windows reported success but {name} is still installed; refusing pending cleanup"
                )
                .into())
            }
        }
    }

    if purge {
        purge_config_state_preserving_foreign_receipt(
            &windows_config_state_root()?,
            &install_root,
        )?;
        report_backup(backup);
    } else {
        if owned_receipt {
            remove_owned_receipt(&receipt, &install_root)?;
        }
        report_preserved(media_has_user_content(&media)?.then_some(media));
    }
    println!("honk300: uninstalled through the verified Windows installer.");
    Ok(())
}

#[cfg(windows)]
fn find_windows_managed_uninstall(
    source: InstallSource,
    current_exe: &Path,
) -> io::Result<Option<(WindowsManagedUninstall, PathBuf)>> {
    use winreg::enums::{
        HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE, KEY_READ, KEY_WOW64_32KEY, KEY_WOW64_64KEY,
    };
    use winreg::RegKey;

    for hive in windows_uninstall_hive_order(source) {
        let root = RegKey::predef(match hive {
            WindowsUninstallHive::CurrentUser => HKEY_CURRENT_USER,
            WindowsUninstallHive::LocalMachine => HKEY_LOCAL_MACHINE,
        });
        for view in [KEY_WOW64_64KEY, KEY_WOW64_32KEY] {
            let uninstall = match root.open_subkey_with_flags(
                "Software\\Microsoft\\Windows\\CurrentVersion\\Uninstall",
                KEY_READ | view,
            ) {
                Ok(key) => key,
                Err(error) if error.kind() == io::ErrorKind::NotFound => continue,
                Err(error) => return Err(error),
            };
            for key_name in uninstall.enum_keys().flatten() {
                let Ok(key) = uninstall.open_subkey_with_flags(&key_name, KEY_READ) else {
                    continue;
                };
                let identity = WindowsUninstallIdentity {
                    key_name,
                    display_name: key.get_value("DisplayName").unwrap_or_default(),
                    publisher: key.get_value("Publisher").unwrap_or_default(),
                    install_location: key
                        .get_value::<String, _>("InstallLocation")
                        .map(PathBuf::from)
                        .unwrap_or_default(),
                    uninstall_command: key.get_value("UninstallString").unwrap_or_default(),
                    windows_installer: key
                        .get_value::<u32, _>("WindowsInstaller")
                        .unwrap_or_default()
                        != 0,
                };
                if let Some(plan) =
                    validate_windows_uninstall_identity(source, current_exe, &identity)
                {
                    return Ok(Some((plan, identity.install_location)));
                }
            }
        }
    }
    Ok(None)
}

#[cfg(target_os = "linux")]
fn linux_user_install_root() -> Result<PathBuf, DynError> {
    Ok(xdg_data_home()?.join(APP_NAME).join("install"))
}

#[cfg(target_os = "linux")]
fn linux_config_state_root() -> Result<PathBuf, DynError> {
    Ok(xdg_data_home()?.join(APP_NAME))
}

#[cfg(target_os = "linux")]
fn linux_media_root() -> Result<PathBuf, DynError> {
    let home = home_dir()?;
    Ok(linux_media_root_from(
        std::env::var_os("XDG_DATA_HOME").as_deref().map(Path::new),
        &home,
    ))
}

#[cfg(target_os = "linux")]
fn linux_receipt_path() -> Result<PathBuf, DynError> {
    Ok(xdg_data_home()?.join(APP_NAME).join("install-receipt.json"))
}

#[cfg(target_os = "linux")]
fn linux_backup_root() -> Result<PathBuf, DynError> {
    Ok(xdg_data_home()?.join("honk300-backups"))
}

#[cfg(target_os = "linux")]
fn linux_user_alias_dir() -> Result<PathBuf, DynError> {
    Ok(home_dir()?.join(".local").join("bin"))
}

#[cfg(target_os = "linux")]
fn linux_applications_dir() -> Result<PathBuf, DynError> {
    Ok(xdg_data_home()?.join("applications"))
}

#[cfg(target_os = "linux")]
fn linux_autostart_path() -> Result<PathBuf, DynError> {
    Ok(xdg_config_home()?.join("autostart").join("honk300.desktop"))
}

#[cfg(target_os = "linux")]
fn linux_desktop_entry(exe: &Path) -> String {
    format!(
        "[Desktop Entry]\nType=Application\nName=Honk300\nComment=Desktop goose for your screen\nExec={} start\nTerminal=false\nCategories=Utility;\nStartupNotify=false\nX-GNOME-Autostart-enabled=true\nX-Honk300-Owner={OWNERSHIP_MARKER}\n",
        desktop_exec_quote(exe)
    )
}

#[cfg(any(target_os = "linux", target_os = "macos"))]
fn install_owned_unix_alias(link: &Path, target: &Path, owned_targets: &[&Path]) -> io::Result<()> {
    match unix_alias_install_decision(link, target, owned_targets)? {
        AliasInstallDecision::Keep => Ok(()),
        AliasInstallDecision::Create => std::os::unix::fs::symlink(target, link),
        AliasInstallDecision::ReplaceOwned => {
            fs::remove_file(link)?;
            std::os::unix::fs::symlink(target, link)
        }
        AliasInstallDecision::PreserveForeign => Err(io::Error::new(
            io::ErrorKind::AlreadyExists,
            format!(
                "refusing to replace foreign command alias {}; move it aside and retry",
                link.display()
            ),
        )),
    }
}

#[cfg(any(target_os = "linux", target_os = "macos"))]
fn unix_alias_install_decision(
    link: &Path,
    target: &Path,
    owned_targets: &[&Path],
) -> io::Result<AliasInstallDecision> {
    let state_target;
    let state = match fs::symlink_metadata(link) {
        Err(error) if error.kind() == io::ErrorKind::NotFound => AliasState::Missing,
        Err(error) => return Err(error),
        Ok(metadata) if metadata.file_type().is_symlink() => {
            let raw_target = fs::read_link(link)?;
            state_target = if raw_target.is_absolute() {
                raw_target
            } else {
                link.parent()
                    .unwrap_or_else(|| Path::new("."))
                    .join(raw_target)
            };
            AliasState::Symlink(&state_target)
        }
        Ok(_) => AliasState::Other,
    };
    Ok(alias_install_decision(state, target, owned_targets))
}

#[cfg(any(target_os = "linux", target_os = "macos"))]
fn remove_owned_unix_alias(link: &Path, owned_targets: &[&Path]) -> io::Result<bool> {
    let metadata = match fs::symlink_metadata(link) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(false),
        Err(error) => return Err(error),
    };
    if !metadata.file_type().is_symlink() {
        return Ok(false);
    }
    let raw_target = fs::read_link(link)?;
    let target = if raw_target.is_absolute() {
        raw_target
    } else {
        link.parent()
            .unwrap_or_else(|| Path::new("."))
            .join(raw_target)
    };
    if !owned_targets
        .iter()
        .any(|owned| paths_match(&target, owned))
    {
        return Ok(false);
    }
    fs::remove_file(link)?;
    Ok(true)
}

#[cfg(target_os = "linux")]
fn make_executable(path: &Path) -> io::Result<()> {
    use std::os::unix::fs::PermissionsExt;

    let mut permissions = fs::metadata(path)?.permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(path, permissions)
}

#[cfg(target_os = "linux")]
fn xdg_data_home() -> Result<PathBuf, DynError> {
    Ok(std::env::var_os("XDG_DATA_HOME")
        .map(PathBuf::from)
        .unwrap_or(home_dir()?.join(".local").join("share")))
}

#[cfg(target_os = "linux")]
fn xdg_config_home() -> Result<PathBuf, DynError> {
    Ok(std::env::var_os("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .unwrap_or(home_dir()?.join(".config")))
}

#[cfg(any(target_os = "linux", target_os = "macos"))]
fn home_dir() -> Result<PathBuf, DynError> {
    Ok(PathBuf::from(
        std::env::var_os("HOME").ok_or("HOME is not set")?,
    ))
}

#[cfg(target_os = "linux")]
fn desktop_exec_quote(path: &Path) -> String {
    let raw = path.to_string_lossy();
    if raw
        .chars()
        .any(|c| c.is_whitespace() || c == '\'' || c == '"')
    {
        format!("'{}'", raw.replace('\'', "'\\''"))
    } else {
        raw.into_owned()
    }
}

#[cfg(any(target_os = "linux", target_os = "macos"))]
fn path_contains(dir: &Path) -> bool {
    let Some(path) = std::env::var_os("PATH") else {
        return false;
    };
    std::env::split_paths(&path).any(|part| part == dir)
}

#[cfg(any(test, target_os = "macos"))]
fn is_exact_macos_managed_executable(executable: &Path, managed_app: &Path) -> bool {
    if fs::symlink_metadata(managed_app)
        .is_ok_and(|metadata| metadata.file_type().is_symlink() || !metadata.is_dir())
    {
        return false;
    }
    paths_match(
        executable,
        &managed_app.join("Contents").join("MacOS").join("honk300"),
    )
}

#[cfg(any(test, target_os = "macos"))]
#[derive(Debug, Clone, PartialEq, Eq)]
enum MacosInstallDisposition {
    ConfigureExisting,
    CopyBundle(PathBuf),
}

#[cfg(any(test, target_os = "macos"))]
fn macos_bundle_replacement_is_owned(destination_exists: bool, receipt_owned: bool) -> bool {
    !destination_exists || receipt_owned
}

#[cfg(any(test, target_os = "macos"))]
#[derive(Debug, Clone, PartialEq, Eq)]
struct MacosBundleMetadata {
    version: String,
    tag: String,
    commit: String,
}

#[cfg(any(test, target_os = "macos"))]
fn macos_install_receipt(
    metadata: &MacosBundleMetadata,
    install_root: &Path,
    autostart: bool,
    payload_hash: &str,
    payload_size: u64,
) -> serde_json::Value {
    let home = install_root
        .parent()
        .and_then(Path::parent)
        .unwrap_or_else(|| Path::new(""));
    let alias = |name: &str| {
        home.join(".local")
            .join("bin")
            .join(name)
            .to_string_lossy()
            .into_owned()
    };
    serde_json::json!({
        "schema": INSTALL_RECEIPT_V2,
        "version": metadata.version,
        "tag": metadata.tag,
        "commit": metadata.commit,
        "channel": "mac-app",
        "origin": "mac-app",
        "installer_family": "dmg",
        "edition": "global",
        "scope": "user",
        "release_track": "stable",
        "layout": "mac-app",
        "target": "universal2-apple-darwin",
        "artifact": {
            "name": "Honk300.app/Contents/MacOS/honk300",
            "sha256": payload_hash,
            "size": payload_size
        },
        "install_root": install_root.to_string_lossy(),
        "owned_root": install_root.to_string_lossy(),
        "active_release": install_root.to_string_lossy(),
        "aliases": [alias("honk300"), alias("honk"), alias("goose")],
        "autostart": { "enabled": autostart, "owner": "honk300-install" }
    })
}

#[cfg(any(test, target_os = "macos"))]
fn macos_install_disposition(
    executable: &Path,
    managed_app: &Path,
) -> io::Result<MacosInstallDisposition> {
    if is_exact_macos_managed_executable(executable, managed_app) {
        return Ok(MacosInstallDisposition::ConfigureExisting);
    }
    let source_app = executable
        .ancestors()
        .find(|path| path.extension().and_then(|ext| ext.to_str()) == Some("app"))
        .map(Path::to_path_buf)
        .ok_or_else(|| {
            io::Error::other(format!(
                "honk300 install: {} is not inside a macOS app bundle",
                executable.display()
            ))
        })?;
    let expected = source_app.join("Contents").join("MacOS").join("honk300");
    if !paths_match(executable, &expected) {
        return Err(io::Error::other(format!(
            "honk300 install: source executable is not the bundle's sealed honk300 binary: {}",
            executable.display()
        )));
    }
    Ok(MacosInstallDisposition::CopyBundle(source_app))
}

#[cfg(test)]
fn macos_external_mutation_paths(home: &Path) -> Vec<PathBuf> {
    let mut paths: Vec<_> = COMMAND_NAMES
        .iter()
        .map(|name| home.join(".local").join("bin").join(name))
        .collect();
    paths.push(
        home.join("Library")
            .join("LaunchAgents")
            .join("dev.emmetts.honk300.plist"),
    );
    paths.push(macos_media_root_from(home));
    paths.push(
        home.join("Library")
            .join("Application Support")
            .join(APP_NAME)
            .join("install-receipt.json"),
    );
    paths
}

#[cfg(target_os = "macos")]
fn plist_value(app: &Path, key: &str) -> io::Result<String> {
    let output = std::process::Command::new("/usr/libexec/PlistBuddy")
        .arg("-c")
        .arg(format!("Print :{key}"))
        .arg(app.join("Contents").join("Info.plist"))
        .output()?;
    if !output.status.success() {
        return Err(io::Error::other(format!(
            "could not read {key} from {}: {}",
            app.display(),
            String::from_utf8_lossy(&output.stderr).trim()
        )));
    }
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_owned())
}

#[cfg(target_os = "macos")]
fn checked_macos_command(program: &str, args: &[&Path]) -> io::Result<()> {
    let output = std::process::Command::new(program).args(args).output()?;
    if output.status.success() {
        Ok(())
    } else {
        Err(io::Error::other(format!(
            "{} failed: {}",
            program,
            String::from_utf8_lossy(&output.stderr).trim()
        )))
    }
}

#[cfg(target_os = "macos")]
fn validate_macos_bundle(app: &Path) -> io::Result<MacosBundleMetadata> {
    validate_real_directory(app)?;
    if plist_value(app, "CFBundleIdentifier")? != "dev.emmetts.honk300" {
        return Err(io::Error::other(format!(
            "refusing app bundle with an unexpected identifier: {}",
            app.display()
        )));
    }
    let version = plist_value(app, "CFBundleShortVersionString")?;
    let tag = plist_value(app, "Honk300ReleaseTag")?;
    let commit = plist_value(app, "Honk300ReleaseCommit")?.to_ascii_lowercase();
    if tag != format!("v{version}") {
        return Err(io::Error::other(format!(
            "bundle release tag {tag} does not match version {version}"
        )));
    }
    if commit.len() != 40 || !commit.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Err(io::Error::other(
            "bundle release commit is not a full hexadecimal SHA",
        ));
    }
    let executable = app.join("Contents").join("MacOS").join("honk300");
    let metadata = fs::symlink_metadata(&executable)?;
    if !metadata.is_file() || metadata.file_type().is_symlink() {
        return Err(io::Error::other(format!(
            "bundle executable is not a regular sealed file: {}",
            executable.display()
        )));
    }
    let verify = std::process::Command::new("/usr/bin/codesign")
        .args(["--verify", "--strict", "--verbose=2"])
        .arg(app)
        .output()?;
    if !verify.status.success() {
        return Err(io::Error::other(format!(
            "bundle signature validation failed: {}",
            String::from_utf8_lossy(&verify.stderr).trim()
        )));
    }
    let lipo = std::process::Command::new("/usr/bin/lipo")
        .arg(&executable)
        .args(["-verify_arch", "x86_64", "arm64"])
        .output()?;
    if !lipo.status.success() {
        return Err(io::Error::other(format!(
            "bundle is not universal Apple Silicon/Intel code: {}",
            String::from_utf8_lossy(&lipo.stderr).trim()
        )));
    }
    Ok(MacosBundleMetadata {
        version,
        tag,
        commit,
    })
}

#[cfg(target_os = "macos")]
struct MacosBundleSwap {
    destination: PathBuf,
    previous: Option<PathBuf>,
    active: bool,
}

#[cfg(target_os = "macos")]
impl MacosBundleSwap {
    fn begin(source: &Path, destination: &Path) -> io::Result<(Self, MacosBundleMetadata)> {
        let source_metadata = validate_macos_bundle(source)?;
        let parent = destination
            .parent()
            .ok_or_else(|| io::Error::other("managed app path has no parent"))?;
        ensure_real_directory(parent)?;
        let stage = parent.join(format!(".Honk300.app.stage.{}", std::process::id()));
        let previous = parent.join(format!(".Honk300.app.previous.{}", std::process::id()));
        for path in [&stage, &previous] {
            if fs::symlink_metadata(path).is_ok() {
                return Err(io::Error::new(
                    io::ErrorKind::AlreadyExists,
                    format!(
                        "stale lifecycle transaction path exists: {}",
                        path.display()
                    ),
                ));
            }
        }
        if let Ok(metadata) = fs::symlink_metadata(destination) {
            if !metadata.is_dir() || metadata.file_type().is_symlink() {
                return Err(io::Error::other(format!(
                    "refusing to replace non-directory or symlinked app {}",
                    destination.display()
                )));
            }
            if plist_value(destination, "CFBundleIdentifier")? != "dev.emmetts.honk300" {
                return Err(io::Error::other(format!(
                    "refusing to replace foreign app bundle {}",
                    destination.display()
                )));
            }
        }

        if let Err(error) = checked_macos_command("/usr/bin/ditto", &[source, &stage]) {
            let _ = remove_path_no_follow(&stage);
            return Err(error);
        }
        let staged_metadata = match validate_macos_bundle(&stage) {
            Ok(metadata) => metadata,
            Err(error) => {
                let _ = remove_path_no_follow(&stage);
                return Err(error);
            }
        };
        if staged_metadata != source_metadata {
            let _ = remove_path_no_follow(&stage);
            return Err(io::Error::other(
                "staged app release metadata changed during bundle copy",
            ));
        }
        let previous = if destination.exists() {
            fs::rename(destination, &previous)?;
            Some(previous)
        } else {
            None
        };
        if let Err(error) = fs::rename(&stage, destination) {
            if let Some(previous) = &previous {
                let _ = fs::rename(previous, destination);
            }
            let _ = remove_path_no_follow(&stage);
            return Err(error);
        }
        Ok((
            Self {
                destination: destination.to_path_buf(),
                previous,
                active: true,
            },
            staged_metadata,
        ))
    }

    fn rollback(&mut self) -> io::Result<()> {
        if !self.active {
            return Ok(());
        }
        remove_path_no_follow(&self.destination)?;
        if let Some(previous) = &self.previous {
            fs::rename(previous, &self.destination)?;
        }
        self.active = false;
        Ok(())
    }

    fn commit(&mut self) -> io::Result<()> {
        if let Some(previous) = &self.previous {
            remove_path_no_follow(previous)?;
        }
        self.previous = None;
        self.active = false;
        Ok(())
    }
}

#[cfg(target_os = "macos")]
impl Drop for MacosBundleSwap {
    fn drop(&mut self) {
        let _ = self.rollback();
    }
}

#[cfg(target_os = "macos")]
enum MacosPathState {
    Missing,
    Symlink(PathBuf),
    Regular {
        bytes: Vec<u8>,
        permissions: fs::Permissions,
    },
}

#[cfg(target_os = "macos")]
struct MacosPathSnapshot {
    path: PathBuf,
    state: MacosPathState,
}

#[cfg(target_os = "macos")]
impl MacosPathSnapshot {
    fn capture(path: &Path) -> io::Result<Self> {
        let state = match fs::symlink_metadata(path) {
            Err(error) if error.kind() == io::ErrorKind::NotFound => MacosPathState::Missing,
            Err(error) => return Err(error),
            Ok(metadata) if metadata.file_type().is_symlink() => {
                MacosPathState::Symlink(fs::read_link(path)?)
            }
            Ok(metadata) if metadata.is_file() => MacosPathState::Regular {
                bytes: fs::read(path)?,
                permissions: metadata.permissions(),
            },
            Ok(_) => {
                return Err(io::Error::other(format!(
                    "cannot snapshot non-file integration {}",
                    path.display()
                )))
            }
        };
        Ok(Self {
            path: path.to_path_buf(),
            state,
        })
    }

    fn restore(&self) -> io::Result<()> {
        remove_path_no_follow(&self.path)?;
        match &self.state {
            MacosPathState::Missing => Ok(()),
            MacosPathState::Symlink(target) => {
                let parent = self
                    .path
                    .parent()
                    .ok_or_else(|| io::Error::other("integration symlink has no parent"))?;
                ensure_real_directory(parent)?;
                std::os::unix::fs::symlink(target, &self.path)
            }
            MacosPathState::Regular { bytes, permissions } => {
                let parent = self
                    .path
                    .parent()
                    .ok_or_else(|| io::Error::other("integration file has no parent"))?;
                ensure_real_directory(parent)?;
                fs::write(&self.path, bytes)?;
                fs::set_permissions(&self.path, permissions.clone())
            }
        }
    }
}

#[cfg(target_os = "macos")]
struct MacosIntegrationTransaction {
    snapshots: Vec<MacosPathSnapshot>,
    media_changes: MediaMigrationChanges,
    active: bool,
}

#[cfg(target_os = "macos")]
impl MacosIntegrationTransaction {
    fn capture(aliases: &[PathBuf], launch_agent: &Path, receipt: &Path) -> io::Result<Self> {
        let mut paths = aliases.to_vec();
        paths.push(launch_agent.to_path_buf());
        paths.push(receipt.to_path_buf());
        let snapshots = paths
            .iter()
            .map(|path| MacosPathSnapshot::capture(path))
            .collect::<io::Result<Vec<_>>>()?;
        Ok(Self {
            snapshots,
            media_changes: MediaMigrationChanges::default(),
            active: false,
        })
    }

    fn begin(&mut self) {
        self.active = true;
    }

    fn record_media_migration(&mut self, mut changes: MediaMigrationChanges) {
        self.media_changes
            .created_files
            .append(&mut changes.created_files);
        self.media_changes
            .created_dirs
            .append(&mut changes.created_dirs);
    }

    fn rollback(&mut self) -> io::Result<()> {
        if !self.active {
            return Ok(());
        }
        let mut failures = Vec::new();
        for snapshot in self.snapshots.iter().rev() {
            if let Err(error) = snapshot.restore() {
                failures.push(format!("{}: {error}", snapshot.path.display()));
            }
        }
        if let Err(error) = self.media_changes.rollback() {
            failures.push(format!("media rollback: {error}"));
        }
        if failures.is_empty() {
            self.active = false;
            Ok(())
        } else {
            Err(io::Error::other(failures.join("; ")))
        }
    }

    fn commit(&mut self) {
        self.active = false;
    }
}

#[cfg(target_os = "macos")]
impl Drop for MacosIntegrationTransaction {
    fn drop(&mut self) {
        let _ = self.rollback();
    }
}

#[cfg(target_os = "macos")]
fn preflight_owned_macos_receipt(path: &Path, root: &Path) -> io::Result<()> {
    match fs::symlink_metadata(path) {
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(error),
        Ok(_) if receipt_is_owned(path, root)? => Ok(()),
        Ok(_) => Err(io::Error::other(format!(
            "refusing to replace foreign install receipt {}",
            path.display()
        ))),
    }
}

#[cfg(any(test, target_os = "linux", target_os = "macos"))]
fn preflight_owned_text_file(path: &Path, marker: &str) -> io::Result<()> {
    match fs::symlink_metadata(path) {
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(error),
        Ok(metadata)
            if metadata.is_file()
                && !metadata.file_type().is_symlink()
                && fs::read_to_string(path)?.contains(marker) =>
        {
            Ok(())
        }
        Ok(_) => Err(io::Error::other(format!(
            "refusing to replace foreign integration {}",
            path.display()
        ))),
    }
}

#[cfg(target_os = "macos")]
fn write_macos_receipt(
    path: &Path,
    root: &Path,
    metadata: &MacosBundleMetadata,
    autostart: bool,
) -> io::Result<()> {
    use std::os::unix::fs::PermissionsExt;

    preflight_owned_macos_receipt(path, root)?;
    let parent = path
        .parent()
        .ok_or_else(|| io::Error::other("install receipt has no parent"))?;
    ensure_real_directory(parent)?;
    let temp = parent.join(format!(".install-receipt.{}.tmp", std::process::id()));
    if fs::symlink_metadata(&temp).is_ok() {
        return Err(io::Error::new(
            io::ErrorKind::AlreadyExists,
            format!("stale receipt transaction exists: {}", temp.display()),
        ));
    }
    let payload = root.join("Contents").join("MacOS").join("honk300");
    let payload_size = fs::metadata(&payload)?.len();
    let payload_hash = sha256_file(&payload)?;
    let mut receipt = macos_install_receipt(metadata, root, autostart, &payload_hash, payload_size);
    let settings = payload.with_file_name(companions::SETTINGS_NAME);
    let settings_metadata = fs::symlink_metadata(&settings)?;
    if !settings_metadata.is_file() || settings_metadata.file_type().is_symlink() {
        return Err(io::Error::other(
            "macOS settings companion is not a regular executable",
        ));
    }
    receipt["settings_app"] = serde_json::json!({
        "name": companions::SETTINGS_NAME, "size": settings_metadata.len(),
        "sha256": sha256_file(&settings)?
    });
    let bytes = serde_json::to_vec_pretty(&receipt)?;
    fs::write(&temp, bytes)?;
    fs::set_permissions(&temp, fs::Permissions::from_mode(0o600))?;
    if let Err(error) = fs::rename(&temp, path) {
        let _ = fs::remove_file(&temp);
        return Err(error);
    }
    Ok(())
}

#[cfg(target_os = "macos")]
fn macos_app_install_path() -> Result<PathBuf, DynError> {
    Ok(home_dir()?.join("Applications").join("Honk300.app"))
}

#[cfg(target_os = "macos")]
fn macos_user_alias_dir() -> Result<PathBuf, DynError> {
    Ok(home_dir()?.join(".local").join("bin"))
}

#[cfg(target_os = "macos")]
fn macos_launch_agent_path() -> Result<PathBuf, DynError> {
    Ok(home_dir()?
        .join("Library")
        .join("LaunchAgents")
        .join("dev.emmetts.honk300.plist"))
}

// Matches honk-config's macOS `default_config_path` root so `--purge` removes the same tree the
// config/state actually lives in.
#[cfg(target_os = "macos")]
fn macos_config_state_root() -> Result<PathBuf, DynError> {
    Ok(home_dir()?
        .join("Library")
        .join("Application Support")
        .join(APP_NAME))
}

#[cfg(target_os = "macos")]
fn macos_media_root() -> Result<PathBuf, DynError> {
    Ok(macos_media_root_from(&home_dir()?))
}

#[cfg(target_os = "macos")]
fn macos_receipt_path() -> Result<PathBuf, DynError> {
    Ok(macos_config_state_root()?.join("install-receipt.json"))
}

#[cfg(target_os = "macos")]
fn macos_backup_root() -> Result<PathBuf, DynError> {
    Ok(home_dir()?
        .join("Library")
        .join("Application Support")
        .join("honk300-backups"))
}

/// The login LaunchAgent that runs the bundle binary with `start` at login (`--autostart`).
#[cfg(target_os = "macos")]
fn macos_launch_agent_plist(program: &Path) -> String {
    let program = xml_escape(&program.to_string_lossy());
    format!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n\
<!-- {OWNERSHIP_MARKER} -->\n\
<!DOCTYPE plist PUBLIC \"-//Apple//DTD PLIST 1.0//EN\"\n  \"http://www.apple.com/DTDs/PropertyList-1.0.dtd\">\n\
<plist version=\"1.0\">\n\
<dict>\n\
  <key>Label</key>\n  <string>dev.emmetts.honk300</string>\n\
  <key>ProgramArguments</key>\n  <array>\n    <string>{program}</string>\n    <string>start</string>\n  </array>\n\
  <key>RunAtLoad</key>\n  <true/>\n\
  <key>ProcessType</key>\n  <string>Interactive</string>\n\
</dict>\n\
</plist>\n"
    )
}

#[cfg(target_os = "macos")]
fn xml_escape(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

#[cfg(test)]
mod tests;
