//! Mutable user-media roots, migration, preservation, and rollback.
use super::*;

#[cfg(any(test, windows))]
pub(super) fn windows_media_root_from(local_app_data: &Path) -> PathBuf {
    local_app_data.join(APP_NAME).join("media")
}

#[cfg(any(test, target_os = "macos"))]
pub(super) fn macos_media_root_from(home: &Path) -> PathBuf {
    home.join("Library")
        .join("Application Support")
        .join(APP_NAME)
        .join("media")
}

#[cfg(any(test, target_os = "linux"))]
pub(super) fn linux_media_root_from(xdg_data_home: Option<&Path>, home: &Path) -> PathBuf {
    xdg_data_home
        .map(Path::to_path_buf)
        .unwrap_or_else(|| home.join(".local").join("share"))
        .join(APP_NAME)
        .join("media")
}

pub(super) fn ensure_external_media_root(media_root: &Path) -> io::Result<()> {
    ensure_real_directory(media_root)?;
    ensure_real_directory(&media_root.join("Memes"))?;
    ensure_real_directory(&media_root.join("Notes"))
}

fn ensure_real_directory_tracked(path: &Path, created: &mut Vec<PathBuf>) -> io::Result<()> {
    match fs::symlink_metadata(path) {
        Ok(_) => validate_real_directory(path),
        Err(error) if error.kind() == io::ErrorKind::NotFound => {
            let parent = path
                .parent()
                .ok_or_else(|| io::Error::other("tracked directory has no parent"))?;
            ensure_real_directory_tracked(parent, created)?;
            fs::create_dir(path)?;
            created.push(path.to_path_buf());
            Ok(())
        }
        Err(error) => Err(error),
    }
}

fn ensure_external_media_root_tracked(
    media_root: &Path,
    created: &mut Vec<PathBuf>,
) -> io::Result<()> {
    ensure_real_directory_tracked(media_root, created)?;
    ensure_real_directory_tracked(&media_root.join("Memes"), created)?;
    ensure_real_directory_tracked(&media_root.join("Notes"), created)
}

fn ensure_media_destination_parent(
    media_root: &Path,
    destination: &Path,
) -> io::Result<Vec<PathBuf>> {
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
    let mut created = Vec::new();
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
        match fs::symlink_metadata(&current) {
            Ok(_) => validate_real_directory(&current)?,
            Err(error) if error.kind() == io::ErrorKind::NotFound => {
                fs::create_dir(&current)?;
                created.push(current.clone());
            }
            Err(error) => return Err(error),
        }
    }
    Ok(created)
}

#[derive(Debug, Default)]
pub(super) struct MediaMigrationChanges {
    pub(super) created_files: Vec<(PathBuf, PathBuf)>,
    pub(super) created_dirs: Vec<PathBuf>,
}

impl MediaMigrationChanges {
    pub(super) fn rollback(&mut self) -> io::Result<()> {
        let mut failures = Vec::new();
        while let Some((source, destination)) = self.created_files.pop() {
            let result = match fs::symlink_metadata(&destination) {
                Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(()),
                Err(error) => Err(error),
                Ok(metadata)
                    if metadata.is_file()
                        && !metadata.file_type().is_symlink()
                        && regular_files_equal(&source, &destination).unwrap_or(false) =>
                {
                    fs::remove_file(&destination)
                }
                Ok(_) => Err(io::Error::other(
                    "created media changed before rollback; preserving it",
                )),
            };
            if let Err(error) = result {
                failures.push(format!("{}: {error}", destination.display()));
            }
        }
        while let Some(directory) = self.created_dirs.pop() {
            match fs::remove_dir(&directory) {
                Ok(()) => {}
                Err(error)
                    if matches!(
                        error.kind(),
                        io::ErrorKind::NotFound | io::ErrorKind::DirectoryNotEmpty
                    ) => {}
                Err(error) => failures.push(format!("{}: {error}", directory.display())),
            }
        }
        if failures.is_empty() {
            Ok(())
        } else {
            Err(io::Error::other(failures.join("; ")))
        }
    }
}

fn media_migration_error(changes: &mut MediaMigrationChanges, error: io::Error) -> io::Error {
    match changes.rollback() {
        Ok(()) => error,
        Err(rollback) => io::Error::other(format!(
            "{error}; additionally failed to roll back migrated media: {rollback}"
        )),
    }
}

pub(super) fn migrate_legacy_user_media(
    legacy_assets: &Path,
    media_root: &Path,
    mode: LegacyMigrationMode,
) -> io::Result<MediaMigrationChanges> {
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
    let mut pending = Vec::new();
    let mut changes = MediaMigrationChanges::default();
    if let Err(error) = ensure_external_media_root_tracked(media_root, &mut changes.created_dirs) {
        return Err(media_migration_error(&mut changes, error));
    }
    for (source, destination) in &files {
        match ensure_media_destination_parent(media_root, destination) {
            Ok(created) => changes.created_dirs.extend(created),
            Err(error) => {
                return Err(media_migration_error(&mut changes, error));
            }
        }
        match fs::symlink_metadata(destination) {
            Ok(metadata) if metadata.is_file() && !metadata.file_type().is_symlink() => {
                let equal = match regular_files_equal(source, destination) {
                    Ok(equal) => equal,
                    Err(error) => return Err(media_migration_error(&mut changes, error)),
                };
                if !equal {
                    let error = io::Error::new(
                        io::ErrorKind::AlreadyExists,
                        format!(
                            "refusing to overwrite existing external media {}",
                            destination.display()
                        ),
                    );
                    return Err(media_migration_error(&mut changes, error));
                }
            }
            Ok(_) => {
                let error = io::Error::new(
                    io::ErrorKind::AlreadyExists,
                    format!(
                        "refusing to overwrite existing external media {}",
                        destination.display()
                    ),
                );
                return Err(media_migration_error(&mut changes, error));
            }
            Err(error) if error.kind() == io::ErrorKind::NotFound => {
                pending.push((source, destination));
            }
            Err(error) => {
                return Err(media_migration_error(&mut changes, error));
            }
        }
    }
    for (source, destination) in pending {
        if let Err(error) = fs::copy(source, destination) {
            return Err(media_migration_error(&mut changes, error));
        }
        changes
            .created_files
            .push((source.to_path_buf(), destination.to_path_buf()));
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
    Ok(changes)
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

pub(super) fn backup_user_content(
    media_root: &Path,
    backup_root: &Path,
) -> io::Result<Option<PathBuf>> {
    backup_user_content_at(media_root, backup_root, unix_timestamp())
}

pub(super) fn backup_user_content_at(
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

pub(super) fn media_has_user_content(media_root: &Path) -> io::Result<bool> {
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum LegacyMigrationMode {
    #[cfg(any(test, windows, target_os = "linux"))]
    Move,
    #[cfg(any(test, windows, target_os = "macos"))]
    Copy,
}
