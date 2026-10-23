//! #646: a Claude Code Orc on Claude Plan joins a `/spawn` crew (owner decision of
//! 2026-10-13, option A: `claude-plan` in `agents.provider_allowlist` also allows
//! Claude Plan panes; with no allowlist the crew stays unrestricted).

use super::*;
use crate::claude_panes::ClaudeProviderProfileKind;
use codex_protocol::crew::CLAUDE_PLAN_PANE_PROVIDER_ID;
use pretty_assertions::assert_eq;

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
