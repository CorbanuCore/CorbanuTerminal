//! Verify bubblewrap arguments enforce filesystem, namespace, and network policies.

use super::*;

use codex_protocol::models::PermissionProfile;
use codex_protocol::protocol::FileSystemAccessMode;
use codex_protocol::protocol::FileSystemPath;
use codex_protocol::protocol::FileSystemSandboxEntry;
use codex_protocol::protocol::FileSystemSandboxPolicy;
use codex_protocol::protocol::FileSystemSpecialPath;
use codex_utils_absolute_path::AbsolutePathBuf;
use pretty_assertions::assert_eq;
use tempfile::TempDir;

#[test]
fn default_unreadable_glob_scan_has_no_depth_cap() {
    assert_eq!(BwrapOptions::default().glob_scan_max_depth, None);
}

#[test]
fn pid_inheritance_and_legacy_proc_modes_read_only() {
    assert_pid_namespace_args(&PermissionProfile::read_only().file_system_sandbox_policy());
}

#[test]
fn pid_inheritance_and_legacy_proc_modes_full_filesystem() {
    assert_pid_namespace_args(&FileSystemSandboxPolicy::unrestricted());
}

fn assert_pid_namespace_args(policy: &FileSystemSandboxPolicy) {
    for (mount_proc, inherit_pid_namespace) in [(true, false), (false, false), (false, true)] {
        let args = create_bwrap_command_args(
            vec!["/bin/true".to_string()],
            policy,
            Path::new("/"),
            Path::new("/"),
            BwrapOptions {
                mount_proc,
                inherit_pid_namespace,
                network_mode: BwrapNetworkMode::Isolated,
                ..Default::default()
            },
        )
        .expect("create bwrap args")
        .args;

        assert_eq!(
            args.iter().any(|arg| arg == "--unshare-pid"),
            !inherit_pid_namespace
        );
        assert_eq!(args.iter().any(|arg| arg == "--proc"), mount_proc);
        assert!(args.iter().any(|arg| arg == "--unshare-user"));
        assert!(args.iter().any(|arg| arg == "--unshare-ipc"));
        assert!(args.iter().any(|arg| arg == "--unshare-net"));
        assert!(args.windows(2).any(|args| args == ["--cap-drop", "ALL"]));
    }
}

fn unreadable_glob_entry(pattern: String) -> FileSystemSandboxEntry {
    FileSystemSandboxEntry {
        path: FileSystemPath::GlobPattern {
            pattern,
            scan: None,
        },
        access: FileSystemAccessMode::Deny,
        missing_path_behavior: None,
    }
}

fn default_policy_with_unreadable_glob(pattern: String) -> FileSystemSandboxPolicy {
    let mut policy = FileSystemSandboxPolicy::default();
    policy.entries.push(unreadable_glob_entry(pattern));
    policy
}

#[test]
fn full_disk_write_full_network_returns_unwrapped_command() {
    let command = vec!["/bin/true".to_string()];
    let args = create_bwrap_command_args(
        command.clone(),
        &FileSystemSandboxPolicy::unrestricted(),
        Path::new("/"),
        Path::new("/"),
        BwrapOptions {
            mount_proc: true,
            network_mode: BwrapNetworkMode::FullAccess,
            ..Default::default()
        },
    )
    .expect("create bwrap args");

    assert_eq!(args.args, command);
}

#[test]
fn full_disk_write_proxy_only_keeps_full_filesystem_but_unshares_network() {
    let command = vec!["/bin/true".to_string()];
    let args = create_bwrap_command_args(
        command,
        &FileSystemSandboxPolicy::unrestricted(),
        Path::new("/"),
        Path::new("/"),
        BwrapOptions {
            mount_proc: true,
            network_mode: BwrapNetworkMode::ProxyOnly,
            ..Default::default()
        },
    )
    .expect("create bwrap args");

    assert_eq!(
        args.args,
        vec![
            "--new-session".to_string(),
            "--die-with-parent".to_string(),
            "--bind".to_string(),
            "/".to_string(),
            "/".to_string(),
            "--dev".to_string(),
            "/dev".to_string(),
            "--bind-try".to_string(),
            "/dev/shm".to_string(),
            "/dev/shm".to_string(),
            "--unshare-user".to_string(),
            "--unshare-ipc".to_string(),
            "--unshare-pid".to_string(),
            "--unshare-net".to_string(),
            "--proc".to_string(),
            "/proc".to_string(),
            "--cap-drop".to_string(),
            "ALL".to_string(),
            "--".to_string(),
            "/bin/true".to_string(),
        ]
    );
}

#[test]
fn full_disk_write_with_unreadable_glob_still_wraps_and_masks_match() {
    if !ripgrep_available() {
        return;
    }

    let temp_dir = TempDir::new().expect("temp dir");
    let root_env = temp_dir.path().join(".env");
    std::fs::write(&root_env, "secret").expect("write env");
    let root_alias = temp_dir.path().join("root-alias");
    std::os::unix::fs::symlink("/", &root_alias).expect("create root symlink");
    let policy = FileSystemSandboxPolicy::restricted(vec![
        FileSystemSandboxEntry {
            path: FileSystemPath::Special {
                value: FileSystemSpecialPath::Root,
            },
            access: FileSystemAccessMode::Write,
            missing_path_behavior: None,
        },
        unreadable_glob_entry(format!("{}/**/*.env", temp_dir.path().display())),
        FileSystemSandboxEntry::new(
            AbsolutePathBuf::from_absolute_path(root_alias)
                .expect("absolute root alias")
                .into(),
            FileSystemAccessMode::Write,
        ),
        FileSystemSandboxEntry::new(
            AbsolutePathBuf::try_from("/dev/zero")
                .expect("device path")
                .into(),
            FileSystemAccessMode::Deny,
        ),
    ]);
    let command = vec!["/bin/true".to_string()];

    let args = create_bwrap_command_args(
        command.clone(),
        &policy,
        temp_dir.path(),
        temp_dir.path(),
        BwrapOptions::default(),
    )
    .expect("create bwrap args");

    assert_ne!(
        args.args, command,
        "full-write policy with unreadable globs must still use bwrap"
    );
    assert_file_masked(&args.args, &root_env);
    assert_file_masked(&args.args, Path::new("/dev/zero"));
    assert_eq!(
        args.args
            .windows(3)
            .filter(|window| *window == ["--bind", "/", "/"])
            .count(),
        1,
    );
    let writable_root = args
        .args
        .windows(3)
        .position(|window| window == ["--bind", "/", "/"])
        .expect("writable root");
    let devices = args
        .args
        .windows(5)
        .rposition(|window| window == ["--dev", "/dev", "--bind-try", "/dev/shm", "/dev/shm"])
        .expect("minimal device tree with shared memory");
    let device_mask = args
        .args
        .windows(3)
        .position(|window| window[0] == "--ro-bind-data" && window[2] == "/dev/zero")
        .expect("explicit device deny mask");
    assert!(writable_root < devices && devices < device_mask);
}

#[cfg(unix)]
#[test]
fn restricted_policy_chdirs_to_canonical_command_cwd() {
    let temp_dir = TempDir::new().expect("temp dir");
    let real_root = temp_dir.path().join("real");
    let real_subdir = real_root.join("subdir");
    let link_root = temp_dir.path().join("link");
    std::fs::create_dir_all(&real_subdir).expect("create real subdir");
    std::os::unix::fs::symlink(&real_root, &link_root).expect("create symlinked root");

    let sandbox_policy_cwd = AbsolutePathBuf::from_absolute_path(&link_root)
        .expect("absolute symlinked root")
        .to_path_buf();
    let command_cwd = link_root.join("subdir");
    let canonical_command_cwd = real_subdir
        .canonicalize()
        .expect("canonicalize command cwd");
    let policy = FileSystemSandboxPolicy::restricted(vec![
        FileSystemSandboxEntry {
            path: FileSystemPath::Special {
                value: FileSystemSpecialPath::Minimal,
            },
            access: FileSystemAccessMode::Read,
            missing_path_behavior: None,
        },
        FileSystemSandboxEntry {
            path: FileSystemPath::Special {
                value: FileSystemSpecialPath::project_roots(/*subpath*/ None),
            },
            access: FileSystemAccessMode::Write,
            missing_path_behavior: None,
        },
    ]);

    let args = create_bwrap_command_args(
        vec!["/bin/true".to_string()],
        &policy,
        sandbox_policy_cwd.as_path(),
        &command_cwd,
        BwrapOptions::default(),
    )
    .expect("create bwrap args");
    let canonical_sandbox_cwd = path_to_string(
        &real_root
            .canonicalize()
            .expect("canonicalize sandbox policy cwd"),
    );
    let canonical_command_cwd = path_to_string(&canonical_command_cwd);
    let link_sandbox_cwd = path_to_string(&link_root);
    let link_command_cwd = path_to_string(&command_cwd);

    assert!(
        args.args
            .windows(2)
            .any(|window| { window == ["--chdir", canonical_command_cwd.as_str()] })
    );
    assert!(args.args.windows(3).any(|window| {
        window
            == [
                "--ro-bind",
                canonical_sandbox_cwd.as_str(),
                canonical_sandbox_cwd.as_str(),
            ]
    }));
    assert!(
        !args
            .args
            .windows(2)
            .any(|window| { window == ["--chdir", link_command_cwd.as_str()] })
    );
    assert!(!args.args.windows(3).any(|window| {
        window
            == [
                "--ro-bind",
                link_sandbox_cwd.as_str(),
                link_sandbox_cwd.as_str(),
            ]
    }));
}

#[cfg(unix)]
#[test]
fn symlinked_writable_roots_bind_real_target_and_remap_carveouts() {
    let temp_dir = TempDir::new().expect("temp dir");
    let real_root = temp_dir.path().join("real");
    let link_root = temp_dir.path().join("link");
    let blocked = real_root.join("blocked");
    std::fs::create_dir_all(&blocked).expect("create blocked dir");
    std::os::unix::fs::symlink(&real_root, &link_root).expect("create symlinked root");

    let link_root =
        AbsolutePathBuf::from_absolute_path(&link_root).expect("absolute symlinked root");
    let link_blocked = link_root.join("blocked");
    let real_root_str = path_to_string(&real_root);
    let real_blocked_str = path_to_string(&blocked);
    let policy = FileSystemSandboxPolicy::restricted(vec![
        FileSystemSandboxEntry {
            path: link_root.into(),
            access: FileSystemAccessMode::Write,
            missing_path_behavior: None,
        },
        FileSystemSandboxEntry {
            path: link_blocked.into(),
            access: FileSystemAccessMode::Deny,
            missing_path_behavior: None,
        },
    ]);

    let args = create_filesystem_args(&policy, temp_dir.path(), BwrapOptions::default())
        .expect("filesystem args");

    assert!(
        args.args
            .windows(3)
            .any(|window| { window == ["--bind", real_root_str.as_str(), real_root_str.as_str()] })
    );
    assert!(args.args.windows(6).any(|window| {
        window
            == [
                "--perms",
                "000",
                "--tmpfs",
                real_blocked_str.as_str(),
                "--remount-ro",
                real_blocked_str.as_str(),
            ]
    }));
}

/// #488: a denial spelled by the real path is masked after the
/// symlinked root is bound, not skipped as already inside a writable
/// root.
#[cfg(unix)]
#[test]
fn symlinked_writable_roots_mask_denials_spelled_by_the_target() {
    let temp_dir = TempDir::new().expect("temp dir");
    let real_root = temp_dir.path().join("real");
    let link_root = temp_dir.path().join("link");
    let blocked = real_root.join("blocked");
    std::fs::create_dir_all(&blocked).expect("create blocked dir");
    std::os::unix::fs::symlink(&real_root, &link_root).expect("create symlinked root");

    let real_root_str = path_to_string(&real_root);
    let real_blocked_str = path_to_string(&blocked);
    let policy = FileSystemSandboxPolicy::restricted(vec![
        FileSystemSandboxEntry {
            path: FileSystemPath::from(
                AbsolutePathBuf::from_absolute_path(&link_root).expect("absolute"),
            ),
            access: FileSystemAccessMode::Write,
            missing_path_behavior: None,
        },
        FileSystemSandboxEntry {
            path: FileSystemPath::from(
                AbsolutePathBuf::from_absolute_path(&blocked).expect("absolute"),
            ),
            access: FileSystemAccessMode::Deny,
            missing_path_behavior: None,
        },
    ]);

    let args = create_filesystem_args(&policy, temp_dir.path(), BwrapOptions::default())
        .expect("filesystem args");

    let bind = args
        .args
        .windows(3)
        .position(|window| window == ["--bind", real_root_str.as_str(), real_root_str.as_str()])
        .expect("bind");
    let mask = args
        .args
        .windows(4)
        .position(|window| window == ["--perms", "000", "--tmpfs", real_blocked_str.as_str()])
        .expect("mask");
    assert!(bind < mask, "{:?}", args.args);
    assert_eq!(
        args.args
            .iter()
            .filter(|arg| arg.as_str() == real_blocked_str)
            .count(),
        2,
        "masked once: {:?}",
        args.args
    );
}

/// #502: a denial spelled through an absolute link to the workspace is
/// masked at the link's target after the workspace bind; bubblewrap is
/// never asked to create a mount point through the link.
#[cfg(unix)]
#[test]
fn denials_spelled_through_a_read_only_link_mask_the_target() {
    let temp_dir = TempDir::new().expect("temp dir");
    let real_root = temp_dir.path().join("real");
    let link_root = temp_dir.path().join("link");
    std::fs::create_dir_all(&real_root).expect("create real root");
    std::fs::write(real_root.join(".env"), "x").expect("create .env");
    std::os::unix::fs::symlink(&real_root, &link_root).expect("create link");
    let entry = |path: &Path, access| FileSystemSandboxEntry {
        path: FileSystemPath::from(AbsolutePathBuf::from_absolute_path(path).expect("absolute")),
        access,
        missing_path_behavior: None,
    };
    // Existing, and not created yet.
    for name in [".env", "later.key"] {
        let policy = FileSystemSandboxPolicy::restricted(vec![
            entry(&real_root, FileSystemAccessMode::Write),
            entry(&link_root.join(name), FileSystemAccessMode::Deny),
        ]);
        let args = create_filesystem_args(&policy, temp_dir.path(), BwrapOptions::default())
            .expect("filesystem args");
        let real_root_str = path_to_string(&real_root);
        let masked = path_to_string(&real_root.join(name));
        let bind = args
            .args
            .windows(3)
            .position(|window| window == ["--bind", real_root_str.as_str(), real_root_str.as_str()])
            .expect("bind");
        let mask = args
            .args
            .iter()
            .position(|arg| arg == &masked)
            .expect("masked at the target");
        assert!(bind < mask, "{name}: {:?}", args.args);
        let through_link = path_to_string(&link_root.join(name));
        assert!(
            !args.args.contains(&through_link),
            "{name}: {:?}",
            args.args
        );
    }
}

