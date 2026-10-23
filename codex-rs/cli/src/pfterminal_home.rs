use anyhow::Context;
use std::ffi::OsString;
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
    if !matches!(entrypoint, "pfterminal-debug" | "corbanu-debug") {
        return Ok(());
    }
    let var = |name: &str| std::env::var_os(name).filter(|value| !value.is_empty());
    let variables = [
        (CORBANU_DEBUG_HOME_ENV, var(CORBANU_DEBUG_HOME_ENV)),
        (LEGACY_DEBUG_HOME_ENV, var(LEGACY_DEBUG_HOME_ENV)),
        ("CORBANU_HOME", var("CORBANU_HOME")),
        ("PFTERMINAL_HOME", var("PFTERMINAL_HOME")),
        ("CODEX_HOME", var("CODEX_HOME")),
    ];
    let Some(home) = debug_home(&variables, dirs::home_dir())
        .with_context(|| format!("could not resolve the isolated home for {entrypoint}"))?
    else {
        // A caller-set CORBANU_HOME/PFTERMINAL_HOME/CODEX_HOME is used as
        // `corbanu` uses it (#418); the shared resolver reports conflicts.
        return Ok(());
    };
    let lossy = variables.each_ref().map(|(name, value)| {
        (
            *name,
            value
                .as_ref()
                .map(|value| value.to_string_lossy().into_owned()),
        )
    });
    let borrowed = lossy
        .each_ref()
        .map(|(name, value)| (*name, value.as_deref()));
    if let Some(warning) = codex_core::config::home_variables_conflict(&borrowed) {
        #[allow(clippy::print_stderr)]
        {
            eprintln!("{warning}");
        }
    }

    // This runs at process entry, before the async runtime or any worker threads exist.
    // A debug home (chosen or default) must not be outranked by stable-home
    // variables when the shared home resolver runs later.
    unsafe {
        std::env::remove_var("CORBANU_HOME");
        std::env::remove_var("PFTERMINAL_HOME");
        std::env::set_var("CODEX_HOME", home);
    }
    Ok(())
}

/// The debug entrypoint's home: `CORBANU_DEBUG_HOME`, then
/// `PFTERMINAL_DEBUG_HOME`, else `Ok(None)` when a stable-home variable is
/// set (it is kept, like the release `corbanu`), else the default debug home.
fn debug_home(
    variables: &[(&str, Option<OsString>); 5],
    user_home: Option<PathBuf>,
) -> Option<Option<PathBuf>> {
    let [
        (_, corbanu_debug),
        (_, legacy_debug),
        (_, corbanu),
        (_, pfterminal),
        (_, codex),
    ] = variables;
    if corbanu_debug.is_none()
        && legacy_debug.is_none()
        && (corbanu.is_some() || pfterminal.is_some() || codex.is_some())
    {
        return Some(None);
    }
    resolve_home(
        corbanu_debug.as_ref().map(PathBuf::from),
        legacy_debug.as_ref().map(PathBuf::from),
        /*codex_override*/ None,
        user_home,
    )
    .map(Some)
}

/// Exit when the Corbanu home cannot be read (#616).
///
/// Denials (macOS privacy protection for external volumes, a drive being
/// remounted, a sandbox) surface as EPERM from arbitrary later reads, which
/// reads as an unexplained "Operation not permitted". Only the `corbanu`
/// entrypoints are checked, and not when argv[1] selects a private helper
/// that runs sandboxed ([`codex_arg0::is_private_helper_argv1`]: the file
/// helper, the arg0 exec helper, apply-patch): helper argv0 aliases and those
/// helpers inherit a sandbox that may deny the home on purpose (#847, #858),
/// so they skip the check and record nothing. Unless this process's own
/// sandbox denies the read, the exit status tells a waiting parent
/// (`corbanu exec` running this as its auth helper) that the failure may be
/// temporary.
pub(crate) fn exit_if_home_unreadable() {
    #[cfg(unix)]
    {
        if !current_entrypoint_is_corbanu()
            || std::env::args_os()
                .nth(1)
                .is_some_and(|argv1| codex_arg0::is_private_helper_argv1(&argv1))
        {
            return;
        }
        let Ok(home) = codex_core::config::find_codex_home() else {
            return;
        };
        if let Err(unreadable) = check_home_readable(
            home.as_path(),
            |path| std::fs::read_dir(path).map(drop),
            codex_core::home_access::sandbox_denies_read,
            codex_core::home_access::record_home_access_failure,
        ) {
            #[allow(clippy::print_stderr)]
            {
                eprintln!("{}", unreadable.message);
            }
            std::process::exit(unreadable.exit_code);
        }
    }
}

#[cfg(unix)]
#[derive(Debug, PartialEq, Eq)]
struct HomeUnreadable {
    message: String,
    exit_code: i32,
}

