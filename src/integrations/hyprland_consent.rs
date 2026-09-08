//! Separate, private consent for read-only Hyprland observations.
use serde::{Deserialize, Serialize};
use std::{
    fs,
    io::{self, Read, Write},
    path::Path,
};

const RECORD: &str = "hyprland.json";
const BOUNDARY: &str = "hyprland-observe-1";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Consent {
    boundary: String,
    nonce: String,
}

impl Consent {
    pub(super) fn current(&self) -> bool {
        self.boundary == BOUNDARY
    }
}

fn invalid(message: &str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message)
}

pub(super) fn read(directory: &Path) -> io::Result<Option<Consent>> {
    match super::installed::check(directory, true) {
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(None),
        result => result?,
    }
    let path = directory.join(RECORD);
    match super::installed::check(&path, false) {
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(None),
        result => result?,
    }
    let mut options = fs::OpenOptions::new();
    options.read(true);
    #[cfg(target_os = "linux")]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.custom_flags(libc::O_NOFOLLOW | libc::O_CLOEXEC);
    }
    let mut bytes = Vec::new();
    options.open(&path)?.take(1025).read_to_end(&mut bytes)?;
    if bytes.len() > 1024 {
        return Err(invalid("Hyprland consent exceeds its bound"));
    }
    let consent: Consent =
        serde_json::from_slice(&bytes).map_err(|_| invalid("Invalid Hyprland consent record"))?;
    if consent.boundary.len() > 64
        || consent.nonce.len() != 32
        || !consent.nonce.bytes().all(|byte| byte.is_ascii_hexdigit())
    {
        return Err(invalid("Invalid Hyprland consent identity"));
    }
    Ok(Some(consent))
}

pub(super) fn install(directory: &Path, nonce: &str) -> io::Result<Consent> {
    if nonce.len() != 32 || !nonce.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Err(invalid("Invalid Hyprland setup identity"));
    }
    let _lock = super::installed::lock_record(directory, ".hyprland-lock")?;
    if let Some(existing) = read(directory)? {
        if existing.current() {
            return Ok(existing);
        }
    }
    let consent = Consent {
        boundary: BOUNDARY.into(),
        nonce: nonce.into(),
    };
    let temporary = directory.join(format!(".hyprland-{nonce}.new"));
    let mut created = false;
    let result = (|| {
        let mut file = super::installed::private_file(&temporary, true)?;
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
    let _lock = super::installed::lock_record(directory, ".hyprland-lock")?;
    if read(directory)?.is_some() {
        fs::remove_file(directory.join(RECORD))?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn hyprland_revocation_preserves_kde_and_unknown_records() {
        let parent = tempfile::tempdir().unwrap();
        let directory = parent.path().join("wayland");
        let nonce = "0123456789abcdef0123456789abcdef";
        let kde = super::super::installed::install(&directory, nonce).unwrap();
        let first = install(&directory, nonce).unwrap();
        assert_eq!(
            install(&directory, "fedcba9876543210fedcba9876543210").unwrap(),
            first
        );
        remove(&directory).unwrap();
        assert!(read(&directory).unwrap().is_none());
        assert_eq!(
            super::super::installed::read(&directory).unwrap(),
            Some(kde)
        );
        let second = install(&directory, "fedcba9876543210fedcba9876543210").unwrap();
        assert_ne!(first, second);
        fs::write(directory.join(RECORD), b"foreign").unwrap();
        assert!(install(&directory, nonce).is_err());
        assert!(remove(&directory).is_err());
        assert_eq!(fs::read(directory.join(RECORD)).unwrap(), b"foreign");
    }
}
