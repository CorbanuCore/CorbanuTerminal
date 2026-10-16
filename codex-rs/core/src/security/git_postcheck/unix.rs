use codex_protocol::ThreadId;
use std::collections::HashMap;
use std::collections::HashSet;
use std::collections::VecDeque;
use std::ffi::OsStr;
use std::ffi::OsString;
use std::io::Read;
use std::os::unix::fs::DirEntryExt;
use std::os::unix::fs::MetadataExt;
use std::os::unix::fs::OpenOptionsExt;
use std::path::Path;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::LazyLock;
use std::sync::Mutex;
use std::sync::PoisonError;
use std::time::SystemTime;
use std::time::UNIX_EPOCH;

/// Entries looked at per repository at most; past it the look is
/// incomplete (and holds, see `held_for`).
const MAX_ENTRIES: usize = 50_000;
const MAX_DEPTH: usize = 16;
const MAX_REPOS: usize = 8;
const MAX_REPOS_PER_THREAD: usize = 16;
const MAX_THREADS: usize = 64;
const SHOWN_FINDINGS: usize = 20;
/// Links followed per chain, like the kernel's limit.
const MAX_LINK_HOPS: usize = 40;
const MAX_TODO_BYTES: u64 = 4 << 20;
/// A `.git` file or `commondir` longer than this is not read.
const SMALL_FILE_BYTES: u64 = 64 << 10;
/// A folder or todo whose status changed this recently is read again next
/// time: a change in the same clock tick would not move its change time.
const RACY_NANOS: i128 = 2_000_000_000;
/// Stores below a git folder. Only their own entries are looked at (and
/// `objects/info`, `objects/pack`): git reads the objects below and puts new
/// ones in place by renaming or linking, it does not write through them.
const STORES: [&str; 3] = ["objects", "lfs", "annex"];
const OBJECT_FOLDERS_LOOKED_AT: [&str; 2] = ["info", "pack"];
/// What a later `git rebase --continue` (or `cherry-pick`/`revert
/// --continue`) runs, in each git folder: the steps, and the merge strategy
/// (git runs `git merge-<strategy>`, as a path where the name holds a `/`).
const STATE_FILES: [&str; 5] = [
    "rebase-merge/git-rebase-todo",
    "rebase-apply/git-rebase-todo",
    "sequencer/todo",
    "rebase-merge/strategy",
    "sequencer/opts",
];
const BUILTIN_STRATEGIES: [&str; 6] = ["ort", "recursive", "resolve", "octopus", "ours", "subtree"];
/// Entries of a repository's own git folder that git reads for config or
/// hooks: a new one is reported (on Linux a missing one isn't held
/// read-only), as is anything new below `hooks`.
const CONFIG_NAMES: [&str; 3] = ["config", "config.worktree", "commondir"];
/// Legacy remote definitions git still reads for a remote with no URL in
/// its config.
const LEGACY_REMOTES: [&str; 2] = ["remotes", "branches"];
const SHOWN_EXEC_LINES: usize = 3;
const SHOWN_CHARS: usize = 160;

/// A project's repository: the `.git` git finds for it and its git folders.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub(crate) struct Repo {
    /// As found (a folder, a `gitdir:` file or a link).
    dot_git: PathBuf,
    /// Real paths: the folder, or a `gitdir:` file's folder and its common
    /// folder.
    git_dirs: Vec<PathBuf>,
}

impl Repo {
    /// Paths that, where an agent command may write them, mean it may write
    /// this repository's git folder.
    pub(crate) fn probes(&self) -> Vec<PathBuf> {
        let mut probes = vec![self.dot_git.join("COMMIT_EDITMSG")];
        for dir in &self.git_dirs {
            for name in [
                "COMMIT_EDITMSG",
                "logs/HEAD",
                STATE_FILES[0],
                STATE_FILES[2],
            ] {
                probes.push(dir.join(name));
            }
        }
        probes
    }
}

/// The repositories `folders` are in (the nearest `.git` at or above each,
/// as git finds it).
pub(crate) fn repos(folders: &[&Path]) -> Vec<Repo> {
    let mut repos: Vec<Repo> = Vec::new();
    for folder in folders {
        let Some(dot_git) = folder
            .ancestors()
            .map(|folder| folder.join(".git"))
            .find(|dot_git| std::fs::symlink_metadata(dot_git).is_ok())
        else {
            continue;
        };
        if repos.iter().any(|repo| repo.dot_git == dot_git) {
            continue;
        }
        if let Some(repo) = repo_at(dot_git) {
            repos.push(repo);
        }
        if repos.len() >= MAX_REPOS {
            break;
        }
    }
    repos
}

fn repo_at(dot_git: PathBuf) -> Option<Repo> {
    let meta = std::fs::metadata(&dot_git).ok()?;
    let mut git_dirs = Vec::new();
    if meta.is_dir() {
        git_dirs.push(std::fs::canonicalize(&dot_git).ok()?);
    } else {
        let text = read_small(&dot_git)?;
        let work_tree = dot_git.parent()?;
        let dir = text.lines().find_map(|line| line.strip_prefix("gitdir:"))?;
        let git_dir = std::fs::canonicalize(work_tree.join(dir.trim())).ok()?;
        if let Some(common) = read_small(&git_dir.join("commondir"))
            .and_then(|common| std::fs::canonicalize(git_dir.join(common.trim())).ok())
        {
            git_dirs.push(common);
        }
        git_dirs.push(git_dir);
    }
    git_dirs.dedup();
    Some(Repo { dot_git, git_dirs })
}

