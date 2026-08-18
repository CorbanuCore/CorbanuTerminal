use codex_utils_absolute_path::AbsolutePathBuf;
use dirs::home_dir;
use std::path::PathBuf;

const DEFAULT_HOME_DIR: &str = ".corbanu";
const LEGACY_HOME_DIR: &str = ".pfterminal";

/// Returns the path to the Codex configuration directory, which can be specified by the
/// `CORBANU_HOME`, `PFTERMINAL_HOME`, or `CODEX_HOME` environment variable.
/// Existing PFTerminal state is used in place; fresh installations default to
/// `~/.corbanu`.
///
/// - If an explicit home variable is set, the value must exist and be a directory. The
///   value will be canonicalized and this function will Err otherwise.
/// - If no explicit home is set, this function does not verify that the
///   directory exists.
pub fn find_codex_home() -> std::io::Result<AbsolutePathBuf> {
    let corbanu_home_env = std::env::var("CORBANU_HOME")
        .ok()
        .filter(|val| !val.is_empty());
    let pfterminal_home_env = std::env::var("PFTERMINAL_HOME")
        .ok()
        .filter(|val| !val.is_empty());
    let codex_home_env = std::env::var("CODEX_HOME")
        .ok()
        .filter(|val| !val.is_empty());
    find_codex_home_from_env(
        corbanu_home_env.as_deref(),
        pfterminal_home_env.as_deref(),
        codex_home_env.as_deref(),
        home_dir(),
    )
}

fn find_codex_home_from_env(
    corbanu_home_env: Option<&str>,
    pfterminal_home_env: Option<&str>,
    codex_home_env: Option<&str>,
    user_home: Option<PathBuf>,
) -> std::io::Result<AbsolutePathBuf> {
    let home_override = corbanu_home_env
        .map(|value| ("CORBANU_HOME", value))
        .or_else(|| pfterminal_home_env.map(|value| ("PFTERMINAL_HOME", value)))
        .or_else(|| codex_home_env.map(|value| ("CODEX_HOME", value)));

    match home_override {
        Some(home_override) => resolve_home_override(home_override),
        None => resolve_default_home(user_home),
    }
}

fn resolve_home_override((variable, value): (&str, &str)) -> std::io::Result<AbsolutePathBuf> {
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
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            format!("{variable} points to {value:?}, but that path is not a directory"),
        ));
    }

    let canonical = path.canonicalize().map_err(|err| {
        std::io::Error::new(
            err.kind(),
            format!("failed to canonicalize {variable} {value:?}: {err}"),
        )
    })?;
    AbsolutePathBuf::from_absolute_path(canonical)
}

fn resolve_default_home(user_home: Option<PathBuf>) -> std::io::Result<AbsolutePathBuf> {
    let user_home = user_home.ok_or_else(|| {
        std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "Could not find home directory",
        )
    })?;
    let corbanu_home = user_home.join(DEFAULT_HOME_DIR);
    let legacy_home = user_home.join(LEGACY_HOME_DIR);
    let selected = match (corbanu_home.is_dir(), legacy_home.is_dir()) {
        (true, true) => corbanu_home,
        (false, true) => legacy_home,
        (true, false) | (false, false) => corbanu_home,
    };
    AbsolutePathBuf::from_absolute_path(selected)
}

#[cfg(test)]
mod tests {
    use super::DEFAULT_HOME_DIR;
    use super::LEGACY_HOME_DIR;
    use super::find_codex_home_from_env;
    use codex_utils_absolute_path::AbsolutePathBuf;
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

        let err = find_codex_home_from_env(
            /*corbanu_home_env*/ None,
            /*pfterminal_home_env*/ None,
            Some(missing_str),
            /*user_home*/ None,
        )
        .expect_err("missing CODEX_HOME");
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

        let err = find_codex_home_from_env(
            /*corbanu_home_env*/ None,
            /*pfterminal_home_env*/ None,
            Some(file_str),
            /*user_home*/ None,
        )
        .expect_err("file CODEX_HOME");
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

        let resolved = find_codex_home_from_env(
            /*corbanu_home_env*/ None,
            /*pfterminal_home_env*/ None,
            Some(temp_str),
            /*user_home*/ None,
        )
        .expect("valid CODEX_HOME");
        let expected = temp_home
            .path()
            .canonicalize()
            .expect("canonicalize temp home");
        let expected = AbsolutePathBuf::from_absolute_path(expected).expect("absolute home");
        assert_eq!(resolved, expected);
    }

    #[test]
    fn find_codex_home_without_env_uses_default_home_dir() {
        let user_home = TempDir::new().expect("user home");
        let resolved =
            find_codex_home_from_env(None, None, None, Some(user_home.path().to_path_buf()))
                .expect("default Corbanu home");
        let expected = user_home.path().join(DEFAULT_HOME_DIR);
        let expected = AbsolutePathBuf::from_absolute_path(expected).expect("absolute home");
        assert_eq!(resolved, expected);
    }

    #[test]
    fn pfterminal_home_beats_codex_home() {
        let pfterminal_home = TempDir::new().expect("PFTerminal home");
        let codex_home = TempDir::new().expect("Codex home");
        let resolved = find_codex_home_from_env(
            None,
            pfterminal_home.path().to_str(),
            codex_home.path().to_str(),
            None,
        )
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

    #[test]
    fn corbanu_home_beats_legacy_and_codex_overrides() {
        let corbanu_home = TempDir::new().expect("Corbanu home");
        let pfterminal_home = TempDir::new().expect("PFTerminal home");
        let codex_home = TempDir::new().expect("Codex home");
        let resolved = find_codex_home_from_env(
            corbanu_home.path().to_str(),
            pfterminal_home.path().to_str(),
            codex_home.path().to_str(),
            None,
        )
        .expect("resolve Corbanu home");
        let expected = AbsolutePathBuf::from_absolute_path(
            corbanu_home.path().canonicalize().expect("canonical home"),
        )
        .expect("absolute home");
        assert_eq!(resolved, expected);
    }

    #[test]
    fn existing_legacy_home_is_used_in_place() {
        let user_home = TempDir::new().expect("user home");
        let legacy_home = user_home.path().join(LEGACY_HOME_DIR);
        fs::create_dir(&legacy_home).expect("legacy home");

        let resolved =
            find_codex_home_from_env(None, None, None, Some(user_home.path().to_path_buf()))
                .expect("resolve legacy home");
        assert_eq!(resolved.as_path(), legacy_home);
    }

    #[test]
    fn new_home_wins_when_both_homes_exist_without_modifying_legacy() {
        let user_home = TempDir::new().expect("user home");
        let corbanu_home = user_home.path().join(DEFAULT_HOME_DIR);
        let legacy_home = user_home.path().join(LEGACY_HOME_DIR);
        fs::create_dir(&corbanu_home).expect("Corbanu home");
        fs::create_dir(&legacy_home).expect("legacy home");
        let sentinel = legacy_home.join("vault-sentinel");
        fs::write(&sentinel, "preserve").expect("legacy sentinel");

        let resolved =
            find_codex_home_from_env(None, None, None, Some(user_home.path().to_path_buf()))
                .expect("resolve Corbanu home");
        assert_eq!(resolved.as_path(), corbanu_home);
        assert_eq!(fs::read_to_string(sentinel).expect("sentinel"), "preserve");
    }
}
