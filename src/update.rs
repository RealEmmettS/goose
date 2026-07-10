use crate::install::{detect_install_source, InstallSource};
#[cfg(windows)]
use crate::install::{system_windows_msiexec_path, system_windows_powershell_path};
use sha2::{Digest, Sha256};
use std::fs;
use std::io::{self, Read};
use std::path::{Path, PathBuf};
use std::process::Command;

const RELEASE_MANIFEST_URL: &str =
    "https://github.com/RealEmmettS/goose/releases/latest/download/release-manifest.json";
const RELEASE_DOWNLOAD_ROOT: &str = "https://github.com/RealEmmettS/goose/releases/download";

type DynError = Box<dyn std::error::Error>;

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ReleaseTarget {
    WindowsX64,
    WindowsArm64,
    LinuxX64Gnu,
    LinuxArm64Gnu,
    LinuxX64Musl,
    LinuxArm64Musl,
    MacosX64,
    MacosArm64,
}

impl ReleaseTarget {
    fn triple(self) -> &'static str {
        match self {
            Self::WindowsX64 => "x86_64-pc-windows-msvc",
            Self::WindowsArm64 => "aarch64-pc-windows-msvc",
            Self::LinuxX64Gnu => "x86_64-unknown-linux-gnu",
            Self::LinuxArm64Gnu => "aarch64-unknown-linux-gnu",
            Self::LinuxX64Musl => "x86_64-unknown-linux-musl",
            Self::LinuxArm64Musl => "aarch64-unknown-linux-musl",
            Self::MacosX64 => "x86_64-apple-darwin",
            Self::MacosArm64 => "aarch64-apple-darwin",
        }
    }

    fn is_windows(self) -> bool {
        matches!(self, Self::WindowsX64 | Self::WindowsArm64)
    }

    fn is_linux(self) -> bool {
        matches!(
            self,
            Self::LinuxX64Gnu | Self::LinuxArm64Gnu | Self::LinuxX64Musl | Self::LinuxArm64Musl
        )
    }

    fn is_macos(self) -> bool {
        matches!(self, Self::MacosX64 | Self::MacosArm64)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum UpdateStrategy {
    MsiGlobal,
    MsiCorporate,
    ExeGlobal,
    ExeCorporate,
    PowerShell,
    Shell,
}

impl UpdateStrategy {
    fn label(self) -> &'static str {
        match self {
            Self::MsiGlobal => "Global MSI installer",
            Self::MsiCorporate => "Corporate MSI installer",
            Self::ExeGlobal => "Global EXE installer",
            Self::ExeCorporate => "Corporate EXE installer",
            Self::PowerShell => "PowerShell installer",
            Self::Shell => "shell installer",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct UpdatePlan {
    strategy: UpdateStrategy,
    artifact: String,
}

#[cfg(any(test, windows))]
#[derive(Debug, Clone, PartialEq, Eq)]
struct WindowsUpdateHelperInvocation {
    args: Vec<String>,
    script: String,
}

#[cfg(any(test, windows))]
struct WindowsUpdateHelperRequest<'a> {
    current_pid: u32,
    strategy: UpdateStrategy,
    artifact: &'a Path,
    expected_hash: &'a str,
    expected_size: u64,
    installed_executable: &'a Path,
    expected_version: &'a str,
    system_msiexec: &'a Path,
    system_powershell: &'a Path,
}

#[cfg(any(test, windows))]
fn windows_strategy_executable(
    strategy: UpdateStrategy,
    source: InstallSource,
    current_exe: &Path,
    program_files: &Path,
) -> Result<PathBuf, String> {
    match strategy {
        UpdateStrategy::MsiGlobal | UpdateStrategy::PowerShell => Ok(program_files
            .join("honk300")
            .join("bin")
            .join("honk300.exe")),
        UpdateStrategy::MsiCorporate => {
            marker_derived_windows_executable(source, InstallSource::MsiCorporate, current_exe)
        }
        UpdateStrategy::ExeGlobal => {
            marker_derived_windows_executable(source, InstallSource::ExeGlobal, current_exe)
        }
        UpdateStrategy::ExeCorporate => {
            marker_derived_windows_executable(source, InstallSource::ExeCorporate, current_exe)
        }
        UpdateStrategy::Shell => Err("shell updates are not a Windows strategy".into()),
    }
}

#[cfg(any(test, windows))]
fn marker_derived_windows_executable(
    source: InstallSource,
    expected_source: InstallSource,
    current_exe: &Path,
) -> Result<PathBuf, String> {
    let normalized = current_exe
        .to_string_lossy()
        .replace('/', "\\")
        .to_ascii_lowercase();
    if source != expected_source
        || !is_absolute_windows_path(current_exe)
        || !["honk300.exe", "honk.exe", "goose.exe"]
            .iter()
            .any(|name| normalized.ends_with(&format!("\\bin\\{name}")))
    {
        return Err(format!(
            "cannot derive the owned post-update executable for {}",
            expected_source.marker_value()
        ));
    }
    Ok(current_exe
        .parent()
        .expect("validated executable has a bin parent")
        .join("honk300.exe"))
}

#[cfg(any(test, windows))]
fn is_absolute_windows_path(path: &Path) -> bool {
    #[cfg(windows)]
    {
        path.is_absolute()
    }
    #[cfg(not(windows))]
    {
        // These helpers are intentionally unit-tested on macOS/Linux runners too, where the host
        // Path implementation does not recognize a drive-letter or UNC path as absolute.
        let normalized = path.to_string_lossy().replace('/', "\\");
        let bytes = normalized.as_bytes();
        (bytes.len() >= 3
            && bytes[0].is_ascii_alphabetic()
            && bytes[1] == b':'
            && bytes[2] == b'\\')
            || normalized.starts_with("\\\\")
    }
}

#[cfg(any(test, not(windows)))]
fn unix_strategy_executable(target: ReleaseTarget, install_root: &Path) -> PathBuf {
    if target.is_macos() {
        install_root.join("Contents").join("MacOS").join("honk300")
    } else {
        install_root.join("bin").join("honk300")
    }
}

#[cfg(any(test, windows))]
fn windows_update_helper_invocation(
    request: WindowsUpdateHelperRequest<'_>,
) -> WindowsUpdateHelperInvocation {
    let WindowsUpdateHelperRequest {
        current_pid,
        strategy,
        artifact,
        expected_hash,
        expected_size,
        installed_executable,
        expected_version,
        system_msiexec,
        system_powershell,
    } = request;
    let artifact_literal = powershell_literal(&artifact.to_string_lossy());
    let artifact_argument_literal =
        powershell_literal(&format!("\"{}\"", artifact.to_string_lossy()));
    let installed_literal = powershell_literal(&installed_executable.to_string_lossy());
    let (installer, arguments, elevated) = match strategy {
        UpdateStrategy::MsiGlobal => (
            system_msiexec.to_string_lossy().into_owned(),
            format!("@('/i','{artifact_argument_literal}','/passive','/norestart')"),
            true,
        ),
        UpdateStrategy::MsiCorporate => (
            system_msiexec.to_string_lossy().into_owned(),
            format!("@('/i','{artifact_argument_literal}','/passive','/norestart')"),
            false,
        ),
        UpdateStrategy::ExeGlobal => (
            artifact.to_string_lossy().into_owned(),
            "@('/SILENT','/SUPPRESSMSGBOXES','/NORESTART')".to_owned(),
            true,
        ),
        UpdateStrategy::ExeCorporate => (
            artifact.to_string_lossy().into_owned(),
            "@('/SILENT','/SUPPRESSMSGBOXES','/NORESTART')".to_owned(),
            false,
        ),
        UpdateStrategy::PowerShell => (
            system_powershell.to_string_lossy().into_owned(),
            format!(
                "@('-NoProfile','-WindowStyle','Hidden','-ExecutionPolicy','Bypass','-File','{artifact_argument_literal}')"
            ),
            false,
        ),
        UpdateStrategy::Shell => (
            "sh".to_owned(),
            format!("@('{artifact_argument_literal}')"),
            false,
        ),
    };
    let elevation = if elevated { " -Verb RunAs" } else { "" };
    let installer_literal = powershell_literal(&installer);
    let expected_hash = powershell_literal(&expected_hash.to_ascii_lowercase());
    let expected_version = powershell_literal(strip_prerelease_metadata(expected_version));
    let script = format!(
        "$ErrorActionPreference='Stop'; $artifact='{artifact_literal}'; $expectedHash='{expected_hash}'; $expectedSize=[int64]{expected_size}; $owned=$false; Wait-Process -Id {current_pid} -ErrorAction SilentlyContinue; try {{ $item=Get-Item -LiteralPath $artifact -Force -ErrorAction Stop; if (($item.Attributes -band [IO.FileAttributes]::ReparsePoint) -ne 0 -or $item.Length -ne $expectedSize) {{ throw 'verified update artifact identity changed' }}; $actualHash=(Get-FileHash -LiteralPath $artifact -Algorithm SHA256).Hash.ToLowerInvariant(); if ($actualHash -ne $expectedHash) {{ throw 'verified update artifact hash changed' }}; $owned=$true; $process=Start-Process -FilePath '{installer_literal}' -ArgumentList {arguments} -WindowStyle Hidden -Wait -PassThru{elevation}; if ($process.ExitCode -notin @(0,1641,3010)) {{ throw \"installer exited with code $($process.ExitCode)\" }}; if (-not (Test-Path -LiteralPath '{installed_literal}' -PathType Leaf)) {{ throw 'installed honk300 executable is missing' }}; $versionOutput=(& '{installed_literal}' --version | Select-Object -Last 1); if ($LASTEXITCODE -ne 0 -or [string]::IsNullOrWhiteSpace($versionOutput)) {{ throw 'installed honk300 version check failed' }}; $reported=($versionOutput.Trim().Split()[-1] -replace '[+-].*$',''); if ($reported -ne '{expected_version}') {{ throw \"installed honk300 version $reported does not match {expected_version}\" }} }} finally {{ if ($owned -and (Test-Path -LiteralPath $artifact -PathType Leaf)) {{ $cleanup=Get-Item -LiteralPath $artifact -Force; if (($cleanup.Attributes -band [IO.FileAttributes]::ReparsePoint) -eq 0 -and $cleanup.Length -eq $expectedSize -and (Get-FileHash -LiteralPath $artifact -Algorithm SHA256).Hash.ToLowerInvariant() -eq $expectedHash) {{ Remove-Item -LiteralPath $artifact -Force }} }} }}"
    );
    WindowsUpdateHelperInvocation {
        args: vec![
            "-NoProfile".into(),
            "-WindowStyle".into(),
            "Hidden".into(),
            "-ExecutionPolicy".into(),
            "Bypass".into(),
            "-Command".into(),
            script.clone(),
        ],
        script,
    }
}

#[cfg(any(test, windows))]
fn powershell_literal(value: &str) -> String {
    value.replace('\'', "''")
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ReleaseArtifact {
    name: String,
    target: String,
    kind: String,
    sha256: String,
    size: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ReleaseManifest {
    version: String,
    tag: String,
    commit: String,
    artifacts: Vec<ReleaseArtifact>,
}

fn parse_release_manifest(body: &str, expected_tag: &str) -> Result<ReleaseManifest, String> {
    let root: serde_json::Value = serde_json::from_str(body)
        .map_err(|error| format!("failed to parse release manifest: {error}"))?;
    let object = root
        .as_object()
        .ok_or("release manifest must be a JSON object")?;
    let string = |key: &str| {
        object
            .get(key)
            .and_then(serde_json::Value::as_str)
            .map(str::to_owned)
            .ok_or_else(|| format!("release manifest field {key} must be a string"))
    };

    let schema = string("schema")?;
    if schema != "honk300.release.v1" {
        return Err(format!("unsupported release manifest schema: {schema}"));
    }
    let version = string("version")?;
    let tag = string("tag")?;
    validate_stable_tag(&tag)?;
    if tag != format!("v{version}") {
        return Err(format!(
            "release manifest tag {tag} does not match version {version}"
        ));
    }
    if !expected_tag.is_empty() && tag != expected_tag {
        return Err(format!(
            "release manifest tag {tag} does not match expected tag {expected_tag}"
        ));
    }
    let commit = string("commit")?;
    if commit.len() != 40
        || !commit
            .chars()
            .all(|character| character.is_ascii_hexdigit())
    {
        return Err("release manifest commit must be a full hexadecimal SHA".into());
    }

    let values = object
        .get("artifacts")
        .and_then(serde_json::Value::as_array)
        .ok_or("release manifest artifacts must be an array")?;
    if values.is_empty() {
        return Err("release manifest artifacts must not be empty".into());
    }
    let mut names = std::collections::HashSet::new();
    let mut artifacts = Vec::with_capacity(values.len());
    for value in values {
        let artifact = value
            .as_object()
            .ok_or("release manifest artifact must be an object")?;
        let artifact_string = |key: &str| {
            artifact
                .get(key)
                .and_then(serde_json::Value::as_str)
                .map(str::to_owned)
                .ok_or_else(|| format!("release artifact field {key} must be a string"))
        };
        let name = artifact_string("name")?;
        validate_artifact_name(&name)?;
        if !names.insert(name.clone()) {
            return Err(format!(
                "release manifest contains duplicate artifact {name}"
            ));
        }
        let target = artifact_string("target")?;
        let kind = artifact_string("kind")?;
        let checksum = artifact_string("checksum")?;
        if checksum != format!("{name}.sha256") {
            return Err(format!(
                "release artifact {name} has an invalid checksum sidecar"
            ));
        }
        validate_artifact_name(&checksum)?;
        if target.is_empty() || kind.is_empty() {
            return Err(format!(
                "release artifact {name} has an empty target or kind"
            ));
        }
        let sha256 = artifact_string("sha256")?.to_ascii_lowercase();
        if sha256.len() != 64
            || !sha256
                .chars()
                .all(|character| character.is_ascii_hexdigit())
        {
            return Err(format!("release artifact {name} has an invalid sha256"));
        }
        let size = artifact
            .get("size")
            .and_then(serde_json::Value::as_u64)
            .ok_or_else(|| format!("release artifact {name} has an invalid size"))?;
        if size == 0 {
            return Err(format!("release artifact {name} has an empty payload"));
        }
        artifacts.push(ReleaseArtifact {
            name,
            target,
            kind,
            sha256,
            size,
        });
    }

    Ok(ReleaseManifest {
        version,
        tag,
        commit: commit.to_ascii_lowercase(),
        artifacts,
    })
}

fn validate_stable_tag(tag: &str) -> Result<(), String> {
    let Some(version) = tag.strip_prefix('v') else {
        return Err(format!("release tag must start with v: {tag}"));
    };
    let parts = version.split('.').collect::<Vec<_>>();
    if parts.len() != 3
        || parts.iter().any(|part| {
            part.is_empty() || !part.chars().all(|character| character.is_ascii_digit())
        })
    {
        return Err(format!(
            "release tag must be a stable vMAJOR.MINOR.PATCH value: {tag}"
        ));
    }
    Ok(())
}

fn validate_artifact_name(artifact: &str) -> Result<(), String> {
    if artifact.is_empty()
        || artifact == "."
        || artifact == ".."
        || artifact.contains("..")
        || artifact.contains('/')
        || artifact.contains('\\')
        || artifact.contains(':')
        || !artifact.chars().all(|character| {
            character.is_ascii_alphanumeric() || matches!(character, '.' | '-' | '_')
        })
    {
        return Err(format!("unsafe release artifact name: {artifact}"));
    }
    Ok(())
}

fn artifact_url_for_tag(tag: &str, artifact: &str) -> Result<String, String> {
    validate_stable_tag(tag)?;
    validate_artifact_name(artifact)?;
    Ok(format!("{RELEASE_DOWNLOAD_ROOT}/{tag}/{artifact}"))
}

pub fn run() -> Result<(), DynError> {
    println!("honk300: checking for updates...");
    let manifest = fetch_latest_release_manifest()?;
    let latest = manifest.version.clone();
    let current = env!("CARGO_PKG_VERSION");
    if !is_newer(current, &latest) {
        println!("honk300: already on the latest version ({current}).");
        return Ok(());
    }

    let target = current_release_target()
        .ok_or("honk300 update: this OS/architecture is not part of the M19 release matrix")?;
    let source = detect_install_source();
    let plan = select_update_plan(source, target)?;
    let artifact = manifest_artifact(&manifest, &plan, target)?;
    let artifact_url = artifact_url_for_tag(&manifest.tag, &artifact.name)?;
    let temp_path = temp_artifact_path(&latest, &plan.artifact);
    let installed_executable = strategy_owned_executable(plan.strategy, source, target)?;

    println!(
        "honk300: update available {current} -> {latest}; using {}.",
        plan.strategy.label()
    );
    prepare_temp_artifact_path(&temp_path)?;
    download_to_file(&artifact_url, &temp_path)?;
    verify_manifest_artifact(&temp_path, artifact)?;

    #[cfg(windows)]
    {
        schedule_windows_update(
            plan.strategy,
            &temp_path,
            artifact,
            &installed_executable,
            &latest,
        )?;
        println!(
            "honk300: verified update handoff scheduled; installation and exact-path verification begin after this process exits."
        );
        Ok(())
    }

    #[cfg(not(windows))]
    {
        run_installer(plan.strategy, &temp_path)?;
        verify_post_install(&installed_executable, &latest)?;
        remove_verified_temp_artifact(&temp_path, artifact)?;
        println!("honk300: updated to {latest}.");
        Ok(())
    }
}

fn current_release_target() -> Option<ReleaseTarget> {
    #[cfg(all(windows, target_arch = "x86_64"))]
    {
        return Some(ReleaseTarget::WindowsX64);
    }
    #[cfg(all(windows, target_arch = "aarch64"))]
    {
        return Some(ReleaseTarget::WindowsArm64);
    }
    #[cfg(all(target_os = "linux", target_arch = "x86_64", target_env = "gnu"))]
    {
        return Some(ReleaseTarget::LinuxX64Gnu);
    }
    #[cfg(all(target_os = "linux", target_arch = "aarch64", target_env = "gnu"))]
    {
        return Some(ReleaseTarget::LinuxArm64Gnu);
    }
    #[cfg(all(target_os = "linux", target_arch = "x86_64", target_env = "musl"))]
    {
        return Some(ReleaseTarget::LinuxX64Musl);
    }
    #[cfg(all(target_os = "linux", target_arch = "aarch64", target_env = "musl"))]
    {
        return Some(ReleaseTarget::LinuxArm64Musl);
    }
    #[cfg(all(target_os = "macos", target_arch = "x86_64"))]
    {
        return Some(ReleaseTarget::MacosX64);
    }
    #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
    {
        return Some(ReleaseTarget::MacosArm64);
    }
    #[allow(unreachable_code)]
    None
}

fn select_update_plan(source: InstallSource, target: ReleaseTarget) -> Result<UpdatePlan, String> {
    if target.is_macos() {
        // Every modern macOS provenance updates through the managed shell installer. The DMG
        // remains a release compatibility asset solely for the already-shipped v0.2.1 updater.
        return Ok(UpdatePlan {
            strategy: UpdateStrategy::Shell,
            artifact: "honk300-installer.sh".into(),
        });
    }
    if target.is_linux() {
        return Ok(UpdatePlan {
            strategy: UpdateStrategy::Shell,
            artifact: "honk300-installer.sh".into(),
        });
    }
    if !target.is_windows() {
        return Err(format!("unsupported release target {}", target.triple()));
    }

    let triple = target.triple();
    let plan = match source {
        InstallSource::MsiGlobal => UpdatePlan {
            strategy: UpdateStrategy::MsiGlobal,
            artifact: format!("honk300-{triple}.msi"),
        },
        InstallSource::MsiCorporate => UpdatePlan {
            strategy: UpdateStrategy::MsiCorporate,
            artifact: format!("honk300-{triple}-corporate.msi"),
        },
        InstallSource::ExeGlobal => UpdatePlan {
            strategy: UpdateStrategy::ExeGlobal,
            artifact: format!("honk300-{triple}-setup.exe"),
        },
        InstallSource::ExeCorporate => UpdatePlan {
            strategy: UpdateStrategy::ExeCorporate,
            artifact: format!("honk300-{triple}-corporate-setup.exe"),
        },
        // Unknown/legacy portable installs converge on the product's primary machine-wide MSI.
        InstallSource::ManualLocal | InstallSource::Unknown | InstallSource::MacApp => UpdatePlan {
            strategy: UpdateStrategy::MsiGlobal,
            artifact: format!("honk300-{triple}.msi"),
        },
        InstallSource::PowerShell | InstallSource::Shell => UpdatePlan {
            strategy: UpdateStrategy::PowerShell,
            artifact: "honk300-installer.ps1".into(),
        },
    };
    Ok(plan)
}

fn temp_artifact_path(latest: &str, artifact: &str) -> PathBuf {
    std::env::temp_dir().join(format!(
        "honk300-update-{}-{}-{}",
        latest.replace(|c: char| !c.is_ascii_alphanumeric() && c != '.', "_"),
        std::process::id(),
        artifact
    ))
}

fn prepare_temp_artifact_path(path: &Path) -> Result<(), DynError> {
    match fs::symlink_metadata(path) {
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(error.into()),
        Ok(_) => Err(format!(
            "refusing to overwrite pre-existing update artifact {}",
            path.display()
        )
        .into()),
    }
}

#[cfg(windows)]
fn schedule_windows_update(
    strategy: UpdateStrategy,
    artifact_path: &Path,
    artifact: &ReleaseArtifact,
    installed_executable: &Path,
    expected_version: &str,
) -> Result<(), DynError> {
    use std::os::windows::process::CommandExt;

    const CREATE_NO_WINDOW: u32 = 0x0800_0000;
    let powershell = system_windows_powershell_path()?;
    let msiexec = system_windows_msiexec_path()?;
    let invocation = windows_update_helper_invocation(WindowsUpdateHelperRequest {
        current_pid: std::process::id(),
        strategy,
        artifact: artifact_path,
        expected_hash: &artifact.sha256,
        expected_size: artifact.size,
        installed_executable,
        expected_version,
        system_msiexec: &msiexec,
        system_powershell: &powershell,
    });
    Command::new(powershell)
        .args(invocation.args)
        .creation_flags(CREATE_NO_WINDOW)
        .spawn()?;
    Ok(())
}

#[cfg(windows)]
fn strategy_owned_executable(
    strategy: UpdateStrategy,
    source: InstallSource,
    _target: ReleaseTarget,
) -> Result<PathBuf, DynError> {
    let program_files =
        PathBuf::from(std::env::var_os("ProgramFiles").ok_or("ProgramFiles is not set")?);
    let current_exe = std::env::current_exe()?;
    let executable = windows_strategy_executable(strategy, source, &current_exe, &program_files)?;
    if matches!(
        strategy,
        UpdateStrategy::MsiCorporate | UpdateStrategy::ExeGlobal | UpdateStrategy::ExeCorporate
    ) && !windows_current_marker_matches(&executable, source)?
    {
        return Err(format!(
            "honk300 update: the {} install marker could not be proven next to {}",
            source.marker_value(),
            executable.display()
        )
        .into());
    }
    Ok(executable)
}

#[cfg(windows)]
fn windows_current_marker_matches(executable: &Path, source: InstallSource) -> io::Result<bool> {
    let Some(bin) = executable.parent() else {
        return Ok(false);
    };
    let mut candidates = vec![bin.join("install-source.txt")];
    if let Some(root) = bin.parent() {
        candidates.push(root.join("install-source.txt"));
    }
    for candidate in candidates {
        let metadata = match fs::symlink_metadata(&candidate) {
            Ok(metadata) => metadata,
            Err(error) if error.kind() == io::ErrorKind::NotFound => continue,
            Err(error) => return Err(error),
        };
        if metadata.is_file()
            && !metadata.file_type().is_symlink()
            && fs::read_to_string(candidate)
                .is_ok_and(|marker| InstallSource::from_marker(&marker) == source)
        {
            return Ok(true);
        }
    }
    Ok(false)
}

#[cfg(not(windows))]
fn strategy_owned_executable(
    strategy: UpdateStrategy,
    _source: InstallSource,
    target: ReleaseTarget,
) -> Result<PathBuf, DynError> {
    if strategy != UpdateStrategy::Shell || !(target.is_linux() || target.is_macos()) {
        return Err("honk300 update: no receipt-owned executable exists for this strategy".into());
    }
    let home = PathBuf::from(std::env::var_os("HOME").ok_or("HOME is not set")?);
    let (receipt, expected_root) = if target.is_macos() {
        (
            home.join("Library")
                .join("Application Support")
                .join("honk300")
                .join("install-receipt.json"),
            home.join("Applications").join("Honk300.app"),
        )
    } else {
        let data_home = std::env::var_os("XDG_DATA_HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|| home.join(".local").join("share"));
        (
            data_home.join("honk300").join("install-receipt.json"),
            data_home.join("honk300").join("install"),
        )
    };
    let metadata = fs::symlink_metadata(&receipt).map_err(|error| {
        format!(
            "honk300 update: cannot read owned install receipt {}: {error}",
            receipt.display()
        )
    })?;
    if !metadata.is_file() || metadata.file_type().is_symlink() {
        return Err(format!(
            "honk300 update: install receipt is not a regular owned file: {}",
            receipt.display()
        )
        .into());
    }
    let value: serde_json::Value = serde_json::from_slice(&fs::read(&receipt)?)?;
    if value.get("schema").and_then(serde_json::Value::as_str) != Some("honk300.install.v1") {
        return Err("honk300 update: install receipt ownership schema is invalid".into());
    }
    let install_root = value
        .get("install_root")
        .and_then(serde_json::Value::as_str)
        .map(PathBuf::from)
        .ok_or("honk300 update: install receipt has no install_root")?;
    if !paths_equivalent(&install_root, &expected_root) {
        return Err(format!(
            "honk300 update: receipt install_root {} does not match managed root {}",
            install_root.display(),
            expected_root.display()
        )
        .into());
    }
    Ok(unix_strategy_executable(target, &install_root))
}

#[cfg(not(windows))]
fn paths_equivalent(left: &Path, right: &Path) -> bool {
    match (left.canonicalize(), right.canonicalize()) {
        (Ok(left), Ok(right)) => left == right,
        _ => left == right,
    }
}

fn fetch_latest_release_manifest() -> Result<ReleaseManifest, String> {
    let body = fetch_text(RELEASE_MANIFEST_URL).map_err(|error| error.to_string())?;
    parse_release_manifest(&body, "")
}

fn download_to_file(url: &str, path: &Path) -> Result<(), DynError> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    download_with_system_tool(url, path)
}

fn manifest_artifact<'a>(
    manifest: &'a ReleaseManifest,
    plan: &UpdatePlan,
    target: ReleaseTarget,
) -> Result<&'a ReleaseArtifact, String> {
    let artifact = manifest
        .artifacts
        .iter()
        .find(|artifact| artifact.name == plan.artifact)
        .ok_or_else(|| format!("release manifest does not contain {}", plan.artifact))?;
    let expected_kind = match plan.strategy {
        UpdateStrategy::MsiGlobal => "msi-global",
        UpdateStrategy::MsiCorporate => "msi-corporate",
        UpdateStrategy::ExeGlobal => "exe-global",
        UpdateStrategy::ExeCorporate => "exe-corporate",
        UpdateStrategy::PowerShell => "bootstrap-powershell",
        UpdateStrategy::Shell => "bootstrap-shell",
    };
    if artifact.kind != expected_kind {
        return Err(format!(
            "release artifact {} has kind {}, expected {expected_kind}",
            artifact.name, artifact.kind
        ));
    }
    let target_matches = match plan.strategy {
        UpdateStrategy::Shell => artifact.target == "universal-unix",
        UpdateStrategy::PowerShell => artifact.target == "windows",
        _ => artifact.target == target.triple(),
    };
    if !target_matches {
        return Err(format!(
            "release artifact {} targets {}, not {}",
            artifact.name,
            artifact.target,
            target.triple()
        ));
    }
    Ok(artifact)
}

fn verify_manifest_artifact(path: &Path, artifact: &ReleaseArtifact) -> Result<(), DynError> {
    let metadata = fs::symlink_metadata(path)?;
    if !metadata.is_file() || metadata.file_type().is_symlink() {
        return Err(format!(
            "artifact is not a regular file: {}; refusing to run installer",
            artifact.name
        )
        .into());
    }
    if metadata.len() != artifact.size {
        return Err(format!(
            "artifact size mismatch for {}: expected {}, downloaded {}; refusing to run installer",
            artifact.name,
            artifact.size,
            metadata.len()
        )
        .into());
    }
    let actual = compute_sha256(path)?;
    checksum_verdict(&artifact.sha256, &actual).map_err(Into::into)
}

fn fetch_text(url: &str) -> Result<String, DynError> {
    fetch_text_with_system_tool(url)
}

#[cfg(windows)]
fn fetch_text_with_system_tool(url: &str) -> Result<String, DynError> {
    use std::os::windows::process::CommandExt;

    const CREATE_NO_WINDOW: u32 = 0x0800_0000;
    let output = Command::new(system_windows_powershell_path()?)
        .args([
            "-NoProfile",
            "-ExecutionPolicy",
            "Bypass",
            "-Command",
            "param($uri) (Invoke-WebRequest -UseBasicParsing -Uri $uri -Headers @{ 'User-Agent' = 'honk300' }).Content",
        ])
        .arg(url)
        .creation_flags(CREATE_NO_WINDOW)
        .output()?;
    if output.status.success() {
        Ok(String::from_utf8(output.stdout)?)
    } else {
        Err(command_error("PowerShell web request", &output.stderr).into())
    }
}

#[cfg(not(windows))]
fn fetch_text_with_system_tool(url: &str) -> Result<String, DynError> {
    if let Ok(output) = Command::new("curl")
        .args(["-fsSL", "--retry", "3", "-H", "User-Agent: honk300", url])
        .output()
    {
        if output.status.success() {
            return Ok(String::from_utf8(output.stdout)?);
        }
    }
    let output = Command::new("wget")
        .args(["-qO-", "--header=User-Agent: honk300", url])
        .output()?;
    if output.status.success() {
        Ok(String::from_utf8(output.stdout)?)
    } else {
        Err(command_error("wget", &output.stderr).into())
    }
}

#[cfg(windows)]
fn download_with_system_tool(url: &str, path: &Path) -> Result<(), DynError> {
    use std::os::windows::process::CommandExt;

    const CREATE_NO_WINDOW: u32 = 0x0800_0000;
    let status = Command::new(system_windows_powershell_path()?)
        .args([
            "-NoProfile",
            "-ExecutionPolicy",
            "Bypass",
            "-Command",
            "param($uri, $out) Invoke-WebRequest -UseBasicParsing -Uri $uri -OutFile $out -Headers @{ 'User-Agent' = 'honk300' }",
        ])
        .arg(url)
        .arg(path)
        .creation_flags(CREATE_NO_WINDOW)
        .status()?;
    if status.success() {
        Ok(())
    } else {
        Err(format!(
            "PowerShell failed to download {url} with exit code {}",
            status.code().unwrap_or(-1)
        )
        .into())
    }
}

#[cfg(not(windows))]
fn download_with_system_tool(url: &str, path: &Path) -> Result<(), DynError> {
    let curl_status = Command::new("curl")
        .args(["-fL", "--retry", "3", "-H", "User-Agent: honk300", "-o"])
        .arg(path)
        .arg(url)
        .status();
    if matches!(curl_status, Ok(status) if status.success()) {
        return Ok(());
    }
    let status = Command::new("wget").arg("-O").arg(path).arg(url).status()?;
    if status.success() {
        Ok(())
    } else {
        Err(format!(
            "wget failed to download {url} with exit code {}",
            status.code().unwrap_or(-1)
        )
        .into())
    }
}

fn command_error(command: &str, stderr: &[u8]) -> String {
    let stderr = String::from_utf8_lossy(stderr);
    if stderr.trim().is_empty() {
        format!("{command} failed")
    } else {
        format!("{command} failed: {}", stderr.trim())
    }
}

#[cfg(test)]
fn parse_sha256_sidecar(text: &str) -> Option<String> {
    let hash = text.split_whitespace().next()?.trim_start_matches('*');
    if hash.len() == 64 && hash.chars().all(|c| c.is_ascii_hexdigit()) {
        Some(hash.to_ascii_lowercase())
    } else {
        None
    }
}

fn compute_sha256(path: &Path) -> io::Result<String> {
    let mut file = fs::File::open(path)?;
    let mut hasher = Sha256::new();
    let mut buffer = [0u8; 64 * 1024];
    loop {
        let read = file.read(&mut buffer)?;
        if read == 0 {
            break;
        }
        hasher.update(&buffer[..read]);
    }
    Ok(format!("{:x}", hasher.finalize()))
}

fn checksum_verdict(expected: &str, actual: &str) -> Result<(), String> {
    if expected.eq_ignore_ascii_case(actual) {
        Ok(())
    } else {
        Err(format!(
            "SHA256 mismatch: expected {expected}, computed {actual}; refusing to run installer"
        ))
    }
}

#[cfg(not(windows))]
fn run_installer(strategy: UpdateStrategy, path: &Path) -> Result<(), DynError> {
    let status = match strategy {
        UpdateStrategy::MsiGlobal | UpdateStrategy::MsiCorporate => Command::new("msiexec")
            .arg("/i")
            .arg(path)
            .arg("/passive")
            .arg("/norestart")
            .status(),
        UpdateStrategy::ExeGlobal | UpdateStrategy::ExeCorporate => Command::new(path)
            .args(["/SILENT", "/SUPPRESSMSGBOXES", "/NORESTART"])
            .status(),
        UpdateStrategy::PowerShell => Command::new("powershell")
            .args(["-NoProfile", "-ExecutionPolicy", "Bypass", "-File"])
            .arg(path)
            .status(),
        UpdateStrategy::Shell => Command::new("sh").arg(path).status(),
    }?;

    if status.success() {
        Ok(())
    } else {
        Err(format!(
            "{} exited with code {}",
            strategy.label(),
            status.code().unwrap_or(-1)
        )
        .into())
    }
}

#[cfg(not(windows))]
fn verify_post_install(installed_executable: &Path, latest: &str) -> Result<(), DynError> {
    let metadata = fs::symlink_metadata(installed_executable)?;
    if !metadata.is_file() || metadata.file_type().is_symlink() {
        return Err(format!(
            "installed executable is not a regular owned file: {}",
            installed_executable.display()
        )
        .into());
    }
    let output = Command::new(installed_executable)
        .arg("--version")
        .output()?;
    if !output.status.success() {
        return Err(format!(
            "{} --version failed with exit code {}",
            installed_executable.display(),
            output.status.code().unwrap_or(-1)
        )
        .into());
    }
    let stdout = String::from_utf8_lossy(&output.stdout);
    let installed = stdout
        .split_whitespace()
        .last()
        .ok_or("honk300 --version returned no version")?;
    if post_install_version_ok(installed, latest) {
        Ok(())
    } else {
        Err(format!(
            "installer completed but {} reports {installed}, not {latest}",
            installed_executable.display()
        )
        .into())
    }
}

#[cfg(not(windows))]
fn remove_verified_temp_artifact(path: &Path, artifact: &ReleaseArtifact) -> Result<(), DynError> {
    verify_manifest_artifact(path, artifact)?;
    fs::remove_file(path)?;
    Ok(())
}

#[cfg(any(test, not(windows)))]
fn post_install_version_ok(installed: &str, latest: &str) -> bool {
    strip_prerelease_metadata(installed) == strip_prerelease_metadata(latest)
}

fn is_newer(current: &str, latest: &str) -> bool {
    let current_core = strip_prerelease_metadata(current);
    let latest_core = strip_prerelease_metadata(latest);
    let current_nums = parse_version_core(current_core);
    let latest_nums = parse_version_core(latest_core);
    let len = current_nums.len().max(latest_nums.len());
    for idx in 0..len {
        let current = *current_nums.get(idx).unwrap_or(&0);
        let latest = *latest_nums.get(idx).unwrap_or(&0);
        if latest > current {
            return true;
        }
        if latest < current {
            return false;
        }
    }
    has_prerelease_or_metadata(current) && !has_prerelease_or_metadata(latest)
}

fn strip_prerelease_metadata(version: &str) -> &str {
    let version = version.strip_prefix('v').unwrap_or(version);
    match version.find(|c: char| !c.is_ascii_digit() && c != '.') {
        Some(index) => &version[..index],
        None => version,
    }
}

fn has_prerelease_or_metadata(version: &str) -> bool {
    version
        .strip_prefix('v')
        .unwrap_or(version)
        .chars()
        .any(|c| c == '-' || c == '+')
}

fn parse_version_core(version: &str) -> Vec<u64> {
    version
        .split('.')
        .filter_map(|part| part.parse::<u64>().ok())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    const VALID_MANIFEST: &str = r#"{
        "schema": "honk300.release.v1",
        "version": "0.3.0",
        "tag": "v0.3.0",
        "commit": "0123456789abcdef0123456789abcdef01234567",
        "artifacts": [
            {
                "name": "honk300-x86_64-pc-windows-msvc.msi",
                "target": "x86_64-pc-windows-msvc",
                "kind": "msi-global",
                "sha256": "abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789",
                "size": 1234,
                "checksum": "honk300-x86_64-pc-windows-msvc.msi.sha256"
            }
        ]
    }"#;

    #[test]
    fn release_manifest_validates_schema_tag_and_artifact_hashes() {
        let manifest = parse_release_manifest(VALID_MANIFEST, "v0.3.0").unwrap();
        assert_eq!(manifest.version, "0.3.0");
        assert_eq!(manifest.tag, "v0.3.0");
        assert_eq!(manifest.artifacts.len(), 1);
        assert_eq!(manifest.artifacts[0].kind, "msi-global");
    }

    #[test]
    fn release_manifest_rejects_mismatched_tag_and_bad_hash() {
        let wrong_tag = parse_release_manifest(VALID_MANIFEST, "v0.3.1").unwrap_err();
        assert!(wrong_tag.contains("tag"));

        let bad_hash = VALID_MANIFEST.replace(
            "abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789",
            "not-a-hash",
        );
        let err = parse_release_manifest(&bad_hash, "v0.3.0").unwrap_err();
        assert!(err.contains("sha256"));
    }

    #[test]
    fn exact_tag_artifact_url_rejects_untrusted_path_components() {
        assert_eq!(
            artifact_url_for_tag("v0.3.0", "honk300-installer.sh").unwrap(),
            "https://github.com/RealEmmettS/goose/releases/download/v0.3.0/honk300-installer.sh"
        );
        assert!(artifact_url_for_tag("../main", "honk300-installer.sh").is_err());
        assert!(artifact_url_for_tag("v0.3.0", "../payload").is_err());
    }

    #[test]
    fn selects_windows_installer_from_install_source() {
        assert_eq!(
            select_update_plan(InstallSource::MsiGlobal, ReleaseTarget::WindowsX64).unwrap(),
            UpdatePlan {
                strategy: UpdateStrategy::MsiGlobal,
                artifact: "honk300-x86_64-pc-windows-msvc.msi".into()
            }
        );
        assert_eq!(
            select_update_plan(InstallSource::ExeCorporate, ReleaseTarget::WindowsArm64).unwrap(),
            UpdatePlan {
                strategy: UpdateStrategy::ExeCorporate,
                artifact: "honk300-aarch64-pc-windows-msvc-corporate-setup.exe".into()
            }
        );
    }

    #[test]
    fn manual_and_unknown_windows_updates_to_global_msi() {
        for source in [InstallSource::ManualLocal, InstallSource::Unknown] {
            let plan = select_update_plan(source, ReleaseTarget::WindowsX64).unwrap();
            assert_eq!(plan.strategy, UpdateStrategy::MsiGlobal);
            assert_eq!(plan.artifact, "honk300-x86_64-pc-windows-msvc.msi");
        }
    }

    #[test]
    fn powershell_source_updates_with_powershell_installer_not_cargo() {
        for source in [InstallSource::PowerShell, InstallSource::Shell] {
            let plan = select_update_plan(source, ReleaseTarget::WindowsX64).unwrap();
            assert_eq!(plan.strategy, UpdateStrategy::PowerShell);
            assert_eq!(plan.artifact, "honk300-installer.ps1");
            assert_ne!(plan.strategy.label(), "cargo install");
        }
    }

    #[test]
    fn linux_arches_use_shell_installer() {
        for target in [
            ReleaseTarget::LinuxX64Gnu,
            ReleaseTarget::LinuxArm64Gnu,
            ReleaseTarget::LinuxX64Musl,
            ReleaseTarget::LinuxArm64Musl,
        ] {
            let plan = select_update_plan(InstallSource::Unknown, target).unwrap();
            assert_eq!(plan.strategy, UpdateStrategy::Shell);
            assert_eq!(plan.artifact, "honk300-installer.sh");
        }
    }

    #[test]
    fn all_macos_installs_update_via_the_managed_shell_installer() {
        for target in [ReleaseTarget::MacosX64, ReleaseTarget::MacosArm64] {
            let app = select_update_plan(InstallSource::MacApp, target).unwrap();
            assert_eq!(app.strategy, UpdateStrategy::Shell);
            assert_eq!(app.artifact, "honk300-installer.sh");

            for source in [
                InstallSource::Shell,
                InstallSource::PowerShell,
                InstallSource::ManualLocal,
                InstallSource::Unknown,
            ] {
                let plan = select_update_plan(source, target).unwrap();
                assert_eq!(plan.strategy, UpdateStrategy::Shell);
                assert_eq!(plan.artifact, "honk300-installer.sh");
            }
        }
    }

    #[test]
    fn release_target_triples_cover_m19_non_macos_matrix() {
        assert_eq!(ReleaseTarget::WindowsX64.triple(), "x86_64-pc-windows-msvc");
        assert_eq!(
            ReleaseTarget::WindowsArm64.triple(),
            "aarch64-pc-windows-msvc"
        );
        assert_eq!(
            ReleaseTarget::LinuxX64Gnu.triple(),
            "x86_64-unknown-linux-gnu"
        );
        assert_eq!(
            ReleaseTarget::LinuxArm64Gnu.triple(),
            "aarch64-unknown-linux-gnu"
        );
        assert_eq!(
            ReleaseTarget::LinuxX64Musl.triple(),
            "x86_64-unknown-linux-musl"
        );
        assert_eq!(
            ReleaseTarget::LinuxArm64Musl.triple(),
            "aarch64-unknown-linux-musl"
        );
    }

    #[test]
    fn parse_sha256_sidecar_accepts_common_formats() {
        let hash = "abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789";
        assert_eq!(
            parse_sha256_sidecar(&format!("{hash}  *honk300.msi")).as_deref(),
            Some(hash)
        );
        assert_eq!(
            parse_sha256_sidecar(&format!("{} honk300.msi", hash.to_uppercase())).as_deref(),
            Some(hash)
        );
        assert_eq!(parse_sha256_sidecar("not-a-hash  file"), None);
    }

    #[test]
    fn checksum_verdict_refuses_mismatch() {
        let empty = "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855";
        assert!(checksum_verdict(empty, &empty.to_uppercase()).is_ok());
        let err = checksum_verdict(empty, "deadbeef").unwrap_err();
        assert!(err.contains("SHA256 mismatch"));
    }

    #[test]
    fn compute_sha256_matches_empty_file() {
        let dir = std::env::temp_dir().join(format!("honk300-update-hash-{}", std::process::id()));
        let _ = fs::create_dir_all(&dir);
        let path = dir.join("empty.bin");
        fs::write(&path, b"").unwrap();
        assert_eq!(
            compute_sha256(&path).unwrap(),
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn version_comparison_handles_prerelease_and_metadata() {
        assert!(is_newer("0.0.0", "0.0.1"));
        assert!(is_newer("0.0.1-rc.1", "0.0.1"));
        assert!(is_newer("0.0.0", "0.0.1-rc.1"));
        assert!(!is_newer("0.0.1", "0.0.1+sha.abc"));
        assert!(!is_newer("1.0.0", "0.9.9"));
        assert_eq!(strip_prerelease_metadata("v1.2.3-rc.4"), "1.2.3");
    }

    #[test]
    fn post_install_version_accepts_matching_core() {
        assert!(post_install_version_ok("0.1.0+sha.abc", "0.1.0"));
        assert!(!post_install_version_ok("0.0.9", "0.1.0"));
    }

    #[test]
    fn windows_update_handoff_is_hidden_waits_reverifies_and_cleans_only_owned_temp() {
        let artifact = Path::new(r"C:\Users\goose\AppData\Local\Temp\honk300-update.msi");
        let installed = Path::new(r"C:\Program Files\honk300\bin\honk300.exe");
        let hash = "abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789";
        let invocation = windows_update_helper_invocation(WindowsUpdateHelperRequest {
            current_pid: 4242,
            strategy: UpdateStrategy::MsiGlobal,
            artifact,
            expected_hash: hash,
            expected_size: 1234,
            installed_executable: installed,
            expected_version: "0.3.0",
            system_msiexec: Path::new(r"C:\Windows\System32\msiexec.exe"),
            system_powershell: Path::new(
                r"C:\Windows\System32\WindowsPowerShell\v1.0\powershell.exe",
            ),
        });

        assert!(invocation
            .args
            .windows(2)
            .any(|args| args == ["-WindowStyle", "Hidden"]));
        assert!(invocation.script.contains("Wait-Process -Id 4242"));
        assert!(invocation.script.contains("Get-FileHash"));
        assert!(invocation.script.contains(hash));
        assert!(invocation
            .script
            .contains(r"C:\Windows\System32\msiexec.exe"));
        assert!(!invocation.script.contains("-FilePath 'msiexec.exe'"));
        assert!(invocation.script.contains("/i"));
        assert!(invocation.script.contains(&installed.to_string_lossy()[..]));
        assert!(invocation.script.contains("--version"));
        assert!(invocation.script.contains("Remove-Item -LiteralPath"));
        assert!(!invocation.script.contains("Remove-Item -Recurse"));
        assert!(!invocation
            .script
            .contains(r"Remove-Item -LiteralPath 'C:\Program Files"));
    }

    #[test]
    fn windows_post_install_verification_uses_strategy_owned_path_not_path_lookup() {
        let program_files = Path::new(r"C:\Program Files");
        let current = Path::new(r"C:\Users\goose\AppData\Local\Programs\honk300\bin\honk300.exe");
        assert!(is_absolute_windows_path(current));
        assert!(is_absolute_windows_path(Path::new(
            r"\\server\share\honk300\bin\honk300.exe"
        )));
        assert!(!is_absolute_windows_path(Path::new(
            r"honk300\bin\honk300.exe"
        )));
        let global = windows_strategy_executable(
            UpdateStrategy::MsiGlobal,
            InstallSource::MsiGlobal,
            current,
            program_files,
        )
        .unwrap();
        assert_eq!(
            global.to_string_lossy().replace('/', "\\"),
            r"C:\Program Files\honk300\bin\honk300.exe"
        );

        let corporate = windows_strategy_executable(
            UpdateStrategy::MsiCorporate,
            InstallSource::MsiCorporate,
            current,
            program_files,
        )
        .unwrap();
        assert_eq!(corporate, current);
        assert_ne!(corporate, PathBuf::from("honk300"));
    }

    #[test]
    fn unix_post_install_verification_uses_receipt_install_root() {
        assert_eq!(
            unix_strategy_executable(
                ReleaseTarget::LinuxX64Gnu,
                Path::new("/home/goose/.local/share/honk300/install"),
            ),
            PathBuf::from("/home/goose/.local/share/honk300/install/bin/honk300")
        );
        assert_eq!(
            unix_strategy_executable(
                ReleaseTarget::MacosArm64,
                Path::new("/Users/goose/Applications/Honk300.app"),
            ),
            PathBuf::from("/Users/goose/Applications/Honk300.app/Contents/MacOS/honk300")
        );
    }
}
