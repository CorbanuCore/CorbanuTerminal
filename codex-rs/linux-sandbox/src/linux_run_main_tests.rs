#[cfg(test)]
use super::*;
#[cfg(test)]
use crate::linux_run_main::install_bwrap_signal_forwarders;
#[cfg(test)]
use crate::linux_run_main::wait_for_bwrap_child;
#[cfg(test)]
use codex_protocol::models::PermissionProfile;
#[cfg(test)]
use codex_protocol::protocol::FileSystemSandboxPolicy;
#[cfg(test)]
use codex_protocol::protocol::NetworkSandboxPolicy;
#[cfg(test)]
use codex_utils_absolute_path::AbsolutePathBuf;
#[cfg(test)]
use pretty_assertions::assert_eq;
#[cfg(test)]
use std::os::unix::fs::PermissionsExt;

fn read_only_permission_profile() -> PermissionProfile {
    PermissionProfile::read_only()
}

fn read_only_file_system_policy() -> FileSystemSandboxPolicy {
    read_only_permission_profile().file_system_sandbox_policy()
}

#[test]
fn detects_proc_mount_invalid_argument_failure() {
    let stderr = "bwrap: Can't mount proc on /newroot/proc: Invalid argument";
    assert!(is_proc_mount_failure(stderr));
}

#[test]
fn detects_proc_mount_operation_not_permitted_failure() {
    let stderr = "bwrap: Can't mount proc on /newroot/proc: Operation not permitted";
    assert!(is_proc_mount_failure(stderr));
}

#[test]
fn detects_proc_mount_permission_denied_failure() {
    let stderr = "bwrap: Can't mount proc on /newroot/proc: Permission denied";
    assert!(is_proc_mount_failure(stderr));
}

#[test]
fn ignores_non_proc_mount_errors() {
    let stderr = "bwrap: Can't bind mount /dev/null: Operation not permitted";
    assert!(!is_proc_mount_failure(stderr));
}

#[test]
fn inserts_bwrap_argv0_before_command_separator() {
    let file_system_sandbox_policy = read_only_file_system_policy();
    let mut argv = build_bwrap_argv(
        vec!["/bin/true".to_string()],
        &file_system_sandbox_policy,
        Path::new("/"),
        Path::new("/"),
        BwrapOptions {
            mount_proc: true,
            network_mode: BwrapNetworkMode::FullAccess,
            ..Default::default()
        },
    )
    .expect("build bwrap argv")
    .args;
    apply_inner_command_argv0_for_launcher(
        &mut argv,
        /*supports_argv0*/ true,
        "/tmp/codex-arg0-session/codex-linux-sandbox".to_string(),
    );
    assert_eq!(
        argv,
        vec![
            "bwrap".to_string(),
            "--new-session".to_string(),
            "--die-with-parent".to_string(),
            "--ro-bind".to_string(),
            "/".to_string(),
            "/".to_string(),
            "--dev".to_string(),
            "/dev".to_string(),
            "--unshare-user".to_string(),
            "--unshare-pid".to_string(),
            "--proc".to_string(),
            "/proc".to_string(),
            "--argv0".to_string(),
            "codex-linux-sandbox".to_string(),
            "--".to_string(),
            "/bin/true".to_string(),
        ]
    );
}

#[test]
fn rewrites_inner_command_path_when_bwrap_lacks_argv0() {
    let file_system_sandbox_policy = read_only_file_system_policy();
    let mut argv = build_bwrap_argv(
        vec!["/bin/true".to_string()],
        &file_system_sandbox_policy,
        Path::new("/"),
        Path::new("/"),
        BwrapOptions {
            mount_proc: true,
            network_mode: BwrapNetworkMode::FullAccess,
            ..Default::default()
        },
    )
    .expect("build bwrap argv")
    .args;
    apply_inner_command_argv0_for_launcher(
        &mut argv,
        /*supports_argv0*/ false,
        "/tmp/codex-arg0-session/codex-linux-sandbox".to_string(),
    );

    assert!(!argv.iter().any(|arg| arg == "--argv0"));
    assert!(
        argv.windows(2)
            .any(|window| { window == ["--", "/tmp/codex-arg0-session/codex-linux-sandbox"] })
    );
}

#[test]
fn rewrites_bwrap_helper_command_not_nested_user_command_when_current_exe_appears_later() {
    let nested_current_exe = std::env::current_exe()
        .expect("current exe")
        .to_string_lossy()
        .into_owned();
    let mut argv = vec![
        "bwrap".to_string(),
        "--".to_string(),
        "/tmp/helper-symlink".to_string(),
        "--sandbox-policy-cwd".to_string(),
        "/tmp/cwd".to_string(),
        "--".to_string(),
        nested_current_exe.clone(),
        "--codex-run-as-apply-patch".to_string(),
        "patch".to_string(),
    ];

    apply_inner_command_argv0_for_launcher(
        &mut argv,
        /*supports_argv0*/ false,
        "/tmp/argv0-fallback-helper".to_string(),
    );

    assert_eq!(
        argv,
        vec![
            "bwrap".to_string(),
            "--".to_string(),
            "/tmp/argv0-fallback-helper".to_string(),
            "--sandbox-policy-cwd".to_string(),
            "/tmp/cwd".to_string(),
            "--".to_string(),
            nested_current_exe,
            "--codex-run-as-apply-patch".to_string(),
            "patch".to_string(),
        ]
    );
}

#[test]
fn inserts_unshare_net_when_network_isolation_requested() {
    let file_system_sandbox_policy = read_only_file_system_policy();
    let argv = build_bwrap_argv(
        vec!["/bin/true".to_string()],
        &file_system_sandbox_policy,
        Path::new("/"),
        Path::new("/"),
        BwrapOptions {
            mount_proc: true,
            network_mode: BwrapNetworkMode::Isolated,
            ..Default::default()
        },
    )
    .expect("build bwrap argv")
    .args;
    assert!(argv.contains(&"--unshare-net".to_string()));
}

