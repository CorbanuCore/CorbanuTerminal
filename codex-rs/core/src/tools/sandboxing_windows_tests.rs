//! #300 on a real Windows host: an agent command launched through the tool
//! path (`SandboxAttempt::env_for`) under the unelevated restricted-token
//! sandbox must not read a file its profile denies. That backend cannot block
//! reads, so the launch is refused up front with a message that says how to
//! switch to the elevated sandbox. Run it in a normal (medium-integrity)
//! session too: the unelevated sandbox is what such a session uses by default.

// The probe's output is the evidence of the measured runs (`--nocapture`).
#![allow(clippy::print_stderr)]

use super::SandboxAttempt;
use crate::exec::ExecCapturePolicy;
use crate::exec::ExecExpiration;
use crate::sandboxing::ExecOptions;
use codex_protocol::config_types::WindowsSandboxLevel;
use codex_protocol::error::CodexErr;
use codex_protocol::models::PermissionProfile;
use codex_protocol::permissions::FileSystemAccessMode;
use codex_protocol::permissions::FileSystemPath;
use codex_protocol::permissions::FileSystemSandboxEntry;
use codex_protocol::permissions::NetworkSandboxPolicy;
use codex_sandboxing::SandboxCommand;
use codex_sandboxing::SandboxManager;
use codex_sandboxing::SandboxType;
use codex_utils_absolute_path::AbsolutePathBuf;
use codex_utils_path_uri::PathUri;
use pretty_assertions::assert_eq;
use std::collections::HashMap;
use std::ffi::OsString;
use std::path::Path;

const SECRET: &str = "sec-win-300-secret";
const PUBLIC: &str = "sec-win-300-public";

struct EnvGuard {
    key: &'static str,
    original: Option<OsString>,
}

impl EnvGuard {
    fn set(key: &'static str, value: &Path) -> Self {
        let original = std::env::var_os(key);
        // SAFETY: serialized with the other CODEX_HOME tests; CI runs each
        // test in its own process.
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

fn absolute(path: &Path) -> AbsolutePathBuf {
    AbsolutePathBuf::from_absolute_path(dunce::canonicalize(path).expect("canonical path"))
        .expect("absolute path")
}

/// Prepares `cmd /C type secret.env & type public.txt` through the tool path
/// agent commands use, under the unelevated sandbox.
fn unelevated_tool_launch(
    permissions: &PermissionProfile,
    cwd: &AbsolutePathBuf,
) -> Result<crate::sandboxing::ExecRequest, CodexErr> {
    let cwd_uri = PathUri::from_abs_path(cwd);
    let manager = SandboxManager::new();
    SandboxAttempt {
        sandbox: SandboxType::WindowsRestrictedToken,
        sandbox_requested: true,
        permissions,
        exec_server_permissions: permissions,
        enforce_managed_network: false,
        manager: &manager,
        sandbox_cwd: &cwd_uri,
        workspace_roots: std::slice::from_ref(&cwd_uri),
        codex_linux_sandbox_exe: None,
        use_legacy_landlock: false,
        windows_sandbox_level: WindowsSandboxLevel::RestrictedToken,
        windows_sandbox_private_desktop: false,
        network_denial_cancellation_token: None,
        network_proxy: None,
    }
    .env_for_with_contract(
        /*contract*/ None,
        SandboxCommand {
            program: "cmd.exe".into(),
            args: vec![
                "/D".to_string(),
                "/C".to_string(),
                "type secret.env & type public.txt".to_string(),
            ],
            cwd: cwd_uri.clone(),
            env: HashMap::new(),
            managed_network: None,
            additional_permissions: None,
        },
        ExecOptions {
            expiration: ExecExpiration::from(60_000),
            capture_policy: ExecCapturePolicy::ShellTool,
        },
        /*network*/ None,
        /*environment_id*/ None,
    )
}

async fn run(request: crate::sandboxing::ExecRequest) -> String {
    let output = crate::sandboxing::execute_env(request, /*stdout_stream*/ None)
        .await
        .expect("unelevated sandbox run");
    format!("{}{}", output.stdout.text, output.stderr.text)
}

#[tokio::test]
#[serial_test::serial(codex_home)]
async fn sec_win_300_unelevated_tool_launch_refuses_deny_read_profiles() {
    let codex_home_dir = tempfile::tempdir().expect("codex home");
    let workspace_dir = tempfile::tempdir().expect("workspace");
    let _codex_home = EnvGuard::set("CODEX_HOME", codex_home_dir.path());
    let cwd = absolute(workspace_dir.path());
    std::fs::write(cwd.join("secret.env"), SECRET).expect("secret file");
    std::fs::write(cwd.join("public.txt"), PUBLIC).expect("public file");

    let base = PermissionProfile::workspace_write()
        .materialize_project_roots_with_workspace_roots(std::slice::from_ref(&cwd));
    // Control: the unelevated sandbox runs commands here, and without a deny
    // entry it reads both files.
    let control = run(unelevated_tool_launch(&base, &cwd).expect("control launch")).await;
    eprintln!("sec-win-300 unelevated, no deny entry: {control}");
    assert!(
        control.contains(SECRET) && control.contains(PUBLIC),
        "{control}"
    );

    let denies = [
        (
            "exact path",
            FileSystemPath::Path {
                path: cwd.join("secret.env"),
            },
        ),
        (
            "glob",
            FileSystemPath::GlobPattern {
                pattern: "**/*.env".to_string(),
            },
        ),
    ];
    for (label, path) in denies {
        let (mut file_system, _) = base.to_runtime_permissions();
        file_system.entries.push(FileSystemSandboxEntry {
            path,
            access: FileSystemAccessMode::Deny,
            missing_path_behavior: None,
        });
        let denying = PermissionProfile::from_runtime_permissions(
            &file_system,
            NetworkSandboxPolicy::Restricted,
        );
        let outcome = match unelevated_tool_launch(&denying, &cwd) {
            Err(refused) => refused.to_string(),
            Ok(request) => {
                match crate::sandboxing::execute_env(request, /*stdout_stream*/ None).await {
                    Ok(output) => format!("ran: {}{}", output.stdout.text, output.stderr.text),
                    Err(err) => format!("failed at spawn: {err}"),
                }
            }
        };
        eprintln!("sec-win-300 unelevated, {label} deny on secret.env: {outcome}");
        assert!(!outcome.contains(SECRET), "{label}: read a denied file");
        assert_eq!(
            outcome,
            format!(
                "unsupported operation: {}",
                codex_sandboxing::UNELEVATED_DENY_READ_REFUSAL
            ),
            "{label}"
        );
    }
}
