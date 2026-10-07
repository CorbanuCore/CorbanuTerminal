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
use invocation::Code;
use paths::Homes;
use shell::SimpleCommand;
use shell::basename;
use std::collections::HashMap;
use std::path::Path;

mod indirect;
mod invocation;
mod outbound;
mod paths;
mod shell;

pub(crate) use paths::USER_PERSISTENCE;
pub(crate) use paths::WORKSPACE_PERSISTENCE;

/// Protected surfaces a tainted session may not reach on model authority.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ProtectedActionKind {
    /// The Corbanu vault (`corbanu vault ...`).
    Vault,
    /// Credential stores: login tokens, SSH/cloud keys, OS keychain.
    Credentials,
    /// Security policy: Corbanu config, rules, hooks, requirements, login state.
    SecurityPolicy,
    /// Code the host cannot read before it runs (a download piped into a
    /// shell, a script it cannot read in full, a variable it cannot see), or
    /// an action it cannot describe.
    UnseenCode,
    /// Local content (a file, another command's output, session data passed
    /// to an outside tool) sent to another machine or service (PF-23-S01).
    Disclosure,
    /// Moving money or tokens (PF-23-S01).
    ValueTransfer,
    /// A tool route the security level has not classified (PF-23-S01).
    UnclassifiedTool,
    /// Writing a file that runs code or sets policy later, outside the
    /// sandbox: shell start-up files, login items, git hooks (PF-23-S02).
    Persistence,
    /// Typing into a process whose sandbox was set before the protected-path
    /// rules applied, so it can still read credentials (PF-23-S02).
    UnconfinedProcess,
}

impl ProtectedActionKind {
    /// The more protected of the two (the one a prompt names).
    pub(crate) fn strongest(self, other: Self) -> Self {
        if rank(other) > rank(self) {
            other
        } else {
            self
        }
    }

    pub(crate) fn describe(self) -> &'static str {
        match self {
            Self::Vault => "vault access",
            Self::Credentials => "credential access",
            Self::SecurityPolicy => "a security policy change",
            Self::UnseenCode => "running code the host cannot read first",
            Self::Disclosure => "sending local data to another machine or service",
            Self::ValueTransfer => "a value transfer",
            Self::UnclassifiedTool => "a tool the security level has not classified",
            Self::Persistence => {
                "changing a file that runs code later (shell start-up, login items, git hooks)"
            }
            Self::UnconfinedProcess => {
                "typing into a process started before the session's protected-path rules applied"
            }
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
    /// The effective level: the stricter of the configured level and the
    /// live policy's (Aggressive when the policy cannot be read).
    pub(crate) level: SecurityLevel,
}

impl PostTaintState {
    /// Whether the sandbox's protected-path rules apply: after untrusted
    /// content, and under a protected level (Moderate or Aggressive) from the
    /// start. Applying them before taint under Moderate closes the files-route
    /// read gap (issue #239): a workspace-write sandbox otherwise lets an
    /// agent read credential files from `$HOME` before any untrusted content
    /// arrives. Sessions without a protected level carry no state today; the
    /// taint arm stays as defense in depth.
    pub(crate) fn protected_paths_apply(&self) -> bool {
        self.taint_generation > 0 || self.level != SecurityLevel::Permissive
    }

    /// Under Moderate (configured and live), a human approval given at taint
    /// generation `approved_at` lifts the protected-path rules for its command
    /// while no new untrusted content has arrived since (issue #239).
    pub(crate) fn human_approval_lifts_rules(&self, approved_at: Option<u64>) -> bool {
        self.moderate_bound() && approved_at == Some(self.taint_generation)
    }

