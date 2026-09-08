//! A private, bounded consent record containing the exact installed companion.
use serde::{Deserialize, Serialize};
use std::{
    fs::{self, File, OpenOptions},
    io::{self, Read, Write},
    path::Path,
};

const RECORD: &str = "kwin.json";
const MAX_RECORD: u64 = 131_072;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Consent {
    protocol: u8,
    nonce: String,
    script: String,
}

impl Consent {
    pub(super) fn current(&self) -> bool {
        self.script.as_bytes() == super::SCRIPT
    }
    pub(super) fn name(&self) -> String {
        format!("honk300-{}", self.nonce)
    }
}

fn invalid(message: &str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message)
}

fn check(path: &Path, directory: bool) -> io::Result<()> {
    let metadata = fs::symlink_metadata(path)?;
    if metadata.file_type().is_symlink()
        || metadata.is_dir() != directory
        || (!directory && !metadata.is_file())
    {
        return Err(invalid(
            "KDE setup path is not an owned regular file or directory",
        ));
    }
    #[cfg(target_os = "linux")]
    {
        use std::os::unix::fs::MetadataExt;
        if metadata.uid() != unsafe { libc::geteuid() } || metadata.mode() & 0o077 != 0 {
            return Err(invalid("KDE setup must be private to the current user"));
        }
    }
    Ok(())
}

pub(super) fn read(directory: &Path) -> io::Result<Option<Consent>> {
    match check(directory, true) {
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(None),
        result => result?,
    }
    let path = directory.join(RECORD);
    match check(&path, false) {
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(None),
        result => result?,
    }
    let mut bytes = Vec::new();
    File::open(path)?
        .take(MAX_RECORD + 1)
        .read_to_end(&mut bytes)?;
    if bytes.len() as u64 > MAX_RECORD {
        return Err(invalid("KDE setup record exceeds its bound"));
    }
    let consent: Consent =
        serde_json::from_slice(&bytes).map_err(|_| invalid("KDE setup record is invalid"))?;
    if consent.protocol != 1
        || consent.nonce.len() != 32
        || !consent.nonce.bytes().all(|b| b.is_ascii_hexdigit())
    {
        return Err(invalid("KDE setup identity is invalid"));
    }
    Ok(Some(consent))
}

fn private_file(path: &Path, exclusive: bool) -> io::Result<File> {
    let mut options = OpenOptions::new();
    options.read(true).write(true);
    if exclusive {
        options.create_new(true);
    } else {
        options.create(true);
    }
    #[cfg(target_os = "linux")]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options
            .mode(0o600)
            .custom_flags(libc::O_NOFOLLOW | libc::O_CLOEXEC);
    }
    let file = options.open(path)?;
    check(path, false)?;
    Ok(file)
}

fn lock(directory: &Path) -> io::Result<File> {
    let mut builder = fs::DirBuilder::new();
    builder.recursive(true);
    #[cfg(target_os = "linux")]
    {
        use std::os::unix::fs::DirBuilderExt;
        builder.mode(0o700);
    }
    builder.create(directory)?;
    check(directory, true)?;
    let file = private_file(&directory.join(".kwin-lock"), false)?;
    file.try_lock().map_err(|_| {
        io::Error::new(
            io::ErrorKind::WouldBlock,
            "KDE setup is being changed elsewhere",
        )
    })?;
    Ok(file)
}

pub(super) fn install(directory: &Path, nonce: &str) -> io::Result<Consent> {
    if nonce.len() != 32 || !nonce.bytes().all(|b| b.is_ascii_hexdigit()) {
        return Err(invalid("Invalid setup identity"));
    }
    let _lock = lock(directory)?;
    let existing = read(directory)?;
    if let Some(existing) = &existing {
        if existing.current() {
            return Ok(existing.clone());
        }
    }
    let consent = Consent {
        protocol: 1,
        nonce: existing.map_or_else(|| nonce.to_owned(), |consent| consent.nonce),
        script: std::str::from_utf8(super::SCRIPT)
            .map_err(|_| invalid("Invalid bundled companion"))?
            .to_owned(),
    };
    let temporary = directory.join(format!(".kwin-{nonce}.new"));
    let mut created = false;
    let result = (|| {
        let mut file = private_file(&temporary, true)?;
        created = true;
        file.write_all(&serde_json::to_vec(&consent).map_err(io::Error::other)?)?;
        file.sync_all()?;
        drop(file);
        fs::rename(&temporary, directory.join(RECORD))
    })();
    if result.is_err() && created {
        let _ = fs::remove_file(&temporary);
    }
    result?;
    Ok(consent)
}

