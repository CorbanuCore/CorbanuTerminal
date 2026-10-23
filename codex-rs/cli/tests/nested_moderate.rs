//! #766 (Travis, 2026-10-14): `corbanu exec` started by an agent command
//! under security level Moderate runs with Moderate enforced, through the
//! real sandbox (Seatbelt on macOS, bubblewrap on Linux). With the same
//! Corbanu home, which the agent command can neither read nor write, it
//! starts in a home of its own; pointed at another Corbanu home, with the
//! account's home moved too, it is still held to Moderate. Permissive is
//! unchanged.
#![cfg(any(target_os = "macos", target_os = "linux"))]

use std::sync::Mutex;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;

use anyhow::Result;
use app_test_support::MockResponsesConfig;
use app_test_support::create_final_assistant_message_sse_response;
use app_test_support::create_shell_command_sse_response;
use pretty_assertions::assert_eq;
use tempfile::TempDir;
use wiremock::Mock;
use wiremock::MockServer;
use wiremock::Request;
use wiremock::Respond;
use wiremock::ResponseTemplate;
use wiremock::matchers::method;
use wiremock::matchers::path_regex;

/// In the model's permissions instructions while Moderate is in force.
const MODERATE_INSTRUCTED: &str = "a command never leaves the sandbox it runs in";
/// The notice a nested run held to Moderate prints when it starts.
const HELD: &str =
    "Security level Moderate is enforced because an agent command under Moderate started this run";
const OUTER_PROMPT: &str = "outer-766-prompt";
const NESTED_PROMPT: &str = "nested-766-prompt";
const NESTED_REPLY: &str = "nested-766-reply";
const VAULT_CANARY: &str = "moderate-766-vault-canary";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Level {
    Permissive,
    Moderate,
    /// `[security] level = "aggressive"` in config (Core's level), with a
    /// stored level file that agent commands cannot read.
    AggressiveInCore,
    /// Moderate in config while `/security` stored Aggressive (Core's level
    /// was not raised with it), in a run the TUI did not start.
    ModerateStoredAggressive,
}

/// What the outer agent's command starts.
#[derive(Clone, Copy, Debug)]
enum Target {
    /// `corbanu exec` in the same Corbanu home: the environment it inherits.
    SameHome,
    /// `corbanu exec` in a home the agent wrote itself in the workspace, with
    /// `HOME` moved there too.
    OtherHome,
    /// A session, a host and a credential command, each in the same home.
    NotAgents,
    /// `corbanu exec` with `CORBANU_SECURITY_ORIGIN` naming a home the agent
    /// made, with an unreadable marker file in it.
    MadeUpMarker,
    /// #826: [`Target::OtherHome`] in a user namespace of the command's own
    /// (`unshare -Ur`), where the marker bubblewrap hides would be readable.
    #[cfg_attr(not(target_os = "linux"), allow(dead_code))]
    UnshareOtherHome,
    /// #826: [`Target::SameHome`] in a user namespace of the command's own,
    /// where the vault store bubblewrap hides would be readable.
    #[cfg_attr(not(target_os = "linux"), allow(dead_code))]
    UnshareSameHome,
}

/// How the outer run is set up.
#[derive(Clone, Copy, Debug)]
struct Setup {
    level: Level,
    target: Target,
    /// The outer agent runs a command before the nested launch, so untrusted
    /// content (its output) is in the session by then.
    untrusted_first: bool,
    /// Whether Core can register the home in the account's registry.
    registry_writable: bool,
    /// The Corbanu home is the account's default `~/.corbanu`, beside a
    /// `~/.pfterminal` that stores a level and a vault (denied whole to agent
    /// commands by the protected-file rules), with `HOME` set to the account.
    default_home: bool,
}

impl Setup {
    fn new(level: Level, target: Target) -> Self {
        Self {
            level,
            target,
            untrusted_first: false,
            registry_writable: true,
            default_home: false,
        }
    }
}

/// Answers each request with the next response of a sequence set after the
/// server's address is known.
struct Sequence {
    calls: AtomicUsize,
    responses: Mutex<Vec<String>>,
}

impl Respond for Sequence {
    fn respond(&self, _: &Request) -> ResponseTemplate {
        let call = self.calls.fetch_add(1, Ordering::SeqCst);
        let responses = self
            .responses
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        match responses.get(call) {
            Some(body) => ResponseTemplate::new(200)
                .insert_header("content-type", "text/event-stream")
                .set_body_raw(body.clone(), "text/event-stream"),
            None => ResponseTemplate::new(500),
        }
    }
}

