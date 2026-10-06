//! Indirect routes to a protected surface the plain word scan misses:
//! string literals assembled by inline interpreter code, encoded payloads,
//! scripts run from a file, opaque decode-and-run pipelines and patches that
//! write commands into files that are run later.

use super::shell::SHELLS;
use super::shell::SimpleCommand;
use super::shell::basename;
use base64::Engine;

/// Interpreters that run a file argument or, without one, their stdin.
pub(super) const INTERPRETERS: &[&str] = &[
    "python",
    "python2",
    "python3",
    "node",
    "nodejs",
    "deno",
    "bun",
    "perl",
    "ruby",
    "php",
    "lua",
    "osascript",
    "tsx",
    "ts-node",
    "pwsh",
];
/// Flags after which an interpreter's next argument is inline code.
const INLINE_CODE_FLAGS: &[&str] = &["-c", "-e", "-E", "--eval", "-r", "-p", "--print"];
/// Commands that only print their visible arguments.
const LITERAL_SOURCES: &[&str] = &["echo", "printf"];
/// Bytes of one script file the classifier reads.
pub(super) const SCRIPT_READ_LIMIT: u64 = 64 * 1024;

/// Text built from adjacent string literals: `'~/.co' + 'dex'`,
/// `join('~', '.codex')` and `' '.join(['corb' + 'anu', 'vault'])` all name
/// a protected surface. Escapes such as `\x2e` are decoded. A run of
/// literals joined only by concatenation or argument punctuation yields two
/// texts: arguments joined as a path (`a/b`) and as words (`a b`).
pub(super) fn joined_literals(text: &str) -> Vec<String> {
    let chars: Vec<char> = text.chars().collect();
    // Each run: (as a path, as words, literal count).
    let mut runs: Vec<(String, String, usize)> = Vec::new();
    let mut run: (String, String, usize) = Default::default();
    // Literals nested in a literal (`"... '~' + '/.co' ..."`).
    let mut nested: Vec<String> = Vec::new();
    let mut index = 0;
    let mut gap = String::new();
    while index < chars.len() {
        let quote = chars[index];
        if !matches!(quote, '\'' | '"' | '`') {
            gap.push(quote);
            index += 1;
            continue;
        }
        let mut literal = String::new();
        let mut escaped = false;
        index += 1;
        while index < chars.len() && chars[index] != quote {
            if chars[index] == '\\' && index + 1 < chars.len() {
                escaped = true;
                let (decoded, used) = decode_escape(&chars[index + 1..]);
                literal.extend(decoded);
                index += 1 + used;
            } else {
                literal.push(chars[index]);
                index += 1;
            }
        }
        index += 1;
        if literal.contains(['\'', '"', '`']) {
            nested.extend(joined_literals(&literal));
        }
        // A lone literal matters too once escapes are decoded (`'\x2ecodex'`).
        if escaped {
            nested.push(literal.clone());
        }
        let joins = gap
            .chars()
            .all(|ch| ch.is_whitespace() || "+.,/()[]".contains(ch));
        if !joins {
            runs.push(std::mem::take(&mut run));
        }
        let (path, words, count) = &mut run;
        if *count > 0 {
            if gap.contains(',') || gap.contains('/') {
                path.push('/');
            }
            if gap.contains(',') {
                words.push(' ');
            }
        }
        gap.clear();
        path.push_str(&literal);
        words.push_str(&literal);
        *count += 1;
    }
    runs.push(run);
    runs.into_iter()
        .filter(|(_, _, count)| *count > 1)
        .flat_map(|(path, words, _)| [path, words])
        .chain(nested)
        .collect()
}

