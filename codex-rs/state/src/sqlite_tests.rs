use codex_utils_absolute_path::test_support::PathExt;
use pretty_assertions::assert_eq;
use std::time::Duration;
use std::time::Instant;

use super::DB_INIT_LOCK_FILENAME;
use super::SHM_RESET_RETRIES;
use super::database_has_content;
use super::open_quick_pool;
use super::retry_shm_reset_race;

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

/// #865: on a loaded machine (Windows CI runners) opening a connection can
/// take longer than the short-wait pool's wait. The pool still opens, with
/// all its connections, and keeps them: a write then waits only for a free
/// one, never for an open. It used to fail to open (and the process used the
/// main pool), or a write failed as busy while a second connection opened.
#[tokio::test]
async fn a_slow_connection_open_keeps_the_short_wait_pool() -> anyhow::Result<()> {
    let sqlite_home = crate::runtime::test_support::unique_temp_dir();
    tokio::fs::create_dir_all(&sqlite_home).await?;
    let _cleanup = scopeguard::guard(sqlite_home.clone(), |sqlite_home| {
        let _ = std::fs::remove_dir_all(sqlite_home);
    });
    let sqlite = crate::SqliteConfig::new_for_testing(sqlite_home.as_path().abs());
    let path = sqlite.state_db_path();
    let writer = sqlite.open_read_write_pool(&path).await?;
    let short_wait = Duration::from_millis(100);
    let options = sqlx::sqlite::SqliteConnectOptions::new()
        .filename(&path)
        .create_if_missing(false)
        .busy_timeout(short_wait);
    let slow_opens = sqlx::sqlite::SqlitePoolOptions::new().after_connect(move |_, _| {
        Box::pin(async move {
            tokio::time::sleep(short_wait * 3).await;
            Ok(())
        })
    });
    let pool = open_quick_pool(options, slow_opens, short_wait).await?;
    let first = pool.acquire().await?;
    let second = pool.acquire().await?;
    drop((first, second));
    assert_eq!(
        (
            pool.options().get_idle_timeout(),
            pool.options().get_max_lifetime()
        ),
        (None, None)
    );
    pool.close().await;
    writer.close().await;
    Ok(())
}

/// A short-wait pool on a database that can't be opened is an error (the
/// runtime then uses the main pool), though the pool opens its own
/// connections in the background.
#[tokio::test]
async fn the_short_wait_pool_reports_a_database_it_cannot_open() {
    let sqlite_home = crate::runtime::test_support::unique_temp_dir();
    let options = sqlx::sqlite::SqliteConnectOptions::new()
        .filename(sqlite_home.join("missing.sqlite"))
        .create_if_missing(false);
    let opened = open_quick_pool(
        options,
        sqlx::sqlite::SqlitePoolOptions::new(),
        Duration::from_millis(100),
    )
    .await;
    assert!(opened.is_err(), "opened a missing database");
}

/// Set for the child processes of
/// [`concurrent_first_launches_on_a_new_home_all_get_a_state_db`].
const RACE_HOME_ENV: &str = "CODEX_STATE_TEST_RACE_HOME";
const RACE_START_ENV: &str = "CODEX_STATE_TEST_RACE_START_MS";

/// One launch of the concurrent-launch test; does nothing unless that test
/// started this process.
#[test]
fn concurrent_launch_child() {
    let (Some(home), Some(start)) = (
        std::env::var_os(RACE_HOME_ENV),
        std::env::var(RACE_START_ENV).ok(),
    ) else {
        return;
    };
    let start =
        std::time::UNIX_EPOCH + Duration::from_millis(start.parse().expect("start time in ms"));
    if let Ok(wait) = start.duration_since(std::time::SystemTime::now()) {
        std::thread::sleep(wait);
    }
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("tokio runtime");
    let started = Instant::now();
    runtime.block_on(async {
        let sqlite = crate::SqliteConfig::new_for_testing(std::path::Path::new(&home).abs());
        crate::StateRuntime::init(sqlite, "test-provider".to_string())
            .await
            .map(drop)
            .unwrap_or_else(|err| panic!("state runtime failed to open: {err:#}"));
    });
    println!("opened the state DB in {:?}", started.elapsed());
}

