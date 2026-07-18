use honk_engine::{Rect, Vec2};
use honk_platform_macos::{AccessibilityState, MacBundleReleaseMetadata};
use serde_json::Value;
use std::fs::{self, DirBuilder, OpenOptions};
use std::io::{self, Write};
use std::os::unix::fs::{DirBuilderExt, MetadataExt, OpenOptionsExt, PermissionsExt};
use std::path::{Path, PathBuf};

const BUNDLE_ID: &str = "dev.emmetts.honk300";
const RECEIPT_SCHEMA_V1: &str = "honk300.install.v1";
const RECEIPT_SCHEMA_V2: &str = "honk300.install.v2";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum PermissionTransition {
    Stable,
    EnterWait,
    ResumeFirstUx,
}

pub(crate) fn transition(
    previous: AccessibilityState,
    current: AccessibilityState,
    managed: bool,
) -> PermissionTransition {
    if !managed || previous == current {
        return PermissionTransition::Stable;
    }
    match (previous, current) {
        (AccessibilityState::Denied, AccessibilityState::Trusted) => {
            PermissionTransition::ResumeFirstUx
        }
        (AccessibilityState::Trusted, AccessibilityState::Denied) => {
            PermissionTransition::EnterWait
        }
        _ => PermissionTransition::Stable,
    }
}

#[derive(Debug)]
pub(crate) struct AccessibilityOnboarding {
    managed: bool,
    marker_path: Option<PathBuf>,
    prompted: bool,
    expected_uid: u32,
    home: Option<PathBuf>,
    version: String,
}

impl AccessibilityOnboarding {
    pub(crate) fn detect(
        home: &Path,
        current_exe: &Path,
        bundle: &MacBundleReleaseMetadata,
    ) -> io::Result<Self> {
        let home_metadata = fs::symlink_metadata(home)?;
        if home_metadata.file_type().is_symlink() || !home_metadata.is_dir() {
            return Err(io::Error::other(format!(
                "refusing unsafe home directory {}",
                home.display()
            )));
        }
        let expected_uid = home_metadata.uid();
        let app = home.join("Applications").join("Honk300.app");
        let expected_exe = app.join("Contents").join("MacOS").join("honk300");
        let state_root = home
            .join("Library")
            .join("Application Support")
            .join("honk300");
        let marker_path = state_root
            .join("state")
            .join("accessibility-prompt-v1")
            .join(&bundle.version);

        let managed = if bundle_metadata_valid(bundle) {
            managed_app_path_valid(home, &expected_exe, expected_uid)?
                && exact_real_file(current_exe, &expected_exe)?
                && receipt_matches(
                    &state_root.join("install-receipt.json"),
                    &app,
                    bundle,
                    expected_uid,
                )?
        } else {
            false
        };
        let prompted = managed && fs::symlink_metadata(&marker_path).is_ok();

        Ok(Self {
            managed,
            marker_path: managed.then_some(marker_path),
            prompted,
            expected_uid,
            home: managed.then(|| home.to_path_buf()),
            version: bundle.version.clone(),
        })
    }

    pub(crate) fn unmanaged() -> Self {
        Self {
            managed: false,
            marker_path: None,
            prompted: false,
            expected_uid: 0,
            home: None,
            version: String::new(),
        }
    }

    pub(crate) fn managed(&self) -> bool {
        self.managed
    }

    pub(crate) fn waiting_for(&self, permission: AccessibilityState) -> bool {
        self.managed && permission == AccessibilityState::Denied
    }

    pub(crate) fn should_prompt(&self, permission: AccessibilityState) -> bool {
        self.waiting_for(permission) && !self.prompted
    }

    #[cfg(test)]
    pub(crate) fn marker_path(&self) -> Option<&Path> {
        self.marker_path.as_deref()
    }

