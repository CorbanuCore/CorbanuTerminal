use std::sync::Arc;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;
use std::time::Duration;

use futures::StreamExt;
use pretty_assertions::assert_eq;
use teloxide::Bot;
use teloxide::update_listeners::AsUpdateStream;
use tokio::io::AsyncReadExt;
use tokio::io::AsyncWriteExt;

use super::guarded_polling;

// A finite script, a bounded request buffer and a timeout keep broken pollers
// from turning a regression test into an unbounded server or allocation.
async fn scripted_bot(
    replies: Vec<serde_json::Value>,
) -> (Bot, Arc<AtomicUsize>, tokio::task::JoinHandle<()>) {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let calls = Arc::new(AtomicUsize::new(0));
    let observed = Arc::clone(&calls);
    let server = tokio::spawn(async move {
        let mut replies = replies.into_iter();
        loop {
            let (mut socket, _) = listener.accept().await.unwrap();
            let mut request = Vec::new();
            let mut buffer = [0; 1024];
            loop {
                let count = socket.read(&mut buffer).await.unwrap();
                assert!(count > 0);
                request.extend_from_slice(&buffer[..count]);
                assert!(request.len() < 16 * 1024);
                if let Some(end) = request.windows(4).position(|part| part == b"\r\n\r\n") {
                    let headers = String::from_utf8_lossy(&request[..end]).to_lowercase();
                    let length: usize = headers
                        .lines()
                        .find_map(|line| line.strip_prefix("content-length:"))
                        .map(|value| value.trim().parse().unwrap())
                        .unwrap_or(0);
                    if request.len() >= end + 4 + length {
                        break;
                    }
                }
            }
            let request = String::from_utf8_lossy(&request).to_lowercase();
            let mut reply = if request.contains("/getwebhookinfo ") {
                serde_json::json!({"ok": true, "result": {
                    "url": "", "has_custom_certificate": false, "pending_update_count": 0
                }})
            } else {
                assert!(request.contains("/getupdates "));
                observed.fetch_add(1, Ordering::SeqCst);
                let Some(reply) = replies.next() else {
                    break;
                };
                reply
            };
            if let Some(expected) = reply.as_object_mut().unwrap().remove("_expected_request") {
                let (_, body) = request.split_once("\r\n\r\n").unwrap();
                let actual: serde_json::Value = serde_json::from_str(body).unwrap();
                for (key, value) in expected.as_object().unwrap() {
                    assert_eq!(&actual[key], value);
                }
            }
            let body = reply.to_string();
            let response = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
                body.len()
            );
            socket.write_all(response.as_bytes()).await.unwrap();
        }
    });
    let bot = Bot::new("123:test").set_api_url(format!("http://{address}").parse().unwrap());
    (bot, calls, server)
}

async fn stop_server(server: tokio::task::JoinHandle<()>) {
    server.abort();
    if let Err(error) = server.await {
        assert!(error.is_cancelled(), "scripted server failed: {error}");
    }
}

#[tokio::test]
async fn replay_does_not_advance_offset_and_shutdown_acknowledges_live_batch() {
    let update = |id| {
        serde_json::json!({
            "update_id": id,
            "message": {"message_id": 1, "date": 1,
                "chat": {"id": 99, "type": "private"}, "text": "hello"}
        })
    };
    let (bot, calls, server) = scripted_bot(vec![
        serde_json::json!({
            "ok": true, "result": [update(41), update(42)],
            "_expected_request": {"offset": 0, "timeout": 10, "allowed_updates": ["message"]}
        }),
        serde_json::json!({
            "ok": true, "result": [],
            "_expected_request": {"offset": 43, "timeout": 0, "limit": 1}
        }),
    ])
    .await;
    let replay = serde_json::from_value(update(9000)).unwrap();
    let mut polling = guarded_polling(bot, /*max_consecutive_failures*/ 3, vec![replay]).await;
    teloxide::update_listeners::UpdateListener::hint_allowed_updates(
        &mut polling,
        &mut [teloxide::types::AllowedUpdate::Message].into_iter(),
    );
    let stop = polling.stop_token();
    let outcome = tokio::time::timeout(Duration::from_secs(5), async {
        let mut stream = polling.as_stream();
        assert_eq!(stream.next().await.unwrap().unwrap().id.0, 9000);
        assert_eq!(calls.load(Ordering::SeqCst), 0);
        assert_eq!(stream.next().await.unwrap().unwrap().id.0, 41);
        // Stopping still drains the fetched batch before acknowledging it.
        stop.stop();
        assert_eq!(stream.next().await.unwrap().unwrap().id.0, 42);
        assert!(stream.next().await.is_none());
    })
    .await;
    stop_server(server).await;
    outcome.unwrap();
    assert_eq!(calls.load(Ordering::SeqCst), 2);
}

fn failure() -> serde_json::Value {
    serde_json::json!({"ok": false, "error_code": 500, "description": "Internal Server Error"})
}