    /// Moderate, both configured and in the live policy: a human approval
    /// may lift the protected-path rules for one command.
    pub(crate) fn moderate_bound(&self) -> bool {
        self.level == SecurityLevel::Moderate
            && matches!(
                self.policy,
                PolicyBinding::Bound {
                    level: SecurityLevel::Moderate,
                    ..
                }
            )
    }
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
/// Commands that write the paths they name (PF-23-S02): naming a persistence
/// file to one of them, or redirecting output to it, is a write.
const WRITERS: &[&str] = &[
    "tee", "cp", "mv", "install", "ln", "rsync", "dd", "truncate", "touch", "ditto", "sed", "perl",
    "ed", "ex", "patch", "chmod", "unzip", "tar", "curl", "wget", "rm", "rmdir", "unlink", "mkdir",
];
/// `git config` keys that make git run a program (PF-23-S02).
const GIT_RUN_KEYS: &[&str] = &[
    "core.hookspath",
    "core.fsmonitor",
    "core.sshcommand",
    "core.pager",
    "core.editor",
    "core.askpass",
    "credential.helper",
];
/// Flags whose value is the folder later relative words resolve against.
const FOLDER_FLAGS: &[&str] = &["-C", "--directory", "--cwd", "--cd", "--chdir"];
/// Nesting limit for scripts, payloads and literals classified inside an
/// action. Deeper nesting fails closed.
const MAX_DEPTH: usize = 4;
/// Bytes of command text one classification lexes; more fail closed.
const MAX_COMMAND_BYTES: usize = 1024 * 1024;
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
        rebuilt: 0,
        not_shell: 0,
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
                let file = path_text(file);
                classifier.path(&file);
                if classifier.homes.is_persistence(&file) {
                    classifier.note(ProtectedActionKind::Persistence);
                }
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
        classifier.note(ProtectedActionKind::UnseenCode);
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
        ProtectedActionKind::UnclassifiedTool => 0,
        ProtectedActionKind::UnseenCode => 1,
        ProtectedActionKind::UnconfinedProcess => 2,
        ProtectedActionKind::Disclosure => 3,
        ProtectedActionKind::Persistence => 4,
        ProtectedActionKind::SecurityPolicy => 5,
        ProtectedActionKind::Credentials => 6,
        ProtectedActionKind::ValueTransfer => 7,
        ProtectedActionKind::Vault => 8,
    }
}

