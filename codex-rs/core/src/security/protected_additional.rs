//! Issue #245: a command's additional permissions never reopen a protected
//! path. The sandbox gives the most specific entry precedence, so an extra
//! read of `~/.ssh/id_ed25519` would override the PF-23 denial of `~/.ssh`
//! wherever the request was approved (a person, Guardian, a hook, the
//! approval cache, or a session or turn grant).
//!
//! Under Moderate and Aggressive, wherever the protected-path rules apply to
//! a run, a request whose additional permissions open a protected path is
//! refused and explained, and a permission grant (`request_permissions`)
//! never carries such entries into later commands or file tools. Where
//! Moderate lets a person's approval lift the rules for one command (issue
//! #239), nothing changes for reads. Permissive has no rules and is
//! unchanged.
//!
//! Issue #513: the same for the PF-23-S02 write protections. A write entry
//! equal to a read-only path wins the tie, a narrower one wins outright, and
//! a broader one opens a path the profile itself could not write (the
//! read-only rule is only added where it could). So additional write
//! permissions never make writable:
//! - the Corbanu home's runtime entries (`tmp` holds the sandbox launchers),
//!   under Moderate and Aggressive, even where an approval lifted the other
//!   rules, and never with a grant (#442);
//! - under Aggressive, the project's `.codex` and `.agents`, never with a
//!   grant either (#520), refused before anyone is asked (#565);
//! - under Aggressive, the other read-only paths (start-up files, git hooks
//!   and config) unless an Aggressive grant for the run lifted them;
//! - under Moderate, the other read-only paths and the project's `.codex`
//!   and `.agents` unless a person's fresh approval of
//!   this command allowed it (Travis, 2026-10-11): not the automatic
//!   reviewer, a permission hook, a remembered approval or a stored grant.
//!   Grants never carry them, at either level.

use std::path::Path;

use codex_protocol::models::AdditionalPermissionProfile;
use codex_protocol::models::FileSystemPermissions;
use codex_protocol::permissions::FileSystemAccessMode;
use codex_protocol::permissions::FileSystemPath;
use codex_protocol::permissions::FileSystemSandboxEntry;
use codex_protocol::permissions::FileSystemSandboxPolicy;
use codex_security_policy::SecurityLevel;
use codex_utils_absolute_path::AbsolutePathBuf;

use super::protected_surface::ReadDenials;

/// Issue #513: the read-only paths additional write permissions may not
/// reopen in a run.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ReadOnlyScope {
    /// Only the Corbanu home's runtime entries: under Moderate where a
    /// person's fresh approval decides the run, and before the approval
    /// wherever one may still lift the others.
    Runtime,
    /// The read-only paths no grant lifts (`ReadDenials::never_lifted_paths`:
    /// the runtime entries and the project's `.codex` and `.agents`): under
    /// Aggressive before the approval, so nobody is asked about a write that
    /// is refused afterwards whatever the answer (#565).
    NeverLifted,
    /// Every read-only path the denials keep (Aggressive, where a grant for
    /// the run has already lifted what it lifts; Moderate without a person's
    /// fresh approval; grants).
    Kept,
}

impl ReadOnlyScope {
    /// The scope before the run's approval. Under Moderate the persistence
    /// files count up front where no person will decide the run
    /// (`person_may_decide`: the automatic reviewer or nobody is asked).
    /// Under Aggressive the paths no grant lifts count up front (#565); the
    /// others wait for the run's grant, known after the approval.
    pub(crate) fn before_approval(level: SecurityLevel, person_may_decide: bool) -> Self {
        match level {
            SecurityLevel::Moderate if !person_may_decide => Self::Kept,
            SecurityLevel::Aggressive => Self::NeverLifted,
            _ => Self::Runtime,
        }
    }

    /// The scope once the run's rules are known. `by_person`: a person
    /// approved this command when asked about it now.
    pub(crate) fn after_approval(level: SecurityLevel, by_person: bool) -> Self {
        match level {
            SecurityLevel::Aggressive => Self::Kept,
            SecurityLevel::Moderate if !by_person => Self::Kept,
            _ => Self::Runtime,
        }
    }

