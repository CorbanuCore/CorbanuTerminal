//! Upstream sync S1c: upstream's state migrations 0047-0062 run as 0054-0069,
//! after Corbanu's 0053, and a state DB written by the latest releases (and by
//! `main` before the sync) upgrades without touching an applied migration.
//!
//! The file layout itself is checked by `migrations_tests.rs`
//! (`upstream_migrations_are_renumbered_after_0053`).
//!
//! Fixtures (`tests/fixtures/state_db_upgrade/`, see `MIGRATIONS.md`): a state DB
//! written by the `rust-v0.1.48` release code (migrations 1-52), one written by
//! `main` 18fa372a (1-53), each with two threads, and one written by the
//! `rust-v0.1.49` release code (1-53 and 0070, #2812) with two threads, a
//! provider reset hint and, in its memories DB, one stage-1 memory.

use std::path::Path;
use std::path::PathBuf;

use codex_state::SqliteConfig;
use codex_state::StateRuntime;
use codex_utils_absolute_path::AbsolutePathBuf;
use pretty_assertions::assert_eq;
use sqlx::Row;

const STATE_DB: &str = "pfterminal_state_5.sqlite";

/// Upstream `rust-v0.167.0` migration, its version here, and its description.
const RENUMBERED: &[(i64, i64, &str)] = &[
    (47, 54, "rollout migration state"),
    (48, 55, "thread section appearance"),
    (49, 56, "projects"),
    (50, 57, "threads section empty preview indexes"),
    (51, 58, "thread artifacts"),
    (52, 59, "projects recency"),
    (53, 60, "threads originator"),
    (54, 61, "threads daybreak enabled"),
    (55, 62, "thread attachments"),
    (56, 63, "threads creator identity"),
    (57, 64, "cleanup guardian thread metadata"),
    (58, 65, "threads archive sort indexes"),
    (59, 66, "thread attachment reverse lookup"),
    (60, 67, "guardian review feedback"),
    (61, 68, "thread read state"),
    (62, 69, "threads attached parent"),
];

const MEMORIES_DB: &str = "pfterminal_memories_1.sqlite";

/// Corbanu's own migration after the renumbered range; `rust-v0.1.49` applied it
/// before any of 0054-0069 existed (#2812).
const PROVIDER_RESET_HINT: (i64, &str) = (70, "provider reset hint");

fn fixture(name: &str) -> PathBuf {
    fixture_file(name, STATE_DB)
}

fn fixture_file(name: &str, file: &str) -> PathBuf {
    let relative = Path::new("tests/fixtures/state_db_upgrade")
        .join(name)
        .join(file);
    let manifest = Path::new(env!("CARGO_MANIFEST_DIR")).join(&relative);
    if manifest.is_file() {
        manifest
    } else {
        // Direct Bazel rust_test targets start at the runfiles workspace root.
        let bazel = Path::new(option_env!("BAZEL_PACKAGE").unwrap_or(".")).join(&relative);
        if bazel.is_file() {
            bazel
        } else {
            // Retain the original helper's crate-relative fallback.
            relative
        }
    }
}

type MigrationRow = (i64, String, Vec<u8>, bool);

fn sqlite_config(home: &Path) -> anyhow::Result<SqliteConfig> {
    Ok(SqliteConfig::new_for_testing(
        AbsolutePathBuf::from_absolute_path(home.canonicalize()?)?,
    ))
}

/// The schema as `(type, name, sql)`, without sqlx's bookkeeping table.
async fn schema(path: &Path) -> anyhow::Result<Vec<(String, String, Option<String>)>> {
    let home = path.parent().ok_or_else(|| anyhow::anyhow!("no parent"))?;
    let pool = sqlite_config(home)?
        .open_read_only_pool(path, /*busy_timeout*/ None)
        .await?;
    let rows = sqlx::query_as(
        "SELECT type, name, sql FROM sqlite_master WHERE name != '_sqlx_migrations' ORDER BY type, name",
    )
    .fetch_all(&pool)
    .await?;
    pool.close().await;
    Ok(rows)
}