/// #501: under restricted reads, a read-only root reached through a link
/// is mounted at the link's path; a denial below its target, spelled
/// either way, is masked at that path too.
#[cfg(unix)]
#[test]
fn restricted_reads_mask_denials_below_a_linked_read_only_root() {
    let temp_dir = TempDir::new().expect("temp dir");
    let real_root = temp_dir.path().join("real");
    let link_root = temp_dir.path().join("link");
    std::fs::create_dir_all(&real_root).expect("create real root");
    std::fs::write(real_root.join("secret"), "x").expect("create secret");
    std::os::unix::fs::symlink(&real_root, &link_root).expect("create link");
    let entry = |path: &Path, access| FileSystemSandboxEntry {
        path: FileSystemPath::from(AbsolutePathBuf::from_absolute_path(path).expect("absolute")),
        access,
        missing_path_behavior: None,
    };
    let link_root_str = path_to_string(&link_root);
    let link_secret = path_to_string(&link_root.join("secret"));
    for denied in [real_root.join("secret"), link_root.join("secret")] {
        let policy = FileSystemSandboxPolicy::restricted(vec![
            entry(&link_root, FileSystemAccessMode::Read),
            entry(&denied, FileSystemAccessMode::Deny),
        ]);
        let args = create_filesystem_args(&policy, temp_dir.path(), BwrapOptions::default())
            .expect("filesystem args");
        let bind = args
            .args
            .windows(3)
            .position(|window| {
                window == ["--ro-bind", link_root_str.as_str(), link_root_str.as_str()]
            })
            .expect("linked root mounted at its path");
        let mask = args
            .args
            .iter()
            .rposition(|arg| arg == &link_secret)
            .expect("masked through the mounted link");
        assert!(bind < mask, "{denied:?}: {:?}", args.args);
    }
}

/// #501 review: a denial holding a linked read-only root's target masks
/// the whole root at its link path; a missing denial below it runs.
#[cfg(unix)]
#[test]
fn restricted_reads_mask_a_linked_root_inside_a_denied_folder() {
    let temp_dir = TempDir::new().expect("temp dir");
    let parent = temp_dir.path().join("parent");
    let real_root = parent.join("real");
    let link_root = temp_dir.path().join("link");
    std::fs::create_dir_all(&real_root).expect("create real root");
    std::fs::write(real_root.join("secret"), "x").expect("create secret");
    std::os::unix::fs::symlink(&real_root, &link_root).expect("create link");
    let entry = |path: &Path, access| FileSystemSandboxEntry {
        path: FileSystemPath::from(AbsolutePathBuf::from_absolute_path(path).expect("absolute")),
        access,
        missing_path_behavior: None,
    };
    let link_root_str = path_to_string(&link_root);
    for denied in [parent, real_root.clone()] {
        let policy = FileSystemSandboxPolicy::restricted(vec![
            entry(&link_root, FileSystemAccessMode::Read),
            entry(&denied, FileSystemAccessMode::Deny),
        ]);
        let args = create_filesystem_args(&policy, temp_dir.path(), BwrapOptions::default())
            .expect("filesystem args");
        let bind = args
            .args
            .windows(3)
            .position(|window| {
                window == ["--ro-bind", link_root_str.as_str(), link_root_str.as_str()]
            })
            .expect("linked root mounted at its path");
        let mask = args
            .args
            .windows(2)
            .rposition(|window| window == ["--tmpfs", link_root_str.as_str()])
            .expect("linked root masked whole");
        assert!(bind < mask, "{denied:?}: {:?}", args.args);
    }
    let policy = FileSystemSandboxPolicy::restricted(vec![
        entry(&link_root, FileSystemAccessMode::Read),
        entry(&real_root.join("later.key"), FileSystemAccessMode::Deny),
    ]);
    create_filesystem_args(&policy, temp_dir.path(), BwrapOptions::default())
        .expect("a missing denial below a linked root runs");
}

/// A denial that is itself a read-only link is masked at its target; a
/// denial inside a folder masked whole is not masked again (bubblewrap
/// could not create it in the read-only mask); a link inside a writable
/// root reached through a read-only link keeps today's handling.
#[cfg(unix)]
#[test]
fn denials_through_links_mask_once_and_keep_writable_links() {
    let temp_dir = TempDir::new().expect("temp dir");
    let base = temp_dir.path();
    let secrets = base.join("secrets");
    std::fs::create_dir_all(&secrets).expect("secrets");
    std::fs::write(secrets.join("key"), "x").expect("key");
    std::os::unix::fs::symlink(secrets.join("key"), base.join("key-link")).expect("link");
    std::os::unix::fs::symlink(&secrets, base.join("secrets-link")).expect("link");
    let entry = |path: &Path, access| FileSystemSandboxEntry {
        path: FileSystemPath::from(AbsolutePathBuf::from_absolute_path(path).expect("absolute")),
        access,
        missing_path_behavior: None,
    };
    let key = path_to_string(&secrets.join("key"));
    let secrets_str = path_to_string(&secrets);
    let policy = FileSystemSandboxPolicy::restricted(vec![
        FileSystemSandboxEntry {
            path: FileSystemPath::Special {
                value: FileSystemSpecialPath::Root,
            },
            access: FileSystemAccessMode::Read,
            missing_path_behavior: None,
        },
        entry(&base.join("key-link"), FileSystemAccessMode::Deny),
    ]);
    let args =
        create_filesystem_args(&policy, base, BwrapOptions::default()).expect("filesystem args");
    assert!(args.args.contains(&key), "{:?}", args.args);
    let policy = FileSystemSandboxPolicy::restricted(vec![
        FileSystemSandboxEntry {
            path: FileSystemPath::Special {
                value: FileSystemSpecialPath::Root,
            },
            access: FileSystemAccessMode::Read,
            missing_path_behavior: None,
        },
        entry(&base.join("secrets-link"), FileSystemAccessMode::Deny),
        entry(&secrets.join("key"), FileSystemAccessMode::Deny),
    ]);
    let args =
        create_filesystem_args(&policy, base, BwrapOptions::default()).expect("filesystem args");
    assert!(
        args.args
            .windows(2)
            .any(|window| window == ["--tmpfs", secrets_str.as_str()]),
        "{:?}",
        args.args
    );
    assert!(!args.args.contains(&key), "{:?}", args.args);

    // `/w/link -> /w/real` is read-only; `/w/real/sub` is a link inside
    // the writable `real`: not resolved, so it still fails closed or
    // masks as before rather than snapshotting the writable link.
    let real = base.join("w").join("real");
    std::fs::create_dir_all(&real).expect("real");
    std::fs::create_dir_all(base.join("t")).expect("t");
    std::os::unix::fs::symlink(base.join("t"), real.join("sub")).expect("sub");
    std::os::unix::fs::symlink(&real, base.join("w").join("link")).expect("link");
    assert_eq!(
        real_path_through_links(
            &base.join("w").join("link").join("sub").join(".env"),
            std::slice::from_ref(&real)
        ),
        None
    );
    assert_eq!(
        real_path_through_links(&base.join("w").join("link").join(".env"), &[]),
        Some(real.join(".env"))
    );
}

#[cfg(unix)]
#[test]
fn writable_roots_under_symlinked_ancestors_bind_real_target() {
    let temp_dir = TempDir::new().expect("temp dir");
    let logical_home = temp_dir.path().join("home");
    let real_codex = temp_dir.path().join("real-codex");
    let logical_codex = logical_home.join(".codex");
    let real_memories = real_codex.join("memories");
    let logical_memories = logical_codex.join("memories");
    std::fs::create_dir_all(&logical_home).expect("create logical home");
    std::fs::create_dir_all(&real_memories).expect("create memories dir");
    std::os::unix::fs::symlink(&real_codex, &logical_codex).expect("create symlinked codex home");

    let logical_memories_root =
        AbsolutePathBuf::from_absolute_path(&logical_memories).expect("absolute memories");
    let real_memories_str = path_to_string(&real_memories);
    let logical_memories_str = path_to_string(&logical_memories);
    let policy = FileSystemSandboxPolicy::restricted(vec![FileSystemSandboxEntry {
        path: logical_memories_root.into(),
        access: FileSystemAccessMode::Write,
        missing_path_behavior: None,
    }]);

    let args = create_filesystem_args(&policy, temp_dir.path(), BwrapOptions::default())
        .expect("filesystem args");

    assert!(args.args.windows(3).any(|window| {
        window
            == [
                "--bind",
                real_memories_str.as_str(),
                real_memories_str.as_str(),
            ]
    }));
    assert!(!args.args.windows(3).any(|window| {
        window
            == [
                "--bind",
                logical_memories_str.as_str(),
                logical_memories_str.as_str(),
            ]
    }));
}

#[cfg(unix)]
#[test]
fn protected_symlinked_directory_subpaths_fail_closed() {
    let temp_dir = TempDir::new().expect("temp dir");
    let root = temp_dir.path().join("root");
    let agents_target = root.join("agents-target");
    let agents_link = root.join(".agents");
    std::fs::create_dir_all(&agents_target).expect("create agents target");
    std::os::unix::fs::symlink(&agents_target, &agents_link).expect("create symlinked .agents");

    let root = AbsolutePathBuf::from_absolute_path(&root).expect("absolute root");
    let agents_link_str = path_to_string(&agents_link);
    let policy = FileSystemSandboxPolicy::restricted(vec![FileSystemSandboxEntry {
        path: root.into(),
        access: FileSystemAccessMode::Write,
        missing_path_behavior: None,
    }]);

    let err = create_filesystem_args(&policy, temp_dir.path(), BwrapOptions::default())
        .expect_err("protected symlinked subpath should fail closed");
    let message = err.to_string();

    assert!(
        message.contains("cannot enforce sandbox read-only path"),
        "{message}"
    );
    assert!(message.contains(&agents_link_str), "{message}");
}

#[cfg(unix)]
#[test]
fn symlinked_writable_roots_nested_symlink_escape_paths_fail_closed() {
    let temp_dir = TempDir::new().expect("temp dir");
    let real_root = temp_dir.path().join("real");
    let link_root = temp_dir.path().join("link");
    let outside = temp_dir.path().join("outside-private");
    let linked_private = real_root.join("linked-private");
    std::fs::create_dir_all(&real_root).expect("create real root");
    std::fs::create_dir_all(&outside).expect("create outside dir");
    std::os::unix::fs::symlink(&real_root, &link_root).expect("create symlinked root");
    std::os::unix::fs::symlink(&outside, &linked_private).expect("create nested escape symlink");

    let link_root =
        AbsolutePathBuf::from_absolute_path(&link_root).expect("absolute symlinked root");
    let link_private = link_root.join("linked-private");
    let real_linked_private_str = path_to_string(&linked_private);
    let policy = FileSystemSandboxPolicy::restricted(vec![
        FileSystemSandboxEntry {
            path: link_root.into(),
            access: FileSystemAccessMode::Write,
            missing_path_behavior: None,
        },
        FileSystemSandboxEntry {
            path: link_private.into(),
            access: FileSystemAccessMode::Deny,
            missing_path_behavior: None,
        },
    ]);

    let err = create_filesystem_args(&policy, temp_dir.path(), BwrapOptions::default())
        .expect_err("deny-read path crossing writable symlink should fail closed");
    let message = err.to_string();

    assert!(
        message.contains("cannot enforce sandbox deny-read path"),
        "{message}"
    );
    assert!(message.contains(&real_linked_private_str), "{message}");
}

#[test]
fn registry_protection_precedes_denied_parent_mask() {
    let registry_root = synthetic_mount_registry_root();
    let temp_root = registry_root.parent().expect("registry parent");
    let policy = FileSystemSandboxPolicy::restricted(vec![
        FileSystemSandboxEntry {
            path: FileSystemPath::Special {
                value: FileSystemSpecialPath::Root,
            },
            access: FileSystemAccessMode::Write,
            missing_path_behavior: None,
        },
        FileSystemSandboxEntry {
            path: AbsolutePathBuf::try_from(temp_root)
                .expect("absolute temp root")
                .into(),
            access: FileSystemAccessMode::Deny,
            missing_path_behavior: None,
        },
    ]);

    let args = create_filesystem_args(&policy, Path::new("/"), BwrapOptions::default())
        .expect("filesystem args");
    let registry_path = path_to_string(&registry_root);
    let temp_path = path_to_string(temp_root);
    let registry_bind = args
        .args
        .windows(3)
        .position(|args| args == ["--ro-bind", registry_path.as_str(), registry_path.as_str()]);
    let temp_mask = args
        .args
        .windows(2)
        .position(|args| args == ["--tmpfs", temp_path.as_str()]);
    assert!(
        registry_bind.expect("registry read-only bind") < temp_mask.expect("temp deny mask"),
        "registry protection must not reopen denied temp paths"
    );
}

#[test]
fn missing_read_only_subpath_uses_empty_file_bind_data() {
    let temp_dir = TempDir::new().expect("temp dir");
    let workspace = temp_dir.path().join("workspace");
    let blocked = workspace.join("blocked");
    let second_blocked = workspace.join("second-blocked");
    std::fs::create_dir_all(&workspace).expect("create workspace");

    let workspace_root =
        AbsolutePathBuf::from_absolute_path(&workspace).expect("absolute workspace");
    let blocked_root = AbsolutePathBuf::from_absolute_path(&blocked).expect("absolute blocked");
    let policy = FileSystemSandboxPolicy::restricted(vec![
        FileSystemSandboxEntry {
            path: workspace_root.into(),
            access: FileSystemAccessMode::Write,
            missing_path_behavior: None,
        },
        FileSystemSandboxEntry {
            path: blocked_root.into(),
            access: FileSystemAccessMode::Read,
            missing_path_behavior: None,
        },
        FileSystemSandboxEntry {
            path: AbsolutePathBuf::from_absolute_path(&second_blocked)
                .expect("absolute second blocked")
                .into(),
            access: FileSystemAccessMode::Read,
            missing_path_behavior: None,
        },
    ]);

    let args = create_filesystem_args(&policy, temp_dir.path(), BwrapOptions::default())
        .expect("filesystem args");

    assert_empty_file_bound_without_perms(&args.args, &blocked);
    assert_empty_file_bound_without_perms(&args.args, &second_blocked);
    assert_empty_directory_mounted_read_only(&args.args, &workspace.join(".git"));
    assert_empty_directory_mounted_read_only(&args.args, &workspace.join(".agents"));
    assert_empty_directory_mounted_read_only(&args.args, &workspace.join(".codex"));
    assert_eq!(args.preserved_files.len(), 2);
    assert_bind_data_uses_distinct_preserved_fds(&args);
    assert_eq!(
        synthetic_mount_target_paths(&args),
        vec![
            blocked.clone(),
            second_blocked.clone(),
            workspace.join(".git"),
            workspace.join(".agents"),
            workspace.join(".codex"),
            workspace.join(".aws"),
        ]
    );
    assert!(
        !blocked.exists() && !second_blocked.exists(),
        "missing path mask should not materialize host-side metadata paths at arg construction time",
    );
}

