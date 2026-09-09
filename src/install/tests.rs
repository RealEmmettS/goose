//! Existing lifecycle contract regressions.
use super::*;
use std::cell::Cell;
use std::rc::Rc;

#[test]
fn linux_application_icon_is_complete_idempotent_and_preserves_foreign_content() {
    let directory = tempfile::tempdir().unwrap();
    let icon = directory.path().join("icon.png");
    write_linux_application_icon(&icon).unwrap();
    assert_eq!(
        fs::read(&icon).unwrap().as_slice(),
        include_bytes!("../../settings/assets/icon.png")
    );
    write_linux_application_icon(&icon).unwrap();
    fs::write(&icon, b"unrelated saved image").unwrap();
    assert_eq!(
        write_linux_application_icon(&icon).unwrap_err().kind(),
        io::ErrorKind::PermissionDenied
    );
    assert_eq!(fs::read(&icon).unwrap(), b"unrelated saved image");
}

#[cfg(target_os = "linux")]
#[test]
fn linux_application_and_login_entries_use_the_owned_icon_and_distinct_actions() {
    let exe = Path::new("/home/goose/install/bin/honk300");
    let icon = Path::new("/home/goose/install/icon.png");
    for (autostart, action) in [(false, "settings"), (true, "start")] {
        let entry = linux_desktop_entry(exe, icon, autostart);
        assert!(entry.contains("Name=Goose\n"));
        assert!(entry.contains(&format!("Exec={} {action}\n", exe.display())));
        assert!(entry.contains("Icon=/home/goose/install/icon.png\n"));
    }
}

#[test]
fn install_source_markers_are_stable() {
    for (marker, source) in [
        ("msi-global", InstallSource::MsiGlobal),
        ("msi-corporate", InstallSource::MsiCorporate),
        ("exe-global", InstallSource::ExeGlobal),
        ("exe-corporate", InstallSource::ExeCorporate),
        ("manual-local", InstallSource::ManualLocal),
        ("shell", InstallSource::Shell),
        ("powershell", InstallSource::PowerShell),
        ("deb", InstallSource::Deb),
        ("mac-app", InstallSource::MacApp),
    ] {
        assert_eq!(InstallSource::from_marker(marker), source);
        assert_eq!(source.marker_value(), marker);
    }
    assert_eq!(InstallSource::from_marker("cargo"), InstallSource::Unknown);
}

#[cfg(windows)]
#[test]
fn windows_installer_temp_helper_can_never_fall_through_to_app_startup() {
    assert!(is_windows_installer_custom_action_path(Path::new(
        r"C:\Windows\Installer\MSIE15D.tmp"
    )));
    assert!(is_windows_installer_custom_action_path(Path::new(
        r"C:\Windows\Installer\msi123.tmp"
    )));
    assert!(!is_windows_installer_custom_action_path(Path::new(
        r"C:\Program Files\honk300\honk300.exe"
    )));
    assert!(!is_windows_installer_custom_action_path(Path::new(
        r"C:\Windows\Temp\MSIE15D.tmp"
    )));
}

#[test]
fn windows_package_uninstall_never_adopts_a_conflicting_active_origin() {
    assert!(windows_package_owns_active_slot(
        InstallSource::MsiGlobal,
        InstallSource::MsiGlobal
    ));
    assert!(windows_package_owns_active_slot(
        InstallSource::MsiGlobal,
        InstallSource::PowerShell
    ));
    assert!(!windows_package_owns_active_slot(
        InstallSource::MsiGlobal,
        InstallSource::ExeGlobal
    ));
    assert!(!windows_package_owns_active_slot(
        InstallSource::MsiGlobal,
        InstallSource::MsiCorporate
    ));
}

