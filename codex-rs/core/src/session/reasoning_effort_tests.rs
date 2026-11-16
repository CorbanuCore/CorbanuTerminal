//! Request-effort pins across initial replay and compaction lookups.

use super::RequestEffortUsage;
use crate::session::step_settings::ResolvedStepSettings;
use crate::session::step_settings::StepSettings;
use crate::session::tests::make_session_and_context_with_auth_and_config_and_rx;
use codex_features::Feature;
use codex_history::InitialHistory;
use codex_history::ResumedHistory;
use codex_login::CodexAuth;
use codex_protocol::ThreadId;
use codex_protocol::openai_models::ReasoningEffort;
use pretty_assertions::assert_eq;
use std::sync::Arc;
use test_case::test_case;

#[test_case(InitialHistory::Forked(Vec::new()); "fork")]
#[test_case(InitialHistory::Resumed(ResumedHistory {
history_revision: None,
    conversation_id: ThreadId::default(),
    history: Arc::new(Vec::new()),
    rollout_path: None,
}); "resume")]
#[tokio::test]
async fn initial_replay_preserves_prewarmed_effort(history: InitialHistory) {
    let (session, turn_context, _events) = make_session_and_context_with_auth_and_config_and_rx(
        CodexAuth::from_api_key("Test API Key"),
        Vec::new(),
        |config| {
            config
                .features
                .enable(Feature::ReasoningEffortOverride)
                .unwrap();
        },
    )
    .await;
    let mut model_info = Arc::clone(&turn_context.initial_settings.model_info);
    Arc::make_mut(&mut model_info).supports_reasoning_effort_updates = true;
    let mut selected = turn_context.initial_settings.selected().clone();
    selected.collaboration_mode.settings.reasoning_effort = Some(ReasoningEffort::Medium);
    let prewarm_settings = ResolvedStepSettings::new(
        Arc::new(selected.clone()),
        Arc::clone(&model_info),
        &codex_features::Features::default(),
    );
    // Force the ordering where prewarm pins its request before initial replay finishes.
    assert_eq!(
        session
            .reasoning_effort_for_request(&prewarm_settings, RequestEffortUsage::Sampling)
            .await,
        Some(ReasoningEffort::Medium),
    );
    session.record_initial_history(history).await;

    selected.collaboration_mode.settings.reasoning_effort = Some(ReasoningEffort::High);
    let turn_settings = ResolvedStepSettings::new(
        Arc::new(selected),
        model_info,
        &codex_features::Features::default(),
    );
    assert_eq!(
        session
            .reasoning_effort_for_request(&turn_settings, RequestEffortUsage::Sampling)
            .await,
        Some(ReasoningEffort::Medium),
    );
}

#[tokio::test]
async fn compaction_effort_lookup_preserves_pin_for_fallback_models() {
    let (session, turn_context, _events) = make_session_and_context_with_auth_and_config_and_rx(
        CodexAuth::from_api_key("Test API Key"),
        Vec::new(),
        |config| {
            config
                .features
                .enable(Feature::ReasoningEffortOverride)
                .unwrap();
        },
    )
    .await;
    session
        .state
        .lock()
        .await
        .reasoning_effort_pin
        .pin("original", ReasoningEffort::Low);
    let mut settings = (*turn_context.initial_settings).clone();
    let effort = ReasoningEffort::Medium;
    let model = Arc::make_mut(&mut settings.model_info);
    model.slug = "fallback".to_string();
    model.supports_reasoning_effort_updates = true;
    model.default_reasoning_level = Some(effort.clone());

    assert_eq!(
        session
            .reasoning_effort_for_request(&settings, RequestEffortUsage::Compaction)
            .await,
        Some(effort)
    );
    assert_eq!(
        session
            .state
            .lock()
            .await
            .reasoning_effort_pin
            .get("original"),
        Some(ReasoningEffort::Low)
    );
}

