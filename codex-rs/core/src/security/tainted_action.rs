//! PF-30-S03 post-taint authority checks.
//!
//! Once a session has taken in content without standing (tool, MCP, agent,
//! memory or unattributed text, PF-30-S02), a model-selected action that
//! reaches credentials, the vault or security policy needs fresh, exact human
//! approval. Neither a cached session approval, a permission hook, the
//! automatic reviewer nor `approval_policy = never` can stand in for it, and
//! the approval is bound to the taint generation and the effective security
//! policy it was given under.
//!
//! This is a deterministic net over the exact command or patch the host will
//! run, not a prompt classifier, and it never relaxes anything: outside
//! `source_envelopes` with Moderate/Aggressive it does nothing. Besides the
//! words of the command it follows variables, `cd`/`-C` folders, symlinks,
//! inline interpreter code, encoded payloads, script files and patched
//! scripts; code it cannot see through (a decode-and-run pipeline) counts as
//! protected.

use crate::tools::sandboxing::ApprovalAction;
use codex_security_policy::ActorChain;
use codex_security_policy::SecurityLevel;
use codex_utils_path_uri::PathUri;
use paths::Homes;
use shell::SimpleCommand;
use shell::basename;
use std::collections::HashMap;
use std::path::Path;

mod indirect;
mod paths;
mod shell;

/// Protected surfaces a tainted session may not reach on model authority.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ProtectedActionKind {
    /// The Corbanu vault (`corbanu vault ...`).
    Vault,
    /// Credential stores: login tokens, SSH/cloud keys, OS keychain.
    Credentials,
    /// Security policy: Corbanu config, rules, hooks, requirements, login
    /// state, and code the host cannot describe.
    SecurityPolicy,
}

impl ProtectedActionKind {
    pub(crate) fn describe(self) -> &'static str {
        match self {
            Self::Vault => "vault access",
            Self::Credentials => "credential access",
            Self::SecurityPolicy => "a security policy change",
        }
    }
}

/// The effective security policy a post-taint approval is bound to.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum PolicyBinding {
    /// No live policy runtime is bound to this session.
    Unbound,
    /// The policy runtime could not be read.
    Unavailable,
    Bound {
        epoch: u64,
        revocation_generation: u64,
        kill_switch_active: bool,
        level: SecurityLevel,
        /// The agent lineage the session acts for.
        actor_chain: ActorChain,
    },
}

/// Session state a post-taint decision depends on.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct PostTaintState {
    /// Count of recorded batches without standing.
    pub(crate) taint_generation: u64,
    pub(crate) policy: PolicyBinding,
}

/// Host-held state for one protected action, captured before approval and
/// checked again right before it runs.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct PostTaintAction {
    pub(crate) kind: ProtectedActionKind,
    pub(crate) state: PostTaintState,
}

impl PostTaintAction {
    /// Shown in the approval prompt so the human sees why it was asked again.
    pub(crate) fn approval_reason(&self) -> String {
        format!(
            "Security level: this is {} after untrusted content entered the session. \
             Approve only if you asked for it; a cached or automatic approval does not count.",
            self.kind.describe()
        )
    }

    pub(crate) fn approvals_off_rejection(&self) -> String {
        format!(
            "Not run: this is {} after untrusted content entered the session, so it needs your \
             approval, and approvals are off (approval_policy = never).",
            self.kind.describe()
        )
    }

    /// Refusal before any prompt: the kill switch denies protected actions.
    pub(crate) fn refused_up_front(&self) -> Option<String> {
        matches!(
            self.state.policy,
            PolicyBinding::Bound {
                kill_switch_active: true,
                ..
            }
        )
        .then(|| {
            format!(
                "Not run: this is {} after untrusted content entered the session, and the \
                 security kill switch is on.",
                self.kind.describe()
            )
        })
    }

