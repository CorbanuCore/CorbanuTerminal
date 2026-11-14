use std::sync::Arc;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;

use pretty_assertions::assert_eq;
use teloxide::types::ChatId;
use teloxide::types::Update;

use super::*;
use crate::dedup::INBOX_CAPACITY;

fn update(id: u32) -> Update {
    serde_json::from_value(serde_json::json!({
        "update_id": id,
        "message": {"message_id": id, "date": 1,
            "chat": {"id": 99, "type": "private"}, "text": "/help"}
    }))
    .unwrap()
}

fn test_home() -> PathBuf {
    static NEXT: AtomicUsize = AtomicUsize::new(0);
    std::env::temp_dir().join(format!(
        "astra-2139-{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ))
}

// One bounded loopback request; no Telegram credentials or live service.
async fn notice_bot(fail_notice: bool) -> (Bot, tokio::task::JoinHandle<serde_json::Value>) {
    use tokio::io::AsyncReadExt;
    use tokio::io::AsyncWriteExt;

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let server = tokio::spawn(async move {
        tokio::time::timeout(std::time::Duration::from_secs(5), async move {
            let (mut socket, _) = listener.accept().await.unwrap();
            let mut request = Vec::new();
            let mut buffer = [0; 1024];
            let payload = loop {
                let count = socket.read(&mut buffer).await.unwrap();
                assert!(count > 0);
                request.extend_from_slice(&buffer[..count]);
                assert!(request.len() < 16 * 1024);
                if let Some(end) = request.windows(4).position(|part| part == b"\r\n\r\n") {
                    let headers = String::from_utf8_lossy(&request[..end]).to_lowercase();
                    assert!(headers.contains("/sendmessage "));
                    let length: usize = headers.lines()
                        .find_map(|line| line.strip_prefix("content-length:"))
                        .unwrap().trim().parse().unwrap();
                    if request.len() >= end + 4 + length {
                        break serde_json::from_slice::<serde_json::Value>(
                            &request[end + 4..end + 4 + length]
                        ).unwrap();
                    }
                }
            };
            let body = if fail_notice {
                serde_json::json!({"ok": false, "error_code": 429,
                    "description": "Too Many Requests", "parameters": {"retry_after": 1}})
            } else {
                serde_json::json!({"ok": true, "result": {
                    "message_id": 1, "date": 1, "chat": {"id": 99, "type": "private"}, "text": "notice"
                }})
            }.to_string();
            let response = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
                body.len()
            );
            socket.write_all(response.as_bytes()).await.unwrap();
            payload
        }).await.unwrap()
    });
    (
        Bot::new("123:test").set_api_url(format!("http://{address}").parse().unwrap()),
        server,
    )
}

async fn finish_failure(inbox: &UpdateDeduplicator, id: u64) {
    let (bot, notice) = notice_bot(/*fail_notice*/ false).await;
    finish_update(
        &bot,
        inbox,
        ConversationKey::from(ChatId(99)),
        id,
        &Err(anyhow::anyhow!("handler failed after an action")),
    )
    .await;
    let payload = notice.await.unwrap();
    assert_eq!(payload["chat_id"], 99);
    let text = payload["text"].as_str().unwrap();
    assert!(text.contains(&format!("update {id} failed")));
    assert!(text.contains("resend your message or command"));
    assert!(text.contains("Some actions may already have completed"));
    assert!(!text.contains("handler failed after an action"));
}

#[tokio::test]
async fn completed_failures_do_not_fill_pending_inbox() {
    let home = test_home();
    let inbox = UpdateDeduplicator::load(&home, /*bot_id*/ 123).await;
    // Finite reproduction beyond the entire pending capacity (256).
    for id in 0..INBOX_CAPACITY as u32 + 44 {
        assert_eq!(inbox.begin_update(&update(id)).await, BeginUpdate::Accepted);
        finish_failure(&inbox, u64::from(id)).await;
    }
    assert!(inbox.pending_updates().await.is_empty());
    let restarted = UpdateDeduplicator::load(&home, /*bot_id*/ 123).await;
    assert!(restarted.pending_updates().await.is_empty());
    assert_eq!(
        restarted.begin_update(&update(500)).await,
        BeginUpdate::Accepted
    );
    restarted.complete_update(/*update_id*/ 500).await.unwrap();
    assert!(restarted.pending_updates().await.is_empty());
    std::fs::remove_dir_all(home).unwrap();
}

#[tokio::test]
async fn completed_failure_does_not_repeat_mutation_after_redelivery_or_restart() {
    let home = test_home();
    let mutations = AtomicUsize::new(0);
    let inbox = UpdateDeduplicator::load(&home, /*bot_id*/ 123).await;
    for _ in 0..2 {
        if inbox.begin_update(&update(42)).await == BeginUpdate::Accepted {
            let (bot, mutation) = notice_bot(/*fail_notice*/ false).await;
            send_notice(
                &bot,
                ConversationKey::from(ChatId(99)),
                "original mutating send",
            )
            .await
            .unwrap();
            assert_eq!(mutation.await.unwrap()["text"], "original mutating send");
            mutations.fetch_add(1, Ordering::SeqCst);
            // A later handler step fails after the mutating send succeeded.
            finish_failure(&inbox, /*id*/ 42).await;
        }
    }
    let restarted = UpdateDeduplicator::load(&home, /*bot_id*/ 123).await;
    if restarted.begin_update(&update(42)).await == BeginUpdate::Accepted {
        mutations.fetch_add(1, Ordering::SeqCst);
    }
    assert_eq!(mutations.load(Ordering::SeqCst), 1);
    let persisted: serde_json::Value =
        serde_json::from_slice(&std::fs::read(home.join("telegram/updates-123.json")).unwrap())
            .unwrap();
    assert_eq!(
        persisted,
        serde_json::json!({
            "completed_update_ids": [], "failed_update_ids": [42], "pending": []
        })
    );
    std::fs::remove_dir_all(home).unwrap();
}

#[tokio::test]
async fn failed_notice_is_not_retried_and_keeps_terminal_failure() {
    let home = test_home();
    let inbox = UpdateDeduplicator::load(&home, /*bot_id*/ 123).await;
    assert_eq!(inbox.begin_update(&update(42)).await, BeginUpdate::Accepted);
    let (bot, notice) = notice_bot(/*fail_notice*/ true).await;
    let conversation = ConversationKey::new(
        ChatId(99),
        Some(teloxide::types::ThreadId(teloxide::types::MessageId(12))),
    );
    tokio::time::timeout(
        std::time::Duration::from_secs(2),
        finish_update(
            &bot,
            &inbox,
            conversation,
            /*update_id*/ 42,
            &Err(anyhow::anyhow!("failed")),
        ),
    )
    .await
    .unwrap();
    let payload = notice.await.unwrap();
    assert_eq!(payload["message_thread_id"], 12);
    let restarted = UpdateDeduplicator::load(&home, /*bot_id*/ 123).await;
    assert!(restarted.pending_updates().await.is_empty());
    assert_eq!(
        restarted.begin_update(&update(42)).await,
        BeginUpdate::Duplicate
    );
    std::fs::remove_dir_all(home).unwrap();
}

#[tokio::test]
async fn queued_receipt_is_retained_until_known_resolution() {
    let home = test_home();
    let inbox = Arc::new(UpdateDeduplicator::load(&home, /*bot_id*/ 123).await);
    let conversation = ConversationKey::from(ChatId(99));
    let (bot, notice) = notice_bot(/*fail_notice*/ false).await;
    let (tx, rx) = tokio::sync::oneshot::channel();
    assert_eq!(inbox.begin_update(&update(42)).await, BeginUpdate::Accepted);
    let monitor = monitor_queued_update(
        bot,
        Arc::clone(&inbox),
        conversation,
        /*update_id*/ 42,
        rx,
    );
    assert_eq!(inbox.pending_updates().await, vec![update(42)]);
    let restarted = UpdateDeduplicator::load(&home, /*bot_id*/ 123).await;
    assert_eq!(restarted.pending_updates().await, vec![update(42)]);
    tx.send(Err("known failure".to_string())).unwrap();
    monitor.await.unwrap();
    notice.await.unwrap();
    let restarted = UpdateDeduplicator::load(&home, /*bot_id*/ 123).await;
    assert!(restarted.pending_updates().await.is_empty());
    assert_eq!(
        restarted.begin_update(&update(42)).await,
        BeginUpdate::Duplicate
    );

    let (tx, rx) = tokio::sync::oneshot::channel();
    assert_eq!(inbox.begin_update(&update(43)).await, BeginUpdate::Accepted);
    let monitor = monitor_queued_update(
        Bot::new("123:test"),
        Arc::clone(&inbox),
        conversation,
        /*update_id*/ 43,
        rx,
    );
    drop(tx);
    monitor.await.unwrap();
    let restarted = UpdateDeduplicator::load(&home, /*bot_id*/ 123).await;
    assert_eq!(restarted.pending_updates().await, vec![update(43)]);
    assert_eq!(
        restarted.begin_update(&update(43)).await,
        BeginUpdate::Accepted
    );

    let (tx, rx) = tokio::sync::oneshot::channel();
    let monitor = monitor_queued_update(
        Bot::new("123:test"),
        Arc::clone(&inbox),
        conversation,
        /*update_id*/ 43,
        rx,
    );
    tx.send(Ok(())).unwrap();
    monitor.await.unwrap();
    let restarted = UpdateDeduplicator::load(&home, /*bot_id*/ 123).await;
    assert!(restarted.pending_updates().await.is_empty());
    assert_eq!(
        restarted.begin_update(&update(43)).await,
        BeginUpdate::Duplicate
    );
    std::fs::remove_dir_all(home).unwrap();
}

#[tokio::test]
async fn crash_mid_handler_retains_raw_update_for_restart() {
    let home = test_home();
    let inbox = Arc::new(UpdateDeduplicator::load(&home, /*bot_id*/ 123).await);
    let (started_tx, started_rx) = tokio::sync::oneshot::channel();
    let handler_inbox = Arc::clone(&inbox);
    let handler = tokio::spawn(async move {
        assert_eq!(
            handler_inbox.begin_update(&update(42)).await,
            BeginUpdate::Accepted
        );
        started_tx.send(()).unwrap();
        std::future::pending::<()>().await;
        handler_inbox
            .complete_update(/*update_id*/ 42)
            .await
            .unwrap();
    });
    started_rx.await.unwrap();
    handler.abort();
    assert!(handler.await.unwrap_err().is_cancelled());
    drop(inbox);
    let restarted = UpdateDeduplicator::load(&home, /*bot_id*/ 123).await;
    assert_eq!(restarted.pending_updates().await, vec![update(42)]);
    assert_eq!(
        restarted.begin_update(&update(42)).await,
        BeginUpdate::Accepted
    );
    restarted.complete_update(/*update_id*/ 42).await.unwrap();
    let recovered = UpdateDeduplicator::load(&home, /*bot_id*/ 123).await;
    assert!(recovered.pending_updates().await.is_empty());
    assert_eq!(
        recovered.begin_update(&update(42)).await,
        BeginUpdate::Duplicate
    );
    std::fs::remove_dir_all(home).unwrap();
}
