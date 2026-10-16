//! #748: under security level Moderate, a command the sandbox stopped because
//! of Moderate's protected-path rules says so where it failed, to the person
//! (a warning) and to the model (a note on the output), as #442 does under
//! Aggressive. Without it both saw only "Operation not permitted".
//!
//! Only a denial the rules add counts: the path is one they protect, and the
//! person's own permissions (the profile without the rules) allow the access.
//! A write outside the project, a network failure or the person's own denial
//! is not Moderate's and is not attributed to it. A path the secretless
//! launch contract also keeps at every level is attributed to that contract
//! (a person's approval may have lifted Moderate's rules for the command).
//! Otherwise the way forward is switching to Permissive, unless the person's
//! own full access runs commands outside a sandbox, which the contract
//! refuses at every level. Nothing is said about approvals lifting the rules
//! (#747), except that Corbanu's security state and keys stay denied even
//! then (#838). Explanation only: it never changes what runs or what is
//! denied.

use std::cell::RefCell;
use std::path::Path;
use std::path::PathBuf;

use codex_protocol::models::PermissionProfile;
use codex_protocol::permissions::FileSystemAccessMode;
use codex_protocol::permissions::FileSystemPath;
use codex_protocol::permissions::FileSystemSandboxEntry;
use codex_protocol::permissions::FileSystemSandboxKind;
use codex_protocol::permissions::FileSystemSandboxPolicy;
use codex_protocol::permissions::FileSystemSpecialPath;
use codex_protocol::permissions::ReadDenyMatcher;
use codex_utils_absolute_path::AbsolutePathBuf;

use super::MAX_REPORTED_LINES;
use super::ReadDenials;
use super::ReadPolicyInputs;
use crate::security::launch_contract::LaunchContract;

/// How a person could allow what Moderate's rules denied.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum WayForward {
    /// Switching to Permissive.
    Permissive,
    /// Nothing at any level: the person's own permissions run commands
    /// outside a sandbox, which the secretless launch contract refuses.
    NoLevel,
}

impl WayForward {
    fn text(self) -> &'static str {
        match self {
            Self::Permissive => {
                "To allow it, a person can switch to Permissive in /security (it applies after a \
                 restart)."
            }
            Self::NoLevel => {
                "Secretless agent launch runs agent commands only inside a sandbox at every \
                 security level, so changing the level would not allow it."
            }
        }
    }
}

/// The explanation for a read of `path` that Moderate's rules denied.
pub(crate) fn read_explanation(path: &str, way: WayForward) -> String {
    format!(
        "Security level Moderate blocked access to `{}`: agent commands can't read credential \
         files in your home folder (such as SSH keys, cloud and CLI tokens, and keychains) or \
         Corbanu's own files (its sign-in, vault, config and history). {}",
        super::shown_path(path),
        way.text()
    )
}

/// #838: the explanation for a read of Corbanu's security state or keys,
/// which Moderate keeps from agent commands even when a person approves one.
pub(crate) fn security_file_explanation(path: &str, way: WayForward) -> String {
    format!(
        "Security level Moderate blocked access to `{}`: agent commands can't read or change \
         Corbanu's security state and keys, even when you approve them. {}",
        super::shown_path(path),
        way.text()
    )
}

/// The explanation for a write to `path` that Moderate's rules denied.
pub(crate) fn write_explanation(path: &str, way: WayForward) -> String {
    format!(
        "Security level Moderate blocked writing `{}`: agent commands can't change files that \
         run code or set policy later (shell start-up files, git hooks and config, the \
         project's .codex and .agents) or Corbanu's own files in its home. {}",
        super::shown_path(path),
        way.text()
    )
}

/// The explanation for a read of `path` the secretless launch contract
/// denies at every level. It is not attributed to Moderate alone: a
/// person's approval may have lifted Moderate's rules for the command.
pub(crate) fn kept_read_explanation(path: &str) -> String {
    format!(
        "Secretless agent launch blocked access to `{}`: at every security level, Moderate \
         included, agent commands can't read Corbanu's own files (such as its sign-in, vault, \
         config and history) or common CLI tokens, even when you approve them, so changing the \
         level would not allow it.",
        super::shown_path(path)
    )
}

/// The explanation for a write to `path` the secretless launch contract
/// keeps read-only at every level (see [`kept_read_explanation`]).
pub(crate) fn kept_write_explanation(path: &str) -> String {
    format!(
        "Secretless agent launch blocked writing `{}`: at every security level, Moderate \
         included, agent commands can't change Corbanu's home or the credential files it \
         protects, even when you approve them, so changing the level would not allow it.",
        super::shown_path(path)
    )
}