    /// Re-check after the human answered: the decision covers only the taint
    /// and policy that were in front of them. `now` is `None` when the checks
    /// no longer apply (the level or flag changed), which also refuses.
    pub(crate) fn recheck(&self, now: Option<&PostTaintState>) -> Result<(), String> {
        let describe = self.kind.describe();
        match now {
            Some(now) if now.taint_generation != self.state.taint_generation => Err(format!(
                "Not run: new untrusted content arrived while {describe} was waiting for \
                 approval. Ask again if it is still needed."
            )),
            Some(now) if *now == self.state => Ok(()),
            Some(_) | None => Err(format!(
                "Not run: the security policy changed while {describe} was waiting for \
                 approval. Ask again if it is still needed."
            )),
        }
    }
}

const CLI_NAMES: &[&str] = &["corbanu", "codex", "pfterminal", "corbanu-debug"];
/// Subcommands of the Corbanu CLI that change policy or login state.
const CLI_POLICY_WORDS: &[&str] = &["config", "features", "login", "logout"];
/// Corbanu CLI flags that widen a child run's policy.
const CLI_POLICY_FLAGS: &[&str] = &["--full-auto", "--yolo", "danger-full-access", "--add-dir"];
/// `-c key=value` overrides of these keys change security policy.
const POLICY_CONFIG_KEYS: &[&str] = &[
    "approval_policy",
    "approvals_reviewer",
    "sandbox_mode",
    "sandbox_workspace_write",
    "sandbox_permissions",
    "permissions",
    "security",
    "features",
    "shell_environment_policy",
    "projects",
];
/// Credential commands: the command name plus words that make it a secret read.
const CREDENTIAL_COMMANDS: &[(&str, &[&str])] = &[
    (
        "security",
        &[
            "find-generic-password",
            "find-internet-password",
            "find-certificate",
            "add-generic-password",
            "add-internet-password",
            "delete-generic-password",
            "delete-internet-password",
            "set-generic-password-partition-list",
            "dump-keychain",
            "export",
            "unlock-keychain",
        ],
    ),
    ("gh", &["token"]),
    ("gcloud", &["print-access-token", "print-identity-token"]),
    ("aws", &["export-credentials", "get-session-token"]),
    ("op", &["read", "item"]),
    ("secret-tool", &["lookup", "search"]),
    ("gpg", &["--export-secret-keys", "--export-secret-subkeys"]),
    ("pass", &["show"]),
];
/// Commands that read a folder tree (with the flag that makes them do so,
/// if any): pointed at a folder holding a home, they take its credentials.
const RECURSIVE_READERS: &[(&str, &[&str])] = &[
    ("tar", &[]),
    ("zip", &[]),
    ("7z", &[]),
    ("7za", &[]),
    ("rsync", &[]),
    ("scp", &[]),
    ("ditto", &[]),
    ("cpio", &[]),
    ("pax", &[]),
    ("cp", &["-r", "-R", "-a", "--recursive", "--archive"]),
    ("grep", &["-r", "-R", "--recursive"]),
    ("rg", &["--hidden", "-u", "-.", "--no-ignore"]),
    ("find", &["-exec", "-execdir", "-ok"]),
];
/// Flags whose value is the folder later relative words resolve against.
const FOLDER_FLAGS: &[&str] = &["-C", "--directory", "--cwd", "--cd", "--chdir"];
/// Nesting limit for scripts, payloads and literals classified inside an
/// action. Deeper nesting fails closed.
const MAX_DEPTH: usize = 4;
/// Script files one classification reads; more fail closed.
const MAX_SCRIPT_FILES: usize = 16;
/// Magic numbers of executables, which are not read as scripts.
const BINARY_MAGIC: &[&[u8]] = &[
    b"\x7fELF",
    b"\xfe\xed\xfa\xce",
    b"\xfe\xed\xfa\xcf",
    b"\xce\xfa\xed\xfe",
    b"\xcf\xfa\xed\xfe",
    b"\xca\xfe\xba\xbe",
    b"MZ",
];

