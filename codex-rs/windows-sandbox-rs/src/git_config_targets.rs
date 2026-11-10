//! D422 (#1775): files and folders a repository's config names that git
//! runs programs from, outside the repository's git folder.
//!
//! Many projects set `core.hooksPath` to a folder in the work tree (Husky's
//! `.husky` or `.husky\_`, a team `.githooks`). The repository's config may
//! be protected, but the folder it names is ordinary, writable project
//! content: a command writes a hook there and the next `git commit` runs it.
//! Config values that name a file in the work tree are the same case: an
//! `include.path` (more config) or a program key set to a script in the
//! project. [`targets`] lists them from every config git would read for a
//! repository (its own `config` and `config.worktree`, the global and system
//! configs, and the files their `include`/`includeIf` lines name); the
//! session records their contents and flags what changed or appeared, as for
//! the repositories themselves (D420). A global or system config (or a file
//! it includes) that lies in a writable root is a target itself. For Husky,
//! whose `_` stubs only run `<folder>\<hook>`, the hook-named files of the
//! folder above a `_` hooks folder count too, and so does the `_` folder
//! below any hooks folder (Husky v5 to v8 hooks source `_\husky.sh`).
//!
//! D517 (#1877 item 7(a)): a global or system config on a network share
//! (a home folder on a share, say) is read on a background thread that the
//! look waits on for at most [`SHARE_TIMEOUT`] in all; one not read in time,
//! or whose read fails, counts as changed after every session (flagged,
//! never blocking). The include paths such a config names are never opened
//! on a share, mapped network drives included.
//!
//! Plain `std::fs`, so it is tested on every platform; only the non-admin
//! Windows sandbox uses it.

use crate::git_watch::Prints;
use std::path::Path;
use std::path::PathBuf;
use std::time::Duration;
use std::time::Instant;

/// Include files followed this deep (git's own limit).
const MAX_INCLUDE_DEPTH: usize = 10;
/// Entries listed in a named folder; past that it counts as changed after
/// every session.
const MAX_ENTRIES: usize = 1024;

/// Keys whose value is a command git runs (`section.key`, or
/// `section.*.key` for any sub-section), lower-cased.
const PROGRAM_KEYS: [&str; 44] = [
    "core.fsmonitor",
    "core.pager",
    "core.editor",
    "core.sshcommand",
    "core.askpass",
    "core.gitproxy",
    "core.alternaterefscommand",
    "sequence.editor",
    "gpg.program",
    "gpg.*.program",
    "gpg.ssh.defaultkeycommand",
    "diff.external",
    "diff.*.textconv",
    "diff.*.command",
    "difftool.*.cmd",
    "difftool.*.path",
    "merge.*.driver",
    "mergetool.*.cmd",
    "mergetool.*.path",
    "credential.helper",
    "credential.*.helper",
    "filter.*.clean",
    "filter.*.smudge",
    "filter.*.process",
    "interactive.difffilter",
    "uploadpack.packobjectshook",
    "remote.*.uploadpack",
    "remote.*.receivepack",
    "remote.*.vcs",
    "trailer.*.cmd",
    "trailer.*.command",
    "sendemail.tocmd",
    "sendemail.cccmd",
    "sendemail.smtpserver",
    "browser.*.cmd",
    "browser.*.path",
    "man.*.cmd",
    "man.*.path",
    "hook.*.command",
    "submodule.*.update",
    "sendemail.sendmailcmd",
    "sendemail.headercmd",
    "guitool.*.cmd",
    "instaweb.httpd",
];

/// Sections all of whose keys name commands or their arguments
/// (`pager.log`, `alias.x`).
const PROGRAM_SECTIONS: [&str; 2] = ["pager", "alias"];

/// Limits on what one look at the configs reads: past them the look counts
/// as changed after every session.
const MAX_CONFIG_FILES: usize = 4_096;
const MAX_CONFIG_BYTES: u64 = 1 << 20;
/// #1877 item 6: bytes of all the config files of one look together.
const MAX_CONFIG_TOTAL_BYTES: u64 = 16 << 20;
const MAX_CONFIG_ENTRIES: usize = 100_000;
const MAX_BASES: usize = 8;
const MAX_TARGETS: usize = 4_096;
/// D517: how long one look waits, in all, for the configs it reads from a
/// network share.
const SHARE_TIMEOUT: Duration = Duration::from_secs(2);

/// How much one look at the configs may still read, the files and bytes it
/// read, and where it stopped (it reads nothing more after that).
#[derive(Default)]
pub(crate) struct ConfigBudget {
    files: usize,
    bytes: u64,
    entries: usize,
    /// The first config file past a limit.
    pub(crate) overflow: Option<PathBuf>,
    /// D517: when the configs on a share must have been read by (set at the
    /// first one).
    share_deadline: Option<Instant>,
    /// D517: the configs on a share not read in time, or whose read failed.
    pub(crate) unread: Vec<PathBuf>,
}

