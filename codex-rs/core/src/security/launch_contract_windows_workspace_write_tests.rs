//! #294 (PF-27-S06 gate, defect 2): under the product's `workspace-write`
//! profile, agent commands launched through the tool path
//! (`SandboxAttempt::env_for`) could read the vault store, because the
//! contract's deny entries never reached the Windows sandbox as ACLs.
//!
//! The end-to-end test needs the elevated sandbox. An elevated session sets
//! it up itself; a normal (medium-integrity) session cannot answer the setup's
//! UAC prompt, so set `CODEX_PF27S06_SETUP_SEED` to a directory holding the
//! `setup_marker.json` and `sandbox_users.json` of an earlier elevated setup
//! on the same machine and user (the sandbox's users are machine-wide). Every
//! elevated setup resets the sandbox users' passwords (which also invalidates
//! a real installation's saved setup), so record the seed with an elevated run
//! of this test (it writes the directory) right before. The seed holds those
//! passwords: keep the directory outside every sandbox read root. A run above
//! medium integrity records the seed; a run at or below it uses the seed.

// The probes' output is the evidence of these measured runs (`--nocapture`).
#![allow(clippy::print_stderr)]

use super::LaunchContract;
use super::windows_tests::EnvGuard;
use super::windows_tests::absolute;
use super::windows_tests::stage_windows_sandbox_helpers;
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

    // Positive control: without the contract the sandbox user reads them. On
    // a fresh machine the elevated setup grants read access to the user
    // profile in the background (this profile reads everything), so wait for
    // that first.
    let mut control = String::new();
    for _ in 0..120 {
        control = run(tool_launch(&base, &cwd, command())).await;
        if control.contains("NOTES-READ") {
            break;
        }
        tokio::time::sleep(std::time::Duration::from_secs(5)).await;
    }
    record_elevated_setup(&codex_home);
    eprintln!("pf27s06 d2 workspace-write, no contract: {control}");
    for readable in ["VAULT", "AUTH", "NOTES"] {
        assert!(
            control.contains(&format!("{readable}-READ")),
            "{readable}: {control}"
        );
    }

    let contract = capture_without_real_credentials(&codex_home);
    let protected = contract
        .protect_permissions(&base, cwd.as_path())
        .expect("protected profile");
    contract
        .protect_new_codex_home_files()
        .expect("deny new CODEX_HOME files to the sandbox");
    assert!(!codex_home.join("wallet").exists());
    let files = run(tool_launch(&protected, &cwd, command())).await;
    eprintln!("pf27s06 d2 workspace-write, contract: {files}");
    assert!(files.contains("NOTES-READ"), "unprotected file: {files}");
    for denied in ["VAULT", "AUTH"] {
        assert!(
            files.contains(&format!("{denied}-DENIED")),
            "{denied}: {files}"
        );
    }
    // The wallet did not exist; the launch's ACL setup created and denied it.
    assert!(codex_home.join("wallet").is_dir(), "wallet not created");

    // While a protected command runs, a wallet file appears and an
    // unprotected launch of the same armed process (a TUI workspace probe)
    // syncs the sandbox's deny ACEs. It must keep the contract's denies, or
    // the running command could read the vault again.
    let running = {
        let request = tool_launch(
            &protected,
            &cwd,
            vec![
                "cmd.exe".into(),
                "/D".into(),
                "/C".into(),
                format!("ping -n 6 127.0.0.1 >NUL & {script}"),
            ],
        );
        tokio::spawn(run(request))
    };
    tokio::time::sleep(std::time::Duration::from_secs(2)).await;
    std::fs::write(
        codex_home.join("wallet").join("seed.json"),
        "pf27s06d2-seed",
    )
    .expect("wallet file");
    let mut unprotected = tool_launch(&base, &cwd, command());
    crate::exec::attach_windows_sandbox_filesystem_overrides(
        &mut unprotected,
        &cwd,
        Some(&contract),
    )
    .expect("armed overrides");
    assert!(
        unprotected
            .windows_sandbox_filesystem_overrides
            .as_ref()
            .is_some_and(|overrides| overrides
                .additional_deny_read_paths
                .contains(&codex_home.join("secrets"))),
        "an armed process's unprotected launch keeps the contract's denies"
    );
    // The file exists, so a failed read below is an access denial.
    assert!(codex_home.join("wallet").join("seed.json").is_file());
    let probe = run(unprotected).await;
    eprintln!("pf27s06 d2 unprotected launch in an armed process: {probe}");
    assert!(probe.contains("VAULT-DENIED"), "{probe}");
    // Deny ACEs are never revoked today (#304), so this end-to-end check holds
    // even without the armed union; the assertion on the overrides above
    // guards the union until revocation works.
    let running = running.await.expect("running command");
    eprintln!("pf27s06 d2 protected command running across it: {running}");
    for denied in ["VAULT", "AUTH", "WALLET"] {
        assert!(
            running.contains(&format!("{denied}-DENIED")),
            "{denied}: {running}"
        );
    }
}