/// Classify the exact action the host is about to run. `codex_home` is the
/// session's Corbanu home; the user's home resolves `~` and `$HOME`.
pub(crate) fn classify(action: &ApprovalAction, codex_home: &Path) -> Option<ProtectedActionKind> {
    classify_with(action, codex_home, dirs::home_dir().as_deref())
}

pub(super) fn classify_with(
    action: &ApprovalAction,
    codex_home: &Path,
    user_home: Option<&Path>,
) -> Option<ProtectedActionKind> {
    let mut classifier = Classifier {
        homes: Homes::new(codex_home, user_home),
        found: None,
        scripts_read: 0,
    };
    match action {
        ApprovalAction::Shell { command, cwd, .. }
        | ApprovalAction::ExecCommand { command, cwd, .. } => {
            let cwd = path_text(cwd);
            classifier.homes.allow_lookups_under(&cwd);
            classifier.path(&cwd);
            classifier.command(command, &cwd, /*depth*/ 0);
        }
        ApprovalAction::ApplyPatch {
            files, patch, cwd, ..
        } => {
            let cwd = path_text(cwd);
            classifier.homes.allow_lookups_under(&cwd);
            for file in files {
                classifier.path(&path_text(file));
            }
            // Commands written into a file that is run later are judged now,
            // whether the file looks runnable by name or by where it leads.
            for (file, body) in indirect::patch_additions(patch) {
                let resolved = classifier.homes.resolve(&file, &cwd);
                let canonical = classifier.homes.canonical(&resolved);
                let runnable = [Some(resolved.to_lowercase()), canonical]
                    .into_iter()
                    .flatten()
                    .any(|path| indirect::is_runnable_file(&path, &body));
                if runnable {
                    classifier.script(&body, &cwd, /*depth*/ 1);
                }
            }
        }
    }
    // A lookup budget ran out: something was not followed.
    if classifier.homes.exhausted() {
        classifier.note(ProtectedActionKind::SecurityPolicy);
    }
    classifier.found
}

fn path_text(path: &PathUri) -> String {
    path.to_abs_path()
        .map(|path| path.as_path().to_string_lossy().into_owned())
        .unwrap_or_else(|_| path.to_string())
}

fn rank(kind: ProtectedActionKind) -> u8 {
    match kind {
        ProtectedActionKind::SecurityPolicy => 0,
        ProtectedActionKind::Credentials => 1,
        ProtectedActionKind::Vault => 2,
    }
}

/// The CLI subcommand: the first word that is neither a flag nor a flag's value.
fn subcommand(args: &[String]) -> Option<&str> {
    const VALUE_FLAGS: &[&str] = &[
        "-c",
        "--config",
        "-p",
        "--profile",
        "-m",
        "--model",
        "--cd",
        "-s",
        "--sandbox",
        "-a",
        "--ask-for-approval",
        "-i",
        "--image",
    ];
    let mut skip = false;
    for word in args {
        if std::mem::take(&mut skip) {
            continue;
        }
        if word.starts_with('-') {
            skip = !word.contains('=') && VALUE_FLAGS.contains(&word.as_str());
            continue;
        }
        return Some(word.as_str());
    }
    None
}

/// Whether `word` sets `flag`: exactly for long flags, and inside a group of
/// short flags (`-rn` sets `-r`).
fn has_flag(word: &str, flag: &str) -> bool {
    if word == flag {
        return true;
    }
    match flag.strip_prefix('-') {
        Some(letter) if letter.len() == 1 && !letter.starts_with('-') => {
            word.starts_with('-') && !word.starts_with("--") && word[1..].contains(letter)
        }
        _ => false,
    }
}

/// Where the shell is while it runs a script: the folder, the previous one
/// (`cd -`) and the `pushd` stack.
struct Folders {
    cwd: String,
    previous: String,
    stack: Vec<String>,
}

struct Classifier {
    homes: Homes,
    found: Option<ProtectedActionKind>,
    scripts_read: usize,
}

impl Classifier {
    fn note(&mut self, kind: ProtectedActionKind) {
        if self.found.is_none_or(|current| rank(kind) > rank(current)) {
            self.found = Some(kind);
        }
    }

