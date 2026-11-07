//! Shared SQLite connection configuration.
//! New databases use DELETE on NFS and WAL elsewhere.
//! Writable pools preserve existing settings unless managed policy requires DELETE on NFS.
//! Startup retries SQLITE_BUSY and reports other errors to callers.
//! The configured logs directory is created and locked only when opening its database.

#![expect(
    clippy::disallowed_methods,
    reason = "this is the centralized SQLite connection shim"
)]

use crate::DbTelemetry;
use crate::migrations::repair_legacy_recency_migration_version;
<<<<<<< HEAD
use crate::migrations::runtime_goals_migrator;
use crate::migrations::runtime_logs_migrator;
use crate::migrations::runtime_memories_migrator;
use crate::migrations::runtime_state_migrator;
use crate::migrations::runtime_thread_history_migrator;
use crate::runtime::RuntimeDbInitError;
use crate::runtime::validate_applied_migrations;
||||||| 413492cd6c
use crate::runtime::RuntimeDbInitError;
=======
use crate::runtime::recovery::RuntimeDbInitError;
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a
use crate::telemetry;
use crate::telemetry::DbKind;
use codex_file_system::LocalLogsDir;
use codex_utils_absolute_path::AbsolutePathBuf;
use log::LevelFilter;
use sqlx::ConnectOptions;
use sqlx::Connection;
use sqlx::Error;
use sqlx::SqlitePool;
use sqlx::migrate::Migrator;
use sqlx::sqlite::SqliteConnectOptions;
use sqlx::sqlite::SqlitePoolOptions;
use sqlx::sqlite::SqliteSynchronous;
use std::path::Path;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use std::time::Instant;
use tokio::sync::OnceCell;

mod busy_retry;
mod connection;
mod nfs;
mod validation;

pub use busy_retry::SqlitePoolWriteExt;
pub(crate) use busy_retry::SqliteWriteQueryExt;
pub(crate) use busy_retry::retry_on_busy;
use connection::JournalModePolicy;
use connection::connect_once;
use validation::SqliteQuickCheckManager;

const DEFAULT_BUSY_TIMEOUT: Duration = Duration::from_secs(5);

<<<<<<< HEAD
const LOGS_DB_FILENAME: &str = "pfterminal_logs_2.sqlite";
const GOALS_DB_FILENAME: &str = "pfterminal_goals_1.sqlite";
const MEMORIES_DB_FILENAME: &str = "pfterminal_memories_1.sqlite";
const STATE_DB_FILENAME: &str = "pfterminal_state_5.sqlite";
const THREAD_HISTORY_DB_FILENAME: &str = "pfterminal_thread_history_1.sqlite";
/// Held while a runtime database is created and migrated (#549). Never
/// deleted: removing a lock file while another process waits on it would let
/// a third process lock a new file at the same path.
const DB_INIT_LOCK_FILENAME: &str = "pfterminal_db_init.lock";
/// How long to wait for another process's database creation or migrations.
/// A migration can rewrite a large state DB, so this is generous.
const DB_INIT_LOCK_WAIT: Duration = Duration::from_secs(120);
const DB_INIT_LOCK_POLL: Duration = Duration::from_millis(20);
/// A wait longer than this is logged, so a stalled holder can be found.
const DB_INIT_LOCK_REPORT_AFTER: Duration = Duration::from_secs(1);
const MIGRATION_RETRY_DELAY: Duration = Duration::from_millis(50);
/// SQLite's extended code SQLITE_IOERR_TRUNCATE; see [`is_shm_reset_race`].
const SQLITE_IOERR_TRUNCATE: &str = "1546";
/// Retries of an open that hit [`is_shm_reset_race`]; the pause before retry
/// `n` is `n` times the delay, about 1.8 s in all.
const SHM_RESET_RETRIES: u32 = 8;
const SHM_RESET_RETRY_DELAY: Duration = Duration::from_millis(50);
/// Connections of the short-wait state DB pool; see [`open_quick_pool`].
const QUICK_POOL_CONNECTIONS: u32 = 2;
/// How long startup waits for that pool's connections to open (milliseconds
/// normally, as one has just been opened).
const QUICK_POOL_OPEN_WAIT: Duration = Duration::from_secs(5);
const QUICK_POOL_OPEN_POLL: Duration = Duration::from_millis(5);
/// How long a connection of a main pool ([`SqliteConfig::open_read_write_pool`])
/// waits for another connection's write lock before SQLITE_BUSY.
pub(crate) const BUSY_TIMEOUT: Duration = Duration::from_secs(5);
||||||| 413492cd6c
const LOGS_DB_FILENAME: &str = "logs_2.sqlite";
const GOALS_DB_FILENAME: &str = "goals_1.sqlite";
const MEMORIES_DB_FILENAME: &str = "memories_1.sqlite";
const STATE_DB_FILENAME: &str = "state_5.sqlite";
const THREAD_HISTORY_DB_FILENAME: &str = "thread_history_1.sqlite";
=======
const LOGS_DB_FILENAME: &str = "logs_2.sqlite";
const GOALS_DB_FILENAME: &str = "goals_1.sqlite";
const MEMORIES_DB_FILENAME: &str = "memories_1.sqlite";
const QUEUE_DB_FILENAME: &str = "queue_1.sqlite";
const STATE_DB_FILENAME: &str = "state_5.sqlite";
const THREAD_HISTORY_DB_FILENAME: &str = "thread_history_1.sqlite";
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a

