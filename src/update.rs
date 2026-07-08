use crate::install::{detect_install_source, InstallSource};
use sha2::{Digest, Sha256};
use std::fs;
use std::io::{self, Read};
use std::path::{Path, PathBuf};
use std::process::Command;

const RELEASES_URL: &str = "https://api.github.com/repos/RealEmmettS/goose/releases/latest";
const DOWNLOAD_BASE: &str = "https://github.com/RealEmmettS/goose/releases/latest/download";

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
    /// macOS `.app` install: replace `~/Applications/Honk300.app` from the universal2 DMG.
    MacDmg,
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
            Self::MacDmg => "universal2 .app DMG",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct UpdatePlan {
    strategy: UpdateStrategy,
    artifact: String,
}

pub fn run() -> Result<(), DynError> {
    println!("honk300: checking for updates...");
    let latest = fetch_latest_version()?;
    let current = env!("CARGO_PKG_VERSION");
    if !is_newer(current, &latest) {
        println!("honk300: already on the latest version ({current}).");
        return Ok(());
    }

    let target = current_release_target()
        .ok_or("honk300 update: this OS/architecture is not part of the M19 release matrix")?;
    let plan = select_update_plan(detect_install_source(), target)?;
    let artifact_url = artifact_url(&plan.artifact);
    let temp_path = temp_artifact_path(&latest, &plan.artifact);

    println!(
        "honk300: update available {current} -> {latest}; using {}.",
        plan.strategy.label()
    );
    download_to_file(&artifact_url, &temp_path)?;
    verify_downloaded_artifact(&temp_path, &artifact_url)?;
    run_installer(plan.strategy, &temp_path)?;
    verify_post_install(&latest)?;
    let _ = fs::remove_file(&temp_path);
    println!("honk300: updated to {latest}.");
    Ok(())
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
        // `.app` bundle installs replace the whole bundle from the universal2 DMG; every other
        // macOS provenance (cargo-dist shell/cargo-home/bare) re-runs the shell installer, exactly
        // like Linux.
        return Ok(match source {
            InstallSource::MacApp => UpdatePlan {
                strategy: UpdateStrategy::MacDmg,
                artifact: "honk300-universal2.dmg".into(),
            },
            _ => UpdatePlan {
                strategy: UpdateStrategy::Shell,
                artifact: "honk300-installer.sh".into(),
            },
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
        // `MacApp` never occurs on a Windows target; it shares the safe per-user EXE fallback.
        InstallSource::ManualLocal | InstallSource::Unknown | InstallSource::MacApp => UpdatePlan {
            strategy: UpdateStrategy::ExeCorporate,
            artifact: format!("honk300-{triple}-corporate-setup.exe"),
        },
        InstallSource::PowerShell | InstallSource::Shell => UpdatePlan {
            strategy: UpdateStrategy::PowerShell,
            artifact: "honk300-installer.ps1".into(),
        },
    };
    Ok(plan)
}

fn artifact_url(artifact: &str) -> String {
    format!("{DOWNLOAD_BASE}/{artifact}")
}

fn temp_artifact_path(latest: &str, artifact: &str) -> PathBuf {
    std::env::temp_dir().join(format!(
        "honk300-update-{}-{}",
        latest.replace(|c: char| !c.is_ascii_alphanumeric() && c != '.', "_"),
        artifact
    ))
}

fn fetch_latest_version() -> Result<String, String> {
    let body = fetch_text(RELEASES_URL).map_err(|err| err.to_string())?;
    let json: serde_json::Value = serde_json::from_str(&body)
        .map_err(|err| format!("failed to parse GitHub response: {err}"))?;
    let tag = json
        .get("tag_name")
        .and_then(|value| value.as_str())
        .ok_or("GitHub response did not include tag_name")?;
    Ok(tag.strip_prefix('v').unwrap_or(tag).to_string())
}

fn download_to_file(url: &str, path: &Path) -> Result<(), DynError> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    download_with_system_tool(url, path)
}

fn verify_downloaded_artifact(path: &Path, artifact_url: &str) -> Result<(), DynError> {
    let expected = fetch_text(&format!("{artifact_url}.sha256"))?;
    let expected =
        parse_sha256_sidecar(&expected).ok_or("downloaded SHA256 sidecar was malformed")?;
    let actual = compute_sha256(path)?;
    checksum_verdict(&expected, &actual).map_err(Into::into)
}

fn fetch_text(url: &str) -> Result<String, DynError> {
    fetch_text_with_system_tool(url)
}

