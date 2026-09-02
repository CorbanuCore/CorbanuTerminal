use std::fs::OpenOptions;
use std::path::Path;
use std::path::PathBuf;
use std::process::Stdio;
use std::thread;
use std::time::Duration;

use codex_vault::AddCredential;
use codex_vault::CredentialType;
use codex_vault::Vault;
use serde::Deserialize;
use serde::Serialize;

use super::TelegramChatCandidate;

pub(super) const TOKEN_LABEL: &str = "telegram/bot_token";

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct TelegramConnectionDefaults {
    pub(crate) model: Option<String>,
    pub(crate) cwd: PathBuf,
    pub(crate) approval_policy: String,
    pub(crate) sandbox_mode: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct TelegramStatus {
    pub(crate) configured: bool,
    pub(crate) token_stored: bool,
    pub(crate) running: bool,
    pub(crate) pid: Option<u32>,
    pub(crate) bot_username: Option<String>,
    pub(crate) allowed_chat_ids: Vec<i64>,
    pub(crate) default_model: Option<String>,
    pub(crate) default_cwd: Option<PathBuf>,
    pub(crate) approval_policy: Option<String>,
    pub(crate) sandbox_mode: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ConnectorRuntime {
    pid: u32,
    started_at: i64,
    executable: PathBuf,
}

const OPERATION_LOCK_RETRIES: usize = 50;
const OPERATION_LOCK_RETRY_DELAY: Duration = Duration::from_millis(50);
const CONNECTOR_LOG_MAX_BYTES: u64 = 10 * 1024 * 1024;

pub(crate) fn connect_chat(
    codex_home: &Path,
    candidate: TelegramChatCandidate,
    defaults: TelegramConnectionDefaults,
) -> Result<String, String> {
    with_operation_lock(codex_home, || {
        write_telegram_config(codex_home, Some((&candidate, &defaults)))?;
        start_connector_unlocked(codex_home)?;
        Ok(format!(
            "Telegram connected to {}. The connector is running in the background.",
            candidate.display_name
        ))
    })
}

pub(crate) fn start_connector(codex_home: &Path) -> Result<String, String> {
    with_operation_lock(codex_home, || start_connector_unlocked(codex_home))
}

pub(crate) fn ensure_connector(codex_home: &Path) -> Result<(), String> {
    with_operation_lock(codex_home, || {
        let config = read_config(codex_home)?;
        let configured = config
            .get("telegram")
            .and_then(toml::Value::as_table)
            .and_then(|table| table.get("enabled"))
            .and_then(toml::Value::as_bool)
            .unwrap_or(false);
        // Startup recovery is irrelevant until Telegram is explicitly enabled.
        // Avoid opening the shared encrypted vault here: an OS-keyring delay in
        // an unconfigured feature must not block provider authentication.
        if !configured {
            return Ok(());
        }
        let status = read_status(codex_home)?;
        if !status.token_stored || status.running {
            return Ok(());
        }
        start_connector_unlocked(codex_home).map(|_| ())
    })
}

fn start_connector_unlocked(codex_home: &Path) -> Result<String, String> {
    if let Some(runtime) = read_runtime(codex_home) {
        if process_matches_connector(&runtime) {
            stop_runtime(codex_home, &runtime)?;
        } else {
            remove_runtime(codex_home)?;
        }
    }
    let executable = std::env::current_exe()
        .map_err(|_| "Could not locate the running Corbanu Terminal executable.".to_string())?;
    run_health_check(&executable, codex_home)?;
    let telegram_dir = codex_home.join("telegram");
    std::fs::create_dir_all(&telegram_dir)
        .map_err(|_| "Could not create the Telegram runtime directory.".to_string())?;
    let log_path = telegram_dir.join("connector.log");
    rotate_connector_log(&log_path)?;
    let stdout = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_path)
        .map_err(|_| "Could not open the Telegram connector log.".to_string())?;
    let stderr = stdout
        .try_clone()
        .map_err(|_| "Could not prepare the Telegram connector log.".to_string())?;
    let mut command = std::process::Command::new(&executable);
    command
        .arg("telegram")
        .env("CODEX_HOME", codex_home)
        .env_remove("PFTERMINAL_TELEGRAM_TOKEN")
        .stdin(Stdio::null())
        .stdout(Stdio::from(stdout))
        .stderr(Stdio::from(stderr));
    detach_command(&mut command);
    let mut child = command
        .spawn()
        .map_err(|_| "Could not start the Telegram connector process.".to_string())?;
    std::thread::sleep(Duration::from_millis(500));
    if child
        .try_wait()
        .map_err(|_| "Could not inspect the Telegram connector process.".to_string())?
        .is_some()
    {
        return Err(format!(
            "Telegram connector exited during startup. Review {}.",
            log_path.display()
        ));
    }
    let runtime = ConnectorRuntime {
        pid: child.id(),
        started_at: chrono::Utc::now().timestamp(),
        executable,
    };
    persist_runtime(codex_home, &runtime)?;
    Ok(format!("Telegram connector started (PID {}).", runtime.pid))
}

pub(crate) fn stop_connector(codex_home: &Path) -> Result<String, String> {
    with_operation_lock(codex_home, || stop_connector_unlocked(codex_home))
}

fn stop_connector_unlocked(codex_home: &Path) -> Result<String, String> {
    let Some(runtime) = read_runtime(codex_home) else {
        return Ok("Telegram connector is already stopped.".to_string());
    };
    if process_matches_connector(&runtime) {
        stop_runtime(codex_home, &runtime)?;
    } else {
        remove_runtime(codex_home)?;
    }
    Ok("Telegram connector stopped.".to_string())
}

pub(crate) fn disconnect(codex_home: &Path) -> Result<String, String> {
    with_operation_lock(codex_home, || {
        let _ = stop_connector_unlocked(codex_home)?;
        let vault = Vault::new(codex_home.to_path_buf());
        if vault.exists(TOKEN_LABEL).unwrap_or(false) {
            vault
                .delete(TOKEN_LABEL)
                .map_err(|_| "Could not delete the Telegram token from the vault.".to_string())?;
        }
        write_telegram_config(codex_home, /*connection*/ None)?;
        Ok("Telegram disconnected. Its token and local authorization were removed.".to_string())
    })
}

fn with_operation_lock<T>(
    codex_home: &Path,
    operation: impl FnOnce() -> Result<T, String>,
) -> Result<T, String> {
    let telegram_dir = codex_home.join("telegram");
    std::fs::create_dir_all(&telegram_dir)
        .map_err(|_| "Could not create the Telegram runtime directory.".to_string())?;
    let lock = OpenOptions::new()
        .create(true)
        .truncate(false)
        .read(true)
        .write(true)
        .open(telegram_dir.join("connector-operation.lock"))
        .map_err(|_| "Could not open the Telegram operation lock.".to_string())?;
    for _ in 0..OPERATION_LOCK_RETRIES {
        match lock.try_lock() {
            Ok(()) => return operation(),
            Err(std::fs::TryLockError::WouldBlock) => thread::sleep(OPERATION_LOCK_RETRY_DELAY),
            Err(_) => return Err("Could not lock Telegram connector operations.".to_string()),
        }
    }
    Err("Another Corbanu Terminal process is changing Telegram. Retry in a moment.".to_string())
}

fn run_health_check(executable: &Path, codex_home: &Path) -> Result<(), String> {
    let output = std::process::Command::new(executable)
        .args(["telegram", "--health"])
        .env("CODEX_HOME", codex_home)
        .env_remove("PFTERMINAL_TELEGRAM_TOKEN")
        .stdin(Stdio::null())
        .output()
        .map_err(|_| "Could not run the Telegram readiness check.".to_string())?;
    if output.status.success() {
        return Ok(());
    }
    let stderr = String::from_utf8_lossy(&output.stderr);
    let detail = stderr
        .lines()
        .rev()
        .find(|line| !line.trim().is_empty())
        .map(str::trim)
        .filter(|line| line.len() <= 500)
        .unwrap_or("the connector is not ready");
    Err(format!("Telegram readiness check failed: {detail}"))
}

fn rotate_connector_log(log_path: &Path) -> Result<(), String> {
    let Ok(metadata) = std::fs::metadata(log_path) else {
        return Ok(());
    };
    if metadata.len() <= CONNECTOR_LOG_MAX_BYTES {
        return Ok(());
    }
    let previous = log_path.with_extension("log.previous");
    let _ = std::fs::remove_file(&previous);
    std::fs::rename(log_path, previous)
        .map_err(|_| "Could not rotate the Telegram connector log.".to_string())
}

pub(super) fn read_status(codex_home: &Path) -> Result<TelegramStatus, String> {
    let config = read_config(codex_home)?;
    let telegram = config.get("telegram").and_then(toml::Value::as_table);
    let configured = telegram
        .and_then(|table| table.get("enabled"))
        .and_then(toml::Value::as_bool)
        .unwrap_or(false);
    let token_stored = Vault::new(codex_home.to_path_buf())
        .exists(TOKEN_LABEL)
        .unwrap_or(false);
    let runtime = read_runtime(codex_home);
    let running = runtime.as_ref().is_some_and(process_matches_connector);
    Ok(TelegramStatus {
        configured,
        token_stored,
        running,
        pid: running
            .then(|| runtime.as_ref().map(|entry| entry.pid))
            .flatten(),
        bot_username: None,
        allowed_chat_ids: telegram
            .and_then(|table| table.get("allowed_chat_ids"))
            .and_then(toml::Value::as_array)
            .map(|values| values.iter().filter_map(toml::Value::as_integer).collect())
            .unwrap_or_default(),
        default_model: telegram
            .and_then(|table| table.get("default_model"))
            .and_then(toml::Value::as_str)
            .map(str::to_string),
        default_cwd: telegram
            .and_then(|table| table.get("default_cwd"))
            .and_then(toml::Value::as_str)
            .map(PathBuf::from),
        approval_policy: telegram
            .and_then(|table| table.get("approval_policy"))
            .and_then(toml::Value::as_str)
            .map(str::to_string),
        sandbox_mode: telegram
            .and_then(|table| table.get("sandbox_mode"))
            .and_then(toml::Value::as_str)
            .map(str::to_string),
    })
}

pub(super) fn reveal_token(codex_home: PathBuf) -> Result<String, String> {
    Vault::new(codex_home)
        .reveal(TOKEN_LABEL)
        .map_err(|_| "Telegram bot token is unavailable; reconnect the bot.".to_string())
}

pub(super) fn store_token(codex_home: &Path, token: String) -> Result<(), String> {
    let vault = Vault::new(codex_home.to_path_buf());
    if vault.exists(TOKEN_LABEL).unwrap_or(false) {
        vault
            .update(
                TOKEN_LABEL,
                Some(token),
                /*provider*/ None,
                /*notes*/ None,
                /*revocation_notes*/ None,
            )
            .map_err(|_| "Could not update the Telegram token in the vault.".to_string())?;
    } else {
        vault
            .add(AddCredential {
                label: TOKEN_LABEL.to_string(),
                credential_type: CredentialType::BearerToken,
                provider: Some("telegram".to_string()),
                notes: Some("Corbanu Terminal Telegram bot token".to_string()),
                revocation_notes: Some("Revoke or rotate this token with @BotFather.".to_string()),
                secret: token,
            })
            .map_err(|_| "Could not store the Telegram token in the vault.".to_string())?;
    }
    Ok(())
}

fn write_telegram_config(
    codex_home: &Path,
    connection: Option<(&TelegramChatCandidate, &TelegramConnectionDefaults)>,
) -> Result<(), String> {
    let mut config = read_config(codex_home)?;
    let table = config
        .as_table_mut()
        .ok_or_else(|| "Corbanu Terminal config root is not a table.".to_string())?;
    match connection {
        Some((candidate, defaults)) => {
            let mut telegram = table
                .remove("telegram")
                .and_then(|value| value.as_table().cloned())
                .unwrap_or_default();
            telegram.insert("enabled".to_string(), toml::Value::Boolean(true));
            telegram.insert(
                "bot_token_env".to_string(),
                toml::Value::String("PFTERMINAL_TELEGRAM_TOKEN".to_string()),
            );
            telegram.insert(
                "allowed_chat_ids".to_string(),
                toml::Value::Array(vec![toml::Value::Integer(candidate.chat_id)]),
            );
            telegram.insert(
                "allowed_user_ids".to_string(),
                toml::Value::Array(vec![toml::Value::Integer(candidate.actor_user_id as i64)]),
            );
            telegram.insert(
                "mode".to_string(),
                toml::Value::String("polling".to_string()),
            );
            telegram.insert(
                "default_cwd".to_string(),
                toml::Value::String(defaults.cwd.display().to_string()),
            );
            telegram.insert(
                "approval_policy".to_string(),
                toml::Value::String(defaults.approval_policy.clone()),
            );
            telegram.insert(
                "sandbox_mode".to_string(),
                toml::Value::String(defaults.sandbox_mode.clone()),
            );
            if let Some(model) = &defaults.model {
                telegram.insert(
                    "default_model".to_string(),
                    toml::Value::String(model.clone()),
                );
            }
            table.insert("telegram".to_string(), toml::Value::Table(telegram));
        }
        None => {
            table.remove("telegram");
        }
    }
    let serialized = toml::to_string_pretty(&config)
        .map_err(|_| "Could not serialize Corbanu Terminal configuration.".to_string())?;
    std::fs::create_dir_all(codex_home)
        .map_err(|_| "Could not create the Corbanu Terminal home.".to_string())?;
    codex_utils_path::write_atomically(&codex_home.join("config.toml"), &serialized)
        .map_err(|_| "Could not persist Telegram configuration.".to_string())
}

fn read_config(codex_home: &Path) -> Result<toml::Value, String> {
    let path = codex_home.join("config.toml");
    if !path.exists() {
        return Ok(toml::Value::Table(Default::default()));
    }
    let contents = std::fs::read_to_string(&path)
        .map_err(|_| "Could not read Corbanu Terminal configuration.".to_string())?;
    toml::from_str(&contents)
        .map_err(|_| "Corbanu Terminal configuration is invalid TOML.".to_string())
}

fn runtime_path(codex_home: &Path) -> PathBuf {
    codex_home.join("telegram").join("connector-runtime.json")
}

fn read_runtime(codex_home: &Path) -> Option<ConnectorRuntime> {
    serde_json::from_slice(&std::fs::read(runtime_path(codex_home)).ok()?).ok()
}

fn persist_runtime(codex_home: &Path, runtime: &ConnectorRuntime) -> Result<(), String> {
    let contents = serde_json::to_string(runtime)
        .map_err(|_| "Could not serialize Telegram runtime state.".to_string())?;
    codex_utils_path::write_atomically(&runtime_path(codex_home), &contents)
        .map_err(|_| "Could not persist Telegram runtime state.".to_string())
}

fn remove_runtime(codex_home: &Path) -> Result<(), String> {
    match std::fs::remove_file(runtime_path(codex_home)) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(_) => Err("Could not remove stale Telegram runtime state.".to_string()),
    }
}

