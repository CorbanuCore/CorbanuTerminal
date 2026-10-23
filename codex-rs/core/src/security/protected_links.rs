//! #623 (milestone "Aggressive ships" re-run, AGG-R13): under Aggressive an
//! agent command may not create a link to the project's `.codex` or
//! `.agents` (or into Corbanu's own read-only files): they are always
//! read-only (#520), and a link to them is not created either (Travis,
//! 2026-10-13). Writes through such a link were already refused; now the
//! command is refused before it runs, so nothing is created, and the person
//! is told why.
//!
//! No operating-system sandbox can refuse a symbolic link by where it points
//! (Seatbelt filters only by the link's name or type; Landlock and seccomp
//! cannot read the target), so the command decides before anything runs:
//! its text for agent commands (see [`parse::link_targets`]), and, where the
//! zsh-fork shell intercepts each program a script starts, that program's
//! own arguments (so a script file's `ln` is seen there too).
//!
//! Keys typed into a running shell (`write_stdin`) are read the same way, one
//! write at a time. A command that may create a link (an `ln`, `gln` or
//! `mklink` word anywhere, a link call in code, a PowerShell link item) and
//! names a `.codex` or `.agents` folder anywhere in its text, outside the
//! paths of its own folder and the workspace roots, is refused too: through
//! a variable, `$(…)`, backticks, `eval` or a pipe into a shell, by a glob
//! (`.agen*`), a brace (`.agent{s,}`) or another letter case. So a link
//! cannot be made in two steps (`ln -s y/.agents x`, then `ln -s . y`).
//!
//! This is defense in depth against ordinary link commands, not against
//! deliberate obfuscation. Out of reach: a target or name built at run time
//! (`ln -s "$(cat f)" x`, `$'\x2eagents'`); a link
//! made by a script file without zsh-fork, or typed across several writes;
//! links copied, moved or unpacked (`cp -a`, `mv`, `rsync -l`, `tar`, `git
//! checkout`); tools not listed (`awk system()`, `Rscript`, PowerShell
//! `-EncodedCommand`); remote environments. Over-refusal is accepted: text
//! that only looks like a link command (a heredoc's documentation, `find
//! -name ln`, `grep "ln -s" .agents`), and, in code that calls a link
//! function, every quoted string is taken as a target.
//! Writes through such a link stay refused by the sandbox; a hard link to a
//! protected file is refused by the sandbox itself.

use std::path::Component;
use std::path::Path;
use std::path::PathBuf;

use codex_utils_absolute_path::AbsolutePathBuf;

use super::protected_reads::shown_path;
use super::protected_surface::ReadDenials;
use crate::session::session::Session;
use crate::session::turn_context::TurnContext;

#[path = "protected_links_names.rs"]
mod names;
#[path = "protected_links_parse.rs"]
mod parse;

use names::settings_name_in;

use parse::Candidate;
#[cfg(test)]
use parse::calls_link;
use parse::link_targets;

/// What a person is told when such a command is refused.
pub(crate) fn explanation(target: &str) -> String {
    format!(
        "Security level Aggressive refused this command before it ran: it creates a link, and \
         `{}` is or may lead to the project's .codex or .agents folders (agent settings) or \
         Corbanu's own files, which are always read-only under Aggressive, so no link to them is \
         created either (nothing was created). This cannot be granted, not even once. To allow \
         it, a person can switch to Permissive in /security (it applies after a restart).",
        shown_path(target)
    )
}

/// The refusal for `command` (run in `cwd`) under Aggressive, if it creates
/// a link to a path no grant lifts; also shown to the person and listed
/// under the inspector's recent denials.
pub(crate) async fn refusal(
    session: &Session,
    turn: &TurnContext,
    command: &[String],
    cwd: &Path,
) -> Option<String> {
    let inputs = Inputs::capture(session, turn)?;
    let found = link_targets(command);
    if !may_link(&found, command) {
        return None;
    }
    let first = found
        .candidates
        .first()
        .map(|candidate| candidate.target.clone())
        .unwrap_or_else(|| "a path it names".to_string());
    let (command, cwd, fallback) = (command.to_vec(), cwd.to_path_buf(), first.clone());
    let started = std::time::Instant::now();
    let found = looked_up(LOOKUP_TIMEOUT, first, move || {
        // It makes a link and names `.codex` or `.agents` anywhere
        // (through a variable, `$(…)`, `eval` or a pipe into a shell
        // too).
        if let Some(named) = inputs.settings_named(&command, &cwd) {
            return Some(Refused {
                target: named,
                by: RefusedBy::SettingsNamed,
            });
        }
        // Too many paths to look at: refused, naming the first.
        if found.overflow {
            return Some(Refused {
                target: fallback,
                by: RefusedBy::Overflow,
            });
        }
        inputs.protected_target(&found.candidates, &cwd)
    })
    .await?;
    log_refusal(&found, started.elapsed());
    let refusal = explanation(&found.target);
    super::protected_reads::notify(session, turn, refusal.clone()).await;
    super::inspection::record_refusal(
        session.thread_id(),
        "creating a link to the project's .codex or .agents, or Corbanu's files",
        "refused before it ran; never granted under Aggressive",
    );
    Some(refusal)
}

