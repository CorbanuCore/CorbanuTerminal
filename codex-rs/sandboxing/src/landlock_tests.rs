use super::*;
use pretty_assertions::assert_eq;

#[test]
fn legacy_landlock_flag_is_included_when_requested() {
    let command = vec!["/bin/true".to_string()];
    let command_cwd = Path::new("/tmp/link");
    let cwd = Path::new("/tmp");

    let default_bwrap = create_linux_sandbox_command_args(
        command.clone(),
        command_cwd,
        cwd,
        /*use_legacy_landlock*/ false,
        /*allow_network_for_proxy*/ false,
    );
    assert_eq!(
        default_bwrap.contains(&"--use-legacy-landlock".to_string()),
        false
    );

    let legacy_landlock = create_linux_sandbox_command_args(
        command,
        command_cwd,
        cwd,
        /*use_legacy_landlock*/ true,
        /*allow_network_for_proxy*/ false,
    );
    assert_eq!(
        legacy_landlock.contains(&"--use-legacy-landlock".to_string()),
        true
    );
}

#[test]
fn proxy_flag_takes_precedence_over_legacy_landlock() {
    let command = vec!["/bin/true".to_string()];
    let command_cwd = Path::new("/tmp/link");
    let cwd = Path::new("/tmp");
    let permission_profile = PermissionProfile::read_only();

    let args = create_linux_sandbox_command_args_for_permission_profile(
        command,
        command_cwd,
        &permission_profile,
        cwd,
        /*use_legacy_landlock*/ true,
        /*allow_network_for_proxy*/ true,
    );
    assert_eq!(
        args.contains(&"--allow-network-for-proxy".to_string()),
        true
    );
    assert_eq!(args.contains(&"--use-legacy-landlock".to_string()), false);
}

#[test]
fn permission_profile_flag_is_included() {
    let command = vec!["/bin/true".to_string()];
    let command_cwd = Path::new("/tmp/link");
    let cwd = Path::new("/tmp");
    let permission_profile = PermissionProfile::read_only();

    let args = create_linux_sandbox_command_args_for_permission_profile(
        command,
        command_cwd,
        &permission_profile,
        cwd,
        /*use_legacy_landlock*/ true,
        /*allow_network_for_proxy*/ false,
    );

    assert_eq!(
        args.windows(2)
            .any(|window| { window[0] == "--permission-profile" && !window[1].is_empty() }),
        true
    );
    assert_eq!(
        args.windows(2)
            .any(|window| window[0] == "--command-cwd" && window[1] == "/tmp/link"),
        true
    );
}

#[test]
fn proxy_network_requires_managed_requirements() {
    assert_eq!(
        allow_network_for_proxy(/*enforce_managed_network*/ false),
        false
    );
    assert_eq!(
        allow_network_for_proxy(/*enforce_managed_network*/ true),
        true
    );
}

#[test]
fn placeholder_record_dir_goes_before_the_command_only_for_an_existing_home() {
    let home = tempfile::TempDir::new().expect("home");
    let mut args = vec![
        "--sandbox-policy-cwd".to_string(),
        "/w".to_string(),
        "--".to_string(),
        "--placeholder-record-dir".to_string(),
    ];
    insert_placeholder_record_dir_arg(&mut args, home.path());
    let dir = home
        .path()
        .join(PLACEHOLDER_RECORD_DIR_NAME)
        .to_string_lossy()
        .into_owned();
    assert_eq!(
        args,
        vec![
            "--sandbox-policy-cwd".to_string(),
            "/w".to_string(),
            PLACEHOLDER_RECORD_DIR_FLAG.to_string(),
            dir,
            "--".to_string(),
            "--placeholder-record-dir".to_string(),
        ]
    );

    // Not for a home that does not exist (the helper never makes it), nor
    // a relative one.
    let mut args = vec!["--".to_string()];
    insert_placeholder_record_dir_arg(&mut args, &home.path().join("missing"));
    insert_placeholder_record_dir_arg(&mut args, Path::new("relative"));
    assert_eq!(args, vec!["--".to_string()]);
}