#[test]
fn inserts_unshare_net_when_proxy_only_network_mode_requested() {
    let file_system_sandbox_policy = read_only_file_system_policy();
    let argv = build_bwrap_argv(
        vec!["/bin/true".to_string()],
        &file_system_sandbox_policy,
        Path::new("/"),
        Path::new("/"),
        BwrapOptions {
            mount_proc: true,
            network_mode: BwrapNetworkMode::ProxyOnly,
            ..Default::default()
        },
    )
    .expect("build bwrap argv")
    .args;
    assert!(argv.contains(&"--unshare-net".to_string()));
}

#[test]
fn proxy_only_mode_takes_precedence_over_full_network_policy() {
    let mode = bwrap_network_mode(
        NetworkSandboxPolicy::Enabled,
        /*allow_network_for_proxy*/ true,
    );
    assert_eq!(mode, BwrapNetworkMode::ProxyOnly);
}

#[test]
fn split_only_filesystem_policy_requires_direct_runtime_enforcement() {
    let temp_dir = tempfile::TempDir::new().expect("tempdir");
    let docs = temp_dir.path().join("docs");
    std::fs::create_dir_all(&docs).expect("create docs");
    let docs = AbsolutePathBuf::from_absolute_path(&docs).expect("absolute docs");
    let policy = FileSystemSandboxPolicy::restricted(vec![
        codex_protocol::permissions::FileSystemSandboxEntry {
            path: codex_protocol::permissions::FileSystemPath::Special {
                value: codex_protocol::permissions::FileSystemSpecialPath::project_roots(
                    /*subpath*/ None,
                ),
            },
            access: codex_protocol::permissions::FileSystemAccessMode::Write,
            missing_path_behavior: None,
        },
        codex_protocol::permissions::FileSystemSandboxEntry {
            path: codex_protocol::permissions::FileSystemPath::Path { path: docs },
            access: codex_protocol::permissions::FileSystemAccessMode::Read,
            missing_path_behavior: None,
        },
    ]);

    assert!(
        policy.needs_direct_runtime_enforcement(NetworkSandboxPolicy::Restricted, temp_dir.path(),)
    );
}

#[test]
fn root_write_read_only_carveout_requires_direct_runtime_enforcement() {
    let temp_dir = tempfile::TempDir::new().expect("tempdir");
    let docs = temp_dir.path().join("docs");
    std::fs::create_dir_all(&docs).expect("create docs");
    let docs = AbsolutePathBuf::from_absolute_path(&docs).expect("absolute docs");
    let policy = FileSystemSandboxPolicy::restricted(vec![
        codex_protocol::permissions::FileSystemSandboxEntry {
            path: codex_protocol::permissions::FileSystemPath::Special {
                value: codex_protocol::permissions::FileSystemSpecialPath::Root,
            },
            access: codex_protocol::permissions::FileSystemAccessMode::Write,
            missing_path_behavior: None,
        },
        codex_protocol::permissions::FileSystemSandboxEntry {
            path: codex_protocol::permissions::FileSystemPath::Path { path: docs },
            access: codex_protocol::permissions::FileSystemAccessMode::Read,
            missing_path_behavior: None,
        },
    ]);

    assert!(
        policy.needs_direct_runtime_enforcement(NetworkSandboxPolicy::Restricted, temp_dir.path(),)
    );
}

#[test]
fn managed_proxy_preflight_argv_unshares_network() {
    let mode = bwrap_network_mode(
        NetworkSandboxPolicy::Enabled,
        /*allow_network_for_proxy*/ true,
    );
    let argv = build_preflight_bwrap_argv(mode)
        .expect("build preflight argv")
        .args;
    assert!(argv.iter().any(|arg| arg == "--"));
    assert!(argv.iter().any(|arg| arg == "--unshare-net"));
}

#[test]
fn proc_mount_preflight_does_not_bind_the_full_filesystem() {
    let argv = build_preflight_bwrap_argv(BwrapNetworkMode::FullAccess)
        .expect("build preflight argv")
        .args;

    assert!(argv.windows(2).any(|window| window == ["--tmpfs", "/"]));
    assert!(argv.windows(2).any(|window| window == ["--proc", "/proc"]));
    assert!(
        !argv
            .windows(3)
            .any(|window| window == ["--ro-bind", "/", "/"])
    );
    assert!(!argv.windows(3).any(|window| window == ["--bind", "/", "/"]));
}

#[test]
fn cleanup_synthetic_mount_targets_removes_only_empty_mount_targets() {
    let temp_dir = tempfile::TempDir::new().expect("tempdir");
    let empty_file = temp_dir.path().join(".git");
    let empty_dir = temp_dir.path().join(".agents");
    let non_empty_file = temp_dir.path().join("non-empty");
    let missing_file = temp_dir.path().join(".missing");
    std::fs::write(&empty_file, "").expect("write empty file");
    std::fs::create_dir(&empty_dir).expect("create empty dir");
    std::fs::write(&non_empty_file, "keep").expect("write nonempty file");

    let registrations = register_synthetic_mount_targets(&[
        crate::bwrap::SyntheticMountTarget::missing(&empty_file),
        crate::bwrap::SyntheticMountTarget::missing_empty_directory(&empty_dir),
        crate::bwrap::SyntheticMountTarget::missing(&non_empty_file),
        crate::bwrap::SyntheticMountTarget::missing(&missing_file),
    ]);
    cleanup_synthetic_mount_targets(&registrations);

    assert!(!empty_file.exists());
    assert!(!empty_dir.exists());
    assert_eq!(
        std::fs::read_to_string(&non_empty_file).expect("read nonempty file"),
        "keep"
    );
    assert!(!missing_file.exists());
}

#[test]
fn synthetic_mount_registry_root_is_unique_to_effective_user() {
    let effective_uid = unsafe { libc::geteuid() };
    assert_eq!(
        synthetic_mount_registry_root(),
        std::env::temp_dir()
            .canonicalize()
            .expect("resolve temp directory")
            .join(format!(
                "codex-bwrap-synthetic-mount-targets-{effective_uid}"
            ))
    );
}