impl ConfigBudget {
    fn over(&mut self, at: &Path) {
        if self.overflow.is_none() {
            self.overflow = Some(at.to_path_buf());
        }
    }
}

/// The names of git's hooks (`githooks(5)`).
const HOOK_NAMES: [&str; 28] = [
    "applypatch-msg",
    "pre-applypatch",
    "post-applypatch",
    "pre-commit",
    "pre-merge-commit",
    "prepare-commit-msg",
    "commit-msg",
    "post-commit",
    "pre-rebase",
    "post-checkout",
    "post-merge",
    "pre-push",
    "pre-receive",
    "update",
    "proc-receive",
    "post-receive",
    "post-update",
    "reference-transaction",
    "push-to-checkout",
    "pre-auto-gc",
    "post-rewrite",
    "sendemail-validate",
    "fsmonitor-watchman",
    "p4-changelist",
    "p4-prepare-changelist",
    "p4-post-changelist",
    "p4-pre-submit",
    "post-index-change",
];

/// A file or folder a config names.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Target {
    pub(crate) path: PathBuf,
    /// The key that names it (`core.hooksPath`, `include.path`, ...).
    pub(crate) why: String,
    pub(crate) kind: TargetKind,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(crate) enum TargetKind {
    /// A hooks folder: every entry.
    Hooks,
    /// The folder above Husky's `_`: the entries named after git hooks.
    HuskyHooks,
    /// A single file.
    File,
}

/// One `section[.sub].key = value` of a config file.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Entry {
    /// `section.key` or `section.sub.key`; section and key lower-cased.
    pub(crate) name: String,
    pub(crate) value: String,
}

/// The config files git reads besides a repository's own: the system and
/// global ones, the union of where git may look (`GIT_CONFIG_SYSTEM`,
/// `GIT_CONFIG_GLOBAL`, Git for Windows' and a user-scope install's
/// `etc\gitconfig`, `%PROGRAMDATA%\Git\config`, `/etc/gitconfig`, the XDG
/// config, and `.gitconfig` in each of `homes`), since the git the person
/// runs later may see another environment than this process.
pub(crate) fn outside_configs(homes: &[PathBuf]) -> Vec<PathBuf> {
    let mut configs = Vec::new();
    for var in ["GIT_CONFIG_SYSTEM", "GIT_CONFIG_GLOBAL"] {
        if let Some(path) = std::env::var_os(var) {
            configs.push(PathBuf::from(path));
        }
    }
    if let Some(data) = std::env::var_os("PROGRAMDATA") {
        configs.push(Path::new(&data).join("Git").join("config"));
    }
    let mut installs: Vec<PathBuf> = ["ProgramFiles", "ProgramFiles(x86)"]
        .iter()
        .filter_map(std::env::var_os)
        .map(|dir| Path::new(&dir).join("Git"))
        .collect();
    if let Some(local) = std::env::var_os("LOCALAPPDATA") {
        installs.push(Path::new(&local).join("Programs").join("Git"));
    }
    for git in installs {
        configs.push(git.join("etc").join("gitconfig"));
        configs.push(git.join("mingw64").join("etc").join("gitconfig"));
    }
    configs.push(PathBuf::from("/etc/gitconfig"));
    if let Some(xdg) = std::env::var_os("XDG_CONFIG_HOME") {
        configs.push(Path::new(&xdg).join("git").join("config"));
    }
    for home in homes {
        configs.push(home.join(".config").join("git").join("config"));
        configs.push(home.join(".gitconfig"));
    }
    configs.dedup();
    configs
}

/// The home folders git may use: `HOME` (Git for Windows prefers it), the
/// user profile, and `HOMEDRIVE` + `HOMEPATH`.
fn homes() -> Vec<PathBuf> {
    let mut homes: Vec<PathBuf> = std::env::var_os("HOME")
        .map(PathBuf::from)
        .into_iter()
        .collect();
    let drive_path = match (std::env::var_os("HOMEDRIVE"), std::env::var_os("HOMEPATH")) {
        (Some(drive), Some(path)) => {
            let mut joined = drive;
            joined.push(path);
            Some(PathBuf::from(joined))
        }
        _ => None,
    };
    for home in dirs_next::home_dir().into_iter().chain(drive_path) {
        if !homes.contains(&home) {
            homes.push(home);
        }
    }
    homes
}

/// Where a config path came from, which decides how it is opened.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Source {
    /// This process's environment (a global or system config, from
    /// [`outside_configs`]): one on a network share is read with a timeout
    /// (D517).
    Environment,
    /// A config or `.git` file a command may have written: never opened on a
    /// share or device.
    Named,
    /// D517: a config read from a share (or a file it includes) names it:
    /// never opened on a share either, mapped network drives included.
    NamedFromShare,
}

