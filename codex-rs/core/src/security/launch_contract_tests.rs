use super::*;
use codex_protocol::models::PermissionProfile;
use codex_protocol::permissions::NetworkSandboxPolicy;
use pretty_assertions::assert_eq;
use std::ffi::OsString;

const RAW: &str = "ghp_pf27s02SyntheticCanary000000000000000000";

struct Fixture {
    _codex_home: tempfile::TempDir,
    _workspace: tempfile::TempDir,
    codex_home: AbsolutePathBuf,
    workspace: AbsolutePathBuf,
    contract: LaunchContract,
}

fn fixture() -> Fixture {
    let codex_home_dir = tempfile::tempdir().expect("codex home");
    let workspace_dir = tempfile::tempdir().expect("workspace");
    let codex_home =
        AbsolutePathBuf::from_absolute_path(codex_home_dir.path()).expect("absolute home");
    let workspace =
        AbsolutePathBuf::from_absolute_path(workspace_dir.path()).expect("absolute workspace");
    let vars = [
        ("GITHUB_TOKEN", RAW),
        ("SHORT_TOKEN", "abc"),
        ("PATH", "/usr/bin:/bin"),
        ("EDITOR", "vim-but-long-enough"),
    ]
    .into_iter()
    .map(|(name, value)| (OsString::from(name), OsString::from(value)));
    let contract = LaunchContract::capture(&codex_home, vars, /*hardened*/ true);
    Fixture {
        _codex_home: codex_home_dir,
        _workspace: workspace_dir,
        codex_home,
        workspace,
        contract,
    }
}

fn workspace_profile(workspace: &AbsolutePathBuf) -> PermissionProfile {
    PermissionProfile::workspace_write_with(
        &[],
        NetworkSandboxPolicy::Enabled,
        /*exclude_tmpdir_env_var*/ false,
        /*exclude_slash_tmp*/ false,
    )
    .materialize_project_roots_with_workspace_roots(std::slice::from_ref(workspace))
}

#[test]
fn pf_27_s02_only_secret_looking_values_are_managed() {
    let fixture = fixture();
    assert!(fixture.contract.contains_managed_value(RAW));
    assert!(!fixture.contract.contains_managed_value("abc"));
    assert!(
        !fixture
            .contract
            .contains_managed_value("vim-but-long-enough")
    );
}

#[test]
fn pf_27_s02_unsandboxed_and_unhardened_launches_are_refused() {
    let fixture = fixture();
    let contract = &fixture.contract;
    if cfg!(any(target_os = "macos", target_os = "linux")) {
        assert_eq!(
            contract.check_sandbox(
                SandboxType::None,
                /*sandbox_requested*/ false,
                /*exec_server*/ false,
                /*windows_elevated*/ false
            ),
            Err(LaunchDenied::Unsandboxed)
        );
        assert_eq!(
            contract.check_sandbox(
                SandboxType::None,
                /*sandbox_requested*/ true,
                /*exec_server*/ false,
                /*windows_elevated*/ false
            ),
            Err(LaunchDenied::Unsandboxed)
        );
        // Remote environments build the child environment on their own host.
        assert_eq!(
            contract.check_sandbox(
                SandboxType::None,
                /*sandbox_requested*/ true,
                /*exec_server*/ true,
                /*windows_elevated*/ false
            ),
            Err(LaunchDenied::RemoteEnvironment)
        );
        let unhardened = LaunchContract::capture(
            &fixture.codex_home,
            std::iter::empty(),
            /*hardened*/ false,
        );
        assert_eq!(
            unhardened.check_sandbox(
                SandboxType::MacosSeatbelt,
                /*sandbox_requested*/ true,
                /*exec_server*/ false,
                /*windows_elevated*/ false
            ),
            Err(LaunchDenied::ProcessHardening)
        );
    }
}

