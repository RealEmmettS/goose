//! Shared optimistic edit sessions for the native and terminal editors.
use super::{
    document_version, migrate_v1_document, persist_document, save_target, Config, ConfigError,
    ConfigLoadState, CONFIG_VERSION,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    fs,
    io::{self, Read},
    path::Path,
};
use toml_edit::DocumentMut;

const MAX_DOCUMENT_BYTES: u64 = 1024 * 1024;

/// Identity of the exact source bytes, including comments and unknown compatible fields.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ConfigRevision(String);

impl Default for ConfigRevision {
    fn default() -> Self {
        Self("missing".into())
    }
}

impl ConfigRevision {
    pub fn reload_token(&self, path: &Path) -> Result<[u8; 32], ConfigError> {
        let mut digest = Sha256::new();
        digest.update(config_path_identity(path)?);
        digest.update(self.0.as_bytes());
        Ok(digest.finalize().into())
    }

    pub fn read(path: &Path) -> Result<Self, ConfigError> {
        Ok(revision(read_document(path)?.as_deref()))
    }
}

#[derive(Debug, Clone)]
pub struct ConfigSnapshot {
    pub config: Config,
    pub revision: ConfigRevision,
    pub warning: Option<String>,
}

impl ConfigSnapshot {
    pub fn load(path: &Path) -> Result<Self, ConfigError> {
        let before = ConfigRevision::read(path)?;
        let (config, warning) = match Config::load(Some(path.to_owned()))? {
            ConfigLoadState::Missing { .. } => (Config::default(), None),
            ConfigLoadState::Loaded(loaded) => (loaded.config, loaded.warning),
            ConfigLoadState::Malformed { error, .. } => {
                return Err(ConfigError::MalformedDocument(error))
            }
            ConfigLoadState::UnsupportedVersion { found, .. } => {
                return Err(ConfigError::WrongVersion(found))
            }
        };
        let after = ConfigRevision::read(path)?;
        if before != after {
            return Err(ConfigError::Conflict);
        }
        Ok(Self {
            config,
            revision: after,
            warning,
        })
    }
}

/// A stable identity for the selected file, without disclosing its path over IPC.
/// Resolve existing symlinks and missing trailing components identically before/after first save.
pub fn config_path_identity(path: &Path) -> Result<[u8; 32], ConfigError> {
    let absolute = std::path::absolute(path)?;
    let mut candidate = absolute.as_path();
    let mut missing = Vec::new();
    let mut resolved = loop {
        match candidate.canonicalize() {
            Ok(path) => break path,
            Err(error) if error.kind() == io::ErrorKind::NotFound => {
                missing.push(
                    candidate
                        .file_name()
                        .ok_or_else(|| io::Error::other("cannot resolve configuration path"))?
                        .to_owned(),
                );
                candidate = candidate
                    .parent()
                    .ok_or_else(|| io::Error::other("cannot resolve configuration parent"))?;
            }
            Err(error) => return Err(error.into()),
        }
    };
    for component in missing.into_iter().rev() {
        resolved.push(component);
    }
    #[cfg(unix)]
    let bytes = {
        use std::os::unix::ffi::OsStrExt;
        resolved.as_os_str().as_bytes().to_vec()
    };
    #[cfg(windows)]
    let bytes = {
        use std::os::windows::ffi::OsStrExt;
        resolved
            .as_os_str()
            .encode_wide()
            .flat_map(|unit| {
                let unit = if (b'A' as u16..=b'Z' as u16).contains(&unit) {
                    unit + 32
                } else {
                    unit
                };
                unit.to_le_bytes()
            })
            .collect::<Vec<_>>()
    };
    #[cfg(not(any(unix, windows)))]
    let bytes = resolved.to_string_lossy().as_bytes().to_vec();
    Ok(Sha256::digest(bytes).into())
}

fn read_document(path: &Path) -> Result<Option<String>, ConfigError> {
    let file = match fs::File::open(path) {
        Ok(file) => file,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(error.into()),
    };
    let mut bytes = Vec::new();
    file.take(MAX_DOCUMENT_BYTES + 1).read_to_end(&mut bytes)?;
    if bytes.len() as u64 > MAX_DOCUMENT_BYTES {
        return Err(ConfigError::InvalidTarget(
            "configuration exceeds the 1 MiB editor limit".into(),
        ));
    }
    String::from_utf8(bytes)
        .map(Some)
        .map_err(|error| ConfigError::MalformedDocument(error.to_string()))
}

fn revision(text: Option<&str>) -> ConfigRevision {
    text.map_or_else(ConfigRevision::default, |text| {
        ConfigRevision(format!("{:x}", Sha256::digest(text.as_bytes())))
    })
}

