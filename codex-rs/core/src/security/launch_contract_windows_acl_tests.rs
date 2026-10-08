//! #294 (PF-27-S06 gate, defect 2): under the product's `workspace-write`
//! profile, agent commands launched through the tool path
//! (`SandboxAttempt::env_for`) could read the vault store, because the
//! contract's deny entries never reached the Windows sandbox as ACLs.
//!
//! The end-to-end test needs the elevated sandbox. An elevated session sets
//! it up itself; a normal (medium-integrity) session cannot answer the setup's
//! UAC prompt, so set `CODEX_PF27S06_SETUP_SEED` to a directory holding the
//! `setup_marker.json` and `sandbox_users.json` of an earlier elevated setup
//! on the same machine and user (the sandbox's users are machine-wide).

use super::LaunchContract;
use crate::exec::ExecCapturePolicy;
use crate::exec::ExecExpiration;
use crate::sandboxing::ExecOptions;
use crate::tools::sandboxing::SandboxAttempt;
use codex_protocol::config_types::WindowsSandboxLevel;
use codex_protocol::models::PermissionProfile;
use codex_sandboxing::SandboxCommand;
use codex_sandboxing::SandboxManager;
use codex_sandboxing::SandboxType;
use codex_utils_absolute_path::AbsolutePathBuf;
use codex_utils_path_uri::PathUri;
use pretty_assertions::assert_eq;
use std::collections::HashMap;
use std::path::Path;

const SETUP_SEED_ENV: &str = "CODEX_PF27S06_SETUP_SEED";

/// The product's default profile: full read, workspace write.
fn product_workspace_write(cwd: &AbsolutePathBuf) -> PermissionProfile {
    PermissionProfile::workspace_write()
        .materialize_project_roots_with_workspace_roots(std::slice::from_ref(cwd))
}

/// Prepares `command` through the same tool path agent commands use.
fn tool_launch(
    profile: &PermissionProfile,
    cwd: &AbsolutePathBuf,
    command: Vec<String>,
) -> crate::sandboxing::ExecRequest {
    let cwd_uri = PathUri::from_abs_path(cwd);
    let manager = SandboxManager::new();
    let attempt = SandboxAttempt {
        sandbox: SandboxType::WindowsRestrictedToken,
        sandbox_requested: true,
        permissions: profile,
        exec_server_permissions: profile,
        enforce_managed_network: false,
        manager: &manager,
        sandbox_cwd: &cwd_uri,
        workspace_roots: std::slice::from_ref(&cwd_uri),
        codex_linux_sandbox_exe: None,
        use_legacy_landlock: false,
        windows_sandbox_level: WindowsSandboxLevel::Elevated,
        windows_sandbox_private_desktop: false,
        network_denial_cancellation_token: None,
        network_proxy: None,
    };
    let (program, args) = command.split_first().expect("command");
    attempt
        .env_for_with_contract(
            // The contract's profile is passed in directly: arming it here
            // would also need the sandbox's users group for the new-files deny.
            None,
            SandboxCommand {
                program: program.clone().into(),
                args: args.to_vec(),
                cwd: cwd_uri.clone(),
                env: HashMap::new(),
                managed_network: None,
                additional_permissions: None,
            },
            ExecOptions {
                expiration: ExecExpiration::from(120_000),
                capture_policy: ExecCapturePolicy::ShellTool,
            },
            None,
            None,
        )
        .expect("tool launch request")
}

#[test]
fn pf_27_s06_d2_tool_launch_carries_contract_denies_to_windows_sandbox() {
    let codex_home_dir = tempfile::tempdir().expect("codex home");
    let workspace_dir = tempfile::tempdir().expect("workspace");
    let codex_home = absolute(codex_home_dir.path());
    let cwd = absolute(workspace_dir.path());
    std::fs::create_dir_all(codex_home.join("secrets")).expect("secrets dir");
    std::fs::write(codex_home.join("secrets").join("local.age"), "vault").expect("vault");
    std::fs::write(codex_home.join("auth.json"), "{}").expect("auth");

    let contract = LaunchContract::capture(&codex_home, std::iter::empty(), /*hardened*/ true);
    let protected = contract
        .protect_permissions(&product_workspace_write(&cwd), cwd.as_path())
        .expect("protected profile");
    let request = tool_launch(&protected, &cwd, vec!["cmd.exe".into()]);

    let overrides = request
        .windows_sandbox_filesystem_overrides
        .expect("the tool path resolves Windows filesystem overrides");
    // Full read stays (no read-root override); the protected paths are denied.
    assert_eq!(overrides.read_roots_override, None);
    for denied in [
        codex_home.join("secrets"),
        codex_home.join("auth.json"),
        // Does not exist yet: the sandbox's ACL setup creates and denies it.
        codex_home.join("wallet"),
        codex_home.join("sessions"),
    ] {
        assert!(
            overrides.additional_deny_read_paths.contains(&denied),
            "{} missing from {:?}",
            denied.display(),
            overrides.additional_deny_read_paths
        );
    }
}