    pub(crate) fn mark_prompted(&mut self) -> io::Result<()> {
        if !self.managed || self.prompted {
            return Ok(());
        }
        let marker = self
            .marker_path
            .as_deref()
            .ok_or_else(|| io::Error::other("managed onboarding has no marker path"))?;
        let prompt_dir = marker
            .parent()
            .ok_or_else(|| io::Error::other("Accessibility marker has no parent"))?;
        let state_dir = prompt_dir
            .parent()
            .ok_or_else(|| io::Error::other("Accessibility prompt directory has no parent"))?;
        let state_root = state_dir
            .parent()
            .ok_or_else(|| io::Error::other("Accessibility state directory has no parent"))?;
        let home = self
            .home
            .as_deref()
            .ok_or_else(|| io::Error::other("managed onboarding has no trusted home"))?;

        require_exact_owned_state_path(home, state_root, marker, &self.version, self.expected_uid)?;
        ensure_owner_only_directory(state_dir, self.expected_uid)?;
        ensure_owner_only_directory(prompt_dir, self.expected_uid)?;

        match OpenOptions::new()
            .write(true)
            .create_new(true)
            .mode(0o600)
            .open(marker)
        {
            Ok(mut file) => {
                file.set_permissions(fs::Permissions::from_mode(0o600))?;
                file.write_all(self.version.as_bytes())?;
                file.write_all(b"\n")?;
                file.sync_all()?;
                validate_marker_metadata(&file.metadata()?, marker, self.expected_uid)?;
            }
            Err(error) if error.kind() == io::ErrorKind::AlreadyExists => {
                validate_existing_marker(marker, self.expected_uid)?;
            }
            Err(error) => return Err(error),
        }
        self.prompted = true;
        Ok(())
    }
}

pub(crate) fn safe_anchor(bounds: Rect) -> Vec2 {
    Vec2::new(
        (bounds.max.x - 120.0).max(bounds.min.x + 40.0),
        (bounds.max.y - 110.0).max(bounds.min.y + 40.0),
    )
}

fn bundle_metadata_valid(bundle: &MacBundleReleaseMetadata) -> bool {
    bundle.bundle_id == BUNDLE_ID
        && bundle.tag == format!("v{}", bundle.version)
        && version_is_safe(&bundle.version)
        && bundle.commit.len() == 40
        && bundle.commit.bytes().all(|byte| byte.is_ascii_hexdigit())
}

fn version_is_safe(version: &str) -> bool {
    let mut parts = version.split('.');
    (0..3).all(|_| {
        parts
            .next()
            .is_some_and(|part| !part.is_empty() && part.bytes().all(|byte| byte.is_ascii_digit()))
    }) && parts.next().is_none()
}

fn exact_real_file(actual: &Path, expected: &Path) -> io::Result<bool> {
    let actual_metadata = fs::symlink_metadata(actual)?;
    let expected_metadata = fs::symlink_metadata(expected)?;
    if actual_metadata.file_type().is_symlink()
        || !actual_metadata.is_file()
        || expected_metadata.file_type().is_symlink()
        || !expected_metadata.is_file()
    {
        return Ok(false);
    }
    let actual_canonical = fs::canonicalize(actual)?;
    let expected_canonical = fs::canonicalize(expected)?;
    Ok(actual == expected && actual_canonical == expected_canonical)
}

fn managed_app_path_valid(home: &Path, expected_exe: &Path, expected_uid: u32) -> io::Result<bool> {
    let applications = home.join("Applications");
    let app = applications.join("Honk300.app");
    let contents = app.join("Contents");
    let macos = contents.join("MacOS");
    for directory in [home, &applications, &app, &contents, &macos] {
        if !real_directory_owned_by(directory, expected_uid)? {
            return Ok(false);
        }
    }
    real_file_owned_by(expected_exe, expected_uid)
}

fn real_directory_owned_by(path: &Path, expected_uid: u32) -> io::Result<bool> {
    match fs::symlink_metadata(path) {
        Ok(metadata) => Ok(!metadata.file_type().is_symlink()
            && metadata.is_dir()
            && metadata.uid() == expected_uid),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(false),
        Err(error) => Err(error),
    }
}

fn real_file_owned_by(path: &Path, expected_uid: u32) -> io::Result<bool> {
    match fs::symlink_metadata(path) {
        Ok(metadata) => Ok(!metadata.file_type().is_symlink()
            && metadata.is_file()
            && metadata.uid() == expected_uid),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(false),
        Err(error) => Err(error),
    }
}

