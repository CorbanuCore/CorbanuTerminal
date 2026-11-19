//! #2524 (D688): PowerShell spelled as Windows runs it. Exec-policy rules
//! read a PowerShell script's commands when the shell is spelled as the
//! script reader knows it (`pwsh`, `powershell`, `powershell.exe`). Windows
//! looks program names up without regard to case and drops trailing dots
//! and spaces, so `PWSH -c`, `PowerShell.EXE -Command` and
//! `C:\…\POWERSHELL.EXE -Command` run the same shell; their scripts must be
//! read too. PowerShell's own flags (`-Command`, `-c`) already compare in
//! any case.

use super::names;
use super::work;

/// `words` with the shell respelled `pwsh.exe` or `powershell.exe` in its
/// own directory, so the same program reads the script, where `words` runs
/// PowerShell on Windows under a spelling the reader does not know. `None`
/// for any other command and where the reader reads `words` already.
/// Sync (#2671): used wherever a command is checked for a Windows executor,
/// on any host. Where this host's reader does not know that path (a `\`
/// path off Windows), the shell is spelled by its file name alone
/// (`pwsh.exe`), which reads the same command line.
pub(crate) fn policy_powershell_script(words: &[String]) -> Option<Vec<String>> {
    respelled(words, names::NameCase::Windows)
}

fn respelled(words: &[String], case: names::NameCase) -> Option<Vec<String>> {
    let (program, rest) = words.split_first()?;
    if case != names::NameCase::Windows
        // #2568: the reader reads any command line of a PowerShell it knows.
        || codex_shell_command::powershell::is_powershell_program(program)
    {
        return None;
    }
    let name = names::launcher_name_for(program, case);
    if !matches!(name.as_ref(), "pwsh" | "powershell") {
        return None;
    }
    // A `\\?\` path skips Win32 normalization (`pwsh.exe.` and `pwsh`
    // name other files there than `pwsh.exe`): respell its letter case only,
    // else read just its literal argv.
    if (program.starts_with(r"\\?\") || program.starts_with("//?/"))
        && !program
            .to_ascii_lowercase()
            .ends_with(&format!("\\{name}.exe"))
    {
        return None;
    }
    // Keep the directory (or drive) as written: `C:\PS\PWSH.EXE` reads as
    // `C:\PS\pwsh.exe`, the same file.
    let path = program.trim_end_matches(['/', '\\']);
    let dir_end = match work::rfind_by(path, |c| matches!(c, '/' | '\\')) {
        Some(separator) => separator + 1,
        None => match path.as_bytes() {
            [drive, b':', ..] if drive.is_ascii_alphabetic() => 2,
            _ => 0,
        },
    };
    let mut shell = format!("{}{name}.exe", &path[..dir_end]);
    if !codex_shell_command::powershell::is_powershell_program(&shell) {
        shell = format!("{name}.exe");
    }
    if shell == *program {
        return None;
    }
    let mut respelled = Vec::with_capacity(words.len());
    respelled.push(shell);
    respelled.extend_from_slice(rest);
    Some(respelled)
}

#[cfg(test)]
mod tests {
    use super::names::NameCase;
    use super::respelled;
    use pretty_assertions::assert_eq;

    fn words(line: &[&str]) -> Vec<String> {
        line.iter().map(ToString::to_string).collect()
    }

    /// `on_windows` where this host's reader knows `\\` paths, else the
    /// file name alone (#2671: a host checking for a Windows executor).
    fn host<'a>(on_windows: &'a str, elsewhere: &'a str) -> Option<&'a str> {
        Some(if cfg!(windows) { on_windows } else { elsewhere })
    }

    #[test]
    fn powershell_respells_only_on_windows() {
        for (program, expected) in [
            ("PWSH", Some("pwsh.exe")),
            ("Pwsh.Exe", Some("pwsh.exe")),
            ("PowerShell.EXE", Some("powershell.exe")),
            ("POWERSHELL.EXE. ", Some("powershell.exe")),
            (
                r"C:\Windows\System32\WindowsPowerShell\v1.0\POWERSHELL.EXE",
                host(
                    r"C:\Windows\System32\WindowsPowerShell\v1.0\powershell.exe",
                    "powershell.exe",
                ),
            ),
            (r"C:PWSH", host("C:pwsh.exe", "pwsh.exe")),
            ("D:/PS/Pwsh", Some("D:/PS/pwsh.exe")),
            // Spellings the reader knows already, and other programs.
            ("pwsh", None),
            ("powershell", None),
            ("powershell.exe", None),
            // The reader drops the last extension, so it reads these too.
            ("powershell.exe. ", None),
            ("PWSH.com", None),
            ("pwshx", None),
            // #2671: off Windows the reader knows no `\` path.
            (r"C:\PS\pwsh.exe", if cfg!(windows) { None } else { Some("pwsh.exe") }),
            (r"\\?\C:\PS\PWSH.EXE", host(r"\\?\C:\PS\pwsh.exe", "pwsh.exe")),
            (r"\\?\C:\PS\PWSH.EXE.", None),
            (r"\\?\C:\PS\pwsh.exe ", None),
            (r"\\?\C:\PS\PWSH", None),
            ("bash", None),
        ] {
            let command = words(&[program, "-NoProfile", "-Command", "git push"]);
            let expected =
                expected.map(|shell| words(&[shell, "-NoProfile", "-Command", "git push"]));
            assert_eq!(
                respelled(&command, NameCase::Windows),
                expected,
                "{program}"
            );
            for case in [NameCase::Sensitive, NameCase::Insensitive] {
                assert_eq!(respelled(&command, case), None, "{program} {case:?}");
            }
        }
        assert_eq!(respelled(&[], NameCase::Windows), None);
    }
}