/// PF-27-S06: Windows passes only under the elevated sandbox backend.
#[cfg(windows)]
#[test]
fn pf_27_s06_windows_requires_the_elevated_sandbox() {
    let fixture = fixture();
    let contract = &fixture.contract;
    let check = |sandbox, requested, elevated| {
        contract.check_sandbox(sandbox, requested, /*exec_server*/ false, elevated)
    };
    assert_eq!(
        check(SandboxType::WindowsRestrictedToken, true, true),
        Ok(())
    );
    assert_eq!(
        check(SandboxType::WindowsRestrictedToken, true, false),
        Err(LaunchDenied::WindowsUnelevatedSandbox)
    );
    assert_eq!(
        check(SandboxType::None, true, true),
        Err(LaunchDenied::Unsandboxed)
    );
    assert_eq!(
        check(SandboxType::WindowsRestrictedToken, false, true),
        Err(LaunchDenied::Unsandboxed)
    );
    let unhardened = LaunchContract::capture(
        &fixture.codex_home,
        std::iter::empty(),
        /*hardened*/ false,
    );
    assert_eq!(
        unhardened.check_sandbox(
            SandboxType::WindowsRestrictedToken,
            /*sandbox_requested*/ true,
            /*exec_server*/ false,
            /*windows_elevated*/ true
        ),
        Err(LaunchDenied::ProcessHardening)
    );
}

/// PF-27-S06: the Windows sandbox drops skip-if-missing entries, so paths
/// that exist are denied with plain entries there.
#[cfg(windows)]
#[test]
fn pf_27_s06_existing_protected_paths_are_denied_outright_on_windows() {
    use codex_protocol::permissions::FileSystemPath;
    let fixture = fixture();
    std::fs::write(fixture.codex_home.join("auth.json"), "{}").expect("auth");
    let protected = fixture
        .contract
        .protect_permissions(
            &workspace_profile(&fixture.workspace),
            fixture.workspace.as_path(),
        )
        .expect("protected");
    let entries = protected.file_system_sandbox_policy().entries;
    let entry_for = |name: &str| {
        let path = fixture.codex_home.join(name);
        entries
            .iter()
            .find(|entry| entry.path == FileSystemPath::Path { path: path.clone() })
            .cloned()
            .unwrap_or_else(|| panic!("no entry for {name}"))
    };
    assert!(!entry_for("auth.json").skips_missing_path());
    assert!(entry_for("provider_auth.json").skips_missing_path());
}

#[test]
fn pf_27_s02_protected_profile_denies_vault_auth_and_policy_store() {
    let fixture = fixture();
    let profile = workspace_profile(&fixture.workspace);
    let cwd = fixture.workspace.as_path();
    // The ordinary workspace profile can read the vault store.
    assert_eq!(
        fixture.contract.verify_permissions(&profile, cwd),
        Err(LaunchDenied::ProtectedPathReadable(
            fixture.codex_home.join("secrets").display().to_string()
        ))
    );

    let protected = fixture
        .contract
        .protect_permissions(&profile, cwd)
        .expect("protectable");
    let file_system = protected.file_system_sandbox_policy();
    for entry in [
        "secrets",
        "auth.json",
        ".env",
        "provider_auth.json",
        "config.toml",
        "wallet",
        "run",
        "shell_snapshots",
        "log",
    ] {
        assert!(
            !file_system.can_read_path_with_cwd(fixture.codex_home.join(entry).as_path(), cwd),
            "{entry} must be unreadable"
        );
    }
    assert!(
        file_system
            .get_unreadable_globs_with_cwd(cwd)
            .contains(&fixture.codex_home.join("*.sqlite*").display().to_string())
    );
    assert!(!file_system.can_write_path_with_cwd(fixture.codex_home.as_path(), cwd));
    assert!(file_system.can_write_path_with_cwd(&fixture.workspace.join("src.rs"), cwd));
    assert!(file_system.can_read_path_with_cwd(&fixture.codex_home.join("skills"), cwd));
    assert_eq!(fixture.contract.verify_permissions(&protected, cwd), Ok(()));
}

#[test]
fn pf_27_s02_full_access_and_writable_codex_home_are_refused() {
    let fixture = fixture();
    let cwd = fixture.workspace.as_path();
    assert_eq!(
        fixture
            .contract
            .protect_permissions(&PermissionProfile::Disabled, cwd),
        Err(LaunchDenied::Unsandboxed)
    );
    assert_eq!(
        fixture.contract.protect_permissions(
            &PermissionProfile::External {
                network: NetworkSandboxPolicy::Enabled
            },
            cwd
        ),
        Err(LaunchDenied::Unsandboxed)
    );
    // Working inside CODEX_HOME would let commands rewrite the policy store.
    let inside_home = workspace_profile(&fixture.codex_home);
    assert_eq!(
        fixture
            .contract
            .protect_permissions(&inside_home, fixture.codex_home.as_path()),
        Err(LaunchDenied::PolicyStoreWritable)
    );
}

