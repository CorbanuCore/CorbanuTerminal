//! #787 (AGG-R14 replay Q11b): a Down + Enter meant for the parent's
//! question answered a child agent's command prompt that had appeared 0.8 s
//! earlier, choosing "don't ask again". A request never covers the request on
//! screen (it waits under it), and input that reaches a request in the moment
//! after it appeared is dropped: the prompt then needs navigation before
//! Enter and says why.

use super::*;
use codex_app_server_protocol::ExecPolicyAmendment;
use pretty_assertions::assert_eq;

const EARLY: &str = "Keys pressed as this request appeared were ignored";

/// A pane with the production guard, as the TUI runs it.
fn guarded_pane(tx: AppEventSender) -> BottomPane {
    let mut pane = test_pane(tx);
    pane.set_request_input_guard(REQUEST_INPUT_GUARD);
    pane
}

/// A child agent's command prompt whose second option is "Yes, and don't
/// ask again for commands that start with `sleep`".
fn child_exec_request() -> ApprovalRequest {
    ApprovalRequest::Exec(ExecApprovalRequest {
        thread_id: codex_protocol::ThreadId::new(),
        thread_label: Some("Agent (01a13a2a)".to_string()),
        id: "child-call".to_string(),
        environment_id: None,
        command: vec![
            "bash".into(),
            "-lc".into(),
            "sleep 60; echo finished > ks-child.txt".into(),
        ],
        reason: None,
        available_decisions: vec![
            CommandExecutionApprovalDecision::Accept,
            CommandExecutionApprovalDecision::AcceptWithExecpolicyAmendment {
                execpolicy_amendment: ExecPolicyAmendment {
                    command: vec!["sleep".to_string()],
                },
            },
            CommandExecutionApprovalDecision::Cancel,
        ],
        network_approval_context: None,
        additional_permissions: None,
    })
}

fn patch_request() -> ApprovalRequest {
    let mut changes = std::collections::HashMap::new();
    changes.insert(
        PathBuf::from("notes.txt"),
        crate::diff_model::FileChange::Add {
            content: "hello\n".to_string(),
        },
    );
    ApprovalRequest::ApplyPatch(ApplyPatchApprovalRequest {
        thread_id: codex_protocol::ThreadId::new(),
        thread_label: Some("Agent (01a13a2a)".to_string()),
        id: "patch".to_string(),
        reason: None,
        cwd: test_path_buf("/tmp").abs(),
        changes,
        response_destination: crate::approval_events::ApprovalResponseDestination::Thread,
    })
}

fn permissions_request() -> ApprovalRequest {
    ApprovalRequest::Permissions(PermissionsApprovalRequest {
        thread_id: codex_protocol::ThreadId::new(),
        thread_label: None,
        call_id: "permissions".to_string(),
        environment_id: None,
        reason: Some("need network".to_string()),
        permissions: codex_protocol::request_permissions::RequestPermissionProfile {
            network: Some(codex_protocol::models::NetworkPermissions {
                enabled: Some(true),
            }),
            file_system: None,
        },
    })
}

fn elicitation_approval_request() -> ApprovalRequest {
    ApprovalRequest::McpElicitation(McpElicitationApprovalRequest {
        thread_id: codex_protocol::ThreadId::new(),
        thread_label: None,
        server_name: "test-server".to_string(),
        request_id: codex_app_server_protocol::RequestId::String("request-1".to_string()),
        message: "Need more information".to_string(),
    })
}

fn press_at(pane: &mut BottomPane, keys: &[KeyCode], at: Instant) {
    for key in keys {
        pane.handle_key_event_at(KeyEvent::new(*key, KeyModifiers::NONE), at);
    }
}

fn exec_decisions(
    rx: &mut tokio::sync::mpsc::UnboundedReceiver<AppEvent>,
) -> Vec<CommandExecutionApprovalDecision> {
    std::iter::from_fn(|| rx.try_recv().ok())
        .filter_map(|event| match event {
            AppEvent::SubmitThreadOp {
                op: Op::ExecApproval { decision, .. },
                ..
            } => Some(decision),
            _ => None,
        })
        .collect()
}