/// One backslash escape (after the backslash): `\xHH`, `\uHHHH`, octal and
/// the usual single letters. Returns the character and the length consumed.
fn decode_escape(chars: &[char]) -> (Option<char>, usize) {
    let hex = |len: usize| {
        let text: String = chars
            .iter()
            .skip(1)
            .take(len)
            .take_while(|ch| ch.is_ascii_hexdigit())
            .collect();
        let decoded = u32::from_str_radix(&text, 16).ok().and_then(char::from_u32);
        (decoded, 1 + text.len())
    };
    match chars.first() {
        Some('x') => hex(2),
        Some('u') => hex(4),
        Some('0'..='7') => {
            let text: String = chars
                .iter()
                .take(3)
                .take_while(|ch| ('0'..='7').contains(*ch))
                .collect();
            let decoded = u32::from_str_radix(&text, 8).ok().and_then(char::from_u32);
            (decoded, text.len())
        }
        Some('n') => (Some('\n'), 1),
        Some('t') => (Some('\t'), 1),
        Some(other) => (Some(*other), 1),
        None => (None, 0),
    }
}

/// Text hidden in a base64 or hex word, when it decodes to readable text.
pub(super) fn decoded_payload(word: &str) -> Option<String> {
    let word = word.trim_matches(|ch: char| !ch.is_ascii_alphanumeric() && ch != '=');
    if word.len() < 12 {
        return None;
    }
    let bytes = if word.len() % 2 == 0 && word.chars().all(|ch| ch.is_ascii_hexdigit()) {
        hex_bytes(word)?
    } else if word
        .chars()
        .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '+' | '/' | '-' | '_' | '='))
    {
        let engine = base64::engine::general_purpose::GeneralPurpose::new(
            &base64::alphabet::STANDARD,
            base64::engine::GeneralPurposeConfig::new()
                .with_decode_padding_mode(base64::engine::DecodePaddingMode::Indifferent),
        );
        let standard = word.replace('-', "+").replace('_', "/");
        engine.decode(standard).ok()?
    } else {
        return None;
    };
    let text = String::from_utf8(bytes).ok()?;
    let readable = text
        .chars()
        .all(|ch| !ch.is_control() || ch.is_whitespace());
    (readable && text.chars().any(char::is_alphabetic)).then_some(text)
}

fn hex_bytes(word: &str) -> Option<Vec<u8>> {
    (0..word.len())
        .step_by(2)
        .map(|at| u8::from_str_radix(&word[at..at + 2], 16).ok())
        .collect()
}

/// Lowercase basename of the command a simple command runs, skipping
/// assignments and common wrappers, with the index of that word.
pub(super) fn command_word(words: &[String]) -> Option<(usize, String)> {
    const WRAPPERS: &[&str] = &[
        "sudo", "env", "nice", "nohup", "time", "timeout", "exec", "command", "builtin", "xargs",
        "stdbuf", "doas",
    ];
    let mut index = 0;
    while let Some(word) = words.get(index) {
        let name = basename(word).to_lowercase();
        let assignment = word.contains('=') && !word.starts_with('-');
        if assignment || WRAPPERS.contains(&name.as_str()) || word.starts_with('-') {
            index += 1;
            // A wrapper's numeric or user argument (`timeout 9`, `-u root`).
            if let Some(next) = words.get(index)
                && (next.parse::<f64>().is_ok() || words[index - 1] == "-u")
            {
                index += 1;
            }
            continue;
        }
        return Some((index, name));
    }
    None
}

/// The file a shell or interpreter command runs, if it names one: the first
/// non-flag argument of `bash x.sh`, `python3 x.py`, `source x`, or the
/// command word itself when it is a path (`./run.sh`).
pub(super) fn script_file(words: &[String]) -> Option<&str> {
    let (index, name) = command_word(words)?;
    let word = words[index].as_str();
    if word.contains('/')
        && !SHELLS.contains(&name.as_str())
        && !INTERPRETERS.contains(&name.as_str())
    {
        return Some(word);
    }
    let runs_files = SHELLS.contains(&name.as_str())
        || INTERPRETERS.contains(&name.as_str())
        || matches!(name.as_str(), "source" | ".");
    if !runs_files {
        return None;
    }
    let args = &words[index + 1..];
    if args
        .iter()
        .any(|arg| INLINE_CODE_FLAGS.contains(&arg.as_str()) || is_shell_c_flag(arg))
    {
        return None;
    }
    args.iter()
        .find(|arg| !arg.starts_with('-'))
        .map(String::as_str)
}

