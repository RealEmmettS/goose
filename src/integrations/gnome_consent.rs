//! Exact private Shell companion files and independent, revocable caller consent.
use super::installed;
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};

pub(super) const UUID: &str = "honk300@emmetts.dev";
const RECORD: &str = "gnome.json";
const BOUNDARY: &str = "gnome-observe-1";
const SCRIPT_TEMPLATE: &str = include_str!("../../integrations/gnome/extension.js");
const METADATA: &[u8] = include_bytes!("../../integrations/gnome/metadata.json");
const MAX_RECORD: u64 = 262_144;
struct Bundle {
    script: String,
    identity: String,
}
fn bundle() -> &'static Bundle {
    static BUNDLE: std::sync::OnceLock<Bundle> = std::sync::OnceLock::new();
    BUNDLE.get_or_init(|| {
        use sha2::{Digest, Sha256};
        let mut digest = Sha256::new();
        digest.update(SCRIPT_TEMPLATE.as_bytes());
        digest.update(METADATA);
        let identity = format!("{:x}", digest.finalize());
        Bundle {
            script: SCRIPT_TEMPLATE.replace("@@HONK300_GNOME_BUILD@@", &identity),
            identity,
        }
    })
}
pub(super) fn build_identity() -> String {
    bundle().identity.clone()
}

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Executable {
    path: String,
    device: String,
    inode: String,
    size: String,
}

impl Executable {
    fn identify(path: &Path) -> io::Result<Self> {
        let canonical = fs::canonicalize(path)?;
        let file = fs::File::open(&canonical)?;
        let metadata = file.metadata()?;
        if !metadata.is_file() {
            return Err(invalid("GNOME caller must be a regular executable"));
        }
        #[cfg(target_os = "linux")]
        let (device, inode) = {
            use std::os::unix::fs::MetadataExt;
            if (metadata.uid() != 0 && metadata.uid() != unsafe { libc::geteuid() })
                || metadata.mode() & 0o022 != 0
            {
                return Err(invalid("GNOME caller executable has unsafe ownership"));
            }
            (metadata.dev(), metadata.ino())
        };
        #[cfg(not(target_os = "linux"))]
        let (device, inode) = (0_u64, 0_u64);
        Ok(Self {
            path: canonical
                .to_str()
                .ok_or_else(|| invalid("GNOME executable path is not UTF-8"))?
                .into(),
            device: device.to_string(),
            inode: inode.to_string(),
            size: metadata.len().to_string(),
        })
    }
}

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Consent {
    phase: Phase,
    previous: Option<Files>,
    boundary: String,
    pub(super) nonce: String,
    script: String,
    metadata: String,
    executable: Executable,
}

#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum Phase {
    Pending,
    Active,
    Revoking,
}

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Files {
    script: String,
    metadata: String,
}

fn invalid(message: &str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message)
}

impl Consent {
    pub(super) fn current(&self, executable: &Path) -> bool {
        self.phase == Phase::Active && self.previous.is_none() && self.payload_current(executable)
    }
    fn payload_current(&self, executable: &Path) -> bool {
        self.boundary == BOUNDARY
            && self.script.as_bytes() == bundle().script.as_bytes()
            && self.metadata.as_bytes() == METADATA
            && Executable::identify(executable).is_ok_and(|current| current == self.executable)
    }
}

fn extension_path(directory: &Path) -> io::Result<PathBuf> {
    let data = directory
        .parent()
        .and_then(Path::parent)
        .ok_or_else(|| invalid("GNOME user data directory is unavailable"))?;
    Ok(data.join("gnome-shell/extensions").join(UUID))
}

fn read_file(path: &Path, limit: u64) -> io::Result<Vec<u8>> {
    installed::check(path, false)?;
    let mut options = fs::OpenOptions::new();
    options.read(true);
    #[cfg(target_os = "linux")]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.custom_flags(libc::O_NOFOLLOW | libc::O_CLOEXEC);
    }
    let mut bytes = Vec::new();
    options
        .open(path)?
        .take(limit + 1)
        .read_to_end(&mut bytes)?;
    if bytes.len() as u64 > limit {
        return Err(invalid("GNOME companion file exceeds its bound"));
    }
    Ok(bytes)
}

