use std::ffi::OsStr;
use std::io;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;

const HIERARCHY_WORKTREE_RESERVE_BYTES: u64 = 60 * 1024 * 1024 * 1024;
const MINIMUM_WORKTREE_ESTIMATE_BYTES: u64 = 1024 * 1024 * 1024;

pub(crate) async fn enforce_hierarchy_worktree_headroom(
    command: &[String],
    agent_role: Option<&str>,
    cwd: Option<&Path>,
) -> Result<(), String> {
    if !is_hierarchy_role(agent_role) || !contains_git_worktree_add(command) {
        return Ok(());
    }

    let cwd = cwd.ok_or_else(|| {
        "PFTerminal hierarchy disk guard rejected `git worktree add`: the target environment is not local, so disk headroom cannot be measured mechanically"
            .to_string()
    })?;
    let cwd = cwd.to_path_buf();
    tokio::task::spawn_blocking(move || enforce_local_worktree_headroom(&cwd))
        .await
        .map_err(|err| format!("PFTerminal hierarchy disk guard failed: {err}"))?
}

fn is_hierarchy_role(agent_role: Option<&str>) -> bool {
    matches!(agent_role, Some("nazgul" | "troll" | "orc"))
}

fn contains_git_worktree_add(command: &[String]) -> bool {
    if command_starts_worktree_add(command) {
        return true;
    }

    let Some((_, script)) = codex_shell_command::parse_command::extract_shell_command(command)
    else {
        return false;
    };

    shell_command_segments(script).iter().any(|segment| {
        let Some(tokens) = shlex::split(segment) else {
            return false;
        };
        command_starts_worktree_add(&tokens)
    })
}

fn shell_command_segments(script: &str) -> Vec<String> {
    let mut segments = Vec::new();
    let mut current = String::new();
    let mut quote = None;
    let mut escaped = false;
    for character in script.chars() {
        if escaped {
            current.push(character);
            escaped = false;
            continue;
        }
        if character == '\\' && quote != Some('\'') {
            current.push(character);
            escaped = true;
            continue;
        }
        if matches!(character, '\'' | '"') {
            if quote == Some(character) {
                quote = None;
            } else if quote.is_none() {
                quote = Some(character);
            }
            current.push(character);
            continue;
        }
        if quote.is_none() && matches!(character, '\n' | ';' | '&' | '|') {
            if !current.trim().is_empty() {
                segments.push(current.trim().to_string());
            }
            current.clear();
            continue;
        }
        current.push(character);
    }
    if !current.trim().is_empty() {
        segments.push(current.trim().to_string());
    }
    segments
}

fn command_starts_worktree_add(tokens: &[String]) -> bool {
    let mut index = 0;
    while tokens.get(index).is_some_and(|token| {
        token == "{" || token == "}" || token == "then" || is_shell_assignment(token)
    }) {
        index += 1;
    }
    while tokens
        .get(index)
        .is_some_and(|token| matches!(token.as_str(), "command" | "exec" | "sudo" | "env"))
    {
        index += 1;
        while tokens
            .get(index)
            .is_some_and(|token| is_shell_assignment(token))
        {
            index += 1;
        }
    }

    let Some(executable) = tokens.get(index) else {
        return false;
    };
    let executable = Path::new(executable)
        .file_name()
        .and_then(OsStr::to_str)
        .unwrap_or(executable);
    if !matches!(executable, "git" | "git.exe") {
        return false;
    }
    index += 1;

    while let Some(argument) = tokens.get(index) {
        match argument.as_str() {
            "-C" | "-c" | "--git-dir" | "--work-tree" | "--namespace" => index += 2,
            value
                if value.starts_with("--git-dir=")
                    || value.starts_with("--work-tree=")
                    || value.starts_with("--namespace=") =>
            {
                index += 1;
            }
            "worktree" => break,
            _ => return false,
        }
    }

    tokens.get(index).is_some_and(|token| token == "worktree")
        && tokens.get(index + 1).is_some_and(|token| token == "add")
}

fn is_shell_assignment(token: &str) -> bool {
    let Some((name, _)) = token.split_once('=') else {
        return false;
    };
    !name.is_empty()
        && name
            .bytes()
            .all(|byte| byte == b'_' || byte.is_ascii_alphanumeric())
}

