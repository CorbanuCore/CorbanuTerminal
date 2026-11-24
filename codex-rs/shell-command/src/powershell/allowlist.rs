//! D729 (A), #2661 (sync port of #2709): which PowerShell commands a literal
//! script may hold and still be read as its commands, where a prompt or
//! forbidden rule exists (D733).
//!
//! Main asks a trusted PowerShell (`powershell_allowlist.ps1`). Sync runs no
//! PowerShell before approval: it lowers scripts in-process, also for a
//! Windows executor seen from another host (#2671). So the same rules are
//! applied here to the lowered words, with main's tables for each edition
//! dumped from clean Windows PowerShell 5.1 and PowerShell 7.7 sessions
//! (`allowlist/dump_tables.ps1`). A script is read as its commands only when
//! every command is
//!   (1) a cmdlet on the allowlist (by name, a default alias, or qualified by
//!       its own system module) whose parameters all resolve, none excluded,
//!       and (for a module's cmdlet) that no program on the command's `PATH`
//!       shadows, or
//!   (2) a native program found on the command's `PATH`, whose argv
//!       PowerShell passes as written,
//! and no profile script may run first. Anything else is read in part: its
//! literal commands still add prompts and refusals, and its remainder asks.
//!
//! Where sync differs from main, it reads in part where main may not:
//! parameters are resolved by name only (no static binding); a cmdlet whose
//! excluded parameter can be given by position (`Receive-Job -ComputerName`)
//! takes no value by position; a bare word starting with `--`, or a
//! parameter-like word with a backtick, asks; and a command for another host
//! assumes a profile.

use std::collections::HashMap;
use std::collections::HashSet;
use std::sync::OnceLock;

use serde::Deserialize;

use super::PowershellKind;
use super::command_scope;
use crate::command_safety::LoweredWord;

/// Why `commands`, the lowered script of `shell` (a PowerShell of `kind`,
/// started with `-NoProfile` or not), is not read as its commands; `None`
/// when it is.
pub(crate) fn review(
    shell: &str,
    kind: PowershellKind,
    no_profile: bool,
    commands: &[Vec<LoweredWord>],
) -> Option<String> {
    if !no_profile && command_scope::profile_may_exist(shell, kind) {
        return Some("a profile script may run first".to_string());
    }
    let Some(tables) = tables(kind) else {
        return Some("allowlist tables unavailable".to_string());
    };
    let mut programs = Vec::new();
    let mut cmdlets = Vec::new();
    for command in commands {
        if let Some(cause) = command_cause(tables, command, &mut programs, &mut cmdlets) {
            return Some(cause);
        }
    }
    if !command_scope::names_resolve(&programs, &cmdlets) {
        return Some("a name is not a program on PATH, or a program shadows a cmdlet".to_string());
    }
    None
}

#[derive(Deserialize)]
struct RawTables {
    aliases: HashMap<String, String>,
    functions: Vec<String>,
    module_names: Vec<String>,
    banned: Vec<String>,
    action_values: Vec<String>,
    system_modules: Vec<String>,
    allow: HashMap<String, RawCmdlet>,
}

#[derive(Deserialize)]
struct RawCmdlet {
    module: String,
    excluded: Vec<String>,
    dynamic: HashMap<String, u8>,
    params: Vec<String>,
}

/// Main's tables for one edition, names lower case.
struct Tables {
    aliases: HashMap<String, String>,
    functions: HashSet<String>,
    module_names: HashSet<String>,
    action_values: HashSet<String>,
    system_modules: HashSet<String>,
    allow: HashMap<String, Cmdlet>,
}

/// An allowlisted cmdlet's parameters, canonical names lower case.
struct Cmdlet {
    /// From another module than PowerShell's core: a program or script of
    /// its name on `PATH` comes first.
    module_cmdlet: bool,
    /// Each name and alias, to the canonical name.
    names: HashMap<String, String>,
    switches: HashSet<String>,
    /// Banned for every cmdlet, or excluded for this one.
    excluded: HashSet<String>,
    /// File-system dynamic parameters: whether each takes a value.
    dynamic: HashMap<String, bool>,
    /// An excluded parameter may take a value given by position.
    excluded_positional: bool,
    /// #2756: an excluded parameter may bind input objects from the
    /// pipeline, by value or by property name (`-ComputerName` of 5.1's
    /// `Get-Process`, `-Credential` of `Get-Content`).
    excluded_pipeline: bool,
}

