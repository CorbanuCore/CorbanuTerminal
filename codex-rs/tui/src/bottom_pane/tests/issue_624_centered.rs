//! #624 under centered views: a request that arrives while the person has a
//! centered dialog open waits under it, and the dialog's first row says so,
//! as the notice above an inline view does. The dialog keeps the keys.

use super::*;
use pretty_assertions::assert_eq;

const WAITING: &str = "A request is waiting; it shows when you close this.";

fn centered_picker() -> SelectionViewParams {
    SelectionViewParams {
        presentation: ViewPresentation::Centered,
        ..panes_picker()
    }
}

/// How the chat surface draws a centered dialog: its backdrop, then the
/// dialog itself (which the owned transcript also paints on its own).
fn render_overlay(pane: &BottomPane, area: Rect) -> String {
    let overlay = view_stack::DialogOverlay {
        backdrop: pane.backdrop_with_options(ComposerRenderOptions::default()),
        dialog: pane.centered_dialog().expect("a centered dialog"),
    };
    render_snapshot(&overlay, area)
}

#[test]
fn issue_624_centered_dialog_over_a_waiting_request_says_so() {
    let (tx_raw, mut rx) = unbounded_channel::<AppEvent>();
    let features = Features::with_defaults();
    let mut pane = test_pane(AppEventSender::new(tx_raw));
    pane.show_selection_view(centered_picker());
    let area = Rect::new(0, 0, 80, 20);
    assert_eq!(pane.centered_dialog().unwrap().notice, None);
    assert!(!render_overlay(&pane, area).contains("A request is waiting"));

    pane.push_approval_request(exec_request(), &features);
    assert_eq!(pane.view_stack.len(), 2);
    assert!(pane.view_stack[0].answers_request());
    let dialog = pane.centered_dialog().expect("the picker stays on top");
    assert_eq!(dialog.notice, Some(WAITING));
    for screen in [render_overlay(&pane, area), render_snapshot(&pane, area)] {
        assert!(screen.contains(WAITING), "{screen}");
        assert!(screen.contains("Corbanu Terminal - B"), "{screen}");
        assert_eq!(
            screen.matches("A request is waiting").count(),
            1,
            "{screen}"
        );
    }
    // The notice takes the dialog's first row; the picker keeps the rest
    // and its own height.
    let without_notice = CenteredView::new(dialog.view);
    assert_eq!(
        dialog.desired_height(/*width*/ 80),
        without_notice.desired_height(/*width*/ 80) + 1
    );
    assert_eq!(
        dialog.cursor_pos(area),
        without_notice.cursor_pos(area).map(|(x, y)| (x, y + 1))
    );

    // A small terminal still shows the notice's start above the picker.
    let small = Rect::new(0, 0, 30, 6);
    let screen = render_overlay(&pane, small);
    assert!(screen.contains("! A request"), "{screen}");

    // The picker keeps the keys: Down, Down, Enter choose its third row.
    for key in [KeyCode::Down, KeyCode::Down, KeyCode::Enter] {
        pane.handle_key_event(KeyEvent::new(key, KeyModifiers::NONE));
    }
    assert!(
        std::iter::from_fn(|| rx.try_recv().ok())
            .any(|event| matches!(event, AppEvent::OpenPanePicker))
    );
    // With the picker gone the approval shows, and nothing says it waits.
    assert_eq!(pane.view_stack.len(), 1);
    assert!(pane.centered_dialog().is_none());
    let area = Rect::new(0, 0, 80, pane.desired_height(/*width*/ 80));
    assert!(!render_snapshot(&pane, area).contains("A request is waiting"));
}
