//! Command lines for `CreateProcess*` (#2702, owner decision D730).
//!
//! Ordinary programs split their command line with the CRT rules, so each
//! argument is quoted with [`quote_crt_arg`]. `cmd.exe` doesn't: after `/C`
//! (or `/K`, `/R`) it re-parses the rest of its command line with its own
//! rules, where `\"` is not an escaped quote and `& | < > ( ) ^ % !` have
//! meaning. CRT quoting there broke commands containing quotes and could
//! change what they meant, so a `cmd.exe` invocation is serialized here with
//! cmd's rules:
//!
//! - One element after the switch is a command line for cmd (the core shell
//!   contract, `cmd /c <script>`). It is passed verbatim as
//!   `/d /v:off /s /c "<line>"`: with `/S` cmd removes only the first and the
//!   last quote character, which are the ones added here, so cmd parses
//!   exactly the caller's text, as if typed (so a script path with spaces
//!   must be quoted, as when typed). `/D` skips registry AutoRun commands and
//!   `/V:OFF` keeps `!` literal unless the caller chose `/V`.
//! - Several elements are a program and its arguments. Each is CRT-quoted and
//!   every cmd metacharacter is escaped with `^`, so no quote ever opens for
//!   cmd, no operator or redirection takes effect, no `%VAR%` or `!VAR!` is
//!   expanded, and the program receives each argument byte for byte.
//!
//! - A batch file started directly also runs through cmd, on the CRT-joined
//!   line, so its arguments may not contain anything cmd acts on.
//!
//! Anything this can't serialize faithfully is refused rather than guessed,
//! including `CALL` as the program, which expands and re-parses its
//! arguments once more.
//!
//! - A program that hands its arguments to cmd again re-parses them after
//!   cmd removed the escapes (#2718): a nested `cmd`, the `start`, `if` and
//!   `for` builtins, and a name cmd resolves through PATHEXT to a batch file
//!   (`npm` for `npm.cmd`). With separate arguments, such a program is
//!   refused when an argument contains anything cmd acts on, and so is any
//!   program token that isn't a plain path, from which cmd might cut such a
//!   name. Other programs that start a shell on their arguments (PowerShell,
//!   `runas`, ...) are not recognized.

use anyhow::Result;
use anyhow::bail;
use std::collections::HashMap;
use std::ffi::OsString;
use std::path::Path;
use std::path::PathBuf;

/// Characters cmd acts on when it parses a command line again.
const CMD_ACTIVE: [char; 13] = [
    '"', '%', '!', '^', '&', '|', '<', '>', '(', ')', '\r', '\n', '\0',
];

/// Quote a single Windows command-line argument following the rules used by
/// CommandLineToArgvW/CRT so that spaces, quotes, and backslashes are preserved.
/// Reference behavior matches Rust std::process::Command on Windows.
pub(crate) fn quote_crt_arg(arg: &str) -> String {
    quote_crt(arg, /*force*/ false)
}

fn quote_crt(arg: &str, force: bool) -> String {
    let needs_quotes = force
        || arg.is_empty()
        || arg
            .chars()
            .any(|c| matches!(c, ' ' | '\t' | '\n' | '\r' | '"'));
    if !needs_quotes {
        return arg.to_string();
    }

    let mut quoted = String::with_capacity(arg.len() + 2);
    quoted.push('"');
    let mut backslashes = 0;
    for ch in arg.chars() {
        match ch {
            '\\' => {
                backslashes += 1;
            }
            '"' => {
                quoted.push_str(&"\\".repeat(backslashes * 2 + 1));
                quoted.push('"');
                backslashes = 0;
            }
            _ => {
                if backslashes > 0 {
                    quoted.push_str(&"\\".repeat(backslashes));
                    backslashes = 0;
                }
                quoted.push(ch);
            }
        }
    }
    if backslashes > 0 {
        quoted.push_str(&"\\".repeat(backslashes * 2));
    }
    quoted.push('"');
    quoted
}