pub(super) fn read(directory: &Path) -> io::Result<Option<Consent>> {
    match installed::check(directory, true) {
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(None),
        result => result?,
    }
    let bytes = match read_file(&directory.join(RECORD), MAX_RECORD) {
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(None),
        result => result?,
    };
    let consent: Consent =
        serde_json::from_slice(&bytes).map_err(|_| invalid("Invalid GNOME consent"))?;
    if consent.boundary.len() > 64
        || consent.nonce.len() != 32
        || !consent.nonce.bytes().all(|byte| byte.is_ascii_hexdigit())
        || consent.script.len() > 65_536
        || consent.metadata.len() > 4096
        || consent
            .previous
            .as_ref()
            .is_some_and(|files| files.script.len() > 65_536 || files.metadata.len() > 4096)
        || (consent.phase == Phase::Active && consent.previous.is_some())
        || consent.executable.path.len() > 4096
        || !Path::new(&consent.executable.path).is_absolute()
        || [
            &consent.executable.device,
            &consent.executable.inode,
            &consent.executable.size,
        ]
        .into_iter()
        .any(|value| value.parse::<u64>().is_err())
    {
        return Err(invalid("Invalid GNOME companion identity"));
    }
    Ok(Some(consent))
}

pub(super) fn files_match(directory: &Path, consent: &Consent) -> io::Result<()> {
    let extension = extension_path(directory)?;
    installed::check(&extension, true)?;
    let mut files = fs::read_dir(&extension)?
        .map(|entry| entry.map(|entry| entry.file_name()))
        .collect::<io::Result<Vec<_>>>()?;
    files.sort();
    if files != ["extension.js", "metadata.json"]
        || read_file(&extension.join("extension.js"), 65_536)? != consent.script.as_bytes()
        || read_file(&extension.join("metadata.json"), 4096)? != consent.metadata.as_bytes()
    {
        return Err(invalid(
            "GNOME companion contains changed or unrelated files",
        ));
    }
    Ok(())
}

pub(super) fn known_files(directory: &Path, consent: &Consent) -> io::Result<()> {
    if consent.phase == Phase::Active {
        return files_match(directory, consent);
    }
    let extension = extension_path(directory)?;
    match installed::check(&extension, true) {
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(()),
        result => result?,
    }
    for entry in fs::read_dir(&extension)? {
        let entry = entry?;
        let name = entry.file_name();
        if name == format!(".honk300-{}.new", consent.nonce).as_str() {
            let bytes = read_file(&entry.path(), 65_536)?;
            if !consent.script.as_bytes().starts_with(&bytes)
                && !consent.metadata.as_bytes().starts_with(&bytes)
            {
                return Err(invalid("Interrupted GNOME temporary file changed"));
            }
            fs::remove_file(entry.path())?;
            continue;
        }
        let (desired, previous, limit) = if name == "extension.js" {
            (
                consent.script.as_bytes(),
                consent
                    .previous
                    .as_ref()
                    .map(|files| files.script.as_bytes()),
                65_536,
            )
        } else if name == "metadata.json" {
            (
                consent.metadata.as_bytes(),
                consent
                    .previous
                    .as_ref()
                    .map(|files| files.metadata.as_bytes()),
                4096,
            )
        } else {
            return Err(invalid("GNOME companion contains unrelated files"));
        };
        let current = read_file(&entry.path(), limit)?;
        if current != desired && previous != Some(current.as_slice()) {
            return Err(invalid(
                "Interrupted GNOME companion contains changed files",
            ));
        }
    }
    Ok(())
}

fn create_extension_parent(directory: &Path) -> io::Result<PathBuf> {
    let extension = extension_path(directory)?;
    let extensions = extension.parent().expect("constant extension suffix");
    let shell = extensions.parent().expect("constant Shell suffix");
    for path in [shell, extensions] {
        match fs::create_dir(path) {
            Err(error) if error.kind() == io::ErrorKind::AlreadyExists => {}
            result => result?,
        }
        let metadata = fs::symlink_metadata(path)?;
        if !metadata.is_dir() || metadata.file_type().is_symlink() {
            return Err(invalid("GNOME extension parent is not an owned directory"));
        }
        #[cfg(target_os = "linux")]
        {
            use std::os::unix::fs::MetadataExt;
            if metadata.uid() != unsafe { libc::geteuid() } || metadata.mode() & 0o022 != 0 {
                return Err(invalid("GNOME extension parent has unsafe ownership"));
            }
        }
    }
    Ok(extension)
}

