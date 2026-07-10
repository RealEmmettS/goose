use std::fs;
use std::io;
use std::path::{Path, PathBuf};

const APP_NAME: &str = "honk300";
#[cfg(windows)]
const DISPLAY_NAME: &str = "Honk300";
const MARKER_FILE: &str = "install-source.txt";
const COMMAND_NAMES: &[&str] = &["honk300", "honk", "goose"];
const OWNERSHIP_MARKER: &str = "honk300.install.v1";
#[cfg(any(test, target_os = "linux", target_os = "macos"))]
const PATH_MARKER_START: &str = "# >>> honk300 managed PATH >>>";
#[cfg(any(test, target_os = "linux", target_os = "macos"))]
const PATH_MARKER_END: &str = "# <<< honk300 managed PATH <<<";

type DynError = Box<dyn std::error::Error>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InstallSource {
    MsiGlobal,
    MsiCorporate,
    ExeGlobal,
    ExeCorporate,
    ManualLocal,
    Shell,
    PowerShell,
    /// A macOS `honk300.app` bundle installed under `~/Applications` (R3, ADR 0017). Distinct
    /// from `ManualLocal` because its `update` path replaces the bundle from a DMG rather than
    /// re-running a per-user installer.
    MacApp,
    Unknown,
}

impl InstallSource {
    pub fn from_marker(value: &str) -> Self {
        match value.trim() {
            "msi-global" => Self::MsiGlobal,
            "msi-corporate" => Self::MsiCorporate,
            "exe-global" => Self::ExeGlobal,
            "exe-corporate" => Self::ExeCorporate,
            "manual-local" => Self::ManualLocal,
            "shell" => Self::Shell,
            "powershell" => Self::PowerShell,
            "mac-app" => Self::MacApp,
            _ => Self::Unknown,
        }
    }

    pub fn marker_value(self) -> &'static str {
        match self {
            Self::MsiGlobal => "msi-global",
            Self::MsiCorporate => "msi-corporate",
            Self::ExeGlobal => "exe-global",
            Self::ExeCorporate => "exe-corporate",
            Self::ManualLocal => "manual-local",
            Self::Shell => "shell",
            Self::PowerShell => "powershell",
            Self::MacApp => "mac-app",
            Self::Unknown => "unknown",
        }
    }
}

pub fn detect_install_source() -> InstallSource {
    #[cfg(windows)]
    if let Some(source) = read_windows_install_source_marker() {
        return source;
    }

    if let Some(source) = read_file_install_source_marker() {
        return source;
    }

    // A macOS bundle install is authoritative from the running executable's own location: the
    // `~/.local/bin` aliases resolve (via `current_exe`) into `…/honk300.app/Contents/MacOS`, so
    // `update` can pick the DMG replacement path. Anything else on macOS (shell/cargo-home/bare)
    // falls through to the shell-installer path like Linux.
    if cfg!(target_os = "macos") && current_exe_is_app_bundle() {
        return InstallSource::MacApp;
    }

    classify_current_exe_install_source()
}

/// True when the running executable lives inside a `*.app` bundle. Kept non-`cfg` (and exercised
/// on every host) so it never reads as dead code on non-macOS builds; the `cfg!` guard above keeps
/// the `MacApp` verdict macOS-only in practice.
fn current_exe_is_app_bundle() -> bool {
    std::env::current_exe()
        .ok()
        .as_deref()
        .map(path_is_in_app_bundle)
        .unwrap_or(false)
}

fn path_is_in_app_bundle(path: &Path) -> bool {
    path.ancestors()
        .any(|ancestor| ancestor.extension().and_then(|ext| ext.to_str()) == Some("app"))
}

