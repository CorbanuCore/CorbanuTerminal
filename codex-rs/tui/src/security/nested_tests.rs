use std::path::PathBuf;

use pretty_assertions::assert_eq;

use super::level::NestedAgents;
use super::*;

fn home_with(level: ChosenLevel, nested: NestedAgents) -> tempfile::TempDir {
    let home = tempfile::tempdir().unwrap();
    level::save(home.path(), level, nested).unwrap();
    home
}

/// What the Aggressive profile does to a home for agent commands: read-only,
/// vault store unreadable (`vault_readable` keeps it readable, as a
/// Permissive sandbox does). `None` when this user can write it anyway (root).
#[cfg(unix)]
fn sandbox_away(home: &Path, vault_readable: bool) -> Option<Restore> {
    use std::os::unix::fs::PermissionsExt;
    let secrets = home.join("secrets");
    std::fs::create_dir_all(&secrets).unwrap();
    let vault_mode = if vault_readable { 0o755 } else { 0o000 };
    std::fs::set_permissions(&secrets, std::fs::Permissions::from_mode(vault_mode)).unwrap();
    std::fs::set_permissions(home, std::fs::Permissions::from_mode(0o555)).unwrap();
    let restore = Restore(home.to_path_buf());
    if tempfile::NamedTempFile::new_in(home).is_ok() {
        #[allow(clippy::print_stderr)]
        {
            eprintln!("skipped: this user can write a read-only directory");
        }
        return None;
    }
    Some(restore)
}

#[cfg(unix)]
struct Restore(PathBuf);

#[cfg(unix)]
impl Drop for Restore {
    fn drop(&mut self) {
        use std::os::unix::fs::PermissionsExt;
        let _ = std::fs::set_permissions(&self.0, std::fs::Permissions::from_mode(0o755));
        let _ = std::fs::set_permissions(
            self.0.join("secrets"),
            std::fs::Permissions::from_mode(0o755),
        );
    }
}