#[test]
fn cleanup_synthetic_mount_targets_waits_for_other_active_registrations() {
    let temp_dir = tempfile::TempDir::new().expect("tempdir");
    let empty_dir = temp_dir.path().join(".git");
    std::fs::create_dir(&empty_dir).expect("create empty dir");
    let target = crate::bwrap::SyntheticMountTarget::missing_empty_directory(&empty_dir);

    let registrations = register_synthetic_mount_targets(std::slice::from_ref(&target));
    let active_marker = registrations[0].marker_dir.join("1");
    std::fs::write(&active_marker, "").expect("write active marker");

    cleanup_synthetic_mount_targets(&registrations);
    assert!(empty_dir.exists());

    std::fs::remove_file(active_marker).expect("remove active marker");
    let registrations = register_synthetic_mount_targets(std::slice::from_ref(&target));
    cleanup_synthetic_mount_targets(&registrations);

    assert!(!empty_dir.exists());
}

#[test]
fn cleanup_synthetic_mount_targets_removes_transient_file_after_concurrent_owner_exits() {
    let temp_dir = tempfile::TempDir::new().expect("tempdir");
    let empty_file = temp_dir.path().join(".git");
    let first_target = crate::bwrap::SyntheticMountTarget::missing(&empty_file);

    let first_registrations = register_synthetic_mount_targets(&[first_target]);
    std::fs::write(&empty_file, "").expect("write transient empty file");
    let active_marker = first_registrations[0].marker_dir.join("1");
    std::fs::write(&active_marker, SYNTHETIC_MOUNT_MARKER_SYNTHETIC).expect("write active marker");
    let metadata = std::fs::symlink_metadata(&empty_file).expect("stat empty file");
    let second_target =
        crate::bwrap::SyntheticMountTarget::existing_empty_file(&empty_file, &metadata);
    let second_registrations = register_synthetic_mount_targets(&[second_target]);

    cleanup_synthetic_mount_targets(&first_registrations);
    assert!(empty_file.exists());

    std::fs::remove_file(active_marker).expect("remove active marker");
    cleanup_synthetic_mount_targets(&second_registrations);

    assert!(!empty_file.exists());
}

/// #562: a placeholder found when the arguments were built but removed by
/// its owner's cleanup before this run registered: bubblewrap makes a new
/// one, which is this run's own, so its marker must say a run made it (a
/// later run then holds it too, and the last one out removes it).
#[test]
fn register_synthetic_mount_targets_owns_a_placeholder_that_vanished_before_registration() {
    let temp_dir = tempfile::TempDir::new().expect("tempdir");
    let placeholder = temp_dir.path().join("AGENTS.md");
    std::fs::write(&placeholder, "").expect("write placeholder");
    let metadata = std::fs::symlink_metadata(&placeholder).expect("stat placeholder");
    let target = crate::bwrap::SyntheticMountTarget::existing_empty_file(&placeholder, &metadata);
    std::fs::remove_file(&placeholder).expect("owner's cleanup");

    let registrations = register_synthetic_mount_targets(&[target]);

    assert!(!registrations[0].target.preserves_pre_existing_path());
    assert_eq!(
        std::fs::read(&registrations[0].marker_file).expect("marker"),
        SYNTHETIC_MOUNT_MARKER_SYNTHETIC
    );
    // bubblewrap's new mount target, then this run's cleanup.
    std::fs::write(&placeholder, "").expect("new placeholder");
    cleanup_synthetic_mount_targets(&registrations);
    assert!(!placeholder.exists());
}

/// #607: another live run's empty placeholder folder (`.codex`, which it
/// holds read-only in its own writable root) that this run's policy denies
/// outside its writable roots is masked and held (registered as found), so
/// its maker's cleanup leaves it while this run masks it; the last run out
/// removes it. Before, only a placeholder file was held, and the maker
/// removed the folder (and the mask with it) from under this run.
#[test]
fn a_live_placeholder_folder_is_held_while_it_is_masked() {
    let temp_dir = tempfile::TempDir::new().expect("tempdir");
    let root = temp_dir.path().canonicalize().expect("canonical");
    let project = root.join("project");
    std::fs::create_dir(&project).expect("project");
    let placeholder = root.join(".codex");
    // Another run, still running, registered and made it.
    let mut maker = std::process::Command::new("sleep")
        .arg("60")
        .spawn()
        .expect("spawn the maker");
    let marker_dir = synthetic_mount_marker_dir(&placeholder);
    std::fs::create_dir_all(&marker_dir).expect("marker dir");
    let made = crate::bwrap::SyntheticMountTarget::missing_empty_directory(&placeholder);
    let maker_marker = marker_dir.join(maker.id().to_string());
    std::fs::write(&maker_marker, SYNTHETIC_MOUNT_MARKER_SYNTHETIC).expect("maker marker");
    write_synthetic_mount_target_record(&marker_dir, &made);
    std::fs::create_dir(&placeholder).expect("placeholder folder");

    let entry = |path: &Path, access| FileSystemSandboxEntry {
        path: FileSystemPath::Path {
            path: AbsolutePathBuf::from_absolute_path(path).expect("absolute"),
        },
        access,
        missing_path_behavior: None,
    };
    let policy = FileSystemSandboxPolicy::restricted(vec![
        FileSystemSandboxEntry {
            path: FileSystemPath::Special {
                value: FileSystemSpecialPath::Root,
            },
            access: FileSystemAccessMode::Read,
            missing_path_behavior: None,
        },
        entry(&project, FileSystemAccessMode::Write),
        entry(&placeholder, FileSystemAccessMode::Deny),
    ]);
    let args = create_bwrap_command_args(
        vec!["/bin/true".to_string()],
        &policy,
        &project,
        &project,
        BwrapOptions::default(),
    )
    .expect("bwrap args");
    let held: Vec<&crate::bwrap::SyntheticMountTarget> = args
        .synthetic_mount_targets
        .iter()
        .filter(|target| target.path() == placeholder)
        .collect();
    assert_eq!(held.len(), 1, "{:?}", args.synthetic_mount_targets);
    assert!(held[0].preserves_pre_existing_path());
    let registrations = register_synthetic_mount_targets(&args.synthetic_mount_targets);

    // The maker ends and cleans up: the folder stays while this run masks it.
    cleanup_synthetic_mount_targets(&[SyntheticMountTargetRegistration {
        target: made,
        marker_file: maker_marker,
        marker_dir: marker_dir.clone(),
        wrote_record: true,
    }]);
    maker.kill().expect("kill the maker");
    maker.wait().expect("reap the maker");
    assert!(placeholder.is_dir(), "the maker removed a held placeholder");

    // This run is the last one out.
    cleanup_synthetic_mount_targets(&registrations);
    assert!(!placeholder.exists());
    assert!(!marker_dir.exists());
}

