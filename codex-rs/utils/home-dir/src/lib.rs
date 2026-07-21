use codex_utils_absolute_path::AbsolutePathBuf;
use dirs::home_dir;
use std::path::PathBuf;

#[cfg(feature = "upstream-home")]
const DEFAULT_HOME_DIR: &str = ".codex";
#[cfg(not(feature = "upstream-home"))]
const DEFAULT_HOME_DIR: &str = ".pfterminal";

/// Returns the path to the Codex configuration directory, which can be specified by the
/// `PFTERMINAL_HOME` or `CODEX_HOME` environment variable. PFTerminal defaults to
/// `~/.pfterminal` regardless of the executable's name.
///
/// - If `CODEX_HOME` is set, the value must exist and be a directory. The
///   value will be canonicalized and this function will Err otherwise.
/// - If `CODEX_HOME` is not set, this function does not verify that the
///   directory exists.
pub fn find_codex_home() -> std::io::Result<AbsolutePathBuf> {
    let pfterminal_home_env = std::env::var("PFTERMINAL_HOME")
        .ok()
        .filter(|val| !val.is_empty());
    let codex_home_env = std::env::var("CODEX_HOME")
        .ok()
        .filter(|val| !val.is_empty());
    find_codex_home_from_env(pfterminal_home_env.as_deref(), codex_home_env.as_deref())
}

fn find_codex_home_from_env(
    pfterminal_home_env: Option<&str>,
    codex_home_env: Option<&str>,
) -> std::io::Result<AbsolutePathBuf> {
    resolve_home_override(
        pfterminal_home_env
            .map(|value| ("PFTERMINAL_HOME", value))
            .or_else(|| codex_home_env.map(|value| ("CODEX_HOME", value))),
    )
}

fn resolve_home_override(home_override: Option<(&str, &str)>) -> std::io::Result<AbsolutePathBuf> {
    match home_override {
        Some((variable, value)) => {
            let path = PathBuf::from(value);
            let metadata = std::fs::metadata(&path).map_err(|err| match err.kind() {
                std::io::ErrorKind::NotFound => std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    format!("{variable} points to {value:?}, but that path does not exist"),
                ),
                _ => std::io::Error::new(
                    err.kind(),
                    format!("failed to read {variable} {value:?}: {err}"),
                ),
            })?;

            if !metadata.is_dir() {
                Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    format!("{variable} points to {value:?}, but that path is not a directory"),
                ))
            } else {
                let canonical = path.canonicalize().map_err(|err| {
                    std::io::Error::new(
                        err.kind(),
                        format!("failed to canonicalize {variable} {value:?}: {err}"),
                    )
                })?;
                AbsolutePathBuf::from_absolute_path(canonical)
            }
        }
        None => {
            let mut p = home_dir().ok_or_else(|| {
                std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    "Could not find home directory",
                )
            })?;
            p.push(DEFAULT_HOME_DIR);
            AbsolutePathBuf::from_absolute_path(p)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::DEFAULT_HOME_DIR;
    use super::find_codex_home_from_env;
    use codex_utils_absolute_path::AbsolutePathBuf;
    use dirs::home_dir;
    use pretty_assertions::assert_eq;
    use std::fs;
    use std::io::ErrorKind;
    use tempfile::TempDir;

    #[test]
    fn find_codex_home_env_missing_path_is_fatal() {
        let temp_home = TempDir::new().expect("temp home");
        let missing = temp_home.path().join("missing-codex-home");
        let missing_str = missing
            .to_str()
            .expect("missing codex home path should be valid utf-8");

        let err =
            find_codex_home_from_env(None, Some(missing_str)).expect_err("missing CODEX_HOME");
        assert_eq!(err.kind(), ErrorKind::NotFound);
        assert!(
            err.to_string().contains("CODEX_HOME"),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn find_codex_home_env_file_path_is_fatal() {
        let temp_home = TempDir::new().expect("temp home");
        let file_path = temp_home.path().join("codex-home.txt");
        fs::write(&file_path, "not a directory").expect("write temp file");
        let file_str = file_path
            .to_str()
            .expect("file codex home path should be valid utf-8");

        let err = find_codex_home_from_env(None, Some(file_str)).expect_err("file CODEX_HOME");
        assert_eq!(err.kind(), ErrorKind::InvalidInput);
        assert!(
            err.to_string().contains("not a directory"),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn find_codex_home_env_valid_directory_canonicalizes() {
        let temp_home = TempDir::new().expect("temp home");
        let temp_str = temp_home
            .path()
            .to_str()
            .expect("temp codex home path should be valid utf-8");

        let resolved = find_codex_home_from_env(None, Some(temp_str)).expect("valid CODEX_HOME");
        let expected = temp_home
            .path()
            .canonicalize()
            .expect("canonicalize temp home");
        let expected = AbsolutePathBuf::from_absolute_path(expected).expect("absolute home");
        assert_eq!(resolved, expected);
    }

    #[test]
    fn find_codex_home_without_env_uses_default_home_dir() {
        let resolved = find_codex_home_from_env(
            /*pfterminal_home_env*/ None, /*codex_home_env*/ None,
        )
        .expect("default PFTerminal home");
        let mut expected = home_dir().expect("home dir");
        expected.push(DEFAULT_HOME_DIR);
        let expected = AbsolutePathBuf::from_absolute_path(expected).expect("absolute home");
        assert_eq!(resolved, expected);
    }

    #[test]
    fn pfterminal_home_beats_codex_home() {
        let pfterminal_home = TempDir::new().expect("PFTerminal home");
        let codex_home = TempDir::new().expect("Codex home");
        let resolved =
            find_codex_home_from_env(pfterminal_home.path().to_str(), codex_home.path().to_str())
                .expect("resolve preferred home");
        let expected = AbsolutePathBuf::from_absolute_path(
            pfterminal_home
                .path()
                .canonicalize()
                .expect("canonical home"),
        )
        .expect("absolute home");
        assert_eq!(resolved, expected);
    }
}