/// #549: processes started at the same moment on a brand-new home race to
/// create and migrate its databases; every one must still get them.
#[test]
fn concurrent_first_launches_on_a_new_home_all_get_a_state_db() {
    const LAUNCHES: usize = 10;
    const ROUNDS: usize = 2;
    let exe = std::env::current_exe().expect("test binary");
    // `sqlite::tests::concurrent_launch_child`, whatever this module is called.
    let child_test = format!(
        "{}::concurrent_launch_child",
        module_path!()
            .split_once("::")
            .map_or(module_path!(), |(_, path)| path)
    );
    for round in 0..ROUNDS {
        let home = crate::runtime::test_support::unique_temp_dir();
        std::fs::create_dir_all(&home).expect("create home");
        let _cleanup = scopeguard::guard(home.clone(), |home| {
            let _ = std::fs::remove_dir_all(home);
        });
        let start = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("clock")
            + Duration::from_millis(1_500);
        // Every launch is started before any is waited for.
        let mut children = Vec::with_capacity(LAUNCHES);
        for index in 0..LAUNCHES {
            children.push(
                std::process::Command::new(&exe)
                    .args(["--exact", &child_test, "--test-threads=1", "--nocapture"])
                    .env(RACE_HOME_ENV, &home)
                    .env(RACE_START_ENV, start.as_millis().to_string())
                    // Files, not pipes: a child that fails loudly never blocks.
                    .stdout(std::fs::File::create(home.join(format!("out-{index}"))).expect("out"))
                    .stderr(std::fs::File::create(home.join(format!("err-{index}"))).expect("err"))
                    .spawn()
                    .expect("spawn launch"),
            );
        }
        // Launches open the databases one at a time (the init lock), so all
        // of them take as long as that many in a row: on a loaded runner, more
        // than a fixed total allows (#865). A launch that stops the others -
        // none finishes for `STALL` - fails readably here rather than at the
        // runner's own timeout.
        const STALL: Duration = Duration::from_secs(20);
        let mut statuses = vec![None; LAUNCHES];
        let mut progressed = Instant::now();
        while statuses.iter().any(Option::is_none) && progressed.elapsed() < STALL {
            for (child, status) in children.iter_mut().zip(statuses.iter_mut()) {
                if status.is_none()
                    && let Some(exited) = child.try_wait().expect("poll launch")
                {
                    *status = Some(exited);
                    progressed = Instant::now();
                }
            }
            std::thread::sleep(Duration::from_millis(20));
        }
        let failures: Vec<String> = children
            .into_iter()
            .zip(statuses)
            .enumerate()
            .filter_map(|(index, (mut child, status))| {
                // A launch that exited after the last poll keeps its status.
                let status = status.or_else(|| {
                    let _ = child.kill();
                    child.wait().ok()
                });
                let stdout =
                    std::fs::read_to_string(home.join(format!("out-{index}"))).unwrap_or_default();
                let stderr =
                    std::fs::read_to_string(home.join(format!("err-{index}"))).unwrap_or_default();
                let passed =
                    status.is_some_and(|status| status.success()) && stdout.contains("1 passed");
                (!passed).then(|| format!("{status:?}\n{stdout}\n{stderr}"))
            })
            .collect();
        assert!(
            failures.is_empty(),
            "round {round}: {} of {LAUNCHES} launches had no state DB:\n{}",
            failures.len(),
            failures.join("\n---\n")
        );
    }
}

/// #549: while another process holds the init lock, a runtime DB is neither
/// created nor migrated; it is once the lock is released.
#[tokio::test]
async fn runtime_databases_wait_for_the_init_lock() -> anyhow::Result<()> {
    let sqlite_home = crate::runtime::test_support::unique_temp_dir();
    tokio::fs::create_dir_all(&sqlite_home).await?;
    let _cleanup = scopeguard::guard(sqlite_home.clone(), |sqlite_home| {
        let _ = std::fs::remove_dir_all(sqlite_home);
    });
    let sqlite = crate::SqliteConfig::new_for_testing(sqlite_home.as_path().abs());
    // Another process's hold: a separate open of the lock file.
    let holder = std::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(false)
        .open(sqlite_home.join(DB_INIT_LOCK_FILENAME))?;
    holder.lock()?;

    let opening = tokio::spawn({
        let sqlite = sqlite.clone();
        async move {
            let migrator = crate::migrations::runtime_goals_migrator();
            sqlite
                .open_goals_db(
                    &migrator, /*telemetry_override*/ None, /*init_lock*/ None,
                )
                .await
        }
    });
    tokio::time::sleep(Duration::from_secs(1)).await;
    assert!(!opening.is_finished(), "opened while the lock was held");
    assert!(!database_has_content(&sqlite.goals_db_path()));

    holder.unlock()?;
    let pool = tokio::time::timeout(Duration::from_secs(10), opening).await???;
    let tables: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM sqlite_master WHERE type = 'table' AND name = '_sqlx_migrations'",
    )
    .fetch_one(&pool)
    .await?;
    assert_eq!(tables, 1);
    pool.close().await;
    Ok(())
}