/// The common parameters whose value decides what happens on an error or a
/// message.
const ACTION_PARAMETERS: &[&str] = &[
    "erroraction",
    "warningaction",
    "informationaction",
    "progressaction",
];

fn tables(kind: PowershellKind) -> Option<&'static Tables> {
    static DESKTOP: OnceLock<Option<Tables>> = OnceLock::new();
    static CORE: OnceLock<Option<Tables>> = OnceLock::new();
    match kind {
        PowershellKind::WindowsPowershell => DESKTOP
            .get_or_init(|| load(include_str!("allowlist/desktop.json")))
            .as_ref(),
        PowershellKind::Pwsh => CORE
            .get_or_init(|| load(include_str!("allowlist/core.json")))
            .as_ref(),
    }
}

fn load(json: &str) -> Option<Tables> {
    let raw: RawTables = serde_json::from_str(json).ok()?;
    let lower = |names: Vec<String>| -> HashSet<String> {
        names.into_iter().map(|name| name.to_lowercase()).collect()
    };
    let banned = lower(raw.banned);
    let mut allow = HashMap::new();
    for (name, cmdlet) in raw.allow {
        let mut excluded = lower(cmdlet.excluded);
        excluded.extend(banned.iter().cloned());
        let mut names = HashMap::new();
        let mut switches = HashSet::new();
        let mut excluded_positional = false;
        let mut excluded_pipeline = false;
        for param in cmdlet.params {
            let mut fields = param.split('|');
            let (Some(canonical), Some(aliases), Some(flags), None) =
                (fields.next(), fields.next(), fields.next(), fields.next())
            else {
                return None;
            };
            let canonical = canonical.to_lowercase();
            if flags.contains('s') {
                switches.insert(canonical.clone());
            }
            if flags.contains('p') && excluded.contains(&canonical) {
                excluded_positional = true;
            }
            if flags.contains('i') && excluded.contains(&canonical) {
                excluded_pipeline = true;
            }
            for alias in aliases.split(',').filter(|alias| !alias.is_empty()) {
                names.insert(alias.to_lowercase(), canonical.clone());
            }
            names.insert(canonical.clone(), canonical);
        }
        let mut dynamic = HashMap::new();
        for (param, takes_value) in cmdlet.dynamic {
            let canonical = param.to_lowercase();
            names
                .entry(canonical.clone())
                .or_insert_with(|| canonical.clone());
            dynamic.insert(canonical, takes_value == 1);
        }
        allow.insert(
            name.to_lowercase(),
            Cmdlet {
                module_cmdlet: !cmdlet.module.is_empty()
                    && !cmdlet
                        .module
                        .eq_ignore_ascii_case("Microsoft.PowerShell.Core"),
                names,
                switches,
                excluded,
                dynamic,
                excluded_positional,
                excluded_pipeline,
            },
        );
    }
    Some(Tables {
        aliases: raw
            .aliases
            .into_iter()
            .map(|(name, target)| (name.to_lowercase(), target.to_lowercase()))
            .collect(),
        functions: lower(raw.functions),
        module_names: lower(raw.module_names),
        action_values: lower(raw.action_values),
        system_modules: lower(raw.system_modules),
        allow,
    })
}