/// Whether `command` may create a link (see [`refusal`]); cheap, no lookups.
pub(crate) fn may_link_command(command: &[String]) -> bool {
    may_link(&link_targets(command), command)
}

fn may_link(found: &parse::Found, command: &[String]) -> bool {
    !found.candidates.is_empty() || found.overflow || parse::makes_links(command)
}

/// Whether the #623 check applies to this session (Aggressive), before any
/// text is read.
pub(crate) fn applies(session: &Session) -> bool {
    session
        .services
        .model_client()
        .post_taint_state()
        .is_some_and(|state| state.level == codex_security_policy::SecurityLevel::Aggressive)
}

/// How long the path lookups may take before the command is refused anyway.
/// #744: 2 s refused ordinary commands on a loaded machine; 10 s (Travis,
/// 2026-10-14) still fails closed.
const LOOKUP_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(10);

/// A refused command: the target it is told about, and which check refused
/// it.
#[derive(Clone, Debug, PartialEq, Eq)]
struct Refused {
    target: String,
    by: RefusedBy,
}

/// #744 (Travis, 2026-10-14): which check refused a command, logged so a
/// rare refusal of an ordinary command can be traced from CI and user logs.
#[derive(Clone, Debug, PartialEq, Eq)]
enum RefusedBy {
    /// The lookups did not finish within the timeout.
    Timeout,
    /// The lookups failed.
    LookupFailed,
    /// It names `.codex` or `.agents` (`target` is the name found).
    SettingsNamed,
    /// Too many paths to look at.
    Overflow,
    /// `target` read from a folder lands in `kept`, a path no grant lifts,
    /// as `landed` (spelled, through a link, or where it finally points).
    ProtectedTarget {
        landed: AbsolutePathBuf,
        kept: AbsolutePathBuf,
    },
}

impl RefusedBy {
    fn name(&self) -> &'static str {
        match self {
            Self::Timeout => "timeout",
            Self::LookupFailed => "lookup_failed",
            Self::SettingsNamed => "settings_named",
            Self::Overflow => "overflow",
            Self::ProtectedTarget { .. } => "protected_target",
        }
    }
}

/// How many refusals [`recent_refusals`] keeps.
const RECENT_LIMIT: usize = 16;

/// The latest refusals, as logged (for tests to report, #744).
static RECENT_REFUSALS: std::sync::Mutex<std::collections::VecDeque<String>> =
    std::sync::Mutex::new(std::collections::VecDeque::new());

/// Logs which check refused a command and keeps it among the latest. The
/// command's text is not logged; its target and where it landed are shown
/// as the refusal shows the target (escaped and bounded, `shown_path`).
fn log_refusal(refused: &Refused, elapsed: std::time::Duration) {
    let mut detail = format!(
        "refused_by={} target={}",
        refused.by.name(),
        shown_path(&refused.target)
    );
    if let RefusedBy::ProtectedTarget { landed, kept } = &refused.by {
        detail.push_str(&format!(
            " landed={} kept={}",
            shown_path(&landed.as_path().to_string_lossy()),
            shown_path(&kept.as_path().to_string_lossy())
        ));
    }
    detail.push_str(&format!(" elapsed_ms={}", elapsed.as_millis()));
    tracing::warn!("Aggressive link check (#623) refused a command: {detail}");
    let mut recent = RECENT_REFUSALS
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    if recent.len() == RECENT_LIMIT {
        recent.pop_front();
    }
    recent.push_back(detail);
}

/// The latest refusals in this process, oldest first, as logged.
pub(crate) fn recent_refusals() -> Vec<String> {
    RECENT_REFUSALS
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .iter()
        .cloned()
        .collect()
}

