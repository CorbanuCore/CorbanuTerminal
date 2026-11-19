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
//!   exactly the caller's text, as if typed. `/D` skips registry AutoRun
//!   commands and `/V:OFF` keeps `!` literal unless the caller chose `/V`.
//! - Several elements are a program and its arguments. Each is CRT-quoted and
//!   every cmd metacharacter is escaped with `^`, so no quote ever opens for
//!   cmd, no operator or redirection takes effect, no `%VAR%` or `!VAR!` is
//!   expanded, and the program receives each argument byte for byte.
//!
//! Anything this can't serialize faithfully is refused rather than guessed.

use anyhow::Result;
use anyhow::bail;
use std::collections::HashMap;

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
/// (see the module docs). `env` is the child's environment.
pub(crate) fn sandbox_command_line(
    argv: &[String],
    env: &HashMap<String, String>,
) -> Result<String> {
    match cmd_invocation(argv)? {
        Some(cmd) => cmd.serialize(argv, env),
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

fn is_cmd_program(program: &str) -> bool {
    let name = program.rsplit(['\\', '/']).next().unwrap_or(program);
    name.eq_ignore_ascii_case("cmd") || name.eq_ignore_ascii_case("cmd.exe")
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
    fn serialize(&self, argv: &[String], env: &HashMap<String, String>) -> Result<String> {
        // A NUL would end the command line early, and with it the closing
        // quote `/S` relies on.
        if argv.iter().any(|arg| arg.contains('\0')) {
            bail!("a cmd.exe command line can't contain NUL (#2702)");
        }
        let tail = &argv[self.command_switch + 1..];
        let line = match tail {
            [] => String::new(),
            [line] => line.clone(),
            [program, args @ ..] => self.escaped_program_line(program, args, env)?,
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
    ) -> Result<String> {
        if self.delayed_expansion == Some(true) {
            bail!("cmd.exe /V:ON can't pass arguments through unchanged (#2702)");
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
        // The program token is quoted unless it is a plain name: cmd treats
        // a leading `@` `:` `(` and the delimiters `,;=` there specially, and
        // a `%` must be followed by a character of its own argument.
        let plain_program = program
            .chars()
            .next()
            .is_some_and(|c| c.is_alphanumeric() || matches!(c, '\\' | '.' | '_'))
            && !program.contains([' ', '\t', '"', ',', ';', '=', '%']);
        let mut quoted = vec![quote_crt(program, !plain_program)];
        quoted.extend(args.iter().map(|arg| quote_crt(arg, arg.contains('%'))));
        Ok(caret_escape(&quoted.join(" ")))
    }
}

/// Escapes CRT-quoted text so that cmd hands it on unchanged. Quotes are
/// escaped too, so cmd never enters a quoted region and every caret applies.
/// A `%` is followed by a caret, which escapes the next character: percent
/// expansion runs before caret removal and must never see a variable name it
/// could resolve. Callers make sure each `%` has a following character in
/// its own argument (a closing quote at least).
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
