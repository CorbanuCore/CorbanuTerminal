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
            contract.check_sandbox(SandboxType::None, /*sandbox_requested*/ false, false),
            Err(LaunchDenied::Unsandboxed)
        );
        assert_eq!(
            contract.check_sandbox(SandboxType::None, /*sandbox_requested*/ true, false),
            Err(LaunchDenied::Unsandboxed)
        );
        // Remote environments build the child environment on their own host.
        assert_eq!(
            contract.check_sandbox(SandboxType::None, /*sandbox_requested*/ true, true),
            Err(LaunchDenied::RemoteEnvironment)
        );
        let unhardened = LaunchContract::capture(
            &fixture.codex_home,
            std::iter::empty(),
            /*hardened*/ false,
        );
        assert_eq!(
            unhardened.check_sandbox(SandboxType::MacosSeatbelt, true, false),
            Err(LaunchDenied::ProcessHardening)
        );
    } else {
        assert_eq!(
            contract.check_sandbox(SandboxType::WindowsRestrictedToken, true, false),
            Err(LaunchDenied::UnsupportedPlatform)
        );
    }
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
    assert_eq!(
        env,
        HashMap::from([
            ("GITHUB_TOKEN".to_string(), dummy),
            ("PATH".to_string(), "/usr/bin".to_string()),
            ("ZDOTDIR".to_string(), PROTECTED_ZDOTDIR.to_string()),
        ])
    );
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
