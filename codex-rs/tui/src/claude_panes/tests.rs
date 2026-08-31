use std::collections::BTreeMap;
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;
use std::sync::Arc;
use std::time::Duration;
use std::time::Instant;

use codex_app_server_protocol::UserInput;
use codex_model_provider_info::AMBIENT_DEFAULT_MODEL;
use codex_model_provider_info::AMBIENT_KIMI_K2_7_CODE_MODEL;
use pretty_assertions::assert_eq;
use serde_json::Value;
use serde_json::json;
use sha2::Digest;
use sha2::Sha256;
use tokio::io::AsyncReadExt;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpListener;
use tokio::net::TcpStream;
use tokio::sync::Mutex;
use tokio_util::sync::CancellationToken;
use uuid::Uuid;

use crate::app_command::AppCommand;
use crate::spawn_orchestration::SpawnRole;

use super::app_integration::new_pane_items;
use super::bridge::ambient_retry_after_delay;
use super::bridge::find_header_end;
use super::bridge::handle_anthropic_passthrough_bridge_connection;
use super::bridge_translate::ambient_chat_messages_from_claude_request;
use super::bridge_translate::ambient_chat_tools_from_claude_request;
use super::bridge_translate::anthropic_stream_error_event;
use super::bridge_translate::anthropic_stream_start_event;
use super::bridge_translate::anthropic_stream_stop_event;
use super::bridge_translate::anthropic_tool_use_response;
use super::bridge_translate::bridge_tool_calls_from_ambient_response;
use super::command_plan::allowed_provider_vault_label;
use super::command_plan::build_claude_command_plan;
use super::command_plan::claude_pane_title;
use super::command_plan::compose_claude_pane_prompt;
use super::command_plan::prompt_from_user_turn;
use super::command_plan::settings_json_with_base_url;
#[cfg(unix)]
use super::execution::ClaudeSecretRedactor;
use super::execution::failed_turn_output;
use super::execution::partial_failed_turn_output;
use super::execution::run_claude_command_plan;
use super::execution::stop_claude_child;
use super::execution::write_turn_audit;
use super::output_parse::parse_claude_output;
use super::output_parse::parsed_from_value;
use super::pane::ClaudeCommandMode;
use super::pane::ClaudePane;
use super::pane::ClaudePaneLiveTurn;
use super::pane::ClaudePaneStatus;
use super::pane::ClaudePaneTurnStatus;
use super::pane::ClaudePaneUsageStatus;
use super::pane::PaneLayoutState;
use super::persistence::CLAUDE_PANE_METADATA_FILE;
use super::persistence::current_unix_ms_i64;
use super::progress::progress_from_claude_value;
use super::progress::progress_status_text;
use super::progress::progresses_from_claude_value;
use super::progress::usage_status_from_summary;
use super::progress_summarize::summarize_tool_call_input;
use super::provider::ClaudeProviderProfileKind;
use super::registry::CODEX_MAIN_PANE_ID;
use super::registry::ClaudePaneRegistry;
use super::registry::PANE_LAYOUT_VERSION;
use super::registry::load_pane_layout;
use super::registry::persist_pane_layout;
use super::smoke_workflows::smoke_provider_profile;
use super::turn_types::ClaudeBridgeKind;
#[cfg(unix)]
use super::turn_types::ClaudeBridgePlan;
use super::turn_types::ClaudeCommandPlan;
use super::turn_types::ClaudePaneReasoningEvent;
use super::turn_types::ClaudePaneToolEvent;
use super::turn_types::ClaudePaneTurnOutput;
use super::turn_types::ClaudePaneTurnProgress;

use std::path::PathBuf;
use tokio::process::Command;

// Re-export items the test helpers use with their original unqualified names.

fn pane(profile: ClaudeProviderProfileKind) -> (tempfile::TempDir, ClaudePane) {
    let dir = tempfile::tempdir().expect("tempdir");
    let id = Uuid::new_v4().to_string();
    let artifact_dir = dir.path().join("panes").join(&id);
    std::fs::create_dir_all(&artifact_dir).expect("artifact dir");
    (
        dir,
        ClaudePane {
            id: format!("claude-{id}"),
            title: profile.profile().title.to_string(),
            profile,
            spawn_role: None,
            spawn_nickname: None,
            spawn_thread_id: None,
            cwd: std::env::current_dir().expect("cwd"),
            claude_session_id: None,
            status: ClaudePaneStatus::Idle,
            latest_usage_summary: None,
            latest_usage_status: None,
            latest_turn_status: None,
            latest_audit_path: None,
            latest_task_message: None,
            latest_result_message: None,
            artifact_dir,
            live_turn: None,
            cancel_token: None,
            lock: Arc::new(Mutex::new(())),
            next_turn_index: 1,
        },
    )
}

async fn read_http_request(stream: &mut TcpStream) -> Vec<u8> {
    let mut request = Vec::new();
    let mut chunk = [0_u8; 4096];
    let mut expected_len = None;
    loop {
        let read = stream.read(&mut chunk).await.expect("read HTTP request");
        if read == 0 {
            break;
        }
        request.extend_from_slice(&chunk[..read]);
        if expected_len.is_none()
            && let Some(header_end) = find_header_end(&request)
        {
            let headers = String::from_utf8_lossy(&request[..header_end]);
            let content_length = headers
                .lines()
                .find_map(|line| {
                    let (name, value) = line.split_once(':')?;
                    name.eq_ignore_ascii_case("content-length")
                        .then(|| value.trim().parse::<usize>().ok())
                        .flatten()
                })
                .unwrap_or(0);
            expected_len = Some(header_end + 4 + content_length);
        }
        if expected_len.is_some_and(|expected_len| request.len() >= expected_len) {
            break;
        }
    }
    request
}

#[test]
fn registry_restores_persisted_pane_metadata() {
    let codex_home = tempfile::tempdir().expect("codex home");
    let cwd = std::env::current_dir().expect("cwd");
    let mut registry = ClaudePaneRegistry::new();
    let pane_id = registry
        .create_pane_with_role(
            ClaudeProviderProfileKind::ClaudePlan,
            cwd.clone(),
            codex_home.path(),
            Some(SpawnRole::Troll),
            Some("Burzum".to_string()),
        )
        .expect("create pane");
    let pane = registry
        .panes()
        .iter()
        .find(|pane| pane.id == pane_id)
        .expect("pane");
    assert!(pane.artifact_dir.join(CLAUDE_PANE_METADATA_FILE).exists());

    let restored = ClaudePaneRegistry::restore_from_disk(codex_home.path(), /*layout*/ None);
    assert!(
        restored.panes().is_empty(),
        "fresh starts should not restore persisted panes without an explicit layout"
    );

    let layout = PaneLayoutState {
        version: PANE_LAYOUT_VERSION,
        codex_thread_id: Some("019f0657-1d67-7103-9d65-89e71587347d".to_string()),
        active_user_pane_id: None,
        spawn_nazgul_pane_id: None,
        claude_pane_ids: vec![pane_id.clone()],
        spawn_parent_by_node: BTreeMap::new(),
        ..Default::default()
    };
    let restored = ClaudePaneRegistry::restore_from_disk(codex_home.path(), Some(&layout));
    assert_eq!(restored.panes().len(), 1);
    let restored_pane = &restored.panes()[0];
    assert_eq!(restored_pane.id, pane_id);
    assert_eq!(restored_pane.profile, ClaudeProviderProfileKind::ClaudePlan);
    assert_eq!(restored_pane.spawn_role, Some(SpawnRole::Troll));
    assert_eq!(restored_pane.spawn_nickname.as_deref(), Some("Burzum"));
    assert_eq!(restored_pane.cwd, cwd);
    assert_eq!(restored.active_user_pane_id(), CODEX_MAIN_PANE_ID);

    let unlisted_pane_id = registry
        .create_pane_with_role(
            ClaudeProviderProfileKind::ClaudePlan,
            cwd,
            codex_home.path(),
            Some(SpawnRole::Orc),
            Some("Snaga".to_string()),
        )
        .expect("create unlisted pane");
    let layout = PaneLayoutState {
        version: PANE_LAYOUT_VERSION,
        codex_thread_id: Some("019f0657-1d67-7103-9d65-89e71587347d".to_string()),
        active_user_pane_id: Some(pane_id.clone()),
        spawn_nazgul_pane_id: None,
        claude_pane_ids: vec![pane_id.clone()],
        spawn_parent_by_node: BTreeMap::new(),
        ..Default::default()
    };
    let restored = ClaudePaneRegistry::restore_from_disk(codex_home.path(), Some(&layout));
    assert_eq!(restored.active_user_pane_id(), pane_id);
    assert_eq!(restored.panes().len(), 1);
    assert!(restored.panes().iter().any(|pane| pane.id == pane_id));
    assert!(
        !restored
            .panes()
            .iter()
            .any(|pane| pane.id == unlisted_pane_id)
    );
}

#[test]
fn registry_removes_idle_operator_pane_artifacts_and_restoration_membership() {
    let codex_home = tempfile::tempdir().expect("codex home");
    let mut registry = ClaudePaneRegistry::new();
    let pane_id = registry
        .create_pane(
            ClaudeProviderProfileKind::ClaudePlan,
            std::env::current_dir().expect("cwd"),
            codex_home.path(),
        )
        .expect("create pane");
    let artifact_dir = codex_home.path().join("panes").join(&pane_id);
    std::fs::write(
        artifact_dir.join("turn-0001.jsonl"),
        "preserved until delete",
    )
    .expect("write artifact");

    let removed = registry
        .remove_operator_pane(&pane_id, codex_home.path())
        .expect("remove pane");

    assert!(!removed.interrupted_running_turn);
    assert_eq!(registry.active_user_pane_id(), CODEX_MAIN_PANE_ID);
    assert!(registry.panes().is_empty());
    assert!(!artifact_dir.exists());
    let layout = PaneLayoutState {
        version: PANE_LAYOUT_VERSION,
        claude_pane_ids: vec![pane_id],
        ..Default::default()
    };
    assert!(
        ClaudePaneRegistry::restore_from_disk(codex_home.path(), Some(&layout))
            .panes()
            .is_empty()
    );
}

#[test]
fn registry_cancels_running_operator_pane_before_removing_artifacts() {
    let codex_home = tempfile::tempdir().expect("codex home");
    let mut registry = ClaudePaneRegistry::new();
    let pane_id = registry
        .create_pane(
            ClaudeProviderProfileKind::ClaudePlan,
            std::env::current_dir().expect("cwd"),
            codex_home.path(),
        )
        .expect("create pane");
    let cancel_token = CancellationToken::new();
    let pane = registry
        .panes
        .iter_mut()
        .find(|pane| pane.id == pane_id)
        .expect("pane");
    pane.status = ClaudePaneStatus::Running;
    pane.cancel_token = Some(cancel_token.clone());

    let removed = registry
        .remove_operator_pane(&pane_id, codex_home.path())
        .expect("remove running pane");

    assert!(removed.interrupted_running_turn);
    assert!(cancel_token.is_cancelled());
    assert!(registry.panes().is_empty());
    assert!(!codex_home.path().join("panes").join(&pane_id).exists());
}

#[test]
fn registry_refuses_independent_managed_claude_pane_removal() {
    let codex_home = tempfile::tempdir().expect("codex home");
    let mut registry = ClaudePaneRegistry::new();
    let pane_id = registry
        .create_pane_with_role(
            ClaudeProviderProfileKind::ClaudePlan,
            std::env::current_dir().expect("cwd"),
            codex_home.path(),
            Some(SpawnRole::Orc),
            Some("Snaga".to_string()),
        )
        .expect("create managed pane");
    let artifact_dir = codex_home.path().join("panes").join(&pane_id);

    let error = registry
        .remove_operator_pane(&pane_id, codex_home.path())
        .expect_err("managed pane must be protected");

    assert!(error.to_string().contains("whole-crew lifecycle"));
    assert!(artifact_dir.exists());
    assert_eq!(registry.panes().len(), 1);
}

#[test]
fn registry_removes_running_managed_pane_only_through_crew_boundary() {
    let codex_home = tempfile::tempdir().expect("codex home");
    let mut registry = ClaudePaneRegistry::new();
    let pane_id = registry
        .create_pane_with_role(
            ClaudeProviderProfileKind::ClaudePlan,
            std::env::current_dir().expect("cwd"),
            codex_home.path(),
            Some(SpawnRole::Troll),
            Some("Burzum".to_string()),
        )
        .expect("create managed pane");
    let cancel_token = CancellationToken::new();
    let pane = registry
        .panes
        .iter_mut()
        .find(|pane| pane.id == pane_id)
        .expect("pane");
    pane.status = ClaudePaneStatus::Running;
    pane.cancel_token = Some(cancel_token.clone());

    let removed = registry
        .remove_managed_crew_pane(&pane_id, codex_home.path())
        .expect("whole-crew boundary removes pane");

    assert!(removed.interrupted_running_turn);
    assert!(cancel_token.is_cancelled());
    assert!(registry.panes().is_empty());
    assert!(!codex_home.path().join("panes").join(&pane_id).exists());
}

#[test]
fn registry_crew_boundary_refuses_operator_created_pane() {
    let codex_home = tempfile::tempdir().expect("codex home");
    let mut registry = ClaudePaneRegistry::new();
    let pane_id = registry
        .create_pane(
            ClaudeProviderProfileKind::ClaudePlan,
            std::env::current_dir().expect("cwd"),
            codex_home.path(),
        )
        .expect("create operator pane");

    let error = registry
        .remove_managed_crew_pane(&pane_id, codex_home.path())
        .expect_err("crew boundary must not remove operator pane");

    assert!(error.to_string().contains("not owned by the managed crew"));
    assert!(codex_home.path().join("panes").join(&pane_id).exists());
    assert_eq!(registry.panes().len(), 1);
}

#[test]
fn registry_restores_legacy_pane_from_latest_audit() {
    let codex_home = tempfile::tempdir().expect("codex home");
    let pane_id = "claude-legacy-pane";
    let artifact_dir = codex_home.path().join("panes").join(pane_id);
    std::fs::create_dir_all(&artifact_dir).expect("artifact dir");
    let artifact_path = artifact_dir.join("turn-0002.jsonl");
    let audit_path = artifact_dir.join("turn-0002.audit.json");
    std::fs::write(
        &artifact_path,
        serde_json::json!({
            "type": "result",
            "subtype": "success",
            "result": "legacy pane result text",
            "session_id": "11111111-2222-4333-8444-555555555555"
        })
        .to_string(),
    )
    .expect("artifact");
    std::fs::write(artifact_dir.join("turn-0003.jsonl"), "{}\n").expect("next artifact");
    std::fs::write(
        &audit_path,
        serde_json::json!({
            "pane_id": pane_id,
            "pane_title": "Claude Code Snaga [orc] - GLM 5.2 Fast Vercel",
            "provider": "Claude Code - GLM 5.2 Fast Vercel",
            "model": "zai/glm-5.2-fast",
            "session_id": "11111111-2222-4333-8444-555555555555",
            "turn_index": 2,
            "command_mode": "resume",
            "max_turns": null,
            "artifact_path": artifact_path,
            "audit_path": audit_path,
            "timeout_ms": null,
            "started_at_unix_ms": current_unix_ms_i64(),
            "ended_at_unix_ms": current_unix_ms_i64(),
            "last_progress_elapsed_ms": null,
            "duration_ms": 123,
            "usage": null,
            "usage_status": "untrusted",
            "terminal_reason": null,
            "status": "success",
            "error_summary": null,
            "reasoning_event_count": 0,
            "reasoning_events": [],
            "tool_use_count": 0,
            "tool_names": [],
            "tool_events": []
        })
        .to_string(),
    )
    .expect("audit");

    let layout = PaneLayoutState {
        version: PANE_LAYOUT_VERSION,
        codex_thread_id: Some("019f0657-1d67-7103-9d65-89e71587347d".to_string()),
        active_user_pane_id: None,
        spawn_nazgul_pane_id: None,
        claude_pane_ids: vec![pane_id.to_string()],
        spawn_parent_by_node: BTreeMap::new(),
        ..Default::default()
    };
    let restored = ClaudePaneRegistry::restore_from_disk(codex_home.path(), Some(&layout));
    assert_eq!(restored.panes().len(), 1);
    let pane = &restored.panes()[0];
    assert_eq!(pane.id, pane_id);
    assert_eq!(pane.profile, ClaudeProviderProfileKind::VercelGlm52Fast);
    assert_eq!(pane.spawn_role, Some(SpawnRole::Orc));
    assert_eq!(pane.spawn_nickname.as_deref(), Some("Snaga"));
    assert_eq!(
        pane.claude_session_id.as_deref(),
        Some("11111111-2222-4333-8444-555555555555")
    );
    assert_eq!(pane.latest_turn_status, Some(ClaudePaneTurnStatus::Success));
    assert_eq!(
        pane.latest_usage_status,
        Some(ClaudePaneUsageStatus::Untrusted)
    );
    assert_eq!(
        pane.latest_result_message.as_deref(),
        Some("legacy pane result text")
    );
    assert_eq!(pane.next_turn_index, 4);
}

