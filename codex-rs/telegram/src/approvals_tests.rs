use codex_app_server_protocol::CommandExecutionRequestApprovalParams;
use codex_app_server_protocol::RequestId;
use pretty_assertions::assert_eq;
use serde_json::json;

use codex_telegram::approvals::ApprovalAction;
use codex_telegram::approvals::ApprovalCallback;
use codex_telegram::approvals::PendingApproval;
use codex_telegram::approvals::PendingApprovalKind;

#[test]
fn callback_round_trips_integer_request_id() {
    let callback = ApprovalCallback {
        action: ApprovalAction::ApproveForSession,
        request_id: RequestId::Integer(12),
    };

    assert_eq!(ApprovalCallback::decode(&callback.encode()), Some(callback));
}

#[test]
fn callback_round_trips_string_request_id() {
    let callback = ApprovalCallback {
        action: ApprovalAction::Decline,
        request_id: RequestId::String("req:with spaces".to_string()),
    };

    assert_eq!(ApprovalCallback::decode(&callback.encode()), Some(callback));
}

#[test]
fn command_approval_renders_escaped_command() {
    let approval = PendingApproval {
        request_id: RequestId::Integer(1),
        kind: PendingApprovalKind::Command(CommandExecutionRequestApprovalParams {
            thread_id: "thread".to_string(),
            turn_id: "turn".to_string(),
            item_id: "item".to_string(),
            started_at_ms: 1,
            approval_id: None,
            environment_id: None,
            reason: Some("needs <network>".to_string()),
            network_approval_context: None,
            command: Some("echo <secret> && true".to_string()),
            cwd: None,
            command_actions: None,
            additional_permissions: None,
            proposed_execpolicy_amendment: None,
            proposed_network_policy_amendments: None,
            available_decisions: None,
        }),
    };

    let message = approval.message();

    assert!(message.contains("echo &lt;secret&gt; &amp;&amp; true"));
    assert!(message.contains("needs &lt;network&gt;"));
}

#[test]
fn command_decline_resolves_with_first_class_decline_decision() {
    let approval = PendingApproval {
        request_id: RequestId::Integer(1),
        kind: PendingApprovalKind::Command(CommandExecutionRequestApprovalParams {
            thread_id: "thread".to_string(),
            turn_id: "turn".to_string(),
            item_id: "item".to_string(),
            started_at_ms: 1,
            approval_id: None,
            environment_id: None,
            reason: None,
            network_approval_context: None,
            command: Some("false".to_string()),
            cwd: None,
            command_actions: None,
            additional_permissions: None,
            proposed_execpolicy_amendment: None,
            proposed_network_policy_amendments: None,
            available_decisions: None,
        }),
    };

    let value = approval
        .resolve_value(ApprovalAction::Decline)
        .expect("decline serializes");

    assert_eq!(value, json!({ "decision": "decline" }));
}
