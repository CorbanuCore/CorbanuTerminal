//! PF-25-S01: the grant option and review in the command approval.

use super::*;
use crate::legacy_core::security_grant::GrantOffer;
use crossterm::event::KeyModifiers;
use insta::assert_snapshot;
use pretty_assertions::assert_eq;
use std::cell::RefCell;
use tokio::sync::mpsc::UnboundedReceiver;
use tokio::sync::mpsc::unbounded_channel;

thread_local! {
    /// The offer Core would make for the test's request.
    pub(super) static TEST_OFFER: RefCell<Option<GrantOffer>> = const { RefCell::new(None) };
}

fn thread() -> ThreadId {
    ThreadId::from_string("019a0000-0000-7000-8000-000000000001").expect("thread id")
}

fn offer() -> GrantOffer {
    GrantOffer {
        thread: thread(),
        approval_id: "call-1".to_string(),
        actor_chain: vec![
            "human:human".to_string(),
            "agent:019a0000-0000-7000-8000-000000000001".to_string(),
        ],
        resource: "protected_data/sandbox_protected_paths".to_string(),
        action: "execute".to_string(),
        operation: "command:sha256:3f2a".to_string(),
        command: vec!["cat".to_string(), "../home/team-notes.txt".to_string()],
        cwd: "/work/project".to_string(),
        lifetime_seconds: 600,
        epoch: 3,
        revocation_generation: 0,
    }
}

fn request() -> ApprovalRequest {
    request_in(thread())
}

fn request_in(thread_id: ThreadId) -> ApprovalRequest {
    ApprovalRequest::Exec(ExecApprovalRequest {
        thread_id,
        thread_label: None,
        id: "call-1".to_string(),
        environment_id: None,
        command: vec!["cat".to_string(), "../home/team-notes.txt".to_string()],
        reason: None,
        available_decisions: vec![
            CommandExecutionApprovalDecision::Accept,
            CommandExecutionApprovalDecision::Cancel,
        ],
        network_approval_context: None,
        additional_permissions: None,
    })
}

fn features(security_levels: bool) -> Features {
    let mut features = Features::with_defaults();
    if security_levels {
        features.enable(Feature::SecurityLevels);
    }
    features
}

fn overlay(
    offered: Option<GrantOffer>,
    security_levels: bool,
) -> (ApprovalOverlay, UnboundedReceiver<AppEvent>) {
    overlay_for(request(), offered, security_levels)
}

fn overlay_for(
    request: ApprovalRequest,
    offered: Option<GrantOffer>,
    security_levels: bool,
) -> (ApprovalOverlay, UnboundedReceiver<AppEvent>) {
    TEST_OFFER.with(|slot| *slot.borrow_mut() = offered);
    let (tx, rx) = unbounded_channel::<AppEvent>();
    let keymap = crate::keymap::RuntimeKeymap::defaults();
    let view = ApprovalOverlay::new(
        request,
        AppEventSender::new(tx),
        features(security_levels),
        keymap.approval,
        keymap.list,
    );
    (view, rx)
}

fn key(view: &mut ApprovalOverlay, code: KeyCode) {
    view.handle_key_event(KeyEvent::new(code, KeyModifiers::NONE));
}

