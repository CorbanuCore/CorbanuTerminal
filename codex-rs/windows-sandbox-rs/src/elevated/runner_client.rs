use crate::app_package::registered_core_requested;
use crate::app_package::verify_registered_core_runner;
use crate::identity::SandboxCreds;
use crate::ipc_framed::ErrorPayload;
use crate::ipc_framed::ErrorStage;
use crate::ipc_framed::FramedMessage;
use crate::ipc_framed::IPC_PROTOCOL_VERSION;
use crate::ipc_framed::Message;
use crate::ipc_framed::SpawnReady;
use crate::ipc_framed::SpawnRequest;
use crate::ipc_framed::read_frame;
use crate::ipc_framed::write_frame;
pub(crate) use crate::logon_launch::LogonError as RunnerLogonError;
use crate::logon_launch::LogonLaunchRequest;
use crate::logon_launch::WindowAccessReaper;
use crate::logon_launch::create_process_with_logon;
use crate::runner_pipe::PIPE_ACCESS_INBOUND;
use crate::runner_pipe::PIPE_ACCESS_OUTBOUND;
use crate::runner_pipe::connect_pipe;
use crate::runner_pipe::create_named_pipe;
use crate::runner_pipe::find_runner_exe;
use crate::runner_pipe::pipe_pair;
use crate::winutil::quote_windows_arg;
use anyhow::Context;
use anyhow::Result;
use std::fs::File;
use std::os::windows::io::AsRawHandle;
use std::os::windows::io::FromRawHandle;
use std::os::windows::io::IntoRawHandle;
use std::os::windows::io::OwnedHandle;
use std::path::Path;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;
use std::time::Instant;
use windows_sys::Win32::Foundation::CloseHandle;
use windows_sys::Win32::Foundation::DUPLICATE_SAME_ACCESS;
use windows_sys::Win32::Foundation::DuplicateHandle;
use windows_sys::Win32::Foundation::ERROR_ACCOUNT_DISABLED;
use windows_sys::Win32::Foundation::ERROR_LOGON_FAILURE;
use windows_sys::Win32::Foundation::ERROR_NO_SUCH_LOGON_SESSION;
use windows_sys::Win32::Foundation::ERROR_NOT_FOUND;
use windows_sys::Win32::Foundation::ERROR_SERVICE_ALREADY_RUNNING;
use windows_sys::Win32::Foundation::GetLastError;
use windows_sys::Win32::Foundation::HANDLE;
use windows_sys::Win32::System::IO::CancelSynchronousIo;
use windows_sys::Win32::System::Threading::GetCurrentProcess;
use windows_sys::Win32::System::Threading::GetCurrentThread;
use windows_sys::Win32::System::Threading::INFINITE;
use windows_sys::Win32::System::Threading::LOGON_WITH_PROFILE;
use windows_sys::Win32::System::Threading::TerminateProcess;
use windows_sys::Win32::System::Threading::WaitForSingleObject;

const RUNNER_SPAWN_READY_TIMEOUT: Duration = Duration::from_secs(15);
const RUNNER_PIPE_CONNECT_TIMEOUT: Duration = Duration::from_secs(15);
const WAIT_OBJECT_0: u32 = 0;

struct NonOwningHandle(HANDLE);

// SAFETY: Pipe and thread handles are opaque process-wide tokens that may be
// passed between threads. This wrapper does not own or extend their lifetime.
unsafe impl Send for NonOwningHandle {}

impl NonOwningHandle {
    fn raw(&self) -> HANDLE {
        self.0
    }
}

#[derive(Debug)]
pub(crate) struct RunnerStartupError {
    payload: ErrorPayload,
}

impl RunnerStartupError {
    pub(crate) fn new(payload: ErrorPayload) -> Self {
        Self { payload }
    }
}

impl std::fmt::Display for RunnerStartupError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "runner failed during {:?}: {}",
            self.payload.stage, self.payload.message
        )?;
        if let Some(code) = self.payload.windows_error_code {
            write!(f, " (Windows error {code})")?;
        }
        Ok(())
    }
}

impl std::error::Error for RunnerStartupError {}

pub(crate) struct RunnerTransport {
    pipe_write: File,
    pipe_read: File,
}

