//! #874, #837 (Travis, 2026-10-16, D6): the post-command `.git` check.
//!
//! Where an agent command under Moderate or Aggressive can write a project's
//! `.git` (full access in your own settings; under the default workspace
//! sandbox `.git` is read-only), it can leave things there that your own git
//! later follows outside the sandbox: a link that git writes through
//! (`COMMIT_EDITMSG`, `MERGE_MSG`, `FETCH_HEAD`, `logs/*`), a hard link, rebase
//! or sequencer steps whose `exec` lines `git rebase --continue` runs, a
//! legacy remote definition (`remotes/`, `branches/`), or a replaced `.git`.
//!
//! It looks before the session's first such command, after each one, before
//! each later command is decided and when a turn ends, and compares the
//! relevant `.git` entries with the last look. Anything new of those
//! kinds is reported to the person with how to remove it; nothing is
//! removed. Until it is removed (and while a look is incomplete), a person's
//! approval does not lift the protected-file rules (`recheck`), so an
//! approved `git commit` cannot write through a planted link into a
//! protected file.
//!
//! Cheap: below `objects`, `lfs` and `annex` only their own entries are
//! looked at (git reads the files below and writes new ones by renaming),
//! and a folder whose status-change time is unchanged is not listed again.

#[cfg(unix)]
mod unix;

/// Told where a look fails (its task panicked or was ended).
pub(crate) const CHECK_FAILED: &str = "Corbanu couldn't check this project's git folder for \
     links and rebase steps an agent command may have left. Look for them before you run git \
     there.";

#[cfg(unix)]
pub(crate) use unix::*;

/// Elsewhere (Windows) there is no check (documented as macOS and Linux
/// only).
#[cfg(not(unix))]
mod other {
    use codex_protocol::ThreadId;
    use std::path::Path;
    use std::path::PathBuf;

    /// None are found here.
    #[derive(Clone, Debug)]
    pub(crate) enum Repo {}

    pub(crate) fn repos_and(_folders: &[&Path], _also: Option<&Path>) -> Vec<Repo> {
        Vec::new()
    }

    pub(crate) fn begin(_thread: ThreadId, _repos: &[Repo], _level: &str) -> Option<String> {
        None
    }

    pub(crate) fn finish(_thread: ThreadId, _repos: &[Repo], _level: &str) -> Option<String> {
        None
    }

    pub(crate) fn recheck(
        _thread: ThreadId,
        _level: &str,
        _folders: &[&Path],
    ) -> (Option<String>, Vec<String>) {
        (None, Vec::new())
    }

    impl Repo {
        pub(crate) fn probes(&self) -> Vec<PathBuf> {
            match *self {}
        }
    }
}

#[cfg(not(unix))]
pub(crate) use other::*;

#[cfg(all(test, unix))]
#[path = "git_postcheck_tests.rs"]
mod tests;
