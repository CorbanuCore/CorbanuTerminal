//! #817 (AGG-R14, second targeted run): the child agent's command prompt,
//! revealed when the parent's question was answered, refused an Enter
//! pressed 11 s after it appeared and only then asked for ↑/↓ first, so the
//! command never ran. Keys are ignored only before a prompt is drawn and in
//! the second after it (#787); after that a deliberate Enter answers it. The
//! prompt asks for ↑/↓ first only when keys really were dropped for it.

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

/// The child agent's command prompt of the AGG-R14 run.
fn child_exec_request() -> ApprovalRequest {
    ApprovalRequest::Exec(ExecApprovalRequest {
        thread_id: codex_protocol::ThreadId::new(),
        thread_label: Some("Agent (01a13d07)".to_string()),
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

fn two_question_request() -> ToolRequestUserInputParams {
    let mut request = user_input_request();
    let mut second = request.questions[0].clone();
    second.id = "q2".to_string();
    second.question = "And then?".to_string();
    request.questions.push(second);
    request
}

fn press_at(pane: &mut BottomPane, keys: &[KeyCode], at: Instant) {
    for key in keys {
        pane.handle_key_event_at(KeyEvent::new(*key, KeyModifiers::NONE), at);
    }
}

fn screen(pane: &BottomPane) -> String {
    let area = Rect::new(0, 0, 100, pane.desired_height(/*width*/ 100));
    render_snapshot(pane, area)
}

/// What answered a request: exec decisions, question answers, MCP
/// elicitation answers and link actions.
#[derive(Debug, PartialEq, Eq)]
enum Answer {
    Exec(CommandExecutionApprovalDecision),
    Question,
    Elicitation,
    Link,
}

fn answers(rx: &mut tokio::sync::mpsc::UnboundedReceiver<AppEvent>) -> Vec<Answer> {
    std::iter::from_fn(|| rx.try_recv().ok())
        .filter_map(|event| match event {
            AppEvent::SubmitThreadOp {
                op: Op::ExecApproval { decision, .. },
                ..
            } => Some(Answer::Exec(decision)),
            AppEvent::SubmitThreadOp {
                op: Op::ResolveElicitation { .. },
                ..
            } => Some(Answer::Elicitation),
            AppEvent::CodexOp(crate::app_command::AppCommand::UserInputAnswer { .. }) => {
                Some(Answer::Question)
            }
            AppEvent::OpenUrlInBrowser { .. } => Some(Answer::Link),
            _ => None,
        })
        .collect()
}

/// The AGG-R14 run: the parent's `wait_agent` question is on screen with the
/// child's command prompt waiting under it. Enter answers the question; the
/// child's prompt shows at once. Nothing is pressed for 11 s, then Enter
/// approves the highlighted "Yes, proceed": no notice asks for ↑/↓ first.
#[test]
fn issue_817_child_prompt_shown_after_the_question_takes_a_later_enter() {
    let (tx_raw, mut rx) = unbounded_channel::<AppEvent>();
    let features = Features::with_defaults();
    let mut pane = guarded_pane(AppEventSender::new(tx_raw));
    let start = Instant::now();
    pane.push_user_input_request(user_input_request());
    pane.pre_draw_tick_at(start);
    pane.push_approval_request(child_exec_request(), &features);
    pane.pre_draw_tick_at(start + Duration::from_secs(1));

    let answered_at = start + Duration::from_secs(3);
    press_at(&mut pane, &[KeyCode::Enter], answered_at);
    assert_eq!(answers(&mut rx), vec![Answer::Question]);
    let shown_at = answered_at + Duration::from_millis(16);
    pane.pre_draw_tick_at(shown_at);
    assert_eq!(pane.view_stack.len(), 1);
    let shown = screen(&pane);
    assert!(shown.contains("sleep 60"), "{shown}");
    assert!(!shown.contains("This is a new request"), "{shown}");
    assert!(!shown.contains(EARLY), "{shown}");

    let enter_at = shown_at + Duration::from_secs(11);
    pane.pre_draw_tick_at(enter_at);
    press_at(&mut pane, &[KeyCode::Enter], enter_at);
    assert_eq!(
        answers(&mut rx),
        vec![Answer::Exec(CommandExecutionApprovalDecision::Accept)],
        "{}",
        screen(&pane)
    );
    assert!(pane.view_stack.is_empty());
}

/// How a prompt comes on screen.
#[derive(Clone, Copy, Debug)]
enum Arrival {
    /// On an empty pane.
    Alone,
    /// A few ms after another prompt was answered.
    AfterAnswer,
    /// Over another view: it waits under it and shows when it closes.
    UnderView,
    /// While the person types: held back until typing pauses (approvals).
    WhileTyping,
    /// The next request of the same prompt's queue.
    Queued,
    /// The next question of the same form.
    NextQuestion,
}

#[derive(Clone, Copy, Debug)]
enum Kind {
    Exec,
    Question,
    McpApproval,
}

/// Puts a prompt of `kind` on screen as `arrival` says and returns when it
/// was first drawn. Earlier prompts are answered deliberately.
fn arrive(
    pane: &mut BottomPane,
    rx: &mut tokio::sync::mpsc::UnboundedReceiver<AppEvent>,
    kind: Kind,
    arrival: Arrival,
    start: Instant,
) -> Instant {
    let features = Features::with_defaults();
    let push = |pane: &mut BottomPane| match kind {
        Kind::Exec => pane.push_approval_request(child_exec_request(), &features),
        Kind::Question => pane.push_user_input_request(user_input_request()),
        Kind::McpApproval => pane.push_mcp_server_elicitation_request(mcp_approval_request()),
    };
    let deliberate = [KeyCode::Down, KeyCode::Up, KeyCode::Enter];
    let later = start + REQUEST_INPUT_GUARD * 2;
    let shown_at = later + Duration::from_millis(16);
    match arrival {
        Arrival::Alone => {
            push(pane);
            pane.pre_draw_tick_at(start);
            return start;
        }
        Arrival::AfterAnswer => {
            pane.push_approval_request(exec_request(), &features);
            pane.pre_draw_tick_at(start);
            press_at(pane, &deliberate, later);
            assert_eq!(answers(rx).len(), 1, "{kind:?} {arrival:?}");
            push(pane);
        }
        Arrival::UnderView => {
            pane.show_selection_view(picker());
            push(pane);
            pane.pre_draw_tick_at(start);
            press_at(pane, &[KeyCode::Esc], later);
            assert!(answers(rx).is_empty(), "{kind:?} {arrival:?}");
        }
        Arrival::WhileTyping => {
            pane.last_composer_activity_at = Some(later - APPROVAL_PROMPT_TYPING_IDLE_DELAY);
            push(pane);
        }
        Arrival::Queued => {
            match kind {
                Kind::Exec => pane.push_approval_request(exec_request(), &features),
                Kind::Question => pane.push_user_input_request(user_input_request()),
                Kind::McpApproval => {
                    pane.push_mcp_server_elicitation_request(mcp_approval_request())
                }
            }
            push(pane);
            pane.pre_draw_tick_at(start);
            press_at(pane, &deliberate, later);
            assert_eq!(answers(rx).len(), 1, "{kind:?} {arrival:?}");
        }
        Arrival::NextQuestion => {
            pane.push_user_input_request(two_question_request());
            pane.pre_draw_tick_at(start);
            press_at(pane, &deliberate, later);
            assert!(answers(rx).is_empty(), "{kind:?} {arrival:?}");
        }
    }
    pane.pre_draw_tick_at(shown_at);
    shown_at
}

/// The arrivals of a kind: a form has no internal queue of approvals, and
/// only questions have a next question.
fn arrivals(kind: Kind) -> Vec<Arrival> {
    let mut arrivals = vec![
        Arrival::Alone,
        Arrival::AfterAnswer,
        Arrival::UnderView,
        Arrival::Queued,
    ];
    match kind {
        Kind::Exec => arrivals.push(Arrival::WhileTyping),
        Kind::Question => arrivals.push(Arrival::NextQuestion),
        Kind::McpApproval => {}
    }
    arrivals
}

fn expected(kind: Kind) -> Answer {
    match kind {
        Kind::Exec => Answer::Exec(CommandExecutionApprovalDecision::Accept),
        Kind::Question => Answer::Question,
        Kind::McpApproval => Answer::Elicitation,
    }
}

/// However a prompt comes on screen, once it has shown for the guard with
/// no key dropped, Enter alone answers its highlighted option, and nothing
/// told the person to press ↑/↓ first.
#[test]
fn issue_817_a_deliberate_enter_after_the_guard_answers_every_prompt() {
    for kind in [Kind::Exec, Kind::Question, Kind::McpApproval] {
        for arrival in arrivals(kind) {
            let (tx_raw, mut rx) = unbounded_channel::<AppEvent>();
            let mut pane = guarded_pane(AppEventSender::new(tx_raw));
            let shown_at = arrive(&mut pane, &mut rx, kind, arrival, Instant::now());
            let shown = screen(&pane);
            assert!(
                !shown.contains("new request") && !shown.contains(EARLY),
                "{kind:?} {arrival:?}: {shown}"
            );
            let enter_at = shown_at + REQUEST_INPUT_GUARD + Duration::from_millis(100);
            pane.pre_draw_tick_at(enter_at);
            press_at(&mut pane, &[KeyCode::Enter], enter_at);
            assert_eq!(
                answers(&mut rx),
                vec![expected(kind)],
                "{kind:?} {arrival:?}: {}",
                screen(&pane)
            );
        }
    }
}

/// #787's protection holds for every arrival: a key that reaches the prompt
/// within the guard after it was drawn (a double Enter) answers nothing; the
/// prompt then says keys were ignored and needs ↑/↓ before Enter.
#[test]
fn issue_817_type_ahead_still_never_answers_a_new_prompt() {
    for kind in [Kind::Exec, Kind::Question, Kind::McpApproval] {
        for arrival in arrivals(kind) {
            let (tx_raw, mut rx) = unbounded_channel::<AppEvent>();
            let mut pane = guarded_pane(AppEventSender::new(tx_raw));
            let shown_at = arrive(&mut pane, &mut rx, kind, arrival, Instant::now());
            press_at(
                &mut pane,
                &[KeyCode::Enter],
                shown_at + Duration::from_millis(200),
            );
            assert_eq!(answers(&mut rx), Vec::new(), "{kind:?} {arrival:?}");
            let told = screen(&pane);
            assert!(told.contains(EARLY), "{kind:?} {arrival:?}: {told}");

            // Keys really were dropped for this prompt: Enter alone still
            // waits after a quiet guard; ↑/↓ then Enter answers.
            let later = shown_at + Duration::from_millis(200) + REQUEST_INPUT_GUARD * 2;
            press_at(&mut pane, &[KeyCode::Enter], later);
            assert_eq!(answers(&mut rx), Vec::new(), "{kind:?} {arrival:?}");
            press_at(
                &mut pane,
                &[KeyCode::Down, KeyCode::Up, KeyCode::Enter],
                later,
            );
            assert_eq!(
                answers(&mut rx),
                vec![expected(kind)],
                "{kind:?} {arrival:?}"
            );
        }
    }
}

/// Keys dropped for the question (in its first second) make only the
/// question ask for ↑/↓ first. The child's prompt shown after it took no
/// dropped key, so a later Enter answers it.
#[test]
fn issue_817_keys_dropped_for_an_earlier_prompt_do_not_hold_the_next() {
    let (tx_raw, mut rx) = unbounded_channel::<AppEvent>();
    let features = Features::with_defaults();
    let mut pane = guarded_pane(AppEventSender::new(tx_raw));
    let start = Instant::now();
    pane.push_user_input_request(user_input_request());
    pane.pre_draw_tick_at(start);
    pane.push_approval_request(child_exec_request(), &features);
    press_at(
        &mut pane,
        &[KeyCode::Enter],
        start + Duration::from_millis(300),
    );
    assert!(screen(&pane).contains(EARLY));

    let answered_at = start + Duration::from_secs(3);
    press_at(&mut pane, &[KeyCode::Enter], answered_at);
    assert_eq!(answers(&mut rx), Vec::new(), "the question needs ↑/↓ first");
    press_at(
        &mut pane,
        &[KeyCode::Down, KeyCode::Up, KeyCode::Enter],
        answered_at,
    );
    assert_eq!(answers(&mut rx), vec![Answer::Question]);

    let shown_at = answered_at + Duration::from_millis(16);
    pane.pre_draw_tick_at(shown_at);
    let shown = screen(&pane);
    assert!(shown.contains("sleep 60"), "{shown}");
    assert!(!shown.contains(EARLY), "{shown}");
    let enter_at = shown_at + Duration::from_secs(2);
    press_at(&mut pane, &[KeyCode::Enter], enter_at);
    assert_eq!(
        answers(&mut rx),
        vec![Answer::Exec(CommandExecutionApprovalDecision::Accept)]
    );
}

/// Typing that goes on across a request swap ("py": `p` for the first
/// request, which another client then answers) never answers the next one:
/// it shows anew, and keys in its first second are dropped (#787).
#[test]
fn issue_817_typing_on_across_a_request_swap_answers_nothing() {
    let (tx_raw, mut rx) = unbounded_channel::<AppEvent>();
    let features = Features::with_defaults();
    let mut pane = guarded_pane(AppEventSender::new(tx_raw));
    let start = Instant::now();
    pane.push_approval_request(exec_request(), &features);
    pane.push_approval_request(child_exec_request(), &features);
    pane.pre_draw_tick_at(start);

    let at = start + Duration::from_secs(2);
    press_at(&mut pane, &[KeyCode::Char('p')], at);
    let ApprovalRequest::Exec(first) = exec_request() else {
        unreachable!("exec request");
    };
    assert!(
        pane.dismiss_app_server_request(&ResolvedAppServerRequest::ExecApproval { id: first.id })
    );
    pane.pre_draw_tick_at(at + Duration::from_millis(16));
    press_at(
        &mut pane,
        &[KeyCode::Char('y'), KeyCode::Enter],
        at + Duration::from_millis(150),
    );
    assert_eq!(answers(&mut rx), Vec::new());
    assert!(screen(&pane).contains(EARLY));
}

/// A server's link request: Enter opens the link and shows the confirmation
/// screen, which takes no double Enter (#514, #787) but a later one.
#[test]
fn issue_817_link_confirmation_takes_a_later_enter_only() {
    for double in [true, false] {
        let (tx_raw, mut rx) = unbounded_channel::<AppEvent>();
        let tx = AppEventSender::new(tx_raw);
        let mut pane = guarded_pane(tx.clone());
        let start = Instant::now();
        pane.show_app_link_view(url_link_view(tx));
        pane.pre_draw_tick_at(start);
        let at = start + Duration::from_secs(2);
        press_at(&mut pane, &[KeyCode::Enter], at);
        assert_eq!(answers(&mut rx), vec![Answer::Link], "double={double}");
        if double {
            press_at(&mut pane, &[KeyCode::Enter], at);
        }
        pane.pre_draw_tick_at(at + Duration::from_millis(16));
        let later = at + REQUEST_INPUT_GUARD * 2;
        press_at(&mut pane, &[KeyCode::Enter], later);
        if double {
            assert_eq!(answers(&mut rx), Vec::new());
            press_at(
                &mut pane,
                &[KeyCode::Down, KeyCode::Up, KeyCode::Enter],
                later,
            );
        }
        assert_eq!(
            answers(&mut rx),
            vec![Answer::Elicitation],
            "double={double}"
        );
    }
}

fn second_mcp_approval_request() -> McpServerElicitationFormRequest {
    let params = codex_app_server_protocol::McpServerElicitationRequestParams {
        thread_id: "thread-1".to_string(),
        turn_id: Some("turn-1".to_string()),
        server_name: "server-1".to_string(),
        request: codex_app_server_protocol::McpServerElicitationRequest::Form {
            meta: None,
            message: "Allow the second request?".to_string(),
            requested_schema: serde_json::from_value(serde_json::json!({
                "type": "object",
                "properties": {},
            }))
            .expect("schema"),
        },
    };
    McpServerElicitationFormRequest::from_app_server_request(
        codex_protocol::ThreadId::default(),
        codex_app_server_protocol::RequestId::String("request-2".to_string()),
        &params,
    )
    .expect("approval request")
}

/// Review r1: a stray character typed into one request, which another
/// client then answers, does not block a deliberate Enter on the next
/// request of the same prompt; typing that goes on as it appears is still
/// dropped (see the swap test above).
#[test]
fn issue_817_text_typed_into_a_resolved_request_does_not_hold_the_next() {
    for kind in [Kind::Exec, Kind::Question, Kind::McpApproval] {
        let (tx_raw, mut rx) = unbounded_channel::<AppEvent>();
        let features = Features::with_defaults();
        let mut pane = guarded_pane(AppEventSender::new(tx_raw));
        let start = Instant::now();
        let resolved = match kind {
            Kind::Exec => {
                pane.push_approval_request(exec_request(), &features);
                pane.push_approval_request(child_exec_request(), &features);
                let ApprovalRequest::Exec(first) = exec_request() else {
                    unreachable!("exec request");
                };
                ResolvedAppServerRequest::ExecApproval { id: first.id }
            }
            Kind::Question => {
                pane.push_user_input_request(user_input_request());
                let mut second = user_input_request();
                second.item_id = "call-2".to_string();
                pane.push_user_input_request(second);
                ResolvedAppServerRequest::UserInput {
                    call_id: "call-1".to_string(),
                }
            }
            Kind::McpApproval => {
                pane.push_mcp_server_elicitation_request(mcp_approval_request());
                pane.push_mcp_server_elicitation_request(second_mcp_approval_request());
                ResolvedAppServerRequest::McpElicitation {
                    server_name: "server-1".to_string(),
                    request_id: codex_app_server_protocol::RequestId::String(
                        "request-1".to_string(),
                    ),
                }
            }
        };
        assert_eq!(
            pane.view_stack.len(),
            1,
            "{kind:?}: one prompt, two requests"
        );
        pane.pre_draw_tick_at(start);

        let at = start + Duration::from_secs(2);
        press_at(&mut pane, &[KeyCode::Char('x')], at);
        assert_eq!(answers(&mut rx), Vec::new(), "{kind:?}");
        assert!(pane.dismiss_app_server_request(&resolved), "{kind:?}");
        let shown_at = at + Duration::from_millis(16);
        pane.pre_draw_tick_at(shown_at);
        let shown = screen(&pane);
        assert!(!shown.contains("Typed text"), "{kind:?}: {shown}");
        assert!(!shown.contains(EARLY), "{kind:?}: {shown}");

        let enter_at = shown_at + REQUEST_INPUT_GUARD + Duration::from_millis(500);
        pane.pre_draw_tick_at(enter_at);
        press_at(&mut pane, &[KeyCode::Enter], enter_at);
        assert_eq!(
            answers(&mut rx),
            vec![expected(kind)],
            "{kind:?}: {}",
            screen(&pane)
        );
    }
}