/// The command line `CreateProcess*` gets for `argv` in the sandbox: the CRT
/// join for ordinary programs, cmd's own rules for a `cmd.exe /C` invocation
/// (see the module docs). `env` is the child's environment and `cwd` its
/// working directory, where cmd looks for a program first.
pub(crate) fn sandbox_command_line(
    argv: &[String],
    env: &HashMap<String, String>,
    cwd: &Path,
) -> Result<String> {
    if argv
        .first()
        .is_some_and(|program| is_batch_program(program))
    {
        check_batch_arguments(argv)?;
    }
    match cmd_invocation(argv)? {
        Some(cmd) => cmd.serialize(argv, env, cwd),
        None => Ok(argv
            .iter()
            .map(|arg| quote_crt_arg(arg))
            .collect::<Vec<_>>()
            .join(" ")),
    }
}

struct CmdInvocation {
    /// Index of the `/C`, `/K` or `/R` switch in argv.
    command_switch: usize,
    has_d: bool,
    has_s: bool,
    /// `None` when the caller left delayed expansion to cmd's default.
    delayed_expansion: Option<bool>,
}

/// The file name Windows resolves `program` to, lowercased: without a drive
/// prefix, an alternate data stream (`::$DATA`) or the trailing dots and
/// spaces Windows ignores.
fn program_file_name(program: &str) -> String {
    let mut name = program.rsplit(['\\', '/']).next().unwrap_or(program);
    if name.as_bytes().get(1) == Some(&b':') && name.as_bytes()[0].is_ascii_alphabetic() {
        name = &name[2..];
    }
    let name = name.split(':').next().unwrap_or(name);
    name.trim_end_matches(['.', ' ']).to_ascii_lowercase()
}

fn is_cmd_program(program: &str) -> bool {
    matches!(program_file_name(program).as_str(), "cmd" | "cmd.exe")
}

/// A batch file: `CreateProcess*` and cmd run it through cmd, which parses
/// its arguments. Any `:`-separated part counts, since Windows looks at the
/// last `.` of a named stream (`x.txt:y.bat`) too. A name cmd resolves
/// through PATHEXT (`npm` for `npm.cmd`) isn't recognized by its spelling;
/// see [`resolves_to_batch_file`].
fn is_batch_program(program: &str) -> bool {
    let name = program.rsplit(['\\', '/']).next().unwrap_or(program);
    name.split(':').any(|part| {
        let part = part.trim_end_matches(['.', ' ']).to_ascii_lowercase();
        part.ends_with(".bat") || part.ends_with(".cmd")
    })
}

/// cmd parses a batch file's CRT-joined command line, and how Windows hands
/// it over can strip quotes, so any character cmd acts on is refused.
fn check_batch_arguments(argv: &[String]) -> Result<()> {
    if let Some(arg) = argv.iter().find(|arg| arg.contains(CMD_ACTIVE)) {
        bail!("a batch file argument can't contain cmd.exe metacharacters: {arg:?} (#2702)");
    }
    Ok(())
}

/// A program token cmd reads as one plain file name: ASCII letters and
/// digits, `_ - . \ :`, and spaces (a token with a space is quoted for cmd),
/// with no `.` or `..` component and no empty one after a leading `\`
/// prefix. cmd may split any other token into a command name and more
/// (#2718: `cmd.exe/c`, `cmd<VT>`, `npm+x`), so with an argument cmd acts on
/// such a program is refused, without modelling cmd's parser.
fn is_plain_program_token(program: &str) -> bool {
    program
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || matches!(c, '_' | '-' | '.' | '\\' | ':' | ' '))
        && !program
            .trim_start_matches('\\')
            .split('\\')
            .any(|component| component.trim_matches(['.', ' ']).is_empty())
}

/// A plain program token (see [`is_plain_program_token`]) that makes cmd
/// parse the following arguments as a command line again (#2718): a nested
/// `cmd`, or the `start`, `if` or `for` builtin, which run a command taken
/// from them (`if 1==1 cmd /c x a&b`). `call` is refused separately.
///
/// A builtin is the first non-empty part of the token, split at `\ : .` and
/// spaces (`start.`, `if\x`). The nested cmd is the file the last component
/// names, read as [`program_file_name`] reads it; a directory named `cmd`
/// (Git's `Git\cmd\git.exe`) doesn't count.
fn is_reparsing_program(program: &str) -> bool {
    let lower = program.to_ascii_lowercase();
    let builtin = lower
        .split(['\\', ':', '.', ' '])
        .find(|part| !part.is_empty())
        .unwrap_or_default();
    if matches!(builtin, "start" | "if" | "for") {
        return true;
    }
    matches!(program_file_name(&lower).trim_start(), "cmd" | "cmd.exe")
}

