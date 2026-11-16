use std::borrow::Cow;
use std::collections::HashMap;

use codex_protocol::config_types::ShellEnvironmentPolicyInherit;
use codex_secret_broker::output_gate::OutputGate;
use codex_shell_command::shell_detect::ShellType;
use pretty_assertions::assert_eq;
use test_case::test_case;

use super::SnapshotCredentialFilter;
use super::split_values;
use crate::protocol::ExecEnvPolicy;

const TOKEN: &str = "ghp_0123456789abcdefghijABCDEFGHIJ012345";

fn filter<'g>(
    values: &[(&str, &str)],
    env_policy: Option<&ExecEnvPolicy>,
    process: &'g OutputGate,
) -> SnapshotCredentialFilter<'g> {
    let values = values
        .iter()
        .map(|(name, value)| (*name, Cow::Borrowed(*value)))
        .collect::<Vec<_>>();
    SnapshotCredentialFilter::with_gate(&values, env_policy, &HashMap::new(), process)
        .expect("filter")
}

#[test]
fn split_values_returns_capture_without_value_section() {
    let output =
        b"# Snapshot file\n\0\0\0A=1\0#values\0GH_TOKEN\0abc\0EMPTY\0\0#end\x002\0trap output";
    let (capture, values) = split_values(ShellType::Bash, output).expect("valid capture");
    assert_eq!(capture, b"# Snapshot file\n\0\0\0A=1\0");
    assert_eq!(
        values,
        vec![
            ("GH_TOKEN", Cow::Borrowed("abc")),
            ("EMPTY", Cow::Borrowed(""))
        ]
    );
}

#[test_case(b"# Snapshot file\n\0\0\0A=1\0" ; "missing_section")]
#[test_case(b"# Snapshot file\n\0\0\0A=1\0#values\0GH_TOKEN\0abc\0#end\x002\0" ; "count_mismatch")]
#[test_case(b"# Snapshot file\n\0\0\0A=1\0#values\0GH_TOKEN\0abc\0" ; "truncated")]
#[test_case(b"# Snapshot file\n\0\0\0A=1\0#values\0BAD NAME\0abc\0#end\x001\0" ; "invalid_name")]
#[test_case(b"# Snapshot file\n\0\0\0A=1\0stray\0#values\0#end\x000\0" ; "non_environment_record")]
fn split_values_rejects_malformed_sections(output: &[u8]) {
    let error = split_values(ShellType::Bash, output).expect_err("malformed capture");
    assert_eq!(error.0, "invalid_capture");
}

#[test]
fn filter_drops_credential_copies_under_any_name() {
    let process = OutputGate::new();
    // GH_TOKEN was never exported: only the value section reveals it.
    let filter = filter(&[("GH_TOKEN", TOKEN)], None, &process);
    for value in [
        TOKEN.to_string(),
        format!("Bearer {TOKEN}"),
        format!("{TOKEN}:suffix"),
    ] {
        assert!(!filter.keep("HIDDEN_HEADER", &value), "{value}");
    }
    assert!(filter.keep("ORDINARY_SETTING", "kept"));
    // #310: secret names are never replayed, whatever their value.
    assert!(!filter.keep("OTHER_TOKEN", "not-a-registered-value"));
    assert!(!filter.keep("__codex_snapshot_value", "plain"));
}

#[test]
fn filter_refuses_state_that_embeds_a_credential() {
    let process = OutputGate::new();
    let filter = filter(&[("GH_TOKEN", TOKEN)], None, &process);
    assert_eq!(
        filter
            .ensure_clean(&format!("leak () {{ printf '%s' '{TOKEN}'; }}\n"))
            .map_err(|(reason, _)| reason),
        Err("credential_value")
    );
    assert!(filter.ensure_clean("alias ll='ls -l'\n").is_ok());
}