fn is_refreshable_windows_error(code: u32) -> bool {
    matches!(
        code,
        ERROR_ACCOUNT_DISABLED | ERROR_LOGON_FAILURE | ERROR_NO_SUCH_LOGON_SESSION
    )
}

fn command_targets_windows_apps(command: &[String]) -> bool {
    command.first().is_some_and(|program| {
        Path::new(program).components().any(|component| {
            component
                .as_os_str()
                .to_string_lossy()
                .eq_ignore_ascii_case("WindowsApps")
        })
    })
}

pub(crate) fn is_refreshable_sandbox_creds_error(err: &anyhow::Error, command: &[String]) -> bool {
    if err
        .downcast_ref::<RunnerLogonError>()
        .is_some_and(|err| is_refreshable_windows_error(err.code))
    {
        return true;
    }

    err.downcast_ref::<RunnerStartupError>().is_some_and(|err| {
        err.payload.stage == ErrorStage::SpawnChild
            && err.payload.windows_error_code.is_some_and(|code| {
                // AppX activation can return 1312 for a healthy sandbox token. Rotating the
                // account password cannot make the same WindowsApps command launch.
                is_refreshable_windows_error(code)
                    && (code != ERROR_NO_SUCH_LOGON_SESSION
                        || !command_targets_windows_apps(command))
            })
    })
}

pub(crate) fn retry_runner_spawn_once<T>(
    sandbox_creds: SandboxCreds,
    command: &[String],
    mut spawn: impl FnMut(SandboxCreds) -> Result<T>,
    refresh: impl FnOnce() -> Result<SandboxCreds>,
) -> Result<T> {
    let result = match spawn(sandbox_creds.clone()) {
        Ok(result) => Ok(result),
        // No runner (and therefore no user command) started. Keep credentials unchanged.
        Err(err)
            if err
                .downcast_ref::<RunnerLogonError>()
                .is_some_and(|err| err.code == ERROR_SERVICE_ALREADY_RUNNING) =>
        {
            spawn(sandbox_creds)
        }
        Err(err) if is_refreshable_sandbox_creds_error(&err, command) => refresh().and_then(spawn),
        Err(err) => Err(err),
    };
    super::runner_metrics::record("startup", if result.is_ok() { "success" } else { "error" });
    result
}

impl RunnerTransport {
    pub(crate) fn send_spawn_request(&mut self, request: SpawnRequest) -> Result<()> {
        let spawn_request = FramedMessage {
            version: IPC_PROTOCOL_VERSION,
            message: Message::SpawnRequest {
                payload: Box::new(request),
            },
        };
        write_frame(&mut self.pipe_write, &spawn_request)
    }

    pub(crate) fn read_spawn_ready(&mut self) -> Result<SpawnReady> {
        crate::framed_io::wait_for_complete_frame(
            &self.pipe_read,
            Instant::now() + RUNNER_SPAWN_READY_TIMEOUT,
        )
        .context("wait for runner spawn_ready")?;
        let msg = read_frame(&mut self.pipe_read)?
            .ok_or_else(|| anyhow::anyhow!("runner pipe closed before spawn_ready"))?;
        match msg.message {
            Message::SpawnReady { payload } => Ok(payload),
            Message::Error { payload } => Err(RunnerStartupError::new(payload).into()),
            other => Err(anyhow::anyhow!(
                "expected spawn_ready from runner, got {other:?}"
            )),
        }
    }

    pub(crate) fn into_files(self) -> (File, File) {
        (self.pipe_write, self.pipe_read)
    }
}

fn try_take_completed_connect_result(
    connect_thread: &mut Option<thread::JoinHandle<()>>,
    connect_result_rx: &mpsc::Receiver<Result<()>>,
    thread_handle: HANDLE,
    pipe_label: &str,
) -> Result<Option<Result<()>>> {
    let thread_wait = unsafe { WaitForSingleObject(thread_handle, 0) };
    if thread_wait != WAIT_OBJECT_0 {
        return Ok(None);
    }

    if let Some(connect_thread) = connect_thread.take() {
        let _ = connect_thread.join();
    }

    let result = connect_result_rx.recv().map_err(|_| {
        anyhow::anyhow!("runner {pipe_label} connect thread exited before reporting its result")
    })?;
    Ok(Some(result))
}