/// Reads a small regular file without blocking on a FIFO or device.
fn read_small(path: &Path) -> Option<String> {
    let mut bytes = Vec::new();
    let complete = read_regular(path, SMALL_FILE_BYTES, &mut bytes)?;
    complete.then(|| String::from_utf8_lossy(&bytes).into_owned())
}

/// Reads up to `max` bytes of the regular file at `path` (followed) into
/// `bytes`; whether that is all of it. `None` if it is not a readable
/// regular file.
fn read_regular(path: &Path, max: u64, bytes: &mut Vec<u8>) -> Option<bool> {
    let file = std::fs::OpenOptions::new()
        .read(true)
        .custom_flags(libc::O_NONBLOCK | libc::O_NOCTTY)
        .open(path)
        .ok()?;
    let meta = file.metadata().ok()?;
    if !meta.is_file() {
        return None;
    }
    file.take(max).read_to_end(bytes).ok()?;
    Some(meta.len() <= max)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Kind {
    Dir,
    File,
    Link,
    Other,
}

impl Kind {
    fn of(file_type: std::fs::FileType) -> Self {
        if file_type.is_symlink() {
            Kind::Link
        } else if file_type.is_dir() {
            Kind::Dir
        } else if file_type.is_file() {
            Kind::File
        } else {
            Kind::Other
        }
    }
}

/// Modification and status-change times. A command can set the first back,
/// not the second.
fn times(meta: &std::fs::Metadata) -> (i128, i128) {
    (
        i128::from(meta.mtime()) * 1_000_000_000 + i128::from(meta.mtime_nsec()),
        i128::from(meta.ctime()) * 1_000_000_000 + i128::from(meta.ctime_nsec()),
    )
}

fn now_nanos() -> i128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(/*default*/ 0, |since| since.as_nanos() as i128)
}

/// Whether a status changed at `ctime` is old enough to trust.
fn settled(ctime: i128) -> bool {
    now_nanos() - ctime > RACY_NANOS
}

/// Where a link inside a git folder leads, followed hop by hop.
#[derive(Clone, Debug, PartialEq, Eq)]
struct Link {
    raw: PathBuf,
    resolved: Option<PathBuf>,
    /// Ends anywhere but a ref (as `core.preferSymlinkRefs` makes): outside
    /// the repository's git folders, at its own config or hooks, at a
    /// folder, at a hard-linked file, or somewhere unknown. Git would write
    /// there through it.
    escapes: bool,
}

impl Link {
    fn read(path: &Path, git_dirs: &[PathBuf]) -> Self {
        let raw = std::fs::read_link(path).unwrap_or_default();
        let (resolved, escapes) = match follow(path) {
            Some((end, meta)) => {
                let not_ref = !git_dirs.iter().any(|dir| end.starts_with(dir.join("refs")));
                let odd = meta.is_some_and(|meta| meta.is_dir() || meta.nlink() > 1);
                (Some(end), not_ref || odd)
            }
            None => (None, true),
        };
        Link {
            raw,
            resolved,
            escapes,
        }
    }
}

/// The path a chain of links starting at `path` ends at (folders on the way
/// resolved), and its status if it exists. A missing end is where git would
/// create the file. `None` if the chain can't be followed.
fn follow(path: &Path) -> Option<(PathBuf, Option<std::fs::Metadata>)> {
    let mut current = path.to_path_buf();
    for _ in 0..MAX_LINK_HOPS {
        let raw = std::fs::read_link(&current).ok()?;
        let full = match current.parent() {
            Some(parent) if raw.is_relative() => parent.join(&raw),
            _ => raw,
        };
        let name = full.file_name()?.to_os_string();
        if name == OsStr::new("..") || name == OsStr::new(".") {
            return None;
        }
        current = std::fs::canonicalize(full.parent()?).ok()?.join(name);
        match std::fs::symlink_metadata(&current) {
            Ok(meta) if meta.file_type().is_symlink() => continue,
            Ok(meta) => return Some((current, Some(meta))),
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => return Some((current, None)),
            Err(_) => return None,
        }
    }
    None
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct Entry {
    kind: Kind,
    dev: u64,
    ino: u64,
    nlink: u64,
    /// Length and times, kept current for legacy remote files.
    stamp: (u64, i128, i128),
    link: Option<Link>,
}

impl Entry {
    fn stat(path: &Path, kind: Kind, ino: u64) -> Self {
        let meta = std::fs::symlink_metadata(path).ok();
        Entry {
            kind,
            dev: meta.as_ref().map_or(/*default*/ 0, MetadataExt::dev),
            ino: meta.as_ref().map_or(ino, MetadataExt::ino),
            nlink: meta.as_ref().map_or(/*default*/ 1, MetadataExt::nlink),
            stamp: meta.as_ref().map_or(/*default*/ (0, 0, 0), |meta| {
                let (mtime, ctime) = times(meta);
                (meta.len(), mtime, ctime)
            }),
            link: None,
        }
    }
}

/// What a folder is to git, from where it sits (never from what it holds:
/// a command could make any folder look like a git folder).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Role {
    /// A git folder: a repository's, a linked worktree's or a submodule's.
    GitDir,
    /// `objects`, `lfs` or `annex` of a git folder.
    Store {
        objects: bool,
    },
    /// `objects/info` and `objects/pack` (git links packs into place).
    StorePart,
    /// `remotes` or `branches` of a git folder, and folders below them.
    Legacy,
    /// `hooks` of a git folder, and folders below it.
    Hooks,
    /// `worktrees` of a git folder: its folders are git folders.
    Worktrees,
    /// Below `modules` of a git folder: a git folder where it has `HEAD` and
    /// `objects` or `commondir` (submodule names may hold slashes). A fake
    /// one only hides its own objects, which git never writes through.
    Modules,
    Plain,
}