#[test]
fn pf_27_s02_command_checks_refuse_login_shells_and_raw_argv() {
    let fixture = fixture();
    let argv = |args: &[&str]| args.iter().map(ToString::to_string).collect::<Vec<_>>();
    let mut env = HashMap::new();
    for login in [
        argv(&["/bin/zsh", "-lc", "true"]),
        argv(&["bash", "--login", "-c", "true"]),
        argv(&["/bin/sh", "-l"]),
        argv(&["-zsh"]),
    ] {
        assert_eq!(
            fixture.contract.check_command(&login, &mut env),
            Err(LaunchDenied::LoginShell),
            "{login:?}"
        );
    }
    assert_eq!(
        fixture
            .contract
            .check_command(&argv(&["/bin/zsh", "-c", "ls -l"]), &mut env),
        Ok(())
    );
    assert_eq!(
        fixture.contract.check_command(
            &argv(&["curl", "-H", &format!("Authorization: Bearer {RAW}")]),
            &mut env
        ),
        Err(LaunchDenied::RawSecretInArgv)
    );
}

#[test]
fn pf_27_s02_environment_keeps_dummies_but_never_raw_values() {
    let fixture = fixture();
    let dummy = "ghp_DummyValueFromTheBrokerXXXXXXXXXXXXXXXX".to_string();
    let mut env = HashMap::from([
        ("GITHUB_TOKEN".to_string(), dummy.clone()),
        ("LEAKED".to_string(), format!("prefix-{RAW}")),
        (
            "HTTPS_PROXY".to_string(),
            "http://me:pw@corp-proxy:3128".to_string(),
        ),
        ("PATH".to_string(), "/usr/bin".to_string()),
    ]);
    fixture
        .contract
        .check_command(&["/bin/sh".to_string(), "-c".to_string()], &mut env)
        .expect("allowed");
    let mut expected = HashMap::from([
        ("GITHUB_TOKEN".to_string(), dummy),
        ("PATH".to_string(), "/usr/bin".to_string()),
    ]);
    // zsh's start-up folder is pinned on Unix only (`check_command`).
    if cfg!(unix) {
        expected.insert("ZDOTDIR".to_string(), PROTECTED_ZDOTDIR.to_string());
    }
    assert_eq!(env, expected);
}

#[test]
fn pf_27_s02_stdin_with_a_managed_secret_is_refused() {
    let fixture = fixture();
    assert_eq!(fixture.contract.check_stdin(b"echo hello\n"), Ok(()));
    assert_eq!(
        fixture
            .contract
            .check_stdin(format!("export T={RAW}\n").as_bytes()),
        Err(LaunchDenied::RawSecretInStdin)
    );
}

#[test]
fn pf_27_s02_refusals_explain_themselves() {
    assert_eq!(
        LaunchDenied::Unsandboxed.to_string(),
        "Protected launch refused: this command would run outside the OS sandbox, which protected launch does not allow."
    );
}

#[test]
fn pf_27_s02_writable_roots_inside_codex_home_are_refused() {
    let fixture = fixture();
    let plugins = fixture.codex_home.join("plugins");
    let profile = PermissionProfile::workspace_write_with(
        std::slice::from_ref(&plugins),
        NetworkSandboxPolicy::Enabled,
        /*exclude_tmpdir_env_var*/ true,
        /*exclude_slash_tmp*/ true,
    )
    .materialize_project_roots_with_workspace_roots(std::slice::from_ref(&fixture.workspace));
    assert_eq!(
        fixture
            .contract
            .protect_permissions(&profile, fixture.workspace.as_path()),
        Err(LaunchDenied::PolicyStoreWritable)
    );
}