async fn applied_migrations(path: &Path) -> anyhow::Result<Vec<MigrationRow>> {
    let home = path.parent().ok_or_else(|| anyhow::anyhow!("no parent"))?;
    let pool = sqlite_config(home)?
        .open_read_only_pool(path, /*busy_timeout*/ None)
        .await?;
    let rows = sqlx::query(
        "SELECT version, description, checksum, success FROM _sqlx_migrations ORDER BY version",
    )
    .fetch_all(&pool)
    .await?
    .into_iter()
    .map(|row| {
        (
            row.get("version"),
            row.get("description"),
            row.get("checksum"),
            row.get("success"),
        )
    })
    .collect();
    pool.close().await;
    Ok(rows)
}

/// Upgrades a copy of `name` with this build's runtime and checks the result.
async fn upgrade(name: &str, released_through: i64) -> anyhow::Result<()> {
    let home = tempfile::tempdir()?;
    std::fs::copy(fixture(name), home.path().join(STATE_DB))?;
    upgrade_in(home.path(), name, released_through).await
}

/// Upgrades the state DB already copied into `home` and checks the result.
async fn upgrade_in(home: &Path, name: &str, released_through: i64) -> anyhow::Result<()> {
    let state_path = home.join(STATE_DB);
    let before = applied_migrations(&state_path).await?;
    assert_eq!(
        before.last().map(|row| row.0),
        Some(released_through),
        "{name}: fixture migration level"
    );

    let runtime = StateRuntime::init(sqlite_config(home)?, "openai".to_string()).await?;
    runtime.close().await;

    let after = applied_migrations(&state_path).await?;
    // Every migration the older build applied keeps its version, name and checksum.
    let kept: Vec<_> = after
        .iter()
        .filter(|row| before.iter().any(|old| old.0 == row.0))
        .cloned()
        .collect();
    assert_eq!(kept, before, "{name}");
    let added: Vec<(i64, String)> = after
        .iter()
        .filter(|row| !before.iter().any(|old| old.0 == row.0))
        .map(|(version, description, _, success)| {
            assert!(success, "{name}: migration {version} failed");
            (*version, description.clone())
        })
        .collect();
    let mut expected = Vec::new();
    if released_through < 53 {
        expected.push((53, "provider request key salt".to_string()));
    }
    expected.extend(
        RENUMBERED
            .iter()
            .map(|(_, ours, description)| (*ours, (*description).to_string())),
    );
    // Corbanu added this migration after the upstream migration range was
    // renumbered; `rust-v0.1.49` already applied it, below which the upgrade
    // then fills in 0054-0069.
    if released_through < PROVIDER_RESET_HINT.0 {
        expected.push((PROVIDER_RESET_HINT.0, PROVIDER_RESET_HINT.1.to_string()));
    }
    assert_eq!(added, expected, "{name}: migrations added by the upgrade");

    // Whatever order the migrations ran in, the schema matches a new database's.
    let fresh = tempfile::tempdir()?;
    StateRuntime::init(sqlite_config(fresh.path())?, "openai".to_string())
        .await?
        .close()
        .await;
    assert_eq!(
        schema(&state_path).await?,
        schema(&fresh.path().join(STATE_DB)).await?,
        "{name}: upgraded schema"
    );

    // The conversations survive, and upstream's schema is in place.
    let pool = sqlite_config(home)?
        .open_read_only_pool(&state_path, /*busy_timeout*/ None)
        .await?;
    let threads: Vec<(String, String, Option<String>, i64)> =
        sqlx::query_as("SELECT id, title, name, archived FROM threads ORDER BY id")
            .fetch_all(&pool)
            .await?;
    assert_eq!(threads.len(), 2, "{name}: threads");
    assert_eq!(threads[0].1, "Released thread");
    assert!(threads[0].2.is_some(), "{name}: the thread name survives");
    assert_eq!((threads[0].3, threads[1].3), (0, 1));
    for column in [
        "project_id",
        "originator",
        "daybreak_enabled",
        "creator_user_id",
        "creator_account_id",
        "attached_thread_parent_id",
    ] {
        let present: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM pragma_table_info('threads') WHERE name = ?")
                .bind(column)
                .fetch_one(&pool)
                .await?;
        assert_eq!(present, 1, "{name}: threads.{column}");
    }
    for table in [
        "rollout_migration_state",
        "projects",
        "thread_attachments",
        "guardian_review_feedback",
        "thread_read_receipts",
    ] {
        let present: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM sqlite_master WHERE type = 'table' AND name = ?",
        )
        .bind(table)
        .fetch_one(&pool)
        .await?;
        assert_eq!(present, 1, "{name}: table {table}");
    }
    pool.close().await;

    // A second start finds nothing to do and accepts the upgraded database.
    StateRuntime::init(sqlite_config(home)?, "openai".to_string())
        .await?
        .close()
        .await;
    assert_eq!(applied_migrations(&state_path).await?, after, "{name}");
    Ok(())
}