#[derive(Clone, Debug)]
struct Folder {
    dev: u64,
    ino: u64,
    ctime: i128,
    /// Listed long enough after its last change to trust its change time.
    settled: bool,
    role: Role,
    entries: HashMap<OsString, Entry>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct StateRead {
    dev: u64,
    ino: u64,
    len: u64,
    times: (i128, i128),
    /// What in it runs commands.
    lines: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
enum DotGit {
    Missing,
    Folder {
        dev: u64,
        ino: u64,
    },
    File {
        dev: u64,
        ino: u64,
        text: Option<String>,
    },
    Link {
        target: Option<PathBuf>,
    },
    Other,
}

impl DotGit {
    fn read(path: &Path) -> Self {
        let Ok(meta) = std::fs::symlink_metadata(path) else {
            return DotGit::Missing;
        };
        match Kind::of(meta.file_type()) {
            Kind::Dir => DotGit::Folder {
                dev: meta.dev(),
                ino: meta.ino(),
            },
            Kind::File => DotGit::File {
                dev: meta.dev(),
                ino: meta.ino(),
                text: read_small(path),
            },
            Kind::Link => DotGit::Link {
                target: std::fs::read_link(path).ok(),
            },
            Kind::Other => DotGit::Other,
        }
    }
}

#[derive(Clone, Debug)]
struct Snapshot {
    folders: HashMap<PathBuf, Folder>,
    states: HashMap<PathBuf, StateRead>,
    dot_git: DotGit,
    /// Why the look is incomplete, if it is.
    incomplete: Option<String>,
    /// `.git` was missing at a later look (a worktree removed, a clone
    /// deleted).
    went_missing: bool,
}

impl Snapshot {
    /// What a first look is compared with: nothing (the `.git` itself
    /// aside).
    fn empty(dot_git: &DotGit) -> Self {
        Snapshot {
            folders: HashMap::new(),
            states: HashMap::new(),
            dot_git: dot_git.clone(),
            incomplete: None,
            went_missing: false,
        }
    }
}

fn walk(repo: &Repo, prev: Option<&Snapshot>) -> Snapshot {
    let mut snapshot = Snapshot::empty(&DotGit::read(&repo.dot_git));
    let home = dirs::home_dir().and_then(|home| std::fs::canonicalize(home).ok());
    if repo
        .git_dirs
        .iter()
        .any(|dir| dir.parent().is_none() || Some(dir) == home.as_ref())
    {
        snapshot.incomplete =
            Some("its git folder is your home folder or the file system root".to_string());
        return snapshot;
    }
    let mut roots = repo.git_dirs.clone();
    let mut count = 0;
    // Breadth first: a git folder's own entries are looked at before a cap
    // can stop the look.
    let mut queue: VecDeque<(PathBuf, usize, Role)> = repo
        .git_dirs
        .iter()
        .map(|dir| (dir.clone(), 0, Role::GitDir))
        .collect();
    while let Some((dir, depth, role)) = queue.pop_front() {
        if snapshot.folders.contains_key(&dir) {
            continue;
        }
        let Ok(meta) = std::fs::symlink_metadata(&dir) else {
            continue;
        };
        if !meta.is_dir() {
            continue;
        }
        let old = prev.and_then(|prev| prev.folders.get(&dir));
        let (_, ctime) = times(&meta);
        let mut folder = match old.filter(|old| {
            old.settled && old.dev == meta.dev() && old.ino == meta.ino() && old.ctime == ctime
        }) {
            Some(old) => old.clone(),
            None => match list(&dir, &meta, old) {
                Some(folder) => folder,
                // Gone meanwhile (a rebase ended): nothing there to follow.
                None if std::fs::symlink_metadata(&dir)
                    .is_err_and(|err| err.kind() == std::io::ErrorKind::NotFound) =>
                {
                    continue;
                }
                None => {
                    snapshot.incomplete = Some(format!("{} can't be read", shown_path(&dir).0));
                    continue;
                }
            },
        };
        let has = |name: &str| folder.entries.contains_key(OsStr::new(name));
        let role = match role {
            Role::Modules if has("HEAD") && (has("objects") || has("commondir")) => Role::GitDir,
            role => role,
        };
        folder.role = role;
        for (name, entry) in &mut folder.entries {
            let path = dir.join(name);
            if entry.kind == Kind::Link {
                entry.link = Some(Link::read(&path, &repo.git_dirs));
            }
            if role == Role::Legacy && entry.kind != Kind::Dir {
                *entry = Entry {
                    link: entry.link.take(),
                    ..Entry::stat(&path, entry.kind, entry.ino)
                };
            }
        }
        count += folder.entries.len();
        if count > MAX_ENTRIES {
            snapshot.incomplete = Some(
                "it has too many entries (git pack-refs --all or git gc may help)".to_string(),
            );
            break;
        }
        if role == Role::GitDir && !roots.contains(&dir) {
            roots.push(dir.clone());
        }
        for (name, entry) in &folder.entries {
            if entry.kind != Kind::Dir {
                continue;
            }
            let is = |names: &[&str]| names.iter().any(|kept| name.as_os_str() == *kept);
            let child = match role {
                Role::GitDir if is(&STORES) => Role::Store {
                    objects: name.as_os_str() == "objects",
                },
                Role::GitDir if is(&LEGACY_REMOTES) => Role::Legacy,
                Role::GitDir if name.as_os_str() == "worktrees" => Role::Worktrees,
                Role::GitDir if name.as_os_str() == "modules" => Role::Modules,
                Role::GitDir if name.as_os_str() == "hooks" => Role::Hooks,
                Role::Hooks => Role::Hooks,
                Role::Worktrees => Role::GitDir,
                Role::Modules => Role::Modules,
                Role::Store { objects: true } if is(&OBJECT_FOLDERS_LOOKED_AT) => Role::StorePart,
                Role::Store { .. } => continue,
                // `remotes/a/b` defines the remote `a/b`.
                Role::Legacy => Role::Legacy,
                Role::GitDir | Role::StorePart | Role::Plain => Role::Plain,
            };
            if depth >= MAX_DEPTH {
                snapshot.incomplete = Some("its folders are nested too deep".to_string());
                continue;
            }
            queue.push_back((dir.join(name), depth + 1, child));
        }
        snapshot.folders.insert(dir, folder);
    }
    // Where the look stopped short, what the last look saw stays the
    // baseline (listed again next time).
    if snapshot.incomplete.is_some()
        && let Some(prev) = prev
    {
        for (dir, folder) in &prev.folders {
            if !snapshot.folders.contains_key(dir) {
                let mut folder = folder.clone();
                folder.settled = false;
                snapshot.folders.insert(dir.clone(), folder);
            }
        }
    }
    for root in roots {
        for name in STATE_FILES {
            let path = root.join(name);
            let Ok(meta) = std::fs::metadata(&path) else {
                continue;
            };
            let now = times(&meta);
            let old = prev
                .and_then(|prev| prev.states.get(&path))
                .filter(|old| {
                    old.dev == meta.dev()
                        && old.ino == meta.ino()
                        && old.len == meta.len()
                        && old.times == now
                        && settled(now.1)
                })
                .cloned();
            let read = old.unwrap_or_else(|| StateRead {
                dev: meta.dev(),
                ino: meta.ino(),
                len: meta.len(),
                times: now,
                lines: state_lines(&path),
            });
            snapshot.states.insert(path, read);
        }
    }
    snapshot
}

fn list(dir: &Path, meta: &std::fs::Metadata, old: Option<&Folder>) -> Option<Folder> {
    let mut entries = HashMap::new();
    // An unreadable folder fails the look (it can hide what git follows).
    for item in std::fs::read_dir(dir).ok()? {
        let item = item.ok()?;
        // Gone meanwhile (git renamed it into place): skipped.
        let file_type = match item.file_type() {
            Ok(file_type) => file_type,
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => continue,
            Err(_) => return None,
        };
        let (name, kind, ino) = (item.file_name(), Kind::of(file_type), item.ino());
        let entry = old
            .and_then(|old| old.entries.get(&name))
            .filter(|old| old.kind == kind && old.ino == ino)
            .cloned()
            .unwrap_or_else(|| Entry::stat(&item.path(), kind, ino));
        entries.insert(name, entry);
    }
    let after = std::fs::symlink_metadata(dir).ok()?;
    let (_, ctime) = times(meta);
    Some(Folder {
        dev: meta.dev(),
        ino: meta.ino(),
        ctime,
        settled: times(&after).1 == ctime && settled(ctime),
        role: Role::Plain,
        entries,
    })
}

/// What in a rebase or sequencer state file runs commands: a todo's `exec`
/// lines, a merge strategy that isn't built in. Fails closed: a file that
/// can't be read or is too long counts as holding one.
fn state_lines(path: &Path) -> Vec<String> {
    let mut bytes = Vec::new();
    let Some(complete) = read_regular(path, MAX_TODO_BYTES, &mut bytes) else {
        return vec!["(the file can't be read)".to_string()];
    };
    let text = String::from_utf8_lossy(&bytes);
    let mut lines = match path.file_name().and_then(OsStr::to_str) {
        Some("strategy") => strategy_lines([text.trim()]),
        // Git config syntax: `strategy = <name>` in `[options]`.
        Some("opts") => strategy_lines(text.lines().filter_map(|line| {
            // Git takes `[options] strategy = x` on one line, any case.
            let line = match line.trim_start().strip_prefix('[') {
                Some(rest) => rest.split_once(']')?.1,
                None => line,
            };
            let (key, value) = line.split_once('=')?;
            key.trim()
                .eq_ignore_ascii_case("strategy")
                .then(|| value.trim().trim_matches('"'))
        })),
        _ => exec_lines(&text),
    };
    if !complete && lines.is_empty() {
        lines.push("(the file is too long to check)".to_string());
    }
    lines
}

fn strategy_lines<'a>(names: impl IntoIterator<Item = &'a str>) -> Vec<String> {
    names
        .into_iter()
        .filter(|name| !name.is_empty() && !BUILTIN_STRATEGIES.contains(name))
        .map(|name| format!("merge strategy {name}"))
        .collect()
}

/// Git's sequencer takes a command name followed by a space, tab, carriage
/// return or the line's end; `exec` runs the rest through the shell.
pub(super) fn exec_lines(text: &str) -> Vec<String> {
    text.split('\n')
        .map(|line| line.trim_start_matches([' ', '\t']))
        .filter(|line| {
            let command = line.split([' ', '\t', '\r']).next().unwrap_or_default();
            (command == "exec" || command == "x")
                && !line[command.len()..]
                    .trim_matches([' ', '\t', '\r'])
                    .is_empty()
        })
        .map(|line| line.trim_end().to_string())
        .collect()
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) enum Finding {
    Link {
        path: PathBuf,
        target: Option<PathBuf>,
    },
    HardLink {
        path: PathBuf,
    },
    /// A rebase or sequencer state file that runs commands.
    Exec {
        todo: PathBuf,
        lines: Vec<String>,
    },
    Remote {
        path: PathBuf,
    },
    /// New config or hooks of a repository's own git folder.
    Config {
        path: PathBuf,
    },
    Replaced {
        dot_git: PathBuf,
    },
    Incomplete {
        dot_git: PathBuf,
        why: String,
    },
}

/// What has changed from `prev` to `cur`. A first look (`prev` empty)
/// reports what git would follow out of the sandbox (links, commands in
/// state files) and hard links: an ordinary repository has legacy remotes,
/// config and hooks of its own, and hard-links only its objects.
fn compare(repo: &Repo, prev: &Snapshot, cur: &Snapshot, first: bool) -> Vec<Finding> {
    let mut findings = Vec::new();
    if prev.dot_git != cur.dot_git {
        findings.push(Finding::Replaced {
            dot_git: repo.dot_git.clone(),
        });
    }
    let mut folders: Vec<_> = cur.folders.iter().collect();
    folders.sort_by(|a, b| a.0.cmp(b.0));
    for (dir, folder) in folders {
        // A folder the last look didn't have counts as empty then (also
        // where that look couldn't read it).
        let prev_folder = prev.folders.get(dir);
        let own = repo.git_dirs.contains(dir);
        let mut names: Vec<_> = folder.entries.iter().collect();
        names.sort_by(|a, b| a.0.cmp(b.0));
        for (name, entry) in names {
            let path = dir.join(name);
            let old = prev_folder.and_then(|folder| folder.entries.get(name));
            let new = old.is_none_or(|old| {
                old.kind != entry.kind || old.ino != entry.ino || old.dev != entry.dev
            });
            // Git runs hooks, it doesn't write through them; links in an
            // existing hooks folder are the person's (the folder is
            // protected), a new one is reported below.
            let hooks = folder.role == Role::Hooks
                || (folder.role == Role::GitDir && name.as_os_str() == "hooks");
            if let Some(link) = entry.link.as_ref().filter(|link| link.escapes)
                && !hooks
                && (new || old.and_then(|old| old.link.as_ref()) != Some(link))
            {
                findings.push(Finding::Link {
                    path: path.clone(),
                    target: link.resolved.clone().or_else(|| Some(link.raw.clone())),
                });
            }
            // Git links packs into place, then removes the temporary name.
            if new
                && entry.kind == Kind::File
                && entry.nlink > 1
                && !matches!(folder.role, Role::Store { .. } | Role::StorePart)
            {
                findings.push(Finding::HardLink { path: path.clone() });
            }
            if first {
                continue;
            }
            if folder.role == Role::Legacy
                && entry.kind != Kind::Dir
                && (new || old.is_some_and(|old| old.stamp != entry.stamp))
            {
                findings.push(Finding::Remote { path: path.clone() });
            }
            // What the protected-file rules can't hold: config or hooks of
            // the repository's own git folder that didn't exist (on Linux a
            // missing one isn't held read-only). Git rewrites existing ones
            // by renaming; that isn't reported.
            if own
                && prev_folder.is_some()
                && old.is_none()
                && (name.as_os_str() == "hooks"
                    || CONFIG_NAMES.iter().any(|kept| name.as_os_str() == *kept))
            {
                findings.push(Finding::Config { path });
            }
        }
    }
    let mut states: Vec<_> = cur.states.iter().collect();
    states.sort_by(|a, b| a.0.cmp(b.0));
    for (path, read) in states {
        // Reported when something runs that didn't before (a rebase with
        // `-x` drops a line at every step).
        let before = prev.states.get(path).map(|old| old.lines.as_slice());
        if read
            .lines
            .iter()
            .any(|line| !before.unwrap_or_default().contains(line))
        {
            findings.push(Finding::Exec {
                todo: path.clone(),
                lines: read.lines.clone(),
            });
        }
    }
    findings
}

/// A finding that stands until the person removes it: while any does, a
/// person's approval doesn't lift the protected-file rules for commands in
/// that repository, in any session of this process (a sub-agent's findings
/// count for its parent too).
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
struct Held {
    /// The repository's git folders (real paths).
    git_dirs: Vec<PathBuf>,
    what: HeldWhat,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
enum HeldWhat {
    Link { path: PathBuf },
    HardLink { path: PathBuf },
    Exec { todo: PathBuf },
    Exists { path: PathBuf },
    Replaced { dot_git: PathBuf, now: DotGit },
}

impl Held {
    fn of(finding: &Finding, repo: &Repo, snapshot: &Snapshot) -> Option<Self> {
        let what = match finding {
            Finding::Link { path, .. } => HeldWhat::Link { path: path.clone() },
            Finding::HardLink { path } => HeldWhat::HardLink { path: path.clone() },
            Finding::Exec { todo, .. } => HeldWhat::Exec { todo: todo.clone() },
            Finding::Remote { path } | Finding::Config { path } => {
                HeldWhat::Exists { path: path.clone() }
            }
            Finding::Replaced { dot_git } => HeldWhat::Replaced {
                dot_git: dot_git.clone(),
                now: snapshot.dot_git.clone(),
            },
            Finding::Incomplete { .. } => return None,
        };
        Some(Held {
            git_dirs: repo.git_dirs.clone(),
            what,
        })
    }