#[test]
fn transient_empty_preserved_file_uses_empty_file_bind_data() {
    let temp_dir = TempDir::new().expect("temp dir");
    let workspace = temp_dir.path().join("workspace");
    let dot_git = workspace.join(".git");
    std::fs::create_dir_all(&workspace).expect("create workspace");
    File::create(&dot_git).expect("create empty .git file");

    let workspace_root =
        AbsolutePathBuf::from_absolute_path(&workspace).expect("absolute workspace");
    let policy = FileSystemSandboxPolicy::restricted(vec![FileSystemSandboxEntry {
        path: workspace_root.into(),
        access: FileSystemAccessMode::Write,
        missing_path_behavior: None,
    }]);

    let args = create_filesystem_args(&policy, temp_dir.path(), BwrapOptions::default())
        .expect("filesystem args");
    let dot_git_str = path_to_string(&dot_git);

    assert_empty_file_bound_without_perms(&args.args, &dot_git);
    assert_empty_directory_mounted_read_only(&args.args, &workspace.join(".agents"));
    assert_empty_directory_mounted_read_only(&args.args, &workspace.join(".codex"));
    assert_eq!(
        synthetic_mount_target_paths(&args),
        vec![
            dot_git.clone(),
            workspace.join(".agents"),
            workspace.join(".codex"),
            workspace.join(".aws"),
        ]
    );
    assert!(
        !args
            .args
            .windows(3)
            .any(|window| window == ["--ro-bind", dot_git_str.as_str(), dot_git_str.as_str()]),
        "transient empty preserved file should not be treated as a stable bind source",
    );
    let metadata = std::fs::symlink_metadata(&dot_git).expect("stat .git");
    assert!(
        !args.synthetic_mount_targets[0].should_remove(
            metadata.file_type().is_file() && metadata.len() == 0,
            metadata.file_type().is_dir(),
            (metadata.dev(), metadata.ino()),
        ),
        "pre-existing empty preserved files must not be cleaned up as synthetic targets",
    );
}

#[test]
fn missing_child_git_under_parent_repo_is_mounted_read_only() {
    let temp_dir = TempDir::new().expect("temp dir");
    let repo = temp_dir.path().join("repo");
    let workspace = repo.join("workspace");
    let dot_git = workspace.join(".git");
    std::fs::create_dir_all(repo.join(".git")).expect("create parent .git");
    std::fs::write(repo.join(".git/HEAD"), "ref: refs/heads/main\n").expect("write HEAD");
    std::fs::create_dir_all(&workspace).expect("create workspace");

    let workspace_root =
        AbsolutePathBuf::from_absolute_path(&workspace).expect("absolute workspace");
    let policy = FileSystemSandboxPolicy::restricted(vec![FileSystemSandboxEntry {
        path: workspace_root.into(),
        access: FileSystemAccessMode::Write,
        missing_path_behavior: None,
    }]);

    let args = create_filesystem_args(&policy, &workspace, BwrapOptions::default())
        .expect("filesystem args");
    assert_empty_directory_mounted_read_only(&args.args, &dot_git);
    assert_empty_directory_mounted_read_only(&args.args, &workspace.join(".agents"));
    assert_empty_directory_mounted_read_only(&args.args, &workspace.join(".codex"));
    assert_eq!(
        synthetic_mount_target_paths(&args),
        vec![
            workspace.join(".codex"),
            dot_git,
            workspace.join(".agents"),
            workspace.join(".aws"),
        ],
    );
}

#[cfg(unix)]
#[test]
fn symlinked_missing_child_git_under_parent_repo_mounts_effective_path_read_only() {
    let temp_dir = TempDir::new().expect("temp dir");
    let repo = temp_dir.path().join("repo");
    let workspace = repo.join("workspace");
    let link_repo = temp_dir.path().join("link-repo");
    let link_workspace = link_repo.join("workspace");
    let dot_git = workspace.join(".git");
    std::fs::create_dir_all(repo.join(".git")).expect("create parent .git");
    std::fs::write(repo.join(".git/HEAD"), "ref: refs/heads/main\n").expect("write HEAD");
    std::fs::create_dir_all(&workspace).expect("create workspace");
    std::os::unix::fs::symlink(&repo, &link_repo).expect("create symlinked repo");

    let link_workspace_root =
        AbsolutePathBuf::from_absolute_path(&link_workspace).expect("absolute symlinked workspace");
    let policy = FileSystemSandboxPolicy::restricted(vec![FileSystemSandboxEntry {
        path: link_workspace_root.into(),
        access: FileSystemAccessMode::Write,
        missing_path_behavior: None,
    }]);

    let args = create_filesystem_args(&policy, &link_workspace, BwrapOptions::default())
        .expect("filesystem args");
    assert_empty_directory_mounted_read_only(&args.args, &dot_git);
    assert_empty_directory_mounted_read_only(&args.args, &workspace.join(".agents"));
    assert_empty_directory_mounted_read_only(&args.args, &workspace.join(".codex"));
    assert_eq!(
        synthetic_mount_target_paths(&args),
        vec![
            workspace.join(".codex"),
            dot_git,
            workspace.join(".agents"),
            workspace.join(".aws"),
        ],
    );
}

#[test]
fn ignores_missing_writable_roots() {
    let temp_dir = TempDir::new().expect("temp dir");
    let existing_root = temp_dir.path().join("existing");
    let missing_root = temp_dir.path().join("missing");
    std::fs::create_dir(&existing_root).expect("create existing root");

    let policy = FileSystemSandboxPolicy::workspace_write(
        &[
            AbsolutePathBuf::try_from(existing_root.as_path()).expect("absolute existing root"),
            AbsolutePathBuf::try_from(missing_root.as_path()).expect("absolute missing root"),
        ],
        /*exclude_tmpdir_env_var*/ true,
        /*exclude_slash_tmp*/ true,
    );

    let args = create_filesystem_args(&policy, temp_dir.path(), BwrapOptions::default())
        .expect("filesystem args");
    let existing_root = path_to_string(&existing_root);
    let missing_root = path_to_string(&missing_root);

    assert!(
        args.args
            .windows(3)
            .any(|window| { window == ["--bind", existing_root.as_str(), existing_root.as_str()] }),
        "existing writable root should be rebound writable",
    );
    assert!(
        !args.args.iter().any(|arg| arg == &missing_root),
        "missing writable root should be skipped",
    );
    assert_eq!(
        args.args.iter().filter(|arg| *arg == "--dev").count(),
        1,
        "a writable root below `/dev`'s parent keeps the single minimal `/dev`",
    );
}

/// #493: the writable `/` of a full-write profile with denials is
/// followed by a fresh minimal `/dev`, before the deny masks and before a
/// deeper writable `/dev/shm`.
#[test]
fn full_write_root_bind_is_followed_by_minimal_dev() {
    let temp_dir = TempDir::new().expect("temp dir");
    let denied = temp_dir.path().join("secret");
    std::fs::create_dir(&denied).expect("create denied dir");
    let entry = |path: &Path, access| FileSystemSandboxEntry {
        path: FileSystemPath::from(AbsolutePathBuf::try_from(path).expect("absolute path")),
        access,
        missing_path_behavior: None,
    };
    let policy = FileSystemSandboxPolicy::restricted(vec![
        FileSystemSandboxEntry {
            path: FileSystemPath::Special {
                value: FileSystemSpecialPath::Root,
            },
            access: FileSystemAccessMode::Write,
            missing_path_behavior: None,
        },
        entry(&denied, FileSystemAccessMode::Deny),
        entry(Path::new("/dev/full"), FileSystemAccessMode::Deny),
        entry(Path::new("/dev/shm"), FileSystemAccessMode::Write),
    ]);

    let args = create_filesystem_args(&policy, temp_dir.path(), BwrapOptions::default())
        .expect("filesystem args");
    let position = |window: &[&str]| {
        args.args
            .windows(window.len())
            .position(|candidate| candidate == window)
    };
    let root_bind = position(&["--bind", "/", "/", "--dev", "/dev"])
        .expect("writable `/` should be followed by `--dev /dev`");
    let denied = path_to_string(&denied);
    let deny_mask = args
        .args
        .iter()
        .rposition(|arg| arg == &denied)
        .expect("denied path should be masked");
    assert!(root_bind < deny_mask, "{:?}", args.args);
    // A denied `/dev` node is masked on top of the fresh `/dev`.
    let dev_mask = args
        .args
        .iter()
        .rposition(|arg| arg == "/dev/full")
        .expect("denied /dev/full should be masked");
    assert!(root_bind < dev_mask, "{:?}", args.args);
    if Path::new("/dev/shm").exists() {
        let shm_bind = position(&["--bind", "/dev/shm", "/dev/shm"])
            .expect("writable `/dev/shm` should be rebound");
        assert!(root_bind < shm_bind, "{:?}", args.args);
    }
}

/// #503: a writable root that is a symlink to `/` is ordered by its
/// target, so it is bound before shallower roots instead of covering
/// them (and their masks) afterwards, and is followed by a fresh `/dev`.
#[cfg(unix)]
#[test]
fn writable_symlink_to_filesystem_root_is_bound_first() {
    let temp_dir = TempDir::new().expect("temp dir");
    let workspace = temp_dir.path().join("ws");
    let denied_ancestor = temp_dir.path().join("a");
    let link = denied_ancestor.join("b").join("c").join("root-link");
    let denied = workspace.join("secret");
    std::fs::create_dir_all(link.parent().expect("link parent")).expect("create link parent");
    std::fs::create_dir(&workspace).expect("create workspace");
    std::fs::write(&denied, "secret").expect("create denied file");
    std::os::unix::fs::symlink("/", &link).expect("create symlink to /");
    let entry = |path: &Path, access| FileSystemSandboxEntry {
        path: FileSystemPath::from(AbsolutePathBuf::try_from(path).expect("absolute path")),
        access,
        missing_path_behavior: None,
    };
    let policy = FileSystemSandboxPolicy::restricted(vec![
        entry(&workspace, FileSystemAccessMode::Write),
        entry(&link, FileSystemAccessMode::Write),
        entry(&denied, FileSystemAccessMode::Deny),
        entry(&denied_ancestor, FileSystemAccessMode::Deny),
    ]);

    let args = create_filesystem_args(&policy, temp_dir.path(), BwrapOptions::default())
        .expect("filesystem args");
    let position = |window: &[&str]| {
        args.args
            .windows(window.len())
            .position(|candidate| candidate == window)
    };
    let root_bind = position(&["--bind", "/", "/", "--dev", "/dev"])
        .expect("the symlink to `/` should bind `/` followed by `--dev /dev`");
    let workspace = path_to_string(&workspace);
    let workspace_bind = position(&["--bind", workspace.as_str(), workspace.as_str()])
        .expect("workspace should be bound");
    assert!(root_bind < workspace_bind, "{:?}", args.args);
    let denied = path_to_string(&denied);
    let deny_mask = args
        .args
        .iter()
        .rposition(|arg| arg == &denied)
        .expect("denied file should be masked");
    assert!(workspace_bind < deny_mask, "{:?}", args.args);
    // The link is bound at its target, never at its own spelling, and its
    // denied ancestor is masked after the `/` bind.
    let link = path_to_string(&link);
    assert_eq!(
        position(&["--bind", link.as_str(), link.as_str()]),
        None,
        "{:?}",
        args.args
    );
    let denied_ancestor = path_to_string(&denied_ancestor);
    let ancestor_mask = args
        .args
        .iter()
        .rposition(|arg| arg == &denied_ancestor)
        .expect("denied ancestor should be masked");
    assert!(root_bind < ancestor_mask, "{:?}", args.args);
}

/// #503: a link whose target lies inside another writable root is bound
/// after it (deeper real path) and re-applies that root's read-only paths
/// below the target, which its own bind would otherwise cover.
#[cfg(unix)]
#[test]
fn symlinked_root_inside_another_root_reapplies_its_read_only_paths() {
    let temp_dir = TempDir::new().expect("temp dir");
    let outer = temp_dir.path().join("outer");
    let target = outer.join("c").join("d");
    let read_only = target.join("ro");
    let link = temp_dir.path().join("link");
    std::fs::create_dir_all(&read_only).expect("create read-only dir");
    std::os::unix::fs::symlink(&target, &link).expect("create link");
    let entry = |path: &Path, access| FileSystemSandboxEntry {
        path: FileSystemPath::from(AbsolutePathBuf::try_from(path).expect("absolute path")),
        access,
        missing_path_behavior: None,
    };
    let policy = FileSystemSandboxPolicy::restricted(vec![
        entry(&outer, FileSystemAccessMode::Write),
        entry(&link, FileSystemAccessMode::Write),
        entry(&read_only, FileSystemAccessMode::Read),
    ]);

    let args = create_filesystem_args(&policy, temp_dir.path(), BwrapOptions::default())
        .expect("filesystem args");
    let target = path_to_string(&target);
    let target_bind = args
        .args
        .windows(3)
        .position(|window| window == ["--bind", target.as_str(), target.as_str()])
        .expect("link should be bound at its target");
    let read_only = path_to_string(&read_only);
    let read_only_bind = args
        .args
        .windows(3)
        .rposition(|window| window == ["--ro-bind", read_only.as_str(), read_only.as_str()])
        .expect("read-only path should be bound read-only");
    assert!(target_bind < read_only_bind, "{:?}", args.args);
}