pub(super) fn remove(directory: &Path) -> io::Result<()> {
    if read(directory)?.is_none() {
        return Ok(());
    }
    let _lock = lock(directory)?;
    if read(directory)?.is_some() {
        fs::remove_file(directory.join(RECORD))?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn explicit_setup_is_idempotent_bounded_and_removes_only_its_record() {
        let parent = tempfile::tempdir().unwrap();
        let root = parent.path().join("wayland");
        assert!(read(&root).unwrap().is_none());
        let consent = install(&root, "0123456789abcdef0123456789abcdef").unwrap();
        assert!(consent.current());
        assert!(consent.name().starts_with("honk300-"));
        assert_eq!(
            install(&root, "fedcba9876543210fedcba9876543210").unwrap(),
            consent
        );
        fs::write(root.join("foreign.txt"), "keep").unwrap();
        remove(&root).unwrap();
        assert!(read(&root).unwrap().is_none());
        assert_eq!(
            fs::read_to_string(root.join("foreign.txt")).unwrap(),
            "keep"
        );
        remove(&root).unwrap();
    }

    #[test]
    fn edited_or_unknown_setup_never_silently_grants_a_changed_companion() {
        let parent = tempfile::tempdir().unwrap();
        let root = parent.path().join("wayland");
        install(&root, "0123456789abcdef0123456789abcdef").unwrap();
        let mut consent = read(&root).unwrap().unwrap();
        consent.script.push_str(" changed");
        fs::write(root.join(RECORD), serde_json::to_vec(&consent).unwrap()).unwrap();
        assert!(!read(&root).unwrap().unwrap().current());
        let updated = install(&root, "fedcba9876543210fedcba9876543210").unwrap();
        assert!(updated.current());
        assert_eq!(
            updated.name(),
            consent.name(),
            "Retain exact cleanup ownership across an approved update"
        );
        fs::write(root.join(RECORD), b"unrelated data").unwrap();
        assert!(install(&root, "0123456789abcdef0123456789abcdef").is_err());
        assert!(remove(&root).is_err());
        assert_eq!(fs::read(root.join(RECORD)).unwrap(), b"unrelated data");
    }

    #[test]
    fn staging_collision_preserves_an_existing_file() {
        let parent = tempfile::tempdir().unwrap();
        let root = parent.path().join("wayland");
        drop(lock(&root).unwrap());
        let nonce = "0123456789abcdef0123456789abcdef";
        let staging = root.join(format!(".kwin-{nonce}.new"));
        fs::write(&staging, b"existing operation").unwrap();
        assert!(install(&root, nonce).is_err());
        assert_eq!(fs::read(&staging).unwrap(), b"existing operation");
        assert!(read(&root).unwrap().is_none());
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn nonprivate_and_linked_permission_records_never_grant_access() {
        use std::os::unix::fs::{symlink, PermissionsExt};
        let parent = tempfile::tempdir().unwrap();
        let root = parent.path().join("wayland");
        install(&root, "0123456789abcdef0123456789abcdef").unwrap();
        let record = root.join(RECORD);
        fs::set_permissions(&record, fs::Permissions::from_mode(0o644)).unwrap();
        assert!(read(&root).is_err());
        assert!(remove(&root).is_err());
        fs::set_permissions(&record, fs::Permissions::from_mode(0o600)).unwrap();
        let saved = root.join("saved.json");
        fs::rename(&record, &saved).unwrap();
        symlink(&saved, &record).unwrap();
        assert!(read(&root).is_err());
        assert!(remove(&root).is_err());
        assert!(saved.is_file());
    }
}
