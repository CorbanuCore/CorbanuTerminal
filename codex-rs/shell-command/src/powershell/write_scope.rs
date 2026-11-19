//! D729 (A), #2661: where a PowerShell script read in full may write.
//!
//! An allowlisted cmdlet that writes (`Set-Content`, `New-Item`, `Copy-Item`, …) keeps a
//! script readable only when every path it writes, resolved as Windows resolves it, lies
//! inside a writable root (outside its read-only parts) and neither inside nor above a
//! folder PowerShell or Windows runs code from (module and profile folders, `PATH`). Paths
//! are compared normalised and case-insensitively, after resolving the part that exists
//! (links, junctions, 8.3 names). Outside a scope (see [`enter_powershell_write_scope`])
//! no write keeps a script readable.

use std::cell::RefCell;
use std::marker::PhantomData;
use std::rc::Rc;

use codex_protocol::protocol::WritableRoot;
use codex_utils_absolute_path::AbsolutePathBuf;

thread_local! {
    static SCOPE: RefCell<Option<Rc<PowershellWriteScope>>> = const { RefCell::new(None) };
}

/// The directory a command runs in and the roots its writes may stay in.
#[derive(Debug, Clone)]
pub struct PowershellWriteScope {
    cwd: String,
    roots: Vec<WritableRoot>,
}

impl PowershellWriteScope {
    pub fn new(cwd: &AbsolutePathBuf, roots: Vec<WritableRoot>) -> Self {
        Self {
            cwd: cwd.to_string_lossy().into_owned(),
            roots,
        }
    }
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

/// Whether every path in `writes` stays where the current scope lets a script write.
pub(crate) fn writes_allowed(writes: &[String], protected: &[String]) -> bool {
    let Some(scope) = SCOPE.with(|current| current.borrow().clone()) else {
        return false;
    };
    let Some(cwd) = WinPath::parse(&scope.cwd, None, Strictness::Lax).and_then(canonical) else {
        return false;
    };
    let roots: Vec<Root> = scope.roots.iter().filter_map(Root::from_writable).collect();
    let protected: Vec<WinPath> = protected
        .iter()
        .filter_map(|dir| WinPath::parse(dir.trim().trim_matches('"'), Some(&cwd), Strictness::Lax))
        .filter_map(canonical)
        .collect();
    allows(writes, &cwd, &roots, &protected)
}

fn allows(writes: &[String], cwd: &WinPath, roots: &[Root], protected: &[WinPath]) -> bool {
    writes.iter().all(|write| {
        let Some(target) = WinPath::parse(write, Some(cwd), Strictness::Write).and_then(canonical)
        else {
            return false;
        };
        roots.iter().any(|root| root.allows(&target))
            && !protected
                .iter()
                .any(|dir| target.is_within(dir) || dir.is_within(&target))
    })
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