#[test]
fn filter_detects_credential_shaped_values_and_url_passwords() {
    let process = OutputGate::new();
    let filter = filter(
        &[
            ("SERVICE_AUTH", "Zx9vQ2mL7pR4tW8k"),
            ("DATABASE_URL", "postgres://user:Pa55w0rd-x9@db.example/app"),
        ],
        None,
        &process,
    );
    assert!(!filter.keep("COPY", "Zx9vQ2mL7pR4tW8k"));
    assert!(!filter.keep("COPY", "--password=Pa55w0rd-x9"));
}

#[test]
fn filter_uses_process_registry() {
    let process = OutputGate::new();
    process
        .register(
            "vault",
            codex_secret_broker::output_gate::SecretClass::Operational,
            "vault-value-0123456789",
        )
        .expect("register");
    let filter = filter(&[], None, &process);
    assert!(!filter.keep("COPY", "x vault-value-0123456789 y"));
}

#[test]
fn excluded_names_protect_credential_values_only() {
    let process = OutputGate::new();
    let policy = ExecEnvPolicy {
        inherit: ShellEnvironmentPolicyInherit::All,
        ignore_default_excludes: false,
        // A provider key name that does not look like a credential, beside a
        // runtime flag whose short value must not refuse every snapshot.
        exclude: vec!["VENDOR_KEYVAL".to_string(), "RUNTIME_FLAG".to_string()],
        r#set: HashMap::new(),
        include_only: Vec::new(),
    };
    let filter = filter(
        &[
            ("VENDOR_KEYVAL", "vendor-Secret-1234"),
            ("RUNTIME_FLAG", "1"),
        ],
        Some(&policy),
        &process,
    );
    assert!(!filter.keep("COPY", "vendor-Secret-1234"));
    assert!(filter.keep("COUNT", "1"));
    assert!(filter.ensure_clean("echo 1\n").is_ok());
}

#[test]
fn broker_marker_is_not_a_credential() {
    let process = OutputGate::new();
    let marker = r#"[["OPENAI_API_KEY","sk-proj-masked0000"]]"#;
    let filter = filter(
        &[("CODEX_NETWORK_PROXY_BROKERED_CREDENTIALS", marker)],
        None,
        &process,
    );
    // Dropped by its #310 name, but its value refuses nothing.
    assert!(!filter.keep("CODEX_NETWORK_PROXY_BROKERED_CREDENTIALS", marker));
    assert!(filter.ensure_clean(&format!("# {marker}\n")).is_ok());
}

#[cfg(unix)]
mod shells {
    use std::collections::HashMap;

    use pretty_assertions::assert_eq;
    use test_case::test_case;

    use super::TOKEN;
    use crate::process_sandbox::NetworkPolicyHooks;
    use crate::process_sandbox::prepare_exec_request_with_telemetry;
    use crate::process_telemetry::ProcessTelemetry;
    use crate::protocol::ExecParams;
    use crate::protocol::ProcessId;
    use crate::protocol::ShellInfo;
    use crate::protocol::ShellSnapshotRequest;
    use crate::shell_snapshot::CapturePurpose;
    use crate::shell_snapshot::ShellSnapshotCache;
    use crate::telemetry::ExecServerTelemetry;

    /// Capture `profile` through the real executor path; returns the launch
    /// command and environment it prepared.
    async fn prepare(
        shell: &str,
        profile: &str,
    ) -> anyhow::Result<Option<(Vec<String>, HashMap<String, String>)>> {
        Ok(prepare_command(shell, profile, "true")
            .await?
            .map(|prepared| (prepared.command, prepared.env)))
    }

    struct Prepared {
        command: Vec<String>,
        env: HashMap<String, String>,
        reader: Option<std::fs::File>,
        home: tempfile::TempDir,
    }