fn process_matches_connector(runtime: &ConnectorRuntime) -> bool {
    process_is_running(runtime.pid) && process_command_matches(runtime)
}

#[cfg(unix)]
fn process_is_running(pid: u32) -> bool {
    let result = unsafe { libc::kill(pid as libc::pid_t, 0) };
    result == 0 || std::io::Error::last_os_error().raw_os_error() == Some(libc::EPERM)
}

#[cfg(windows)]
fn process_is_running(pid: u32) -> bool {
    use windows_sys::Win32::Foundation::CloseHandle;
    use windows_sys::Win32::System::Threading::OpenProcess;
    use windows_sys::Win32::System::Threading::PROCESS_QUERY_LIMITED_INFORMATION;
    let handle = unsafe { OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, 0, pid) };
    if handle == 0 {
        return false;
    }
    unsafe { CloseHandle(handle) };
    true
}

#[cfg(target_os = "linux")]
fn process_command_matches(runtime: &ConnectorRuntime) -> bool {
    let command = std::fs::read(format!("/proc/{}/cmdline", runtime.pid)).unwrap_or_default();
    let executable = runtime.executable.as_os_str().as_encoded_bytes();
    command
        .windows(executable.len())
        .any(|part| part == executable)
        && command
            .windows(b"telegram".len())
            .any(|part| part == b"telegram")
}