fn atomic_file(path: &Path, bytes: &[u8], nonce: &str) -> io::Result<()> {
    let temporary = path.with_file_name(format!(".honk300-{nonce}.new"));
    match fs::symlink_metadata(&temporary) {
        Err(error) if error.kind() == io::ErrorKind::NotFound => {}
        Err(error) => return Err(error),
        Ok(_) => {
            let leftover = read_file(&temporary, MAX_RECORD)?;
            let mut known = bytes.starts_with(&leftover);
            if path.file_name().is_some_and(|name| name == RECORD) {
                if let Some(current) = read(path.parent().expect("record parent"))? {
                    if current.nonce == nonce {
                        for phase in [Phase::Pending, Phase::Active, Phase::Revoking] {
                            let mut alternative = current.clone();
                            alternative.phase = phase;
                            if phase == Phase::Active {
                                alternative.previous = None;
                            }
                            known |= serde_json::to_vec(&alternative)
                                .map_err(io::Error::other)?
                                .starts_with(&leftover);
                        }
                    }
                }
            }
            if !known {
                return Err(invalid("GNOME transaction temporary file has changed"));
            }
            fs::remove_file(&temporary)?;
        }
    }
    let mut created = false;
    let result = (|| {
        let mut file = installed::private_file(&temporary, true)?;
        created = true;
        file.write_all(bytes)?;
        file.sync_all()?;
        drop(file);
        fs::rename(&temporary, path)
    })();
    if created && result.is_err() {
        let _ = fs::remove_file(temporary);
    }
    result
}

pub(super) fn install(directory: &Path, nonce: &str, executable: &Path) -> io::Result<Consent> {
    if nonce.len() != 32 || !nonce.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Err(invalid("Invalid GNOME setup identity"));
    }
    let _lock = installed::lock_record(directory, ".gnome-lock")?;
    let previous = read(directory)?;
    if let Some(previous) = &previous {
        known_files(directory, previous)?;
        if previous.current(executable) {
            return Ok(previous.clone());
        }
        if previous.phase == Phase::Revoking {
            return Err(invalid(
                "Finish removing the interrupted GNOME setup before enabling it",
            ));
        }
        if previous.phase == Phase::Pending && !previous.payload_current(executable) {
            return Err(invalid(
                "Remove the interrupted GNOME setup before changing its executable",
            ));
        }
    }
    let extension = create_extension_parent(directory)?;
    if previous.is_none() {
        match fs::symlink_metadata(&extension) {
            Err(error) if error.kind() == io::ErrorKind::NotFound => {}
            Err(error) => return Err(error),
            Ok(_) => {
                return Err(invalid(
                    "GNOME extension already exists without Honk300 ownership",
                ))
            }
        }
    }
    let mut consent = if let Some(previous) = &previous {
        if previous.phase == Phase::Pending {
            previous.clone()
        } else {
            Consent {
                phase: Phase::Pending,
                previous: Some(Files {
                    script: previous.script.clone(),
                    metadata: previous.metadata.clone(),
                }),
                boundary: BOUNDARY.into(),
                nonce: nonce.into(),
                script: bundle().script.clone(),
                metadata: std::str::from_utf8(METADATA)
                    .map_err(|_| invalid("Invalid bundled GNOME metadata"))?
                    .into(),
                executable: Executable::identify(executable)?,
            }
        }
    } else {
        Consent {
            phase: Phase::Pending,
            previous: None,
            boundary: BOUNDARY.into(),
            nonce: nonce.into(),
            script: bundle().script.clone(),
            metadata: std::str::from_utf8(METADATA)
                .map_err(|_| invalid("Invalid bundled GNOME metadata"))?
                .into(),
            executable: Executable::identify(executable)?,
        }
    };
    // A durable inactive record owns only these exact files across interruption.
    // Shell and Rust both refuse this phase; permission becomes active last.
    atomic_file(
        &directory.join(RECORD),
        &serde_json::to_vec(&consent).map_err(io::Error::other)?,
        &consent.nonce,
    )?;
    if !extension.try_exists()? {
        let mut builder = fs::DirBuilder::new();
        builder.recursive(false);
        #[cfg(target_os = "linux")]
        {
            use std::os::unix::fs::DirBuilderExt;
            builder.mode(0o700);
        }
        builder.create(&extension)?;
    }
    atomic_file(
        &extension.join("extension.js"),
        bundle().script.as_bytes(),
        &consent.nonce,
    )?;
    atomic_file(&extension.join("metadata.json"), METADATA, &consent.nonce)?;
    consent.phase = Phase::Active;
    consent.previous = None;
    atomic_file(
        &directory.join(RECORD),
        &serde_json::to_vec(&consent).map_err(io::Error::other)?,
        &consent.nonce,
    )?;
    Ok(consent)
}

pub(super) fn revoke(directory: &Path) -> io::Result<Option<Consent>> {
    if read(directory)?.is_none() {
        return Ok(None);
    }
    let _lock = installed::lock_record(directory, ".gnome-lock")?;
    let mut previous = read(directory)?;
    if let Some(previous) = &mut previous {
        previous.phase = Phase::Revoking;
        atomic_file(
            &directory.join(RECORD),
            &serde_json::to_vec(previous).map_err(io::Error::other)?,
            &previous.nonce,
        )?;
    }
    Ok(previous)
}

