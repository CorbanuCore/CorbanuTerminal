//! #2568 (D553): read a PowerShell command line the way PowerShell reads it
//! (`CommandLineParameterParser` in PowerShell 7.5, and in Windows
//! PowerShell 5.1, which PowerShell 6.0 started from). Anything this reader
//! cannot be sure of reads as [`Run::Unreadable`], so rules ask instead of
//! approving a script PowerShell would read differently.

use std::borrow::Cow;

use base64::Engine as _;

use super::PowershellKind;

/// What a PowerShell command line runs.
#[derive(Debug, PartialEq, Eq)]
pub(crate) enum Run<'a> {
    /// `-Command` (every remaining argument, joined with spaces, as
    /// PowerShell joins them), or `-CommandWithArgs` given no arguments.
    /// `start` is the index of the script's first argument.
    Command { start: usize, script: Cow<'a, str> },
    /// `-EncodedCommand`, decoded.
    Encoded(String),
    /// `-File` (and PowerShell 7's first positional argument): the file and
    /// its arguments. Its content is not read.
    File(&'a [String]),
    /// PowerShell runs code this reader cannot determine: an unknown or
    /// unsupported parameter, a script read from stdin, `-CommandWithArgs`
    /// given arguments, an `-EncodedCommand` that does not decode.
    Unreadable,
    /// No script: an interactive shell, as `pwsh` alone.
    Interactive,
}

/// A PowerShell command line as read by [`read`].
#[derive(Debug, PartialEq, Eq)]
pub(crate) struct Invocation<'a> {
    pub(crate) run: Run<'a>,
    /// `-Command` with no other parameter than `-NoLogo`, `-NoProfile`,
    /// `-NonInteractive`, `-STA` and `-MTA`.
    pub(crate) plain: bool,
}

/// PowerShell's command-line parameters, as far as this reader tells them
/// apart.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Param {
    Command,
    CommandWithArgs,
    EncodedCommand,
    File,
    /// No value, no effect on what runs, and plain.
    PlainSwitch,
    /// No value, no effect on what runs (`-NoProfileLoadTime`).
    Switch,
    /// Takes one value with no effect on which code runs
    /// (`-ExecutionPolicy`, `-WindowStyle`, `-InputFormat`,
    /// `-OutputFormat`, `-WorkingDirectory`).
    Value,
    /// Any other parameter PowerShell knows (`-NoExit`, `-Version`,
    /// `-ConfigurationName`, `-SettingsFile`, …): not supported here.
    Other,
}

/// `(name, shortest accepted prefix, parameter)` in the order PowerShell
/// 7.5 tries them: a prefix of `name` at least as long as the shortest
/// matches, and the first match wins.
const PWSH_PARAMS: &[(&str, &str, Param)] = &[
    ("version", "v", Param::Other),
    ("help", "h", Param::Other),
    ("?", "?", Param::Other),
    ("login", "l", Param::Other),
    ("noexit", "noe", Param::Other),
    ("noprofile", "nop", Param::PlainSwitch),
    ("nologo", "nol", Param::PlainSwitch),
    ("noninteractive", "noni", Param::PlainSwitch),
    ("socketservermode", "so", Param::Other),
    ("servermode", "s", Param::Other),
    ("namedpipeservermode", "nam", Param::Other),
    ("sshservermode", "sshs", Param::Other),
    ("noprofileloadtime", "noprofileloadtime", Param::Switch),
    ("interactive", "i", Param::Other),
    ("configurationfile", "configurationfile", Param::Other),
    ("configurationname", "config", Param::Other),
    ("custompipename", "cus", Param::Other),
    ("commandwithargs", "commandwithargs", Param::CommandWithArgs),
    ("cwa", "cwa", Param::CommandWithArgs),
    ("command", "c", Param::Command),
    ("windowstyle", "w", Param::Value),
    ("file", "f", Param::File),
    ("isswait", "isswait", Param::Other),
    ("outputformat", "o", Param::Value),
    ("of", "o", Param::Value),
    ("inputformat", "inp", Param::Value),
    ("if", "if", Param::Value),
    ("executionpolicy", "ex", Param::Value),
    ("ep", "ep", Param::Value),
    ("encodedcommand", "e", Param::EncodedCommand),
    ("ec", "e", Param::EncodedCommand),
    ("encodedarguments", "encodeda", Param::Other),
    ("ea", "ea", Param::Other),
    ("settingsfile", "settings", Param::Other),
    ("sta", "sta", Param::PlainSwitch),
    ("mta", "mta", Param::PlainSwitch),
    ("workingdirectory", "wo", Param::Value),
    ("wd", "wd", Param::Value),
    (
        "removeworkingdirectorytrailingcharacter",
        "removeworkingdirectorytrailingcharacter",
        Param::Other,
    ),
];

