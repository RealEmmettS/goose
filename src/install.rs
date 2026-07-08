use std::fs;
use std::io;
use std::path::{Path, PathBuf};

const APP_NAME: &str = "honk300";
#[cfg(windows)]
const DISPLAY_NAME: &str = "Honk300";
const MARKER_FILE: &str = "install-source.txt";
const COMMAND_NAMES: &[&str] = &["honk300", "honk", "goose"];

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
    let bin_dir = root.join("bin");
    fs::create_dir_all(&bin_dir)?;

    copy_current_exe_to_aliases(&bin_dir)?;
    copy_assets_next_to_binary(&bin_dir)?;
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
    let bin_dir = root.join("bin");
    fs::create_dir_all(&bin_dir)?;

    let installed = bin_dir.join("honk300");
    copy_current_exe(&installed)?;
    make_executable(&installed)?;
    copy_assets_next_to_binary(&bin_dir)?;
    write_install_marker(&root, InstallSource::ManualLocal)?;

    let aliases_dir = linux_user_alias_dir()?;
    fs::create_dir_all(&aliases_dir)?;
    for name in COMMAND_NAMES {
        replace_unix_symlink(&aliases_dir.join(name), &installed)?;
    }

    let desktop = linux_desktop_entry(&installed);
    let desktop_path = linux_applications_dir()?.join("honk300.desktop");
    write_text_file(&desktop_path, &desktop)?;
    if autostart {
        write_text_file(&linux_autostart_path()?, &desktop)?;
    } else {
        remove_file_if_exists(&linux_autostart_path()?)?;
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
    let contents = app_dir.join("Contents");
    let macos_dir = contents.join("MacOS");
    let resources = contents.join("Resources");
    let installed_bin = macos_dir.join("honk300");

    // Two provenance cases, mirroring package_macos_app.sh's staging: when running from an
    // already-staged bundle (DMG/app), copy the whole `.app`; when running as a bare binary,
    // synthesize the same bundle layout (Info.plist + Contents/MacOS/honk300 + Resources/Assets).
    let source_bundle = current_exe_app_bundle();
    let already_in_place = source_bundle
        .as_deref()
        .map(|bundle| same_file_best_effort(bundle, &app_dir))
        .unwrap_or(false);

    if !already_in_place {
        remove_dir_if_exists(&app_dir)?;
        match &source_bundle {
            Some(bundle) => copy_dir_recursive(bundle, &app_dir)?,
            None => stage_macos_app_bundle(&macos_dir, &resources, &installed_bin)?,
        }
    }

    write_install_marker(&resources, InstallSource::MacApp)?;

    let aliases_dir = macos_user_alias_dir()?;
    fs::create_dir_all(&aliases_dir)?;
    for name in COMMAND_NAMES {
        replace_unix_symlink(&aliases_dir.join(name), &installed_bin)?;
    }

    let plist_path = macos_launch_agent_path()?;
    if autostart {
        write_text_file(&plist_path, &macos_launch_agent_plist(&installed_bin))?;
    } else {
        remove_file_if_exists(&plist_path)?;
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

/// Build the `honk300.app` layout for a bare-binary install, matching `script/package_macos_app.sh`:
/// the running executable becomes `Contents/MacOS/honk300`, `Assets/` land in
/// `Contents/Resources/Assets`, and an LSUIElement `Info.plist` (version stamped from the running
/// build) is written.
#[cfg(target_os = "macos")]
fn stage_macos_app_bundle(
    macos_dir: &Path,
    resources: &Path,
    installed_bin: &Path,
) -> Result<(), DynError> {
    fs::create_dir_all(macos_dir)?;
    fs::create_dir_all(resources)?;
    copy_current_exe(installed_bin)?;
    make_executable(installed_bin)?;
    copy_assets_into(&resources.join("Assets"))?;
    write_text_file(
        &macos_dir
            .parent()
            .ok_or("app bundle Contents directory is missing")?
            .join("Info.plist"),
        &macos_info_plist(env!("CARGO_PKG_VERSION")),
    )?;
    Ok(())
}

#[cfg(not(any(windows, target_os = "linux", target_os = "macos")))]
pub fn install(_autostart: bool) -> Result<(), DynError> {
    Err("honk300 install: this OS is not supported by M19 lifecycle installers.".into())
}

#[cfg(windows)]
pub fn uninstall(purge: bool) -> Result<(), DynError> {
    let root = windows_user_install_root()?;
    let bin_dir = root.join("bin");
    let assets = bin_dir.join("Assets");
    let backup_root = windows_backup_root()?;
    // Rescue user-supplied memes/notes before the install tree is deleted: `--purge` backs them up
    // (then still removes them), while a plain uninstall preserves them and reports where.
    let (backup, preserved) = if purge {
        (backup_user_content(&assets, &backup_root)?, None)
    } else {
        (None, preserve_user_content(&assets, &backup_root)?)
    };

    set_windows_autostart(None)?;
    remove_windows_start_menu_shortcut()?;
    remove_windows_user_path(&bin_dir)?;
    remove_windows_install_source_marker()?;
    remove_dir_if_exists(&root)?;

    if purge {
        purge_config_state(windows_config_state_root()?)?;
        report_backup(backup);
    } else {
        report_preserved(preserved);
    }

    println!("honk300: uninstalled.");
    Ok(())
}

#[cfg(target_os = "linux")]
pub fn uninstall(purge: bool) -> Result<(), DynError> {
    let root = linux_user_install_root()?;
    let assets = root.join("bin").join("Assets");
    let backup_root = linux_backup_root()?;
    // Rescue user-supplied memes/notes before the install tree is deleted: `--purge` backs them up
    // (then still removes them), while a plain uninstall preserves them and reports where.
    let (backup, preserved) = if purge {
        (backup_user_content(&assets, &backup_root)?, None)
    } else {
        (None, preserve_user_content(&assets, &backup_root)?)
    };

    for name in COMMAND_NAMES {
        remove_file_if_exists(&linux_user_alias_dir()?.join(name))?;
    }
    remove_file_if_exists(&linux_applications_dir()?.join("honk300.desktop"))?;
    remove_file_if_exists(&linux_autostart_path()?)?;
    remove_dir_if_exists(&root)?;

    if purge {
        purge_config_state(linux_config_state_root()?)?;
        report_backup(backup);
    } else {
        report_preserved(preserved);
    }

    println!("honk300: uninstalled.");
    Ok(())
}

#[cfg(target_os = "macos")]
pub fn uninstall(purge: bool) -> Result<(), DynError> {
    let app_dir = macos_app_install_path()?;
    let assets = app_dir.join("Contents").join("Resources").join("Assets");
    let backup_root = macos_backup_root()?;
    // Rescue user-supplied memes/notes before the bundle is deleted: `--purge` backs them up
    // (then still removes them), while a plain uninstall preserves them and reports where.
    let (backup, preserved) = if purge {
        (backup_user_content(&assets, &backup_root)?, None)
    } else {
        (None, preserve_user_content(&assets, &backup_root)?)
    };

    for name in COMMAND_NAMES {
        remove_file_if_exists(&macos_user_alias_dir()?.join(name))?;
    }
    remove_file_if_exists(&macos_launch_agent_path()?)?;
    remove_dir_if_exists(&app_dir)?;

    if purge {
        purge_config_state(macos_config_state_root()?)?;
        report_backup(backup);
    } else {
        report_preserved(preserved);
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

fn same_file_best_effort(left: &Path, right: &Path) -> bool {
    match (left.canonicalize(), right.canonicalize()) {
        (Ok(left), Ok(right)) => left == right,
        _ => false,
    }
}

#[cfg(any(windows, target_os = "linux"))]
fn copy_assets_next_to_binary(bin_dir: &Path) -> io::Result<()> {
    copy_assets_into(&bin_dir.join("Assets"))
}

/// Copy the source `Assets/` tree to `dest_assets`, or scaffold an empty tree when no source is
/// found. Shared by the Windows/Linux "assets next to the binary" layout and the macOS
/// "assets in `Contents/Resources/Assets`" bundle layout.
fn copy_assets_into(dest_assets: &Path) -> io::Result<()> {
    let Some(source_assets) = find_source_assets() else {
        println!("honk300: no source Assets directory found; installed asset folders were created empty.");
        create_empty_asset_tree(dest_assets)?;
        return Ok(());
    };
    copy_dir_recursive(&source_assets, dest_assets)
}

fn find_source_assets() -> Option<PathBuf> {
    let cwd_assets = std::env::current_dir().ok()?.join("Assets");
    if cwd_assets.exists() {
        return Some(cwd_assets);
    }
    let exe_assets = std::env::current_exe()
        .ok()
        .and_then(|exe| exe.parent().map(|parent| parent.join("Assets")))?;
    exe_assets.exists().then_some(exe_assets)
}

fn create_empty_asset_tree(root: &Path) -> io::Result<()> {
    for subdir in [
        "Images/Memes/originals",
        "Images/Memes/custom",
        "Images/Memes/user",
        "Text/NotepadMessages/originals",
        "Text/NotepadMessages/custom",
        "Text/NotepadMessages/user",
    ] {
        fs::create_dir_all(root.join(subdir))?;
    }
    Ok(())
}

fn copy_dir_recursive(source: &Path, dest: &Path) -> io::Result<()> {
    fs::create_dir_all(dest)?;
    for entry in fs::read_dir(source)? {
        let entry = entry?;
        let source_path = entry.path();
        let dest_path = dest.join(entry.file_name());
        let metadata = entry.metadata()?;
        if metadata.is_dir() {
            copy_dir_recursive(&source_path, &dest_path)?;
        } else if metadata.is_file() {
            if let Some(parent) = dest_path.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::copy(&source_path, &dest_path)?;
        }
    }
    Ok(())
}

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

fn backup_user_content(assets_dir: &Path, backup_root: &Path) -> io::Result<Option<PathBuf>> {
    backup_user_content_at(assets_dir, backup_root, unix_timestamp())
}

fn backup_user_content_at(
    assets_dir: &Path,
    backup_root: &Path,
    timestamp: u64,
) -> io::Result<Option<PathBuf>> {
    copy_user_content_dirs(assets_dir, &backup_root.join(format!("purge-{timestamp}")))
}

/// Non-purge uninstall relocates the user's own memes/notes out of the install tree (which is
/// deleted wholesale) into the backups area, so plain `uninstall` never destroys user-supplied
/// content. Returns the preserved directory, or `None` when the user added nothing of their own.
fn preserve_user_content(assets_dir: &Path, backup_root: &Path) -> io::Result<Option<PathBuf>> {
    preserve_user_content_at(assets_dir, backup_root, unix_timestamp())
}

fn preserve_user_content_at(
    assets_dir: &Path,
    backup_root: &Path,
    timestamp: u64,
) -> io::Result<Option<PathBuf>> {
    copy_user_content_dirs(
        assets_dir,
        &backup_root.join(format!("preserved-{timestamp}")),
    )
}

/// Copy only the user-provenance asset directories (`.../Memes/user`, `.../NotepadMessages/user`)
/// into `dest`, preserving their relative layout. Bundled `originals`/`custom` assets are left
/// out on purpose — only user-supplied content is worth carrying across an uninstall.
fn copy_user_content_dirs(assets_dir: &Path, dest: &Path) -> io::Result<Option<PathBuf>> {
    let mappings = [
        (
            assets_dir.join("Images").join("Memes").join("user"),
            PathBuf::from("Images").join("Memes").join("user"),
        ),
        (
            assets_dir.join("Text").join("NotepadMessages").join("user"),
            PathBuf::from("Text").join("NotepadMessages").join("user"),
        ),
    ];

    let existing: Vec<_> = mappings
        .iter()
        .filter(|(source, _)| source.exists())
        .collect();
    if existing.is_empty() {
        return Ok(None);
    }

    for (source, relative) in existing {
        copy_dir_recursive(source, &dest.join(relative))?;
    }
    Ok(Some(dest.to_path_buf()))
}

fn unix_timestamp() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

fn purge_config_state(root: PathBuf) -> io::Result<()> {
    remove_dir_if_exists(&root)
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

fn write_text_file(path: &Path, text: &str) -> io::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, text)
}

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
    use winreg::enums::HKEY_CURRENT_USER;
    use winreg::RegKey;

    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let key = hkcu.open_subkey("Software\\Honk300").ok()?;
    let value: String = key.get_value("InstallSource").ok()?;
    let source = InstallSource::from_marker(&value);
    (source != InstallSource::Unknown).then_some(source)
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
    let shortcut = windows_start_menu_shortcut_path()?;
    let working_dir = exe.parent().unwrap_or_else(|| Path::new(""));
    let script = format!(
        "$w = New-Object -ComObject WScript.Shell; $s = $w.CreateShortcut('{}'); $s.TargetPath = '{}'; $s.Arguments = 'start'; $s.WorkingDirectory = '{}'; $s.Save()",
        ps_quote(&shortcut.to_string_lossy()),
        ps_quote(&exe.to_string_lossy()),
        ps_quote(&working_dir.to_string_lossy())
    );
    let status = std::process::Command::new("powershell")
        .args([
            "-NoProfile",
            "-ExecutionPolicy",
            "Bypass",
            "-Command",
            &script,
        ])
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

#[cfg(target_os = "linux")]
fn linux_user_install_root() -> Result<PathBuf, DynError> {
    Ok(xdg_data_home()?.join(APP_NAME).join("install"))
}

#[cfg(target_os = "linux")]
fn linux_config_state_root() -> Result<PathBuf, DynError> {
    Ok(xdg_data_home()?.join(APP_NAME))
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
        "[Desktop Entry]\nType=Application\nName=Honk300\nComment=Desktop goose for your screen\nExec={} start\nTerminal=false\nCategories=Utility;\nStartupNotify=false\nX-GNOME-Autostart-enabled=true\n",
        desktop_exec_quote(exe)
    )
}

#[cfg(any(target_os = "linux", target_os = "macos"))]
fn replace_unix_symlink(link: &Path, target: &Path) -> io::Result<()> {
    remove_file_if_exists(link)?;
    std::os::unix::fs::symlink(target, link)
}

#[cfg(any(target_os = "linux", target_os = "macos"))]
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
fn macos_backup_root() -> Result<PathBuf, DynError> {
    Ok(home_dir()?
        .join("Library")
        .join("Application Support")
        .join("honk300-backups"))
}

/// The `*.app` directory that contains the running executable, when there is one. Used by `install`
/// to copy an already-staged bundle into `~/Applications`.
#[cfg(target_os = "macos")]
fn current_exe_app_bundle() -> Option<PathBuf> {
    let exe = std::env::current_exe().ok()?;
    exe.ancestors()
        .find(|ancestor| ancestor.extension().and_then(|ext| ext.to_str()) == Some("app"))
        .map(Path::to_path_buf)
}

/// The LSUIElement agent `Info.plist`, kept 1:1 with `script/package_macos_app.sh` (bundle id,
/// executable name, LSUIElement, high-res) with the version stamped from the running build.
#[cfg(target_os = "macos")]
fn macos_info_plist(version: &str) -> String {
    let version = xml_escape(version);
    format!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n\
<!DOCTYPE plist PUBLIC \"-//Apple//DTD PLIST 1.0//EN\"\n  \"http://www.apple.com/DTDs/PropertyList-1.0.dtd\">\n\
<plist version=\"1.0\">\n\
<dict>\n\
  <key>CFBundleDevelopmentRegion</key>\n  <string>en</string>\n\
  <key>CFBundleDisplayName</key>\n  <string>Honk300</string>\n\
  <key>CFBundleExecutable</key>\n  <string>honk300</string>\n\
  <key>CFBundleIdentifier</key>\n  <string>dev.emmetts.honk300</string>\n\
  <key>CFBundleInfoDictionaryVersion</key>\n  <string>6.0</string>\n\
  <key>CFBundleName</key>\n  <string>Honk300</string>\n\
  <key>CFBundlePackageType</key>\n  <string>APPL</string>\n\
  <key>CFBundleShortVersionString</key>\n  <string>{version}</string>\n\
  <key>CFBundleVersion</key>\n  <string>{version}</string>\n\
  <key>LSMinimumSystemVersion</key>\n  <string>11.0</string>\n\
  <key>LSUIElement</key>\n  <true/>\n\
  <key>NSHighResolutionCapable</key>\n  <true/>\n\
</dict>\n\
</plist>\n"
    )
}

/// The login LaunchAgent that runs the bundle binary with `start` at login (`--autostart`).
#[cfg(target_os = "macos")]
fn macos_launch_agent_plist(program: &Path) -> String {
    let program = xml_escape(&program.to_string_lossy());
    format!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n\
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
        let assets = root.join("Assets");
        let memes = assets.join("Images").join("Memes").join("user");
        let notes = assets.join("Text").join("NotepadMessages").join("user");
        fs::create_dir_all(&memes).unwrap();
        fs::create_dir_all(&notes).unwrap();
        fs::write(memes.join("mine.png"), b"png").unwrap();
        fs::write(notes.join("mine.txt"), b"note").unwrap();

        let backup = backup_user_content_at(&assets, &root.join("backups"), 123)
            .unwrap()
            .unwrap();

        assert!(backup.join("Images/Memes/user/mine.png").exists());
        assert!(backup.join("Text/NotepadMessages/user/mine.txt").exists());
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn purge_backup_reports_none_without_user_content() {
        let root = test_dir("empty-backup");
        let assets = root.join("Assets");
        fs::create_dir_all(&assets).unwrap();
        assert_eq!(
            backup_user_content_at(&assets, &root.join("backups"), 123)
                .unwrap()
                .as_ref(),
            None
        );
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn non_purge_uninstall_preserves_user_memes_and_notes() {
        let root = test_dir("preserve");
        let assets = root.join("Assets");
        let memes = assets.join("Images").join("Memes").join("user");
        let notes = assets.join("Text").join("NotepadMessages").join("user");
        fs::create_dir_all(&memes).unwrap();
        fs::create_dir_all(&notes).unwrap();
        fs::write(memes.join("mine.png"), b"png").unwrap();
        fs::write(notes.join("mine.txt"), b"note").unwrap();

        let preserved = preserve_user_content_at(&assets, &root.join("backups"), 123)
            .unwrap()
            .unwrap();

        // Preserved under a distinct `preserved-*` dir (not the purge backup namespace).
        assert!(preserved.ends_with("preserved-123"));
        assert!(preserved.join("Images/Memes/user/mine.png").exists());
        assert!(preserved
            .join("Text/NotepadMessages/user/mine.txt")
            .exists());
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn non_purge_uninstall_reports_none_without_user_content() {
        let root = test_dir("empty-preserve");
        let assets = root.join("Assets");
        fs::create_dir_all(&assets).unwrap();
        assert_eq!(
            preserve_user_content_at(&assets, &root.join("backups"), 123)
                .unwrap()
                .as_ref(),
            None
        );
        let _ = fs::remove_dir_all(root);
    }

    fn test_dir(name: &str) -> PathBuf {
        std::env::temp_dir().join(format!(
            "honk300-install-{name}-{}-{}",
            std::process::id(),
            unix_timestamp()
        ))
    }
}
