//! D729 (A), #2661: where a PowerShell script read in full may write, and which
//! names it may run.
//!
//! An allowlisted cmdlet that writes (`Set-Content`, `New-Item`, `Copy-Item`, …) keeps a
//! script readable only when every path it writes or removes, resolved as Windows
//! resolves it, lies inside a writable root (outside its read-only parts; a removed path
//! not above them either), is not a file Windows runs from a folder (a `PATHEXT`
//! extension, `.dll`), and is neither inside nor above a folder PowerShell or Windows runs
//! code from (module and profile folders, `PATH`). Paths are compared normalised and
//! case-insensitively, after resolving the part that exists (links, junctions, 8.3
//! names). Outside a scope (see [`enter_powershell_write_scope`]) no write keeps a script
//! readable.
//!
//! A program name must be a program on the command's `PATH` (else PowerShell may load a
//! module's command or alias of that name, or `Get-<name>`), and a module cmdlet must not
//! be shadowed by a program or script of its name there.

use std::cell::RefCell;
use std::marker::PhantomData;
use std::rc::Rc;

use codex_protocol::protocol::WritableRoot;
use codex_utils_absolute_path::AbsolutePathBuf;

thread_local! {
    static SCOPE: RefCell<Option<Rc<PowershellWriteScope>>> = const { RefCell::new(None) };
}

/// The directory a command runs in, the roots its writes may stay in, and the
/// environment variables its shell finds programs and modules with.
#[derive(Debug, Clone)]
pub struct PowershellWriteScope {
    cwd: String,
    roots: Vec<WritableRoot>,
    path: Option<std::ffi::OsString>,
    module_path: Option<String>,
    extensions: Option<String>,
}

impl PowershellWriteScope {
    /// `env` is the command's environment where known (`PATH`, `PATHEXT`,
    /// `PSModulePath`, in any case); otherwise Corbanu's own.
    pub fn new(
        cwd: &AbsolutePathBuf,
        roots: Vec<WritableRoot>,
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
            cwd: cwd.to_string_lossy().into_owned(),
            roots,
            path: var("PATH").map(Into::into),
            module_path: var("PSModulePath"),
            extensions: var("PATHEXT"),
        }
    }
}

fn current_scope() -> Option<Rc<PowershellWriteScope>> {
    SCOPE.with(|current| current.borrow().clone())
}

/// While the returned guard lives, PowerShell scripts read on this thread may write
/// inside `scope`. The guard is not `Send`, so it cannot be held across an `.await`.
#[must_use]
pub fn enter_powershell_write_scope(scope: PowershellWriteScope) -> PowershellWriteScopeGuard {
    let previous = SCOPE.with(|current| current.replace(Some(Rc::new(scope))));
    PowershellWriteScopeGuard {
        previous,
        _not_send: PhantomData,
    }
}

pub struct PowershellWriteScopeGuard {
    previous: Option<Rc<PowershellWriteScope>>,
    _not_send: PhantomData<*const ()>,
}

impl Drop for PowershellWriteScopeGuard {
    fn drop(&mut self) {
        let previous = self.previous.take();
        SCOPE.with(|current| *current.borrow_mut() = previous);
    }
}

/// Whether every path in `writes` and `removes` stays where the current
/// scope lets a script write (`protected`: the parser's folders, to which
/// the scope's `PATH` and `PSModulePath` folders are added).
pub(crate) fn writes_allowed(writes: &[String], removes: &[String], protected: &[String]) -> bool {
    let Some(scope) = current_scope() else {
        return false;
    };
    let Some(cwd) = WinPath::parse(&scope.cwd, None, Strictness::Lax).and_then(canonical) else {
        return false;
    };
    let roots: Vec<Root> = scope.roots.iter().filter_map(Root::from_writable).collect();
    let scope_dirs = scope
        .path
        .iter()
        .flat_map(std::env::split_paths)
        .map(|dir| dir.to_string_lossy().into_owned())
        .chain(
            scope
                .module_path
                .iter()
                .flat_map(|value| value.split(';').map(str::to_string)),
        )
        .collect::<Vec<_>>();
    let protected: Vec<WinPath> = protected
        .iter()
        .chain(&scope_dirs)
        .map(|dir| dir.trim().trim_matches('"'))
        .filter(|dir| !dir.is_empty())
        .filter_map(|dir| WinPath::parse(dir, Some(&cwd), Strictness::Lax))
        .filter_map(canonical)
        .collect();
    let extensions = executable_extensions(scope.extensions.as_deref());
    allows(
        writes,
        removes,
        &Places {
            cwd: &cwd,
            roots: &roots,
            protected: &protected,
            extensions: &extensions,
        },
    )
}

