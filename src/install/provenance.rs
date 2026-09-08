//! Authoritative installer origin, receipts, markers, and registration evidence.
use super::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InstallSource {
    MsiGlobal,
    MsiCorporate,
    ExeGlobal,
    ExeCorporate,
    ManualLocal,
    Shell,
    PowerShell,
    /// A machine-wide Debian package rooted at `/usr/lib/honk300` with stable `/usr/bin`
    /// aliases. Its updater selects the architecture-matched `.deb` from the immutable tag.
    Deb,
    /// A macOS `Honk300.app` bundle installed under `~/Applications` (ADR 0020). Distinct from
    /// `ManualLocal` because its update path replaces the managed bundle from the exact-tag
    /// universal app ZIP selected by the pinned bootstrap; the DMG is the graphical install path.
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
            "deb" => Self::Deb,
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
            Self::Deb => "deb",
            Self::MacApp => "mac-app",
            Self::Unknown => "unknown",
        }
    }
}

pub fn detect_install_source() -> InstallSource {
    let receipt_evidence = read_install_receipt_source();
    if receipt_evidence != InstallSourceEvidence::Missing {
        return receipt_evidence.source_or_unknown();
    }

    #[cfg(windows)]
    {
        let registration = read_windows_registration_install_source();
        if registration != InstallSourceEvidence::Missing {
            return registration.source_or_unknown();
        }
        windows_install_source_precedence(
            read_file_install_source_marker(),
            classify_current_exe_install_source(),
        )
    }

    #[cfg(not(windows))]
    {
        if let Some(source) = read_file_install_source_marker() {
            return source;
        }

        classify_current_exe_install_source()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum InstallSourceEvidence {
    Missing,
    Valid(InstallSource),
    InvalidOrConflicting,
}

impl InstallSourceEvidence {
    pub(super) fn source_or_unknown(self) -> InstallSource {
        match self {
            Self::Valid(source) => source,
            Self::Missing | Self::InvalidOrConflicting => InstallSource::Unknown,
        }
    }
}

fn read_install_receipt_source() -> InstallSourceEvidence {
    let Ok(executable) = std::env::current_exe() else {
        return InstallSourceEvidence::Missing;
    };
    let owned = install_receipt_source_from_candidates(
        &current_owned_receipt_candidates(&executable),
        &executable,
    );
    if owned != InstallSourceEvidence::Missing {
        return owned;
    }
    install_receipt_source_from_candidates(&external_receipt_candidates(), &executable)
}

#[cfg(windows)]
pub(crate) fn detected_windows_install_root(
    expected_source: InstallSource,
) -> Result<Option<PathBuf>, DynError> {
    let executable = std::env::current_exe()?;
    for candidates in [
        current_owned_receipt_candidates(&executable),
        external_receipt_candidates(),
    ] {
        let mut found = false;
        let mut source = None;
        let mut root: Option<PathBuf> = None;
        for candidate in candidates {
            let metadata = match fs::symlink_metadata(&candidate) {
                Ok(metadata) => metadata,
                Err(error) if error.kind() == io::ErrorKind::NotFound => continue,
                Err(error) => return Err(error.into()),
            };
            found = true;
            if !metadata.is_file() || metadata.file_type().is_symlink() {
                return Err(format!(
                    "install receipt is not a regular owned file: {}",
                    candidate.display()
                )
                .into());
            }
            let value: serde_json::Value = serde_json::from_slice(&fs::read(&candidate)?)?;
            let candidate_source =
                validated_receipt_source(&value, &executable).ok_or_else(|| {
                    format!(
                        "install receipt identity is invalid: {}",
                        candidate.display()
                    )
                })?;
            let candidate_root = value
                .get("install_root")
                .and_then(serde_json::Value::as_str)
                .map(PathBuf::from)
                .ok_or_else(|| format!("install receipt has no root: {}", candidate.display()))?;
            if source.is_some_and(|existing| existing != candidate_source)
                || root
                    .as_ref()
                    .is_some_and(|existing| !paths_match(existing, &candidate_root))
            {
                return Err("conflicting protected Windows install receipts".into());
            }
            source = Some(candidate_source);
            root = Some(candidate_root);
        }
        if found {
            if source != Some(expected_source) {
                return Err(
                    "protected Windows receipt origin conflicts with update strategy".into(),
                );
            }
            return Ok(root);
        }
    }
    Ok(None)
}

pub(super) fn install_receipt_source_from_candidates(
    candidates: &[PathBuf],
    executable: &Path,
) -> InstallSourceEvidence {
    let mut source = None;
    let mut found = false;
    for candidate in candidates {
        let metadata = match fs::symlink_metadata(candidate) {
            Ok(metadata) => metadata,
            Err(error) if error.kind() == io::ErrorKind::NotFound => continue,
            Err(_) => return InstallSourceEvidence::InvalidOrConflicting,
        };
        found = true;
        if !metadata.is_file() || metadata.file_type().is_symlink() {
            return InstallSourceEvidence::InvalidOrConflicting;
        }
        let Ok(bytes) = fs::read(candidate) else {
            return InstallSourceEvidence::InvalidOrConflicting;
        };
        let Ok(value) = serde_json::from_slice::<serde_json::Value>(&bytes) else {
            return InstallSourceEvidence::InvalidOrConflicting;
        };
        let Some(candidate_source) = validated_receipt_source(&value, executable) else {
            return InstallSourceEvidence::InvalidOrConflicting;
        };
        if source.is_some_and(|existing| existing != candidate_source) {
            return InstallSourceEvidence::InvalidOrConflicting;
        }
        source = Some(candidate_source);
    }
    match (found, source) {
        (false, _) => InstallSourceEvidence::Missing,
        (true, Some(source)) => InstallSourceEvidence::Valid(source),
        (true, None) => InstallSourceEvidence::InvalidOrConflicting,
    }
}

pub(super) fn current_owned_receipt_candidates(executable: &Path) -> Vec<PathBuf> {
    let mut candidates = Vec::new();
    for ancestor in executable.ancestors().skip(1).take(8).filter(|path| {
        path.file_name().is_some_and(|name| {
            let name = name.to_string_lossy();
            name.eq_ignore_ascii_case("honk300")
                || name.eq_ignore_ascii_case("Honk300.app")
                || name.eq_ignore_ascii_case("install")
        })
    }) {
        let candidate = ancestor.join("install-receipt.json");
        if !candidates.contains(&candidate) {
            candidates.push(candidate);
        }
    }
    candidates
}

pub(super) fn external_receipt_candidates() -> Vec<PathBuf> {
    let mut candidates = Vec::new();
    #[cfg(windows)]
    if let Ok(path) = windows_receipt_path() {
        candidates.push(path);
    }
    #[cfg(target_os = "linux")]
    if let Ok(path) = linux_receipt_path() {
        if !candidates.contains(&path) {
            candidates.push(path);
        }
    }
    #[cfg(target_os = "macos")]
    if let Ok(path) = macos_receipt_path() {
        if !candidates.contains(&path) {
            candidates.push(path);
        }
    }
    candidates
}

pub(super) fn validated_receipt_source(
    value: &serde_json::Value,
    executable: &Path,
) -> Option<InstallSource> {
    let schema = value.get("schema")?.as_str()?;
    let install_root = value.get("install_root")?.as_str().map(Path::new)?;
    if !path_is_within(executable, install_root) {
        return None;
    }
    if schema == OWNERSHIP_MARKER {
        let channel = value.get("channel")?.as_str()?;
        return legacy_receipt_source(channel);
    }
    if schema != INSTALL_RECEIPT_V2 {
        return None;
    }
    let source = InstallSource::from_marker(value.get("origin")?.as_str()?);
    let family = value.get("installer_family")?.as_str()?;
    let edition = value.get("edition")?.as_str()?;
    let scope = value.get("scope")?.as_str()?;
    let track = value.get("release_track")?.as_str()?;
    let target = value.get("target")?.as_str()?;
    let active_release = value.get("active_release")?.as_str()?;
    let artifact = value.get("artifact")?.as_object()?;
    let artifact_name = artifact.get("name")?.as_str()?;
    let artifact_hash = artifact.get("sha256")?.as_str()?;
    let artifact_size = artifact.get("size")?.as_u64()?;
    if track != "stable"
        || target.is_empty()
        || active_release.is_empty()
        || artifact_name.is_empty()
        || artifact_size == 0
        || artifact_hash.len() != 64
        || !artifact_hash.chars().all(|c| c.is_ascii_hexdigit())
    {
        return None;
    }
    let identity_matches = match source {
        InstallSource::MsiGlobal => ("msi", "global", "machine"),
        InstallSource::MsiCorporate => ("msi", "corporate", "user"),
        InstallSource::ExeGlobal => ("exe", "global", "machine"),
        InstallSource::ExeCorporate => ("exe", "corporate", "user"),
        InstallSource::PowerShell => ("powershell", "global", "machine"),
        InstallSource::Shell => ("shell", "global", "user"),
        InstallSource::Deb => ("deb", "global", "machine"),
        InstallSource::MacApp => ("dmg", "global", "user"),
        InstallSource::ManualLocal | InstallSource::Unknown => return None,
    };
    (family, edition, scope == identity_matches.2)
        .eq(&(identity_matches.0, identity_matches.1, true))
        .then_some(source)
}

fn legacy_receipt_source(channel: &str) -> Option<InstallSource> {
    match channel {
        "msi-global" => Some(InstallSource::MsiGlobal),
        "msi-corporate" => Some(InstallSource::MsiCorporate),
        "exe-global" => Some(InstallSource::ExeGlobal),
        "exe-corporate" => Some(InstallSource::ExeCorporate),
        "powershell" | "powershell-global-msi" => Some(InstallSource::PowerShell),
        "shell" => Some(InstallSource::Shell),
        "deb" => Some(InstallSource::Deb),
        "dmg" | "mac-app" => Some(InstallSource::MacApp),
        _ => None,
    }
}

#[cfg(any(test, windows))]
pub(super) fn windows_install_source_precedence(
    file_marker: Option<InstallSource>,
    classified_path: InstallSource,
) -> InstallSource {
    file_marker.unwrap_or(classified_path)
}

#[cfg(windows)]
fn read_windows_registration_install_source() -> InstallSourceEvidence {
    let Ok(current_exe) = std::env::current_exe() else {
        return InstallSourceEvidence::InvalidOrConflicting;
    };
    let mut matches = Vec::new();
    for source in [
        InstallSource::MsiGlobal,
        InstallSource::MsiCorporate,
        InstallSource::ExeGlobal,
        InstallSource::ExeCorporate,
    ] {
        match find_windows_managed_uninstall(source, &current_exe) {
            Ok(Some(_)) => matches.push(source),
            Ok(None) => {}
            Err(_) => return InstallSourceEvidence::InvalidOrConflicting,
        }
    }
    windows_registration_evidence(&matches, read_windows_install_source_marker())
}

#[cfg(any(test, windows))]
pub(super) fn windows_registration_evidence(
    matches: &[InstallSource],
    registered_origin: Option<InstallSource>,
) -> InstallSourceEvidence {
    let [source] = matches else {
        return if matches.is_empty() {
            InstallSourceEvidence::Missing
        } else {
            InstallSourceEvidence::InvalidOrConflicting
        };
    };
    match registered_origin {
        None => InstallSourceEvidence::Valid(*source),
        Some(origin) if origin == *source => InstallSourceEvidence::Valid(origin),
        Some(InstallSource::PowerShell) if *source == InstallSource::MsiGlobal => {
            InstallSourceEvidence::Valid(InstallSource::PowerShell)
        }
        Some(_) => InstallSourceEvidence::InvalidOrConflicting,
    }
}

#[cfg(test)]
pub(super) fn path_is_in_app_bundle(path: &Path) -> bool {
    path.ancestors()
        .any(|ancestor| ancestor.extension().and_then(|ext| ext.to_str()) == Some("app"))
}

#[cfg(any(windows, target_os = "linux"))]
pub(super) fn write_install_marker(root: &Path, source: InstallSource) -> io::Result<()> {
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

pub(super) fn classify_install_path(path: &str) -> InstallSource {
    let lower = path.to_ascii_lowercase().replace('/', "\\");
    // Program Files and LocalAppData alone cannot distinguish MSI from EXE ownership. They are
    // deliberately ambiguous unless a receipt, registration, or adjacent owned marker says more.
    if lower.contains("\\.local\\share\\honk300\\install\\") {
        InstallSource::Shell
    } else {
        InstallSource::Unknown
    }
}

#[cfg(windows)]
pub(super) fn write_windows_install_source_marker(source: InstallSource) -> io::Result<()> {
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

    // This is a fallback for older layouts without an adjacent marker. The marker next to the
    // running executable wins first so supported Global and Corporate installs can coexist.
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
