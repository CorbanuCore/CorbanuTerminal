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
use std::collections::HashSet;
use std::path::Path;

mod arithmetic;
mod code_variables;
mod git_helpers;
mod grammar;
mod indirect;
mod invocation;
mod launched_git;
mod named_shell;
mod nested;
mod outbound;
mod outputs;
mod paths;
mod quoted;
mod sed;
mod shadow;
mod shell;
/// #1296: also expands the Windows home spellings for protected-read
/// explanations (`protected_reads`).
pub(crate) mod windows_paths;
mod windows_shell;
mod work;

pub(crate) use paths::USER_PERSISTENCE;
pub(crate) use paths::WORKSPACE_AGENT_SETTINGS;
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
            // #841: also a read of Corbanu's own files, not only a change.
            Self::SecurityPolicy => {
                "access to Corbanu's own files or security settings (its config, rules, hooks, skills, history or sign-in state)"
            }
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

/// Appended to the prompt and the approvals-off refusal when
/// [`PostTaintAction::reads_stay_denied`] holds.
const AGGRESSIVE_READS_STAY_DENIED: &str = " Under security level Aggressive, credential files, \
     the vault and the Corbanu home stay unreadable to agent commands even if you approve, and \
     that is never granted, not even once; to allow it, switch to Permissive in /security (it \
     applies after a restart).";

impl PostTaintAction {
    /// Shown in the approval prompt so the human sees why it was asked again.
    pub(crate) fn approval_reason(&self) -> String {
        let mut reason = format!(
            "Security level: this is {} after untrusted content entered the session. \
             Approve only if you asked for it; a cached or automatic approval does not count.",
            self.kind.describe()
        );
        if self.reads_stay_denied() {
            reason.push_str(AGGRESSIVE_READS_STAY_DENIED);
        }
        reason
    }

    /// #442: under Aggressive, what this action protects is denied by the
    /// sandbox whatever the person answers, and no grant lifts it, so none
    /// is offered for it.
    pub(crate) fn reads_stay_denied(&self) -> bool {
        self.state.level == SecurityLevel::Aggressive
            && !matches!(self.state.policy, PolicyBinding::Unavailable)
            && matches!(
                self.kind,
                ProtectedActionKind::Credentials | ProtectedActionKind::Vault
            )
    }