/// Where a script's paths resolve and may go.
struct Places<'a> {
    cwd: &'a WinPath,
    roots: &'a [Root],
    protected: &'a [WinPath],
    /// Extensions of files Windows runs from a folder, folded.
    extensions: &'a [String],
}

fn allows(writes: &[String], removes: &[String], places: &Places<'_>) -> bool {
    let resolve = |path: &String| {
        WinPath::parse(path, Some(places.cwd), Strictness::Write).and_then(canonical)
    };
    let stays = |target: &WinPath| {
        places.roots.iter().any(|root| root.allows(target))
            && !places
                .protected
                .iter()
                .any(|dir| target.is_within(dir) || dir.is_within(target))
            && !target.parts.last().is_some_and(|name| {
                places
                    .extensions
                    .iter()
                    .any(|extension| name.ends_with(extension.as_str()))
            })
    };
    writes
        .iter()
        .all(|write| resolve(write).is_some_and(|target| stays(&target)))
        && removes.iter().all(|remove| {
            // Removing (or moving away) a folder above a root's read-only
            // part removes that part too.
            resolve(remove).is_some_and(|target| {
                stays(&target)
                    && !places
                        .roots
                        .iter()
                        .any(|root| root.holds_read_only(&target))
            })
        })
}

/// `PATHEXT` (or Windows' default) and `.dll`, folded: files Windows runs, or
/// loads, from a folder by name (cmd.exe looks in the current one first).
fn executable_extensions(pathext: Option<&str>) -> Vec<String> {
    pathext
        .unwrap_or(".COM;.EXE;.BAT;.CMD;.VBS;.VBE;.JS;.JSE;.WSF;.WSH;.MSC")
        .split(';')
        .chain(std::iter::once(".dll"))
        .map(str::trim)
        .filter(|extension| extension.starts_with('.') && extension.len() > 1)
        .map(fold)
        .collect()
}

/// D729 (A): whether each of `programs` is a program (or script) on the
/// command's `PATH`, and none of the module cmdlets `cmdlets` is shadowed by
/// one there. Corbanu's own `PATH` outside a scope. Only absolute local
/// folders are looked in; a relative or network folder leaves a cmdlet's
/// shadow unknown (no).
pub(crate) fn names_resolve(programs: &[String], cmdlets: &[String]) -> bool {
    if programs.is_empty() && cmdlets.is_empty() {
        return true;
    }
    let scope = current_scope();
    let (path, extensions) = match &scope {
        Some(scope) => (scope.path.clone(), scope.extensions.clone()),
        None => (std::env::var_os("PATH"), std::env::var("PATHEXT").ok()),
    };
    let lookup = PathLookup::new(path.as_deref(), extensions.as_deref());
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

/// A writable root as compared here.
struct Root {
    root: WinPath,
    read_only: Vec<WinPath>,
    metadata_names: Vec<String>,
}

impl Root {
    fn from_writable(root: &WritableRoot) -> Option<Self> {
        let parse = |path: &AbsolutePathBuf| {
            WinPath::parse(&path.to_string_lossy(), None, Strictness::Lax).and_then(canonical)
        };
        Some(Self {
            root: parse(&root.root)?,
            // A read-only part that cannot be resolved leaves the root out.
            read_only: root
                .read_only_subpaths
                .iter()
                .map(parse)
                .collect::<Option<Vec<_>>>()?,
            metadata_names: root
                .protected_metadata_names
                .iter()
                .map(|name| fold(name))
                .collect(),
        })
    }

    fn allows(&self, target: &WinPath) -> bool {
        target.is_within(&self.root)
            && !self.read_only.iter().any(|path| target.is_within(path))
            && target
                .parts
                .get(self.root.parts.len())
                .is_none_or(|first| !self.metadata_names.contains(first))
    }

    /// Whether `target` holds one of this root's read-only parts (the root
    /// itself, a folder above it, or one above a read-only path).
    fn holds_read_only(&self, target: &WinPath) -> bool {
        self.root.is_within(target) || self.read_only.iter().any(|path| path.is_within(target))
    }
}

/// How strictly a path is read: a written path may hold no wildcard, `~` (home, 8.3
/// names) or escape; a configured folder may.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Strictness {
    Write,
    Lax,
}