/// #536: with pinned roots, each writable root is bound with `--bind-fd`
/// from the folder opened at its pinned location (never `--bind` by
/// path), each `--ro-bind-data` mask reads its own `/dev/null`
/// descriptor, a root swapped for a link is refused, and so is a
/// bubblewrap without `--bind-fd`.
#[test]
fn pinned_writable_roots_are_bound_from_checked_folders() {
    let temp_dir = TempDir::new().expect("temp dir");
    let base = temp_dir.path().canonicalize().expect("canonical");
    let workspace = base.join("ws");
    std::fs::create_dir_all(&workspace).expect("workspace");
    let secret = workspace.join("secret.txt");
    std::fs::write(&secret, "x").expect("secret");
    let entry = |path: &Path, access| FileSystemSandboxEntry {
        path: FileSystemPath::from(AbsolutePathBuf::try_from(path).expect("absolute path")),
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
        entry(&workspace, FileSystemAccessMode::Write),
        entry(&secret, FileSystemAccessMode::Deny),
    ]);
    let roots: &'static [PinnedWritableRoot] = Box::leak(
        vec![PinnedWritableRoot {
            root: AbsolutePathBuf::try_from(workspace.as_path()).expect("absolute"),
            real: workspace.clone(),
        }]
        .into_boxed_slice(),
    );
    let pins = |bind_fd| Some(WritableRootPins { roots, bind_fd });
    let args = create_filesystem_args(
        &policy,
        &workspace,
        BwrapOptions {
            pins: pins(true),
            ..BwrapOptions::default()
        },
    )
    .expect("filesystem args");
    let ws = path_to_string(&workspace);
    assert!(
        !args
            .args
            .windows(3)
            .any(|window| window == ["--bind", ws.as_str(), ws.as_str()]),
        "{:?}",
        args.args
    );
    let bind = args
        .args
        .windows(3)
        .find(|window| window[0] == "--bind-fd" && window[2] == ws)
        .expect("bound with --bind-fd");
    let fd_target =
        |fd: &str| std::fs::read_link(format!("/proc/self/fd/{fd}")).expect("preserved fd");
    assert_eq!(fd_target(&bind[1]), workspace);
    // The masks read `/dev/null`, each through its own descriptor.
    let masks = args
        .args
        .windows(3)
        .filter(|window| window[0] == "--ro-bind-data")
        .map(|window| window[1].clone())
        .collect::<Vec<_>>();
    assert!(!masks.is_empty(), "{:?}", args.args);
    for fd in &masks {
        assert_eq!(fd_target(fd), PathBuf::from("/dev/null"));
    }
    assert_eq!(
        masks.iter().collect::<std::collections::HashSet<_>>().len(),
        masks.len(),
        "{masks:?}"
    );

    assert!(
        create_filesystem_args(
            &policy,
            &workspace,
            BwrapOptions {
                pins: pins(false),
                ..BwrapOptions::default()
            }
        )
        .is_err()
    );
    std::fs::rename(&workspace, base.join("moved")).expect("move away");
    std::fs::create_dir(base.join("elsewhere")).expect("elsewhere");
    std::os::unix::fs::symlink(base.join("elsewhere"), &workspace).expect("link");
    let err = create_filesystem_args(
        &policy,
        &workspace,
        BwrapOptions {
            pins: pins(true),
            ..BwrapOptions::default()
        },
    )
    .expect_err("a swapped root is refused");
    assert!(
        err.to_string().contains("Refused to run the command"),
        "{err}"
    );
}

/// #503: a writable link to a workspace's `.git` cannot make it writable;
/// since #535 the link is no writable root at all.
#[cfg(unix)]
#[test]
fn writable_link_to_protected_git_keeps_it_read_only() {
    let temp_dir = TempDir::new().expect("temp dir");
    let workspace = temp_dir.path().join("p").join("q").join("w");
    let git = workspace.join(".git");
    let link = temp_dir.path().join("l");
    std::fs::create_dir_all(&git).expect("create .git");
    std::fs::write(git.join("HEAD"), "ref: refs/heads/main\n").expect("write HEAD");
    std::os::unix::fs::symlink(&git, &link).expect("create link");
    let entry = |path: &Path| FileSystemSandboxEntry {
        path: FileSystemPath::from(AbsolutePathBuf::try_from(path).expect("absolute path")),
        access: FileSystemAccessMode::Write,
        missing_path_behavior: None,
    };
    let policy = FileSystemSandboxPolicy::restricted(vec![entry(&workspace), entry(&link)]);

    let args = create_filesystem_args(&policy, temp_dir.path(), BwrapOptions::default())
        .expect("filesystem args");
    let git = path_to_string(&git);
    // #535 (Travis, 2026-10-12): the link is no grant, so `.git` is never
    // bound writable, and stays read-only.
    assert!(
        !args
            .args
            .windows(3)
            .any(|window| window == ["--bind", git.as_str(), git.as_str()]),
        "{:?}",
        args.args
    );
    assert!(
        args.args
            .windows(3)
            .any(|window| window == ["--ro-bind", git.as_str(), git.as_str()]),
        "{:?}",
        args.args
    );
}

/// #503: a denial spelled through a link is masked at the link's target.
/// A writable root bound later inside that target (deeper, or the same
/// directory) masks it again after its own bind.
#[cfg(unix)]
#[test]
fn denial_spelled_through_a_link_survives_later_binds_of_its_target() {
    let entry = |path: &Path, access| FileSystemSandboxEntry {
        path: FileSystemPath::from(AbsolutePathBuf::try_from(path).expect("absolute path")),
        access,
        missing_path_behavior: None,
    };
    let masked_after_last_bind = |args: &BwrapArgs, bound: &Path, denied: &Path| {
        let bound = path_to_string(bound);
        let denied = path_to_string(denied);
        let last_bind = args
            .args
            .windows(3)
            .rposition(|window| window == ["--bind", bound.as_str(), bound.as_str()])
            .expect("writable root should be bound");
        let deny_mask = args
            .args
            .iter()
            .rposition(|arg| arg == &denied)
            .expect("denial should be masked at the link target");
        assert!(last_bind < deny_mask, "{:?}", args.args);
    };

    // The link is spelled deeper than its target, which holds a deeper
    // writable root.
    let temp_dir = TempDir::new().expect("temp dir");
    let target = temp_dir.path().join("x");
    let nested = target.join("sub");
    let link = temp_dir.path().join("a").join("b").join("c").join("l");
    std::fs::create_dir_all(&nested).expect("create nested root");
    std::fs::write(nested.join("secret"), "secret").expect("create secret");
    std::fs::create_dir_all(link.parent().expect("link parent")).expect("link parent");
    std::os::unix::fs::symlink(&target, &link).expect("create link");
    let policy = FileSystemSandboxPolicy::restricted(vec![
        entry(&link, FileSystemAccessMode::Write),
        entry(&nested, FileSystemAccessMode::Write),
        entry(&link.join("sub").join("secret"), FileSystemAccessMode::Deny),
    ]);
    let args = create_filesystem_args(&policy, temp_dir.path(), BwrapOptions::default())
        .expect("filesystem args");
    masked_after_last_bind(&args, &nested, &nested.join("secret"));

    // The link and a writable root bind the same directory.
    let temp_dir = TempDir::new().expect("temp dir");
    let target = temp_dir.path().join("x");
    let link = temp_dir.path().join("a").join("b").join("l");
    std::fs::create_dir_all(&target).expect("create target");
    std::fs::write(target.join("secret"), "secret").expect("create secret");
    std::fs::create_dir_all(link.parent().expect("link parent")).expect("link parent");
    std::os::unix::fs::symlink(&target, &link).expect("create link");
    let policy = FileSystemSandboxPolicy::restricted(vec![
        entry(&link, FileSystemAccessMode::Write),
        entry(&target, FileSystemAccessMode::Write),
        entry(&link.join("secret"), FileSystemAccessMode::Deny),
    ]);
    let args = create_filesystem_args(&policy, temp_dir.path(), BwrapOptions::default())
        .expect("filesystem args");
    masked_after_last_bind(&args, &target, &target.join("secret"));
}

/// #503: with `:root = write`, an extra writable link to `/` binds `/`
/// again; read-only paths and denials outside every workspace are applied
/// after the last `/` bind.
#[cfg(unix)]
#[test]
fn full_write_root_and_link_to_root_keep_protections_outside_workspaces() {
    let temp_dir = TempDir::new().expect("temp dir");
    let read_only = temp_dir.path().join("ro");
    let denied = temp_dir.path().join("secret");
    let link = temp_dir.path().join("root-link");
    std::fs::create_dir(&read_only).expect("create read-only dir");
    std::fs::write(&denied, "secret").expect("create denied file");
    std::os::unix::fs::symlink("/", &link).expect("create symlink to /");
    let entry = |path: &Path, access| FileSystemSandboxEntry {
        path: FileSystemPath::from(AbsolutePathBuf::try_from(path).expect("absolute path")),
        access,
        missing_path_behavior: None,
    };
    let policy = FileSystemSandboxPolicy::restricted(vec![
        FileSystemSandboxEntry {
            path: FileSystemPath::Special {
                value: FileSystemSpecialPath::Root,
            },
            access: FileSystemAccessMode::Write,
            missing_path_behavior: None,
        },
        entry(&link, FileSystemAccessMode::Write),
        entry(&read_only, FileSystemAccessMode::Read),
        entry(&denied, FileSystemAccessMode::Deny),
    ]);

    let args = create_filesystem_args(&policy, temp_dir.path(), BwrapOptions::default())
        .expect("filesystem args");
    let last_root_bind = args
        .args
        .windows(3)
        .rposition(|window| window == ["--bind", "/", "/"])
        .expect("`/` should be bound writable");
    let read_only = path_to_string(&read_only);
    let read_only_bind = args
        .args
        .windows(3)
        .rposition(|window| window == ["--ro-bind", read_only.as_str(), read_only.as_str()])
        .expect("read-only path should be bound read-only");
    assert!(last_root_bind < read_only_bind, "{:?}", args.args);
    let denied = path_to_string(&denied);
    let deny_mask = args
        .args
        .iter()
        .rposition(|arg| arg == &denied)
        .expect("denied file should be masked");
    assert!(last_root_bind < deny_mask, "{:?}", args.args);
    // Neither pass asks for mount points at the filesystem root.
    assert!(
        !args
            .args
            .iter()
            .any(|arg| arg == "/.git" || arg == "/.codex"),
        "{:?}",
        args.args
    );
}

#[test]
fn missing_project_root_metadata_carveouts_use_metadata_path_masks() {
    let temp_dir = TempDir::new().expect("temp dir");
    let policy = FileSystemSandboxPolicy::restricted(vec![
        FileSystemSandboxEntry {
            path: FileSystemPath::Special {
                value: FileSystemSpecialPath::Root,
            },
            access: FileSystemAccessMode::Read,
            missing_path_behavior: None,
        },
        FileSystemSandboxEntry {
            path: FileSystemPath::Special {
                value: FileSystemSpecialPath::project_roots(/*subpath*/ None),
            },
            access: FileSystemAccessMode::Write,
            missing_path_behavior: None,
        },
        FileSystemSandboxEntry {
            path: FileSystemPath::Special {
                value: FileSystemSpecialPath::project_roots(Some(".git".into())),
            },
            access: FileSystemAccessMode::Read,
            missing_path_behavior: None,
        },
        FileSystemSandboxEntry {
            path: FileSystemPath::Special {
                value: FileSystemSpecialPath::project_roots(Some(".agents".into())),
            },
            access: FileSystemAccessMode::Read,
            missing_path_behavior: None,
        },
        FileSystemSandboxEntry {
            path: FileSystemPath::Special {
                value: FileSystemSpecialPath::project_roots(Some(".codex".into())),
            },
            access: FileSystemAccessMode::Read,
            missing_path_behavior: None,
        },
    ]);

    let args = create_filesystem_args(&policy, temp_dir.path(), BwrapOptions::default())
        .expect("filesystem args");
    let dot_git = path_to_string(&temp_dir.path().join(".git"));
    let dot_agents = path_to_string(&temp_dir.path().join(".agents"));
    let dot_codex = path_to_string(&temp_dir.path().join(".codex"));

    assert_empty_directory_mounted_read_only(&args.args, Path::new(&dot_git));
    assert_empty_directory_mounted_read_only(&args.args, Path::new(&dot_agents));
    assert_empty_directory_mounted_read_only(&args.args, Path::new(&dot_codex));
    assert!(args.preserved_files.is_empty());
    let synthetic_targets = synthetic_mount_target_paths(&args);
    assert!(synthetic_targets.contains(&PathBuf::from(&dot_git)));
    assert!(synthetic_targets.contains(&PathBuf::from(&dot_agents)));
    assert!(synthetic_targets.contains(&PathBuf::from(&dot_codex)));
}

#[test]
fn missing_user_project_root_subpath_rules_are_still_enforced() {
    let temp_dir = TempDir::new().expect("temp dir");
    let policy = FileSystemSandboxPolicy::restricted(vec![
        FileSystemSandboxEntry {
            path: FileSystemPath::Special {
                value: FileSystemSpecialPath::Root,
            },
            access: FileSystemAccessMode::Read,
            missing_path_behavior: None,
        },
        FileSystemSandboxEntry {
            path: FileSystemPath::Special {
                value: FileSystemSpecialPath::project_roots(/*subpath*/ None),
            },
            access: FileSystemAccessMode::Write,
            missing_path_behavior: None,
        },
        FileSystemSandboxEntry {
            path: FileSystemPath::Special {
                value: FileSystemSpecialPath::project_roots(Some(".vscode".into())),
            },
            access: FileSystemAccessMode::Read,
            missing_path_behavior: None,
        },
        FileSystemSandboxEntry {
            path: FileSystemPath::Special {
                value: FileSystemSpecialPath::project_roots(Some(".secrets".into())),
            },
            access: FileSystemAccessMode::Deny,
            missing_path_behavior: None,
        },
    ]);

    let args = create_filesystem_args(&policy, temp_dir.path(), BwrapOptions::default())
        .expect("filesystem args");
    let dot_vscode = path_to_string(&temp_dir.path().join(".vscode"));
    let dot_secrets = path_to_string(&temp_dir.path().join(".secrets"));

    assert_empty_file_bound_without_perms(&args.args, Path::new(&dot_vscode));
    assert_empty_file_bound_without_perms(&args.args, Path::new(&dot_secrets));
}

/// PF-23: full access with denials (`:root = write`) masks no repository
/// metadata at `/`, which an unprivileged bwrap could not create, also
/// when the command runs in `/`.
#[test]
fn full_write_root_with_denials_creates_nothing_at_the_filesystem_root() {
    let policy = FileSystemSandboxPolicy::restricted(vec![
        FileSystemSandboxEntry {
            path: FileSystemPath::Special {
                value: FileSystemSpecialPath::Root,
            },
            access: FileSystemAccessMode::Write,
            missing_path_behavior: None,
        },
        FileSystemSandboxEntry {
            path: FileSystemPath::from(
                AbsolutePathBuf::try_from(Path::new("/tmp/pf23-nonexistent-secret"))
                    .expect("absolute"),
            ),
            access: FileSystemAccessMode::Deny,
            missing_path_behavior: None,
        },
    ]);
    for cwd in ["/", "/tmp"] {
        let args = create_filesystem_args(&policy, Path::new(cwd), BwrapOptions::default())
            .expect("bwrap fs args");
        let at_root: Vec<PathBuf> = synthetic_mount_target_paths(&args)
            .into_iter()
            .filter(|path| {
                [".git", ".agents", ".codex"]
                    .iter()
                    .any(|name| path == &Path::new("/").join(name))
            })
            .collect();
        assert_eq!(at_root, Vec::<PathBuf>::new(), "cwd {cwd}");
        assert!(
            !args
                .args
                .iter()
                .any(|arg| arg == "/.git" || arg == "/.codex"),
            "cwd {cwd}: {:?}",
            args.args
        );
    }
}