pub(super) fn save(
    config: &Config,
    path: &Path,
    expected: Option<&ConfigRevision>,
) -> Result<ConfigRevision, ConfigError> {
    config.validate()?;
    let target = save_target(path)?;
    if let Some(parent) = target.parent() {
        fs::create_dir_all(parent)?;
    }
    let mut lock_name = target.as_os_str().to_owned();
    lock_name.push(".lock");
    let lock_path = Path::new(&lock_name);
    if fs::symlink_metadata(lock_path).is_ok_and(|metadata| metadata.file_type().is_symlink()) {
        return Err(ConfigError::InvalidTarget(
            "configuration lock must not be a symlink".into(),
        ));
    }
    let mut options = fs::OpenOptions::new();
    options.read(true).write(true).create(true).truncate(false);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    // Retain the inode after closing: unlinking an advisory lock allows two owners.
    // GUI/TUI saves fail promptly when another editor is committing.
    let lock = options.open(lock_path)?;
    lock.try_lock().map_err(|error| {
        ConfigError::Io(io::Error::other(format!(
            "another configuration save is active: {error}"
        )))
    })?;
    let source = read_document(&target)?;
    if expected.is_some_and(|expected| *expected != revision(source.as_deref())) {
        return Err(ConfigError::Conflict);
    }
    let mut document = match source.as_deref() {
        Some(text) => text
            .parse::<DocumentMut>()
            .map_err(|error| ConfigError::MalformedDocument(error.to_string()))?,
        None => DocumentMut::new(),
    };
    if let Some(found) = document_version(&document).map_err(ConfigError::MalformedDocument)? {
        if found != 1 && found != CONFIG_VERSION {
            return Err(ConfigError::WrongVersion(found));
        }
        if found == 1 {
            migrate_v1_document(&mut document).map_err(ConfigError::MalformedDocument)?;
        }
    }
    config.write_to_document(&mut document);
    // Recheck non-cooperating editors after parsing; cooperating GUI/TUI/CLI writers
    // hold the same OS lock throughout replace. No stale source is merged silently.
    if revision(read_document(&target)?.as_deref()) != revision(source.as_deref()) {
        return Err(ConfigError::Conflict);
    }
    persist_document(&target, &document)?;
    Ok(revision(Some(&document.to_string())))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn path_identity_stays_stable_through_first_save_and_distinguishes_other_files() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("new").join("config.toml");
        let before = config_path_identity(&path).unwrap();
        Config::default().save_atomic(&path).unwrap();
        assert_eq!(before, config_path_identity(&path).unwrap());
        assert_ne!(
            before,
            config_path_identity(&directory.path().join("other.toml")).unwrap()
        );
    }

    #[test]
    fn second_editor_cannot_overwrite_first_edit_or_comments() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("config.toml");
        fs::write(
            &path,
            "# mine\n[audio]\nenabled = true\nfuture = 42 # keep\n",
        )
        .unwrap();
        let first = ConfigSnapshot::load(&path).unwrap();
        let mut second = ConfigSnapshot::load(&path).unwrap();
        let mut changed = first.config;
        changed.audio.enabled = false;
        let saved = changed.save_if_revision(&path, &first.revision).unwrap();
        second.config.appearance.calm_goose = true;
        assert!(matches!(
            second.config.save_if_revision(&path, &second.revision),
            Err(ConfigError::Conflict)
        ));
        let current = ConfigSnapshot::load(&path).unwrap();
        assert_eq!(current.revision, saved);
        assert!(!current.config.audio.enabled && !current.config.appearance.calm_goose);
        let bytes = fs::read_to_string(path).unwrap();
        assert!(bytes.contains("# mine") && bytes.contains("future = 42 # keep"));
    }

    #[test]
    fn missing_file_creation_is_conditional_and_comment_only_edits_conflict() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("config.toml");
        let draft = ConfigSnapshot::load(&path).unwrap();
        assert!(!path.exists());
        fs::write(&path, "# another editor created this\n").unwrap();
        assert!(matches!(
            draft.config.save_if_revision(&path, &draft.revision),
            Err(ConfigError::Conflict)
        ));
        let draft = ConfigSnapshot::load(&path).unwrap();
        fs::write(&path, "# changed comment\n").unwrap();
        assert!(matches!(
            draft.config.save_if_revision(&path, &draft.revision),
            Err(ConfigError::Conflict)
        ));
    }

    #[test]
    fn oversized_document_is_rejected_before_parsing() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("config.toml");
        fs::write(&path, vec![b' '; MAX_DOCUMENT_BYTES as usize + 1]).unwrap();
        assert!(ConfigSnapshot::load(&path).is_err());
    }
}