#[cfg(windows)]
pub fn install(autostart: bool) -> Result<(), DynError> {
    let root = windows_user_install_root()?;
    ensure_owned_install_root(&root, &[InstallSource::ManualLocal])?;
    let bin_dir = root.join("bin");
    fs::create_dir_all(&bin_dir)?;

    let media = windows_media_root()?;
    ensure_external_media_root(&media)?;
    migrate_legacy_user_media(&bin_dir.join("Assets"), &media, LegacyMigrationMode::Move)?;
    copy_current_exe_to_aliases(&bin_dir)?;
    write_install_marker(&root, InstallSource::ManualLocal)?;
    write_windows_install_source_marker(InstallSource::ManualLocal)?;
    add_windows_user_path(&bin_dir)?;
    create_windows_start_menu_shortcut(&bin_dir.join("honk300.exe"))?;

    if autostart {
        set_windows_autostart(Some(&bin_dir.join("honk300.exe")))?;
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
    let bin_dir = root.join("bin");
    fs::create_dir_all(&bin_dir)?;

    let media = linux_media_root()?;
    ensure_external_media_root(&media)?;
    migrate_legacy_user_media(&bin_dir.join("Assets"), &media, LegacyMigrationMode::Move)?;
    let installed = bin_dir.join("honk300");
    copy_current_exe(&installed)?;
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
    if !is_exact_macos_managed_executable(&current_exe, &app_dir) {
        return Err(format!(
            "honk300 install: this command may only configure the sealed app already installed at {}. Install it with the official shell installer first, then run `{}` install.",
            app_dir.display(),
            installed_bin.display()
        )
        .into());
    }

    let media = macos_media_root()?;
    ensure_external_media_root(&media)?;
    migrate_legacy_user_media(
        &app_dir.join("Contents").join("Resources").join("Assets"),
        &media,
        LegacyMigrationMode::Copy,
    )?;

    let aliases_dir = macos_user_alias_dir()?;
    fs::create_dir_all(&aliases_dir)?;
    let owned_targets = [&installed_bin as &Path];
    for name in COMMAND_NAMES {
        install_owned_unix_alias(&aliases_dir.join(name), &installed_bin, &owned_targets)?;
    }

    let plist_path = macos_launch_agent_path()?;
    if autostart {
        write_owned_text_file(
            &plist_path,
            &macos_launch_agent_plist(&installed_bin),
            OWNERSHIP_MARKER,
        )?;
    } else {
        remove_owned_text_file(&plist_path, OWNERSHIP_MARKER)?;
    }

    println!("honk300: installed {}.", app_dir.display());
    println!("honk300: aliases linked in {}.", aliases_dir.display());
    if !path_contains(&aliases_dir) {
        println!(
            "honk300: {} is not currently on PATH; add it or open a shell that loads it.",
            aliases_dir.display()
        );
    }
    println!("honk300: first launch is Gatekeeper-quarantined (unsigned) — right-click the app in Finder and choose Open once to approve it.");
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
    if matches!(
        source,
        InstallSource::MsiGlobal
            | InstallSource::MsiCorporate
            | InstallSource::ExeGlobal
            | InstallSource::ExeCorporate
    ) {
        return uninstall_windows_managed(source, purge);
    }

    let root = windows_user_install_root()?;
    let bin_dir = root.join("bin");
    ensure_owned_install_root(&root, &[InstallSource::ManualLocal])?;
    let media = windows_media_root()?;
    ensure_external_media_root(&media)?;
    migrate_legacy_user_media(&bin_dir.join("Assets"), &media, LegacyMigrationMode::Move)?;
    let backup_root = windows_backup_root()?;
    let backup = if purge {
        backup_user_content(&media, &backup_root)?
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

#[cfg(target_os = "linux")]
pub fn uninstall(purge: bool) -> Result<(), DynError> {
    let root = linux_user_install_root()?;
    let receipt = linux_receipt_path()?;
    ensure_owned_install_root_or_receipt(
        &root,
        &[InstallSource::ManualLocal, InstallSource::Shell],
        &receipt,
    )?;
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

#[cfg(any(windows, target_os = "linux"))]
fn write_install_marker(root: &Path, source: InstallSource) -> io::Result<()> {
    write_text_file(&root.join(MARKER_FILE), source.marker_value())
}

fn read_file_install_source_marker() -> Option<InstallSource> {
    for marker in current_root_marker_candidates() {
        let Ok(value) = fs::read_to_string(marker) else {
            continue;
        };
        let source = InstallSource::from_marker(&value);
        if source != InstallSource::Unknown {
            return Some(source);
        }
    }
    None
}

fn current_root_marker_candidates() -> Vec<PathBuf> {
    let mut candidates = Vec::new();
    if let Ok(exe) = std::env::current_exe() {
        if let Some(parent) = exe.parent() {
            candidates.push(parent.join(MARKER_FILE));
            if parent.file_name().and_then(|name| name.to_str()) == Some("bin") {
                if let Some(root) = parent.parent() {
                    candidates.push(root.join(MARKER_FILE));
                }
            }
        }
    }
    candidates
}

fn classify_current_exe_install_source() -> InstallSource {
    let Ok(exe) = std::env::current_exe() else {
        return InstallSource::Unknown;
    };
    classify_install_path(&exe.to_string_lossy())
}

fn classify_install_path(path: &str) -> InstallSource {
    let lower = path.to_ascii_lowercase().replace('/', "\\");
    if lower.contains("\\program files\\honk300\\") {
        InstallSource::MsiGlobal
    } else if lower.contains("\\appdata\\local\\programs\\honk300\\") {
        InstallSource::MsiCorporate
    } else if lower.contains("\\.local\\share\\honk300\\install\\") {
        InstallSource::Shell
    } else {
        InstallSource::Unknown
    }
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

#[cfg(any(test, windows))]
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

#[cfg(any(test, windows))]
fn windows_media_root_from(local_app_data: &Path) -> PathBuf {
    local_app_data.join(APP_NAME).join("media")
}

#[cfg(any(test, target_os = "macos"))]
fn macos_media_root_from(home: &Path) -> PathBuf {
    home.join("Library")
        .join("Application Support")
        .join(APP_NAME)
        .join("media")
}

#[cfg(any(test, target_os = "linux"))]
fn linux_media_root_from(xdg_data_home: Option<&Path>, home: &Path) -> PathBuf {
    xdg_data_home
        .map(Path::to_path_buf)
        .unwrap_or_else(|| home.join(".local").join("share"))
        .join(APP_NAME)
        .join("media")
}

fn ensure_external_media_root(media_root: &Path) -> io::Result<()> {
    ensure_real_directory(media_root)?;
    ensure_real_directory(&media_root.join("Memes"))?;
    ensure_real_directory(&media_root.join("Notes"))
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

fn ensure_media_destination_parent(media_root: &Path, destination: &Path) -> io::Result<()> {
    let parent = destination
        .parent()
        .ok_or_else(|| io::Error::other("external media destination has no parent"))?;
    let relative = parent.strip_prefix(media_root).map_err(|_| {
        io::Error::other(format!(
            "media destination escaped external root: {}",
            destination.display()
        ))
    })?;
    let mut current = media_root.to_path_buf();
    for component in relative.components() {
        match component {
            std::path::Component::Normal(name) => current.push(name),
            _ => {
                return Err(io::Error::other(format!(
                    "invalid external media destination {}",
                    destination.display()
                )));
            }
        }
        ensure_real_directory(&current)?;
    }
    Ok(())
}

fn migrate_legacy_user_media(
    legacy_assets: &Path,
    media_root: &Path,
    mode: LegacyMigrationMode,
) -> io::Result<()> {
    let mappings = [
        (
            legacy_assets.join("Images").join("Memes").join("user"),
            media_root.join("Memes"),
        ),
        (
            legacy_assets
                .join("Text")
                .join("NotepadMessages")
                .join("user"),
            media_root.join("Notes"),
        ),
    ];
    let mut files = Vec::new();
    for (source, destination) in &mappings {
        collect_migration_files(source, destination, &mut files)?;
    }
    ensure_external_media_root(media_root)?;
    let mut pending = Vec::new();
    for (source, destination) in &files {
        ensure_media_destination_parent(media_root, destination)?;
        match fs::symlink_metadata(destination) {
            Ok(metadata) if metadata.is_file() && !metadata.file_type().is_symlink() => {
                if !regular_files_equal(source, destination)? {
                    return Err(io::Error::new(
                        io::ErrorKind::AlreadyExists,
                        format!(
                            "refusing to overwrite existing external media {}",
                            destination.display()
                        ),
                    ));
                }
            }
            Ok(_) => {
                return Err(io::Error::new(
                    io::ErrorKind::AlreadyExists,
                    format!(
                        "refusing to overwrite existing external media {}",
                        destination.display()
                    ),
                ));
            }
            Err(error) if error.kind() == io::ErrorKind::NotFound => {
                pending.push((source, destination));
            }
            Err(error) => return Err(error),
        }
    }
    for (source, destination) in pending {
        fs::copy(source, destination)?;
    }
    #[cfg(all(target_os = "macos", not(test)))]
    let _ = mode;
    #[cfg(any(test, windows, target_os = "linux"))]
    if mode == LegacyMigrationMode::Move {
        for (source, _) in &mappings {
            if source.exists() {
                fs::remove_dir_all(source)?;
            }
        }
    }
    Ok(())
}

fn regular_files_equal(left: &Path, right: &Path) -> io::Result<bool> {
    use std::io::Read;

    if fs::metadata(left)?.len() != fs::metadata(right)?.len() {
        return Ok(false);
    }
    let mut left = fs::File::open(left)?;
    let mut right = fs::File::open(right)?;
    let mut left_buffer = [0_u8; 16 * 1024];
    let mut right_buffer = [0_u8; 16 * 1024];
    loop {
        let left_read = left.read(&mut left_buffer)?;
        let right_read = right.read(&mut right_buffer)?;
        if left_read != right_read || left_buffer[..left_read] != right_buffer[..right_read] {
            return Ok(false);
        }
        if left_read == 0 {
            return Ok(true);
        }
    }
}

fn collect_migration_files(
    source: &Path,
    destination: &Path,
    files: &mut Vec<(PathBuf, PathBuf)>,
) -> io::Result<()> {
    let metadata = match fs::symlink_metadata(source) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(()),
        Err(error) => return Err(error),
    };
    if !metadata.is_dir() || metadata.file_type().is_symlink() {
        return Err(io::Error::other(format!(
            "legacy media path is not a real directory: {}",
            source.display()
        )));
    }
    for entry in fs::read_dir(source)? {
        let entry = entry?;
        let file_type = entry.file_type()?;
        let next_destination = destination.join(entry.file_name());
        if file_type.is_dir() {
            collect_migration_files(&entry.path(), &next_destination, files)?;
        } else if file_type.is_file() {
            files.push((entry.path(), next_destination));
        } else {
            return Err(io::Error::other(format!(
                "refusing to migrate symlink or special file {}",
                entry.path().display()
            )));
        }
    }
    Ok(())
}

fn backup_user_content(media_root: &Path, backup_root: &Path) -> io::Result<Option<PathBuf>> {
    backup_user_content_at(media_root, backup_root, unix_timestamp())
}

fn backup_user_content_at(
    media_root: &Path,
    backup_root: &Path,
    timestamp: u64,
) -> io::Result<Option<PathBuf>> {
    if !media_has_user_content(media_root)? {
        return Ok(None);
    }
    let destination = backup_root.join(format!("purge-{timestamp}"));
    for name in ["Memes", "Notes"] {
        let source = media_root.join(name);
        if source.exists() {
            copy_dir_recursive(&source, &destination.join(name))?;
        }
    }
    Ok(Some(destination))
}

fn media_has_user_content(media_root: &Path) -> io::Result<bool> {
    for name in ["Memes", "Notes"] {
        if directory_has_entries(&media_root.join(name))? {
            return Ok(true);
        }
    }
    Ok(false)
}

fn directory_has_entries(path: &Path) -> io::Result<bool> {
    match fs::read_dir(path) {
        Ok(mut entries) => Ok(entries.next().transpose()?.is_some()),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(false),
        Err(error) => Err(error),
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
    Ok(schema == Some(OWNERSHIP_MARKER)
        && root.is_some_and(|recorded| paths_match(recorded, install_root)))
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
fn write_windows_install_source_marker(source: InstallSource) -> io::Result<()> {
    use winreg::enums::HKEY_CURRENT_USER;
    use winreg::RegKey;

    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let (key, _) = hkcu.create_subkey("Software\\Honk300")?;
    key.set_value("InstallSource", &source.marker_value())
}

#[cfg(windows)]
fn read_windows_install_source_marker() -> Option<InstallSource> {
    use winreg::enums::{HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE};
    use winreg::RegKey;

    // Machine-wide installers are authoritative even if an older per-user/manual marker remains.
    for hive in [HKEY_LOCAL_MACHINE, HKEY_CURRENT_USER] {
        let root = RegKey::predef(hive);
        let Ok(key) = root.open_subkey("Software\\Honk300") else {
            continue;
        };
        let Ok(value) = key.get_value::<String, _>("InstallSource") else {
            continue;
        };
        let source = InstallSource::from_marker(&value);
        if source != InstallSource::Unknown {
            return Some(source);
        }
    }
    None
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum LegacyMigrationMode {
    #[cfg(any(test, windows, target_os = "linux"))]
    Move,
    #[cfg(any(test, windows, target_os = "macos"))]
    Copy,
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
        "$w = New-Object -ComObject WScript.Shell; $s = $w.CreateShortcut('{}'); $s.TargetPath = '{}'; $s.Arguments = 'start'; $s.WorkingDirectory = '{}'; $s.Save()",
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
fn set_windows_autostart(exe: Option<&Path>) -> io::Result<()> {
    use winreg::enums::HKEY_CURRENT_USER;
    use winreg::RegKey;

    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let (run, _) = hkcu.create_subkey("Software\\Microsoft\\Windows\\CurrentVersion\\Run")?;
    if let Some(exe) = exe {
        run.set_value("Honk300", &format!("\"{}\" start", exe.display()))
    } else {
        let _ = run.delete_value("Honk300");
        Ok(())
    }
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
fn windows_post_exit_helper_invocation(
    current_pid: u32,
    plan: &WindowsManagedUninstall,
    receipt: Option<&Path>,
    _install_root: &Path,
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
    let receipt_cleanup = receipt.map_or_else(String::new, |path| {
        format!(
            "; Remove-Item -LiteralPath '{}' -Force -ErrorAction SilentlyContinue",
            powershell_literal(&path.to_string_lossy())
        )
    });
    let script = format!(
        "$ErrorActionPreference='Stop'; Wait-Process -Id {current_pid} -ErrorAction SilentlyContinue; $process = Start-Process -FilePath '{}' -ArgumentList {arguments} -WindowStyle Hidden -Wait -PassThru{elevation}; if ($process.ExitCode -notin @(0,1605,1641,3010)) {{ exit $process.ExitCode }}{receipt_cleanup}",
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
fn powershell_literal(value: &str) -> String {
    value.replace('\'', "''")
}

#[cfg(windows)]
fn uninstall_windows_managed(source: InstallSource, purge: bool) -> Result<(), DynError> {
    use std::os::windows::process::CommandExt;

    const CREATE_NO_WINDOW: u32 = 0x0800_0000;
    let current_exe = std::env::current_exe()?;
    let Some((plan, install_root)) = find_windows_managed_uninstall(source, &current_exe)? else {
        return Err("honk300 uninstall: the Windows installer identity could not be proven, so no installed files were touched. Uninstall Honk300 from Windows Installed Apps instead.".into());
    };

    let media = windows_media_root()?;
    ensure_external_media_root(&media)?;
    if let Some(bin_dir) = current_exe.parent() {
        migrate_legacy_user_media(&bin_dir.join("Assets"), &media, LegacyMigrationMode::Copy)?;
    }
    let backup = if purge {
        backup_user_content(&media, &windows_backup_root()?)?
    } else {
        None
    };
    let receipt = windows_receipt_path()?;
    let owned_receipt = receipt_is_owned(&receipt, &install_root)?;
    if purge {
        purge_config_state_preserving_foreign_receipt(
            &windows_config_state_root()?,
            &install_root,
        )?;
        report_backup(backup);
    } else {
        report_preserved(media_has_user_content(&media)?.then_some(media));
    }

    let receipt_for_helper = (!purge && owned_receipt).then_some(receipt.as_path());
    let system_msiexec = system_windows_msiexec_path()?;
    let invocation = windows_post_exit_helper_invocation(
        std::process::id(),
        &plan,
        receipt_for_helper,
        &install_root,
        &system_msiexec,
    );
    std::process::Command::new(system_windows_powershell_path()?)
        .args(&invocation.args)
        .creation_flags(CREATE_NO_WINDOW)
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()?;
    println!("honk300: verified Windows installer uninstall will begin after this process exits.");
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

    let hive = match source {
        InstallSource::MsiGlobal | InstallSource::ExeGlobal => HKEY_LOCAL_MACHINE,
        InstallSource::MsiCorporate | InstallSource::ExeCorporate => HKEY_CURRENT_USER,
        _ => return Ok(None),
    };
    let root = RegKey::predef(hive);
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
            if let Some(plan) = validate_windows_uninstall_identity(source, current_exe, &identity)
            {
                return Ok(Some((plan, identity.install_location)));
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
    match alias_install_decision(state, target, owned_targets) {
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
mod tests {
    use super::*;

    #[test]
    fn install_source_markers_are_stable() {
        for (marker, source) in [
            ("msi-global", InstallSource::MsiGlobal),
            ("msi-corporate", InstallSource::MsiCorporate),
            ("exe-global", InstallSource::ExeGlobal),
            ("exe-corporate", InstallSource::ExeCorporate),
            ("manual-local", InstallSource::ManualLocal),
            ("shell", InstallSource::Shell),
            ("powershell", InstallSource::PowerShell),
            ("mac-app", InstallSource::MacApp),
        ] {
            assert_eq!(InstallSource::from_marker(marker), source);
            assert_eq!(source.marker_value(), marker);
        }
        assert_eq!(InstallSource::from_marker("cargo"), InstallSource::Unknown);
    }

    #[test]
    fn app_bundle_detection_matches_only_app_ancestors() {
        assert!(path_is_in_app_bundle(Path::new(
            "/Users/a/Applications/Honk300.app/Contents/MacOS/honk300"
        )));
        assert!(path_is_in_app_bundle(Path::new(
            "/Volumes/Honk300/Honk300.app/Contents/MacOS/goose"
        )));
        assert!(!path_is_in_app_bundle(Path::new(
            "/Users/a/.local/share/honk300/install/bin/honk300"
        )));
        assert!(!path_is_in_app_bundle(Path::new("/usr/local/bin/honk300")));
    }

    #[test]
    fn install_path_classification_never_selects_cargo_update_path() {
        assert_eq!(
            classify_install_path(r"C:\Program Files\honk300\bin\honk300.exe"),
            InstallSource::MsiGlobal
        );
        assert_eq!(
            classify_install_path(r"C:\Users\a\AppData\Local\Programs\honk300\bin\goose.exe"),
            InstallSource::MsiCorporate
        );
        assert_eq!(
            classify_install_path("/home/a/.local/share/honk300/install/bin/honk300"),
            InstallSource::Shell
        );
        assert_eq!(
            classify_install_path(r"C:\Users\a\.cargo\bin\honk300.exe"),
            InstallSource::Unknown
        );
    }

    #[test]
    fn purge_backup_copies_user_memes_and_notes() {
        let root = test_dir("backup");
        let media = root.join("media");
        let memes = media.join("Memes");
        let notes = media.join("Notes");
        fs::create_dir_all(&memes).unwrap();
        fs::create_dir_all(&notes).unwrap();
        fs::write(memes.join("mine.png"), b"png").unwrap();
        fs::write(notes.join("mine.txt"), b"note").unwrap();

        let backup = backup_user_content_at(&media, &root.join("backups"), 123)
            .unwrap()
            .unwrap();

        assert!(backup.join("Memes/mine.png").exists());
        assert!(backup.join("Notes/mine.txt").exists());
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn purge_backup_reports_none_without_user_content() {
        let root = test_dir("empty-backup");
        let media = root.join("media");
        ensure_external_media_root(&media).unwrap();
        assert_eq!(
            backup_user_content_at(&media, &root.join("backups"), 123)
                .unwrap()
                .as_ref(),
            None
        );
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn non_purge_uninstall_leaves_external_user_media_in_place() {
        let root = test_dir("preserve");
        let media = root.join("media");
        let memes = media.join("Memes");
        let notes = media.join("Notes");
        fs::create_dir_all(&memes).unwrap();
        fs::create_dir_all(&notes).unwrap();
        fs::write(memes.join("mine.png"), b"png").unwrap();
        fs::write(notes.join("mine.txt"), b"note").unwrap();

        assert!(media_has_user_content(&media).unwrap());
        assert_eq!(fs::read(memes.join("mine.png")).unwrap(), b"png");
        assert_eq!(fs::read(notes.join("mine.txt")).unwrap(), b"note");
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn macos_install_requires_exact_managed_app_and_mutates_only_external_paths() {
        let home = Path::new("/Users/goose");
        let app = home.join("Applications/Honk300.app");
        let exact = app.join("Contents/MacOS/honk300");
        assert!(is_exact_macos_managed_executable(&exact, &app));
        assert!(!is_exact_macos_managed_executable(
            Path::new("/Volumes/Honk300/Honk300.app/Contents/MacOS/honk300"),
            &app
        ));
        assert!(!is_exact_macos_managed_executable(
            Path::new("/Users/goose/.local/bin/honk300"),
            &app
        ));

        for mutation in macos_external_mutation_paths(home) {
            assert!(
                !mutation.starts_with(&app),
                "sealed app mutation leaked into plan: {}",
                mutation.display()
            );
        }
    }

    #[test]
    fn unix_alias_decisions_preserve_foreign_files_and_symlinks() {
        let desired = Path::new("/home/goose/.local/share/honk300/install/bin/honk300");
        let old_owned = Path::new("/home/goose/Applications/Honk300.app/Contents/MacOS/honk300");
        let owned = [desired, old_owned];
        assert_eq!(
            alias_install_decision(AliasState::Missing, desired, &owned),
            AliasInstallDecision::Create
        );
        assert_eq!(
            alias_install_decision(AliasState::Symlink(desired), desired, &owned),
            AliasInstallDecision::Keep
        );
        assert_eq!(
            alias_install_decision(AliasState::Symlink(old_owned), desired, &owned),
            AliasInstallDecision::ReplaceOwned
        );
        assert_eq!(
            alias_install_decision(
                AliasState::Symlink(Path::new("/opt/foreign/goose")),
                desired,
                &owned,
            ),
            AliasInstallDecision::PreserveForeign
        );
        assert_eq!(
            alias_install_decision(AliasState::Other, desired, &owned),
            AliasInstallDecision::PreserveForeign
        );
    }

    #[test]
    fn owned_text_integrations_never_replace_or_remove_foreign_files() {
        let root = test_dir("owned-text");
        let path = root.join("honk300.desktop");
        fs::create_dir_all(&root).unwrap();
        fs::write(&path, "foreign desktop entry\n").unwrap();
        assert!(write_owned_text_file(&path, "replacement", OWNERSHIP_MARKER).is_err());
        assert_eq!(
            fs::read_to_string(&path).unwrap(),
            "foreign desktop entry\n"
        );
        assert!(!remove_owned_text_file(&path, OWNERSHIP_MARKER).unwrap());
        assert!(path.exists());

        fs::write(&path, format!("# {OWNERSHIP_MARKER}\nowned\n")).unwrap();
        write_owned_text_file(
            &path,
            &format!("# {OWNERSHIP_MARKER}\nupdated\n"),
            OWNERSHIP_MARKER,
        )
        .unwrap();
        assert!(remove_owned_text_file(&path, OWNERSHIP_MARKER).unwrap());
        assert!(!path.exists());
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn managed_path_blocks_are_removed_without_touching_profile_content() {
        let profile = "export EDITOR=vim\n\n# >>> honk300 managed PATH >>>\nexport PATH=\"$HOME/.local/bin:$PATH\"\n# <<< honk300 managed PATH <<<\nalias ll='ls -l'\n";
        let (updated, changed) = strip_managed_path_blocks(profile);
        assert!(changed);
        assert!(updated.contains("export EDITOR=vim\n"));
        assert!(updated.contains("alias ll='ls -l'\n"));
        assert!(!updated.contains("honk300 managed PATH"));

        let foreign = "# >>> somebody else PATH >>>\nexport PATH=/opt/foreign:$PATH\n";
        assert_eq!(strip_managed_path_blocks(foreign), (foreign.into(), false));
    }

    #[test]
    fn external_receipts_are_removed_only_for_matching_owned_install_root() {
        let root = test_dir("receipt");
        let receipt = root.join("install-receipt.json");
        let install_root = root.join("install");
        fs::create_dir_all(&root).unwrap();
        fs::write(
            &receipt,
            format!(
                "{{\"schema\":\"honk300.install.v1\",\"install_root\":{:?}}}",
                install_root.to_string_lossy()
            ),
        )
        .unwrap();
        assert!(remove_owned_receipt(&receipt, &install_root).unwrap());
        assert!(!receipt.exists());

        fs::write(
            &receipt,
            "{\"schema\":\"foreign.install.v1\",\"install_root\":\"x\"}",
        )
        .unwrap();
        assert!(!remove_owned_receipt(&receipt, &install_root).unwrap());
        assert!(receipt.exists());
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn platform_media_roots_are_external_user_data_locations() {
        assert_eq!(
            windows_media_root_from(Path::new(r"C:\Users\goose\AppData\Local")),
            PathBuf::from(r"C:\Users\goose\AppData\Local\honk300\media")
        );
        assert_eq!(
            macos_media_root_from(Path::new("/Users/goose")),
            PathBuf::from("/Users/goose/Library/Application Support/honk300/media")
        );
        assert_eq!(
            linux_media_root_from(Some(Path::new("/data")), Path::new("/home/goose")),
            PathBuf::from("/data/honk300/media")
        );
        assert_eq!(
            linux_media_root_from(None, Path::new("/home/goose")),
            PathBuf::from("/home/goose/.local/share/honk300/media")
        );
    }

    #[test]
    fn legacy_user_media_migrates_to_external_memes_and_notes_without_overwrite() {
        let root = test_dir("media-migrate");
        let assets = root.join("Assets");
        let media = root.join("media");
        let legacy_memes = assets.join("Images/Memes/user");
        let legacy_notes = assets.join("Text/NotepadMessages/user");
        fs::create_dir_all(&legacy_memes).unwrap();
        fs::create_dir_all(&legacy_notes).unwrap();
        fs::write(legacy_memes.join("mine.png"), b"png").unwrap();
        fs::write(legacy_notes.join("mine.txt"), b"note").unwrap();

        migrate_legacy_user_media(&assets, &media, LegacyMigrationMode::Move).unwrap();
        assert_eq!(fs::read(media.join("Memes/mine.png")).unwrap(), b"png");
        assert_eq!(fs::read(media.join("Notes/mine.txt")).unwrap(), b"note");
        assert!(!legacy_memes.join("mine.png").exists());
        assert!(!legacy_notes.join("mine.txt").exists());

        fs::create_dir_all(&legacy_memes).unwrap();
        fs::write(legacy_memes.join("mine.png"), b"png").unwrap();
        migrate_legacy_user_media(&assets, &media, LegacyMigrationMode::Copy).unwrap();
        assert_eq!(fs::read(media.join("Memes/mine.png")).unwrap(), b"png");

        fs::write(legacy_memes.join("mine.png"), b"new").unwrap();
        assert!(migrate_legacy_user_media(&assets, &media, LegacyMigrationMode::Move).is_err());
        assert_eq!(fs::read(media.join("Memes/mine.png")).unwrap(), b"png");
        assert_eq!(fs::read(legacy_memes.join("mine.png")).unwrap(), b"new");
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn windows_managed_uninstall_identity_is_conservative() {
        let current_exe = Path::new(r"C:\Program Files\honk300\bin\honk300.exe");
        let entry = WindowsUninstallIdentity {
            key_name: "{01234567-89AB-CDEF-0123-456789ABCDEF}".into(),
            display_name: "honk300".into(),
            publisher: "Emmett S".into(),
            install_location: PathBuf::from(r"C:\Program Files\honk300"),
            uninstall_command: r"MsiExec.exe /I{01234567-89AB-CDEF-0123-456789ABCDEF}".into(),
            windows_installer: true,
        };
        assert!(matches!(
            validate_windows_uninstall_identity(InstallSource::MsiGlobal, current_exe, &entry),
            Some(WindowsManagedUninstall::Msi { .. })
        ));

        let mut foreign = entry.clone();
        foreign.publisher = "Somebody Else".into();
        assert!(validate_windows_uninstall_identity(
            InstallSource::MsiGlobal,
            current_exe,
            &foreign
        )
        .is_none());
        foreign = entry;
        foreign.install_location = PathBuf::from(r"C:\Program Files\Foreign");
        assert!(validate_windows_uninstall_identity(
            InstallSource::MsiGlobal,
            current_exe,
            &foreign
        )
        .is_none());
    }

    #[test]
    fn windows_post_exit_helper_is_hidden_waits_and_never_deletes_install_root() {
        let plan = WindowsManagedUninstall::Msi {
            product_code: "{01234567-89AB-CDEF-0123-456789ABCDEF}".into(),
            elevated: true,
        };
        let invocation = windows_post_exit_helper_invocation(
            4242,
            &plan,
            None,
            Path::new(r"C:\Program Files\honk300"),
            Path::new(r"C:\Windows\System32\msiexec.exe"),
        );
        assert!(invocation
            .args
            .windows(2)
            .any(|args| args == ["-WindowStyle", "Hidden"]));
        assert!(invocation.script.contains("Wait-Process -Id 4242"));
        assert!(invocation
            .script
            .contains(r"C:\Windows\System32\msiexec.exe"));
        assert!(!invocation.script.contains("-FilePath 'msiexec.exe'"));
        assert!(invocation.script.contains("/x"));
        assert!(!invocation.script.contains("Remove-Item -Recurse"));
    }

    fn test_dir(name: &str) -> PathBuf {
        std::env::temp_dir().join(format!(
            "honk300-install-{name}-{}-{}",
            std::process::id(),
            unix_timestamp()
        ))
    }
}