/// A pid no live process has: a child that has been reaped.
fn ended_pid() -> u32 {
    let mut child = std::process::Command::new("true").spawn().expect("spawn");
    child.wait().expect("wait");
    child.id()
}

/// #582: a placeholder made (and recorded) by a run that ended without
/// cleaning up, with `others` markers (pid, contents) written first: the
/// record comes last, so a concurrent sweep never sees a state the test
/// does not mean.
fn plant_stale_placeholder(placeholder: &Path, others: &[(u32, &[u8])]) -> PathBuf {
    let marker_dir = synthetic_mount_marker_dir(placeholder);
    std::fs::create_dir_all(&marker_dir).expect("marker dir");
    for (pid, contents) in others {
        std::fs::write(marker_dir.join(pid.to_string()), contents).expect("marker");
    }
    std::fs::write(
        marker_dir.join(ended_pid().to_string()),
        SYNTHETIC_MOUNT_MARKER_SYNTHETIC,
    )
    .expect("the maker's marker");
    write_synthetic_mount_target_record(
        &marker_dir,
        &crate::bwrap::SyntheticMountTarget::missing(placeholder),
    );
    marker_dir
}

/// #582: the next run removes a placeholder whose runs all ended without
/// cleaning up (as their cleanup would have), its record and marker folder.
/// One that is no longer an empty file stays, without its record.
#[test]
fn stale_placeholders_of_ended_runs_are_removed() {
    let temp_dir = tempfile::TempDir::new().expect("tempdir");
    let placeholder = temp_dir.path().join("Library");
    std::fs::write(&placeholder, "").expect("placeholder");
    let changed = temp_dir.path().join("changed");
    std::fs::write(&changed, "real contents").expect("a real file now");
    // Other tests in this process sweep the same registry, and these records
    // are stale on purpose: plant and inspect them under the registry lock,
    // which every sweep takes, so none removes them before the mode check.
    let (marker_dir, changed_marker_dir) = with_synthetic_mount_registry_lock(|| {
        let marker_dir = plant_stale_placeholder(&placeholder, &[]);
        let changed_marker_dir = plant_stale_placeholder(&changed, &[]);
        let record = marker_dir.join(SYNTHETIC_MOUNT_TARGET_RECORD);
        assert_eq!(
            std::fs::metadata(&record)
                .expect("record")
                .permissions()
                .mode()
                & 0o777,
            0o600
        );
        (marker_dir, changed_marker_dir)
    });

    remove_stale_synthetic_mount_targets();

    assert!(!placeholder.exists());
    assert!(!marker_dir.exists());
    assert_eq!(std::fs::read(&changed).expect("changed"), b"real contents");
    assert!(!changed_marker_dir.exists());
}

/// #607: a sweep leaves a placeholder whose run is in another PID namespace
/// (a container sharing `TMPDIR`), where `kill(pid, 0)` cannot see it, and
/// the person's own empty file made where a stale placeholder was removed
/// (it changed long after the record). One of an ended run in this
/// namespace still goes.
#[test]
fn stale_placeholder_sweep_leaves_other_namespaces_and_later_files() {
    let temp_dir = tempfile::TempDir::new().expect("tempdir");

    let other_namespace = temp_dir.path().join("other-namespace");
    std::fs::write(&other_namespace, "").expect("placeholder");
    let maker = ended_pid();
    let marker_dir = plant_stale_placeholder(
        &other_namespace,
        &[(maker, SYNTHETIC_MOUNT_MARKER_SYNTHETIC)],
    );
    std::fs::write(
        crate::placeholder_registry::pid_namespace_file(&marker_dir.join(maker.to_string())),
        "1",
    )
    .expect("another namespace");

    let persons = temp_dir.path().join("persons");
    std::fs::write(&persons, "").expect("the person's own empty file");
    let persons_marker_dir = plant_stale_placeholder(&persons, &[]);
    std::fs::File::options()
        .write(true)
        .open(persons_marker_dir.join(SYNTHETIC_MOUNT_TARGET_RECORD))
        .expect("record")
        .set_modified(std::time::SystemTime::now() - std::time::Duration::from_secs(3600))
        .expect("an hour-old record");

    let stale = temp_dir.path().join("stale");
    std::fs::write(&stale, "").expect("placeholder");
    let stale_marker_dir = plant_stale_placeholder(&stale, &[]);

    remove_stale_synthetic_mount_targets();

    assert!(other_namespace.exists());
    assert!(marker_dir.exists());
    assert!(persons.exists());
    assert!(!persons_marker_dir.exists());
    assert!(!stale.exists());
    assert!(!stale_marker_dir.exists());

    for entry in std::fs::read_dir(&marker_dir).expect("markers") {
        std::fs::remove_file(entry.expect("entry").path()).expect("remove");
    }
    std::fs::remove_dir(&marker_dir).expect("marker dir");
}