/// The entries of the configs at `paths` (and the files they include, each
/// once), each with the file it came from; and the files read (includes
/// too). Counts against `budget`. Never opens a network or device path.
pub(crate) fn read_configs(
    paths: &[PathBuf],
    homes: &[PathBuf],
    budget: &mut ConfigBudget,
) -> (Vec<(Entry, PathBuf)>, Vec<PathBuf>) {
    read_configs_from(paths, Source::Named, homes, budget)
}

/// [`read_configs`] for the global and system configs at `paths` (from
/// [`outside_configs`]): one on a network share is read with a timeout, and
/// counted in `budget.unread` when it isn't read in time (D517).
pub(crate) fn read_outside_configs(
    paths: &[PathBuf],
    homes: &[PathBuf],
    budget: &mut ConfigBudget,
) -> (Vec<(Entry, PathBuf)>, Vec<PathBuf>) {
    read_configs_from(paths, Source::Environment, homes, budget)
}

fn read_configs_from(
    paths: &[PathBuf],
    source: Source,
    homes: &[PathBuf],
    budget: &mut ConfigBudget,
) -> (Vec<(Entry, PathBuf)>, Vec<PathBuf>) {
    let mut entries = Vec::new();
    let mut files = Vec::new();
    // Each file once per look (an include cycle).
    let mut visited = std::collections::HashSet::new();
    for path in paths {
        read_config(
            path,
            source,
            homes,
            MAX_INCLUDE_DEPTH,
            budget,
            &mut visited,
            &mut |entry, from| {
                if !files.iter().any(|file: &PathBuf| file == from) {
                    files.push(from.to_path_buf());
                }
                entries.push((entry, from.to_path_buf()));
            },
        );
    }
    (entries, files)
}

/// The targets the configs of the repository with git folder `git_dir`
/// (and work tree `work_tree`, `None` for a bare one) name: its own configs
/// and `outside` (the system and global entries, from [`read_configs`]),
/// kept when they lie in one of `roots` (canonical) by name or after links,
/// and not inside a root's own (protected) `.git`. `homes` expand `~/`.
/// Counts against `budget`.
pub(crate) fn targets(
    git_dir: &Path,
    work_tree: Option<&Path>,
    outside: &[(Entry, PathBuf)],
    homes: &[PathBuf],
    roots: &[PathBuf],
    budget: &mut ConfigBudget,
) -> Vec<Target> {
    let common = std::fs::read(git_dir.join("commondir")).ok().map(|bytes| {
        // As git reads it: only trailing line ends go.
        let value = String::from_utf8_lossy(&bytes);
        let value = Path::new(value.trim_end_matches(['\n', '\r']));
        if value.is_absolute() {
            value.to_path_buf()
        } else {
            git_dir.join(value)
        }
    });
    let own = [
        common.as_deref().unwrap_or(git_dir).join("config"),
        git_dir.join("config.worktree"),
    ];
    let (own_entries, _) = read_configs(&own, homes, budget);
    // Hooks and program paths are relative to the work tree (the git folder
    // for a bare repository), or to `core.worktree` when set.
    let mut bases: Vec<PathBuf> = vec![work_tree.unwrap_or(git_dir).to_path_buf()];
    for (entry, from) in outside.iter().chain(&own_entries) {
        if entry.name == "core.worktree" {
            if bases.len() > MAX_BASES {
                budget.over(from);
                break;
            }
            bases.push(resolve(git_dir, &entry.value, homes));
        }
    }
    let mut found = Vec::new();
    'entries: for (entry, from) in outside.iter().chain(&own_entries) {
        for base in &bases {
            target_of(entry, from, base, homes, &mut found);
            if found.len() > MAX_TARGETS {
                budget.over(from);
                break 'entries;
            }
        }
    }
    // `receive-pack` runs hooks from the git folder, so a relative
    // `core.hooksPath` also names a folder there.
    if work_tree.is_some() {
        for (entry, from) in outside.iter().chain(&own_entries) {
            if entry.name == "core.hookspath" {
                target_of(entry, from, git_dir, homes, &mut found);
            }
        }
    }
    keep_in_roots(found, roots)
}

/// `found`, normalized, each path once, those in `roots`. A path named
/// twice keeps the kind that records the most (a hooks folder over Husky's
/// hook-named files over a file).
fn keep_in_roots(found: Vec<Target>, roots: &[PathBuf]) -> Vec<Target> {
    fn rank(kind: TargetKind) -> u8 {
        match kind {
            TargetKind::Hooks => 2,
            TargetKind::HuskyHooks => 1,
            TargetKind::File => 0,
        }
    }
    let mut index = std::collections::HashMap::new();
    let mut kept: Vec<Target> = Vec::new();
    for mut target in found {
        target.path = crate::git_watch::normalize(&target.path);
        if let Some(&at) = index.get(&target.path) {
            let existing: &mut Target = &mut kept[at];
            if rank(target.kind) > rank(existing.kind) {
                *existing = target;
            }
            continue;
        }
        if in_roots(&target.path, roots) {
            index.insert(target.path.clone(), kept.len());
            kept.push(target);
        }
    }
    kept
}