/// What `lookup` finds, run off the async threads; `first` (refused, nothing
/// is created) when it does not finish within `timeout` or fails.
async fn looked_up(
    timeout: std::time::Duration,
    first: String,
    lookup: impl FnOnce() -> Option<Refused> + Send + 'static,
) -> Option<Refused> {
    match tokio::time::timeout(timeout, tokio::task::spawn_blocking(lookup)).await {
        Ok(Ok(found)) => found,
        Ok(Err(_)) => Some(Refused {
            target: first,
            by: RefusedBy::LookupFailed,
        }),
        Err(_) => Some(Refused {
            target: first,
            by: RefusedBy::Timeout,
        }),
    }
}

/// What the protected paths are built from (see `protected_reads`).
struct Inputs {
    codex_home: PathBuf,
    sqlite_home: PathBuf,
    turn_cwd: AbsolutePathBuf,
    workspace_roots: Vec<AbsolutePathBuf>,
    profile: codex_protocol::models::PermissionProfile,
    user_home: Option<PathBuf>,
}

impl Inputs {
    /// Only under Aggressive, for a local command. The level is the one the
    /// session's protections run at (`post_taint_state`, as #442/#637 and
    /// Grant once read it).
    fn capture(session: &Session, turn: &TurnContext) -> Option<Self> {
        session
            .services
            .model_client()
            .post_taint_state()
            .filter(|state| state.level == codex_security_policy::SecurityLevel::Aggressive)?;
        if turn
            .environments
            .primary()
            .is_some_and(|environment| environment.environment.is_remote())
        {
            return None;
        }
        #[allow(deprecated)]
        let turn_cwd = turn.cwd.clone();
        let workspace_roots = turn
            .environments
            .primary()
            .map(|environment| {
                environment
                    .workspace_roots()
                    .iter()
                    .filter_map(|root| root.to_abs_path().ok())
                    .collect()
            })
            .unwrap_or_default();
        Some(Self {
            codex_home: turn.config.codex_home.to_path_buf(),
            sqlite_home: turn.config.sqlite.home().to_path_buf(),
            turn_cwd,
            workspace_roots,
            profile: turn.config.permissions.permission_profile().clone(),
            user_home: dirs::home_dir(),
        })
    }

    /// The first candidate, as spelled, that lands in a path no grant lifts,
    /// and where it lands.
    fn protected_target(&self, candidates: &[Candidate], cwd: &Path) -> Option<Refused> {
        let denials = ReadDenials::for_turn(
            &self.codex_home,
            &self.sqlite_home,
            &self.turn_cwd,
            &self.workspace_roots,
            &self.profile,
        );
        let mut kept: Vec<AbsolutePathBuf> = denials.never_lifted_paths().cloned().collect();
        let real: Vec<AbsolutePathBuf> =
            kept.iter().filter_map(ReadDenials::real_location).collect();
        kept.extend(real);
        let writable: Vec<&AbsolutePathBuf> = denials.home_writable_paths().collect();
        candidates.iter().find_map(|candidate| {
            self.bases(candidate, cwd)
                .iter()
                .flat_map(|base| self.resolve(&candidate.target, base))
                .find_map(|path| landing(&path, &kept, &writable))
                .map(|(landed, kept)| Refused {
                    target: candidate.target.clone(),
                    by: RefusedBy::ProtectedTarget { landed, kept },
                })
        })
    }

    /// The agent settings name `command` spells anywhere, with the command's
    /// folder, the turn's and the workspace roots (each by its real location
    /// too) left out: a project kept below a folder named `.codex` is not
    /// one.
    fn settings_named(&self, command: &[String], cwd: &Path) -> Option<String> {
        let mut text = command.join(" ");
        let mut roots: Vec<PathBuf> = std::iter::once(cwd.to_path_buf())
            .chain(std::iter::once(self.turn_cwd.to_path_buf()))
            .chain(
                self.workspace_roots
                    .iter()
                    .map(AbsolutePathBuf::to_path_buf),
            )
            .collect();
        let real: Vec<PathBuf> = roots
            .iter()
            .filter_map(|root| std::fs::canonicalize(root).ok())
            .collect();
        roots.extend(real);
        // Longest first, so a root inside another is removed whole.
        roots.sort_by_key(|root| std::cmp::Reverse(root.as_os_str().len()));
        for root in roots {
            let root = root.to_string_lossy();
            if root.len() > 1 {
                text = text.replace(root.as_ref(), ".");
            }
        }
        settings_name_in(&text)
    }

