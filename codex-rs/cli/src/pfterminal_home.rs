use anyhow::Context;
use std::path::PathBuf;

const STABLE_HOME_ENV: &str = "PFTERMINAL_HOME";
const DEBUG_HOME_ENV: &str = "PFTERMINAL_DEBUG_HOME";

pub(crate) fn configure_for_current_process() -> anyhow::Result<()> {
    let Some(arg0) = std::env::args_os().next() else {
        return Ok(());
    };
    let Some(entrypoint) = entrypoint_from_argv0(&arg0) else {
        return Ok(());
    };
    configure_for_entrypoint(&entrypoint)
}

pub(crate) fn configure_for_entrypoint(entrypoint: &str) -> anyhow::Result<()> {
    let home = match entrypoint {
        "pfterminal" => resolve_home(
            std::env::var_os(STABLE_HOME_ENV).map(PathBuf::from),
            dirs::home_dir(),
            ".pfterminal",
        ),
        "pfterminal-debug" => resolve_home(
            std::env::var_os(DEBUG_HOME_ENV).map(PathBuf::from),
            dirs::home_dir(),
            ".pfterminal-debug",
        ),
        _ => return Ok(()),
    }
    .with_context(|| format!("could not resolve the isolated home for {entrypoint}"))?;

    // This runs at process entry, before the async runtime or any worker threads exist.
    // Restricting CODEX_HOME here prevents PF stable, PF debug, and stock Codex state from
    // sharing databases even when the invoking shell exports a stock CODEX_HOME.
    unsafe { std::env::set_var("CODEX_HOME", home) };
    Ok(())
}

fn entrypoint_from_argv0(arg0: &std::ffi::OsStr) -> Option<String> {
    let arg0 = arg0.to_string_lossy();
    let file_name = arg0
        .rsplit(['/', '\\'])
        .next()
        .filter(|name| !name.is_empty())?;
    Some(
        file_name
            .strip_suffix(".exe")
            .unwrap_or(file_name)
            .to_owned(),
    )
}

fn resolve_home(
    override_home: Option<PathBuf>,
    user_home: Option<PathBuf>,
    leaf: &str,
) -> Option<PathBuf> {
    override_home.or_else(|| user_home.map(|home| home.join(leaf)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stable_and_debug_defaults_are_distinct_from_each_other_and_stock_codex() {
        let user_home = PathBuf::from("/home/tester");
        let stable = resolve_home(
            /*override_home*/ None,
            Some(user_home.clone()),
            ".pfterminal",
        )
        .unwrap();
        let debug = resolve_home(
            /*override_home*/ None,
            Some(user_home.clone()),
            ".pfterminal-debug",
        )
        .unwrap();

        assert_eq!(stable, user_home.join(".pfterminal"));
        assert_eq!(debug, user_home.join(".pfterminal-debug"));
        assert_ne!(stable, debug);
        assert_ne!(stable, user_home.join(".codex"));
        assert_ne!(debug, user_home.join(".codex"));
    }

    #[test]
    fn explicit_pf_home_overrides_only_its_matching_entrypoint() {
        let user_home = PathBuf::from("/home/tester");
        let stable_override = PathBuf::from("/tmp/pf-stable");
        let debug_override = PathBuf::from("/tmp/pf-debug");

        assert_eq!(
            resolve_home(
                Some(stable_override.clone()),
                Some(user_home.clone()),
                ".pfterminal",
            ),
            Some(stable_override),
        );
        assert_eq!(
            resolve_home(
                Some(debug_override.clone()),
                Some(user_home),
                ".pfterminal-debug"
            ),
            Some(debug_override),
        );
    }

    #[test]
    fn runtime_entrypoint_handles_paths_and_windows_executables() {
        assert_eq!(
            entrypoint_from_argv0(std::ffi::OsStr::new("/usr/local/bin/pfterminal")),
            Some("pfterminal".to_string())
        );
        assert_eq!(
            entrypoint_from_argv0(std::ffi::OsStr::new(r"C:\Tools\pfterminal-debug.exe")),
            Some("pfterminal-debug".to_string())
        );
    }
}
