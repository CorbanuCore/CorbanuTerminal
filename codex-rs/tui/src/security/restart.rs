//! PF-24-S02 "restart now": after `/security` saves a level that takes effect
//! at the next start, Corbanu Terminal can start again at once. The new
//! process runs the same program with the same arguments, in the same folder
//! and environment. It is marked with [`RESTARTED_ENV`], so it does not send
//! the initial prompt or images again (they were already sent); the arguments
//! themselves are never edited.

use std::ffi::OsString;

/// Set on the restarted process; its initial prompt and images are dropped.
pub const RESTARTED_ENV: &str = "CORBANU_RESTARTED_FOR_SECURITY";

static RESTARTED: std::sync::OnceLock<bool> = std::sync::OnceLock::new();

/// Read and remove [`RESTARTED_ENV`] at process entry, before any thread
/// starts, so agent commands and other children never inherit it.
pub fn take_restart_marker() {
    let restarted = std::env::var_os(RESTARTED_ENV).is_some_and(|value| value == "1");
    // Safe: called at process entry, while the process is single-threaded.
    unsafe {
        std::env::remove_var(RESTARTED_ENV);
    }
    let _ = RESTARTED.set(restarted);
}

/// Whether this process was started by "restart now".
pub fn restarted() -> bool {
    RESTARTED.get().copied().unwrap_or(false)
}

/// #444 (AGG-37): the line printed before a restart (without a newline, see
/// [`RESTART_BANNER`]) stayed above the new session. The restarted process
/// erases it before it draws.
pub(crate) fn clear_restart_banner() {
    use std::io::IsTerminal as _;
    if !restarted() || !std::io::stdout().is_terminal() {
        return;
    }
    // Back to the start of the line, then erase it (crossterm also handles
    // consoles without escape-code processing).
    let _ = crossterm::execute!(
        std::io::stdout(),
        crossterm::cursor::MoveToColumn(0),
        crossterm::terminal::Clear(crossterm::terminal::ClearType::CurrentLine)
    );
}

/// Printed, without a newline, just before "restart now" replaces the
/// process.
pub const RESTART_BANNER: &str = "Restarting Corbanu Terminal to apply the saved security level…";
/// #436: why "restart now" cannot be used, or `None`. Provider keys this
/// process handed from its environment to the credential broker are gone
/// from the environment a restart inherits (the broker never hands a value
/// back), so a restarted session would start without them.
pub(crate) fn restart_blocker() -> Option<String> {
    restart_blocker_for(&crate::legacy_core::broker_removed_environment_keys())
}

pub(crate) fn restart_blocker_for(removed_env_keys: &[String]) -> Option<String> {
    if removed_env_keys.is_empty() {
        return None;
    }
    let names = removed_env_keys.join(", ");
    Some(format!(
        "Restart now is not available: this session removed {names} from its environment (the \
         credential broker holds the key, or it was discarded when the broker could not start), \
         so a restart from here would not have it. Quit Corbanu Terminal (/quit) and start it \
         again from your shell to apply the saved level."
    ))
}

/// The program and arguments to start again with.
fn restart_command(args: Vec<OsString>) -> std::io::Result<std::process::Command> {
    let mut command = std::process::Command::new(std::env::current_exe()?);
    command
        .args(args.into_iter().skip(1))
        .env(RESTARTED_ENV, "1");
    Ok(command)
}

/// Replace this process with a fresh start (on Windows: run it and exit
/// with its status). Returns only on failure.
pub fn restart_process() -> std::io::Error {
    // The new process must not see this one's Aggressive lock (Windows keeps
    // this process alive while the new one runs).
    super::launch::release_aggressive_lock();
    let mut command = match restart_command(std::env::args_os().collect()) {
        Ok(command) => command,
        Err(err) => return err,
    };
    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        command.exec()
    }
    #[cfg(not(unix))]
    {
        match command.status() {
            Ok(status) => std::process::exit(status.code().unwrap_or(1)),
            Err(err) => err,
        }
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn security_confirm_restart_keeps_every_argument_and_marks_the_process() {
        let args = [
            "corbanu",
            "--model=x",
            "do it",
            "--sandbox=read-only",
            "-i",
            "a,b",
        ]
        .map(OsString::from)
        .to_vec();
        let command = restart_command(args).unwrap();
        assert_eq!(
            command.get_args().collect::<Vec<_>>(),
            ["--model=x", "do it", "--sandbox=read-only", "-i", "a,b"]
        );
        assert_eq!(
            command.get_envs().collect::<Vec<_>>(),
            [(
                std::ffi::OsStr::new(RESTARTED_ENV),
                Some(std::ffi::OsStr::new("1"))
            )]
        );
    }
}