/// #586: once a launch holds the init lock, the file is not empty. On Linux an
/// empty file at that path can be a sandbox placeholder, which the command
/// run's cleanup removes even while it is locked.
#[tokio::test]
async fn the_init_lock_file_is_not_left_empty() -> anyhow::Result<()> {
    let sqlite_home = crate::runtime::test_support::unique_temp_dir();
    tokio::fs::create_dir_all(&sqlite_home).await?;
    let _cleanup = scopeguard::guard(sqlite_home.clone(), |sqlite_home| {
        let _ = std::fs::remove_dir_all(sqlite_home);
    });
    let lock = sqlite_home.join(DB_INIT_LOCK_FILENAME);
    std::fs::write(&lock, b"")?;
    let sqlite = crate::SqliteConfig::new_for_testing(sqlite_home.as_path().abs());
    let held = sqlite.lock_runtime_db_init().await;
    assert_eq!(std::fs::metadata(&lock)?.len(), 1);
    drop(held);
    // Later launches leave it as it is.
    let held = sqlite.lock_runtime_db_init().await;
    assert_eq!(std::fs::metadata(&lock)?.len(), 1);
    drop(held);
    Ok(())
}

/// #586: a Linux sandbox placeholder (an empty `0444` file) left at the lock
/// path does not turn the lock off: it is made writable and becomes the lock.
#[cfg(unix)]
#[tokio::test]
async fn a_leftover_read_only_placeholder_becomes_the_lock() -> anyhow::Result<()> {
    use std::os::unix::fs::PermissionsExt;
    let sqlite_home = crate::runtime::test_support::unique_temp_dir();
    tokio::fs::create_dir_all(&sqlite_home).await?;
    let _cleanup = scopeguard::guard(sqlite_home.clone(), |sqlite_home| {
        let _ = std::fs::remove_dir_all(sqlite_home);
    });
    let lock = sqlite_home.join(DB_INIT_LOCK_FILENAME);
    std::fs::write(&lock, b"")?;
    std::fs::set_permissions(&lock, std::fs::Permissions::from_mode(0o444))?;
    if std::fs::OpenOptions::new().write(true).open(&lock).is_ok() {
        // Root ignores the mode: nothing to repair, nothing to prove.
        eprintln!("skipped: the read-only placeholder is writable for this user");
        return Ok(());
    }
    let sqlite = crate::SqliteConfig::new_for_testing(sqlite_home.as_path().abs());
    let held = sqlite.lock_runtime_db_init().await;
    assert!(held._file.is_some(), "the placeholder is locked");
    assert_eq!(std::fs::metadata(&lock)?.len(), 1);
    // Only the owner's write bit is added.
    assert_eq!(
        std::fs::metadata(&lock)?.permissions().mode() & 0o777,
        0o644
    );
    // Another holder now waits for it.
    let other = std::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .open(&lock)?;
    assert!(other.try_lock().is_err());
    drop(held);
    assert!(other.try_lock().is_ok());
    Ok(())
}

/// #756: dropping a launch's init lock releases it at once, also while a
/// child process forked meanwhile (by any spawn in this process) still holds
/// a copy of the lock's descriptor because it has not started its program
/// yet. Closing the file alone released the lock only when that copy closed:
/// on macOS a concurrent spawn made `a_leftover_read_only_placeholder_becomes_the_lock`
/// find the lock still held after its holder was dropped.
#[cfg(unix)]
#[tokio::test]
async fn the_init_lock_is_released_while_a_forked_child_holds_its_descriptor() -> anyhow::Result<()>
{
    let sqlite_home = crate::runtime::test_support::unique_temp_dir();
    tokio::fs::create_dir_all(&sqlite_home).await?;
    let _cleanup = scopeguard::guard(sqlite_home.clone(), |sqlite_home| {
        let _ = std::fs::remove_dir_all(sqlite_home);
    });
    let sqlite = crate::SqliteConfig::new_for_testing(sqlite_home.as_path().abs());
    let held = sqlite.lock_runtime_db_init().await;
    assert!(held._file.is_some(), "the lock is held");

    // A child forked now has a copy of every descriptor, the lock's too, until
    // it starts its program; this one keeps its copy open until `release`.
    let (release, spawner) = crate::runtime::test_support::fork_child_holding_descriptors()?;

    let other = std::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .open(sqlite_home.join(DB_INIT_LOCK_FILENAME))?;
    assert!(other.try_lock().is_err(), "the lock is held");
    drop(held);
    assert!(
        other.try_lock().is_ok(),
        "the dropped lock is still held through the forked child's copy"
    );
    drop(release);
    let status = spawner.join().expect("spawner thread")?;
    assert!(status.success(), "{status:?}");
    Ok(())
}