/// The same for Windows PowerShell 5.1 (as PowerShell 6.0 inherited it,
/// plus `-PSConsoleFile` and `-Version`, which 6.0 dropped). A
/// `powershell` command line reads only where this and [`PWSH_PARAMS`]
/// agree, so a `powershell` that is PowerShell 7 reads the same.
const WINDOWS_POWERSHELL_PARAMS: &[(&str, &str, Param)] = &[
    ("psconsolefile", "psc", Param::Other),
    ("version", "v", Param::Other),
    ("help", "h", Param::Other),
    ("?", "?", Param::Other),
    ("noexit", "noe", Param::Other),
    ("importsystemmodules", "imp", Param::Other),
    ("noprofile", "nop", Param::PlainSwitch),
    ("nologo", "nol", Param::PlainSwitch),
    ("noninteractive", "noni", Param::PlainSwitch),
    ("socketservermode", "so", Param::Other),
    ("servermode", "s", Param::Other),
    ("namedpipeservermode", "nam", Param::Other),
    ("sshservermode", "sshs", Param::Other),
    ("configurationname", "config", Param::Other),
    ("command", "c", Param::Command),
    ("windowstyle", "w", Param::Value),
    ("file", "f", Param::File),
    ("wait", "w", Param::Other),
    ("iss", "iss", Param::Other),
    ("isswait", "isswait", Param::Other),
    ("modules", "mod", Param::Other),
    ("outputformat", "o", Param::Value),
    ("of", "o", Param::Value),
    ("inputformat", "i", Param::Value),
    ("if", "i", Param::Value),
    ("executionpolicy", "ex", Param::Value),
    ("ep", "ep", Param::Value),
    ("encodedcommand", "e", Param::EncodedCommand),
    ("ec", "e", Param::EncodedCommand),
    ("encodedarguments", "encodeda", Param::Other),
    ("ea", "ea", Param::Other),
    ("sta", "s", Param::PlainSwitch),
    ("mta", "mta", Param::PlainSwitch),
];

/// PowerShell's dashes: `-`, en dash, em dash, horizontal bar.
fn is_dash(c: char) -> bool {
    matches!(c, '-' | '\u{2013}' | '\u{2014}' | '\u{2015}')
}

/// The white space every PowerShell trims from a parameter.
fn is_ascii_space(c: char) -> bool {
    matches!(c, ' ' | '\t' | '\n' | '\u{b}' | '\u{c}' | '\r')
}

/// The parameter `switch` (`-c`, `/Command`, `--nop`) names in `params`;
/// `None` where it names none. `doubled_dash_any` is Windows PowerShell's
/// rule that drops any dash after the first character (`-—c`, `/-c`);
/// PowerShell 7 drops only a second dash equal to the first (`--c`).
fn lookup(switch: &str, params: &[(&str, &str, Param)], doubled_dash_any: bool) -> Option<Param> {
    let mut chars = switch.chars();
    let first = chars.next()?;
    if !is_dash(first) && first != '/' {
        return None;
    }
    let mut key = chars.as_str();
    if let Some(second) = key.chars().next()
        && ((is_dash(first) && second == first) || (doubled_dash_any && is_dash(second)))
    {
        key = &key[second.len_utf8()..];
    }
    // Other letters compare case-insensitively in ways that differ between
    // .NET versions (`ı`, `ſ`, `K`); read only ASCII names.
    if key.is_empty() || !key.is_ascii() {
        return None;
    }
    let key = key.to_ascii_lowercase();
    params
        .iter()
        .find(|(name, shortest, _)| key.len() >= shortest.len() && name.starts_with(&key))
        .map(|(_, _, param)| *param)
}