/// #607 review: a run removes its own placeholder whatever the record's time
/// says (a file server's clock, a run stopped in between); and a sweep
/// removes an older run's placeholder recorded without its location under a
/// folder reached through a link, resolved then, as before.
#[test]
fn own_placeholders_and_older_records_are_still_removed() {
    let temp_dir = tempfile::TempDir::new().expect("tempdir");
    let base = temp_dir.path().canonicalize().expect("canonical");
    let own = base.join("own");
    let registrations =
        register_synthetic_mount_targets(&[crate::bwrap::SyntheticMountTarget::missing(&own)]);
    std::fs::write(&own, "").expect("placeholder");
    std::fs::File::options()
        .write(true)
        .open(
            registrations[0]
                .marker_dir
                .join(SYNTHETIC_MOUNT_TARGET_RECORD),
        )
        .expect("record")
        .set_modified(std::time::SystemTime::now() - std::time::Duration::from_secs(3600))
        .expect("an hour-old record");
    cleanup_synthetic_mount_targets(&registrations);
    assert!(!own.exists());

    let dotfiles = base.join("dotfiles");
    std::fs::create_dir_all(&dotfiles).expect("dotfiles");
    std::os::unix::fs::symlink(&dotfiles, base.join("config")).expect("linked folder");
    let older = base.join("config").join("gh");
    std::fs::write(&older, "").expect("placeholder");
    let marker_dir = synthetic_mount_marker_dir(&older);
    std::fs::create_dir_all(&marker_dir).expect("marker dir");
    std::fs::write(
        marker_dir.join(ended_pid().to_string()),
        SYNTHETIC_MOUNT_MARKER_SYNTHETIC,
    )
    .expect("the maker's marker");
    let record = marker_dir.join(SYNTHETIC_MOUNT_TARGET_RECORD);
    let mut contents = b"file\n".to_vec();
    contents.extend_from_slice(older.as_os_str().as_bytes());
    std::fs::write(&record, contents).expect("an older run's record");
    std::fs::set_permissions(&record, std::fs::Permissions::from_mode(0o600)).expect("0600");

    remove_stale_synthetic_mount_targets();

    assert!(!dotfiles.join("gh").exists());
    assert!(!marker_dir.exists());
}

/// #607: a run's cleanup removes its placeholder where it was registered,
/// without following a link: a folder above it swapped for a link (by the
/// command, in its writable root) cannot redirect the removal to an empty
/// file elsewhere.
#[test]
fn cleanup_does_not_follow_a_link_swapped_in_above_the_placeholder() {
    let temp_dir = tempfile::TempDir::new().expect("tempdir");
    let base = temp_dir.path().canonicalize().expect("canonical");
    let folder = base.join("folder");
    let elsewhere = base.join("elsewhere");
    std::fs::create_dir_all(&folder).expect("folder");
    std::fs::create_dir_all(&elsewhere).expect("elsewhere");
    let placeholder = folder.join(".aws");
    let registrations =
        register_synthetic_mount_targets(&[crate::bwrap::SyntheticMountTarget::missing(
            &placeholder,
        )]);
    // bubblewrap's mount target, then the command's swap.
    std::fs::write(&placeholder, "").expect("placeholder");
    std::fs::rename(&folder, base.join("moved")).expect("move the folder");
    std::os::unix::fs::symlink(&elsewhere, &folder).expect("a link in its place");
    std::fs::write(elsewhere.join(".aws"), "").expect("an empty file elsewhere");

    cleanup_synthetic_mount_targets(&registrations);

    assert!(elsewhere.join(".aws").exists(), "removed through the link");
    assert!(!registrations[0].marker_dir.exists());
}

/// #582: a recorded placeholder a live run still uses (its maker, or a run
/// that holds it after its maker was killed) stays, as does a marker folder
/// with a marker of another kind.
#[test]
fn stale_placeholder_sweep_leaves_placeholders_in_use() {
    let temp_dir = tempfile::TempDir::new().expect("tempdir");

    let made = temp_dir.path().join("made");
    std::fs::write(&made, "").expect("placeholder");
    let made_registrations =
        register_synthetic_mount_targets(&[crate::bwrap::SyntheticMountTarget::missing(&made)]);

    // A run holding it (registered first), then its killed maker's record.
    let held = temp_dir.path().join("held");
    std::fs::write(&held, "").expect("placeholder");
    let metadata = std::fs::symlink_metadata(&held).expect("stat");
    let held_registrations = register_synthetic_mount_targets(&[
        crate::bwrap::SyntheticMountTarget::existing_empty_file(&held, &metadata),
    ]);
    let held_marker_dir = plant_stale_placeholder(&held, &[]);
    // A run that meets it now holds it too (see `is_live_placeholder`).
    assert!(is_live_placeholder(&held));

    let other = temp_dir.path().join("other");
    std::fs::write(&other, "").expect("placeholder");
    let other_marker_dir =
        plant_stale_placeholder(&other, &[(ended_pid(), PROTECTED_CREATE_MARKER)]);

    remove_stale_synthetic_mount_targets();

    assert!(made.exists());
    assert!(held.exists());
    assert!(held_marker_dir.exists());
    assert!(other.exists());

    // The last run out removes each as its maker would have (#582 review:
    // a holder whose maker was killed must not leave it behind).
    cleanup_synthetic_mount_targets(&made_registrations);
    cleanup_synthetic_mount_targets(&held_registrations);
    assert!(!made.exists());
    assert!(!held.exists());
    assert!(!held_marker_dir.exists());

    for entry in std::fs::read_dir(&other_marker_dir).expect("other markers") {
        std::fs::remove_file(entry.expect("entry").path()).expect("remove");
    }
    std::fs::remove_dir(&other_marker_dir).expect("other marker dir");
}

/// Whether another thread could take the registry lock now.
fn registry_lock_is_free() -> bool {
    std::thread::spawn(|| {
        let lock = std::fs::File::open(synthetic_mount_registry_root().join("lock")).expect("lock");
        // SAFETY: tries to lock a descriptor this closure owns.
        unsafe { libc::flock(lock.as_raw_fd(), libc::LOCK_EX | libc::LOCK_NB) == 0 }
    })
    .join()
    .expect("try the lock")
}

