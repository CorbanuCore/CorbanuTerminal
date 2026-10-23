//! #439: a command that security level Aggressive stopped says so, to the
//! person (a warning in the transcript) and to the model (a note on the
//! command's output), with the way forward. Without it the person saw only
//! "Operation not permitted" or a connection error.
//!
//! Only the sandbox that Corbanu Terminal's Aggressive launch selected counts
//! ([`crate::config::Permissions::launch_enforces_aggressive`]); a sandbox the
//! person configured themselves is not attributed to the level.

use crate::config::Config;

/// Added to the permissions instructions under Aggressive, so the model
/// names the level when it cannot do something (AGG-13: asked for a web
/// search, it tried `curl` instead).
pub(crate) const AGGRESSIVE_INSTRUCTIONS: &str = "The person chose security level Aggressive in Corbanu Terminal's /security. Under it, agent commands stay in the sandbox even when the person approves them: they can write only inside the workspace, have no network access, and can never read protected files (such as .env and keychains), not even with a grant; web search is off, and agents cannot use the Corbanu vault (the person can use /vault themselves). When a request needs something this blocks, do not try another way around it (for example, do not fetch web pages with curl when web search is off): tell the person that security level Aggressive blocks it and that they can choose another level with /security.";

/// #747 (decision D3): added to the permissions instructions while security
/// level Moderate is in force, so the model does not promise that an
/// approved request runs outside the sandbox.
pub(crate) const MODERATE_INSTRUCTIONS: &str = "The person chose security level Moderate in Corbanu Terminal's /security. Under it, a command never leaves the sandbox it runs in, even when the person approves a request to run outside it (`require_escalated`): such an approval can lift Moderate's protection of credential and Corbanu files for that one command, but a write outside the writable folders, or network the sandbox blocks, still fails. When a request needs that, do not try another way around it: tell the person that security level Moderate keeps commands in the sandbox and that they can choose another level with /security.";

/// #739 (option B): added to the permissions instructions under security
/// level Permissive while protected launch is armed, so the model does not
/// request runs outside the sandbox that are refused without asking.
pub(crate) const PROTECTED_LAUNCH_INSTRUCTIONS: &str = "Corbanu Terminal's protected launch (secretless_agent_launch) is on: every agent command runs inside the OS sandbox, also at security level Permissive, and nothing can run outside it, not even with the person's approval. A request to run outside the sandbox (`require_escalated`) never leaves it (it is normally refused without asking the person), no retry without the sandbox is offered, and a remembered approval or allow rule that would skip the sandbox runs the command inside it. When a request needs a write outside the writable folders or network the sandbox blocks, do not try another way around it: tell the person that protected launch keeps commands in the sandbox.";

/// What the Aggressive sandbox stopped.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Blocked {
    /// A write outside the project or a read of a protected file.
    Files,
    /// Network access.
    Network,
}

/// Lowercase fragments of common "no network" failures (curl, git, Python,
/// Node, pip). Under Aggressive no connection is possible at all, so a
/// failed connection to a local server counts too; the notice says the
/// command only "looks" blocked.
const NETWORK_FAILURES: [&str; 8] = [
    "could not resolve host",
    "couldn't resolve host",
    "failed to connect to",
    "network is unreachable",
    "temporary failure in name resolution",
    "nodename nor servname",
    "name or service not known",
    "getaddrinfo",
];

/// Lowercase fragments of a line that reports a denied file operation.
const DENIAL_LINES: [&str; 3] = [
    "operation not permitted",
    "permission denied",
    "read-only file system",
];

/// Lines the macOS developer-tool shims (`/usr/bin/python3`, `git`) print when
/// they cannot write their cache. They name no file the command asked for and
/// would make every failing `git` or `python3` command look blocked.
const SHIM_NOISE: [&str; 4] = ["xcrun_db", "darwin_user_", "xcodebuild", "dvtfilepath"];

/// A denied file operation the command itself reported.
fn denial_line(lower: &str) -> bool {
    lower.lines().any(|line| {
        DENIAL_LINES.iter().any(|denial| line.contains(denial))
            && !SHIM_NOISE.iter().any(|noise| line.contains(noise))
    })
}

/// What the Aggressive sandbox stopped in a finished command, if anything.
/// `sandbox_denied` is the sandbox-denial verdict for the command; a file
/// denial also needs a denial line that is not developer-tool noise.
pub(crate) fn blocked(
    config: &Config,
    sandbox_denied: bool,
    exit_code: Option<i32>,
    output: &str,
) -> Option<Blocked> {
    if !config.permissions.launch_enforces_aggressive() || exit_code.is_none_or(|code| code == 0) {
        return None;
    }
    let lower = output.to_lowercase();
    if NETWORK_FAILURES
        .iter()
        .any(|failure| lower.contains(failure))
    {
        Some(Blocked::Network)
    } else if sandbox_denied && denial_line(&lower) {
        Some(Blocked::Files)
    } else {
        None
    }
}

impl Blocked {
    /// Shown to the person in the transcript.
    pub(crate) fn notice(self) -> &'static str {
        match self {
            Self::Files => {
                "Security level Aggressive blocked this command: agent commands can write only inside this project, even when you approve them, and can never read protected files (such as .env and keychains), not even once. To lift these limits, choose another level with /security."
            }
            Self::Network => {
                "This command looks blocked by security level Aggressive: agent commands have no network access, even when you approve them, and web search is off. To use the network, choose another level with /security."
            }
        }
    }

    /// #622: what the inspector's recent denials say was refused.
    pub(crate) fn inspected(self) -> &'static str {
        match self {
            Self::Files => "a write outside the current folder or a protected read",
            Self::Network => "network access",
        }
    }

    /// #622: the outcome the inspector shows for it.
    pub(crate) fn inspected_outcome(self) -> &'static str {
        match self {
            Self::Files => "blocked by the Aggressive sandbox",
            Self::Network => "looks blocked by the Aggressive sandbox",
        }
    }

    /// Added to the command's output for the model.
    pub(crate) fn model_note(self) -> String {
        format!(
            "\n[Corbanu Terminal: {} If this is why the command failed, tell the person instead of working around it.]",
            self.notice()
        )
    }
}

#[cfg(test)]
#[path = "refusal_notice_tests.rs"]
mod tests;