#[test]
fn decisions_by_kind_and_setting() {
    let origin = PathBuf::from("/h");
    let refused = |kind, nested| match decide("exec", kind, &[(origin.clone(), nested)]) {
        NestedLaunch::Refuse(message) => message,
        other => panic!("{other:?}"),
    };
    assert_eq!(
        decide("exec", NestedKind::Agent, &[]),
        NestedLaunch::NotNested
    );
    assert_eq!(
        decide(
            "exec",
            NestedKind::Agent,
            &[(origin.clone(), NestedAgents::Pass)]
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
        assert!(refused(NestedKind::Interactive, nested).contains("approval prompts"));
        assert!(refused(NestedKind::Host, nested).contains("never allowed"));
        assert!(refused(NestedKind::Credentials, nested).contains("credentials"));
    }
    match decide(
        "",
        NestedKind::Interactive,
        &[(origin.clone(), NestedAgents::Pass)],
    ) {
        NestedLaunch::Refuse(message) => assert!(message.starts_with("`corbanu` was"), "{message}"),
        other => panic!("{other:?}"),
    }
    // Refuse wins over pass.
    assert!(matches!(
        decide(
            "exec",
            NestedKind::Agent,
            &[
                (PathBuf::from("/a"), NestedAgents::Pass),
                (origin, NestedAgents::Refuse)
            ]
        ),
        NestedLaunch::Refuse(message) if message.contains("/h")
    ));
}

/// A person's own launch: the home stores Aggressive but is writable. The
/// probe leaves nothing behind.
#[test]
fn writable_aggressive_home_is_not_an_origin() {
    let home = home_with(ChosenLevel::Aggressive, NestedAgents::Refuse);
    assert_eq!(nested_origins(vec![home.path().to_path_buf()]), Vec::new());
    assert_eq!(std::fs::read_dir(home.path()).unwrap().count(), 2);
}

#[cfg(unix)]
#[test]
fn only_the_aggressive_sandbox_makes_an_origin() {
    let home = home_with(ChosenLevel::Aggressive, NestedAgents::Pass);
    // A Permissive sandbox: the home is read-only but its vault is readable.
    {
        let Some(_restore) = sandbox_away(home.path(), /*vault_readable*/ true) else {
            return;
        };
        assert_eq!(nested_origins(vec![home.path().to_path_buf()]), Vec::new());
    }
    let Some(_restore) = sandbox_away(home.path(), /*vault_readable*/ false) else {
        return;
    };
    assert_eq!(
        nested_origins(vec![home.path().to_path_buf()]),
        vec![(home.path().to_path_buf(), NestedAgents::Pass)]
    );
}

/// An agent cannot turn refuse into pass with a home it wrote itself, or
/// drop the real origin by pointing the marker or `CODEX_HOME` elsewhere.
#[cfg(unix)]
#[test]
fn forged_pass_home_does_not_override_the_real_refuse() {
    let real = home_with(ChosenLevel::Aggressive, NestedAgents::Refuse);
    let forged = home_with(ChosenLevel::Aggressive, NestedAgents::Pass);
    let Some(_restore) = sandbox_away(real.path(), /*vault_readable*/ false) else {
        return;
    };
    let origins = nested_origins(vec![forged.path().to_path_buf(), real.path().to_path_buf()]);
    assert_eq!(
        origins,
        vec![(real.path().to_path_buf(), NestedAgents::Refuse)]
    );
    assert!(matches!(
        decide("exec", NestedKind::Agent, &origins),
        NestedLaunch::Refuse(_)
    ));
}

/// An Aggressive session that saved Permissive keeps its rule file and stays
/// Aggressive until it restarts; its commands are still nested.
#[cfg(unix)]
#[test]
fn saved_permissive_with_the_rule_file_still_refuses() {
    let home = home_with(ChosenLevel::Aggressive, NestedAgents::Pass);
    level::save(home.path(), ChosenLevel::Permissive, NestedAgents::Refuse).unwrap();
    assert!(level::rules_path(home.path()).exists());
    let Some(_restore) = sandbox_away(home.path(), /*vault_readable*/ false) else {
        return;
    };
    assert_eq!(
        nested_origins(vec![home.path().to_path_buf()]),
        vec![(home.path().to_path_buf(), NestedAgents::Refuse)]
    );
}

#[test]
fn registry_records_and_forgets_origins() {
    let registry = tempfile::tempdir().unwrap();
    let registry = registry.path().join("aggressive-homes");
    let home = tempfile::tempdir().unwrap();
    registry_entry(&registry, home.path(), /*aggressive*/ true).unwrap();
    registry_entry(&registry, home.path(), /*aggressive*/ true).unwrap();
    std::fs::write(registry.join("relative"), "not/absolute").unwrap();
    assert_eq!(registered_homes(&registry), vec![home.path().to_path_buf()]);
    registry_entry(&registry, home.path(), /*aggressive*/ false).unwrap();
    registry_entry(&registry, home.path(), /*aggressive*/ false).unwrap();
    assert_eq!(registered_homes(&registry), Vec::<PathBuf>::new());
}

#[test]
fn exec_overrides_write_the_rule_and_carry_the_marker() {
    let origin = home_with(ChosenLevel::Aggressive, NestedAgents::Pass);
    let child = tempfile::tempdir().unwrap();
    let overrides = prepare_nested_exec(child.path(), origin.path()).unwrap();
    assert_eq!(
        std::fs::read_to_string(level::rules_path(child.path())).unwrap(),
        level::rules_contents()
    );
    assert!(overrides.contains(&(
        format!("shell_environment_policy.set.{ORIGIN_ENV}"),
        toml::Value::String(origin.path().to_string_lossy().into_owned()),
    )));
}

#[test]
fn account_home_is_absolute_when_known() {
    if let Ok(Some(home)) = account_home() {
        assert!(home.is_absolute(), "{}", home.display());
    }
}
