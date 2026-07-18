#[cfg(windows)]
use crate::install::detected_windows_install_root;
use crate::install::{detect_install_source, InstallSource};
#[cfg(windows)]
use crate::install::{system_windows_msiexec_path, system_windows_powershell_path};
#[cfg(not(windows))]
use honk_control::LifecycleLease;
use sha2::{Digest, Sha256};
#[cfg(any(test, windows))]
use std::ffi::OsString;
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
    Deb,
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
            Self::Deb => "Debian package",
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
    strategy: UpdateStrategy,
    artifact: &'a Path,
    expected_hash: &'a str,
    expected_size: u64,
    lifecycle_archive: &'a Path,
    lifecycle_expected_hash: &'a str,
    lifecycle_expected_size: u64,
    installed_executable: &'a Path,
    expected_version: &'a str,
    system_msiexec: &'a Path,
    system_powershell: &'a Path,
}

#[cfg(windows)]
struct WindowsUpdateDownloads<'a> {
    artifact_path: &'a Path,
    artifact: &'a ReleaseArtifact,
    lifecycle_archive_path: &'a Path,
    lifecycle_artifact: &'a ReleaseArtifact,
}

#[cfg(any(test, windows))]
#[derive(Debug, Clone, PartialEq, Eq)]
struct WindowsWebRequestInvocation {
    args: Vec<OsString>,
    environment: Vec<(&'static str, OsString)>,
}

#[cfg(any(test, windows))]
fn windows_fetch_text_invocation(url: &str) -> WindowsWebRequestInvocation {
    WindowsWebRequestInvocation {
        args: vec![
            "-NoProfile".into(),
            "-NonInteractive".into(),
            "-ExecutionPolicy".into(),
            "Bypass".into(),
            "-Command".into(),
            "$ErrorActionPreference='Stop'; $ProgressPreference='SilentlyContinue'; $response=Invoke-WebRequest -UseBasicParsing -Uri $env:HONK300_INTERNAL_WEB_REQUEST_URI -Headers @{ 'User-Agent' = 'honk300' }; $content=$response.Content; if ($content -is [byte[]]) { $content=[Text.Encoding]::UTF8.GetString($content) }; [Console]::OutputEncoding=[Text.UTF8Encoding]::new($false); [Console]::Out.Write([string]$content)".into(),
        ],
        environment: vec![("HONK300_INTERNAL_WEB_REQUEST_URI", url.into())],
    }
}

#[cfg(any(test, windows))]
fn windows_download_invocation(url: &str, path: &Path) -> WindowsWebRequestInvocation {
    WindowsWebRequestInvocation {
        args: vec![
            "-NoProfile".into(),
            "-NonInteractive".into(),
            "-ExecutionPolicy".into(),
            "Bypass".into(),
            "-Command".into(),
            "$ErrorActionPreference='Stop'; $ProgressPreference='SilentlyContinue'; Invoke-WebRequest -UseBasicParsing -Uri $env:HONK300_INTERNAL_WEB_REQUEST_URI -OutFile $env:HONK300_INTERNAL_WEB_REQUEST_OUT -Headers @{ 'User-Agent' = 'honk300' }".into(),
        ],
        environment: vec![
            ("HONK300_INTERNAL_WEB_REQUEST_URI", url.into()),
            (
                "HONK300_INTERNAL_WEB_REQUEST_OUT",
                path.as_os_str().to_os_string(),
            ),
        ],
    }
}

#[cfg(any(test, windows))]
const WINDOWS_PINNED_FILE_HELPER: &str = r#"
function Open-HonkPinnedFile([string] $Path, [int64] $ExpectedSize, [string] $ExpectedHash, [string] $Label) {
  $item=Get-Item -LiteralPath $Path -Force -ErrorAction Stop
  if ($item.PSIsContainer -or (($item.Attributes -band [IO.FileAttributes]::ReparsePoint) -ne 0)) { throw "$Label is not a regular file" }
  $stream=[IO.File]::Open($Path,[IO.FileMode]::Open,[IO.FileAccess]::Read,[IO.FileShare]::Read)
  try {
    if ($stream.Length -ne $ExpectedSize) { throw "$Label size changed" }
    $sha=[Security.Cryptography.SHA256]::Create()
    try { $stream.Position=0; $digest=$sha.ComputeHash($stream) } finally { $sha.Dispose() }
    $actual=[BitConverter]::ToString($digest).Replace('-','').ToLowerInvariant()
    if ($actual -ne $ExpectedHash) { throw "$Label hash changed" }
    $stream.Position=0
    return $stream
  } catch { $stream.Dispose(); throw }
}
function Get-HonkFileSha256([string] $Path, [string] $Label) {
  $item=Get-Item -LiteralPath $Path -Force -ErrorAction Stop
  if ($item.PSIsContainer -or (($item.Attributes -band [IO.FileAttributes]::ReparsePoint) -ne 0)) { throw "$Label is not a regular file" }
  $stream=[IO.File]::Open($Path,[IO.FileMode]::Open,[IO.FileAccess]::Read,[IO.FileShare]::Read)
  try {
    $sha=[Security.Cryptography.SHA256]::Create()
    try { $digest=$sha.ComputeHash($stream) } finally { $sha.Dispose() }
    return [BitConverter]::ToString($digest).Replace('-','').ToLowerInvariant()
  } finally { $stream.Dispose() }
}
"#;

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
        UpdateStrategy::Shell | UpdateStrategy::Deb => {
            Err("Unix updates are not a Windows strategy".into())
        }
    }
}