/// Whether cmd could resolve the plain program token `program` through
/// PATHEXT to a batch file: a `<program>.bat` or `<program>.cmd` (either
/// case, or as PATHEXT spells it) in the working directory or a directory on
/// any spelling of the child's PATH (for a bare name; each entry also as cmd
/// reads it, #2783), or next to the given
/// path. Any match counts, even one an earlier executable would shadow, and
/// these extensions are tried whatever PATHEXT says. A drive-relative
/// program (`C:npm`) is resolved by the child against its own directory on
/// that drive and always counts. Anything but a missing file counts too, so
/// a failure only refuses more (a PATH entry on a drive that isn't ready
/// refuses every bare name). A file created after the launch isn't seen.
///
/// The name is looked up as given: cmd doesn't trim trailing dots or spaces
/// before it tries the extensions, so `npm.`, `"npm "` and `"npm ."` don't
/// find `npm.cmd` (checked natively on Windows 11 build 26200, #2745). A
/// spelled-out extension with trailing dots or spaces (`"npm.cmd "`) runs
/// and is caught by [`is_batch_program`].
fn resolves_to_batch_file(program: &str, env: &HashMap<String, String>, cwd: &Path) -> bool {
    let bytes = program.as_bytes();
    if bytes.len() >= 2
        && bytes[0].is_ascii_alphabetic()
        && bytes[1] == b':'
        && !matches!(bytes.get(2), Some(b'\\' | b'/'))
    {
        return true;
    }
    // Each PATH (and PATHEXT) entry is looked up as before (split at every
    // `;`, trimmed, outer quotes dropped) and, when that differs, as cmd
    // reads it ([`cmd_list_entries`]). As above, any failure but a missing
    // file counts, except an invalid name in cmd's spelling (not `strict`).
    let values = |key: &str| {
        let mut spellings: Vec<(String, bool)> = Vec::new();
        for (_, value) in env
            .iter()
            .filter(|(name, _)| name.eq_ignore_ascii_case(key))
        {
            for entry in value.split(';') {
                let trimmed = entry.trim().trim_matches('"');
                if !trimmed.is_empty() {
                    spellings.push((trimmed.to_string(), true));
                }
            }
            for entry in cmd_list_entries(value) {
                if !entry.is_empty() && !spellings.iter().any(|(known, _)| *known == entry) {
                    spellings.push((entry, false));
                }
            }
        }
        spellings
    };
    let bases: Vec<(PathBuf, bool)> = if program.contains(['\\', '/', ':']) {
        vec![(cwd.join(program), true)]
    } else {
        std::iter::once((cwd.to_path_buf(), true))
            .chain(
                values("PATH")
                    .into_iter()
                    .map(|(dir, strict)| (cwd.join(dir), strict)),
            )
            .map(|(dir, strict)| (dir.join(program), strict))
            .collect()
    };
    let pathext = values("PATHEXT");
    let extensions: Vec<&str> = [".bat", ".cmd", ".BAT", ".CMD"]
        .into_iter()
        .chain(pathext.iter().map(|(ext, _)| ext.as_str()).filter(|ext| {
            let ext = ext.trim().to_ascii_lowercase();
            ext == ".bat" || ext == ".cmd"
        }))
        .collect();
    bases.iter().any(|(base, strict)| {
        extensions.iter().any(|ext| {
            let mut candidate = OsString::from(base.as_os_str());
            candidate.push(ext);
            match std::fs::metadata(&candidate) {
                Ok(metadata) => !metadata.is_dir(),
                // A spelling of cmd's no file can have (` C:\bin` joined to
                // the working directory) holds nothing cmd could run.
                Err(err) => {
                    !matches!(
                        err.kind(),
                        std::io::ErrorKind::NotFound | std::io::ErrorKind::NotADirectory
                    ) && (*strict || err.kind() != std::io::ErrorKind::InvalidFilename)
                }
            }
        })
    })
}