    /// The scope for a permission grant, which every later command carries
    /// with nobody asked about it.
    pub(crate) fn for_grant(level: SecurityLevel) -> Self {
        match level {
            SecurityLevel::Aggressive | SecurityLevel::Moderate => Self::Kept,
            SecurityLevel::Permissive => Self::Runtime,
        }
    }
}

/// Issue #513: what may lift a read-only path's protection for one command.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum WriteRule {
    /// Nothing: the Corbanu home's runtime entries.
    Runtime,
    /// Under Moderate a person's fresh approval; under Aggressive nothing
    /// (`ReadDenials::never_lifted_paths`).
    NeverGranted,
    /// Under Moderate a person's fresh approval; under Aggressive a grant.
    Liftable,
}

/// What an additional permission would open, and so why it is refused.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum Opened {
    /// A protected path it would make readable.
    Read(String),
    /// A read-only path it would make writable, and what may lift its
    /// protection.
    Write { path: String, rule: WriteRule },
    /// The check itself failed (#524): refused.
    Unchecked,
}

impl Opened {
    /// The refusal shown to the person and the model.
    pub(crate) fn refusal(&self, level: SecurityLevel) -> String {
        match self {
            Self::Read(path) => refusal(level, path),
            Self::Write { path, rule } => write_refusal(level, path, *rule),
            Self::Unchecked => unchecked_refusal(level),
        }
    }
}

/// What `additional` may not open in a run.
#[derive(Clone, Copy, Debug)]
pub(crate) struct Check<'a> {
    pub(crate) denials: &'a ReadDenials,
    /// The run's file system policy with the rules applied, for writes above
    /// a read-only path. `None` when unknown: any such write is refused.
    pub(crate) protected: Option<&'a FileSystemSandboxPolicy>,
    /// Whether reads of protected paths are refused (not where an approval
    /// lifted the rules for the run, #239).
    pub(crate) reads: bool,
    pub(crate) writes: ReadOnlyScope,
}

/// The first protected path (as an absolute path) that an entry of
/// `additional` would make readable or writable, with relative and
/// project-root entries resolved against each of `cwds` (the folders the
/// sandbox may resolve them against). Deny entries only narrow.
pub(crate) fn opened_protected_path(
    denials: &ReadDenials,
    additional: &AdditionalPermissionProfile,
    cwds: &[&Path],
) -> Option<String> {
    let entries = &additional.file_system.as_ref()?.entries;
    if entries.is_empty() {
        return None;
    }
    let matcher = denials.matcher()?;
    entries
        .iter()
        .filter(|entry| entry.access != FileSystemAccessMode::Deny)
        // One entry at a time: an entry for `/` must not hide a narrower one.
        .flat_map(|entry| {
            let alone = FileSystemSandboxPolicy::restricted(vec![entry.clone()]);
            cwds.iter()
                .flat_map(move |cwd| alone.get_readable_roots_with_cwd(cwd))
        })
        .find(|root| matcher.is_read_denied(root.as_path()))
        .map(|root| root.display().to_string())
}

/// Issue #513: the first read-only path in `scope` that a write entry of
/// `additional` would make writable, resolved against each of `cwds`.
pub(crate) fn opened_read_only_path(
    denials: &ReadDenials,
    protected: Option<&FileSystemSandboxPolicy>,
    additional: &AdditionalPermissionProfile,
    cwds: &[&Path],
    scope: ReadOnlyScope,
) -> Option<AbsolutePathBuf> {
    let entries = &additional.file_system.as_ref()?.entries;
    let paths: Vec<&AbsolutePathBuf> = match scope {
        ReadOnlyScope::Runtime => denials.runtime_read_only_paths().collect(),
        ReadOnlyScope::NeverLifted => denials.never_lifted_paths().collect(),
        ReadOnlyScope::Kept => denials.read_only_paths().collect(),
    };
    if paths.is_empty() {
        return None;
    }
    let writable: Vec<&AbsolutePathBuf> = denials.home_writable_paths().collect();
    entries
        .iter()
        .filter(|entry| entry.access.can_write())
        .flat_map(|entry| cwds.iter().map(move |cwd| (entry, *cwd)))
        .find_map(|(entry, cwd)| reopened(&paths, &writable, protected, entry, cwd))
}