fn receipt_matches(
    path: &Path,
    app: &Path,
    bundle: &MacBundleReleaseMetadata,
    expected_uid: u32,
) -> io::Result<bool> {
    let Some(app) = app.to_str() else {
        return Ok(false);
    };
    let metadata = match fs::symlink_metadata(path) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(false),
        Err(error) => return Err(error),
    };
    if metadata.file_type().is_symlink() || !metadata.is_file() || metadata.uid() != expected_uid {
        return Ok(false);
    }
    let value: Value = match serde_json::from_slice(&fs::read(path)?) {
        Ok(value) => value,
        Err(_) => return Ok(false),
    };
    let schema = value.get("schema").and_then(Value::as_str);
    let common = value.get("version").and_then(Value::as_str) == Some(bundle.version.as_str())
        && value.get("tag").and_then(Value::as_str) == Some(bundle.tag.as_str())
        && value.get("commit").and_then(Value::as_str) == Some(bundle.commit.as_str())
        && value.get("install_root").and_then(Value::as_str) == Some(app);
    if !common {
        return Ok(false);
    }
    if schema == Some(RECEIPT_SCHEMA_V1) {
        return Ok(true);
    }
    let artifact = value.get("artifact").and_then(Value::as_object);
    Ok(schema == Some(RECEIPT_SCHEMA_V2)
        && value.get("origin").and_then(Value::as_str) == Some("mac-app")
        && value.get("installer_family").and_then(Value::as_str) == Some("dmg")
        && value.get("edition").and_then(Value::as_str) == Some("global")
        && value.get("scope").and_then(Value::as_str) == Some("user")
        && value.get("release_track").and_then(Value::as_str) == Some("stable")
        && value.get("target").and_then(Value::as_str) == Some("universal2-apple-darwin")
        && value.get("owned_root").and_then(Value::as_str) == Some(app)
        && value.get("active_release").and_then(Value::as_str) == Some(app)
        && artifact
            .and_then(|artifact| artifact.get("name"))
            .and_then(Value::as_str)
            .is_some_and(|name| {
                name == "honk300-universal2.app.zip" || name.ends_with("/Contents/MacOS/honk300")
            })
        && artifact
            .and_then(|artifact| artifact.get("sha256"))
            .and_then(Value::as_str)
            .is_some_and(|hash| {
                hash.len() == 64 && hash.bytes().all(|byte| byte.is_ascii_hexdigit())
            })
        && artifact
            .and_then(|artifact| artifact.get("size"))
            .and_then(Value::as_u64)
            .is_some_and(|size| size > 0))
}

fn require_exact_owned_state_path(
    home: &Path,
    state_root: &Path,
    marker: &Path,
    version: &str,
    expected_uid: u32,
) -> io::Result<()> {
    let library = home.join("Library");
    let application_support = library.join("Application Support");
    let expected_state_root = application_support.join("honk300");
    let expected_marker = expected_state_root
        .join("state")
        .join("accessibility-prompt-v1")
        .join(version);
    if state_root != expected_state_root || marker != expected_marker {
        return Err(io::Error::other(format!(
            "refusing unexpected Honk300 Accessibility marker path {}",
            marker.display()
        )));
    }
    for directory in [home, &library, &application_support, &expected_state_root] {
        require_owned_real_directory(directory, expected_uid)?;
    }
    Ok(())
}

fn require_owned_real_directory(path: &Path, expected_uid: u32) -> io::Result<()> {
    let metadata = fs::symlink_metadata(path)?;
    if metadata.file_type().is_symlink() || !metadata.is_dir() || metadata.uid() != expected_uid {
        return Err(io::Error::other(format!(
            "refusing unsafe Honk300 state directory {}",
            path.display()
        )));
    }
    Ok(())
}

fn ensure_owner_only_directory(path: &Path, expected_uid: u32) -> io::Result<()> {
    let mut builder = DirBuilder::new();
    builder.mode(0o700);
    match builder.create(path) {
        Ok(()) => {}
        Err(error) if error.kind() == io::ErrorKind::AlreadyExists => {}
        Err(error) => return Err(error),
    }
    let metadata = fs::symlink_metadata(path)?;
    if metadata.file_type().is_symlink()
        || !metadata.is_dir()
        || metadata.uid() != expected_uid
        || metadata.permissions().mode() & 0o777 != 0o700
    {
        return Err(io::Error::other(format!(
            "refusing unsafe Honk300 state directory {}",
            path.display()
        )));
    }
    Ok(())
}

fn validate_existing_marker(path: &Path, expected_uid: u32) -> io::Result<()> {
    let metadata = fs::symlink_metadata(path)?;
    validate_marker_metadata(&metadata, path, expected_uid)
}