/// The parameter `switch` names for a PowerShell of `kind`; `None` where
/// this reader cannot be sure (unknown, or read differently by the two
/// PowerShells a `powershell` may be).
fn param(switch: &str, kind: PowershellKind) -> Option<Param> {
    let pwsh = lookup(switch, PWSH_PARAMS, /*doubled_dash_any*/ false)?;
    match kind {
        PowershellKind::Pwsh => Some(pwsh),
        PowershellKind::WindowsPowershell => {
            let windows = lookup(
                switch,
                WINDOWS_POWERSHELL_PARAMS,
                /*doubled_dash_any*/ true,
            )?;
            (windows == pwsh).then_some(pwsh)
        }
    }
}

/// `-EncodedCommand`'s value decoded as PowerShell does (base64 of
/// UTF-16LE); `None` where it does not decode exactly.
fn decode(value: &str) -> Option<String> {
    let bytes = base64::engine::general_purpose::STANDARD
        .decode(value)
        .ok()?;
    if bytes.len() % 2 != 0 {
        return None;
    }
    let units: Vec<u16> = bytes
        .as_chunks::<2>()
        .0
        .iter()
        .map(|pair| u16::from_le_bytes(*pair))
        .collect();
    String::from_utf16(&units).ok()
}

/// Read `command`, a PowerShell of `kind` (`command[0]`) and its arguments.
pub(crate) fn read(command: &[String], kind: PowershellKind) -> Invocation<'_> {
    let mut plain = true;
    let mut encoded = None;
    let unreadable = Invocation {
        run: Run::Unreadable,
        plain: false,
    };
    let mut i = 1;
    while i < command.len() {
        let arg = command[i].trim_matches(is_ascii_space);
        if arg.is_empty() {
            // Every PowerShell skips an empty parameter.
            i += 1;
            continue;
        }
        if arg.starts_with(char::is_whitespace) || arg.ends_with(char::is_whitespace) {
            // Other white space: PowerShells trim different sets.
            return unreadable;
        }
        let first = arg.chars().next().unwrap_or_default();
        if !is_dash(first) && first != '/' {
            // PowerShell 7 runs this file; Windows PowerShell reads the rest
            // as a script, but a `powershell` may be PowerShell 7.
            return match kind {
                PowershellKind::Pwsh if encoded.is_none() => Invocation {
                    run: Run::File(&command[i..]),
                    plain: false,
                },
                _ => unreadable,
            };
        }
        if encoded.is_some() {
            // Only parameters with no effect on what runs may follow.
            match param(arg, kind) {
                Some(Param::PlainSwitch | Param::Switch) => {}
                Some(Param::Value) if i + 1 < command.len() => i += 1,
                _ => return unreadable,
            }
            i += 1;
            continue;
        }
        let Some(param) = param(arg, kind) else {
            return unreadable;
        };
        match param {
            Param::PlainSwitch => {}
            Param::Switch => plain = false,
            Param::Value => {
                if i + 1 >= command.len() {
                    return unreadable;
                }
                plain = false;
                i += 1;
            }
            Param::Command | Param::CommandWithArgs => {
                let start = i + 1;
                let Some(first_word) = command.get(start) else {
                    return unreadable;
                };
                if first_word == "-" {
                    // The script comes from stdin.
                    return unreadable;
                }
                let script = if param == Param::CommandWithArgs {
                    if start + 1 != command.len() {
                        // The script's `$args`: code may run them.
                        return unreadable;
                    }
                    plain = false;
                    Cow::Borrowed(first_word.as_str())
                } else if start + 1 == command.len() {
                    Cow::Borrowed(first_word.as_str())
                } else {
                    Cow::Owned(command[start..].join(" "))
                };
                return Invocation {
                    run: Run::Command { start, script },
                    plain,
                };
            }
            Param::EncodedCommand => {
                let Some(script) = command.get(i + 1).and_then(|value| decode(value)) else {
                    return unreadable;
                };
                encoded = Some(script);
                i += 1;
            }
            Param::File => {
                return match command.get(i + 1) {
                    Some(file) if file != "-" => Invocation {
                        run: Run::File(&command[i + 1..]),
                        plain: false,
                    },
                    // No file, or the script comes from stdin.
                    _ => unreadable,
                };
            }
            Param::Other => return unreadable,
        }
        i += 1;
    }
    match encoded {
        Some(script) => Invocation {
            run: Run::Encoded(script),
            plain: false,
        },
        None => Invocation {
            run: Run::Interactive,
            plain,
        },
    }
}

#[cfg(test)]
#[path = "cli_tests.rs"]
mod tests;