/// The read-only path of `paths` that the write `entry` reopens: one at or
/// above the paths it can write (equal entries tie in its favour, narrower
/// ones win), or one below them that `protected` with the entry added could
/// write. Links are followed as the sandbox does. A path the entry writes
/// inside a folder of `writable` below the read-only path (#583: `worktrees`
/// in the read-only Corbanu home) reopens nothing.
fn reopened(
    paths: &[&AbsolutePathBuf],
    writable: &[&AbsolutePathBuf],
    protected: Option<&FileSystemSandboxPolicy>,
    entry: &FileSystemSandboxEntry,
    cwd: &Path,
) -> Option<AbsolutePathBuf> {
    let alone = FileSystemSandboxPolicy::restricted(vec![entry.clone()]);
    let roots: Vec<AbsolutePathBuf> = if alone.has_full_disk_write_access() {
        let root = cwd.ancestors().last()?;
        vec![AbsolutePathBuf::from_absolute_path(root).ok()?]
    } else {
        alone
            .get_writable_roots_with_cwd(cwd)
            .into_iter()
            .map(|writable| writable.root)
            .collect()
    };
    let mut spellings = roots.clone();
    for root in &roots {
        if let Some(real) = ReadDenials::real_location(root)
            && !spellings.contains(&real)
        {
            spellings.push(real);
        }
    }
    let merged = protected.map(|protected| {
        let mut merged = protected.clone();
        merged.entries.extend(spellings.iter().map(|root| {
            FileSystemSandboxEntry::new(
                FileSystemPath::Path { path: root.clone() },
                FileSystemAccessMode::Write,
            )
        }));
        merged
    });
    paths
        .iter()
        .find(|path| {
            spellings.iter().any(|root| {
                root.as_path().starts_with(path.as_path())
                    && !writable.iter().any(|folder| {
                        root.as_path().starts_with(folder.as_path())
                            && folder.as_path().starts_with(path.as_path())
                    })
            }) || match &merged {
                Some(merged) => merged.can_write_path_with_cwd(path.as_path(), cwd),
                None => spellings
                    .iter()
                    .any(|root| path.as_path().starts_with(root.as_path())),
            }
        })
        .map(|path| (*path).clone())
}

/// Issues #245 and #513: what `additional` would open under `check`.
pub(crate) fn opened(
    check: Check<'_>,
    additional: &AdditionalPermissionProfile,
    cwds: &[&Path],
) -> Option<Opened> {
    if check.reads
        && let Some(path) = opened_protected_path(check.denials, additional, cwds)
    {
        return Some(Opened::Read(path));
    }
    let path = opened_read_only_path(
        check.denials,
        check.protected,
        additional,
        cwds,
        check.writes,
    )?;
    let rule = if check
        .denials
        .runtime_read_only_paths()
        .any(|kept| *kept == path)
    {
        WriteRule::Runtime
    } else if check.denials.never_lifted_paths().any(|kept| *kept == path) {
        WriteRule::NeverGranted
    } else {
        WriteRule::Liftable
    };
    Some(Opened::Write {
        path: path.display().to_string(),
        rule,
    })
}

/// `additional` without the entries `check` refuses (for a grant, which
/// every later command carries).
pub(crate) fn without_opened(
    check: Check<'_>,
    additional: Option<AdditionalPermissionProfile>,
    cwds: &[&Path],
) -> Option<AdditionalPermissionProfile> {
    let mut additional = additional?;
    let Some(file_system) = additional.file_system.take() else {
        return Some(additional);
    };
    let FileSystemPermissions {
        entries,
        glob_scan_max_depth,
    } = file_system;
    let entries: Vec<_> = entries
        .into_iter()
        .filter(|entry| {
            let alone = AdditionalPermissionProfile {
                network: None,
                file_system: Some(FileSystemPermissions {
                    entries: vec![entry.clone()],
                    glob_scan_max_depth: None,
                }),
            };
            opened(check, &alone, cwds).is_none()
        })
        .collect();
    if !entries.is_empty() {
        additional.file_system = Some(FileSystemPermissions {
            entries,
            glob_scan_max_depth,
        });
    }
    (!additional.is_empty()).then_some(additional)
}

/// `additional` without the entries that would open a protected path to
/// reads (#245).
#[cfg(all(test, unix))]
pub(crate) fn without_protected_paths(
    denials: &ReadDenials,
    additional: Option<AdditionalPermissionProfile>,
    cwds: &[&Path],
) -> Option<AdditionalPermissionProfile> {
    let check = Check {
        denials,
        protected: None,
        reads: true,
        writes: ReadOnlyScope::Runtime,
    };
    without_opened(check, additional, cwds)
}