#[tokio::test]
async fn unsupported_model_compaction_uses_selected_effort_without_mutating_pin() {
    let (session, turn_context, _events) = make_session_and_context_with_auth_and_config_and_rx(
        CodexAuth::from_api_key("Test API Key"),
        Vec::new(),
        |config| {
            config
                .features
                .enable(Feature::ReasoningEffortOverride)
                .unwrap();
            config.model_reasoning_effort = Some(ReasoningEffort::High);
        },
    )
    .await;
    let mut settings = (*turn_context.initial_settings).clone();
    Arc::make_mut(&mut settings.model_info).supports_reasoning_effort_updates = false;
    session
        .state
        .lock()
        .await
        .reasoning_effort_pin
        .pin(&settings.model_info.slug, ReasoningEffort::Low);

    assert_eq!(
        session
            .reasoning_effort_for_request(&settings, RequestEffortUsage::Compaction)
            .await,
        Some(ReasoningEffort::High),
    );
    assert_eq!(
        session
            .state
            .lock()
            .await
            .reasoning_effort_pin
            .get(&settings.model_info.slug),
        Some(ReasoningEffort::Low),
    );
    // Sampling retires the old baseline, so a later supported request starts afresh.
    assert_eq!(
        session
            .reasoning_effort_for_request(&settings, RequestEffortUsage::Sampling)
            .await,
        Some(ReasoningEffort::High),
    );
    Arc::make_mut(&mut settings.model_info).supports_reasoning_effort_updates = true;
    assert_eq!(
        session
            .reasoning_effort_for_request(&settings, RequestEffortUsage::Sampling)
            .await,
        Some(ReasoningEffort::High),
    );
}

/// The effort `spawn_agent` names for its inherited runtime, for one turn at `effort`.
async fn spawn_agent_inherited_effort(effort_updates: bool, effort: ReasoningEffort) -> String {
    let mut config = crate::config::test_config().await;
    if effort_updates {
        config
            .features
            .enable(Feature::ReasoningEffortOverride)
            .unwrap();
    } else {
        config
            .features
            .disable(Feature::ReasoningEffortOverride)
            .unwrap();
    }
    let (session, mut turn) =
        crate::session::tests::make_session_and_context_for_config(config).await;
    let mut model_info = Arc::clone(&turn.initial_settings.model_info);
    Arc::make_mut(&mut model_info).supports_reasoning_effort_updates = true;
    let mut selected: StepSettings = turn.initial_settings.selected().clone();
    selected.collaboration_mode.settings.reasoning_effort = Some(effort);
    let settings = Arc::new(ResolvedStepSettings::new(
        Arc::new(selected),
        model_info,
        &codex_features::Features::default(),
    ));
    turn.initial_settings = Arc::clone(&settings);
    turn.next_step_settings.store(settings);
    turn.multi_agent_version = codex_protocol::protocol::MultiAgentVersion::V1;
    let turn = Arc::new(turn);
    let step_context = crate::session::step_context::StepContext::for_test(Arc::clone(&turn));
    let dynamic_tools = session.dynamic_tools().await;
    let router = crate::tools::spec_plan::build_tool_router(
        &session,
        step_context.turn.as_ref(),
        &step_context.settings.model_info,
        &step_context.environments,
        &step_context.mcp,
        /*apps_enabled*/ false,
        &turn.extension_data,
        /*tool_suggest_candidates*/ None,
        dynamic_tools.as_ref(),
    )
    .expect("build tool router");
    let specs = serde_json::to_string(&*router.model_visible_specs()).expect("specs");
    let start = specs
        .find("Current inherited runtime:")
        .expect("spawn_agent describes its inherited runtime");
    let effort = start + specs[start..].find("; effort ").expect("effort");
    let end = effort + specs[effort..].find('.').expect("sentence end");
    specs[effort..end].to_string()
}

/// #1532 (`thread_sample`): with effort updates, an effort-only turn change reaches the model as
/// a configuration update, so `spawn_agent` must not name the turn's effort: its definition (and
/// the prompt-cache prefix) then stays the same from turn to turn.
#[tokio::test]
async fn spawn_agent_description_does_not_follow_effort_only_changes() {
    assert_eq!(
        spawn_agent_inherited_effort(/*effort_updates*/ true, ReasoningEffort::High).await,
        "; effort as in the current turn"
    );
    assert_eq!(
        spawn_agent_inherited_effort(/*effort_updates*/ true, ReasoningEffort::Medium).await,
        "; effort as in the current turn"
    );
    // Without effort updates each request carries its own effort; the text names it, as before.
    assert_eq!(
        spawn_agent_inherited_effort(/*effort_updates*/ false, ReasoningEffort::Medium).await,
        "; effort medium"
    );
}