/// #673: a run holds the registry lock from before it looks at the paths it
/// mounts until it has registered them, so no other run's cleanup can
/// remove a placeholder in between.
#[test]
fn the_registry_stays_locked_from_building_the_arguments_to_registering() {
    let temp_dir = tempfile::TempDir::new().expect("tempdir");
    let project = temp_dir.path().canonicalize().expect("canonical");
    let policy = FileSystemSandboxPolicy::restricted(vec![
        FileSystemSandboxEntry {
            path: FileSystemPath::Special {
                value: FileSystemSpecialPath::Root,
            },
            access: FileSystemAccessMode::Read,
            missing_path_behavior: None,
        },
        FileSystemSandboxEntry {
            path: FileSystemPath::Path {
                path: AbsolutePathBuf::from_absolute_path(&project).expect("absolute"),
            },
            access: FileSystemAccessMode::Write,
            missing_path_behavior: None,
        },
    ]);
    let args = create_bwrap_command_args(
        vec!["/bin/true".to_string()],
        &policy,
        &project,
        &project,
        BwrapOptions::default(),
    )
    .expect("bwrap args");
    assert!(
        !registry_lock_is_free(),
        "the arguments were built unlocked"
    );

    let (synthetic, protected) = register_mount_targets(
        &args.synthetic_mount_targets,
        &args.protected_create_targets,
        args.registry_lock,
        &args.args,
    );
    // (Other tests may hold it now.)
    assert!(!REGISTRY_LOCK_HELD.get(), "registering kept the lock");
    cleanup_synthetic_mount_targets(&synthetic);
    cleanup_protected_create_targets(&protected);
}

/// #673: another live run's placeholder (`~/.aws`, denied outside this
/// run's writable roots) found while this run builds its arguments stays
/// until this run has registered it, also when its maker ends and cleans up
/// in between: the cleanup waits for the registry lock, then leaves the
/// placeholder to this run, the last one out. Before, it could be removed
/// between this run finding it and checking it was a placeholder; this run
/// then masked it unregistered, and bubblewrap failed the command (`Can't
/// create file at ~/.aws: Read-only file system`).
#[test]
fn a_placeholder_found_while_building_stays_until_it_is_registered() {
    let temp_dir = tempfile::TempDir::new().expect("tempdir");
    let root = temp_dir.path().canonicalize().expect("canonical");
    let project = root.join("project");
    std::fs::create_dir(&project).expect("project");
    let placeholder = root.join(".aws");
    // Another run, still running, registered and made it.
    let mut maker = std::process::Command::new("sleep")
        .arg("60")
        .spawn()
        .expect("spawn the maker");
    let marker_dir = synthetic_mount_marker_dir(&placeholder);
    std::fs::create_dir_all(&marker_dir).expect("marker dir");
    let made = crate::bwrap::SyntheticMountTarget::missing(&placeholder);
    let maker_marker = marker_dir.join(maker.id().to_string());
    std::fs::write(&maker_marker, SYNTHETIC_MOUNT_MARKER_SYNTHETIC).expect("maker marker");
    write_synthetic_mount_target_record(&marker_dir, &made);
    std::fs::write(&placeholder, "").expect("placeholder");

    let entry = |path: &Path, access| FileSystemSandboxEntry {
        path: FileSystemPath::Path {
            path: AbsolutePathBuf::from_absolute_path(path).expect("absolute"),
        },
        access,
        missing_path_behavior: None,
    };
    let policy = FileSystemSandboxPolicy::restricted(vec![
        FileSystemSandboxEntry {
            path: FileSystemPath::Special {
                value: FileSystemSpecialPath::Root,
            },
            access: FileSystemAccessMode::Read,
            missing_path_behavior: None,
        },
        entry(&project, FileSystemAccessMode::Write),
        entry(&placeholder, FileSystemAccessMode::Deny),
    ]);
    let args = create_bwrap_command_args(
        vec!["/bin/true".to_string()],
        &policy,
        &project,
        &project,
        BwrapOptions::default(),
    )
    .expect("bwrap args");
    let found: Vec<&crate::bwrap::SyntheticMountTarget> = args
        .synthetic_mount_targets
        .iter()
        .filter(|target| target.path() == placeholder)
        .collect();
    assert_eq!(found.len(), 1, "{:?}", args.synthetic_mount_targets);
    assert!(found[0].preserves_pre_existing_path());

    // The maker ends and cleans up while this run has not registered yet.
    maker.kill().expect("kill the maker");
    maker.wait().expect("reap the maker");
    let maker_cleanup = std::thread::spawn({
        let marker_dir = marker_dir.clone();
        move || {
            cleanup_synthetic_mount_targets(&[SyntheticMountTargetRegistration {
                target: made,
                marker_file: maker_marker,
                marker_dir,
                wrote_record: true,
            }]);
        }
    });
    std::thread::sleep(Duration::from_millis(300));
    assert!(
        placeholder.exists(),
        "the maker removed a placeholder this run found"
    );
    assert!(!maker_cleanup.is_finished(), "the cleanup did not wait");

    let (registrations, _) = register_mount_targets(
        &args.synthetic_mount_targets,
        &args.protected_create_targets,
        args.registry_lock,
        &args.args,
    );
    maker_cleanup.join().expect("the maker's cleanup");
    assert!(
        placeholder.exists(),
        "the maker removed a placeholder this run registered"
    );

    // This run is the last one out.
    cleanup_synthetic_mount_targets(&registrations);
    assert!(!placeholder.exists());
    assert!(!marker_dir.exists());
}

#[test]
fn cleanup_synthetic_mount_targets_preserves_real_pre_existing_empty_file() {
    let temp_dir = tempfile::TempDir::new().expect("tempdir");
    let empty_file = temp_dir.path().join(".git");
    std::fs::write(&empty_file, "").expect("write pre-existing empty file");
    let metadata = std::fs::symlink_metadata(&empty_file).expect("stat empty file");
    let first_target =
        crate::bwrap::SyntheticMountTarget::existing_empty_file(&empty_file, &metadata);
    let second_target =
        crate::bwrap::SyntheticMountTarget::existing_empty_file(&empty_file, &metadata);

    let first_registrations = register_synthetic_mount_targets(&[first_target]);
    let second_registrations = register_synthetic_mount_targets(&[second_target]);

    cleanup_synthetic_mount_targets(&first_registrations);
    cleanup_synthetic_mount_targets(&second_registrations);

    assert!(empty_file.exists());
}