/// Issues #245 and #513: the level whose protected-path rules apply to the
/// session now, if any (Moderate, Aggressive).
pub(crate) fn protected_level(session: &crate::session::session::Session) -> Option<SecurityLevel> {
    session
        .services
        .model_client()
        .post_taint_state()
        .filter(crate::security::tainted_action::PostTaintState::protected_paths_apply)
        .map(|state| state.level)
}

/// #747 (Travis, 2026-10-14, decision D3): under security level Moderate no
/// command runs outside the OS sandbox: not a request to run outside it
/// (`require_escalated`), not the retry after a sandbox denial, not an
/// exec-policy rule that would skip the sandbox. A person's approval still
/// lifts Moderate's protected-path rules for one command (#239, #513), inside
/// the sandbox. Follows the level in force now (also right after `/security`
/// confirms Moderate).
pub(crate) fn moderate_keeps_sandbox(session: &crate::session::session::Session) -> bool {
    session.services.model_client().live_level() == SecurityLevel::Moderate
}

/// #747: added where a person is asked to approve a command that asked to
/// run outside the sandbox under Moderate.
pub(crate) const MODERATE_STAYS_SANDBOXED: &str = "Under security level Moderate it still runs inside the sandbox, even if you approve: writes outside your writable folders and network your sandbox blocks still fail.";

/// #840: what approving a request to run outside the sandbox does under
/// Moderate before untrusted content (#239): it lifts the protected-file
/// rules for that one run, which is why the person is asked even where the
/// command would otherwise run unasked (unless a saved allow rule covers it:
/// then it runs unasked with the rules kept, D6). Said in terms of the
/// files, so a command that has nothing to do with credentials does not read
/// as a credential request.
pub(crate) const MODERATE_ESCALATION_LIFTS: &str = "This command asks to run outside the sandbox. Approving it lifts Moderate's protected-file rules for this one run: it may then read protected files, such as your SSH and cloud credentials, and write files that run code later in your writable folders.";

/// #840: the note on a request to run outside the sandbox under Moderate
/// before untrusted content: what approving lifts, and that it stays in the
/// sandbox (#747).
pub(crate) fn moderate_escalation_note() -> String {
    format!("{MODERATE_ESCALATION_LIFTS} {MODERATE_STAYS_SANDBOXED}")
}

/// #739 (Travis, 2026-10-14, option B, security sign-off): under security
/// level Permissive with protected launch armed (`secretless_agent_launch`),
/// whose launch contract refuses every command outside the OS sandbox, no
/// approval to run outside it is offered. A request to run outside it
/// (`require_escalated`) is refused before anyone is asked (where the
/// policy's own denied reads keep it in the sandbox anyway, it is asked
/// about and runs there, as without protected launch), "retry without
/// sandbox?" is not offered, and a remembered approval or allow rule that
/// would skip the sandbox runs the command inside it, with a note. Permissive
/// without protected launch is unchanged; Moderate and Aggressive keep their
/// own handling (#747).
pub(crate) fn permissive_launch_keeps_sandbox(session: &crate::session::session::Session) -> bool {
    crate::security::launch_contract::active().is_some()
        && session.services.model_client().live_level() == SecurityLevel::Permissive
}

/// Whether no command leaves the sandbox in `session` now: Moderate (#747)
/// or Permissive under protected launch (#739).
pub(crate) fn keeps_sandbox_always(session: &crate::session::session::Session) -> bool {
    moderate_keeps_sandbox(session) || permissive_launch_keeps_sandbox(session)
}

/// #739: what the model and the person get instead of an approval prompt for
/// a request to run outside the sandbox under protected launch.
pub(crate) const PROTECTED_LAUNCH_NO_ESCALATION: &str = "Not run: this command asked to run outside the OS sandbox, and that isn't possible here. Protected launch (secretless_agent_launch) keeps every agent command inside the sandbox, also at security level Permissive and even with an approval, so no approval was asked for. Inside the sandbox it cannot write outside your writable folders or use network the sandbox blocks.";