    async fn prepare_command(
        shell: &str,
        profile: &str,
        script: &str,
    ) -> anyhow::Result<Option<Prepared>> {
        let path = format!("/bin/{shell}");
        if !std::path::Path::new(&path).exists() {
            eprintln!("skipping: {path} is unavailable");
            return Ok(None);
        }
        let home = tempfile::TempDir::new()?;
        let profile_name = match shell {
            "bash" => ".bashrc",
            "zsh" => ".zshrc",
            _ => ".snapshot-env",
        };
        std::fs::write(home.path().join(profile_name), profile)?;
        let home_path = home.path().to_string_lossy().into_owned();
        let mut env = HashMap::from([
            ("HOME".to_string(), home_path.clone()),
            ("PATH".to_string(), "/usr/bin:/bin".to_string()),
        ]);
        if shell == "sh" {
            env.insert("ENV".to_string(), format!("{home_path}/.snapshot-env"));
        }
        let params = ExecParams {
            metadata: Default::default(),
            process_id: ProcessId::from("snapshot-filter"),
            argv: vec![path.clone(), "-lc".to_string(), script.to_string()],
            cwd: codex_utils_path_uri::PathUri::from_host_native_path(home.path())?,
            env,
            env_policy: None,
            shell_snapshot: Some(ShellSnapshotRequest {
                scope_id: "attachment-1".to_string(),
                shell: ShellInfo {
                    name: shell.to_string(),
                    path,
                },
            }),
            tty: false,
            pipe_stdin: false,
            arg0: None,
            sandbox: None,
            enforce_managed_network: false,
            managed_network: None,
            network_proxy: None,
        };
        let brokers = codex_network_proxy::RemoteCredentialBrokerCache::default();
        let mut prepared = prepare_exec_request_with_telemetry(
            &params,
            params.env.clone(),
            /*runtime_paths*/ None,
            NetworkPolicyHooks::default(),
            &brokers,
            &ProcessTelemetry::default(),
        )
        .await
        .map_err(|error| anyhow::anyhow!("prepare: {error:?}"))?;
        let reader = ShellSnapshotCache::default()
            .prepare(
                &params,
                &mut prepared,
                &ExecServerTelemetry::default(),
                CapturePurpose::Execution,
                &brokers,
            )
            .await
            .map_err(|error| anyhow::anyhow!("snapshot: {error:?}"))?;
        Ok(Some(Prepared {
            command: prepared.command,
            env: prepared.env,
            reader,
            home,
        }))
    }

    fn uses_snapshot(command: &[String]) -> bool {
        command
            .last()
            .is_some_and(|script| script.contains("failed to restore shell snapshot"))
    }

    #[test_case("bash"; "bash")]
    #[test_case("sh"; "sh")]
    #[test_case("zsh"; "zsh")]
    #[tokio::test]
    async fn startup_credential_copies_are_not_replayed(shell: &str) -> anyhow::Result<()> {
        // The credential is never exported; a startup file copies it into an
        // innocuously named export, alone and inside a header value.
        let profile = format!(
            "GH_TOKEN={TOKEN}\nexport HIDDEN_HEADER=\"Bearer $GH_TOKEN\"\nexport HIDDEN_COPY=\"$GH_TOKEN\"\nexport ORDINARY_SETTING=kept\n"
        );
        let Some((command, env)) = prepare(shell, &profile).await? else {
            return Ok(());
        };
        assert!(uses_snapshot(&command), "{command:?}");
        assert_eq!(
            env.get("ORDINARY_SETTING").map(String::as_str),
            Some("kept")
        );
        assert_eq!(env.get("HIDDEN_HEADER"), None);
        assert_eq!(env.get("HIDDEN_COPY"), None);
        assert!(env.values().all(|value| !value.contains(TOKEN)));
        assert!(command.iter().all(|arg| !arg.contains(TOKEN)));
        Ok(())
    }

