use super::*;
use crossterm::event::KeyCode;
use crossterm::event::KeyEvent;
use crossterm::event::KeyModifiers;
use pretty_assertions::assert_eq;
use std::task::Poll;

#[tokio::test]
async fn modal_input_uses_app_receiver_and_returns_unconsumed_keys() {
    let (tx, mut rx) = mpsc::unbounded_channel();
    let mut watchdog = TuiInputDrainWatchdog::new();
    watchdog.started_at = Instant::now() - Duration::from_secs(20);
    let cancel = KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE);
    let composer_key = KeyEvent::new(KeyCode::Char('x'), KeyModifiers::NONE);
    for event in [
        TuiEvent::Paste("saved chat".to_string()),
        TuiEvent::Draw,
        TuiEvent::Resize,
        TuiEvent::Key(cancel),
        TuiEvent::Key(composer_key),
    ] {
        watchdog.note_drained();
        tx.send(event).expect("app receiver remains open");
    }

    {
        let mut modal_events = modal_tui_events(&mut rx, &watchdog);
        assert!(
            matches!(modal_events.next().await, Some(TuiEvent::Paste(text)) if text == "saved chat")
        );
        assert!(matches!(modal_events.next().await, Some(TuiEvent::Draw)));
        assert!(matches!(modal_events.next().await, Some(TuiEvent::Resize)));
        assert!(matches!(modal_events.next().await, Some(TuiEvent::Key(key)) if key == cancel));
        assert_eq!(watchdog.pending_events.load(Ordering::Relaxed), 1);
        assert!(watchdog.last_handled_ms.load(Ordering::Relaxed) >= 20_000);
    }

    assert!(matches!(rx.recv().await, Some(TuiEvent::Key(key)) if key == composer_key));
    watchdog.note_handled();
    assert_eq!(watchdog.pending_events.load(Ordering::Relaxed), 0);

    watchdog.note_drained();
    tx.send(TuiEvent::Key(composer_key))
        .expect("returning from the modal must not close app input");
    assert!(matches!(rx.recv().await, Some(TuiEvent::Key(key)) if key == composer_key));
    watchdog.note_handled();
    assert_eq!(watchdog.pending_events.load(Ordering::Relaxed), 0);
}

#[tokio::test]
async fn modal_input_pending_or_closed_stream_does_not_acknowledge_events() {
    let (tx, mut rx) = mpsc::unbounded_channel();
    let mut watchdog = TuiInputDrainWatchdog::new();
    watchdog.started_at = Instant::now() - Duration::from_secs(20);

    {
        let mut modal_events = modal_tui_events(&mut rx, &watchdog);
        let next_event = modal_events.next();
        tokio::pin!(next_event);
        assert!(matches!(futures::poll!(next_event.as_mut()), Poll::Pending));
        assert_eq!(watchdog.last_handled_ms.load(Ordering::Relaxed), 0);
    }

    // Cancelling an outstanding modal read does not reserve or lose the next app event.
    watchdog.note_drained();
    tx.send(TuiEvent::Draw).expect("app receiver remains open");
    assert!(matches!(rx.recv().await, Some(TuiEvent::Draw)));
    watchdog.note_handled();
    let last_handled = watchdog.last_handled_ms.load(Ordering::Relaxed);
    drop(tx);

    let mut modal_events = modal_tui_events(&mut rx, &watchdog);
    assert!(modal_events.next().await.is_none());
    assert_eq!(watchdog.pending_events.load(Ordering::Relaxed), 0);
    assert_eq!(
        watchdog.last_handled_ms.load(Ordering::Relaxed),
        last_handled
    );
}