/// `path` lies in one of `roots`, by name or after links, and not inside a
/// root's own `.git` (protected, and D332's). A network or device path is
/// never resolved (a command may have written it); links are only resolved
/// when the name alone isn't in a root.
fn in_roots(path: &Path, roots: &[PathBuf]) -> bool {
    if crate::git_watch::is_remote_or_device(path) {
        return false;
    }
    let inside = |path: &Path| {
        roots.iter().any(|root| {
            crate::git_watch::is_at_or_below(path, root)
                && !crate::git_watch::is_at_or_below(path, &root.join(".git"))
        })
    };
    inside(path) || dunce::canonicalize(path).is_ok_and(|canonical| inside(&canonical))
}

/// Calls `each` with every entry of the config file at `path` and the file
/// it came from, following `include.path` and `includeIf.*.path` (whatever
/// the condition). Each file is read once per look; never a network or device path
/// (a command may have written it), except a global or system config on a
/// share (D517, see [`Source`]); past `budget`'s limits it stops and
/// records where.
fn read_config(
    path: &Path,
    source: Source,
    homes: &[PathBuf],
    depth: usize,
    budget: &mut ConfigBudget,
    visited: &mut std::collections::HashSet<PathBuf>,
    each: &mut dyn FnMut(Entry, &Path),
) {
    use std::io::Read as _;
    // #1877 item 6: past a limit, nothing more is read.
    if budget.overflow.is_some() {
        return;
    }
    let path = &local(path);
    let network = || crate::local_drive::is_network_path(path);
    let on_share =
        source == Source::Environment && !crate::git_watch::has_device_name(path) && network();
    let refused = !on_share
        && (crate::git_watch::is_remote_or_device(path)
            || (source == Source::NamedFromShare && network()));
    if refused {
        return;
    }
    if !visited.insert(crate::git_watch::normalize(path)) {
        return;
    }
    // Only files that exist count.
    let count = |budget: &mut ConfigBudget| {
        budget.files += 1;
        if budget.files > MAX_CONFIG_FILES {
            budget.over(path);
            return false;
        }
        true
    };
    // #1877 item 6: no more than what is left of the total either.
    let limit = MAX_CONFIG_BYTES.min(MAX_CONFIG_TOTAL_BYTES.saturating_sub(budget.bytes));
    let bytes = if on_share {
        let deadline = *budget
            .share_deadline
            .get_or_insert_with(|| Instant::now() + SHARE_TIMEOUT);
        let owned = path.to_path_buf();
        let read = read_by(path, deadline, move || {
            let mut bytes = Vec::new();
            std::fs::File::open(&owned)?
                .take(limit + 1)
                .read_to_end(&mut bytes)?;
            Ok(bytes)
        });
        match read {
            Ok(Some(bytes)) => {
                if !count(budget) {
                    return;
                }
                bytes
            }
            Ok(None) => return,
            Err(()) => {
                budget.unread.push(path.to_path_buf());
                return;
            }
        }
    } else {
        let Ok(file) = std::fs::File::open(path) else {
            return;
        };
        if !count(budget) {
            return;
        }
        let mut bytes = Vec::new();
        if file.take(limit + 1).read_to_end(&mut bytes).is_err() {
            return;
        }
        bytes
    };
    if bytes.len() as u64 > limit {
        budget.over(path);
        return;
    }
    budget.bytes += bytes.len() as u64;
    for entry in parse(&String::from_utf8_lossy(&bytes)) {
        budget.entries += 1;
        if budget.entries > MAX_CONFIG_ENTRIES {
            budget.over(path);
            return;
        }
        let included = is_include(&entry.name)
            .then(|| resolve(path.parent().unwrap_or(Path::new("")), &entry.value, homes));
        each(entry, path);
        if let Some(included) = included
            && depth > 0
        {
            let source = if on_share || source == Source::NamedFromShare {
                Source::NamedFromShare
            } else {
                Source::Named
            };
            read_config(&included, source, homes, depth - 1, budget, visited, each);
        }
    }
}

/// D517: runs `read` (of the config at `path` on a network share) on a
/// background thread and waits for it until `deadline` (not at all once it
/// has passed). `Ok(None)` when the file isn't there; `Err` when it isn't
/// read by then, the read fails, or an earlier read of `path` is still
/// waiting (a share that hangs holds one thread per path, not one per look).
/// A read not waited for is left to finish on its own, so the next look
/// doesn't wait for every hung path in turn.
fn read_by(
    path: &Path,
    deadline: Instant,
    read: impl FnOnce() -> std::io::Result<Vec<u8>> + Send + 'static,
) -> Result<Option<Vec<u8>>, ()> {
    static WAITING: std::sync::Mutex<Vec<PathBuf>> = std::sync::Mutex::new(Vec::new());
    let waiting = || {
        WAITING
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    };
    let path = path.to_path_buf();
    {
        let mut waiting = waiting();
        if waiting.contains(&path) {
            return Err(());
        }
        waiting.push(path.clone());
    }
    let (send, receive) = std::sync::mpsc::channel();
    let thread_path = path.clone();
    let spawned = std::thread::Builder::new()
        .name("git-config-share".to_string())
        .spawn(move || {
            let result = read();
            waiting().retain(|other| *other != thread_path);
            let _ = send.send(result);
        });
    if spawned.is_err() {
        waiting().retain(|other| *other != path);
        return Err(());
    }
    match receive.recv_timeout(deadline.saturating_duration_since(Instant::now())) {
        Ok(Ok(bytes)) => Ok(Some(bytes)),
        Ok(Err(err)) if is_absent(&err) => Ok(None),
        _ => Err(()),
    }
}