#[test]
fn mounts_dev_before_writable_dev_binds() {
    let sandbox_policy = FileSystemSandboxPolicy::workspace_write(
        &[AbsolutePathBuf::try_from(Path::new("/dev")).expect("/dev path")],
        /*exclude_tmpdir_env_var*/ true,
        /*exclude_slash_tmp*/ true,
    );

    let args = create_filesystem_args(&sandbox_policy, Path::new("/"), BwrapOptions::default())
        .expect("bwrap fs args");
    assert!(args.preserved_files.is_empty());
    assert_eq!(
        synthetic_mount_target_paths(&args),
        vec![
            PathBuf::from("/.aws"),
            PathBuf::from("/.git"),
            PathBuf::from("/.agents"),
            PathBuf::from("/.codex"),
            PathBuf::from("/dev/.git"),
            PathBuf::from("/dev/.agents"),
            PathBuf::from("/dev/.codex"),
            PathBuf::from("/dev/.aws"),
        ]
    );
    let dev_mounts = args
        .args
        .windows(2)
        .enumerate()
        .filter_map(|(index, args)| (args == ["--dev", "/dev"]).then_some(index))
        .collect::<Vec<_>>();
    let [initial_dev_mount, restored_dev_mount] = dev_mounts.as_slice() else {
        panic!("expected exactly two /dev mounts, got {dev_mounts:?}");
    };
    let root_bind = args
        .args
        .windows(3)
        .position(|args| args == ["--bind", "/", "/"])
        .expect("root bind");
    let dev_bind = args
        .args
        .windows(3)
        .position(|args| args == ["--bind", "/dev", "/dev"])
        .expect("/dev bind");
    assert!(*initial_dev_mount < root_bind && root_bind < *restored_dev_mount);
    assert!(*restored_dev_mount < dev_bind);
}

#[test]
fn non_utf8_daemon_socket_mask_path_is_rejected() {
    let daemon_directory = PathBuf::from(OsString::from_vec(b"/tmp/codex-daemon-\xff".to_vec()));
    assert!(
        append_daemon_socket_masks(
            &mut Vec::new(),
            Path::new("/"),
            &BTreeSet::from([daemon_directory]),
        )
        .is_err()
    );
}

#[test]
fn read_only_tmp_carveout_keeps_daemon_sockets_masked() {
    let policy = FileSystemSandboxPolicy::restricted(
        [
            ("/", FileSystemAccessMode::Write),
            ("/tmp", FileSystemAccessMode::Read),
        ]
        .into_iter()
        .map(|(path, access)| {
            FileSystemSandboxEntry::new(
                AbsolutePathBuf::from_absolute_path(path).unwrap().into(),
                access,
            )
        })
        .collect(),
    );
    let args = create_filesystem_args(&policy, Path::new("/"), BwrapOptions::default())
        .expect("filesystem args");
    let tmp_bind = args
        .args
        .windows(3)
        .position(|window| window == ["--ro-bind", "/tmp", "/tmp"])
        .expect("read-only tmp bind");
    let daemon_directory = path_to_string(&codex_uds::shared_daemon_socket_directory().unwrap());
    assert_eq!(
        &args.args[tmp_bind + 3..tmp_bind + 9],
        [
            "--perms",
            "000",
            "--tmpfs",
            &daemon_directory,
            "--remount-ro",
            &daemon_directory
        ],
    );
}

/// S3a review L1: a missing denied path below a read-only `/tmp` makes
/// `/tmp` a #473 snapshot, whose entries are bound back from the host; the
/// app-server socket folder among them is masked again afterwards.
#[test]
fn read_only_tmp_snapshot_masks_daemon_sockets_again() {
    let missing = std::env::temp_dir().join(format!("s3a-l1-missing-{}", std::process::id()));
    let policy = FileSystemSandboxPolicy::restricted(
        [
            (Path::new("/"), FileSystemAccessMode::Write),
            (Path::new("/tmp"), FileSystemAccessMode::Read),
            (missing.as_path(), FileSystemAccessMode::Deny),
        ]
        .into_iter()
        .map(|(path, access)| {
            FileSystemSandboxEntry::new(
                AbsolutePathBuf::from_absolute_path(path).unwrap().into(),
                access,
            )
        })
        .collect(),
    );
    let args = create_filesystem_args(&policy, Path::new("/"), BwrapOptions::default())
        .expect("filesystem args")
        .args;
    let daemon_directory = path_to_string(&codex_uds::shared_daemon_socket_directory().unwrap());
    let last_rebind = args
        .windows(2)
        .rposition(|window| window[0] == "--ro-bind-try" && window[1] == daemon_directory)
        .expect("the /tmp snapshot binds the socket folder back");
    let last_mask = args
        .windows(2)
        .rposition(|window| window == ["--tmpfs", daemon_directory.as_str()])
        .expect("daemon socket mask");
    assert!(last_rebind < last_mask, "{args:?}");
}

#[test]
fn restricted_read_only_uses_scoped_read_roots_instead_of_erroring() {
    let temp_dir = TempDir::new().expect("temp dir");
    let readable_root = temp_dir.path().join("readable");
    std::fs::create_dir(&readable_root).expect("create readable root");

    let policy = FileSystemSandboxPolicy::restricted(vec![FileSystemSandboxEntry {
        path: AbsolutePathBuf::try_from(readable_root.as_path())
            .expect("absolute readable root")
            .into(),
        access: FileSystemAccessMode::Read,
        missing_path_behavior: None,
    }]);

    let args = create_filesystem_args(&policy, temp_dir.path(), BwrapOptions::default())
        .expect("filesystem args");

    assert_eq!(args.args[0..4], ["--tmpfs", "/", "--dev", "/dev"]);

    let readable_root_str = path_to_string(&readable_root);
    assert!(args.args.windows(3).any(|window| {
        window
            == [
                "--ro-bind",
                readable_root_str.as_str(),
                readable_root_str.as_str(),
            ]
    }));
}

#[test]
fn restricted_read_only_with_platform_defaults_includes_usr_when_present() {
    let temp_dir = TempDir::new().expect("temp dir");
    let policy = FileSystemSandboxPolicy::restricted(vec![FileSystemSandboxEntry {
        path: FileSystemPath::Special {
            value: FileSystemSpecialPath::Minimal,
        },
        access: FileSystemAccessMode::Read,
        missing_path_behavior: None,
    }]);

    let args = create_filesystem_args(&policy, temp_dir.path(), BwrapOptions::default())
        .expect("filesystem args");

    assert!(
        args.args
            .starts_with(&["--tmpfs".to_string(), "/".to_string()])
    );

    if Path::new("/usr").exists() {
        assert!(
            args.args
                .windows(3)
                .any(|window| window == ["--ro-bind", "/usr", "/usr"])
        );
    }
}

#[test]
fn split_policy_reapplies_unreadable_carveouts_after_writable_binds() {
    let temp_dir = TempDir::new().expect("temp dir");
    let writable_root = temp_dir.path().join("workspace");
    let blocked = writable_root.join("blocked");
    std::fs::create_dir_all(&blocked).expect("create blocked dir");
    let writable_root =
        AbsolutePathBuf::from_absolute_path(&writable_root).expect("absolute writable root");
    let blocked = AbsolutePathBuf::from_absolute_path(&blocked).expect("absolute blocked dir");
    let writable_root_str = path_to_string(writable_root.as_path());
    let blocked_str = path_to_string(blocked.as_path());
    let policy = FileSystemSandboxPolicy::restricted(vec![
        FileSystemSandboxEntry {
            path: writable_root.into(),
            access: FileSystemAccessMode::Write,
            missing_path_behavior: None,
        },
        FileSystemSandboxEntry {
            path: blocked.into(),
            access: FileSystemAccessMode::Deny,
            missing_path_behavior: None,
        },
    ]);

    let args = create_filesystem_args(&policy, temp_dir.path(), BwrapOptions::default())
        .expect("filesystem args");

    assert!(args.args.windows(3).any(|window| {
        window
            == [
                "--bind",
                writable_root_str.as_str(),
                writable_root_str.as_str(),
            ]
    }));
    let blocked_mask_index = args
        .args
        .windows(6)
        .position(|window| {
            window
                == [
                    "--perms",
                    "000",
                    "--tmpfs",
                    blocked_str.as_str(),
                    "--remount-ro",
                    blocked_str.as_str(),
                ]
        })
        .expect("blocked directory should be remounted unreadable");

    let writable_root_bind_index = args
        .args
        .windows(3)
        .position(|window| {
            window
                == [
                    "--bind",
                    writable_root_str.as_str(),
                    writable_root_str.as_str(),
                ]
        })
        .expect("writable root should be rebound writable");

    assert!(
        writable_root_bind_index < blocked_mask_index,
        "expected unreadable carveout to be re-applied after writable bind: {:#?}",
        args.args
    );
}

#[test]
fn split_policy_reenables_nested_writable_subpaths_after_read_only_parent() {
    let temp_dir = TempDir::new().expect("temp dir");
    let writable_root = temp_dir.path().join("workspace");
    let docs = writable_root.join("docs");
    let docs_public = docs.join("public");
    std::fs::create_dir_all(&docs_public).expect("create docs/public");
    let writable_root =
        AbsolutePathBuf::from_absolute_path(&writable_root).expect("absolute writable root");
    let docs = AbsolutePathBuf::from_absolute_path(&docs).expect("absolute docs");
    let docs_public =
        AbsolutePathBuf::from_absolute_path(&docs_public).expect("absolute docs/public");
    let mut entries = vec![
        FileSystemSandboxEntry::new(writable_root.into(), FileSystemAccessMode::Write),
        FileSystemSandboxEntry::new(docs.clone().into(), FileSystemAccessMode::Read),
        FileSystemSandboxEntry::new(docs_public.clone().into(), FileSystemAccessMode::Write),
    ];
    for name in [".git", ".agents", ".codex", ".aws"] {
        entries.push(FileSystemSandboxEntry::skip_missing_path(
            docs_public.join(name).into(),
            FileSystemAccessMode::Read,
        ));
    }
    let policy = FileSystemSandboxPolicy::restricted(entries);

    let args = create_filesystem_args(&policy, temp_dir.path(), BwrapOptions::default())
        .expect("filesystem args");
    let docs_str = path_to_string(docs.as_path());
    let docs_public_str = path_to_string(docs_public.as_path());
    let docs_ro_index = args
        .args
        .windows(3)
        .position(|window| window == ["--ro-bind", docs_str.as_str(), docs_str.as_str()])
        .expect("docs should be remounted read-only");
    let docs_public_rw_index = args
        .args
        .windows(3)
        .position(|window| window == ["--bind", docs_public_str.as_str(), docs_public_str.as_str()])
        .expect("docs/public should be rebound writable");

    assert!(
        docs_ro_index < docs_public_rw_index,
        "expected read-only parent remount before nested writable bind: {:#?}",
        args.args
    );
    for name in [".git", ".agents", ".codex", ".aws"] {
        let metadata_path = path_to_string(docs_public.join(name).as_path());
        let mount_indices = args
            .args
            .windows(2)
            .enumerate()
            .filter_map(|(index, window)| {
                (window == ["--tmpfs", metadata_path.as_str()]).then_some(index)
            })
            .collect::<Vec<_>>();
        assert_eq!(mount_indices.len(), 1, "one mount for {metadata_path}");
        assert!(
            docs_public_rw_index < mount_indices[0],
            "metadata must be mounted after its writable root: {:#?}",
            args.args
        );
    }
}

#[test]
fn split_policy_reenables_writable_subpaths_after_unreadable_parent() {
    let temp_dir = TempDir::new().expect("temp dir");
    let blocked = temp_dir.path().join("blocked");
    let allowed = blocked.join("allowed");
    std::fs::create_dir_all(&allowed).expect("create blocked/allowed");
    let blocked = AbsolutePathBuf::from_absolute_path(&blocked).expect("absolute blocked");
    let allowed = AbsolutePathBuf::from_absolute_path(&allowed).expect("absolute allowed");
    let policy = FileSystemSandboxPolicy::restricted(vec![
        FileSystemSandboxEntry {
            path: FileSystemPath::Special {
                value: FileSystemSpecialPath::Root,
            },
            access: FileSystemAccessMode::Read,
            missing_path_behavior: None,
        },
        FileSystemSandboxEntry {
            path: blocked.clone().into(),
            access: FileSystemAccessMode::Deny,
            missing_path_behavior: None,
        },
        FileSystemSandboxEntry {
            path: allowed.clone().into(),
            access: FileSystemAccessMode::Write,
            missing_path_behavior: None,
        },
    ]);

    let args = create_filesystem_args(&policy, temp_dir.path(), BwrapOptions::default())
        .expect("filesystem args");
    let blocked_str = path_to_string(blocked.as_path());
    let allowed_str = path_to_string(allowed.as_path());
    let blocked_none_index = args
        .args
        .windows(4)
        .position(|window| window == ["--perms", "111", "--tmpfs", blocked_str.as_str()])
        .expect("blocked should be masked first");
    let allowed_dir_index = args
        .args
        .windows(2)
        .position(|window| window == ["--dir", allowed_str.as_str()])
        .expect("allowed mount target should be recreated");
    let blocked_remount_ro_index = args
        .args
        .windows(2)
        .position(|window| window == ["--remount-ro", blocked_str.as_str()])
        .expect("blocked directory should be remounted read-only");
    let allowed_bind_index = args
        .args
        .windows(3)
        .position(|window| window == ["--bind", allowed_str.as_str(), allowed_str.as_str()])
        .expect("allowed path should be rebound writable");

    assert!(
        blocked_none_index < allowed_dir_index
            && allowed_dir_index < blocked_remount_ro_index
            && blocked_remount_ro_index < allowed_bind_index,
        "expected writable child target recreation before remounting and rebinding under unreadable parent: {:#?}",
        args.args
    );
}

