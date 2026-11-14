use super::*;
use pretty_assertions::assert_eq;

#[test]
fn missing_optional_binary_returns_error_instead_of_panicking() {
    let name = "corbanu_missing_optional_helper_regression_1962";
    for key in cargo_bin_env_keys(name) {
        assert!(
            std::env::var_os(key).is_none(),
            "fixture name must be unused"
        );
    }
    assert!(matches!(
        cargo_bin(name),
        Err(CargoBinError::NotFound { .. })
    ));
}

#[test]
fn legacy_lookup_preserves_secondary_fallback_and_target_layouts() {
    let temp = tempfile::tempdir().unwrap();
    let profile = temp.path().join("target-platform").join("debug");
    std::fs::create_dir_all(profile.join("deps")).unwrap();
    let expected = profile.join(format!("secondary{}", std::env::consts::EXE_SUFFIX));
    std::fs::write(&expected, []).unwrap();
    for host in [profile.join("deps/test-host"), profile.join("test-host")] {
        let actual = legacy_cargo_bin("missing", &host)
            .or_else(|_| legacy_cargo_bin("secondary", &host))
            .unwrap();
        assert_eq!(actual, expected);
    }
}

#[test]
fn directories_are_not_legacy_binaries() {
    let temp = tempfile::tempdir().unwrap();
    std::fs::create_dir(
        temp.path()
            .join(format!("helper{}", std::env::consts::EXE_SUFFIX)),
    )
    .unwrap();
    assert!(matches!(
        legacy_cargo_bin("helper", &temp.path().join("test-host")),
        Err(CargoBinError::NotFound { .. })
    ));
}

#[test]
fn explicit_existing_path_remains_authoritative() {
    let path = std::env::current_exe().unwrap();
    assert_eq!(
        resolve_bin_from_env("CARGO_BIN_EXE_fixture", path.clone().into_os_string()).unwrap(),
        path
    );
}

#[test]
fn missing_explicit_path_is_an_error() {
    let temp = tempfile::tempdir().unwrap();
    let path = temp.path().join("missing");
    assert!(matches!(
        resolve_bin_from_env("CARGO_BIN_EXE_fixture", path.into_os_string()),
        Err(CargoBinError::ResolvedPathDoesNotExist { .. })
    ));
}