fn connect_pipe_with_timeout(
    h_pipe: HANDLE,
    expected_runner_pid: u32,
    pipe_label: &str,
) -> Result<()> {
    let pipe_label = pipe_label.to_string();
    let pipe_label_for_thread = pipe_label.clone();
    let (thread_handle_tx, thread_handle_rx) = mpsc::sync_channel(1);
    let (connect_result_tx, connect_result_rx) = mpsc::sync_channel(1);
    let h_pipe = NonOwningHandle(h_pipe);
    let mut connect_thread = Some(
        thread::Builder::new()
            .name(format!("codex-runner-connect-{pipe_label}"))
            .spawn(move || {
                let current_process = unsafe { GetCurrentProcess() };
                let mut thread_handle = std::ptr::null_mut();
                let duplicate_ok = unsafe {
                    DuplicateHandle(
                        current_process,
                        GetCurrentThread(),
                        current_process,
                        &mut thread_handle,
                        0,
                        0,
                        DUPLICATE_SAME_ACCESS,
                    )
                };
                if duplicate_ok == 0 {
                    let _ = thread_handle_tx.send(Err(anyhow::anyhow!(
                        "DuplicateHandle failed for runner {pipe_label_for_thread} connect thread: {}",
                        unsafe { GetLastError() }
                    )));
                    return;
                }

                // Publish the helper thread HANDLE before the blocking pipe connect so the
                // parent can cancel this specific operation if it times out.
                let _ = thread_handle_tx.send(Ok(NonOwningHandle(thread_handle)));

                let result = connect_pipe(h_pipe.raw(), expected_runner_pid)
                    .map_err(anyhow::Error::from)
                    .context(format!("connect {pipe_label_for_thread}"));
                let _ = connect_result_tx.send(result);
            })?,
    );
    let thread_handle = thread_handle_rx.recv().map_err(|_| {
        anyhow::anyhow!("runner {pipe_label} connect thread exited before publishing its handle")
    })??;

    let result = match connect_result_rx.recv_timeout(RUNNER_PIPE_CONNECT_TIMEOUT) {
        Ok(result) => {
            if let Some(connect_thread) = connect_thread.take() {
                let _ = connect_thread.join();
            }
            result
        }
        Err(mpsc::RecvTimeoutError::Timeout) => {
            if let Some(result) = try_take_completed_connect_result(
                &mut connect_thread,
                &connect_result_rx,
                thread_handle.raw(),
                &pipe_label,
            )? {
                result
            } else {
                let cancel_ok = unsafe { CancelSynchronousIo(thread_handle.raw()) };
                if cancel_ok == 0 {
                    let err = unsafe { GetLastError() };
                    if err != ERROR_NOT_FOUND {
                        Err(anyhow::anyhow!(
                            "CancelSynchronousIo failed for runner {pipe_label} connect thread: {err}"
                        ))
                    } else if let Some(result) = try_take_completed_connect_result(
                        &mut connect_thread,
                        &connect_result_rx,
                        thread_handle.raw(),
                        &pipe_label,
                    )? {
                        result
                    } else {
                        Err(anyhow::anyhow!(
                            "timed out after {}ms connecting runner {pipe_label}",
                            RUNNER_PIPE_CONNECT_TIMEOUT.as_millis()
                        ))
                    }
                } else {
                    // Do not join the helper thread on the timeout path. Parent-side cleanup will
                    // close the pipe handles, which lets the blocked connect unwind without
                    // risking another indefinite wait here.
                    Err(anyhow::anyhow!(
                        "timed out after {}ms connecting runner {pipe_label}",
                        RUNNER_PIPE_CONNECT_TIMEOUT.as_millis()
                    ))
                }
            }
        }
        Err(mpsc::RecvTimeoutError::Disconnected) => {
            if let Some(connect_thread) = connect_thread.take() {
                let _ = connect_thread.join();
            }
            Err(anyhow::anyhow!(
                "runner {pipe_label} connect thread exited before reporting its result"
            ))
        }
    };

    unsafe {
        CloseHandle(thread_handle.raw());
    }

    result
}