    pub(crate) fn approvals_off_rejection(&self) -> String {
        let mut reason = format!(
            "Not run: Security level {}: this is {} after untrusted content entered the session, \
             so it needs your approval, and approvals are off (approval_policy = never). \
             Review the action and turn approval prompts on (/permissions, or approval_policy \
             in config.toml) before asking again. Other security protections still apply.",
            level_name(self.state.level),
            self.kind.describe()
        );
        // #941 review: approving would not make these reads work under Aggressive.
        if self.reads_stay_denied() {
            reason.push_str(AGGRESSIVE_READS_STAY_DENIED);
        }
        reason
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
/// macOS `security` verbs that read or change keychain items.
pub(crate) const KEYCHAIN_VERBS: &[&str] = &[
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
];
/// Credential commands: the command name plus words that make it a secret read.
const CREDENTIAL_COMMANDS: &[(&str, &[&str])] = &[
    ("security", KEYCHAIN_VERBS),
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
pub(super) const WRITERS: &[&str] = &[
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
/// Bytes of variable values one classification substitutes (#1065): a
/// value doubled on each step (`a=$a$a`) or used many times would otherwise
/// grow without bound. Past it the line asks, and values up to
/// `MAX_SMALL_VALUE` bytes are still substituted, up to the same number of
/// bytes again, so the line still names what it reaches; a larger value is
/// judged once on its own.
const MAX_SUBSTITUTED_BYTES: usize = 1024 * 1024;
const MAX_SMALL_VALUE: usize = 64;
/// Bytes of command text, and command lines, one classification lexes in
/// all, nested scripts, decoded payloads and script files included (#1065);
/// more fail closed.
const MAX_CLASSIFIED_BYTES: usize = 4 * 1024 * 1024;
const MAX_CLASSIFIED_LINES: usize = 4096;
/// Folders `pushd` may stack, and bytes of a folder name the classifier
/// follows (#1065); past them the line asks and the folder stays. The
/// stack is shared, not copied, by the readings of quoted text.
const MAX_FOLDER_STACK: usize = 64;
const MAX_FOLDER_BYTES: usize = 1024;
/// `-C`-style folders one command chains (#1141); more fail closed.
const MAX_CHAINED_FOLDERS: usize = 8;
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

/// [`classify`] for an action the host runs exactly as its argv says: a
/// shell command line runs under the shell it names (not, for example,
/// under the zsh exec bridge, which runs any `-c` script under zsh). D85:
/// then a line that provably never reaches zsh skips the zsh-only checks.
pub(crate) fn classify_as_named(
    action: &ApprovalAction,
    codex_home: &Path,
) -> Option<ProtectedActionKind> {
    classify_as_named_with(action, codex_home, dirs::home_dir().as_deref())
}

pub(super) fn classify_with(
    action: &ApprovalAction,
    codex_home: &Path,
    user_home: Option<&Path>,
) -> Option<ProtectedActionKind> {
    classified(action, codex_home, user_home, /*as_named*/ false).found
}

pub(super) fn classify_as_named_with(
    action: &ApprovalAction,
    codex_home: &Path,
    user_home: Option<&Path>,
) -> Option<ProtectedActionKind> {
    classified(action, codex_home, user_home, /*as_named*/ true).found
}

/// The classifier after judging the action, with its counters. `as_named`:
/// the host runs the action as its argv names (see [`classify_as_named`]).
/// #1240: on Windows the action is also judged under Windows path rules
/// (`windows_paths`, `windows_shell`), and the stronger finding is kept:
/// the first judgment is the same as on every other platform, so the
/// Windows view only adds findings.
fn classified(
    action: &ApprovalAction,
    codex_home: &Path,
    user_home: Option<&Path>,
    as_named: bool,
) -> Classifier {
    let mut classifier = judged(action, Homes::new(codex_home, user_home), as_named);
    if cfg!(windows) {
        let windows = judged(action, Homes::windows(codex_home, user_home), as_named);
        if let Some(kind) = windows.found {
            classifier.note(kind);
        }
    }
    classifier
}

/// #1240: [`classify_with`] of a command run in `cwd` as on Windows, on any
/// platform (an action cannot hold a Windows folder elsewhere). Filesystem
/// lookups under Windows rules happen only on Windows.
#[cfg(test)]
pub(super) fn classify_as_windows(
    command: &[String],
    cwd: &str,
    codex_home: &Path,
    user_home: Option<&Path>,
) -> Option<ProtectedActionKind> {
    let mut classifier =
        Classifier::new(Homes::new(codex_home, user_home), /*avoids_zsh*/ false);
    classifier.judge_command(command, cwd);
    let mut windows = Classifier::new(
        Homes::windows(codex_home, user_home),
        /*avoids_zsh*/ false,
    );
    windows.judge_command(command, cwd);
    if let Some(kind) = windows.found {
        classifier.note(kind);
    }
    classifier.found
}

/// One judgment of the action, under the path rules of `homes`.
fn judged(action: &ApprovalAction, homes: Homes, as_named: bool) -> Classifier {
    let avoids_zsh = as_named
        && match action {
            ApprovalAction::Shell { command, .. } | ApprovalAction::ExecCommand { command, .. } => {
                named_shell::avoids_zsh(command) || named_shell::grammar_proves_data(command)
            }
            ApprovalAction::ApplyPatch { .. } => false,
        };
    let mut classifier = Classifier::new(homes, avoids_zsh);
    match action {
        ApprovalAction::Shell { command, cwd, .. }
        | ApprovalAction::ExecCommand { command, cwd, .. } => {
            classifier.judge_command(command, &path_text(cwd));
        }
        ApprovalAction::ApplyPatch {
            files, patch, cwd, ..
        } => {
            let cwd = path_text(cwd);
            classifier.homes.allow_lookups_under(&cwd);
            for file in files {
                work::step();
                let file = path_text(file);
                classifier.path(&file);
                if classifier.homes.is_persistence(&file) {
                    classifier.note(ProtectedActionKind::Persistence);
                }
            }
            // Commands written into a file that is run later are judged now,
            // whether the file looks runnable by name or by where it leads.
            for (file, body) in indirect::patch_additions(patch) {
                work::step();
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
    classifier
}

impl Classifier {
    fn new(homes: Homes, avoids_zsh: bool) -> Self {
        Self {
            avoids_zsh,
            homes,
            found: None,
            scripts_read: 0,
            rebuilt: 0,
            not_shell: 0,
            sed_scope: None,
            ran: Vec::new(),
            ran_unknown: false,
            written: Vec::new(),
            nested_scripts: 0,
            unknown_folder: 0,
            launched: launched_git::Launch::default(),
            substitution_budget: Budget {
                left: MAX_SUBSTITUTED_BYTES,
                small_left: MAX_SUBSTITUTED_BYTES,
            },
            classified_bytes: 0,
            classified_lines: 0,
            compared: shadow::Budget::new(),
            windows_decoded: HashSet::new(),
            windows_budget: windows_shell::RESOLVE_BUDGET,
        }
    }

    /// Judge a command line run in `cwd`.
    fn judge_command(&mut self, command: &[String], cwd: &str) {
        self.homes.allow_lookups_under(cwd);
        self.path(cwd);
        self.command(command, cwd, /*depth*/ 0);
    }
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
/// cron, launchd and systemd user units (PF-23-S02; git config is judged by
/// [`GitConfigWrites`]), given the words after `name` (`later` from `from`).
fn persistence_command(name: &str, later: &mut LaterWords, from: usize) -> bool {
    match name {
        "crontab" => !later.holds(from, &["-l"]),
        "launchctl" => later.holds(from, &["load", "bootstrap", "submit", "enable"]),
        "systemctl" => {
            later.holds(from, &["--user"]) && later.holds(from, &["enable", "link", "edit"])
        }
        _ => false,
    }
}

/// #1275: what the words after each word of a simple command hold, for the
/// checks made on each command word (credential, persistence and Corbanu
/// CLI commands). Each is worked out once, on its first use, so judging
/// every copy of a command word (`gh gh gh …`) stays linear.
struct LaterWords<'a> {
    /// The command's words, lowercase.
    words: &'a [String],
    /// For a set of words, whether each word or one after it is in it.
    holding: Vec<(&'static [&'static str], Vec<bool>)>,
    /// Whether each word or one after it widens a Corbanu CLI run's policy.
    policy_flags: Option<Vec<bool>>,
    /// The CLI subcommand of the words from each word on.
    subcommands: Option<Vec<Option<usize>>>,
}

impl<'a> LaterWords<'a> {
    fn new(words: &'a [String]) -> Self {
        Self {
            words,
            holding: Vec::new(),
            policy_flags: None,
            subcommands: None,
        }
    }

    /// Whether `words[from..]` holds one of `set`.
    fn holds(&mut self, from: usize, set: &'static [&'static str]) -> bool {
        let words = self.words;
        // Sets are constants: there are a few of them.
        let at = match self.holding.iter().position(|(known, _)| {
            work::step();
            *known == set
        }) {
            Some(at) => at,
            None => {
                let held = suffix_any(words.len(), |at| set.contains(&words[at].as_str()));
                self.holding.push((set, held));
                self.holding.len() - 1
            }
        };
        self.holding[at].1.get(from).copied().unwrap_or(false)
    }

    /// Whether a Corbanu CLI run given `words[from..]` has its policy
    /// widened by a flag or a `-c` override.
    fn policy_flag(&mut self, from: usize) -> bool {
        let words = self.words;
        let flags = self.policy_flags.get_or_insert_with(|| {
            let policy_key =
                |value: &str| POLICY_CONFIG_KEYS.iter().any(|key| value.starts_with(key));
            suffix_any(words.len(), |at| {
                let word = &words[at];
                CLI_POLICY_FLAGS.iter().any(|flag| word.contains(flag))
                    || word.starts_with("--dangerously")
                    || word.strip_prefix("--config=").is_some_and(policy_key)
                    || (word.len() > 2 && word.strip_prefix("-c").is_some_and(policy_key))
                    || (matches!(word.as_str(), "-c" | "--config")
                        && words.get(at + 1).is_some_and(|value| policy_key(value)))
            })
        });
        flags.get(from).copied().unwrap_or(false)
    }

    /// The CLI subcommand of `words[from..]`: the first word that is neither
    /// a flag nor a flag's value.
    fn subcommand(&mut self, from: usize) -> Option<&'a str> {
        let words = self.words;
        let subcommands = self.subcommands.get_or_insert_with(|| {
            let mut subcommands = vec![None; words.len() + 2];
            for (at, word) in words.iter().enumerate().rev() {
                work::step();
                subcommands[at] = if !word.starts_with('-') {
                    Some(at)
                } else if !word.contains('=') && CLI_VALUE_FLAGS.contains(&word.as_str()) {
                    subcommands[at + 2]
                } else {
                    subcommands[at + 1]
                };
            }
            subcommands
        });
        subcommands
            .get(from)
            .copied()
            .flatten()
            .map(|at| words[at].as_str())
    }
}

/// For each of `len` words and the end, whether `check` holds for it or a
/// word after it.
fn suffix_any(len: usize, check: impl Fn(usize) -> bool) -> Vec<bool> {
    let mut held = vec![false; len + 1];
    for at in (0..len).rev() {
        work::step();
        held[at] = held[at + 1] || check(at);
    }
    held
}

/// #1159: whether `git config` writes global config or a git key that runs
/// a program, for each git word of a simple command (`git`, `git.exe`, a
/// dashed helper such as `git-config` that gives its subcommand apart),
/// or opens it in an editor (#1255: `git config -e`). What follows each
/// word is worked out once, on the first git word, so judging every git
/// word of a line (`git config git config …`) stays linear.
struct GitConfigWrites<'a> {
    /// The command's words, lowercase.
    words: &'a [String],
    /// #1283: which of them the quote-aware reading cannot spell as a name,
    /// and which it cannot know (see `Classifier::word_marks`); empty when
    /// unknown.
    hidden: &'a [bool],
    unchecked: &'a [bool],
    seen: Option<FollowingWords>,
}

/// For each word of a command (and its end), what it and the words after it
/// hold.
struct FollowingWords {
    /// Where the first `config` is.
    next_config: Vec<Option<usize>>,
    /// Whether one is `--global`/`--system` or a key that runs a program.
    persisting: Vec<bool>,
    /// Whether, as the words after `config`, they only read.
    reads: Vec<bool>,
    /// #1255: whether, as the words after `config`, they open an editor.
    edits: Vec<bool>,
    /// #1255: whether a `config` among them is followed by words that open
    /// an editor (any of them: the first may be an option's value, as in
    /// `git -C config config -e`).
    config_edits: Vec<bool>,
    /// #1283: whether one of them is unchecked, or may open an editor.
    unchecked: Vec<bool>,
    editor_words: Vec<bool>,
}

impl<'a> GitConfigWrites<'a> {
    fn new(words: &'a [String], hidden: &'a [bool], unchecked: &'a [bool]) -> Self {
        Self {
            words,
            hidden,
            unchecked,
            seen: None,
        }
    }

    /// Whether `git <subcommand> words[from..]` (or `git words[from..]`)
    /// writes such config.
    fn persists(&mut self, from: usize, subcommand: Option<&str>) -> bool {
        let words = self.words;
        let unchecked = self.unchecked;
        let seen = self.seen.get_or_insert_with(|| {
            let mut next_config = vec![None; words.len() + 1];
            let mut persisting = vec![false; words.len() + 1];
            let edits = outputs::git_config_edits_each(words);
            let mut config_edits = vec![false; words.len() + 1];
            for (at, word) in words.iter().enumerate().rev() {
                work::step();
                next_config[at] = if word == "config" {
                    Some(at)
                } else {
                    next_config[at + 1]
                };
                config_edits[at] = config_edits[at + 1] || (word == "config" && edits[at + 1]);
                persisting[at] = persisting[at + 1]
                    || matches!(word.as_str(), "--global" | "--system")
                    || GIT_RUN_KEYS.contains(&word.as_str())
                    || word.starts_with("alias.");
            }
            FollowingWords {
                unchecked: suffix_any(words.len(), |at| {
                    unchecked.get(at).copied().unwrap_or(false)
                }),
                editor_words: suffix_any(words.len(), |at| outputs::may_open_editor(&words[at])),
                next_config,
                persisting,
                // #1141: a reading word given as a value (`git config -f f k
                // get`) does not make it a read.
                reads: outputs::git_config_reads_each(words),
                edits,
                config_edits,
            }
        });
        let flag = |flags: &[bool], at: usize| flags.get(at).copied().unwrap_or(false);
        let edits = flag(&seen.config_edits, from)
            || (subcommand == Some("config") && flag(&seen.edits, from));
        let start = if subcommand == Some("config") {
            from
        } else {
            match seen.next_config.get(from).copied().flatten() {
                Some(at) => at + 1,
                None => return edits,
            }
        };
        // #1283: git or its `config` spelled by text the quote-aware reading
        // cannot see (`{git,config}`, `$g config`): that reading, which keeps
        // quoted values whole, cannot check what this one makes of them.
        let hidden = |at: usize| self.hidden.get(at).copied().unwrap_or(false);
        if hidden(from - 1) || hidden(start - 1) {
            return true;
        }
        // A word it cannot know (`"$k"` with `k=core.pager`): judged here
        // only, where quoted text is split, so a read is not trusted and any
        // editor option counts.
        if flag(&seen.unchecked, from) {
            return edits || flag(&seen.editor_words, start) || flag(&seen.persisting, start);
        }
        edits || (start < seen.reads.len() && !seen.reads[start] && seen.persisting[start])
    }
}

/// The folders a command's words resolve against: the last `-C`-style
/// folder (resolved against the working folder), each folder of the chain
/// (each resolved against the one before) and the working folder, without
/// repeats.
fn folder_bases(folder: &str, chained: &[String], cwd: &str) -> Vec<String> {
    let mut bases = vec![folder.to_string()];
    for base in chained.iter().map(String::as_str).chain([cwd]) {
        work::step();
        if !bases.iter().any(|known| known == base) {
            bases.push(base.to_string());
        }
    }
    bases
}

/// Corbanu CLI flags that take the next word as their value.
const CLI_VALUE_FLAGS: &[&str] = &[
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

/// Bytes of variable values left to substitute (#1065): any value, then
/// only small ones once that runs out.
struct Budget {
    left: usize,
    small_left: usize,
}

/// Where the shell is while it runs a script: the folder, the previous one
/// (`cd -`) and the `pushd` stack.
struct Folders {
    cwd: String,
    previous: String,
    stack: std::rc::Rc<Vec<String>>,
    /// A `cd`/`pushd` past the folder limits was not followed (#1065).
    unfollowed: bool,
    /// #1283: a `cd`/`pushd` took a word the quote-aware reading cannot
    /// know (`D=~; cd "$D"`), so that reading's folder may be wrong.
    unchecked: bool,
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
    /// #841: the sed programs of the command line being judged, when a real
    /// shell parse of it is available.
    sed_scope: Option<SedScope>,
    /// #885: script files the action runs (as resolved), and whether it runs
    /// one whose name the classifier cannot see.
    ran: Vec<String>,
    ran_unknown: bool,
    /// #885: files the action's command lines write (`None`: one the
    /// classifier cannot name).
    written: Vec<Option<String>>,
    /// #885: nested shell scripts judged so far.
    nested_scripts: usize,
    /// #1283: inside a nested script whose folder the quote-aware reading
    /// cannot know (run after `cd "$d"`, or by `env -C`).
    unknown_folder: usize,
    /// #1268: inside a script a launcher runs with its input added after
    /// it (`xargs env -S'git config'`) or put into its text (`xargs -I% sh
    /// -c 'git config %'`, where any word may be input).
    launched: launched_git::Launch,
    /// #1065: bytes of variable values left to substitute.
    substitution_budget: Budget,
    /// #1065: bytes of command text, and command lines, lexed so far.
    classified_bytes: usize,
    classified_lines: usize,
    /// D85: the action's own command line never reaches zsh
    /// ([`named_shell::avoids_zsh`]); nested lines are not covered.
    avoids_zsh: bool,
    /// #932: comparisons left with the grammar.
    compared: shadow::Budget,
    /// #1240: PowerShell encoded commands the Windows view judged, with
    /// the folder each was judged in: once each.
    windows_decoded: HashSet<(String, String)>,
    /// #1240: bytes of words and folders the Windows view may still
    /// resolve; past it the action fails closed.
    windows_budget: usize,
}

/// #841 (MOD-06): the word classifier splits quoted text into commands on
/// purpose (it over-approximates the shell), so a sed program such as
/// `'/f.write(...)/a\ ...'` yields "commands" like `/a` that name no file.
/// A word is sed's data, not a command the shell runs, when a real shell
/// parse shows the line is plain commands with literal words, the word occurs
/// in a sed program that only appends, inserts or changes text (text sed
/// never runs), and it occurs in no other word of the line.
struct SedScope {
    /// The line's text-only sed programs.
    programs: Vec<String>,
    /// Every other word of the line, command words included.
    others: Vec<String>,
}

impl SedScope {
    /// `None` (nothing is sed's data) unless the parse is plain, a sed
    /// command there has a text-only program, and the line's other words
    /// carry no backslash or brace.
    fn of(command: &[String]) -> Option<Self> {
        let commands = codex_shell_command::bash::parse_shell_lc_plain_commands(command)?;
        let piped = codex_shell_command::bash::extract_bash_command(command)
            .is_some_and(|(_, script)| script.contains('|'));
        let mut scope = Self {
            programs: Vec::new(),
            others: Vec::new(),
        };
        // A line that could change what `sed` means (an assignment such as
        // `PATH=…`, `export`, `alias`, `hash`, `enable`, sourcing) gives
        // nothing up.
        let redefines = commands.iter().any(|words| {
            words.first().is_some_and(|word| {
                word.contains('=')
                    || matches!(
                        word.as_str(),
                        "export"
                            | "alias"
                            | "hash"
                            | "enable"
                            | "set"
                            | "declare"
                            | "typeset"
                            | "readonly"
                            | "source"
                            | "."
                            | "builtin"
                            | "command"
                    )
            })
        });
        if redefines {
            return None;
        }
        for words in commands {
            work::step();
            // Only `sed` or `gsed` as written, not a path to another program.
            let sed = words
                .first()
                .is_some_and(|word| matches!(word.as_str(), "sed" | "gsed"));
            // Only plain flags: no attached or long-form program (`-es…`,
            // `-ee`, `--expression=`) and no script file (`-f`, `--file`);
            // `e` only ends a cluster.
            let plain_flags = words.iter().skip(1).all(|word| {
                let Some(cluster) = word.strip_prefix('-') else {
                    return true;
                };
                let flags = cluster.strip_suffix('e').unwrap_or(cluster);
                !cluster.is_empty() && flags.chars().all(|flag| "nEirsuz".contains(flag))
            });
            if sed && plain_flags {
                // The programs: each word after `-e`, or else the first word
                // that is not a flag; the rest are files.
                let with_e = words
                    .iter()
                    .any(|word| word.starts_with('-') && word.contains('e'));
                // #885: text sed saves (in place or through a pipe) may be
                // run later in the line, so only the address and command
                // letter of such a program are data.
                let saves = piped
                    || words
                        .iter()
                        .skip(1)
                        .any(|word| word.starts_with('-') && word.contains('i'));
                // macOS `sed -i ''`: the empty word after `-i` is its suffix.
                let mut program_next = false;
                let mut suffix_next = false;
                let mut positional_taken = false;
                for (index, word) in words.into_iter().enumerate() {
                    work::step();
                    let suffix = std::mem::take(&mut suffix_next) && word.is_empty();
                    let program = index > 0
                        && !suffix
                        && !word.starts_with('-')
                        && if with_e {
                            program_next
                        } else {
                            !std::mem::replace(&mut positional_taken, true)
                        };
                    program_next = word.starts_with('-') && word.ends_with('e');
                    suffix_next = word.starts_with('-') && word.ends_with('i');
                    if program && let Some(prefix) = sed_text_prefix(&word) {
                        let data = if saves {
                            word[..prefix].to_string()
                        } else {
                            word
                        };
                        scope.programs.push(data);
                    } else {
                        scope.others.push(word);
                    }
                }
            } else {
                scope.others.extend(words);
            }
        }
        // The word lexer drops backslashes and expands braces; the parse
        // keeps both, so such a word could hide a run of a data word.
        let usable = !scope.programs.is_empty()
            && !scope.others.iter().any(|word| word.contains(['\\', '{']));
        usable.then_some(scope)
    }

    fn is_data(&self, word: &str) -> bool {
        !word.is_empty()
            && self.programs.iter().any(|program| program.contains(word))
            && !self.others.iter().any(|other| other.contains(word))
    }
}

/// #841: whether a sed program is one `a`, `i` or `c` command (append,
/// insert or change text) after at most one address or range of line
/// numbers, `$` or `/regex/`: its text runs to the first newline not
/// escaped by a backslash, and only blank space may follow it. Sed never
/// runs such text; anything else (`s`, `e`, several commands) is not
/// text-only.
#[cfg(test)]
fn sed_text_only(program: &str) -> bool {
    sed_text_prefix(program).is_some()
}

/// For a text-only sed program (see `sed_text_only`), the length of its
/// address and command letter: the part that is not text sed outputs.
fn sed_text_prefix(program: &str) -> Option<usize> {
    fn address(rest: &str) -> Option<&str> {
        if let Some(regex) = rest.strip_prefix('/') {
            let mut escaped = false;
            for (index, ch) in regex.char_indices() {
                work::step();
                match ch {
                    _ if escaped => escaped = false,
                    '\\' => escaped = true,
                    '\n' => return None,
                    '/' => return Some(&regex[index + 1..]),
                    _ => {}
                }
            }
            return None;
        }
        if let Some(rest) = rest.strip_prefix('$') {
            return Some(rest);
        }
        Some(rest.trim_start_matches(|ch: char| ch.is_ascii_digit()))
    }
    let mut rest = address(program.trim_start_matches([' ', '\t']))?;
    if let Some(second) = rest.strip_prefix(',') {
        match address(second) {
            Some(after) if after.len() < second.len() => rest = after,
            _ => return None,
        }
    }
    let rest = rest.trim_start_matches([' ', '\t']);
    let text = rest.strip_prefix(['a', 'i', 'c'])?;
    let prefix = program.len() - text.len();
    let mut escaped = false;
    for (index, ch) in text.char_indices() {
        work::step();
        match ch {
            _ if escaped => escaped = false,
            '\\' => escaped = true,
            '\n' => return text[index + 1..].trim().is_empty().then_some(prefix),
            _ => {}
        }
    }
    Some(prefix)
}

/// A file read as code.
enum FileText {
    Missing,
    /// Not a regular file, too large, past the file limit or where lookups
    /// are not allowed.
    Unreadable,
    /// An executable (by its magic number).
    Binary,
    Text(String),
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
        // A command line too long to classify fails closed, as does more
        // text in all than one classification lexes.
        let bytes = command.iter().map(String::len).sum::<usize>();
        self.classified_bytes = self.classified_bytes.saturating_add(bytes);
        self.classified_lines += 1;
        if bytes > MAX_COMMAND_BYTES
            || self.classified_bytes > MAX_CLASSIFIED_BYTES
            || self.classified_lines > MAX_CLASSIFIED_LINES
        {
            self.note(ProtectedActionKind::UnseenCode);
            return;
        }
        // #1240: the same text as cmd and PowerShell read it.
        if self.homes.is_windows() {
            self.windows_text(command, cwd, depth);
        }
        // #841: this line's own sed programs (nested lines have theirs).
        let outer_scope = std::mem::replace(&mut self.sed_scope, SedScope::of(command));
        let lexed = shell::simple_commands(command);
        let pipe_sources: std::collections::HashSet<usize> = lexed
            .commands
            .iter()
            .filter_map(|command| command.pipe_from)
            .collect();
        // #1012: bash `${v@P}` expands the value as a prompt, running the
        // substitutions in it, like `eval`. Also in non-shell and rebuilt
        // text, which may reach a shell (`os.system`); the lexed words are
        // checked below.
        if (lexed.incomplete && self.not_shell == 0 && self.rebuilt == 0)
            || command.iter().any(|arg| shell::prompt_expansion(arg))
            || lexed.too_deep
            || indirect::opaque_execution(&lexed.commands)
        {
            self.note(ProtectedActionKind::UnseenCode);
        }
        let mut folders = Folders {
            cwd: cwd.to_string(),
            previous: cwd.to_string(),
            stack: std::rc::Rc::default(),
            unfollowed: false,
            unchecked: self.unknown_folder > 0,
        };
        let mut variables: HashMap<String, String> = HashMap::new();
        for (name, value) in [
            ("PWD", Some(cwd)),
            ("HOME", self.homes.user_home()),
            ("CODEX_HOME", Some(self.homes.codex_home())),
            // #442: the same home under its other names.
            ("CORBANU_HOME", Some(self.homes.codex_home())),
            ("PFTERMINAL_HOME", Some(self.homes.codex_home())),
        ] {
            work::step();
            if let Some(value) = value {
                variables.insert(name.to_string(), value.to_string());
            }
        }
        let mut lexed_seds = 0;
        let mut lexed_shells = 0;
        let mut arithmetic = arithmetic::Arithmetic::new(command);
        // Values too large to substitute, judged once each (by name and
        // length) while the lexing limits allow.
        let mut judged: HashSet<(String, usize)> = HashSet::new();
        // #1268: the word lexer drops braces, so a launcher's placeholders
        // (`{}`, `{.}`) are only seen in the text as written.
        let braces = command.iter().any(|arg| arg.contains('{'));
        let mut loops = launched_git::PositionalLoops::default();
        for (index, simple) in lexed.commands.iter().enumerate() {
            work::step();
            let (words, unseen, skipped) =
                substitute(simple, &variables, &mut self.substitution_budget);
            if self.substitution_budget.left == 0 {
                self.note(ProtectedActionKind::UnseenCode);
            }
            for name in skipped {
                work::step();
                let Some(value) = variables.get(&name) else {
                    continue;
                };
                if !judged.insert((name, value.len())) {
                    continue;
                }
                if value.len() > MAX_COMMAND_BYTES
                    || self.classified_bytes.saturating_add(value.len()) > MAX_CLASSIFIED_BYTES
                {
                    self.note(ProtectedActionKind::UnseenCode);
                    continue;
                }
                self.script(value, &folders.cwd, depth + 1);
            }
            // Decoded, quote-removed and with known variables substituted:
            // `eval "echo \${v@"P"}"`, `$'\x40'`, `p=@P; eval "echo \${v$p}"`.
            if words.iter().any(|word| shell::prompt_expansion(word)) {
                self.note(ProtectedActionKind::UnseenCode);
            }
            arithmetic.note(simple, &words);
            lexed_seds += sed::positions(&words).len();
            lexed_shells += nested::shell_commands(&words);
            let feeds_pipe = pipe_sources.contains(&index);
            self.simple_command(
                simple,
                &words,
                &unseen,
                feeds_pipe,
                braces,
                &mut folders,
                &mut variables,
                &mut loops,
                depth,
            );
        }
        // #937: arithmetic on a variable filled from command output or
        // `read` runs the substitutions in its value, like `eval`.
        // #1028: zsh `e`/`+` glob qualifiers run code the classifier does
        // not read as commands; #1084: so may a value under glob
        // substitution. D85: not on the action's own line when it never
        // reaches zsh. #1142: a variable named at run time may be one git
        // runs (`let "$n=1"`, `${(P)n=x}`).
        let zsh_possible = !(depth == 0 && self.avoids_zsh);
        if (self.rebuilt == 0
            && self.not_shell == 0
            && (arithmetic.runs_unseen(command)
                || arithmetic.assigns_unseen_name(command)
                || (zsh_possible
                    && (arithmetic.runs_qualifier_code(command)
                        || arithmetic.uses_glob_substitution(command)))))
            || folders.unfollowed
        {
            self.note(ProtectedActionKind::UnseenCode);
        }
        self.path(&folders.cwd);
        // Strings assembled from literals by inline code (`'~/.co' + 'dex'`).
        for arg in command {
            work::step();
            for joined in indirect::joined_literals(arg) {
                work::step();
                let resolved = self.homes.resolve(&joined, &folders.cwd);
                self.word_path(&resolved, &joined, depth);
                self.rebuilt += 1;
                self.script(&joined, &folders.cwd, depth + 1);
                self.rebuilt -= 1;
            }
        }
        // #885: nested shells' scripts and sed programs, read with quotes.
        self.quoted_pass(command, cwd, depth, (lexed_seds, lexed_shells));
        self.sed_scope = outer_scope;
    }

    #[allow(clippy::too_many_arguments)]
    fn simple_command(
        &mut self,
        simple: &SimpleCommand,
        words: &[String],
        unseen: &[bool],
        feeds_pipe: bool,
        braces: bool,
        folders: &mut Folders,
        variables: &mut HashMap<String, String>,
        loops: &mut launched_git::PositionalLoops,
        depth: usize,
    ) {
        let lower: Vec<String> = words.iter().map(|word| word.to_lowercase()).collect();
        let (hidden, mut unchecked) = self.word_marks(simple, words, variables);
        loops.read(&simple.words);
        let looped = loops.marks(&simple.words);
        // #1283: a loop variable over the positional parameters is a value
        // neither reading knows, wherever its quoting takes effect.
        for (unchecked, looped) in unchecked.iter_mut().zip(&looped) {
            work::step();
            *unchecked |= *looped;
        }
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
                work::step();
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
            folders.unchecked |= matches!(name.as_str(), "cd" | "pushd")
                && unchecked[index + 1..].iter().any(|word| *word);
            self.change_folder(name, args, folders, variables);
        }
        let mut git_config = GitConfigWrites::new(&lower, &hidden, &unchecked);
        let mut launched_git = launched_git::LaunchedGit::new(
            words,
            &lower,
            braces,
            self.launched,
            launched_git::Marks {
                hidden: &hidden,
                unchecked: &unchecked,
                looped: &looped,
            },
        );
        let mut later = LaterWords::new(&lower);
        // Not only at command position: wrappers (`nice -n 5`, `sudo -u x`,
        // `timeout 9`, `eval`, `npx`, nested `sh -c`) put the real command later.
        for (index, word) in lower.iter().enumerate() {
            work::step();
            let name = basename(word);
            let imported = index > 0 && matches!(lower[index - 1].as_str(), "from" | "import");
            if CLI_NAMES.contains(&name) && !imported {
                self.cli(&mut later, index + 1);
            }
            // #1159: git under another name (`git.exe`), or a dashed helper
            // (`/usr/lib/git-core/git-config`) that runs `git <subcommand>`.
            // #1276: or a finder's exec option with git attached
            // (`fd --exec=git`).
            let git_word = launched_git.attached(index).unwrap_or(word);
            let is_git = basename(git_word) == "git";
            let helper = git_helpers::git_name(git_word);
            let subcommand = helper.as_ref().and_then(Option::as_deref);
            if persistence_command(name, &mut later, index + 1)
                || ((is_git || helper.is_some()) && git_config.persists(index + 1, subcommand))
            {
                self.note(ProtectedActionKind::Persistence);
            }
            // #1268: git given words from a launcher's input (`xargs git`).
            if let Some(kind) = launched_git.judge(index, helper.as_ref()) {
                self.note(kind);
            }
            launched_git.step(index);
            // A helper named by text the classifier cannot see may be any
            // subcommand (`git-$x`, `git-conf*`), like `git $x`.
            if subcommand.is_some_and(|subcommand| {
                subcommand.contains(['$', '*', '?', '['])
                    || subcommand.contains(shell::UNSEEN_OUTPUT)
                    || simple
                        .words
                        .get(index)
                        .is_some_and(|raw| basename(raw).contains('$'))
            }) {
                self.note(ProtectedActionKind::UnseenCode);
            }
            // #1142: `git -c core.pager=…` runs the value (args as written).
            // A dashed helper takes no git options.
            let git_args = (
                &words[index + 1..],
                simple.words.get(index + 1..).unwrap_or_default(),
            );
            if (is_git && code_variables::git_runs_code_after_git(git_args.0, git_args.1))
                || (helper == Some(None) && code_variables::git_runs_code(git_args.0, git_args.1))
            {
                self.note(ProtectedActionKind::UnseenCode);
            }
            for (credential_command, verbs) in CREDENTIAL_COMMANDS {
                work::step();
                if name == *credential_command && later.holds(index + 1, verbs) {
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
        // #1142: setting a variable whose value git or a pager runs.
        if code_variables::sets_code_variable(simple, words, command.as_ref()) {
            self.note(ProtectedActionKind::UnseenCode);
        }
        // #1141: an output option makes a writer of its command, and its
        // value is a file written (`git log --output=~/.zshrc`, `-o.zshrc`).
        let outputs = command.as_ref().and_then(|(index, name)| {
            // #1283: a `config` the quote-aware reading cannot spell.
            // A wrapper's words count too (`env -C "$d" git`), and so does a
            // folder changed through one (`cd "$d"`).
            let unchecked =
                hidden[*index] || folders.unchecked || unchecked.iter().any(|word| *word);
            outputs::output_files(name, &words[index + 1..], unchecked)
        });
        if command
            .as_ref()
            .is_some_and(|(index, name)| outputs::joins_names(name, &words[index + 1..]))
        {
            self.note(ProtectedActionKind::Persistence);
        }
        let writer = command.as_ref().map(|(index, name)| {
            (
                *index,
                WRITERS.contains(&name.as_str()) || outputs.is_some(),
            )
        });
        let cwd = folders.cwd.clone();
        // Words resolve against the folder of a `-C`/`--directory` flag
        // (`tar -C ~ ...`, `git -C ~/.codex ...`) once one has been given.
        let mut folder = cwd.clone();
        // #1141: each relative `-C` resolves against the one before it
        // (`git -C ~ -C . ...` runs in `~`). Words resolve against every
        // folder of the chain too: the word lexer drops empty words, so
        // `-C ~ -C '' log` reads as `-C ~ -C log`.
        let mut chained: Vec<String> = Vec::new();
        let mut previous: Option<&str> = None;
        let mut bases = vec![cwd.clone()];
        for (position, word) in words.iter().enumerate() {
            work::step();
            let written = simple.writes_to.contains(&position)
                || writer.is_some_and(|(index, writes)| writes && position > index);
            // A folder flag's value is where the command runs, not what it reads.
            let mut folder_value = false;
            if let Some(flag) = previous
                && FOLDER_FLAGS.contains(&flag)
            {
                folder = self.homes.resolve(word, &cwd);
                self.chain_folder(&mut chained, word, &cwd);
                folder_value = true;
            } else if let Some((flag, value)) = word.split_once('=')
                && FOLDER_FLAGS.contains(&flag)
            {
                folder = self.homes.resolve(value, &cwd);
                self.chain_folder(&mut chained, value, &cwd);
                folder_value = true;
            }
            if folder_value {
                bases = folder_bases(&folder, &chained, &cwd);
            }
            previous = Some(word.as_str());
            // Attached, assigned and volume forms (`-o/path`, `f=@/path`,
            // `~/.aws:/c`) are paths too.
            let pieces = std::iter::once(word.as_str()).chain(
                word.split(['=', ':'])
                    .filter(|piece| piece.len() < word.len()),
            );
            for candidate in pieces {
                work::step();
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
                for base in &bases {
                    work::step();
                    let resolved = self.homes.resolve(candidate, base);
                    self.word_path(&resolved, candidate, depth);
                    if written
                        && (self.homes.is_persistence(&resolved) || self.unknown_place(candidate))
                    {
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
        for file in outputs.iter().flatten() {
            work::step();
            for base in &bases {
                work::step();
                let resolved = self.homes.resolve(file, base);
                self.word_path(&resolved, file, depth);
                // #1159: files of any name written in a folder (`--work-tree
                // ~`, `--output-directory ~/`) may be the home's.
                if self.homes.is_persistence(&resolved)
                    || (file.ends_with('/') && self.homes.holds_a_home(&resolved))
                    || self.unknown_place(file)
                {
                    self.note(ProtectedActionKind::Persistence);
                }
            }
        }
        self.scripts_of(simple, words, unseen, command.as_ref(), &cwd, depth);
    }

    /// #1283: for each of a simple command's words, whether the quote-aware
    /// reading (`nested.rs`), which checks git on the words as the shell
    /// passes them on, cannot spell it as a name (`hidden`: `$g` for `git`,
    /// a brace expansion, `$IFS`, a substitution's output; not
    /// `"$HOME/bin/git"`), or cannot know its text (`unchecked`: also any
    /// variable it does not resolve, as in `k=core.pager; git config "$k"`).
    /// `variables`: as the words were substituted.
    fn word_marks(
        &self,
        simple: &SimpleCommand,
        words: &[String],
        variables: &HashMap<String, String>,
    ) -> (Vec<bool>, Vec<bool>) {
        let user_home = self.homes.user_home();
        let codex_home = self.homes.codex_home();
        // The homes the quote-aware reading resolves itself.
        let resolved = |name: &str, value: &str| match name {
            "HOME" => user_home == Some(value),
            "CODEX_HOME" | "CORBANU_HOME" | "PFTERMINAL_HOME" => codex_home == value,
            _ => false,
        };
        let mut hidden = Vec::with_capacity(words.len());
        let mut unchecked = Vec::with_capacity(words.len());
        for (word, raw) in words.iter().zip(&simple.words) {
            work::step();
            hidden.push(basename(&word.to_lowercase()) != basename(&raw.to_lowercase()));
            unchecked.push(
                word != raw
                    && raw.match_indices('$').any(|(at, _)| {
                        work::step();
                        let after = &raw[at + 1..];
                        let end = after
                            .find(|ch: char| !(ch.is_ascii_alphanumeric() || ch == '_'))
                            .unwrap_or(after.len());
                        let name = &after[..end];
                        variables
                            .get(name)
                            .is_some_and(|value| !resolved(name, value))
                    }),
            );
        }
        for at in &simple.hidden {
            work::step();
            if let (Some(hidden), Some(unchecked)) = (hidden.get_mut(*at), unchecked.get_mut(*at)) {
                *hidden = true;
                *unchecked = true;
            }
        }
        (hidden, unchecked)
    }

    /// Follow a relative `-C` from the folder before it. Past
    /// `MAX_CHAINED_FOLDERS` folders, or a folder name past
    /// `MAX_FOLDER_BYTES`, the chain is not followed (#1102: `-C a -C a ...`
    /// would grow it without bound) and the line asks.
    fn chain_folder(&mut self, chained: &mut Vec<String>, value: &str, cwd: &str) {
        let next = self
            .homes
            .resolve(value, chained.last().map_or(cwd, String::as_str));
        if chained.len() >= MAX_CHAINED_FOLDERS || next.len() > MAX_FOLDER_BYTES {
            self.note(ProtectedActionKind::UnseenCode);
        } else {
            chained.push(next);
        }
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
                    // #841 (MOD-06): a "command" split out of a quoted sed
                    // program (an address such as `/a`) is sed's data, not a
                    // command the shell runs: read it if it is a file, but a
                    // missing one does not fail closed.
                    let need = if self
                        .sed_scope
                        .as_ref()
                        .is_some_and(|scope| scope.is_data(word))
                    {
                        Script::IfPresent
                    } else {
                        Script::Required
                    };
                    self.script_file(&file, cwd, depth, need);
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
                    work::step();
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
            work::step();
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
            self.ran_unknown = true;
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

    /// #1283: in a folder the classifier cannot know, a file named relative
    /// to it (`.zshenv`, `../x`, `$d/x`) may be any file.
    fn unknown_place(&self, word: &str) -> bool {
        self.unknown_folder > 0
            && (word.contains("..")
                || self.homes.resolve(word, "/a") != self.homes.resolve(word, "/b"))
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
            ("popd", _) => std::rc::Rc::make_mut(&mut folders.stack).pop(),
            _ => None,
        };
        if let Some(next) = next {
            if next.len() > MAX_FOLDER_BYTES
                || (name == "pushd" && folders.stack.len() >= MAX_FOLDER_STACK)
            {
                folders.unfollowed = true;
                return;
            }
            if name == "pushd" {
                std::rc::Rc::make_mut(&mut folders.stack).push(folders.cwd.clone());
            }
            folders.previous = std::mem::replace(&mut folders.cwd, next);
            variables.insert("PWD".to_string(), folders.cwd.clone());
            variables.insert("OLDPWD".to_string(), folders.previous.clone());
        }
    }

    /// The checks on a Corbanu CLI word, given the words after it (`later`
    /// from `from`).
    fn cli(&mut self, later: &mut LaterWords, from: usize) {
        if later.holds(from, &["vault"]) {
            self.note(ProtectedActionKind::Vault);
        }
        if later
            .subcommand(from)
            .is_some_and(|sub| CLI_POLICY_WORDS.contains(&sub))
        {
            self.note(ProtectedActionKind::SecurityPolicy);
        }
        if later.policy_flag(from) {
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
        // #1240: the first judgment reads script files; the Windows view's
        // paths are not ones to open (and where it may not look a missing
        // optional script would read as unreadable).
        if self.homes.is_windows() {
            return;
        }
        self.ran.push(file.to_string());
        let unreadable = |classifier: &mut Self, missing: bool| {
            if need == Script::Required || !missing {
                classifier.note(ProtectedActionKind::UnseenCode);
            }
        };
        match self.read_file(file) {
            FileText::Missing => unreadable(self, /*missing*/ true),
            FileText::Unreadable => unreadable(self, /*missing*/ false),
            FileText::Binary => {}
            FileText::Text(text) => self.script(&text, cwd, depth + 1),
        }
    }

    /// Read a file the action runs or uses as code, within the lookup and
    /// file limits.
    fn read_file(&mut self, file: &str) -> FileText {
        use std::io::Read;
        if !self.homes.lookups_allowed(file) {
            return FileText::Unreadable;
        }
        // Check before opening: opening a FIFO or device would block.
        let Ok(metadata) = std::fs::metadata(file) else {
            return FileText::Missing;
        };
        if !metadata.is_file() {
            return FileText::Unreadable;
        }
        self.scripts_read += 1;
        if self.scripts_read > MAX_SCRIPT_FILES {
            return FileText::Unreadable;
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
            return FileText::Unreadable;
        };
        if !handle.metadata().is_ok_and(|metadata| metadata.is_file()) {
            return FileText::Unreadable;
        }
        let mut bytes = Vec::new();
        let mut reader = handle.take(indirect::SCRIPT_READ_LIMIT + 1);
        if reader.read_to_end(&mut bytes).is_err() {
            return FileText::Unreadable;
        }
        // Executables first: their size does not matter.
        if BINARY_MAGIC.iter().any(|magic| bytes.starts_with(magic)) {
            return FileText::Binary;
        }
        if bytes.len() as u64 > indirect::SCRIPT_READ_LIMIT {
            return FileText::Unreadable;
        }
        // The shell skips NUL bytes; so does the scan.
        bytes.retain(|byte| *byte != 0);
        FileText::Text(String::from_utf8_lossy(&bytes).into_owned())
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
/// `budget` is the bytes of values left to substitute (#1065); a value
/// that does not fit is unseen output, runs the budget out, and is
/// returned by name.
fn substitute(
    command: &SimpleCommand,
    variables: &HashMap<String, String>,
    budget: &mut Budget,
) -> (Vec<String>, Vec<bool>, HashSet<String>) {
    let mut skipped = HashSet::new();
    let (words, unseen): (Vec<String>, Vec<bool>) = command
        .words
        .iter()
        .map(|word| {
            let mut out = String::new();
            let mut unseen = word.contains(shell::UNSEEN_OUTPUT);
            let mut rest = word.as_str();
            while let Some(at) = rest.find('$') {
                work::step();
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
                    Some(value) if len > 0 && value.len() < budget.left => {
                        budget.left -= value.len();
                        unseen |= value.contains(shell::UNSEEN_OUTPUT);
                        out.push_str(value);
                    }
                    Some(value)
                        if len > 0
                            && value.len() <= MAX_SMALL_VALUE
                            && value.len() <= budget.small_left =>
                    {
                        budget.left = 0;
                        budget.small_left -= value.len();
                        unseen |= value.contains(shell::UNSEEN_OUTPUT);
                        out.push_str(value);
                    }
                    Some(_) if len > 0 => {
                        budget.left = 0;
                        unseen = true;
                        if !skipped.contains(name) {
                            skipped.insert(name.to_string());
                        }
                        out.push_str(shell::UNSEEN_OUTPUT);
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
        .unzip();
    (words, unseen, skipped)
}

/// The level as other product messages name it ("security level Moderate").
fn level_name(level: SecurityLevel) -> &'static str {
    match level {
        SecurityLevel::Permissive => "Permissive",
        SecurityLevel::Moderate => "Moderate",
        SecurityLevel::Aggressive => "Aggressive",
    }
}

#[cfg(test)]
#[path = "tainted_action_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "tainted_action_refusal_tests.rs"]
mod tainted_action_refusal_tests;