/// The explanation for a command that failed with `denial_lines`, when
/// Moderate's rules denied a path it names (or reports) and the person's
/// own permissions would have allowed it.
pub(super) async fn denial_explanation(
    inputs: ReadPolicyInputs,
    command: &[String],
    cwd: &Path,
    denial_lines: Vec<String>,
) -> Option<String> {
    let contract = crate::security::launch_contract::active();
    let (command, cwd) = (command.to_vec(), cwd.to_path_buf());
    super::off_thread(move || explain(&inputs, contract, &command, &cwd, &denial_lines)).await
}

fn explain(
    inputs: &ReadPolicyInputs,
    contract: Option<&LaunchContract>,
    command: &[String],
    cwd: &Path,
    denial_lines: &[String],
) -> Option<String> {
    let own_profile = own_profile(inputs);
    let base = own_profile.file_system_sandbox_policy();
    let denials = inputs.denials();
    // A malformed pattern denies every path; explain nothing then.
    let own = ReadDenyMatcher::try_new(&base, cwd).ok()?;
    let protected = denials.apply(&inputs.profile)?.file_system_sandbox_policy();
    let protected = ReadDenyMatcher::try_new(&protected, cwd).ok().flatten()?;
    let (user_home, codex_home) = (inputs.user_home.as_deref(), inputs.codex_home.as_path());
    // The secretless launch contract refuses a launch outside a sandbox at
    // every level, so with the person's own full access no level change
    // lets the command run at all.
    let unsandboxed = base.kind != FileSystemSandboxKind::Restricted;
    let way = if contract.is_some() && unsandboxed {
        WayForward::NoLevel
    } else {
        WayForward::Permissive
    };
    // What the contract narrows: the command's sandbox without Moderate's
    // rules; with full access, full write, as Moderate's rules leave it.
    let launched = if unsandboxed {
        FileSystemSandboxPolicy::restricted(vec![FileSystemSandboxEntry::new(
            FileSystemPath::Special {
                value: FileSystemSpecialPath::Root,
            },
            FileSystemAccessMode::Write,
        )])
    } else {
        base.clone()
    };
    // The path the last accepted word or line names.
    let found: RefCell<Option<PathBuf>> = RefCell::new(None);

    let read = |path: &Path| {
        let blocked = protected.is_read_denied(path)
            && base.can_read_path_with_cwd(path, cwd)
            && !own.as_ref().is_some_and(|own| own.is_read_denied(path));
        if blocked {
            found.replace(Some(path.to_path_buf()));
        }
        blocked
    };
    let lines = &denial_lines[..denial_lines.len().min(MAX_REPORTED_LINES)];
    let read_word = read(cwd)
        .then(|| cwd.display().to_string())
        .or_else(|| {
            super::denied_word(
                command,
                cwd,
                user_home,
                codex_home,
                read,
                Some(denial_lines),
            )
        })
        // #621: a path built at run time that the denial line reports.
        .or_else(|| super::path_in_lines(command, cwd, user_home, codex_home, lines, &read));
    if let Some(word) = read_word {
        let path = found.take();
        let kept = path
            .as_ref()
            .zip(contract)
            .is_some_and(|(path, contract)| contract.keeps_unreadable(path, cwd));
        let security = path
            .as_ref()
            .is_some_and(|path| ReadDenials::is_security_file(codex_home, user_home, path));
        return Some(if kept {
            kept_read_explanation(&word)
        } else if security {
            security_file_explanation(&word, way)
        } else {
            read_explanation(&word, way)
        });
    }

    let writable: Vec<&AbsolutePathBuf> = denials.home_writable_paths().collect();
    let write = |path: &Path| {
        // A write lands where a link finally points.
        let target = AbsolutePathBuf::from_absolute_path(path)
            .ok()
            .and_then(|path| ReadDenials::real_location(&path))
            .map_or_else(|| path.to_path_buf(), AbsolutePathBuf::into_path_buf);
        let blocked = super::lands_in(path, denials.read_only_paths(), &writable)
            && base.can_write_path_with_cwd(&target, cwd);
        if blocked {
            found.replace(Some(target));
        }
        blocked
    };
    let word = super::written_word(
        command,
        cwd,
        user_home,
        codex_home,
        &write,
        Some(denial_lines),
    )?;
    let kept = found
        .take()
        .zip(contract)
        .is_some_and(|(path, contract)| contract.keeps_read_only(&launched, &path, cwd));
    Some(if kept {
        kept_write_explanation(&word)
    } else {
        write_explanation(&word, way)
    })
}

/// The person's own permissions for the command: the profile without
/// Moderate's rules, with the project roots as the sandbox resolves them.
fn own_profile(inputs: &ReadPolicyInputs) -> PermissionProfile {
    inputs
        .profile
        .clone()
        .materialize_project_roots_with_workspace_roots(&inputs.roots())
}

#[cfg(test)]
#[path = "moderate_tests.rs"]
mod tests;
