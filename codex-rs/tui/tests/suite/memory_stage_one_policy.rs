use std::fs;
use std::time::Duration;
use anyhow::Result;
use anyhow::ensure;
use codex_protocol::ThreadId;
use codex_protocol::models::ContentItem;
use codex_protocol::models::ResponseItem;
use codex_protocol::protocol::RolloutItem;
use codex_protocol::protocol::RolloutLine;
use codex_protocol::protocol::SessionSource;
use core_test_support::responses;
use tempfile::tempdir;
use wiremock::Mock;
use wiremock::MockServer;
use wiremock::ResponseTemplate;
use crate::support::tmux::CommandSpec;
use crate::support::tmux::SessionSpec;
use crate::support::tmux::TerminalSize;
use crate::support::tmux::TmuxKey;
use crate::support::tmux::TmuxServer;

const CANARY: &str = "PF30S04_SYNTHETIC_ROLLOUT_CANARY";
const TIMEOUT: Duration = Duration::from_secs(45);

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn tmux_memory_worker_policy_canary_permissive_and_protected() -> Result<()> {
    if !TmuxServer::should_run("PF30S04 memory dispatch")? { return Ok(()); }
    let repo = codex_utils_cargo_bin::repo_root()?;
    let binary = codex_utils_cargo_bin::cargo_bin("codex")?;
    for level in ["permissive", "moderate", "aggressive"] {
        let home = tempdir()?;
        let server = MockServer::start().await;
        Mock::given(wiremock::matchers::method("POST"))
            .respond_with(|request: &wiremock::Request| {
                let body = String::from_utf8_lossy(&request.body);
                let text = if body.contains(CANARY) {
                    r#"{"raw_memory":"synthetic extracted result","rollout_summary":"synthetic summary","rollout_slug":"fixture"}"#
                } else { "foreground fixture complete" };
                ResponseTemplate::new(200).insert_header("content-type", "text/event-stream")
                    .set_body_string(responses::sse(vec![responses::ev_response_created("fixture"),
                        responses::ev_assistant_message("fixture-message", text), responses::ev_completed("fixture")]))
            }).mount(&server).await;
        let config = format!("model = \"gpt-5.4\"\nmodel_provider = \"openai\"\nopenai_base_url = \"{}/v1\"\ncli_auth_credentials_store = \"file\"\ncheck_for_update_on_startup = false\nsuppress_unstable_features_warning = true\nlog_dir = {:?}\n[features]\nsqlite = true\nmemory_tool = true\n[memories]\ngenerate_memories = true\nmin_rollout_idle_hours = 0\n[security]\nversion = 1\nlevel = \"{level}\"\n[projects.{}]\ntrust_level = \"trusted\"\n[tui]\nanimations = false\n",
            server.uri(), home.path().join("logs"), serde_json::to_string(&repo.display().to_string())?);
        fs::write(home.path().join("config.toml"), config)?;
        fs::write(home.path().join("auth.json"), r#"{"OPENAI_API_KEY":"synthetic-memory-test","tokens":null,"last_refresh":null}"#)?;
        let db = codex_state::StateRuntime::init(codex_state::SqliteConfig::new_for_testing(
            codex_utils_absolute_path::AbsolutePathBuf::try_from(home.path())?), "openai".into()).await?;
        db.mark_backfill_complete(None).await?;
        let source = ThreadId::new();
        let timestamp = chrono::Utc::now() - chrono::Duration::hours(2);
        let rollout = home.path().join(format!("rollout-{source}.jsonl"));
        let line = RolloutLine { timestamp: timestamp.to_rfc3339(), ordinal: None,
            item: RolloutItem::ResponseItem(ResponseItem::Message { id: None, role: "user".into(),
                content: vec![ContentItem::InputText { text: CANARY.into() }], phase: None,
                internal_chat_message_metadata_passthrough: None }) };
        fs::write(&rollout, format!("{}\n", serde_json::to_string(&line)?))?;
        let mut metadata = codex_state::ThreadMetadataBuilder::new(source, rollout, timestamp, SessionSource::Cli);
        metadata.cwd = repo.clone();
        db.upsert_thread(&metadata.build("openai")).await?;
        db.set_thread_memory_mode(source, "enabled").await?;
        let tmux = TmuxServer::start(&format!("memory_policy_{level}"))?;
        tmux.register_artifact("codex-tui.log", home.path().join("logs/codex-tui.log"));
        let session = tmux.new_session(SessionSpec::new(level, TerminalSize::new(120, 42),
            CommandSpec::new(&binary).env("CODEX_HOME", home.path()).env("CORBANU_HOME", home.path())
                .env("OPENAI_API_KEY", "synthetic-memory-test").env("RUST_LOG", "trace")
                .arg("--no-alt-screen").arg("-C").arg(&repo)).current_dir(&repo))?;
        let pane = session.primary_pane();
        pane.wait_stable_contains("Corbanu Terminal", TIMEOUT)?;
        pane.send_literal("Run the synthetic foreground fixture.")?;
        pane.wait_stable_contains("Run the synthetic foreground fixture.", TIMEOUT)?;
        pane.send_key(TmuxKey::Enter)?;
        let deadline = tokio::time::Instant::now() + TIMEOUT;
        loop {
            let outputs = db.memories().list_stage1_outputs_for_global(10).await?;
            let log = fs::read_to_string(home.path().join("logs/codex-tui.log")).unwrap_or_default();
            let done = if level == "permissive" { outputs.iter().any(|out| out.thread_id == source) }
                else { log.contains("protected stage-one memory input is unavailable") };
            if done { break; }
            ensure!(tokio::time::Instant::now() < deadline, "memory worker did not finish {level}");
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
        let requests = server.received_requests().await.unwrap();
        let canaries = requests.iter().filter(|r| String::from_utf8_lossy(&r.body).contains(CANARY)).count();
        ensure!(canaries == usize::from(level == "permissive"), "unexpected canary dispatch count {canaries} for {level}");
        if level != "permissive" { ensure!(db.memories().list_stage1_outputs_for_global(10).await?.is_empty(), "denied output persisted"); }
        if let Some(root) = std::env::var_os("CORBANU_MEMORY_EVIDENCE") {
            let root = std::path::PathBuf::from(root); fs::create_dir_all(&root)?;
            fs::write(root.join(format!("{level}-worker.txt")), pane.capture_viewport()?)?;
            fs::write(root.join(format!("{level}-counts.txt")), format!("canary_requests={canaries}\n"))?;
        }
        pane.send_literal("/exit")?;
        pane.wait_stable_contains("/exit", TIMEOUT)?;
        pane.send_key(TmuxKey::Enter)?;
        session.wait_for_exit(TIMEOUT)?;
    }
    Ok(())
}