#[tokio::test]
async fn release_0_1_48_state_db_upgrades_without_changing_applied_migrations() -> anyhow::Result<()>
{
    upgrade("release-0.1.48", /*released_through*/ 52).await
}

#[tokio::test]
async fn main_state_db_upgrades_without_changing_applied_migrations() -> anyhow::Result<()> {
    upgrade("main-18fa372a", /*released_through*/ 53).await
}

/// #2812: `rust-v0.1.49` applied 1-53 and 0070, so the upgrade adds 0054-0069
/// below an applied migration. Its provider reset hint and memories survive.
#[tokio::test]
async fn release_0_1_49_state_db_upgrades_without_changing_applied_migrations() -> anyhow::Result<()>
{
    const NAME: &str = "release-0.1.49";
    let home = tempfile::tempdir()?;
    let memories_path = home.path().join(MEMORIES_DB);
    std::fs::copy(fixture_file(NAME, STATE_DB), home.path().join(STATE_DB))?;
    std::fs::copy(fixture_file(NAME, MEMORIES_DB), &memories_path)?;
    let memories_before = applied_migrations(&memories_path).await?;
    assert_eq!(
        memories_before.iter().map(|row| row.0).collect::<Vec<_>>(),
        vec![1]
    );
    upgrade_in(
        home.path(),
        NAME,
        /*released_through*/ PROVIDER_RESET_HINT.0,
    )
    .await?;

    let pool = sqlite_config(home.path())?
        .open_read_only_pool(&home.path().join(STATE_DB), /*busy_timeout*/ None)
        .await?;
    let provider_state: Vec<(String, String, String, i64, i64, i64)> = sqlx::query_as(
        "SELECT provider_id, model, key_fingerprint, cooldown_until_ms, provider_reset_at_ms, consecutive_429_count FROM provider_request_state",
    )
    .fetch_all(&pool)
    .await?;
    pool.close().await;
    assert_eq!(
        provider_state,
        vec![(
            "claude-plan".to_string(),
            "claude-opus-5-5-plan".to_string(),
            "config:auth-command".to_string(),
            3_601_000,
            308_375_000,
            1,
        )],
        "the 0070 provider reset hint survives"
    );

    // The memories DB keeps its row and gains only the new migration.
    let memories_after = applied_migrations(&memories_path).await?;
    assert_eq!(memories_after[..1], memories_before[..]);
    assert_eq!(
        memories_after[1..]
            .iter()
            .map(|row| (row.0, row.1.as_str(), row.3))
            .collect::<Vec<_>>(),
        vec![(2, "consolidation progress", true)]
    );
    let pool = sqlite_config(home.path())?
        .open_read_only_pool(&memories_path, /*busy_timeout*/ None)
        .await?;
    let memories: Vec<(String, String, Option<String>)> =
        sqlx::query_as("SELECT thread_id, raw_memory, rollout_slug FROM stage1_outputs")
            .fetch_all(&pool)
            .await?;
    pool.close().await;
    assert_eq!(
        memories,
        vec![(
            "00000000-0000-4000-8000-000000000149".to_string(),
            "Fixture memory: the user prefers concise answers.".to_string(),
            Some("fixture-slug".to_string()),
        )]
    );
    Ok(())
}
