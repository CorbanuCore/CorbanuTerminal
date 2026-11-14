//! #646: a Claude Code Orc on Claude Plan joins a `/spawn` crew (owner decision of
//! 2026-10-13, option A: `claude-plan` in `agents.provider_allowlist` also allows
//! Claude Plan panes; with no allowlist the crew stays unrestricted).

use super::*;
use crate::claude_panes::ClaudeProviderProfileKind;
use codex_protocol::crew::CLAUDE_PLAN_PANE_PROVIDER_ID;
use pretty_assertions::assert_eq;

#[tokio::test]
async fn issue_2150_empty_provider_startup_warning_snapshot() {
    let home = tempfile::tempdir().expect("home");
    let repo = tempfile::tempdir().expect("repo");
    std::fs::create_dir(repo.path().join(".git")).expect("git directory");
    std::fs::write(repo.path().join(".git/HEAD"), "ref: refs/heads/main\n").expect("HEAD");
    std::fs::create_dir(repo.path().join(".codex")).expect("project config directory");
    std::fs::write(
        repo.path().join(".codex/config.toml"),
        "[agents]\nprovider_allowlist = ['openai']\n",
    )
    .expect("project config");
    let nested = repo.path().join("nested");
    std::fs::create_dir_all(nested.join(".codex")).expect("nested project config directory");
    std::fs::write(
        nested.join(".codex/config.toml"),
        "[agents]\nprovider_allowlist = ['anthropic']\n",
    )
    .expect("nested project config");
    let project = repo.path().to_string_lossy().replace('\\', "\\\\");
    std::fs::write(
        home.path().join("config.toml"),
        format!("[projects.\"{project}\"]\ntrust_level = 'trusted'\n"),
    )
    .expect("trust project");
    let config = crate::legacy_core::config::ConfigBuilder::default()
        .loader_overrides(codex_config::LoaderOverrides::without_managed_config_for_tests())
        .codex_home(home.path().to_path_buf())
        .harness_overrides(crate::legacy_core::config::ConfigOverrides {
            cwd: Some(nested),
            ..Default::default()
        })
        .build()
        .await
        .expect("config");
    assert_eq!(config.agent_provider_allowlist, Some(Vec::new()));
    let warning = config
        .startup_warnings
        .iter()
        .find(|warning| warning.starts_with("No provider is allowed for spawned agents"))
        .expect("empty policy warning");
    let rendered = lines_to_single_string(
        &history_cell::new_warning_event(warning.clone()).display_lines(/*width*/ 80),
    );
    insta::with_settings!({snapshot_path => "../../snapshots"}, {
        insta::assert_snapshot!("issue_2150_empty_provider_startup_warning", rendered);
    });
}

#[tokio::test]
async fn issue_2150_empty_narrowed_policy_cannot_create_crew_records() {
    let mut app = make_test_app().await;
    app.config.agent_provider_allowlist = Some(Vec::new());
    assert_eq!(app.authorized_spawn_providers(), Vec::<String>::new());
    assert!(
        app.ensure_native_spawn_provider_authorized(Some(OPENAI_PROVIDER_ID))
            .is_err()
    );

    let result = app.prepare_custom_spawn_root(
        "Restricted crew".to_string(),
        crate::dispatch_queue::SavedNativeSpawnRuntime {
            model: "gpt-6-sol".to_string(),
            provider: OPENAI_PROVIDER_ID.to_string(),
            reasoning_effort: None,
        },
    );
    assert!(result.is_err(), "an empty policy must not create a crew");
    assert!(app.spawn_crew.is_none());
}

/// The runtime a Claude Code pane records when it joins a crew, as
/// `create_claude_spawn_pane` and `custom_spawn_crew_runtime_for_node` build it.
fn claude_pane_runtime(provider: &str) -> crate::dispatch_queue::SavedNativeSpawnRuntime {
    crate::dispatch_queue::SavedNativeSpawnRuntime {
        model: ClaudeProviderProfileKind::ClaudePlan
            .profile()
            .provider_model
            .to_string(),
        provider: provider.to_string(),
        reasoning_effort: None,
    }
}

/// Adds a Claude Plan Orc under Main and returns the result.
fn add_claude_plan_orc(app: &mut App) -> color_eyre::Result<()> {
    app.ensure_custom_spawn_root(crate::claude_panes::CODEX_MAIN_PANE_ID)?;
    app.record_custom_spawn_member(
        "pane:claude-orc",
        crate::claude_panes::CODEX_MAIN_PANE_ID,
        crate::spawn_orchestration::SpawnRole::Orc,
        "Snaga".to_string(),
        claude_pane_runtime("external-claude:claudeplan"),
    )
}