#[test]
fn split_policy_reenables_writable_files_after_unreadable_parent() {
    let temp_dir = TempDir::new().expect("temp dir");
    let blocked = temp_dir.path().join("blocked");
    let allowed_dir = blocked.join("allowed");
    let allowed_file = allowed_dir.join("note.txt");
    std::fs::create_dir_all(&allowed_dir).expect("create blocked/allowed");
    std::fs::write(&allowed_file, "ok").expect("create note");
    let blocked = AbsolutePathBuf::from_absolute_path(&blocked).expect("absolute blocked");
    let allowed_dir =
        AbsolutePathBuf::from_absolute_path(&allowed_dir).expect("absolute allowed dir");
    let allowed_file =
        AbsolutePathBuf::from_absolute_path(&allowed_file).expect("absolute allowed file");
    let policy = FileSystemSandboxPolicy::restricted(vec![
        FileSystemSandboxEntry {
            path: FileSystemPath::Special {
                value: FileSystemSpecialPath::Root,
            },
            access: FileSystemAccessMode::Read,
            missing_path_behavior: None,
        },
        FileSystemSandboxEntry {
            path: blocked.clone().into(),
            access: FileSystemAccessMode::Deny,
            missing_path_behavior: None,
        },
        FileSystemSandboxEntry {
            path: allowed_file.clone().into(),
            access: FileSystemAccessMode::Write,
            missing_path_behavior: None,
        },
    ]);

    let args = create_filesystem_args(&policy, temp_dir.path(), BwrapOptions::default())
        .expect("filesystem args");
    let blocked_str = path_to_string(blocked.as_path());
    let allowed_dir_str = path_to_string(allowed_dir.as_path());
    let allowed_file_str = path_to_string(allowed_file.as_path());

    assert!(
        args.args
            .windows(2)
            .any(|window| window == ["--dir", allowed_dir_str.as_str()]),
        "expected ancestor directory to be recreated: {:#?}",
        args.args
    );
    assert!(
        !args
            .args
            .windows(2)
            .any(|window| window == ["--dir", allowed_file_str.as_str()]),
        "writable file target should not be converted into a directory: {:#?}",
        args.args
    );
    let blocked_none_index = args
        .args
        .windows(4)
        .position(|window| window == ["--perms", "111", "--tmpfs", blocked_str.as_str()])
        .expect("blocked should be masked first");
    let allowed_bind_index = args
        .args
        .windows(3)
        .position(|window| {
            window
                == [
                    "--bind",
                    allowed_file_str.as_str(),
                    allowed_file_str.as_str(),
                ]
        })
        .expect("allowed file should be rebound writable");

    assert!(
        blocked_none_index < allowed_bind_index,
        "expected unreadable parent mask before rebinding writable file child: {:#?}",
        args.args
    );
}

#[test]
fn split_policy_reenables_nested_writable_roots_after_unreadable_parent() {
    let temp_dir = TempDir::new().expect("temp dir");
    let writable_root = temp_dir.path().join("workspace");
    let blocked = writable_root.join("blocked");
    let allowed = blocked.join("allowed");
    std::fs::create_dir_all(&allowed).expect("create blocked/allowed dir");
    let writable_root =
        AbsolutePathBuf::from_absolute_path(&writable_root).expect("absolute writable root");
    let blocked = AbsolutePathBuf::from_absolute_path(&blocked).expect("absolute blocked dir");
    let allowed = AbsolutePathBuf::from_absolute_path(&allowed).expect("absolute allowed dir");
    let blocked_str = path_to_string(blocked.as_path());
    let allowed_str = path_to_string(allowed.as_path());
    let policy = FileSystemSandboxPolicy::restricted(vec![
        FileSystemSandboxEntry {
            path: writable_root.into(),
            access: FileSystemAccessMode::Write,
            missing_path_behavior: None,
        },
        FileSystemSandboxEntry {
            path: blocked.into(),
            access: FileSystemAccessMode::Deny,
            missing_path_behavior: None,
        },
        FileSystemSandboxEntry {
            path: allowed.into(),
            access: FileSystemAccessMode::Write,
            missing_path_behavior: None,
        },
    ]);

    let args = create_filesystem_args(&policy, temp_dir.path(), BwrapOptions::default())
        .expect("filesystem args");
    let blocked_none_index = args
        .args
        .windows(4)
        .position(|window| window == ["--perms", "111", "--tmpfs", blocked_str.as_str()])
        .expect("blocked should be masked first");
    let allowed_dir_index = args
        .args
        .windows(2)
        .position(|window| window == ["--dir", allowed_str.as_str()])
        .expect("allowed mount target should be recreated");
    let allowed_bind_index = args
        .args
        .windows(3)
        .position(|window| window == ["--bind", allowed_str.as_str(), allowed_str.as_str()])
        .expect("allowed path should be rebound writable");

    assert!(
        blocked_none_index < allowed_dir_index && allowed_dir_index < allowed_bind_index,
        "expected unreadable parent mask before recreating and rebinding writable child: {:#?}",
        args.args
    );
}

#[test]
fn split_policy_masks_root_read_directory_carveouts() {
    let temp_dir = TempDir::new().expect("temp dir");
    let blocked = temp_dir.path().join("blocked");
    std::fs::create_dir_all(&blocked).expect("create blocked dir");
    let blocked = AbsolutePathBuf::from_absolute_path(&blocked).expect("absolute blocked dir");
    let policy = FileSystemSandboxPolicy::restricted(vec![
        FileSystemSandboxEntry {
            path: FileSystemPath::Special {
                value: FileSystemSpecialPath::Root,
            },
            access: FileSystemAccessMode::Read,
            missing_path_behavior: None,
        },
        FileSystemSandboxEntry {
            path: blocked.clone().into(),
            access: FileSystemAccessMode::Deny,
            missing_path_behavior: None,
        },
    ]);

    let args = create_filesystem_args(&policy, temp_dir.path(), BwrapOptions::default())
        .expect("filesystem args");
    let blocked_str = path_to_string(blocked.as_path());

    assert!(
        args.args
            .windows(3)
            .any(|window| window == ["--ro-bind", "/", "/"])
    );
    assert!(
        args.args
            .windows(4)
            .any(|window| { window == ["--perms", "000", "--tmpfs", blocked_str.as_str()] })
    );
    assert!(
        args.args
            .windows(2)
            .any(|window| window == ["--remount-ro", blocked_str.as_str()])
    );
}

#[test]
fn split_policy_masks_root_read_file_carveouts() {
    let temp_dir = TempDir::new().expect("temp dir");
    let blocked_file = temp_dir.path().join("blocked.txt");
    let second_blocked_file = temp_dir.path().join("second-blocked.txt");
    std::fs::write(&blocked_file, "secret").expect("create blocked file");
    std::fs::write(&second_blocked_file, "dummy secret").expect("create second blocked file");
    let blocked_file =
        AbsolutePathBuf::from_absolute_path(&blocked_file).expect("absolute blocked file");
    let policy = FileSystemSandboxPolicy::restricted(vec![
        FileSystemSandboxEntry {
            path: FileSystemPath::Special {
                value: FileSystemSpecialPath::Root,
            },
            access: FileSystemAccessMode::Read,
            missing_path_behavior: None,
        },
        FileSystemSandboxEntry {
            path: blocked_file.clone().into(),
            access: FileSystemAccessMode::Deny,
            missing_path_behavior: None,
        },
        FileSystemSandboxEntry {
            path: AbsolutePathBuf::from_absolute_path(&second_blocked_file)
                .expect("absolute second blocked file")
                .into(),
            access: FileSystemAccessMode::Deny,
            missing_path_behavior: None,
        },
    ]);

    let args = create_filesystem_args(&policy, temp_dir.path(), BwrapOptions::default())
        .expect("filesystem args");
    assert_eq!(args.preserved_files.len(), 2);
    assert_bind_data_uses_distinct_preserved_fds(&args);
    assert!(args.synthetic_mount_targets.is_empty());
    for blocked_file in [blocked_file.as_path(), second_blocked_file.as_path()] {
        assert!(args.args.windows(5).any(|window| {
            window[0] == "--perms"
                && window[1] == "000"
                && window[2] == "--ro-bind-data"
                && window[4] == path_to_string(blocked_file)
        }));
    }
}

fn assert_bind_data_uses_distinct_preserved_fds(args: &BwrapArgs) {
    let mount_fds = args
        .args
        .windows(3)
        .filter(|window| window[0] == "--ro-bind-data")
        .map(|window| window[1].parse::<i32>().expect("bind-data fd"))
        .collect::<Vec<_>>();
    let distinct_fds = mount_fds.iter().copied().collect::<HashSet<_>>();
    assert_eq!(mount_fds.len(), distinct_fds.len());
    assert_eq!(
        distinct_fds,
        args.preserved_files
            .iter()
            .map(AsRawFd::as_raw_fd)
            .collect()
    );
}

/// #636: bubblewrap closes a `--ro-bind-data` descriptor once it has read
/// it, so two masked files must not share one.
#[test]
fn every_masked_file_gets_its_own_null_descriptor() {
    let temp_dir = TempDir::new().expect("temp dir");
    let mut entries = vec![FileSystemSandboxEntry {
        path: FileSystemPath::Special {
            value: FileSystemSpecialPath::Root,
        },
        access: FileSystemAccessMode::Read,
        missing_path_behavior: None,
    }];
    let mut blocked = Vec::new();
    for name in ["first.txt", "second.txt", "third.txt"] {
        let path = temp_dir.path().join(name);
        std::fs::write(&path, "secret").expect("create blocked file");
        entries.push(FileSystemSandboxEntry {
            path: FileSystemPath::from(
                AbsolutePathBuf::from_absolute_path(&path).expect("absolute"),
            ),
            access: FileSystemAccessMode::Deny,
            missing_path_behavior: None,
        });
        blocked.push(path_to_string(&path));
    }
    let policy = FileSystemSandboxPolicy::restricted(entries);

    let args = create_filesystem_args(&policy, temp_dir.path(), BwrapOptions::default())
        .expect("filesystem args");

    let masks = args
        .args
        .windows(3)
        .filter(|window| window[0] == "--ro-bind-data")
        .map(|window| (window[2].clone(), window[1].clone()))
        .collect::<Vec<_>>();
    let mut masked = masks
        .iter()
        .map(|(path, _)| path.clone())
        .collect::<Vec<_>>();
    masked.sort();
    assert_eq!(masked, blocked);
    let fds = masks
        .iter()
        .map(|(_, fd)| fd.clone())
        .collect::<std::collections::HashSet<_>>();
    assert_eq!(fds.len(), blocked.len(), "{masks:?}");
    for fd in fds {
        assert_eq!(
            std::fs::read_link(format!("/proc/self/fd/{fd}")).expect("preserved fd"),
            PathBuf::from("/dev/null")
        );
    }
    assert_eq!(args.preserved_files.len(), blocked.len());
}

#[test]
fn unreadable_globs_expand_existing_matches_with_configured_depth() {
    if !ripgrep_available() {
        return;
    }

    let temp_dir = TempDir::new().expect("temp dir");
    let root_env = temp_dir.path().join(".env");
    let nested_env = temp_dir.path().join("app").join(".env");
    let too_deep_env = temp_dir.path().join("app").join("deep").join(".env");
    std::fs::create_dir_all(too_deep_env.parent().expect("parent")).expect("create parent");
    std::fs::write(temp_dir.path().join(".gitignore"), ".env\n").expect("write gitignore");
    std::fs::write(&root_env, "secret").expect("write root env");
    std::fs::write(&nested_env, "secret").expect("write nested env");
    std::fs::write(&too_deep_env, "secret").expect("write deep env");
    let policy =
        default_policy_with_unreadable_glob(format!("{}/**/*.env", temp_dir.path().display()));

    let args = create_filesystem_args(
        &policy,
        temp_dir.path(),
        BwrapOptions {
            glob_scan_max_depth: Some(2),
            ..Default::default()
        },
    )
    .expect("filesystem args");

    assert_file_masked(&args.args, &root_env);
    assert_file_masked(&args.args, &nested_env);
    assert!(
        !args
            .args
            .iter()
            .any(|arg| arg == &path_to_string(&too_deep_env)),
        "max depth should keep deeper matches out of bwrap args: {:#?}",
        args.args
    );
}

/// #442 (Travis, 2026-10-12): the credential globs hide `.env` and
/// `.env.local` but not the templates `.env.example`, `.env.sample` and
/// `.env.template`; a person's own glob that covers a template still
/// hides it.
#[test]
fn credential_globs_leave_templates_readable() {
    let temp_dir = TempDir::new().expect("temp dir");
    let root = temp_dir.path();
    for name in [
        ".env",
        ".env.local",
        ".env.example",
        ".env.sample",
        ".env.template",
    ] {
        std::fs::write(root.join(name), "x").expect("write");
    }
    let expand = |globs: Vec<String>| {
        let globs: Vec<FileSystemDenyGlob> = globs
            .into_iter()
            .map(|pattern| FileSystemDenyGlob {
                pattern,
                scan: None,
            })
            .collect();
        expand_unreadable_globs(&globs, root, /*max_depth*/ None, &path_ripgrep_files)
            .expect("expand")
            .into_iter()
            .map(|path| path.as_path().file_name().expect("name").to_os_string())
            .collect::<Vec<_>>()
    };
    let credential = codex_protocol::permissions::credential_file_globs(&root.to_string_lossy());
    assert_eq!(expand(credential.clone()), vec![".env", ".env.local"]);
    let mut own = credential;
    own.push(format!("{}/**/*.sample", root.display()));
    assert_eq!(expand(own), vec![".env", ".env.local", ".env.sample"]);
    // A person's own `**/.env.*` alone is not the credential rule.
    assert_eq!(
        expand(vec![format!("{}/**/.env.*", root.display())]),
        vec![".env.example", ".env.local", ".env.sample", ".env.template"]
    );
}

#[cfg(unix)]
#[test]
fn unreadable_globs_add_canonical_targets_for_symlink_matches() {
    if !ripgrep_available() {
        return;
    }

    let temp_dir = TempDir::new().expect("temp dir");
    let real_root = temp_dir.path().join("real");
    let link_root = temp_dir.path().join("link");
    let real_secret = real_root.join("secret.env");
    std::fs::create_dir_all(&real_root).expect("create real root");
    std::fs::write(&real_secret, "secret").expect("write real secret");
    std::os::unix::fs::symlink(&real_root, &link_root).expect("create symlink");
    let policy = default_policy_with_unreadable_glob(format!("{}/**/*.env", link_root.display()));

    let args = create_filesystem_args(
        &policy,
        temp_dir.path(),
        BwrapOptions {
            glob_scan_max_depth: Some(2),
            ..Default::default()
        },
    )
    .expect("filesystem args");

    assert_file_masked(&args.args, &real_secret);
}

