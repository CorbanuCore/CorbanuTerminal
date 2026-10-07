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
    ApprovalRequest::Exec(ExecApprovalRequest {
        thread_id: thread(),
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
    TEST_OFFER.with(|slot| *slot.borrow_mut() = offered);
    let (tx, rx) = unbounded_channel::<AppEvent>();
    let keymap = crate::keymap::RuntimeKeymap::defaults();
    let view = ApprovalOverlay::new(
        request(),
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
    key(&mut view, KeyCode::Char('g'));
    key(&mut view, KeyCode::Enter);
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
    for code in [KeyCode::Char('y'), KeyCode::Char('n'), KeyCode::Down] {
        key(&mut view, code);
    }
    assert!(view.grant_review.is_some());
    assert_eq!(exec_decisions(&mut rx), Vec::new());
}