#[test]
fn cleanup_protected_create_targets_removes_created_path_and_reports_violation() {
    let temp_dir = tempfile::TempDir::new().expect("tempdir");
    let dot_git = temp_dir.path().join(".git");
    let target = crate::bwrap::ProtectedCreateTarget::missing(&dot_git);

    let registrations = register_protected_create_targets(&[target]);
    std::fs::create_dir(&dot_git).expect("create protected path");
    let violation = cleanup_protected_create_targets(&registrations);

    assert!(violation);
    assert!(!dot_git.exists());
}

#[test]
fn cleanup_protected_create_targets_removes_path_despite_active_marker() {
    let temp_dir = tempfile::TempDir::new().expect("tempdir");
    let dot_git = temp_dir.path().join(".git");
    let target = crate::bwrap::ProtectedCreateTarget::missing(&dot_git);

    let registrations = register_protected_create_targets(std::slice::from_ref(&target));
    let active_marker = registrations[0].marker_dir.join("1");
    std::fs::write(&active_marker, PROTECTED_CREATE_MARKER).expect("write active marker");
    std::fs::write(&dot_git, "").expect("create protected path");

    let violation = cleanup_protected_create_targets(&registrations);
    assert!(violation);
    assert!(!dot_git.exists());
}

#[test]
fn cleanup_protected_create_targets_removes_read_only_directory_and_reports_violation() {
    let temp_dir = tempfile::TempDir::new().expect("tempdir");
    let dot_git = temp_dir.path().join(".git");
    let outside = temp_dir.path().join("outside");
    let target = crate::bwrap::ProtectedCreateTarget::missing(&dot_git);

    let registrations = register_protected_create_targets(&[target]);
    std::fs::create_dir(&outside).expect("create outside directory");
    std::fs::set_permissions(&outside, std::fs::Permissions::from_mode(0o755))
        .expect("set outside directory permissions");
    std::fs::create_dir(&dot_git).expect("create protected path");
    std::fs::write(dot_git.join("config"), "[core]\n").expect("write protected child");
    std::os::unix::fs::symlink(&outside, dot_git.join("outside-link"))
        .expect("link outside directory");
    std::fs::set_permissions(&dot_git, std::fs::Permissions::from_mode(0o000))
        .expect("make protected path read-only");

    let violation = cleanup_protected_create_targets(&registrations);

    assert!(violation);
    assert!(!dot_git.exists());
    assert_eq!(
        std::fs::metadata(&outside)
            .expect("outside directory remains")
            .permissions()
            .mode()
            & 0o777,
        0o755
    );
}

#[test]
fn bwrap_signal_forwarder_terminates_child_and_keeps_parent_alive() {
    let supervisor_pid = unsafe { libc::fork() };
    assert!(supervisor_pid >= 0, "failed to fork supervisor");

    if supervisor_pid == 0 {
        run_bwrap_signal_forwarder_test_supervisor();
    }

    let status = wait_for_bwrap_child(supervisor_pid);
    assert!(libc::WIFEXITED(status), "supervisor status: {status}");
    assert_eq!(libc::WEXITSTATUS(status), 0);
}

#[cfg(test)]
fn run_bwrap_signal_forwarder_test_supervisor() -> ! {
    let child_pid = unsafe { libc::fork() };
    if child_pid < 0 {
        unsafe {
            libc::_exit(2);
        }
    }

    if child_pid == 0 {
        loop {
            unsafe {
                libc::pause();
            }
        }
    }

    install_bwrap_signal_forwarders(child_pid);
    unsafe {
        libc::raise(libc::SIGTERM);
    }

    let status = wait_for_bwrap_child(child_pid);
    let child_terminated_by_sigterm =
        libc::WIFSIGNALED(status) && libc::WTERMSIG(status) == libc::SIGTERM;
    unsafe {
        libc::_exit(if child_terminated_by_sigterm { 0 } else { 1 });
    }
}

#[test]
fn managed_proxy_inner_command_includes_route_spec() {
    let permission_profile = read_only_permission_profile();
    let args = build_inner_seccomp_command(InnerSeccompCommandArgs {
        sandbox_policy_cwd: Path::new("/tmp"),
        command_cwd: Some(Path::new("/tmp/link")),
        permission_profile: &permission_profile,
        allow_network_for_proxy: true,
        proxy_route_spec: Some("{\"routes\":[]}".to_string()),
        close_inherited_fd: None,
        command: vec!["/bin/true".to_string()],
    });

    assert!(args.iter().any(|arg| arg == "--proxy-route-spec"));
    assert!(args.iter().any(|arg| arg == "{\"routes\":[]}"));
}

#[test]
fn inner_command_includes_permission_profile_flag() {
    let permission_profile = read_only_permission_profile();
    let args = build_inner_seccomp_command(InnerSeccompCommandArgs {
        sandbox_policy_cwd: Path::new("/tmp"),
        command_cwd: Some(Path::new("/tmp/link")),
        permission_profile: &permission_profile,
        allow_network_for_proxy: false,
        proxy_route_spec: None,
        close_inherited_fd: None,
        command: vec!["/bin/true".to_string()],
    });

    assert!(args.iter().any(|arg| arg == "--permission-profile"));
    assert!(
        args.windows(2)
            .any(|window| { window == ["--command-cwd", "/tmp/link"] })
    );
}

/// #581: the inner stage is told which inherited descriptor to close (the
/// verified bundled bubblewrap run through `aa-exec`), and only then.
#[test]
fn inner_command_names_the_inherited_descriptor_to_close() {
    let permission_profile = read_only_permission_profile();
    for (fd, expected) in [(Some(7), true), (None, false)] {
        let args = build_inner_seccomp_command(InnerSeccompCommandArgs {
            sandbox_policy_cwd: Path::new("/tmp"),
            command_cwd: None,
            permission_profile: &permission_profile,
            allow_network_for_proxy: false,
            proxy_route_spec: None,
            close_inherited_fd: fd,
            command: vec!["/bin/true".to_string()],
        });
        let separator = args.iter().position(|arg| arg == "--").expect("separator");
        assert_eq!(
            args[..separator]
                .windows(2)
                .any(|window| window == ["--close-inherited-fd", "7"]),
            expected
        );
    }
}