/// #739: shown to the person, once per session, when a remembered approval
/// or allow rule would have run a command outside the sandbox under
/// protected launch.
pub(crate) const PROTECTED_LAUNCH_RULE_STAYS_SANDBOXED: &str = "A remembered approval or allow rule would run this command outside the OS sandbox, which isn't possible here: protected launch (secretless_agent_launch) keeps every agent command inside it, also at security level Permissive. This command, and others such rules match in this session, still skip the prompt but run inside the sandbox, so writes outside your writable folders and network the sandbox blocks fail.";

/// #739: whether the session `thread` has not yet been told
/// [`PROTECTED_LAUNCH_RULE_STAYS_SANDBOXED`] (it is told once).
pub(crate) fn first_rule_note(thread: codex_protocol::ThreadId) -> bool {
    static NOTED: std::sync::OnceLock<
        std::sync::Mutex<std::collections::HashSet<codex_protocol::ThreadId>>,
    > = std::sync::OnceLock::new();
    match NOTED.get_or_init(Default::default).lock() {
        Ok(mut noted) => noted.insert(thread),
        Err(_) => true,
    }
}

/// #739: shown to the person when the sandbox blocked a command (or failed to
/// start) and a retry without it would have been offered (or run) under
/// Permissive.
pub(crate) const PROTECTED_LAUNCH_NO_RETRY: &str = "Running this command outside the sandbox isn't possible here: protected launch (secretless_agent_launch) keeps every agent command inside it, also at security level Permissive, so \"retry without sandbox?\" is not offered.";

/// Issues #245 and #513: a permission grant (`request_permissions`)
/// without the entries that would open a protected path or reopen a
/// read-only one under `level`'s rules, and the refusal to show when any
/// were dropped. Every later command and file tool carries the grant with
/// nobody asked about it, so under Aggressive and Moderate every read-only
/// path counts (#513). Each of `configs` is checked as the orchestrator
/// checks a command; an entry any of them refuses is dropped. Reads folders: call it
/// off the async thread ([`strip_grant_off_thread`]).
pub(crate) fn strip_grant(
    level: SecurityLevel,
    codex_home: &Path,
    sqlite_home: &Path,
    configs: &[GrantCheckConfig],
    grant: AdditionalPermissionProfile,
) -> (Option<AdditionalPermissionProfile>, Option<String>) {
    let mut kept = Some(grant);
    let mut refusal = None;
    for config in configs {
        let Some(grant) = kept.take() else {
            break;
        };
        let GrantCheckConfig {
            cwd,
            workspace_roots,
            profile,
        } = config;
        // As the orchestrator sees a command: project roots expanded.
        let profile = profile
            .clone()
            .materialize_project_roots_with_workspace_roots(workspace_roots);
        let denials =
            ReadDenials::for_turn(codex_home, sqlite_home, cwd, workspace_roots, &profile);
        let protected = denials
            .apply(&profile)
            .map(|protected| protected.file_system_sandbox_policy());
        let check = Check {
            denials: &denials,
            protected: protected.as_ref(),
            reads: true,
            writes: ReadOnlyScope::for_grant(level),
        };
        let mut cwds = vec![cwd.as_path()];
        cwds.extend(workspace_roots.iter().map(AbsolutePathBuf::as_path));
        let Some(opened) = opened(check, &grant, &cwds) else {
            kept = Some(grant);
            continue;
        };
        refusal.get_or_insert_with(|| opened.refusal(level));
        kept = without_opened(check, Some(grant), &cwds);
    }
    if refusal.is_some() {
        tracing::warn!(
            target: "codex_core::security::tainted_action",
            "a permission grant would open a protected path; dropped from it"
        );
    }
    (kept, refusal)
}

/// A folder, its workspace roots and the permission profile a grant is
/// checked against (issue #524).
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct GrantCheckConfig {
    pub(crate) cwd: AbsolutePathBuf,
    pub(crate) workspace_roots: Vec<AbsolutePathBuf>,
    pub(crate) profile: codex_protocol::models::PermissionProfile,
}