/// The file (or a folder above it) isn't there. On Windows only
/// `ERROR_FILE_NOT_FOUND` and `ERROR_PATH_NOT_FOUND`: a share that can't be
/// reached (`ERROR_BAD_NETPATH`, `ERROR_BAD_NET_NAME` ...) also reads as
/// `NotFound`, and is a failure.
fn is_absent(err: &std::io::Error) -> bool {
    if cfg!(windows) {
        matches!(err.raw_os_error(), Some(2 | 3))
    } else {
        err.kind() == std::io::ErrorKind::NotFound
    }
}

fn is_include(name: &str) -> bool {
    name == "include.path" || (name.starts_with("includeif.") && name.ends_with(".path"))
}

/// The targets `entry` (from the config file `from`) names, relative to
/// `base`.
fn target_of(entry: &Entry, from: &Path, base: &Path, homes: &[PathBuf], found: &mut Vec<Target>) {
    let name = entry.name.as_str();
    if name == "core.hookspath" {
        let path = resolve(base, &entry.value, homes);
        let why = || "core.hooksPath".to_string();
        // Husky v9 (`husky <folder>` sets `<folder>\_`): its stubs run
        // `<folder>\<hook>`, and `~/.config/husky/init.sh` first.
        if path.file_name().is_some_and(|name| name == "_")
            && let Some(parent) = path.parent()
        {
            found.push(Target {
                path: parent.to_path_buf(),
                why: why(),
                kind: TargetKind::HuskyHooks,
            });
            for home in homes {
                found.push(Target {
                    path: home.join(".config").join("husky").join("init.sh"),
                    why: why(),
                    kind: TargetKind::File,
                });
            }
        }
        // Husky v5 to v8: each hook sources `<folder>\_\husky.sh`.
        found.push(Target {
            path: path.join("_"),
            why: why(),
            kind: TargetKind::Hooks,
        });
        found.push(Target {
            path,
            why: why(),
            kind: TargetKind::Hooks,
        });
        return;
    }
    if is_include(name) {
        found.push(Target {
            path: resolve(from.parent().unwrap_or(Path::new("")), &entry.value, homes),
            why: "include.path".to_string(),
            kind: TargetKind::File,
        });
        return;
    }
    if name == "init.templatedir" {
        found.push(Target {
            path: resolve(base, &entry.value, homes).join("hooks"),
            why: entry.name.clone(),
            kind: TargetKind::Hooks,
        });
        return;
    }
    let section = name.split('.').next().unwrap_or_default();
    let program = PROGRAM_KEYS.iter().any(|key| key_matches(key, name))
        || PROGRAM_SECTIONS.contains(&section);
    if !program {
        return;
    }
    // A command's first word is a file when it looks like a path (a bare
    // name is looked up on PATH); every later word may be a file it runs or
    // reads (`!sh x.sh`, `node scripts/cred.js`, `rebase -x ./t.sh`,
    // `-oProxyCommand=./x`).
    // At most 64 words of one value.
    for (index, word) in words(&entry.value).into_iter().take(64).enumerate() {
        let word = word.trim_start_matches('!');
        let path_like = word.contains('/') || word.contains('\\') || word.starts_with('.');
        let skipped = word.is_empty()
            || word.contains("://")
            || word.starts_with('-')
            || (word.starts_with('$') && !word.starts_with("$HOME"))
            || (index == 0 && !path_like);
        if skipped {
            continue;
        }
        let mut paths = resolve_all(base, word, homes);
        // `"$(git rev-parse --show-toplevel)/x.sh"` leaves `/x.sh`.
        if let Some(rest) = word.strip_prefix('/') {
            paths.push(base.join(rest));
        }
        for path in paths {
            found.push(Target {
                path,
                why: entry.name.clone(),
                kind: TargetKind::File,
            });
        }
    }
}

/// `pattern` (`a.b` or `a.*.b`) matches the entry name `name`.
fn key_matches(pattern: &str, name: &str) -> bool {
    match pattern.split_once(".*.") {
        Some((section, key)) => name
            .strip_prefix(section)
            .and_then(|rest| rest.strip_prefix('.'))
            .and_then(|rest| rest.rsplit_once('.'))
            .is_some_and(|(sub, last)| !sub.is_empty() && last == key),
        None => name == pattern,
    }
}