fn render(view: &ApprovalOverlay, width: u16) -> String {
    let height = view.desired_height(width);
    let mut buf = Buffer::empty(Rect::new(0, 0, width, height));
    view.render(Rect::new(0, 0, width, height), &mut buf);
    (0..buf.area.height)
        .map(|row| {
            (0..buf.area.width)
                .map(|col| buf[(col, row)].symbol().to_string())
                .collect::<String>()
                .trim_end()
                .to_string()
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn exec_decisions(rx: &mut UnboundedReceiver<AppEvent>) -> Vec<CommandExecutionApprovalDecision> {
    let mut decisions = Vec::new();
    while let Ok(event) = rx.try_recv() {
        if let AppEvent::SubmitThreadOp {
            op: Op::ExecApproval { decision, .. },
            ..
        } = event
        {
            decisions.push(decision);
        }
    }
    decisions
}

/// Only an offer from Core, with `security_levels` on, adds the option.
#[test]
fn pf_25_s01_grant_option_only_when_core_offers_one() {
    for (offered, flag, expected) in [
        (Some(offer()), true, true),
        (Some(offer()), false, false),
        (None, true, false),
    ] {
        let (view, _rx) = overlay(offered, flag);
        assert_eq!(
            view.options
                .iter()
                .any(|option| matches!(option.decision, ApprovalDecision::SecurityGrant)),
            expected
        );
    }
}

/// `g` highlights the option; Enter opens the review with every field, and
/// nothing is answered yet.
#[test]
fn pf_25_s01_grant_review_shows_exact_scope() {
    let (mut view, mut rx) = overlay(Some(offer()), true);
    assert_snapshot!("pf_25_s01_grant_option", render(&view, 100));
    key(&mut view, KeyCode::Char('g'));
    key(&mut view, KeyCode::Enter);
    assert!(view.grant_review.is_some());
    assert_snapshot!("pf_25_s01_grant_review", render(&view, 100));
    key(&mut view, KeyCode::Char('u'));
    assert_snapshot!("pf_25_s01_grant_review_until_expiry", render(&view, 100));
    assert_eq!(exec_decisions(&mut rx), Vec::new());
    assert!(!view.is_complete());
}

/// Esc in the review goes back to the approval: nothing granted or answered.
#[test]
fn pf_25_s01_esc_returns_to_the_approval() {
    let (mut view, mut rx) = overlay(Some(offer()), true);
    assert!(!view.prefer_esc_to_handle_key_event());
    key(&mut view, KeyCode::Char('g'));
    key(&mut view, KeyCode::Enter);
    // The pane hands Esc to the review instead of cancelling the request.
    assert!(view.prefer_esc_to_handle_key_event());
    key(&mut view, KeyCode::Esc);
    assert!(view.grant_review.is_none());
    assert!(!view.is_complete());
    assert_eq!(exec_decisions(&mut rx), Vec::new());
    // The approval itself still answers normally.
    key(&mut view, KeyCode::Char('y'));
    key(&mut view, KeyCode::Enter);
    assert_eq!(
        exec_decisions(&mut rx),
        vec![CommandExecutionApprovalDecision::Accept]
    );
}

/// An offer Core does not hold (a forged or ended one) grants nothing: the
/// review shows why and stays open; the command is not approved.
#[test]
fn pf_25_s01_unknown_offer_is_refused_visibly() {
    let (mut view, mut rx) = overlay(Some(offer()), true);
    key(&mut view, KeyCode::Char('g'));
    key(&mut view, KeyCode::Enter);
    render(&view, 100);
    key(&mut view, KeyCode::Down);
    key(&mut view, KeyCode::Enter);
    assert!(view.grant_review.is_some());
    assert!(!view.is_complete());
    assert_eq!(exec_decisions(&mut rx), Vec::new());
    assert_snapshot!("pf_25_s01_grant_refused", render(&view, 100));
}

/// Typed text and other keys in the review do nothing.
#[test]
fn pf_25_s01_review_ignores_other_keys() {
    let (mut view, mut rx) = overlay(Some(offer()), true);
    key(&mut view, KeyCode::Char('g'));
    key(&mut view, KeyCode::Enter);
    for code in [KeyCode::Char('y'), KeyCode::Char('n'), KeyCode::Char('g')] {
        key(&mut view, code);
    }
    assert!(view.grant_review.is_some());
    assert_eq!(exec_decisions(&mut rx), Vec::new());
}

/// Through the pane, as in the terminal: Esc in the review goes back to the
/// approval instead of declining the command.
#[test]
fn pf_25_s01_pane_esc_in_review_keeps_the_request() {
    use crate::bottom_pane::BottomPane;
    use crate::bottom_pane::BottomPaneParams;
    TEST_OFFER.with(|slot| *slot.borrow_mut() = Some(offer()));
    let (tx, mut rx) = unbounded_channel::<AppEvent>();
    let mut pane = BottomPane::new(BottomPaneParams {
        app_event_tx: AppEventSender::new(tx),
        frame_requester: crate::tui::FrameRequester::test_dummy(),
        has_input_focus: true,
        enhanced_keys_supported: false,
        placeholder_text: "Ask".to_string(),
        disable_paste_burst: true,
        animations_enabled: true,
        skills: Some(Vec::new()),
    });
    pane.push_approval_request(request(), &features(/*security_levels*/ true));
    for code in [KeyCode::Char('g'), KeyCode::Enter, KeyCode::Esc] {
        pane.handle_key_event(KeyEvent::new(code, KeyModifiers::NONE));
    }
    assert!(pane.has_active_view());
    assert_eq!(exec_decisions(&mut rx), Vec::new());
    // A second Esc, on the approval itself, declines as before.
    pane.handle_key_event(KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE));
    assert_eq!(
        exec_decisions(&mut rx),
        vec![CommandExecutionApprovalDecision::Cancel]
    );
}

/// A double or held Enter cannot grant: the review opens on "Back", so the
/// second Enter goes back, and the approval then needs a fresh choice.
#[test]
fn pf_25_s01_double_enter_grants_nothing() {
    let (mut view, mut rx) = overlay(Some(offer()), true);
    key(&mut view, KeyCode::Char('g'));
    for _ in 0..4 {
        key(&mut view, KeyCode::Enter);
    }
    assert!(view.grant_review.is_none());
    assert!(!view.is_complete());
    assert_eq!(exec_decisions(&mut rx), Vec::new());
}

/// With Core's real offer: "Grant and run" records the choice on the offer
/// and approves the command; u records "until it expires".
#[test]
fn pf_25_s01_grant_records_the_choice_and_approves() {
    use crate::legacy_core::security_grant::GrantUses;
    use crate::legacy_core::security_grant::open_test_offer;
    for (toggle, uses) in [(false, GrantUses::Once), (true, GrantUses::UntilExpiry)] {
        let thread = ThreadId::new();
        let offer = open_test_offer(
            thread,
            "call-1",
            vec!["cat".to_string(), "../home/team-notes.txt".to_string()],
        );
        let (mut view, mut rx) = overlay_for(
            request_in(thread),
            /*offered*/ None,
            /*security_levels*/ true,
        );
        key(&mut view, KeyCode::Char('g'));
        key(&mut view, KeyCode::Enter);
        if toggle {
            key(&mut view, KeyCode::Char('u'));
        }
        render(&view, 100);
        key(&mut view, KeyCode::Down);
        key(&mut view, KeyCode::Enter);
        assert_eq!(offer.confirmed_uses(), Some(uses));
        assert!(view.is_complete());
        let mut granted = false;
        let mut decisions = Vec::new();
        while let Ok(event) = rx.try_recv() {
            match event {
                AppEvent::InsertHistoryCell(cell) => {
                    granted |= cell
                        .display_lines(200)
                        .iter()
                        .any(|line| line.to_string().contains("You granted"));
                }
                AppEvent::SubmitThreadOp {
                    op: Op::ExecApproval { decision, .. },
                    ..
                } => decisions.push(decision),
                _ => {}
            }
        }
        assert!(granted);
        assert_eq!(decisions, vec![CommandExecutionApprovalDecision::Accept]);
    }
}

/// A review cut off by a short pane cannot grant.
#[test]
fn pf_25_s01_clipped_review_cannot_grant() {
    let (mut view, mut rx) = overlay(Some(offer()), true);
    key(&mut view, KeyCode::Char('g'));
    key(&mut view, KeyCode::Enter);
    let mut buf = Buffer::empty(Rect::new(0, 0, 100, 8));
    view.render(Rect::new(0, 0, 100, 8), &mut buf);
    key(&mut view, KeyCode::Down);
    key(&mut view, KeyCode::Enter);
    assert!(view.grant_review.is_some());
    assert_eq!(exec_decisions(&mut rx), Vec::new());
    assert!(render(&view, 100).contains("does not fit on screen"));
}

/// The request answered elsewhere while the review is open closes it;
/// Ctrl+C in the review cancels the request.
#[test]
fn pf_25_s01_review_follows_the_request() {
    let (mut view, mut rx) = overlay(Some(offer()), true);
    key(&mut view, KeyCode::Char('g'));
    key(&mut view, KeyCode::Enter);
    assert!(
        view.dismiss_app_server_request(&ResolvedAppServerRequest::ExecApproval {
            id: "call-1".to_string(),
        })
    );
    assert!(view.is_complete());
    assert_eq!(exec_decisions(&mut rx), Vec::new());

    let (mut view, mut rx) = overlay(Some(offer()), true);
    key(&mut view, KeyCode::Char('g'));
    key(&mut view, KeyCode::Enter);
    assert_eq!(view.on_ctrl_c(), CancellationEvent::Handled);
    assert_eq!(
        exec_decisions(&mut rx),
        vec![CommandExecutionApprovalDecision::Cancel]
    );
}

/// No option when the offer is for another command, or the approval cannot
/// be accepted.
#[test]
fn pf_25_s01_no_option_for_another_command_or_without_accept() {
    let mut other = offer();
    other.command = vec!["cat".to_string(), "../home/other.txt".to_string()];
    let (view, _rx) = overlay(Some(other), true);
    assert!(
        !view
            .options
            .iter()
            .any(|option| matches!(option.decision, ApprovalDecision::SecurityGrant))
    );

    let ApprovalRequest::Exec(mut no_accept) = request() else {
        unreachable!()
    };
    no_accept.available_decisions = vec![CommandExecutionApprovalDecision::Cancel];
    let (view, _rx) = overlay_for(ApprovalRequest::Exec(no_accept), Some(offer()), true);
    assert!(
        !view
            .options
            .iter()
            .any(|option| matches!(option.decision, ApprovalDecision::SecurityGrant))
    );
}

/// A command re-split from one string still matches Core's copy.
#[test]
fn pf_25_s01_same_command_accepts_the_joined_form() {
    let command = vec![
        "/bin/zsh".to_string(),
        "-lc".to_string(),
        "cat a b".to_string(),
    ];
    assert!(same_command(&command, &command));
    assert!(same_command(
        &command,
        &["/bin/zsh -lc 'cat a b'".to_string()]
    ));
    assert!(!same_command(&command, &["cat".to_string()]));
}

/// Model-chosen text cannot add rows or reorder the review: the folder and
/// command are shown escaped.
#[test]
fn pf_25_s01_review_escapes_model_text() {
    let mut offered = offer();
    offered.cwd = "/work\nFolder      /safe\u{202e}".to_string();
    let (mut view, _rx) = overlay(Some(offered), true);
    key(&mut view, KeyCode::Char('g'));
    key(&mut view, KeyCode::Enter);
    let rendered = render(&view, 100);
    assert!(rendered.contains("/work\\nFolder"), "{rendered}");
    assert!(rendered.contains("\\u{202e}"), "{rendered}");
    assert!(
        !rendered
            .lines()
            .any(|line| line.trim_start().starts_with("Folder      /safe")),
        "{rendered}"
    );
}