/// Commands that install something to run later outside the sandbox:
/// global git config or a git key that runs a program, cron, launchd and
/// systemd user units (PF-23-S02). `args` are lowercase.
fn persistence_command(name: &str, args: &[String]) -> bool {
    let has = |word: &str| args.iter().any(|arg| arg == word);
    match name {
        "git" => {
            let Some(at) = args.iter().position(|arg| arg == "config") else {
                return false;
            };
            let config = &args[at + 1..];
            let reading = config.iter().any(|arg| {
                matches!(
                    arg.as_str(),
                    "--get" | "--get-all" | "--get-regexp" | "--list" | "-l" | "get" | "list"
                )
            });
            !reading
                && (config
                    .iter()
                    .any(|arg| matches!(arg.as_str(), "--global" | "--system"))
                    || config.iter().any(|arg| {
                        GIT_RUN_KEYS.contains(&arg.as_str()) || arg.starts_with("alias.")
                    }))
        }
        "crontab" => !has("-l"),
        "launchctl" => ["load", "bootstrap", "submit", "enable"]
            .iter()
            .any(|verb| has(verb)),
        "systemctl" => has("--user") && ["enable", "link", "edit"].iter().any(|verb| has(verb)),
        _ => false,
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
    /// Inside text rebuilt from joined literals: it names things but is not
    /// itself run, so its variables are not judged as unseen code.
    rebuilt: usize,
    /// Inside a file that is not a shell script (Python, JavaScript, ...):
    /// shell brace expansion does not apply to it.
    not_shell: usize,
}

/// Whether a script the command runs must be read in full (it is what runs)
/// or is read only if it is there (a module that may be installed).
#[derive(Clone, Copy, PartialEq, Eq)]
enum Script {
    Required,
    IfPresent,
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

    /// A word that may be a path. Inside scripts, payloads and patches only
    /// path-looking words are looked up on disk (the rest stay lexical), so a
    /// long body cannot spend the lookup budget.
    fn word_path(&mut self, resolved: &str, word: &str, depth: usize) {
        if depth == 0 || word.contains('/') || word.starts_with(['.', '~', '$']) {
            self.path(resolved);
        } else if let Some(kind) = self.homes.classify_path(resolved) {
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
            self.note(ProtectedActionKind::UnseenCode);
            return;
        }
        // A command line too long to classify fails closed.
        if command.iter().map(String::len).sum::<usize>() > MAX_COMMAND_BYTES {
            self.note(ProtectedActionKind::UnseenCode);
            return;
        }
        let lexed = shell::simple_commands(command);
        let pipe_sources: std::collections::HashSet<usize> = lexed
            .commands
            .iter()
            .filter_map(|command| command.pipe_from)
            .collect();
        if (lexed.incomplete && self.not_shell == 0 && self.rebuilt == 0)
            || indirect::opaque_execution(&lexed.commands)
        {
            self.note(ProtectedActionKind::UnseenCode);
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
        for (index, simple) in lexed.commands.iter().enumerate() {
            let (words, unseen) = substitute(simple, &variables);
            let feeds_pipe = pipe_sources.contains(&index);
            self.simple_command(
                simple,
                &words,
                &unseen,
                feeds_pipe,
                &mut folders,
                &mut variables,
                depth,
            );
        }
        self.path(&folders.cwd);
        // Strings assembled from literals by inline code (`'~/.co' + 'dex'`).
        for arg in command {
            for joined in indirect::joined_literals(arg) {
                let resolved = self.homes.resolve(&joined, &folders.cwd);
                self.word_path(&resolved, &joined, depth);
                self.rebuilt += 1;
                self.script(&joined, &folders.cwd, depth + 1);
                self.rebuilt -= 1;
            }
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn simple_command(
        &mut self,
        simple: &SimpleCommand,
        words: &[String],
        unseen: &[bool],
        feeds_pipe: bool,
        folders: &mut Folders,
        variables: &mut HashMap<String, String>,
        depth: usize,
    ) {
        let lower: Vec<String> = words.iter().map(|word| word.to_lowercase()).collect();
        let command = indirect::command_word(words);
        // Assignments (`A=x`, `export A=x`) feed later `$A` words.
        let comment = words.first().is_some_and(|word| word.starts_with('#'));
        let assigns = !comment
            && command.as_ref().is_none_or(|(_, name)| {
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
                    // Output the classifier cannot see keeps the variable
                    // unseen; an environment reference (`$JAVA_HOME/bin/java`)
                    // is kept as written.
                    // Once unseen, a variable stays unseen for the action
                    // (a later assignment may not run).
                    let value = if value.contains(shell::UNSEEN_OUTPUT)
                        || variables
                            .get(name)
                            .is_some_and(|old| old.contains(shell::UNSEEN_OUTPUT))
                    {
                        shell::UNSEEN_OUTPUT.to_string()
                    } else {
                        value.to_string()
                    };
                    variables.insert(name.to_string(), value);
                }
            }
        }
        if let Some((index, name)) = &command {
            let args = &words[index + 1..];
            // Code named by a variable or substitution the classifier cannot
            // see: the command word (`$X`, `bash -c "$X"`), an interpreter's
            // inline code (`python3 -c "$X"`) or `eval "$x"`.
            // Inline code, or an argument after it (`sh -c 'eval "$1"' _ "$X"`).
            let inline_unseen = indirect::invocation(words).is_some_and(|invocation| {
                matches!(invocation.code, Code::Inline(Some(code))
                    if words
                        .iter()
                        // Attached code (`-c$X`) ends its option word.
                        .position(|word| word.ends_with(code))
                        .is_some_and(|at| unseen[at..].iter().any(|unseen| *unseen)))
            });
            let eval_unseen = name == "eval" && unseen[index + 1..].iter().any(|unseen| *unseen);
            if self.rebuilt == 0 && (unseen[*index] || inline_unseen || eval_unseen) {
                self.note(ProtectedActionKind::UnseenCode);
            }
            self.change_folder(name, args, folders, variables);
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
            if persistence_command(name, args) {
                self.note(ProtectedActionKind::Persistence);
            }
            for (credential_command, verbs) in CREDENTIAL_COMMANDS {
                if name == *credential_command
                    && args.iter().any(|word| verbs.contains(&word.as_str()))
                {
                    self.note(ProtectedActionKind::Credentials);
                }
            }
        }
        let stdin_fed =
            simple.pipe_from.is_some() || simple.stdin_from.is_some() || simple.reads_outer_output;
        if let Some((index, _)) = &command
            && let Some(kind) = outbound::classify(
                words,
                *index,
                stdin_fed,
                variables
                    .keys()
                    .any(|name| name.to_lowercase().ends_with("_proxy")),
            )
        {
            self.note(kind);
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
        let writer = command
            .as_ref()
            .map(|(index, name)| (*index, WRITERS.contains(&name.as_str())));
        let cwd = folders.cwd.clone();
        // Words resolve against the folder of a `-C`/`--directory` flag
        // (`tar -C ~ ...`, `git -C ~/.codex ...`) once one has been given.
        let mut folder = cwd.clone();
        let mut previous: Option<&str> = None;
        for (position, word) in words.iter().enumerate() {
            let written = simple.writes_to.contains(&position)
                || writer.is_some_and(|(index, writes)| writes && position > index);
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
            // Attached, assigned and volume forms (`-o/path`, `f=@/path`,
            // `~/.aws:/c`) are paths too.
            let pieces = std::iter::once(word.as_str()).chain(
                word.split(['=', ':'])
                    .filter(|piece| piece.len() < word.len()),
            );
            for candidate in pieces {
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
                    self.word_path(&resolved, candidate, depth);
                    if written && self.homes.is_persistence(&resolved) {
                        self.note(ProtectedActionKind::Persistence);
                    }
                    if recursive && !folder_value && self.homes.holds_a_home(&resolved) {
                        self.note(ProtectedActionKind::Credentials);
                    }
                }
            }
            if let Some(payload) = indirect::decoded_payload(word) {
                self.script(&payload, &cwd, depth + 1);
            }
        }
        self.scripts_of(simple, words, unseen, command.as_ref(), &cwd, depth);
    }

    /// Read what the command runs from files: its script, a local Python
    /// module, preloaded files, a redirected stdin, or the command itself when
    /// it is a path. A required path the classifier cannot resolve (it holds
    /// a variable) fails closed.
    fn scripts_of(
        &mut self,
        simple: &SimpleCommand,
        words: &[String],
        unseen: &[bool],
        command: Option<&(usize, String)>,
        cwd: &str,
        depth: usize,
    ) {
        let Some((index, name)) = command else {
            return;
        };
        let local = |word: &str| word.starts_with(['.', '/', '~']);
        let Some(invocation) = indirect::invocation(words) else {
            let word = &words[*index];
            if word.contains('/') {
                // A binary through an environment path (`$JAVA_HOME/bin/java`)
                // is not a script; an unseen one was already judged.
                if !word.contains('$') {
                    let file = self.homes.resolve(word, cwd);
                    self.script_file(&file, cwd, depth, Script::Required);
                }
            }
            return;
        };
        let shell =
            shell::SHELLS.contains(&name.as_str()) || matches!(name.as_str(), "source" | ".");
        match invocation.code {
            Code::File(file) => self.required_script(file, cwd, depth, shell),
            Code::Module(module) if module.contains('$') => {
                self.note(ProtectedActionKind::UnseenCode);
            }
            Code::Module(module) => {
                let module = module.replace('.', "/");
                for candidate in [format!("{module}.py"), format!("{module}/__main__.py")] {
                    let file = self.homes.resolve(&candidate, cwd);
                    self.not_shell += 1;
                    self.script_file(&file, cwd, depth, Script::IfPresent);
                    self.not_shell -= 1;
                }
            }
            Code::Stdin => {
                if let Some(at) = simple.stdin_from {
                    self.required_script(&words[at], cwd, depth, shell);
                }
                if simple.here_string.is_some_and(|at| unseen[at]) {
                    self.note(ProtectedActionKind::UnseenCode);
                }
            }
            Code::Inline(_) | Code::Tool => {}
        }
        for preload in invocation.preloads {
            if local(preload) {
                self.required_script(preload, cwd, depth, /*shell*/ false);
            } else {
                let file = self.homes.resolve(preload, cwd);
                self.not_shell += 1;
                self.script_file(&file, cwd, depth, Script::IfPresent);
                self.not_shell -= 1;
            }
        }
    }

    /// A script the command certainly runs: read in full or fail closed.
    fn required_script(&mut self, file: &str, cwd: &str, depth: usize, shell: bool) {
        if file.contains('$') || file.contains(shell::UNSEEN_OUTPUT) {
            self.note(ProtectedActionKind::UnseenCode);
            return;
        }
        let file = self.homes.resolve(file, cwd);
        if !shell {
            self.not_shell += 1;
        }
        self.script_file(&file, cwd, depth, Script::Required);
        if !shell {
            self.not_shell -= 1;
        }
    }

    /// Follow `cd`, `pushd` and `popd` (also behind `builtin`/`command`).
    fn change_folder(
        &self,
        name: &str,
        args: &[String],
        folders: &mut Folders,
        variables: &mut HashMap<String, String>,
    ) {
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
            variables.insert("PWD".to_string(), folders.cwd.clone());
            variables.insert("OLDPWD".to_string(), folders.previous.clone());
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
    /// later, `bash x.sh`, `./run`). Executables are recognised by their
    /// magic number and skipped. A required script the classifier cannot read
    /// in full (missing, written by the same command, not a regular file, too
    /// large, past the file limit, or where lookups are not allowed) fails
    /// closed; an optional one is skipped when it is not there.
    fn script_file(&mut self, file: &str, cwd: &str, depth: usize, need: Script) {
        use std::io::Read;
        let unreadable = |classifier: &mut Self, missing: bool| {
            if need == Script::Required || !missing {
                classifier.note(ProtectedActionKind::UnseenCode);
            }
        };
        if !self.homes.lookups_allowed(file) {
            return unreadable(self, /*missing*/ false);
        }
        // Check before opening: opening a FIFO or device would block.
        let Ok(metadata) = std::fs::metadata(file) else {
            return unreadable(self, /*missing*/ true);
        };
        if !metadata.is_file() {
            return unreadable(self, /*missing*/ false);
        }
        self.scripts_read += 1;
        if self.scripts_read > MAX_SCRIPT_FILES {
            return unreadable(self, /*missing*/ false);
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
            return unreadable(self, /*missing*/ false);
        };
        if !handle.metadata().is_ok_and(|metadata| metadata.is_file()) {
            return unreadable(self, /*missing*/ false);
        }
        let mut bytes = Vec::new();
        let mut reader = handle.take(indirect::SCRIPT_READ_LIMIT + 1);
        if reader.read_to_end(&mut bytes).is_err() {
            return unreadable(self, /*missing*/ false);
        }
        // Executables first: their size does not matter.
        if BINARY_MAGIC.iter().any(|magic| bytes.starts_with(magic)) {
            return;
        }
        if bytes.len() as u64 > indirect::SCRIPT_READ_LIMIT {
            return unreadable(self, /*missing*/ false);
        }
        // The shell skips NUL bytes; so does the scan.
        bytes.retain(|byte| *byte != 0);
        let text = String::from_utf8_lossy(&bytes);
        self.script(&text, cwd, depth + 1);
    }
}

/// Shell parameters that stand for the command's own arguments or status,
/// which are already in front of the classifier.
fn is_special_parameter(name: &str) -> bool {
    matches!(name, "@" | "*" | "#" | "?" | "$" | "!" | "-" | "0")
        || (name.len() == 1 && name.chars().all(|ch| ch.is_ascii_digit()))
}

/// Words with known `$NAME` variables replaced, left to right as the shell
/// reads them (`$a$b` is `a` then `b`), and for each word whether it holds a
/// value the classifier cannot see: a variable never assigned in this action
/// (other than special parameters) or one set from an unseen substitution.
fn substitute(
    command: &SimpleCommand,
    variables: &HashMap<String, String>,
) -> (Vec<String>, Vec<bool>) {
    command
        .words
        .iter()
        .map(|word| {
            let mut out = String::new();
            let mut unseen = word.contains(shell::UNSEEN_OUTPUT);
            let mut rest = word.as_str();
            while let Some(at) = rest.find('$') {
                out.push_str(&rest[..at]);
                let after = &rest[at + 1..];
                let len = match after.chars().next() {
                    Some(ch) if "@*#?$!-".contains(ch) || ch.is_ascii_digit() => ch.len_utf8(),
                    _ => after
                        .find(|ch: char| !(ch.is_ascii_alphanumeric() || ch == '_'))
                        .unwrap_or(after.len()),
                };
                let name = &after[..len];
                match variables.get(name) {
                    Some(value) if len > 0 => {
                        unseen |= value.contains(shell::UNSEEN_OUTPUT);
                        out.push_str(value);
                    }
                    _ => {
                        unseen |= len > 0 && !is_special_parameter(name);
                        out.push('$');
                        out.push_str(name);
                    }
                }
                rest = &after[len..];
            }
            out.push_str(rest);
            (out, unseen)
        })
        .unzip()
}

#[cfg(test)]
#[path = "tainted_action_tests.rs"]
mod tests;
