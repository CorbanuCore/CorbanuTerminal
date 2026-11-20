//! Shared Windows sandbox behavior over the real exec-server RPC connection.

use super::*;
use codex_exec_server::ExecEnvPolicy;
use codex_protocol::config_types::ShellEnvironmentPolicyInherit;
use codex_protocol::models::PermissionProfile;
use codex_protocol::permissions::FileSystemAccessMode;
use codex_protocol::permissions::FileSystemPath;
use codex_protocol::permissions::FileSystemSandboxEntry;
use codex_protocol::permissions::FileSystemSandboxPolicy;
use codex_protocol::permissions::FileSystemSpecialPath;
use codex_protocol::permissions::NetworkSandboxPolicy;
use pretty_assertions::assert_eq;

/// Match the protected user-owned roots used by native sandbox tests (#158).
/// Runner temp ACLs may already grant writes to ordinary users, which would
/// make a parent canary writable independently of the sandbox's grants.
#[cfg(windows)]
fn protect_temp_fixture_root(root: &std::path::Path) -> Result<()> {
    let system32 =
        std::path::PathBuf::from(std::env::var_os("SystemRoot").context("SystemRoot is missing")?)
            .join("System32");
    let output = std::process::Command::new(system32.join("whoami.exe"))
        .arg("/user")
        .output()?;
    anyhow::ensure!(output.status.success(), "fixture user SID lookup failed");
    let stdout = String::from_utf8_lossy(&output.stdout);
    let sid = stdout
        .split_whitespace()
        .find(|token| token.starts_with("S-1-"))
        .context("fixture user SID was not reported")?;
    let output = std::process::Command::new(system32.join("icacls.exe"))
        .arg(root)
        .args([
            "/inheritance:r",
            "/grant:r",
            "*S-1-5-18:(OI)(CI)F",
            "*S-1-5-32-544:(OI)(CI)F",
        ])
        .arg(format!("*{sid}:(OI)(CI)F"))
        .output()?;
    anyhow::ensure!(
        output.status.success(),
        "protect fixture root: {}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    Ok(())
}

/// Windows alias casing must not prevent fallback when the primary runtime is absent from PATH.
#[test_case::test_case("pwsh.exe"; "lowercase_pwsh")]
#[test_case::test_case("PWSH.exe"; "uppercase_pwsh")]
#[test_case::test_case("powershell.exe"; "lowercase_powershell")]
#[test_case::test_case("PowerShell.exe"; "mixed_case_powershell")]
#[cfg_attr(not(windows), ignore = "requires a native Windows sandbox")]
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
#[serial_test::serial(remote_exec_server)]
async fn powershell_alias_falls_back_without_primary_runtime_over_rpc(
    shell_name: &str,
) -> Result<()> {
    crate::skip_if_mxc_unavailable!(Ok(()));
    let root = TempDir::new()?;
    let system32 = std::path::PathBuf::from(std::env::var("SystemRoot")?).join("System32");
    let server =
        common::exec_server::exec_server_with_env([("PATH", system32.as_os_str())], &[]).await?;
    let environment = Environment::create_for_tests(Some(server.websocket_url().to_owned()))?;
    let cwd = PathUri::from_host_native_path(root.path())?;
    let mut sandbox = FileSystemSandboxContext::from_permission_profile(
        PermissionProfile::read_only(),
        cwd.clone(),
    );
    sandbox.windows_sandbox_selection = codex_exec_server::WindowsSandboxSelection::Mxc;
    let started = environment
        .get_exec_backend()
        .start(ExecParams {
            process_id: ProcessId::from("windows-powershell-fallback"),
            metadata: None,
            argv: vec![
                root.path()
                    .join("WindowsApps")
                    .join(shell_name)
                    .to_string_lossy()
                    .into_owned(),
                "-NoProfile".to_owned(),
                "-Command".to_owned(),
                "if (!$env:SystemDrive -or !$env:LOCALAPPDATA -or !$env:USERPROFILE -or !$env:PATH) { throw 'missing Windows defaults' }; Write-Output 'fallback-ok'".to_owned(),
            ],
            cwd,
            env_policy: None,
            shell_snapshot: None,
            env: HashMap::new(),
            tty: false,
            pipe_stdin: false,
            arg0: None,
            sandbox: Some(sandbox),
            enforce_managed_network: false,
            managed_network: None,
            network_proxy: None,
        })
        .await?;
    assert_eq!(
        started.sandbox_type,
        Some(codex_sandboxing::SandboxType::WindowsMxc)
    );
    let (stdout, stderr, exit_code, exited) = collect_process_output_from_events_with_timeout(
        started.process,
        Duration::from_secs(/*secs*/ 30),
    )
    .await?;
    assert_eq!(
        (stdout.trim(), stderr, exit_code, exited),
        ("fallback-ok", String::new(), Some(0), true)
    );
    Ok(())
}

#[test_case::test_case(WindowsSandboxSelection::Mxc, codex_sandboxing::SandboxType::WindowsMxc; "mxc")]
#[test_case::test_case(WindowsSandboxSelection::Elevated, codex_sandboxing::SandboxType::WindowsRestrictedToken; "elevated")]
#[cfg_attr(not(windows), ignore = "requires a native Windows sandbox")]
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
#[serial_test::serial(remote_exec_server)]
async fn tmpdir_uses_command_environment_over_rpc(
    selection: WindowsSandboxSelection,
    expected_type: codex_sandboxing::SandboxType,
) -> Result<()> {
    if selection == WindowsSandboxSelection::Mxc {
        crate::skip_if_mxc_unavailable!(Ok(()));
    }
    #[cfg(windows)]
    let _account_guard = if selection == WindowsSandboxSelection::Elevated {
        let guard = codex_windows_sandbox_test_support::WindowsSandboxAccountTestGuard::acquire()?;
        // The RPC server re-enters this test executable, as in file_system_windows.
        let executable = std::env::current_exe()?;
        let resources = executable
            .parent()
            .context("Windows test executable should have a parent directory")?
            .join("codex-resources");
        if let Err(error) = std::fs::create_dir_all(&resources)
            && !(error.kind() == std::io::ErrorKind::PermissionDenied && resources.is_dir())
        {
            return Err(error).context("create Windows sandbox test resources");
        }
        for name in ["codex-windows-sandbox-setup", "codex-command-runner"] {
            let source = codex_utils_cargo_bin::cargo_bin(name)?;
            let destination = resources.join(std::path::Path::new(name).with_extension("exe"));
            if let Err(error) = codex_utils_cargo_bin::copy_executable(&source, &destination)
                && !(error.kind() == std::io::ErrorKind::PermissionDenied && destination.is_file())
            {
                return Err(error).with_context(|| format!("stage Windows sandbox helper {name}"));
            }
        }
        Some(guard)
    } else {
        None
    };
    let root = TempDir::new()?;
    #[cfg(windows)]
    protect_temp_fixture_root(root.path())?;
    let command_temp = root.path().join("command temp");
    let server_temp = root.path().join("server temp");
    std::fs::create_dir(&command_temp)?;
    std::fs::create_dir(&server_temp)?;
    let outside = server_temp.join("outside.txt");
    std::fs::write(&outside, "original")?;
    let command_temp_file = command_temp.join("outside-session.txt");
    std::fs::write(&command_temp_file, "original")?;
    // Use a batch fixture, as the native D377 probes do. (Since #2720 an
    // inline /C command line reaches cmd.exe as typed; the fixture just keeps
    // the probe's own quoting out of it.)
    let probe_name = "probe-session-temp.cmd";
    std::fs::write(
        root.path().join(probe_name),
        concat!(
            "@echo off\r\n",
            "echo CHILD-TEMP:%TEMP%\r\n",
            "echo allowed>\"%TEMP%\\allowed.txt\"\r\n",
            "if exist \"%TEMP%\\allowed.txt\" echo TEMP-WRITABLE\r\n",
            "echo modified>%COMMAND_TEMP_FILE%\r\n",
            "echo modified>%OUTSIDE_FILE%\r\n",
            "exit /b 0\r\n",
        ),
    )?;
    let server = common::exec_server::exec_server_with_env(
        [
            ("TEMP", server_temp.as_os_str()),
            ("TMP", server_temp.as_os_str()),
        ],
        &[],
    )
    .await?;
    let environment = Environment::create_for_tests(Some(server.websocket_url().to_owned()))?;
    let cwd = PathUri::from_host_native_path(root.path())?;
    let fs = FileSystemSandboxPolicy::restricted(vec![
        FileSystemSandboxEntry::new(
            FileSystemPath::Special {
                value: FileSystemSpecialPath::Root,
            },
            FileSystemAccessMode::Read,
        ),
        FileSystemSandboxEntry::new(
            FileSystemPath::Special {
                value: FileSystemSpecialPath::Tmpdir,
            },
            FileSystemAccessMode::Write,
        ),
    ]);
    let mut sandbox = FileSystemSandboxContext::from_permission_profile(
        PermissionProfile::from_runtime_permissions(&fs, NetworkSandboxPolicy::Restricted),
        cwd.clone(),
    );
    sandbox.windows_sandbox_selection = selection;
    let env = HashMap::from([
        ("SystemRoot".to_owned(), std::env::var("SystemRoot")?),
        (
            "COMMAND_TEMP_FILE".to_owned(),
            format!("\"{}\"", command_temp_file.display()),
        ),
        (
            "OUTSIDE_FILE".to_owned(),
            format!("\"{}\"", outside.display()),
        ),
    ]);
    // Raw params have no TEMP. The executor must finish the env policy before
    // resolving permissions. No inheritance: omitted TMP must not grant server_temp.
    let mut configured_temp = HashMap::from([(
        "TEMP".to_owned(),
        command_temp.to_string_lossy().into_owned(),
    )]);
    if selection == WindowsSandboxSelection::Mxc {
        configured_temp.insert(
            "TMP".to_owned(),
            command_temp.to_string_lossy().into_owned(),
        );
    }
    let started = environment
        .get_exec_backend()
        .start(ExecParams {
            process_id: ProcessId::from("windows-sandbox-temp"),
            metadata: None,
            argv: vec![
                r"C:\Windows\System32\cmd.exe".to_owned(),
                "/D".to_owned(),
                "/S".to_owned(),
                "/C".to_owned(),
                probe_name.to_owned(),
            ],
            cwd,
            env_policy: Some(ExecEnvPolicy {
                inherit: ShellEnvironmentPolicyInherit::None,
                ignore_default_excludes: false,
                exclude: Vec::new(),
                r#set: configured_temp,
                include_only: Vec::new(),
                drop_url_password_values: false,
            }),
            shell_snapshot: None,
            env,
            tty: false,
            pipe_stdin: false,
            arg0: None,
            sandbox: Some(sandbox),
            enforce_managed_network: false,
            managed_network: None,
            network_proxy: None,
        })
        .await?;
    assert_eq!(started.sandbox_type, Some(expected_type));
    let (stdout, stderr, code, exited) = collect_process_output_from_events_with_timeout(
        started.process,
        Duration::from_secs(/*secs*/ 30),
    )
    .await?;
    assert_eq!(
        (code, exited),
        (Some(0), true),
        "unexpected launch result, stderr: {stderr}"
    );
    let actual_temp = std::path::PathBuf::from(
        stdout
            .lines()
            .next()
            .context("child TEMP line")?
            .trim()
            .strip_prefix("CHILD-TEMP:")
            .context("child TEMP output")?,
    );
    assert!(
        stdout.lines().any(|line| line.trim() == "TEMP-WRITABLE"),
        "child could not write its TEMP {actual_temp:?}; stdout={stdout:?} stderr={stderr:?}"
    );
    if selection == WindowsSandboxSelection::Elevated {
        // D377: only the private session subfolder is writable, never the
        // command's parent temp folder or the server's separate temp folder.
        assert_eq!(
            std::fs::canonicalize(actual_temp.parent().context("session temp parent")?)?,
            std::fs::canonicalize(&command_temp)?,
        );
        assert!(
            actual_temp
                .file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| name.starts_with("corbanu-sandbox-tmp-"))
        );
        assert_eq!(std::fs::read_to_string(&command_temp_file)?, "original");
    } else {
        assert_eq!(
            std::fs::read_to_string(&command_temp_file)?.trim_end(),
            "modified"
        );
        if actual_temp != command_temp {
            // #2797: Windows gives an AppContainer child its own
            // `%LOCALAPPDATA%\Packages\<container>\AC\Temp` as TEMP and TMP,
            // whatever the supplied environment says (natively on Windows 11
            // build 26200). MXC makes a new `sandbox.{GUID}` container per
            // launch and removes it when the command ends, so the folder is
            // private to this command. The command's TEMP stays the only temp
            // folder we grant (D725, checked above).
            let packages = actual_temp.ancestors().nth(3);
            let container = actual_temp
                .ancestors()
                .nth(2)
                .and_then(std::path::Path::file_name)
                .and_then(|name| name.to_str())
                .unwrap_or_default();
            let local_app_data = std::fs::canonicalize(
                std::env::var_os("LOCALAPPDATA").context("LOCALAPPDATA is missing")?,
            )?;
            assert!(
                actual_temp.ends_with(r"AC\Temp")
                    && container.len() > "sandbox.{}".len()
                    && container.to_ascii_lowercase().starts_with("sandbox.{")
                    && container.ends_with('}')
                    && packages
                        .and_then(std::path::Path::file_name)
                        .is_some_and(|name| name.eq_ignore_ascii_case("Packages"))
                    && packages
                        .and_then(std::path::Path::parent)
                        .map(std::fs::canonicalize)
                        .transpose()?
                        .is_some_and(|parent| parent == local_app_data),
                "unexpected MXC child TEMP {actual_temp:?}"
            );
            // Removal can lag the exit slightly; a folder left behind would
            // keep the command's temp files.
            let container_dir = actual_temp
                .ancestors()
                .nth(2)
                .context("container folder")?
                .to_path_buf();
            let deadline = std::time::Instant::now() + Duration::from_secs(/*secs*/ 20);
            while container_dir.exists() && std::time::Instant::now() < deadline {
                tokio::time::sleep(Duration::from_millis(/*millis*/ 100)).await;
            }
            assert!(
                !container_dir.exists(),
                "MXC container folder outlived the command: {container_dir:?}"
            );
            assert_eq!(std::fs::read_to_string(&outside)?, "original");
            return Ok(());
        }
    }
    assert_eq!(
        (
            std::fs::read_to_string(actual_temp.join("allowed.txt"))
                .with_context(|| format!(
                    "read private TEMP child {actual_temp:?}; stderr={stderr:?}"
                ))?
                .trim_end()
                .to_owned(),
            std::fs::read_to_string(outside)?
        ),
        ("allowed".to_owned(), "original".to_owned())
    );
    Ok(())
}
