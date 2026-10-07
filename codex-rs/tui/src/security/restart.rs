//! PF-24-S02 "restart now": after `/security` saves a level that takes effect
//! at the next start, Corbanu Terminal can start again at once. The new
//! process runs the same program with the same arguments, in the same folder
//! and environment, except an initial prompt (it was already sent).

use std::ffi::OsString;

/// The arguments to start again with: these, minus the last one equal to
/// the initial `prompt`.
pub fn restart_args(args: Vec<OsString>, prompt: Option<&str>) -> Vec<OsString> {
    let mut args = args;
    if let Some(prompt) = prompt
        && let Some(index) = args.iter().rposition(|arg| arg.to_str() == Some(prompt))
    {
        args.remove(index);
    }
    args
}

/// Replace this process with a fresh start (on Windows: run it and exit
/// with its status). Returns only on failure.
pub fn restart_process(prompt: Option<&str>) -> std::io::Error {
    let program = match std::env::current_exe() {
        Ok(program) => program,
        Err(err) => return err,
    };
    let args = restart_args(std::env::args_os().skip(1).collect(), prompt);
    let mut command = std::process::Command::new(program);
    command.args(args);
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

    fn args(values: &[&str]) -> Vec<OsString> {
        values.iter().map(OsString::from).collect()
    }

    #[test]
    fn security_confirm_restart_keeps_options_and_drops_the_prompt() {
        assert_eq!(
            restart_args(
                args(&["-c", "model=\"m\"", "--enable", "security_levels", "fix it"]),
                Some("fix it")
            ),
            args(&["-c", "model=\"m\"", "--enable", "security_levels"])
        );
        assert_eq!(
            restart_args(args(&["resume", "--last"]), None),
            args(&["resume", "--last"])
        );
    }
}