#[test]
fn unreadable_glob_root_metadata_errors_fail_before_scanning() {
    let temp_dir = TempDir::new().expect("temp dir");
    let root = temp_dir.path().join("loop");
    std::os::unix::fs::symlink(&root, &root).expect("create symlink loop");
    let error = expand_unreadable_globs_with_ripgrep(
        &[format!("{}/**/.env", root.display())],
        temp_dir.path(),
        || Some(temp_dir.path().join("scanner-must-not-run")),
        /*max_depth*/ None,
        |_| panic!("scanner must not run after a root metadata error"),
    )
    .expect_err("metadata errors must reject deny-glob expansion")
    .to_string();

    assert!(error.contains(&root.display().to_string()));
    assert!(error.contains(&io::Error::from_raw_os_error(libc::ELOOP).to_string()));
}

#[test]
fn unreadable_globs_skip_absent_and_non_directory_roots() {
    let temp_dir = TempDir::new().expect("temp dir");
    let missing = temp_dir.path().join("missing");
    let file = temp_dir.path().join("file");
    fs::write(&file, "ordinary file").expect("write file");

    for root in [missing, file.join("child"), file] {
        let paths = expand_unreadable_globs_with_ripgrep(
            &[format!("{}/**/.env", root.display())],
            temp_dir.path(),
            || None,
            /*max_depth*/ None,
            |_| {},
        )
        .expect("roots without possible matches should be skipped");

        assert_eq!(paths, Vec::<AbsolutePathBuf>::new());
    }
}

#[test]
fn unreadable_globs_follow_symlink_roots_without_ripgrep() {
    let temp_dir = TempDir::new().expect("temp dir");
    let root = temp_dir.path().join("data");
    fs::create_dir(&root).expect("create directory");
    let secret = root.join(".env");
    fs::write(&secret, "secret").expect("write secret");
    let link = temp_dir.path().join("link");
    std::os::unix::fs::symlink(&root, &link).expect("create directory symlink");

    let paths = expand_unreadable_globs_with_ripgrep(
        &[format!("{}/**/.env", link.display())],
        temp_dir.path(),
        || None,
        /*max_depth*/ None,
        |_| panic!("fallback scanner must not launch ripgrep"),
    )
    .expect("expand deny glob");
    let expected = [secret, link.join(".env")]
        .into_iter()
        .map(|path| AbsolutePathBuf::from_absolute_path_checked(path).expect("absolute path"))
        .collect::<Vec<_>>();

    assert_eq!(paths, expected);
}

#[test]
fn root_prefix_unreadable_globs_fail_closed_on_linux() {
    let policy = default_policy_with_unreadable_glob("/**/*.env".to_string());
    let error = create_bwrap_command_args(
        vec!["/bin/true".to_string()],
        &policy,
        Path::new("/tmp"),
        Path::new("/tmp"),
        BwrapOptions::default(),
    )
    .expect_err("root-prefix deny-read glob must reject sandbox construction");

    assert_eq!(
        error.to_string(),
        "Fatal error: unreadable glob `/**/*.env` cannot be safely expanded; use a pattern with a non-root directory prefix"
    );
}

#[test]
fn unclosed_character_classes_are_escaped_for_ripgrep() {
    let (search_root, glob) =
        split_pattern_for_ripgrep("/tmp/[*.env", Path::new("/")).expect("split pattern");

    assert_eq!(search_root.as_path(), Path::new("/tmp"));
    assert_eq!(glob, r"\[*.env");
}

fn ripgrep_available() -> bool {
    Command::new(GLOB_SCAN_PROGRAM)
        .arg("--version")
        .output()
        .is_ok_and(|output| output.status.success())
}

fn path_entry(
    path: &Path,
    access: FileSystemAccessMode,
    skip_missing: bool,
) -> FileSystemSandboxEntry {
    let path =
        FileSystemPath::from(AbsolutePathBuf::from_absolute_path(path).expect("absolute path"));
    if skip_missing {
        FileSystemSandboxEntry::skip_missing_path(path, access)
    } else {
        FileSystemSandboxEntry::new(path, access)
    }
}

/// #587: with denials, each folder between the writable root and a
/// protected path is bound onto itself before the pass's masks (a mount
/// point cannot be renamed); the protected folder itself and folders
/// below it are not. Without denials (Permissive) nothing is pinned.
#[test]
fn folders_above_protected_paths_are_pinned_before_their_masks() {
    let temp_dir = TempDir::new().expect("temp dir");
    let base = temp_dir.path().canonicalize().expect("canonical");
    let home = base.join("outer/home");
    std::fs::create_dir_all(home.join("skills/inner")).expect("home");
    std::fs::write(home.join("auth.json"), "x").expect("store");
    let writable = FileSystemSandboxEntry {
        path: FileSystemPath::Special {
            value: FileSystemSpecialPath::Root,
        },
        access: FileSystemAccessMode::Write,
        missing_path_behavior: None,
    };
    let read_only = path_entry(
        &home.join("skills"),
        FileSystemAccessMode::Read,
        /*skip_missing*/ true,
    );
    let policy = FileSystemSandboxPolicy::restricted(vec![
        writable.clone(),
        path_entry(
            &home.join("auth.json"),
            FileSystemAccessMode::Deny,
            /*skip_missing*/ true,
        ),
        read_only.clone(),
    ]);
    // A folder the user owns but cannot write right now could be made
    // writable first: its entries are pinned too.
    let outer = base.join("outer");
    let args = {
        let _read_only = ModeGuard::set(&outer, /*mode*/ 0o555);
        create_filesystem_args(&policy, &base, BwrapOptions::default())
            .expect("bwrap fs args")
            .args
    };
    let binds = pinned_folders(&args);
    let path = |path: &Path| path_to_string(path);
    for folder in [&base, &base.join("outer"), &home] {
        assert!(
            binds.contains(&path(folder).as_str()),
            "{folder:?}: {args:?}"
        );
    }
    // A folder whose parent the user neither owns nor can write (the
    // temporary folder's top-level folder, in `/`) cannot be renamed.
    let top = base
        .ancestors()
        .find(|folder| folder.parent() == Some(Path::new("/")))
        .expect("top-level folder");
    // SAFETY: `geteuid` has no preconditions.
    if unsafe { libc::geteuid() } != 0 {
        assert!(!binds.contains(&path(top).as_str()), "{top:?}: {args:?}");
    }
    for folder in [home.join("skills"), home.join("skills/inner")] {
        assert!(
            !binds.contains(&path(&folder).as_str()),
            "{folder:?}: {args:?}"
        );
    }
    let position = |flag: &str, target: &Path| {
        args.windows(2)
            .position(|window| window[0] == flag && window[1] == path(target))
            .expect("mount")
    };
    assert!(position("--bind-try", &home) < position("--ro-bind", &home.join("skills")));

    let workspace = FileSystemSandboxPolicy::restricted(vec![writable, read_only]);
    let args = create_filesystem_args(&workspace, &base, BwrapOptions::default())
        .expect("bwrap fs args")
        .args;
    // (The `/` bind's shared memory is the only `--bind-try` left.)
    assert!(
        !args
            .windows(2)
            .any(|window| window[0] == "--bind-try" && window[1] != "/dev/shm"),
        "{args:?}"
    );
}

/// Restores a folder's mode when dropped, also when the test fails.
struct ModeGuard<'a>(&'a Path);

impl<'a> ModeGuard<'a> {
    fn set(path: &'a Path, mode: u32) -> Self {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(path, std::fs::Permissions::from_mode(mode)).expect("chmod");
        Self(path)
    }
}

impl Drop for ModeGuard<'_> {
    fn drop(&mut self) {
        use std::os::unix::fs::PermissionsExt;
        let _ = std::fs::set_permissions(self.0, std::fs::Permissions::from_mode(0o755));
    }
}

/// The folders a run binds onto themselves (`--bind-try d d`).
fn pinned_folders(args: &[String]) -> Vec<&str> {
    args.windows(3)
        .filter(|window| window[0] == "--bind-try" && window[1] == window[2])
        .map(|window| window[1].as_str())
        .collect()
}

/// #594/#598: a path the policy says may not be moved (the Corbanu home
/// of a command a person approved: no denials, only its read-only
/// runtime entries) is pinned at its real location, before the read-only
/// mounts inside it, and so is every folder above it and above a link
/// on its way. The link itself cannot be pinned. Without the entry
/// (Permissive) nothing is pinned.
#[test]
fn unmovable_paths_pin_the_real_folder_and_the_folders_above_a_link() {
    let temp_dir = TempDir::new().expect("temp dir");
    let base = temp_dir.path().canonicalize().expect("canonical");
    let home = base.join("dot/home");
    std::fs::create_dir_all(home.join("skills")).expect("home");
    std::fs::create_dir_all(base.join("links")).expect("links");
    let link = base.join("links/home");
    std::os::unix::fs::symlink(&home, &link).expect("link");
    let writable = FileSystemSandboxEntry::new(
        FileSystemPath::Special {
            value: FileSystemSpecialPath::Root,
        },
        FileSystemAccessMode::Write,
    );
    let read_only = path_entry(
        &home.join("skills"),
        FileSystemAccessMode::Read,
        /*skip_missing*/ true,
    );
    let absolute = |path: &Path| AbsolutePathBuf::from_absolute_path(path).expect("absolute");
    let policy = FileSystemSandboxPolicy::restricted(vec![
        writable.clone(),
        read_only.clone(),
        FileSystemSandboxEntry::deny_moving(&absolute(&link)),
    ]);
    assert_eq!(
        unmovable_pin_targets(&policy),
        vec![
            UnmovableTarget {
                path: home.clone(),
                pin_itself: true,
            },
            UnmovableTarget {
                path: link.clone(),
                pin_itself: false,
            },
        ]
    );
    let args = create_filesystem_args(&policy, &base, BwrapOptions::default())
        .expect("bwrap fs args")
        .args;
    let binds = pinned_folders(&args);
    let path = |path: &Path| path_to_string(path);
    for folder in [&base, &base.join("dot"), &home, &base.join("links")] {
        assert!(
            binds.contains(&path(folder).as_str()),
            "{folder:?}: {args:?}"
        );
    }
    for folder in [&link, &home.join("skills")] {
        assert!(
            !binds.contains(&path(folder).as_str()),
            "{folder:?}: {args:?}"
        );
    }
    let position = |flag: &str, target: &Path| {
        args.windows(2)
            .position(|window| window[0] == flag && window[1] == path(target))
            .expect("mount")
    };
    assert!(position("--bind-try", &home) < position("--ro-bind", &home.join("skills")));

    let permissive = FileSystemSandboxPolicy::restricted(vec![writable, read_only]);
    let args = create_filesystem_args(&permissive, &base, BwrapOptions::default())
        .expect("bwrap fs args")
        .args;
    // (The `/` bind's shared memory is the only `--bind-try` left.)
    assert!(
        !args
            .windows(2)
            .any(|window| window[0] == "--bind-try" && window[1] != "/dev/shm"),
        "{args:?}"
    );
}

/// #473: `~/.claude` read-only (it runs hooks) with no
/// `.credentials.json` yet. A placeholder cannot be made in the
/// read-only folder, so the folder becomes a snapshot of what it holds.
#[test]
fn missing_denied_path_below_read_only_folder_gets_a_snapshot() {
    let temp_dir = TempDir::new().expect("temp dir");
    let home = temp_dir.path().join("home");
    let claude = home.join(".claude");
    std::fs::create_dir_all(claude.join("projects")).expect("create .claude");
    std::fs::write(claude.join("settings.json"), "{}").expect("settings");
    std::os::unix::fs::symlink("settings.json", claude.join("link")).expect("link");
    let credentials = claude.join(".credentials.json");
    let secret = claude.join("secret.json");
    std::fs::write(&secret, "secret").expect("secret");
    let policy = FileSystemSandboxPolicy::restricted(vec![
        path_entry(
            &home,
            FileSystemAccessMode::Write,
            /*skip_missing*/ false,
        ),
        path_entry(
            &claude,
            FileSystemAccessMode::Read,
            /*skip_missing*/ true,
        ),
        path_entry(
            &credentials,
            FileSystemAccessMode::Deny,
            /*skip_missing*/ true,
        ),
        path_entry(
            &secret,
            FileSystemAccessMode::Deny,
            /*skip_missing*/ true,
        ),
    ]);

    let args = create_filesystem_args(&policy, &home, BwrapOptions::default())
        .expect("filesystem args")
        .args;

    let claude_arg = path_to_string(&claude);
    let position = |window: &[&str]| {
        args.windows(window.len())
            .position(|candidate| candidate == window)
            .unwrap_or_else(|| panic!("expected {window:?} in {args:#?}"))
    };
    let read_only_bind = position(&["--ro-bind", &claude_arg, &claude_arg]);
    let snapshot = position(&["--perms", "555", "--tmpfs", &claude_arg]);
    let remount = position(&["--remount-ro", &claude_arg]);
    assert!(read_only_bind < snapshot && snapshot < remount, "{args:#?}");
    for child in ["projects", "secret.json", "settings.json"] {
        let child = path_to_string(&claude.join(child));
        let bind = position(&["--ro-bind-try", &child, &child]);
        assert!(snapshot < bind && bind < remount, "{args:#?}");
    }
    // An existing denied entry stays masked over its snapshot bind.
    let secret_arg = path_to_string(&secret);
    let secret_mask = args
        .windows(5)
        .position(|window| {
            window[0] == "--perms"
                && window[1] == "000"
                && window[2] == "--ro-bind-data"
                && window[4] == secret_arg
        })
        .unwrap_or_else(|| panic!("expected a mask for {secret_arg}: {args:#?}"));
    assert!(remount < secret_mask, "{args:#?}");
    position(&[
        "--symlink",
        "settings.json",
        &path_to_string(&claude.join("link")),
    ]);
    assert!(
        !args.contains(&path_to_string(&credentials)),
        "nothing is mounted at the missing file: {args:#?}"
    );
    assert!(!credentials.exists(), "no placeholder on the host");
}

/// #473: `~/.config/gh` and `~/.config/gcloud` with no `~/.config`
/// share one placeholder; bubblewrap cannot mount a second over it.
#[test]
fn missing_denied_paths_sharing_a_missing_folder_mask_it_once() {
    let temp_dir = TempDir::new().expect("temp dir");
    let home = temp_dir.path().join("home");
    std::fs::create_dir_all(&home).expect("create home");
    let config = home.join(".config");
    let policy = FileSystemSandboxPolicy::restricted(vec![
        path_entry(
            &home,
            FileSystemAccessMode::Write,
            /*skip_missing*/ false,
        ),
        path_entry(
            &config.join("gh"),
            FileSystemAccessMode::Deny,
            /*skip_missing*/ true,
        ),
        path_entry(
            &config.join("gcloud"),
            FileSystemAccessMode::Deny,
            /*skip_missing*/ true,
        ),
    ]);

    let args =
        create_filesystem_args(&policy, &home, BwrapOptions::default()).expect("filesystem args");

    let config_arg = path_to_string(&config);
    assert_eq!(
        args.args
            .windows(3)
            .filter(|window| window[0] == "--ro-bind-data" && window[2] == config_arg)
            .count(),
        1,
        "{:#?}",
        args.args
    );
    assert_eq!(
        synthetic_mount_target_paths(&args)
            .into_iter()
            .filter(|path| path == &config)
            .count(),
        1
    );
}