pub(super) fn remove_files(directory: &Path, consent: &Consent) -> io::Result<()> {
    let _lock = installed::lock_record(directory, ".gnome-lock")?;
    if read(directory)?.as_ref() != Some(consent) || consent.phase != Phase::Revoking {
        return Err(invalid("GNOME setup changed during removal"));
    }
    known_files(directory, consent)?;
    let extension = extension_path(directory)?;
    for name in ["extension.js", "metadata.json"] {
        match fs::remove_file(extension.join(name)) {
            Err(error) if error.kind() == io::ErrorKind::NotFound => {}
            result => result?,
        }
    }
    match fs::remove_dir(extension) {
        Err(error) if error.kind() == io::ErrorKind::NotFound => {}
        result => result?,
    }
    fs::remove_file(directory.join(RECORD))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn native_companion_is_explicit_idempotent_and_preserves_unrelated_state() {
        let parent = tempfile::tempdir().unwrap();
        let directory = parent.path().join("honk300/wayland");
        let executable = std::env::current_exe().unwrap();
        let nonce = "0123456789abcdef0123456789abcdef";
        let first = install(&directory, nonce, &executable).unwrap();
        assert!(first.current(&executable));
        assert!(first.script.contains(&build_identity()));
        assert!(
            install(&directory, "fedcba9876543210fedcba9876543210", &executable).unwrap() == first
        );
        files_match(&directory, &first).unwrap();
        fs::write(directory.join("foreign.txt"), "keep").unwrap();
        let previous = revoke(&directory).unwrap().unwrap();
        assert!(!read(&directory).unwrap().unwrap().current(&executable));
        remove_files(&directory, &previous).unwrap();
        assert!(read(&directory).unwrap().is_none());
        assert_eq!(
            fs::read_to_string(directory.join("foreign.txt")).unwrap(),
            "keep"
        );
        install(&directory, nonce, &executable).unwrap();
    }
    #[test]
    fn edited_companion_is_never_overwritten_or_removed() {
        let parent = tempfile::tempdir().unwrap();
        let directory = parent.path().join("honk300/wayland");
        let executable = std::env::current_exe().unwrap();
        let nonce = "0123456789abcdef0123456789abcdef";
        install(&directory, nonce, &executable).unwrap();
        let file = extension_path(&directory).unwrap().join("extension.js");
        fs::write(&file, "unrelated code").unwrap();
        assert!(install(&directory, nonce, &executable).is_err());
        let previous = revoke(&directory).unwrap().unwrap();
        assert!(remove_files(&directory, &previous).is_err());
        assert_eq!(fs::read_to_string(file).unwrap(), "unrelated code");
    }
    #[test]
    fn interrupted_install_and_removal_remain_inactive_and_recover_exact_files() {
        let parent = tempfile::tempdir().unwrap();
        let directory = parent.path().join("honk300/wayland");
        let executable = std::env::current_exe().unwrap();
        let nonce = "0123456789abcdef0123456789abcdef";
        let mut pending = install(&directory, nonce, &executable).unwrap();
        pending.phase = Phase::Pending;
        atomic_file(
            &directory.join(RECORD),
            &serde_json::to_vec(&pending).unwrap(),
            nonce,
        )
        .unwrap();
        let extension = extension_path(&directory).unwrap();
        fs::remove_file(extension.join("metadata.json")).unwrap();
        let mut temporary =
            installed::private_file(&extension.join(format!(".honk300-{nonce}.new")), true)
                .unwrap();
        temporary
            .write_all(&bundle().script.as_bytes()[..40])
            .unwrap();
        drop(temporary);
        assert!(!read(&directory).unwrap().unwrap().current(&executable));
        let recovered =
            install(&directory, "fedcba9876543210fedcba9876543210", &executable).unwrap();
        assert!(recovered.current(&executable));
        assert!(recovered.nonce == nonce);
        files_match(&directory, &recovered).unwrap();
        let revoking = revoke(&directory).unwrap().unwrap();
        fs::remove_file(extension.join("extension.js")).unwrap();
        assert!(!read(&directory).unwrap().unwrap().current(&executable));
        let resumed = revoke(&directory).unwrap().unwrap();
        assert!(resumed == revoking);
        remove_files(&directory, &resumed).unwrap();
        assert!(read(&directory).unwrap().is_none());
        assert!(!extension.exists());
    }
}