#[derive(Clone, Copy, Eq, PartialEq)]
enum RecoveryMode {
    BackupAndRebuild,
    Unavailable,
}

#[derive(Clone, Copy)]
struct RuntimeDbSpec {
    label: &'static str,
    filename: &'static str,
    legacy_filename: &'static str,
    kind: DbKind,
    open_phase: &'static str,
    migrate_phase: &'static str,
    /// Opt in only after auditing writers for deferred read-to-write upgrades:
    /// an intervening reclamation commit can make those fail with SQLITE_BUSY_SNAPSHOT.
    background_reclamation: bool,
    recovery: RecoveryMode,
}

impl RuntimeDbSpec {
    fn path(self, sqlite: &SqliteConfig) -> PathBuf {
        let home = if matches!(self.kind, DbKind::Logs) {
            sqlite.logs_home()
        } else {
            sqlite.home()
        };
        home.join(self.filename)
    }
}

const STATE_DB: RuntimeDbSpec = RuntimeDbSpec {
    label: "state DB",
    filename: STATE_DB_FILENAME,
    legacy_filename: "state_5.sqlite",
    kind: DbKind::State,
    open_phase: "open_state",
    migrate_phase: "migrate_state",
    background_reclamation: false,
    recovery: RecoveryMode::BackupAndRebuild,
};

const LOGS_DB: RuntimeDbSpec = RuntimeDbSpec {
    label: "log DB",
    filename: LOGS_DB_FILENAME,
    legacy_filename: "logs_2.sqlite",
    kind: DbKind::Logs,
    open_phase: "open_logs",
    migrate_phase: "migrate_logs",
    // Log transactions write before reading, so they already hold the writer lock.
    background_reclamation: true,
    recovery: RecoveryMode::BackupAndRebuild,
};

const GOALS_DB: RuntimeDbSpec = RuntimeDbSpec {
    label: "goals DB",
    filename: GOALS_DB_FILENAME,
    legacy_filename: "goals_1.sqlite",
    kind: DbKind::Goals,
    open_phase: "open_goals",
    migrate_phase: "migrate_goals",
    background_reclamation: false,
    recovery: RecoveryMode::BackupAndRebuild,
};

const MEMORIES_DB: RuntimeDbSpec = RuntimeDbSpec {
    label: "memories DB",
    filename: MEMORIES_DB_FILENAME,
    legacy_filename: "memories_1.sqlite",
    kind: DbKind::Memories,
    open_phase: "open_memories",
    migrate_phase: "migrate_memories",
    background_reclamation: false,
    recovery: RecoveryMode::BackupAndRebuild,
};

const MEMORIES_V2_DB: RuntimeDbSpec = RuntimeDbSpec {
    label: "memories v2 DB",
    filename: "memories_v2_1.sqlite",
    background_reclamation: false,
    ..MEMORIES_DB
};

const QUEUE_DB: RuntimeDbSpec = RuntimeDbSpec {
    label: "queue DB",
    filename: QUEUE_DB_FILENAME,
    kind: DbKind::Queue,
    open_phase: "open_queue",
    migrate_phase: "migrate_queue",
    background_reclamation: false,
    recovery: RecoveryMode::BackupAndRebuild,
};

const THREAD_HISTORY_DB: RuntimeDbSpec = RuntimeDbSpec {
    label: "thread history DB",
    filename: THREAD_HISTORY_DB_FILENAME,
    legacy_filename: "thread_history_1.sqlite",
    kind: DbKind::ThreadHistory,
    open_phase: "open_thread_history",
    migrate_phase: "migrate_thread_history",
    background_reclamation: false,
    recovery: RecoveryMode::Unavailable,
};

const RUNTIME_DBS: [RuntimeDbSpec; 7] = [
    STATE_DB,
    LOGS_DB,
    GOALS_DB,
    MEMORIES_DB,
    MEMORIES_V2_DB,
    QUEUE_DB,
    THREAD_HISTORY_DB,
];

/// Every runtime database's file name, current and legacy, in the database
/// folder (#651: a sandbox protects them one by one in a shared folder).
pub fn runtime_database_filenames() -> impl Iterator<Item = &'static str> {
    RUNTIME_DBS
        .iter()
        .flat_map(|db| [db.filename, db.legacy_filename])
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RuntimeDbPath {
    pub label: &'static str,
    pub path: PathBuf,
    pub(crate) background_reclamation: bool,
}

/// Resolved configuration shared by all Codex SQLite connections.
/// Clones share directory locks, filesystem probes, and quick-check attempts.
#[derive(Clone, Debug)]
pub struct SqliteConfig {
    sqlite_home: AbsolutePathBuf,
<<<<<<< HEAD
    logs_db_path_override: Option<AbsolutePathBuf>,
||||||| 413492cd6c
=======
    local_logs_dir: Option<LocalLogsDir>,
    is_nfs: Arc<OnceCell<bool>>,
    logs_is_nfs: Arc<OnceCell<bool>>,
    enforce_delete_on_nfs: bool,
    quick_check_manager: SqliteQuickCheckManager,
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a
}

// Directory locks, cached filesystem information, and validation history are operational state,
// not part of configuration equality.
impl PartialEq for SqliteConfig {
    fn eq(&self, other: &Self) -> bool {
        self.sqlite_home == other.sqlite_home
            && self.enforce_delete_on_nfs == other.enforce_delete_on_nfs
            && self.local_logs_dir.as_ref().map(LocalLogsDir::path)
                == other.local_logs_dir.as_ref().map(LocalLogsDir::path)
    }
}

impl Eq for SqliteConfig {}