/// The shell words of `command`, unquoted, split also at `;`, `&`, `|`,
/// `(`, `)`, `{`, `}`, `<`, `>` and `=` (`-oProxyCommand=./x` names
/// `./x`).
fn words(command: &str) -> Vec<String> {
    let mut words = Vec::new();
    let mut word = String::new();
    let mut quote: Option<char> = None;
    for c in command.chars() {
        match (quote, c) {
            (None, '"' | '\'') => quote = Some(c),
            (Some(q), c) if c == q => quote = None,
            (None, c) if c.is_ascii_whitespace() || ";&|(){}<>=".contains(c) => {
                if !word.is_empty() {
                    words.push(std::mem::take(&mut word));
                }
            }
            (_, c) => word.push(c),
        }
    }
    if !word.is_empty() {
        words.push(word);
    }
    words
}

/// `value` (a path in a config) relative to `base`; `~/` and `$HOME/` in
/// the first home folder (see [`resolve_all`]).
fn resolve(base: &Path, value: &str, homes: &[PathBuf]) -> PathBuf {
    resolve_all(base, value, homes)
        .into_iter()
        .next()
        .unwrap_or_else(|| base.to_path_buf())
}

/// `value` relative to `base`, or in each home folder for `~/` and
/// `$HOME/`. Each [`local`].
fn resolve_all(base: &Path, value: &str, homes: &[PathBuf]) -> Vec<PathBuf> {
    let value = value.trim_ascii();
    let in_home = ["~/", "~\\", "$HOME/", "$HOME\\"]
        .iter()
        .find_map(|prefix| value.strip_prefix(prefix));
    if let Some(rest) = in_home
        && !homes.is_empty()
    {
        return homes.iter().map(|home| local(&home.join(rest))).collect();
    }
    let path = Path::new(value);
    if path.is_absolute() || value.starts_with('/') || value.starts_with('\\') {
        vec![local(path)]
    } else {
        vec![local(&base.join(path))]
    }
}

/// #1877 item 7(b): `path` with a `\\?\` drive-letter prefix (`\\?\C:\x`)
/// written as Windows would read it without one (`C:\x`), when that names
/// the same file (`dunce`'s rule: no `..`, reserved or trailing-dot names,
/// not too long) from the top of the drive (not `\\?\C:`, the volume,
/// which would become `C:`, a folder relative to the current one); any
/// other path as it is. A network path (`\\?\UNC\...`) keeps its prefix,
/// so it is still never opened.
fn local(path: &Path) -> PathBuf {
    let simple = dunce::simplified(path);
    if simple.has_root() {
        simple.to_path_buf()
    } else {
        path.to_path_buf()
    }
}

/// The entries of a config file's text: `[section]`, `[section "sub"]` and
/// the deprecated `[section.sub]` headers, `key = value` lines (a key alone
/// is `true`), quotes, escapes, comments and continuation lines.
pub(crate) fn parse(text: &str) -> Vec<Entry> {
    let mut entries = Vec::new();
    let mut section = String::new();
    // Git skips a UTF-8 byte-order mark.
    let text = text.strip_prefix('\u{FEFF}').unwrap_or(text);
    let mut chars = text.chars().peekable();
    loop {
        // Skip blanks and comments up to the next header or key.
        while let Some(&c) = chars.peek() {
            if c.is_ascii_whitespace() {
                chars.next();
            } else if c == '#' || c == ';' {
                for c in chars.by_ref() {
                    if c == '\n' {
                        break;
                    }
                }
            } else {
                break;
            }
        }
        let Some(&c) = chars.peek() else {
            break;
        };
        if c == '[' {
            chars.next();
            let mut header = String::new();
            let mut in_quotes = false;
            while let Some(c) = chars.next() {
                match c {
                    '"' => {
                        in_quotes = !in_quotes;
                        header.push('\u{1}');
                    }
                    '\\' if in_quotes => {
                        if let Some(next) = chars.next() {
                            header.push(next);
                        }
                    }
                    ']' if !in_quotes => break,
                    c => header.push(c),
                }
            }
            section = match header.split_once('\u{1}') {
                Some((name, sub)) => format!(
                    "{}.{}",
                    name.trim_ascii().to_ascii_lowercase(),
                    sub.trim_end_matches('\u{1}')
                ),
                None => match header.trim_ascii().split_once('.') {
                    Some((name, sub)) => {
                        format!("{}.{}", name.to_ascii_lowercase(), sub.to_ascii_lowercase())
                    }
                    None => header.trim_ascii().to_ascii_lowercase(),
                },
            };
            continue;
        }
        let mut key = String::new();
        while let Some(&c) = chars.peek() {
            if c.is_ascii_alphanumeric() || c == '-' {
                key.push(c);
                chars.next();
            } else {
                break;
            }
        }
        if key.is_empty() {
            // Not a key: skip the line.
            for c in chars.by_ref() {
                if c == '\n' {
                    break;
                }
            }
            continue;
        }
        while chars.peek().is_some_and(|c| *c == ' ' || *c == '\t') {
            chars.next();
        }
        let value = if chars.peek() == Some(&'=') {
            chars.next();
            read_value(&mut chars)
        } else {
            for c in chars.by_ref() {
                if c == '\n' {
                    break;
                }
            }
            "true".to_string()
        };
        entries.push(Entry {
            name: format!("{section}.{}", key.to_ascii_lowercase()),
            value,
        });
    }
    entries
}