    fn path(&self) -> &Path {
        match &self.what {
            HeldWhat::Link { path } | HeldWhat::HardLink { path } | HeldWhat::Exists { path } => {
                path
            }
            HeldWhat::Exec { todo } => todo,
            HeldWhat::Replaced { dot_git, .. } => dot_git,
        }
    }

    /// Whether what was found is still there (looked at again, not by
    /// identity: a freed inode may be reused).
    fn stands(&self) -> bool {
        match &self.what {
            HeldWhat::Link { path } => {
                std::fs::symlink_metadata(path).is_ok_and(|meta| meta.file_type().is_symlink())
                    && Link::read(path, &self.git_dirs).escapes
            }
            HeldWhat::HardLink { path } => std::fs::symlink_metadata(path)
                .is_ok_and(|meta| meta.file_type().is_file() && meta.nlink() > 1),
            HeldWhat::Exec { todo } => {
                std::fs::metadata(todo).is_ok() && !state_lines(todo).is_empty()
            }
            HeldWhat::Exists { path } => std::fs::symlink_metadata(path).is_ok(),
            HeldWhat::Replaced { dot_git, now } => DotGit::read(dot_git) == *now,
        }
    }
}

/// A repository's last look, locked while one is taken. Lock order: a
/// repository's lock may be taken without `GLOBAL`, and `GLOBAL` while
/// holding it is never needed (it is released first).
type Slot = Arc<Mutex<Option<Snapshot>>>;

#[derive(Default)]
struct ThreadState {
    repos: HashMap<PathBuf, (Repo, Slot)>,
    /// When each repository was last used, for forgetting the oldest.
    used: HashMap<PathBuf, u64>,
    last_used: u64,
}

#[derive(Default)]
struct Global {
    threads: HashMap<ThreadId, ThreadState>,
    held: HashSet<Held>,
    /// Repositories (by their git folders) whose latest look was
    /// incomplete, and why.
    incomplete: HashMap<Vec<PathBuf>, String>,
    uses: u64,
}

static GLOBAL: LazyLock<Mutex<Global>> = LazyLock::new(Default::default);

fn with_global<T>(f: impl FnOnce(&mut Global) -> T) -> T {
    f(&mut GLOBAL.lock().unwrap_or_else(PoisonError::into_inner))
}

/// `repo`'s slot in this session, counted as used now where `used` (a
/// command runs there). A forgotten repository (past the limits) gets a
/// first look again, which reports links, steps and hard links there.
fn slot_of(thread: ThreadId, repo: &Repo, used: bool) -> Slot {
    with_global(|global| {
        global.uses += 1;
        let uses = global.uses;
        if !global.threads.contains_key(&thread)
            && global.threads.len() >= MAX_THREADS
            && let Some(oldest) = global
                .threads
                .iter()
                .min_by_key(|(_, state)| state.last_used)
                .map(|(thread, _)| *thread)
        {
            global.threads.remove(&oldest);
        }
        let state = global.threads.entry(thread).or_default();
        state.last_used = uses;
        if !state.repos.contains_key(&repo.dot_git)
            && state.repos.len() >= MAX_REPOS_PER_THREAD
            && let Some(oldest) = state
                .used
                .iter()
                .min_by_key(|(_, used)| **used)
                .map(|(path, _)| path.clone())
        {
            state.repos.remove(&oldest);
            state.used.remove(&oldest);
        }
        if used || !state.used.contains_key(&repo.dot_git) {
            state.used.insert(repo.dot_git.clone(), uses);
        }
        state
            .repos
            .entry(repo.dot_git.clone())
            .or_insert_with(|| (repo.clone(), Slot::default()))
            .1
            .clone()
    })
}

/// Before a command that may write these repositories' git folders: the
/// first look, where this session has none yet, with what it found there
/// already for the person.
pub(crate) fn begin(thread: ThreadId, repos: &[Repo], level: &str) -> Option<String> {
    report(
        &look(thread, repos, /*first_only*/ true, /*used*/ true),
        level,
    )
}

/// After such a command: what is new since the last look, as a message for
/// the person (`None` if nothing).
pub(crate) fn finish(thread: ThreadId, repos: &[Repo], level: &str) -> Option<String> {
    report(
        &look(thread, repos, /*first_only*/ false, /*used*/ true),
        level,
    )
}

/// Before a command is decided, and at the end of a turn: the repositories
/// this session looked at before, looked at again. Returns the message for
/// the person and what holds the rules for a command in `folders` (found in
/// its repository by any session, a sub-agent's too).
pub(crate) fn recheck(
    thread: ThreadId,
    level: &str,
    folders: &[&Path],
) -> (Option<String>, Vec<String>) {
    let repos: Vec<Repo> = with_global(|global| {
        global.threads.get(&thread).map_or_else(Vec::new, |state| {
            state.repos.values().map(|(repo, _)| repo.clone()).collect()
        })
    });
    let message = report(
        &look(
            thread, &repos, /*first_only*/ false, /*used*/ false,
        ),
        level,
    );
    // Held for a command in the project it runs in.
    let watched: Vec<PathBuf> = self::repos(folders)
        .into_iter()
        .flat_map(|repo| repo.git_dirs)
        .collect();
    (message, held_for(&watched))
}

#[cfg(test)]
pub(super) fn check(thread: ThreadId, repos: &[Repo]) -> Vec<Finding> {
    look(thread, repos, /*first_only*/ false, /*used*/ true)
}

/// Looks at `repos` (only those without a look yet, where `first_only`).
fn look(thread: ThreadId, repos: &[Repo], first_only: bool, used: bool) -> Vec<Finding> {
    let mut findings = Vec::new();
    let mut held = Vec::new();
    let mut incomplete = Vec::new();
    for repo in repos {
        let slot = slot_of(thread, repo, used);
        let mut snapshot = slot.lock().unwrap_or_else(PoisonError::into_inner);
        if first_only && snapshot.is_some() {
            continue;
        }
        // Gone (a worktree removed, a scratch clone deleted): nothing to
        // follow. Should the same one come back, it is compared with the
        // last look; a new one there gets a first look.
        if DotGit::read(&repo.dot_git) == DotGit::Missing {
            if let Some(prev) = snapshot.as_mut() {
                prev.went_missing = true;
            }
            incomplete.push((repo.git_dirs.clone(), None));
            continue;
        }
        // The last look stays until this one is done (a panic keeps it).
        let prev = snapshot.as_ref();
        let cur = walk(repo, prev);
        let prev = prev.filter(|prev| !(prev.went_missing && prev.dot_git != cur.dot_git));
        let mut found = match prev {
            Some(prev) => compare(repo, prev, &cur, /*first*/ false),
            None => compare(
                repo,
                &Snapshot::empty(&cur.dot_git),
                &cur,
                /*first*/ true,
            ),
        };
        // What went again meanwhile (a pack git linked into place) isn't
        // reported.
        found.retain(|finding| Held::of(finding, repo, &cur).is_none_or(|held| held.stands()));
        held.extend(
            found
                .iter()
                .filter_map(|finding| Held::of(finding, repo, &cur)),
        );
        // Told each time a look becomes incomplete; held while it is.
        if let Some(why) = &cur.incomplete
            && prev.is_none_or(|prev| prev.incomplete.as_ref() != Some(why))
        {
            found.push(Finding::Incomplete {
                dot_git: repo.dot_git.clone(),
                why: why.clone(),
            });
        }
        incomplete.push((repo.git_dirs.clone(), cur.incomplete.clone()));
        findings.extend(found);
        *snapshot = Some(cur);
    }
    with_global(|global| {
        for (git_dirs, why) in incomplete {
            match why {
                Some(why) => {
                    global.incomplete.insert(git_dirs, why);
                }
                None => {
                    global.incomplete.remove(&git_dirs);
                }
            }
        }
        global.held.extend(held);
    });
    findings
}

/// What any session (a sub-agent's too) found in the git folders `watched`
/// that is still there, and their incomplete looks: while there is any, a
/// person's approval doesn't lift the protected-file rules.
pub(super) fn held_for(watched: &[PathBuf]) -> Vec<String> {
    let shares = |git_dirs: &Vec<PathBuf>| git_dirs.iter().any(|dir| watched.contains(dir));
    let (held, incomplete) = with_global(|global| {
        (
            global
                .held
                .iter()
                .filter(|held| shares(&held.git_dirs))
                .cloned()
                .collect::<Vec<_>>(),
            global
                .incomplete
                .keys()
                .filter(|git_dirs| shares(git_dirs))
                .cloned()
                .collect::<Vec<_>>(),
        )
    });
    let (standing, gone): (Vec<Held>, Vec<Held>) = held.into_iter().partition(Held::stands);
    with_global(|global| {
        for held in &gone {
            global.held.remove(held);
        }
    });
    let mut out: Vec<String> = standing
        .iter()
        .map(|held| shown_path(held.path()).0)
        .collect();
    out.sort();
    out.extend(incomplete.iter().filter_map(|git_dirs| {
        git_dirs
            .first()
            .map(|dir| format!("{} (not checked fully)", shown_path(dir).0))
    }));
    out
}

/// Characters that could disguise shown text: controls and bidirectional
/// overrides.
fn disguising(c: char) -> bool {
    c.is_control()
        || matches!(
            c,
            '\u{200e}' | '\u{200f}' | '\u{061c}' | '\u{202a}'..='\u{202e}' | '\u{2066}'..='\u{2069}'
        )
}

/// Shown text, and whether it is exactly the original.
fn shown(text: &str) -> (String, bool) {
    let mut exact = true;
    let mut out: String = text
        .chars()
        .map(|c| {
            if disguising(c) {
                exact = false;
                '?'
            } else {
                c
            }
        })
        .take(SHOWN_CHARS)
        .collect();
    if text.chars().count() > SHOWN_CHARS {
        exact = false;
        out.push('…');
    }
    (out, exact)
}

fn shown_path(path: &Path) -> (String, bool) {
    match path.to_str() {
        Some(text) => shown(text),
        None => (shown(&path.to_string_lossy()).0, false),
    }
}

/// How to remove `path` (`rm` plus `flags`), or what to do where its name
/// can't be shown exactly.
fn removal(flags: &str, path: &Path) -> String {
    match shown_path(path) {
        (text, true) => format!("rm {flags}'{}'", text.replace('\'', r"'\''")),
        (_, false) => "remove it by hand (its name has characters that aren't shown)".to_string(),
    }
}

pub(super) fn report(findings: &[Finding], level: &str) -> Option<String> {
    if findings.is_empty() {
        return None;
    }
    let mut lines = vec![format!(
        "Security level {level}: agent commands could write this project's git folder, and \
         Corbanu found things there that your own git would follow outside the sandbox. \
         Nothing was removed. Check them, and remove what you didn't make:"
    )];
    for finding in findings.iter().take(SHOWN_FINDINGS) {
        lines.push(match finding {
            Finding::Link { path, target } => format!(
                "- {} is a link to {}: git writes through it (for example on your next commit, \
                 fetch or pull). Remove the link (its target stays): {}",
                shown_path(path).0,
                target.as_deref().map_or_else(
                    || "a place that can't be resolved".to_string(),
                    |target| shown_path(target).0
                ),
                removal("", path)
            ),
            Finding::HardLink { path } => format!(
                "- {} is a hard link: the same file has another name, maybe outside the project, \
                 and git writes into it. Remove this name: {}",
                shown_path(path).0,
                removal("", path)
            ),
            Finding::Exec { todo, lines } => {
                let state = todo.parent().unwrap_or(todo);
                let continues = if state.ends_with("sequencer") {
                    "git cherry-pick --continue or git revert --continue"
                } else {
                    "git rebase --continue"
                };
                let mut shown_lines: Vec<String> = lines
                    .iter()
                    .take(SHOWN_EXEC_LINES)
                    .map(|line| format!("\"{}\"", shown(line).0))
                    .collect();
                if lines.len() > SHOWN_EXEC_LINES {
                    shown_lines.push(format!("and {} more", lines.len() - SHOWN_EXEC_LINES));
                }
                format!(
                    "- {} holds steps that run commands outside the sandbox if you continue \
                     ({continues}): {}. If you didn't start this yourself, remove it without \
                     continuing: {}",
                    shown_path(state).0,
                    shown_lines.join(", "),
                    removal("-r ", state)
                )
            }
            Finding::Remote { path } => format!(
                "- {} defines a git remote, which git fetch, pull and push use for a remote with \
                 no URL in your git config. Remove it: {}",
                shown_path(path).0,
                removal("", path)
            ),
            Finding::Config { path } => format!(
                "- {} is new: git reads it for config or hooks, which can run commands. If you \
                 didn't add it, remove it: {}",
                shown_path(path).0,
                removal(
                    if std::fs::symlink_metadata(path).is_ok_and(|meta| meta.is_dir()) {
                        "-r "
                    } else {
                        ""
                    },
                    path
                )
            ),
            Finding::Replaced { dot_git } => format!(
                "- {} was replaced. If you didn't do this, check its hooks folder and config \
                 before you run git in this project.",
                shown_path(dot_git).0
            ),
            Finding::Incomplete { dot_git, why } => format!(
                "- {} couldn't be checked fully: {}. Look for links in it before you run git in \
                 this project.",
                shown_path(dot_git).0,
                shown(why).0
            ),
        });
    }
    if findings.len() > SHOWN_FINDINGS {
        lines.push(format!("- and {} more.", findings.len() - SHOWN_FINDINGS));
    }
    lines.push(format!(
        "Until these are removed (a replaced git folder: until it changes again or Corbanu \
         restarts), approving a command in this project doesn't lift {level}'s protected-file \
         rules."
    ));
    Some(lines.join("\n"))
}