fn validate_marker_metadata(
    metadata: &fs::Metadata,
    path: &Path,
    expected_uid: u32,
) -> io::Result<()> {
    if metadata.file_type().is_symlink()
        || !metadata.is_file()
        || metadata.uid() != expected_uid
        || metadata.permissions().mode() & 0o777 != 0o600
    {
        return Err(io::Error::other(format!(
            "refusing unsafe Honk300 Accessibility marker {}",
            path.display()
        )));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use honk_platform_macos::{AccessibilityState, MacBundleReleaseMetadata};
    use serde_json::json;
    use std::fs;
    use std::os::unix::fs::{symlink, PermissionsExt};
    use std::path::{Path, PathBuf};
    use tempfile::TempDir;

    const SHA: &str = "0123456789abcdef0123456789abcdef01234567";

    struct ManagedFixture {
        temp: TempDir,
        executable: PathBuf,
        metadata: MacBundleReleaseMetadata,
    }

    impl ManagedFixture {
        fn new() -> Self {
            let temp = tempfile::tempdir().expect("temp home");
            let home = temp.path();
            let app = home.join("Applications/Honk300.app");
            let executable = app.join("Contents/MacOS/honk300");
            fs::create_dir_all(executable.parent().expect("MacOS parent")).expect("app tree");
            fs::write(&executable, b"fixture").expect("fixture executable");

            let state_root = home.join("Library/Application Support/honk300");
            fs::create_dir_all(&state_root).expect("state root");
            fs::write(
                state_root.join("install-receipt.json"),
                serde_json::to_vec_pretty(&json!({
                    "schema": "honk300.install.v1",
                    "version": "1.0.1",
                    "tag": "v1.0.1",
                    "commit": SHA,
                    "install_root": app.to_string_lossy(),
                }))
                .expect("receipt json"),
            )
            .expect("receipt");

            Self {
                temp,
                executable,
                metadata: MacBundleReleaseMetadata {
                    bundle_id: "dev.emmetts.honk300".into(),
                    version: "1.0.1".into(),
                    tag: "v1.0.1".into(),
                    commit: SHA.into(),
                },
            }
        }

        fn home(&self) -> &Path {
            self.temp.path()
        }

        fn receipt(&self) -> PathBuf {
            self.home()
                .join("Library/Application Support/honk300/install-receipt.json")
        }

        fn rewrite_receipt(&self, update: impl FnOnce(&mut Value)) {
            let mut value: Value =
                serde_json::from_slice(&fs::read(self.receipt()).expect("receipt bytes"))
                    .expect("receipt json");
            update(&mut value);
            fs::write(
                self.receipt(),
                serde_json::to_vec_pretty(&value).expect("updated receipt"),
            )
            .expect("write receipt");
        }
    }

    #[test]
    fn exact_receipted_release_prompts_once_per_version_with_owner_only_marker() {
        let fixture = ManagedFixture::new();
        let mut onboarding =
            AccessibilityOnboarding::detect(fixture.home(), &fixture.executable, &fixture.metadata)
                .expect("detect managed app");

        assert!(onboarding.managed());
        assert!(onboarding.should_prompt(AccessibilityState::Denied));
        onboarding.mark_prompted().expect("write prompt marker");
        assert!(!onboarding.should_prompt(AccessibilityState::Denied));
        assert_eq!(
            fs::metadata(onboarding.marker_path().expect("managed marker"))
                .expect("marker metadata")
                .permissions()
                .mode()
                & 0o777,
            0o600
        );
        let prompt_dir = onboarding
            .marker_path()
            .expect("managed marker")
            .parent()
            .expect("prompt directory");
        let state_dir = prompt_dir.parent().expect("state directory");
        for directory in [state_dir, prompt_dir] {
            assert_eq!(
                fs::metadata(directory)
                    .expect("state directory metadata")
                    .permissions()
                    .mode()
                    & 0o777,
                0o700
            );
        }
    }

    #[test]
    fn managed_release_waits_when_denied_but_never_prompts_when_granted() {
        let fixture = ManagedFixture::new();
        let onboarding =
            AccessibilityOnboarding::detect(fixture.home(), &fixture.executable, &fixture.metadata)
                .expect("detect managed app");

        assert!(onboarding.waiting_for(AccessibilityState::Denied));
        assert!(!onboarding.waiting_for(AccessibilityState::Trusted));
        assert!(!onboarding.should_prompt(AccessibilityState::Trusted));
    }

    #[test]
    fn protected_v2_dmg_receipt_preserves_accessibility_eligibility() {
        let fixture = ManagedFixture::new();
        let app = fixture.home().join("Applications/Honk300.app");
        fixture.rewrite_receipt(|value| {
            value["schema"] = json!("honk300.install.v2");
            value["channel"] = json!("mac-app");
            value["origin"] = json!("mac-app");
            value["installer_family"] = json!("dmg");
            value["edition"] = json!("global");
            value["scope"] = json!("user");
            value["release_track"] = json!("stable");
            value["target"] = json!("universal2-apple-darwin");
            value["owned_root"] = json!(app.to_string_lossy());
            value["active_release"] = json!(app.to_string_lossy());
            value["artifact"] = json!({
                "name": "honk300-universal2.app.zip",
                "sha256": "0".repeat(64),
                "size": 1
            });
        });

        assert!(AccessibilityOnboarding::detect(
            fixture.home(),
            &fixture.executable,
            &fixture.metadata
        )
        .expect("v2 managed receipt")
        .managed());
    }

    #[test]
    fn path_bundle_and_receipt_mismatches_are_ineligible_for_automatic_ui() {
        let fixture = ManagedFixture::new();
        let bare = fixture.home().join("bin/honk300");
        fs::create_dir_all(bare.parent().expect("bare parent")).expect("bare dir");
        fs::write(&bare, b"fixture").expect("bare executable");
        assert!(
            !AccessibilityOnboarding::detect(fixture.home(), &bare, &fixture.metadata)
                .expect("bare detection")
                .managed()
        );

        let mut wrong_bundle = fixture.metadata.clone();
        wrong_bundle.bundle_id = "example.foreign.app".into();
        assert!(!AccessibilityOnboarding::detect(
            fixture.home(),
            &fixture.executable,
            &wrong_bundle
        )
        .expect("bundle mismatch")
        .managed());

        let mut wrong_version = fixture.metadata.clone();
        wrong_version.version = "0.3.2".into();
        assert!(!AccessibilityOnboarding::detect(
            fixture.home(),
            &fixture.executable,
            &wrong_version
        )
        .expect("version mismatch")
        .managed());

        fixture.rewrite_receipt(|value| {
            value["commit"] = json!("ffffffffffffffffffffffffffffffffffffffff");
        });
        assert!(!AccessibilityOnboarding::detect(
            fixture.home(),
            &fixture.executable,
            &fixture.metadata
        )
        .expect("receipt mismatch")
        .managed());
    }

    #[test]
    fn every_receipt_identity_field_must_match_exactly() {
        for (field, mismatch) in [
            ("schema", json!("foreign.install.v1")),
            ("version", json!("0.3.2")),
            ("tag", json!("v0.3.2")),
            ("commit", json!("0123456789ABCDEF0123456789ABCDEF01234567")),
            ("install_root", json!("/Applications/Honk300.app")),
        ] {
            let fixture = ManagedFixture::new();
            fixture.rewrite_receipt(|value| value[field] = mismatch);

            assert!(
                !AccessibilityOnboarding::detect(
                    fixture.home(),
                    &fixture.executable,
                    &fixture.metadata,
                )
                .expect("receipt mismatch is an ineligible launch")
                .managed(),
                "receipt field {field} must match exactly"
            );
        }
    }

    #[test]
    fn symlinked_current_executable_and_app_components_are_ineligible() {
        let alias_fixture = ManagedFixture::new();
        let alias = alias_fixture.home().join("honk300-alias");
        symlink(&alias_fixture.executable, &alias).expect("executable alias");
        assert!(!AccessibilityOnboarding::detect(
            alias_fixture.home(),
            &alias,
            &alias_fixture.metadata,
        )
        .expect("symlinked current executable is ineligible")
        .managed());

        let component_fixture = ManagedFixture::new();
        let contents = component_fixture
            .home()
            .join("Applications/Honk300.app/Contents");
        let outside = component_fixture.home().join("outside-contents");
        fs::rename(&contents, &outside).expect("move Contents directory");
        symlink(&outside, &contents).expect("symlink Contents directory");
        assert!(!AccessibilityOnboarding::detect(
            component_fixture.home(),
            &component_fixture.executable,
            &component_fixture.metadata,
        )
        .expect("symlinked app component is ineligible")
        .managed());
    }

    #[test]
    fn current_executable_canonicalization_failures_are_explicit() {
        let fixture = ManagedFixture::new();
        let missing = fixture.home().join("missing-honk300");

        assert!(
            AccessibilityOnboarding::detect(fixture.home(), &missing, &fixture.metadata).is_err()
        );
    }

    #[test]
    fn existing_marker_suppresses_repeat_prompt() {
        let fixture = ManagedFixture::new();
        let marker = fixture
            .home()
            .join("Library/Application Support/honk300/state/accessibility-prompt-v1/1.0.1");
        fs::create_dir_all(marker.parent().expect("marker parent")).expect("marker dirs");
        fs::write(&marker, b"1.0.1\n").expect("marker");

        let onboarding =
            AccessibilityOnboarding::detect(fixture.home(), &fixture.executable, &fixture.metadata)
                .expect("detect managed app");
        assert!(onboarding.managed());
        assert!(!onboarding.should_prompt(AccessibilityState::Denied));
        assert!(onboarding.waiting_for(AccessibilityState::Denied));
    }

    #[test]
    fn symlinked_marker_directory_fails_closed_without_opening_ui() {
        let fixture = ManagedFixture::new();
        let state = fixture
            .home()
            .join("Library/Application Support/honk300/state");
        let outside = fixture.home().join("outside");
        fs::create_dir_all(&outside).expect("outside");
        symlink(&outside, &state).expect("symlink state");

        let mut onboarding =
            AccessibilityOnboarding::detect(fixture.home(), &fixture.executable, &fixture.metadata)
                .expect("detect managed app");
        assert!(onboarding.managed());
        assert!(onboarding.should_prompt(AccessibilityState::Denied));
        assert!(onboarding.mark_prompted().is_err());
    }

    #[test]
    fn symlinked_state_ancestor_fails_closed_without_writing_a_marker() {
        let fixture = ManagedFixture::new();
        let library = fixture.home().join("Library");
        let outside = fixture.home().join("outside-library");
        fs::rename(&library, &outside).expect("move Library directory");
        symlink(&outside, &library).expect("symlink Library directory");

        let mut onboarding =
            AccessibilityOnboarding::detect(fixture.home(), &fixture.executable, &fixture.metadata)
                .expect("detect managed app");
        assert!(onboarding.managed());
        assert!(onboarding.should_prompt(AccessibilityState::Denied));
        assert!(onboarding.mark_prompted().is_err());
        assert!(!outside
            .join("Application Support/honk300/state/accessibility-prompt-v1/1.0.1")
            .exists());
    }

    #[test]
    fn foreign_owned_state_fails_closed_without_writing_a_marker() {
        let fixture = ManagedFixture::new();
        let mut onboarding =
            AccessibilityOnboarding::detect(fixture.home(), &fixture.executable, &fixture.metadata)
                .expect("detect managed app");
        onboarding.expected_uid = if onboarding.expected_uid == 0 { 1 } else { 0 };

        assert!(onboarding.mark_prompted().is_err());
        assert!(!fixture
            .home()
            .join("Library/Application Support/honk300/state/accessibility-prompt-v1/1.0.1")
            .exists());
    }

    #[test]
    fn existing_non_owner_only_state_directory_fails_closed() {
        let fixture = ManagedFixture::new();
        let state = fixture
            .home()
            .join("Library/Application Support/honk300/state");
        fs::create_dir(&state).expect("state directory");
        fs::set_permissions(&state, fs::Permissions::from_mode(0o755))
            .expect("insecure state mode");

        let mut onboarding =
            AccessibilityOnboarding::detect(fixture.home(), &fixture.executable, &fixture.metadata)
                .expect("detect managed app");
        assert!(onboarding.mark_prompted().is_err());
        assert!(!state.join("accessibility-prompt-v1/1.0.1").exists());
    }

    #[test]
    fn safe_anchor_is_inset_and_clamped_for_small_displays() {
        assert_eq!(
            safe_anchor(honk_engine::Rect {
                min: honk_engine::Vec2::new(0.0, 0.0),
                max: honk_engine::Vec2::new(1440.0, 900.0),
            }),
            honk_engine::Vec2::new(1320.0, 790.0)
        );
        assert_eq!(
            safe_anchor(honk_engine::Rect {
                min: honk_engine::Vec2::new(-200.0, 10.0),
                max: honk_engine::Vec2::new(-100.0, 80.0),
            }),
            honk_engine::Vec2::new(-160.0, 50.0)
        );
    }

    #[test]
    fn permission_transitions_resume_wait_and_stabilize_deterministically() {
        assert_eq!(
            transition(
                AccessibilityState::Denied,
                AccessibilityState::Trusted,
                true,
            ),
            PermissionTransition::ResumeFirstUx
        );
        assert_eq!(
            transition(
                AccessibilityState::Trusted,
                AccessibilityState::Denied,
                true,
            ),
            PermissionTransition::EnterWait
        );
        assert_eq!(
            transition(AccessibilityState::Denied, AccessibilityState::Denied, true,),
            PermissionTransition::Stable
        );
        assert_eq!(
            transition(
                AccessibilityState::Denied,
                AccessibilityState::Trusted,
                false,
            ),
            PermissionTransition::Stable
        );
    }
}
