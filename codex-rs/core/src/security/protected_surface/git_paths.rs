//! What git reads to decide which hooks and config run, for the protected-path
//! rules (PF-23-S02): these paths become read-only.

use codex_utils_absolute_path::AbsolutePathBuf;
use std::path::Path;

/// Folders below `.git/modules` visited at most, and how deep. Past either
/// limit the whole `modules` folder becomes read-only instead (fail closed).
const MAX_MODULE_DIRS: usize = 512;
const MAX_MODULE_DEPTH: usize = 8;

/// For the repository `root` is in (the nearest `.git` at or above it, found
/// the way git finds it): hooks, config, config.worktree and commondir of its
/// git folder; for a `.git` file (worktree, submodule) the file itself and the
/// same entries in the folder it points to and its common folder; the folders
/// `core.hooksPath` names in its config, `~/.gitconfig` or
/// `~/.config/git/config`; and the same entries of every folder below
/// `.git/modules` (submodule git folders, also nested ones).
///
/// On Linux a missing commondir, hooks folder or module entry is left out:
/// the sandbox would put an empty placeholder there, which breaks git.
/// Includes (`[include]`) are not
/// followed, and a new nested `.git` stays with the command-text net.
pub(super) fn git_persistence_paths(
    root: &AbsolutePathBuf,
    user_home: Option<&Path>,
) -> Vec<AbsolutePathBuf> {
    let Some(dot_git) = root
        .as_path()
        .ancestors()
        .map(|folder| folder.join(".git"))
        .find(|dot_git| std::fs::symlink_metadata(dot_git).is_ok())
        .and_then(|dot_git| AbsolutePathBuf::from_absolute_path(dot_git).ok())
    else {
        return Vec::new();
    };
    let Some(work_tree) = dot_git.parent() else {
        return Vec::new();
    };
    let (mut paths, git_dirs) = if dot_git.as_path().is_dir() {
        (Vec::new(), vec![dot_git])
    } else {
        let mut git_dirs = Vec::new();
        if let Some(git_dir) = std::fs::read_to_string(dot_git.as_path())
            .ok()
            .and_then(|text| {
                text.lines()
                    .find_map(|line| line.strip_prefix("gitdir:"))
                    .map(|dir| dir.trim().to_string())
            })
        {
            let git_dir = AbsolutePathBuf::resolve_path_against_base(git_dir, work_tree.as_path());
            let common = std::fs::read_to_string(git_dir.join("commondir").as_path())
                .ok()
                .map(|dir| {
                    AbsolutePathBuf::resolve_path_against_base(dir.trim(), git_dir.as_path())
                });
            git_dirs.push(git_dir);
            git_dirs.extend(common);
        }
        (vec![dot_git], git_dirs)
    };
    let mut configs: Vec<AbsolutePathBuf> = Vec::new();
    for dir in &git_dirs {
        paths.extend(entries(dir, /*below_modules*/ false));
        configs.push(dir.join("config"));
        configs.push(dir.join("config.worktree"));
        paths.extend(module_paths(dir));
    }
    let home = user_home.and_then(|home| AbsolutePathBuf::from_absolute_path(home).ok());
    if let Some(home) = &home {
        configs.push(home.join(".gitconfig"));
        configs.push(home.join(".config/git/config"));
    }
    for config in configs {
        if let Some(hooks) = hooks_path(&config, &work_tree, home.as_ref())
            && (!cfg!(target_os = "linux") || hooks.as_path().exists())
        {
            paths.push(hooks);
        }
    }
    paths
}

/// The hooks, config, config.worktree and commondir of one git folder. On
/// Linux a missing commondir is left out; below `.git/modules` (where most
/// folders are not git folders) every missing entry is.
fn entries(git_dir: &AbsolutePathBuf, below_modules: bool) -> Vec<AbsolutePathBuf> {
    ["hooks", "config", "config.worktree", "commondir"]
        .into_iter()
        .map(|entry| git_dir.join(entry))
        .filter(|path| {
            let may_be_missing = below_modules || path.as_path().ends_with("commondir");
            !cfg!(target_os = "linux")
                || !may_be_missing
                || std::fs::symlink_metadata(path.as_path()).is_ok()
        })
        .collect()
}

/// Every folder below `git_dir/modules`, without following links: its four
/// entries, whether or not it looks like a git folder (removing `HEAD` must
/// not unprotect one). A git folder (one with `objects`) is entered only at
/// its own `modules`. Past the limits, `modules` as a whole.
fn module_paths(git_dir: &AbsolutePathBuf) -> Vec<AbsolutePathBuf> {
    let modules = git_dir.join("modules");
    let mut paths = Vec::new();
    let mut pending = vec![(modules.clone(), 0)];
    let mut visited = 0;
    while let Some((folder, depth)) = pending.pop() {
        let Ok(children) = std::fs::read_dir(folder.as_path()) else {
            continue;
        };
        for child in children.flatten() {
            if !child.file_type().is_ok_and(|kind| kind.is_dir()) {
                continue;
            }
            visited += 1;
            if visited > MAX_MODULE_DIRS || depth >= MAX_MODULE_DEPTH {
                return vec![modules];
            }
            let module = folder.join(child.file_name());
            paths.extend(entries(&module, /*below_modules*/ true));
            if module.join("objects").as_path().is_dir() {
                pending.push((module.join("modules"), depth + 1));
            } else {
                // A submodule path with slashes nests its git folder.
                pending.push((module, depth + 1));
            }
        }
    }
    paths
}

/// The `core.hooksPath` a git config file sets (the last one wins), resolved
/// like git: `~/` against the user's home, a relative path against the work
/// tree. Comments and quotes are handled; escapes and includes are not.
fn hooks_path(
    config: &AbsolutePathBuf,
    work_tree: &AbsolutePathBuf,
    home: Option<&AbsolutePathBuf>,
) -> Option<AbsolutePathBuf> {
    let text = std::fs::read_to_string(config.as_path()).ok()?;
    let mut in_core = false;
    let mut found = None;
    for line in text.lines() {
        let mut line = line.trim();
        if let Some(rest) = line.strip_prefix('[') {
            let Some((section, after)) = rest.split_once(']') else {
                in_core = false;
                continue;
            };
            in_core = section.trim().eq_ignore_ascii_case("core");
            // `[core] hooksPath = x` on one line.
            line = after.trim();
        }
        let Some((key, value)) = line.split_once('=') else {
            continue;
        };
        if in_core && key.trim().eq_ignore_ascii_case("hookspath") {
            found = Some(config_value(value));
        }
    }
    let value = found.filter(|value| !value.is_empty())?;
    match value.strip_prefix("~/") {
        Some(rest) => home.map(|home| home.join(rest)),
        None if value.starts_with('~') => None,
        None => Some(AbsolutePathBuf::resolve_path_against_base(
            value,
            work_tree.as_path(),
        )),
    }
}

/// A git config value without its comment (`#` or `;` outside quotes) and
/// quotes.
fn config_value(raw: &str) -> String {
    let mut value = String::new();
    let mut quoted = false;
    for character in raw.trim().chars() {
        match character {
            '"' => quoted = !quoted,
            '#' | ';' if !quoted => break,
            other => value.push(other),
        }
    }
    value.trim().to_string()
}

#[cfg(test)]
#[path = "git_paths_tests.rs"]
mod tests;