/// Main's `Get-D729CommandCause`: why `command` is not read as plain; adds
/// the program names and module cmdlets the command's `PATH` must decide.
fn command_cause(
    tables: &Tables,
    command: &[LoweredWord],
    programs: &mut Vec<String>,
    cmdlets: &mut Vec<String>,
) -> Option<String> {
    let (name, args) = command.split_first()?;
    // A quoted first word is an expression to PowerShell, not a command.
    if !name.bare {
        return Some(format!("command name is not a bare word: {}", name.text));
    }
    // PowerShell may fold a non-ASCII name onto an ASCII alias or cmdlet
    // (`ıex`), which these tables would not find.
    if !name.text.is_ascii() {
        return Some(format!("command name is not ASCII: {}", name.text));
    }
    let lower = name.text.to_lowercase();
    if lower.ends_with(['.', ' ']) {
        return Some(format!(
            "command name ends in a dot or space: {}",
            name.text
        ));
    }
    if lower.contains(['\\', '/']) {
        // Qualified by a system module (`Microsoft.PowerShell.Management\Get-Item`), or a path.
        if let Some((module, rest)) = lower.split_once(['\\', '/'])
            && tables.system_modules.contains(module)
            && !rest.contains(['\\', '/'])
            && let Some(cmdlet) = tables.allow.get(rest)
        {
            return cmdlet_cause(tables, cmdlet, rest, name.piped, args);
        }
        if lower.ends_with(".exe") || lower.ends_with(".com") {
            return native_cause(args);
        }
        return Some(format!("script or module-qualified command: {}", name.text));
    }
    if lower.contains(['*', '?', '[', ']', '`']) {
        return Some(format!("command name holds a wildcard: {}", name.text));
    }
    let target = tables.aliases.get(&lower).unwrap_or(&lower);
    if let Some(cmdlet) = tables.allow.get(target) {
        // PowerShell finds a module's command it has not loaded yet only
        // after a program or script of that name on `PATH`.
        if cmdlet.module_cmdlet {
            cmdlets.push(target.clone());
        }
        return cmdlet_cause(tables, cmdlet, target, name.piped, args);
    }
    if tables.aliases.contains_key(&lower) || tables.functions.contains(&lower) {
        return Some(format!("command not on the allowlist: {target}"));
    }
    if let Some((_, extension)) = lower.rsplit_once('.')
        && !matches!(extension, "exe" | "com" | "cmd" | "bat")
    {
        return Some(format!("script file: {}", name.text));
    }
    if tables.module_names.contains(&lower) {
        return Some(format!("command not on the allowlist: {lower}"));
    }
    // A Verb-Noun name not found as a program on `PATH` is a module's command.
    if verb_noun(&lower) && !command_scope::program_on_path(&lower) {
        return Some(format!("command not on the allowlist: {lower}"));
    }
    // Any other name must be a program on the command's `PATH`, or it may be a
    // module's command or alias, or `Get-<name>`, loaded on use.
    programs.push(lower);
    native_cause(args)
}

/// `^[a-z]+-[a-z]`, as main matches a lower-case name.
fn verb_noun(lower: &str) -> bool {
    lower.split_once('-').is_some_and(|(verb, noun)| {
        !verb.is_empty()
            && verb.chars().all(|ch| ch.is_ascii_lowercase())
            && noun.starts_with(|ch: char| ch.is_ascii_lowercase())
    })
}

/// A network path (`\\host\share`, `//host`) reaches another computer with
/// the user's credentials. Any text holding `::` (a provider qualifier) or
/// starting with `\\` or `//` (also device paths) counts, and (#2756) a
/// drive-letter path on a drive that is not a local disk ([`remote_drive`]).
fn network_path(text: &str) -> bool {
    let mut start = text.trim_start().chars();
    text.contains("::")
        || matches!(
            (start.next(), start.next()),
            (Some('\\' | '/'), Some('\\' | '/'))
        )
        || remote_drive(text)
}

/// #2756 (main's `Test-D729RemoteDrive`): a drive-letter path (`Z:\s`,
/// `Z:s`) on a drive that is not a local disk now: a mapped network drive,
/// or a letter an earlier command may map (`net use`, `subst`). Not covered:
/// a link (`mklink /d`) to a network path, which a path or a recursive
/// listing may pass through.
fn remote_drive(text: &str) -> bool {
    let mut start = text.trim_start().chars();
    match (start.next(), start.next()) {
        (Some(letter), Some(':')) if letter.is_ascii_alphabetic() => !local_drive(letter),
        _ => false,
    }
}

/// Whether drive `letter` is a local disk of this host (removable, fixed,
/// CD-ROM or RAM disk), as `GetDriveTypeW` says without reaching it.
#[cfg(windows)]
fn local_drive(letter: char) -> bool {
    #[link(name = "kernel32")]
    unsafe extern "system" {
        fn GetDriveTypeW(root_path_name: *const u16) -> u32;
    }
    let root: Vec<u16> = format!("{letter}:\\")
        .encode_utf16()
        .chain(std::iter::once(0))
        .collect();
    // SAFETY: `root` is a NUL-terminated UTF-16 string that outlives the call,
    // which only reads it.
    let kind = unsafe { GetDriveTypeW(root.as_ptr()) };
    // DRIVE_REMOVABLE, DRIVE_FIXED, DRIVE_CDROM, DRIVE_RAMDISK.
    matches!(kind, 2 | 3 | 5 | 6)
}

/// Off Windows no drive letter names a disk of this host. A command for a
/// Windows executor checked here runs elsewhere, so it finds no names: its
/// path cmdlets (module cmdlets) are read in part anyway.
#[cfg(not(windows))]
fn local_drive(_letter: char) -> bool {
    true
}