fn enforce_local_worktree_headroom(cwd: &Path) -> Result<(), String> {
    let repository_root = git_repository_root(cwd)?;
    let source_bytes = measure_working_tree_bytes(&repository_root).map_err(|err| {
        format!(
            "PFTerminal hierarchy disk guard could not measure `{}`: {err}",
            repository_root.display()
        )
    })?;
    let estimated_peak_bytes = source_bytes
        .saturating_add(source_bytes / 10)
        .max(MINIMUM_WORKTREE_ESTIMATE_BYTES);
    let free_bytes = available_space_bytes(&repository_root).map_err(|err| {
        format!(
            "PFTerminal hierarchy disk guard could not measure free space at `{}`: {err}",
            repository_root.display()
        )
    })?;
    let projected_free_bytes = free_bytes.saturating_sub(estimated_peak_bytes);
    if projected_free_bytes < HIERARCHY_WORKTREE_RESERVE_BYTES {
        return Err(format!(
            "PFTerminal hierarchy disk guard rejected `git worktree add`: free={:.1} GiB, measured repository={:.1} GiB, conservative peak={:.1} GiB, projected free={:.1} GiB, required reserve=60.0 GiB. Reclaim existing temporary worktrees or reuse an existing checkout; do not retry unchanged.",
            gib(free_bytes),
            gib(source_bytes),
            gib(estimated_peak_bytes),
            gib(projected_free_bytes),
        ));
    }
    Ok(())
}

fn git_repository_root(cwd: &Path) -> Result<PathBuf, String> {
    let output = Command::new("git")
        .arg("-C")
        .arg(cwd)
        .args(["rev-parse", "--show-toplevel"])
        .output()
        .map_err(|err| format!("could not start git: {err}"))?;
    if !output.status.success() {
        return Err(format!(
            "could not resolve repository root from `{}`: {}",
            cwd.display(),
            String::from_utf8_lossy(&output.stderr).trim()
        ));
    }
    let root = String::from_utf8(output.stdout)
        .map_err(|err| format!("git returned a non-UTF-8 repository root: {err}"))?;
    Ok(PathBuf::from(root.trim()))
}

fn measure_working_tree_bytes(root: &Path) -> io::Result<u64> {
    let mut total = 0_u64;
    let mut pending = vec![root.to_path_buf()];
    while let Some(directory) = pending.pop() {
        for entry in std::fs::read_dir(&directory)? {
            let entry = entry?;
            if directory == root && entry.file_name() == OsStr::new(".git") {
                continue;
            }
            let metadata = entry.path().symlink_metadata()?;
            if metadata.is_dir() {
                pending.push(entry.path());
            } else if metadata.is_file() {
                total = total.saturating_add(metadata.len());
            }
        }
    }
    Ok(total)
}

#[cfg(unix)]
fn available_space_bytes(path: &Path) -> io::Result<u64> {
    use std::ffi::CString;
    use std::mem::MaybeUninit;
    use std::os::unix::ffi::OsStrExt;

    let path = CString::new(path.as_os_str().as_bytes())
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "path contains NUL"))?;
    let mut stats = MaybeUninit::<libc::statvfs>::uninit();
    // SAFETY: `path` is NUL-terminated and `stats` points to writable storage for statvfs.
    if unsafe { libc::statvfs(path.as_ptr(), stats.as_mut_ptr()) } != 0 {
        return Err(io::Error::last_os_error());
    }
    // SAFETY: statvfs returned success and initialized `stats`.
    let stats = unsafe { stats.assume_init() };
    Ok(stats.f_bavail.saturating_mul(stats.f_frsize))
}

#[cfg(windows)]
fn available_space_bytes(path: &Path) -> io::Result<u64> {
    use std::os::windows::ffi::OsStrExt;

    #[link(name = "kernel32")]
    unsafe extern "system" {
        fn GetDiskFreeSpaceExW(
            directory_name: *const u16,
            free_bytes_available: *mut u64,
            total_number_of_bytes: *mut u64,
            total_number_of_free_bytes: *mut u64,
        ) -> i32;
    }

    let wide = path
        .as_os_str()
        .encode_wide()
        .chain(std::iter::once(0))
        .collect::<Vec<_>>();
    let mut available = 0_u64;
    // SAFETY: `wide` is NUL-terminated and `available` is valid writable storage.
    let success = unsafe {
        GetDiskFreeSpaceExW(
            wide.as_ptr(),
            &mut available,
            std::ptr::null_mut(),
            std::ptr::null_mut(),
        )
    };
    if success == 0 {
        return Err(io::Error::last_os_error());
    }
    Ok(available)
}

fn gib(bytes: u64) -> f64 {
    bytes as f64 / (1024 * 1024 * 1024) as f64
}

#[cfg(test)]
#[path = "disk_safety_tests.rs"]
mod tests;