#[cfg(all(unix, not(target_os = "linux")))]
fn process_command_matches(runtime: &ConnectorRuntime) -> bool {
    std::process::Command::new("ps")
        .args(["-p", &runtime.pid.to_string(), "-o", "command="])
        .output()
        .ok()
        .filter(|output| output.status.success())
        .is_some_and(|output| {
            let command = String::from_utf8_lossy(&output.stdout);
            command.contains(&runtime.executable.display().to_string())
                && command.split_whitespace().any(|part| part == "telegram")
        })
}

#[cfg(windows)]
fn process_command_matches(runtime: &ConnectorRuntime) -> bool {
    std::process::Command::new("powershell.exe")
        .args([
            "-NoProfile",
            "-NonInteractive",
            "-Command",
            &format!(
                "(Get-CimInstance Win32_Process -Filter 'ProcessId = {}').CommandLine",
                runtime.pid
            ),
        ])
        .output()
        .ok()
        .filter(|output| output.status.success())
        .is_some_and(|output| {
            let command = String::from_utf8_lossy(&output.stdout);
            command.contains(&runtime.executable.display().to_string())
                && command.split_whitespace().any(|part| part == "telegram")
        })
}

#[cfg(unix)]
fn detach_command(command: &mut std::process::Command) {
    use std::os::unix::process::CommandExt;
    command.process_group(0);
}

