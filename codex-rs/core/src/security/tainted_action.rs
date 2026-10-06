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
const CLI_POLICY_SUBCOMMANDS: &[&str] = &["config", "features", "login", "logout"];
const CREDENTIAL_FILES: &[&str] = &[
    "auth.json",
    ".credentials.json",
    "source-origin.key",
    ".netrc",
    ".git-credentials",
    "id_rsa",
    "id_ecdsa",
    "id_ed25519",
    "id_dsa",
];
const CREDENTIAL_DIRS: &[&str] = &[".ssh/", ".aws/", ".gnupg/", ".docker/config.json"];
const KEYCHAIN_VERBS: &[&str] = &[
    "find-generic-password",
    "find-internet-password",
    "dump-keychain",
    "export",
    "unlock-keychain",
];
const POLICY_FILES: &[&str] = &[
    "config.toml",
    "requirements.toml",
    "managed_config.toml",
    "hooks.json",
];

/// Classify the exact action the host is about to run. `codex_home` is the
/// session's Corbanu home: anything under it is policy or credentials.
pub(crate) fn classify(action: &ApprovalAction, codex_home: &Path) -> Option<ProtectedActionKind> {
    let home = codex_home.to_string_lossy().to_lowercase();
    match action {
        ApprovalAction::Shell { command, .. } | ApprovalAction::ExecCommand { command, .. } => {
            classify_command(command, &home)
        }
        ApprovalAction::ApplyPatch { files, .. } => files
            .iter()
            .filter_map(|file| classify_path(&path_text(file), &home))
            .max_by_key(|kind| rank(*kind)),
    }
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

/// Every word of the argv and of any script inside it (`bash -lc "..."`),
/// split on whitespace and shell punctuation, lowercased.
fn words(command: &[String]) -> Vec<String> {
    command
        .iter()
        .flat_map(|arg| {
            arg.split(|ch: char| {
                ch.is_whitespace()
                    || matches!(ch, ';' | '|' | '&' | '(' | ')' | '<' | '>' | '`')
                    || matches!(ch, '$' | '"' | '\'' | '=' | ',' | '{' | '}')
            })
            .filter(|word| !word.is_empty())
            // `c\odex` is `codex` to the shell.
            .map(|word| word.replace('\\', "").to_lowercase())
            .collect::<Vec<_>>()
        })
        .collect()
}

fn basename(word: &str) -> &str {
    word.rsplit('/').next().unwrap_or(word)
}

fn classify_command(command: &[String], home: &str) -> Option<ProtectedActionKind> {
    let words = words(command);
    let mut found: Option<ProtectedActionKind> = None;
    let mut note = |kind: ProtectedActionKind| {
        if found.is_none_or(|current| rank(kind) > rank(current)) {
            found = Some(kind);
        }
    };
    for (index, word) in words.iter().enumerate() {
        let name = basename(word);
        let later = &words[index + 1..];
        if CLI_NAMES.contains(&name) {
            if later.iter().any(|word| word == "vault") {
                note(ProtectedActionKind::Vault);
            }
            if later
                .first()
                .is_some_and(|sub| CLI_POLICY_SUBCOMMANDS.contains(&sub.as_str()))
            {
                note(ProtectedActionKind::SecurityPolicy);
            }
        }
        if name == "security"
            && later
                .iter()
                .any(|word| KEYCHAIN_VERBS.contains(&word.as_str()))
        {
            note(ProtectedActionKind::Credentials);
        }
        if let Some(kind) = classify_path(word, home) {
            note(kind);
        }
    }
    found
}

fn classify_path(path: &str, home: &str) -> Option<ProtectedActionKind> {
    let name = basename(path);
    let under_home = !home.is_empty() && path.starts_with(home);
    if CREDENTIAL_FILES.contains(&name) || CREDENTIAL_DIRS.iter().any(|dir| path.contains(dir)) {
        return Some(ProtectedActionKind::Credentials);
    }
    if under_home && name.contains("vault") {
        return Some(ProtectedActionKind::Vault);
    }
    let corbanu_dir = path.contains(".corbanu/") || path.contains(".codex/");
    if under_home || name.ends_with(".rules") || (POLICY_FILES.contains(&name) && corbanu_dir) {
        return Some(ProtectedActionKind::SecurityPolicy);
    }
    None
}

#[cfg(test)]
#[path = "tainted_action_tests.rs"]
mod tests;