pub(crate) fn spawn_runner_transport(
    codex_home: &Path,
    cwd: &Path,
    sandbox_creds: &SandboxCreds,
    log_dir: Option<&Path>,
    spawn_request: SpawnRequest,
) -> Result<RunnerTransport> {
    let runner_exe = find_runner_exe(codex_home, log_dir)?;
    let registered_alias = if registered_core_requested() {
        Some(crate::app_package::registered_runner_alias(
            codex_home,
            &sandbox_creds.username,
        )?)
    } else {
        None
    };
    // Upstream (rust-v0.167.0) has Core create a private desktop shared by
    // every command of the sandbox account with the same policy, open to that
    // account. Corbanu keeps main's per-command desktop: the runner creates
    // it, open only to its own logon (#345, and D324's per-command identity),
    // so a command cannot reach another command's windows. A caller that
    // names a desktop (the `codex sandbox` wrapper) still gets that one.
    let (pipe_in_name, pipe_out_name) = pipe_pair();
    let pipe_write = unsafe {
        File::from_raw_handle(create_named_pipe(
            &pipe_in_name,
            PIPE_ACCESS_OUTBOUND,
            &sandbox_creds.username,
        )? as _)
    };
    let pipe_read = unsafe {
        File::from_raw_handle(create_named_pipe(
            &pipe_out_name,
            PIPE_ACCESS_INBOUND,
            &sandbox_creds.username,
        )? as _)
    };
    let h_pipe_in = pipe_write.as_raw_handle() as HANDLE;
    let h_pipe_out = pipe_read.as_raw_handle() as HANDLE;

    let runner_cmdline = registered_alias
        .as_deref()
        .unwrap_or(&runner_exe)
        .to_str()
        .map(str::to_owned)
        .unwrap_or_else(|| "codex-command-runner.exe".to_string());
    let runner_full_cmd = format!(
        "{} {} {}",
        quote_windows_arg(&runner_cmdline),
        quote_windows_arg(&format!("--pipe-in={pipe_in_name}")),
        quote_windows_arg(&format!("--pipe-out={pipe_out_name}"))
    );
    // #295: through the logon launcher when this process is protected; #341:
    // off the interactive window station the runner's logon gets this
    // session's window station and desktop. Execution aliases require the
    // registered account's profile; other launches stay profile-free.
    let mut launched = create_process_with_logon(
        &LogonLaunchRequest {
            username: &sandbox_creds.username,
            password: &sandbox_creds.password,
            application: Path::new(&runner_cmdline),
            command_line: &runner_full_cmd,
            cwd,
            logon_flags: if registered_alias.is_some() {
                LOGON_WITH_PROFILE
            } else {
                0
            },
        },
        &runner_exe,
    )?;
    if launched.via_launcher {
        crate::logging::log_note(
            "runner started through the logon launcher (protected process, #295)",
            log_dir,
        );
    }
    if let Some(err) = &launched.window_access_error {
        crate::logging::log_note(
            &format!("runner may not start: could not give its logon this window station: {err}"),
            log_dir,
        );
    }
    // SAFETY: create_process_with_logon transfers the process handle to the caller.
    let runner_process = unsafe { OwnedHandle::from_raw_handle(launched.process as _) };
    let expected_runner_pid = launched.pid;
    // #345: removed when the runner exits, or on the failure paths below once
    // it has been ended.
    let window_access = launched.window_access.take().map(|mut access| {
        access.set_log_dir(log_dir);
        access
    });
    let mut transport = RunnerTransport {
        pipe_write,
        pipe_read,
    };
    let startup_result = (|| -> Result<SpawnReady> {
        // An update can retarget the alias after setup. Check the selected image
        // and authenticate its pipes before sending it the command.
        if registered_alias.is_some() {
            verify_registered_core_runner(runner_process.as_raw_handle() as HANDLE, &runner_exe)?;
        }
        connect_pipe_with_timeout(h_pipe_in, expected_runner_pid, "pipe-in")
            .map_err(|err| runner_connect_error(err, launched.window_access_error.as_deref()))?;
        connect_pipe_with_timeout(h_pipe_out, expected_runner_pid, "pipe-out")
            .map_err(|err| runner_connect_error(err, launched.window_access_error.as_deref()))?;
        transport.send_spawn_request(spawn_request)?;
        transport.read_spawn_ready()
    })();
    let spawn_ready = match startup_result {
        Ok(spawn_ready) => spawn_ready,
        Err(err) => {
            // Keep the process handle alive until the handshake finishes, so a
            // failure can still stop the runner instead of leaving it behind.
            unsafe {
                let _ = TerminateProcess(runner_process.as_raw_handle() as HANDLE, 1);
            }
            return Err(err);
        }
    };

    if registered_alias.is_some() {
        crate::logging::debug_log(
            &format!(
                "registered_core_runner_ready pid={expected_runner_pid} package_verified=true pipes_authenticated=true"
            ),
            log_dir,
        );
    }

    let Some(window_access) = window_access else {
        // Startup is complete: the transport pipes are the session's only anchor.
        return Ok(transport);
    };
    // #345: the runner has started its command on a private desktop; leave the
    // command only what it and its children need to start, and remove that once
    // the runner exits. A ConPTY's console host starts on Core's desktop: keep
    // that entry if the runner could not tell it had started.
    let commands_use_this_desktop = spawn_ready.console_host_started == Some(false);
    let runner_process = runner_process.into_raw_handle() as HANDLE;
    let mut window_access = match WindowAccessReaper::start(
        window_access,
        runner_process,
        &runner_exe,
    ) {
        Ok(mut reaper) => {
            // The reaper removes it when the runner exits, even after Core.
            if let Err(err) = reaper.narrow_for_commands(commands_use_this_desktop) {
                crate::logging::log_note(&format!("{err:#}"), log_dir);
            }
            unsafe { CloseHandle(runner_process) };
            return Ok(transport);
        }
        Err((window_access, err)) => {
            crate::logging::log_note(
                &format!(
                    "no reaper for the runner's window station access ({err:#}); Core removes it, unless it exits first"
                ),
                log_dir,
            );
            window_access
        }
    };
    if let Err(err) = window_access.narrow_for_commands(commands_use_this_desktop) {
        crate::logging::log_note(
            &format!("could not narrow the runner's window station access: {err:#}"),
            log_dir,
        );
    }
    let runner_raw = runner_process;
    let runner_process = NonOwningHandle(runner_process);
    let watch = thread::Builder::new()
        .name("codex-runner-window-access".to_string())
        .spawn(move || {
            let runner_process = runner_process;
            // SAFETY: the runner's process handle, owned by this thread from here on.
            unsafe {
                WaitForSingleObject(runner_process.raw(), INFINITE);
                CloseHandle(runner_process.raw());
            }
            drop(window_access);
        });
    if let Err(err) = watch {
        // The closure, and with it the access, is dropped with the error: the access goes now
        // rather than with the runner (its command has started already).
        crate::logging::log_note(
            &format!("could not watch the runner to remove its window station access: {err}"),
            log_dir,
        );
        // SAFETY: the thread never started, so the handle is still ours.
        unsafe { CloseHandle(runner_raw) };
    }

    Ok(transport)
}