#[tokio::test]
async fn consecutive_failures_report_cap_and_stop() {
    let (bot, calls, server) = scripted_bot(vec![failure(), failure(), failure(), empty()]).await;
    let mut polling = guarded_polling(bot, /*max_consecutive_failures*/ 3, vec![]).await;
    let fatal = polling.fatal_flag();
    let outcome = tokio::time::timeout(Duration::from_secs(15), async {
        let mut stream = polling.as_stream();
        for _ in 0..3 {
            assert!(stream.next().await.unwrap().is_err());
        }
        assert!(stream.next().await.is_none());
    })
    .await;
    stop_server(server).await;
    outcome.unwrap();
    assert!(matches!(
        fatal.error(),
        Some(crate::error::TelegramError::PollingFailureCapExceeded)
    ));
    // Three failures followed by the bounded shutdown acknowledgement.
    assert_eq!(calls.load(Ordering::SeqCst), 4);
}

#[tokio::test]
async fn rate_limits_still_count_as_consecutive_failures() {
    let limited = serde_json::json!({
        "ok": false, "error_code": 429, "description": "Too Many Requests",
        "parameters": {"retry_after": 1}
    });
    let (bot, calls, server) = scripted_bot(vec![limited.clone(), limited, empty()]).await;
    let mut polling = guarded_polling(bot, /*max_consecutive_failures*/ 2, vec![]).await;
    let fatal = polling.fatal_flag();
    let outcome = tokio::time::timeout(Duration::from_secs(5), async {
        let mut stream = polling.as_stream();
        assert!(stream.next().await.unwrap().is_err());
        assert!(stream.next().await.unwrap().is_err());
        assert!(stream.next().await.is_none());
    })
    .await;
    stop_server(server).await;
    outcome.unwrap();
    assert!(matches!(
        fatal.error(),
        Some(crate::error::TelegramError::PollingFailureCapExceeded)
    ));
    assert_eq!(calls.load(Ordering::SeqCst), 3);
}

#[tokio::test]
async fn conflict_reports_conflict_without_waiting_for_cap() {
    let conflict = serde_json::json!({
        "ok": false, "error_code": 409,
        "description": "Conflict: terminated by other getUpdates request; make sure that only one bot instance is running"
    });
    let (bot, calls, server) = scripted_bot(vec![conflict, empty()]).await;
    let mut polling = guarded_polling(bot, /*max_consecutive_failures*/ 3, vec![]).await;
    let fatal = polling.fatal_flag();
    let outcome = tokio::time::timeout(Duration::from_secs(5), async {
        let mut stream = polling.as_stream();
        assert!(stream.next().await.unwrap().is_err());
        assert!(stream.next().await.is_none());
    })
    .await;
    stop_server(server).await;
    outcome.unwrap();
    assert!(matches!(
        fatal.error(),
        Some(crate::error::TelegramError::PollingConflict)
    ));
    assert_eq!(calls.load(Ordering::SeqCst), 2);
}

#[tokio::test]
async fn shutdown_conflict_does_not_replace_failure_cap() {
    let conflict = serde_json::json!({
        "ok": false, "error_code": 409,
        "description": "Conflict: terminated by other getUpdates request; make sure that only one bot instance is running"
    });
    for cap in [1, 2] {
        let (bot, _, server) = scripted_bot(vec![failure(), conflict.clone()]).await;
        let mut polling = guarded_polling(bot, cap, vec![]).await;
        let fatal = polling.fatal_flag();
        let stop = polling.stop_token();
        let outcome = tokio::time::timeout(Duration::from_secs(5), async {
            let mut stream = polling.as_stream();
            assert!(stream.next().await.unwrap().is_err());
            stop.stop();
            assert!(stream.next().await.is_none());
        })
        .await;
        stop_server(server).await;
        outcome.unwrap();
        match cap {
            1 => assert!(matches!(
                fatal.error(),
                Some(crate::error::TelegramError::PollingFailureCapExceeded)
            )),
            2 => assert!(matches!(
                fatal.error(),
                Some(crate::error::TelegramError::PollingConflict)
            )),
            _ => unreachable!(),
        }
    }
}

fn empty() -> serde_json::Value {
    serde_json::json!({"ok": true, "result": []})
}

#[tokio::test]
async fn empty_successful_polls_reset_consecutive_failure_cap() {
    let (bot, calls, server) = scripted_bot(vec![
        failure(),
        empty(),
        failure(),
        empty(),
        failure(),
        serde_json::json!({"ok": true, "result": [{
            "update_id": 42,
            "message": {"message_id": 1, "date": 1,
                "chat": {"id": 99, "type": "private"}, "text": "recovered"}
        }]}),
    ])
    .await;
    let mut polling = guarded_polling(bot, /*max_consecutive_failures*/ 3, vec![]).await;
    let fatal = polling.fatal_flag();
    let outcome = tokio::time::timeout(Duration::from_secs(15), async {
        let mut stream = polling.as_stream();
        for _ in 0..4 {
            match stream.next().await {
                Some(Ok(update)) => return Some(update.id.0),
                Some(Err(_)) => {}
                None => return None,
            }
        }
        None
    })
    .await;
    stop_server(server).await;
    assert_eq!(outcome.unwrap(), Some(42));
    assert!(!fatal.is_fatal());
    assert_eq!(calls.load(Ordering::SeqCst), 6);
}