fn read_value(chars: &mut std::iter::Peekable<std::str::Chars<'_>>) -> String {
    let mut value = String::new();
    let mut in_quotes = false;
    // Trailing unquoted blanks are dropped.
    let mut pending_space = String::new();
    while let Some(c) = chars.next() {
        match c {
            '\n' => break,
            '"' => {
                value.push_str(&pending_space);
                pending_space.clear();
                in_quotes = !in_quotes;
            }
            '\\' => match chars.next() {
                Some('\n') => {}
                Some('\r') if chars.peek() == Some(&'\n') => {
                    chars.next();
                }
                Some('n') => {
                    value.push_str(&pending_space);
                    pending_space.clear();
                    value.push('\n');
                }
                Some('t') => {
                    value.push_str(&pending_space);
                    pending_space.clear();
                    value.push('\t');
                }
                Some('b') => {
                    value.push_str(&pending_space);
                    pending_space.clear();
                    value.push('\u{8}');
                }
                Some(other) => {
                    value.push_str(&pending_space);
                    pending_space.clear();
                    value.push(other);
                }
                None => break,
            },
            '#' | ';' if !in_quotes => {
                for c in chars.by_ref() {
                    if c == '\n' {
                        break;
                    }
                }
                break;
            }
            '\r' if !in_quotes && chars.peek() == Some(&'\n') => {}
            // Git keeps an unquoted run of blanks inside a value as spaces.
            c if c.is_ascii_whitespace() && !in_quotes => {
                if !value.is_empty() {
                    pending_space.push(' ');
                }
            }
            c => {
                value.push_str(&pending_space);
                pending_space.clear();
                value.push(c);
            }
        }
    }
    value
}

/// The git folder and work tree (`None` for a bare repository) of each of
/// `entries`: a `.git` folder, a `.git` file (the folder its `gitdir:` line
/// names) or a bare repository's folder. A `.git` link isn't followed.
pub(crate) fn repo_dirs(entries: &[PathBuf]) -> Vec<(PathBuf, Option<PathBuf>)> {
    entries
        .iter()
        .filter_map(|entry| {
            if !entry
                .file_name()
                .is_some_and(|name| name.eq_ignore_ascii_case(".git"))
            {
                return Some((entry.clone(), None));
            }
            let work_tree = entry.parent()?.to_path_buf();
            let metadata = std::fs::symlink_metadata(entry).ok()?;
            if metadata.is_dir() {
                Some((entry.clone(), Some(work_tree)))
            } else if metadata.is_file() {
                // As git reads it; never a network or device path.
                let target = crate::git_watch::gitdir_target(entry)?;
                (!crate::git_watch::is_remote_or_device(&target))
                    .then_some((target, Some(work_tree)))
            } else {
                None
            }
        })
        .collect()
}

/// A target and what was in it, when a session started or ended.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(crate) struct TargetState {
    pub(crate) path: PathBuf,
    pub(crate) why: String,
    pub(crate) kind: TargetKind,
    pub(crate) prints: Prints,
}

impl TargetState {
    /// What is in it now.
    pub(crate) fn now(&self) -> Prints {
        target_prints(&Target {
            path: self.path.clone(),
            why: self.why.clone(),
            kind: self.kind,
        })
    }
}

/// The targets the configs of `repos` (each a git folder and its work tree,
/// `None` for a bare repository) and the system and global configs name
/// inside `roots`, with what is in each; and those configs themselves (and
/// the files they include) when they lie in `roots`.
pub(crate) fn target_states(
    repos: &[(PathBuf, Option<PathBuf>)],
    roots: &[PathBuf],
) -> Vec<TargetState> {
    let homes = homes();
    target_states_with(repos, roots, &outside_configs(&homes), &homes)
}