/// The entries of a PATH-style list as cmd reads them (checked natively on
/// Windows 11, #2783): a `;` inside quotes doesn't split, every quote is
/// dropped, and the spaces around an entry are kept (`"C:\bin" ` and
/// `C:\bin ` both search a folder named `bin `). An unclosed quote runs to
/// the end of the value (checked natively on build 26200, #2793): cmd
/// searches no entry after it, so `"C:\a;C:\bin ` doesn't search `bin `.
fn cmd_list_entries(value: &str) -> Vec<String> {
    let mut entries = vec![String::new()];
    let mut quoted = false;
    for ch in value.chars() {
        match ch {
            '"' => quoted = !quoted,
            ';' if !quoted => entries.push(String::new()),
            _ => {
                if let Some(entry) = entries.last_mut() {
                    entry.push(ch);
                }
            }
        }
    }
    entries
}

/// Recognizes `cmd[.exe] [/switch ...] /C|/K|/R [command ...]`. Returns `None`
/// for other programs and for a cmd without a command switch (it parses no
/// command line then). A cmd switch this can't classify is refused.
fn cmd_invocation(argv: &[String]) -> Result<Option<CmdInvocation>> {
    let Some(program) = argv.first() else {
        return Ok(None);
    };
    if !is_cmd_program(program) {
        return Ok(None);
    }
    let mut has_d = false;
    let mut has_s = false;
    let mut delayed_expansion = None;
    for (index, arg) in argv.iter().enumerate().skip(1) {
        let Some((letter, value)) = parse_cmd_switch(arg) else {
            // Not a plain switch: cmd would read the rest of its command line
            // in ways that can't be serialized for it faithfully.
            bail!("unsupported cmd.exe argument before /C: {arg:?} (#2702)");
        };
        match (letter.to_ascii_lowercase(), value) {
            ('c' | 'k' | 'r', None) => {
                return Ok(Some(CmdInvocation {
                    command_switch: index,
                    has_d,
                    has_s,
                    delayed_expansion,
                }));
            }
            ('d', None) => has_d = true,
            ('s', None) => has_s = true,
            ('v', None) => delayed_expansion = Some(true),
            ('v', Some(value)) if value.eq_ignore_ascii_case("on") => {
                delayed_expansion = Some(true);
            }
            ('v', Some(value)) if value.eq_ignore_ascii_case("off") => {
                delayed_expansion = Some(false);
            }
            ('c' | 'k' | 'r' | 'v', Some(_)) => {
                bail!("unsupported cmd.exe switch before /C: {arg:?} (#2702)");
            }
            _ => {}
        }
    }
    // No command switch: an interactive cmd, which parses no command line.
    Ok(None)
}

/// `/X` or `/X:value` with an alphanumeric value, as cmd's switches are.
fn parse_cmd_switch(arg: &str) -> Option<(char, Option<&str>)> {
    let rest = arg.strip_prefix('/')?;
    let mut chars = rest.chars();
    let letter = chars.next().filter(char::is_ascii_alphabetic)?;
    let tail = chars.as_str();
    if tail.is_empty() {
        return Some((letter, None));
    }
    let value = tail.strip_prefix(':')?;
    (!value.is_empty() && value.len() <= 8 && value.chars().all(|c| c.is_ascii_alphanumeric()))
        .then_some((letter, Some(value)))
}

impl CmdInvocation {
    fn serialize(
        &self,
        argv: &[String],
        env: &HashMap<String, String>,
        cwd: &Path,
    ) -> Result<String> {
        // A NUL would end the command line early, and with it the closing
        // quote `/S` relies on.
        if argv.iter().any(|arg| arg.contains('\0')) {
            bail!("a cmd.exe command line can't contain NUL (#2702)");
        }
        let tail = &argv[self.command_switch + 1..];
        let line = match tail {
            [] => String::new(),
            [line] => line.clone(),
            [program, args @ ..] => self.escaped_program_line(program, args, env, cwd)?,
        };
        let mut parts = vec![quote_crt_arg(&argv[0])];
        if !self.has_d {
            parts.push("/d".to_string());
        }
        parts.extend(argv[1..self.command_switch].iter().cloned());
        if self.delayed_expansion.is_none() {
            parts.push("/v:off".to_string());
        }
        if !self.has_s {
            parts.push("/s".to_string());
        }
        parts.push(argv[self.command_switch].clone());
        parts.push(format!("\"{line}\""));
        Ok(parts.join(" "))
    }

