use std::fs::OpenOptions;
use std::io::ErrorKind;
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;

use color_eyre::eyre::Result;
use color_eyre::eyre::eyre;

pub(crate) struct SessionWriterLock {
    path: PathBuf,
    identity: String,
}

impl SessionWriterLock {
    pub(crate) fn acquire(root: &Path) -> Result<Self> {
        std::fs::create_dir_all(root)?;
        let path = root.join(".pfterminal-writer.lock");
        let identity = process_identity(std::process::id())
            .ok_or_else(|| eyre!("cannot determine this process start time"))?;
        for _ in 0..2 {
            match OpenOptions::new().write(true).create_new(true).open(&path) {
                Ok(mut file) => {
                    file.write_all(identity.as_bytes())?;
                    file.sync_all()?;
                    return Ok(Self { path, identity });
                }
                Err(error) if error.kind() == ErrorKind::AlreadyExists => {
                    let owner = std::fs::read_to_string(&path).unwrap_or_default();
                    if owner.trim().is_empty() || !process_identity_is_live(owner.trim()) {
                        match std::fs::remove_file(&path) {
                            Ok(()) => continue,
                            Err(error) if error.kind() == ErrorKind::NotFound => continue,
                            Err(error) => return Err(error.into()),
                        }
                    }
                    return Err(eyre!(
                        "writable PFTerminal state is already owned by process {owner}; close that session before opening `{}`",
                        root.display()
                    ));
                }
                Err(error) => return Err(error.into()),
            }
        }
        Err(eyre!(
            "could not acquire writable PFTerminal state lock at `{}`",
            path.display()
        ))
    }
}

impl Drop for SessionWriterLock {
    fn drop(&mut self) {
        if std::fs::read_to_string(&self.path).is_ok_and(|owner| owner == self.identity) {
            let _ = std::fs::remove_file(&self.path);
        }
    }
}

fn process_identity(pid: u32) -> Option<String> {
    let stat = std::fs::read_to_string(format!("/proc/{pid}/stat")).ok()?;
    let close = stat.rfind(')')?;
    let start_time = stat[close + 2..].split_whitespace().nth(19)?;
    Some(format!("{pid} {start_time}"))
}

fn process_identity_is_live(identity: &str) -> bool {
    let mut fields = identity.split_whitespace();
    let Some(pid) = fields.next().and_then(|pid| pid.parse::<u32>().ok()) else {
        return false;
    };
    fields.next().is_some_and(|expected_start| {
        process_identity(pid)
            .and_then(|actual| actual.split_whitespace().nth(1).map(str::to_string))
            .as_deref()
            == Some(expected_start)
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn live_owner_is_rejected_and_drop_releases_lock() {
        let root = tempfile::tempdir().expect("root");
        let lock = SessionWriterLock::acquire(root.path()).expect("first owner");
        assert!(SessionWriterLock::acquire(root.path()).is_err());
        drop(lock);
        SessionWriterLock::acquire(root.path()).expect("released owner");
    }

    #[test]
    fn stale_owner_is_reclaimed() {
        let root = tempfile::tempdir().expect("root");
        std::fs::write(root.path().join(".pfterminal-writer.lock"), "999999 1")
            .expect("stale lock");
        SessionWriterLock::acquire(root.path()).expect("stale takeover");
    }
}