/// [`target_states`] with the system and global configs and the home
/// folders passed in.
pub(crate) fn target_states_with(
    repos: &[(PathBuf, Option<PathBuf>)],
    roots: &[PathBuf],
    outside_paths: &[PathBuf],
    homes: &[PathBuf],
) -> Vec<TargetState> {
    let mut budget = ConfigBudget::default();
    // Read once for all repositories.
    let (outside, outside_files) = read_outside_configs(outside_paths, homes, &mut budget);
    let mut found: Vec<Target> = outside_files
        .into_iter()
        .map(|path| Target {
            path,
            why: "git config".to_string(),
            kind: TargetKind::File,
        })
        .collect();
    // What the system and global configs name absolutely (or in a home
    // folder) counts with no repository in the roots too.
    let mut absolute = Vec::new();
    for (entry, from) in &outside {
        target_of(entry, from, Path::new(""), homes, &mut absolute);
    }
    found.extend(
        absolute
            .into_iter()
            .filter(|target| target.path.is_absolute()),
    );
    for (git_dir, work_tree) in repos {
        found.extend(targets(
            git_dir,
            work_tree.as_deref(),
            &outside,
            homes,
            roots,
            &mut budget,
        ));
        if budget.overflow.is_some() {
            break;
        }
    }
    // #1877: several targets at a time.
    let mut states: Vec<TargetState> =
        crate::par_map::par_map(&keep_in_roots(found, roots), |target| TargetState {
            prints: target_prints(target),
            path: target.path.clone(),
            why: target.why.clone(),
            kind: target.kind,
        });
    // Past a limit, or not read from a share in time (D517), the configs
    // count as changed after every session.
    let unread = budget
        .unread
        .into_iter()
        .map(|at| (at, "git config on a network share not read in time"));
    for (at, why) in budget
        .overflow
        .map(|at| (at, "git config too large to check"))
        .into_iter()
        .chain(unread)
    {
        let mut prints = Prints::new();
        prints.insert(String::new(), crate::git_watch::truncated());
        states.push(TargetState {
            path: at,
            why: why.to_string(),
            kind: TargetKind::File,
            prints,
        });
    }
    states
}

/// What changed in each target of `now` since `before` (taken when the
/// session started), for the person: `core.hooksPath: pre-commit added`
/// ... A target only `now` has counts with what is in it (a new repository
/// with a hooks folder a global config names, a link replaced by a folder).
pub(crate) fn target_changes(
    before: &[TargetState],
    now: Vec<TargetState>,
) -> Vec<(PathBuf, Vec<String>)> {
    // #1877: the first of `before` at the same path both ways by
    // `is_at_or_below` (each component equal ignoring ASCII case), looked up
    // rather than compared with each.
    let mut by_path = std::collections::HashMap::new();
    for old in before {
        by_path.entry(path_key(&old.path)).or_insert(old);
    }
    now.into_iter()
        .filter_map(|state| {
            let old = by_path.get(&path_key(&state.path));
            let empty = Prints::new();
            let changes: Vec<String> =
                crate::git_watch::changes(old.map_or(&empty, |old| &old.prints), &state.prints)
                    .into_iter()
                    .map(|change| format!("{}: {}", state.why, change.trim_start()))
                    .collect();
            (!changes.is_empty()).then_some((state.path, changes))
        })
        .collect()
}

/// `path` by component, ASCII lower-cased: two paths have the same key
/// exactly when each is [`crate::git_watch::is_at_or_below`] the other.
pub(crate) fn path_key(path: &Path) -> Vec<String> {
    path.components()
        .map(|part| part.as_os_str().to_string_lossy().to_ascii_lowercase())
        .collect()
}

/// What is in `target` now: every entry of a hooks folder (a link to one is
/// followed, and recorded), the hook-named entries of Husky's folder, or a
/// file's content.
pub(crate) fn target_prints(target: &Target) -> Prints {
    let mut prints = Prints::new();
    match target.kind {
        TargetKind::File => {
            if std::fs::symlink_metadata(&target.path).is_ok() {
                prints.insert(String::new(), crate::git_watch::file_print(&target.path));
            }
        }
        TargetKind::Hooks | TargetKind::HuskyHooks => {
            if std::fs::symlink_metadata(&target.path)
                .is_ok_and(|metadata| metadata.file_type().is_symlink())
            {
                prints.insert(
                    "(link)".to_string(),
                    crate::git_watch::file_print(&target.path),
                );
            }
            let entries = match std::fs::read_dir(&target.path) {
                Ok(entries) => entries,
                Err(err) if err.kind() == std::io::ErrorKind::NotFound => return prints,
                Err(_) => {
                    prints.insert(String::new(), "unreadable".to_string());
                    return prints;
                }
            };
            for (count, entry) in entries.enumerate() {
                if count >= MAX_ENTRIES {
                    prints.insert(String::new(), crate::git_watch::truncated());
                    break;
                }
                let Ok(entry) = entry else {
                    prints.insert(String::new(), "unreadable".to_string());
                    break;
                };
                let name = entry.file_name().to_string_lossy().into_owned();
                if target.kind == TargetKind::HuskyHooks
                    && !HOOK_NAMES
                        .iter()
                        .any(|hook| hook.eq_ignore_ascii_case(&name))
                {
                    continue;
                }
                prints.insert(name, crate::git_watch::file_print(&entry.path()));
            }
        }
    }
    prints
}

#[cfg(test)]
#[path = "git_config_targets_tests.rs"]
mod tests;
