//! #1011 item 3: the #942 once-only options also apply to elicitation and
//! Aggressive mode, while an offered persistent approval still covers a call.

use codex_features::Feature;
use codex_protocol::approvals::ElicitationRequest;
use codex_protocol::mcp_approval_meta::PERSIST_ALWAYS;
use codex_protocol::mcp_approval_meta::PERSIST_KEY;
use codex_protocol::mcp_approval_meta::PERSIST_SESSION;
use codex_protocol::protocol::AskForApproval;
use codex_protocol::protocol::ElicitationAction;
use codex_protocol::protocol::EventMsg;
use codex_protocol::protocol::Op;
use codex_security_policy::SecurityLevel;
use core_test_support::responses::ResponseMock;
use core_test_support::skip_if_no_network;
use core_test_support::test_codex::TestCodex;
use core_test_support::wait_for_event;
use pretty_assertions::assert_eq;
use serde_json::Value;
use serde_json::json;

use super::QUESTION_PREFIX;
use super::answer;
use super::done_step;
use super::insert_rmcp_server;
use super::option_labels;
use super::start_turn;
use super::sync_step;

enum ApprovalPrompt {
    Question,
    Elicitation,
}

async fn sync_turn(
    level: SecurityLevel,
    prompt: ApprovalPrompt,
) -> anyhow::Result<(TestCodex, ResponseMock)> {
    start_turn(
        level,
        AskForApproval::OnRequest,
        vec![
            sync_step("call-sync-1"),
            sync_step("call-sync-2"),
            done_step(),
        ],
        move |config| {
            insert_rmcp_server(config).expect("rmcp server");
            let mut servers = config.mcp_servers.get().clone();
            servers
                .get_mut("rmcp")
                .expect("rmcp server")
                .default_tools_approval_mode = None;
            config
                .mcp_servers
                .set(servers)
                .expect("test mcp servers accept any configuration");
            match prompt {
                ApprovalPrompt::Question => config
                    .features
                    .disable(Feature::ToolCallMcpElicitation)
                    .expect("disable MCP elicitation"),
                ApprovalPrompt::Elicitation => config
                    .features
                    .enable(Feature::ToolCallMcpElicitation)
                    .expect("enable MCP elicitation"),
            };
        },
    )
    .await
}

async fn next_approval_or_completion(test: &TestCodex) -> EventMsg {
    wait_for_event(&test.codex, |event| {
        matches!(
            event,
            EventMsg::ElicitationRequest(_)
                | EventMsg::RequestUserInput(_)
                | EventMsg::TurnComplete(_)
        )
    })
    .await
}

pub(super) fn assert_second_sync_output(captured: &ResponseMock) {
    let requests = captured.requests();
    assert_eq!(requests.len(), 3);
    let output = requests[2]
        .function_call_output_text("call-sync-2")
        .expect("the second sync result reaches the model");
    // Source envelopes label untrusted MCP output; the wall time varies.
    // Compare the entire structured fixture result inside those wrappers.
    let output = if let Some(enveloped) = output.strip_prefix("<corbanu_untrusted_data>\n") {
        enveloped
            .strip_suffix("\n</corbanu_untrusted_data>")
            .expect("complete source envelope")
    } else {
        &output
    };
    let (_, structured) = output.split_once("\nOutput:\n").expect("MCP output header");
    let result: Value = serde_json::from_str(structured).expect("structured sync result");
    assert_eq!(result, json!({ "result": "ok" }));
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn elicitation_offers_only_applicable_lasting_approval() -> anyhow::Result<()> {
    skip_if_no_network!(Ok(()));
    for level in [SecurityLevel::Moderate, SecurityLevel::Permissive] {
        let (test, captured) = sync_turn(level, ApprovalPrompt::Elicitation).await?;
        let EventMsg::ElicitationRequest(first) = next_approval_or_completion(&test).await else {
            panic!("the first sync call asks through elicitation");
        };
        let ElicitationRequest::Form { meta, .. } = &first.request else {
            panic!("the MCP approval is a form");
        };
        let offered_persistence = meta.as_ref().and_then(|meta| meta.get(PERSIST_KEY));
        let expected_persistence = match level {
            SecurityLevel::Moderate => None,
            SecurityLevel::Permissive => Some(json!([PERSIST_SESSION, PERSIST_ALWAYS])),
            SecurityLevel::Aggressive => {
                unreachable!("only Moderate and Permissive are exercised here")
            }
        };
        assert_eq!(offered_persistence, expected_persistence.as_ref());

        // Answer Always allow only when the prompt actually offered it.
        let response_meta = match level {
            SecurityLevel::Moderate => None,
            SecurityLevel::Permissive => Some(json!({ (PERSIST_KEY): PERSIST_ALWAYS })),
            SecurityLevel::Aggressive => {
                unreachable!("only Moderate and Permissive are exercised here")
            }
        };
        test.codex
            .submit(Op::ResolveElicitation {
                server_name: first.server_name,
                request_id: first.id,
                decision: ElicitationAction::Accept,
                content: None,
                meta: response_meta,
            })
            .await?;

        if level == SecurityLevel::Moderate {
            let EventMsg::RequestUserInput(second) = next_approval_or_completion(&test).await
            else {
                panic!("Moderate requires a fresh approval for the second sync");
            };
            assert!(second.questions[0].id.starts_with(QUESTION_PREFIX));
            answer(&test, &second, "Allow once").await;
        }
        assert!(matches!(
            next_approval_or_completion(&test).await,
            EventMsg::TurnComplete(_)
        ));
        assert_second_sync_output(&captured);
    }
    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn aggressive_requires_fresh_approval_for_the_second_call() -> anyhow::Result<()> {
    skip_if_no_network!(Ok(()));
    let (test, captured) = sync_turn(SecurityLevel::Aggressive, ApprovalPrompt::Question).await?;
    let EventMsg::RequestUserInput(first) = next_approval_or_completion(&test).await else {
        panic!("the first sync call asks for approval");
    };
    assert!(first.questions[0].id.starts_with("mcp_tool_call_approval"));
    assert_eq!(option_labels(&first), vec!["Allow", "Cancel"]);
    answer(&test, &first, "Allow").await;

    let EventMsg::RequestUserInput(second) = next_approval_or_completion(&test).await else {
        panic!("Aggressive requires a fresh approval for the second sync");
    };
    assert!(second.questions[0].id.starts_with(QUESTION_PREFIX));
    answer(&test, &second, "Allow once").await;
    assert!(matches!(
        next_approval_or_completion(&test).await,
        EventMsg::TurnComplete(_)
    ));
    assert_second_sync_output(&captured);
    Ok(())
}