#[cfg(windows)]
#[test]
fn windows_login_start_uses_only_the_gui_launcher_and_recognizes_the_owned_legacy_value() {
    let launcher = Path::new(r"C:\Program Files\honk300\bin\honk300-app.exe");
    assert_eq!(
        windows_autostart_command(launcher),
        r#""C:\Program Files\honk300\bin\honk300-app.exe""#
    );
    assert_eq!(
        legacy_windows_autostart_command(launcher).as_deref(),
        Some(r#""C:\Program Files\honk300\bin\honk300.exe" start"#)
    );
    assert_eq!(
        legacy_windows_autostart_command(Path::new(r"C:\Program Files\honk300\bin\foreign.exe")),
        None
    );
    assert_eq!(
        manual_autostart_program(PathBuf::from(
            r"C:\Users\goose\AppData\Local\Programs\honk300\bin\honk300.exe"
        )),
        PathBuf::from(r"C:\Users\goose\AppData\Local\Programs\honk300\bin\honk300-app.exe")
    );
}

#[test]
fn lifecycle_lease_is_held_for_the_entire_mutation() {
    struct Lease(Rc<Cell<bool>>);
    impl Drop for Lease {
        fn drop(&mut self) {
            self.0.set(true);
        }
    }

    let dropped = Rc::new(Cell::new(false));
    mutate_with_lifecycle_lease(
        || Ok(Lease(Rc::clone(&dropped))),
        || {
            assert!(!dropped.get(), "singleton lease dropped before mutation");
            Ok(())
        },
    )
    .expect("mutation under lifecycle lease");
    assert!(
        dropped.get(),
        "singleton lease should release after mutation"
    );
}

#[test]
fn lifecycle_lease_failure_leaves_files_untouched() {
    let root = test_dir("quiesce-before-mutation");
    let marker = root.join("mutation-started");
    let result = mutate_with_lifecycle_lease(
        || {
            Err::<(), _>(io::Error::new(
                io::ErrorKind::TimedOut,
                "singleton still held",
            ))
        },
        || {
            fs::create_dir_all(&root)?;
            fs::write(&marker, b"mutated")
        },
    );

    assert_eq!(result.unwrap_err().kind(), io::ErrorKind::TimedOut);
    assert!(!root.exists());
    assert!(!marker.exists());
}

#[test]
fn app_bundle_detection_matches_only_app_ancestors() {
    assert!(path_is_in_app_bundle(Path::new(
        "/Users/a/Applications/Honk300.app/Contents/MacOS/honk300"
    )));
    assert!(path_is_in_app_bundle(Path::new(
        "/Volumes/Honk300/Honk300.app/Contents/MacOS/goose"
    )));
    assert!(!path_is_in_app_bundle(Path::new(
        "/Users/a/.local/share/honk300/install/bin/honk300"
    )));
    assert!(!path_is_in_app_bundle(Path::new("/usr/local/bin/honk300")));
}

#[test]
fn install_path_classification_never_selects_cargo_update_path() {
    assert_eq!(
        classify_install_path(r"C:\Program Files\honk300\bin\honk300.exe"),
        InstallSource::Unknown
    );
    assert_eq!(
        classify_install_path(r"C:\Users\a\AppData\Local\Programs\honk300\bin\goose.exe"),
        InstallSource::Unknown
    );
    assert_eq!(
        classify_install_path("/home/a/.local/share/honk300/install/bin/honk300"),
        InstallSource::Shell
    );
    assert_eq!(
        classify_install_path(r"C:\Users\a\.cargo\bin\honk300.exe"),
        InstallSource::Unknown
    );
}

#[test]
fn receipt_v2_requires_a_complete_consistent_install_identity() {
    let root = test_dir("receipt-v2-identity");
    let executable = root.join("current/bin/honk300.exe");
    fs::create_dir_all(executable.parent().unwrap()).unwrap();
    fs::write(&executable, b"fixture").unwrap();
    let receipt = serde_json::json!({
        "schema": INSTALL_RECEIPT_V2,
        "version": "1.2.3",
        "tag": "v1.2.3",
        "commit": "0".repeat(40),
        "origin": "msi-global",
        "installer_family": "msi",
        "edition": "global",
        "scope": "machine",
        "release_track": "stable",
        "layout": "windows-slots-v1",
        "target": "x86_64-pc-windows-msvc",
        "artifact": { "name": "honk300-x86_64-pc-windows-msvc.msi", "sha256": "a".repeat(64), "size": 123 },
        "install_root": root.to_string_lossy(),
        "active_release": root.join("releases/1.2.3-x86_64-pc-windows-msvc").to_string_lossy(),
        "aliases": [],
        "autostart": { "enabled": false, "owner": "msi" }
    });
    assert_eq!(
        validated_receipt_source(&receipt, &executable),
        Some(InstallSource::MsiGlobal)
    );

    let mut inconsistent = receipt.clone();
    inconsistent["scope"] = "user".into();
    assert_eq!(validated_receipt_source(&inconsistent, &executable), None);

    let mut bad_hash = receipt;
    bad_hash["artifact"]["sha256"] = "not-a-hash".into();
    assert_eq!(validated_receipt_source(&bad_hash, &executable), None);
    let _ = fs::remove_dir_all(root);
}

#[test]
fn receipt_evidence_accepts_v1_but_rejects_foreign_and_conflicting_receipts() {
    let root = test_dir("receipt-evidence");
    let executable = root.join("bin/honk300");
    fs::create_dir_all(executable.parent().unwrap()).unwrap();
    fs::write(&executable, b"fixture").unwrap();
    let first = root.join("first.json");
    let second = root.join("second.json");
    fs::write(
        &first,
        serde_json::to_vec(&serde_json::json!({
            "schema": OWNERSHIP_MARKER,
            "channel": "shell",
            "install_root": root.to_string_lossy(),
        }))
        .unwrap(),
    )
    .unwrap();
    assert_eq!(
        install_receipt_source_from_candidates(std::slice::from_ref(&first), &executable),
        InstallSourceEvidence::Valid(InstallSource::Shell)
    );

    fs::write(
        &second,
        serde_json::to_vec(&serde_json::json!({
            "schema": OWNERSHIP_MARKER,
            "channel": "deb",
            "install_root": root.to_string_lossy(),
        }))
        .unwrap(),
    )
    .unwrap();
    assert_eq!(
        install_receipt_source_from_candidates(&[first.clone(), second.clone()], &executable),
        InstallSourceEvidence::InvalidOrConflicting
    );

    fs::write(&second, b"foreign").unwrap();
    assert_eq!(
        install_receipt_source_from_candidates(&[second], &executable),
        InstallSourceEvidence::InvalidOrConflicting
    );
    let _ = fs::remove_dir_all(root);
}

#[test]
fn purge_backup_copies_user_memes_and_notes() {
    let root = test_dir("backup");
    let media = root.join("media");
    let memes = media.join("Memes");
    let notes = media.join("Notes");
    fs::create_dir_all(&memes).unwrap();
    fs::create_dir_all(&notes).unwrap();
    fs::write(memes.join("mine.png"), b"png").unwrap();
    fs::write(notes.join("mine.txt"), b"note").unwrap();

    let backup = backup_user_content_at(&media, &root.join("backups"), 123)
        .unwrap()
        .unwrap();

    assert!(backup.join("Memes/mine.png").exists());
    assert!(backup.join("Notes/mine.txt").exists());
    let _ = fs::remove_dir_all(root);
}

#[test]
fn purge_backup_reports_none_without_user_content() {
    let root = test_dir("empty-backup");
    let media = root.join("media");
    ensure_external_media_root(&media).unwrap();
    assert_eq!(
        backup_user_content_at(&media, &root.join("backups"), 123)
            .unwrap()
            .as_ref(),
        None
    );
    let _ = fs::remove_dir_all(root);
}

#[test]
fn non_purge_uninstall_leaves_external_user_media_in_place() {
    let root = test_dir("preserve");
    let media = root.join("media");
    let memes = media.join("Memes");
    let notes = media.join("Notes");
    fs::create_dir_all(&memes).unwrap();
    fs::create_dir_all(&notes).unwrap();
    fs::write(memes.join("mine.png"), b"png").unwrap();
    fs::write(notes.join("mine.txt"), b"note").unwrap();

    assert!(media_has_user_content(&media).unwrap());
    assert_eq!(fs::read(memes.join("mine.png")).unwrap(), b"png");
    assert_eq!(fs::read(notes.join("mine.txt")).unwrap(), b"note");
    let _ = fs::remove_dir_all(root);
}

#[test]
fn macos_install_accepts_managed_or_mounted_source_bundle_and_mutates_only_owned_paths() {
    let home = Path::new("/Users/goose");
    let app = home.join("Applications/Honk300.app");
    let exact = app.join("Contents/MacOS/honk300");
    assert_eq!(
        macos_install_disposition(&exact, &app).unwrap(),
        MacosInstallDisposition::ConfigureExisting
    );
    assert_eq!(
        macos_install_disposition(
            Path::new("/Volumes/Honk300/Honk300.app/Contents/MacOS/honk300"),
            &app
        )
        .unwrap(),
        MacosInstallDisposition::CopyBundle(PathBuf::from("/Volumes/Honk300/Honk300.app"))
    );
    assert!(macos_install_disposition(Path::new("/Users/goose/.local/bin/honk300"), &app).is_err());

    for mutation in macos_external_mutation_paths(home) {
        assert!(
            !mutation.starts_with(&app),
            "sealed app mutation leaked into plan: {}",
            mutation.display()
        );
    }
}

#[test]
fn macos_dmg_receipt_is_updater_compatible_and_release_bound() {
    let root = Path::new("/Users/goose/Applications/Honk300.app");
    let metadata = MacosBundleMetadata {
        version: "1.0.1".into(),
        tag: "v1.0.1".into(),
        commit: "0123456789abcdef0123456789abcdef01234567".into(),
    };
    let receipt = macos_install_receipt(&metadata, root, false, &"a".repeat(64), 123);

    assert_eq!(receipt["schema"], INSTALL_RECEIPT_V2);
    assert_eq!(receipt["install_root"], root.to_string_lossy().as_ref());
    assert_eq!(receipt["version"], "1.0.1");
    assert_eq!(receipt["tag"], "v1.0.1");
    assert_eq!(
        receipt["commit"],
        "0123456789abcdef0123456789abcdef01234567"
    );
    assert_eq!(receipt["origin"], "mac-app");
    assert_eq!(receipt["installer_family"], "dmg");
    assert_eq!(receipt["release_track"], "stable");
    assert_eq!(receipt["artifact"]["size"], 123);
    assert_eq!(receipt["active_release"], root.to_string_lossy().as_ref());
    assert_eq!(receipt["layout"], "mac-app");
}

#[test]
fn macos_existing_bundle_replacement_requires_an_owned_receipt() {
    assert!(macos_bundle_replacement_is_owned(false, false));
    assert!(macos_bundle_replacement_is_owned(false, true));
    assert!(macos_bundle_replacement_is_owned(true, true));
    assert!(!macos_bundle_replacement_is_owned(true, false));
}

#[cfg(target_os = "macos")]
#[test]
fn macos_integration_snapshot_is_inert_until_mutations_begin() {
    let home = test_dir("macos-integration-inert");
    let launch_agent = home.join("Library/LaunchAgents/dev.emmetts.honk300.plist");
    let receipt = home.join("Library/Application Support/honk300/install-receipt.json");
    fs::create_dir_all(launch_agent.parent().unwrap()).unwrap();
    fs::create_dir_all(receipt.parent().unwrap()).unwrap();
    fs::write(&launch_agent, b"before").unwrap();
    let transaction = MacosIntegrationTransaction::capture(&[], &launch_agent, &receipt).unwrap();
    fs::write(&launch_agent, b"after capture").unwrap();

    drop(transaction);

    assert_eq!(fs::read(&launch_agent).unwrap(), b"after capture");
    let _ = fs::remove_dir_all(home);
}

#[cfg(target_os = "macos")]
#[test]
fn macos_integration_transaction_removes_fresh_mutations_after_receipt_collision() {
    let home = test_dir("macos-integration-rollback");
    let app = home.join("Applications/Honk300.app");
    let installed = app.join("Contents/MacOS/honk300");
    let aliases = home.join(".local/bin");
    let launch_agent = home.join("Library/LaunchAgents/dev.emmetts.honk300.plist");
    let receipt = home.join("Library/Application Support/honk300/install-receipt.json");
    let media = home.join("Library/Application Support/honk300/media");
    let legacy_assets = app.join("Contents/Resources/Assets");
    fs::create_dir_all(installed.parent().unwrap()).unwrap();
    fs::create_dir_all(&aliases).unwrap();
    fs::create_dir_all(receipt.parent().unwrap()).unwrap();
    fs::write(&installed, b"fixture").unwrap();
    ensure_external_media_root(&media).unwrap();
    fs::write(media.join("Memes/keep.png"), b"keep").unwrap();
    let legacy_note = legacy_assets.join("Text/NotepadMessages/user/nested/new.txt");
    fs::create_dir_all(legacy_note.parent().unwrap()).unwrap();
    fs::write(&legacy_note, b"new").unwrap();
    let alias_paths = COMMAND_NAMES
        .iter()
        .map(|name| aliases.join(name))
        .collect::<Vec<_>>();
    let mut transaction =
        MacosIntegrationTransaction::capture(&alias_paths, &launch_agent, &receipt).unwrap();
    transaction.begin();
    let media_changes =
        migrate_legacy_user_media(&legacy_assets, &media, LegacyMigrationMode::Copy).unwrap();
    transaction.record_media_migration(media_changes);
    let owned_targets = [installed.as_path()];
    for alias in &alias_paths {
        install_owned_unix_alias(alias, &installed, &owned_targets).unwrap();
    }
    write_owned_text_file(
        &launch_agent,
        &format!("<!-- {OWNERSHIP_MARKER} -->\nnew\n"),
        OWNERSHIP_MARKER,
    )
    .unwrap();
    let stale = receipt
        .parent()
        .unwrap()
        .join(format!(".install-receipt.{}.tmp", std::process::id()));
    fs::write(&stale, b"collision").unwrap();
    let metadata = MacosBundleMetadata {
        version: "1.0.1".into(),
        tag: "v1.0.1".into(),
        commit: "0123456789abcdef0123456789abcdef01234567".into(),
    };

    assert!(write_macos_receipt(&receipt, &app, &metadata, false).is_err());
    transaction.rollback().unwrap();

    for alias in &alias_paths {
        assert!(
            fs::symlink_metadata(alias).is_err(),
            "{} leaked",
            alias.display()
        );
    }
    assert!(!launch_agent.exists());
    assert!(!receipt.exists());
    assert_eq!(fs::read(&stale).unwrap(), b"collision");
    assert_eq!(fs::read(media.join("Memes/keep.png")).unwrap(), b"keep");
    assert!(!media.join("Notes/nested/new.txt").exists());
    assert!(!media.join("Notes/nested").exists());
    let _ = fs::remove_dir_all(home);
}

#[cfg(target_os = "macos")]
#[test]
fn macos_integration_transaction_restores_existing_owned_files() {
    let home = test_dir("macos-integration-restore");
    let app = home.join("Applications/Honk300.app");
    let installed = app.join("Contents/MacOS/honk300");
    let aliases = home.join(".local/bin");
    let launch_agent = home.join("Library/LaunchAgents/dev.emmetts.honk300.plist");
    let receipt = home.join("Library/Application Support/honk300/install-receipt.json");
    fs::create_dir_all(installed.parent().unwrap()).unwrap();
    fs::create_dir_all(&aliases).unwrap();
    fs::create_dir_all(launch_agent.parent().unwrap()).unwrap();
    fs::create_dir_all(receipt.parent().unwrap()).unwrap();
    fs::write(&installed, b"fixture").unwrap();
    let alias_paths = COMMAND_NAMES
        .iter()
        .map(|name| aliases.join(name))
        .collect::<Vec<_>>();
    for alias in &alias_paths {
        std::os::unix::fs::symlink(&installed, alias).unwrap();
    }
    let old_launch_agent = format!("<!-- {OWNERSHIP_MARKER} -->\nold\n");
    fs::write(&launch_agent, &old_launch_agent).unwrap();
    let old_receipt = format!(
        "{{\"schema\":\"{OWNERSHIP_MARKER}\",\"install_root\":{:?},\"version\":\"0.3.2\"}}",
        app.to_string_lossy()
    );
    fs::write(&receipt, &old_receipt).unwrap();
    let mut transaction =
        MacosIntegrationTransaction::capture(&alias_paths, &launch_agent, &receipt).unwrap();
    transaction.begin();

    for alias in &alias_paths {
        fs::remove_file(alias).unwrap();
    }
    fs::write(&launch_agent, format!("<!-- {OWNERSHIP_MARKER} -->\nnew\n")).unwrap();
    fs::write(&receipt, b"new receipt").unwrap();
    transaction.rollback().unwrap();

    for alias in &alias_paths {
        assert_eq!(fs::read_link(alias).unwrap(), installed);
    }
    assert_eq!(fs::read_to_string(&launch_agent).unwrap(), old_launch_agent);
    assert_eq!(fs::read_to_string(&receipt).unwrap(), old_receipt);
    let _ = fs::remove_dir_all(home);
}

#[cfg(target_os = "macos")]
#[test]
fn macos_receipt_preflight_preserves_a_dangling_foreign_symlink() {
    let home = test_dir("macos-dangling-receipt");
    let app = home.join("Applications/Honk300.app");
    let receipt = home.join("Library/Application Support/honk300/install-receipt.json");
    let foreign_target = home.join("missing-foreign-receipt.json");
    fs::create_dir_all(receipt.parent().unwrap()).unwrap();
    std::os::unix::fs::symlink(&foreign_target, &receipt).unwrap();
    let metadata = MacosBundleMetadata {
        version: "1.0.1".into(),
        tag: "v1.0.1".into(),
        commit: "0123456789abcdef0123456789abcdef01234567".into(),
    };

    assert!(preflight_owned_macos_receipt(&receipt, &app).is_err());
    assert!(write_macos_receipt(&receipt, &app, &metadata, false).is_err());
    assert_eq!(fs::read_link(&receipt).unwrap(), foreign_target);
    assert!(fs::symlink_metadata(&receipt)
        .unwrap()
        .file_type()
        .is_symlink());

    let _ = fs::remove_dir_all(home);
}

#[test]
fn unix_alias_decisions_preserve_foreign_files_and_symlinks() {
    let desired = Path::new("/home/goose/.local/share/honk300/install/bin/honk300");
    let old_owned = Path::new("/home/goose/Applications/Honk300.app/Contents/MacOS/honk300");
    let owned = [desired, old_owned];
    assert_eq!(
        alias_install_decision(AliasState::Missing, desired, &owned),
        AliasInstallDecision::Create
    );
    assert_eq!(
        alias_install_decision(AliasState::Symlink(desired), desired, &owned),
        AliasInstallDecision::Keep
    );
    assert_eq!(
        alias_install_decision(AliasState::Symlink(old_owned), desired, &owned),
        AliasInstallDecision::ReplaceOwned
    );
    assert_eq!(
        alias_install_decision(
            AliasState::Symlink(Path::new("/opt/foreign/goose")),
            desired,
            &owned,
        ),
        AliasInstallDecision::PreserveForeign
    );
    assert_eq!(
        alias_install_decision(AliasState::Other, desired, &owned),
        AliasInstallDecision::PreserveForeign
    );
}

#[test]
fn owned_text_integrations_never_replace_or_remove_foreign_files() {
    let root = test_dir("owned-text");
    let path = root.join("honk300.desktop");
    fs::create_dir_all(&root).unwrap();
    fs::write(&path, "foreign desktop entry\n").unwrap();
    assert!(!owned_text_autostart_state(&path, OWNERSHIP_MARKER).unwrap());
    reconcile_owned_text_autostart_file(
        &path,
        false,
        &format!("# {OWNERSHIP_MARKER}\nowned\n"),
        OWNERSHIP_MARKER,
    )
    .unwrap();
    assert!(reconcile_owned_text_autostart_file(
        &path,
        true,
        &format!("# {OWNERSHIP_MARKER}\nowned\n"),
        OWNERSHIP_MARKER,
    )
    .is_err());
    assert!(write_owned_text_file(&path, "replacement", OWNERSHIP_MARKER).is_err());
    assert_eq!(
        fs::read_to_string(&path).unwrap(),
        "foreign desktop entry\n"
    );
    assert!(!remove_owned_text_file(&path, OWNERSHIP_MARKER).unwrap());
    assert!(path.exists());

    fs::write(&path, format!("# {OWNERSHIP_MARKER}\nowned\n")).unwrap();
    assert!(owned_text_autostart_state(&path, OWNERSHIP_MARKER).unwrap());
    write_owned_text_file(
        &path,
        &format!("# {OWNERSHIP_MARKER}\nupdated\n"),
        OWNERSHIP_MARKER,
    )
    .unwrap();
    assert!(remove_owned_text_file(&path, OWNERSHIP_MARKER).unwrap());
    assert!(!path.exists());

    let foreign_directory = root.join("foreign-directory.desktop");
    fs::create_dir(&foreign_directory).unwrap();
    assert!(!owned_text_autostart_state(&foreign_directory, OWNERSHIP_MARKER).unwrap());
    let _ = fs::remove_dir_all(root);
}

#[test]
fn autostart_preparation_rejects_a_stale_config_without_overwriting_it() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("config.toml");
    let mut old = honk_config::Config::default();
    old.save_atomic(&path).unwrap();
    let mut current = old.clone();
    current.audio.enabled = false;
    current.save_atomic(&path).unwrap();
    let bytes = fs::read(&path).unwrap();
    let result = prepare_config_autostart(&path, &mut old);
    assert!(matches!(
        result
            .unwrap_err()
            .downcast_ref::<honk_config::ConfigError>(),
        Some(honk_config::ConfigError::Conflict)
    ));
    assert_eq!(fs::read(&path).unwrap(), bytes);
}

