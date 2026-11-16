//! D598: native foreign-owner records refuse launches instead of disabling tracking.

use super::*;
use pretty_assertions::assert_eq;

#[test]
fn sec_win_1953_foreign_records_refuse_elevated_capture_and_watch() {
    let fixture = user_owned_test_root();
    let local = fixture.path().join("LocalAppData");
    fs::create_dir(&local).expect("private LocalAppData");
    let _local = crate::integrity_leases::use_test_local_app_data(&local);
    let Some(home) = elevated_home("sec-1953-records") else {
        return;
    };
    let _guard = legacy_process_test_guard();
    let workspace = user_owned_test_root();
    let workspace = canonical(workspace.path());
    assert_eq!(
        run_elevated_capture(&workspace, home.path(), "echo foreign> foreign.json"),
        0
    );
    let foreign = workspace.join("foreign.json");
    // The producer was a real sandbox account, not a simulated owner string.
    let error = crate::integrity_registry::check_path(&foreign).expect_err("sandbox owner");
    assert!(error.is::<crate::integrity_registry::UntrustedRecords>());
    let registry = crate::integrity_leases::registry_parent_dir()
        .expect("parent")
        .join("integrity-labels");
    let profile = PermissionProfile::workspace_write();
    let roots = workspace_roots_for(&workspace);
    let permissions = crate::resolved_permissions::ResolvedWindowsSandboxPermissions::try_from_permission_profile_for_workspace_roots(
        &profile, roots.as_slice(),
    ).expect("permissions");

    for name in [
        "planted-metadata.json",
        "registry.lock",
        "labels.json",
        "leftover.tmp",
    ] {
        let record = registry.join(name);
        let backup = workspace.join("host-backup");
        let existed = record.exists();
        if existed {
            fs::rename(&record, &backup).expect("preserve host record");
        }
        // Rename on the same volume preserves the sandbox account's ownership.
        fs::rename(&foreign, &record).expect("plant foreign record");
        assert!(
            crate::planted_project_metadata().is_err(),
            "{name}: trusted planted records"
        );
        assert!(
            crate::planted_stamps().is_err(),
            "{name}: trusted record stamps"
        );
        let watched = crate::spawn_prep::watch_elevated_write_roots(
            &permissions,
            &workspace,
            std::slice::from_ref(&workspace),
            /*log_dir*/ None,
        );
        assert!(
            matches!(watched, Err(ref err) if err.is::<crate::integrity_registry::UntrustedRecords>()),
            "{name}: watch silently disabled"
        );
        let legacy = crate::integrity_leases::acquire_watching(
            std::slice::from_ref(&workspace),
            &[],
            /*log_dir*/ None,
        );
        assert!(
            matches!(legacy, Err(ref err) if err.is::<crate::integrity_registry::UntrustedRecords>()),
            "{name}: legacy accepted records"
        );
        let result = crate::run_windows_sandbox_capture_for_permission_profile_elevated(
            crate::ElevatedSandboxProfileCaptureRequest {
                permission_profile: &profile,
                workspace_roots: roots.as_slice(),
                codex_home: home.path(),
                command: vec![
                    system32_exe("cmd.exe").display().to_string(),
                    "/d".into(),
                    "/c".into(),
                    "echo bad> executed.txt".into(),
                ],
                cwd: &workspace,
                env_map: std::env::vars().collect(),
                timeout_ms: Some(90_000),
                cancellation: None,
                use_private_desktop: true,
                proxy_enforced: false,
                network_proxy_restricting_sid: None,
                read_roots_override: None,
                read_roots_include_platform_defaults: true,
                write_roots_override: None,
                deny_read_override: &crate::DenyReadTargets::default(),
                deny_write_paths_override: &[],
            },
        );
        assert!(
            matches!(result, Err(ref err) if err.is::<crate::integrity_registry::UntrustedRecords>()),
            "{name}: capture did not refuse untrusted records"
        );
        assert!(
            !workspace.join("executed.txt").exists(),
            "{name}: command ran"
        );
        assert_eq!(
            fs::read_to_string(&record).expect("foreign bytes"),
            "foreign\r\n"
        );
        fs::rename(&record, &foreign).expect("remove test-owned foreign record");
        if existed {
            fs::rename(&backup, &record).expect("restore host record");
        }
    }
    assert_eq!(
        run_elevated_capture(&workspace, home.path(), "echo recovered> recovered.txt"),
        0
    );
    assert!(workspace.join("recovered.txt").exists());
}