struct Run {
    /// Every request the mock model received.
    bodies: Vec<String>,
    stderr: String,
}

impl Run {
    /// The nested run's request, if it reached the model.
    fn nested_request(&self) -> Option<&String> {
        self.bodies
            .iter()
            .find(|body| body.contains(NESTED_PROMPT) && !body.contains(OUTER_PROMPT))
    }

    /// The outer run's requests, the command's output included.
    fn outer_requests(&self) -> String {
        self.bodies
            .iter()
            .filter(|body| body.contains(OUTER_PROMPT))
            .cloned()
            .collect::<Vec<_>>()
            .join("\n")
    }

    /// What the outer agent's command printed, as the model got it back.
    fn command_output(&self) -> String {
        self.bodies
            .iter()
            .filter_map(|body| serde_json::from_str::<serde_json::Value>(body).ok())
            .flat_map(|body| body["input"].as_array().cloned().unwrap_or_default())
            .filter(|item| item["type"] == "function_call_output")
            .map(|item| item["output"].to_string())
            .collect::<Vec<_>>()
            .join("\n")
    }
}

/// `value` as one POSIX shell word.
fn quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\\''"))
}

/// The flags an agent passes so the nested run reaches `server_uri`.
fn provider_flags(server_uri: &str) -> Vec<String> {
    vec![
        "-c".to_string(),
        "model_provider=\"mock_provider\"".to_string(),
        "-c".to_string(),
        format!(
            "model_providers.mock_provider={{ name = \"mock\", base_url = \"{server_uri}/v1\", wire_api = \"responses\", request_max_retries = 0, stream_max_retries = 0 }}"
        ),
        "-m".to_string(),
        "mock-model".to_string(),
    ]
}