/// #473 review: a broader writable root's pass leaves an empty
/// placeholder on the host; the nested root's `--bind` shows that live
/// file again, so its own pass must mask the path again.
#[test]
fn missing_denied_path_is_masked_again_after_a_nested_writable_bind() {
    let temp_dir = TempDir::new().expect("temp dir");
    let home = temp_dir.path().join("home");
    let work = home.join("work");
    std::fs::create_dir_all(&work).expect("create work");
    let secret = work.join("secret");
    let policy = FileSystemSandboxPolicy::restricted(vec![
        path_entry(
            &home,
            FileSystemAccessMode::Write,
            /*skip_missing*/ false,
        ),
        path_entry(
            &work,
            FileSystemAccessMode::Write,
            /*skip_missing*/ false,
        ),
        path_entry(
            &secret,
            FileSystemAccessMode::Deny,
            /*skip_missing*/ true,
        ),
    ]);

    let args = create_filesystem_args(&policy, &home, BwrapOptions::default())
        .expect("filesystem args")
        .args;

    let work_arg = path_to_string(&work);
    let secret_arg = path_to_string(&secret);
    let inner_bind = args
        .windows(3)
        .position(|window| window == ["--bind", work_arg.as_str(), work_arg.as_str()])
        .unwrap_or_else(|| panic!("expected a bind of {work_arg}: {args:#?}"));
    let masks: Vec<usize> = args
        .windows(3)
        .enumerate()
        .filter(|(_, window)| window[0] == "--ro-bind-data" && window[2] == secret_arg)
        .map(|(index, _)| index)
        .collect();
    assert_eq!(masks.len(), 2, "{args:#?}");
    assert!(masks[0] < inner_bind && inner_bind < masks[1], "{args:#?}");
}

/// #473: a missing read-only path below a read-only folder needs no
/// placeholder; the folder already stops it being created.
#[test]
fn missing_read_only_path_below_read_only_folder_is_left_to_the_folder() {
    let temp_dir = TempDir::new().expect("temp dir");
    let home = temp_dir.path().join("home");
    let claude = home.join(".claude");
    std::fs::create_dir_all(&claude).expect("create .claude");
    let hooks = claude.join("missing/hooks");
    let policy = FileSystemSandboxPolicy::restricted(vec![
        path_entry(
            &home,
            FileSystemAccessMode::Write,
            /*skip_missing*/ false,
        ),
        path_entry(
            &claude,
            FileSystemAccessMode::Read,
            /*skip_missing*/ true,
        ),
        path_entry(
            &hooks,
            FileSystemAccessMode::Read,
            /*skip_missing*/ true,
        ),
    ]);

    let args =
        create_filesystem_args(&policy, &home, BwrapOptions::default()).expect("filesystem args");

    let missing = path_to_string(&claude.join("missing"));
    assert!(
        !args.args.iter().any(|arg| arg.starts_with(&missing)),
        "{:#?}",
        args.args
    );
    assert!(!synthetic_mount_target_paths(&args).contains(&claude.join("missing")));
}

/// Assert that `path` is masked due to a bwrap arg sequence like:
///
/// `bwrap ... --perms 000 --ro-bind-data FD PATH`
fn assert_file_masked(args: &[String], path: &Path) {
    let path = path_to_string(path);
    assert!(
        args.windows(5).any(|window| {
            window[0] == "--perms"
                && window[1] == "000"
                && window[2] == "--ro-bind-data"
                && window[4] == path
        }),
        "expected file mask for {path}: {args:#?}"
    );
}

/// Assert that `path` is backed by an fd-supplied empty file without
/// changing the next mount operation's permissions.
fn assert_empty_file_bound_without_perms(args: &[String], path: &Path) {
    let path = path_to_string(path);
    assert!(
        args.windows(3)
            .any(|window| { window[0] == "--ro-bind-data" && window[2] == path }),
        "expected empty file bind for {path}: {args:#?}"
    );
    assert!(
        !args.windows(5).any(|window| {
            window[0] == "--perms"
                && window[1] == "000"
                && window[2] == "--ro-bind-data"
                && window[4] == path
        }),
        "missing path bind should not set explicit file perms for {path}: {args:#?}"
    );
}

fn assert_empty_directory_mounted_read_only(args: &[String], path: &Path) {
    let path = path_to_string(path);
    assert!(
        args.windows(4)
            .any(|window| window == ["--perms", "555", "--tmpfs", path.as_str()]),
        "expected empty directory mount for {path}: {args:#?}"
    );
    assert!(
        args.windows(2)
            .any(|window| window == ["--remount-ro", path.as_str()]),
        "expected read-only remount for {path}: {args:#?}"
    );
}

fn synthetic_mount_target_paths(args: &BwrapArgs) -> Vec<PathBuf> {
    args.synthetic_mount_targets
        .iter()
        .map(|target| target.path().to_path_buf())
        .collect()
}

/// #506: an unlistable directory becomes an empty mode-0111 folder with
/// its entries put back, in the pass that exposes it (the read baseline,
/// or the writable root above it), before that pass's deny masks, and is
/// made read-only last.
#[test]
fn issue_506_unlistable_directory_is_rebuilt_without_listing() {
    let temp_dir = TempDir::new().expect("temp dir");
    let base = temp_dir.path().canonicalize().expect("canonical");
    let home = base.join("home");
    std::fs::create_dir_all(home.join("tmp")).expect("tmp");
    std::fs::write(home.join("secret.txt"), "secret").expect("secret");
    std::os::unix::fs::symlink("tmp", home.join("link")).expect("link");
    let home_path = AbsolutePathBuf::from_absolute_path(&home).expect("absolute");
    let home_str = path_to_string(&home);
    let tmp_str = path_to_string(&home.join("tmp"));
    let secret_str = path_to_string(&home.join("secret.txt"));
    for (root_access, bind) in [
        (FileSystemAccessMode::Read, "--ro-bind-try"),
        (FileSystemAccessMode::Write, "--bind-try"),
    ] {
        let policy = FileSystemSandboxPolicy::restricted(vec![
            FileSystemSandboxEntry::new(
                FileSystemPath::Special {
                    value: FileSystemSpecialPath::Root,
                },
                root_access,
            ),
            FileSystemSandboxEntry::new(
                FileSystemPath::from(home_path.join("secret.txt")),
                FileSystemAccessMode::Deny,
            ),
            FileSystemSandboxEntry::deny_directory_listing(&home_path),
        ]);
        let args = create_filesystem_args(&policy, &base, BwrapOptions::default())
            .expect("filesystem args")
            .args;
        let position = |window: &[&str]| {
            args.windows(window.len())
                .position(|candidate| candidate.iter().zip(window).all(|(a, b)| a == b))
                .unwrap_or_else(|| panic!("{root_access:?}: {window:?} missing in {args:?}"))
        };
        let tmpfs = position(&["--perms", "111", "--tmpfs", &home_str]);
        let tmp = position(&[bind, &tmp_str, &tmp_str]);
        let link = position(&["--symlink", "tmp", &path_to_string(&home.join("link"))]);
        let secret = position(&[bind, &secret_str, &secret_str]);
        let secret_mask = position(&["--perms", "000", "--ro-bind-data"]);
        let remount = position(&["--remount-ro", &home_str]);
        assert!(tmpfs < tmp && tmpfs < link && tmpfs < secret, "{args:?}");
        assert!(secret < secret_mask, "{args:?}");
        assert_eq!(remount, args.len() - 2, "{args:?}");
        if root_access == FileSystemAccessMode::Write {
            assert!(position(&["--bind", "/", "/"]) < tmpfs, "{args:?}");
        }
    }
}

/// #506: full write with only an unlistable directory still needs bwrap.
#[test]
fn issue_506_full_write_with_an_unlistable_directory_is_wrapped() {
    let temp_dir = TempDir::new().expect("temp dir");
    let home = AbsolutePathBuf::from_absolute_path(temp_dir.path()).expect("absolute");
    let policy = FileSystemSandboxPolicy::restricted(vec![
        FileSystemSandboxEntry::new(
            FileSystemPath::Special {
                value: FileSystemSpecialPath::Root,
            },
            FileSystemAccessMode::Write,
        ),
        FileSystemSandboxEntry::deny_directory_listing(&home),
    ]);
    let command = vec!["/bin/true".to_string()];
    let args = create_bwrap_command_args(
        command.clone(),
        &policy,
        Path::new("/"),
        Path::new("/"),
        BwrapOptions::default(),
    )
    .expect("bwrap args");
    assert_ne!(args.args, command);
    assert!(args.args.iter().any(|arg| arg == "--remount-ro"));
}

/// #506 review: a home named `.codex` (or `.agents`) directly under a
/// writable root is also a read-only subpath of that root. Its read-only
/// bind must come before the unlistable folder (not over it), the entries
/// go back read-only, and a missing denied entry's read-only snapshot of
/// the home is made unlistable itself.
#[test]
fn issue_506_unlistable_home_under_a_writable_root_stays_unlistable() {
    let temp_dir = TempDir::new().expect("temp dir");
    let base = temp_dir.path().canonicalize().expect("canonical");
    let workspace = base.join("ws");
    let home = workspace.join(".codex");
    std::fs::create_dir_all(home.join("tmp")).expect("tmp");
    let home_path = AbsolutePathBuf::from_absolute_path(&home).expect("absolute");
    let home_str = path_to_string(&home);
    let tmp_str = path_to_string(&home.join("tmp"));
    let entries = |missing_denied: bool| {
        let mut entries = vec![
            FileSystemSandboxEntry::new(
                FileSystemPath::Special {
                    value: FileSystemSpecialPath::Root,
                },
                FileSystemAccessMode::Read,
            ),
            FileSystemSandboxEntry::new(
                FileSystemPath::from(AbsolutePathBuf::from_absolute_path(&workspace).expect("ws")),
                FileSystemAccessMode::Write,
            ),
            FileSystemSandboxEntry::new(
                FileSystemPath::from(home_path.clone()),
                FileSystemAccessMode::Read,
            ),
            FileSystemSandboxEntry::deny_directory_listing(&home_path),
        ];
        if missing_denied {
            entries.push(FileSystemSandboxEntry::skip_missing_path(
                FileSystemPath::from(home_path.join("wallet")),
                FileSystemAccessMode::Deny,
            ));
        }
        FileSystemSandboxPolicy::restricted(entries)
    };
    let find = |args: &[String], window: &[&str]| {
        args.windows(window.len())
            .position(|candidate| candidate.iter().zip(window).all(|(a, b)| a == b))
    };

    let args = create_filesystem_args(&entries(false), &workspace, BwrapOptions::default())
        .expect("filesystem args")
        .args;
    let read_only = find(&args, &["--ro-bind", &home_str, &home_str]).expect("ro bind");
    let tmpfs = find(&args, &["--perms", "111", "--tmpfs", &home_str]).expect("tmpfs");
    assert!(read_only < tmpfs, "{args:?}");
    assert!(
        find(&args, &["--ro-bind-try", &tmp_str, &tmp_str]).is_some(),
        "{args:?}"
    );
    assert!(
        find(&args, &["--bind-try", &tmp_str, &tmp_str]).is_none(),
        "{args:?}"
    );
    assert_eq!(
        find(&args, &["--remount-ro", &home_str]),
        Some(args.len() - 2),
        "{args:?}"
    );

    let args = create_filesystem_args(&entries(true), &workspace, BwrapOptions::default())
        .expect("filesystem args")
        .args;
    assert!(
        find(&args, &["--perms", "555", "--tmpfs", &home_str]).is_none(),
        "{args:?}"
    );
    let snapshot = find(&args, &["--perms", "111", "--tmpfs", &home_str]).expect("snapshot");
    assert!(read_only_after(&args, snapshot, &home_str), "{args:?}");
    assert!(
        !args
            .iter()
            .any(|arg| arg == &path_to_string(&home.join("wallet"))),
        "{args:?}"
    );
}

/// Nothing binds the real folder back over `dir` after `from`.
fn read_only_after(args: &[String], from: usize, dir: &str) -> bool {
    !args[from..]
        .windows(3)
        .any(|window| window[0].contains("bind") && window[2] == dir)
}

/// #506 review round 2: in a restricted-read profile whose readable root
/// is a symlink, the unlistable folder is mounted where the command sees
/// it (below the root's own spelling), its entries bound from the real
/// location, and nothing is mounted at the real location.
#[test]
fn issue_506_unlistable_home_under_a_symlinked_readable_root() {
    let temp_dir = TempDir::new().expect("temp dir");
    let base = temp_dir.path().canonicalize().expect("canonical");
    let real = base.join("real");
    std::fs::create_dir_all(real.join("home/tmp")).expect("tmp");
    std::fs::write(real.join("home/auth.json"), "secret").expect("auth");
    let link = base.join("link");
    std::os::unix::fs::symlink(&real, &link).expect("link");
    let policy = FileSystemSandboxPolicy::restricted(vec![
        FileSystemSandboxEntry::new(
            FileSystemPath::from(AbsolutePathBuf::from_absolute_path(&link).expect("absolute")),
            FileSystemAccessMode::Read,
        ),
        FileSystemSandboxEntry::deny_directory_listing(
            &AbsolutePathBuf::from_absolute_path(link.join("home")).expect("absolute"),
        ),
    ]);
    let args = create_filesystem_args(&policy, &base, BwrapOptions::default())
        .expect("filesystem args")
        .args;
    let logical = path_to_string(&link.join("home"));
    let real_home = path_to_string(&real.join("home"));
    assert!(
        args.windows(4)
            .any(|window| window == ["--perms", "111", "--tmpfs", logical.as_str()]),
        "{args:?}"
    );
    assert!(
        args.windows(3).any(|window| window
            == [
                "--ro-bind-try",
                path_to_string(&real.join("home/auth.json")).as_str(),
                path_to_string(&link.join("home/auth.json")).as_str(),
            ]),
        "{args:?}"
    );
    // Nothing is mounted at the real spelling, which the profile does not
    // expose: it appears only as a bind source.
    for (index, arg) in args.iter().enumerate() {
        if arg.starts_with(&real_home) {
            assert_eq!(args[index - 1].as_str(), "--ro-bind-try", "{args:?}");
        }
    }
}
