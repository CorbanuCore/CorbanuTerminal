use codex_app_server_protocol::CommandExecutionApprovalDecision;
use codex_app_server_protocol::CommandExecutionRequestApprovalParams;
use codex_app_server_protocol::ExecPolicyAmendment;
use codex_app_server_protocol::RequestId;
use pretty_assertions::assert_eq;
use serde_json::json;

use codex_telegram::approvals::ApprovalCallback;
use codex_telegram::approvals::PendingApproval;
use codex_telegram::approvals::PendingApprovalKind;

#[test]
fn callback_round_trips_integer_request_id() {
    let callback = ApprovalCallback {
        decision_index: 2,
        request_id: RequestId::Integer(12),
    };

    assert_eq!(ApprovalCallback::decode(&callback.encode()), Some(callback));
}

#[test]
fn callback_round_trips_string_request_id() {
    let callback = ApprovalCallback {
        decision_index: 1,
        request_id: RequestId::String("req:with spaces".to_string()),
    };

    assert_eq!(ApprovalCallback::decode(&callback.encode()), Some(callback));
}

#[test]
fn command_approval_renders_escaped_command() {
    let approval = PendingApproval {
        request_id: RequestId::Integer(1),
        kind: PendingApprovalKind::Command(command_params(
            Some("echo <secret> && true"),
            Some("needs <network>"),
            /*available_decisions*/ None,
        )),
    };

    let message = approval.message();

    assert!(message.contains("echo &lt;secret&gt; &amp;&amp; true"));
    assert!(message.contains("needs &lt;network&gt;"));
}

#[test]
fn command_decline_resolves_with_first_class_decline_decision() {
    let approval = PendingApproval {
        request_id: RequestId::Integer(1),
        kind: PendingApprovalKind::Command(command_params(
            Some("false"),
            /*reason*/ None,
            Some(vec![CommandExecutionApprovalDecision::Decline]),
        )),
    };

    let value = approval.resolve_value(/*decision_index*/ 0).expect("decline serializes");

    assert_eq!(value, json!({ "decision": "decline" }));
}

#[test]
fn command_keyboard_uses_advertised_accept_and_cancel_decisions() {
    let approval = PendingApproval {
        request_id: RequestId::Integer(1),
        kind: PendingApprovalKind::Command(command_params(
            Some("true"),
            /*reason*/ None,
            Some(vec![
                CommandExecutionApprovalDecision::Accept,
                CommandExecutionApprovalDecision::Cancel,
            ]),
        )),
    };

    let labels = keyboard_labels(&approval);

    assert_eq!(labels, vec!["Approve".to_string(), "Cancel".to_string()]);
}

#[test]
fn command_keyboard_shows_execpolicy_amendment_when_advertised() {
    let approval = PendingApproval {
        request_id: RequestId::Integer(1),
        kind: PendingApprovalKind::Command(command_params(
            Some("true"),
            /*reason*/ None,
            Some(vec![
                CommandExecutionApprovalDecision::AcceptWithExecpolicyAmendment {
                    execpolicy_amendment: ExecPolicyAmendment {
                        command: vec!["cargo".to_string(), "test".to_string()],
                    },
                },
                CommandExecutionApprovalDecision::Cancel,
            ]),
        )),
    };

    let labels = keyboard_labels(&approval);

    assert_eq!(
        labels,
        vec![
            "Approve and remember command".to_string(),
            "Cancel".to_string()
        ]
    );
}

#[test]
fn command_callback_for_unadvertised_decision_is_rejected() {
    let approval = PendingApproval {
        request_id: RequestId::Integer(1),
        kind: PendingApprovalKind::Command(command_params(
            Some("true"),
            /*reason*/ None,
            Some(vec![CommandExecutionApprovalDecision::Accept]),
        )),
    };

    let err = approval
        .resolve_value(/*decision_index*/ 1)
        .expect_err("unadvertised decision rejected");

    assert!(
        err.to_string()
            .contains("approval decision 1 is not available")
    );
}

fn command_params(
    command: Option<&str>,
    reason: Option<&str>,
    available_decisions: Option<Vec<CommandExecutionApprovalDecision>>,
) -> CommandExecutionRequestApprovalParams {
    CommandExecutionRequestApprovalParams {
        thread_id: "thread".to_string(),
        turn_id: "turn".to_string(),
        item_id: "item".to_string(),
        started_at_ms: 1,
        approval_id: None,
        environment_id: None,
        reason: reason.map(str::to_string),
        network_approval_context: None,
        command: command.map(str::to_string),
        cwd: None,
        command_actions: None,
        additional_permissions: None,
        proposed_execpolicy_amendment: None,
        proposed_network_policy_amendments: None,
        available_decisions,
    }
}

fn keyboard_labels(approval: &PendingApproval) -> Vec<String> {
    approval
        .keyboard()
        .inline_keyboard
        .into_iter()
        .flatten()
        .map(|button| button.text)
        .collect()
}