async fn run(setup: Setup) -> Result<Run> {
    let Setup {
        level,
        target,
        untrusted_first,
        registry_writable,
        default_home,
    } = setup;
    let temp_home = TempDir::new()?;
    let root = TempDir::new()?;
    // Resolved as the session resolves its folder (macOS temp folders sit
    // behind a symlink).
    let root_path = root.path().canonicalize()?;
    let cwd = root_path.join("workspace");
    // Stands in for the account database's home directory (the registry).
    let account = root_path.join("account");
    std::fs::create_dir_all(&cwd)?;
    std::fs::create_dir_all(&account)?;
    let home = if default_home {
        let other = account.join(".pfterminal");
        std::fs::create_dir_all(other.join("secrets"))?;
        std::fs::create_dir_all(other.join("tmp"))?;
        std::fs::write(
            other.join("security_level.toml"),
            "version = 1\nlevel = \"permissive\"\n",
        )?;
        account.join(".corbanu")
    } else {
        temp_home.path().to_path_buf()
    };
    std::fs::create_dir_all(&home)?;
    if !registry_writable {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&account, std::fs::Permissions::from_mode(0o555))?;
    }
    // A vault store exists, as it does once the person has used /vault.
    std::fs::create_dir_all(home.join("secrets"))?;
    std::fs::write(home.join("secrets/probe.txt"), VAULT_CANARY)?;

    let server = MockServer::start().await;
    let sequence = Sequence {
        calls: AtomicUsize::new(0),
        responses: Mutex::new(Vec::new()),
    };

    let bin = codex_utils_cargo_bin::cargo_bin("codex")?;
    let mut nested = vec![bin.to_string_lossy().into_owned(), "exec".to_string()];
    nested.push("--skip-git-repo-check".to_string());
    nested.extend(provider_flags(&server.uri()));
    nested.push(NESTED_PROMPT.to_string());
    let nested = nested
        .iter()
        .map(|arg| quote(arg))
        .collect::<Vec<_>>()
        .join(" ");
    let bin_word = quote(&bin.to_string_lossy());
    let script = match target {
        Target::SameHome => format!("{nested} 2>&1"),
        Target::UnshareSameHome => format!(
            "echo marker=${{CODEX_SANDBOX_USER_NAMESPACES_DENIED:-unset}}; unshare -Ur {nested} 2>&1; echo \"exit=$?\""
        ),
        Target::MadeUpMarker => format!(
            "mkdir -p made-up/tmp && : > made-up/tmp/security-moderate-marker && chmod 000 made-up/tmp/security-moderate-marker && CORBANU_SECURITY_ORIGIN=\"$PWD/made-up\" {nested} 2>&1; echo \"exit=$?\"; env -u CORBANU_HOME -u PFTERMINAL_HOME CODEX_HOME=\"$PWD/made-up\" {nested} 2>&1; echo \"exit=$?\""
        ),
        Target::NotAgents => [
            "resume --last",
            "app-server",
            "vault auth-helper provider/zai_api_key",
        ]
        .iter()
        .map(|args| format!("{bin_word} {args} </dev/null 2>&1; echo \"exit=$?\""))
        .collect::<Vec<_>>()
        .join("; "),
        Target::OtherHome | Target::UnshareOtherHome => {
            let other = cwd.join("other-home");
            std::fs::create_dir_all(&other)?;
            MockResponsesConfig::new(&server.uri())
                .with_sandbox_mode("workspace-write")
                .write(&other)?;
            let unshare = if matches!(target, Target::UnshareOtherHome) {
                "echo marker=${CODEX_SANDBOX_USER_NAMESPACES_DENIED:-unset}; unshare -Ur "
            } else {
                ""
            };
            format!(
                "{unshare}env -u CORBANU_HOME -u PFTERMINAL_HOME CODEX_HOME={} HOME={} {nested} 2>&1; echo \"exit=$?\"",
                quote(&other.to_string_lossy()),
                quote(&cwd.to_string_lossy()),
            )
        }
    };
    let mut responses = Vec::new();
    if untrusted_first {
        responses.push(create_shell_command_sse_response(
            vec!["echo".to_string(), "before".to_string()],
            /*workdir*/ None,
            Some(60_000),
            "before",
        )?);
    }
    responses.push(create_shell_command_sse_response(
        vec!["sh".to_string(), "-c".to_string(), script],
        /*workdir*/ None,
        Some(120_000),
        "nested",
    )?);
    if matches!(
        target,
        Target::SameHome | Target::OtherHome | Target::UnshareOtherHome | Target::UnshareSameHome
    ) {
        responses.push(create_final_assistant_message_sse_response(NESTED_REPLY)?);
    }
    responses.push(create_final_assistant_message_sse_response("outer done")?);
    *sequence
        .responses
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner) = responses;
    Mock::given(method("POST"))
        .and(path_regex(".*/responses$"))
        .respond_with(sequence)
        .mount(&server)
        .await;

    let level_config = match level {
        Level::Permissive => "",
        Level::Moderate => "[security]\nversion = 1\nlevel = \"moderate\"\n",
        Level::ModerateStoredAggressive => {
            std::fs::write(
                home.join("security_level.toml"),
                "version = 1\nlevel = \"aggressive\"\n",
            )?;
            "[security]\nversion = 1\nlevel = \"moderate\"\n"
        }
        Level::AggressiveInCore => {
            std::fs::write(
                home.join("security_level.toml"),
                "version = 1\nlevel = \"aggressive\"\n",
            )?;
            "[security]\nversion = 1\nlevel = \"aggressive\"\n"
        }
    };
    MockResponsesConfig::new(&server.uri())
        .with_sandbox_mode("workspace-write")
        .enable_feature(codex_features::Feature::SourceEnvelopes)
        .with_extra_config(&format!(
            "{level_config}\n[sandbox_workspace_write]\nnetwork_access = true\n\n[projects.\"{}\"]\ntrust_level = \"trusted\"\n",
            cwd.display()
        ))
        .write(&home)?;

    let mut command = assert_cmd::Command::new(&bin);
    for name in [
        "CORBANU_HOME",
        "PFTERMINAL_HOME",
        "CODEX_SQLITE_HOME",
        "CORBANU_SECURITY_ORIGIN",
    ] {
        command.env_remove(name);
    }
    if default_home {
        command.env("HOME", &account);
    }
    let output = command
        .current_dir(&cwd)
        .env("CODEX_HOME", &home)
        .env("CORBANU_TEST_ACCOUNT_HOME", &account)
        .env("CORBANU_TEST_NO_NATIVE_KEYRING", "1")
        .args(["exec", "--skip-git-repo-check", OUTER_PROMPT])
        .timeout(std::time::Duration::from_secs(300))
        .output()?;
    if !registry_writable {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&account, std::fs::Permissions::from_mode(0o755))?;
    }
    let stderr = String::from_utf8_lossy(&output.stderr).into_owned();
    let bodies = server
        .received_requests()
        .await
        .unwrap_or_default()
        .iter()
        .map(|request| String::from_utf8_lossy(&request.body).into_owned())
        .collect::<Vec<_>>();
    let run = Run { bodies, stderr };
    assert!(
        run.outer_requests().contains("\"nested\""),
        "the outer run never ran its command: {}",
        run.stderr
    );
    assert!(
        !run.bodies.iter().any(|body| body.contains(VAULT_CANARY)),
        "the vault store reached the model"
    );
    Ok(run)
}

