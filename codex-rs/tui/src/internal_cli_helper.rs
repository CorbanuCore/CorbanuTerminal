use std::ffi::OsStr;
use std::path::Path;
use std::path::PathBuf;

use anyhow::Result;
use anyhow::anyhow;

const INTERNAL_CLI_NAMES: [&str; 3] = ["corbanu", "pfterminal", "codex"];

/// Resolves the multitool CLI that owns Corbanu's hidden helper commands.
///
/// Normal branded launches already run that CLI, so the current executable is
/// authoritative. The separately built `codex-tui` entrypoint needs a sibling
/// or PATH-installed CLI instead; invoking itself would parse the hidden helper
/// name as ordinary TUI input and eventually time out.
pub(crate) fn internal_cli_helper_executable() -> Result<PathBuf> {
    let current_executable = std::env::current_exe()
        .map_err(|error| anyhow!("failed to locate the running Corbanu executable: {error}"))?;
    let path_directories = std::env::var_os("PATH")
        .map(|path| std::env::split_paths(&path).collect::<Vec<_>>())
        .unwrap_or_default();
    resolve_internal_cli_helper(&current_executable, &path_directories, Path::is_file)
}

fn resolve_internal_cli_helper(
    current_executable: &Path,
    path_directories: &[PathBuf],
    is_file: impl Fn(&Path) -> bool,
) -> Result<PathBuf> {
    if !is_standalone_tui_executable(current_executable) {
        return Ok(current_executable.to_path_buf());
    }

    let sibling_directory = current_executable.parent();
    for name in INTERNAL_CLI_NAMES {
        let file_name = format!("{name}{}", std::env::consts::EXE_SUFFIX);
        if let Some(directory) = sibling_directory {
            let candidate = directory.join(&file_name);
            if is_file(&candidate) {
                return Ok(candidate);
            }
        }
        for directory in path_directories {
            let candidate = directory.join(&file_name);
            if is_file(&candidate) {
                return Ok(candidate);
            }
        }
    }

    Err(anyhow!(
        "the standalone codex-tui executable could not locate corbanu, pfterminal, or codex to run its internal authentication helper"
    ))
}

fn is_standalone_tui_executable(executable: &Path) -> bool {
    executable.file_stem() == Some(OsStr::new("codex-tui"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn multitool_launch_uses_current_executable() {
        let executable = Path::new("/opt/corbanu/bin/corbanu");
        assert_eq!(
            resolve_internal_cli_helper(executable, &[], |_| false).expect("helper"),
            executable
        );
    }

    #[test]
    fn standalone_tui_prefers_sibling_corbanu() {
        let executable = Path::new("/opt/corbanu/bin/codex-tui");
        let sibling =
            Path::new("/opt/corbanu/bin").join(format!("corbanu{}", std::env::consts::EXE_SUFFIX));
        assert_eq!(
            resolve_internal_cli_helper(executable, &[], |candidate| candidate == sibling)
                .expect("helper"),
            sibling
        );
    }

    #[test]
    fn standalone_tui_finds_path_installed_corbanu() {
        let executable = Path::new("/tmp/codex-tui");
        let path_directory = PathBuf::from("/opt/corbanu/bin");
        let installed = path_directory.join(format!("corbanu{}", std::env::consts::EXE_SUFFIX));
        assert_eq!(
            resolve_internal_cli_helper(executable, &[path_directory], |candidate| {
                candidate == installed
            })
            .expect("helper"),
            installed
        );
    }

    #[test]
    fn standalone_tui_without_multitool_fails_before_authentication() {
        let error = resolve_internal_cli_helper(Path::new("/tmp/codex-tui"), &[], |_| false)
            .expect_err("missing helper must fail");
        assert!(error.to_string().contains("could not locate"));
    }
}