/// Main's `Get-D729NativeCause`: why a native program's argv may not be what
/// PowerShell passes.
fn native_cause(args: &[LoweredWord]) -> Option<String> {
    for arg in args {
        let text = &arg.text;
        if text.is_empty() {
            return Some("empty native argument".to_string());
        }
        if text.contains('"') {
            return Some("native argument holds a double quote".to_string());
        }
        if text.contains(char::is_whitespace) && text.ends_with('\\') {
            return Some("native argument with a space ends in a backslash".to_string());
        }
    }
    None
}

/// How PowerShell reads a word of a cmdlet's command.
enum Token<'a> {
    /// A parameter name (`-Path`), without its dash.
    Parameter(&'a str),
    /// An argument.
    Argument,
    /// A spelling this reader does not tell apart.
    Unknown,
}

fn token(word: &LoweredWord) -> Token<'_> {
    if !word.bare {
        return Token::Argument;
    }
    let Some(name) = word.text.strip_prefix('-') else {
        return Token::Argument;
    };
    // `--` ends the parameters; `--x` and escaped names are not read here.
    if name.starts_with('-') || (word.escaped && !name.is_empty()) {
        return Token::Unknown;
    }
    if name.starts_with(|ch: char| ch.is_alphabetic() || ch == '_' || ch == '?') {
        Token::Parameter(name)
    } else {
        Token::Argument
    }
}

/// Main's `Resolve-D729Parameter`: the canonical parameter `name` names, if
/// exactly one. Exact names and aliases win; otherwise a unique prefix.
fn resolve<'a>(cmdlet: &'a Cmdlet, name: &str) -> Option<&'a str> {
    let lower = name.to_lowercase();
    if let Some(canonical) = cmdlet.names.get(&lower) {
        return Some(canonical);
    }
    let mut found: Option<&str> = None;
    for (key, canonical) in &cmdlet.names {
        if key.starts_with(&lower) {
            match found {
                Some(other) if other != canonical => return None,
                _ => found = Some(canonical),
            }
        }
    }
    found
}

/// Main's `Get-D729CmdletCause`: why an allowlisted cmdlet is not read as
/// plain.
fn cmdlet_cause(
    tables: &Tables,
    cmdlet: &Cmdlet,
    target: &str,
    piped: bool,
    args: &[LoweredWord],
) -> Option<String> {
    // #2756: past the first command of a pipeline, input objects bind to
    // parameters too (`Import-Csv hosts.csv | Get-Process` binds
    // -ComputerName in 5.1): no excluded parameter may take pipeline input.
    if piped && cmdlet.excluded_pipeline {
        return Some(format!(
            "an excluded parameter takes pipeline input: {target}"
        ));
    }
    let mut positional = 0usize;
    let mut words = args.iter();
    while let Some(word) = words.next() {
        if network_path(&word.text) {
            return Some(format!("network path: {target}"));
        }
        let name = match token(word) {
            Token::Argument => {
                positional += 1;
                continue;
            }
            Token::Unknown => return Some(format!("unread parameter {}: {target}", word.text)),
            Token::Parameter(name) => name,
        };
        let Some(canonical) = resolve(cmdlet, name) else {
            return Some(format!("unknown or ambiguous parameter -{name}: {target}"));
        };
        if cmdlet.excluded.contains(canonical) {
            return Some(format!("excluded parameter -{canonical}: {target}"));
        }
        let takes_value = match cmdlet.dynamic.get(canonical) {
            Some(takes_value) => *takes_value,
            None => !cmdlet.switches.contains(canonical),
        };
        if !takes_value {
            continue;
        }
        // Its value is the next word, an argument.
        let Some(value) = words.next() else {
            return Some(format!("parameter without a value -{canonical}: {target}"));
        };
        if network_path(&value.text) {
            return Some(format!("network path: {target}"));
        }
        if !matches!(token(value), Token::Argument) {
            return Some(format!("parameter without a value -{canonical}: {target}"));
        }
        if ACTION_PARAMETERS.contains(&canonical)
            && !tables.action_values.contains(&value.text.to_lowercase())
        {
            return Some(format!("excluded parameter value -{canonical}: {target}"));
        }
    }
    if positional > 0 && cmdlet.excluded_positional {
        return Some(format!(
            "a value by position may bind to an excluded parameter: {target}"
        ));
    }
    None
}

#[cfg(test)]
#[path = "allowlist_tests.rs"]
mod tests;