    fn path(&mut self, path: &str) {
        if let Some(kind) = self.homes.classify_resolved(path) {
            self.note(kind);
        }
    }

    /// Text that is itself a command line (a script file, a decoded payload,
    /// joined literals, patched lines).
    fn script(&mut self, text: &str, cwd: &str, depth: usize) {
        self.command(&["sh".into(), "-c".into(), text.into()], cwd, depth);
    }

    fn command(&mut self, command: &[String], cwd: &str, depth: usize) {
        if depth > MAX_DEPTH {
            // Too deeply nested to follow: fail closed.
            self.note(ProtectedActionKind::SecurityPolicy);
            return;
        }
        let commands = shell::simple_commands(command);
        if indirect::opaque_execution(&commands) {
            self.note(ProtectedActionKind::SecurityPolicy);
        }
        let mut folders = Folders {
            cwd: cwd.to_string(),
            previous: cwd.to_string(),
            stack: Vec::new(),
        };
        let mut variables: HashMap<String, String> = HashMap::new();
        for (name, value) in [
            ("PWD", Some(cwd)),
            ("HOME", self.homes.user_home()),
            ("CODEX_HOME", Some(self.homes.codex_home())),
        ] {
            if let Some(value) = value {
                variables.insert(name.to_string(), value.to_string());
            }
        }
        for (index, simple) in commands.iter().enumerate() {
            let words = substitute(simple, &variables);
            let feeds_pipe = commands.iter().any(|later| later.pipe_from == Some(index));
            self.simple_command(&words, feeds_pipe, &mut folders, &mut variables, depth);
        }
        self.path(&folders.cwd);
        // Strings assembled from literals by inline code (`'~/.co' + 'dex'`).
        for arg in command {
            for joined in indirect::joined_literals(arg) {
                let resolved = self.homes.resolve(&joined, &folders.cwd);
                self.path(&resolved);
                self.script(&joined, &folders.cwd, depth + 1);
            }
        }
    }

