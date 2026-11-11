use super::RunnerTransportRequest;
use super::spawn_runner_transport_with_retry;
use super::start_runner_off_async_thread;
use crate::WindowsSandboxProxySettingsMode;
use crate::identity::SandboxCreds;
use crate::ipc_framed::ErrorPayload;
use crate::ipc_framed::ErrorStage;
use crate::ipc_framed::SpawnRequest;
use crate::resolved_permissions::ResolvedWindowsSandboxPermissions;
use crate::runner_client::RunnerStartupError;
use codex_protocol::models::PermissionProfile;
use codex_utils_absolute_path::AbsolutePathBuf;
use pretty_assertions::assert_eq;
use serde_json::Value;
use std::cell::Cell;
use std::cell::RefCell;
use std::collections::HashMap;
use std::path::Path;
use std::path::PathBuf;
use windows_sys::Win32::Foundation::ERROR_NO_SUCH_LOGON_SESSION;

#[derive(Debug, Clone, PartialEq, Eq)]
struct SpawnObservation {
    codex_home: PathBuf,
    cwd: PathBuf,
    username: String,
    password: String,
    logs_base_dir: Option<PathBuf>,
    spawn_request: Value,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct RefreshObservation {
    permissions: ResolvedWindowsSandboxPermissions,
    cwd: PathBuf,
    env_map: HashMap<String, String>,
    codex_home: PathBuf,
    read_roots_override: Option<Vec<PathBuf>>,
    read_roots_include_platform_defaults: bool,
    write_roots_override: Option<Vec<PathBuf>>,
    deny_read_override: crate::DenyReadTargets,
    deny_write_paths_override: Vec<PathBuf>,
    proxy_enforced: bool,
    proxy_settings_mode: WindowsSandboxProxySettingsMode,
}

#[test]
fn retry_uses_original_unified_exec_request_and_stops_after_second_failure() {
    let workspace_root = AbsolutePathBuf::from_absolute_path(PathBuf::from(r"C:\workspace"))
        .expect("absolute workspace root");
    let permission_profile = PermissionProfile::workspace_write();
    let permissions =
        ResolvedWindowsSandboxPermissions::try_from_permission_profile_for_workspace_roots(
            &permission_profile,
            std::slice::from_ref(&workspace_root),
        )
        .expect("resolved permissions");
    let env_map = HashMap::from([
        ("Path".to_string(), r"C:\tools".to_string()),
        (
            "HTTPS_PROXY".to_string(),
            "http://localhost:1234".to_string(),
        ),
    ]);
    let request = RunnerTransportRequest {
        permissions: permissions.clone(),
        codex_home: PathBuf::from(r"C:\Users\codex"),
        cwd: PathBuf::from(r"C:\workspace"),
        env_map: env_map.clone(),
        logs_base_dir: Some(PathBuf::from(r"C:\Users\codex\.sandbox")),
        spawn_request: SpawnRequest {
            command: vec!["pwsh.exe".to_string(), "-NoProfile".to_string()],
            cwd: PathBuf::from(r"C:\workspace"),
            env: env_map.clone(),
            permission_profile,
            workspace_roots: vec![workspace_root],
            codex_home: PathBuf::from(r"C:\Users\codex\.sandbox"),
            real_codex_home: PathBuf::from(r"C:\Users\codex"),
            cap_sids: vec!["S-1-15-3-1024-1".to_string()],
            network_proxy_restricting_sid: Some("S-1-5-21-100-200-300-400".to_string()),
            timeout_ms: Some(5_000),
            tty: true,
            stdin_open: true,
            use_private_desktop: true,
        },
        read_roots_override: Some(vec![PathBuf::from(r"C:\workspace\read")]),
        read_roots_include_platform_defaults: true,
        write_roots_override: Some(vec![PathBuf::from(r"C:\workspace\write")]),
        deny_read_override: crate::DenyReadTargets::from_exact_paths([
            AbsolutePathBuf::from_absolute_path(PathBuf::from(r"C:\secrets"))
                .expect("absolute deny path"),
        ]),
        deny_write_paths_override: vec![PathBuf::from(r"C:\workspace\.codex")],
        proxy_enforced: true,
        proxy_settings_mode: WindowsSandboxProxySettingsMode::Preserve,
    };
    let expected_spawn_request =
        serde_json::to_value(&request.spawn_request).expect("serialize spawn request");
    let expected_refresh = RefreshObservation {
        permissions,
        cwd: request.cwd.clone(),
        env_map,
        codex_home: request.codex_home.clone(),
        read_roots_override: request.read_roots_override.clone(),
        read_roots_include_platform_defaults: true,
        write_roots_override: request.write_roots_override.clone(),
        deny_read_override: request.deny_read_override.clone(),
        deny_write_paths_override: request.deny_write_paths_override.clone(),
        proxy_enforced: true,
        proxy_settings_mode: WindowsSandboxProxySettingsMode::Preserve,
    };
    let spawn_observations = RefCell::new(Vec::new());
    let refresh_observations = RefCell::new(Vec::new());
    let spawn_attempts = Cell::new(0);

    let result = spawn_runner_transport_with_retry(
        SandboxCreds {
            username: "stale".to_string(),
            password: "old".to_string(),
        },
        &request,
        |codex_home, cwd, sandbox_creds, logs_base_dir, spawn_request| {
            spawn_observations.borrow_mut().push(SpawnObservation {
                codex_home: codex_home.to_path_buf(),
                cwd: cwd.to_path_buf(),
                username: sandbox_creds.username.clone(),
                password: sandbox_creds.password.clone(),
                logs_base_dir: logs_base_dir.map(Path::to_path_buf),
                spawn_request: serde_json::to_value(spawn_request)
                    .expect("serialize spawn request"),
            });
            spawn_attempts.set(spawn_attempts.get() + 1);
            Err::<(), _>(anyhow::Error::new(RunnerStartupError::new(ErrorPayload {
                message: format!("spawn attempt {} failed", spawn_attempts.get()),
                stage: ErrorStage::SpawnChild,
                windows_error_code: Some(ERROR_NO_SUCH_LOGON_SESSION),
            })))
        },
        |permissions,
         cwd,
         env_map,
         codex_home,
         read_roots_override,
         read_roots_include_platform_defaults,
         write_roots_override,
         deny_read_override,
         deny_write_paths_override,
         proxy_enforced,
         proxy_settings_mode| {
            refresh_observations.borrow_mut().push(RefreshObservation {
                permissions: permissions.clone(),
                cwd: cwd.to_path_buf(),
                env_map: env_map.clone(),
                codex_home: codex_home.to_path_buf(),
                read_roots_override: read_roots_override.map(<[PathBuf]>::to_vec),
                read_roots_include_platform_defaults,
                write_roots_override: write_roots_override.map(<[PathBuf]>::to_vec),
                deny_read_override: deny_read_override.clone(),
                deny_write_paths_override: deny_write_paths_override.to_vec(),
                proxy_enforced,
                proxy_settings_mode,
            });
            Ok(SandboxCreds {
                username: "refreshed".to_string(),
                password: "new".to_string(),
            })
        },
    );

    let err = result.expect_err("second spawn should fail");
    assert_eq!(
        "runner failed during SpawnChild: spawn attempt 2 failed (Windows error 1312)",
        err.to_string()
    );
    assert_eq!(
        vec![
            SpawnObservation {
                codex_home: request.codex_home.clone(),
                cwd: request.cwd.clone(),
                username: "stale".to_string(),
                password: "old".to_string(),
                logs_base_dir: request.logs_base_dir.clone(),
                spawn_request: expected_spawn_request.clone(),
            },
            SpawnObservation {
                codex_home: request.codex_home.clone(),
                cwd: request.cwd.clone(),
                username: "refreshed".to_string(),
                password: "new".to_string(),
                logs_base_dir: request.logs_base_dir,
                spawn_request: expected_spawn_request,
            },
        ],
        *spawn_observations.borrow()
    );
    assert_eq!(vec![expected_refresh], *refresh_observations.borrow());
}

/// A request for tests that don't look at it.
fn plain_request() -> RunnerTransportRequest {
    let workspace_root = AbsolutePathBuf::from_absolute_path(PathBuf::from(r"C:\workspace"))
        .expect("absolute workspace root");
    let permission_profile = PermissionProfile::workspace_write();
    let permissions =
        ResolvedWindowsSandboxPermissions::try_from_permission_profile_for_workspace_roots(
            &permission_profile,
            std::slice::from_ref(&workspace_root),
        )
        .expect("resolved permissions");
    RunnerTransportRequest {
        permissions,
        codex_home: PathBuf::from(r"C:\Users\codex"),
        cwd: PathBuf::from(r"C:\workspace"),
        env_map: HashMap::new(),
        logs_base_dir: None,
        spawn_request: SpawnRequest {
            command: vec!["cmd.exe".to_string()],
            cwd: PathBuf::from(r"C:\workspace"),
            env: HashMap::new(),
            permission_profile,
            workspace_roots: vec![workspace_root],
            codex_home: PathBuf::from(r"C:\Users\codex\.sandbox"),
            real_codex_home: PathBuf::from(r"C:\Users\codex"),
            cap_sids: Vec::new(),
            network_proxy_restricting_sid: None,
            timeout_ms: None,
            tty: false,
            stdin_open: false,
            use_private_desktop: false,
        },
        read_roots_override: None,
        read_roots_include_platform_defaults: false,
        write_roots_override: None,
        deny_read_override: crate::DenyReadTargets::default(),
        deny_write_paths_override: Vec::new(),
        proxy_enforced: false,
        proxy_settings_mode: WindowsSandboxProxySettingsMode::Preserve,
    }
}

/// Stands in for the planted-folder watch: records the thread it ends on.
struct Watch(std::sync::Arc<std::sync::Mutex<Option<std::thread::ThreadId>>>);

impl Drop for Watch {
    fn drop(&mut self) {
        *self.0.lock().expect("lock") = Some(std::thread::current().id());
    }
}

fn runner_cannot_start(
    _codex_home: &Path,
    _cwd: &Path,
    _sandbox_creds: &SandboxCreds,
    _log_dir: Option<&Path>,
    _spawn_request: SpawnRequest,
) -> anyhow::Result<()> {
    anyhow::bail!("the runner can't start")
}

fn runner_starts(
    _codex_home: &Path,
    _cwd: &Path,
    _sandbox_creds: &SandboxCreds,
    _log_dir: Option<&Path>,
    _spawn_request: SpawnRequest,
) -> anyhow::Result<()> {
    Ok(())
}

fn creds() -> SandboxCreds {
    SandboxCreds {
        username: "sandbox".to_string(),
        password: "secret".to_string(),
    }
}

/// #1909 item 6 (#1903 review): when the elevated runner can't start, the
/// error comes back and the planted-folder watch has already ended, off the
/// async thread (ending it walks the writable roots); when it starts, the
/// watch comes back with it, still running.
#[tokio::test(flavor = "current_thread")]
async fn planted_watch_ends_off_the_async_thread_when_the_runner_cannot_start() {
    let async_thread = std::thread::current().id();
    let ended = std::sync::Arc::new(std::sync::Mutex::new(None));

    let failed = start_runner_off_async_thread(
        creds(),
        plain_request(),
        Watch(std::sync::Arc::clone(&ended)),
        runner_cannot_start,
    )
    .await
    .map(|_| ())
    .map_err(|err| err.to_string());
    let ended_on = ended.lock().expect("lock").take();

    let started = start_runner_off_async_thread(
        creds(),
        plain_request(),
        Watch(std::sync::Arc::clone(&ended)),
        runner_starts,
    )
    .await
    .expect("runner starts");
    let still_running = ended.lock().expect("lock").is_none();
    drop(started);

    assert_eq!(
        (
            failed,
            ended_on.is_some_and(|thread| thread != async_thread),
            still_running,
        ),
        (Err("the runner can't start".to_string()), true, true)
    );
}