#[test]
fn registry_restores_session_id_from_artifact_when_interrupted_audit_lost_it() {
    let codex_home = tempfile::tempdir().expect("codex home");
    let pane_id = "claude-interrupted-pane";
    let artifact_dir = codex_home.path().join("panes").join(pane_id);
    std::fs::create_dir_all(&artifact_dir).expect("artifact dir");
    let artifact_path = artifact_dir.join("turn-0001.jsonl");
    let audit_path = artifact_dir.join("turn-0001.audit.json");
    std::fs::write(
        &artifact_path,
        r#"{"type":"system","subtype":"init","session_id":"33333333-3333-4333-8333-333333333333"}
{"type":"assistant","message":{"content":[{"type":"text","text":"working"}]},"session_id":"33333333-3333-4333-8333-333333333333"}"#,
    )
    .expect("artifact");
    std::fs::write(
        &audit_path,
        serde_json::json!({
            "pane_id": pane_id,
            "pane_title": "Claude Code - GLM 5.2 Fast Vercel",
            "provider": "Claude Code - GLM 5.2 Fast Vercel",
            "model": "zai/glm-5.2-fast",
            "session_id": null,
            "turn_index": 1,
            "command_mode": "new-session",
            "max_turns": null,
            "artifact_path": artifact_path,
            "audit_path": audit_path,
            "timeout_ms": null,
            "started_at_unix_ms": current_unix_ms_i64(),
            "ended_at_unix_ms": current_unix_ms_i64(),
            "last_progress_elapsed_ms": null,
            "duration_ms": 123,
            "usage": null,
            "usage_status": "missing",
            "terminal_reason": "interrupted",
            "status": "interrupted",
            "error_summary": "Claude pane turn interrupted by user.",
            "reasoning_event_count": 0,
            "reasoning_events": [],
            "tool_use_count": 0,
            "tool_names": [],
            "tool_events": []
        })
        .to_string(),
    )
    .expect("audit");

    let layout = PaneLayoutState {
        version: PANE_LAYOUT_VERSION,
        codex_thread_id: Some("019f0657-1d67-7103-9d65-89e71587347d".to_string()),
        active_user_pane_id: None,
        spawn_nazgul_pane_id: None,
        claude_pane_ids: vec![pane_id.to_string()],
        spawn_parent_by_node: BTreeMap::new(),
        ..Default::default()
    };
    let mut restored = ClaudePaneRegistry::restore_from_disk(codex_home.path(), Some(&layout));
    assert_eq!(restored.panes().len(), 1);
    let pane = &restored.panes()[0];
    assert_eq!(
        pane.claude_session_id.as_deref(),
        Some("33333333-3333-4333-8333-333333333333")
    );

    codex_vault::Vault::new(codex_home.path().to_path_buf())
        .add(codex_vault::AddCredential {
            label: "provider/ai_gateway_api_key".to_string(),
            credential_type: codex_vault::CredentialType::ApiKey,
            provider: Some("vercel".to_string()),
            notes: None,
            revocation_notes: None,
            secret: "vercel-test-key".to_string(),
        })
        .expect("store test Vercel key");

    let next = restored
        .prepare_turn(pane_id, "continue".to_string(), codex_home.path())
        .expect("next turn");
    assert!(
        next.plan.args.windows(2).any(|window| {
            window[0] == "--resume" && window[1] == "33333333-3333-4333-8333-333333333333"
        }),
        "restored pane should resume the session recovered from the JSONL artifact"
    );
}

#[test]
fn registry_restores_legacy_claude_plan_pane_from_old_audit_title() {
    let codex_home = tempfile::tempdir().expect("codex home");
    let pane_id = "claude-legacy-plan-pane";
    let artifact_dir = codex_home.path().join("panes").join(pane_id);
    std::fs::create_dir_all(&artifact_dir).expect("artifact dir");
    let artifact_path = artifact_dir.join("turn-0001.jsonl");
    let audit_path = artifact_dir.join("turn-0001.audit.json");
    std::fs::write(
        &artifact_path,
        serde_json::json!({
            "type": "result",
            "subtype": "success",
            "result": "legacy Claude Plan result",
            "session_id": "22222222-3333-4444-8555-666666666666"
        })
        .to_string(),
    )
    .expect("artifact");
    std::fs::write(
        &audit_path,
        serde_json::json!({
            "pane_id": pane_id,
            "pane_title": "Claude Code Burzum [troll] - Claude Plan",
            "provider": "Claude Code - Claude Plan",
            "model": "sonnet",
            "session_id": "22222222-3333-4444-8555-666666666666",
            "turn_index": 1,
            "command_mode": "new-session",
            "max_turns": null,
            "artifact_path": artifact_path,
            "audit_path": audit_path,
            "timeout_ms": null,
            "started_at_unix_ms": current_unix_ms_i64(),
            "ended_at_unix_ms": current_unix_ms_i64(),
            "last_progress_elapsed_ms": null,
            "duration_ms": 123,
            "usage": null,
            "usage_status": "untrusted",
            "terminal_reason": null,
            "status": "success",
            "error_summary": null,
            "reasoning_event_count": 0,
            "reasoning_events": [],
            "tool_use_count": 0,
            "tool_names": [],
            "tool_events": []
        })
        .to_string(),
    )
    .expect("audit");

    let layout = PaneLayoutState {
        version: PANE_LAYOUT_VERSION,
        codex_thread_id: Some("019f0657-1d67-7103-9d65-89e71587347d".to_string()),
        active_user_pane_id: None,
        spawn_nazgul_pane_id: None,
        claude_pane_ids: vec![pane_id.to_string()],
        spawn_parent_by_node: BTreeMap::new(),
        ..Default::default()
    };
    let restored = ClaudePaneRegistry::restore_from_disk(codex_home.path(), Some(&layout));
    assert_eq!(restored.panes().len(), 1);
    let pane = &restored.panes()[0];
    assert_eq!(pane.id, pane_id);
    assert_eq!(pane.profile, ClaudeProviderProfileKind::ClaudePlan);
    assert_eq!(pane.spawn_role, Some(SpawnRole::Troll));
    assert_eq!(pane.spawn_nickname.as_deref(), Some("Burzum"));
    assert_eq!(
        pane.claude_session_id.as_deref(),
        Some("22222222-3333-4444-8555-666666666666")
    );
    assert_eq!(
        pane.latest_result_message.as_deref(),
        Some("legacy Claude Plan result")
    );
    assert_eq!(pane.next_turn_index, 2);
}

#[test]
fn pane_layout_persistence_round_trips_root_binding_and_parent_map() {
    let codex_home = tempfile::tempdir().expect("codex home");
    let mut parents = BTreeMap::new();
    parents.insert("pane:orc".to_string(), "pane:troll".to_string());
    let mut whips = BTreeMap::new();
    whips.insert(
        "whip-3".to_string(),
        crate::orchestrate::Whip {
            id: "whip-3".to_string(),
            holder: Some("pane:troll".to_string()),
            target: "pane:orc".to_string(),
            instructions: "keep-going".to_string(),
            mode: crate::orchestrate::WhipMode::Review,
            kind: crate::orchestrate::WhipKind::LegacyNudge,
            expires_at: None,
            max_fires: 20,
            cooldown_s: 60,
            stop_marker: "WHIP_DONE".to_string(),
            fires: 2,
            last_fire_utc: None,
            state: crate::orchestrate::WhipState::Armed,
            last_idle_generation_fired: Some(4),
            empty_output_fires: 0,
            consecutive_failed_turns: 0,
            assignment_unreachable_since_utc: None,
            pending_review_fire: None,
            ignored_review_fires: 0,
            expiry_notified: false,
            last_target_output: None,
            last_dispatch_result: None,
        },
    );
    let mut pending_native_dispatches = BTreeMap::new();
    pending_native_dispatches.insert(
        "019f0657-1d67-7103-9d65-89e71587347d".to_string(),
        vec![crate::spawn_orchestration::PendingSpawnDispatch::new(
            "native queued task".to_string(),
            Vec::new(),
        )],
    );
    let mut pending_claude_dispatches = BTreeMap::new();
    pending_claude_dispatches.insert(
        "claude-active".to_string(),
        vec![crate::spawn_orchestration::PendingSpawnDispatch::new(
            "claude queued task".to_string(),
            Vec::new(),
        )],
    );
    let mut crew =
        crate::crew_state::CrewInstanceState::begin(crate::crew_presets::standard_crew_spec())
            .expect("standard crew");
    for (index, member_id) in ["nazgul", "troll", "orc-1", "orc-2", "orc-3"]
        .into_iter()
        .enumerate()
    {
        crew.record_member(
            member_id,
            &format!("thread:00000000-0000-7000-8000-{index:012}"),
        )
        .expect("crew member");
    }
    crew.mark_ready().expect("ready crew");
    let layout = PaneLayoutState {
        version: 0,
        codex_thread_id: Some("019f0657-1d67-7103-9d65-89e71587347d".to_string()),
        codex_user_pane_ids: vec!["019f0e22-e6e9-7e02-9cca-9dc18667b3e5".to_string()],
        active_user_pane_id: Some("claude-active".to_string()),
        spawn_nazgul_pane_id: Some("claude-root".to_string()),
        spawn_nazgul_rebind_required: true,
        claude_pane_ids: vec!["claude-root".to_string(), "claude-active".to_string()],
        spawn_parent_by_node: parents.clone(),
        spawn_native_runtime_by_node: BTreeMap::new(),
        spawn_native_endpoint_by_node: BTreeMap::from([(
            "thread:019f0657-1d67-7103-9d65-89e71587347d".to_string(),
            "019f0e22-e6e9-7e02-9cca-9dc18667b3e5".to_string(),
        )]),
        spawn_crew: Some(crew.clone()),
        orchestrate_whips: whips.clone(),
        orchestrate_next_whip_seq: 3,
        spawn_pending_dispatches: BTreeMap::new(),
        spawn_pending_dispatches_by_thread: pending_native_dispatches.clone(),
        spawn_pending_dispatches_by_pane: pending_claude_dispatches.clone(),
        spawn_next_dispatch_seq: 42,
        spawn_processed_dispatch_seq_ids: vec![39, 41],
        spawn_processed_dispatch_origin_ids: Vec::new(),
        spawn_accepted_delivery_ids: Vec::new(),
    };

    persist_pane_layout(codex_home.path(), &layout).expect("persist layout");
    let restored = load_pane_layout(
        codex_home.path(),
        Some("019f0657-1d67-7103-9d65-89e71587347d"),
    )
    .expect("layout");
    assert_eq!(restored.version, PANE_LAYOUT_VERSION);
    assert_eq!(restored.codex_thread_id, layout.codex_thread_id);
    assert_eq!(restored.codex_user_pane_ids, layout.codex_user_pane_ids);
    assert_eq!(
        restored.active_user_pane_id.as_deref(),
        Some("claude-active")
    );
    assert_eq!(
        restored.spawn_nazgul_pane_id.as_deref(),
        Some("claude-root")
    );
    assert!(restored.spawn_nazgul_rebind_required);
    assert_eq!(restored.claude_pane_ids, layout.claude_pane_ids);
    assert_eq!(restored.spawn_parent_by_node, parents);
    assert_eq!(restored.spawn_crew, Some(crew));
    assert_eq!(
        restored.spawn_native_endpoint_by_node["thread:019f0657-1d67-7103-9d65-89e71587347d"],
        "019f0e22-e6e9-7e02-9cca-9dc18667b3e5"
    );
    assert_eq!(restored.orchestrate_whips, whips);
    assert_eq!(restored.orchestrate_next_whip_seq, 3);
    let restored_native =
        &restored.spawn_pending_dispatches["thread:019f0657-1d67-7103-9d65-89e71587347d"][0];
    assert_eq!(restored_native.task, "native queued task");
    assert!(!restored_native.dispatch_id.is_empty());
    assert_eq!(
        restored_native.target_pane_id,
        "thread:019f0657-1d67-7103-9d65-89e71587347d"
    );
    let restored_claude = &restored.spawn_pending_dispatches["pane:claude-active"][0];
    assert_eq!(restored_claude.task, "claude queued task");
    assert!(!restored_claude.dispatch_id.is_empty());
    assert_eq!(restored_claude.target_pane_id, "pane:claude-active");
    assert_eq!(restored.spawn_next_dispatch_seq, 42);
    assert_eq!(restored.spawn_processed_dispatch_seq_ids, vec![39, 41]);
}

#[test]
fn pane_layout_load_finds_owner_when_layout_contains_only_codex_user_panes() {
    let codex_home = tempfile::tempdir().expect("codex home");
    let root_thread = "019f1e4e-c71e-7e23-893f-ffb64b8744bb";
    let user_pane_thread = "019f1e4e-f4f6-7ff3-892a-b836a10f2957";
    let layout = PaneLayoutState {
        version: PANE_LAYOUT_VERSION,
        codex_thread_id: Some(root_thread.to_string()),
        codex_user_pane_ids: vec![user_pane_thread.to_string()],
        ..Default::default()
    };

    persist_pane_layout(codex_home.path(), &layout).expect("persist user-pane-only layout");

    let by_owner = load_pane_layout(codex_home.path(), Some(root_thread))
        .expect("owner should load its user-pane-only layout");
    assert_eq!(by_owner.codex_user_pane_ids, vec![user_pane_thread]);

    let by_member = load_pane_layout(codex_home.path(), Some(user_pane_thread))
        .expect("member should resolve its owning Main layout");
    assert_eq!(by_member.codex_thread_id.as_deref(), Some(root_thread));
    assert_eq!(by_member.codex_user_pane_ids, vec![user_pane_thread]);
}

#[test]
fn pane_layout_persistence_is_thread_scoped() {
    let codex_home = tempfile::tempdir().expect("codex home");
    let first_thread = "019f0657-1d67-7103-9d65-89e71587347d";
    let second_thread = "019f0e22-e6e9-7e02-9cca-9dc18667b3e5";
    let first_layout = PaneLayoutState {
        version: 0,
        codex_thread_id: Some(first_thread.to_string()),
        active_user_pane_id: Some("claude-first".to_string()),
        spawn_nazgul_pane_id: None,
        claude_pane_ids: vec!["claude-first".to_string()],
        spawn_parent_by_node: BTreeMap::new(),
        ..Default::default()
    };
    let second_layout = PaneLayoutState {
        version: 0,
        codex_thread_id: Some(second_thread.to_string()),
        active_user_pane_id: Some("claude-second".to_string()),
        spawn_nazgul_pane_id: None,
        claude_pane_ids: vec!["claude-second".to_string()],
        spawn_parent_by_node: BTreeMap::new(),
        ..Default::default()
    };

    persist_pane_layout(codex_home.path(), &first_layout).expect("persist first layout");
    persist_pane_layout(codex_home.path(), &second_layout).expect("persist second layout");

    let first_restored = load_pane_layout(codex_home.path(), Some(first_thread)).expect("first");
    let second_restored = load_pane_layout(codex_home.path(), Some(second_thread)).expect("second");
    assert_eq!(first_restored.claude_pane_ids, vec!["claude-first"]);
    assert_eq!(second_restored.claude_pane_ids, vec!["claude-second"]);
    assert!(load_pane_layout(codex_home.path(), /*codex_thread_id*/ None).is_none());
}

#[test]
fn pane_layout_load_finds_related_root_layout_for_native_spawn_thread() {
    let codex_home = tempfile::tempdir().expect("codex home");
    let root_thread = "019f1e4e-c71e-7e23-893f-ffb64b8744bb";
    let nazgul_thread = "019f1e4e-f4f6-7ff3-892a-b836a10f2957";
    let troll_thread = "019f1e4f-329f-7c81-a706-7ab5a03b705f";
    let mut parents = BTreeMap::new();
    parents.insert(
        format!("thread:{nazgul_thread}"),
        "pane:codex-main".to_string(),
    );
    parents.insert(
        format!("thread:{troll_thread}"),
        format!("thread:{nazgul_thread}"),
    );
    let root_layout = PaneLayoutState {
        version: 0,
        codex_thread_id: Some(root_thread.to_string()),
        active_user_pane_id: Some("codex-main".to_string()),
        spawn_nazgul_pane_id: Some(format!("thread:{nazgul_thread}")),
        claude_pane_ids: Vec::new(),
        spawn_parent_by_node: parents.clone(),
        ..Default::default()
    };
    let empty_child_layout = PaneLayoutState {
        version: 0,
        codex_thread_id: Some(nazgul_thread.to_string()),
        active_user_pane_id: Some("codex-main".to_string()),
        spawn_nazgul_pane_id: None,
        claude_pane_ids: Vec::new(),
        spawn_parent_by_node: BTreeMap::new(),
        ..Default::default()
    };

    persist_pane_layout(codex_home.path(), &root_layout).expect("persist root layout");
    persist_pane_layout(codex_home.path(), &empty_child_layout)
        .expect("persist empty child layout");

    let restored =
        load_pane_layout(codex_home.path(), Some(nazgul_thread)).expect("related root layout");

    assert_eq!(restored.codex_thread_id.as_deref(), Some(root_thread));
    assert_eq!(
        restored.spawn_nazgul_pane_id.as_deref(),
        Some(format!("thread:{nazgul_thread}").as_str())
    );
    assert_eq!(restored.spawn_parent_by_node, parents);
}

#[test]
fn pane_layout_load_prefers_most_complete_matching_crew_when_exact_owner_has_stale_subset() {
    let codex_home = tempfile::tempdir().expect("codex home");
    let resumable_main = "019f1e50-0000-7000-8000-000000000001";
    let empty_new_main = "019f1e50-0000-7000-8000-000000000002";
    let nazgul_node = "thread:019f1e50-0000-7000-8000-000000000003";

    let mut complete_crew =
        crate::crew_state::CrewInstanceState::begin(crate::crew_presets::standard_crew_spec())
            .expect("valid crew");
    for member in complete_crew.spec.members.clone() {
        complete_crew
            .record_member(
                &member.logical_member_id,
                &format!("thread:{}", member.logical_member_id),
            )
            .expect("record crew member");
    }
    complete_crew.mark_ready().expect("complete crew is ready");

    let mut stale_crew = complete_crew.clone();
    stale_crew.spec.members.truncate(1);
    stale_crew
        .member_node_by_id
        .retain(|member_id, _| member_id == "nazgul");
    stale_crew
        .mark_ready()
        .expect("stale root-only crew is valid");

    let complete_layout = PaneLayoutState {
        version: PANE_LAYOUT_VERSION,
        codex_thread_id: Some(empty_new_main.to_string()),
        active_user_pane_id: Some(CODEX_MAIN_PANE_ID.to_string()),
        spawn_nazgul_pane_id: Some(nazgul_node.to_string()),
        spawn_crew: Some(complete_crew.clone()),
        ..Default::default()
    };
    let stale_exact_layout = PaneLayoutState {
        version: PANE_LAYOUT_VERSION,
        codex_thread_id: Some(resumable_main.to_string()),
        active_user_pane_id: Some(CODEX_MAIN_PANE_ID.to_string()),
        spawn_nazgul_pane_id: Some(nazgul_node.to_string()),
        spawn_crew: Some(stale_crew),
        ..Default::default()
    };

    // Persist the stale exact match last. Recovery must prioritize durable crew completeness over
    // mtime because a resumed Main thread can receive a fresh layout id without ever producing a
    // rollout, making that newer id impossible to select from `resume`.
    persist_pane_layout(codex_home.path(), &complete_layout).expect("complete layout");
    persist_pane_layout(codex_home.path(), &stale_exact_layout).expect("stale exact layout");

    let restored =
        load_pane_layout(codex_home.path(), Some(resumable_main)).expect("related complete layout");
    assert_eq!(
        restored.codex_thread_id.as_deref(),
        Some(empty_new_main),
        "the richest matching CrewSpec must survive even when its layout owner has no rollout"
    );
    assert_eq!(
        restored
            .spawn_crew
            .as_ref()
            .map(|crew| crew.spec.members.len()),
        Some(complete_crew.spec.members.len())
    );
}

