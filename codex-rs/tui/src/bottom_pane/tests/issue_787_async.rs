//! #787 for async questions: the expanded question panel takes keys when no
//! view is open, so input reaching it before the question on show was drawn,
//! or in the second after (an arrival, the next question after an answer, a
//! reply from another client, expanding the panel, or the panel uncovered by
//! a closing view) is dropped like input reaching a request view; its footer
//! says so. Ctrl+C and deliberate answers keep working.

use super::*;
use codex_protocol::items::AsyncUserInputQuestion;
use pretty_assertions::assert_eq;

const EARLY: &str = "Keys pressed as this question appeared were ignored.";

fn guarded_pane(tx: AppEventSender) -> BottomPane {
    let mut pane = test_pane(tx);
    pane.set_request_input_guard(REQUEST_INPUT_GUARD);
    pane
}

fn question(title: &str, options: Option<&[&str]>) -> AsyncUserInputQuestion {
    AsyncUserInputQuestion {
        title: title.to_string(),
        options: options.map(|options| options.iter().map(ToString::to_string).collect()),
    }
}

fn question_id(message_id: &str, index: usize) -> String {
    serde_json::json!(["request_user_input_async", message_id, index]).to_string()
}

/// A pane showing these questions expanded, not yet drawn.
fn pane_with_questions(questions: &[AsyncUserInputQuestion]) -> BottomPane {
    let (tx_raw, _rx) = unbounded_channel::<AppEvent>();
    let mut pane = guarded_pane(AppEventSender::new(tx_raw));
    pane.push_async_questions("message", questions);
    pane.questions
        .as_mut()
        .unwrap()
        .set_expanded(/*expanded*/ true);
    pane
}

/// As the TUI draws: the pre-draw tick, then the frame (which also records
/// the options fully on screen, needed before a choice can answer).
fn draw_at(pane: &mut BottomPane, at: Instant) -> String {
    pane.pre_draw_tick_at(at);
    let area = Rect::new(0, 0, 80, pane.desired_height(/*width*/ 80));
    render_snapshot(pane, area)
}

fn press_at(pane: &mut BottomPane, keys: &[KeyEvent], at: Instant) {
    for key in keys {
        pane.handle_key_event_at(*key, at);
    }
}

fn plain(code: KeyCode) -> KeyEvent {
    KeyEvent::new(code, KeyModifiers::NONE)
}

fn text(text: &str) -> Vec<KeyEvent> {
    text.chars().map(|ch| plain(KeyCode::Char(ch))).collect()
}

/// The answer the pane submitted, if any; as `ChatWidget` does once it is
/// delivered, the answered question then leaves.
fn take_answer(pane: &mut BottomPane) -> Option<String> {
    let questions = pane.questions.as_mut().unwrap();
    let answer = match questions.submission.take()? {
        QuestionSubmission::Submit(text) | QuestionSubmission::Queue(text) => text,
    };
    questions.accept_answer();
    Some(answer)
}

fn draft(pane: &BottomPane) -> String {
    pane.questions.as_ref().unwrap().composer.current_text()
}

fn unanswered(pane: &BottomPane) -> usize {
    pane.questions.as_ref().unwrap().unanswered_count()
}

/// Enter, a number key, the skip key, and Esc (which would interrupt the
/// turn) reaching a question before it was drawn or in the second after are
/// dropped, and each dropped key restarts the second; afterwards a deliberate
/// Enter answers.
#[test]
fn issue_787_async_question_takes_no_key_before_it_is_drawn_or_in_the_second_after() {
    let (tx_raw, mut rx) = unbounded_channel::<AppEvent>();
    let mut pane = guarded_pane(AppEventSender::new(tx_raw));
    pane.push_async_questions("message", &[question("Deploy?", Some(&["Yes", "No"]))]);
    pane.questions
        .as_mut()
        .unwrap()
        .set_expanded(/*expanded*/ true);
    let start = Instant::now();
    let skip = KeyEvent::new(KeyCode::Char(']'), KeyModifiers::CONTROL);

    press_at(
        &mut pane,
        &[plain(KeyCode::Enter), plain(KeyCode::Char('2')), skip],
        start,
    );
    let screen = draw_at(&mut pane, start);
    assert!(screen.contains(EARLY), "{screen}");
    for step in 1..=3 {
        press_at(
            &mut pane,
            &[
                plain(KeyCode::Char('2')),
                plain(KeyCode::Esc),
                plain(KeyCode::Enter),
            ],
            start + Duration::from_millis(900 * step),
        );
    }
    assert_eq!(take_answer(&mut pane), None);
    assert_eq!(unanswered(&pane), 1);
    assert!(
        !std::iter::from_fn(|| rx.try_recv().ok())
            .any(|event| matches!(event, AppEvent::CodexOp(_))),
        "a dropped key reached the turn"
    );

    let later = start + Duration::from_millis(2700) + REQUEST_INPUT_GUARD;
    draw_at(&mut pane, later);
    press_at(&mut pane, &[plain(KeyCode::Enter)], later);
    let answer = take_answer(&mut pane).expect("a deliberate Enter answers");
    assert!(answer.contains("Yes"), "{answer}");
}