    /// A program and its arguments, each reaching the program unchanged.
    fn escaped_program_line(
        &self,
        program: &str,
        args: &[String],
        env: &HashMap<String, String>,
        cwd: &Path,
    ) -> Result<String> {
        if self.delayed_expansion == Some(true) {
            bail!("cmd.exe /V:ON can't pass arguments through unchanged (#2702)");
        }
        // cmd hands a batch file its arguments unescaped, and the batch file
        // parses them again (`%1`, `%*`).
        if is_batch_program(program) {
            check_batch_arguments(args)?;
        }
        // A nested cmd, `start`, `if`, `for`, or a batch file found through
        // PATHEXT gets the arguments with the escapes removed and parses
        // them again.
        if let Some(arg) = args.iter().find(|arg| arg.contains(CMD_ACTIVE)) {
            if !is_plain_program_token(program) {
                bail!(
                    "cmd.exe could read {program:?} as a program that would re-parse its arguments; only a plain path (ASCII letters, digits, `_ - . \\ :` and spaces, with no empty, `.` or `..` component) may take {arg:?} (#2718)"
                );
            }
            if is_reparsing_program(program) {
                bail!(
                    "a nested cmd.exe, START, IF or FOR would re-parse its arguments: {program:?} {arg:?} (#2718)"
                );
            }
            if resolves_to_batch_file(program, env, cwd) {
                bail!(
                    "{program:?} resolves to a batch file, whose arguments can't contain cmd.exe metacharacters: {arg:?} (#2718)"
                );
            }
        }
        let all = std::iter::once(program).chain(args.iter().map(String::as_str));
        if all.clone().any(|arg| arg.contains(['\r', '\n'])) {
            bail!("a cmd.exe argument can't contain a line break (#2702)");
        }
        // Every `%` is emitted as `%^`, so any `%name%` cmd could expand has
        // a name starting with `^`; no such variable may exist.
        if all.clone().any(|arg| arg.contains('%'))
            && let Some(name) = env.keys().find(|name| name.starts_with('^'))
        {
            bail!("cmd.exe variable {name:?} would defeat `%` escaping (#2702)");
        }
        // `call` runs a second expansion and quote pass over its arguments.
        let call = program
            .get(..4)
            .is_some_and(|head| head.eq_ignore_ascii_case("call"))
            && !program[4..]
                .chars()
                .next()
                .is_some_and(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-');
        if call {
            bail!("cmd.exe CALL would expand and re-parse its arguments (#2702)");
        }
        // cmd finds the program by its own rules: a caret-escaped quote
        // doesn't group a path with spaces there, so the program token gets
        // real quotes unless it is a plain name (cmd treats a leading `@`
        // `:` `(` and the delimiters `,;=` there specially). Inside those
        // quotes everything but `%` is literal, and paths hold no quotes.
        if program.contains(['"', '%']) {
            bail!("a cmd.exe program can't contain `\"` or `%`: {program:?} (#2702)");
        }
        let plain_program = program
            .chars()
            .next()
            .is_some_and(|c| c.is_alphanumeric() || matches!(c, '\\' | '.' | '_'))
            && !program.contains([' ', '\t', ',', ';', '=']);
        let program = if plain_program {
            caret_escape(program)
        } else {
            format!("\"{program}\"")
        };
        let args = args
            .iter()
            .map(|arg| caret_escape(&quote_crt(arg, arg.contains('%'))))
            .collect::<Vec<_>>();
        Ok(std::iter::once(program)
            .chain(args)
            .collect::<Vec<_>>()
            .join(" "))
    }
}

/// Escapes CRT-quoted text so that cmd hands it on unchanged. Quotes are
/// escaped too, so cmd never enters a quoted region and every caret applies.
/// A `%` is followed by a caret, which escapes the next character: percent
/// expansion runs before caret removal and must never see a variable name it
/// could resolve. Callers make sure each `%` has a following character in
/// its own argument (a closing quote at least), and escape each argument
/// separately.
fn caret_escape(text: &str) -> String {
    let mut out = String::with_capacity(text.len() * 2);
    let mut escaped_by_previous = false;
    for ch in text.chars() {
        if ch == '%' {
            out.push_str("%^");
            escaped_by_previous = true;
            continue;
        }
        if !escaped_by_previous && matches!(ch, '^' | '&' | '|' | '<' | '>' | '(' | ')' | '"' | '!')
        {
            out.push('^');
        }
        out.push(ch);
        escaped_by_previous = false;
    }
    out
}

#[cfg(test)]
#[path = "command_line_tests.rs"]
mod tests;
