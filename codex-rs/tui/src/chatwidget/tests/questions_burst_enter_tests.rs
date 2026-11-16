//! Exercise D642 at the question delivery boundary, including delayed event replay.
use super::*;
use codex_context_fragments::AnsweredQuestion;
use codex_context_fragments::ContextualUserFragment;
use pretty_assertions::assert_eq;

const FIRST_LINE: &str = "a sufficiently long answer";

fn type_keys(chat: &mut ChatWidget, text: &str) {
    for ch in text.chars() {
        chat.handle_key_event(KeyEvent::from(KeyCode::Char(ch)));
    }
}

fn open_questions(chat: &mut ChatWidget) {
    chat.thread_id = Some(ThreadId::new());
    chat.add_async_questions("message", &questions());
    chat.handle_key_event(KeyEvent::new(KeyCode::Up, KeyModifiers::ALT));
}

async fn held_enter_event(rx: &mut tokio::sync::mpsc::UnboundedReceiver<AppEvent>) -> u64 {
    loop {
        if let AppEvent::AsyncQuestionBurstEnter { token } =
            tokio::time::timeout(Duration::from_secs(1), rx.recv())
                .await
                .expect("question timer deadline")
                .expect("question timer event")
        {
            return token;
        }
    }
}

fn assert_first_answer(op: Op, text: &str) {
    let Op::UserTurn { items, .. } = op else {
        panic!("expected user turn");
    };
    let id = serde_json::json!(["request_user_input_async", "message", 0]).to_string();
    assert_eq!(
        items,
        vec![UserInput::Text {
            text: AnsweredQuestion::new(&id, "Which way?", text)
                .render()
                .expect_text(),
            text_elements: Vec::new(),
        }]
    );
}

#[tokio::test]
async fn quiet_enter_replays_once_through_question_key_delivery() {
    let (mut chat, mut rx, mut ops) = make_chatwidget_manual(/*model_override*/ None).await;
    tokio::time::pause();
    open_questions(&mut chat);
    type_keys(&mut chat, FIRST_LINE);
    chat.handle_key_event(KeyEvent::from(KeyCode::Enter));
    assert!(ops.try_recv().is_err());
    assert_eq!(question_count(&chat), 2);

    let token = held_enter_event(&mut rx).await;
    // Even a tick after the timer expires cannot produce an answer.
    chat.handle_paste_burst_tick(FrameRequester::test_dummy());
    assert!(ops.try_recv().is_err());
    assert!(
        chat.bottom_pane
            .questions
            .as_ref()
            .unwrap()
            .submission
            .is_none()
    );
    chat.handle_async_question_burst_enter(token);
    assert_first_answer(ops.try_recv().unwrap(), FIRST_LINE);
    assert_eq!(question_count(&chat), 1);
    chat.handle_async_question_burst_enter(token);
    assert!(ops.try_recv().is_err());
    assert_eq!(question_count(&chat), 1);
}

#[tokio::test]
async fn following_paste_keeps_held_enter_as_a_newline() {
    for bracketed in [false, true] {
        let (mut chat, mut rx, mut ops) = make_chatwidget_manual(/*model_override*/ None).await;
        tokio::time::pause();
        open_questions(&mut chat);
        type_keys(&mut chat, FIRST_LINE);
        chat.handle_key_event(KeyEvent::from(KeyCode::Enter));
        assert!(ops.try_recv().is_err());
        tokio::time::advance(Duration::from_millis(8)).await;
        if bracketed {
            chat.handle_paste("second line".into());
        } else {
            type_keys(&mut chat, "second line");
        }
        let token = held_enter_event(&mut rx).await;
        chat.handle_async_question_burst_enter(token);
        assert!(ops.try_recv().is_err());
        assert_eq!(question_count(&chat), 2);
        tokio::time::advance(Duration::from_millis(200)).await;
        chat.handle_paste_burst_tick(FrameRequester::test_dummy());
        chat.handle_key_event(KeyEvent::from(KeyCode::Enter));
        assert_first_answer(
            ops.try_recv().unwrap(),
            &format!("{FIRST_LINE}\nsecond line"),
        );
        assert!(ops.try_recv().is_err());
        tokio::time::resume();
    }
}

#[tokio::test]
async fn question_transitions_cancel_delayed_enter_even_after_returning() {
    for transition in [
        "navigate",
        "collapse",
        "resolve",
        "modal",
        "modal_closed",
        "clear",
    ] {
        let (mut chat, mut rx, mut ops) = make_chatwidget_manual(/*model_override*/ None).await;
        tokio::time::pause();
        open_questions(&mut chat);
        type_keys(&mut chat, FIRST_LINE);
        chat.handle_key_event(KeyEvent::from(KeyCode::Enter));
        assert!(ops.try_recv().is_err());
        let token = held_enter_event(&mut rx).await;
        match transition {
            "navigate" => {
                let questions = chat.bottom_pane.questions.as_mut().unwrap();
                assert!(questions.navigate(/*forward*/ true));
                assert!(questions.navigate(/*forward*/ false));
            }
            "collapse" => {
                let questions = chat.bottom_pane.questions.as_mut().unwrap();
                questions.set_expanded(/*expanded*/ false);
                questions.set_expanded(/*expanded*/ true);
            }
            "resolve" => {
                chat.bottom_pane
                    .questions
                    .as_mut()
                    .unwrap()
                    .resolve_answers(&[serde_json::json!([
                        "request_user_input_async",
                        "message",
                        0
                    ])
                    .to_string()]);
            }
            "modal" | "modal_closed" => {
                chat.bottom_pane
                    .show_selection_view(crate::bottom_pane::SelectionViewParams {
                        view_id: Some("burst-test"),
                        ..Default::default()
                    });
                if transition == "modal" {
                    chat.handle_async_question_burst_enter(token);
                    assert!(ops.try_recv().is_err());
                }
                assert!(chat.bottom_pane.dismiss_active_view_if_id("burst-test"));
            }
            "clear" => chat.bottom_pane.clear_pending_questions(),
            _ => unreachable!(),
        }
        let count = question_count(&chat);
        chat.handle_paste_burst_tick(FrameRequester::test_dummy());
        chat.handle_async_question_burst_enter(token);
        assert!(ops.try_recv().is_err(), "{transition}");
        assert_eq!(question_count(&chat), count, "{transition}");
        assert!(
            chat.bottom_pane
                .questions
                .as_ref()
                .unwrap()
                .submission
                .is_none()
        );
        tokio::time::resume();
    }
}

#[tokio::test]
async fn delayed_enter_preserves_answer_when_delivery_is_restricted() {
    let (mut chat, mut rx, mut ops) = make_chatwidget_manual(/*model_override*/ None).await;
    tokio::time::pause();
    open_questions(&mut chat);
    type_keys(&mut chat, FIRST_LINE);
    chat.handle_key_event(KeyEvent::from(KeyCode::Enter));
    let token = held_enter_event(&mut rx).await;
    chat.blocks_direct_input = true;
    chat.handle_async_question_burst_enter(token);
    assert!(ops.try_recv().is_err());
    assert_eq!(question_count(&chat), 2);
    chat.blocks_direct_input = false;
    chat.handle_key_event(KeyEvent::from(KeyCode::Enter));
    assert_first_answer(ops.try_recv().unwrap(), FIRST_LINE);
    assert!(ops.try_recv().is_err());
}
