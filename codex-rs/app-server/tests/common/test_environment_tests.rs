use super::isolate_profile;
use pretty_assertions::assert_eq;
use std::collections::BTreeMap;
use std::ffi::OsString;
use std::process::Command;

#[test]
fn fixture_profile_overrides_host_aliases_and_guard_removal() {
    let home = tempfile::tempdir().expect("fixture home");
    let mut cmd = Command::new("unused-test-program");
    for name in ["CORBANU_HOME", "PFTERMINAL_HOME", "CODEX_HOME"] {
        cmd.env(name, "host-profile-canary");
    }
    cmd.env_remove("CORBANU_TEST_NO_NATIVE_KEYRING");
    cmd.env("FIXTURE_VALUE", "synthetic");
    isolate_profile(&mut cmd, home.path());
    let actual: BTreeMap<_, _> = cmd
        .get_envs()
        .map(|(key, value)| (key.to_owned(), value.map(OsString::from)))
        .collect();
    let expected = BTreeMap::from([
        (OsString::from("CORBANU_HOME"), Some(home.path().into())),
        (OsString::from("PFTERMINAL_HOME"), Some(home.path().into())),
        (OsString::from("CODEX_HOME"), Some(home.path().into())),
        (
            OsString::from("CORBANU_TEST_NO_NATIVE_KEYRING"),
            Some("1".into()),
        ),
        (OsString::from("FIXTURE_VALUE"), Some("synthetic".into())),
    ]);
    assert_eq!(actual, expected);
}

#[test]
fn fixture_profile_is_explicit_even_without_caller_overrides() {
    let home = tempfile::tempdir().expect("fixture home");
    let mut cmd = Command::new("unused-test-program");
    isolate_profile(&mut cmd, home.path());
    let overrides: Vec<_> = cmd.get_envs().collect();
    assert_eq!(overrides.len(), 4);
    assert!(overrides.iter().all(|(_, value)| value.is_some()));
}