    fn simple_command(
        &mut self,
        words: &[String],
        feeds_pipe: bool,
        folders: &mut Folders,
        variables: &mut HashMap<String, String>,
        depth: usize,
    ) {
        let lower: Vec<String> = words.iter().map(|word| word.to_lowercase()).collect();
        let command = indirect::command_word(words);
        // Assignments (`A=x`, `export A=x`) feed later `$A` words. A value
        // the classifier cannot see makes the variable unknown.
        let assigns = command.as_ref().is_none_or(|(_, name)| {
            matches!(
                name.as_str(),
                "export" | "local" | "declare" | "readonly" | "typeset"
            )
        });
        if assigns {
            for word in words {
                if let Some((name, value)) = word.split_once('=')
                    && !name.is_empty()
                    && name
                        .chars()
                        .all(|ch| ch.is_ascii_alphanumeric() || ch == '_')
                {
                    if value.contains('$') {
                        variables.remove(name);
                    } else {
                        variables.insert(name.to_string(), value.to_string());
                    }
                }
            }
        }
        if let Some((index, name)) = &command {
            let args = &words[index + 1..];
            // Code named by a variable or a substitution the classifier
            // cannot see (`$X`, `bash -c "$X"`, `python3 -c "$X"`).
            let inline_unseen = args.windows(2).any(|pair| {
                matches!(pair[0].as_str(), "-c" | "-e" | "--eval") && pair[1].starts_with('$')
            });
            if name.starts_with('$')
                || inline_unseen
                || (name == "eval" && args.iter().any(|arg| arg.contains('$')))
            {
                self.note(ProtectedActionKind::SecurityPolicy);
            }
            self.change_folder(name, args, folders);
        }
        // Not only at command position: wrappers (`nice -n 5`, `sudo -u x`,
        // `timeout 9`, `eval`, `npx`, nested `sh -c`) put the real command later.
        for (index, word) in lower.iter().enumerate() {
            let name = basename(word);
            let args = &lower[index + 1..];
            let imported = index > 0 && matches!(lower[index - 1].as_str(), "from" | "import");
            if CLI_NAMES.contains(&name) && !imported {
                self.cli(args);
            }
            for (credential_command, verbs) in CREDENTIAL_COMMANDS {
                if name == *credential_command
                    && args.iter().any(|word| verbs.contains(&word.as_str()))
                {
                    self.note(ProtectedActionKind::Credentials);
                }
            }
        }
        let recursive = command.as_ref().is_some_and(|(_, name)| {
            RECURSIVE_READERS.iter().any(|(reader, flags)| {
                *reader == name.as_str()
                    && (flags.is_empty()
                        || (name == "find" && feeds_pipe)
                        || lower
                            .iter()
                            .any(|word| flags.iter().any(|flag| has_flag(word, flag))))
            })
        });
        let cwd = folders.cwd.clone();
        // Words resolve against the folder of a `-C`/`--directory` flag
        // (`tar -C ~ ...`, `git -C ~/.codex ...`) once one has been given.
        let mut folder = cwd.clone();
        let mut previous: Option<&str> = None;
        for word in words {
            // A folder flag's value is where the command runs, not what it reads.
            let mut folder_value = false;
            if let Some(flag) = previous
                && FOLDER_FLAGS.contains(&flag)
            {
                folder = self.homes.resolve(word, &cwd);
                folder_value = true;
            } else if let Some((flag, value)) = word.split_once('=')
                && FOLDER_FLAGS.contains(&flag)
            {
                folder = self.homes.resolve(value, &cwd);
                folder_value = true;
            }
            previous = Some(word.as_str());
            // Attached and assigned forms (`-o/path`, `f=@/path`) are paths too.
            for candidate in word.split('=') {
                let candidate = candidate.trim_start_matches('@');
                if candidate.is_empty() || candidate.starts_with('-') && !candidate.contains('/') {
                    continue;
                }
                let candidate = match candidate.find('/') {
                    Some(slash) if candidate.starts_with('-') => &candidate[slash..],
                    _ => candidate,
                };
                // `-C` means something else to some commands (`grep -C 3`):
                // resolve against both folders.
                let bases = if folder == cwd {
                    vec![folder.clone()]
                } else {
                    vec![folder.clone(), cwd.clone()]
                };
                for base in bases {
                    let resolved = self.homes.resolve(candidate, &base);
                    self.path(&resolved);
                    if recursive && !folder_value && self.homes.holds_a_home(&resolved) {
                        self.note(ProtectedActionKind::Credentials);
                    }
                }
            }
            if let Some(payload) = indirect::decoded_payload(word) {
                self.script(&payload, &cwd, depth + 1);
            }
        }
        match indirect::script_source(words) {
            Some(indirect::ScriptSource::File(file)) => {
                let file = self.homes.resolve(file, &cwd);
                self.script_file(&file, &cwd, depth);
            }
            Some(indirect::ScriptSource::Unseen) => self.note(ProtectedActionKind::SecurityPolicy),
            None => {}
        }
    }

    /// Follow `cd`, `pushd` and `popd` (also behind `builtin`/`command`).
    fn change_folder(&self, name: &str, args: &[String], folders: &mut Folders) {
        let target = args
            .iter()
            .find(|word| !word.starts_with('-') || *word == "-");
        let next = match (name, target) {
            ("cd", None) => self.homes.user_home().map(str::to_string),
            ("cd", Some(target)) if target == "-" => Some(folders.previous.clone()),
            ("cd" | "pushd", Some(target)) => Some(self.homes.resolve(target, &folders.cwd)),
            ("popd", _) => folders.stack.pop(),
            _ => None,
        };
        if let Some(next) = next {
            if name == "pushd" {
                folders.stack.push(folders.cwd.clone());
            }
            folders.previous = std::mem::replace(&mut folders.cwd, next);
        }
    }