/// A contract for `codex_home` whose home-directory credential paths point
/// into an empty temporary directory, so the test leaves no deny ACE on the
/// machine's real credential files.
fn capture_without_real_credentials(codex_home: &AbsolutePathBuf) -> LaunchContract {
    let fake = tempfile::tempdir().expect("fake home");
    let _guards = [
        "HOME",
        "USERPROFILE",
        "APPDATA",
        "CARGO_HOME",
        "CLAUDE_CONFIG_DIR",
    ]
    .map(|key| EnvGuard::set(key, fake.path()));
    LaunchContract::capture(codex_home, std::iter::empty(), /*hardened*/ true)
}

async fn run(request: crate::sandboxing::ExecRequest) -> String {
    let output = crate::sandboxing::execute_env(request, /*stdout_stream*/ None)
        .await
        .expect("elevated sandbox run");
    format!("{}{}", output.stdout.text, output.stderr.text)
}

const SETUP_FILES: [(&str, &str); 2] = [
    ("setup_marker.json", ".sandbox"),
    ("sandbox_users.json", ".sandbox-secrets"),
];

/// See the module docs: seeds the elevated setup for a normal session.
fn seed_elevated_setup(codex_home: &AbsolutePathBuf) {
    let Some(seed) = std::env::var_os(SETUP_SEED_ENV) else {
        return;
    };
    if above_medium_integrity() {
        return;
    }
    let seed = Path::new(&seed);
    for (file, dir) in SETUP_FILES {
        if !seed.join(file).exists() {
            continue;
        }
        let dir = codex_home.join(dir);
        std::fs::create_dir_all(&dir).expect("seed dir");
        std::fs::copy(seed.join(file), dir.join(file)).expect("seed elevated setup");
    }
}

/// Records the setup an elevated run did into the seed directory.
fn record_elevated_setup(codex_home: &AbsolutePathBuf) {
    let Some(seed) = std::env::var_os(SETUP_SEED_ENV) else {
        return;
    };
    if !above_medium_integrity() {
        return;
    }
    let seed = Path::new(&seed);
    std::fs::create_dir_all(seed).expect("seed dir");
    for (file, dir) in SETUP_FILES {
        std::fs::copy(codex_home.join(dir).join(file), seed.join(file))
            .expect("record elevated setup");
    }
}

/// True when this process runs above medium integrity (an elevated session).
fn above_medium_integrity() -> bool {
    use windows_sys::Win32::Foundation::CloseHandle;
    use windows_sys::Win32::Security::GetSidSubAuthority;
    use windows_sys::Win32::Security::GetSidSubAuthorityCount;
    use windows_sys::Win32::Security::GetTokenInformation;
    use windows_sys::Win32::Security::TOKEN_MANDATORY_LABEL;
    use windows_sys::Win32::Security::TOKEN_QUERY;
    use windows_sys::Win32::Security::TokenIntegrityLevel;
    use windows_sys::Win32::System::Threading::GetCurrentProcess;
    use windows_sys::Win32::System::Threading::OpenProcessToken;
    /// `SECURITY_MANDATORY_MEDIUM_RID`.
    const MEDIUM: u32 = 0x2000;
    let mut token = 0;
    // SAFETY: opens this process's token for query; closed below.
    if unsafe { OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut token) } == 0 {
        return false;
    }
    let mut buffer = vec![0u8; 256];
    let mut length = 0;
    // SAFETY: `buffer` is large enough for a mandatory label and its SID.
    let ok = unsafe {
        GetTokenInformation(
            token,
            TokenIntegrityLevel,
            buffer.as_mut_ptr().cast(),
            buffer.len() as u32,
            &mut length,
        )
    };
    // SAFETY: opened above.
    unsafe { CloseHandle(token) };
    if ok == 0 {
        return false;
    }
    // SAFETY: the call filled `buffer` with a TOKEN_MANDATORY_LABEL whose SID
    // points into it.
    let level = unsafe {
        let label = &*(buffer.as_ptr() as *const TOKEN_MANDATORY_LABEL);
        let count = *GetSidSubAuthorityCount(label.Label.Sid);
        *GetSidSubAuthority(label.Label.Sid, u32::from(count) - 1)
    };
    level > MEDIUM
}