/// The Q11b sequence: the parent's question is on screen, the child's
/// command prompt arrives, and 0.8 s later Down, Enter (meant to pick the
/// question's second option) arrive. They answer the question; the child's
/// prompt waits under it, then shows and takes no key of that moment.
#[test]
fn issue_787_keys_for_the_question_never_answer_the_childs_prompt() {
    let (tx_raw, mut rx) = unbounded_channel::<AppEvent>();
    let features = Features::with_defaults();
    let mut pane = guarded_pane(AppEventSender::new(tx_raw));
    let start = Instant::now();
    pane.push_user_input_request(user_input_request());
    pane.pre_draw_tick_at(start);

    let child_at = start + Duration::from_secs(5);
    pane.push_approval_request(child_exec_request(), &features);
    pane.pre_draw_tick_at(child_at);
    assert_eq!(pane.view_stack.len(), 2);
    let area = Rect::new(0, 0, 80, pane.desired_height(/*width*/ 80));
    let screen = render_snapshot(&pane, area);
    assert!(
        screen.contains("Continue?"),
        "the question stays on top: {screen}"
    );
    assert!(!screen.contains("sleep 60"), "{screen}");
    assert!(
        screen.contains("Another request is waiting; it shows after you answer this one."),
        "{screen}"
    );

    let keys_at = child_at + Duration::from_millis(800);
    press_at(&mut pane, &[KeyCode::Down, KeyCode::Enter], keys_at);
    let events: Vec<AppEvent> = std::iter::from_fn(|| rx.try_recv().ok()).collect();
    assert!(
        events.iter().any(|event| matches!(
            event,
            AppEvent::CodexOp(crate::app_command::AppCommand::UserInputAnswer { .. })
        )),
        "the question took its keys"
    );
    assert!(
        !events
            .iter()
            .any(|event| matches!(event, AppEvent::SubmitThreadOp { .. })),
        "no key reached the child's prompt"
    );

    // The question closed; the child's prompt shows now. A second Down,
    // Enter typed on (a double press) is dropped, and so is a third pair
    // that follows within the guard of the last dropped key.
    let shown_at = keys_at + Duration::from_millis(16);
    pane.pre_draw_tick_at(shown_at);
    assert_eq!(pane.view_stack.len(), 1);
    press_at(
        &mut pane,
        &[KeyCode::Down, KeyCode::Enter],
        shown_at + Duration::from_millis(100),
    );
    press_at(
        &mut pane,
        &[KeyCode::Down, KeyCode::Enter],
        shown_at + Duration::from_millis(1050),
    );
    assert_eq!(exec_decisions(&mut rx), Vec::new());
    let area = Rect::new(0, 0, 80, pane.desired_height(/*width*/ 80));
    assert_snapshot!(
        "issue_787_child_prompt_after_dropped_keys",
        render_snapshot(&pane, area)
    );

    // After a quiet second, Enter alone still answers nothing; a deliberate
    // choice does, and the first option is still highlighted.
    let later = shown_at + Duration::from_millis(1050) + REQUEST_INPUT_GUARD;
    press_at(&mut pane, &[KeyCode::Enter], later);
    assert_eq!(exec_decisions(&mut rx), Vec::new());
    press_at(
        &mut pane,
        &[KeyCode::Down, KeyCode::Up, KeyCode::Enter],
        later,
    );
    assert_eq!(
        exec_decisions(&mut rx),
        vec![CommandExecutionApprovalDecision::Accept]
    );
}