#[tokio::test]
async fn pf_27_s06_d2_vault_unreadable_through_tool_launch_under_workspace_write() {
    // Inside the user profile, like the default `~/.codex`.
    let profile_dir = std::env::var_os("USERPROFILE").expect("USERPROFILE");
    let codex_home_dir = tempfile::Builder::new()
        .prefix("pf27s06d2-home-")
        .tempdir_in(profile_dir)
        .expect("codex home");
    let workspace_dir = tempfile::tempdir().expect("workspace");
    let codex_home = absolute(codex_home_dir.path());
    let cwd = absolute(workspace_dir.path());
    let _codex_home = EnvGuard::set("CODEX_HOME", codex_home.as_path());
    stage_windows_sandbox_helpers();
    seed_elevated_setup(&codex_home);

    std::fs::create_dir_all(codex_home.join("secrets")).expect("secrets dir");
    std::fs::create_dir_all(codex_home.join("skills")).expect("skills dir");
    for (name, body) in [
        ("secrets\\local.age", "pf27s06d2-vault"),
        ("auth.json", "pf27s06d2-auth"),
        ("skills\\notes.txt", "unprotected control"),
    ] {
        std::fs::write(codex_home.join(name), body).expect("fixture file");
    }
    let home = codex_home.as_path().display().to_string();
    assert!(!home.contains(' '), "CODEX_HOME path has a space: {home}");
    let read = |label: &str, file: &str| {
        format!("(type {home}\\{file} 1>NUL 2>NUL && echo {label}-READ || echo {label}-DENIED)")
    };
    let script = [
        read("VAULT", "secrets\\local.age"),
        read("AUTH", "auth.json"),
        read("NOTES", "skills\\notes.txt"),
        read("WALLET", "wallet\\seed.json"),
    ]
    .join(" & ");
    let command = || vec!["cmd.exe".into(), "/D".into(), "/C".into(), script.clone()];
    let base = product_workspace_write(&cwd);

    // Positive control: without the contract the sandbox user reads them.
    let control = run(tool_launch(&base, &cwd, command())).await;
    eprintln!("pf27s06 d2 workspace-write, no contract: {control}");
    for readable in ["VAULT", "AUTH", "NOTES"] {
        assert!(
            control.contains(&format!("{readable}-READ")),
            "{readable}: {control}"
        );
    }

    let contract = LaunchContract::capture(&codex_home, std::iter::empty(), /*hardened*/ true);
    let protected = contract
        .protect_permissions(&base, cwd.as_path())
        .expect("protected profile");
    contract
        .protect_new_codex_home_files()
        .expect("deny new CODEX_HOME files to the sandbox");
    let files = run(tool_launch(&protected, &cwd, command())).await;
    eprintln!("pf27s06 d2 workspace-write, contract: {files}");
    assert!(files.contains("NOTES-READ"), "unprotected file: {files}");
    for denied in ["VAULT", "AUTH"] {
        assert!(
            files.contains(&format!("{denied}-DENIED")),
            "{denied}: {files}"
        );
    }

    // A protected directory that did not exist at launch (the wallet) was
    // created and denied by that launch, so a file written there later is
    // denied to the next command too.
    std::fs::create_dir_all(codex_home.join("wallet")).expect("wallet dir");
    std::fs::write(codex_home.join("wallet").join("seed.json"), "pf27s06d2-seed")
        .expect("wallet file");
    let later = run(tool_launch(&protected, &cwd, command())).await;
    eprintln!("pf27s06 d2 workspace-write, contract, later wallet: {later}");
    assert!(later.contains("WALLET-DENIED"), "{later}");
    assert!(later.contains("VAULT-DENIED"), "{later}");
}

async fn run(request: crate::sandboxing::ExecRequest) -> String {
    let output = crate::sandboxing::execute_env(request, /*stdout_stream*/ None)
        .await
        .expect("elevated sandbox run");
    format!("{}{}", output.stdout.text, output.stderr.text)
}

/// See the module docs: seeds the elevated setup for a normal session.
fn seed_elevated_setup(codex_home: &AbsolutePathBuf) {
    let Some(seed) = std::env::var_os(SETUP_SEED_ENV) else {
        return;
    };
    let seed = Path::new(&seed);
    for (file, dir) in [
        ("setup_marker.json", ".sandbox"),
        ("sandbox_users.json", ".sandbox-secrets"),
    ] {
        let dir = codex_home.join(dir);
        std::fs::create_dir_all(&dir).expect("seed dir");
        std::fs::copy(seed.join(file), dir.join(file)).expect("seed elevated setup");
    }
}

fn absolute(path: &Path) -> AbsolutePathBuf {
    AbsolutePathBuf::from_absolute_path(dunce::canonicalize(path).expect("canonical path"))
        .expect("absolute path")
}

/// Copies the elevated sandbox helpers next to this test binary, where the
/// sandbox looks for them.
fn stage_windows_sandbox_helpers() {
    let exe = std::env::current_exe().expect("test binary");
    let resources = exe.parent().expect("test dir").join("codex-resources");
    std::fs::create_dir_all(&resources).expect("resources dir");
    for helper in ["codex-windows-sandbox-setup", "codex-command-runner"] {
        let source = codex_utils_cargo_bin::cargo_bin(helper).expect("sandbox helper built");
        let destination = resources.join(format!("{helper}.exe"));
        if let Err(error) = std::fs::copy(&source, &destination)
            && !destination.exists()
        {
            panic!("stage {helper}: {error}");
        }
    }
}

struct EnvGuard {
    key: &'static str,
    original: Option<std::ffi::OsString>,
}

impl EnvGuard {
    fn set(key: &'static str, value: &Path) -> Self {
        let original = std::env::var_os(key);
        // SAFETY: the pf_27_s06 Windows tests run alone in their process.
        unsafe { std::env::set_var(key, value) };
        Self { key, original }
    }
}

impl Drop for EnvGuard {
    fn drop(&mut self) {
        // SAFETY: as in `set`.
        unsafe {
            match &self.original {
                Some(value) => std::env::set_var(self.key, value),
                None => std::env::remove_var(self.key),
            }
        }
    }
}
