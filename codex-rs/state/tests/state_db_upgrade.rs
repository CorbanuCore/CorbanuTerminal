//! Upstream sync S1c: upstream's state migrations 0047-0062 run as 0054-0069,
//! after Corbanu's 0053, and a state DB written by the latest release (and by
//! `main` before the sync) upgrades without touching an applied migration.
//!
//! Fixtures (`tests/fixtures/state_db_upgrade/`, see `MIGRATIONS.md`): a state DB
//! written by the `rust-v0.1.48` release code (migrations 1-52) and one written
//! by `main` 18fa372a (1-53), each with two threads.

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

fn fixture(name: &str) -> PathBuf {
    let relative = Path::new("tests/fixtures/state_db_upgrade")
        .join(name)
        .join(STATE_DB);
    let manifest = Path::new(env!("CARGO_MANIFEST_DIR")).join(&relative);
    if manifest.is_file() {
        manifest
    } else {
        // Bazel runs integration tests from a Cargo-like crate directory.
        relative
    }
}

type MigrationRow = (i64, String, Vec<u8>, bool);

fn sqlite_config(home: &Path) -> anyhow::Result<SqliteConfig> {
    Ok(SqliteConfig::new_for_testing(
        AbsolutePathBuf::from_absolute_path(home.canonicalize()?)?,
    ))
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
    let state_path = home.path().join(STATE_DB);
    std::fs::copy(fixture(name), &state_path)?;
    let before = applied_migrations(&state_path).await?;
    assert_eq!(
        before.last().map(|row| row.0),
        Some(released_through),
        "{name}: fixture migration level"
    );

    let runtime = StateRuntime::init(sqlite_config(home.path())?, "openai".to_string()).await?;
    runtime.close().await;

    let after = applied_migrations(&state_path).await?;
    // Every migration the older build applied keeps its version, name and checksum.
    assert_eq!(&after[..before.len()], before.as_slice(), "{name}");
    let added: Vec<(i64, String)> = after[before.len()..]
        .iter()
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
    assert_eq!(added, expected, "{name}: migrations added by the upgrade");

    // The conversations survive, and upstream's schema is in place.
    let pool = sqlite_config(home.path())?
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
    StateRuntime::init(sqlite_config(home.path())?, "openai".to_string())
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

#[test]
fn upstream_migrations_are_renumbered_after_0053() -> anyhow::Result<()> {
    let migrations = Path::new(env!("CARGO_MANIFEST_DIR")).join("migrations");
    let migrations = if migrations.is_dir() {
        migrations
    } else {
        PathBuf::from("migrations")
    };
    let mut names: Vec<String> = std::fs::read_dir(&migrations)?
        .map(|entry| entry.map(|entry| entry.file_name().to_string_lossy().into_owned()))
        .collect::<Result<_, _>>()?;
    names.sort();
    // One file per version: no upstream file kept a number Corbanu already uses.
    let versions: Vec<i64> = names
        .iter()
        .map(|name| name[..4].parse::<i64>())
        .collect::<Result<_, _>>()?;
    assert_eq!(versions, (1..=69).collect::<Vec<_>>());
    for (upstream, ours, description) in RENUMBERED {
        let expected = format!("{ours:04}_{}.sql", description.replace(' ', "_"));
        assert!(
            names.contains(&expected),
            "upstream {upstream:04} should be {expected}"
        );
    }
    Ok(())
}