/// Asserts that the nested run started, held to Moderate, and reached the
/// model, which saw Moderate's instructions.
fn assert_held_to_moderate(run: &Run) {
    let output = run.command_output();
    eprintln!("nested run output: {output}");
    let nested = run
        .nested_request()
        .unwrap_or_else(|| panic!("the nested run never reached the model:\n{output}"));
    assert!(output.contains(HELD), "no Moderate notice:\n{output}");
    assert!(
        nested.contains(MODERATE_INSTRUCTED),
        "the nested run's model was not told about Moderate:\n{output}"
    );
    assert!(
        output.contains(NESTED_REPLY),
        "the nested run did not finish:\n{output}"
    );
}

/// Same home: before #766 the nested run stopped at start, unable to read
/// the Corbanu config inside the sandbox.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn issue_766_same_home_starts_held_to_moderate() -> Result<()> {
    let run = run(Setup::new(Level::Moderate, Target::SameHome)).await?;
    assert_held_to_moderate(&run);
    let output = run.command_output();
    assert!(
        !output.contains("Error loading config.toml") && !output.contains("Aggressive"),
        "{output}"
    );
    Ok(())
}

/// Another home the agent wrote (Permissive, no level), `HOME` moved and no
/// marker in the environment: before #766 it ran at that home's level.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn issue_766_other_home_is_held_to_moderate() -> Result<()> {
    let run = run(Setup::new(Level::Moderate, Target::OtherHome)).await?;
    assert_held_to_moderate(&run);
    Ok(())
}

/// What cannot be held to Moderate is refused, naming Moderate: a session
/// (the agent would answer its prompts), a host and a credential command.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn issue_766_sessions_hosts_and_credentials_are_refused() -> Result<()> {
    let run = run(Setup::new(Level::Moderate, Target::NotAgents)).await?;
    let output = run.command_output();
    assert_eq!(
        (
            output
                .matches(
                    "was started by an agent command while security level Moderate is enforced"
                )
                .count(),
            output.matches("exit=1").count(),
            output.contains("Aggressive"),
        ),
        (3, 3, false),
        "{output}"
    );
    Ok(())
}

/// Permissive is unchanged: a nested run in the same home, before any
/// untrusted content, starts at Permissive as before.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn issue_766_permissive_nested_run_is_unchanged() -> Result<()> {
    let run = run(Setup::new(Level::Permissive, Target::SameHome)).await?;
    let output = run.command_output();
    let nested = run
        .nested_request()
        .unwrap_or_else(|| panic!("the nested run never reached the model:\n{output}"));
    assert_eq!(
        (
            output.contains("Moderate"),
            nested.contains(MODERATE_INSTRUCTED)
        ),
        (false, false),
        "{output}"
    );
    Ok(())
}

/// Permissive after a command's output (labelled untrusted) is unchanged
/// too: nothing marks Moderate, and the nested run is not held to it.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn issue_766_permissive_after_a_command_is_unchanged() -> Result<()> {
    let run = run(Setup {
        untrusted_first: true,
        ..Setup::new(Level::Permissive, Target::SameHome)
    })
    .await?;
    let output = run.command_output();
    let nested = run
        .nested_request()
        .unwrap_or_else(|| panic!("the nested run never reached the model:\n{output}"));
    assert_eq!(
        (
            output.contains("Moderate"),
            nested.contains(MODERATE_INSTRUCTED)
        ),
        (false, false),
        "{output}"
    );
    Ok(())
}