    /// The folders a candidate's relative target may be read from: the
    /// command's, and each of the candidate's folders (expanded as targets
    /// are, from the command's folder).
    fn bases(&self, candidate: &Candidate, cwd: &Path) -> Vec<PathBuf> {
        let mut bases = vec![cwd.to_path_buf()];
        for folder in &candidate.link_folders {
            let folder = match self.expand(folder, cwd) {
                Some(folder) => cwd.join(folder),
                None => continue,
            };
            if !bases.contains(&folder) {
                bases.push(folder);
            }
        }
        bases
    }

    /// Where `word` points from `base`: as written (`..` taken from the text)
    /// and as the system reads it (`..` taken after following each link on
    /// the way, `ln -s evil/../.agents` with `evil` a link).
    fn resolve(&self, word: &str, base: &Path) -> Vec<AbsolutePathBuf> {
        let Some(expanded) = self.expand(word, base) else {
            return Vec::new();
        };
        let written = AbsolutePathBuf::resolve_path_against_base(&expanded, base);
        let mut resolved = vec![written];
        if let Some(physical) = physical(base, Path::new(&expanded))
            && !resolved.contains(&physical)
        {
            resolved.push(physical);
        }
        resolved
    }

    /// `word` with the home, Corbanu home and shell folder variables
    /// expanded (the shell's folder is `base`); `None` where another
    /// variable is left.
    fn expand(&self, word: &str, base: &Path) -> Option<String> {
        let codex_home = self.codex_home.to_string_lossy();
        let base_text = base.to_string_lossy();
        let mut expanded = word.to_string();
        for name in ["CODEX_HOME", "CORBANU_HOME", "PFTERMINAL_HOME"] {
            for spelling in [format!("${{{name}}}"), format!("${name}")] {
                expanded = expanded.replace(&spelling, &codex_home);
            }
        }
        // The shell's folder.
        for spelling in ["${PWD}", "$PWD", "$(pwd)", "`pwd`"] {
            expanded = expanded.replace(spelling, &base_text);
        }
        if let Some(home) = self.user_home.as_deref().map(Path::to_string_lossy) {
            for spelling in ["${HOME}", "$HOME"] {
                expanded = expanded.replace(spelling, &home);
            }
            if expanded == "~" {
                expanded = home.into_owned();
            } else if let Some(rest) = expanded.strip_prefix("~/") {
                expanded = format!("{home}/{rest}");
            }
        }
        (!expanded.is_empty() && !expanded.contains('$') && !expanded.starts_with('~'))
            .then_some(expanded)
    }
}

/// `path` from `base`, each existing name followed to where it links before
/// the next `..` applies, as the system reads it. Bounded by the path's
/// length; `None` where a name cannot be read.
fn physical(base: &Path, path: &Path) -> Option<AbsolutePathBuf> {
    let mut current = if path.is_absolute() {
        PathBuf::new()
    } else {
        std::fs::canonicalize(base).unwrap_or_else(|_| base.to_path_buf())
    };
    for component in path.components() {
        match component {
            Component::Prefix(_) | Component::RootDir => current.push(component.as_os_str()),
            Component::CurDir => {}
            Component::ParentDir => {
                current.pop();
            }
            Component::Normal(name) => {
                current.push(name);
                if let Ok(real) = std::fs::canonicalize(&current) {
                    current = real;
                }
            }
        }
    }
    AbsolutePathBuf::from_absolute_path(current).ok()
}

/// Where `path`, as spelled, through a link on the way or where it finally
/// points, is in one of `kept` and not in a folder of `writable` inside it
/// (`worktrees` in the Corbanu home, #583): that spelling and the kept path.
fn landing(
    path: &AbsolutePathBuf,
    kept: &[AbsolutePathBuf],
    writable: &[&AbsolutePathBuf],
) -> Option<(AbsolutePathBuf, AbsolutePathBuf)> {
    let mut spellings = vec![path.clone()];
    spellings.extend(ReadDenials::real_location(path));
    spellings.into_iter().find_map(|spelling| {
        kept.iter()
            .find(|kept| {
                spelling.as_path().starts_with(kept.as_path())
                    && !writable.iter().any(|folder| {
                        spelling.as_path().starts_with(folder.as_path())
                            && folder.as_path().starts_with(kept.as_path())
                    })
            })
            .map(|kept| (spelling, kept.clone()))
    })
}

#[cfg(test)]
#[path = "protected_links_tests.rs"]
mod tests;
