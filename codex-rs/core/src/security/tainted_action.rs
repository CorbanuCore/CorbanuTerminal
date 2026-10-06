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
    "projects",
];
/// Folders whose contents are credentials wherever they are.
const CREDENTIAL_SEGMENTS: &[&str] = &[".ssh", ".aws", ".gnupg", ".kube"];
/// Credential folders only in the user's home (projects often have their own).
const USER_CREDENTIAL_SEGMENTS: &[&str] = &[".docker", ".azure", ".config/gh", ".config/gcloud"];
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
            normalize_path(&word)
        } else {
            normalize_path(&format!("{cwd}/{word}"))
        }
    }

    /// The protected kind of one absolute path. Glob segments count as any
    /// protected folder they could match.
    fn classify_path(&self, path: &str) -> Option<ProtectedActionKind> {
        let path = normalize_path(&path.to_lowercase());
        let segments: Vec<&str> = path
            .split('/')
            .filter(|segment| !segment.is_empty())
            .collect();
        let name = segments.last().copied().unwrap_or_default();
        let under_user_home = |folder: &str| {
            self.user_home
                .as_ref()
                .is_some_and(|home| path.starts_with(&format!("{home}/{folder}")))
        };
        if segments
            .iter()
            .any(|segment| matches_any(segment, CREDENTIAL_SEGMENTS))
            || USER_CREDENTIAL_SEGMENTS
                .iter()
                .any(|folder| under_user_home(folder))
            || CREDENTIAL_FILES.iter().any(|file| glob_matches(name, file))
            || (name == "hosts.yml" && path.contains("/gh/"))
            || name.starts_with("id_rsa")
            || name.starts_with("id_ed25519")
        {
            return Some(ProtectedActionKind::Credentials);
        }
        let below_home = self.below_home(&path, &segments)?;
        let first = below_home
            .split('/')
            .find(|segment| !segment.is_empty())
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

    /// The part of `path` below a Corbanu home, if it is inside one: the
    /// configured home as a whole-segment run anywhere in the path, or a
    /// default home folder name (or a glob that could match one) as a segment.
    fn below_home<'a>(&self, path: &'a str, segments: &[&str]) -> Option<&'a str> {
        if !self.codex_home.is_empty() {
            let found = path
                .match_indices(self.codex_home.as_str())
                .find_map(|(index, _)| {
                    let rest = &path[index + self.codex_home.len()..];
                    (rest.is_empty() || rest.starts_with('/')).then_some(rest)
                });
            if found.is_some() {
                return found;
            }
        }
        let index = segments
            .iter()
            .position(|segment| matches_any(segment, HOME_SEGMENTS))?;
        let marker = format!("/{}", segments[index]);
        path.match_indices(&marker)
            .map(|(at, _)| &path[at + marker.len()..])
            .find(|rest| rest.is_empty() || rest.starts_with('/'))
    }
}

/// Resolve `.`, `..` and repeated slashes without touching the filesystem.
fn normalize_path(path: &str) -> String {
    let mut parts: Vec<&str> = Vec::new();
    for segment in path.split('/') {
        match segment {
            "" | "." => {}
            ".." => {
                parts.pop();
            }
            segment => parts.push(segment),
        }
    }
    format!("/{}", parts.join("/"))
}

fn matches_any(segment: &str, names: &[&str]) -> bool {
    names.iter().any(|name| glob_matches(segment, name))
}

/// Whether the shell glob `pattern` (`*`, `?`, `[...]`) could match `name`.
/// Without glob characters this is equality.
fn glob_matches(pattern: &str, name: &str) -> bool {
    fn matches(pattern: &[char], name: &[char]) -> bool {
        match pattern.first() {
            None => name.is_empty(),
            Some('*') => (0..=name.len()).any(|skip| matches(&pattern[1..], &name[skip..])),
            Some('?') => !name.is_empty() && matches(&pattern[1..], &name[1..]),
            Some('[') => match pattern.iter().position(|ch| *ch == ']') {
                Some(close) => !name.is_empty() && matches(&pattern[close + 1..], &name[1..]),
                None => name.first() == Some(&'[') && matches(&pattern[1..], &name[1..]),
            },
            Some(ch) => name.first() == Some(ch) && matches(&pattern[1..], &name[1..]),
        }
    }
    let pattern: Vec<char> = pattern.chars().collect();
    let name: Vec<char> = name.chars().collect();
    matches(&pattern, &name)
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
        let flush = |word: &mut String, commands: &mut Vec<Vec<String>>| {
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
    let policy_key = |value: &str| POLICY_CONFIG_KEYS.iter().any(|key| value.starts_with(key));
    let mut cwd = cwd.to_string();
    for words in simple_commands(command) {
        // `cd` changes the folder later relative words resolve against.
        if words.first().is_some_and(|word| word == "cd")
            && let Some(target) = words.get(1)
        {
            cwd = homes.resolve(target, &cwd);
        }
        // Not only at command position: wrappers (`nice -n 5`, `sudo -u x`,
        // `timeout 9`, `eval`, `npx`, nested `sh -c`) put the real command later.
        for (index, word) in words.iter().enumerate() {
            let name = basename(word);
            let args = &words[index + 1..];
            if CLI_NAMES.contains(&name) {
                if args.iter().any(|word| word == "vault") {
                    note(ProtectedActionKind::Vault);
                }
                if subcommand(args).is_some_and(|sub| CLI_POLICY_WORDS.contains(&sub)) {
                    note(ProtectedActionKind::SecurityPolicy);
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
                if let Some(kind) = homes.classify_path(&homes.resolve(candidate, &cwd)) {
                    note(kind);
                }
            }
        }
    }
    if let Some(kind) = homes.classify_path(&cwd) {
        note(kind);
    }
    found
}

#[cfg(test)]
#[path = "tainted_action_tests.rs"]
mod tests;