#[test]
fn pf_27_s02_file_tools_fail_closed_when_unprotectable() {
    let fixture = fixture();
    let cwd = fixture.workspace.as_path();
    let denied = fixture.contract.file_tool_permissions(
        &PermissionProfile::Disabled,
        /*sandboxed*/ true,
        cwd,
    );
    let file_system = denied.file_system_sandbox_policy();
    assert!(!file_system.can_read_path_with_cwd(&fixture.workspace.join("a.txt"), cwd));
    assert!(!file_system.can_write_path_with_cwd(&fixture.workspace.join("a.txt"), cwd));

    let protected = fixture.contract.file_tool_permissions(
        &workspace_profile(&fixture.workspace),
        /*sandboxed*/ true,
        cwd,
    );
    let file_system = protected.file_system_sandbox_policy();
    assert!(file_system.can_write_path_with_cwd(&fixture.workspace.join("a.txt"), cwd));
    assert!(!file_system.can_read_path_with_cwd(&fixture.codex_home.join("auth.json"), cwd));
}

#[test]
fn pf_27_s02_only_policy_permitted_broker_keys_are_sourced() {
    use codex_protocol::config_types::EnvironmentVariablePattern;
    use codex_protocol::config_types::ShellEnvironmentPolicy;
    let vars = || {
        [
            ("GITHUB_TOKEN", "ghp_x"),
            ("GH_TOKEN", "ghp_y"),
            ("OPENAI_API_KEY", "sk-x"),
            ("PATH", "/usr/bin"),
        ]
        .map(|(name, value)| (name.to_string(), value.to_string()))
    };
    let mut keys = permitted_brokered_env_keys(vars(), &ShellEnvironmentPolicy::default(), &[]);
    keys.sort();
    // Provider keys Core strips for the shell tool are never brokered.
    assert_eq!(
        keys,
        vec!["GH_TOKEN".to_string(), "GITHUB_TOKEN".to_string()]
    );

    let excluding = ShellEnvironmentPolicy {
        exclude: vec![EnvironmentVariablePattern::new_case_insensitive("GH_TOKEN")],
        ..ShellEnvironmentPolicy::default()
    };
    assert_eq!(
        permitted_brokered_env_keys(vars(), &excluding, &["GITHUB_TOKEN".to_string()]),
        Vec::<String>::new()
    );
}

/// #218: an external agent CLI launch (contained Claude panes) gets the same
/// checks: sandbox required, raw managed values refused in argv and removed
/// from the environment, protected reads denied.
#[test]
fn contained_external_agent_launch_gets_the_whole_contract() {
    if !cfg!(any(target_os = "macos", target_os = "linux")) {
        return;
    }
    let fixture = fixture();
    let contract = &fixture.contract;
    let cwd = fixture.workspace.as_path();
    let profile = workspace_profile(&fixture.workspace);
    let sandbox = if cfg!(target_os = "macos") {
        SandboxType::MacosSeatbelt
    } else {
        SandboxType::LinuxSeccomp
    };
    let argv = ["claude".to_string(), "-p".to_string()];
    let mut env = HashMap::from([
        ("PATH".to_string(), "/usr/bin".to_string()),
        ("LEAKED".to_string(), format!("prefix-{RAW}")),
    ]);

    assert_eq!(
        contract.protect_external_launch(SandboxType::None, &argv, &mut env, &profile, cwd),
        Err(LaunchDenied::Unsandboxed)
    );
    let raw_argv = ["claude".to_string(), RAW.to_string()];
    assert_eq!(
        contract.protect_external_launch(sandbox, &raw_argv, &mut env, &profile, cwd),
        Err(LaunchDenied::RawSecretInArgv)
    );

    let protected = contract
        .protect_external_launch(sandbox, &argv, &mut env, &profile, cwd)
        .expect("protectable");
    assert!(!env.contains_key("LEAKED"));
    assert_eq!(env.get("PATH").map(String::as_str), Some("/usr/bin"));
    let file_system = protected.file_system_sandbox_policy();
    assert!(!file_system.can_read_path_with_cwd(fixture.codex_home.join("secrets").as_path(), cwd));
    assert!(!file_system.can_write_path_with_cwd(fixture.codex_home.as_path(), cwd));
    assert!(file_system.can_write_path_with_cwd(&fixture.workspace.join("src.rs"), cwd));
}