#[test]
fn managed_path_blocks_are_removed_without_touching_profile_content() {
    let profile = "export EDITOR=vim\n\n# >>> honk300 managed PATH >>>\nexport PATH=\"$HOME/.local/bin:$PATH\"\n# <<< honk300 managed PATH <<<\nalias ll='ls -l'\n";
    let (updated, changed) = strip_managed_path_blocks(profile);
    assert!(changed);
    assert!(updated.contains("export EDITOR=vim\n"));
    assert!(updated.contains("alias ll='ls -l'\n"));
    assert!(!updated.contains("honk300 managed PATH"));

    let foreign = "# >>> somebody else PATH >>>\nexport PATH=/opt/foreign:$PATH\n";
    assert_eq!(strip_managed_path_blocks(foreign), (foreign.into(), false));
}

#[test]
fn external_receipts_are_removed_only_for_matching_owned_install_root() {
    let root = test_dir("receipt");
    let receipt = root.join("install-receipt.json");
    let install_root = root.join("install");
    fs::create_dir_all(&root).unwrap();
    fs::write(
        &receipt,
        format!(
            "{{\"schema\":\"honk300.install.v1\",\"install_root\":{:?}}}",
            install_root.to_string_lossy()
        ),
    )
    .unwrap();
    assert!(remove_owned_receipt(&receipt, &install_root).unwrap());
    assert!(!receipt.exists());

    fs::write(
        &receipt,
        "{\"schema\":\"foreign.install.v1\",\"install_root\":\"x\"}",
    )
    .unwrap();
    assert!(!remove_owned_receipt(&receipt, &install_root).unwrap());
    assert!(receipt.exists());
    let _ = fs::remove_dir_all(root);
}

