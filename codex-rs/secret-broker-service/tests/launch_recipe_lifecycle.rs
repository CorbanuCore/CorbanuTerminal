#![cfg(all(target_os = "linux", feature = "synthetic-fixture"))]
#![forbid(unsafe_code)]
#![allow(clippy::unwrap_used)]
use codex_secret_broker_service::SyntheticChildRole;
use codex_secret_broker_service::SyntheticLaunchRecipe;
use pretty_assertions::assert_eq;
use std::os::unix::fs::PermissionsExt;

#[test]
fn pf_27_s01_recipe_transports_exact_args_with_null_stdio_and_no_home() {
    assert!(
        !nix::unistd::geteuid().is_root(),
        "synthetic recipe test must not run as root"
    );
    assert!(
        std::env::var_os("HOME").is_some(),
        "HOME prerequisite for environment-removal check"
    );
    let temp = tempfile::tempdir().unwrap();
    let executable = temp.path().join("recipe-fixture");
    // The script receives numeric synthetic argv only. Record presence of HOME,
    // never its contents or other inherited environment values, even on failure.
    std::fs::write(&executable, "#!/bin/sh\nnulls=yes\nfor fd in 0 1 2; do [ /proc/self/fd/$fd -ef /dev/null ] || nulls=no; done\nprintf '%s\\n' \"$nulls\" \"${HOME+present}\" \"$PWD\" \"$@\" > \"$0.args\"\n").unwrap();
    std::fs::set_permissions(&executable, std::fs::Permissions::from_mode(0o700)).unwrap();
    let recipe = SyntheticLaunchRecipe::new(
        executable.clone(),
        [(101, 201), (102, 202), (103, 203)],
        204,
    )
    .unwrap();
    for (role, name, uid, gid, anchor) in [
        (SyntheticChildRole::Journal, "journal", "101", "201", "204"),
        (SyntheticChildRole::Policy, "policy", "102", "202", "204"),
        (SyntheticChildRole::Worker, "worker", "103", "203", "none"),
    ] {
        let mut command = recipe.command(role);
        assert_eq!(command.get_program(), executable.as_os_str());
        assert_eq!(command.get_current_dir(), Some(std::path::Path::new("/")));
        assert!(command.status().unwrap().success());
        assert_eq!(
            std::fs::read_to_string(temp.path().join("recipe-fixture.args")).unwrap(),
            format!("yes\n\n/\n--prepare-synthetic-child\n{name}\n{uid}\n{gid}\n{anchor}\n")
        );
    }
}