#[test]
fn pf_27_s06_protected_codex_home_dirs_are_protected_entries() {
    for dir in super::PROTECTED_CODEX_HOME_DIRS {
        assert!(super::PROTECTED_CODEX_HOME_ENTRIES.contains(dir), "{dir}");
    }
}

/// #1107: what Corbanu keeps in the home besides its databases (recovery's
/// `db-backups`, pending accounting records and the GPU controller's lock)
/// is unreadable, as in a separate database folder; the folders are denied
/// before they exist on Windows.
#[test]
fn issue_1107_the_homes_database_folder_entries_are_protected() {
    let fixture = fixture();
    let cwd = fixture.workspace.as_path();
    let backup = fixture.codex_home.join("db-backups/1/state_5.sqlite");
    std::fs::create_dir_all(backup.as_path().parent().expect("parent")).expect("backups");
    std::fs::write(backup.as_path(), "db").expect("backup");
    let pending = fixture.codex_home.join("accounting-pending/x.json");
    std::fs::create_dir_all(pending.as_path().parent().expect("parent")).expect("pending");
    std::fs::write(pending.as_path(), "{}").expect("record");
    std::fs::write(fixture.codex_home.join("gpu-controller.lock"), "").expect("gpu lock");
    let protected = fixture
        .contract
        .protect_permissions(&workspace_profile(&fixture.workspace), cwd)
        .expect("protectable");
    let file_system = protected.file_system_sandbox_policy();
    for path in [
        fixture.codex_home.join("db-backups"),
        backup,
        fixture.codex_home.join("accounting-pending"),
        pending,
        fixture.codex_home.join("gpu-controller.lock"),
    ] {
        assert!(
            !file_system.can_read_path_with_cwd(path.as_path(), cwd),
            "{}",
            path.display()
        );
    }
    assert_eq!(fixture.contract.verify_permissions(&protected, cwd), Ok(()));
    for entry in super::super::preflight::DATABASE_FOLDER_ENTRIES {
        assert!(
            super::PROTECTED_CODEX_HOME_ENTRIES.contains(entry),
            "{entry}"
        );
    }
    for dir in ["db-backups", "accounting-pending"] {
        assert!(super::PROTECTED_CODEX_HOME_DIRS.contains(&dir), "{dir}");
    }
}

/// The Google Cloud CLI's folder (Windows only, so placed by hand here) and
/// what is in it are told apart from CLI token files.
#[test]
fn the_google_cloud_cli_folder_is_its_own_kind_of_kept_file() {
    let mut fixture = fixture();
    let cwd = fixture.workspace.as_path();
    // By its real location, as the deny matcher compares paths (macOS
    // temporary folders are reached through `/var -> /private/var`).
    let real_workspace = AbsolutePathBuf::from_absolute_path(
        fixture
            .workspace
            .as_path()
            .canonicalize()
            .expect("real workspace"),
    )
    .expect("absolute");
    let folder = real_workspace.join("AppData/Roaming/gcloud");
    std::fs::create_dir_all(folder.as_path()).expect("gcloud folder");
    std::fs::write(folder.join("credentials.db"), "").expect("credentials");
    fixture.contract.protected_read_paths.push(folder.clone());
    fixture
        .contract
        .google_cloud_cli_folders
        .push(folder.clone());
    let kept = |path: &Path| fixture.contract.kept_file(path, cwd);
    assert_eq!(kept(folder.as_path()), Some(KeptFile::GoogleCloudCliFolder));
    assert_eq!(
        kept(folder.join("credentials.db").as_path()),
        Some(KeptFile::InGoogleCloudCliFolder)
    );
    assert_eq!(kept(real_workspace.join("src.rs").as_path()), None);
}

/// On Windows the contract keeps `%APPDATA%\gcloud` as the Google Cloud
/// CLI's folder, not as a CLI token file.
#[cfg(windows)]
#[test]
fn the_google_cloud_cli_folder_is_kept_on_windows() {
    let Some(app_data) = std::env::var_os("APPDATA") else {
        return;
    };
    let fixture = fixture();
    let folder = AbsolutePathBuf::from_absolute_path(app_data)
        .expect("absolute APPDATA")
        .join("gcloud");
    assert_eq!(
        fixture.contract.google_cloud_cli_folders,
        vec![folder.clone()]
    );
    assert!(!fixture.contract.cli_token_paths.contains(&folder));
    assert_eq!(
        fixture
            .contract
            .kept_file(folder.as_path(), fixture.workspace.as_path()),
        Some(KeptFile::GoogleCloudCliFolder)
    );
}