#[test]
fn platform_media_roots_are_external_user_data_locations() {
    assert_eq!(
        windows_media_root_from(Path::new(r"C:\Users\goose\AppData\Local"))
            .to_string_lossy()
            .replace('/', "\\"),
        r"C:\Users\goose\AppData\Local\honk300\media"
    );
    assert_eq!(
        macos_media_root_from(Path::new("/Users/goose")),
        PathBuf::from("/Users/goose/Library/Application Support/honk300/media")
    );
    assert_eq!(
        linux_media_root_from(Some(Path::new("/data")), Path::new("/home/goose")),
        PathBuf::from("/data/honk300/media")
    );
    assert_eq!(
        linux_media_root_from(None, Path::new("/home/goose")),
        PathBuf::from("/home/goose/.local/share/honk300/media")
    );
}

#[test]
fn legacy_user_media_migrates_to_external_memes_and_notes_without_overwrite() {
    let root = test_dir("media-migrate");
    let assets = root.join("Assets");
    let media = root.join("media");
    let legacy_memes = assets.join("Images/Memes/user");
    let legacy_notes = assets.join("Text/NotepadMessages/user");
    fs::create_dir_all(&legacy_memes).unwrap();
    fs::create_dir_all(&legacy_notes).unwrap();
    fs::write(legacy_memes.join("mine.png"), b"png").unwrap();
    fs::write(legacy_notes.join("mine.txt"), b"note").unwrap();

    migrate_legacy_user_media(&assets, &media, LegacyMigrationMode::Move).unwrap();
    assert_eq!(fs::read(media.join("Memes/mine.png")).unwrap(), b"png");
    assert_eq!(fs::read(media.join("Notes/mine.txt")).unwrap(), b"note");
    assert!(!legacy_memes.join("mine.png").exists());
    assert!(!legacy_notes.join("mine.txt").exists());

    fs::create_dir_all(&legacy_memes).unwrap();
    fs::write(legacy_memes.join("mine.png"), b"png").unwrap();
    migrate_legacy_user_media(&assets, &media, LegacyMigrationMode::Copy).unwrap();
    assert_eq!(fs::read(media.join("Memes/mine.png")).unwrap(), b"png");

    fs::write(legacy_memes.join("mine.png"), b"new").unwrap();
    assert!(migrate_legacy_user_media(&assets, &media, LegacyMigrationMode::Move).is_err());
    assert_eq!(fs::read(media.join("Memes/mine.png")).unwrap(), b"png");
    assert_eq!(fs::read(legacy_memes.join("mine.png")).unwrap(), b"new");
    let _ = fs::remove_dir_all(root);
}