/// #341: the likely reason a runner never connected.
fn runner_connect_error(err: anyhow::Error, window_access_error: Option<&str>) -> anyhow::Error {
    match window_access_error {
        Some(access) => anyhow::anyhow!(
            "{err:#} (could not give the runner's logon access to this session's window station: {access})"
        ),
        None => err,
    }
}

#[cfg(test)]
mod tests {
    use super::RunnerLogonError;
    use super::RunnerStartupError;
    use super::is_refreshable_sandbox_creds_error;
    use super::retry_runner_spawn_once;
    use crate::identity::SandboxCreds;
    use crate::ipc_framed::ErrorPayload;
    use crate::ipc_framed::ErrorStage;
    use pretty_assertions::assert_eq;
    use windows_sys::Win32::Foundation::ERROR_ACCOUNT_DISABLED;
    use windows_sys::Win32::Foundation::ERROR_LOGON_FAILURE;
    use windows_sys::Win32::Foundation::ERROR_NO_SUCH_LOGON_SESSION;
    use windows_sys::Win32::Foundation::ERROR_NOT_FOUND;
    use windows_sys::Win32::Foundation::ERROR_SERVICE_ALREADY_RUNNING;

    #[test]
    fn logon_service_already_running_retries_once_without_refresh() {
        let mut attempts = 0;
        let result = retry_runner_spawn_once(
            SandboxCreds {
                username: "sandbox-user".into(),
                password: "test-only".into(),
            },
            &[],
            |creds| {
                attempts += 1;
                assert_eq!(
                    (creds.username.as_str(), creds.password.as_str()),
                    ("sandbox-user", "test-only")
                );
                if attempts == 1 {
                    Err(anyhow::Error::new(RunnerLogonError {
                        code: ERROR_SERVICE_ALREADY_RUNNING,
                    })
                    .context("runner launch failed"))
                } else {
                    Ok(42)
                }
            },
            || panic!("1056 must not refresh credentials"),
        );
        assert_eq!((result.unwrap(), attempts), (42, 2));
    }