/// The report for a home that `read_dir` cannot read with EPERM, after
/// `record` has written the diagnostics; other outcomes pass.
#[cfg(unix)]
fn check_home_readable(
    home: &std::path::Path,
    read_dir: impl FnOnce(&std::path::Path) -> std::io::Result<()>,
    sandbox_denies_read: impl FnOnce(&std::path::Path) -> bool,
    record: impl FnOnce(&str, &std::path::Path, &str) -> Option<PathBuf>,
) -> Result<(), HomeUnreadable> {
    const EPERM: i32 = 1;
    let Err(error) = read_dir(home) else {
        return Ok(());
    };
    if error.raw_os_error() != Some(EPERM) {
        return Ok(());
    }
    let error = error.to_string();
    let log = record("read_dir (opendir)", home, &error);
    let mut message =
        codex_core::home_access::unreadable_home_message(home, &error, log.as_deref());
    // Waiting cannot lift this process's own sandbox: fail as before.
    let exit_code = if sandbox_denies_read(home) {
        message.push_str("\nThis process's sandbox denies reading it.");
        1
    } else {
        codex_protocol::home_unreadable::HOME_UNREADABLE_EXIT_CODE
    };
    Err(HomeUnreadable { message, exit_code })
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
                    (true, true) => corbanu_home,
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

    fn variables(
        corbanu_debug: Option<&str>,
        corbanu: Option<&str>,
        codex: Option<&str>,
    ) -> [(&'static str, Option<OsString>); 5] {
        [
            (CORBANU_DEBUG_HOME_ENV, corbanu_debug.map(OsString::from)),
            (LEGACY_DEBUG_HOME_ENV, None),
            ("CORBANU_HOME", corbanu.map(OsString::from)),
            ("PFTERMINAL_HOME", None),
            ("CODEX_HOME", codex.map(OsString::from)),
        ]
    }

    /// #418: `corbanu-debug` keeps a caller-set `CORBANU_HOME` (or
    /// `CODEX_HOME`) like `corbanu`, instead of replacing it with the debug home.
    #[test]
    fn debug_entrypoint_keeps_a_caller_set_stable_home() {
        let user_home = Some(PathBuf::from("/home/tester"));
        for vars in [
            variables(
                /*corbanu_debug*/ None,
                Some("/homes/b"),
                /*codex*/ None,
            ),
            variables(
                /*corbanu_debug*/ None,
                Some("/homes/b"),
                Some("/homes/c"),
            ),
            variables(
                /*corbanu_debug*/ None,
                /*corbanu*/ None,
                Some("/homes/c"),
            ),
        ] {
            assert_eq!(debug_home(&vars, user_home.clone()), Some(None));
        }
        assert_eq!(
            debug_home(
                &variables(
                    /*corbanu_debug*/ None, /*corbanu*/ None, /*codex*/ None
                ),
                user_home
            ),
            Some(Some(PathBuf::from("/home/tester/.corbanu-debug")))
        );
    }

    /// #418: a debug home outranks `CORBANU_HOME`, and the conflict is reported.
    #[test]
    fn debug_home_outranks_corbanu_home_with_one_warning() {
        let vars = variables(Some("/homes/d"), Some("/homes/b"), /*codex*/ None);
        assert_eq!(
            debug_home(&vars, Some(PathBuf::from("/home/tester"))),
            Some(Some(PathBuf::from("/homes/d")))
        );
        let lossy = vars
            .each_ref()
            .map(|(name, value)| (*name, value.as_ref().and_then(|value| value.to_str())));
        assert_eq!(
            codex_core::config::home_variables_conflict(&lossy).as_deref(),
            Some(
                "warning: CORBANU_DEBUG_HOME (/homes/d) overrides CORBANU_HOME (/homes/b); \
                 using /homes/d. To use another home, set CORBANU_DEBUG_HOME to it."
            )
        );
    }

    /// #616: EPERM from the home is recorded and reported with its possible
    /// causes, as temporary unless this process's sandbox denies the read;
    /// other outcomes pass the check.
    #[cfg(unix)]
    #[test]
    fn an_unreadable_home_is_recorded_and_reported() {
        let home = std::path::Path::new("/Volumes/Drive/home");
        let eperm = |_: &std::path::Path| Err(std::io::Error::from_raw_os_error(1));
        let mut recorded = Vec::new();
        let unreadable = check_home_readable(
            home,
            eperm,
            |_| false,
            |operation, path, error| {
                recorded.push((operation.to_string(), path.to_path_buf(), error.to_string()));
                Some(PathBuf::from("/logs/home-access.log"))
            },
        )
        .expect_err("EPERM is reported");
        pretty_assertions::assert_eq!(
            recorded,
            vec![(
                "read_dir (opendir)".to_string(),
                home.to_path_buf(),
                "Operation not permitted (os error 1)".to_string()
            )]
        );
        assert_eq!(unreadable.exit_code, 75);
        let message = &unreadable.message;
        assert!(
            message.starts_with(
                "Corbanu could not read its home (Operation not permitted): /Volumes/Drive/home"
            ),
            "{message}"
        );
        assert!(
            message.ends_with("Details: /logs/home-access.log"),
            "{message}"
        );

        let sandboxed = check_home_readable(home, eperm, |_| true, |_, _, _| None)
            .expect_err("EPERM is reported");
        assert_eq!(sandboxed.exit_code, 1, "waiting cannot lift a sandbox");
        assert!(
            sandboxed
                .message
                .ends_with("This process's sandbox denies reading it."),
            "{}",
            sandboxed.message
        );

        let not_recorded = |_: &str, _: &std::path::Path, _: &str| -> Option<PathBuf> {
            panic!("only EPERM is recorded")
        };
        let missing = std::io::Error::from(std::io::ErrorKind::NotFound);
        assert_eq!(
            check_home_readable(home, |_| Err(missing), |_| false, not_recorded),
            Ok(())
        );
        assert_eq!(
            check_home_readable(home, |_| Ok(()), |_| false, not_recorded),
            Ok(())
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