fn is_shell_c_flag(arg: &str) -> bool {
    arg.starts_with('-') && !arg.starts_with("--") && arg.ends_with('c')
}

/// Whether the command runs code it reads from stdin or a substitution:
/// `eval`, `source`/`.`, a shell or interpreter with no file or inline code,
/// or a bare substitution used as a command (`$(curl ...)`).
fn executes_input(command: &SimpleCommand) -> bool {
    let Some((index, name)) = command_word(&command.words) else {
        return command.takes_substitution;
    };
    if matches!(name.as_str(), "eval" | "source" | ".") {
        return true;
    }
    let runs_code = SHELLS.contains(&name.as_str()) || INTERPRETERS.contains(&name.as_str());
    let args = &command.words[index + 1..];
    let inline = args
        .iter()
        .any(|arg| INLINE_CODE_FLAGS.contains(&arg.as_str()));
    // Stdin (no file, no inline code), or inline code that is a substitution.
    runs_code
        && if inline {
            command.takes_substitution
        } else {
            args.iter().all(|arg| arg.starts_with('-'))
        }
}

/// Whether a stage only prints text the classifier already sees.
fn literal_stage(command: &SimpleCommand) -> bool {
    command_word(&command.words).is_some_and(|(_, name)| LITERAL_SOURCES.contains(&name.as_str()))
}

/// Opaque execution: code fed to an executor through a pipe or a
/// substitution, by any stage that is not a plain `echo`/`printf` of visible
/// text (a decoder, a download, `cat` of a file, a transform). The host
/// cannot describe what will run, so it counts as protected.
pub(super) fn opaque_execution(commands: &[SimpleCommand]) -> bool {
    commands.iter().enumerate().any(|(index, command)| {
        if !executes_input(command) {
            return false;
        }
        let piped_from = command.piped.then(|| {
            commands[..index]
                .iter()
                .rev()
                .scan(true, |more, stage| {
                    let take = *more;
                    *more = stage.piped;
                    take.then_some(stage)
                })
                .collect::<Vec<_>>()
        });
        let substituted = command.takes_substitution.then(|| {
            commands[index + 1..]
                .iter()
                .take_while(|stage| stage.substituted || stage.piped)
                .collect::<Vec<_>>()
        });
        piped_from
            .into_iter()
            .chain(substituted)
            .flatten()
            .any(|stage| !literal_stage(stage))
    })
}

/// Files in an `apply_patch` body whose added lines are run later (scripts,
/// task files, git hooks, shell start-up files), with those lines.
pub(super) fn runnable_patch_additions(patch: &str) -> Vec<(String, String)> {
    const SCRIPT_EXTENSIONS: &[&str] = &[
        "sh", "bash", "zsh", "fish", "ksh", "command", "py", "js", "mjs", "cjs", "ts", "rb", "pl",
        "php", "lua", "ps1", "bat", "cmd",
    ];
    const RUNNABLE_NAMES: &[&str] = &[
        "makefile",
        "justfile",
        "package.json",
        ".envrc",
        ".bashrc",
        ".zshrc",
        ".profile",
        ".bash_profile",
        ".zprofile",
    ];
    let mut files: Vec<(String, String)> = Vec::new();
    for line in patch.lines() {
        let header = ["*** Add File: ", "*** Update File: ", "*** Move to: "]
            .iter()
            .find_map(|prefix| line.strip_prefix(prefix));
        if let Some(path) = header {
            files.push((path.trim().to_string(), String::new()));
            continue;
        }
        if let (Some(added), Some((_, body))) = (line.strip_prefix('+'), files.last_mut()) {
            body.push_str(added);
            body.push('\n');
        }
    }
    files
        .into_iter()
        .filter(|(path, body)| {
            let lower = path.to_lowercase();
            let name = basename(&lower);
            let extension = name.rsplit_once('.').map(|(_, extension)| extension);
            body.starts_with("#!")
                || lower.contains(".git/hooks/")
                || lower.contains("/bin/")
                || RUNNABLE_NAMES.contains(&name)
                || extension.is_some_and(|extension| SCRIPT_EXTENSIONS.contains(&extension))
        })
        .collect()
}