#[cfg(any(test, windows))]
fn marker_derived_windows_executable(
    source: InstallSource,
    expected_source: InstallSource,
    current_exe: &Path,
) -> Result<PathBuf, String> {
    let windows_path = current_exe.to_string_lossy().replace('/', "\\");
    let normalized = windows_path.to_ascii_lowercase();
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
    let parent = windows_path
        .rsplit_once('\\')
        .map(|(parent, _)| parent)
        .ok_or_else(|| {
            format!(
                "cannot derive the owned post-update executable for {}",
                expected_source.marker_value()
            )
        })?;
    Ok(PathBuf::from(format!("{parent}\\honk300.exe")))
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
        strategy,
        artifact,
        expected_hash,
        expected_size,
        lifecycle_archive,
        lifecycle_expected_hash,
        lifecycle_expected_size,
        installed_executable,
        expected_version,
        system_msiexec,
        system_powershell,
    } = request;
    let artifact_literal = powershell_literal(&artifact.to_string_lossy());
    let lifecycle_archive_literal = powershell_literal(&lifecycle_archive.to_string_lossy());
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
        UpdateStrategy::Deb => unreachable!("Debian updates never use the Windows handoff"),
    };
    let elevation = if elevated { " -Verb RunAs" } else { "" };
    let installer_literal = powershell_literal(&installer);
    let expected_hash = powershell_literal(&expected_hash.to_ascii_lowercase());
    let lifecycle_expected_hash = powershell_literal(&lifecycle_expected_hash.to_ascii_lowercase());
    let expected_version = powershell_literal(strip_prerelease_metadata(expected_version));
    let delegate_reacquires = if strategy == UpdateStrategy::PowerShell {
        "$true"
    } else {
        "$false"
    };
    let pinned_file_helper = WINDOWS_PINNED_FILE_HELPER;
    let script = format!(
        "$ErrorActionPreference='Stop'; \
         {pinned_file_helper} \
         $artifact='{artifact_literal}'; $expectedHash='{expected_hash}'; $expectedSize=[int64]{expected_size}; \
         $lifecycleArchive='{lifecycle_archive_literal}'; $lifecycleExpectedHash='{lifecycle_expected_hash}'; $lifecycleExpectedSize=[int64]{lifecycle_expected_size}; \
         $artifactOwned=$false; $lifecycleOwned=$false; $artifactStream=$null; $lifecycleStream=$null; $lease=$null; $leaseRoot=$null; $delegateReacquires={delegate_reacquires}; \
         try {{ \
           $artifactStream=Open-HonkPinnedFile $artifact $expectedSize $expectedHash 'verified update artifact'; \
           $artifactOwned=$true; \
           $lifecycleStream=Open-HonkPinnedFile $lifecycleArchive $lifecycleExpectedSize $lifecycleExpectedHash 'verified lifecycle archive'; \
           $lifecycleOwned=$true; \
           $leaseRoot=Join-Path ([IO.Path]::GetTempPath()) ('honk300-update-lease-' + [guid]::NewGuid().ToString('N')); \
           New-Item -ItemType Directory -Path $leaseRoot -ErrorAction Stop | Out-Null; \
           Add-Type -AssemblyName System.IO.Compression; \
           Add-Type -AssemblyName System.IO.Compression.FileSystem; \
           $zip=[IO.Compression.ZipArchive]::new($lifecycleStream,[IO.Compression.ZipArchiveMode]::Read,$true); \
           try {{ $entries=@($zip.Entries | Where-Object {{ $_.Name -ieq 'honk300.exe' }}); if ($entries.Count -ne 1) {{ throw \"portable lifecycle archive must contain exactly one honk300.exe; found $($entries.Count)\" }}; $entry=$entries[0]; $segments=@($entry.FullName.Replace('\','/').Split('/') | Where-Object {{ $_ -ne '' }}); if ($entry.FullName.Contains('\') -or $entry.FullName.Contains(':') -or $segments.Count -eq 0 -or $segments -contains '.' -or $segments -contains '..') {{ throw 'portable lifecycle archive contains an unsafe executable path' }}; $leaseBinary=Join-Path $leaseRoot 'lifecycle-honk300.exe'; [IO.Compression.ZipFileExtensions]::ExtractToFile($entry,$leaseBinary,$false) }} finally {{ $zip.Dispose() }}; \
           $start=[Diagnostics.ProcessStartInfo]::new(); $start.FileName=$leaseBinary; $start.UseShellExecute=$false; $start.CreateNoWindow=$true; $start.RedirectStandardInput=$true; $start.RedirectStandardOutput=$true; $start.RedirectStandardError=$true; $start.EnvironmentVariables['HONK300_INTERNAL_HOLD_LIFECYCLE_LEASE']='1'; \
           $lease=[Diagnostics.Process]::new(); $lease.StartInfo=$start; if (-not $lease.Start()) {{ throw 'failed to start verified lifecycle lease holder' }}; \
           $ready=$lease.StandardOutput.ReadLine(); if ($ready -ne 'HONK300_INTERNAL_LIFECYCLE_LEASE_READY') {{ $failure=$lease.StandardError.ReadToEnd().Trim(); throw \"verified lifecycle helper could not acquire exclusive runtime ownership: $failure\" }}; \
           if ($delegateReacquires) {{ \
             $lease.StandardInput.Close(); \
             if (-not $lease.WaitForExit(10000)) {{ $lease.Kill(); $lease.WaitForExit(); $lease.Dispose(); $lease=$null; throw 'outer lifecycle helper did not release before delegated bootstrap' }}; \
             $leaseExit=$lease.ExitCode; $lease.Dispose(); $lease=$null; \
             if ($leaseExit -ne 0) {{ throw \"outer lifecycle helper exited with code $leaseExit before delegated bootstrap\" }} \
           }}; \
           $process=Start-Process -FilePath '{installer_literal}' -ArgumentList {arguments} -WindowStyle Hidden -Wait -PassThru{elevation}; \
           if ($process.ExitCode -ne 0) {{ throw \"installer exited with code $($process.ExitCode); pending or reboot-deferred replacement is not accepted\" }}; \
           if (-not (Test-Path -LiteralPath '{installed_literal}' -PathType Leaf)) {{ throw 'installed honk300 executable is missing' }}; \
           $versionStart=[Diagnostics.ProcessStartInfo]::new(); $versionStart.FileName='{installed_literal}'; $versionStart.Arguments='--version'; $versionStart.UseShellExecute=$false; $versionStart.CreateNoWindow=$true; $versionStart.RedirectStandardOutput=$true; $versionStart.RedirectStandardError=$true; \
           $versionProcess=[Diagnostics.Process]::new(); $versionProcess.StartInfo=$versionStart; if (-not $versionProcess.Start()) {{ throw 'installed honk300 version verification could not start' }}; $versionOutput=$versionProcess.StandardOutput.ReadToEnd(); $versionError=$versionProcess.StandardError.ReadToEnd(); $versionProcess.WaitForExit(); if ($versionProcess.ExitCode -ne 0 -or [string]::IsNullOrWhiteSpace($versionOutput)) {{ throw ('installed honk300 version check failed: ' + $versionError) }}; \
           $reported=($versionOutput.Trim().Split()[-1] -replace '[+-].*$',''); \
           if ($reported -ne '{expected_version}') {{ throw \"installed honk300 version $reported does not match {expected_version}\" }} \
         }} finally {{ \
           if ($null -ne $lease) {{ try {{ $lease.StandardInput.Close(); if (-not $lease.WaitForExit(10000)) {{ $lease.Kill(); $lease.WaitForExit() }} }} finally {{ $lease.Dispose() }} }}; \
           if ($null -ne $lifecycleStream) {{ $lifecycleStream.Dispose() }}; \
           if ($null -ne $artifactStream) {{ $artifactStream.Dispose() }}; \
           if ($null -ne $leaseRoot -and (Test-Path -LiteralPath $leaseRoot -PathType Container)) {{ Remove-Item -LiteralPath $leaseRoot -Recurse -Force }}; \
           if ($artifactOwned -and (Test-Path -LiteralPath $artifact -PathType Leaf)) {{ $cleanup=Get-Item -LiteralPath $artifact -Force; if (($cleanup.Attributes -band [IO.FileAttributes]::ReparsePoint) -eq 0 -and $cleanup.Length -eq $expectedSize -and (Get-HonkFileSha256 $artifact 'verified update artifact cleanup') -eq $expectedHash) {{ Remove-Item -LiteralPath $artifact -Force }} }}; \
           if ($lifecycleOwned -and (Test-Path -LiteralPath $lifecycleArchive -PathType Leaf)) {{ $cleanup=Get-Item -LiteralPath $lifecycleArchive -Force; if (($cleanup.Attributes -band [IO.FileAttributes]::ReparsePoint) -eq 0 -and $cleanup.Length -eq $lifecycleExpectedSize -and (Get-HonkFileSha256 $lifecycleArchive 'verified lifecycle archive cleanup') -eq $lifecycleExpectedHash) {{ Remove-Item -LiteralPath $lifecycleArchive -Force }} }} \
         }}"
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

#[derive(Debug, Default)]
struct UpdateReport {
    origin: Option<String>,
    previous_version: String,
    installed_version: Option<String>,
    target: Option<String>,
    artifact: Option<String>,
    result: String,
    activation_state: String,
    cleanup_state: String,
    message: String,
}

impl UpdateReport {
    fn new() -> Self {
        Self {
            previous_version: env!("CARGO_PKG_VERSION").to_owned(),
            installed_version: Some(env!("CARGO_PKG_VERSION").to_owned()),
            result: "failed".into(),
            activation_state: "unchanged".into(),
            cleanup_state: "clean".into(),
            ..Self::default()
        }
    }

    fn json(&self) -> serde_json::Value {
        serde_json::json!({
            "action": "update",
            "success": matches!(self.result.as_str(), "updated" | "up_to_date"),
            "origin": self.origin,
            "previous_version": self.previous_version,
            "installed_version": self.installed_version,
            "target": self.target,
            "artifact": self.artifact,
            "result": self.result,
            "activation_state": self.activation_state,
            "cleanup_state": self.cleanup_state,
            "message": self.message,
        })
    }
}

pub fn run(json: bool) -> Result<(), DynError> {
    let mut report = UpdateReport::new();
    let result = run_inner(&mut report, json);
    if let Err(error) = &result {
        report.message = error.to_string();
    }
    if json {
        println!("{}", serde_json::to_string(&report.json())?);
    } else if result.is_ok() {
        eprintln!("{}", report.message);
    }
    result
}

fn run_inner(report: &mut UpdateReport, json: bool) -> Result<(), DynError> {
    eprintln!("honk300: checking for updates...");
    let manifest = fetch_latest_release_manifest()?;
    let latest = manifest.version.clone();
    let current = env!("CARGO_PKG_VERSION");
    let target = current_release_target()
        .ok_or("honk300 update: this OS/architecture is not part of the M19 release matrix")?;
    let source = detect_install_source();
    report.origin = Some(source.marker_value().into());
    report.target = Some(target.triple().into());
    let plan = select_update_plan(source, target)?;
    report.artifact = Some(plan.artifact.clone());
    #[allow(unused_mut)]
    let mut cleanup_retried = false;
    #[cfg(windows)]
    if let Some(root) = crate::install::detected_windows_install_root(source)? {
        if crate::install::windows_owner_cleanup_is_pending(&root)
            || crate::install::discover_windows_owner_cleanup(&root)?
        {
            eprintln!("honk300: retrying pending conflicting-owner cleanup...");
            if let Err(error) = crate::install::retry_windows_owner_cleanup(&root, source) {
                report.installed_version = Some(current.into());
                report.result = "cleanup_pending".into();
                report.activation_state = "verified".into();
                report.cleanup_state = "pending".into();
                return Err(error);
            }
            report.cleanup_state = "clean".into();
            cleanup_retried = true;
        }
    }
    if !is_newer(current, &latest) {
        report.installed_version = Some(current.into());
        report.result = "up_to_date".into();
        report.activation_state = if cleanup_retried {
            "verified".into()
        } else {
            "unchanged".into()
        };
        report.message = if cleanup_retried {
            format!(
                "honk300: retired the pending conflicting owner and verified version {current}."
            )
        } else {
            format!("honk300: already on the latest version ({current}).")
        };
        return Ok(());
    }

    let artifact = manifest_artifact(&manifest, &plan, target)?;
    let artifact_url = artifact_url_for_tag(&manifest.tag, &artifact.name)?;
    let temp_path = temp_artifact_path(&latest, &plan.artifact);
    #[cfg(windows)]
    let installed_executable = strategy_owned_executable(plan.strategy, source, target)?;

    eprintln!(
        "honk300: update available {current} -> {latest}; using {}.",
        plan.strategy.label()
    );
    prepare_temp_artifact_path(&temp_path)?;
    download_to_file(&artifact_url, &temp_path)?;
    verify_manifest_artifact(&temp_path, artifact)?;

    #[cfg(windows)]
    {
        let lifecycle_artifact = windows_lifecycle_artifact(&manifest, target)?;
        let lifecycle_url = artifact_url_for_tag(&manifest.tag, &lifecycle_artifact.name)?;
        let lifecycle_temp_path = temp_artifact_path(&latest, &lifecycle_artifact.name);
        prepare_temp_artifact_path(&lifecycle_temp_path)?;
        download_to_file(&lifecycle_url, &lifecycle_temp_path)?;
        verify_manifest_artifact(&lifecycle_temp_path, lifecycle_artifact)?;
        if let Err(error) = run_windows_update(
            plan.strategy,
            WindowsUpdateDownloads {
                artifact_path: &temp_path,
                artifact,
                lifecycle_archive_path: &lifecycle_temp_path,
                lifecycle_artifact,
            },
            &installed_executable,
            &manifest,
            target,
            json,
        ) {
            let message = error.to_string();
            if message.contains("cleanup remains pending") || message.contains("cleanup_pending") {
                report.installed_version = Some(latest.clone());
                report.result = "cleanup_pending".into();
                report.activation_state = "verified".into();
                report.cleanup_state = "pending".into();
            }
            return Err(error);
        }
        report.installed_version = Some(latest.clone());
        report.result = "updated".into();
        report.activation_state = "verified".into();
        report.cleanup_state = "inactive_releases_retained".into();
        report.message = format!("honk300: updated to {latest}.");
        Ok(())
    }

    #[cfg(not(windows))]
    {
        let _lifecycle_lease = if plan.strategy == UpdateStrategy::Deb {
            Some(LifecycleLease::acquire()?)
        } else {
            None
        };
        run_installer(plan.strategy, source, &temp_path, json)?;
        let activated_executable = strategy_owned_executable(plan.strategy, source, target)?;
        verify_post_install(&activated_executable, &latest)?;
        remove_verified_temp_artifact(&temp_path, artifact)?;
        report.installed_version = Some(latest.clone());
        report.result = "updated".into();
        report.activation_state = "verified".into();
        report.message = format!("honk300: updated to {latest}.");
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
        if matches!(
            source,
            InstallSource::MsiGlobal
                | InstallSource::MsiCorporate
                | InstallSource::ExeGlobal
                | InstallSource::ExeCorporate
                | InstallSource::Deb
        ) {
            return Err(format!(
                "{} install provenance is incompatible with {}",
                source.marker_value(),
                target.triple()
            ));
        }
        // The signed DMG remains the fresh graphical install. Only an owned app bundle or an
        // explicitly shell-managed install may use the exact-tag app ZIP/bootstrap transaction.
        // A mounted DMG, foreign app, or bare executable must not be silently claimed.
        return match source {
            InstallSource::MacApp | InstallSource::Shell => Ok(UpdatePlan {
                strategy: UpdateStrategy::Shell,
                artifact: "honk300-installer.sh".into(),
            }),
            _ => Err(format!(
                "{} install provenance is not managed; open https://github.com/RealEmmettS/goose/releases/latest/download/honk300-universal2.dmg and reinstall Honk300",
                source.marker_value(),
            )),
        };
    }
    if target.is_linux() {
        if matches!(
            source,
            InstallSource::MsiGlobal
                | InstallSource::MsiCorporate
                | InstallSource::ExeGlobal
                | InstallSource::ExeCorporate
                | InstallSource::PowerShell
                | InstallSource::MacApp
        ) {
            return Err(format!(
                "{} install provenance is incompatible with {}",
                source.marker_value(),
                target.triple()
            ));
        }
        if source == InstallSource::Deb {
            let artifact = match target {
                ReleaseTarget::LinuxX64Gnu => "honk300-amd64.deb",
                ReleaseTarget::LinuxArm64Gnu => "honk300-arm64.deb",
                _ => {
                    return Err(format!(
                        "Debian package provenance is incompatible with {}",
                        target.triple()
                    ))
                }
            };
            return Ok(UpdatePlan {
                strategy: UpdateStrategy::Deb,
                artifact: artifact.into(),
            });
        }
        return match source {
            InstallSource::Shell => Ok(UpdatePlan {
                strategy: UpdateStrategy::Shell,
                artifact: "honk300-installer.sh".into(),
            }),
            _ => Err(format!(
                "{} install provenance is not managed; download https://github.com/RealEmmettS/goose/releases/latest/download/honk300-installer.sh and run `sh honk300-installer.sh`",
                source.marker_value(),
            )),
        };
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
        InstallSource::ManualLocal | InstallSource::Unknown => {
            return Err(format!(
                "{} install provenance is not authoritative; download and run https://github.com/RealEmmettS/goose/releases/latest/download/honk300-{triple}.msi",
                source.marker_value(),
            ))
        }
        InstallSource::MacApp | InstallSource::Deb => {
            return Err(format!(
                "{} install provenance is incompatible with {}",
                source.marker_value(),
                target.triple()
            ));
        }
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
fn run_windows_update(
    strategy: UpdateStrategy,
    downloads: WindowsUpdateDownloads<'_>,
    installed_executable: &Path,
    manifest: &ReleaseManifest,
    target: ReleaseTarget,
    suppress_stdout: bool,
) -> Result<(), DynError> {
    use std::os::windows::process::CommandExt;

    const CREATE_NO_WINDOW: u32 = 0x0800_0000;
    let WindowsUpdateDownloads {
        artifact_path,
        artifact,
        lifecycle_archive_path,
        lifecycle_artifact,
    } = downloads;
    let powershell = system_windows_powershell_path()?;
    let msiexec = system_windows_msiexec_path()?;
    let invocation = windows_update_helper_invocation(WindowsUpdateHelperRequest {
        strategy,
        artifact: artifact_path,
        expected_hash: &artifact.sha256,
        expected_size: artifact.size,
        lifecycle_archive: lifecycle_archive_path,
        lifecycle_expected_hash: &lifecycle_artifact.sha256,
        lifecycle_expected_size: lifecycle_artifact.size,
        installed_executable,
        expected_version: &manifest.version,
        system_msiexec: &msiexec,
        system_powershell: &powershell,
    });
    let mut coordinator = Command::new(powershell);
    coordinator
        .args(invocation.args)
        .creation_flags(CREATE_NO_WINDOW);
    if suppress_stdout {
        coordinator.stdout(std::process::Stdio::null());
    }
    let status = coordinator.status()?;
    let verification = verify_windows_coordinator_result(
        installed_executable,
        strategy,
        artifact,
        manifest,
        target,
    );
    if status.success() {
        return verification;
    }
    match verification {
        Err(error) if error.to_string().contains("cleanup remains pending") => Err(error),
        Ok(()) => Err(format!(
            "honk300 update: selected release is activated and verified, but the installer exited with {}; cleanup_pending",
            status.code().unwrap_or(-1)
        )
        .into()),
        Err(verification_error) => Err(format!(
            "honk300 update: verified Windows coordinator exited with {}; post-install verification also failed: {verification_error}",
            status.code().unwrap_or(-1)
        )
        .into()),
    }
}

#[cfg(windows)]
fn verify_windows_coordinator_result(
    installed_executable: &Path,
    strategy: UpdateStrategy,
    requested_artifact: &ReleaseArtifact,
    manifest: &ReleaseManifest,
    target: ReleaseTarget,
) -> Result<(), DynError> {
    let root = installed_executable
        .parent()
        .and_then(Path::parent)
        .ok_or("honk300 update: stable Windows command path has no owned root")?;
    let receipt_path = root.join("install-receipt.json");
    if root.join(".slot-transaction.json").exists()
        || root.join(".slot-committed.json").exists()
        || crate::install::windows_owner_cleanup_is_pending(root)
    {
        return Err(
            "honk300 update: selector activation is verified but cleanup remains pending".into(),
        );
    }
    let metadata = fs::symlink_metadata(&receipt_path)?;
    if !metadata.is_file() || metadata.file_type().is_symlink() {
        return Err("honk300 update: protected Windows receipt is not a regular file".into());
    }
    let receipt: serde_json::Value = serde_json::from_slice(&fs::read(&receipt_path)?)?;
    let expected_origin = match strategy {
        UpdateStrategy::MsiGlobal => InstallSource::MsiGlobal,
        UpdateStrategy::MsiCorporate => InstallSource::MsiCorporate,
        UpdateStrategy::ExeGlobal => InstallSource::ExeGlobal,
        UpdateStrategy::ExeCorporate => InstallSource::ExeCorporate,
        UpdateStrategy::PowerShell => InstallSource::PowerShell,
        UpdateStrategy::Shell | UpdateStrategy::Deb => {
            return Err("honk300 update: Unix strategy reached Windows verification".into())
        }
    };
    let receipt_string = |key: &str| {
        receipt
            .get(key)
            .and_then(serde_json::Value::as_str)
            .ok_or_else(|| format!("honk300 update: protected receipt has no {key}"))
    };
    if receipt_string("schema")? != "honk300.install.v2"
        || InstallSource::from_marker(receipt_string("origin")?) != expected_origin
        || receipt_string("version")? != manifest.version
        || receipt_string("tag")? != manifest.tag
        || receipt_string("commit")? != manifest.commit
        || receipt_string("target")? != target.triple()
        || receipt_string("layout")? != "windows-slots-v1"
    {
        return Err(
            "honk300 update: activated receipt identity does not match the selected release".into(),
        );
    }
    let active_release = PathBuf::from(receipt_string("active_release")?);
    let expected_slot = format!("{}-{}", manifest.version, target.triple());
    if !active_release.starts_with(root)
        || active_release.file_name().and_then(|name| name.to_str()) != Some(expected_slot.as_str())
    {
        return Err("honk300 update: active Windows release slot is outside the owned root or has the wrong identity".into());
    }
    let receipt_artifact = receipt
        .get("artifact")
        .and_then(serde_json::Value::as_object)
        .ok_or("honk300 update: protected receipt has no artifact identity")?;
    if strategy != UpdateStrategy::PowerShell
        && (receipt_artifact
            .get("name")
            .and_then(serde_json::Value::as_str)
            != Some(requested_artifact.name.as_str())
            || receipt_artifact
                .get("sha256")
                .and_then(serde_json::Value::as_str)
                != Some(requested_artifact.sha256.as_str())
            || receipt_artifact
                .get("size")
                .and_then(serde_json::Value::as_u64)
                != Some(requested_artifact.size))
    {
        return Err(
            "honk300 update: protected receipt does not match the exact installer bytes".into(),
        );
    }
    let mut alias_hash = None;
    for name in ["honk300.exe", "honk.exe", "goose.exe"] {
        let alias = root.join("bin").join(name);
        verify_post_install(&alias, &manifest.version)?;
        let hash = compute_sha256(&alias)?;
        if alias_hash
            .as_ref()
            .is_some_and(|expected| expected != &hash)
        {
            return Err(
                "honk300 update: public aliases do not resolve to identical release bytes".into(),
            );
        }
        alias_hash = Some(hash);
    }
    let launcher_identity = receipt
        .get("app_launcher")
        .and_then(serde_json::Value::as_object)
        .ok_or("honk300 update: protected receipt has no Windows app-launcher identity")?;
    let launcher_path = launcher_identity
        .get("path")
        .and_then(serde_json::Value::as_str)
        .map(PathBuf::from)
        .ok_or("honk300 update: protected receipt has no Windows app-launcher path")?;
    let expected_launcher = root.join("bin").join("honk300-app.exe");
    if !launcher_path
        .to_string_lossy()
        .eq_ignore_ascii_case(&expected_launcher.to_string_lossy())
    {
        return Err(
            "honk300 update: protected receipt names an unexpected Windows app launcher".into(),
        );
    }
    let launcher_metadata = fs::symlink_metadata(&expected_launcher)?;
    if !launcher_metadata.is_file() || launcher_metadata.file_type().is_symlink() {
        return Err("honk300 update: Windows app launcher is not a regular file".into());
    }
    let launcher_hash = compute_sha256(&expected_launcher)?;
    if launcher_identity
        .get("sha256")
        .and_then(serde_json::Value::as_str)
        .is_none_or(|expected| !expected.eq_ignore_ascii_case(&launcher_hash))
    {
        return Err(
            "honk300 update: Windows app launcher does not match its protected receipt hash".into(),
        );
    }
    Ok(())
}

#[cfg(windows)]
fn strategy_owned_executable(
    strategy: UpdateStrategy,
    source: InstallSource,
    _target: ReleaseTarget,
) -> Result<PathBuf, DynError> {
    if let Some(executable) = windows_receipt_owned_executable(source)? {
        return Ok(executable);
    }
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
fn windows_receipt_owned_executable(source: InstallSource) -> Result<Option<PathBuf>, DynError> {
    Ok(detected_windows_install_root(source)?.map(|root| root.join("bin/honk300.exe")))
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
    source: InstallSource,
    target: ReleaseTarget,
) -> Result<PathBuf, DynError> {
    if strategy == UpdateStrategy::Deb {
        if source != InstallSource::Deb
            || !matches!(
                target,
                ReleaseTarget::LinuxX64Gnu | ReleaseTarget::LinuxArm64Gnu
            )
        {
            return Err(
                "honk300 update: Debian package provenance does not match this target".into(),
            );
        }
        return crate::debian::prove_current_executable(&std::env::current_exe()?);
    }
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
    let schema = value.get("schema").and_then(serde_json::Value::as_str);
    if !matches!(schema, Some("honk300.install.v1" | "honk300.install.v2")) {
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
    if schema == Some("honk300.install.v2") {
        let recorded_origin = value
            .get("origin")
            .and_then(serde_json::Value::as_str)
            .ok_or("honk300 update: v2 receipt has no origin")?;
        if InstallSource::from_marker(recorded_origin) != source {
            return Err("honk300 update: receipt origin conflicts with detected provenance".into());
        }
    }
    let active_release = if schema == Some("honk300.install.v2") {
        value
            .get("active_release")
            .and_then(serde_json::Value::as_str)
            .map(PathBuf::from)
            .ok_or("honk300 update: v2 receipt has no active_release")?
    } else {
        install_root.clone()
    };
    if !target.is_macos() && !paths_equivalent_or_within(&active_release, &install_root) {
        return Err("honk300 update: active release escapes the receipt-owned root".into());
    }
    Ok(unix_strategy_executable(target, &active_release))
}

#[cfg(not(windows))]
fn paths_equivalent(left: &Path, right: &Path) -> bool {
    match (left.canonicalize(), right.canonicalize()) {
        (Ok(left), Ok(right)) => left == right,
        _ => left == right,
    }
}

#[cfg(not(windows))]
fn paths_equivalent_or_within(path: &Path, root: &Path) -> bool {
    match (path.canonicalize(), root.canonicalize()) {
        (Ok(path), Ok(root)) => path.starts_with(root),
        _ => path.starts_with(root),
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
        UpdateStrategy::Deb => "deb",
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
        UpdateStrategy::Deb => artifact.target == target.triple(),
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

#[cfg(any(test, windows))]
fn windows_lifecycle_artifact(
    manifest: &ReleaseManifest,
    target: ReleaseTarget,
) -> Result<&ReleaseArtifact, String> {
    if !target.is_windows() {
        return Err("lifecycle portable helper is only valid for Windows updates".into());
    }
    let name = format!("honk300-{}.zip", target.triple());
    let artifact = manifest
        .artifacts
        .iter()
        .find(|artifact| artifact.name == name)
        .ok_or_else(|| format!("release manifest does not contain {name}"))?;
    if artifact.target != target.triple() || artifact.kind != "portable" {
        return Err(format!(
            "lifecycle helper {} must be the portable artifact for {}",
            artifact.name,
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
    let invocation = windows_fetch_text_invocation(url);
    let mut command = Command::new(system_windows_powershell_path()?);
    command.args(invocation.args);
    for (name, value) in invocation.environment {
        command.env(name, value);
    }
    let output = command.creation_flags(CREATE_NO_WINDOW).output()?;
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
    let invocation = windows_download_invocation(url, path);
    let mut command = Command::new(system_windows_powershell_path()?);
    command.args(invocation.args);
    for (name, value) in invocation.environment {
        command.env(name, value);
    }
    let status = command.creation_flags(CREATE_NO_WINDOW).status()?;
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

pub(crate) fn compute_sha256_for_install(path: &Path) -> io::Result<String> {
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

fn compute_sha256(path: &Path) -> io::Result<String> {
    compute_sha256_for_install(path)
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
fn run_installer(
    strategy: UpdateStrategy,
    source: InstallSource,
    path: &Path,
    suppress_stdout: bool,
) -> Result<(), DynError> {
    if strategy == UpdateStrategy::Deb {
        return crate::debian::install_package(path, suppress_stdout);
    }
    let mut command = match strategy {
        UpdateStrategy::MsiGlobal | UpdateStrategy::MsiCorporate => {
            let mut command = Command::new("msiexec");
            command
                .arg("/i")
                .arg(path)
                .arg("/passive")
                .arg("/norestart");
            command
        }
        UpdateStrategy::ExeGlobal | UpdateStrategy::ExeCorporate => {
            let mut command = Command::new(path);
            command.args(["/SILENT", "/SUPPRESSMSGBOXES", "/NORESTART"]);
            command
        }
        UpdateStrategy::PowerShell => {
            let mut command = Command::new("powershell");
            command
                .args(["-NoProfile", "-ExecutionPolicy", "Bypass", "-File"])
                .arg(path);
            command
        }
        UpdateStrategy::Shell => {
            let mut command = Command::new("sh");
            command
                .arg(path)
                .env("HONK300_UPDATE_ORIGIN", shell_update_origin(source)?);
            command
        }
        UpdateStrategy::Deb => unreachable!("handled before the platform installer match"),
    };
    let status = run_installer_command(&mut command, suppress_stdout)?;

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

#[cfg(any(test, not(windows)))]
fn shell_update_origin(source: InstallSource) -> Result<&'static str, DynError> {
    match source {
        InstallSource::MacApp => Ok("mac-app"),
        InstallSource::Shell => Ok("shell"),
        _ => Err(format!(
            "{} provenance cannot delegate to the managed shell updater",
            source.marker_value()
        )
        .into()),
    }
}

#[cfg(not(windows))]
fn run_installer_command(
    command: &mut Command,
    suppress_stdout: bool,
) -> io::Result<std::process::ExitStatus> {
    if suppress_stdout {
        return command.stdout(std::process::Stdio::null()).status();
    }
    let mut child = command.stdout(std::process::Stdio::piped()).spawn()?;
    let mut stdout = child
        .stdout
        .take()
        .ok_or_else(|| io::Error::other("installer stdout pipe was not created"))?;
    let forward = std::thread::spawn(move || io::copy(&mut stdout, &mut io::stderr()));
    let status = child.wait()?;
    forward
        .join()
        .map_err(|_| io::Error::other("installer stdout forwarding thread panicked"))??;
    Ok(status)
}

fn verify_post_install(installed_executable: &Path, latest: &str) -> Result<(), DynError> {
    let metadata = fs::symlink_metadata(installed_executable)?;
    if !metadata.is_file() || metadata.file_type().is_symlink() {
        return Err(format!(
            "installed executable is not a regular owned file: {}",
            installed_executable.display()
        )
        .into());
    }
    let mut command = Command::new(installed_executable);
    command.arg("--version");
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x0800_0000;
        command.creation_flags(CREATE_NO_WINDOW);
    }
    let output = command.output()?;
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
            },
            {
                "name": "honk300-x86_64-pc-windows-msvc.zip",
                "target": "x86_64-pc-windows-msvc",
                "kind": "portable",
                "sha256": "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef",
                "size": 4321,
                "checksum": "honk300-x86_64-pc-windows-msvc.zip.sha256"
            }
        ]
    }"#;

    #[test]
    fn release_manifest_validates_schema_tag_and_artifact_hashes() {
        let manifest = parse_release_manifest(VALID_MANIFEST, "v0.3.0").unwrap();
        assert_eq!(manifest.version, "0.3.0");
        assert_eq!(manifest.tag, "v0.3.0");
        assert_eq!(manifest.artifacts.len(), 2);
        assert_eq!(manifest.artifacts[0].kind, "msi-global");
        assert_eq!(
            windows_lifecycle_artifact(&manifest, ReleaseTarget::WindowsX64)
                .unwrap()
                .kind,
            "portable"
        );
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
    fn manual_and_unknown_windows_updates_fail_closed_instead_of_guessing_global_msi() {
        for source in [InstallSource::ManualLocal, InstallSource::Unknown] {
            let error = select_update_plan(source, ReleaseTarget::WindowsX64).unwrap_err();
            assert!(error.contains("not authoritative"));
            assert!(error.contains("releases/latest/download/honk300-x86_64-pc-windows-msvc.msi"));
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
    fn shell_managed_linux_arches_use_shell_installer_and_unknown_refuses() {
        for target in [
            ReleaseTarget::LinuxX64Gnu,
            ReleaseTarget::LinuxArm64Gnu,
            ReleaseTarget::LinuxX64Musl,
            ReleaseTarget::LinuxArm64Musl,
        ] {
            let plan = select_update_plan(InstallSource::Shell, target).unwrap();
            assert_eq!(plan.strategy, UpdateStrategy::Shell);
            assert_eq!(plan.artifact, "honk300-installer.sh");
            assert!(select_update_plan(InstallSource::Unknown, target).is_err());
        }
    }

    #[test]
    fn debian_provenance_updates_with_the_matching_stable_package_name() {
        for (target, artifact) in [
            (ReleaseTarget::LinuxX64Gnu, "honk300-amd64.deb"),
            (ReleaseTarget::LinuxArm64Gnu, "honk300-arm64.deb"),
        ] {
            let plan = select_update_plan(InstallSource::Deb, target).unwrap();
            assert_eq!(plan.strategy, UpdateStrategy::Deb);
            assert_eq!(plan.artifact, artifact);
            let manifest = ReleaseManifest {
                version: "1.0.1".into(),
                tag: "v1.0.1".into(),
                commit: "0".repeat(40),
                artifacts: vec![ReleaseArtifact {
                    name: artifact.into(),
                    target: target.triple().into(),
                    kind: "deb".into(),
                    sha256: "a".repeat(64),
                    size: 123,
                }],
            };
            assert_eq!(
                manifest_artifact(&manifest, &plan, target).unwrap().target,
                target.triple()
            );
            let mut wrong_kind = manifest.clone();
            wrong_kind.artifacts[0].kind = "portable".into();
            assert!(manifest_artifact(&wrong_kind, &plan, target).is_err());
            let mut wrong_target = manifest;
            wrong_target.artifacts[0].target = "a-different-target".into();
            assert!(manifest_artifact(&wrong_target, &plan, target).is_err());
        }
        assert!(select_update_plan(InstallSource::Deb, ReleaseTarget::LinuxX64Musl).is_err());
    }

    #[test]
    fn cross_platform_install_provenance_fails_closed() {
        assert!(select_update_plan(InstallSource::Deb, ReleaseTarget::WindowsX64).is_err());
        assert!(select_update_plan(InstallSource::MacApp, ReleaseTarget::LinuxX64Gnu).is_err());
        assert!(select_update_plan(InstallSource::MsiGlobal, ReleaseTarget::MacosArm64).is_err());
    }

    #[test]
    fn only_receipted_macos_installs_update_via_the_managed_shell_installer() {
        for target in [ReleaseTarget::MacosX64, ReleaseTarget::MacosArm64] {
            let app = select_update_plan(InstallSource::MacApp, target).unwrap();
            assert_eq!(app.strategy, UpdateStrategy::Shell);
            assert_eq!(app.artifact, "honk300-installer.sh");

            let shell = select_update_plan(InstallSource::Shell, target).unwrap();
            assert_eq!(shell.strategy, UpdateStrategy::Shell);
            assert_eq!(shell.artifact, "honk300-installer.sh");
            for source in [
                InstallSource::PowerShell,
                InstallSource::ManualLocal,
                InstallSource::Unknown,
            ] {
                assert!(select_update_plan(source, target).is_err());
            }
        }
    }

    #[test]
    fn shell_delegate_carries_only_the_proven_update_origin() {
        assert_eq!(
            shell_update_origin(InstallSource::MacApp).unwrap(),
            "mac-app"
        );
        assert_eq!(shell_update_origin(InstallSource::Shell).unwrap(), "shell");
        for source in [
            InstallSource::Deb,
            InstallSource::PowerShell,
            InstallSource::ManualLocal,
            InstallSource::Unknown,
        ] {
            assert!(shell_update_origin(source).is_err());
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
    fn windows_web_requests_keep_url_and_output_path_out_of_command_text() {
        let url = "https://example.invalid/release manifest.json?value='goose'";
        let output = Path::new(r"C:\Users\goose\AppData\Local\Temp\honk 'latest'.json");

        let fetch = windows_fetch_text_invocation(url);
        assert_eq!(
            fetch.environment,
            vec![("HONK300_INTERNAL_WEB_REQUEST_URI", OsString::from(url))]
        );
        assert!(fetch.args.iter().all(|argument| argument != url));
        let fetch_script = fetch
            .args
            .iter()
            .find(|argument| argument.to_string_lossy().contains("Invoke-WebRequest"))
            .unwrap()
            .to_string_lossy();
        assert!(fetch_script.contains("$content -is [byte[]]"));
        assert!(fetch_script.contains("[Text.Encoding]::UTF8.GetString($content)"));
        assert!(fetch_script.contains("[Console]::OutputEncoding"));

        let download = windows_download_invocation(url, output);
        assert_eq!(
            download.environment,
            vec![
                ("HONK300_INTERNAL_WEB_REQUEST_URI", OsString::from(url)),
                (
                    "HONK300_INTERNAL_WEB_REQUEST_OUT",
                    output.as_os_str().to_os_string()
                ),
            ]
        );
        assert!(download.args.iter().all(|argument| argument != url));
        assert!(download
            .args
            .iter()
            .all(|argument| argument != output.as_os_str()));
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
        assert!(is_newer("0.3.2", "1.0.1"));
        assert!(is_newer("1.0.1", "1.0.2"));
        assert!(!is_newer("0.0.1", "0.0.1+sha.abc"));
        assert!(!is_newer("1.0.1", "0.9.9"));
        assert!(!is_newer("1.0.2", "1.0.1"));
        assert_eq!(strip_prerelease_metadata("v1.2.3-rc.4"), "1.2.3");
    }

    #[test]
    fn post_install_version_accepts_matching_core() {
        assert!(post_install_version_ok("0.1.0+sha.abc", "0.1.0"));
        assert!(!post_install_version_ok("0.0.9", "0.1.0"));
    }

    #[test]
    fn json_report_is_one_complete_final_contract_object() {
        let mut report = UpdateReport::new();
        report.origin = Some("exe-global".into());
        report.installed_version = Some("1.2.3".into());
        report.target = Some("x86_64-pc-windows-msvc".into());
        report.artifact = Some("honk300-x86_64-pc-windows-msvc-setup.exe".into());
        report.result = "updated".into();
        report.activation_state = "verified".into();
        report.cleanup_state = "inactive_releases_retained".into();
        report.message = "honk300: updated to 1.2.3.".into();
        let value = report.json();
        let object = value.as_object().unwrap();
        assert_eq!(object.len(), 11);
        for key in [
            "action",
            "success",
            "origin",
            "previous_version",
            "installed_version",
            "target",
            "artifact",
            "result",
            "activation_state",
            "cleanup_state",
            "message",
        ] {
            assert!(object.contains_key(key), "missing JSON update field {key}");
        }
        assert_eq!(value["action"], "update");
        assert_eq!(value["success"], true);
    }

    #[test]
    fn windows_update_coordinator_is_hidden_synchronous_reverifies_and_cleans_owned_temp() {
        let artifact = Path::new(r"C:\Users\goose\AppData\Local\Temp\honk300-update.msi");
        let lifecycle_archive =
            Path::new(r"C:\Users\goose\AppData\Local\Temp\honk300-portable.zip");
        let installed = Path::new(r"C:\Program Files\honk300\bin\honk300.exe");
        let hash = "abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789";
        let invocation = windows_update_helper_invocation(WindowsUpdateHelperRequest {
            strategy: UpdateStrategy::MsiGlobal,
            artifact,
            expected_hash: hash,
            expected_size: 1234,
            lifecycle_archive,
            lifecycle_expected_hash: hash,
            lifecycle_expected_size: 4321,
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
        assert!(!invocation.script.contains("Wait-Process"));
        assert!(invocation
            .script
            .contains("HONK300_INTERNAL_HOLD_LIFECYCLE_LEASE"));
        assert!(invocation
            .script
            .contains("HONK300_INTERNAL_LIFECYCLE_LEASE_READY"));
        assert!(invocation.script.contains("RedirectStandardInput"));
        assert!(invocation.script.contains("[IO.FileShare]::Read"));
        assert!(invocation.script.contains("$sha.ComputeHash($stream)"));
        assert!(invocation
            .script
            .contains("[IO.Compression.ZipArchive]::new($lifecycleStream"));
        assert!(!invocation.script.contains("rstrtmgr.dll"));
        assert!(!invocation.script.contains("AssertUnlocked"));
        assert!(invocation.script.contains("$delegateReacquires=$false"));
        assert!(!invocation.script.contains("3010"));
        assert!(!invocation
            .script
            .contains("HONK300_INTERNAL_LIFECYCLE_LEASE_HELD"));
        assert!(invocation
            .script
            .contains(&lifecycle_archive.to_string_lossy()[..]));
        assert!(invocation.script.contains("function Get-HonkFileSha256"));
        assert!(!invocation.script.contains("Get-FileHash"));
        assert!(invocation.script.contains(hash));
        assert!(invocation
            .script
            .contains(r"C:\Windows\System32\msiexec.exe"));
        assert!(!invocation.script.contains("-FilePath 'msiexec.exe'"));
        assert!(invocation.script.contains("/i"));
        assert!(invocation.script.contains(&installed.to_string_lossy()[..]));
        assert!(invocation.script.contains("--version"));
        assert!(invocation
            .script
            .contains("$versionStart.CreateNoWindow=$true"));
        assert!(!invocation
            .script
            .contains("=(& 'C:\\Program Files\\honk300\\bin\\honk300.exe' --version"));
        assert!(!invocation.script.contains("Update-HonkBootstrapReceipt"));
        assert!(invocation.script.contains("Remove-Item -LiteralPath"));
        assert!(invocation
            .script
            .contains("Remove-Item -LiteralPath $leaseRoot -Recurse -Force"));
        assert!(!invocation
            .script
            .contains(r"Remove-Item -LiteralPath 'C:\Program Files"));
        assert!(
            invocation.script.find("Start-Process -FilePath").unwrap()
                < invocation
                    .script
                    .rfind("$artifactStream.Dispose()")
                    .unwrap()
        );

        let delegated = windows_update_helper_invocation(WindowsUpdateHelperRequest {
            strategy: UpdateStrategy::PowerShell,
            artifact,
            expected_hash: hash,
            expected_size: 1234,
            lifecycle_archive,
            lifecycle_expected_hash: hash,
            lifecycle_expected_size: 4321,
            installed_executable: installed,
            expected_version: "0.3.0",
            system_msiexec: Path::new(r"C:\Windows\System32\msiexec.exe"),
            system_powershell: Path::new(
                r"C:\Windows\System32\WindowsPowerShell\v1.0\powershell.exe",
            ),
        });
        assert!(delegated.script.contains("$delegateReacquires=$true"));
        assert!(!delegated
            .script
            .contains("HONK300_INTERNAL_LIFECYCLE_LEASE_HELD"));
        let release = delegated
            .script
            .find("$lease.StandardInput.Close()")
            .unwrap();
        let dispose = delegated
            .script
            .find("$leaseExit=$lease.ExitCode; $lease.Dispose(); $lease=$null")
            .unwrap();
        let invoke = delegated
            .script
            .find("$process=Start-Process -FilePath")
            .unwrap();
        assert!(release < dispose && dispose < invoke);

        // PowerShell is available on Windows CI and on the macOS qualification host. Keep the
        // assertions above portable, but syntax-check the fully rendered one-line handoff whenever
        // either supported executable is present.
        for shell in ["pwsh", "powershell"] {
            use std::io::Write as _;
            use std::process::Stdio;

            let parser = "$source=[Console]::In.ReadToEnd(); $tokens=$null; $errors=$null; [System.Management.Automation.Language.Parser]::ParseInput($source,[ref]$tokens,[ref]$errors) > $null; if ($errors.Count) { $errors | ForEach-Object { [Console]::Error.WriteLine($_.Message) }; exit 1 }";
            let mut child = match Command::new(shell)
                .args(["-NoProfile", "-Command", parser])
                .stdin(Stdio::piped())
                .stdout(Stdio::null())
                .stderr(Stdio::piped())
                .spawn()
            {
                Ok(child) => child,
                Err(error) if error.kind() == io::ErrorKind::NotFound => continue,
                Err(error) => panic!("failed to start {shell} syntax parser: {error}"),
            };
            child
                .stdin
                .take()
                .unwrap()
                .write_all(invocation.script.as_bytes())
                .unwrap();
            let output = child.wait_with_output().unwrap();
            assert!(
                output.status.success(),
                "generated Windows update helper is invalid PowerShell: {}",
                String::from_utf8_lossy(&output.stderr)
            );
            break;
        }
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

        let alias = Path::new(r"C:\Users\goose\AppData\Local\Programs\honk300\bin\goose.exe");
        assert_eq!(
            windows_strategy_executable(
                UpdateStrategy::MsiCorporate,
                InstallSource::MsiCorporate,
                alias,
                program_files,
            )
            .unwrap(),
            current
        );
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