/// Type-ahead: keys that arrive before a new prompt was drawn, or less than
/// [`REQUEST_INPUT_GUARD`] after, never answer it, for every kind of
/// request; a paste neither. After the guard a deliberate choice answers.
#[test]
fn issue_787_type_ahead_never_answers_any_kind_of_request() {
    use crate::claude_panes::approval::ApprovalResponder;
    use crate::claude_panes::approval::ClaudeApprovalRequest;

    #[derive(Clone, Copy, Debug)]
    enum Kind {
        Exec,
        Patch,
        Permissions,
        Elicitation,
        McpApproval,
        Question,
        Link,
        Claude,
    }
    for kind in [
        Kind::Exec,
        Kind::Patch,
        Kind::Permissions,
        Kind::Elicitation,
        Kind::McpApproval,
        Kind::Question,
        Kind::Link,
        Kind::Claude,
    ] {
        for drawn in [false, true] {
            let (tx_raw, mut rx) = unbounded_channel::<AppEvent>();
            let tx = AppEventSender::new(tx_raw);
            let features = Features::with_defaults();
            let mut pane = guarded_pane(tx.clone());
            let mut claude_rx = None;
            match kind {
                Kind::Exec => pane.push_approval_request(child_exec_request(), &features),
                Kind::Patch => pane.push_approval_request(patch_request(), &features),
                Kind::Permissions => pane.push_approval_request(permissions_request(), &features),
                Kind::Elicitation => {
                    pane.push_approval_request(elicitation_approval_request(), &features)
                }
                Kind::McpApproval => {
                    pane.push_mcp_server_elicitation_request(mcp_approval_request())
                }
                Kind::Question => pane.push_user_input_request(user_input_request()),
                Kind::Link => pane.show_app_link_view(url_link_view(tx.clone())),
                Kind::Claude => {
                    let (responder, answer) = ApprovalResponder::new();
                    pane.show_claude_approval(ClaudeApprovalRequest {
                        pane_id: "claude-1234abcd".to_string(),
                        pane_title: "Claude".to_string(),
                        cwd: PathBuf::from("/w"),
                        tool_name: "Bash".to_string(),
                        tool_use_id: None,
                        details: crate::claude_panes::approval::details(
                            "Bash",
                            &serde_json::json!({ "command": "touch x" }),
                            str::to_string,
                        ),
                        responder,
                    });
                    claude_rx = Some(answer);
                }
            }
            let start = Instant::now();
            let keys_at = if drawn {
                pane.pre_draw_tick_at(start);
                start + Duration::from_millis(900)
            } else {
                start
            };
            pane.handle_paste_at("y".to_string(), keys_at);
            for keys in [
                &[KeyCode::Enter][..],
                &[KeyCode::Down, KeyCode::Enter],
                &[KeyCode::Right, KeyCode::Enter],
                &[KeyCode::Char('1'), KeyCode::Enter],
                &[KeyCode::Char('y'), KeyCode::Enter],
                &[KeyCode::Char('2'), KeyCode::Enter],
                &[KeyCode::Esc],
            ] {
                press_at(&mut pane, keys, keys_at);
            }
            let answered = std::iter::from_fn(|| rx.try_recv().ok()).any(|event| {
                matches!(
                    event,
                    AppEvent::SubmitThreadOp { .. }
                        | AppEvent::CodexOp(_)
                        | AppEvent::OpenUrlInBrowser { .. }
                )
            });
            let claude_answered = claude_rx
                .as_mut()
                .is_some_and(|answer| answer.try_recv().is_ok());
            assert!(
                !answered && !claude_answered,
                "{kind:?} drawn={drawn}: answered by type-ahead"
            );
            assert_eq!(pane.view_stack.len(), 1, "{kind:?} drawn={drawn}");
            if !matches!(kind, Kind::Claude | Kind::Link) {
                let area = Rect::new(0, 0, 100, pane.desired_height(/*width*/ 100));
                let screen = render_snapshot(&pane, area);
                assert!(screen.contains(EARLY), "{kind:?} drawn={drawn}: {screen}");
            }

            // A deliberate choice after a quiet guard answers it.
            pane.pre_draw_tick_at(keys_at);
            let later = keys_at + REQUEST_INPUT_GUARD * 2;
            if let Some(answer) = claude_rx.as_mut() {
                // Allow needs →, a pause, then Enter (#218); its own guard
                // runs on the clock from its first draw.
                let area = Rect::new(0, 0, 100, pane.desired_height(/*width*/ 100));
                render_snapshot(&pane, area);
                let pause = crate::bottom_pane::claude_approval_view::INPUT_GUARD
                    + Duration::from_millis(50);
                std::thread::sleep(pause);
                press_at(&mut pane, &[KeyCode::Right], later);
                std::thread::sleep(pause);
                press_at(&mut pane, &[KeyCode::Enter], later);
                assert_eq!(answer.try_recv(), Ok(true), "{kind:?} drawn={drawn}");
            } else {
                press_at(
                    &mut pane,
                    &[KeyCode::Down, KeyCode::Up, KeyCode::Enter],
                    later,
                );
                let answered = std::iter::from_fn(|| rx.try_recv().ok()).any(|event| {
                    matches!(
                        event,
                        AppEvent::SubmitThreadOp { .. }
                            | AppEvent::CodexOp(_)
                            | AppEvent::OpenUrlInBrowser { .. }
                    )
                });
                assert!(
                    answered,
                    "{kind:?} drawn={drawn}: a deliberate choice answers"
                );
            }
        }
    }
}

