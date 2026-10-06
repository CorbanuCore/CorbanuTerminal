//! PF-30-S03 post-taint authority checks.
//!
//! Once a session has taken in content without standing (tool, MCP, agent,
//! memory or unattributed text, PF-30-S02), a model-selected action that
//! reaches credentials, the vault or security policy needs fresh, exact human
//! approval. Neither a cached session approval, a permission hook, the
//! automatic reviewer nor `approval_policy = never` can stand in for it.
//!
//! This is a deterministic lexical net over the exact command or patch the
//! host will run, not a prompt classifier, and it never relaxes anything:
//! outside `source_envelopes` with Moderate/Aggressive it does nothing.

use crate::tools::sandboxing::ApprovalAction;
use codex_utils_path_uri::PathUri;
use std::path::Path;

/// Protected surfaces a tainted session may not reach on model authority.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ProtectedActionKind {
    /// The Corbanu vault (`corbanu vault ...`).
    Vault,
    /// Credential stores: login tokens, SSH/cloud keys, OS keychain.
    Credentials,
    /// Security policy: Corbanu config, rules, hooks, requirements, login state.
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

/// Host-held state for one protected action, captured before approval and
/// checked again right before it runs.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct PostTaintAction {
    pub(crate) kind: ProtectedActionKind,
    /// Count of recorded batches without standing when the check started.
    pub(crate) taint_generation: u64,
}

impl PostTaintAction {
    /// Shown in the approval prompt so the human sees why it was asked again.
    pub(crate) fn approval_reason(self) -> String {
        format!(
            "Security level: this is {} after untrusted content entered the session. \
             Approve only if you asked for it; a cached or automatic approval does not count.",
            self.kind.describe()
        )
    }

    pub(crate) fn approvals_off_rejection(self) -> String {
        format!(
            "Not run: this is {} after untrusted content entered the session, so it needs your \
             approval, and approvals are off (approval_policy = never).",
            self.kind.describe()
        )
    }

    pub(crate) fn stale_rejection(self) -> String {
        format!(
            "Not run: new untrusted content arrived while {} was waiting for approval. \
             Ask again if it is still needed.",
            self.kind.describe()
        )
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
];
/// Folders whose contents are credentials wherever they are.
const CREDENTIAL_SEGMENTS: &[&str] = &[".ssh", ".aws", ".gnupg", ".kube", ".docker", ".azure"];
const CREDENTIAL_FILES: &[&str] = &[
    ".credentials.json",
    ".netrc",
    ".git-credentials",
    ".npmrc",
    ".pypirc",
    "id_rsa",
    "id_ecdsa",
    "id_ed25519",
    "id_dsa",
];
/// Corbanu home folder names (besides the configured `CODEX_HOME`).
const HOME_SEGMENTS: &[&str] = &[".corbanu", ".codex", ".pfterminal"];
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
/// Words that only run another command.
const PREFIX_COMMANDS: &[&str] = &[
    "sudo", "env", "command", "exec", "nohup", "time", "nice", "builtin", "xargs",
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
    let homes = Homes {
        codex_home: normalized(&codex_home.to_string_lossy()),
        user_home: user_home.map(|home| normalized(&home.to_string_lossy())),
    };
    let strongest =
        |kinds: &mut dyn Iterator<Item = ProtectedActionKind>| kinds.max_by_key(|kind| rank(*kind));
    match action {
        ApprovalAction::Shell { command, cwd, .. }
        | ApprovalAction::ExecCommand { command, cwd, .. } => {
            let cwd = path_text(cwd);
            strongest(
                &mut homes
                    .classify_path(&cwd)
                    .into_iter()
                    .chain(classify_command(command, &homes, &cwd)),
            )
        }
        ApprovalAction::ApplyPatch { files, .. } => strongest(
            &mut files
                .iter()
                .filter_map(|file| homes.classify_path(&path_text(file))),
        ),
    }
}

struct Homes {
    codex_home: String,
    user_home: Option<String>,
}

fn normalized(path: &str) -> String {
    path.trim_end_matches('/').to_lowercase()
}

fn path_text(path: &PathUri) -> String {
    path.to_abs_path()
        .map(|path| path.as_path().to_string_lossy().into_owned())
        .unwrap_or_else(|_| path.to_string())
        .to_lowercase()
}

fn rank(kind: ProtectedActionKind) -> u8 {
    match kind {
        ProtectedActionKind::SecurityPolicy => 0,
        ProtectedActionKind::Credentials => 1,
        ProtectedActionKind::Vault => 2,
    }
}

fn basename(word: &str) -> &str {
    word.trim_end_matches('/')
        .rsplit('/')
        .next()
        .unwrap_or(word)
}

impl Homes {
    /// Expand `~`, `$HOME` and `$CODEX_HOME` spellings and make a relative
    /// word absolute against the command's working folder.
    fn resolve(&self, word: &str, cwd: &str) -> String {
        let mut word = word.to_string();
        for (spelling, value) in [
            ("${codex_home}", Some(&self.codex_home)),
            ("$codex_home", Some(&self.codex_home)),
            ("${home}", self.user_home.as_ref()),
            ("$home", self.user_home.as_ref()),
        ] {
            if let Some(value) = value {
                word = word.replace(spelling, value);
            }
        }
        if let Some(home) = &self.user_home
            && (word == "~" || word.starts_with("~/"))
        {
            word = format!("{home}{}", &word[1..]);
        }
        if word.starts_with('/') || cwd.is_empty() {
            word
        } else {
            format!("{cwd}/{word}")
        }
    }

