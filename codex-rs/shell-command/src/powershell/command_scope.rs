//! D729 (A), #2661: the command a PowerShell script read in full runs as, and
//! which names its shell finds.
//!
//! A program name must be a program on the command's `PATH` (else PowerShell
//! may load a module's command or alias of that name, or `Get-<name>`), and a
//! module cmdlet must not be shadowed by a program or script of its name
//! there. A command for a remote environment finds no names on this host.
//!
//! Sync: no PowerShell runs before approval (scripts are lowered in-process,
//! also for a Windows executor seen from another host, #2671), so this also
//! answers what main asks the parser's PowerShell: whether a name is a program
//! on `PATH` ([`program_on_path`]) and whether a profile script may run first
//! ([`profile_may_exist`]).

use std::cell::RefCell;
use std::marker::PhantomData;
use std::path::Path;
use std::path::PathBuf;
use std::rc::Rc;

use super::PowershellKind;

use codex_utils_absolute_path::AbsolutePathBuf;

thread_local! {
    static SCOPE: RefCell<Option<Rc<PowershellCommandScope>>> = const { RefCell::new(None) };
}

/// Where a command runs, and the environment variables its shell finds
/// programs with.
#[derive(Debug, Clone)]
pub struct PowershellCommandScope {
    /// Sync: whether there is a command to check; otherwise (heuristics,
    /// tests) names are not looked up, as outside a scope.
    checked: bool,
    /// Whether the command runs on this host (not a remote environment).
    local: bool,
    /// Whether the allowlist applies: a prompt or forbidden rule exists, which
    /// a script read in part might hide a command from (D733: otherwise the
    /// literal reading, as before D729).
    allowlist: bool,
    path: Option<std::ffi::OsString>,
    extensions: Option<String>,
    /// #2756: where the command's environment has no `PATHEXT`, Corbanu's
    /// own, which its spawn may fill in; also looked for when deciding that
    /// no program shadows a module cmdlet.
    fallback_extensions: Option<String>,
    /// Sync: the user's Documents folder on this host, where PowerShell keeps
    /// the user's profile scripts; `None`: unknown (a profile may exist).
    documents: Option<PathBuf>,
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
        let extensions = var("PATHEXT");
        let fallback_extensions = if env.is_some() && extensions.is_none() {
            std::env::var("PATHEXT").ok()
        } else {
            None
        };
        Self {
            checked: true,
            local: cwd.is_some(),
            allowlist: true,
            path: var("PATH").map(Into::into),
            extensions,
            fallback_extensions,
            documents: None,
        }
    }

    /// Test support: the `PATHEXT` Corbanu's own environment would give a
    /// command whose environment has none.
    #[cfg(test)]
    fn with_fallback_extensions(mut self, extensions: Option<&str>) -> Self {
        self.fallback_extensions = extensions.map(str::to_string);
        self
    }

    /// Sync: no command to check (heuristics, tests): names are not looked up,
    /// as outside a scope, and only whether the allowlist applies is set, so
    /// the approval path reads scripts alike with or without a command
    /// (D733).
    pub fn unchecked() -> Self {
        Self {
            checked: false,
            local: false,
            allowlist: true,
            path: None,
            extensions: None,
            fallback_extensions: None,
            documents: None,
        }
    }

    /// Sync: the user's Documents folder on this host (see
    /// [`profile_may_exist`]).
    pub fn with_documents(mut self, documents: Option<PathBuf>) -> Self {
        self.documents = documents;
        self
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
/// #2756: a command whose environment has no `PATH` finds no name either:
/// where it is spawned, `PATH` may be filled in (the Windows sandbox gives
/// it Corbanu's own), so a cmdlet's shadow there is unknown too.
pub(crate) fn names_resolve(programs: &[String], cmdlets: &[String]) -> bool {
    if programs.is_empty() && cmdlets.is_empty() {
        return true;
    }
    let Some(scope) = current_scope().filter(|scope| scope.checked) else {
        return true;
    };
    if !scope.local || scope.path.is_none() {
        return false;
    }
    let lookup = PathLookup::new(scope.path.as_deref(), scope.extensions.as_deref());
    // #2756: a command whose environment has no `PATHEXT` may be spawned with
    // Corbanu's (`.VBS`, `.JS`, …): a program with any of those extensions
    // may shadow a cmdlet too.
    let shadow_extensions = match (&scope.extensions, &scope.fallback_extensions) {
        (None, Some(fallback)) => Some(format!("{DEFAULT_PATHEXT};{fallback}")),
        (extensions, _) => extensions.clone(),
    };
    let shadows = PathLookup::new(scope.path.as_deref(), shadow_extensions.as_deref());
    programs.iter().all(|name| lookup.find(name) == Found::Yes)
        && cmdlets.iter().all(|name| shadows.find(name) == Found::No)
}

/// The extensions a `PATH` search tries where `PATHEXT` is not set.
const DEFAULT_PATHEXT: &str = ".COM;.EXE;.BAT;.CMD";

/// Sync (D729 A, main's `Find-D729Program`): whether a program (not a
/// PowerShell script) named `name` is on the command's `PATH`. A command
/// running elsewhere finds none; outside a scope, Corbanu's `PATH` is looked
/// in, as main's parser looks in its own.
pub(crate) fn program_on_path(name: &str) -> bool {
    let lookup = match current_scope().filter(|scope| scope.checked) {
        Some(scope) if !scope.local => return false,
        Some(scope) => PathLookup::new(scope.path.as_deref(), scope.extensions.as_deref()),
        None => PathLookup::new(
            std::env::var_os("PATH").as_deref(),
            std::env::var("PATHEXT").ok().as_deref(),
        ),
    };
    lookup.without_scripts().find(name) == Found::Yes
}

/// Sync (D729 A, main's `Test-D729Profile`): whether a profile script may run
/// before the script of `shell`, a PowerShell of `kind` started without
/// `-NoProfile`: one of the four console profiles exists in its install
/// folder (`$PSHOME`) or the user's Documents folder. Only a command on this
/// host can tell; for one running elsewhere, or where a folder is unknown or
/// cannot be read, one may. Outside a scope none is looked for, as names are
/// not (see [`names_resolve`]).
pub(crate) fn profile_may_exist(shell: &str, kind: PowershellKind) -> bool {
    let Some(scope) = current_scope().filter(|scope| scope.checked) else {
        return false;
    };
    if !scope.local {
        return true;
    }
    let Some(documents) = scope.documents.as_deref() else {
        return true;
    };
    let shell_path = Path::new(shell);
    let given_home = shell_path
        .is_absolute()
        .then(|| shell_path.parent().map(Path::to_path_buf))
        .flatten();
    let (homes, user_folders) = match kind {
        PowershellKind::WindowsPowershell => {
            // Windows PowerShell's `$PSHOME` is fixed; a path given also counts.
            let Some(root) = std::env::var_os("SystemRoot") else {
                return true;
            };
            let mut homes = vec![
                PathBuf::from(root)
                    .join("System32")
                    .join("WindowsPowerShell")
                    .join("v1.0"),
            ];
            homes.extend(given_home);
            // A `powershell` that is PowerShell 7 reads its user's profile
            // there: both are looked in.
            (homes, &["WindowsPowerShell", "PowerShell"][..])
        }
        PowershellKind::Pwsh => {
            let home = match given_home {
                Some(home) => home,
                None => {
                    let lookup =
                        PathLookup::new(scope.path.as_deref(), scope.extensions.as_deref());
                    match lookup.locate(shell) {
                        Ok(Some(found)) => match found.parent() {
                            Some(parent) => parent.to_path_buf(),
                            None => return true,
                        },
                        // Not found (it cannot start) or unknown.
                        _ => return true,
                    }
                }
            };
            // A folder that is not PowerShell's install folder (a launcher
            // shim such as a dotnet tool's) leaves `$PSHOME` unknown.
            if !home.join("System.Management.Automation.dll").is_file() {
                return true;
            }
            (vec![home], &["PowerShell"][..])
        }
    };
    let folders = homes.into_iter().chain(
        user_folders
            .iter()
            .map(|user_folder| documents.join(user_folder)),
    );
    folders
        .flat_map(|folder| {
            ["profile.ps1", "Microsoft.PowerShell_profile.ps1"].map(|name| folder.join(name))
        })
        .any(|profile| match std::fs::metadata(&profile) {
            Ok(_) => true,
            Err(error) => error.kind() != std::io::ErrorKind::NotFound,
        })
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
            .unwrap_or(DEFAULT_PATHEXT)
            .split(';')
            .chain(std::iter::once(".ps1"))
            .map(str::trim)
            .filter(|extension| extension.starts_with('.') && extension.len() > 1)
            .map(str::to_string)
            .collect();
        Self { dirs, extensions }
    }

    /// This lookup without PowerShell scripts (`.ps1`): programs only.
    fn without_scripts(mut self) -> Self {
        self.extensions
            .retain(|extension| !extension.eq_ignore_ascii_case(".ps1"));
        self
    }

    fn find(&self, name: &str) -> Found {
        match self.locate(name) {
            Ok(Some(_)) => Found::Yes,
            Ok(None) => Found::No,
            Err(()) => Found::Unknown,
        }
    }

    /// The file `name` names on `PATH`; `Err` where unknown.
    fn locate(&self, name: &str) -> Result<Option<PathBuf>, ()> {
        if name.is_empty() || name.contains(['/', '\\', ':', '*', '?']) {
            return Err(());
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
                return Err(());
            }
            let found = if has_extension {
                Some(dir.join(name)).filter(|file| file.is_file())
            } else {
                self.extensions
                    .iter()
                    .map(|extension| dir.join(format!("{name}{extension}")))
                    .find(|file| file.is_file())
            };
            if found.is_some() {
                return Ok(found);
            }
        }
        if unknown { Err(()) } else { Ok(None) }
    }
}

#[cfg(test)]
#[path = "command_scope_tests.rs"]
mod tests;