fn assert_crew_ready_and_valid(app: &App) {
    let crew = app.spawn_crew.as_ref().expect("custom crew");
    assert!(
        matches!(crew.status, crate::crew_state::CrewCreationStatus::Ready),
        "crew must stay ready, got {:?}",
        crew.status
    );
    crew.spec
        .validate()
        .expect("a restart must not find the crew invalid");
}

#[test]
fn issue_646_claude_plan_pane_records_the_aliased_provider() {
    assert_eq!(
        ClaudeProviderProfileKind::ClaudePlan.crew_runtime_provider_id(),
        CLAUDE_PLAN_PANE_PROVIDER_ID
    );
}

#[tokio::test]
async fn issue_646_claude_plan_orc_joins_a_crew_without_an_allowlist() {
    let mut app = make_test_app().await;
    assert!(app.config.agent_provider_allowlist.is_none());

    add_claude_plan_orc(&mut app).expect("a Claude Plan Orc must join an unrestricted crew");
    assert_crew_ready_and_valid(&app);
}

#[tokio::test]
async fn issue_646_claude_plan_pane_can_be_the_crew_root_without_an_allowlist() {
    let mut app = make_test_app().await;
    let pane_id = app
        .claude_panes
        .create_pane_without_vault_for_test(
            ClaudeProviderProfileKind::ClaudePlan,
            app.config.cwd.to_path_buf(),
            app.config.codex_home.as_ref(),
        )
        .expect("create Claude pane");

    app.ensure_custom_spawn_root(&crate::spawn_orchestration::pane_node_id(&pane_id))
        .expect("a Claude Plan pane must be able to root an unrestricted crew");
    assert_crew_ready_and_valid(&app);
}

/// Without an allowlist the crew ceiling lists every Claude Code pane, not only
/// those whose provider is also configured natively (here Z.AI's pane).
#[tokio::test]
async fn issue_646_any_claude_code_pane_joins_a_crew_without_an_allowlist() {
    let mut app = make_test_app().await;
    let authorized = app.authorized_spawn_providers();
    for profile in ClaudeProviderProfileKind::creation_options() {
        assert!(
            authorized.contains(&profile.crew_runtime_provider_id()),
            "{profile:?} missing from {authorized:?}"
        );
    }
    app.ensure_custom_spawn_root(crate::claude_panes::CODEX_MAIN_PANE_ID)
        .expect("bind custom root");
    app.record_custom_spawn_member(
        "pane:claude-zai-orc",
        crate::claude_panes::CODEX_MAIN_PANE_ID,
        crate::spawn_orchestration::SpawnRole::Orc,
        "Grishnakh".to_string(),
        claude_pane_runtime(&ClaudeProviderProfileKind::ZaiGlm52.crew_runtime_provider_id()),
    )
    .expect("a Z.AI Claude Code Orc must join an unrestricted crew");
    assert_crew_ready_and_valid(&app);
}

#[tokio::test]
async fn issue_646_claude_plan_in_the_allowlist_allows_claude_plan_panes() {
    let mut app = make_test_app().await;
    app.config.agent_provider_allowlist = Some(vec![
        app.config.model_provider_id.clone(),
        CLAUDE_PLAN_PROVIDER_ID.to_string(),
    ]);

    add_claude_plan_orc(&mut app).expect("`claude-plan` must allow a Claude Plan Orc");
    assert_crew_ready_and_valid(&app);
}

#[tokio::test]
async fn issue_646_an_allowlist_without_claude_plan_still_refuses_claude_plan_panes() {
    let mut app = make_test_app().await;
    app.config.agent_provider_allowlist = Some(vec![app.config.model_provider_id.clone()]);

    let error = add_claude_plan_orc(&mut app).expect_err("not authorized");
    assert!(
        error.to_string().contains("external-claude:claudeplan"),
        "unexpected error: {error}"
    );
}

#[tokio::test]
async fn issue_646_claude_plan_does_not_allow_other_claude_code_panes() {
    let mut app = make_test_app().await;
    app.config.agent_provider_allowlist = Some(vec![
        CLAUDE_PLAN_PROVIDER_ID.to_string(),
        app.config.model_provider_id.clone(),
    ]);
    app.ensure_custom_spawn_root(crate::claude_panes::CODEX_MAIN_PANE_ID)
        .expect("bind custom root");

    let error = app
        .record_custom_spawn_member(
            "pane:claude-zai-orc",
            crate::claude_panes::CODEX_MAIN_PANE_ID,
            crate::spawn_orchestration::SpawnRole::Orc,
            "Grishnakh".to_string(),
            claude_pane_runtime(&ClaudeProviderProfileKind::ZaiGlm52.crew_runtime_provider_id()),
        )
        .expect_err("a Z.AI Claude Code pane needs its own entry");
    assert!(
        error.to_string().contains("external-claude:zaiglm52"),
        "unexpected error: {error}"
    );
}