#[cfg(windows)]
fn detach_command(command: &mut std::process::Command) {
    use std::os::windows::process::CommandExt;
    use windows_sys::Win32::System::Threading::CREATE_NEW_PROCESS_GROUP;
    use windows_sys::Win32::System::Threading::DETACHED_PROCESS;
    command.creation_flags(CREATE_NEW_PROCESS_GROUP | DETACHED_PROCESS);
}

#[cfg(unix)]
fn stop_runtime(codex_home: &Path, runtime: &ConnectorRuntime) -> Result<(), String> {
    let result = unsafe { libc::kill(-(runtime.pid as libc::pid_t), libc::SIGTERM) };
    if result != 0 && std::io::Error::last_os_error().raw_os_error() != Some(libc::ESRCH) {
        return Err("Could not stop the Telegram connector process.".to_string());
    }
    remove_runtime(codex_home)
}

#[cfg(windows)]
fn stop_runtime(codex_home: &Path, runtime: &ConnectorRuntime) -> Result<(), String> {
    let status = std::process::Command::new("taskkill.exe")
        .args(["/PID", &runtime.pid.to_string(), "/T"])
        .status()
        .map_err(|_| "Could not stop the Telegram connector process.".to_string())?;
    if !status.success() && process_is_running(runtime.pid) {
        return Err("Could not stop the Telegram connector process.".to_string());
    }
    remove_runtime(codex_home)
}

#[cfg(test)]
#[path = "service_tests.rs"]
mod tests;