#[cfg(windows)]
fn fetch_text_with_system_tool(url: &str) -> Result<String, DynError> {
    let output = Command::new("powershell")
        .args([
            "-NoProfile",
            "-ExecutionPolicy",
            "Bypass",
            "-Command",
            "param($uri) (Invoke-WebRequest -UseBasicParsing -Uri $uri -Headers @{ 'User-Agent' = 'honk300' }).Content",
        ])
        .arg(url)
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
    let status = Command::new("powershell")
        .args([
            "-NoProfile",
            "-ExecutionPolicy",
            "Bypass",
            "-Command",
            "param($uri, $out) Invoke-WebRequest -UseBasicParsing -Uri $uri -OutFile $out -Headers @{ 'User-Agent' = 'honk300' }",
        ])
        .arg(url)
        .arg(path)
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

fn run_installer(strategy: UpdateStrategy, path: &Path) -> Result<(), DynError> {
    // The DMG replacement is a multi-step mount/copy/detach rather than a single installer run.
    if strategy == UpdateStrategy::MacDmg {
        return run_macos_dmg_update(path);
    }

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
        UpdateStrategy::MacDmg => unreachable!("handled above"),
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

/// Replace `~/Applications/Honk300.app` from a verified universal2 DMG: mount read-only, copy the
/// bundle out with `ditto` (preserving the ad-hoc signature), then detach. Non-`cfg` so the match
/// arm above compiles everywhere; only ever reached on macOS.
fn run_macos_dmg_update(dmg: &Path) -> Result<(), DynError> {
    let mount = macos_attach_dmg(dmg)?;
    let outcome = macos_replace_app_from_mount(&mount);
    // Always detach, even when the copy failed, so a retry can re-mount.
    let _ = Command::new("hdiutil")
        .args(["detach", "-quiet"])
        .arg(&mount)
        .status();
    outcome
}

fn macos_attach_dmg(dmg: &Path) -> Result<String, DynError> {
    let output = Command::new("hdiutil")
        .args(["attach", "-nobrowse", "-readonly"])
        .arg(dmg)
        .output()?;
    if !output.status.success() {
        return Err(command_error("hdiutil attach", &output.stderr).into());
    }
    let stdout = String::from_utf8_lossy(&output.stdout);
    // The last `/Volumes/...` token in the attach table is the mount point.
    stdout
        .split_whitespace()
        .rfind(|token| token.starts_with("/Volumes/"))
        .map(str::to_string)
        .ok_or_else(|| "hdiutil attach did not report a /Volumes mount point".into())
}

fn macos_replace_app_from_mount(mount: &str) -> Result<(), DynError> {
    let source_app = Path::new(mount).join("Honk300.app");
    if !source_app.exists() {
        return Err(format!("mounted DMG did not contain Honk300.app at {mount}").into());
    }
    let dest_app = macos_home_dir()?.join("Applications").join("Honk300.app");
    if let Some(parent) = dest_app.parent() {
        fs::create_dir_all(parent)?;
    }
    let _ = fs::remove_dir_all(&dest_app);
    let status = Command::new("ditto")
        .arg(&source_app)
        .arg(&dest_app)
        .status()?;
    if status.success() {
        Ok(())
    } else {
        Err(format!(
            "ditto failed to install the app bundle with exit code {}",
            status.code().unwrap_or(-1)
        )
        .into())
    }
}

fn macos_home_dir() -> Result<PathBuf, DynError> {
    Ok(PathBuf::from(
        std::env::var_os("HOME").ok_or("HOME is not set")?,
    ))
}

fn verify_post_install(latest: &str) -> Result<(), DynError> {
    let output = Command::new("honk300").arg("--version").output()?;
    let stdout = String::from_utf8_lossy(&output.stdout);
    let installed = stdout
        .split_whitespace()
        .last()
        .ok_or("honk300 --version returned no version")?;
    if post_install_version_ok(installed, latest) {
        Ok(())
    } else {
        Err(format!("installer completed but honk300 reports {installed}, not {latest}").into())
    }
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
    fn manual_and_unknown_windows_updates_to_per_user_exe_not_cargo() {
        for source in [InstallSource::ManualLocal, InstallSource::Unknown] {
            let plan = select_update_plan(source, ReleaseTarget::WindowsX64).unwrap();
            assert_eq!(plan.strategy, UpdateStrategy::ExeCorporate);
            assert_eq!(
                plan.artifact,
                "honk300-x86_64-pc-windows-msvc-corporate-setup.exe"
            );
            assert_ne!(plan.strategy.label(), "cargo install");
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
    fn macos_app_updates_via_dmg_others_via_shell() {
        for target in [ReleaseTarget::MacosX64, ReleaseTarget::MacosArm64] {
            let app = select_update_plan(InstallSource::MacApp, target).unwrap();
            assert_eq!(app.strategy, UpdateStrategy::MacDmg);
            assert_eq!(app.artifact, "honk300-universal2.dmg");

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
}
