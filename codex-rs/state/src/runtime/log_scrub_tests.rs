use std::path::Path;
use std::time::Duration;

use codex_utils_absolute_path::test_support::PathExt;
use pretty_assertions::assert_eq;
use sqlx::SqlitePool;

use super::DONE;
use super::ROWS_SCRUBBED_AND_VACUUMED;
use crate::LogQuery;
use crate::StateRuntime;
use crate::migrations::LOGS_MIGRATOR;
use crate::runtime::test_support::unique_temp_dir;

const KEY: &str = "fake-scrub-provider-key-0001";
const COOKIE: &str = "fake-scrub-cookie-0002";
const QUERY: &str = "fake-scrub-query-0003";
const ENV: &str = "fake-scrub-env-0004";
const DELETED: &str = "fake-scrub-deleted-row-0005";
const SECRETS: [&str; 5] = [KEY, COOKIE, QUERY, ENV, DELETED];

async fn open_pool(home: &Path) -> SqlitePool {
    let sqlite = crate::SqliteConfig::new_for_testing(home.abs());
    sqlite
        .open_read_write_pool(&sqlite.logs_db_path())
        .await
        .expect("open logs db")
}

async fn insert(pool: &SqlitePool, body: &[u8]) {
    sqlx::query(
        "INSERT INTO logs (ts, ts_nanos, level, target, feedback_log_body, estimated_bytes) VALUES (?, 0, 'DEBUG', 't', CAST(? AS TEXT), ?)",
    )
    .bind(chrono::Utc::now().timestamp())
    .bind(body)
    .bind(body.len() as i64)
    .execute(pool)
    .await
    .expect("insert log row");
}

/// The logs DB files under `home` that contain any of `secrets`.
fn files_containing(home: &Path, secrets: &[&str]) -> Vec<String> {
    let mut found = Vec::new();
    for entry in std::fs::read_dir(home).expect("read home") {
        let path = entry.expect("dir entry").path();
        if !path.to_string_lossy().contains("logs") {
            continue;
        }
        let bytes = std::fs::read(&path).expect("read db file");
        for secret in secrets {
            if bytes
                .windows(secret.len())
                .any(|window| window == secret.as_bytes())
            {
                found.push(format!("{secret} in {}", path.display()));
            }
        }
    }
    found
}

async fn stage(runtime: &StateRuntime) -> i64 {
    sqlx::query_scalar::<_, i64>("PRAGMA user_version")
        .fetch_one(runtime.logs_pool.as_ref())
        .await
        .expect("read user_version")
}