#[test]
fn pane_layout_recovers_verified_previous_generation_after_corruption() {
    let codex_home = tempfile::tempdir().expect("codex home");
    let thread_id = "019f2b89-775c-7dc1-9d20-4fdf7e990199";
    let first = PaneLayoutState {
        version: 0,
        codex_thread_id: Some(thread_id.to_string()),
        active_user_pane_id: Some("first".to_string()),
        ..Default::default()
    };
    let second = PaneLayoutState {
        active_user_pane_id: Some("second".to_string()),
        ..first.clone()
    };
    persist_pane_layout(codex_home.path(), &first).expect("first generation");
    persist_pane_layout(codex_home.path(), &second).expect("second generation");
    let primary = codex_home
        .path()
        .join("panes")
        .join("pane-layouts")
        .join(format!("{thread_id}.json"));
    std::fs::write(&primary, b"{truncated").expect("corrupt primary");

    let restored =
        load_pane_layout(codex_home.path(), Some(thread_id)).expect("verified previous generation");

    assert_eq!(restored.active_user_pane_id.as_deref(), Some("first"));
}

fn persisted_layout_path(codex_home: &std::path::Path, thread_id: &str) -> PathBuf {
    codex_home
        .join("panes")
        .join("pane-layouts")
        .join(format!("{thread_id}.json"))
}

#[test]
fn pane_layout_bad_checksum_recovers_verified_previous_generation() {
    let codex_home = tempfile::tempdir().expect("codex home");
    let thread_id = "019f2b89-775c-7dc1-9d20-4fdf7e990299";
    let first = PaneLayoutState {
        codex_thread_id: Some(thread_id.to_string()),
        active_user_pane_id: Some("checksum-previous".to_string()),
        ..Default::default()
    };
    let second = PaneLayoutState {
        active_user_pane_id: Some("checksum-primary".to_string()),
        ..first.clone()
    };
    persist_pane_layout(codex_home.path(), &first).expect("first generation");
    persist_pane_layout(codex_home.path(), &second).expect("second generation");
    let primary = persisted_layout_path(codex_home.path(), thread_id);
    let mut json: Value =
        serde_json::from_slice(&std::fs::read(&primary).expect("primary")).expect("persisted JSON");
    json["checksum"] = Value::String("bad-checksum".to_string());
    std::fs::write(&primary, serde_json::to_vec_pretty(&json).expect("JSON"))
        .expect("corrupt checksum");

    let restored = load_pane_layout(codex_home.path(), Some(thread_id)).expect("previous");
    assert_eq!(
        restored.active_user_pane_id.as_deref(),
        Some("checksum-previous")
    );
}

#[test]
fn pane_layout_checksum_accepts_a_verified_prior_schema_with_new_default_fields_absent() {
    let codex_home = tempfile::tempdir().expect("codex home");
    let thread_id = "019f2b89-775c-7dc1-9d20-4fdf7e990349";
    let layout = PaneLayoutState {
        codex_thread_id: Some(thread_id.to_string()),
        spawn_nazgul_pane_id: Some(format!("thread:{thread_id}")),
        spawn_parent_by_node: BTreeMap::from([(
            "thread:019f2b89-775c-7dc1-9d20-4fdf7e990350".to_string(),
            format!("thread:{thread_id}"),
        )]),
        ..Default::default()
    };
    persist_pane_layout(codex_home.path(), &layout).expect("current schema");
    let primary = persisted_layout_path(codex_home.path(), thread_id);
    let mut persisted: Value =
        serde_json::from_slice(&std::fs::read(&primary).expect("primary")).expect("JSON");
    let raw_layout = persisted["layout"].as_object_mut().expect("layout object");
    assert!(raw_layout.shift_remove("codex_user_pane_ids").is_some());
    let checksum = format!(
        "{:x}",
        Sha256::digest(serde_json::to_vec(raw_layout).expect("raw prior-schema layout"))
    );
    persisted["checksum"] = Value::String(checksum);
    std::fs::write(
        &primary,
        serde_json::to_vec_pretty(&persisted).expect("prior-schema JSON"),
    )
    .expect("write prior schema");

    let restored =
        load_pane_layout(codex_home.path(), Some(thread_id)).expect("verified prior-schema layout");

    assert!(restored.codex_user_pane_ids.is_empty());
    assert_eq!(restored.spawn_nazgul_pane_id, layout.spawn_nazgul_pane_id);
    assert_eq!(restored.spawn_parent_by_node, layout.spawn_parent_by_node);
}

#[test]
fn pane_layout_persistence_refuses_to_destroy_the_only_unverified_generation() {
    let codex_home = tempfile::tempdir().expect("codex home");
    let thread_id = "019f2b89-775c-7dc1-9d20-4fdf7e990379";
    let original = PaneLayoutState {
        codex_thread_id: Some(thread_id.to_string()),
        spawn_nazgul_pane_id: Some(format!("thread:{thread_id}")),
        ..Default::default()
    };
    persist_pane_layout(codex_home.path(), &original).expect("original");
    let primary = persisted_layout_path(codex_home.path(), thread_id);
    let corrupt_contents = b"{recoverable but currently unreadable".to_vec();
    std::fs::write(&primary, &corrupt_contents).expect("corrupt primary");

    let replacement = PaneLayoutState {
        codex_thread_id: Some(thread_id.to_string()),
        ..Default::default()
    };
    let error = persist_pane_layout(codex_home.path(), &replacement)
        .expect_err("unverified state must fail closed");

    assert!(
        error
            .to_string()
            .contains("refusing to overwrite pane layout")
    );
    assert_eq!(
        std::fs::read(&primary).expect("preserved primary"),
        corrupt_contents
    );
    assert!(!primary.with_extension("json.previous").exists());
}

#[test]
fn pane_layout_persistence_refuses_to_destroy_two_unverified_generations() {
    let codex_home = tempfile::tempdir().expect("codex home");
    let thread_id = "019f2b89-775c-7dc1-9d20-4fdf7e990389";
    let first = PaneLayoutState {
        codex_thread_id: Some(thread_id.to_string()),
        active_user_pane_id: Some("first-generation".to_string()),
        ..Default::default()
    };
    persist_pane_layout(codex_home.path(), &first).expect("first generation");
    let second = PaneLayoutState {
        codex_thread_id: Some(thread_id.to_string()),
        active_user_pane_id: Some("second-generation".to_string()),
        ..Default::default()
    };
    persist_pane_layout(codex_home.path(), &second).expect("second generation");

    let primary = persisted_layout_path(codex_home.path(), thread_id);
    let previous = primary.with_extension("json.previous");
    let corrupt_primary = b"{unverified current generation".to_vec();
    let corrupt_previous = b"{unverified recovery generation".to_vec();
    std::fs::write(&primary, &corrupt_primary).expect("corrupt primary");
    std::fs::write(&previous, &corrupt_previous).expect("corrupt previous");

    let replacement = PaneLayoutState {
        codex_thread_id: Some(thread_id.to_string()),
        ..Default::default()
    };
    let error = persist_pane_layout(codex_home.path(), &replacement)
        .expect_err("two unverified generations must fail closed");

    assert!(
        error
            .to_string()
            .contains("refusing to overwrite pane layout")
    );
    assert_eq!(
        std::fs::read(&primary).expect("preserved primary"),
        corrupt_primary
    );
    assert_eq!(
        std::fs::read(&previous).expect("preserved previous"),
        corrupt_previous
    );
}

#[test]
fn pane_layout_corrupt_primary_without_previous_fails_closed() {
    let codex_home = tempfile::tempdir().expect("codex home");
    let thread_id = "019f2b89-775c-7dc1-9d20-4fdf7e990399";
    let layout = PaneLayoutState {
        codex_thread_id: Some(thread_id.to_string()),
        active_user_pane_id: Some("only-generation".to_string()),
        ..Default::default()
    };
    persist_pane_layout(codex_home.path(), &layout).expect("primary generation");
    std::fs::write(
        persisted_layout_path(codex_home.path(), thread_id),
        b"{truncated",
    )
    .expect("truncate primary");

    assert!(load_pane_layout(codex_home.path(), Some(thread_id)).is_none());
}

#[test]
fn pane_layout_incompatible_version_fails_closed() {
    let codex_home = tempfile::tempdir().expect("codex home");
    let thread_id = "019f2b89-775c-7dc1-9d20-4fdf7e990499";
    let layout = PaneLayoutState {
        codex_thread_id: Some(thread_id.to_string()),
        ..Default::default()
    };
    persist_pane_layout(codex_home.path(), &layout).expect("primary generation");
    let primary = persisted_layout_path(codex_home.path(), thread_id);
    let mut json: Value =
        serde_json::from_slice(&std::fs::read(&primary).expect("primary")).expect("persisted JSON");
    json["format_version"] = serde_json::json!(PANE_LAYOUT_VERSION + 1);
    std::fs::write(&primary, serde_json::to_vec_pretty(&json).expect("JSON"))
        .expect("incompatible version");

    assert!(load_pane_layout(codex_home.path(), Some(thread_id)).is_none());
}

const CRASH_MATRIX_THREAD_ID: &str = "019f2b89-775c-7dc1-9d20-4fdf7e991000";
const CRASH_MATRIX_TARGET_ID: &str = "019f2b89-775c-7dc1-9d20-4fdf7e991001";

fn crash_matrix_layout(state: Option<crate::dispatch_queue::DispatchState>) -> PaneLayoutState {
    let target = format!("thread:{CRASH_MATRIX_TARGET_ID}");
    let mut layout = PaneLayoutState {
        codex_thread_id: Some(CRASH_MATRIX_THREAD_ID.to_string()),
        spawn_native_endpoint_by_node: BTreeMap::from([(
            target.clone(),
            CRASH_MATRIX_TARGET_ID.to_string(),
        )]),
        ..Default::default()
    };
    if let Some(state) = state {
        let mut dispatch = crate::spawn_orchestration::PendingSpawnDispatch::new(
            "crash matrix task".to_string(),
            Vec::new(),
        );
        dispatch.assign_identity(
            /*seq*/ 1,
            "pane:codex-main",
            &target,
            Some("crash-matrix-origin"),
        );
        dispatch.state = state;
        layout
            .spawn_processed_dispatch_origin_ids
            .push(dispatch.origin.origin_id.clone());
        layout
            .spawn_pending_dispatches
            .insert(target, vec![dispatch]);
    }
    layout
}

#[test]
fn dispatch_process_cut_child() {
    let Ok(home) = std::env::var("PFTERMINAL_DISPATCH_CRASH_HOME") else {
        return;
    };
    let cut = std::env::var("PFTERMINAL_DISPATCH_CRASH_CUT").expect("crash cut");
    let home = PathBuf::from(home);
    persist_pane_layout(&home, &crash_matrix_layout(/*state*/ None)).expect("baseline generation");
    let queued = crate::dispatch_queue::DispatchState::Queued;
    let submitting = crate::dispatch_queue::DispatchState::Submitting {
        delivery_id: "delivery-crash-matrix".to_string(),
        ordered_dispatch_ids: vec!["dispatch-crash-matrix".to_string()],
    };
    let marker = |name: &str| std::fs::write(home.join(name), b"durable").expect("marker");
    match cut.as_str() {
        "before_enqueue_commit" => {}
        "after_enqueue_before_receipt" => {
            persist_pane_layout(&home, &crash_matrix_layout(Some(queued))).expect("queued commit");
        }
        "after_submitting_before_send" => {
            persist_pane_layout(&home, &crash_matrix_layout(Some(submitting)))
                .expect("submitting commit");
        }
        "request_bytes_before_server_receipt" => {
            persist_pane_layout(&home, &crash_matrix_layout(Some(submitting)))
                .expect("submitting commit");
            marker("rpc-bytes-sent");
        }
        "server_accept_before_response" => {
            persist_pane_layout(&home, &crash_matrix_layout(Some(submitting)))
                .expect("submitting commit");
            marker("rpc-bytes-sent");
            marker("server-durable-acceptance");
        }
        "response_before_local_tombstone" => {
            persist_pane_layout(&home, &crash_matrix_layout(Some(submitting)))
                .expect("submitting commit");
            marker("rpc-bytes-sent");
            marker("server-durable-acceptance");
            marker("rpc-response-received");
        }
        "during_atomic_snapshot_write" => {
            // The registry fault exits after the replacement temp file is synced but before the
            // verified baseline generation is rotated.
            persist_pane_layout(&home, &crash_matrix_layout(Some(queued)))
                .expect("fault exits before return");
            unreachable!("atomic snapshot fault must terminate the process");
        }
        "during_thread_replacement_migration" => {
            let mut in_memory = crash_matrix_layout(Some(queued));
            in_memory.spawn_native_endpoint_by_node.insert(
                format!("thread:{CRASH_MATRIX_TARGET_ID}"),
                "019f2b89-775c-7dc1-9d20-4fdf7e991099".to_string(),
            );
            marker("replacement-mutation-started");
            std::hint::black_box(in_memory);
        }
        "event_stream_disconnected" => {
            persist_pane_layout(&home, &crash_matrix_layout(Some(queued))).expect("queued commit");
            marker("event-stream-disconnected");
        }
        "during_tui_shutdown" => {
            persist_pane_layout(&home, &crash_matrix_layout(Some(queued))).expect("queued commit");
            marker("shutdown-started");
        }
        other => panic!("unknown crash cut {other}"),
    }
    std::process::exit(86);
}

fn crash_matrix_loaded_layout(home: &std::path::Path) -> Option<PaneLayoutState> {
    std::fs::read_dir(home.join("panes").join("pane-layouts"))
        .ok()?
        .flatten()
        .filter_map(|entry| {
            let path = entry.path();
            (path.extension().and_then(|extension| extension.to_str()) == Some("json"))
                .then(|| path.file_stem()?.to_str().map(str::to_string))
                .flatten()
        })
        .find_map(|thread_id| load_pane_layout(home, Some(&thread_id)))
}

fn crash_tree_contains(path: &std::path::Path, needle: &str) -> bool {
    if path.is_file() {
        return std::fs::read(path)
            .ok()
            .is_some_and(|bytes| String::from_utf8_lossy(&bytes).contains(needle));
    }
    std::fs::read_dir(path)
        .ok()
        .into_iter()
        .flatten()
        .flatten()
        .any(|entry| crash_tree_contains(&entry.path(), needle))
}