/// A held or repeated key keeps the prompt guarded: each dropped key
/// restarts the guard. Esc is dropped too (it would cancel the request or
/// interrupt the turn); after a quiet guard it declines.
#[test]
fn issue_787_held_keys_and_esc_stay_dropped_until_quiet() {
    let (tx_raw, mut rx) = unbounded_channel::<AppEvent>();
    let features = Features::with_defaults();
    let mut pane = guarded_pane(AppEventSender::new(tx_raw));
    pane.push_approval_request(child_exec_request(), &features);
    let start = Instant::now();
    pane.pre_draw_tick_at(start);
    let mut last = start;
    for step in 1..=6 {
        last = start + Duration::from_millis(400 * step);
        press_at(&mut pane, &[KeyCode::Down], last);
        pane.handle_key_event_at(
            KeyEvent::new_with_kind(KeyCode::Enter, KeyModifiers::NONE, KeyEventKind::Repeat),
            last,
        );
    }
    press_at(
        &mut pane,
        &[KeyCode::Esc],
        last + Duration::from_millis(900),
    );
    assert_eq!(exec_decisions(&mut rx), Vec::new());
    assert_eq!(pane.view_stack.len(), 1);

    press_at(
        &mut pane,
        &[KeyCode::Esc],
        last + Duration::from_millis(900) + REQUEST_INPUT_GUARD,
    );
    assert_eq!(
        exec_decisions(&mut rx),
        vec![CommandExecutionApprovalDecision::Cancel]
    );
}

/// The next request of one prompt's queue is new too: after Enter answers
/// the first, the keys right after it (a double press) never answer the
/// second, which was not drawn yet.
#[test]
fn issue_787_next_queued_request_takes_no_key_of_the_answer() {
    let (tx_raw, mut rx) = unbounded_channel::<AppEvent>();
    let features = Features::with_defaults();
    let mut pane = guarded_pane(AppEventSender::new(tx_raw));
    pane.push_approval_request(exec_request(), &features);
    pane.push_approval_request(child_exec_request(), &features);
    let start = Instant::now();
    pane.pre_draw_tick_at(start);
    let at = start + REQUEST_INPUT_GUARD;
    press_at(&mut pane, &[KeyCode::Down, KeyCode::Up, KeyCode::Enter], at);
    assert_eq!(exec_decisions(&mut rx).len(), 1);
    press_at(&mut pane, &[KeyCode::Down, KeyCode::Enter], at);
    pane.pre_draw_tick_at(at);
    press_at(
        &mut pane,
        &[KeyCode::Down, KeyCode::Enter],
        at + Duration::from_millis(500),
    );
    assert_eq!(exec_decisions(&mut rx), Vec::new());
}

/// A request that arrives while another of a different kind is on screen
/// waits under it, first in, first out.
#[test]
fn issue_787_new_requests_wait_under_the_one_on_screen() {
    let (tx_raw, _rx) = unbounded_channel::<AppEvent>();
    let features = Features::with_defaults();
    let mut pane = test_pane(AppEventSender::new(tx_raw));
    let screen = |pane: &BottomPane| {
        let area = Rect::new(0, 0, 80, pane.desired_height(/*width*/ 80));
        render_snapshot(pane, area)
    };
    pane.push_approval_request(exec_request(), &features);
    pane.push_user_input_request(user_input_request());
    pane.push_mcp_server_elicitation_request(mcp_approval_request());
    assert_eq!(pane.view_stack.len(), 3);
    let first = screen(&pane);
    assert!(first.contains("echo ok"), "{first}");
    assert!(first.contains("Another request is waiting"), "{first}");
    pane.view_stack.pop();
    let second = screen(&pane);
    assert!(second.contains("Continue?"), "{second}");
    assert!(second.contains("Another request is waiting"), "{second}");
    pane.view_stack.pop();
    let third = screen(&pane);
    assert!(third.contains("Allow this request?"), "{third}");
    assert!(!third.contains("Another request is waiting"), "{third}");
}