    fn cli(&mut self, args: &[String]) {
        let policy_key = |value: &str| POLICY_CONFIG_KEYS.iter().any(|key| value.starts_with(key));
        if args.iter().any(|word| word == "vault") {
            self.note(ProtectedActionKind::Vault);
        }
        if subcommand(args).is_some_and(|sub| CLI_POLICY_WORDS.contains(&sub)) {
            self.note(ProtectedActionKind::SecurityPolicy);
        }
        let policy_flag = args.iter().enumerate().any(|(index, word)| {
            CLI_POLICY_FLAGS.iter().any(|flag| word.contains(flag))
                || word.starts_with("--dangerously")
                || word.strip_prefix("--config=").is_some_and(policy_key)
                || (word.len() > 2 && word.strip_prefix("-c").is_some_and(policy_key))
                || (matches!(word.as_str(), "-c" | "--config")
                    && args.get(index + 1).is_some_and(|value| policy_key(value)))
        });
        if policy_flag {
            self.note(ProtectedActionKind::SecurityPolicy);
        }
    }

    /// A script run from a file is judged by its text (a patched script run
    /// later, `bash x.sh`, `./run`). A script the classifier cannot read in
    /// full (missing, written by the same command, not a regular file, too
    /// large, past the file limit, or where lookups are not allowed) fails
    /// closed; executables are skipped by their magic number.
    fn script_file(&mut self, file: &str, cwd: &str, depth: usize) {
        use std::io::Read;
        let unreadable = |classifier: &mut Self| {
            classifier.note(ProtectedActionKind::SecurityPolicy);
        };
        self.scripts_read += 1;
        if self.scripts_read > MAX_SCRIPT_FILES || !self.homes.lookups_allowed(file) {
            return unreadable(self);
        }
        // Check before opening: opening a FIFO or device would block.
        let Ok(metadata) = std::fs::metadata(file) else {
            return unreadable(self);
        };
        if !metadata.is_file() || metadata.len() > indirect::SCRIPT_READ_LIMIT {
            return unreadable(self);
        }
        let mut options = std::fs::OpenOptions::new();
        options.read(true);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            // Never block if the file was swapped for a FIFO after the check.
            options.custom_flags(libc::O_NONBLOCK);
        }
        let Ok(handle) = options.open(file) else {
            return unreadable(self);
        };
        let mut bytes = Vec::new();
        if !handle.metadata().is_ok_and(|metadata| metadata.is_file())
            || handle
                .take(indirect::SCRIPT_READ_LIMIT + 1)
                .read_to_end(&mut bytes)
                .is_err()
            || bytes.len() as u64 > indirect::SCRIPT_READ_LIMIT
        {
            return unreadable(self);
        }
        if BINARY_MAGIC.iter().any(|magic| bytes.starts_with(magic)) {
            return;
        }
        // The shell skips NUL bytes; so does the scan.
        bytes.retain(|byte| *byte != 0);
        let text = String::from_utf8_lossy(&bytes);
        self.script(&text, cwd, depth + 1);
    }
}

/// Words with known `$NAME` variables replaced, left to right as the shell
/// reads them (`$a$b` is `a` then `b`).
fn substitute(command: &SimpleCommand, variables: &HashMap<String, String>) -> Vec<String> {
    command
        .words
        .iter()
        .map(|word| {
            let mut out = String::new();
            let mut rest = word.as_str();
            while let Some(at) = rest.find('$') {
                out.push_str(&rest[..at]);
                let after = &rest[at + 1..];
                let len = after
                    .find(|ch: char| !(ch.is_ascii_alphanumeric() || ch == '_'))
                    .unwrap_or(after.len());
                match variables.get(&after[..len]) {
                    Some(value) if len > 0 => out.push_str(value),
                    _ => {
                        out.push('$');
                        out.push_str(&after[..len]);
                    }
                }
                rest = &after[len..];
            }
            out.push_str(rest);
            out
        })
        .collect()
}

#[cfg(test)]
#[path = "tainted_action_tests.rs"]
mod tests;
