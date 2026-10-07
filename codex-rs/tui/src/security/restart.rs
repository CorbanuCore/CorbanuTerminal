//! PF-24-S02 "restart now": after `/security` saves a level that takes effect
//! at the next start, Corbanu Terminal can start again at once. The new
//! process runs the same program with the same arguments, in the same folder
//! and environment, except an initial prompt (it was already sent).

use std::ffi::OsString;

/// `argv` (program first) without the values clap parsed as the initial
/// `prompt`, at any subcommand level; the program is left out. Unparseable
/// arguments are kept as they are.
pub fn restart_args(command: clap::Command, argv: Vec<OsString>) -> Vec<OsString> {
    let mut drop = Vec::new();
    if let Ok(matches) = command.try_get_matches_from(argv.clone()) {
        // A subcommand's indices count from its own name.
        let mut level = Some((&matches, 0));
        while let Some((current, offset)) = level {
            if let Ok(Some(_)) = current.try_get_raw("prompt")
                && let Some(indices) = current.indices_of("prompt")
            {
                drop.extend(indices.map(|index| offset + index));
            }
            level = current.subcommand().and_then(|(name, sub)| {
                let position = argv
                    .iter()
                    .enumerate()
                    .skip(offset + 1)
                    .find(|(_, arg)| arg.to_str() == Some(name))?
                    .0;
                Some((sub, position))
            });
        }
    }
    argv.into_iter()
        .enumerate()
        .skip(1)
        .filter(|(index, _)| !drop.contains(index))
        .map(|(_, arg)| arg)
        .collect()
}

/// Replace this process with a fresh start (on Windows: run it and exit
/// with its status). `command` is the program's argument parser. Returns
/// only on failure.
pub fn restart_process(command: clap::Command) -> std::io::Error {
    // The new process must not see this one's Aggressive lock (Windows keeps
    // this process alive while the new one runs).
    super::launch::release_aggressive_lock();
    let program = match std::env::current_exe() {
        Ok(program) => program,
        Err(err) => return err,
    };
    let args = restart_args(command, std::env::args_os().collect());
    let mut process = std::process::Command::new(program);
    process.args(args);
    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        process.exec()
    }
    #[cfg(not(unix))]
    {
        match process.status() {
            Ok(status) => std::process::exit(status.code().unwrap_or(1)),
            Err(err) => err,
        }
    }
}

#[cfg(test)]
mod tests {
    use clap::Arg;
    use clap::Command;
    use pretty_assertions::assert_eq;

    use super::*;

    fn args(values: &[&str]) -> Vec<OsString> {
        values.iter().map(OsString::from).collect()
    }

    fn command() -> Command {
        Command::new("corbanu")
            .arg(Arg::new("model").short('m').long("model"))
            .arg(
                Arg::new("config")
                    .short('c')
                    .action(clap::ArgAction::Append),
            )
            .arg(Arg::new("prompt"))
            .subcommand(
                Command::new("resume")
                    .arg(Arg::new("session_id"))
                    .arg(Arg::new("prompt")),
            )
    }

    #[test]
    fn security_confirm_restart_keeps_options_and_drops_the_prompt() {
        // The prompt equals an option value: only the prompt goes.
        assert_eq!(
            restart_args(
                command(),
                args(&["corbanu", "-c", "x=1", "fix", "-m", "fix"])
            ),
            args(&["-c", "x=1", "-m", "fix"])
        );
        assert_eq!(
            restart_args(command(), args(&["corbanu", "resume", "abc", "go on"])),
            args(&["resume", "abc"])
        );
        assert_eq!(
            restart_args(command(), args(&["corbanu", "-m", "m"])),
            args(&["-m", "m"])
        );
    }
}