#[test]
fn media_migration_rollback_removes_fresh_tree_and_preserves_existing_content() {
    let root = test_dir("media-migrate-rollback");
    let assets = root.join("Assets");
    let fresh_media = root.join("fresh/state/media");
    let legacy_note = assets.join("Text/NotepadMessages/user/nested/new.txt");
    fs::create_dir_all(legacy_note.parent().unwrap()).unwrap();
    fs::write(&legacy_note, b"new").unwrap();

    let mut fresh =
        migrate_legacy_user_media(&assets, &fresh_media, LegacyMigrationMode::Copy).unwrap();
    assert_eq!(
        fs::read(fresh_media.join("Notes/nested/new.txt")).unwrap(),
        b"new"
    );
    fresh.rollback().unwrap();
    assert!(!fresh_media.exists());

    let existing_media = root.join("existing/media");
    ensure_external_media_root(&existing_media).unwrap();
    fs::write(existing_media.join("Memes/keep.png"), b"keep").unwrap();
    let mut existing =
        migrate_legacy_user_media(&assets, &existing_media, LegacyMigrationMode::Copy).unwrap();
    existing.rollback().unwrap();
    assert_eq!(
        fs::read(existing_media.join("Memes/keep.png")).unwrap(),
        b"keep"
    );
    assert!(!existing_media.join("Notes/nested").exists());
    assert!(existing_media.exists());
    let _ = fs::remove_dir_all(root);
}

