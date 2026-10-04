use codex_utils_absolute_path::test_support::PathExt;
use pretty_assertions::assert_eq;
use std::time::Duration;
use std::time::Instant;

#[tokio::test]
async fn connecting_to_an_existing_database_does_not_wait_for_its_writer() -> anyhow::Result<()> {
    let sqlite_home = crate::runtime::test_support::unique_temp_dir();
    tokio::fs::create_dir_all(&sqlite_home).await?;
    let _cleanup = scopeguard::guard(sqlite_home.clone(), |sqlite_home| {
        let _ = std::fs::remove_dir_all(sqlite_home);
    });
    let sqlite = crate::SqliteConfig::new_for_testing(sqlite_home.as_path().abs());
    let path = sqlite.state_db_path();
    // A database this call creates still gets incremental auto-vacuum.
    let writer = sqlite.open_read_write_pool(&path).await?;
    sqlx::query("CREATE TABLE t (v INTEGER)")
        .execute(&writer)
        .await?;
    assert_eq!(
        sqlx::query_scalar::<_, i64>("PRAGMA auto_vacuum")
            .fetch_one(&writer)
            .await?,
        2
    );

    // Another process holds the write lock for longer than the busy timeout.
    let mut held = writer.begin_with("BEGIN IMMEDIATE").await?;
    sqlx::query("INSERT INTO t VALUES (1)")
        .execute(&mut *held)
        .await?;

    // A new connection - a reader here - must not need that lock to open.
    let started = Instant::now();
    let reader = sqlite.open_read_write_pool(&path).await?;
    let rows: i64 = sqlx::query_scalar("SELECT count(*) FROM t")
        .fetch_one(&reader)
        .await?;
    assert_eq!(rows, 0);
    assert!(
        started.elapsed() < Duration::from_secs(2),
        "connecting waited {:?} for another writer",
        started.elapsed()
    );
    assert_eq!(
        sqlx::query_scalar::<_, i64>("PRAGMA auto_vacuum")
            .fetch_one(&reader)
            .await?,
        2
    );
    held.rollback().await?;
    reader.close().await;
    writer.close().await;
    Ok(())
}