    /// The protected kind of one absolute path (wildcards keep their fixed prefix).
    fn classify_path(&self, path: &str) -> Option<ProtectedActionKind> {
        let path = normalized(path);
        let segments: Vec<&str> = path
            .split('/')
            .filter(|segment| !segment.is_empty())
            .collect();
        let name = segments.last().copied().unwrap_or_default();
        if segments
            .iter()
            .any(|segment| CREDENTIAL_SEGMENTS.contains(segment))
            || CREDENTIAL_FILES.contains(&name)
            || (name == "hosts.yml" && path.contains("/gh/"))
            || name.starts_with("id_rsa")
            || name.starts_with("id_ed25519")
        {
            return Some(ProtectedActionKind::Credentials);
        }
        // Inside a Corbanu home: the rest of the path below it.
        let below_home = if !self.codex_home.is_empty()
            && (path == self.codex_home || path.starts_with(&format!("{}/", self.codex_home)))
        {
            Some(&path[self.codex_home.len()..])
        } else {
            HOME_SEGMENTS.iter().find_map(|home| {
                let marker = format!("/{home}");
                path.match_indices(&marker)
                    .map(|(index, _)| &path[index + marker.len()..])
                    .find(|rest| rest.is_empty() || rest.starts_with('/'))
            })
        }?;
        let first = below_home
            .trim_start_matches('/')
            .split('/')
            .next()
            .unwrap_or("");
        // Agent worktrees kept under the home are ordinary workspaces.
        if first == "worktrees" {
            return None;
        }
        if name.contains("vault") || first.contains("vault") {
            return Some(ProtectedActionKind::Vault);
        }
        if name.starts_with("auth") || name.contains("credential") || name.ends_with(".key") {
            return Some(ProtectedActionKind::Credentials);
        }
        Some(ProtectedActionKind::SecurityPolicy)
    }
}

/// Simple commands of the argv and of any script inside it (`bash -lc
/// "..."`): words split on whitespace, with quotes and backslashes removed as
/// the shell does, and command boundaries kept.
fn simple_commands(command: &[String]) -> Vec<Vec<String>> {
    let mut commands = vec![Vec::new()];
    let mut previous: Option<&str> = None;
    for arg in command {
        // The script after `-c`/`-lc` is its own command line.
        if previous.is_some_and(|flag| {
            flag.starts_with('-') && !flag.starts_with("--") && flag.ends_with('c')
        }) {
            commands.push(Vec::new());
        }
        previous = Some(arg.as_str());
        let mut word = String::new();
        let mut flush = |word: &mut String, commands: &mut Vec<Vec<String>>| {
            if !word.is_empty() {
                commands
                    .last_mut()
                    .expect("one command")
                    .push(std::mem::take(word).to_lowercase());
            }
        };
        for ch in arg.chars() {
            match ch {
                '\'' | '"' | '\\' | '{' | '}' => {}
                ';' | '|' | '&' | '(' | ')' | '`' | '\n' => {
                    flush(&mut word, &mut commands);
                    commands.push(Vec::new());
                }
                ch if ch.is_whitespace() || matches!(ch, '<' | '>' | ',') => {
                    flush(&mut word, &mut commands);
                }
                ch => word.push(ch),
            }
        }
        flush(&mut word, &mut commands);
    }
    commands.retain(|words| !words.is_empty());
    commands
}

/// The command name of a simple command: skips `VAR=value` assignments and
/// wrappers such as `sudo` and `env`.
fn command_index(words: &[String]) -> Option<usize> {
    words.iter().position(|word| {
        let name = basename(word);
        !(word.contains('=') && !word.starts_with('-'))
            && !PREFIX_COMMANDS.contains(&name)
            && !word.starts_with('-')
    })
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
        "-C",
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

fn classify_command(command: &[String], homes: &Homes, cwd: &str) -> Option<ProtectedActionKind> {
    let mut found: Option<ProtectedActionKind> = None;
    let mut note = |kind: ProtectedActionKind| {
        if found.is_none_or(|current| rank(kind) > rank(current)) {
            found = Some(kind);
        }
    };
    for words in simple_commands(command) {
        if let Some(index) = command_index(&words) {
            let name = basename(&words[index]);
            let args = &words[index + 1..];
            if CLI_NAMES.contains(&name) {
                if args.iter().any(|word| word == "vault") {
                    note(ProtectedActionKind::Vault);
                }
                if subcommand(args).is_some_and(|sub| CLI_POLICY_WORDS.contains(&sub)) {
                    note(ProtectedActionKind::SecurityPolicy);
                }
                let policy_key =
                    |value: &str| POLICY_CONFIG_KEYS.iter().any(|key| value.starts_with(key));
                let policy_flag = args.iter().enumerate().any(|(index, word)| {
                    CLI_POLICY_FLAGS.contains(&word.as_str())
                        || word.starts_with("--dangerously")
                        || word.strip_prefix("--config=").is_some_and(policy_key)
                        || (matches!(word.as_str(), "-c" | "--config")
                            && args.get(index + 1).is_some_and(|value| policy_key(value)))
                });
                if policy_flag {
                    note(ProtectedActionKind::SecurityPolicy);
                }
            }
            for (credential_command, verbs) in CREDENTIAL_COMMANDS {
                if name == *credential_command
                    && args.iter().any(|word| verbs.contains(&word.as_str()))
                {
                    note(ProtectedActionKind::Credentials);
                }
            }
        }
        for word in &words {
            // `$HOME`-style words were split from `$`; resolve them whole.
            for candidate in word.split('=') {
                if candidate.is_empty() || candidate.starts_with('-') && !candidate.contains('/') {
                    continue;
                }
                if let Some(kind) = homes.classify_path(&homes.resolve(candidate, cwd)) {
                    note(kind);
                }
            }
        }
    }
    found
}

#[cfg(test)]
#[path = "tainted_action_tests.rs"]
mod tests;