/// #586: a link at the lock path is never repaired through: its target keeps
/// its mode, and the launch opens the databases unlocked.
#[cfg(unix)]
#[tokio::test]
async fn a_link_at_the_lock_path_is_not_repaired() -> anyhow::Result<()> {
    use std::os::unix::fs::PermissionsExt;
    let sqlite_home = crate::runtime::test_support::unique_temp_dir();
    tokio::fs::create_dir_all(&sqlite_home).await?;
    let _cleanup = scopeguard::guard(sqlite_home.clone(), |sqlite_home| {
        let _ = std::fs::remove_dir_all(sqlite_home);
    });
    let target = sqlite_home.join("target");
    std::fs::write(&target, b"")?;
    std::fs::set_permissions(&target, std::fs::Permissions::from_mode(0o444))?;
    if std::fs::OpenOptions::new()
        .write(true)
        .open(&target)
        .is_ok()
    {
        eprintln!("skipped: the read-only target is writable for this user");
        return Ok(());
    }
    std::os::unix::fs::symlink(&target, sqlite_home.join(DB_INIT_LOCK_FILENAME))?;
    let sqlite = crate::SqliteConfig::new_for_testing(sqlite_home.as_path().abs());
    let held = sqlite.lock_runtime_db_init().await;
    assert!(held._file.is_none());
    assert_eq!(
        std::fs::metadata(&target)?.permissions().mode() & 0o777,
        0o444
    );
    assert_eq!(std::fs::metadata(&target)?.len(), 0);
    Ok(())
}

/// A database error with a given extended result code.
#[derive(Debug)]
struct CodedError(&'static str);

impl std::fmt::Display for CodedError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "(code: {}) test error", self.0)
    }
}

impl std::error::Error for CodedError {}

impl sqlx::error::DatabaseError for CodedError {
    fn message(&self) -> &str {
        "test error"
    }

    fn code(&self) -> Option<std::borrow::Cow<'_, str>> {
        Some(self.0.into())
    }

    fn as_error(&self) -> &(dyn std::error::Error + Send + Sync + 'static) {
        self
    }

    fn as_error_mut(&mut self) -> &mut (dyn std::error::Error + Send + Sync + 'static) {
        self
    }

    fn into_error(self: Box<Self>) -> Box<dyn std::error::Error + Send + Sync + 'static> {
        self
    }

    fn kind(&self) -> sqlx::error::ErrorKind {
        sqlx::error::ErrorKind::Other
    }
}

/// Opens with `retry_shm_reset_race`, failing the first `failures` attempts
/// with `code`; returns the result and how many attempts were made.
async fn open_failing(code: &'static str, failures: u32) -> (Result<(), sqlx::Error>, u32) {
    let attempts = std::cell::Cell::new(0);
    let result = retry_shm_reset_race(std::path::Path::new("test.sqlite"), || {
        attempts.set(attempts.get() + 1);
        let attempt = attempts.get();
        async move {
            if attempt <= failures {
                Err(sqlx::Error::Database(Box::new(CodedError(code))))
            } else {
                Ok(())
            }
        }
    })
    .await;
    (result, attempts.get())
}

/// #691: on Windows a launch that resets a WAL database's `-shm` file while
/// an exiting process still maps it gets SQLITE_IOERR_TRUNCATE (1546); the
/// open is retried, and only that error is.
#[tokio::test]
async fn an_open_retries_only_the_shm_reset_race() {
    let (result, attempts) = open_failing("1546", /*failures*/ 2).await;
    assert!(result.is_ok(), "{result:?}");
    assert_eq!(attempts, 3);

    // Any other error (here SQLITE_IOERR_SHMOPEN) is returned at once.
    let (result, attempts) = open_failing("4618", /*failures*/ 1).await;
    assert!(result.is_err());
    assert_eq!(attempts, 1);

    // A truncation that keeps failing is reported after the retries.
    let (result, attempts) = open_failing("1546", u32::MAX).await;
    let code = result
        .expect_err("still failing")
        .as_database_error()
        .and_then(sqlx::error::DatabaseError::code)
        .map(std::borrow::Cow::into_owned);
    assert_eq!(code.as_deref(), Some("1546"));
    assert_eq!(attempts, SHM_RESET_RETRIES + 1);
}