#[test]
fn non_managed_inner_command_omits_route_spec() {
    let permission_profile = read_only_permission_profile();
    let args = build_inner_seccomp_command(InnerSeccompCommandArgs {
        sandbox_policy_cwd: Path::new("/tmp"),
        command_cwd: Some(Path::new("/tmp/link")),
        permission_profile: &permission_profile,
        allow_network_for_proxy: false,
        proxy_route_spec: None,
        close_inherited_fd: None,
        command: vec!["/bin/true".to_string()],
    });

    assert!(!args.iter().any(|arg| arg == "--proxy-route-spec"));
}

#[test]
fn managed_proxy_inner_command_requires_route_spec() {
    let result = std::panic::catch_unwind(|| {
        let permission_profile = read_only_permission_profile();
        build_inner_seccomp_command(InnerSeccompCommandArgs {
            sandbox_policy_cwd: Path::new("/tmp"),
            command_cwd: Some(Path::new("/tmp/link")),
            permission_profile: &permission_profile,
            allow_network_for_proxy: true,
            proxy_route_spec: None,
            close_inherited_fd: None,
            command: vec!["/bin/true".to_string()],
        })
    });
    assert!(result.is_err());
}

#[test]
fn resolve_permission_profile_derives_runtime_policies() {
    let permission_profile = read_only_permission_profile();
    let resolved = resolve_permission_profile(Some(permission_profile.clone()))
        .expect("profile should resolve");

    assert_eq!(resolved.permission_profile, permission_profile);
    assert_eq!(
        resolved.file_system_sandbox_policy,
        read_only_file_system_policy()
    );
    assert_eq!(
        resolved.network_sandbox_policy,
        NetworkSandboxPolicy::Restricted
    );
}

#[test]
fn resolve_permission_profile_preserves_direct_runtime_profile() {
    let temp_dir = tempfile::TempDir::new().expect("tempdir");
    let docs = temp_dir.path().join("docs");
    std::fs::create_dir_all(&docs).expect("create docs");
    let docs = AbsolutePathBuf::from_absolute_path(&docs).expect("absolute docs");
    let file_system_sandbox_policy = FileSystemSandboxPolicy::restricted(vec![
        codex_protocol::permissions::FileSystemSandboxEntry {
            path: codex_protocol::permissions::FileSystemPath::Special {
                value: codex_protocol::permissions::FileSystemSpecialPath::Root,
            },
            access: codex_protocol::permissions::FileSystemAccessMode::Read,
            missing_path_behavior: None,
        },
        codex_protocol::permissions::FileSystemSandboxEntry {
            path: codex_protocol::permissions::FileSystemPath::Path { path: docs },
            access: codex_protocol::permissions::FileSystemAccessMode::Write,
            missing_path_behavior: None,
        },
    ]);
    let permission_profile = PermissionProfile::from_runtime_permissions(
        &file_system_sandbox_policy,
        NetworkSandboxPolicy::Restricted,
    );
    let resolved = resolve_permission_profile(Some(permission_profile.clone()))
        .expect("profile should resolve");

    assert_eq!(resolved.permission_profile, permission_profile);
    assert_eq!(
        resolved.file_system_sandbox_policy,
        file_system_sandbox_policy
    );
    assert_eq!(
        resolved.network_sandbox_policy,
        NetworkSandboxPolicy::Restricted
    );
}

#[test]
fn resolve_permission_profile_rejects_missing_configuration() {
    let err = resolve_permission_profile(/*permission_profile*/ None)
        .expect_err("missing profile should fail");

    assert_eq!(err, ResolvePermissionProfileError::MissingConfiguration);
}

#[test]
fn apply_seccomp_then_exec_with_legacy_landlock_panics() {
    let result = std::panic::catch_unwind(|| {
        ensure_inner_stage_mode_is_valid(
            /*apply_seccomp_then_exec*/ true, /*use_legacy_landlock*/ true,
        )
    });
    assert!(result.is_err());
}

#[test]
fn legacy_landlock_rejects_split_only_filesystem_policies() {
    let temp_dir = tempfile::TempDir::new().expect("tempdir");
    let docs = temp_dir.path().join("docs");
    std::fs::create_dir_all(&docs).expect("create docs");
    let docs = AbsolutePathBuf::from_absolute_path(&docs).expect("absolute docs");
    let policy = FileSystemSandboxPolicy::restricted(vec![
        codex_protocol::permissions::FileSystemSandboxEntry {
            path: codex_protocol::permissions::FileSystemPath::Special {
                value: codex_protocol::permissions::FileSystemSpecialPath::Root,
            },
            access: codex_protocol::permissions::FileSystemAccessMode::Read,
            missing_path_behavior: None,
        },
        codex_protocol::permissions::FileSystemSandboxEntry {
            path: codex_protocol::permissions::FileSystemPath::Path { path: docs },
            access: codex_protocol::permissions::FileSystemAccessMode::Write,
            missing_path_behavior: None,
        },
    ]);

    let result = std::panic::catch_unwind(|| {
        ensure_legacy_landlock_mode_supports_policy(
            /*use_legacy_landlock*/ true,
            &policy,
            NetworkSandboxPolicy::Restricted,
            temp_dir.path(),
        );
    });

    assert!(result.is_err());
}

#[test]
fn valid_inner_stage_modes_do_not_panic() {
    ensure_inner_stage_mode_is_valid(
        /*apply_seccomp_then_exec*/ false, /*use_legacy_landlock*/ false,
    );
    ensure_inner_stage_mode_is_valid(
        /*apply_seccomp_then_exec*/ false, /*use_legacy_landlock*/ true,
    );
    ensure_inner_stage_mode_is_valid(
        /*apply_seccomp_then_exec*/ true, /*use_legacy_landlock*/ false,
    );
}
