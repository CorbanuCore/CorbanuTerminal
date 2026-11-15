//! PF-27-S02 parity with local MCP and the stdio executor launcher (#2202).
use super::*;
use codex_exec_server::ExecBackendFuture;
use codex_protocol::config_types::EnvironmentVariablePattern;
use codex_protocol::config_types::ShellEnvironmentPolicy;
use codex_protocol::shell_environment;
use pretty_assertions::assert_eq;
use std::sync::Mutex;

const CREDENTIALS: &[&str] = &[
    "ANTHROPIC_API_KEY",
    "ZAI_API_KEY",
    "OPENROUTER_API_KEY",
    "CORBANU_VAULT_PASSWORD",
    "TASKNODE_API_TOKEN",
    "CUSTOM_PROVIDER_CREDENTIAL",
    "CARGO_REGISTRY_TOKEN",
    "HOMEBREW_GITHUB_API_TOKEN",
    "SSH_AUTH_SOCK",
    "NODE_OPTIONS",
    "UNLISTED_SETTING",
];

fn isolated(test: &str, check: impl FnOnce()) {
    const CHILD: &str = "CORBANU_MCP_SECRETLESS_TEST_CHILD";
    if std::env::var_os(CHILD).is_some() {
        check();
        return;
    }
    let mut command = std::process::Command::new(std::env::current_exe().expect("test binary"));
    command
        .args(["--exact", test, "--nocapture"])
        .env(CHILD, "1")
        .env("CORBANU_TEST_NO_NATIVE_KEYRING", "1")
        .env("CORBANU_MCP_FIXTURE_DIR", "/synthetic/host");
    for name in CREDENTIALS {
        command.env(name, "synthetic-inherited");
    }
    let output = command.output().expect("isolated test");
    assert!(
        output.status.success() && String::from_utf8_lossy(&output.stdout).contains("1 passed"),
        "{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

/// Records the actual launch request. No remote process or real credentials
/// are used, and executor-side environment evaluation deliberately does not
/// consult the orchestrator's process-wide armed flag.
#[derive(Default)]
struct RecordingExecutor(Mutex<Option<ExecParams>>);

impl ExecBackend for RecordingExecutor {
    fn start(&self, params: ExecParams) -> ExecBackendFuture<'_> {
        *self.0.lock().unwrap() = Some(params);
        Box::pin(async {
            Err(codex_exec_server::ExecServerError::Protocol(
                "fixture captured launch".to_string(),
            ))
        })
    }
}

#[derive(Clone, Copy)]
enum ExecutorPlatform {
    Host,
    Windows,
    Unix,
}

fn remote_env(env_vars: Vec<McpServerEnvVar>) -> HashMap<String, String> {
    remote_env_on_platform(env_vars, ExecutorPlatform::Host)
}

fn remote_env_on_platform(
    env_vars: Vec<McpServerEnvVar>,
    platform: ExecutorPlatform,
) -> HashMap<String, String> {
    let backend = Arc::new(RecordingExecutor::default());
    let command = StdioServerCommand::new(
        OsString::from("fixture-mcp"),
        Vec::new(),
        Some(HashMap::from([(
            OsString::from("ANTHROPIC_API_KEY"),
            OsString::from("synthetic-server-config"),
        )])),
        env_vars,
        Some(
            std::env::current_dir()
                .unwrap()
                .to_string_lossy()
                .into_owned(),
        ),
        McpProtocolMode::Legacy,
    );
    let runtime = tokio::runtime::Runtime::new().unwrap();
    let error = runtime
        .block_on(ExecutorStdioServerLauncher::launch_server(
            command,
            Arc::clone(&backend) as Arc<dyn ExecBackend>,
        ))
        .err()
        .expect("fixture stops before spawning");
    assert!(error.to_string().contains("fixture captured launch"));
    let params = backend.0.lock().unwrap().take().expect("launch request");
    let exec_policy = params.env_policy.expect("explicit environment policy");
    let mut policy = ShellEnvironmentPolicy {
        inherit: exec_policy.inherit,
        ignore_default_excludes: exec_policy.ignore_default_excludes,
        exclude: exec_policy
            .exclude
            .iter()
            .map(|name| EnvironmentVariablePattern::new_case_insensitive(name))
            .collect(),
        r#set: exec_policy.r#set,
        include_only: exec_policy
            .include_only
            .iter()
            .map(|name| EnvironmentVariablePattern::new_case_insensitive(name))
            .collect(),
        use_profile: false,
    };
    let mut inherited: HashMap<String, String> = CREDENTIALS
        .iter()
        .map(|name| (name.to_string(), "synthetic-remote".to_string()))
        .collect();
    inherited.extend([
        ("PATH".to_string(), "/synthetic/remote/bin".to_string()),
        ("HOME".to_string(), "/synthetic/remote/home".to_string()),
        (
            "CORBANU_MCP_FIXTURE_DIR".to_string(),
            "/synthetic/remote".to_string(),
        ),
        (
            "CORBANU_OTHER_TOKEN".to_string(),
            "synthetic-remote".to_string(),
        ),
    ]);
    inherited.insert("LC_CTYPE".to_string(), "synthetic-locale".to_string());
    if matches!(platform, ExecutorPlatform::Windows) {
        inherited.extend([
            ("SYSTEMROOT".to_string(), "C:\\Windows".to_string()),
            (
                "COMSPEC".to_string(),
                "C:\\Windows\\System32\\cmd.exe".to_string(),
            ),
            ("TEMP".to_string(), "C:\\Temp".to_string()),
            ("TMP".to_string(), "C:\\Temp".to_string()),
            ("USERNAME".to_string(), "fixture-user".to_string()),
            ("HOMEPATH".to_string(), "\\Users\\fixture".to_string()),
            (
                "APPDATA".to_string(),
                "C:\\Users\\fixture\\AppData".to_string(),
            ),
        ]);
    }
    let receiver_core = match platform {
        ExecutorPlatform::Host => None,
        ExecutorPlatform::Windows => Some(shell_environment::WINDOWS_CORE_ENV_VARS),
        ExecutorPlatform::Unix => Some(shell_environment::UNIX_CORE_ENV_VARS),
    };
    if let Some(core_names) = receiver_core
        && policy.inherit == ShellEnvironmentPolicyInherit::Core
    {
        // Model the receiver's Core selection before the shared policy
        // filters; using the host's Core list would hide cross-OS regressions.
        inherited.retain(|name, _| {
            core_names
                .iter()
                .any(|core| name.eq_ignore_ascii_case(core))
        });
        policy.inherit = ShellEnvironmentPolicyInherit::All;
    }
    let mut env =
        shell_environment::create_env_from_vars(inherited, &policy, /*thread_id*/ None);
    env.extend(params.env);
    env
}

fn assert_protected(env: &HashMap<String, String>) {
    assert_eq!(
        env.get("ANTHROPIC_API_KEY").map(String::as_str),
        Some("synthetic-server-config")
    );
    for name in &CREDENTIALS[1..] {
        assert!(
            !env.contains_key(*name),
            "inherited {name} escaped the launch boundary"
        );
    }
    assert!(!env.contains_key("CORBANU_OTHER_TOKEN"));
    assert_eq!(
        env.get("PATH").map(String::as_str),
        Some("/synthetic/remote/bin")
    );
}

#[test]
fn pf_27_s02_remote_source_credentials_are_filtered() {
    isolated(
        "stdio_server_launcher::secretless_tests::pf_27_s02_remote_source_credentials_are_filtered",
        || {
            let vars = || {
                CREDENTIALS
                    .iter()
                    .chain([&"CORBANU_MCP_FIXTURE_DIR"])
                    .map(|name| McpServerEnvVar::Config {
                        name: name.to_string(),
                        source: Some("remote".to_string()),
                    })
                    .collect()
            };
            let unarmed = remote_env(vars());
            assert_eq!(
                unarmed.get("ZAI_API_KEY").map(String::as_str),
                Some("synthetic-remote")
            );
            codex_protocol::secretless_launch::arm();
            let armed = remote_env(vars());
            assert_protected(&armed);
            assert_eq!(
                armed.get("CORBANU_MCP_FIXTURE_DIR").map(String::as_str),
                Some("/synthetic/remote")
            );
            // No named variables still retains the executor's basic environment.
            assert_protected(&remote_env(Vec::new()));
        },
    );
}

#[test]
fn pf_27_s02_remote_host_overlay_matches_local_mcp() {
    isolated(
        "stdio_server_launcher::secretless_tests::pf_27_s02_remote_host_overlay_matches_local_mcp",
        || {
            let vars: Vec<McpServerEnvVar> = CREDENTIALS
                .iter()
                .chain([&"CORBANU_MCP_FIXTURE_DIR"])
                .map(|name| (*name).into())
                .collect();
            let unarmed = remote_env(vars.clone());
            assert_eq!(
                unarmed.get("ZAI_API_KEY").map(String::as_str),
                Some("synthetic-inherited")
            );
            codex_protocol::secretless_launch::arm();
            let remote = remote_env(vars.clone());
            assert_protected(&remote);
            assert_eq!(
                remote.get("CORBANU_MCP_FIXTURE_DIR").map(String::as_str),
                Some("/synthetic/host")
            );
            let local = crate::utils::create_env_for_mcp_server(
                Some(HashMap::from([(
                    OsString::from("ANTHROPIC_API_KEY"),
                    OsString::from("synthetic-server-config"),
                )])),
                &vars,
            )
            .unwrap();
            for name in CREDENTIALS.iter().chain([&"CORBANU_MCP_FIXTURE_DIR"]) {
                assert_eq!(
                    local
                        .get(std::ffi::OsStr::new(name))
                        .map(|value| value.to_string_lossy().into_owned()),
                    remote.get(*name).cloned()
                );
            }
            assert_eq!(
                local.get(std::ffi::OsStr::new("CORBANU_TEST_NO_NATIVE_KEYRING")),
                Some(&OsString::from("1"))
            );
        },
    );
}

#[test]
fn pf_27_s02_remote_platform_defaults_survive_on_any_host() {
    isolated(
        "stdio_server_launcher::secretless_tests::pf_27_s02_remote_platform_defaults_survive_on_any_host",
        || {
            codex_protocol::secretless_launch::arm();
            for vars in [
                Vec::new(),
                vec![McpServerEnvVar::Config {
                    name: "ZAI_API_KEY".to_string(),
                    source: Some("remote".to_string()),
                }],
            ] {
                let unix = remote_env_on_platform(vars.clone(), ExecutorPlatform::Unix);
                let env = remote_env_on_platform(vars, ExecutorPlatform::Windows);
                assert_protected(&env);
                let names = [
                    "SYSTEMROOT",
                    "COMSPEC",
                    "TEMP",
                    "TMP",
                    "USERNAME",
                    "HOMEPATH",
                ];
                assert_eq!(
                    names.map(|name| env.get(name).map(String::as_str)),
                    [
                        Some("C:\\Windows"),
                        Some("C:\\Windows\\System32\\cmd.exe"),
                        Some("C:\\Temp"),
                        Some("C:\\Temp"),
                        Some("fixture-user"),
                        Some("\\Users\\fixture")
                    ]
                );
                assert!(
                    !env.contains_key("APPDATA"),
                    "off-allowlist defaults stay filtered"
                );
                assert_protected(&unix);
                assert_eq!(
                    unix.get("LC_CTYPE").map(String::as_str),
                    Some("synthetic-locale")
                );
            }
        },
    );
}

#[test]
fn pf_27_s02_remote_env_names_cannot_expand_as_globs() {
    isolated(
        "stdio_server_launcher::secretless_tests::pf_27_s02_remote_env_names_cannot_expand_as_globs",
        || {
            codex_protocol::secretless_launch::arm();
            for name in [
                "CORBANU_*",
                "CORBANU_OTHER_?????",
                "*PATH",
                "CORBANU_OTHER_TO\u{212a}EN",
                "CARGO_REGISTRY_TO\u{212a}EN",
                "HOMEBREW_GITHUB_API_TO\u{212a}EN",
            ] {
                let env = remote_env(vec![McpServerEnvVar::Config {
                    name: name.to_string(),
                    source: Some("remote".to_string()),
                }]);
                assert_protected(&env);
            }
        },
    );
}