/// Answering a question shows the next one: keys typed on (a double Enter,
/// or a word) neither answer nor edit it, before it is drawn or in the
/// second after; once quiet, typing answers it.
#[test]
fn issue_787_type_ahead_never_answers_or_edits_the_next_async_question() {
    let mut pane = pane_with_questions(&[
        question("Deploy?", Some(&["Yes", "No"])),
        question("Why?", /*options*/ None),
    ]);
    let start = Instant::now();
    draw_at(&mut pane, start);
    let at = start + REQUEST_INPUT_GUARD;
    press_at(&mut pane, &[plain(KeyCode::Char('1'))], at);
    assert!(take_answer(&mut pane).unwrap().contains("Yes"));

    let mut type_ahead = text("oops");
    type_ahead.push(plain(KeyCode::Enter));
    press_at(&mut pane, &type_ahead, at);
    pane.handle_paste_at("pasted".to_string(), at);
    draw_at(&mut pane, at);
    press_at(&mut pane, &type_ahead, at + Duration::from_millis(500));
    assert_eq!(draft(&pane), "");
    assert_eq!(take_answer(&mut pane), None);
    assert_eq!(unanswered(&pane), 1);

    let later = at + Duration::from_millis(500) + REQUEST_INPUT_GUARD;
    pane.handle_paste_at("because".to_string(), later);
    press_at(&mut pane, &[plain(KeyCode::Enter)], later);
    assert!(take_answer(&mut pane).unwrap().contains("because"));
}

/// Another client answering the question on show brings up the next one;
/// a key typed for the first never answers it.
#[test]
fn issue_787_question_answered_elsewhere_hands_no_key_to_the_next() {
    let mut pane = pane_with_questions(&[
        question("First?", Some(&["One", "Two"])),
        question("Second?", Some(&["Three", "Four"])),
    ]);
    let start = Instant::now();
    draw_at(&mut pane, start);
    let at = start + REQUEST_INPUT_GUARD;
    assert!(
        pane.questions
            .as_mut()
            .unwrap()
            .resolve_answers(&[question_id("message", 0)])
    );
    press_at(&mut pane, &[plain(KeyCode::Enter)], at);
    draw_at(&mut pane, at);
    press_at(
        &mut pane,
        &[plain(KeyCode::Char('2'))],
        at + Duration::from_millis(800),
    );
    assert_eq!(take_answer(&mut pane), None);
    assert_eq!(unanswered(&pane), 1);

    let later = at + Duration::from_millis(800) + REQUEST_INPUT_GUARD;
    press_at(&mut pane, &[plain(KeyCode::Char('2'))], later);
    assert!(take_answer(&mut pane).unwrap().contains("Four"));
}

/// The panel uncovered by a closing view, or expanded again, shows anew:
/// the key that closed the view and keys after it never answer; a paste
/// waits too. Ctrl+C still clears the draft at once.
#[test]
fn issue_787_uncovered_or_reexpanded_question_waits_for_a_draw() {
    let mut pane = pane_with_questions(&[question("Deploy?", Some(&["Yes", "No"]))]);
    let start = Instant::now();
    draw_at(&mut pane, start);
    let at = start + REQUEST_INPUT_GUARD;
    pane.show_selection_view(panes_picker());
    draw_at(&mut pane, at);
    press_at(
        &mut pane,
        &[plain(KeyCode::Esc), plain(KeyCode::Enter)],
        at + REQUEST_INPUT_GUARD,
    );
    assert!(pane.view_stack.is_empty());
    assert_eq!(take_answer(&mut pane), None);

    let shown = at + REQUEST_INPUT_GUARD;
    draw_at(&mut pane, shown);
    let quiet = shown + REQUEST_INPUT_GUARD;
    pane.handle_paste_at("typed".to_string(), quiet);
    assert_eq!(draft(&pane), "typed");

    let questions = pane.questions.as_mut().unwrap();
    questions.set_expanded(/*expanded*/ false);
    questions.set_expanded(/*expanded*/ true);
    pane.handle_paste_at(" more".to_string(), quiet);
    press_at(&mut pane, &[plain(KeyCode::Enter)], quiet);
    assert_eq!(draft(&pane), "typed");
    assert_eq!(take_answer(&mut pane), None);
    assert!(matches!(pane.on_ctrl_c(), CancellationEvent::Handled));
    assert_eq!(draft(&pane), "");
}