async fn wait_for_stage(runtime: &StateRuntime, expected: i64) {
    for _ in 0..300 {
        if stage(runtime).await >= expected {
            return;
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
    panic!("log scrub did not reach stage {expected}");
}

/// A logs database written by an older build: leaked rows are redacted,
/// deleted rows leave nothing in the files, and the pass runs once.
#[tokio::test]
async fn startup_scrubs_leaked_secrets_from_existing_logs_db_once() {
    let home = unique_temp_dir();
    std::fs::create_dir_all(&home).expect("create home");
    let pool = open_pool(&home).await;
    LOGS_MIGRATOR.run(&pool).await.expect("apply logs schema");
    let leaked = [
        format!(
            "Configuring session: ModelProviderInfo {{ experimental_bearer_token: Some(\"{KEY}\"), http_headers: Some({{\"X-Custom\": \"{KEY}\"}}), query_params: Some({{\"sig\": \"{QUERY}\"}}) }}"
        ),
        format!(
            "Request completed headers={{\"content-type\": \"text/event-stream\", \"set-cookie\": \"session={COOKIE}; Path=/\"}}"
        ),
        format!("Request failed url=https://api.example.com/v1/models?key={QUERY}"),
        format!(
            "spawn_child_async: \"/bin/sh\" {{\"HOME\": \"/home/me\", \"SEED_WORDS\": \"{ENV}\"}}"
        ),
    ];
    for body in &leaked {
        insert(&pool, body.as_bytes()).await;
    }
    let clean = "Request completed status=200 url=https://api.example.com/v1/responses";
    insert(&pool, clean.as_bytes()).await;
    // Not valid UTF-8: must not stop the pass.
    insert(&pool, b"bytes \xff\xfe Bearer fake-scrub-binary-0006 end").await;
    for _ in 0..200 {
        insert(
            &pool,
            format!("Bearer {DELETED} {}", "x".repeat(400)).as_bytes(),
        )
        .await;
    }
    sqlx::query("DELETE FROM logs WHERE feedback_log_body LIKE 'Bearer %'")
        .execute(&pool)
        .await
        .expect("delete rows");
    pool.close().await;
    assert_eq!(
        files_containing(&home, &SECRETS).len(),
        SECRETS.len(),
        "every secret is in the fixture before the scrub"
    );

    let runtime = StateRuntime::init(
        crate::SqliteConfig::new_for_testing(home.abs()),
        "test-provider".to_string(),
    )
    .await
    .expect("initialize runtime");
    wait_for_stage(&runtime, DONE).await;
    let messages = runtime
        .query_logs(&LogQuery::default())
        .await
        .expect("query logs")
        .into_iter()
        .filter_map(|row| row.message)
        .collect::<Vec<_>>();
    assert_eq!(
        messages,
        vec![
            "Configuring session: ModelProviderInfo { experimental_bearer_token: Some(\"REDACTED\"), http_headers: Some({\"X-Custom\": \"REDACTED\"}), query_params: Some({\"sig\": \"REDACTED\"}) }".to_string(),
            "Request completed headers={\"content-type\": \"text/event-stream\", \"set-cookie\": \"REDACTED\"}".to_string(),
            "Request failed url=https://api.example.com/v1/models?key=REDACTED".to_string(),
            "spawn_child_async: \"/bin/sh\" {\"HOME\": \"REDACTED\", \"SEED_WORDS\": \"REDACTED\"}".to_string(),
            clean.to_string(),
            "bytes \u{fffd}\u{fffd} Bearer REDACTED end".to_string(),
        ]
    );
    // Checked while the runtime still holds the database open, so the
    // scrub's own checkpoint (not a close) emptied the WAL.
    assert_eq!(files_containing(&home, &SECRETS), Vec::<String>::new());

    // Once done, later starts leave rows alone (new builds do not leak).
    let later = "?key=fake-scrub-after-marker-0007";
    insert(runtime.logs_pool.as_ref(), later.as_bytes()).await;
    runtime
        .scrub_logged_secrets_once()
        .await
        .expect("second pass");
    let last = runtime
        .query_logs(&LogQuery::default())
        .await
        .expect("query logs")
        .pop()
        .and_then(|row| row.message);
    assert_eq!(last.as_deref(), Some(later));
    let _ = std::fs::remove_dir_all(home);
}

/// A reader holding an old snapshot keeps the WAL from being truncated: the
/// pass stops after the rewrite and finishes on a later start.
#[tokio::test]
async fn busy_checkpoint_is_finished_on_a_later_start() {
    let home = unique_temp_dir();
    std::fs::create_dir_all(&home).expect("create home");
    let pool = open_pool(&home).await;
    LOGS_MIGRATOR.run(&pool).await.expect("apply logs schema");
    insert(&pool, format!("?key={QUERY}").as_bytes()).await;
    let mut reader = pool.acquire().await.expect("reader connection");
    sqlx::query("BEGIN")
        .execute(&mut *reader)
        .await
        .expect("begin");
    sqlx::query("SELECT COUNT(*) FROM logs")
        .execute(&mut *reader)
        .await
        .expect("read");

    let runtime = StateRuntime::init(
        crate::SqliteConfig::new_for_testing(home.abs()),
        "test-provider".to_string(),
    )
    .await
    .expect("initialize runtime");
    wait_for_stage(&runtime, ROWS_SCRUBBED_AND_VACUUMED).await;
    // Let the background checkpoint give up.
    tokio::time::sleep(Duration::from_secs(7)).await;
    assert_eq!(stage(&runtime).await, ROWS_SCRUBBED_AND_VACUUMED);

    sqlx::query("ROLLBACK")
        .execute(&mut *reader)
        .await
        .expect("end read");
    drop(reader);
    pool.close().await;
    runtime
        .scrub_logged_secrets_once()
        .await
        .expect("later start");
    assert_eq!(stage(&runtime).await, DONE);
    assert_eq!(files_containing(&home, &[QUERY]), Vec::<String>::new());
    let _ = std::fs::remove_dir_all(home);
}