impl SqliteConfig {
    #[deprecated(
        note = "Use SqliteConfig::from_sqlite_home_with_nfs_enforcement with the managed unsafe_enforce_sqlite_journal_mode requirement, or SqliteConfig::new_for_testing in tests"
    )]
    pub fn from_sqlite_home(sqlite_home: AbsolutePathBuf) -> Self {
<<<<<<< HEAD
        Self {
            sqlite_home,
            logs_db_path_override: None,
        }
    }

    /// Override the logs database path while retaining the shared SQLite home for other runtime
    /// databases.
    pub fn with_logs_db_path(mut self, logs_db_path: AbsolutePathBuf) -> Self {
        self.logs_db_path_override = Some(logs_db_path);
||||||| 413492cd6c
        Self { sqlite_home }
=======
        Self::from_sqlite_home_with_nfs_enforcement(
            sqlite_home,
            /*enforce_delete_on_nfs*/ false,
        )
    }

    /// Construct the configuration with the resolved `unsafe_enforce_sqlite_journal_mode` requirement.
    pub fn from_sqlite_home_with_nfs_enforcement(
        sqlite_home: AbsolutePathBuf,
        enforce_delete_on_nfs: bool,
    ) -> Self {
        Self {
            sqlite_home,
            local_logs_dir: None,
            is_nfs: Arc::new(OnceCell::new()),
            logs_is_nfs: Arc::new(OnceCell::new()),
            enforce_delete_on_nfs,
            quick_check_manager: SqliteQuickCheckManager::new(),
        }
    }

    /// When enforcement is enabled, migrate existing WAL databases on NFS before opening them.
    /// Other clients on all hosts must remain stopped during migration.
    pub fn enable_delete_mode_on_nfs(mut self, enabled: bool) -> Self {
        self.enforce_delete_on_nfs = enabled;
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a
        self
    }

    pub fn new_for_testing(sqlite_home: AbsolutePathBuf) -> Self {
        Self::from_sqlite_home_with_nfs_enforcement(
            sqlite_home,
            /*enforce_delete_on_nfs*/ false,
        )
    }

    pub fn home(&self) -> &Path {
        self.sqlite_home.as_path()
    }

    /// Select the logs directory without accessing it until its database is opened.
    pub fn with_local_logs_home(mut self, home: PathBuf) -> Self {
        self.local_logs_dir = Some(LocalLogsDir::new(home));
        self.logs_is_nfs = Arc::new(OnceCell::new());
        self
    }

    pub(crate) fn logs_home(&self) -> &Path {
        self.local_logs_dir
            .as_ref()
            .map_or(self.home(), LocalLogsDir::path)
    }

    /// Detects if the sqlite_home directory is under the NFS mount (once per the lifetime)
    pub async fn is_nfs(&self) -> anyhow::Result<bool> {
        probe_nfs(self.home(), &self.is_nfs).await
    }

    pub(crate) async fn logs_are_on_nfs(&self) -> anyhow::Result<bool> {
        probe_nfs(self.logs_home(), &self.logs_is_nfs).await
    }

    /// Return the path to the primary state database.
    pub fn state_db_path(&self) -> PathBuf {
        STATE_DB.path(self)
    }

    /// Return whether this home already contains state that can supply runtime overlays.
    ///
    /// Configuration loading uses this check before opening the state runtime so read-only CLI
    /// commands do not create an otherwise absent state database. The legacy filename remains a
    /// valid source because runtime initialization adopts it into the PF namespace.
    pub fn has_existing_state_db(&self) -> bool {
        self.state_db_path().exists() || self.home().join(STATE_DB.legacy_filename).exists()
    }

    /// Return the path to the logs database.
    pub fn logs_db_path(&self) -> PathBuf {
<<<<<<< HEAD
        self.logs_db_path_override.as_ref().map_or_else(
            || LOGS_DB.path(self.home()),
            codex_utils_absolute_path::AbsolutePathBuf::to_path_buf,
        )
||||||| 413492cd6c
        LOGS_DB.path(self.home())
=======
        LOGS_DB.path(self)
    }

    pub(crate) fn legacy_logs_db_path(&self) -> Option<PathBuf> {
        let path = self.home().join(LOGS_DB_FILENAME);
        (path != self.logs_db_path()).then_some(path)
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a
    }

    /// Return the path to the goals database.
    pub fn goals_db_path(&self) -> PathBuf {
        GOALS_DB.path(self)
    }

    /// Return the path to the memories database.
    pub fn memories_db_path(&self) -> PathBuf {
        MEMORIES_DB.path(self)
    }

    pub(crate) fn memories_v2_db_path(&self) -> PathBuf {
        MEMORIES_V2_DB.path(self)
    }

    pub(crate) async fn open_memories_v2_db(&self) -> anyhow::Result<SqlitePool> {
        self.open_runtime_db(
            MEMORIES_V2_DB,
            &crate::migrations::runtime_memories_migrator(),
            /*telemetry_override*/ None,
        )
        .await
    }

    /// Return the path to the durable user-message queue database.
    pub fn queue_db_path(&self) -> PathBuf {
        QUEUE_DB.path(self)
    }

    /// Return the path to the paginated thread-history database.
    pub fn thread_history_db_path(&self) -> PathBuf {
        THREAD_HISTORY_DB.path(self)
    }

    /// Return the paths to every database managed by the state runtime.
    pub fn runtime_db_paths(&self) -> Vec<RuntimeDbPath> {
        RUNTIME_DBS
            .iter()
            .map(|spec| RuntimeDbPath {
                label: spec.label,
                path: spec.path(self),
                background_reclamation: spec.background_reclamation,
            })
            .collect()
    }