    #[test_case("bash", "leak() { printf '%s' 'TOKEN'; }"; "bash_function")]
    #[test_case("bash", "alias leak='printf TOKEN'"; "bash_alias")]
    #[test_case("sh", "leak() { printf '%s' 'TOKEN'; }"; "sh_function")]
    #[test_case("zsh", "leak() { printf '%s' 'TOKEN'; }"; "zsh_function")]
    #[test_case("zsh", "alias leak='printf TOKEN'"; "zsh_alias")]
    #[tokio::test]
    async fn credential_in_function_or_alias_refuses_snapshot(
        shell: &str,
        definition: &str,
    ) -> anyhow::Result<()> {
        let profile = format!(
            "GH_TOKEN={TOKEN}\nexport ORDINARY_SETTING=kept\n{}\n",
            definition.replace("TOKEN", TOKEN)
        );
        let Some((command, env)) = prepare(shell, &profile).await? else {
            return Ok(());
        };
        assert!(env.values().all(|value| !value.contains(TOKEN)));
        assert!(command.iter().all(|arg| !arg.contains(TOKEN)));
        // A POSIX sh without typeset/declare (dash) captures no functions, so
        // its snapshot has nothing to refuse; the assertions above still hold.
        let captures_functions = shell != "sh"
            || std::process::Command::new("/bin/sh")
                .args([
                    "-c",
                    "command -v typeset >/dev/null 2>&1 || command -v declare >/dev/null 2>&1",
                ])
                .status()?
                .success();
        if captures_functions {
            // Normal startup instead: nothing captured is replayed.
            assert_eq!(
                command,
                vec![
                    format!("/bin/{shell}"),
                    "-lc".to_string(),
                    "true".to_string()
                ]
            );
            assert_eq!(env.get("ORDINARY_SETTING"), None);
        }
        Ok(())
    }

    /// Bash sources ~/.bashrc for `bash -c` when stdin is a socket (as under
    /// the Linux sandbox) or SSH_CLIENT is set, privileged mode or not. Replay
    /// must not rerun startup: it would reintroduce exports the snapshot filtered.
    #[tokio::test]
    async fn bash_replay_never_reruns_startup_files() -> anyhow::Result<()> {
        use std::os::unix::process::CommandExt;

        let profile = format!(
            "printf x >> \"$HOME/rc-runs\"\nGH_TOKEN={TOKEN}\nexport HIDDEN_HEADER=\"Bearer $GH_TOKEN\"\nexport ORDINARY_SETTING=kept\n"
        );
        let Some(prepared) = prepare_command(
            "bash",
            &profile,
            "printf '%s|%s' \"$ORDINARY_SETTING\" \"${HIDDEN_HEADER-missing}\"",
        )
        .await?
        else {
            return Ok(());
        };
        assert!(uses_snapshot(&prepared.command), "{:?}", prepared.command);
        let fd = prepared
            .reader
            .as_ref()
            .map(std::os::fd::AsRawFd::as_raw_fd);
        let (stdin, _peer) = std::os::unix::net::UnixStream::pair()?;
        let mut command = std::process::Command::new(&prepared.command[0]);
        command
            .args(&prepared.command[1..])
            .env_clear()
            .envs(&prepared.env)
            // A top-level shell, as when the policy does not pass SHLVL.
            .env_remove("SHLVL")
            .env("SSH_CLIENT", "192.0.2.1 22 22")
            .stdin(std::process::Stdio::from(std::os::fd::OwnedFd::from(stdin)));
        if let Some(fd) = fd {
            // SAFETY: only clears CLOEXEC on the reader this test keeps open.
            unsafe {
                command.pre_exec(move || {
                    if libc::fcntl(fd, libc::F_SETFD, 0) == -1 {
                        return Err(std::io::Error::last_os_error());
                    }
                    Ok(())
                });
            }
        }
        let output = command.output()?;
        assert_eq!(
            (
                String::from_utf8_lossy(&output.stdout).into_owned(),
                output.status.success(),
            ),
            ("kept|missing".to_string(), true),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        // Captured once; the replay did not run the startup file again.
        assert_eq!(
            std::fs::read_to_string(prepared.home.path().join("rc-runs"))?,
            "x"
        );
        Ok(())
    }
}