#[test]
fn windows_managed_uninstall_identity_is_conservative() {
    let current_exe = Path::new(r"C:\Program Files\honk300\bin\honk300.exe");
    let entry = WindowsUninstallIdentity {
        key_name: "{01234567-89AB-CDEF-0123-456789ABCDEF}".into(),
        display_name: "honk300".into(),
        publisher: "Emmett S".into(),
        install_location: PathBuf::from(r"C:\Program Files\honk300"),
        uninstall_command: r"MsiExec.exe /I{01234567-89AB-CDEF-0123-456789ABCDEF}".into(),
        windows_installer: true,
    };
    assert!(matches!(
        validate_windows_uninstall_identity(InstallSource::MsiGlobal, current_exe, &entry),
        Some(WindowsManagedUninstall::Msi { .. })
    ));

    let mut foreign = entry.clone();
    foreign.publisher = "Somebody Else".into();
    assert!(
        validate_windows_uninstall_identity(InstallSource::MsiGlobal, current_exe, &foreign)
            .is_none()
    );
    foreign = entry;
    foreign.install_location = PathBuf::from(r"C:\Program Files\Foreign");
    assert!(
        validate_windows_uninstall_identity(InstallSource::MsiGlobal, current_exe, &foreign)
            .is_none()
    );
}

#[test]
fn windows_registration_hive_follows_installer_registration_semantics() {
    assert!(windows_registration_hive_is_valid(
        InstallSource::MsiGlobal,
        true
    ));
    assert!(!windows_registration_hive_is_valid(
        InstallSource::MsiGlobal,
        false
    ));
    assert!(windows_registration_hive_is_valid(
        InstallSource::ExeGlobal,
        true
    ));
    assert!(!windows_registration_hive_is_valid(
        InstallSource::ExeGlobal,
        false
    ));
    assert!(windows_registration_hive_is_valid(
        InstallSource::MsiCorporate,
        true
    ));
    assert!(windows_registration_hive_is_valid(
        InstallSource::MsiCorporate,
        false
    ));
    assert!(!windows_registration_hive_is_valid(
        InstallSource::ExeCorporate,
        true
    ));
    assert!(windows_registration_hive_is_valid(
        InstallSource::ExeCorporate,
        false
    ));
}

