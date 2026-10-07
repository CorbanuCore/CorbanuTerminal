//! PF-24-S02: the human `/security` confirmation.
//!
//! The picker builds one [`TransitionRequest`] when a person presses the
//! confirm key on a review screen; nothing else constructs one, and no model,
//! tool or app-server request reaches [`run`]. It saves the picker's level
//! file and commits Core's transition (`codex_core::security_level_change`)
//! off the async runtime (the Core store may wait up to 2 s for its lock).
//!
//! Order, so a crash or failure never leaves a weaker state than reviewed:
//! - Aggressive: receipt, level file, then Core. A Core refusal restores the
//!   previous files. A crash before Core leaves the level file stricter,
//!   which the next start enforces and reports.
//! - Permissive: level file, then Core. A Core failure restores the previous
//!   files. A crash before Core leaves Core's record stricter than the file,
//!   which the next start reads as changed outside `/security` and enforces
//!   Aggressive.

use std::path::Path;
use std::path::PathBuf;
use std::sync::mpsc;

use super::level;
use super::level::ChosenLevel;
use super::level::NestedAgents;
use super::preflight;
use crate::legacy_core::protected_preflight::Preflight;
use crate::legacy_core::security_level_change::LevelBasis;
use crate::legacy_core::security_level_change::LevelChangeError;
use crate::legacy_core::security_level_change::LevelChangeReport;
use crate::legacy_core::security_level_change::Probes;
use crate::legacy_core::security_level_change::SecurityLevel;
use crate::legacy_core::security_level_change::StoredSecurityState;
use crate::legacy_core::security_level_change::commit_human_level_change;

const CHANGED: &str = "Not saved: the security state changed since you reviewed it (another session may have changed it). Nothing changed; review it again";

/// Core's level for a picker level.
pub(crate) fn core_level(level: ChosenLevel) -> SecurityLevel {
    match level {
        ChosenLevel::Permissive => SecurityLevel::Permissive,
        ChosenLevel::Aggressive => SecurityLevel::Aggressive,
    }
}

/// A confirmed choice. Built only by the picker's confirm key.
pub(crate) struct TransitionRequest {
    pub(crate) codex_home: PathBuf,
    pub(crate) target: ChosenLevel,
    pub(crate) nested: NestedAgents,
    /// The session's configured `[security]` level (without the stored one).
    pub(crate) configured: SecurityLevel,
    /// The session the confirmation was made in.
    pub(crate) thread: Option<codex_protocol::ThreadId>,
    /// What the review showed.
    pub(crate) reviewed: LevelBasis,
    /// The preflight that passed review and its recheck; its receipt is
    /// written with an Aggressive save.
    pub(crate) passed_preflight: Option<Preflight>,
    /// Whether Core's level is raised with an Aggressive save: only after a
    /// passed PF-29 preflight.
    pub(crate) raise_core: bool,
}

impl TransitionRequest {
    /// Whether Core's record must change for this choice.
    pub(crate) fn needs_core(&self) -> bool {
        core_commit_needed(&self.reviewed, self.target, self.raise_core)
    }
}

