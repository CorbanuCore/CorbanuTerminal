use pretty_assertions::assert_eq;

use super::*;

fn home_with(level: ChosenLevel, nested: NestedAgents) -> tempfile::TempDir {
    let home = tempfile::tempdir().unwrap();
    level::save(home.path(), level, nested).unwrap();
    home
}

/// Read-only as the Aggressive profile makes the home for agent commands.
/// `None` when this user can write it anyway (root).
#[cfg(unix)]
fn read_only(home: &Path) -> Option<ReadOnly> {
    use std::os::unix::fs::PermissionsExt;
    std::fs::set_permissions(home, std::fs::Permissions::from_mode(0o555)).unwrap();
    let guard = ReadOnly(home.to_path_buf());
    tempfile::NamedTempFile::new_in(home)
        .is_err()
        .then_some(guard)
}

#[cfg(unix)]
struct ReadOnly(PathBuf);

#[cfg(unix)]
impl Drop for ReadOnly {
    fn drop(&mut self) {
        use std::os::unix::fs::PermissionsExt;
        let _ = std::fs::set_permissions(&self.0, std::fs::Permissions::from_mode(0o755));
    }
}

#[test]
fn decisions_by_kind_and_setting() {
    let origin = PathBuf::from("/h");
    let refused = |kind, nested| match decide("exec", kind, Some((origin.clone(), nested))) {
        NestedLaunch::Refuse(message) => message,
        other => panic!("{other:?}"),
    };
    assert_eq!(
        decide("exec", NestedKind::Agent, None),
        NestedLaunch::NotNested
    );
    assert_eq!(
        decide(
            "exec",
            NestedKind::Agent,
            Some((origin.clone(), NestedAgents::Pass))
        ),
        NestedLaunch::EnforceAggressive(origin.clone())
    );
    let message = refused(NestedKind::Agent, NestedAgents::Refuse);
    assert!(
        message.starts_with("`corbanu exec` was started by an agent command")
            && message.contains("/security"),
        "{message}"
    );
    for nested in [NestedAgents::Refuse, NestedAgents::Pass] {
        assert!(refused(NestedKind::Host, nested).contains("never allowed"));
        assert!(refused(NestedKind::Credentials, nested).contains("credentials"));
    }
    match decide("", NestedKind::Agent, Some((origin, NestedAgents::Refuse))) {
        NestedLaunch::Refuse(message) => assert!(message.starts_with("`corbanu` was"), "{message}"),
        other => panic!("{other:?}"),
    }
}

#[test]
fn marker_counts_only_when_its_home_stores_aggressive() {
    let aggressive = home_with(ChosenLevel::Aggressive, NestedAgents::Pass);
    let permissive = home_with(ChosenLevel::Permissive, NestedAgents::Refuse);
    let empty = tempfile::tempdir().unwrap();
    assert_eq!(
        aggressive_origin(Some(aggressive.path().to_path_buf()), Vec::new()),
        Some((aggressive.path().to_path_buf(), NestedAgents::Pass))
    );
    for forged in [permissive.path(), empty.path()] {
        assert_eq!(
            aggressive_origin(Some(forged.to_path_buf()), Vec::new()),
            None
        );
    }
}

/// A person's own launch: the home stores Aggressive but is writable.
#[test]
fn writable_aggressive_home_is_not_nested() {
    let home = home_with(ChosenLevel::Aggressive, NestedAgents::Refuse);
    assert_eq!(
        aggressive_origin(None, vec![home.path().to_path_buf()]),
        None
    );
    assert_eq!(std::fs::read_dir(home.path()).unwrap().count(), 2);
}

/// Dropping the marker or pointing `CODEX_HOME` elsewhere does not help: the
/// real home is still read-only to the command.
#[cfg(unix)]
#[test]
fn unwritable_aggressive_home_is_nested_without_the_marker() {
    let real = home_with(ChosenLevel::Aggressive, NestedAgents::Refuse);
    let fake = tempfile::tempdir().unwrap();
    let Some(_guard) = read_only(real.path()) else {
        return;
    };
    assert_eq!(
        aggressive_origin(
            Some(fake.path().to_path_buf()),
            vec![fake.path().to_path_buf(), real.path().to_path_buf()]
        ),
        Some((real.path().to_path_buf(), NestedAgents::Refuse))
    );
}

#[test]
fn exec_overrides_write_the_rule_and_carry_the_marker() {
    let origin = home_with(ChosenLevel::Aggressive, NestedAgents::Pass);
    let child = tempfile::tempdir().unwrap();
    let overrides = aggressive_cli_overrides(child.path(), origin.path()).unwrap();
    assert_eq!(
        std::fs::read_to_string(level::rules_path(child.path())).unwrap(),
        level::rules_contents()
    );
    assert!(overrides.contains(&(
        format!("shell_environment_policy.set.{ORIGIN_ENV}"),
        toml::Value::String(origin.path().to_string_lossy().into_owned()),
    )));
}
