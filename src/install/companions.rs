//! Identity of the separate native settings payload in the existing owned install.
use super::*;
use std::fs::File;

pub(crate) const SETTINGS_NAME: &str = if cfg!(windows) {
    "honk300-settings.exe"
} else {
    "honk300-settings"
};

#[cfg(windows)]
pub(super) fn current_settings_hash() -> Result<String, DynError> {
    let path = std::env::current_exe()?.with_file_name(SETTINGS_NAME);
    Ok(crate::update::compute_sha256_for_install(&path)?)
}

/// Source builds may intentionally install only the Rust CLI/TUI. A complete
/// portable distribution also carries its exact settings sibling and notices.
#[cfg(any(windows, target_os = "linux"))]
pub(super) fn copy_settings_if_present(destination: &Path) -> io::Result<()> {
    let source_dir = std::env::current_exe()?
        .parent()
        .ok_or_else(|| io::Error::other("executable has no parent"))?
        .to_owned();
    match fs::symlink_metadata(source_dir.join(SETTINGS_NAME)) {
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(()),
        Err(error) => return Err(error),
        Ok(_) => {}
    }
    for name in [
        SETTINGS_NAME,
        "NATIVE_SDK_LICENSE.txt",
        "NATIVE_SDK_FONT_LICENSE.txt",
    ] {
        let source = source_dir.join(name);
        let metadata = fs::symlink_metadata(&source)?;
        if !metadata.is_file() || metadata.file_type().is_symlink() {
            return Err(io::Error::other(
                "settings payload contains a non-regular file",
            ));
        }
        let target = destination.join(name);
        if !same_file_best_effort(&source, &target) {
            fs::copy(source, &target)?;
        }
        #[cfg(unix)]
        if name == SETTINGS_NAME {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(target, fs::Permissions::from_mode(0o755))?;
        }
    }
    Ok(())
}

/// Keep the verified file open through process creation. On Windows the handle
/// also denies replacement or writes until the new image has been mapped.
pub(crate) fn verify_settings_companion(current: &Path, settings: &Path) -> Result<File, DynError> {
    let metadata = fs::symlink_metadata(settings)?;
    if !metadata.is_file() || metadata.file_type().is_symlink() {
        return Err("settings companion is not a regular sibling executable".into());
    }
    let mut options = fs::OpenOptions::new();
    options.read(true);
    #[cfg(windows)]
    {
        use std::os::windows::fs::OpenOptionsExt;
        options.share_mode(1); // FILE_SHARE_READ; no write or delete sharing.
    }
    let file = options.open(settings)?;
    let source = detect_install_source();
    if matches!(source, InstallSource::Unknown | InstallSource::ManualLocal) {
        return Ok(file);
    }
    for path in current_owned_receipt_candidates(current)
        .into_iter()
        .chain(external_receipt_candidates())
    {
        let metadata = match fs::symlink_metadata(&path) {
            Ok(metadata) => metadata,
            Err(error) if error.kind() == io::ErrorKind::NotFound => continue,
            Err(error) => return Err(error.into()),
        };
        if !metadata.is_file() || metadata.file_type().is_symlink() {
            return Err("settings install receipt is not a regular file".into());
        }
        let receipt: serde_json::Value = serde_json::from_slice(&fs::read(&path)?)?;
        if validated_receipt_source(&receipt, current) != Some(source) {
            continue;
        }
        verify_identity(&receipt, &file, settings)?;
        return Ok(file);
    }
    Err("cannot verify the settings companion against its owned installation receipt".into())
}

fn verify_identity(
    receipt: &serde_json::Value,
    file: &File,
    settings: &Path,
) -> Result<(), DynError> {
    use sha2::{Digest, Sha256};
    let identity = receipt.get("settings_app").ok_or(
        "this install receipt has no settings companion; reinstall the complete current package",
    )?;
    if identity.get("name").and_then(serde_json::Value::as_str)
        != settings.file_name().and_then(|name| name.to_str())
        || identity.get("size").and_then(serde_json::Value::as_u64) != Some(file.metadata()?.len())
    {
        return Err("settings companion does not match its receipt identity".into());
    }
    let mut digest = Sha256::new();
    io::copy(&mut &*file, &mut digest)?;
    let actual = format!("{:x}", digest.finalize());
    if identity.get("sha256").and_then(serde_json::Value::as_str) != Some(actual.as_str()) {
        return Err("settings companion does not match its receipt hash".into());
    }
    Ok(())
}

#[cfg(windows)]
pub(crate) fn verify_receipted_settings(
    receipt: &serde_json::Value,
    settings: &Path,
) -> Result<(), DynError> {
    let metadata = fs::symlink_metadata(settings)?;
    if !metadata.is_file() || metadata.file_type().is_symlink() {
        return Err("settings companion is not a regular file".into());
    }
    verify_identity(receipt, &File::open(settings)?, settings)
}

#[cfg(test)]
mod tests {
    use super::*;
    use sha2::{Digest, Sha256};

    #[test]
    fn receipt_rejects_missing_swapped_and_changed_settings_bytes() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join(SETTINGS_NAME);
        fs::write(&path, b"verified-settings").unwrap();
        let receipt = serde_json::json!({"settings_app": {
            "name": SETTINGS_NAME, "size": 17,
            "sha256": format!("{:x}", Sha256::digest(b"verified-settings"))
        }});
        assert!(verify_identity(&receipt, &File::open(&path).unwrap(), &path).is_ok());
        assert!(
            verify_identity(&serde_json::json!({}), &File::open(&path).unwrap(), &path).is_err()
        );
        assert!(verify_identity(
            &receipt,
            &File::open(&path).unwrap(),
            &path.with_file_name("other")
        )
        .is_err());
        fs::write(&path, b"modified-settings").unwrap();
        assert!(verify_identity(&receipt, &File::open(&path).unwrap(), &path).is_err());
    }
}