#[test]
fn deterministic_dispatch_process_crash_matrix_recovers_readable_state() {
    let cuts = [
        "before_enqueue_commit",
        "after_enqueue_before_receipt",
        "after_submitting_before_send",
        "request_bytes_before_server_receipt",
        "server_accept_before_response",
        "response_before_local_tombstone",
        "during_atomic_snapshot_write",
        "during_thread_replacement_migration",
        "event_stream_disconnected",
        "during_tui_shutdown",
    ];
    for cut in cuts {
        let home = tempfile::tempdir().expect("crash home");
        let rpc_cut = matches!(
            cut,
            "server_accept_before_response" | "response_before_local_tombstone"
        );
        // Each cut must terminate at its exact persisted checkpoint. The separate dispatch
        // integration suite exercises the live mailbox RPC; it cannot substitute for injecting
        // a process death between an accepted request and its local acknowledgement.
        let test_filter = "claude_panes::tests::dispatch_process_cut_child";
        let output = std::process::Command::new(std::env::current_exe().expect("test binary"))
            .args(["--exact", test_filter, "--nocapture"])
            .env("PFTERMINAL_DISPATCH_CRASH_HOME", home.path())
            .env("PFTERMINAL_DISPATCH_CRASH_CUT", cut)
            .output()
            .expect("spawn crash child");
        assert_eq!(
            output.status.code(),
            Some(86),
            "cut={cut}; stdout={}; stderr={}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );

        if rpc_cut {
            let restored = crash_matrix_loaded_layout(home.path())
                .unwrap_or_else(|| panic!("cut={cut}: no persisted RPC layout"));
            let queue = restored
                .spawn_pending_dispatches
                .values()
                .find(|queue| !queue.is_empty())
                .unwrap_or_else(|| panic!("cut={cut}: submitting RPC record was lost"));
            assert_eq!(queue.len(), 1, "cut={cut}: duplicate RPC presentation");
            let crate::dispatch_queue::DispatchState::Submitting { delivery_id, .. } =
                &queue[0].state
            else {
                panic!("cut={cut}: RPC cut did not retain Submitting state")
            };
            assert!(
                crash_tree_contains(home.path(), delivery_id),
                "cut={cut}: core rollout lacks the durable client delivery identity"
            );
            let owner = restored.codex_thread_id.expect("layout owner");
            let primary = persisted_layout_path(home.path(), &owner);
            let stable = std::fs::read(&primary).expect("stable RPC primary bytes");
            std::thread::sleep(Duration::from_millis(30));
            assert_eq!(
                std::fs::read(&primary).expect("RPC primary after child death"),
                stable,
                "cut={cut}: stale RPC writer continued after termination"
            );
            continue;
        }

        let restored = load_pane_layout(home.path(), Some(CRASH_MATRIX_THREAD_ID))
            .unwrap_or_else(|| panic!("cut={cut}: restart could not read a verified generation"));
        let target = format!("thread:{CRASH_MATRIX_TARGET_ID}");
        let queue = restored.spawn_pending_dispatches.get(&target);
        match cut {
            "before_enqueue_commit"
            | "during_atomic_snapshot_write"
            | "during_thread_replacement_migration" => assert!(
                queue.is_none_or(Vec::is_empty),
                "cut={cut}: uncommitted work became visible"
            ),
            "after_enqueue_before_receipt"
            | "event_stream_disconnected"
            | "during_tui_shutdown" => assert!(matches!(
                queue
                    .and_then(|queue| queue.first())
                    .map(|item| &item.state),
                Some(crate::dispatch_queue::DispatchState::Queued)
            )),
            _ => assert!(matches!(
                queue
                    .and_then(|queue| queue.first())
                    .map(|item| &item.state),
                Some(crate::dispatch_queue::DispatchState::Submitting { .. })
            )),
        }
        assert!(queue.is_none_or(|queue| queue.len() <= 1));
        if matches!(
            cut,
            "server_accept_before_response" | "response_before_local_tombstone"
        ) {
            assert!(home.path().join("server-durable-acceptance").exists());
        }
        if cut == "during_thread_replacement_migration" {
            assert_eq!(
                restored
                    .spawn_native_endpoint_by_node
                    .get(&target)
                    .map(String::as_str),
                Some(CRASH_MATRIX_TARGET_ID),
                "replacement must restore wholly before or wholly after migration"
            );
        }

        let primary = persisted_layout_path(home.path(), CRASH_MATRIX_THREAD_ID);
        let stable = std::fs::read(&primary).expect("stable primary bytes");
        std::thread::sleep(Duration::from_millis(30));
        assert_eq!(
            std::fs::read(&primary).expect("primary after child death"),
            stable,
            "cut={cut}: stale process continued writing after termination"
        );
    }
}

#[test]
fn pane_layout_v1_migrates_legacy_batch_once() {
    let codex_home = tempfile::tempdir().expect("codex home");
    let thread_id = "019f2b89-cba2-7c81-ae8f-f48106932d0b";
    let first = "first legacy task";
    let second = "second legacy task";
    let envelope = format!(
        "Multiple spawn dispatches were queued while you were busy. Execute each task below in order, do not skip any task, and treat every section as assigned work.\n\n## Queued dispatch 1 (bytes={})\n{}\n## Queued dispatch 2 (bytes={})\n{}",
        first.len(),
        first,
        second.len(),
        second
    );
    let mut queues = BTreeMap::new();
    queues.insert(
        thread_id.to_string(),
        vec![crate::spawn_orchestration::PendingSpawnDispatch::new(
            envelope,
            Vec::new(),
        )],
    );
    let legacy = PaneLayoutState {
        version: 1,
        codex_thread_id: Some(thread_id.to_string()),
        spawn_nazgul_pane_id: Some(format!("thread:{thread_id}")),
        spawn_pending_dispatches_by_thread: queues,
        ..Default::default()
    };
    let primary = codex_home
        .path()
        .join("panes")
        .join("pane-layouts")
        .join(format!("{thread_id}.json"));
    std::fs::create_dir_all(primary.parent().expect("layout parent")).expect("layout dir");
    std::fs::write(
        &primary,
        serde_json::to_vec_pretty(&legacy).expect("legacy JSON"),
    )
    .expect("legacy layout");

    let restored = load_pane_layout(codex_home.path(), Some(thread_id)).expect("migrated layout");
    let migrated = restored
        .spawn_pending_dispatches
        .get(&format!("thread:{thread_id}"))
        .expect("migrated queue");

    assert_eq!(
        migrated
            .iter()
            .map(|dispatch| dispatch.task.as_str())
            .collect::<Vec<_>>(),
        vec![first, second]
    );
    assert!(
        migrated
            .iter()
            .all(|dispatch| !dispatch.dispatch_id.is_empty())
    );
    let persisted = std::fs::read_to_string(primary).expect("version 2 layout");
    assert!(persisted.contains("\"format_version\": 2"));
    assert!(persisted.contains("\"checksum\""));
}

#[test]
fn settings_json_uses_helper_without_secret_material() {
    let profile = ClaudeProviderProfileKind::ZaiGlm52.profile();
    let settings =
        settings_json_with_base_url(profile, Some("corbanu"), /*base_url_override*/ None);
    let rendered = settings.to_string();

    assert!(rendered.contains("https://api.z.ai/api/anthropic"));
    assert!(rendered.contains("glm-5.2[1m]"));
    assert!(rendered.contains("CLAUDE_CODE_DISABLE_EXPERIMENTAL_BETAS"));
    assert!(rendered.contains("corbanu vault auth-helper provider/zai_api_key"));
    assert!(!rendered.contains("zai-secret"));
}

#[test]
fn claude_plan_generated_settings_route_through_the_local_credential_bridge() {
    let (dir, pane) = pane(ClaudeProviderProfileKind::ClaudePlan);
    let plan = build_claude_command_plan(&pane, "hello".to_string(), dir.path())
        .expect("Claude Plan command");
    let settings: serde_json::Value = serde_json::from_slice(
        &std::fs::read(plan.artifact_path.with_file_name("settings.json"))
            .expect("generated settings"),
    )
    .expect("parse generated settings");

    assert!(
        settings
            .pointer("/env/ANTHROPIC_BASE_URL")
            .and_then(Value::as_str)
            .is_some_and(|base_url| base_url.starts_with("http://127.0.0.1:"))
    );
    assert_eq!(settings.pointer("/apiKeyHelper"), None);
    let bridge = plan.bridge.as_ref().expect("Claude Plan bridge");
    assert_eq!(bridge.kind, ClaudeBridgeKind::AnthropicOauthPassthrough);
    assert_eq!(bridge.upstream_base_url, "https://api.anthropic.com");
    assert!(bridge.upstream_api_key.is_none());
    assert!(bridge.deferred_vault_secret.is_none());
    let client_auth_token = plan
        .env
        .get("ANTHROPIC_AUTH_TOKEN")
        .expect("per-turn bridge credential");
    assert_eq!(client_auth_token, &bridge.client_auth_token);
    assert!(Uuid::parse_str(client_auth_token).is_ok());
}

#[tokio::test]
async fn anthropic_passthrough_bridge_replaces_client_auth_and_forwards_oauth_beta() {
    let upstream_listener = TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind fake Anthropic upstream");
    let upstream_addr = upstream_listener
        .local_addr()
        .expect("fake upstream address");
    let upstream = tokio::spawn(async move {
        let (mut stream, _) = upstream_listener.accept().await.expect("accept upstream");
        let request = read_http_request(&mut stream).await;
        stream
            .write_all(
                b"HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: 2\r\nConnection: close\r\n\r\n{}",
            )
            .await
            .expect("write fake upstream response");
        request
    });

    let bridge_listener = TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind credential bridge");
    let bridge_addr = bridge_listener.local_addr().expect("bridge address");
    let bridge = tokio::spawn(async move {
        for _ in 0..2 {
            let (stream, _) = bridge_listener.accept().await.expect("accept bridge");
            handle_anthropic_passthrough_bridge_connection(
                stream,
                Arc::new("local-bridge-capability".to_string()),
                Arc::new("bridge-upstream-secret-not-real".to_string()),
                Arc::new(format!("http://{upstream_addr}")),
                reqwest::Client::new(),
                /*proxy_count_tokens*/ true,
            )
            .await
            .expect("proxy request");
        }
    });

    let mut unauthorized_client = TcpStream::connect(bridge_addr)
        .await
        .expect("connect unauthorized client");
    unauthorized_client
        .write_all(
            b"POST /v1/messages/count_tokens HTTP/1.1\r\nHost: localhost\r\nAuthorization: Bearer wrong-capability\r\nContent-Type: application/json\r\nContent-Length: 2\r\nConnection: close\r\n\r\n{}",
        )
        .await
        .expect("write unauthorized request");
    let mut unauthorized_response = Vec::new();
    unauthorized_client
        .read_to_end(&mut unauthorized_response)
        .await
        .expect("read unauthorized response");
    assert!(String::from_utf8_lossy(&unauthorized_response).starts_with("HTTP/1.1 401"));

    let mut client = TcpStream::connect(bridge_addr)
        .await
        .expect("connect to bridge");
    client
        .write_all(
            b"POST /v1/messages/count_tokens HTTP/1.1\r\nHost: localhost\r\nAuthorization: Bearer local-bridge-capability\r\nAnthropic-Version: 2023-06-01\r\nAnthropic-Beta: prompt-caching-2024-07-31\r\nContent-Type: application/json\r\nContent-Length: 2\r\nConnection: close\r\n\r\n{}",
        )
        .await
        .expect("write bridge request");
    let mut response = Vec::new();
    client
        .read_to_end(&mut response)
        .await
        .expect("read bridge response");

    bridge.await.expect("bridge task");
    let upstream_request = upstream.await.expect("upstream task");
    let upstream_request = String::from_utf8(upstream_request).expect("UTF-8 upstream request");
    let upstream_request = upstream_request.to_ascii_lowercase();
    assert!(upstream_request.contains("authorization: bearer bridge-upstream-secret-not-real"));
    assert!(!upstream_request.contains("local-bridge-capability"));
    assert!(upstream_request.starts_with("post /v1/messages/count_tokens http/1.1"));
    assert!(
        upstream_request.contains("anthropic-beta: prompt-caching-2024-07-31,oauth-2025-04-20")
    );
    assert!(String::from_utf8_lossy(&response).starts_with("HTTP/1.1 200 OK"));
}

#[test]
fn direct_provider_plan_uses_auth_helper_without_secret_env() {
    let (dir, pane) = pane(ClaudeProviderProfileKind::ZaiGlm52);
    codex_vault::Vault::new(dir.path().to_path_buf())
        .add(codex_vault::AddCredential {
            label: "provider/zai_api_key".to_string(),
            credential_type: codex_vault::CredentialType::ApiKey,
            provider: Some("zai".to_string()),
            notes: None,
            revocation_notes: None,
            secret: "zai-test-key".to_string(),
        })
        .expect("store test Z.AI key");

    let plan = build_claude_command_plan(&pane, "hello".to_string(), dir.path()).expect("plan");
    let settings = std::fs::read_to_string(pane.artifact_dir.join("settings.json"))
        .expect("settings should be written");
    let settings: Value = serde_json::from_str(&settings).expect("settings json");

    assert!(plan.bridge.is_none());
    assert_eq!(
        settings.pointer("/apiKeyHelper"),
        Some(&json!("corbanu vault auth-helper provider/zai_api_key"))
    );
    assert!(plan.env_remove.iter().any(|key| key == "ANTHROPIC_API_KEY"));
    assert!(
        plan.env_remove
            .iter()
            .any(|key| key == "ANTHROPIC_AUTH_TOKEN")
    );
    assert_eq!(
        plan.env.get("ANTHROPIC_API_KEY").map(String::as_str),
        Some("")
    );
    assert!(!plan.env.contains_key("ANTHROPIC_AUTH_TOKEN"));
    assert!(
        !plan
            .env
            .values()
            .any(|value| value.contains("zai-test-key"))
    );
    assert!(!plan.args.iter().any(|arg| arg.contains("zai-test-key")));
    assert!(!settings.to_string().contains("zai-test-key"));
}

#[test]
fn tool_call_previews_are_readable_blurbs() {
    let bash_with_description = json!({
        "command": "mkdir -p /tmp/gemology-mock && echo ok",
        "description": "Create directory for gemology website mock"
    });
    assert_eq!(
        summarize_tool_call_input("Bash", &bash_with_description),
        "Create directory for gemology website mock"
    );

    let bash_heredoc_without_description = json!({
        "command": "cat > /tmp/gemology-mock/index.html <<'HTMLEOF'\n<!DOCTYPE html>\n<html><body>large page body</body></html>\nHTMLEOF"
    });
    assert_eq!(
        summarize_tool_call_input("Bash", &bash_heredoc_without_description),
        "writing index.html"
    );

    let bash_redirect_with_fd_dup = json!({
        "command": "npm test > /tmp/test-output.log 2>&1"
    });
    assert_eq!(
        summarize_tool_call_input("Bash", &bash_redirect_with_fd_dup),
        "writing test-output.log"
    );

    let bash_dev_null_redirect = json!({
        "command": "npm test > /dev/null 2>&1"
    });
    assert_eq!(
        summarize_tool_call_input("Bash", &bash_dev_null_redirect),
        "npm test > /dev/null 2>&1"
    );

    let edit = json!({
        "file_path": "src/app.rs",
        "old_string": "before",
        "new_string": "after"
    });
    assert_eq!(
        summarize_tool_call_input("Edit", &edit),
        "editing src/app.rs"
    );

    let read = json!({ "file_path": "README.md" });
    assert_eq!(
        summarize_tool_call_input("Read", &read),
        "reading README.md"
    );
}

#[test]
fn tool_call_progress_uses_blurb_not_raw_json_preview() {
    let (dir, pane) = pane(ClaudeProviderProfileKind::ClaudePlan);
    let plan =
        build_claude_command_plan(&pane, "make a mock".to_string(), dir.path()).expect("plan");
    let started_at = Instant::now();
    let value = json!({
        "type": "assistant",
        "message": {
            "content": [{
                "type": "tool_use",
                "name": "Bash",
                "input": {
                    "command": "cat > /tmp/gemology-mock/index.html <<'HTMLEOF'\n<!DOCTYPE html>\n<html>blob</html>\nHTMLEOF"
                }
            }]
        }
    });

    let progress = progress_from_claude_value(&plan, &started_at, &value).expect("tool progress");
    assert_eq!(
        progress.summary,
        "Claude tool call: Bash: writing index.html"
    );
    assert_eq!(progress.hint, None);
    assert!(!progress.summary.contains("{\"command\""));
    assert!(!progress.summary.contains("<!DOCTYPE html>"));
}

#[test]
fn reasoning_progress_uses_thinking_blocks() {
    let (dir, pane) = pane(ClaudeProviderProfileKind::ClaudePlan);
    let plan = build_claude_command_plan(&pane, "review".to_string(), dir.path()).expect("plan");
    let started_at = Instant::now();
    let value = json!({
        "type": "assistant",
        "message": {
            "content": [{
                "type": "thinking",
                "thinking": "I need to inspect the Troll and Orc hierarchy before assigning work."
            }]
        }
    });

    let progress =
        progress_from_claude_value(&plan, &started_at, &value).expect("reasoning progress");
    assert_eq!(progress.phase, "reasoning");
    assert_eq!(
        progress.summary,
        "Claude reasoning: I need to inspect the Troll and Orc hierarchy before assigning work."
    );
    assert_eq!(progress.hint, None);
}

#[test]
fn thinking_token_system_events_render_as_reasoning_progress() {
    let (dir, pane) = pane(ClaudeProviderProfileKind::ClaudePlan);
    let plan = build_claude_command_plan(&pane, "review".to_string(), dir.path()).expect("plan");
    let started_at = Instant::now();
    let value = json!({
        "type": "system",
        "subtype": "thinking_tokens",
        "estimated_tokens": 3136,
        "estimated_tokens_delta": 1,
        "session_id": "11111111-2222-4333-8444-555555555555"
    });

    let progress =
        progress_from_claude_value(&plan, &started_at, &value).expect("thinking progress");
    assert_eq!(progress.phase, "reasoning-tokens");
    assert_eq!(
        progress.summary,
        "Claude reasoning: thinking: 3.1K reasoning tokens"
    );
    assert_eq!(progress.hint.as_deref(), Some("thinking-token-bucket:31"));
    assert_ne!(progress_status_text(&progress), "session initialized");
}

#[test]
fn assistant_text_progress_carries_streaming_delta() {
    let (dir, pane) = pane(ClaudeProviderProfileKind::ClaudePlan);
    let plan = build_claude_command_plan(&pane, "dispatch".to_string(), dir.path()).expect("plan");
    let started_at = Instant::now();
    let value = json!({
        "type": "assistant",
        "message": {
            "content": [{
                "type": "text",
                "text": "```pfterminal-send-task\ntarget: Snaga\ntask:\nbuild site\n```"
            }]
        }
    });

    let progress = progresses_from_claude_value(&plan, &started_at, &value)
        .into_iter()
        .find(|progress| progress.phase == "assistant-text")
        .expect("assistant text progress");
    assert_eq!(
        progress.assistant_text_delta.as_deref(),
        Some("```pfterminal-send-task\ntarget: Snaga\ntask:\nbuild site\n```")
    );
}

#[test]
fn finished_turn_fenced_dispatch_blocks_are_filtered_once() {
    // Streaming deltas no longer collect dispatches (truncated turns must never dispatch);
    // dispatch extraction happens once on the finished turn's full text, deduplicated per turn.
    let mut live_turn = ClaudePaneLiveTurn::starting();
    let (_, dispatches) = crate::spawn_orchestration::extract_spawn_task_dispatches(
        "Before ```pfterminal-send-task\ntarget: Snaga\ntask:\nbuild site\n``` after",
    );
    assert_eq!(dispatches.len(), 1);

    let first = live_turn.filter_new_dispatches(dispatches.clone());
    assert_eq!(first.len(), 1);
    assert_eq!(first[0].target, "Snaga");
    assert_eq!(first[0].task, "build site");

    let duplicate = live_turn.filter_new_dispatches(dispatches);
    assert!(duplicate.is_empty());
}

#[test]
fn finished_turn_xmlish_dispatch_preserves_code_fences() {
    let mut live_turn = ClaudePaneLiveTurn::starting();
    let (_, dispatches) = crate::spawn_orchestration::extract_spawn_task_dispatches(
        "Before <pfterminal_send_task target=\"Burzum\">\nProblem A:\n```systemd\nExecStart=/bin/postfiat\n```\nProblem B: verify writes.\n</pfterminal_send_task> after",
    );
    assert_eq!(dispatches.len(), 1);

    let first = live_turn.filter_new_dispatches(dispatches.clone());
    assert_eq!(first.len(), 1);
    assert_eq!(first[0].target, "Burzum");
    assert!(first[0].task.contains("Problem A:"));
    assert!(first[0].task.contains("```systemd"));
    assert!(first[0].task.contains("ExecStart=/bin/postfiat"));
    assert!(first[0].task.contains("Problem B: verify writes."));

    let duplicate = live_turn.filter_new_dispatches(dispatches);
    assert!(duplicate.is_empty());
}

#[test]
fn visible_assistant_transcript_delta_hides_dispatch_payloads() {
    let mut live_turn = ClaudePaneLiveTurn::starting();
    live_turn
        .assistant_commentary_buffer
        .push_str("I reviewed the failure and will assign it now.\n");
    assert_eq!(
        live_turn.take_visible_assistant_transcript_delta(),
        Some("I reviewed the failure and will assign it now.".to_string())
    );

    live_turn.assistant_commentary_buffer.push_str(
        "<pfterminal_send_task target=\"Ghash\">\nfix the proxy comment\n</pfterminal_send_task>\n",
    );
    assert_eq!(live_turn.take_visible_assistant_transcript_delta(), None);

    live_turn
        .assistant_commentary_buffer
        .push_str("Task queued; I am waiting for the report.");
    let delta = live_turn
        .take_visible_assistant_transcript_delta()
        .expect("post-dispatch commentary delta");
    assert!(delta.contains("Task queued; I am waiting for the report."));
    assert!(!delta.contains("pfterminal_send_task"));
    assert!(!delta.contains("fix the proxy comment"));
}

#[test]
fn live_status_panel_tracks_assistant_commentary_without_dispatch_payload() {
    let dir = tempfile::tempdir().expect("tempdir");
    let mut registry = ClaudePaneRegistry::new();
    let pane_id = registry
        .create_pane(
            ClaudeProviderProfileKind::ClaudePlan,
            std::env::current_dir().expect("cwd"),
            dir.path(),
        )
        .expect("pane");
    let artifact_path = dir.path().join("turn-0001.jsonl");
    let audit_path = dir.path().join("turn-0001.audit.json");

    let commentary = ClaudePaneTurnProgress {
        pane_id: pane_id.clone(),
        phase: "assistant-text".to_string(),
        summary: "Claude assistant text.".to_string(),
        assistant_text_delta: Some(
            "Let me trace the allow flags and wrap_owned relationship.".to_string(),
        ),
        hint: None,
        elapsed_ms: 25_000,
        artifact_path: artifact_path.clone(),
        audit_path: audit_path.clone(),
    };
    let status = registry.update_live_progress(&commentary).expect("status");
    let details = status.details.expect("details");
    assert!(details.contains(
        "Current: Claude note: Let me trace the allow flags and wrap_owned relationship."
    ));
    assert!(details.contains("Claude notes:"));
    assert!(details.contains("Let me trace the allow flags and wrap_owned relationship."));
    assert!(!details.contains("artifact:"));
    assert!(!details.contains("audit:"));

    let partial_dispatch = ClaudePaneTurnProgress {
        pane_id: pane_id.clone(),
        phase: "assistant-text".to_string(),
        summary: "Claude assistant text.".to_string(),
        assistant_text_delta: Some(
            "\nDispatching to Snaga.\n```pfterminal-send-task\ntarget: Snaga\n".to_string(),
        ),
        hint: None,
        elapsed_ms: 30_000,
        artifact_path: artifact_path.clone(),
        audit_path: audit_path.clone(),
    };
    let status = registry
        .update_live_progress(&partial_dispatch)
        .expect("partial dispatch status");
    let details = status.details.expect("details");
    assert!(details.contains("Dispatching to Snaga."));
    assert!(!details.contains("pfterminal-send-task"));
    assert!(!details.contains("target: Snaga"));

    let complete_dispatch = ClaudePaneTurnProgress {
        pane_id,
        phase: "assistant-text".to_string(),
        summary: "Claude assistant text.".to_string(),
        assistant_text_delta: Some(
            "task:\nbuild site\n```\nBack to reviewing the result.".to_string(),
        ),
        hint: None,
        elapsed_ms: 35_000,
        artifact_path,
        audit_path,
    };
    let status = registry
        .update_live_progress(&complete_dispatch)
        .expect("complete dispatch status");
    let details = status.details.expect("details");
    assert!(details.contains("Back to reviewing the result."));
    assert!(!details.contains("pfterminal-send-task"));
    assert!(!details.contains("target: Snaga"));
    assert!(!details.contains("build site"));
}

#[test]
fn live_status_panel_does_not_slice_assistant_notes_mid_sentence() {
    let dir = tempfile::tempdir().expect("tempdir");
    let mut registry = ClaudePaneRegistry::new();
    let pane_id = registry
        .create_pane(
            ClaudeProviderProfileKind::ClaudePlan,
            std::env::current_dir().expect("cwd"),
            dir.path(),
        )
        .expect("pane");
    let artifact_path = dir.path().join("turn-0001.jsonl");
    let audit_path = dir.path().join("turn-0001.audit.json");
    let repeated = "I checked the RPC allow flags; it succeeded. ".repeat(12);
    let commentary = format!(
        "{repeated}Now npm run build failed because the command ran from the wrong directory. \
             JS tests passed. Python tests failed to find the path, so I am switching cwd before retrying."
    );

    let progress = ClaudePaneTurnProgress {
        pane_id,
        phase: "assistant-text".to_string(),
        summary: "Claude assistant text.".to_string(),
        assistant_text_delta: Some(commentary),
        hint: None,
        elapsed_ms: 69_000,
        artifact_path,
        audit_path,
    };
    let status = registry.update_live_progress(&progress).expect("status");
    assert_eq!(status.header, "Claude running · 1m09s");
    let details = status.details.expect("details");
    assert!(details.contains("Claude notes:"));
    assert!(
        details
            .contains("Now npm run build failed because the command ran from the wrong directory.")
    );
    assert!(details.contains("Python tests failed to find the path"));
    assert!(!details.contains("Current: Claude: s; it succeeded"));
    assert!(!details.contains("\n  s; it succeeded"));
    assert!(!details.contains("artifact:"));
    assert!(!details.contains("audit:"));
}

#[test]
fn live_status_panel_tracks_tools_without_artifact_log_spam() {
    let dir = tempfile::tempdir().expect("tempdir");
    let mut registry = ClaudePaneRegistry::new();
    let pane_id = registry
        .create_pane(
            ClaudeProviderProfileKind::ClaudePlan,
            std::env::current_dir().expect("cwd"),
            dir.path(),
        )
        .expect("pane");
    let artifact_path = dir.path().join("turn-0001.jsonl");
    let audit_path = dir.path().join("turn-0001.audit.json");

    let first_tool = ClaudePaneTurnProgress {
        pane_id: pane_id.clone(),
        phase: "tool-call".to_string(),
        summary:
            "Claude tool call: Bash: Create directory for the mock donkey riding course website"
                .to_string(),
        assistant_text_delta: None,
        hint: None,
        elapsed_ms: 30_000,
        artifact_path: artifact_path.clone(),
        audit_path: audit_path.clone(),
    };
    let status = registry.update_live_progress(&first_tool).expect("status");
    assert_eq!(status.header, "Claude running · 30s");
    let details = status.details.expect("details");
    assert!(
        details
            .contains("Current: Bash: Create directory for the mock donkey riding course website")
    );
    assert!(
        details
            .contains("running Bash: Create directory for the mock donkey riding course website")
    );
    assert!(!details.contains("artifact:"));
    assert!(!details.contains("audit:"));

    let heartbeat = ClaudePaneTurnProgress {
        pane_id: pane_id.clone(),
        phase: "waiting".to_string(),
        summary: "Claude running.".to_string(),
        assistant_text_delta: None,
        hint: None,
        elapsed_ms: 90_000,
        artifact_path: artifact_path.clone(),
        audit_path: audit_path.clone(),
    };
    let status = registry.update_live_progress(&heartbeat).expect("status");
    assert_eq!(status.header, "Claude running · 1m30s");
    let details = status.details.expect("details");
    assert!(
        details
            .contains("Current: Bash: Create directory for the mock donkey riding course website")
    );
    assert!(!details.contains("Claude pane still running"));

    let second_tool = ClaudePaneTurnProgress {
        pane_id: pane_id.clone(),
        phase: "tool-call".to_string(),
        summary: "Claude tool call: Bash: Write the donkey riding course mock website HTML file"
            .to_string(),
        assistant_text_delta: None,
        hint: None,
        elapsed_ms: 150_000,
        artifact_path: artifact_path.clone(),
        audit_path: audit_path.clone(),
    };
    let status = registry.update_live_progress(&second_tool).expect("status");
    let details = status.details.expect("details");
    assert!(
        details
            .contains("done    Bash: Create directory for the mock donkey riding course website")
    );
    assert!(
        details.contains("running Bash: Write the donkey riding course mock website HTML file")
    );

    let result = ClaudePaneTurnProgress {
        pane_id,
        phase: "assistant-result".to_string(),
        summary: "Claude returned a result.".to_string(),
        assistant_text_delta: None,
        hint: None,
        elapsed_ms: 180_000,
        artifact_path,
        audit_path,
    };
    let status = registry.update_live_progress(&result).expect("status");
    let details = status.details.expect("details");
    assert!(details.contains("Current: finalizing result"));
    assert!(
        details.contains("done    Bash: Write the donkey riding course mock website HTML file")
    );
}

#[test]
fn live_status_panel_tracks_reasoning_without_artifact_log_spam() {
    let dir = tempfile::tempdir().expect("tempdir");
    let mut registry = ClaudePaneRegistry::new();
    let pane_id = registry
        .create_pane(
            ClaudeProviderProfileKind::ClaudePlan,
            std::env::current_dir().expect("cwd"),
            dir.path(),
        )
        .expect("pane");
    let artifact_path = dir.path().join("turn-0001.jsonl");
    let audit_path = dir.path().join("turn-0001.audit.json");

    let reasoning = ClaudePaneTurnProgress {
        pane_id,
        phase: "reasoning".to_string(),
        summary: "Claude reasoning: Inspect the hierarchy before asking Orcs to execute."
            .to_string(),
        assistant_text_delta: None,
        hint: None,
        elapsed_ms: 12_000,
        artifact_path,
        audit_path,
    };
    let status = registry.update_live_progress(&reasoning).expect("status");
    assert_eq!(status.header, "Claude running · 12s");
    let details = status.details.expect("details");
    assert!(
        details.contains("Current: thinking: Inspect the hierarchy before asking Orcs to execute.")
    );
    assert!(details.contains("Thinking:"));
    assert!(details.contains("Inspect the hierarchy before asking Orcs to execute."));
    assert!(!details.contains("artifact:"));
    assert!(!details.contains("audit:"));
}

#[test]
fn live_status_panel_shows_thinking_tokens_and_marks_prior_tool_done() {
    let dir = tempfile::tempdir().expect("tempdir");
    let mut registry = ClaudePaneRegistry::new();
    let pane_id = registry
        .create_pane(
            ClaudeProviderProfileKind::ClaudePlan,
            std::env::current_dir().expect("cwd"),
            dir.path(),
        )
        .expect("pane");
    let artifact_path = dir.path().join("turn-0001.jsonl");
    let audit_path = dir.path().join("turn-0001.audit.json");

    let tool = ClaudePaneTurnProgress {
        pane_id: pane_id.clone(),
        phase: "tool-call".to_string(),
        summary: "Claude tool call: Bash: Run all Rust tests".to_string(),
        assistant_text_delta: None,
        hint: None,
        elapsed_ms: 30_000,
        artifact_path: artifact_path.clone(),
        audit_path: audit_path.clone(),
    };
    registry.update_live_progress(&tool).expect("tool status");

    let thinking = ClaudePaneTurnProgress {
        pane_id,
        phase: "reasoning-tokens".to_string(),
        summary: "Claude reasoning: thinking: 3.1K reasoning tokens".to_string(),
        assistant_text_delta: None,
        hint: Some("thinking-token-bucket:31".to_string()),
        elapsed_ms: 90_000,
        artifact_path,
        audit_path,
    };
    let status = registry
        .update_live_progress(&thinking)
        .expect("thinking status");

    assert_eq!(status.header, "Claude running · 1m30s");
    let details = status.details.expect("details");
    assert!(details.contains("Current: thinking: 3.1K reasoning tokens"));
    assert!(details.contains("Thinking:"));
    assert!(details.contains("thinking: 3.1K reasoning tokens"));
    assert!(details.contains("done    Bash: Run all Rust tests"));
    assert!(!details.contains("running Bash: Run all Rust tests"));
    assert!(!details.contains("session initialized"));
}

#[test]
fn vercel_profiles_are_creation_options() {
    assert!(
        ClaudeProviderProfileKind::creation_options()
            .contains(&ClaudeProviderProfileKind::VercelGlm52)
    );
    assert!(
        ClaudeProviderProfileKind::creation_options()
            .contains(&ClaudeProviderProfileKind::VercelGlm52Fast)
    );
}

#[test]
fn top_level_new_pane_items_are_collapsed() {
    let items = new_pane_items();
    let names = items
        .iter()
        .map(|item| item.name.as_str())
        .collect::<Vec<_>>();

    assert_eq!(names, vec!["+ Corbanu Terminal Pane", "+ Claude Pane"]);
    assert!(
        names
            .iter()
            .all(|name| !name.contains("GLM") && !name.contains("Vercel")),
        "top-level /panes must not list provider-specific Claude rows"
    );
}

#[test]
fn vercel_profile_settings_use_ai_gateway_anthropic_endpoint() {
    let profile = ClaudeProviderProfileKind::VercelGlm52.profile();
    let settings =
        settings_json_with_base_url(profile, Some("corbanu"), /*base_url_override*/ None);

    assert_eq!(
        settings.pointer("/env/ANTHROPIC_BASE_URL"),
        Some(&json!("https://ai-gateway.vercel.sh"))
    );
    assert_eq!(
        settings.pointer("/env/ANTHROPIC_DEFAULT_OPUS_MODEL"),
        Some(&json!("zai/glm-5.2"))
    );
    assert_eq!(
        settings.pointer("/env/ANTHROPIC_DEFAULT_HAIKU_MODEL"),
        Some(&json!("zai/glm-5.2-fast"))
    );
    assert_eq!(
        settings.pointer("/apiKeyHelper"),
        Some(&json!(
            "corbanu vault auth-helper provider/ai_gateway_api_key"
        ))
    );
}

#[test]
fn vercel_fast_profile_uses_fast_model_for_all_claude_aliases() {
    let profile = ClaudeProviderProfileKind::VercelGlm52Fast.profile();
    let settings =
        settings_json_with_base_url(profile, Some("corbanu"), /*base_url_override*/ None);

    assert_eq!(
        settings.pointer("/env/ANTHROPIC_DEFAULT_OPUS_MODEL"),
        Some(&json!("zai/glm-5.2-fast"))
    );
    assert_eq!(
        settings.pointer("/env/ANTHROPIC_DEFAULT_SONNET_MODEL"),
        Some(&json!("zai/glm-5.2-fast"))
    );
    assert_eq!(
        settings.pointer("/env/ANTHROPIC_DEFAULT_HAIKU_MODEL"),
        Some(&json!("zai/glm-5.2-fast"))
    );
}

#[test]
fn vercel_fast_command_plan_uses_count_tokens_passthrough_bridge() {
    let (dir, pane) = pane(ClaudeProviderProfileKind::VercelGlm52Fast);
    codex_vault::Vault::new(dir.path().to_path_buf())
        .add(codex_vault::AddCredential {
            label: "provider/ai_gateway_api_key".to_string(),
            credential_type: codex_vault::CredentialType::ApiKey,
            provider: Some("vercel".to_string()),
            notes: None,
            revocation_notes: None,
            secret: "vercel-test-key".to_string(),
        })
        .expect("store test Vercel key");

    let plan = build_claude_command_plan(&pane, "hello".to_string(), dir.path()).expect("plan");
    let settings = std::fs::read_to_string(pane.artifact_dir.join("settings.json"))
        .expect("settings should be written");
    let settings: Value = serde_json::from_str(&settings).expect("settings json");
    let bridge = plan.bridge.as_ref().expect("Vercel should use bridge");

    assert_eq!(bridge.kind, ClaudeBridgeKind::AnthropicPassthrough);
    assert_eq!(bridge.upstream_base_url, "https://ai-gateway.vercel.sh");
    assert!(bridge.upstream_api_key.is_none());
    assert_eq!(
        bridge
            .deferred_vault_secret
            .as_ref()
            .map(|secret| secret.label.as_str()),
        Some("provider/ai_gateway_api_key")
    );
    let client_auth_token = plan
        .env
        .get("ANTHROPIC_AUTH_TOKEN")
        .expect("per-turn bridge credential");
    assert_eq!(client_auth_token, &bridge.client_auth_token);
    assert!(Uuid::parse_str(client_auth_token).is_ok());
    assert!(
        plan.env_remove
            .iter()
            .any(|key| key == "ANTHROPIC_AUTH_TOKEN")
    );
    assert!(
        settings
            .pointer("/env/ANTHROPIC_BASE_URL")
            .and_then(Value::as_str)
            .is_some_and(|base_url| base_url.starts_with("http://127.0.0.1:"))
    );
    assert!(
        !plan
            .env
            .values()
            .any(|value| value.contains("vercel-test-key"))
    );
    assert!(!plan.args.iter().any(|arg| arg.contains("vercel-test-key")));
    assert!(!settings.to_string().contains("vercel-test-key"));
}

#[test]
fn ambient_kimi_profile_uses_ambient_bridge_model() {
    let (dir, pane) = pane(ClaudeProviderProfileKind::AmbientKimiK27);
    codex_vault::Vault::new(dir.path().to_path_buf())
        .add(codex_vault::AddCredential {
            label: "provider/ambient_api_key".to_string(),
            credential_type: codex_vault::CredentialType::ApiKey,
            provider: Some("ambient".to_string()),
            notes: None,
            revocation_notes: None,
            secret: "ambient-test-key".to_string(),
        })
        .expect("store test Ambient key");

    let plan = build_claude_command_plan(&pane, "hello".to_string(), dir.path()).expect("plan");
    let settings = std::fs::read_to_string(pane.artifact_dir.join("settings.json"))
        .expect("settings should be written");
    let settings: Value = serde_json::from_str(&settings).expect("settings json");
    let bridge = plan.bridge.as_ref().expect("Ambient should use bridge");

    assert_eq!(bridge.kind, ClaudeBridgeKind::AmbientChat);
    assert_eq!(bridge.upstream_model, AMBIENT_KIMI_K2_7_CODE_MODEL);
    assert!(bridge.upstream_api_key.is_none());
    assert_eq!(
        bridge
            .deferred_vault_secret
            .as_ref()
            .map(|secret| secret.label.as_str()),
        Some("provider/ambient_api_key")
    );
    assert_eq!(
        settings.pointer("/env/ANTHROPIC_DEFAULT_OPUS_MODEL"),
        Some(&json!(AMBIENT_KIMI_K2_7_CODE_MODEL))
    );
    assert_eq!(
        settings.pointer("/env/ANTHROPIC_DEFAULT_HAIKU_MODEL"),
        Some(&json!(AMBIENT_KIMI_K2_7_CODE_MODEL))
    );
    assert!(!plan.args.iter().any(|arg| arg.contains("ambient-test-key")));
    assert!(!settings.to_string().contains("ambient-test-key"));
}

#[test]
fn ambient_glm_profile_uses_native_ambient_model_slug() {
    let (dir, pane) = pane(ClaudeProviderProfileKind::AmbientGlm52);
    codex_vault::Vault::new(dir.path().to_path_buf())
        .add(codex_vault::AddCredential {
            label: "provider/ambient_api_key".to_string(),
            credential_type: codex_vault::CredentialType::ApiKey,
            provider: Some("ambient".to_string()),
            notes: None,
            revocation_notes: None,
            secret: "ambient-test-key".to_string(),
        })
        .expect("store test Ambient key");

    let plan = build_claude_command_plan(&pane, "hello".to_string(), dir.path()).expect("plan");
    let bridge = plan.bridge.as_ref().expect("Ambient should use bridge");

    assert_eq!(plan.provider_model, AMBIENT_DEFAULT_MODEL);
    assert_eq!(bridge.upstream_model, AMBIENT_DEFAULT_MODEL);
}

#[test]
fn bridge_command_plan_defers_vault_reveal() {
    let (dir, pane) = pane(ClaudeProviderProfileKind::AmbientGlm52);

    let plan = build_claude_command_plan(&pane, "hello".to_string(), dir.path())
        .expect("planning must not read the vault");
    let bridge = plan.bridge.expect("bridge plan");

    assert!(bridge.upstream_api_key.is_none());
    assert_eq!(
        bridge
            .deferred_vault_secret
            .as_ref()
            .map(|secret| secret.label.as_str()),
        Some("provider/ambient_api_key")
    );
}

#[cfg(unix)]
fn bridge_redaction_plan(
    dir: &tempfile::TempDir,
    command: String,
    secret: &str,
) -> ClaudeCommandPlan {
    let listener = std::net::TcpListener::bind("127.0.0.1:0").expect("bind listener");
    listener
        .set_nonblocking(true)
        .expect("nonblocking listener");
    let bind_addr = listener.local_addr().expect("listener address");
    ClaudeCommandPlan {
        executable: "sh".to_string(),
        args: vec!["-c".to_string(), command],
        env: BTreeMap::new(),
        env_remove: Vec::new(),
        cwd: dir.path().to_path_buf(),
        pane_id: "claude-redaction-test".to_string(),
        pane_title: "Claude Redaction Test".to_string(),
        profile_title: "Claude Redaction Test".to_string(),
        provider_model: "test-model".to_string(),
        turn_index: 1,
        command_mode: ClaudeCommandMode::NewSession,
        command_session_id: "11111111-1111-4111-8111-111111111111".to_string(),
        max_turns: None,
        artifact_path: dir.path().join("turn-0001.jsonl"),
        audit_path: dir.path().join("turn-0001.audit.json"),
        timeout_ms: None,
        deferred_claude_plan_auth: None,
        bridge: Some(ClaudeBridgePlan {
            kind: ClaudeBridgeKind::AnthropicPassthrough,
            listener,
            bind_addr,
            client_auth_token: "bridge-test-client-token".to_string(),
            upstream_base_url: "https://example.invalid".to_string(),
            upstream_api_key: Some(secret.to_string()),
            deferred_vault_secret: None,
            upstream_model: "test-model".to_string(),
        }),
    }
}

#[cfg(unix)]
#[test]
fn claude_secret_redactor_redacts_bridge_credentials() {
    let dir = tempfile::tempdir().expect("tempdir");
    let plan = bridge_redaction_plan(&dir, "true".to_string(), "bridge-secret-for-redaction-test");

    let redacted = ClaudeSecretRedactor::from_plan(&plan, /*additional_secret*/ None)
        .redact("leaked bridge-secret-for-redaction-test and bridge-test-client-token");

    assert_eq!(redacted, "leaked [REDACTED_SECRET] and [REDACTED_SECRET]");
}

#[cfg(unix)]
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn claude_plan_pane_brokers_selected_auth_without_inheriting_the_token() {
    let (dir, mut pane) = pane(ClaudeProviderProfileKind::ClaudePlan);
    let pane_cwd = dir.path().join("pane-cwd");
    std::fs::create_dir(&pane_cwd).expect("pane cwd");
    pane.cwd = pane_cwd.clone();
    let secret = "pane-auth-secret-canary-not-real";
    let helper = dir.path().join("auth-helper");
    let helper_home_log = dir.path().join("auth-helper-home");
    std::fs::write(
        &helper,
        format!(
            "#!/bin/sh\nprintf '%s\\n' \"$PWD\" \"$CORBANU_HOME\" \"$PFTERMINAL_HOME\" \"$CODEX_HOME\" > '{}'\nprintf '%s' '{secret}'\n",
            helper_home_log.display()
        ),
    )
    .expect("write helper");
    let claude = dir.path().join("claude");
    std::fs::write(
        &claude,
        concat!(
            "#!/bin/sh\nset -eu\n",
            "[ -z \"${CLAUDE_CODE_OAUTH_TOKEN+x}\" ] || exit 7\n",
            "[ -n \"$ANTHROPIC_AUTH_TOKEN\" ] || exit 8\n",
            "[ \"$ANTHROPIC_AUTH_TOKEN\" != \"pane-auth-secret-canary-not-real\" ] || exit 11\n",
            "case \"$ANTHROPIC_BASE_URL\" in http://127.0.0.1:*) ;; *) exit 9 ;; esac\n",
            "env | grep -F 'pane-auth-secret-canary-not-real' >/dev/null && exit 10\n",
            "printf '{\"type\":\"result\",\"subtype\":\"success\",",
            "\"session_id\":\"11111111-1111-4111-8111-111111111111\",",
            "\"result\":\"bound through local bridge\"}\\n'\n",
        ),
    )
    .expect("write claude");
    for executable in [&helper, &claude] {
        let mut permissions = std::fs::metadata(executable)
            .expect("executable metadata")
            .permissions();
        permissions.set_mode(0o700);
        std::fs::set_permissions(executable, permissions).expect("make executable");
    }
    let mut plan = build_claude_command_plan(&pane, "hello".to_string(), dir.path()).expect("plan");
    assert!(!plan.args.iter().any(|arg| arg.contains(secret)));
    assert!(!plan.env.values().any(|value| value.contains(secret)));
    assert!(
        plan.env_remove
            .iter()
            .any(|key| key == "CLAUDE_CODE_OAUTH_TOKEN")
    );
    for key in [
        "ANTHROPIC_API_KEY",
        "ANTHROPIC_AUTH_TOKEN",
        "ANTHROPIC_BASE_URL",
        "CLAUDE_CODE_USE_BEDROCK",
        "CLAUDE_CODE_USE_VERTEX",
        "CLAUDE_CODE_USE_FOUNDRY",
    ] {
        assert!(plan.env_remove.iter().any(|removed| removed == key));
    }
    let deferred = plan
        .deferred_claude_plan_auth
        .as_mut()
        .expect("deferred selected auth");
    deferred.helper_executable = helper;
    let bridge = plan.bridge.as_ref().expect("Claude Plan bridge");
    assert!(bridge.upstream_api_key.is_none());
    assert!(bridge.deferred_vault_secret.is_none());
    plan.executable = claude.to_string_lossy().into_owned();
    let artifact_path = plan.artifact_path.clone();

    let output = run_claude_command_plan(plan, CancellationToken::new(), /*progress_tx*/ None)
        .await
        .expect("turn output");
    let artifact = std::fs::read_to_string(artifact_path).expect("artifact");

    assert_eq!(output.status, ClaudePaneTurnStatus::Success);
    assert_eq!(output.text, "bound through local bridge");
    assert!(!artifact.contains(secret));
    let helper_log = std::fs::read_to_string(helper_home_log).expect("helper home log");
    let mut helper_log = helper_log.lines();
    let helper_cwd = PathBuf::from(helper_log.next().expect("helper cwd"));
    assert_eq!(
        std::fs::canonicalize(helper_cwd).expect("canonical helper cwd"),
        std::fs::canonicalize(&pane_cwd).expect("canonical pane cwd")
    );
    let home = dir.path().to_string_lossy().into_owned();
    assert_eq!(helper_log.collect::<Vec<_>>(), vec![home.as_str(); 3]);
}

#[cfg(unix)]
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn unavailable_selected_auth_fails_before_claude_pane_spawn_without_disclosure() {
    let (dir, pane) = pane(ClaudeProviderProfileKind::ClaudePlan);
    let secret = "pane-auth-failure-secret-not-real";
    let helper = dir.path().join("auth-helper-failure");
    std::fs::write(
        &helper,
        format!("#!/bin/sh\nprintf '%s' '{secret}'\nexit 1\n"),
    )
    .expect("write helper");
    let spawn_marker = dir.path().join("claude-started");
    let claude = dir.path().join("claude-failure");
    std::fs::write(
        &claude,
        format!("#!/bin/sh\ntouch '{}'\n", spawn_marker.display()),
    )
    .expect("write claude");
    for executable in [&helper, &claude] {
        let mut permissions = std::fs::metadata(executable)
            .expect("executable metadata")
            .permissions();
        permissions.set_mode(0o700);
        std::fs::set_permissions(executable, permissions).expect("make executable");
    }
    let mut plan = build_claude_command_plan(&pane, "hello".to_string(), dir.path()).expect("plan");
    plan.deferred_claude_plan_auth
        .as_mut()
        .expect("deferred selected auth")
        .helper_executable = helper;
    plan.executable = claude.to_string_lossy().into_owned();

    let error = run_claude_command_plan(plan, CancellationToken::new(), /*progress_tx*/ None)
        .await
        .expect_err("unavailable selected auth must fail before spawn");

    assert!(error.to_string().contains("open Providers to recover"));
    assert!(!error.to_string().contains(secret));
    assert!(!spawn_marker.exists());
}

#[cfg(unix)]
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn cancelling_deferred_auth_kills_helper_and_never_spawns_claude() {
    let (dir, pane) = pane(ClaudeProviderProfileKind::ClaudePlan);
    let helper_started = dir.path().join("helper-started");
    let helper_finished = dir.path().join("helper-finished");
    let helper = dir.path().join("auth-helper-slow");
    std::fs::write(
        &helper,
        format!(
            "#!/bin/sh\ntouch '{}'\nsleep 30\ntouch '{}'\nprintf token\n",
            helper_started.display(),
            helper_finished.display()
        ),
    )
    .expect("write helper");
    let claude_started = dir.path().join("claude-started");
    let claude = dir.path().join("claude-sentinel");
    std::fs::write(
        &claude,
        format!("#!/bin/sh\ntouch '{}'\n", claude_started.display()),
    )
    .expect("write claude");
    for executable in [&helper, &claude] {
        let mut permissions = std::fs::metadata(executable)
            .expect("executable metadata")
            .permissions();
        permissions.set_mode(0o700);
        std::fs::set_permissions(executable, permissions).expect("make executable");
    }
    let mut plan = build_claude_command_plan(&pane, "hello".to_string(), dir.path()).expect("plan");
    plan.deferred_claude_plan_auth
        .as_mut()
        .expect("deferred selected auth")
        .helper_executable = helper;
    plan.executable = claude.to_string_lossy().into_owned();
    let cancellation = CancellationToken::new();
    let task_cancellation = cancellation.clone();
    let turn = tokio::spawn(async move {
        run_claude_command_plan(plan, task_cancellation, /*progress_tx*/ None).await
    });
    for _ in 0..100 {
        if helper_started.exists() {
            break;
        }
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
    assert!(helper_started.exists(), "auth helper did not start");

    cancellation.cancel();
    let output = tokio::time::timeout(Duration::from_secs(3), turn)
        .await
        .expect("cancelled auth returned promptly")
        .expect("turn task")
        .expect("turn output");
    tokio::time::sleep(Duration::from_millis(200)).await;

    assert_eq!(output.status, ClaudePaneTurnStatus::Interrupted);
    assert_eq!(
        output.terminal_reason.as_deref(),
        Some("interrupted_during_auth")
    );
    assert!(!helper_finished.exists());
    assert!(!claude_started.exists());
}

#[cfg(unix)]
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn bridge_secret_is_redacted_from_stdout_artifact_and_output() {
    let dir = tempfile::tempdir().expect("tempdir");
    let secret = "vercel-test-key-for-redaction";
    let line = json!({
        "type": "result",
        "subtype": "success",
        "session_id": "11111111-1111-4111-8111-111111111111",
        "result": format!("leaked {secret}"),
    })
    .to_string();
    let plan = bridge_redaction_plan(&dir, format!("printf '%s\\n' '{line}'"), secret);
    let artifact_path = plan.artifact_path.clone();

    let output = run_claude_command_plan(plan, CancellationToken::new(), /*progress_tx*/ None)
        .await
        .expect("turn output");
    let artifact = std::fs::read_to_string(artifact_path).expect("artifact");

    assert_eq!(output.status, ClaudePaneTurnStatus::Success);
    assert_eq!(output.text, "leaked [REDACTED_SECRET]");
    assert!(!artifact.contains(secret));
    assert!(artifact.contains("[REDACTED_SECRET]"));
}

#[cfg(unix)]
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn bridge_secret_is_redacted_from_stderr_failure_and_audit() {
    let dir = tempfile::tempdir().expect("tempdir");
    let secret = "vercel-test-key-for-stderr-redaction";
    let plan = bridge_redaction_plan(
        &dir,
        format!("printf '%s\\n' 'upstream leaked {secret}' >&2; exit 42"),
        secret,
    );
    let audit_path = plan.audit_path.clone();

    let output = run_claude_command_plan(plan, CancellationToken::new(), /*progress_tx*/ None)
        .await
        .expect("turn output");
    let audit = std::fs::read_to_string(audit_path).expect("audit");

    assert_eq!(output.status, ClaudePaneTurnStatus::ProviderError);
    assert_eq!(
        output.error_summary.as_deref(),
        Some("Claude exited with status exit status: 42: upstream leaked [REDACTED_SECRET]")
    );
    assert!(!audit.contains(secret));
    assert!(audit.contains("[REDACTED_SECRET]"));
}

#[test]
fn claude_provider_picker_labels_are_compact() {
    assert_eq!(
        ClaudeProviderProfileKind::AmbientGlm52.status_model_label(),
        "GLM 5.2 Ambient"
    );
    assert_eq!(
        ClaudeProviderProfileKind::AmbientKimiK27.status_model_label(),
        "Kimi K2.7 Ambient"
    );
    assert_eq!(
        ClaudeProviderProfileKind::ClaudePlan.status_model_label(),
        "Opus 5 Claude Plan"
    );
}

#[test]
fn smoke_provider_profile_accepts_vercel_aliases() {
    assert_eq!(
        smoke_provider_profile("vercel"),
        Some(ClaudeProviderProfileKind::VercelGlm52)
    );
    assert_eq!(
        smoke_provider_profile("vercel-glm-52-fast"),
        Some(ClaudeProviderProfileKind::VercelGlm52Fast)
    );
}

#[test]
fn smoke_provider_profile_accepts_kimi_aliases() {
    assert_eq!(
        smoke_provider_profile("ambient-kimi-k2-7"),
        Some(ClaudeProviderProfileKind::AmbientKimiK27)
    );
    assert_eq!(
        smoke_provider_profile("kimi-k27"),
        Some(ClaudeProviderProfileKind::AmbientKimiK27)
    );
}

#[test]
fn ambient_profile_is_first_creation_option() {
    assert_eq!(
        ClaudeProviderProfileKind::creation_options()
            .first()
            .copied(),
        Some(ClaudeProviderProfileKind::AmbientGlm52)
    );
    assert!(
        ClaudeProviderProfileKind::creation_options()
            .contains(&ClaudeProviderProfileKind::AmbientKimiK27)
    );
}

#[test]
fn parse_single_json_output() {
    let parsed = parse_claude_output(
            r#"{"type":"result","result":"stored.","session_id":"11111111-2222-4333-8444-555555555555","usage":{"input_tokens":12,"output_tokens":3}}"#,
        )
        .expect("parse");

    assert_eq!(parsed.text, "stored.");
    assert_eq!(parsed.status, ClaudePaneTurnStatus::Success);
    assert_eq!(
        parsed.session_id.as_deref(),
        Some("11111111-2222-4333-8444-555555555555")
    );
    assert_eq!(
        parsed.usage_summary.as_deref(),
        Some(r#"{"input_tokens":12,"output_tokens":3}"#)
    );
}

#[test]
fn parse_stream_json_output_prefers_final_result() {
    let parsed = parse_claude_output(
            r#"{"type":"assistant","message":{"content":[{"type":"text","text":"he"}],"usage":{"input_tokens":1}}}
{"type":"assistant","message":{"content":[{"type":"text","text":"llo"}]}}
{"type":"result","result":"hello","session_id":"22222222-2222-4222-8222-222222222222"}"#,
        )
        .expect("parse");

    assert_eq!(parsed.text, "hello");
    assert_eq!(parsed.status, ClaudePaneTurnStatus::Success);
    assert_eq!(
        parsed.session_id.as_deref(),
        Some("22222222-2222-4222-8222-222222222222")
    );
    assert_eq!(
        parsed.usage_summary.as_deref(),
        Some(r#"{"input_tokens":1}"#)
    );
}

#[test]
fn parse_stream_json_without_final_result_is_incomplete() {
    let error = parse_claude_output(
            r#"{"type":"system","subtype":"init","session_id":"22222222-2222-4222-8222-222222222222"}
{"type":"assistant","message":{"content":[{"type":"text","text":"Now let me restart the dev server and run the full test:"}]},"session_id":"22222222-2222-4222-8222-222222222222"}
{"type":"assistant","message":{"content":[{"type":"tool_use","id":"call_restart","name":"Bash","input":{"command":"pkill -f vite","description":"Kill old vite processes"}}]},"session_id":"22222222-2222-4222-8222-222222222222"}"#,
        )
        .expect_err("dangling tool call without final result should not be success");

    assert!(
        error
            .to_string()
            .contains("ended before a final result event")
    );
}

#[test]
fn parse_stream_json_provider_error_is_structured() {
    let parsed = parse_claude_output(
            r#"{"type":"system","subtype":"init","session_id":"22222222-2222-4222-8222-222222222222"}
{"type":"result","subtype":"success","is_error":true,"result":"API Error: [1305][temporarily overloaded]","session_id":"22222222-2222-4222-8222-222222222222"}"#,
        )
        .expect("provider error should still produce a structured pane result");

    assert_eq!(parsed.status, ClaudePaneTurnStatus::ProviderError);
    assert_eq!(
        parsed.session_id.as_deref(),
        Some("22222222-2222-4222-8222-222222222222")
    );
    assert!(
        parsed
            .error_summary
            .as_deref()
            .unwrap_or_default()
            .contains("temporarily overloaded")
    );
}

#[test]
fn parse_stream_json_max_turns_is_resumable_pause() {
    let parsed = parse_claude_output(
            r#"{"type":"system","subtype":"init","session_id":"33333333-3333-4333-8333-333333333333"}
{"type":"assistant","message":{"content":[{"type":"tool_use","name":"Read","input":{"file_path":"README.md"}}],"usage":{"input_tokens":42}}}
{"type":"result","subtype":"error_max_turns","is_error":true,"terminal_reason":"max_turns","result":"Reached maximum number of turns (8)","session_id":"33333333-3333-4333-8333-333333333333"}"#,
        )
        .expect("max-turn should be parsed as a structured pause");

    assert_eq!(parsed.status, ClaudePaneTurnStatus::MaxTurnsPause);
    assert_eq!(
        parsed.session_id.as_deref(),
        Some("33333333-3333-4333-8333-333333333333")
    );
    assert_eq!(parsed.terminal_reason.as_deref(), Some("max_turns"));
    assert_eq!(parsed.tool_names, vec!["Read"]);
}

#[test]
fn zero_usage_summary_is_untrusted_not_reported() {
    assert_eq!(
        usage_status_from_summary(Some(r#"{"input_tokens":0,"output_tokens":0}"#)),
        ClaudePaneUsageStatus::Untrusted
    );
    assert_eq!(
        usage_status_from_summary(Some(r#"{"input_tokens":10,"output_tokens":0}"#)),
        ClaudePaneUsageStatus::Untrusted
    );
    assert_eq!(
        usage_status_from_summary(Some(r#"{"input_tokens":10,"output_tokens":1}"#)),
        ClaudePaneUsageStatus::Reported
    );
}

#[test]
fn timeout_pause_failure_message_is_not_provider_error() {
    let (dir, pane) = pane(ClaudeProviderProfileKind::ClaudePlan);
    let plan = build_claude_command_plan(&pane, "hello".to_string(), dir.path()).expect("plan");
    let output = failed_turn_output(
        &plan,
        /*duration_ms*/ 150_000,
        ClaudePaneTurnStatus::TimeoutPause,
        Some("timeout".to_string()),
        "local timeout".to_string(),
    );

    assert!(output.failure_message().contains("timed out locally"));
    assert!(!output.failure_message().contains("provider error"));
}

#[test]
fn interrupt_turn_cancels_prepared_claude_token_and_finishes_cleanly() {
    let dir = tempfile::tempdir().expect("tempdir");
    let mut registry = ClaudePaneRegistry::new();
    let pane_id = registry
        .create_pane(
            ClaudeProviderProfileKind::ClaudePlan,
            std::env::current_dir().expect("cwd"),
            dir.path(),
        )
        .expect("pane");

    let prepared = registry
        .prepare_turn(&pane_id, "long running task".to_string(), dir.path())
        .expect("prepared");
    let cancel_token = prepared.cancel_token.clone();
    assert!(!cancel_token.is_cancelled());
    assert!(registry.interrupt_turn(&pane_id).is_ok());
    assert!(cancel_token.is_cancelled());
    assert!(
        registry
            .prepare_turn(&pane_id, "overlap".to_string(), dir.path())
            .is_err(),
        "interrupted turns remain running until the child process exits"
    );
    drop(prepared);

    let result = Ok(ClaudePaneTurnOutput {
        text: String::new(),
        status: ClaudePaneTurnStatus::Interrupted,
        session_id: None,
        usage_summary: None,
        usage_status: ClaudePaneUsageStatus::Missing,
        artifact_path: dir.path().join("turn-0001.jsonl"),
        audit_path: dir.path().join("turn-0001.audit.json"),
        duration_ms: 1,
        terminal_reason: Some("interrupted".to_string()),
        error_summary: Some("Claude pane turn interrupted by user.".to_string()),
        tool_names: Vec::new(),
        tool_events: Vec::new(),
        reasoning_events: Vec::new(),
        command_mode: ClaudeCommandMode::NewSession,
    });
    registry.finish_turn(&pane_id, &result);

    let pane = registry
        .panes()
        .iter()
        .find(|pane| pane.id == pane_id)
        .expect("pane exists");
    assert_eq!(pane.status, ClaudePaneStatus::Idle);
    assert!(pane.cancel_token.is_none());
    assert_eq!(
        pane.latest_turn_status,
        Some(ClaudePaneTurnStatus::Interrupted)
    );

    let next = registry
        .prepare_turn(&pane_id, "next task".to_string(), dir.path())
        .expect("next turn");
    assert!(!next.cancel_token.is_cancelled());
}

#[cfg(unix)]
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn stop_claude_child_reaps_running_process() {
    let mut command = Command::new("sh");
    command.args(["-c", "sleep 60"]);
    command.kill_on_drop(true);
    command.process_group(0);
    let mut child = command.spawn().expect("spawn sleep");

    stop_claude_child(&mut child)
        .await
        .expect("child should be killed and reaped");

    assert!(child.try_wait().expect("query child status").is_some());
}

#[cfg(target_os = "linux")]
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn stop_claude_child_kills_detached_tool_process_group() {
    let dir = tempfile::tempdir().expect("tempdir");
    let detached_pid_path = dir.path().join("detached-tool.pid");
    let mut command = Command::new("sh");
    command.args([
        "-c",
        "setsid sh -c 'echo $$ > \"$PFTERMINAL_DETACHED_PID_FILE\"; exec sleep 60' & wait",
    ]);
    command.env("PFTERMINAL_DETACHED_PID_FILE", &detached_pid_path);
    command.kill_on_drop(true);
    command.process_group(0);
    let mut child = command.spawn().expect("spawn detached tool fixture");

    let detached_pid = tokio::time::timeout(Duration::from_secs(5), async {
        loop {
            if let Ok(contents) = tokio::fs::read_to_string(&detached_pid_path).await
                && let Ok(pid) = contents.trim().parse::<libc::pid_t>()
            {
                break pid;
            }
            tokio::time::sleep(Duration::from_millis(20)).await;
        }
    })
    .await
    .expect("detached tool should publish its pid");

    stop_claude_child(&mut child)
        .await
        .expect("Claude root and detached tool should be killed");

    tokio::time::timeout(Duration::from_secs(5), async {
        while std::path::Path::new(&format!("/proc/{detached_pid}")).exists() {
            tokio::time::sleep(Duration::from_millis(20)).await;
        }
    })
    .await
    .expect("detached tool should be reaped after tree cleanup");
}

#[cfg(unix)]
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn cancelling_running_command_returns_interrupted_output() {
    let dir = tempfile::tempdir().expect("tempdir");
    let artifact_path = dir.path().join("turn-0001.jsonl");
    let audit_path = dir.path().join("turn-0001.audit.json");
    let plan = ClaudeCommandPlan {
            executable: "sh".to_string(),
            args: vec![
                "-c".to_string(),
                "printf '%s\\n' '{\"type\":\"system\",\"subtype\":\"init\",\"session_id\":\"55555555-5555-4555-8555-555555555555\"}'; sleep 60".to_string(),
            ],
            env: BTreeMap::new(),
            env_remove: Vec::new(),
            cwd: dir.path().to_path_buf(),
            pane_id: "claude-test".to_string(),
            pane_title: "Claude Test".to_string(),
            profile_title: "Claude Test".to_string(),
            provider_model: "test-model".to_string(),
            turn_index: 1,
            command_mode: ClaudeCommandMode::NewSession,
            command_session_id: "55555555-5555-4555-8555-555555555555".to_string(),
            max_turns: None,
            artifact_path: artifact_path.clone(),
            audit_path: audit_path.clone(),
            timeout_ms: None,
            deferred_claude_plan_auth: None,
            bridge: None,
        };
    let cancel_token = CancellationToken::new();
    let cancel_handle = cancel_token.clone();
    let runner = tokio::spawn(run_claude_command_plan(
        plan,
        cancel_token,
        /*progress_tx*/ None,
    ));

    tokio::time::sleep(Duration::from_millis(100)).await;
    cancel_handle.cancel();
    let output = tokio::time::timeout(Duration::from_secs(5), runner)
        .await
        .expect("runner should stop promptly after cancellation")
        .expect("runner join")
        .expect("turn output");

    assert_eq!(output.status, ClaudePaneTurnStatus::Interrupted);
    assert_eq!(output.terminal_reason.as_deref(), Some("interrupted"));
    assert_eq!(
        output.session_id.as_deref(),
        Some("55555555-5555-4555-8555-555555555555")
    );
    assert!(artifact_path.exists());
    assert!(audit_path.exists());
}

#[test]
fn interrupted_partial_output_keeps_planned_session_id_without_stdout_session() {
    let (dir, pane) = pane(ClaudeProviderProfileKind::ClaudePlan);
    let plan =
        build_claude_command_plan(&pane, "start task".to_string(), dir.path()).expect("plan");
    let planned_session_id = plan.command_session_id.clone();

    let output = partial_failed_turn_output(
        &plan,
        /*duration_ms*/ 10,
        ClaudePaneTurnStatus::Interrupted,
        Some("interrupted".to_string()),
        "interrupted by user".to_string(),
        "",
    );

    assert_eq!(output.status, ClaudePaneTurnStatus::Interrupted);
    assert_eq!(
        output.session_id.as_deref(),
        Some(planned_session_id.as_str())
    );
}

#[test]
fn interrupted_partial_output_prefers_stdout_session_id() {
    let (dir, pane) = pane(ClaudeProviderProfileKind::ClaudePlan);
    let plan =
        build_claude_command_plan(&pane, "start task".to_string(), dir.path()).expect("plan");
    let stdout =
        r#"{"type":"system","subtype":"init","session_id":"66666666-6666-4666-8666-666666666666"}"#;

    let output = partial_failed_turn_output(
        &plan,
        /*duration_ms*/ 10,
        ClaudePaneTurnStatus::Interrupted,
        Some("interrupted".to_string()),
        "interrupted by user".to_string(),
        stdout,
    );

    assert_eq!(
        output.session_id.as_deref(),
        Some("66666666-6666-4666-8666-666666666666")
    );
}

#[test]
fn prompt_from_user_turn_rejects_images() {
    let op = AppCommand::UserTurn {
        items: vec![UserInput::Image {
            url: "data:image/png;base64,abc".to_string(),
            detail: None,
        }],
        cwd: PathBuf::from("/tmp"),
        approval_policy: codex_app_server_protocol::AskForApproval::Never,
        approvals_reviewer: None,
        active_permission_profile: None,
        model: "glm-5.2".to_string(),
        effort: None,
        summary: None,
        service_tier: None,
        final_output_json_schema: None,
        collaboration_mode: None,
        personality: None,
    };

    assert!(prompt_from_user_turn(&op).is_err());
}

#[test]
fn compose_claude_pane_prompt_prepends_spawn_context() {
    let prompt = compose_claude_pane_prompt(
        "who are your trolls and orcs".to_string(),
        Some("<pfterminal_spawn_context>\nTrolls: none spawned yet.\n</pfterminal_spawn_context>"),
    );

    assert!(prompt.starts_with("<pfterminal_spawn_context>"));
    assert!(prompt.contains("Trolls: none spawned yet."));
    assert!(prompt.ends_with("User message:\nwho are your trolls and orcs"));
    assert_eq!(
        compose_claude_pane_prompt("hello".to_string(), Some("   ")),
        "hello"
    );
}

#[test]
fn claude_spawn_pane_title_includes_role() {
    assert_eq!(
        claude_pane_title(
            ClaudeProviderProfileKind::VercelGlm52Fast,
            Some(SpawnRole::Troll),
            Some("Burzum")
        ),
        "Claude Code Burzum [troll] - GLM 5.2 Fast Vercel"
    );
    assert_eq!(
        claude_pane_title(
            ClaudeProviderProfileKind::ZaiGlm52,
            Some(SpawnRole::Orc),
            /*spawn_nickname*/ None
        ),
        "Claude Code Orc - GLM 5.2 Z.AI"
    );
    assert_eq!(
        claude_pane_title(
            ClaudeProviderProfileKind::ClaudePlan,
            /*spawn_role*/ None,
            /*spawn_nickname*/ None
        ),
        "Claude Code - Opus 5 Claude Plan"
    );
}

#[test]
fn create_pane_with_role_sets_spawn_role_and_title() {
    let dir = tempfile::tempdir().expect("tempdir");
    let mut registry = ClaudePaneRegistry::new();
    let pane_id = registry
        .create_pane_with_role(
            ClaudeProviderProfileKind::ClaudePlan,
            dir.path().to_path_buf(),
            dir.path(),
            Some(SpawnRole::Troll),
            Some("Burzum".to_string()),
        )
        .expect("create pane");
    let pane = registry
        .panes()
        .iter()
        .find(|pane| pane.id == pane_id)
        .expect("pane");

    assert_eq!(pane.spawn_role, Some(SpawnRole::Troll));
    assert_eq!(pane.spawn_nickname.as_deref(), Some("Burzum"));
    assert_eq!(
        pane.title,
        "Claude Code Burzum [troll] - Opus 5 Claude Plan"
    );
}

#[test]
fn command_plan_uses_session_id_then_resume_without_secret_in_args() {
    let (dir, mut pane) = pane(ClaudeProviderProfileKind::ClaudePlan);
    let first =
        build_claude_command_plan(&pane, "hello".to_string(), dir.path()).expect("first plan");
    let first_session_id = first
        .args
        .windows(2)
        .find_map(|w| (w[0] == "--session-id").then(|| w[1].clone()))
        .expect("first plan should start a Claude session");
    assert!(
        Uuid::parse_str(&first_session_id).is_ok(),
        "Claude session id should be a fresh UUID"
    );
    assert_ne!(
        first_session_id,
        pane.id.trim_start_matches("claude-"),
        "fresh Claude session id must not reuse the pane id"
    );
    assert!(
        first
            .args
            .windows(2)
            .any(|w| w[0] == "--output-format" && w[1] == "stream-json")
    );
    assert!(first.args.iter().any(|arg| arg == "--verbose"));
    assert!(!first.args.iter().any(|arg| arg == "--max-turns"));
    assert_eq!(first.max_turns, None);
    assert_eq!(first.timeout_ms, None);
    assert!(
        first
            .args
            .iter()
            .any(|arg| arg == "--exclude-dynamic-system-prompt-sections")
    );
    assert!(
        first
            .args
            .windows(2)
            .any(|w| w[0] == "--effort" && w[1] == "high")
    );
    assert!(
        first
            .args
            .windows(2)
            .any(|w| w[0] == "--setting-sources" && w[1] == "project")
    );
    assert!(!first.args.iter().any(|arg| arg == "--tools"));
    assert!(!first.args.iter().any(|arg| arg.contains("secret")));
    assert!(first.deferred_claude_plan_auth.is_some());
    assert!(
        first
            .env_remove
            .iter()
            .any(|key| key == "CLAUDE_CODE_OAUTH_TOKEN")
    );

    pane.claude_session_id = Some("11111111-2222-4333-8444-555555555555".to_string());
    let second =
        build_claude_command_plan(&pane, "again".to_string(), dir.path()).expect("second plan");
    assert!(
        second
            .args
            .windows(2)
            .any(|w| { w[0] == "--resume" && w[1] == "11111111-2222-4333-8444-555555555555" })
    );
    assert!(!second.args.iter().any(|arg| arg.contains("secret")));
}

#[test]
fn registry_locks_turns_and_resumes_stored_session() {
    let dir = tempfile::tempdir().expect("tempdir");
    let mut registry = ClaudePaneRegistry::new();
    let pane_id = registry
        .create_pane(
            ClaudeProviderProfileKind::ClaudePlan,
            std::env::current_dir().expect("cwd"),
            dir.path(),
        )
        .expect("create pane");

    let first = registry
        .prepare_turn(&pane_id, "first".to_string(), dir.path())
        .expect("first turn");
    assert!(
        registry
            .prepare_turn(&pane_id, "overlap".to_string(), dir.path())
            .is_err(),
        "a pane must not accept overlapping turns"
    );
    drop(first);

    let result = Ok(ClaudePaneTurnOutput {
        text: "done".to_string(),
        status: ClaudePaneTurnStatus::Success,
        session_id: Some("11111111-2222-4333-8444-555555555555".to_string()),
        usage_summary: None,
        usage_status: ClaudePaneUsageStatus::Missing,
        artifact_path: dir.path().join("turn-0001.jsonl"),
        audit_path: dir.path().join("turn-0001.audit.json"),
        duration_ms: 1,
        terminal_reason: None,
        error_summary: None,
        tool_names: Vec::new(),
        tool_events: Vec::new(),
        reasoning_events: Vec::new(),
        command_mode: ClaudeCommandMode::NewSession,
    });
    registry.finish_turn(&pane_id, &result);

    let second = registry
        .prepare_turn(&pane_id, "second".to_string(), dir.path())
        .expect("second turn");
    assert!(
        second
            .plan
            .args
            .windows(2)
            .any(|w| { w[0] == "--resume" && w[1] == "11111111-2222-4333-8444-555555555555" })
    );
}

#[test]
fn interrupted_turn_with_planned_session_resumes_next_turn() {
    let dir = tempfile::tempdir().expect("tempdir");
    let mut registry = ClaudePaneRegistry::new();
    let pane_id = registry
        .create_pane(
            ClaudeProviderProfileKind::ClaudePlan,
            std::env::current_dir().expect("cwd"),
            dir.path(),
        )
        .expect("create pane");

    let first = registry
        .prepare_turn(&pane_id, "first".to_string(), dir.path())
        .expect("first turn");
    let planned_session_id = first.plan.command_session_id.clone();
    let output = partial_failed_turn_output(
        &first.plan,
        /*duration_ms*/ 10,
        ClaudePaneTurnStatus::Interrupted,
        Some("interrupted".to_string()),
        "interrupted by user".to_string(),
        "",
    );
    drop(first);

    registry.finish_turn(&pane_id, &Ok(output));

    let second = registry
        .prepare_turn(&pane_id, "second".to_string(), dir.path())
        .expect("second turn");
    assert!(
        second
            .plan
            .args
            .windows(2)
            .any(|w| { w[0] == "--resume" && w[1] == planned_session_id })
    );
    assert!(!second.plan.args.iter().any(|arg| arg == "--session-id"));
}

#[test]
fn provider_error_clears_resume_session_for_next_turn() {
    let dir = tempfile::tempdir().expect("tempdir");
    let mut registry = ClaudePaneRegistry::new();
    let pane_id = registry
        .create_pane(
            ClaudeProviderProfileKind::ClaudePlan,
            std::env::current_dir().expect("cwd"),
            dir.path(),
        )
        .expect("create pane");
    {
        let pane = registry
            .panes
            .iter_mut()
            .find(|pane| pane.id == pane_id)
            .expect("pane");
        pane.claude_session_id = Some("11111111-2222-4333-8444-555555555555".to_string());
    }

    let result = Ok(ClaudePaneTurnOutput {
        text: "API Error: The model request was rejected.".to_string(),
        status: ClaudePaneTurnStatus::ProviderError,
        session_id: Some("11111111-2222-4333-8444-555555555555".to_string()),
        usage_summary: None,
        usage_status: ClaudePaneUsageStatus::Untrusted,
        artifact_path: dir.path().join("turn-0001.jsonl"),
        audit_path: dir.path().join("turn-0001.audit.json"),
        duration_ms: 1,
        terminal_reason: Some("completed".to_string()),
        error_summary: Some("model request rejected".to_string()),
        tool_names: Vec::new(),
        tool_events: Vec::new(),
        reasoning_events: Vec::new(),
        command_mode: ClaudeCommandMode::Resume,
    });
    registry.finish_turn(&pane_id, &result);

    let next = registry
        .prepare_turn(&pane_id, "try again".to_string(), dir.path())
        .expect("next turn");
    let next_session_id = next
        .plan
        .args
        .windows(2)
        .find_map(|w| (w[0] == "--session-id").then(|| w[1].clone()))
        .expect("provider-error should force a fresh Claude session");
    assert!(
        Uuid::parse_str(&next_session_id).is_ok(),
        "fresh Claude session should be a UUID"
    );
    assert_ne!(
        next_session_id,
        pane_id.trim_start_matches("claude-"),
        "fresh Claude session must not reuse pane id"
    );
    assert_ne!(
        next_session_id, "11111111-2222-4333-8444-555555555555",
        "fresh Claude session must not reuse failed provider session"
    );
    assert!(!next.plan.args.iter().any(|arg| arg == "--resume"));
}

#[test]
fn max_turn_output_keeps_resume_guidance_and_audit_hint() {
    let (dir, _pane) = pane(ClaudeProviderProfileKind::AmbientGlm52);
    let output = ClaudePaneTurnOutput {
        text: String::new(),
        status: ClaudePaneTurnStatus::MaxTurnsPause,
        session_id: Some("44444444-4444-4444-8444-444444444444".to_string()),
        usage_summary: Some(r#"{"input_tokens":10}"#.to_string()),
        usage_status: ClaudePaneUsageStatus::Reported,
        artifact_path: dir.path().join("turn-0001.jsonl"),
        audit_path: dir.path().join("turn-0001.audit.json"),
        duration_ms: 10,
        terminal_reason: Some("max_turns".to_string()),
        error_summary: Some("Reached maximum number of turns (24)".to_string()),
        tool_names: vec!["Read".to_string()],
        tool_events: vec![ClaudePaneToolEvent {
            name: "Read".to_string(),
            preview: r#"{"file_path":"README.md"}"#.to_string(),
        }],
        reasoning_events: Vec::new(),
        command_mode: ClaudeCommandMode::NewSession,
    };

    assert!(output.failure_message().contains("Type `continue`"));
    let hint = output.audit_hint();
    assert!(hint.contains("status: max-turn-pause"));
    assert!(hint.contains("artifact:"));
    assert!(hint.contains("audit:"));
    assert!(hint.contains("tools: Read"));
}

#[test]
fn turn_audit_serializes_without_prompt_or_secret() {
    let (dir, pane) = pane(ClaudeProviderProfileKind::ClaudePlan);
    let plan = build_claude_command_plan(
        &pane,
        "this prompt must not be serialized into audit".to_string(),
        dir.path(),
    )
    .expect("plan");
    let output = failed_turn_output(
        &plan,
        /*duration_ms*/ 5,
        ClaudePaneTurnStatus::ProviderError,
        Some("provider_error".to_string()),
        "simulated provider failure".to_string(),
    );

    write_turn_audit(
        &plan,
        &output,
        /*started_at_unix_ms*/ 1,
        /*ended_at_unix_ms*/ 2,
        Some(1),
    )
    .expect("write audit");
    let audit = std::fs::read_to_string(&plan.audit_path).expect("read audit");
    assert!(audit.contains("simulated provider failure"));
    assert!(!audit.contains("this prompt must not be serialized"));
    assert!(!audit.contains("ambient-secret"));
}

#[test]
fn turn_audit_counts_tool_events_not_unique_tool_names() {
    let (dir, pane) = pane(ClaudeProviderProfileKind::ClaudePlan);
    let plan = build_claude_command_plan(&pane, "review".to_string(), dir.path()).expect("plan");
    let output = ClaudePaneTurnOutput {
        text: "done".to_string(),
        status: ClaudePaneTurnStatus::Success,
        session_id: Some("11111111-2222-4333-8444-555555555555".to_string()),
        usage_summary: None,
        usage_status: ClaudePaneUsageStatus::Missing,
        artifact_path: plan.artifact_path.clone(),
        audit_path: plan.audit_path.clone(),
        duration_ms: 10,
        terminal_reason: None,
        error_summary: None,
        tool_names: vec!["Read".to_string(), "Bash".to_string()],
        tool_events: vec![
            ClaudePaneToolEvent {
                name: "Read".to_string(),
                preview: "{}".to_string(),
            },
            ClaudePaneToolEvent {
                name: "Read".to_string(),
                preview: "{}".to_string(),
            },
            ClaudePaneToolEvent {
                name: "Bash".to_string(),
                preview: "{}".to_string(),
            },
        ],
        reasoning_events: Vec::new(),
        command_mode: ClaudeCommandMode::NewSession,
    };

    write_turn_audit(
        &plan,
        &output,
        /*started_at_unix_ms*/ 1,
        /*ended_at_unix_ms*/ 2,
        Some(1),
    )
    .expect("write audit");
    let audit = std::fs::read_to_string(&plan.audit_path).expect("read audit");
    let audit: Value = serde_json::from_str(&audit).expect("audit json");
    assert_eq!(audit.get("tool_use_count").and_then(Value::as_u64), Some(3));
}

#[test]
fn turn_audit_serializes_reasoning_events() {
    let (dir, pane) = pane(ClaudeProviderProfileKind::ClaudePlan);
    let plan = build_claude_command_plan(&pane, "review".to_string(), dir.path()).expect("plan");
    let output = ClaudePaneTurnOutput {
        text: "done".to_string(),
        status: ClaudePaneTurnStatus::Success,
        session_id: Some("11111111-2222-4333-8444-555555555555".to_string()),
        usage_summary: None,
        usage_status: ClaudePaneUsageStatus::Missing,
        artifact_path: plan.artifact_path.clone(),
        audit_path: plan.audit_path.clone(),
        duration_ms: 10,
        terminal_reason: None,
        error_summary: None,
        tool_names: Vec::new(),
        tool_events: Vec::new(),
        reasoning_events: vec![ClaudePaneReasoningEvent {
            preview: "Inspect Orc output before reporting to the Nazgul.".to_string(),
        }],
        command_mode: ClaudeCommandMode::NewSession,
    };

    write_turn_audit(
        &plan,
        &output,
        /*started_at_unix_ms*/ 1,
        /*ended_at_unix_ms*/ 2,
        Some(1),
    )
    .expect("write audit");
    let audit = std::fs::read_to_string(&plan.audit_path).expect("read audit");
    let audit: Value = serde_json::from_str(&audit).expect("audit json");
    assert_eq!(
        audit.get("reasoning_event_count").and_then(Value::as_u64),
        Some(1)
    );
    assert_eq!(
        audit.pointer("/reasoning_events/0/preview"),
        Some(&json!("Inspect Orc output before reporting to the Nazgul."))
    );
}

#[test]
fn allowed_auth_helper_labels_are_provider_scoped() {
    assert!(allowed_provider_vault_label("provider/zai_api_key"));
    assert!(allowed_provider_vault_label("provider/anthropic_api_key"));
    assert!(allowed_provider_vault_label("provider/ambient_api_key"));
    assert!(allowed_provider_vault_label("provider/kimi_api_key"));
    assert!(allowed_provider_vault_label("provider/baseten_api_key"));
    assert!(allowed_provider_vault_label("provider/openrouter_api_key"));
    assert!(allowed_provider_vault_label("provider/ai_gateway_api_key"));
    assert!(!allowed_provider_vault_label("random"));
}

#[test]
fn parsed_message_content_can_be_nested() {
    let value = json!({
        "type": "assistant",
        "message": {
            "content": [
                {"type": "text", "text": "one"},
                {"type": "tool_use", "name": "Read"}
            ]
        }
    });
    let parsed = parsed_from_value(&value).expect("parse");
    assert_eq!(parsed.text, "one");
}

#[test]
fn parse_stream_json_collects_thinking_blocks() {
    let stdout = r#"{"type":"assistant","message":{"content":[{"type":"thinking","thinking":"The Troll should inspect the Orc output before reporting up."}]},"session_id":"11111111-2222-4333-8444-555555555555"}
{"type":"assistant","message":{"content":[{"type":"text","text":"Reviewed."}]}}
{"type":"result","subtype":"success","result":"Reviewed.","session_id":"11111111-2222-4333-8444-555555555555","usage":{"input_tokens":10,"output_tokens":4}}"#;

    let parsed = parse_claude_output(stdout).expect("parse stream");
    assert_eq!(parsed.text, "Reviewed.");
    assert_eq!(parsed.reasoning_events.len(), 1);
    assert_eq!(
        parsed.reasoning_events[0].preview,
        "The Troll should inspect the Orc output before reporting up."
    );
}

#[test]
fn progress_can_emit_reasoning_and_tool_for_one_message() {
    let (dir, pane) = pane(ClaudeProviderProfileKind::ClaudePlan);
    let plan = build_claude_command_plan(&pane, "review".to_string(), dir.path()).expect("plan");
    let started_at = Instant::now();
    let value = json!({
        "type": "assistant",
        "message": {
            "content": [
                {
                    "type": "thinking",
                    "thinking": "Read the file before editing."
                },
                {
                    "type": "tool_use",
                    "name": "Read",
                    "input": {"file_path": "README.md"}
                }
            ]
        }
    });

    let progresses = progresses_from_claude_value(&plan, &started_at, &value);
    assert_eq!(progresses.len(), 2);
    assert_eq!(progresses[0].phase, "reasoning");
    assert_eq!(progresses[1].phase, "tool-call");
    assert_eq!(
        progresses[1].summary,
        "Claude tool call: Read: reading README.md"
    );
}

#[test]
fn claude_tools_are_translated_to_ambient_chat_tools() {
    let request = json!({
        "tools": [{
            "name": "Read",
            "description": "Read a file",
            "input_schema": {
                "type": "object",
                "properties": {
                    "path": { "type": "string" }
                },
                "required": ["path"]
            }
        }]
    });
    let tools = ambient_chat_tools_from_claude_request(&request);

    assert_eq!(tools.len(), 1);
    assert_eq!(
        tools[0].pointer("/type").and_then(Value::as_str),
        Some("function")
    );
    assert_eq!(
        tools[0].pointer("/function/name").and_then(Value::as_str),
        Some("Read")
    );
    assert_eq!(
        tools[0]
            .pointer("/function/parameters/required/0")
            .and_then(Value::as_str),
        Some("path")
    );
}

#[test]
fn claude_tool_history_is_translated_to_ambient_chat_messages() {
    let request = json!({
        "messages": [
            {
                "role": "assistant",
                "content": [{
                    "type": "tool_use",
                    "id": "toolu_1",
                    "name": "Read",
                    "input": { "path": "README.md" }
                }]
            },
            {
                "role": "user",
                "content": [{
                    "type": "tool_result",
                    "tool_use_id": "toolu_1",
                    "content": "hello"
                }]
            }
        ]
    });
    let messages = ambient_chat_messages_from_claude_request(&request).expect("messages");

    assert_eq!(messages.len(), 2);
    assert_eq!(
        messages[0]
            .pointer("/tool_calls/0/id")
            .and_then(Value::as_str),
        Some("toolu_1")
    );
    assert_eq!(
        messages[1].pointer("/tool_call_id").and_then(Value::as_str),
        Some("toolu_1")
    );
    assert_eq!(
        messages[1].pointer("/content").and_then(Value::as_str),
        Some("hello")
    );
}

#[test]
fn ambient_tool_calls_are_translated_to_anthropic_tool_uses() {
    let upstream = json!({
        "choices": [{
            "message": {
                "tool_calls": [
                    {
                        "id": "chatcmpl-tool-1",
                        "type": "function",
                        "function": {
                            "name": "Read",
                            "arguments": "{\"path\":\"README.md\"}"
                        }
                    },
                    {
                        "id": "chatcmpl-tool-2",
                        "type": "function",
                        "function": {
                            "name": "Bash",
                            "arguments": "{\"command\":\"git status --short\"}"
                        }
                    }
                ]
            }
        }]
    });
    let calls = bridge_tool_calls_from_ambient_response(&upstream);
    let response = anthropic_tool_use_response(
        "z-ai/glm-5.2",
        &calls,
        &json!({"prompt_tokens": 5, "cached_tokens": 2, "completion_tokens": 3}),
    );

    assert_eq!(calls.len(), 2);
    assert_eq!(calls[0].name, "Read");
    assert_eq!(calls[1].name, "Bash");
    assert_eq!(
        response.pointer("/content/0/type").and_then(Value::as_str),
        Some("tool_use")
    );
    assert_eq!(
        response.pointer("/content/1/name").and_then(Value::as_str),
        Some("Bash")
    );
    assert_eq!(
        response.pointer("/stop_reason").and_then(Value::as_str),
        Some("tool_use")
    );
    assert_eq!(
        response
            .pointer("/usage/cache_read_input_tokens")
            .and_then(Value::as_u64),
        Some(2)
    );
}

#[test]
fn ambient_retry_after_delay_parses_seconds_and_caps_large_values() {
    let mut headers = reqwest::header::HeaderMap::new();
    headers.insert(reqwest::header::RETRY_AFTER, "42".parse().expect("header"));
    assert_eq!(
        ambient_retry_after_delay(&headers),
        Some(Duration::from_secs(42))
    );

    headers.insert(reqwest::header::RETRY_AFTER, "999".parse().expect("header"));
    assert_eq!(
        ambient_retry_after_delay(&headers),
        Some(Duration::from_secs(300))
    );
}

#[test]
fn anthropic_stream_events_preserve_upstream_usage_in_protocol_fields() {
    let start = anthropic_stream_start_event(
        "z-ai/glm-5.2",
        &serde_json::json!({
            "prompt_tokens": 120,
            "cached_tokens": 80,
            "completion_tokens": 34
        }),
    );
    let stop = anthropic_stream_stop_event(
        "end_turn",
        &serde_json::json!({
            "prompt_tokens": 120,
            "cached_tokens": 80,
            "completion_tokens": 34
        }),
    );

    assert_eq!(
        start
            .pointer("/message/usage/input_tokens")
            .and_then(Value::as_u64),
        Some(120)
    );
    assert_eq!(
        start
            .pointer("/message/usage/cache_read_input_tokens")
            .and_then(Value::as_u64),
        Some(80)
    );
    assert_eq!(
        start
            .pointer("/message/usage/output_tokens")
            .and_then(Value::as_u64),
        Some(0)
    );
    assert_eq!(
        stop.pointer("/usage/output_tokens").and_then(Value::as_u64),
        Some(34)
    );
    assert!(stop.pointer("/usage/input_tokens").is_none());
}

#[test]
fn anthropic_stream_error_event_is_protocol_error() {
    let event = anthropic_stream_error_event("upstream_transport_error", "boom");

    assert_eq!(event.get("type").and_then(Value::as_str), Some("error"));
    assert_eq!(
        event.pointer("/error/type").and_then(Value::as_str),
        Some("upstream_transport_error")
    );
    assert_eq!(
        event.pointer("/error/message").and_then(Value::as_str),
        Some("boom")
    );
    assert!(event.get("content").is_none());
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
#[ignore = "requires claude CLI and a live provider/ambient_api_key vault credential"]
async fn live_ambient_bridge_runs_claude_headless_for_two_turns() {
    let codex_home = std::env::var("PFTERMINAL_LIVE_CODEX_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("/home/postfiat/.pfterminal"));
    let (_dir, mut pane) = pane(ClaudeProviderProfileKind::AmbientGlm52);

    let first_plan = build_claude_command_plan(
        &pane,
        "Reply with exactly: OK-PFTERMINAL-LIVE".to_string(),
        &codex_home,
    )
    .expect("first live plan");
    let first = run_claude_command_plan(
        first_plan,
        CancellationToken::new(),
        /*progress_tx*/ None,
    )
    .await
    .expect("first live Claude turn");
    assert!(
        first.text.contains("OK-PFTERMINAL-LIVE"),
        "first turn did not return the requested marker: {}",
        first.text
    );
    pane.claude_session_id = first.session_id;
    pane.next_turn_index = 2;

    let second_plan = build_claude_command_plan(
        &pane,
        "What exact marker did you just return? Reply with only that marker.".to_string(),
        &codex_home,
    )
    .expect("second live plan");
    let second = run_claude_command_plan(
        second_plan,
        CancellationToken::new(),
        /*progress_tx*/ None,
    )
    .await
    .expect("second live Claude turn");
    assert!(
        second.text.contains("OK-PFTERMINAL-LIVE"),
        "second turn did not retain session context: {}",
        second.text
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
#[ignore = "requires claude CLI and a live provider/ambient_api_key vault credential"]
async fn live_ambient_bridge_runs_claude_tool_loop() {
    let codex_home = std::env::var("PFTERMINAL_LIVE_CODEX_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("/home/postfiat/.pfterminal"));
    let (_dir, pane) = pane(ClaudeProviderProfileKind::AmbientGlm52);

    let plan = build_claude_command_plan(
            &pane,
            "Use your LS tool to inspect the current working directory. If Cargo.toml is present, reply exactly: FOUND-CARGO-TOML. Do not explain."
                .to_string(),
            &codex_home,
        )
        .expect("tool-loop live plan");
    let output = run_claude_command_plan(plan, CancellationToken::new(), /*progress_tx*/ None)
        .await
        .expect("tool-loop live Claude turn");
    assert!(
        output.text.contains("FOUND-CARGO-TOML"),
        "tool loop did not return expected marker: {}",
        output.text
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
#[ignore = "requires claude CLI and a live provider/ambient_api_key vault credential"]
async fn live_ambient_bridge_runs_substantive_code_review() {
    let codex_home = std::env::var("PFTERMINAL_LIVE_CODEX_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("/home/postfiat/.pfterminal"));
    let (_dir, pane) = pane(ClaudeProviderProfileKind::AmbientGlm52);

    let plan = build_claude_command_plan(
        &pane,
        concat!(
            "Perform a read-only code review of codex-rs/tui/src/claude_panes.rs. ",
            "Use filesystem tools to inspect the file. Reply with marker ",
            "PFT_REVIEW_OK and two concrete findings or risks. Do not edit files."
        )
        .to_string(),
        &codex_home,
    )
    .expect("review live plan");
    let output = run_claude_command_plan(plan, CancellationToken::new(), /*progress_tx*/ None)
        .await
        .expect("review live Claude turn");
    assert_eq!(output.status, ClaudePaneTurnStatus::Success);
    assert!(
        output.text.contains("PFT_REVIEW_OK"),
        "review did not return expected marker: {}",
        output.text
    );
    assert!(
        !output.tool_names.is_empty(),
        "review should use Claude Code tools; audit: {}",
        output.audit_path.display()
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
#[ignore = "requires claude CLI and a live provider/ambient_api_key vault credential"]
async fn live_ambient_bridge_runs_disposable_edit_task() {
    let codex_home = std::env::var("PFTERMINAL_LIVE_CODEX_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("/home/postfiat/.pfterminal"));
    let (dir, mut pane) = pane(ClaudeProviderProfileKind::AmbientGlm52);
    pane.cwd = dir.path().to_path_buf();
    let target = dir.path().join("sample.txt");
    std::fs::write(&target, "before\n").expect("seed fixture");

    let plan = build_claude_command_plan(
            &pane,
            "Edit sample.txt so it contains exactly PFT_EDIT_OK followed by a newline. Then reply exactly: PFT_EDIT_DONE"
                .to_string(),
            &codex_home,
        )
        .expect("edit live plan");
    let output = run_claude_command_plan(plan, CancellationToken::new(), /*progress_tx*/ None)
        .await
        .expect("edit live Claude turn");
    assert_eq!(output.status, ClaudePaneTurnStatus::Success);
    assert!(
        output.text.contains("PFT_EDIT_DONE"),
        "edit did not return expected marker: {}",
        output.text
    );
    assert_eq!(
        std::fs::read_to_string(&target).expect("read edited fixture"),
        "PFT_EDIT_OK\n"
    );
}