/// [`strip_grant`] on a blocking thread (issue #524). If that fails the
/// grant is dropped and the person is told (fail closed).
pub(crate) async fn strip_grant_off_thread(
    level: SecurityLevel,
    codex_home: std::path::PathBuf,
    sqlite_home: std::path::PathBuf,
    configs: Vec<GrantCheckConfig>,
    grant: AdditionalPermissionProfile,
) -> (Option<AdditionalPermissionProfile>, Option<String>) {
    tokio::task::spawn_blocking(move || {
        strip_grant(level, &codex_home, &sqlite_home, &configs, grant)
    })
    .await
    .unwrap_or_else(|_| {
        tracing::warn!(
            target: "codex_core::security::tainted_action",
            "the permission grant could not be checked; dropped"
        );
        (None, Some(unchecked_refusal(level)))
    })
}

/// Issue #524: the refusal when the protected-path check itself failed.
pub(crate) fn unchecked_refusal(level: SecurityLevel) -> String {
    let level = match level {
        SecurityLevel::Aggressive => "Aggressive",
        _ => "Moderate",
    };
    format!(
        "Security level {level} could not check the additional permissions against the \
         protected paths, so they were refused. Try again; to allow it without the check, a \
         person can switch to Permissive in /security (it applies after a restart)."
    )
}

/// The refusal for an additional permission that would open `path`, in the
/// words #442 uses for a protected read under Aggressive.
pub(crate) fn refusal(level: SecurityLevel, path: &str) -> String {
    match level {
        SecurityLevel::Aggressive => super::protected_reads::explanation(path),
        // Only Moderate reaches here: Permissive has no protected paths.
        _ => {
            let path = super::protected_reads::shown_path(path);
            format!(
                "Security level Moderate blocked access to `{path}`: an additional permission \
                 can never open protected files (credential files such as .env, SSH, cloud and \
                 CLI tokens, and Corbanu's sign-in, vault, config, history, security state and \
                 keys). To allow it, a \
                 person can switch to Permissive in /security (it applies after a restart)."
            )
        }
    }
}

/// Issue #513: the note on a Moderate approval prompt whose additional
/// write permission opens the persistence file `path`: approving it is what
/// lets this one command change it.
pub(crate) fn persistence_note(path: &str) -> String {
    let path = super::protected_reads::shown_path(path);
    format!(
        "Approving lets this command write `{path}`, which runs code or sets policy later \
         (Moderate keeps it read-only otherwise)."
    )
}

/// Issue #513: the refusal for an additional permission that would make the
/// read-only `path` writable under `level`, by what may lift it (`rule`).
pub(crate) fn write_refusal(level: SecurityLevel, path: &str, rule: WriteRule) -> String {
    let path = super::protected_reads::shown_path(path);
    let switch = "a person can switch to Permissive in /security (it applies after a restart).";
    match (rule, level) {
        (WriteRule::Runtime, _) => {
            let level = match level {
                SecurityLevel::Aggressive => "Aggressive",
                _ => "Moderate",
            };
            format!(
                "Security level {level} blocked writing to `{path}`: Corbanu's runtime files (the \
                 sandbox helpers in its tmp folder, shell snapshots, skills, plugins, packages \
                 and AGENTS.md) stay read-only, and this cannot be granted, not even once. To \
                 allow it, {switch}"
            )
        }
        (WriteRule::Liftable, SecurityLevel::Aggressive) => format!(
            "Security level Aggressive blocked writing to `{path}`: files that run code or set \
             policy later (shell start-up files, git hooks and config) stay read-only, and an \
             additional permission cannot change that. Where a \
             person approving the command is offered \"Grant once\", it lifts this for that \
             command; otherwise {switch}"
        ),
        (WriteRule::NeverGranted, SecurityLevel::Aggressive) => format!(
            "Security level Aggressive blocked writing to `{path}`: it stays read-only under \
             Aggressive, and neither an additional permission nor \"Grant once\" can change \
             that. To allow it, {switch}"
        ),
        // Moderate (Permissive has no read-only paths).
        (WriteRule::Liftable | WriteRule::NeverGranted, _) => format!(
            "Security level Moderate blocked writing to `{path}`: files that run code or set \
             policy later (shell start-up files, git hooks and config, the project's .codex and \
             .agents) stay read-only. Only a person's own approval of this command, asked about \
             it now, lifts that for the command; the automatic reviewer, a permission hook, an \
             earlier \"don't ask again\" answer or a permission grant cannot. Otherwise {switch}"
        ),
    }
}

#[cfg(test)]
#[path = "protected_additional_tests.rs"]
mod tests;