    #[test]
    fn logon_service_already_running_stops_after_two_failures() {
        let mut attempts = 0;
        let err = retry_runner_spawn_once::<()>(
            SandboxCreds {
                username: "sandbox-user".into(),
                password: "test-only".into(),
            },
            &[],
            |_| {
                attempts += 1;
                Err(RunnerLogonError {
                    code: ERROR_SERVICE_ALREADY_RUNNING,
                }
                .into())
            },
            || panic!("1056 must not refresh credentials"),
        )
        .unwrap_err();
        assert_eq!(
            (
                attempts,
                err.downcast_ref::<RunnerLogonError>().unwrap().code
            ),
            (2, ERROR_SERVICE_ALREADY_RUNNING)
        );
    }

    #[test]
    fn child_service_already_running_does_not_replay_command() {
        let mut attempts = 0;
        let err = retry_runner_spawn_once::<()>(
            SandboxCreds {
                username: "sandbox-user".into(),
                password: "test-only".into(),
            },
            &[],
            |_| {
                attempts += 1;
                Err(RunnerStartupError::new(ErrorPayload {
                    message: "child startup failed".into(),
                    stage: ErrorStage::SpawnChild,
                    windows_error_code: Some(ERROR_SERVICE_ALREADY_RUNNING),
                })
                .into())
            },
            || panic!("1056 must not refresh credentials"),
        )
        .unwrap_err();
        assert_eq!(
            (
                attempts,
                err.downcast_ref::<RunnerStartupError>()
                    .unwrap()
                    .payload
                    .windows_error_code,
            ),
            (1, Some(ERROR_SERVICE_ALREADY_RUNNING))
        );
    }

    #[test]
    fn refreshable_sandbox_creds_error_recognizes_credential_and_child_start_failures() {
        assert_eq!(
            [
                ERROR_ACCOUNT_DISABLED,
                ERROR_LOGON_FAILURE,
                ERROR_NO_SUCH_LOGON_SESSION,
                ERROR_NOT_FOUND,
            ]
            .map(|code| {
                let err =
                    anyhow::Error::new(RunnerLogonError { code }).context("runner launch failed");
                is_refreshable_sandbox_creds_error(&err, &[])
            }),
            [true, true, true, false]
        );

        assert_eq!(
            [
                (ErrorStage::SpawnChild, ERROR_NO_SUCH_LOGON_SESSION),
                (ErrorStage::SpawnChild, ERROR_NOT_FOUND),
                (ErrorStage::ReadSpawnRequest, ERROR_NO_SUCH_LOGON_SESSION),
            ]
            .map(|(stage, windows_error_code)| {
                let err = anyhow::Error::new(RunnerStartupError::new(ErrorPayload {
                    message: "runner startup failed".to_string(),
                    stage,
                    windows_error_code: Some(windows_error_code),
                }));
                is_refreshable_sandbox_creds_error(&err, &["cmd.exe".to_string()])
            }),
            [true, false, false]
        );

        let windows_apps_commands = [
            vec![
                r"C:\Users\user\AppData\Local\Microsoft\WindowsApps\pwsh.exe".to_string(),
            ],
            vec![
                r"C:\Program Files\WindowsApps\Microsoft.PowerShell_7.6.3.0_x64__8wekyb3d8bbwe\pwsh.exe"
                    .to_string(),
            ],
        ];
        assert_eq!(
            windows_apps_commands.map(|command| {
                let err = anyhow::Error::new(RunnerStartupError::new(ErrorPayload {
                    message: "runner startup failed".to_string(),
                    stage: ErrorStage::SpawnChild,
                    windows_error_code: Some(ERROR_NO_SUCH_LOGON_SESSION),
                }));
                is_refreshable_sandbox_creds_error(&err, &command)
            }),
            [false, false]
        );
    }
}
