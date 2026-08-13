use anyhow::Context;
use std::path::PathBuf;

const CORBANU_DEBUG_HOME_ENV: &str = "CORBANU_DEBUG_HOME";
const LEGACY_DEBUG_HOME_ENV: &str = "PFTERMINAL_DEBUG_HOME";

pub(crate) fn configure_for_current_process() -> anyhow::Result<()> {
    let Some(arg0) = std::env::args_os().next() else {
        return Ok(());
    };
    let Some(entrypoint) = entrypoint_from_argv0(&arg0) else {
        return Ok(());
    };
    configure_for_entrypoint(&entrypoint)
}

pub(crate) fn current_entrypoint_is_corbanu() -> bool {
    std::env::args_os()
        .next()
        .and_then(|arg0| entrypoint_from_argv0(&arg0))
        .is_some_and(|entrypoint| entrypoint == "corbanu" || entrypoint == "corbanu-debug")
}

pub(crate) fn configure_for_entrypoint(entrypoint: &str) -> anyhow::Result<()> {
    let home = match entrypoint {
        "pfterminal" | "corbanu" => return Ok(()),
        "pfterminal-debug" | "corbanu-debug" => resolve_home(
            std::env::var_os(CORBANU_DEBUG_HOME_ENV).map(PathBuf::from),
            std::env::var_os(LEGACY_DEBUG_HOME_ENV).map(PathBuf::from),
            std::env::var_os("CODEX_HOME").map(PathBuf::from),
            dirs::home_dir(),
        ),
        _ => return Ok(()),
    }
    .with_context(|| format!("could not resolve the isolated home for {entrypoint}"))?;

    // This runs at process entry, before the async runtime or any worker threads exist.
    // This process is specifically the isolated debug entrypoint. Stable-home
    // variables must not outrank its selected debug home when the shared home
    // resolver runs later.
    unsafe {
        std::env::remove_var("CORBANU_HOME");
        std::env::remove_var("PFTERMINAL_HOME");
        std::env::set_var("CODEX_HOME", home);
    }
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
    corbanu_override: Option<PathBuf>,
    legacy_override: Option<PathBuf>,
    codex_override: Option<PathBuf>,
    user_home: Option<PathBuf>,
) -> Option<PathBuf> {
    corbanu_override
        .or(legacy_override)
        .or(codex_override)
        .or_else(|| {
            user_home.map(|home| {
                let corbanu_home = home.join(".corbanu-debug");
                let legacy_home = home.join(".pfterminal-debug");
                match (corbanu_home.is_dir(), legacy_home.is_dir()) {
                    (true, true) => {
                        eprintln!(
                            "Both {} and {} exist; using {} without merging or deleting either home.",
                            corbanu_home.display(),
                            legacy_home.display(),
                            corbanu_home.display()
                        );
                        corbanu_home
                    }
                    (false, true) => legacy_home,
                    (true, false) | (false, false) => corbanu_home,
                }
            })
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn debug_default_is_distinct_from_stock_codex() {
        let user_home = tempfile::TempDir::new().expect("user home");
        let debug = resolve_home(
            /*corbanu_override*/ None,
            /*legacy_override*/ None,
            /*codex_override*/ None,
            Some(user_home.path().to_path_buf()),
        )
        .unwrap();

        assert_eq!(debug, user_home.path().join(".corbanu-debug"));
        assert_ne!(debug, user_home.path().join(".codex"));
    }

    #[test]
    fn corbanu_debug_home_beats_legacy_and_codex_overrides() {
        let user_home = PathBuf::from("/home/tester");
        let corbanu_override = PathBuf::from("/tmp/corbanu-debug");
        let debug_override = PathBuf::from("/tmp/pf-debug");
        let codex_override = PathBuf::from("/tmp/codex");

        assert_eq!(
            resolve_home(
                Some(corbanu_override.clone()),
                Some(debug_override),
                Some(codex_override),
                Some(user_home),
            ),
            Some(corbanu_override),
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
        assert_eq!(
            entrypoint_from_argv0(std::ffi::OsStr::new(r"C:\Tools\corbanu-debug.exe")),
            Some("corbanu-debug".to_string())
        );
    }
}
