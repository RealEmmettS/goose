//! Identity of the separate native settings payload in the existing owned install.
use super::*;
use std::fs::File;

pub(crate) const SETTINGS_NAME: &str = if cfg!(windows) {
    "honk300-settings.exe"
} else {
    "honk300-settings"
};

#[cfg(windows)]
pub(crate) const ACCESSIBILITY_NAME: &str = "honk_settings_accessibility.dll";

#[cfg(windows)]
pub(super) fn current_settings_hash() -> Result<String, DynError> {
    let path = std::env::current_exe()?.with_file_name(SETTINGS_NAME);
    Ok(crate::update::compute_sha256_for_install(&path)?)
}

#[cfg(windows)]
pub(super) fn current_accessibility_hash() -> Result<String, DynError> {
    Ok(crate::update::compute_sha256_for_install(
        &std::env::current_exe()?.with_file_name(ACCESSIBILITY_NAME),
    )?)
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
        #[cfg(windows)]
        ACCESSIBILITY_NAME,
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
pub(crate) fn verify_settings_companion(
    current: &Path,
    settings: &Path,
) -> Result<Vec<File>, DynError> {
    verify_settings_with_candidates(
        current,
        settings,
        current_owned_receipt_candidates(current),
        external_receipt_candidates(),
        detect_install_source(),
    )
}

fn verify_settings_with_candidates(
    current: &Path,
    settings: &Path,
    owned: Vec<PathBuf>,
    external: Vec<PathBuf>,
    fallback: InstallSource,
) -> Result<Vec<File>, DynError> {
    let files = open_settings_files(settings)?;
    let mut candidates = owned;
    for path in external {
        if candidates.contains(&path) {
            continue;
        }
        // A healthy external receipt for another installation must not turn a
        // separate source/portable launch into that installation. Damaged evidence
        // remains an error; only a fully valid foreign receipt is ignored.
        let regular = fs::symlink_metadata(&path)
            .is_ok_and(|metadata| metadata.is_file() && !metadata.file_type().is_symlink());
        let foreign = regular
            && fs::read(&path)
                .ok()
                .and_then(|bytes| serde_json::from_slice::<serde_json::Value>(&bytes).ok())
                .is_some_and(|receipt| {
                    receipt
                        .get("install_root")
                        .and_then(serde_json::Value::as_str)
                        .map(Path::new)
                        .is_some_and(|root| {
                            !path_is_within(current, root)
                                && validated_receipt_source(&receipt, &root.join("honk300"))
                                    .is_some()
                        })
                });
        if !foreign {
            candidates.push(path);
        }
    }
    let source = match install_receipt_source_from_candidates(&candidates, current) {
        InstallSourceEvidence::InvalidOrConflicting => {
            return Err("settings install receipt is invalid or conflicting".into())
        }
        InstallSourceEvidence::Valid(source) => source,
        InstallSourceEvidence::Missing => fallback,
    };
    if matches!(source, InstallSource::Unknown | InstallSource::ManualLocal) {
        return Ok(files);
    }
    for path in candidates {
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
        verify_files(&receipt, &files, settings)?;
        return Ok(files);
    }
    Err("cannot verify the settings companion against its owned installation receipt".into())
}

/// Linux exec resolves this retained descriptor before closing CLOEXEC handles.
/// Pathname replacement cannot redirect the launch to an unverified inode.
#[cfg(target_os = "linux")]
pub(crate) fn verified_settings_program(files: &[File]) -> io::Result<PathBuf> {
    use std::os::fd::AsRawFd;
    let executable = files
        .first()
        .ok_or_else(|| io::Error::other("verified settings executable is missing"))?;
    Ok(PathBuf::from(format!(
        "/proc/self/fd/{}",
        executable.as_raw_fd()
    )))
}

fn open_verified_file(settings: &Path) -> Result<File, DynError> {
    let metadata = fs::symlink_metadata(settings)?;
    if !metadata.is_file() || metadata.file_type().is_symlink() {
        return Err("settings companion is not a regular sibling executable".into());
    }
    let mut options = fs::OpenOptions::new();
    options.read(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.custom_flags(libc::O_NOFOLLOW);
    }
    #[cfg(windows)]
    {
        use std::os::windows::fs::OpenOptionsExt;
        options.share_mode(1); // FILE_SHARE_READ; no write or delete sharing.
    }
    Ok(options.open(settings)?)
}

fn open_settings_files(settings: &Path) -> Result<Vec<File>, DynError> {
    Ok(vec![
        open_verified_file(settings)?,
        #[cfg(windows)]
        open_verified_file(&settings.with_file_name(ACCESSIBILITY_NAME))?,
    ])
}

fn verify_files(
    receipt: &serde_json::Value,
    files: &[File],
    settings: &Path,
) -> Result<(), DynError> {
    verify_identity(&receipt["settings_app"], &files[0], settings)?;
    #[cfg(windows)]
    verify_identity(
        &receipt["settings_app"]["accessibility"],
        &files[1],
        &settings.with_file_name(ACCESSIBILITY_NAME),
    )?;
    Ok(())
}

fn verify_identity(
    identity: &serde_json::Value,
    file: &File,
    settings: &Path,
) -> Result<(), DynError> {
    use sha2::{Digest, Sha256};
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
    verify_files(receipt, &open_settings_files(settings)?, settings)
}

#[cfg(test)]
mod tests {
    use super::*;
    use sha2::{Digest, Sha256};

    #[test]
    fn invalid_receipts_cannot_downgrade_companion_verification_to_unmanaged() {
        let directory = tempfile::tempdir().unwrap();
        let current = directory.path().join("honk300");
        let settings = directory.path().join(SETTINGS_NAME);
        fs::write(&settings, b"settings").unwrap();
        #[cfg(windows)]
        fs::write(settings.with_file_name(ACCESSIBILITY_NAME), b"bridge").unwrap();
        let receipt = directory.path().join("install-receipt.json");
        for fallback in [InstallSource::Unknown, InstallSource::ManualLocal] {
            assert!(
                verify_settings_with_candidates(&current, &settings, vec![], vec![], fallback)
                    .is_ok()
            );
            for bytes in [
                b"invalid receipt".as_slice(),
                br#"{"schema":"wrong","install_root":"/other"}"#,
            ] {
                fs::write(&receipt, bytes).unwrap();
                assert!(verify_settings_with_candidates(
                    &current,
                    &settings,
                    vec![receipt.clone()],
                    vec![],
                    fallback
                )
                .is_err());
                assert!(verify_settings_with_candidates(
                    &current,
                    &settings,
                    vec![],
                    vec![receipt.clone()],
                    fallback
                )
                .is_err());
            }
        }
        let foreign = directory.path().join("other-install");
        fs::write(
            &receipt,
            serde_json::to_vec(&serde_json::json!({
                "schema": OWNERSHIP_MARKER, "install_root": foreign, "channel": "shell",
            }))
            .unwrap(),
        )
        .unwrap();
        assert!(verify_settings_with_candidates(
            &current,
            &settings,
            vec![],
            vec![receipt],
            InstallSource::Unknown
        )
        .is_ok());
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn verified_linux_exec_ignores_a_replaced_sibling_path() {
        use std::os::unix::process::CommandExt;
        let directory = tempfile::tempdir().unwrap();
        let settings = directory.path().join(SETTINGS_NAME);
        fs::copy(std::env::current_exe().unwrap(), &settings).unwrap();
        let identity = serde_json::json!({ "name": SETTINGS_NAME,
            "size": fs::metadata(&settings).unwrap().len(),
            "sha256": format!("{:x}", Sha256::digest(fs::read(&settings).unwrap())),
        });
        let files = open_settings_files(&settings).unwrap();
        verify_identity(&identity, &files[0], &settings).unwrap();
        fs::rename(&settings, directory.path().join("verified-original")).unwrap();
        fs::write(&settings, b"unverified replacement").unwrap();
        let status = std::process::Command::new(verified_settings_program(&files).unwrap())
            .arg0(&settings)
            .arg("--list")
            .stdout(std::process::Stdio::null())
            .status()
            .unwrap();
        assert!(status.success());
        assert_eq!(fs::read(&settings).unwrap(), b"unverified replacement");
    }

    #[test]
    fn receipt_rejects_missing_swapped_and_changed_settings_bytes() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join(SETTINGS_NAME);
        fs::write(&path, b"verified-settings").unwrap();
        let receipt = serde_json::json!({
            "name": SETTINGS_NAME, "size": 17,
            "sha256": format!("{:x}", Sha256::digest(b"verified-settings"))
        });
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

    #[cfg(windows)]
    #[test]
    fn bridge_identity_and_both_windows_file_leases_are_required() {
        let directory = tempfile::tempdir().unwrap();
        let settings = directory.path().join(SETTINGS_NAME);
        let bridge = directory.path().join(ACCESSIBILITY_NAME);
        fs::write(&settings, b"settings").unwrap();
        assert!(open_settings_files(&settings).is_err());
        fs::write(&bridge, b"bridge").unwrap();
        let receipt = serde_json::json!({"settings_app": {
            "name": SETTINGS_NAME, "size": 8,
            "sha256": format!("{:x}", Sha256::digest(b"settings")),
            "accessibility": {"name": ACCESSIBILITY_NAME, "size": 6,
                "sha256": format!("{:x}", Sha256::digest(b"bridge"))}
        }});
        let files = open_settings_files(&settings).unwrap();
        assert!(verify_files(&receipt, &files, &settings).is_ok());
        for path in [&settings, &bridge] {
            assert!(fs::write(path, b"tamper").is_err());
            assert!(fs::rename(path, path.with_extension("replaced")).is_err());
        }
        drop(files);
        fs::write(&bridge, b"tamper").unwrap();
        assert!(verify_files(
            &receipt,
            &open_settings_files(&settings).unwrap(),
            &settings
        )
        .is_err());
        fs::write(&bridge, b"bridge").unwrap();
        let mut incomplete = receipt;
        incomplete["settings_app"]["accessibility"] = serde_json::Value::Null;
        assert!(verify_files(
            &incomplete,
            &open_settings_files(&settings).unwrap(),
            &settings
        )
        .is_err());
    }
}