/// #651: the contract also denies the home's database init lock and, once a
/// config names a database folder set apart from the home, that folder's
/// databases (its path escaped as a glob) and what Corbanu keeps there. A
/// folder that is the home adds nothing.
#[test]
fn issue_651_a_separate_database_folder_is_protected() {
    let fixture = fixture();
    let cwd = fixture.workspace.as_path();
    let databases_dir = tempfile::tempdir().expect("databases");
    let databases = AbsolutePathBuf::from_absolute_path(databases_dir.path().join("db[1]{2"))
        .expect("absolute");
    std::fs::create_dir_all(databases.as_path()).expect("database folder");
    fixture
        .contract
        .note_database_folder(fixture.codex_home.as_path());
    assert_eq!(fixture.contract.database_folders(), Vec::new());
    fixture.contract.note_database_folder(databases.as_path());
    let protected = fixture
        .contract
        .protect_permissions(&workspace_profile(&fixture.workspace), cwd)
        .expect("protectable");
    let file_system = protected.file_system_sandbox_policy();
    let globs = file_system.get_unreadable_globs_with_cwd(cwd);
    // Built with the platform's separator, as the policy resolves globs.
    let escaped = databases_dir
        .path()
        .join("db[[]1][{]2")
        .join("*.sqlite*")
        .display()
        .to_string();
    assert!(globs.contains(&escaped), "{globs:?}");
    for path in [
        fixture.codex_home.join("pfterminal_db_init.lock"),
        databases.join("pfterminal_db_init.lock"),
        databases.join("gpu-controller.lock"),
        databases.join("accounting-pending"),
        databases.join("db-backups"),
    ] {
        assert!(
            !file_system.can_read_path_with_cwd(path.as_path(), cwd),
            "{}",
            path.display()
        );
    }
    assert_eq!(fixture.contract.verify_permissions(&protected, cwd), Ok(()));
}

/// #651 item 7: a database folder missing at launch, in a folder the command
/// could write, is made (private, with what Corbanu keeps there) on Linux,
/// so bubblewrap holds no missing path with a placeholder file on the host;
/// its entries are then denied as when it exists. Backups beside a folder
/// are denied only when present on Linux (a missing one would get a
/// placeholder there too), and before they exist elsewhere.
#[cfg(unix)]
#[test]
fn issue_651_a_missing_database_folder_is_made_on_linux() {
    let fixture = fixture();
    let cwd = fixture.workspace.as_path();
    let databases = fixture.workspace.join("data/db");
    fixture.contract.note_database_folder(databases.as_path());
    let protected = fixture
        .contract
        .protect_permissions(&workspace_profile(&fixture.workspace), cwd)
        .expect("protectable");
    let linux = cfg!(target_os = "linux");
    assert_eq!(databases.as_path().is_dir(), linux);
    assert_eq!(
        databases.join("accounting-pending").as_path().is_dir(),
        linux
    );
    let file_system = protected.file_system_sandbox_policy();
    for path in [
        databases.join("pfterminal_db_init.lock"),
        databases.join("gpu-controller.lock"),
        databases.join("accounting-pending/x.json"),
    ] {
        assert!(
            !file_system.can_read_path_with_cwd(path.as_path(), cwd),
            "{}",
            path.display()
        );
    }
    let backups = fixture.workspace.join("data/db.db-backups");
    let denies_backups = |file_system: &FileSystemSandboxPolicy| {
        file_system
            .entries
            .iter()
            .any(|entry| matches!(&entry.path, FileSystemPath::Path { path } if *path == backups))
    };
    assert_eq!(denies_backups(&file_system), !linux);
    std::fs::create_dir_all(backups.as_path()).expect("backups");
    let protected = fixture
        .contract
        .protect_permissions(&workspace_profile(&fixture.workspace), cwd)
        .expect("protectable");
    assert!(denies_backups(&protected.file_system_sandbox_policy()));
}