/// See [`TransitionRequest::needs_core`].
pub(crate) fn core_commit_needed(basis: &LevelBasis, target: ChosenLevel, raise: bool) -> bool {
    match target {
        ChosenLevel::Aggressive => {
            raise
                && (basis.next_start != SecurityLevel::Aggressive
                    || basis.stored != StoredSecurityState::Level(SecurityLevel::Aggressive))
        }
        ChosenLevel::Permissive => {
            basis.next_start != SecurityLevel::Permissive
                || matches!(basis.stored, StoredSecurityState::Unreadable(_))
                || matches!(basis.stored, StoredSecurityState::Level(level) if level != SecurityLevel::Permissive)
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct TransitionOutcome {
    pub(crate) level: ChosenLevel,
    /// `None` when Core's record did not need to change.
    pub(crate) core: Option<LevelChangeReport>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct TransitionFailure {
    pub(crate) message: String,
    /// The state changed since the review: show it again before retrying.
    pub(crate) review_again: bool,
}

pub(crate) type TransitionResult = Result<TransitionOutcome, TransitionFailure>;

/// A commit in progress.
pub(crate) enum Pending {
    Running(mpsc::Receiver<TransitionResult>),
    Finished(TransitionResult),
}

impl Pending {
    /// The result, once there is one.
    pub(crate) fn poll(&mut self) -> Option<TransitionResult> {
        match self {
            Self::Finished(result) => Some(result.clone()),
            Self::Running(receiver) => match receiver.try_recv() {
                Ok(result) => Some(result),
                Err(mpsc::TryRecvError::Empty) => None,
                Err(mpsc::TryRecvError::Disconnected) => Some(Err(TransitionFailure {
                    message: "the save stopped unexpectedly; check /security again".to_string(),
                    review_again: true,
                })),
            },
        }
    }
}

/// Start the commit on its own thread, or run it now when `inline`.
pub(crate) fn start(request: TransitionRequest, inline: bool) -> Pending {
    if inline {
        return Pending::Finished(run(request));
    }
    let (sender, receiver) = mpsc::channel();
    let spawned = std::thread::Builder::new()
        .name("security-confirm".to_string())
        .spawn(move || {
            let _ = sender.send(run(request));
        });
    match spawned {
        Ok(_) => Pending::Running(receiver),
        Err(err) => Pending::Finished(Err(TransitionFailure {
            message: format!("could not start the save: {err}. Nothing changed"),
            review_again: false,
        })),
    }
}

/// `security_confirm.lock` in the Corbanu home, held exclusively while a
/// confirmation saves (not Core's `security_state.lock`, which the commit
/// takes itself).
struct ConfirmLock {
    /// Unlocked when dropped.
    _file: std::fs::File,
}

impl ConfirmLock {
    const FILE: &str = "security_confirm.lock";
    const WAIT: std::time::Duration = std::time::Duration::from_secs(2);

    fn acquire(codex_home: &Path) -> Result<Self, String> {
        let file = std::fs::OpenOptions::new()
            .create(true)
            .truncate(false)
            .write(true)
            .open(codex_home.join(Self::FILE))
            .map_err(|err| format!("cannot open {}: {err}", Self::FILE))?;
        let deadline = std::time::Instant::now() + Self::WAIT;
        loop {
            match file.try_lock() {
                Ok(()) => return Ok(Self { _file: file }),
                Err(std::fs::TryLockError::WouldBlock) if std::time::Instant::now() < deadline => {
                    std::thread::sleep(std::time::Duration::from_millis(25));
                }
                Err(std::fs::TryLockError::WouldBlock) => {
                    return Err("another /security confirmation is saving".to_string());
                }
                Err(std::fs::TryLockError::Error(err)) => {
                    return Err(format!("cannot lock {}: {err}", Self::FILE));
                }
            }
        }
    }
}

/// Files the picker writes, as they were before the save.
struct Snapshot(Vec<(PathBuf, Option<Vec<u8>>)>);

impl Snapshot {
    fn take(codex_home: &Path) -> Self {
        Self(
            [
                level::state_path(codex_home),
                level::rules_path(codex_home),
                preflight::receipt_path(codex_home),
            ]
            .into_iter()
            .map(|path| {
                let contents = std::fs::read(&path).ok();
                (path, contents)
            })
            .collect(),
        )
    }

    fn restore(&self) -> Result<(), String> {
        let mut errors = Vec::new();
        for (path, contents) in &self.0 {
            let result = match contents {
                Some(contents) => write_atomically(path, contents),
                None => match std::fs::remove_file(path) {
                    Err(err) if err.kind() != std::io::ErrorKind::NotFound => Err(err),
                    Ok(()) | Err(_) => Ok(()),
                },
            };
            if let Err(err) = result {
                errors.push(format!("{}: {err}", path.display()));
            }
        }
        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors.join("; "))
        }
    }
}

fn write_atomically(path: &Path, contents: &[u8]) -> std::io::Result<()> {
    use std::io::Write;
    let parent = path
        .parent()
        .ok_or_else(|| std::io::Error::other("path has no parent"))?;
    std::fs::create_dir_all(parent)?;
    let mut file = tempfile::NamedTempFile::new_in(parent)?;
    file.write_all(contents)?;
    file.as_file().sync_all()?;
    file.persist(path).map_err(|err| err.error)?;
    Ok(())
}

/// Save, then commit; on failure put the files back. Blocking.
pub(crate) fn run(request: TransitionRequest) -> TransitionResult {
    let home = request.codex_home.as_path();
    let failed = |message: String, review_again: bool| TransitionFailure {
        message,
        review_again,
    };
    // Nothing is written unless Core's state is still what was reviewed
    // (Core checks again under its lock).
    // One confirmation at a time per home, across processes: a restore must
    // never undo another confirmation's files.
    let _lock = match ConfirmLock::acquire(home) {
        Ok(lock) => lock,
        Err(err) => {
            return Err(failed(format!("Not saved: {err}. Nothing changed"), false));
        }
    };
    if LevelBasis::read(home, request.configured, request.thread) != request.reviewed {
        return Err(failed(CHANGED.to_string(), true));
    }
    let snapshot = Snapshot::take(home);
    let saved = match (request.target, &request.passed_preflight) {
        (ChosenLevel::Aggressive, Some(passed)) => preflight::save_receipt(home, passed),
        (ChosenLevel::Aggressive, None) => Ok(()),
        (ChosenLevel::Permissive, _) => preflight::remove_receipt(home),
    }
    .and_then(|()| level::save(home, request.target, request.nested));
    if let Err(err) = saved {
        let restored = snapshot.restore();
        return Err(failed(
            match restored {
                Ok(()) => format!("{err}. Nothing changed"),
                Err(restore) => {
                    format!("{err}, and the previous files could not be put back ({restore})")
                }
            },
            false,
        ));
    }
    if !request.needs_core() {
        return Ok(TransitionOutcome {
            level: request.target,
            core: None,
        });
    }
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |elapsed| {
            i64::try_from(elapsed.as_secs()).unwrap_or(i64::MAX)
        });
    match commit_human_level_change(
        home,
        request.configured,
        request.thread,
        &request.reviewed,
        core_level(request.target),
        Probes::Passed,
        now,
    ) {
        Ok(report) if report.next_start == core_level(request.target) => Ok(TransitionOutcome {
            level: request.target,
            core: Some(report),
        }),
        // Core kept another level for the next start: the level file must
        // not disagree with it.
        Ok(report) => {
            let restored = snapshot.restore();
            Err(failed(
                match restored {
                    Ok(()) => format!(
                        "Not saved: Core keeps {} for the next start. Nothing changed",
                        report.next_start
                    ),
                    Err(restore) => format!(
                        "Not saved: Core keeps {} for the next start, and the previous files could not be put back ({restore})",
                        report.next_start
                    ),
                },
                true,
            ))
        }
        Err(error) => {
            let review_again = matches!(error, LevelChangeError::Changed);
            let message = match snapshot.restore() {
                Ok(()) if review_again => CHANGED.to_string(),
                Ok(()) => format!("Not saved: {error}. Nothing changed"),
                Err(restore) => format!(
                    "Not saved: {error}. The previous files could not be put back ({restore}); the next start enforces Aggressive until you choose a level again"
                ),
            };
            Err(failed(message, review_again))
        }
    }
}

#[cfg(test)]
#[path = "confirm_tests.rs"]
mod tests;