/// A local drive path, normalised as Windows normalises it, with case-folded parts.
#[derive(Debug, Clone, PartialEq, Eq)]
struct WinPath {
    drive: char,
    parts: Vec<String>,
}

impl WinPath {
    /// `raw` as Windows resolves it against `cwd`. `None` for anything else: another
    /// provider (`env:`, `HKCU:`, `FileSystem::`), a stream (`a:b`), UNC and device paths,
    /// paths relative to a drive's current folder, reserved device names, and parts
    /// Windows would trim (`x.`).
    fn parse(raw: &str, cwd: Option<&WinPath>, strictness: Strictness) -> Option<Self> {
        // A configured folder may be spelled as a verbatim path (`\\?\C:\x`).
        let raw = match strictness {
            Strictness::Lax => raw.strip_prefix(r"\\?\").unwrap_or(raw),
            Strictness::Write => raw,
        };
        if raw.is_empty() || raw.chars().any(char::is_control) || raw.contains(['<', '>', '|', '"'])
        {
            return None;
        }
        if strictness == Strictness::Write && raw.contains(['*', '?', '[', ']', '`', '~']) {
            return None;
        }
        let bytes = raw.as_bytes();
        let (mut path, rest) = if bytes.len() >= 2 && bytes[1] == b':' {
            let drive = char::from(bytes[0]);
            if !drive.is_ascii_alphabetic() || !matches!(bytes.get(2), Some(b'\\' | b'/')) {
                return None;
            }
            let drive_path = Self {
                drive: drive.to_ascii_uppercase(),
                parts: Vec::new(),
            };
            (drive_path, &raw[3..])
        } else if raw.starts_with(['\\', '/']) {
            return None;
        } else {
            (cwd?.clone(), raw)
        };
        if rest.contains(':') {
            return None;
        }
        for part in rest.split(['\\', '/']) {
            match part {
                "" | "." => {}
                ".." => {
                    path.parts.pop()?;
                }
                _ if part.ends_with(['.', ' ']) || is_device_name(part) => return None,
                _ => path.parts.push(fold(part)),
            }
        }
        Some(path)
    }

    fn is_within(&self, other: &WinPath) -> bool {
        self.drive == other.drive
            && self.parts.len() >= other.parts.len()
            && self.parts.iter().zip(&other.parts).all(|(a, b)| a == b)
    }
}

fn fold(part: &str) -> String {
    part.to_lowercase()
}

/// `CON`, `NUL`, `COM1`, … with or without an extension.
fn is_device_name(part: &str) -> bool {
    let stem = part
        .split('.')
        .next()
        .unwrap_or(part)
        .trim_end()
        .to_ascii_lowercase();
    matches!(
        stem.as_str(),
        "con" | "prn" | "aux" | "nul" | "conin$" | "conout$" | "clock$"
    ) || ((stem.starts_with("com") || stem.starts_with("lpt"))
        && stem.len() == 4
        && stem.as_bytes()[3].is_ascii_digit())
}

/// `path` with its longest existing part resolved on disk (links, junctions, 8.3 names);
/// `None` where that part cannot be resolved or resolves off a local drive.
#[cfg(windows)]
fn canonical(path: WinPath) -> Option<WinPath> {
    let mut existing = path.parts.len();
    loop {
        let mut prefix = std::path::PathBuf::from(format!("{}:\\", path.drive));
        prefix.extend(&path.parts[..existing]);
        if std::fs::symlink_metadata(&prefix).is_ok() {
            let resolved = std::fs::canonicalize(&prefix).ok()?;
            let resolved = resolved.to_string_lossy();
            let mut canonical = WinPath::parse(&resolved, None, Strictness::Lax)?;
            canonical
                .parts
                .extend(path.parts[existing..].iter().cloned());
            return Some(canonical);
        }
        if existing == 0 {
            return None;
        }
        existing -= 1;
    }
}

/// Elsewhere no PowerShell script is read; paths stay as written.
#[cfg(not(windows))]
fn canonical(path: WinPath) -> Option<WinPath> {
    Some(path)
}

#[cfg(test)]
#[path = "write_scope_tests.rs"]
mod tests;