<<<<<<< HEAD
    /// Adopt pre-namespace PF Terminal databases once, while refusing to touch an upstream
    /// `.codex` home or a database whose applied migrations do not match this distribution.
    pub(crate) async fn migrate_legacy_runtime_db_names(&self) -> anyhow::Result<()> {
        let has_legacy_db = RUNTIME_DBS
            .iter()
            .any(|spec| self.home().join(spec.legacy_filename).exists());
        if !has_legacy_db {
            return Ok(());
        }
        if self.home().file_name().and_then(|name| name.to_str()) == Some(".codex") {
            anyhow::bail!(
                "refusing to rename upstream-named databases in {}: this database appears to belong to a different Codex/Corbanu Terminal distribution; follow the repair recipe in pfterminal_codex_home_collision_incident_20260710.md",
                self.home().display()
            );
        }

        for spec in RUNTIME_DBS {
            let legacy_path = self.home().join(spec.legacy_filename);
            let namespaced_path = spec.path(self.home());
            if !tokio::fs::try_exists(&legacy_path).await?
                || tokio::fs::try_exists(&namespaced_path).await?
            {
                continue;
            }
            let migrator = match spec.kind {
                DbKind::State => runtime_state_migrator(),
                DbKind::Logs => runtime_logs_migrator(),
                DbKind::Goals => runtime_goals_migrator(),
                DbKind::Memories => runtime_memories_migrator(),
                DbKind::ThreadHistory => runtime_thread_history_migrator(),
            };
            validate_applied_migrations(self, &legacy_path, &migrator).await?;
            rename_if_source_still_exists(&legacy_path, &namespaced_path).await?;
            for suffix in ["-wal", "-shm"] {
                let legacy_sidecar = PathBuf::from(format!("{}{suffix}", legacy_path.display()));
                let namespaced_sidecar =
                    PathBuf::from(format!("{}{suffix}", namespaced_path.display()));
                rename_if_source_still_exists(&legacy_sidecar, &namespaced_sidecar).await?;
            }
        }
        Ok(())
||||||| 413492cd6c
=======
    pub(crate) fn reclamation_db_paths(&self) -> Vec<RuntimeDbPath> {
        self.runtime_db_paths()
            .into_iter()
            .filter(|db| db.background_reclamation)
            .chain(self.legacy_logs_db_path().map(|path| RuntimeDbPath {
                label: "legacy log DB",
                path,
                background_reclamation: true,
            }))
            .collect()
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a
    }

    pub(super) async fn open_state_db(
        &self,
        migrator: &Migrator,
        telemetry_override: Option<&dyn DbTelemetry>,
        init_lock: Option<&RuntimeDbInitLock>,
    ) -> anyhow::Result<SqlitePool> {
        // New state DBs should use incremental auto-vacuum, but retrofitting an
        // existing DB requires a full VACUUM. Do not attempt that during process
        // startup: it is maintenance work that can contend with foreground writers.
        self.open_runtime_db(STATE_DB, migrator, telemetry_override, init_lock)
            .await
    }

    /// A second, small pool on the already opened state DB whose connections
    /// wait at most `busy_timeout` for another connection's write lock, for
    /// writes a model request waits on (see `StateRuntime::quick`).
    pub(super) async fn open_state_db_quick(
        &self,
        busy_timeout: Duration,
    ) -> Result<SqlitePool, Error> {
        let options = SqliteConnectOptions::new()
            .filename(self.state_db_path())
            .create_if_missing(false)
            .journal_mode(SqliteJournalMode::Wal)
            .synchronous(SqliteSynchronous::Normal)
            .busy_timeout(busy_timeout)
            .log_statements(LevelFilter::Off);
        open_quick_pool(options, SqlitePoolOptions::new(), busy_timeout).await
    }

    pub(super) async fn open_logs_db(
        &self,
        migrator: &Migrator,
        telemetry_override: Option<&dyn DbTelemetry>,
        init_lock: Option<&RuntimeDbInitLock>,
    ) -> anyhow::Result<SqlitePool> {
        self.open_runtime_db(LOGS_DB, migrator, telemetry_override, init_lock)
            .await
    }

    pub(super) async fn open_goals_db(
        &self,
        migrator: &Migrator,
        telemetry_override: Option<&dyn DbTelemetry>,
        init_lock: Option<&RuntimeDbInitLock>,
    ) -> anyhow::Result<SqlitePool> {
        self.open_runtime_db(GOALS_DB, migrator, telemetry_override, init_lock)
            .await
    }

    pub(super) async fn open_memories_db(
        &self,
        migrator: &Migrator,
        telemetry_override: Option<&dyn DbTelemetry>,
        init_lock: Option<&RuntimeDbInitLock>,
    ) -> anyhow::Result<SqlitePool> {
        self.open_runtime_db(MEMORIES_DB, migrator, telemetry_override, init_lock)
            .await
    }

    pub(super) async fn open_queue_db(
        &self,
        migrator: &Migrator,
        telemetry_override: Option<&dyn DbTelemetry>,
    ) -> anyhow::Result<SqlitePool> {
        self.open_runtime_db(QUEUE_DB, migrator, telemetry_override)
            .await
    }

    pub(super) async fn open_thread_history_db(
        &self,
        migrator: &Migrator,
        telemetry_override: Option<&dyn DbTelemetry>,
        init_lock: Option<&RuntimeDbInitLock>,
    ) -> anyhow::Result<SqlitePool> {
        self.open_runtime_db(THREAD_HISTORY_DB, migrator, telemetry_override, init_lock)
            .await
    }

    /// Opens and migrates one runtime database under the init lock (#549):
    /// `init_lock` when the caller holds it already, otherwise taken here.
    async fn open_runtime_db(
        &self,
        spec: RuntimeDbSpec,
        migrator: &Migrator,
        telemetry_override: Option<&dyn DbTelemetry>,
        init_lock: Option<&RuntimeDbInitLock>,
    ) -> anyhow::Result<SqlitePool> {
<<<<<<< HEAD
        let path = spec.path(self.home());
        let _own_lock = match init_lock {
            Some(_) => None,
            None => Some(self.lock_runtime_db_init().await),
        };
||||||| 413492cd6c
        let path = spec.path(self.home());
=======
        let path = spec.path(self);
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a
        let started = Instant::now();
        let pool_result = self
            .open_read_write_pool_with_spec(path.as_path(), Some(spec), telemetry_override)
            .await;
        telemetry::record_init_result(
            telemetry_override,
            spec.kind,
            spec.open_phase,
            started.elapsed(),
            &pool_result,
        );
        let pool = pool_result.map_err(|source| {
            RuntimeDbInitError::new(spec.label, "open", path.as_path(), source)
        })?;
        let started = Instant::now();
        let migrate_result = async {
            if matches!(spec.kind, DbKind::State) {
                repair_legacy_recency_migration_version(&pool, migrator).await?;
            }
            match migrator.run(&pool).await {
                // A process that does not take the init lock (an older
                // build, or one that timed out waiting) may have applied the
                // same migration first; it is then recorded, and one more run
                // usually completes (best effort: a process still applying a
                // long series can collide again). A failed migration is rolled
                // back, so the retry starts clean.
                Err(first) => {
                    tracing::info!("retrying the {} migration after: {first}", spec.label);
                    tokio::time::sleep(MIGRATION_RETRY_DELAY).await;
                    migrator.run(&pool).await.map_err(anyhow::Error::from)
                }
                Ok(()) => Ok(()),
            }
        }
        .await;
        telemetry::record_init_result(
            telemetry_override,
            spec.kind,
            spec.migrate_phase,
            started.elapsed(),
            &migrate_result,
        );
        if let Err(source) = migrate_result {
            pool.close().await;
            return Err(
                RuntimeDbInitError::new(spec.label, "migrate", path.as_path(), source).into(),
            );
        }
        Ok(pool)
    }

    /// Serializes creating and migrating the runtime databases across the
    /// processes sharing this home (#549). SQLite migrations take no lock of
    /// their own: processes started together on a new home all ran
    /// `CREATE TABLE`, and the losers failed and ran without a state DB.
    ///
    /// The lock is an OS file lock on [`DB_INIT_LOCK_FILENAME`], released
    /// when the returned guard is dropped, including when a process dies. It
    /// is not re-entrant: take it once and pass it to the `open_*_db` calls.
    /// If it can't be had within [`DB_INIT_LOCK_WAIT`] (or the file can't be
    /// opened), the guard holds nothing and the caller goes on unlocked; the
    /// migration retry in [`Self::open_runtime_db`] still covers the race.
    pub(crate) async fn lock_runtime_db_init(&self) -> RuntimeDbInitLock {
        let path = self.home().join(DB_INIT_LOCK_FILENAME);
        let unlocked = RuntimeDbInitLock { _file: None };
        let file = match open_lock_file(&path) {
            Ok(file) => file,
            Err(err) => {
                log_unlocked(&path, &err);
                return unlocked;
            }
        };
        let started = Instant::now();
        let mut reported_wait = false;
        loop {
            match file.try_lock() {
                Ok(()) => {
                    keep_lock_file_nonempty(&file);
                    return RuntimeDbInitLock { _file: Some(file) };
                }
                Err(std::fs::TryLockError::WouldBlock) => {
                    let waited = started.elapsed();
                    if waited >= DB_INIT_LOCK_WAIT {
                        tracing::warn!(
                            "another process held the database init lock {} for over {:?}; opening unlocked",
                            path.display(),
                            DB_INIT_LOCK_WAIT
                        );
                        return unlocked;
                    }
                    if !reported_wait && waited >= DB_INIT_LOCK_REPORT_AFTER {
                        reported_wait = true;
                        tracing::info!(
                            "waiting for another process to create or migrate the databases ({})",
                            path.display()
                        );
                    }
                    tokio::time::sleep(DB_INIT_LOCK_POLL).await;
                }
                Err(std::fs::TryLockError::Error(err)) => {
                    log_unlocked(&path, &err);
                    return unlocked;
                }
            }
        }
    }

    /// Open a writable Codex SQLite database, creating it if necessary.
<<<<<<< HEAD
    pub async fn open_read_write_pool(&self, path: &Path) -> Result<SqlitePool, Error> {
        retry_shm_reset_race(path, || self.open_read_write_pool_once(path)).await
    }

    async fn open_read_write_pool_once(&self, path: &Path) -> Result<SqlitePool, Error> {
||||||| 413492cd6c
    pub async fn open_read_write_pool(&self, path: &Path) -> Result<SqlitePool, Error> {
=======
    /// Keep this configuration alive to retain the cleanup lock for local logs.
    pub async fn open_read_write_pool(&self, path: &Path) -> anyhow::Result<SqlitePool> {
        self.open_read_write_pool_with_spec(
            path, /*recover_spec*/ None, /*telemetry_override*/ None,
        )
        .await
    }

    async fn open_read_write_pool_with_spec(
        &self,
        path: &Path,
        recover_spec: Option<RuntimeDbSpec>,
        telemetry_override: Option<&dyn DbTelemetry>,
    ) -> anyhow::Result<SqlitePool> {
        self.prepare_logs_directory(path).await?;
        let directory = path
            .parent()
            .filter(|parent| !parent.as_os_str().is_empty())
            .unwrap_or_else(|| Path::new("."));
        let is_nfs = if self.local_logs_dir.is_some() && directory == self.logs_home() {
            self.logs_are_on_nfs().await?
        } else if directory == self.home() {
            self.is_nfs().await?
        } else {
            probe_nfs(directory, &OnceCell::new()).await?
        };
        // Keep the migration lock through pool initialization so another managed startup
        // cannot open the old inode or initialize a database during cutover.
        let migration = if self.enforce_delete_on_nfs && is_nfs {
            Some(nfs::prepare_database(self, path).await?)
        } else {
            None
        };
        let journal_mode_policy = if migration.is_some() {
            JournalModePolicy::RequireDelete
        } else if is_nfs {
            JournalModePolicy::InitializeDelete
        } else {
            JournalModePolicy::InitializeWal
        };
        let recovery = recover_spec.map_or(RecoveryMode::Unavailable, |spec| spec.recovery);
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a
        let options = SqliteConnectOptions::new()
            .filename(path)
            .create_if_missing(true)
            .synchronous(SqliteSynchronous::Normal)
<<<<<<< HEAD
            .busy_timeout(BUSY_TIMEOUT)
||||||| 413492cd6c
            .auto_vacuum(SqliteAutoVacuum::Incremental)
            .busy_timeout(Duration::from_secs(5))
=======
            .busy_timeout(DEFAULT_BUSY_TIMEOUT)
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a
            .log_statements(LevelFilter::Off);
<<<<<<< HEAD
        // Setting `auto_vacuum` takes the write lock even when the value is
        // unchanged, so it must not be a per-connection option: every pooled
        // connection would run it as it connects, and on a database another
        // process is writing, opening a connection - for a reader too - waited
        // out the busy timeout and failed. It only takes effect before the first
        // table exists (later changes need a VACUUM), so set it once, on the one
        // connection that creates the database, and leave existing ones alone.
        if !database_has_content(path) {
            let mut creator = options
                .clone()
                .auto_vacuum(SqliteAutoVacuum::Incremental)
                .connect()
                .await?;
            // An empty file keeps no settings; writing the header persists them.
            sqlx::query("VACUUM").execute(&mut creator).await?;
            creator.close().await?;
        }
        let pool = SqlitePoolOptions::new()
            .max_connections(5)
            .connect_with(options)
            .await?;
        open_wal_index(pool).await
||||||| 413492cd6c
        SqlitePoolOptions::new()
            .max_connections(5)
            .connect_with(options)
            .await
=======
        // Each failed attempt closes its pool before initialization is retried.
        let connect = || async {
            connect_once(&options, journal_mode_policy)
                .await
                .map_err(anyhow::Error::from)
        };
        let pool = retry_on_busy(connect).await?;
        let validation_result = self
            .quick_check_manager
            .quick_check_once(
                &pool,
                path,
                // Limit startup validation to 100 ms.
                Duration::from_millis(/*millis*/ 100),
            )
            .await;

        let Ok(result) = validation_result else {
            pool.close().await;
            return validation_result.map(|_| pool);
        };

        if result == validation::QuickCheckOutcome::CorruptedNeedsFixed {
            tracing::error!(database = %path.display(), "SQLite quick check detected corruption");
            telemetry::record_corruption(telemetry_override, recover_spec.map(|spec| spec.kind));
            if recovery == RecoveryMode::BackupAndRebuild {
                pool.close().await;
                let backups = crate::backup_runtime_db_for_fresh_start(path).await?;
                for backup in backups {
                    tracing::warn!(
                        database = %backup.original_path.display(),
                        backup = %backup.backup_path.display(),
                        "Preserved corrupt SQLite database before rebuilding"
                    );
                }

                return retry_on_busy(connect).await;
            }
        }

        Ok(pool)
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a
    }

    /// Open an existing Codex SQLite database without creating or modifying it.
<<<<<<< HEAD
    pub async fn open_read_only_pool(&self, path: &Path) -> Result<SqlitePool, Error> {
        retry_shm_reset_race(path, || async {
            let options = SqliteConnectOptions::new()
                .filename(path)
                .create_if_missing(false)
                .read_only(true)
                .log_statements(LevelFilter::Off);
            let pool = SqlitePoolOptions::new()
                .max_connections(1)
                .connect_with(options)
                .await?;
            open_wal_index(pool).await
        })
        .await
||||||| 413492cd6c
    pub async fn open_read_only_pool(&self, path: &Path) -> Result<SqlitePool, Error> {
        let options = SqliteConnectOptions::new()
            .filename(path)
            .create_if_missing(false)
            .read_only(true)
            .log_statements(LevelFilter::Off);
        SqlitePoolOptions::new()
            .max_connections(1)
            .connect_with(options)
            .await
=======
    /// Keep this configuration alive to retain the cleanup lock for local logs.
    pub async fn open_read_only_pool(
        &self,
        path: &Path,
        busy_timeout: Option<Duration>,
    ) -> Result<SqlitePool, Error> {
        self.prepare_logs_directory(path).await.map_err(Error::Io)?;
        let mut options = SqliteConnectOptions::new()
            .filename(path)
            .create_if_missing(false)
            .read_only(true)
            .log_statements(LevelFilter::Off);
        if let Some(busy_timeout) = busy_timeout {
            options = options.busy_timeout(busy_timeout);
        }
        SqlitePoolOptions::new()
            .max_connections(1)
            .connect_with(options)
            .await
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a
    }

    /// Open old logs for cleanup without creating, migrating, or rebuilding the database.
    /// Skip legacy NFS logs when managed DELETE enforcement is active.
    pub(crate) async fn open_legacy_logs_for_cleanup(&self) -> anyhow::Result<Option<SqlitePool>> {
        let Some(path) = self.legacy_logs_db_path() else {
            return Ok(None);
        };
        if !tokio::fs::try_exists(&path).await? {
            return Ok(None);
        }
        if self.enforce_delete_on_nfs && self.is_nfs().await? {
            return Ok(None);
        }
        let options = SqliteConnectOptions::new()
            .filename(path)
            .create_if_missing(false)
            .busy_timeout(Duration::ZERO)
            .log_statements(LevelFilter::Off);
        SqlitePoolOptions::new()
            .max_connections(1)
            .connect_with(options)
            .await
            .map(Some)
            .map_err(anyhow::Error::from)
    }

    async fn prepare_logs_directory(&self, path: &Path) -> std::io::Result<()> {
        // Login, login status, and logout load configuration without opening SQLite.
        // Create and lock the logs directory only when opening its database.
        // An unavailable log_dir must not prevent those commands from loading configuration.
        if let Some(directory) = &self.local_logs_dir
            && path == self.logs_db_path()
        {
            directory.initialize().await?;
        }
        Ok(())
    }
}

<<<<<<< HEAD
/// Builds the short-wait pool from `pool_options` (tests add a slow
/// `after_connect`). Waiting for one of its connections counts against the
/// same short wait, and a timeout reads as a busy DB, so opening a connection
/// must not (#865): on a loaded machine (Windows CI runners) an open can take
/// longer than the short wait. The pool used to open its connections on
/// demand, inside that wait, and to close them when idle: such a write then
/// failed as busy with no lock held, and when the open at startup missed it,
/// the process used the main pool and SQLite's 5 s wait for good. Now its
/// connections are opened here, without that limit, and kept.
async fn open_quick_pool(
    options: SqliteConnectOptions,
    pool_options: SqlitePoolOptions,
    busy_timeout: Duration,
) -> Result<SqlitePool, Error> {
    // A database that can't be opened is reported here (the caller then uses
    // the main pool); the pool's own opens below don't report errors.
    options.connect().await?.close().await?;
    let pool = pool_options
        .max_connections(QUICK_POOL_CONNECTIONS)
        .min_connections(QUICK_POOL_CONNECTIONS)
        .idle_timeout(None)
        .max_lifetime(None)
        .acquire_timeout(busy_timeout)
        .connect_lazy_with(options);
    // The pool opens its connections in the background (`min_connections`);
    // wait for them. Should they still be opening, the pool works anyway.
    let deadline = Instant::now() + QUICK_POOL_OPEN_WAIT;
    while pool.num_idle() < QUICK_POOL_CONNECTIONS as usize && Instant::now() < deadline {
        tokio::time::sleep(QUICK_POOL_OPEN_POLL).await;
    }
    Ok(pool)
}

/// Held while the runtime databases are created and migrated (see
/// [`SqliteConfig::lock_runtime_db_init`]); holds nothing when the lock could
/// not be had.
pub(crate) struct RuntimeDbInitLock {
    _file: Option<std::fs::File>,
}

impl Drop for RuntimeDbInitLock {
    /// #756: unlock explicitly. Closing the file releases the lock only once
    /// every copy of its descriptor is closed, and a child process forked
    /// meanwhile (any spawn in this process) keeps a copy until it starts its
    /// program, so the next launch would wait on a lock nobody holds.
    fn drop(&mut self) {
        if let Some(file) = &self._file {
            let _ = file.unlock();
        }
    }
}

fn lock_file_options() -> std::fs::OpenOptions {
    let mut options = std::fs::OpenOptions::new();
    options.read(true).write(true).create(true).truncate(false);
    options
}

fn open_lock_file(path: &Path) -> std::io::Result<std::fs::File> {
    let result = lock_file_options().open(path);
    #[cfg(unix)]
    if let Err(err) = &result
        && err.kind() == std::io::ErrorKind::PermissionDenied
    {
        if let Some(file) = reopen_placeholder(path) {
            return Ok(file);
        }
        // Once more, plainly: the placeholder's run may have ended and
        // removed it meanwhile. Anything still read-only fails again.
        return lock_file_options().open(path);
    }
    result
}

/// #586: the lock is denied to protected agent commands even before it
/// exists, so on Linux bubblewrap holds a missing one with an empty `0444`
/// placeholder for the command's run. A placeholder left behind by a killed
/// sandbox helper would turn the lock off for every later launch, so an
/// empty regular file at the path is made writable for its owner (only the
/// owner can) and opened. The mode change goes through a descriptor of the
/// file checked at the path (not a link), and the read-write descriptor
/// must be that same file. The sandbox sees its own mount, not this mode.
/// (A FIFO swapped in after the check would block the read-only open; that
/// needs write access to the lock path, which protected commands lack.)
#[cfg(unix)]
fn reopen_placeholder(path: &Path) -> Option<std::fs::File> {
    use std::os::unix::fs::MetadataExt;
    use std::os::unix::fs::PermissionsExt;
    let at_path = std::fs::symlink_metadata(path).ok()?;
    if !at_path.file_type().is_file() || at_path.len() != 0 {
        return None;
    }
    let same = |metadata: &std::fs::Metadata| {
        metadata.dev() == at_path.dev() && metadata.ino() == at_path.ino()
    };
    let placeholder = std::fs::File::open(path).ok()?;
    let metadata = placeholder.metadata().ok()?;
    if !same(&metadata) || metadata.len() != 0 {
        return None;
    }
    let mode = (metadata.mode() & 0o7777) | 0o200;
    placeholder
        .set_permissions(std::fs::Permissions::from_mode(mode))
        .ok()?;
    let file = lock_file_options().open(path).ok()?;
    same(&file.metadata().ok()?).then_some(file)
}

/// #586: the lock file is never left empty. On Linux an empty file at its
/// path can be the sandbox's placeholder for a protected command's run (see
/// [`reopen_placeholder`]). The run's cleanup removes an empty placeholder
/// even while a launch holds it locked, and the next launch would then lock
/// a new file. (Cleanup can still race a launch between its length check and
/// the removal; then two launches lock different files once, which the
/// migration retry covers.)
fn keep_lock_file_nonempty(mut file: &std::fs::File) {
    use std::io::Write;
    if file.metadata().is_ok_and(|metadata| metadata.len() == 0) {
        let _ = file.write_all(b"\n");
    }
}

/// A read-only or sandboxed home is expected not to take the lock: quiet.
fn log_unlocked(path: &Path, err: &std::io::Error) {
    if matches!(
        err.kind(),
        std::io::ErrorKind::PermissionDenied | std::io::ErrorKind::ReadOnlyFilesystem
    ) {
        tracing::debug!(
            "no database init lock at {}; opening unlocked: {err}",
            path.display()
        );
    } else {
        tracing::warn!(
            "could not take the database init lock {}; opening unlocked: {err}",
            path.display()
        );
    }
}

/// Whether `path` is an existing, non-empty database file.
pub(crate) fn database_has_content(path: &Path) -> bool {
    std::fs::metadata(path).is_ok_and(|metadata| metadata.len() > 0)
}

/// Whether `err` is the Windows WAL reset race. A connection that finds no
/// other process using a WAL database resets (truncates) its `-shm` file. A
/// process that has just exited without closing its connections can still
/// have that file mapped for a moment after its lock is gone; the truncation
/// then fails with SQLITE_IOERR_TRUNCATE ("disk I/O error"), and succeeds once
/// the mapping is released. Seen when launches start on a new home while
/// earlier ones exit (#691). Other platforms don't report this race, and a
/// real truncation failure there still fails after the retries.
fn is_shm_reset_race(err: &Error) -> bool {
    err.as_database_error()
        .and_then(sqlx::error::DatabaseError::code)
        .is_some_and(|code| code == SQLITE_IOERR_TRUNCATE)
}

/// Reads the schema once, which opens a WAL database's index (`-shm`), so
/// [`is_shm_reset_race`] shows while opening, where it is retried, rather
/// than at the caller's first query.
async fn open_wal_index(pool: SqlitePool) -> Result<SqlitePool, Error> {
    match sqlx::query("SELECT 1 FROM sqlite_master LIMIT 1")
        .fetch_optional(&pool)
        .await
    {
        Ok(_) => Ok(pool),
        Err(err) => {
            pool.close().await;
            Err(err)
        }
    }
}

/// Runs `open`, retrying it after [`is_shm_reset_race`] errors.
async fn retry_shm_reset_race<T, F, Fut>(path: &Path, mut open: F) -> Result<T, Error>
where
    F: FnMut() -> Fut,
    Fut: std::future::Future<Output = Result<T, Error>>,
{
    let mut retries = 0;
    loop {
        match open().await {
            Err(err) if retries < SHM_RESET_RETRIES && is_shm_reset_race(&err) => {
                retries += 1;
                tracing::info!(
                    "retrying to open {} (attempt {}) after: {err}",
                    path.display(),
                    retries + 1
                );
                tokio::time::sleep(SHM_RESET_RETRY_DELAY * retries).await;
            }
            result => return result,
        }
    }
}

/// Keep fault-fixture connection options and checkout barriers intact at the SQLite boundary.
#[cfg(test)]
pub(crate) async fn open_pool_for_testing(
    pool_options: SqlitePoolOptions,
    connect_options: SqliteConnectOptions,
) -> Result<SqlitePool, Error> {
    pool_options.connect_with(connect_options).await
}

async fn rename_if_source_still_exists(source: &Path, destination: &Path) -> anyhow::Result<()> {
    match tokio::fs::rename(source, destination).await {
        Ok(()) => Ok(()),
        Err(err)
            if err.kind() == std::io::ErrorKind::NotFound
                && tokio::fs::try_exists(destination).await? =>
        {
            Ok(())
        }
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(err) => Err(err.into()),
    }
||||||| 413492cd6c
=======
async fn probe_nfs(home: &Path, probe: &OnceCell<bool>) -> anyhow::Result<bool> {
    probe
        .get_or_try_init(|| async {
            let home = home.to_path_buf();
            Ok(tokio::task::spawn_blocking(move || codex_file_system::is_nfs(&home)).await??)
        })
        .await
        .copied()
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a
}

#[cfg(test)]
#[path = "sqlite_tests.rs"]
mod tests;