/// The same home when Core cannot register it (the account's registry is
/// read-only): the run that moved to a home of its own stays held to
/// Moderate, although it can no longer find the origin itself.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn issue_766_same_home_without_registry_stays_held() -> Result<()> {
    let run = run(Setup {
        registry_writable: false,
        ..Setup::new(Level::Moderate, Target::SameHome)
    })
    .await?;
    assert_held_to_moderate(&run);
    Ok(())
}

/// Aggressive in Core refuses nested launches (its level file is unreadable
/// to agent commands). A marker an agent command made in a home of its own,
/// named in `CORBANU_SECURITY_ORIGIN` or as the run's `CODEX_HOME`, does not
/// turn that into a run held only to Moderate, also beside another default
/// home the rules deny whole.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn issue_766_made_up_marker_does_not_relax_aggressive() -> Result<()> {
    let run = run(Setup {
        default_home: true,
        ..Setup::new(Level::AggressiveInCore, Target::MadeUpMarker)
    })
    .await?;
    let output = run.command_output();
    assert_eq!(
        (
            output
                .matches("while security level Aggressive is enforced")
                .count(),
            output.matches("exit=1").count(),
            output.contains("Moderate"),
            run.nested_request().is_some(),
        ),
        (2, 2, false, false),
        "{output}"
    );
    Ok(())
}

/// The account's default home beside another default home that stores a
/// level and a vault (denied whole to agent commands, so its level reads as
/// unreadable): the nested run still starts, held to Moderate.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn issue_766_default_home_beside_another_stored_level_starts_held() -> Result<()> {
    let run = run(Setup {
        default_home: true,
        ..Setup::new(Level::Moderate, Target::SameHome)
    })
    .await?;
    assert_held_to_moderate(&run);
    Ok(())
}

/// Aggressive stored in `/security` while Core's level is Moderate: Core
/// leaves the home unmarked, and the nested launch is refused naming
/// Aggressive, as before.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn issue_766_stored_aggressive_is_refused_as_before() -> Result<()> {
    let run = run(Setup::new(
        Level::ModerateStoredAggressive,
        Target::SameHome,
    ))
    .await?;
    let output = run.command_output();
    assert_eq!(
        (
            output.contains("while security level Aggressive is enforced"),
            output.contains("Moderate is enforced"),
            run.nested_request().is_some(),
        ),
        (true, false, false),
        "{output}"
    );
    Ok(())
}

/// #826 (Travis, 2026-10-15): an agent command cannot get round nested-launch
/// detection in a user namespace of its own (`unshare -Ur`), where the files
/// bubblewrap hides (the Moderate marker, the vault store) would be readable:
/// under Moderate and Aggressive the sandbox refuses the namespace, so the
/// nested run never starts outside the level's rules. Before #826, pointed at
/// another home under Moderate it ran at that home's level (Permissive), and
/// in the same home under Aggressive it was no longer refused. Linux only.
#[cfg(target_os = "linux")]
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn issue_826_user_namespace_does_not_hide_the_origin() -> Result<()> {
    let mut wrong = Vec::new();
    for (level, target) in [
        (Level::Moderate, Target::UnshareOtherHome),
        (Level::Moderate, Target::UnshareSameHome),
        (Level::AggressiveInCore, Target::UnshareSameHome),
    ] {
        let run = run(Setup::new(level, target)).await?;
        let output = run.command_output();
        eprintln!("{level:?} {target:?}: {output}");
        let seen = (
            output.contains("marker=1"),
            output.contains("unshare failed: Operation not permitted"),
            output.contains("exit=1"),
            run.nested_request().is_some(),
        );
        if seen != (true, true, true, false) {
            wrong.push(format!("{level:?} {target:?}: {seen:?}\n{output}"));
        }
    }
    assert_eq!(wrong, Vec::<String>::new());
    Ok(())
}

/// #826: Permissive is unchanged: the sandbox neither refuses the namespace
/// nor marks the command, and a nested run in one starts as before where the
/// host allows it.
#[cfg(target_os = "linux")]
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn issue_826_permissive_user_namespace_is_unchanged() -> Result<()> {
    let run = run(Setup::new(Level::Permissive, Target::UnshareOtherHome)).await?;
    let output = run.command_output();
    eprintln!(
        "Permissive: nested={} {output}",
        run.nested_request().is_some()
    );
    assert!(
        output.contains("marker=unset") && !output.contains("Moderate"),
        "{output}"
    );
    Ok(())
}