#[test]
fn windows_adjacent_marker_is_only_used_after_registration_evidence_is_missing() {
    assert_eq!(
        windows_install_source_precedence(
            Some(InstallSource::MsiCorporate),
            InstallSource::MsiCorporate,
        ),
        InstallSource::MsiCorporate
    );
    assert_eq!(
        windows_install_source_precedence(None, InstallSource::MsiCorporate),
        InstallSource::MsiCorporate
    );
}

#[test]
fn windows_registration_identity_is_unique_and_preserves_powershell_origin() {
    assert_eq!(
        windows_registration_evidence(&[], None),
        InstallSourceEvidence::Missing
    );
    assert_eq!(
        windows_registration_evidence(&[InstallSource::MsiGlobal], None),
        InstallSourceEvidence::Valid(InstallSource::MsiGlobal)
    );
    assert_eq!(
        windows_registration_evidence(&[InstallSource::MsiGlobal], Some(InstallSource::PowerShell)),
        InstallSourceEvidence::Valid(InstallSource::PowerShell)
    );
    assert_eq!(
        windows_registration_evidence(
            &[InstallSource::MsiGlobal, InstallSource::ExeGlobal],
            Some(InstallSource::ExeGlobal)
        ),
        InstallSourceEvidence::InvalidOrConflicting
    );
    assert_eq!(
        windows_registration_evidence(
            &[InstallSource::MsiCorporate],
            Some(InstallSource::MsiGlobal)
        ),
        InstallSourceEvidence::InvalidOrConflicting
    );
}

#[test]
fn windows_active_owner_path_cleanup_removes_only_the_exact_stable_bin() {
    let removed = Path::new(r"C:\Program Files\honk300\bin");
    assert_eq!(
        windows_path_without_entry(r"C:\Windows;C:\Program Files\honk300\bin;C:\Tools", removed),
        Some(r"C:\Windows;C:\Tools".into())
    );
    assert_eq!(
        windows_path_without_entry(r"C:\Windows;c:\program files\HONK300\BIN;C:\Tools", removed),
        Some(r"C:\Windows;C:\Tools".into())
    );
    assert_eq!(
        windows_path_without_entry(r"C:\Windows;C:\Tools", removed),
        None
    );
    assert_eq!(
        windows_path_without_entry(
            r"C:\Windows;C:\Program Files\honk300\bin\;C:\Tools",
            removed
        ),
        Some(r"C:\Windows;C:\Tools".into())
    );
    assert!(windows_path_entry_matches(
        r" C:\Program Files\honk300\bin\ ",
        removed
    ));
}

