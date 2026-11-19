//! D729 (A), #2661: the command a PowerShell script read in full runs as, and
//! which names its shell finds.
//!
//! A program name must be a program on the command's `PATH` (else PowerShell
//! may load a module's command or alias of that name, or `Get-<name>`), and a
//! module cmdlet must not be shadowed by a program or script of its name
//! there. A command for a remote environment finds no names on this host.

use std::cell::RefCell;
use std::marker::PhantomData;
use std::rc::Rc;

use codex_utils_absolute_path::AbsolutePathBuf;

thread_local! {
    static SCOPE: RefCell<Option<Rc<PowershellCommandScope>>> = const { RefCell::new(None) };
}

/// Where a command runs, and the environment variables its shell finds
/// programs with.
#[derive(Debug, Clone)]
pub struct PowershellCommandScope {
    /// Whether the command runs on this host (not a remote environment).
    local: bool,
    /// Whether the allowlist applies: a prompt or forbidden rule exists, which
    /// a script read in part might hide a command from (D733: otherwise the
    /// literal reading, as before D729).
    allowlist: bool,
    path: Option<std::ffi::OsString>,
    extensions: Option<String>,
}

impl PowershellCommandScope {
    /// `cwd` is where the command runs on this host (`None`: elsewhere, so no
    /// program name can be found); `env` is the command's environment where
    /// known (`PATH`, `PATHEXT`, in any case), otherwise Corbanu's own.
    pub fn new(
        cwd: Option<&AbsolutePathBuf>,
        env: Option<&std::collections::HashMap<String, String>>,
    ) -> Self {
        let var = |name: &str| match env {
            Some(env) => env
                .iter()
                .find(|(key, _)| key.eq_ignore_ascii_case(name))
                .map(|(_, value)| value.clone()),
            None => std::env::var(name).ok(),
        };
        Self {
            local: cwd.is_some(),
            allowlist: true,
            path: var("PATH").map(Into::into),
            extensions: var("PATHEXT"),
        }
    }

    /// Whether the allowlist applies (default: yes); see [`Self::allowlist`].
    pub fn with_allowlist(mut self, allowlist: bool) -> Self {
        self.allowlist = allowlist;
        self
    }
}

/// Whether scripts read on this thread are read with the allowlist: yes
/// outside a scope (no command to check, as in tests).
pub(crate) fn allowlist_applies() -> bool {
    current_scope().is_none_or(|scope| scope.allowlist)
}

fn current_scope() -> Option<Rc<PowershellCommandScope>> {
    SCOPE.with(|current| current.borrow().clone())
}

/// While the returned guard lives, PowerShell scripts read on this thread are
/// those of the command `scope` describes. The guard is not `Send`, so it
/// cannot be held across an `.await`.
#[must_use]
pub fn enter_powershell_command_scope(
    scope: PowershellCommandScope,
) -> PowershellCommandScopeGuard {
    let previous = SCOPE.with(|current| current.replace(Some(Rc::new(scope))));
    PowershellCommandScopeGuard {
        previous,
        _not_send: PhantomData,
    }
}

pub struct PowershellCommandScopeGuard {
    previous: Option<Rc<PowershellCommandScope>>,
    _not_send: PhantomData<*const ()>,
}

impl Drop for PowershellCommandScopeGuard {
    fn drop(&mut self) {
        let previous = self.previous.take();
        SCOPE.with(|current| *current.borrow_mut() = previous);
    }
}

/// D729 (A): whether each of `programs` is a program (or script) on the
/// command's `PATH`, and none of the module cmdlets `cmdlets` is shadowed by
/// one there. Only absolute local folders are looked in; a relative or
/// network folder leaves a cmdlet's shadow unknown (no). A command running
/// elsewhere (no local folder) has no names this host can find. Outside a
/// scope (no command to check, as in safety heuristics and tests) names are
/// not looked up: the approval path always checks within one.
pub(crate) fn names_resolve(programs: &[String], cmdlets: &[String]) -> bool {
    if programs.is_empty() && cmdlets.is_empty() {
        return true;
    }
    let Some(scope) = current_scope() else {
        return true;
    };
    if !scope.local {
        return false;
    }
    let lookup = PathLookup::new(scope.path.as_deref(), scope.extensions.as_deref());
    programs.iter().all(|name| lookup.find(name) == Found::Yes)
        && cmdlets.iter().all(|name| lookup.find(name) == Found::No)
}

#[derive(Debug, PartialEq, Eq)]
enum Found {
    Yes,
    No,
    /// A folder that was not looked in may hold it.
    Unknown,
}

/// `PATH` as PowerShell searches it for a name: each folder in turn, the
/// name with each `PATHEXT` extension or `.ps1` (or as given if it has an
/// extension).
struct PathLookup {
    dirs: Vec<Option<std::path::PathBuf>>,
    extensions: Vec<String>,
}

impl PathLookup {
    fn new(path: Option<&std::ffi::OsStr>, pathext: Option<&str>) -> Self {
        // An entry rooted at the command's drive (`/usr/bin`, `\\tools`) is
        // not looked in, as a relative one.
        let dirs = path
            .map(std::env::split_paths)
            .into_iter()
            .flatten()
            .filter(|dir| !dir.as_os_str().is_empty())
            .map(|dir| {
                // Never a network folder (`\\server\share`, `//server`): looking
                // there before approval would reach out to it.
                let text = dir.to_string_lossy();
                let local =
                    dir.is_absolute() && !text.starts_with("\\\\") && !text.starts_with("//");
                local.then_some(dir)
            })
            .collect();
        let extensions = pathext
            .unwrap_or(".COM;.EXE;.BAT;.CMD")
            .split(';')
            .chain(std::iter::once(".ps1"))
            .map(str::trim)
            .filter(|extension| extension.starts_with('.') && extension.len() > 1)
            .map(str::to_string)
            .collect();
        Self { dirs, extensions }
    }

    fn find(&self, name: &str) -> Found {
        if name.is_empty() || name.contains(['/', '\\', ':', '*', '?']) {
            return Found::Unknown;
        }
        let has_extension = name
            .rsplit_once('.')
            .is_some_and(|(stem, _)| !stem.is_empty());
        let mut unknown = false;
        for dir in &self.dirs {
            let Some(dir) = dir else {
                unknown = true;
                continue;
            };
            if unknown {
                // A folder not looked in comes first: it may hold the name.
                return Found::Unknown;
            }
            let found = if has_extension {
                dir.join(name).is_file()
            } else {
                self.extensions
                    .iter()
                    .any(|extension| dir.join(format!("{name}{extension}")).is_file())
            };
            if found {
                return Found::Yes;
            }
        }
        if unknown { Found::Unknown } else { Found::No }
    }
}

#[cfg(test)]
#[path = "command_scope_tests.rs"]
mod tests;