#[cfg(windows)]
#[test]
fn windows_owner_retirement_uses_one_hidden_elevated_active_slot_coordinator() {
    let owner = WindowsRegisteredOwner {
        source: InstallSource::ExeGlobal,
        install_root: PathBuf::from(r"C:\Program Files\honk300"),
        uninstall: WindowsManagedUninstall::Exe {
            uninstaller: PathBuf::from(r"C:\Program Files\honk300\unins000.exe"),
            elevated: true,
        },
        registration: "HKLM:64:{5A94FBD0-DA02-4F63-9363-7D9CE0E280F5}_is1".into(),
        logical_registration: "HKLM:{5a94fbd0-da02-4f63-9363-7d9ce0e280f5}_is1".into(),
    };
    let invocation = windows_owner_retirement_invocation(
        Path::new(r"C:\Users\user\AppData\Local\Programs\honk300\bin\honk300.exe"),
        Path::new(r"C:\Users\user\AppData\Local\Programs\honk300\"),
        InstallSource::MsiCorporate,
        &owner,
    );
    assert!(invocation
        .args
        .windows(2)
        .any(|args| args == ["-WindowStyle", "Hidden"]));
    assert!(invocation.script.contains("__windows-retire-owner"));
    assert!(invocation.script.contains("-Verb RunAs"));
    assert!(invocation.script.contains("msi-corporate"));
    assert!(invocation
        .script
        .contains(r#"'"C:\Users\user\AppData\Local\Programs\honk300"'"#));
    assert!(!invocation
        .script
        .contains(r#"C:\Users\user\AppData\Local\Programs\honk300\"'"#));
    assert!(!invocation.script.contains("unins000.exe"));
    assert_eq!(invocation.script.matches("Start-Process").count(), 1);
}

#[cfg(windows)]
#[test]
fn windows_owner_inventory_collapses_only_identical_shared_view_records() {
    let owner = WindowsRegisteredOwner {
        source: InstallSource::ExeCorporate,
        install_root: PathBuf::from(r"C:\Users\user\AppData\Local\Programs\honk300"),
        uninstall: WindowsManagedUninstall::Exe {
            uninstaller: PathBuf::from(
                r"C:\Users\user\AppData\Local\Programs\honk300\unins000.exe",
            ),
            elevated: false,
        },
        registration: "HKCU:64:{A072F01B-0AE8-4ED9-B67F-845ADF7831F9}_is1".into(),
        logical_registration: "HKCU:{a072f01b-0ae8-4ed9-b67f-845adf7831f9}_is1".into(),
    };
    let mut shared_views = vec![
        owner.clone(),
        WindowsRegisteredOwner {
            registration: owner.registration.replacen(":64:", ":32:", 1),
            ..owner.clone()
        },
    ];
    deduplicate_windows_registered_owners(&mut shared_views);
    assert_eq!(shared_views.len(), 1);

    let mut conflicting_views = vec![
        owner.clone(),
        WindowsRegisteredOwner {
            install_root: PathBuf::from(r"C:\Users\user\AppData\Local\Programs\foreign"),
            registration: owner.registration.replacen(":64:", ":32:", 1),
            ..owner
        },
    ];
    deduplicate_windows_registered_owners(&mut conflicting_views);
    assert_eq!(conflicting_views.len(), 2);
}

#[test]
fn windows_owner_cleanup_preserves_only_the_active_registration_identity() {
    let global = Path::new(r"C:\Program Files\honk300");
    let corporate = Path::new(r"C:\Users\user\AppData\Local\Programs\honk300");

    assert!(!windows_owner_conflicts(
        InstallSource::MsiGlobal,
        global,
        InstallSource::MsiGlobal,
        global,
    ));
    assert!(!windows_owner_conflicts(
        InstallSource::PowerShell,
        global,
        InstallSource::MsiGlobal,
        global,
    ));
    assert!(windows_owner_conflicts(
        InstallSource::ExeGlobal,
        global,
        InstallSource::MsiGlobal,
        global,
    ));
    assert!(windows_owner_conflicts(
        InstallSource::MsiCorporate,
        corporate,
        InstallSource::MsiGlobal,
        global,
    ));
    assert!(windows_owner_conflicts(
        InstallSource::MsiGlobal,
        global,
        InstallSource::MsiGlobal,
        corporate,
    ));
}

#[cfg(windows)]
#[test]
fn windows_cleanup_discovery_does_not_rewrite_an_already_current_receipt() {
    let root = test_dir("cleanup-receipt-idempotent");
    fs::create_dir_all(&root).unwrap();
    let receipt_path = root.join("install-receipt.json");
    let receipt = serde_json::json!({
        "schema": INSTALL_RECEIPT_V2,
        "cleanup": { "state": "inactive_releases_retained" }
    });
    let original = serde_json::to_vec_pretty(&receipt).unwrap();
    fs::write(&receipt_path, &original).unwrap();

    let original_permissions = fs::metadata(&receipt_path).unwrap().permissions();
    let mut read_only_permissions = original_permissions.clone();
    read_only_permissions.set_readonly(true);
    fs::set_permissions(&receipt_path, read_only_permissions).unwrap();

    let result = set_windows_receipt_cleanup_state(&root, receipt, "inactive_releases_retained");

    fs::set_permissions(&receipt_path, original_permissions).unwrap();
    assert!(result.is_ok());
    assert_eq!(fs::read(&receipt_path).unwrap(), original);
    assert!(!root
        .join(format!(
            ".install-receipt.cleanup.{}.tmp",
            std::process::id()
        ))
        .exists());

    let receipt: serde_json::Value =
        serde_json::from_slice(&fs::read(&receipt_path).unwrap()).unwrap();
    set_windows_receipt_cleanup_state(&root, receipt, "cleanup_pending").unwrap();
    let updated: serde_json::Value =
        serde_json::from_slice(&fs::read(&receipt_path).unwrap()).unwrap();
    assert_eq!(updated["cleanup"]["state"], "cleanup_pending");
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn corporate_uninstall_searches_user_then_machine_registration() {
    assert_eq!(
        windows_uninstall_hive_order(InstallSource::MsiCorporate),
        &[
            WindowsUninstallHive::CurrentUser,
            WindowsUninstallHive::LocalMachine,
        ]
    );
    assert_eq!(
        windows_uninstall_hive_order(InstallSource::ExeCorporate),
        &[
            WindowsUninstallHive::CurrentUser,
            WindowsUninstallHive::LocalMachine,
        ]
    );
    assert_eq!(
        windows_uninstall_hive_order(InstallSource::MsiGlobal),
        &[WindowsUninstallHive::LocalMachine]
    );
}

#[test]
fn windows_managed_uninstall_helper_is_hidden_and_never_deletes_install_root() {
    let plan = WindowsManagedUninstall::Msi {
        product_code: "{01234567-89AB-CDEF-0123-456789ABCDEF}".into(),
        elevated: true,
    };
    let invocation = windows_managed_uninstall_invocation(
        &plan,
        Path::new(r"C:\Program Files\honk300\bin\honk300.exe"),
        Path::new(r"C:\Windows\System32\msiexec.exe"),
    );
    assert!(invocation
        .args
        .windows(2)
        .any(|args| args == ["-WindowStyle", "Hidden"]));
    assert!(!invocation.script.contains("Wait-Process"));
    assert!(invocation.script.contains("rstrtmgr.dll"));
    assert!(invocation
        .script
        .contains("[Honk300RestartManagerProbe]::AssertUnlocked"));
    assert!(invocation
        .script
        .contains(r"C:\Windows\System32\msiexec.exe"));
    assert!(!invocation.script.contains("-FilePath 'msiexec.exe'"));
    assert!(invocation.script.contains("/x"));
    assert!(invocation.script.contains("@(0,1605)"));
    assert!(!invocation.script.contains("1641"));
    assert!(!invocation.script.contains("3010"));
    assert!(!invocation.script.contains("Remove-Item -Recurse"));
}

#[test]
fn windows_parent_wait_treats_an_already_exited_parent_as_success() {
    let script = windows_wait_for_parent_script(4242);
    assert!(script.contains("$ErrorActionPreference='Stop'"));
    assert!(script.contains("-ErrorAction SilentlyContinue"));
    assert!(script.contains("$process.WaitForExit()"));
    assert!(script.ends_with("exit 0"));
}

fn test_dir(name: &str) -> PathBuf {
    std::env::temp_dir().join(format!(
        "honk300-install-{name}-{}-{}",
        std::process::id(),
        unix_timestamp()
    ))
}
