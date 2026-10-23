#![cfg(unix)]

use super::*;
use codex_protocol::permissions::NetworkSandboxPolicy;
use pretty_assertions::assert_eq;

fn abs(path: &Path) -> AbsolutePathBuf {
    AbsolutePathBuf::from_absolute_path(path).unwrap()
}

struct Fixture {
    _dir: tempfile::TempDir,
    user_home: std::path::PathBuf,
    codex_home: std::path::PathBuf,
}

fn fixture() -> Fixture {
    let dir = tempfile::tempdir().unwrap();
    let user_home = dir.path().canonicalize().unwrap().join("user");
    let codex_home = user_home.join(".corbanu");
    for folder in [
        "tmp/arg0",
        "shell_snapshots",
        "skills",
        "sessions",
        "worktrees/w",
    ] {
        std::fs::create_dir_all(codex_home.join(folder)).unwrap();
    }
    for file in ["auth.json", "config.toml", "notes.txt", "state_5.sqlite"] {
        std::fs::write(codex_home.join(file), "x").unwrap();
    }
    std::fs::create_dir_all(user_home.join(".ssh")).unwrap();
    std::fs::write(user_home.join(".ssh/id_ed25519"), "x").unwrap();
    std::fs::create_dir_all(user_home.join("project")).unwrap();
    Fixture {
        _dir: dir,
        user_home,
        codex_home,
    }
}

fn workspace(cwd: &Path) -> PermissionProfile {
    PermissionProfile::workspace_write().materialize_project_roots_with_workspace_roots(&[abs(cwd)])
}

#[test]
fn pf_23_s01_corbanu_home_is_denied_by_default_and_runtime_entries_stay() {
    let fx = fixture();
    let cwd = fx.user_home.join("project");
    let denials = ReadDenials::collect(
        &fx.codex_home,
        Some(&fx.user_home),
        /*claude_config_dir*/ None,
        &[abs(&cwd)],
    );
    let profile = denials.apply(&workspace(&cwd)).unwrap();
    let policy = profile.file_system_sandbox_policy();
    let readable = |path: &Path| policy.can_read_path_with_cwd(path, &cwd);
    for denied in [
        "auth.json",
        "config.toml",
        "notes.txt",
        "sessions",
        "secrets",
        "source-origin.key",
        "memories",
    ] {
        assert!(!readable(&fx.codex_home.join(denied)), "{denied}");
    }
    for kept in ["tmp/arg0", "shell_snapshots", "skills", "worktrees/w"] {
        assert!(readable(&fx.codex_home.join(kept)), "{kept}");
    }
    for denied in [
        ".ssh/id_ed25519",
        ".aws/credentials",
        ".codex",
        ".config/solana",
    ] {
        assert!(!readable(&fx.user_home.join(denied)), "{denied}");
    }
    assert!(readable(&cwd.join("src/main.rs")));
    assert_eq!(
        policy.get_unreadable_globs_with_cwd(&cwd),
        vec![
            fx.codex_home
                .join("*.sqlite*")
                .to_string_lossy()
                .into_owned()
        ]
    );
    // Denials can never be dropped by running the command unsandboxed.
    assert!(policy.has_denied_read_restrictions());
}

#[test]
fn pf_23_s01_full_access_gets_a_sandbox_that_only_denies_reads() {
    let fx = fixture();
    let cwd = fx.user_home.join("project");
    let denials = ReadDenials::collect(
        &fx.codex_home,
        Some(&fx.user_home),
        /*claude_config_dir*/ None,
        &[abs(&cwd)],
    );
    let profile = denials.apply(&PermissionProfile::Disabled).unwrap();
    let policy = profile.file_system_sandbox_policy();
    assert_eq!(policy.kind, FileSystemSandboxKind::Restricted);
    assert!(!policy.can_read_path_with_cwd(&fx.user_home.join(".ssh/id_ed25519"), &cwd));
    assert!(policy.can_write_path_with_cwd(Path::new("/usr/local/anything"), &cwd));
    assert_eq!(
        profile.network_sandbox_policy(),
        NetworkSandboxPolicy::Enabled
    );
    // An external sandbox cannot take the rules.
    assert_eq!(
        denials.apply(&PermissionProfile::External {
            network: NetworkSandboxPolicy::Restricted
        }),
        None
    );
}

#[test]
fn pf_23_s01_a_denial_holding_the_workspace_is_skipped() {
    let fx = fixture();
    // Working inside `~/.ssh` (or a home entry) must not deny the workspace.
    let cwd = fx.user_home.join(".ssh");
    let denials = ReadDenials::collect(
        &fx.codex_home,
        Some(&fx.user_home),
        /*claude_config_dir*/ None,
        &[abs(&cwd)],
    );
    assert!(denials.skipped.contains(&abs(&cwd)));
    assert!(denials.paths().all(|path| !cwd.starts_with(path.as_path())));
    // The working folder inside the Corbanu home skips only its own entry.
    let cwd = fx.codex_home.join("notes.txt");
    let denials = ReadDenials::collect(
        &fx.codex_home,
        Some(&fx.user_home),
        /*claude_config_dir*/ None,
        &[abs(&cwd)],
    );
    assert_eq!(denials.skipped, vec![abs(&cwd)]);
    assert!(
        denials
            .paths()
            .any(|path| path == &abs(&fx.codex_home.join("auth.json")))
    );
}

#[test]
fn pf_23_s01_claude_config_dir_credentials_are_denied() {
    let fx = fixture();
    let cwd = fx.user_home.join("project");
    let claude = fx.user_home.join("claude-config");
    let denials = ReadDenials::collect(
        &fx.codex_home,
        Some(&fx.user_home),
        Some(&claude),
        &[abs(&cwd)],
    );
    assert!(
        denials
            .paths()
            .any(|path| path == &abs(&claude.join(".credentials.json")))
    );
}

/// Review 1: in-process file tools (patch pre-check, structured edits, image
/// view, extension tools) get the same denials once the session is tainted.
/// Under a protected level (here Moderate) the denials also apply before taint
/// (issue #239): a workspace-write sandbox otherwise reads `$HOME` credentials.
#[tokio::test]
async fn pf_23_s01_file_tools_get_the_denials_after_taint() {
    let (session, turn) = crate::session::tests::make_session_and_context().await;
    let client = (*session.services.model_client())
        .clone()
        .with_ingress_level(codex_security_policy::SecurityLevel::Moderate)
        .with_source_envelopes(/*enabled*/ true);
    session.services.replace_model_client(client);
    let environment = turn
        .environments
        .primary()
        .expect("a primary environment")
        .clone();
    let auth = turn.config.codex_home.join("auth.json");
    let readable = |context: codex_file_system::FileSystemSandboxContext| {
        let profile = PermissionProfile::try_from(context.permissions).unwrap();
        #[allow(deprecated)]
        let cwd = turn.cwd.clone();
        profile
            .file_system_sandbox_policy()
            .can_read_path_with_cwd(auth.as_path(), cwd.as_path())
    };
    let context =
        || turn.file_system_sandbox_context(/*additional_permissions*/ None, &environment);
    // Issue #239: under Moderate the denials apply before taint too.
    let before = protect_file_tool_context(&session, &turn, context());
    assert!(readable(context()));
    assert!(!readable(before));

    session
        .services
        .model_client()
        .note_unrecorded_input_for_taint();
    let after = protect_file_tool_context(&session, &turn, context());
    assert!(readable(context()));
    assert!(!readable(after));
}

/// Review 2: host reads of model-named files (Codex Apps uploads) follow the
/// same denials after taint, through symlinks too. Under a protected level
/// the policy is also active before taint (issue #239).
#[tokio::test]
async fn pf_23_s01_upload_reads_follow_the_denials_after_taint() {
    let (session, turn) = crate::session::tests::make_session_and_context().await;
    let client = (*session.services.model_client())
        .clone()
        .with_ingress_level(codex_security_policy::SecurityLevel::Moderate)
        .with_source_envelopes(/*enabled*/ true);
    session.services.replace_model_client(client);
    #[allow(deprecated)]
    let cwd = turn.cwd.clone();
    let home = turn.config.codex_home.to_path_buf();
    std::fs::create_dir_all(&home).unwrap();
    std::fs::write(home.join("auth.json"), "x").unwrap();
    // Issue #239: under Moderate the upload read policy is active before taint.
    let before = post_taint_read_policy(&session, &turn).expect("protected level active");
    assert!(!readable_under(
        &before,
        &home.join("auth.json"),
        cwd.as_path()
    ));
    session
        .services
        .model_client()
        .note_unrecorded_input_for_taint();
    let policy = post_taint_read_policy(&session, &turn).expect("tainted");
    let outside = tempfile::tempdir().unwrap();
    let link = outside.path().join("innocent.txt");
    std::os::unix::fs::symlink(home.join("auth.json"), &link).unwrap();
    let plain = outside.path().join("plain.txt");
    std::fs::write(&plain, "x").unwrap();
    assert!(!readable_under(
        &policy,
        &home.join("auth.json"),
        cwd.as_path()
    ));
    assert!(!readable_under(&policy, &link, cwd.as_path()));
    assert!(readable_under(&policy, &plain, cwd.as_path()));
}

/// Issue #562: the Corbanu home's runtime entries stay read-only where they
/// are missing too (on Linux too: bubblewrap holds a missing one with a
/// placeholder), and `AGENTS.override.md` is denied before it exists.
#[test]
fn issue_562_missing_runtime_entries_stay_read_only() {
    let fx = fixture();
    let cwd = fx.user_home.join("project");
    for missing in [
        "plugins",
        "packages",
        ".tmp",
        "AGENTS.md",
        "AGENTS.override.md",
    ] {
        assert!(!fx.codex_home.join(missing).exists(), "{missing}");
    }
    let denials = ReadDenials::collect(
        &fx.codex_home,
        Some(&fx.user_home),
        /*claude_config_dir*/ None,
        &[abs(&cwd)],
    );
    let policy = denials
        .apply(&PermissionProfile::Disabled)
        .unwrap()
        .file_system_sandbox_policy();
    let writable = |path: &Path| policy.can_write_path_with_cwd(path, &cwd);
    let readable = |path: &Path| policy.can_read_path_with_cwd(path, &cwd);
    for runtime in [
        fx.codex_home.join("AGENTS.md"),
        fx.codex_home.join("plugins/p/plugin.json"),
        fx.codex_home.join("packages/x"),
        fx.codex_home.join(".tmp/x"),
    ] {
        assert!(!writable(&runtime), "{}", runtime.display());
        assert!(readable(&runtime), "{}", runtime.display());
    }
    let override_md = fx.codex_home.join("AGENTS.override.md");
    assert!(!writable(&override_md));
    assert!(!readable(&override_md));
}

/// #583: a protected command cannot create a missing Corbanu-home entry
/// that Corbanu reads later (exec-policy `rules/`, or any other name): the
/// home itself is read-only wherever the profile could write it, and no
/// grant lifts it. `worktrees` stays writable, also when the command works
/// in one, without becoming a root of its own there (whose default rules
/// would make a git worktree's git folder read-only). The profile is never
/// widened.
#[test]
fn issue_583_the_corbanu_home_takes_no_new_entries() {
    let fx = fixture();
    let project = fx.user_home.join("project");
    let worktree = fx.codex_home.join("worktrees/w");
    let created = [
        fx.codex_home.join("rules"),
        fx.codex_home.join("rules/x.rules"),
        fx.codex_home.join("hooks.json"),
        fx.codex_home.join("new.txt"),
    ];
    for (cwd, profile) in [
        (project.clone(), PermissionProfile::Disabled),
        (project.clone(), workspace(&fx.user_home)),
        (worktree.clone(), PermissionProfile::Disabled),
        (worktree.clone(), workspace(&worktree)),
    ] {
        let mut denials = ReadDenials::collect(
            &fx.codex_home,
            Some(&fx.user_home),
            /*claude_config_dir*/ None,
            &[abs(&cwd)],
        );
        // An Aggressive grant lifts the other write rules, not this one.
        denials.lift_write_rules();
        let policy = denials
            .apply(&profile)
            .unwrap()
            .file_system_sandbox_policy();
        let writable = |path: &Path| policy.can_write_path_with_cwd(path, &cwd);
        for path in &created {
            assert!(!writable(path), "{}: {cwd:?} {profile:?}", path.display());
        }
        assert!(!writable(&fx.codex_home), "{cwd:?} {profile:?}");
        assert!(writable(&worktree.join("file")), "{cwd:?} {profile:?}");
        assert!(writable(&cwd.join("file")), "{cwd:?} {profile:?}");
        assert!(
            policy.can_read_path_with_cwd(&fx.codex_home.join("skills/s"), &cwd),
            "{cwd:?} {profile:?}"
        );
        if profile == PermissionProfile::Disabled {
            let roots: Vec<AbsolutePathBuf> = policy
                .get_writable_roots_with_cwd(&cwd)
                .into_iter()
                .map(|root| root.root)
                .collect();
            assert!(!roots.contains(&abs(&worktree)), "{roots:?}");
        }
    }
    // The profile is never widened: one that writes only the project keeps
    // the home's folders read-only.
    let project_only = PermissionProfile::from_runtime_permissions(
        &FileSystemSandboxPolicy::restricted(vec![
            FileSystemSandboxEntry::new(
                FileSystemPath::Special {
                    value: FileSystemSpecialPath::Root,
                },
                FileSystemAccessMode::Read,
            ),
            FileSystemSandboxEntry::new(
                FileSystemPath::Path {
                    path: abs(&project),
                },
                FileSystemAccessMode::Write,
            ),
        ]),
        NetworkSandboxPolicy::Restricted,
    );
    let denials = ReadDenials::collect(
        &fx.codex_home,
        Some(&fx.user_home),
        /*claude_config_dir*/ None,
        &[abs(&project)],
    );
    let policy = denials
        .apply(&project_only)
        .unwrap()
        .file_system_sandbox_policy();
    assert!(!policy.can_write_path_with_cwd(&worktree.join("file"), &project));
    // The home stays writable for a person's approval of one command (#540),
    // a command working in the home itself, or in a folder of it outside
    // `worktrees` (#603 protects only the names Corbanu reads there).
    let other = fx.codex_home.join("other");
    std::fs::create_dir_all(&other).unwrap();
    for (cwd, runtime_only) in [
        (project, true),
        (fx.codex_home.clone(), false),
        (other, false),
    ] {
        let denials = ReadDenials::collect(
            &fx.codex_home,
            Some(&fx.user_home),
            /*claude_config_dir*/ None,
            &[abs(&cwd)],
        );
        let denials = if runtime_only {
            denials.runtime_only()
        } else {
            denials
        };
        let policy = denials
            .apply(&PermissionProfile::Disabled)
            .unwrap()
            .file_system_sandbox_policy();
        assert!(
            policy.can_write_path_with_cwd(&fx.codex_home.join("new.txt"), &cwd),
            "{cwd:?}"
        );
    }
}

/// #603 (owner decision 16): a command working in the Corbanu home itself
/// (its folder, a workspace root or a writable root), or in a folder of it
/// outside `worktrees`, cannot create the names Corbanu reads later, even
/// before they exist; other new names and `worktrees` stay writable, and no
/// grant lifts it. A person's approval of one command under Moderate does
/// (owner decision 15).
#[test]
fn issue_603_names_corbanu_reads_later_are_protected_in_the_home() {
    let fx = fixture();
    let other = fx.codex_home.join("other");
    std::fs::create_dir_all(&other).unwrap();
    let project = fx.user_home.join("project");
    let protected = [
        "rules",
        "rules/x.rules",
        "hooks.json",
        "agents",
        "agents/x",
        "themes/t.tmTheme",
        "environments.toml",
        "app-server-control/app-server-control.sock",
    ];
    // (working folder, extra kept folder, profile)
    let cases = [
        (fx.codex_home.clone(), None, PermissionProfile::Disabled),
        (fx.codex_home.clone(), None, workspace(&fx.codex_home)),
        (other, None, PermissionProfile::Disabled),
        (
            project,
            Some(fx.codex_home.clone()),
            PermissionProfile::Disabled,
        ),
    ];
    for (cwd, root, profile) in cases {
        let mut keep = vec![abs(&cwd)];
        keep.extend(root.as_deref().map(abs));
        let mut denials = ReadDenials::collect(
            &fx.codex_home,
            Some(&fx.user_home),
            /*claude_config_dir*/ None,
            &keep,
        );
        denials.lift_write_rules();
        let policy = denials
            .apply(&profile)
            .unwrap()
            .file_system_sandbox_policy();
        let writable = |path: &Path| policy.can_write_path_with_cwd(path, &cwd);
        for name in protected {
            assert!(!writable(&fx.codex_home.join(name)), "{name}: {cwd:?}");
        }
        for free in ["new.txt", "worktrees/w/file"] {
            assert!(writable(&fx.codex_home.join(free)), "{free}: {cwd:?}");
        }
        assert!(writable(&cwd.join("file")), "{cwd:?}");
        // A person's approval under Moderate lifts it, as it lifts the
        // read-only home (#540).
        let policy = ReadDenials::collect(
            &fx.codex_home,
            Some(&fx.user_home),
            /*claude_config_dir*/ None,
            &keep,
        )
        .runtime_only()
        .apply(&profile)
        .unwrap()
        .file_system_sandbox_policy();
        assert!(
            policy.can_write_path_with_cwd(&fx.codex_home.join("rules/x.rules"), &cwd),
            "{cwd:?}"
        );
    }
    // A command working in a name it would protect keeps its folder.
    let rules = fx.codex_home.join("rules");
    std::fs::create_dir_all(&rules).unwrap();
    let policy = ReadDenials::collect(
        &fx.codex_home,
        Some(&fx.user_home),
        /*claude_config_dir*/ None,
        &[abs(&rules)],
    )
    .apply(&PermissionProfile::Disabled)
    .unwrap()
    .file_system_sandbox_policy();
    assert!(policy.can_write_path_with_cwd(&rules.join("x.rules"), &rules));
    assert!(!policy.can_write_path_with_cwd(&fx.codex_home.join("hooks.json"), &rules));
}

/// #603 (Linux): the missing folders Corbanu reads later are made, private,
/// only for a command working in the home where it could create them, so
/// bubblewrap puts no placeholder file there.
#[test]
fn issue_603_read_later_folders_are_created_only_for_a_command_in_the_home() {
    let fx = fixture();
    let home = abs(&fx.codex_home);
    let project = abs(&fx.user_home.join("project"));
    for (cwd, profile) in [
        (project, PermissionProfile::Disabled),
        (home.clone(), PermissionProfile::read_only()),
    ] {
        ReadDenials::for_turn(
            &fx.codex_home,
            &fx.codex_home,
            &cwd,
            std::slice::from_ref(&cwd),
            &profile,
        );
        assert!(!fx.codex_home.join("rules").exists(), "{cwd:?}");
    }
    ReadDenials::for_turn(
        &fx.codex_home,
        &fx.codex_home,
        &home,
        std::slice::from_ref(&home),
        &PermissionProfile::Disabled,
    );
    for folder in ["rules", "agents", "themes", "app-server-control"] {
        let path = fx.codex_home.join(folder);
        assert_eq!(path.is_dir(), cfg!(target_os = "linux"), "{folder}");
        if cfg!(target_os = "linux") {
            use std::os::unix::fs::PermissionsExt;
            let mode = std::fs::metadata(&path).unwrap().permissions().mode();
            assert_eq!(mode & 0o077, 0, "{folder}: {mode:o}");
        }
    }
    for lock in ["security_level.lock", "gpu-controller.lock"] {
        let path = fx.codex_home.join(lock);
        assert_eq!(path.is_file(), cfg!(target_os = "linux"), "{lock}");
        if cfg!(target_os = "linux") {
            use std::os::unix::fs::PermissionsExt;
            let mode = std::fs::metadata(&path).unwrap().permissions().mode();
            assert_eq!(mode & 0o077, 0, "{lock}: {mode:o}");
        }
    }
    assert!(!fx.codex_home.join("hooks.json").exists());
}

/// #603 review: a kept folder that is a link (`build -> ~/.ssh`) does not
/// skip the denial of what it points to; only top-level aliases are
/// resolved when a denial is matched against the kept folders.
#[cfg(unix)]
#[test]
fn issue_603_a_kept_link_does_not_skip_a_credential_denial() {
    let fx = fixture();
    let ssh = fx.user_home.join(".ssh");
    std::fs::create_dir_all(&ssh).unwrap();
    std::fs::write(ssh.join("id_ed25519"), "key").unwrap();
    let project = fx.user_home.join("project");
    std::fs::create_dir_all(&project).unwrap();
    let build = project.join("build");
    std::os::unix::fs::symlink(&ssh, &build).unwrap();
    for cwd in [project.clone(), build.clone()] {
        let keep = [abs(&project), abs(&build)];
        let policy = ReadDenials::collect(
            &fx.codex_home,
            Some(&fx.user_home),
            /*claude_config_dir*/ None,
            &keep,
        )
        .apply(&PermissionProfile::Disabled)
        .unwrap()
        .file_system_sandbox_policy();
        let key = ssh.join("id_ed25519");
        assert!(!policy.can_read_path_with_cwd(&key, &cwd), "{cwd:?}");
        assert!(!policy.can_write_path_with_cwd(&key, &cwd), "{cwd:?}");
    }
}

/// #603 review: a `worktrees` link (to the home, above it, or `/`) does not
/// switch the protection off for a command working in the home.
#[cfg(unix)]
#[test]
fn issue_603_a_worktrees_link_keeps_the_names_protected() {
    for target in [".", "..", "/"] {
        let fx = fixture();
        let worktrees = fx.codex_home.join("worktrees");
        if worktrees.exists() {
            std::fs::rename(&worktrees, fx.codex_home.join("worktrees.old")).unwrap();
        }
        std::os::unix::fs::symlink(target, &worktrees).unwrap();
        let cwd = fx.codex_home.clone();
        let policy = ReadDenials::collect(
            &fx.codex_home,
            Some(&fx.user_home),
            /*claude_config_dir*/ None,
            &[abs(&cwd)],
        )
        .apply(&PermissionProfile::Disabled)
        .unwrap()
        .file_system_sandbox_policy();
        for name in ["rules/x.rules", "hooks.json", "agents/x"] {
            assert!(
                !policy.can_write_path_with_cwd(&fx.codex_home.join(name), &cwd),
                "{target}: {name}"
            );
        }
    }
}

/// #583 review: a `worktrees` link (to a runtime entry, or to the home
/// itself) opens nothing: only a real `worktrees` folder stays writable.
/// Where the profile could write the home, a missing `worktrees` is made.
#[test]
fn issue_583_a_worktrees_link_opens_nothing() {
    for target in ["tmp", "."] {
        let fx = fixture();
        std::fs::remove_dir_all(fx.codex_home.join("worktrees")).unwrap();
        std::os::unix::fs::symlink(target, fx.codex_home.join("worktrees")).unwrap();
        let cwd = fx.user_home.join("project");
        let mut denials = ReadDenials::collect(
            &fx.codex_home,
            Some(&fx.user_home),
            /*claude_config_dir*/ None,
            &[abs(&cwd)],
        );
        denials.lift_write_rules();
        assert_eq!(denials.home_writable_paths().count(), 0, "{target}");
        let policy = denials
            .apply(&PermissionProfile::Disabled)
            .unwrap()
            .file_system_sandbox_policy();
        for path in ["tmp/x", "rules/x.rules", "worktrees/x"] {
            assert!(
                !policy.can_write_path_with_cwd(&fx.codex_home.join(path), &cwd),
                "{target}: {path}"
            );
        }
    }
    let fx = fixture();
    std::fs::remove_dir_all(fx.codex_home.join("worktrees")).unwrap();
    let cwd = abs(&fx.user_home.join("project"));
    ReadDenials::for_turn(
        &fx.codex_home,
        &fx.codex_home,
        &cwd,
        std::slice::from_ref(&cwd),
        &PermissionProfile::read_only(),
    );
    assert!(!fx.codex_home.join("worktrees").exists());
    ReadDenials::for_turn(
        &fx.codex_home,
        &fx.codex_home,
        &cwd,
        std::slice::from_ref(&cwd),
        &PermissionProfile::Disabled,
    );
    assert!(fx.codex_home.join("worktrees").is_dir());
}

/// #603: the profile of a protected command with full access (the
/// read-only Corbanu home, with the `worktrees` carve-out) and a grant below
/// `worktrees` (an additional-permission write at `worktrees/w`).
fn worktrees_carve_out_policy(fx: &Fixture) -> (FileSystemSandboxPolicy, std::path::PathBuf) {
    let cwd = fx.user_home.join("project");
    let denials = ReadDenials::collect(
        &fx.codex_home,
        Some(&fx.user_home),
        /*claude_config_dir*/ None,
        &[abs(&cwd)],
    );
    let worktrees = abs(&fx.codex_home.join("worktrees"));
    assert_eq!(
        denials.home_writable_paths().collect::<Vec<_>>(),
        vec![&worktrees]
    );
    let mut policy = denials
        .apply(&PermissionProfile::Disabled)
        .unwrap()
        .file_system_sandbox_policy();
    policy.entries.push(FileSystemSandboxEntry::new(
        FileSystemPath::Path {
            path: abs(&fx.codex_home.join("worktrees/w")),
        },
        FileSystemAccessMode::Write,
    ));
    assert!(policy.can_write_path_with_cwd(&fx.codex_home.join("worktrees/w/x"), &cwd));
    assert!(!policy.can_write_path_with_cwd(&fx.codex_home.join("hooks.json"), &cwd));
    (policy, cwd)
}

/// Replaces the home's `worktrees` with a link to `target`, as a command a
/// person approved under Moderate, or another session, could.
fn swap_worktrees_for_a_link(fx: &Fixture, target: &str) {
    let worktrees = fx.codex_home.join("worktrees");
    std::fs::rename(&worktrees, fx.codex_home.join("worktrees.old")).unwrap();
    std::os::unix::fs::symlink(target, &worktrees).unwrap();
}

/// #603 (launch race): `worktrees` is checked to be a real folder when the
/// rules are built; a link swapped in before the sandbox is set up used to
/// be the root's first sight, so the sandboxes granted writes where it led
/// (the home itself, or its runtime `tmp`). Now the command is refused, as
/// is a grant below `worktrees` the swap would move. (A link into a
/// read-only entry such as `tmp` is no writable root at all, #535.)
#[test]
fn issue_603_a_worktrees_link_swapped_in_after_the_check_is_refused() {
    for target in [".", "tmp"] {
        let fx = fixture();
        let (policy, cwd) = worktrees_carve_out_policy(&fx);
        swap_worktrees_for_a_link(&fx, target);
        let worktrees = fx.codex_home.join("worktrees");
        match codex_sandboxing::writable_root_ledger::check_writable_roots(&policy, &cwd) {
            Err(codex_sandboxing::writable_root_ledger::WritableRootRefusal::Changed {
                root,
                ..
            }) => assert_eq!(root, worktrees, "{target}"),
            Err(refusal) => panic!("{target}: {refusal}"),
            Ok(pinned) => {
                let granted: Vec<_> = pinned
                    .iter()
                    .filter(|pinned| pinned.root.as_path().starts_with(&worktrees))
                    .collect();
                assert!(granted.is_empty(), "{target}: {granted:?}");
            }
        }
    }
}

/// #603 review: a grant below `worktrees` (an additional-permission write at
/// `worktrees/w`) where the profile cannot write `worktrees` itself (no
/// carve-out in the policy) is refused too once `worktrees` leads elsewhere;
/// before, it was first seen, and granted, at the link's target.
#[test]
fn issue_603_a_grant_below_a_swapped_worktrees_is_refused() {
    let fx = fixture();
    let cwd = fx.user_home.join("project");
    let denials = ReadDenials::collect(
        &fx.codex_home,
        Some(&fx.user_home),
        /*claude_config_dir*/ None,
        &[abs(&cwd)],
    );
    // The project only: no temporary folders, which hold the fixture.
    let project_only = PermissionProfile::workspace_write_with(
        &[],
        NetworkSandboxPolicy::Restricted,
        /*exclude_tmpdir_env_var*/ true,
        /*exclude_slash_tmp*/ true,
    )
    .materialize_project_roots_with_workspace_roots(&[abs(&cwd)]);
    let mut policy = denials
        .apply(&project_only)
        .unwrap()
        .file_system_sandbox_policy();
    assert!(!policy.can_write_path_with_cwd(&fx.codex_home.join("worktrees/x"), &cwd));
    let grant = fx.codex_home.join("worktrees/w");
    policy.entries.push(FileSystemSandboxEntry::new(
        FileSystemPath::Path { path: abs(&grant) },
        FileSystemAccessMode::Write,
    ));
    swap_worktrees_for_a_link(&fx, ".");
    let refused = codex_sandboxing::writable_root_ledger::check_writable_roots(&policy, &cwd);
    assert!(
        matches!(
            &refused,
            Err(codex_sandboxing::writable_root_ledger::WritableRootRefusal::Changed { root, .. })
                if root == &fx.codex_home.join("worktrees")
        ),
        "{refused:?}"
    );
}

/// #603 review: a `worktrees` left as it was, or replaced by another real
/// folder (removed and made again), runs as before.
#[test]
fn issue_603_a_real_worktrees_is_never_refused() {
    let fx = fixture();
    let (policy, cwd) = worktrees_carve_out_policy(&fx);
    assert!(codex_sandboxing::writable_root_ledger::check_writable_roots(&policy, &cwd).is_ok());
    let worktrees = fx.codex_home.join("worktrees");
    std::fs::rename(&worktrees, fx.codex_home.join("worktrees.old")).unwrap();
    std::fs::create_dir_all(worktrees.join("w")).unwrap();
    assert!(codex_sandboxing::writable_root_ledger::check_writable_roots(&policy, &cwd).is_ok());
    let (policy, cwd) = worktrees_carve_out_policy(&fx);
    assert!(codex_sandboxing::writable_root_ledger::check_writable_roots(&policy, &cwd).is_ok());
}

/// #603 review: a kept folder typed in another case than stored (macOS
/// ignores case) still keeps a denial that holds it from applying, as when
/// `canonicalize` gave the stored case.
#[cfg(target_os = "macos")]
#[test]
fn issue_603_a_kept_folder_in_another_case_is_kept() {
    let fx = fixture();
    let typed = fx.user_home.join(".CORBANU").join("worktrees").join("w");
    if !typed.exists() {
        // A volume that does not ignore case.
        return;
    }
    let denied = abs(&fx.codex_home.join("worktrees"));
    assert!(ReadDenials::holds_kept(&denied, &[abs(&typed)]));
}

/// #603 (launch race) with the real sandbox (`sandbox-exec`, bubblewrap): a
/// command whose `worktrees` was swapped for a link to the home after the
/// rules were built is refused, and creates no `hooks.json` in the home.
/// Before, it ran with the root granted at the home (the home's own
/// read-only rule still refused that write).
#[cfg(any(target_os = "macos", target_os = "linux"))]
#[test]
fn issue_603_a_worktrees_link_swapped_in_refuses_the_command_in_the_real_sandbox() {
    use codex_sandboxing::SandboxCommand;
    use codex_sandboxing::SandboxManager;
    use codex_sandboxing::SandboxTransformRequest;
    use codex_sandboxing::SandboxType;
    use codex_utils_path_uri::PathUri;

    #[cfg(target_os = "linux")]
    let Ok(linux_sandbox_exe) = codex_utils_cargo_bin::cargo_bin("codex-linux-sandbox") else {
        // Skipped where the helper is not built.
        return;
    };
    #[cfg(not(target_os = "linux"))]
    let linux_sandbox_exe = std::path::PathBuf::new();
    let fx = fixture();
    let (policy, cwd) = worktrees_carve_out_policy(&fx);
    swap_worktrees_for_a_link(&fx, ".");
    let profile =
        PermissionProfile::from_runtime_permissions(&policy, NetworkSandboxPolicy::Restricted);
    let cwd_uri = PathUri::from_abs_path(&abs(&cwd));
    let script = format!(
        "printf x > '{}'; echo wrote=$?",
        fx.codex_home.join("worktrees/hooks.json").display()
    );
    let transformed = SandboxManager::new().transform(SandboxTransformRequest {
        command: SandboxCommand {
            program: "/bin/sh".into(),
            args: vec!["-c".to_string(), script],
            cwd: cwd_uri.clone(),
            env: std::collections::HashMap::new(),
            managed_network: None,
            additional_permissions: None,
        },
        permissions: &profile,
        sandbox: if cfg!(target_os = "macos") {
            SandboxType::MacosSeatbelt
        } else {
            SandboxType::LinuxSeccomp
        },
        enforce_managed_network: false,
        environment_id: None,
        network: None,
        sandbox_policy_cwd: &cwd_uri,
        codex_linux_sandbox_exe: Some(&linux_sandbox_exe),
        use_legacy_landlock: false,
        windows_sandbox_level: codex_protocol::config_types::WindowsSandboxLevel::Disabled,
        windows_sandbox_private_desktop: false,
    });
    let output = match transformed {
        Err(codex_sandboxing::SandboxTransformError::WritableRootRefused(refused)) => {
            format!("refused: {refused}")
        }
        Err(err) => panic!("{err}"),
        Ok(request) => {
            let mut command = std::process::Command::new(&request.command[0]);
            command.args(&request.command[1..]).current_dir(&cwd);
            if let Some(arg0) = &request.arg0 {
                std::os::unix::process::CommandExt::arg0(&mut command, arg0);
            }
            let output = command.output().unwrap();
            format!(
                "{}{}",
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr)
            )
        }
    };
    assert!(
        !fx.codex_home.join("hooks.json").exists(),
        "created through the link: {output}"
    );
    assert!(output.starts_with("refused: "), "{output}");
}

/// #583 review: a Corbanu home reached through a link inside the workspace
/// (`~/.corbanu -> /elsewhere/home`) gets no `worktrees` carve-out where the
/// profile never covered the real folder: Seatbelt would resolve the link
/// spelling and make the real `worktrees` writable.
#[test]
fn issue_583_a_linked_home_outside_the_workspace_gets_no_carve_out() {
    let dir = tempfile::tempdir().unwrap();
    let base = dir.path().canonicalize().unwrap();
    let user_home = base.join("user");
    let real_home = base.join("outside/home");
    std::fs::create_dir_all(real_home.join("worktrees/w")).unwrap();
    std::fs::create_dir_all(user_home.join("project")).unwrap();
    std::os::unix::fs::symlink(&real_home, user_home.join(".corbanu")).unwrap();
    let codex_home = user_home.join(".corbanu");
    let cwd = user_home.join("project");
    // A workspace of the user's home only (no temporary-folder roots, which
    // would hold the real home here).
    let home_only = PermissionProfile::from_runtime_permissions(
        &FileSystemSandboxPolicy::restricted(vec![
            FileSystemSandboxEntry::new(
                FileSystemPath::Special {
                    value: FileSystemSpecialPath::Root,
                },
                FileSystemAccessMode::Read,
            ),
            FileSystemSandboxEntry::new(
                FileSystemPath::Path {
                    path: abs(&user_home),
                },
                FileSystemAccessMode::Write,
            ),
        ]),
        NetworkSandboxPolicy::Restricted,
    );
    let denials = ReadDenials::collect(
        &codex_home,
        Some(&user_home),
        /*claude_config_dir*/ None,
        &[abs(&cwd)],
    );
    let policy = denials
        .apply(&home_only)
        .unwrap()
        .file_system_sandbox_policy();
    let worktree_writes: Vec<&FileSystemSandboxEntry> = policy
        .entries
        .iter()
        .filter(|entry| {
            entry.access == FileSystemAccessMode::Write
                && matches!(&entry.path, FileSystemPath::Path { path } if path.as_path().ends_with("worktrees"))
        })
        .collect();
    assert_eq!(worktree_writes, Vec::<&FileSystemSandboxEntry>::new());
}

/// #619: a Corbanu home below a writable root spelled through the `/tmp`
/// link (recorded at `/private/tmp`, where Seatbelt matches the root) gets
/// its read-only entries and the `worktrees` carve-out. A home named `.codex`
/// at that root's top stays protected as the root's metadata: no carve-out
/// opens its `worktrees` (#619 review).
#[cfg(target_os = "macos")]
#[test]
fn issue_619_a_home_below_a_tmp_spelled_root_is_protected() {
    let dir = tempfile::tempdir_in("/tmp").unwrap();
    let logical = dir.path().to_path_buf();
    let real = logical.canonicalize().unwrap();
    assert!(real.starts_with("/private/tmp"), "{real:?}");
    let workspace = |root: &Path| {
        PermissionProfile::from_runtime_permissions(
            &FileSystemSandboxPolicy::restricted(vec![
                FileSystemSandboxEntry::new(
                    FileSystemPath::Special {
                        value: FileSystemSpecialPath::Root,
                    },
                    FileSystemAccessMode::Read,
                ),
                FileSystemSandboxEntry::new(
                    FileSystemPath::Path { path: abs(root) },
                    FileSystemAccessMode::Write,
                ),
            ]),
            NetworkSandboxPolicy::Restricted,
        )
    };
    let worktree_writes = |policy: &FileSystemSandboxPolicy| -> Vec<FileSystemSandboxEntry> {
        policy
            .entries
            .iter()
            .filter(|entry| {
                entry.access == FileSystemAccessMode::Write
                    && matches!(&entry.path, FileSystemPath::Path { path } if path.as_path().ends_with("worktrees"))
            })
            .cloned()
            .collect()
    };
    let cwd = real.join("project");
    std::fs::create_dir_all(&cwd).unwrap();
    // The home below the root.
    let home = real.join("dot/home");
    std::fs::create_dir_all(home.join("worktrees")).unwrap();
    let profile = workspace(&logical);
    let denials = ReadDenials::collect(
        &home,
        /*user_home*/ None,
        /*claude_config_dir*/ None,
        &[abs(&cwd)],
    );
    let policy = denials
        .apply(&profile)
        .unwrap()
        .file_system_sandbox_policy();
    for path in ["AGENTS.md", "rules/x.rules"] {
        assert!(
            !policy.can_write_effective_path_with_cwd(&home.join(path), &cwd),
            "{path}"
        );
    }
    assert!(policy.can_write_effective_path_with_cwd(&home.join("worktrees/w"), &cwd));
    assert_eq!(worktree_writes(&policy).len(), 1);
    // A home that is the root's `.codex`: the root's metadata.
    let home = real.join(".codex");
    std::fs::create_dir_all(home.join("worktrees")).unwrap();
    let denials = ReadDenials::collect(
        &home,
        /*user_home*/ None,
        /*claude_config_dir*/ None,
        &[abs(&cwd)],
    );
    let policy = denials
        .apply(&profile)
        .unwrap()
        .file_system_sandbox_policy();
    assert_eq!(
        worktree_writes(&policy),
        Vec::<FileSystemSandboxEntry>::new()
    );
    assert!(!policy.can_write_effective_path_with_cwd(&home.join("worktrees/w"), &cwd));
}

/// Issue #652: a profile naming the folder holding the Corbanu home in two
/// spellings with different access (`/tmp/x` writable, `/private/tmp/x`
/// read-only). Seatbelt grants the writable one, so the home's entries are
/// still made read-only, but no `worktrees` write carve-out is added: one
/// reading of the profile keeps it read-only.
#[cfg(target_os = "macos")]
#[test]
fn issue_652_a_folder_named_twice_adds_no_worktrees_carve_out() {
    let dir = tempfile::tempdir_in("/tmp").unwrap();
    let logical = dir.path().to_path_buf();
    let real = logical.canonicalize().unwrap();
    let entry = |path: &Path, access| {
        FileSystemSandboxEntry::new(FileSystemPath::Path { path: abs(path) }, access)
    };
    let profile = PermissionProfile::from_runtime_permissions(
        &FileSystemSandboxPolicy::restricted(vec![
            FileSystemSandboxEntry::new(
                FileSystemPath::Special {
                    value: FileSystemSpecialPath::Root,
                },
                FileSystemAccessMode::Read,
            ),
            entry(&logical, FileSystemAccessMode::Write),
            entry(&real, FileSystemAccessMode::Read),
        ]),
        NetworkSandboxPolicy::Restricted,
    );
    let cwd = real.join("project");
    std::fs::create_dir_all(&cwd).unwrap();
    let home = real.join("home");
    std::fs::create_dir_all(home.join("worktrees")).unwrap();
    let denials = ReadDenials::collect(
        &home,
        /*user_home*/ None,
        /*claude_config_dir*/ None,
        &[abs(&cwd)],
    );
    let policy = denials
        .apply(&profile)
        .unwrap()
        .file_system_sandbox_policy();
    assert!(!policy.can_write_effective_path_with_cwd(&home.join("AGENTS.md"), &cwd));
    assert!(!policy.can_write_effective_path_with_cwd(&home.join("worktrees/w"), &cwd));
    assert!(
        !policy.entries.iter().any(|entry| entry.access == FileSystemAccessMode::Write
            && matches!(&entry.path, FileSystemPath::Path { path } if path.as_path().ends_with("worktrees"))),
        "{:?}",
        policy.entries
    );
}

/// Issue #586: the database init lock is denied before it exists, so a
/// protected command can neither create it nor hold it, also where #583
/// leaves the Corbanu home writable (a command working in the home or in a
/// folder of it outside `worktrees`).
#[test]
fn issue_586_db_init_lock_denied_before_it_exists() {
    let fx = fixture();
    let lock = fx.codex_home.join("pfterminal_db_init.lock");
    assert!(!lock.exists());
    std::fs::create_dir_all(fx.codex_home.join("other")).unwrap();
    for cwd in [
        fx.user_home.join("project"),
        fx.codex_home.clone(),
        fx.codex_home.join("other"),
    ] {
        let denials = ReadDenials::collect(
            &fx.codex_home,
            Some(&fx.user_home),
            /*claude_config_dir*/ None,
            &[abs(&cwd)],
        );
        let policy = denials
            .apply(&PermissionProfile::Disabled)
            .unwrap()
            .file_system_sandbox_policy();
        assert!(
            !policy.can_write_path_with_cwd(&lock, &cwd),
            "{}",
            cwd.display()
        );
        assert!(
            !policy.can_read_path_with_cwd(&lock, &cwd),
            "{}",
            cwd.display()
        );
    }
}

/// Issue #611: a database folder set apart from the Corbanu home is denied
/// whole (all Corbanu keeps there). Where it holds the command's folder, the
/// databases (reads also of one created later, writes of existing ones),
/// the init lock, the GPU controller's lock, pending accounting records and
/// backups are denied one by one, also before they exist; other files stay.
/// Nothing is added when the folder is the home (by another spelling).
#[test]
fn issue_611_a_separate_database_folder_is_denied_like_the_home() {
    let fx = fixture();
    let databases = fx.user_home.join("db");
    std::fs::create_dir_all(databases.join("accounting-pending")).unwrap();
    std::fs::write(databases.join("state_5.sqlite"), "x").unwrap();
    let profile = PermissionProfile::Disabled;
    let denied = |cwd: &AbsolutePathBuf, name: &str| {
        let denials = ReadDenials::for_turn(
            &fx.codex_home,
            &databases,
            cwd,
            std::slice::from_ref(cwd),
            &profile,
        );
        let policy = denials
            .apply(&profile)
            .unwrap()
            .file_system_sandbox_policy();
        let path = databases.join(name);
        let read = denials.matcher().unwrap().is_read_denied(&path);
        let write = !policy.can_write_path_with_cwd(&path, cwd.as_path());
        (read, write)
    };
    // Elsewhere: the whole folder.
    let project = abs(&fx.user_home.join("project"));
    for name in [
        "state_5.sqlite",
        "later.sqlite-wal",
        "pfterminal_db_init.lock",
        "gpu-controller.lock",
        "accounting-pending/x.json",
        "db-backups/state_5.sqlite",
        "notes.txt",
    ] {
        assert_eq!(denied(&project, name), (true, true), "{name}");
    }
    // Working inside it: what Corbanu keeps there, one by one.
    let inside = abs(&databases.join("work"));
    std::fs::create_dir_all(inside.as_path()).unwrap();
    for (name, expected) in [
        ("state_5.sqlite", (true, true)),
        ("pfterminal_db_init.lock", (true, true)),
        ("gpu-controller.lock", (true, true)),
        ("accounting-pending/x.json", (true, true)),
        ("db-backups/x", (true, true)),
        ("later.sqlite", (true, false)),
        ("notes.txt", (false, false)),
        ("work/file", (false, false)),
    ] {
        assert_eq!(denied(&inside, name), expected, "{name}");
    }
    // The database folder is the home (by another spelling): nothing added.
    let same = ReadDenials::for_turn(
        &fx.codex_home,
        &fx.codex_home.join("."),
        &project,
        std::slice::from_ref(&project),
        &profile,
    );
    let home = ReadDenials::for_turn(
        &fx.codex_home,
        &fx.codex_home,
        &project,
        std::slice::from_ref(&project),
        &profile,
    );
    assert_eq!(same, home);
}

/// Issue #651: a database folder (or Corbanu home) whose path holds glob
/// syntax is escaped in the database glob, so a database created later is
/// still covered and nothing else is; databases recovery set aside beside
/// the folder (`<folder>.db-backups`) are denied when present; and a command
/// working in the home cannot create `accounting-pending` or `db-backups`.
#[test]
fn issue_651_glob_syntax_and_backups_of_the_database_folder() {
    let fx = fixture();
    let profile = PermissionProfile::Disabled;
    let databases = fx.user_home.join("db[1]{2*?");
    let inside = abs(&databases.join("work"));
    std::fs::create_dir_all(inside.as_path()).unwrap();
    let backups = fx.user_home.join("db[1]{2*?.db-backups/sqlite-1-0");
    std::fs::create_dir_all(&backups).unwrap();
    let denials = ReadDenials::for_turn(
        &fx.codex_home,
        &databases,
        &inside,
        std::slice::from_ref(&inside),
        &profile,
    );
    let matcher = denials.matcher().expect("a valid matcher");
    for denied in [
        databases.join("later.sqlite"),
        databases.join("later.sqlite-wal"),
        backups.join("state_5.sqlite"),
    ] {
        assert!(matcher.is_read_denied(&denied), "{}", denied.display());
    }
    for readable in [
        databases.join("notes.txt"),
        inside.join("file").to_path_buf(),
        fx.user_home.join("db1/later.sqlite"),
    ] {
        assert!(!matcher.is_read_denied(&readable), "{}", readable.display());
    }

    // A Corbanu home whose path holds glob syntax.
    let home = fx.user_home.join("h[o]me");
    std::fs::create_dir_all(&home).unwrap();
    let project = abs(&fx.user_home.join("project"));
    let denials = ReadDenials::for_turn(
        &home,
        &home,
        &project,
        std::slice::from_ref(&project),
        &profile,
    );
    let matcher = denials.matcher().expect("a valid matcher");
    assert!(matcher.is_read_denied(&home.join("later.sqlite")));
    assert!(!matcher.is_read_denied(&fx.user_home.join("home/later.sqlite")));

    // Working in the home: neither may be created.
    let in_home = abs(&fx.codex_home);
    let denials = ReadDenials::for_turn(
        &fx.codex_home,
        &fx.codex_home,
        &in_home,
        std::slice::from_ref(&in_home),
        &profile,
    );
    let policy = denials
        .apply(&profile)
        .unwrap()
        .file_system_sandbox_policy();
    for name in ["accounting-pending/x.json", "db-backups/x"] {
        let path = fx.codex_home.join(name);
        assert!(
            !policy.can_write_path_with_cwd(&path, in_home.as_path()),
            "{name}"
        );
    }
}

/// Issue #651: a pattern below a folder reached through a link whose real
/// location holds glob syntax is copied with that location escaped.
#[test]
fn issue_651_real_location_of_a_pattern_is_escaped() {
    let dir = tempfile::tempdir().unwrap();
    let base = dir.path().canonicalize().unwrap();
    let real = base.join("db[1]{2");
    std::fs::create_dir_all(&real).unwrap();
    std::os::unix::fs::symlink(&real, base.join("link")).unwrap();
    let pattern = format!("{}/*.sqlite*", base.join("link").display());
    assert_eq!(
        real_pattern(&pattern),
        Some(format!("{}/db[[]1][{{]2/*.sqlite*", base.display()))
    );
}

/// Issue #651 item 7: a separate database folder missing at launch (state
/// start-up failed) is denied whole, as when it exists, so a command can
/// neither create it nor put anything in it. On Linux it is first made,
/// private, with what Corbanu keeps there (bubblewrap would otherwise hold
/// its path with a placeholder file on the host); elsewhere it stays missing
/// and is denied before it exists. Not made where the command could not
/// make it.
#[test]
fn issue_651_a_missing_database_folder_is_denied_whole() {
    let fx = fixture();
    let profile = PermissionProfile::Disabled;
    let databases = fx.user_home.join("data/db");
    let project = abs(&fx.user_home.join("project"));
    let denials = ReadDenials::for_turn(
        &fx.codex_home,
        &databases,
        &project,
        std::slice::from_ref(&project),
        &profile,
    );
    let folder = abs(&databases);
    let linux = cfg!(target_os = "linux");
    assert_eq!(databases.is_dir(), linux);
    if linux {
        use std::os::unix::fs::PermissionsExt;
        let mode = std::fs::metadata(&databases).unwrap().permissions().mode();
        assert_eq!(mode & 0o777, 0o700);
        assert!(databases.join("accounting-pending").is_dir());
        assert!(databases.join("gpu-controller.lock").is_file());
        assert!(denials.existing.contains(&folder), "{denials:?}");
    } else {
        assert!(denials.fixed.contains(&folder), "{denials:?}");
    }
    // Whole: nothing is denied one by one in it.
    assert!(
        !denials
            .paths()
            .any(|path| path.as_path().starts_with(&databases) && *path != folder),
        "{denials:?}"
    );
    let policy = denials
        .apply(&profile)
        .unwrap()
        .file_system_sandbox_policy();
    let matcher = denials.matcher().unwrap();
    for path in ["notes.txt", "later.sqlite", "sub/x"]
        .map(|name| databases.join(name))
        .into_iter()
        .chain([databases])
    {
        assert!(matcher.is_read_denied(&path), "{}", path.display());
        assert!(
            !policy.can_write_path_with_cwd(&path, project.as_path()),
            "{}",
            path.display()
        );
    }

    // Where the command could not make it, it is not made.
    let elsewhere = fx.user_home.join("other/db");
    let denials = ReadDenials::for_turn(
        &fx.codex_home,
        &elsewhere,
        &project,
        std::slice::from_ref(&project),
        &PermissionProfile::read_only(),
    );
    assert!(!fx.user_home.join("other").exists());
    assert!(denials.fixed.contains(&abs(&elsewhere)), "{denials:?}");

    // Nor where a folder above it is reached through a link (one a command
    // may have planted): it would be made wherever the link points.
    let target = fx.user_home.join("target");
    std::fs::create_dir_all(&target).unwrap();
    std::os::unix::fs::symlink(&target, fx.user_home.join("linked")).unwrap();
    let linked = fx.user_home.join("linked/sub/db");
    let denials = ReadDenials::for_turn(
        &fx.codex_home,
        &linked,
        &project,
        std::slice::from_ref(&project),
        &profile,
    );
    assert!(!target.join("sub").exists());
    assert!(denials.fixed.contains(&abs(&linked)), "{denials:?}");
}

/// Issue #562: on Linux the missing runtime folders are created, empty,
/// only where the profile could write the Corbanu home, so bubblewrap never
/// needs a placeholder file where Corbanu expects a folder. `AGENTS.md` is
/// never created.
#[test]
fn issue_562_missing_runtime_folders_are_created_only_where_writable() {
    let fx = fixture();
    let cwd = abs(&fx.user_home.join("project"));
    let folders = ["plugins", "packages", ".tmp"];
    ReadDenials::for_turn(
        &fx.codex_home,
        &fx.codex_home,
        &cwd,
        std::slice::from_ref(&cwd),
        &PermissionProfile::read_only(),
    );
    for folder in folders {
        assert!(!fx.codex_home.join(folder).exists(), "{folder}");
    }
    ReadDenials::for_turn(
        &fx.codex_home,
        &fx.codex_home,
        &cwd,
        std::slice::from_ref(&cwd),
        &PermissionProfile::Disabled,
    );
    for folder in folders {
        assert_eq!(
            fx.codex_home.join(folder).is_dir(),
            cfg!(target_os = "linux"),
            "{folder}"
        );
    }
    assert!(!fx.codex_home.join("AGENTS.md").exists());
    assert!(!fx.codex_home.join("AGENTS.override.md").exists());
}

/// PF-23-S02: files that run code later become read-only wherever the
/// command could write them; ordinary files stay writable.
#[test]
fn pf_23_s02_persistence_files_become_read_only() {
    let fx = fixture();
    let cwd = fx.user_home.join("project");
    std::fs::create_dir_all(cwd.join(".git/hooks")).unwrap();
    // An existing start-up file; the ones below do not exist.
    std::fs::write(fx.user_home.join(".bashrc"), "x").unwrap();
    let denials = ReadDenials::collect(
        &fx.codex_home,
        Some(&fx.user_home),
        /*claude_config_dir*/ None,
        &[abs(&cwd)],
    );
    let policy = denials
        .apply(&PermissionProfile::Disabled)
        .unwrap()
        .file_system_sandbox_policy();
    let writable = |path: &Path| policy.can_write_path_with_cwd(path, &cwd);
    let readable = |path: &Path| policy.can_read_path_with_cwd(path, &cwd);
    let missing_home_files = [
        fx.user_home.join(".zshrc"),
        fx.user_home.join(".config/fish/config.fish"),
        fx.user_home.join("Library/LaunchAgents/x.plist"),
        fx.user_home.join(".local/bin/corbanu"),
        fx.user_home.join(".claude/settings.json"),
    ];
    // Linux leaves missing home files to the command-text net: the sandbox
    // would put an empty placeholder in the real home for the command's run.
    for missing in &missing_home_files {
        assert_eq!(
            writable(missing),
            cfg!(target_os = "linux"),
            "{}",
            missing.display()
        );
    }
    for protected in [
        fx.user_home.join(".bashrc"),
        cwd.join(".git/hooks/pre-commit"),
        cwd.join(".git/config"),
        cwd.join(".codex/config.toml"),
        fx.codex_home.join("skills/x/SKILL.md"),
        fx.codex_home.join("shell_snapshots/s.sh"),
        fx.codex_home.join("tmp/arg0/apply_patch"),
    ] {
        assert!(!writable(&protected), "{}", protected.display());
        assert!(readable(&protected), "{}", protected.display());
    }
    for open in [
        cwd.join("src/main.rs"),
        cwd.join(".git/objects/ab"),
        fx.user_home.join("notes.txt"),
        fx.codex_home.join("worktrees/w/file"),
    ] {
        assert!(writable(&open), "{}", open.display());
    }
    // A credential stays unreadable inside a read-only folder.
    assert!(!readable(&fx.user_home.join(".claude/.credentials.json")));
}

/// PF-23-S02: the rules only narrow. A read-only entry never makes a path
/// readable that the profile did not let the command read, and every
/// denial of the profile itself stays.
#[test]
fn pf_23_s02_rules_never_widen_and_keep_existing_denials() {
    let fx = fixture();
    let cwd = fx.user_home.join("project");
    let secret = fx.user_home.join("private");
    let entry = |path: &Path, access| {
        FileSystemSandboxEntry::new(FileSystemPath::Path { path: abs(path) }, access)
    };
    let base = PermissionProfile::from_runtime_permissions(
        &FileSystemSandboxPolicy::restricted(vec![
            entry(&cwd, FileSystemAccessMode::Write),
            entry(&secret, FileSystemAccessMode::Deny),
        ]),
        NetworkSandboxPolicy::Restricted,
    );
    let denials = ReadDenials::collect(
        &fx.codex_home,
        Some(&fx.user_home),
        /*claude_config_dir*/ None,
        &[abs(&cwd)],
    );
    let policy = denials.apply(&base).unwrap().file_system_sandbox_policy();
    for unreadable in [fx.user_home.join(".zshrc"), fx.codex_home.join("skills")] {
        assert!(!policy.can_read_path_with_cwd(&unreadable, &cwd));
    }
    assert!(!policy.can_read_path_with_cwd(&secret, &cwd));
    assert!(policy.can_write_path_with_cwd(&cwd.join("a.txt"), &cwd));
    assert!(!policy.can_write_path_with_cwd(&cwd.join(".codex/config.toml"), &cwd));
}

/// PF-23-S02 / issue #239: under Aggressive and Moderate the rules apply
/// from the start of the session, before any untrusted content; Permissive
/// never gets them.
#[tokio::test]
async fn pf_23_s02_protected_levels_apply_the_rules_from_the_start() {
    let (session, turn) = crate::session::tests::make_session_and_context().await;
    let environment = turn
        .environments
        .primary()
        .expect("a primary environment")
        .clone();
    let auth = turn.config.codex_home.join("auth.json");
    #[allow(deprecated)]
    let cwd = turn.cwd.clone();
    let readable = |context: codex_file_system::FileSystemSandboxContext| {
        PermissionProfile::try_from(context.permissions)
            .unwrap()
            .file_system_sandbox_policy()
            .can_read_path_with_cwd(auth.as_path(), cwd.as_path())
    };
    // Issue #239: Moderate too, not only Aggressive.
    for (level, protected) in [
        (codex_security_policy::SecurityLevel::Permissive, false),
        (codex_security_policy::SecurityLevel::Moderate, true),
        (codex_security_policy::SecurityLevel::Aggressive, true),
    ] {
        let client = (*session.services.model_client())
            .clone()
            .with_ingress_level(level)
            .with_source_envelopes(/*enabled*/ true);
        session.services.replace_model_client(client);
        let context =
            turn.file_system_sandbox_context(/*additional_permissions*/ None, &environment);
        let protected_context = protect_file_tool_context(&session, &turn, context.clone());
        assert_eq!(readable(protected_context), !protected, "{level:?}");
        assert_eq!(
            post_taint_read_policy(&session, &turn).is_some(),
            protected,
            "{level:?}"
        );
    }
}

/// PF-23-S02: worktrees keep hooks in the git folder their `.git` file
/// points to; a path below a regular file is left out (the Linux sandbox
/// cannot mount there); a symlinked dotfile is protected where it really is.
#[test]
fn pf_23_s02_worktree_hooks_and_symlinked_dotfiles_are_protected() {
    let fx = fixture();
    let main = fx.user_home.join("main");
    let common = main.join(".git");
    let own = common.join("worktrees/w");
    std::fs::create_dir_all(own.as_path()).unwrap();
    std::fs::create_dir_all(common.join("hooks")).unwrap();
    std::fs::write(own.join("commondir"), "../..\n").unwrap();
    let cwd = fx.user_home.join("w");
    std::fs::create_dir_all(&cwd).unwrap();
    std::fs::write(cwd.join(".git"), format!("gitdir: {}\n", own.display())).unwrap();
    let dotfiles = fx.user_home.join("dotfiles");
    std::fs::create_dir_all(&dotfiles).unwrap();
    std::fs::write(dotfiles.join("zshrc"), "x").unwrap();
    std::os::unix::fs::symlink(dotfiles.join("zshrc"), fx.user_home.join(".zshrc")).unwrap();

    let denials = ReadDenials::collect(
        &fx.codex_home,
        Some(&fx.user_home),
        /*claude_config_dir*/ None,
        &[abs(&cwd)],
    );
    let read_only: Vec<_> = denials.read_only_paths().cloned().collect();
    let mut protected_paths = vec![
        cwd.join(".git"),
        common.join("hooks"),
        common.join("config"),
        own.join("config.worktree"),
        own.join("commondir"),
        dotfiles.join("zshrc"),
    ];
    if cfg!(target_os = "macos") {
        protected_paths.push(common.join("commondir"));
    }
    for protected in protected_paths {
        assert!(
            read_only.contains(&abs(&protected)),
            "{}",
            protected.display()
        );
    }
    // Nothing below the `.git` file.
    assert!(
        read_only
            .iter()
            .all(|path| path.as_path() == cwd.join(".git")
                || !path.as_path().starts_with(cwd.join(".git"))),
        "{read_only:?}"
    );
    let policy = denials
        .apply(&PermissionProfile::Disabled)
        .unwrap()
        .file_system_sandbox_policy();
    assert!(!policy.can_write_path_with_cwd(&common.join("hooks/pre-commit"), &cwd));
    assert!(!policy.can_write_path_with_cwd(&dotfiles.join("zshrc"), &cwd));
    assert!(policy.can_write_path_with_cwd(&cwd.join("src.rs"), &cwd));
}

/// PF-23-S02 review 2: a session in a subfolder still protects the
/// repository's hooks and config; a link to a missing target protects the
/// target's path.
#[test]
fn pf_23_s02_enclosing_repo_and_dangling_links_are_protected() {
    let fx = fixture();
    let repo = fx.user_home.join("repo");
    std::fs::create_dir_all(repo.join(".git/hooks")).unwrap();
    let cwd = repo.join("codex-rs");
    std::fs::create_dir_all(&cwd).unwrap();
    std::os::unix::fs::symlink(
        fx.user_home.join("dotfiles/zshrc"),
        fx.user_home.join(".zshrc"),
    )
    .unwrap();
    let denials = ReadDenials::collect(
        &fx.codex_home,
        Some(&fx.user_home),
        /*claude_config_dir*/ None,
        &[abs(&cwd)],
    );
    let read_only: Vec<_> = denials.read_only_paths().cloned().collect();
    let mut protected_paths = vec![repo.join(".git/hooks"), repo.join(".git/config")];
    if cfg!(target_os = "macos") {
        // Linux leaves out a missing commondir and a dangling home link.
        protected_paths.push(repo.join(".git/commondir"));
        protected_paths.push(fx.user_home.join("dotfiles/zshrc"));
    }
    for protected in protected_paths {
        assert!(
            read_only.contains(&abs(&protected)),
            "{}",
            protected.display()
        );
    }
}

/// PF-23-S02 follow-up: submodule git folders (nested ones and ones whose
/// path has a slash) and a `core.hooksPath` folder inside the workspace
/// (`.husky`) are read-only too; the rest of the workspace stays writable.
#[test]
fn pf_23_s02_submodules_and_hooks_path_are_protected() {
    let fx = fixture();
    let cwd = fx.user_home.join("project");
    let git = cwd.join(".git");
    for module in ["modules/a", "modules/a/modules/inner", "modules/libs/b"] {
        for folder in ["hooks", "objects"] {
            std::fs::create_dir_all(git.join(module).join(folder)).unwrap();
        }
        std::fs::write(git.join(module).join("HEAD"), "ref: refs/heads/main\n").unwrap();
        std::fs::write(git.join(module).join("config"), "").unwrap();
    }
    std::fs::write(
        git.join("config"),
        "[core]\n\tbare = false\n[core \"x\"]\n[CORE]\n\thooksPath = \".husky\"\n",
    )
    .unwrap();
    std::fs::create_dir_all(cwd.join(".husky")).unwrap();

    let denials = ReadDenials::collect(
        &fx.codex_home,
        Some(&fx.user_home),
        /*claude_config_dir*/ None,
        &[abs(&cwd)],
    );
    let policy = denials
        .apply(&PermissionProfile::Disabled)
        .unwrap()
        .file_system_sandbox_policy();
    let writable = |path: &Path| policy.can_write_path_with_cwd(path, &cwd);
    for protected in [
        git.join("modules/a/hooks/pre-commit"),
        git.join("modules/a/config"),
        git.join("modules/a/modules/inner/hooks/post-checkout"),
        git.join("modules/libs/b/hooks/pre-push"),
        cwd.join(".husky/pre-commit"),
    ] {
        assert!(!writable(&protected), "{}", protected.display());
    }
    for open in [
        git.join("modules/a/objects/ab"),
        cwd.join("src/main.rs"),
        cwd.join(".huskyrc"),
    ] {
        assert!(writable(&open), "{}", open.display());
    }
}

/// Issue #239 regression: under a protected level (Moderate or Aggressive)
/// with a live policy and `source_envelopes` on, the read denials apply
/// *before* any untrusted content. A workspace-write sandbox otherwise lets
/// an agent read credential files from `$HOME` before the session is tainted.
#[tokio::test]
async fn issue_239_read_denials_apply_under_protected_level_before_taint() {
    use codex_security_policy::SecurityLevel;
    let (session, turn) = crate::session::tests::make_session_and_context().await;
    // Bind the effective security policy so the level is live (as the real
    // session does), not just an ingress-level hint.
    let control = session
        .services
        .agent_control
        .clone()
        .with_effective_security_policy(
            SecurityLevel::Moderate,
            session.thread_id,
            /*inherits_from_spawn_parent*/ false,
        )
        .expect("policy");
    let client = (*session.services.model_client())
        .clone()
        .with_ingress_policy(SecurityLevel::Moderate, control.effective_security_policy())
        .with_source_envelopes(/*enabled*/ true);
    session.services.replace_model_client(client);

    let environment = turn
        .environments
        .primary()
        .expect("a primary environment")
        .clone();
    let auth = turn.config.codex_home.join("auth.json");
    let readable = |context: codex_file_system::FileSystemSandboxContext| {
        let profile = PermissionProfile::try_from(context.permissions).unwrap();
        #[allow(deprecated)]
        let cwd = turn.cwd.clone();
        profile
            .file_system_sandbox_policy()
            .can_read_path_with_cwd(auth.as_path(), cwd.as_path())
    };
    let context =
        || turn.file_system_sandbox_context(/*additional_permissions*/ None, &environment);

    // No taint has occurred yet, but the level is Moderate: denials apply.
    assert_eq!(
        session
            .services
            .model_client()
            .post_taint_state()
            .expect("source envelopes on")
            .taint_generation,
        0
    );
    let before_taint = protect_file_tool_context(&session, &turn, context());
    assert!(
        readable(context()),
        "baseline workspace-write reads the credential file"
    );
    assert!(
        !readable(before_taint),
        "issue #239: read denials must apply under a protected level before taint"
    );

    // The host upload read policy is also active before taint under a level.
    let policy = post_taint_read_policy(&session, &turn).expect("protected level active");
    #[allow(deprecated)]
    let cwd = turn.cwd.clone();
    assert!(
        !readable_under(&policy, &auth, cwd.as_path()),
        "issue #239: upload read policy must deny credentials before taint"
    );
}

/// #442: under Aggressive a grant lifts the write protections only. Reads of
/// credential files and the Corbanu home stay denied, not even once granted.
#[test]
fn issue_442_a_grant_keeps_every_read_denial() {
    let fx = fixture();
    let cwd = fx.user_home.join("project");
    std::fs::create_dir_all(cwd.join(".git/hooks")).unwrap();
    let mut denials = ReadDenials::collect(
        &fx.codex_home,
        Some(&fx.user_home),
        /*claude_config_dir*/ None,
        &[abs(&cwd)],
    );
    denials.lift_write_rules();
    let policy = denials
        .apply(&PermissionProfile::Disabled)
        .unwrap()
        .file_system_sandbox_policy();
    for denied in [
        fx.user_home.join(".ssh/id_ed25519"),
        fx.user_home.join(".aws/credentials"),
        fx.codex_home.join("auth.json"),
        fx.codex_home.join("config.toml"),
        fx.codex_home.join("secrets/local.age"),
    ] {
        assert!(
            !policy.can_read_path_with_cwd(&denied, &cwd),
            "{}",
            denied.display()
        );
    }
    assert!(policy.can_write_path_with_cwd(&cwd.join(".git/hooks/pre-commit"), &cwd));
    // The Corbanu home's runtime entries (sandbox launchers in `tmp`) stay
    // read-only even with a grant, wherever the command could write.
    let full = denials
        .apply(&PermissionProfile::Disabled)
        .unwrap()
        .file_system_sandbox_policy();
    for runtime in [
        fx.codex_home.join("tmp/arg0/codex-linux-sandbox"),
        fx.codex_home.join("skills/x/SKILL.md"),
        fx.codex_home.join("shell_snapshots/s.sh"),
    ] {
        assert!(
            !full.can_write_path_with_cwd(&runtime, &cwd),
            "{}",
            runtime.display()
        );
    }
}

/// #512: the upload check resolves links on both sides. A Corbanu home
/// reached through a link (`/tmp` and `/var` on macOS, a linked home) spells
/// its denials through the link; neither a link to a denied file nor the
/// file's real name may pass. A more specific allow inside a denied folder
/// still applies under either spelling, and nothing outside is narrowed.
#[test]
fn issue_512_upload_reads_resolve_links_on_both_sides() {
    let dir = tempfile::tempdir().unwrap();
    let base = dir.path().canonicalize().unwrap();
    let real_home = base.join("real-home");
    std::fs::create_dir_all(real_home.join("vault")).unwrap();
    std::fs::write(real_home.join("auth.json"), "x").unwrap();
    std::fs::write(real_home.join("vault/key"), "x").unwrap();
    std::fs::write(real_home.join("vault/open.txt"), "x").unwrap();
    let linked_home = base.join("linked-home");
    std::os::unix::fs::symlink(&real_home, &linked_home).unwrap();
    let outside = base.join("outside");
    std::fs::create_dir_all(&outside).unwrap();
    let to_auth = outside.join("innocent.txt");
    std::os::unix::fs::symlink(linked_home.join("auth.json"), &to_auth).unwrap();
    let plain = outside.join("plain.txt");
    std::fs::write(&plain, "x").unwrap();

    let entry = |path: &Path, access| {
        FileSystemSandboxEntry::new(FileSystemPath::Path { path: abs(path) }, access)
    };
    let policy = FileSystemSandboxPolicy::restricted(vec![
        FileSystemSandboxEntry::new(
            FileSystemPath::Special {
                value: codex_protocol::permissions::FileSystemSpecialPath::Root,
            },
            FileSystemAccessMode::Read,
        ),
        entry(&linked_home.join("auth.json"), FileSystemAccessMode::Deny),
        entry(&linked_home.join("vault"), FileSystemAccessMode::Deny),
        entry(
            &linked_home.join("vault/open.txt"),
            FileSystemAccessMode::Read,
        ),
    ]);
    let cwd = outside.as_path();
    for denied in [
        linked_home.join("auth.json"),
        real_home.join("auth.json"),
        to_auth,
        linked_home.join("vault/key"),
        real_home.join("vault/key"),
    ] {
        assert!(
            !readable_under(&policy, &denied, cwd),
            "{}",
            denied.display()
        );
    }
    for open in [
        plain,
        linked_home.join("vault/open.txt"),
        real_home.join("vault/open.txt"),
    ] {
        assert!(readable_under(&policy, &open, cwd), "{}", open.display());
    }
}

/// #506: with the home's listing denied, the sandbox gets a listing denial
/// for the Corbanu home and nothing else changes: its runtime entries stay
/// readable, its stores stay denied. Without it (Moderate), no listing
/// denial; a command working in the home itself keeps listing it.
#[test]
fn issue_506_home_listing_denial_is_added_only_when_asked() {
    let fx = fixture();
    let cwd = fx.user_home.join("project");
    let home = abs(&fx.codex_home);
    let unlistable = |denials: &ReadDenials| {
        denials
            .apply(&workspace(&cwd))
            .unwrap()
            .file_system_sandbox_policy()
            .get_unlistable_directories()
    };
    let mut denials = ReadDenials::collect(
        &fx.codex_home,
        Some(&fx.user_home),
        /*claude_config_dir*/ None,
        &[abs(&cwd)],
    );
    assert!(unlistable(&denials).is_empty());
    denials.deny_home_listing();
    assert_eq!(unlistable(&denials), vec![home.clone()]);
    let policy = denials
        .apply(&workspace(&cwd))
        .unwrap()
        .file_system_sandbox_policy();
    assert!(policy.can_read_path_with_cwd(&fx.codex_home.join("skills"), &cwd));
    assert!(!policy.can_read_path_with_cwd(&fx.codex_home.join("auth.json"), &cwd));

    // Full access gets a sandbox that only denies reads and the listing.
    let full = denials.apply(&PermissionProfile::Disabled).unwrap();
    let full = full.file_system_sandbox_policy();
    assert!(full.can_write_path_with_cwd(&cwd.join("file.txt"), &cwd));
    assert_eq!(full.get_unlistable_directories(), vec![home.clone()]);

    let mut in_home = ReadDenials::collect(
        &fx.codex_home,
        Some(&fx.user_home),
        /*claude_config_dir*/ None,
        &[home],
    );
    in_home.deny_home_listing();
    assert_eq!(in_home.unlistable_home(), None);
}

/// #506: under Aggressive, in-process file tools (which list folders inside
/// the sandbox) cannot list the Corbanu home either; under Moderate they can.
#[tokio::test]
async fn issue_506_file_tools_cannot_list_the_home_under_aggressive() {
    use codex_security_policy::SecurityLevel;
    for (level, denied) in [
        (SecurityLevel::Aggressive, true),
        (SecurityLevel::Moderate, false),
    ] {
        let (session, turn) = crate::session::tests::make_session_and_context().await;
        let client = (*session.services.model_client())
            .clone()
            .with_ingress_level(level)
            .with_source_envelopes(/*enabled*/ true);
        session.services.replace_model_client(client);
        let environment = turn
            .environments
            .primary()
            .expect("a primary environment")
            .clone();
        let context =
            turn.file_system_sandbox_context(/*additional_permissions*/ None, &environment);
        let protected = protect_file_tool_context(&session, &turn, context);
        let profile = PermissionProfile::try_from(protected.permissions).unwrap();
        let unlistable = profile
            .file_system_sandbox_policy()
            .get_unlistable_directories();
        // The home as the sandbox sees it (its real location).
        assert_eq!(
            unlistable
                .iter()
                .any(|path| path.as_path().file_name()
                    == turn.config.codex_home.as_path().file_name()),
            denied,
            "{level:?}: {unlistable:?}"
        );
        assert!(unlistable.len() <= 1, "{unlistable:?}");
    }
}

/// #540: where a person's approval lifts Moderate's rules for one command,
/// only the runtime entries' rule stays: they remain read-only, also under
/// full access (which then gets a sandbox for it), while credential reads
/// and persistence writes are lifted.
#[test]
fn issue_540_runtime_entries_stay_read_only_when_the_rules_are_lifted() {
    let fx = fixture();
    let cwd = fx.user_home.join("project");
    std::fs::write(fx.user_home.join(".bashrc"), "x").unwrap();
    for profile in [workspace(&fx.user_home), PermissionProfile::Disabled] {
        let all = ReadDenials::collect(
            &fx.codex_home,
            Some(&fx.user_home),
            /*claude_config_dir*/ None,
            &[abs(&cwd)],
        );
        assert!(all.read_only_paths().count() > all.runtime_read_only_paths().count());
        let denials = all.runtime_only();
        // #838: only Corbanu's security state and keys stay denied.
        let mut denied: Vec<&str> = denials
            .paths()
            .map(|path| path.as_path().file_name().unwrap().to_str().unwrap())
            .collect();
        denied.sort_unstable();
        assert_eq!(
            denied,
            [
                "security_confirm.lock",
                "security_level.lock",
                "security_level.toml",
                "security_migration.toml",
                "security_preflight.toml",
                "security_state.json",
                "security_state.lock",
                "source-origin.key",
            ]
        );
        assert_eq!(
            denials.read_only_paths().count(),
            denials.runtime_read_only_paths().count()
        );
        let lifted = denials.apply(&profile).expect("not an external sandbox");
        let policy = lifted.file_system_sandbox_policy();
        assert_eq!(
            policy.kind,
            FileSystemSandboxKind::Restricted,
            "{profile:?}"
        );
        let writable = |path: &Path| policy.can_write_path_with_cwd(path, &cwd);
        for runtime in ["tmp", "tmp/arg0/launcher", "shell_snapshots", "skills"] {
            assert!(
                !writable(&fx.codex_home.join(runtime)),
                "{runtime}: {profile:?}"
            );
        }
        // Persistence files are lifted (the profile's own rules aside).
        assert!(writable(&fx.user_home.join(".bashrc")), "{profile:?}");
        assert!(writable(&cwd.join("notes.txt")), "{profile:?}");
        assert!(
            policy.can_read_path_with_cwd(&fx.user_home.join(".ssh/id_ed25519"), &cwd),
            "{profile:?}"
        );
        assert!(
            policy.can_read_path_with_cwd(&fx.codex_home.join("auth.json"), &cwd),
            "{profile:?}"
        );
    }
    // Full access keeps its network.
    let lifted = ReadDenials::collect(
        &fx.codex_home,
        Some(&fx.user_home),
        /*claude_config_dir*/ None,
        &[abs(&cwd)],
    )
    .runtime_only()
    .apply(&PermissionProfile::Disabled)
    .unwrap();
    assert_eq!(
        lifted.network_sandbox_policy(),
        NetworkSandboxPolicy::Enabled
    );
}

/// #838 (Travis, 2026-10-15): where a person's approval lifts Moderate's
/// rules for one command, Corbanu's security state and keys stay denied,
/// read and write, existing or not, also under full access and for a command
/// working in the home, while the other stores are lifted (#540). Before the
/// approval they are denied as before.
#[test]
fn issue_838_security_state_and_keys_stay_denied_when_the_rules_are_lifted() {
    let fx = fixture();
    let cwd = fx.user_home.join("project");
    let existing = [
        "source-origin.key",
        "security_state.json",
        "security_preflight.toml",
        "security_confirm.lock",
    ];
    for name in existing {
        std::fs::write(fx.codex_home.join(name), "x").unwrap();
    }
    // Missing: denied before they exist, as before the approval.
    let missing = [
        "security_level.toml",
        "security_state.lock",
        "security_state.lock/x",
    ];
    for (cwd, profile) in [
        (cwd.clone(), workspace(&cwd)),
        (cwd.clone(), PermissionProfile::Disabled),
        (fx.codex_home.clone(), PermissionProfile::Disabled),
    ] {
        let all = ReadDenials::collect(
            &fx.codex_home,
            Some(&fx.user_home),
            /*claude_config_dir*/ None,
            &[abs(&cwd)],
        );
        let before = all.apply(&profile).unwrap().file_system_sandbox_policy();
        let policy = all
            .runtime_only()
            .apply(&profile)
            .unwrap()
            .file_system_sandbox_policy();
        for name in existing.iter().chain(&missing) {
            let path = fx.codex_home.join(name);
            for policy in [&before, &policy] {
                assert!(
                    !policy.can_read_path_with_cwd(&path, &cwd),
                    "{name}: {cwd:?} {profile:?}"
                );
                assert!(
                    !policy.can_write_path_with_cwd(&path, &cwd),
                    "{name}: {cwd:?} {profile:?}"
                );
            }
        }
        // The other stores are lifted for the approved command.
        assert!(
            policy.can_read_path_with_cwd(&fx.codex_home.join("auth.json"), &cwd),
            "{cwd:?} {profile:?}"
        );
        assert!(
            policy.can_read_path_with_cwd(&fx.codex_home.join("config.toml"), &cwd),
            "{cwd:?} {profile:?}"
        );
    }
    // The same files of another Corbanu home (denied whole before).
    let other = fx.user_home.join(".pfterminal");
    std::fs::create_dir_all(&other).unwrap();
    std::fs::write(other.join("source-origin.key"), "x").unwrap();
    std::fs::write(other.join("auth.json"), "x").unwrap();
    let all = ReadDenials::collect(
        &fx.codex_home,
        Some(&fx.user_home),
        /*claude_config_dir*/ None,
        &[abs(&cwd)],
    );
    let before = all
        .apply(&PermissionProfile::Disabled)
        .unwrap()
        .file_system_sandbox_policy();
    let policy = all
        .runtime_only()
        .apply(&PermissionProfile::Disabled)
        .unwrap()
        .file_system_sandbox_policy();
    for name in ["source-origin.key", "security_state.json"] {
        for policy in [&before, &policy] {
            assert!(
                !policy.can_read_path_with_cwd(&other.join(name), &cwd),
                "{name}"
            );
            assert!(
                !policy.can_write_path_with_cwd(&other.join(name), &cwd),
                "{name}"
            );
        }
    }
    assert!(policy.can_read_path_with_cwd(&other.join("auth.json"), &cwd));
    // Explanations name them by any spelling of the home.
    let link = fx.user_home.join("home-link");
    std::os::unix::fs::symlink(&fx.codex_home, &link).unwrap();
    // A home spelled through a link: denied by their real location (which
    // bubblewrap can deny; it refuses a deny through a writable link).
    let approved = ReadDenials::collect(
        &link,
        Some(&fx.user_home),
        /*claude_config_dir*/ None,
        &[abs(&cwd)],
    )
    .runtime_only();
    let denied: Vec<&AbsolutePathBuf> = approved.paths().collect();
    for name in [
        "source-origin.key",
        "security_state.json",
        "security_level.toml",
    ] {
        assert!(
            denied.contains(&&abs(&fx.codex_home.join(name))),
            "{name}: {denied:?}"
        );
    }
    assert_eq!(
        denied.iter().any(|path| path.as_path().starts_with(&link)),
        cfg!(target_os = "macos"),
        "{denied:?}"
    );
    for path in [
        fx.codex_home.join("security_state.json"),
        link.join("source-origin.key"),
        link.join("security_level.toml"),
    ] {
        assert!(
            ReadDenials::is_security_file(&fx.codex_home, Some(&fx.user_home), &path),
            "{path:?}"
        );
        assert!(
            ReadDenials::is_security_file(&link, Some(&fx.user_home), &path),
            "{path:?}"
        );
    }
    for path in [
        fx.codex_home.join("auth.json"),
        fx.user_home.join("security_state.json"),
    ] {
        assert!(
            !ReadDenials::is_security_file(&fx.codex_home, Some(&fx.user_home), &path),
            "{path:?}"
        );
    }
    assert!(ReadDenials::is_security_file(
        &fx.codex_home,
        Some(&fx.user_home),
        &other.join("source-origin.key")
    ));
    // A project's own `.codex` is not a Corbanu home.
    assert!(!ReadDenials::is_security_file(
        &fx.codex_home,
        Some(&fx.user_home),
        &cwd.join(".codex/security_level.toml")
    ));
}

/// #838 (review r2): where an approved command could write the folder of a
/// missing security file, Linux leaves it out (bubblewrap would hold it with
/// an empty placeholder Corbanu reads as a broken security state) and makes
/// the missing security locks first, empty and private; macOS denies it
/// before it exists. Existing ones stay denied everywhere.
#[test]
fn issue_838_missing_security_files_where_the_home_is_writable() {
    let fx = fixture();
    let cwd = abs(&fx.user_home.join("project"));
    std::fs::write(fx.codex_home.join("source-origin.key"), "x").unwrap();
    let denials = ReadDenials::for_turn(
        &fx.codex_home,
        &fx.codex_home,
        &cwd,
        std::slice::from_ref(&cwd),
        &PermissionProfile::Disabled,
    )
    .runtime_only();
    let denied: Vec<&AbsolutePathBuf> = denials.paths().collect();
    let denied_here = |name: &str| denied.contains(&&abs(&fx.codex_home.join(name)));
    assert!(denied_here("source-origin.key"), "{denied:?}");
    assert_eq!(
        denied_here("security_state.json"),
        !cfg!(target_os = "linux"),
        "{denied:?}"
    );
    for lock in [
        "security_state.lock",
        "security_confirm.lock",
        "security_level.lock",
    ] {
        let path = fx.codex_home.join(lock);
        if cfg!(target_os = "linux") {
            let metadata = std::fs::metadata(&path).expect(lock);
            assert_eq!(metadata.len(), 0, "{lock}");
            assert_eq!(
                std::os::unix::fs::PermissionsExt::mode(&metadata.permissions()) & 0o777,
                0o600,
                "{lock}"
            );
        }
        assert!(denied_here(lock), "{lock}: {denied:?}");
    }
    // Where the home is not writable (the project only: no temporary
    // folders, which hold the fixture), every one stays denied before it
    // exists, also on Linux.
    let project_only = PermissionProfile::workspace_write_with(
        &[],
        NetworkSandboxPolicy::Restricted,
        /*exclude_tmpdir_env_var*/ true,
        /*exclude_slash_tmp*/ true,
    )
    .materialize_project_roots_with_workspace_roots(std::slice::from_ref(&cwd));
    let denials = ReadDenials::for_turn(
        &fx.codex_home,
        &fx.codex_home,
        &cwd,
        std::slice::from_ref(&cwd),
        &project_only,
    )
    .runtime_only();
    assert!(
        denials
            .paths()
            .any(|path| *path == abs(&fx.codex_home.join("security_migration.toml")))
    );
}

/// A user home reached through a link (`user -> real-user`) holding a
/// dummy SSH key, with a workspace whose `.agents` links to that key.
struct LinkedHome {
    _dir: tempfile::TempDir,
    real_home: std::path::PathBuf,
    linked_home: std::path::PathBuf,
    codex_home: std::path::PathBuf,
    cwd: std::path::PathBuf,
}

fn linked_home_with_agents_link_to_key() -> LinkedHome {
    let dir = tempfile::tempdir().unwrap();
    let base = dir.path().canonicalize().unwrap();
    let real_home = base.join("real-user");
    std::fs::create_dir_all(real_home.join(".ssh")).unwrap();
    std::fs::write(real_home.join(".ssh/id_ed25519"), "DUMMY_SSH_KEY_518").unwrap();
    std::fs::create_dir_all(real_home.join("project")).unwrap();
    std::fs::write(real_home.join("project/README.md"), "readme").unwrap();
    std::os::unix::fs::symlink("../.ssh/id_ed25519", real_home.join("project/.agents")).unwrap();
    let linked_home = base.join("user");
    std::os::unix::fs::symlink(&real_home, &linked_home).unwrap();
    let codex_home = linked_home.join(".corbanu");
    std::fs::create_dir_all(codex_home.join("tmp")).unwrap();
    let cwd = linked_home.join("project");
    LinkedHome {
        _dir: dir,
        real_home,
        linked_home,
        codex_home,
        cwd,
    }
}

/// #518: a read-only entry never reopens a denied file. With the home
/// reached through a link, `.agents -> ../.ssh/id_ed25519` became a `Read`
/// entry at the key's real path, more specific than the `.ssh` denial
/// spelled through the link, so a full-write profile could read the key.
#[test]
fn issue_518_read_only_entries_never_reopen_a_denial_through_a_linked_home() {
    let fx = linked_home_with_agents_link_to_key();
    let cwd = fx.cwd.as_path();
    let denials = ReadDenials::collect(
        &fx.codex_home,
        Some(&fx.linked_home),
        /*claude_config_dir*/ None,
        &[abs(cwd)],
    );
    for profile in [PermissionProfile::Disabled, workspace(cwd)] {
        let policy = denials
            .apply(&profile)
            .unwrap()
            .file_system_sandbox_policy();
        let real_key = fx.real_home.join(".ssh/id_ed25519");
        // No entry at or above the key's real path grants a read the
        // denial does not: the sandboxes see the more specific entry first.
        assert!(
            !policy.entries.iter().any(|entry| entry.access.can_read()
                && matches!(&entry.path, FileSystemPath::Path { path }
                    if real_key.starts_with(path.as_path()) && path.as_path() != Path::new("/"))),
            "{profile:?}: {:?}",
            policy.entries
        );
        for key in [
            real_key.clone(),
            fx.linked_home.join(".ssh/id_ed25519"),
            cwd.join(".agents"),
            fx.real_home.join("project/.agents"),
        ] {
            assert!(
                !readable_under(&policy, &key, cwd),
                "{profile:?}: {}",
                key.display()
            );
        }
        assert!(
            !policy.can_write_path_with_cwd(&real_key, cwd),
            "{profile:?}"
        );
        assert!(readable_under(&policy, &cwd.join("README.md"), cwd));
    }
    // Nothing else is narrowed: the runtime entries stay readable in both
    // spellings, and read-only at their real location (where every
    // platform keeps the entry; macOS also keeps the link spelling).
    let policy = denials
        .apply(&PermissionProfile::Disabled)
        .unwrap()
        .file_system_sandbox_policy();
    let real_runtime = fx.real_home.join(".corbanu/tmp");
    for runtime in [fx.codex_home.join("tmp"), real_runtime.clone()] {
        assert!(
            readable_under(&policy, &runtime.join("arg0"), cwd),
            "{runtime:?}"
        );
    }
    assert!(!policy.can_write_path_with_cwd(&real_runtime.join("arg0"), cwd));
}

/// #518 review: a workspace link to a key that does not exist yet is
/// checked where it would lead, so it never becomes readable when created.
#[test]
fn issue_518_a_dangling_link_to_a_denied_folder_stays_denied() {
    let fx = linked_home_with_agents_link_to_key();
    let cwd = fx.cwd.as_path();
    std::fs::remove_file(cwd.join(".agents")).unwrap();
    std::os::unix::fs::symlink("../.ssh/id_new", cwd.join(".agents")).unwrap();
    let denials = ReadDenials::collect(
        &fx.codex_home,
        Some(&fx.linked_home),
        /*claude_config_dir*/ None,
        &[abs(cwd)],
    );
    let policy = denials
        .apply(&PermissionProfile::Disabled)
        .unwrap()
        .file_system_sandbox_policy();
    assert!(!readable_under(&policy, &cwd.join(".agents"), cwd));
    let new_key = fx.real_home.join(".ssh/id_new");
    assert!(!policy.entries.iter().any(|entry| entry.access.can_read()
        && matches!(&entry.path, FileSystemPath::Path { path } if path.as_path() == new_key)));
    std::fs::write(&new_key, "DUMMY").unwrap();
    assert!(!readable_under(&policy, &cwd.join(".agents"), cwd));
    assert!(!readable_under(&policy, &new_key, cwd));
}

/// #518 review: another Corbanu home name linking to the active home is
/// the active home: its runtime entries stay readable, its stores denied.
#[test]
fn issue_518_a_home_alias_link_keeps_runtime_entries_readable() {
    let fx = fixture();
    let alias = fx.user_home.join(".codex");
    std::os::unix::fs::symlink(&fx.codex_home, &alias).unwrap();
    let cwd = fx.user_home.join("project");
    let denials = ReadDenials::collect(
        &fx.codex_home,
        Some(&fx.user_home),
        /*claude_config_dir*/ None,
        &[abs(&cwd)],
    );
    assert!(!denials.paths().any(|path| path.as_path() == alias));
    let policy = denials
        .apply(&PermissionProfile::Disabled)
        .unwrap()
        .file_system_sandbox_policy();
    for runtime in [fx.codex_home.join("tmp/arg0"), alias.join("tmp/arg0")] {
        assert!(readable_under(&policy, &runtime, &cwd), "{runtime:?}");
    }
    for store in [fx.codex_home.join("auth.json"), alias.join("auth.json")] {
        assert!(!readable_under(&policy, &store, &cwd), "{store:?}");
    }
}

/// #518 with the real macOS sandbox: a full-write command could `cat` the
/// key through `.agents` and under its real name.
#[cfg(target_os = "macos")]
#[test]
fn issue_518_sandboxed_commands_cannot_read_the_key_through_a_linked_home() {
    use codex_sandboxing::seatbelt::CreateSeatbeltCommandArgsParams;
    use codex_sandboxing::seatbelt::MACOS_PATH_TO_SEATBELT_EXECUTABLE;
    use codex_sandboxing::seatbelt::create_seatbelt_command_args;

    let fx = linked_home_with_agents_link_to_key();
    let cwd = fx.cwd.as_path();
    let denials = ReadDenials::collect(
        &fx.codex_home,
        Some(&fx.linked_home),
        /*claude_config_dir*/ None,
        &[abs(cwd)],
    );
    let policy = denials
        .apply(&PermissionProfile::Disabled)
        .unwrap()
        .file_system_sandbox_policy();
    let script = format!(
        "cat .agents; echo agents=$?; cat '{real}'; echo real=$?; cat '{link}'; echo link=$?; \
         cat README.md; echo readme=$?",
        real = fx.real_home.join(".ssh/id_ed25519").display(),
        link = fx.linked_home.join(".ssh/id_ed25519").display(),
    );
    let args = create_seatbelt_command_args(CreateSeatbeltCommandArgsParams {
        command: vec!["/bin/sh".to_string(), "-c".to_string(), script],
        file_system_sandbox_policy: &policy,
        network_sandbox_policy: NetworkSandboxPolicy::Restricted,
        sandbox_policy_cwd: cwd,
        enforce_managed_network: false,
        managed_network: None,
        environment_id: None,
        network: None,
        extra_allow_unix_sockets: &[],
    })
    .unwrap();
    let output = std::process::Command::new(MACOS_PATH_TO_SEATBELT_EXECUTABLE)
        .args(&args)
        .current_dir(cwd)
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(!stdout.contains("DUMMY_SSH_KEY_518"), "{stdout}");
    for line in ["agents=1", "real=1", "link=1", "readme=0"] {
        assert!(stdout.contains(line), "{line} not in {stdout}");
    }
}

/// #518 review: a dangling link whose target climbs through another link
/// (`d/../.ssh/id_new` with `d -> ../Documents`) is checked where the
/// system would resolve it, and a chain too long to follow is not readable.
#[test]
fn issue_518_dangling_links_resolve_through_the_folders_they_name() {
    let fx = linked_home_with_agents_link_to_key();
    let cwd = fx.cwd.as_path();
    std::fs::create_dir_all(fx.real_home.join("Documents")).unwrap();
    std::os::unix::fs::symlink("../Documents", cwd.join("d")).unwrap();
    std::fs::remove_file(cwd.join(".agents")).unwrap();
    std::os::unix::fs::symlink("d/../.ssh/id_new", cwd.join(".agents")).unwrap();
    for hop in 0..9 {
        let next = if hop == 8 {
            "missing".to_string()
        } else {
            format!("chain{}", hop + 1)
        };
        std::os::unix::fs::symlink(&next, cwd.join(format!("chain{hop}"))).unwrap();
    }
    let denials = ReadDenials::collect(
        &fx.codex_home,
        Some(&fx.linked_home),
        /*claude_config_dir*/ None,
        &[abs(cwd)],
    );
    let policy = denials
        .apply(&PermissionProfile::Disabled)
        .unwrap()
        .file_system_sandbox_policy();
    assert!(!readable_under(&policy, &cwd.join(".agents"), cwd));
    assert!(!readable_under(&policy, &cwd.join("chain0"), cwd));
    assert!(readable_under(&policy, &cwd.join("not-yet.txt"), cwd));
}

/// #518 review: a home name whose real folder only holds the active home
/// (`~/.corbanu -> ~/.codex/corbanu`) stays denied whole: its own stores
/// are not the active home's.
#[test]
fn issue_518_a_home_name_above_the_active_home_stays_denied() {
    let dir = tempfile::tempdir().unwrap();
    let user_home = dir.path().canonicalize().unwrap();
    let legacy = user_home.join(".codex");
    let codex_home = legacy.join("corbanu");
    std::fs::create_dir_all(codex_home.join("tmp")).unwrap();
    std::fs::write(legacy.join("auth.json"), "DUMMY").unwrap();
    std::os::unix::fs::symlink(&codex_home, user_home.join(".corbanu")).unwrap();
    let active = user_home.join(".corbanu");
    let cwd = user_home.join("project");
    std::fs::create_dir_all(&cwd).unwrap();
    let denials = ReadDenials::collect(
        &active,
        Some(&user_home),
        /*claude_config_dir*/ None,
        &[abs(&cwd)],
    );
    let policy = denials
        .apply(&workspace(&cwd))
        .unwrap()
        .file_system_sandbox_policy();
    assert!(!readable_under(&policy, &legacy.join("auth.json"), &cwd));
}

/// #442 (Travis, 2026-10-11): a grant never lifts the project's `.codex` and
/// `.agents`, existing or not; it still lifts git hooks and config.
#[test]
fn issue_442_a_grant_keeps_codex_and_agents_read_only() {
    let fx = fixture();
    let cwd = fx.user_home.join("project");
    std::fs::create_dir_all(cwd.join(".git/hooks")).unwrap();
    std::fs::create_dir_all(cwd.join(".codex")).unwrap();
    let mut denials = ReadDenials::collect(
        &fx.codex_home,
        Some(&fx.user_home),
        /*claude_config_dir*/ None,
        &[abs(&cwd)],
    );
    denials.lift_write_rules();
    let policy = denials
        .apply(&PermissionProfile::Disabled)
        .unwrap()
        .file_system_sandbox_policy();
    for kept in [
        ".codex/config.toml",
        ".codex",
        ".agents/skills/x/SKILL.md",
        ".agents",
    ] {
        assert!(
            !policy.can_write_path_with_cwd(&cwd.join(kept), &cwd),
            "{kept}"
        );
    }
    for lifted in [".git/hooks/pre-commit", ".git/config"] {
        assert!(
            policy.can_write_path_with_cwd(&cwd.join(lifted), &cwd),
            "{lifted}"
        );
    }
}

/// #442 (Travis, 2026-10-12): exactly what "Grant once" lifts, as the grant
/// review, the inspector and the docs describe it (`GRANT_ONCE_LIFTS` in the
/// TUI): the shell and git start-up files, login agents and services and
/// `.claude` in the user's home, a `core.hooksPath` hooks folder, and git
/// files inside the project's `.git` (which the Aggressive sandbox keeps
/// read-only anyway, with all of `.git`). A new kind of lifted path
/// fails here until the descriptions name it.
#[test]
fn issue_442_a_grant_lifts_only_the_described_kinds() {
    let fx = fixture();
    // The current folder holds the user's home, the case where the most is
    // lifted.
    let cwd = fx.user_home.clone();
    let git = cwd.join(".git");
    std::fs::create_dir_all(git.join("hooks")).unwrap();
    std::fs::write(git.join("HEAD"), "ref: refs/heads/main\n").unwrap();
    std::fs::write(git.join("config"), "[core]\n\thooksPath = .husky\n").unwrap();
    std::fs::create_dir_all(cwd.join(".husky")).unwrap();
    for entry in USER_PERSISTENCE {
        let path = cwd.join(entry);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        if !path.exists() {
            std::fs::write(&path, "").unwrap();
        }
    }
    let denials = ReadDenials::collect(
        &fx.codex_home,
        Some(&fx.user_home),
        /*claude_config_dir*/ None,
        &[abs(&cwd)],
    );
    let never: Vec<_> = denials.never_lifted_paths().cloned().collect();
    let lifted: Vec<_> = denials
        .read_only_paths()
        .filter(|path| !never.contains(path))
        .cloned()
        .collect();
    let described = |path: &AbsolutePathBuf| {
        let Ok(relative) = path.as_path().strip_prefix(&cwd) else {
            return false;
        };
        // Inside the project's `.git` (hooks, config, `config.worktree`,
        // `commondir`): read-only under Aggressive whatever the grant.
        USER_PERSISTENCE
            .iter()
            .any(|entry| relative == Path::new(entry))
            || relative.starts_with(".git")
            || relative == Path::new(".husky")
    };
    assert!(!lifted.is_empty());
    let undescribed: Vec<_> = lifted.iter().filter(|path| !described(path)).collect();
    assert_eq!(undescribed, Vec::<&AbsolutePathBuf>::new());
    for kind in [
        ".zshrc",
        ".claude",
        "Library/LaunchAgents",
        ".config/systemd",
        ".husky",
    ] {
        assert!(
            lifted.iter().any(|path| path.as_path() == cwd.join(kind)),
            "{kind} not lifted: {lifted:?}"
        );
    }
}

/// #594/#598: the Corbanu home may not be moved in any spelling (as given,
/// its real location, and a home name or home variable linking to it), also
/// for a command a person approved (#540); a name for another folder is not
/// recorded.
#[test]
fn issue_594_598_the_home_is_unmovable_in_each_spelling() {
    let fx = fixture();
    let real = fx.user_home.join("dotfiles/corbanu");
    std::fs::create_dir_all(fx.user_home.join("dotfiles")).unwrap();
    std::fs::rename(&fx.codex_home, &real).unwrap();
    std::os::unix::fs::symlink(&real, &fx.codex_home).unwrap();
    let elsewhere = fx.user_home.join("project");
    let cwd = abs(&elsewhere);
    let mut denials = ReadDenials::collect(
        &real,
        Some(&fx.user_home),
        /*claude_config_dir*/ None,
        std::slice::from_ref(&cwd),
    );
    denials.add_home_spellings(
        &abs(&real),
        [elsewhere, fx.user_home.join("missing"), "relative".into()],
    );
    let expected = vec![abs(&real), abs(&fx.codex_home)];
    assert_eq!(
        denials.unmovable_paths().cloned().collect::<Vec<_>>(),
        expected
    );
    let approved = denials.runtime_only();
    assert_eq!(
        approved.unmovable_paths().cloned().collect::<Vec<_>>(),
        expected
    );
    let lifted = approved
        .apply(&PermissionProfile::Disabled)
        .expect("not an external sandbox");
    let mut unmovable = lifted.file_system_sandbox_policy().get_unmovable_paths();
    unmovable.sort();
    let mut sorted = expected;
    sorted.sort();
    assert_eq!(unmovable, sorted);
}

/// Issue #651 item 6: which database folders count as well-known shared
/// folders, by spelling or by real location; a folder inside one does not.
#[test]
fn issue_651_well_known_shared_folders() {
    let fx = fixture();
    let temp = fx.user_home.join("temp");
    std::fs::create_dir_all(temp.join("db")).unwrap();
    std::os::unix::fs::symlink(&temp, fx.user_home.join("temp-link")).unwrap();
    let shared = |folder: &Path| is_well_known_shared_folder(folder, Some(&fx.user_home), &temp);
    for folder in [
        "/tmp",
        "/tmp/",
        "/var/tmp",
        "/private/tmp",
        "/private/var/tmp",
    ] {
        assert!(shared(Path::new(folder)), "{folder}");
    }
    assert!(shared(&temp));
    assert!(shared(&fx.user_home.join("temp-link")));
    assert!(shared(&fx.user_home));
    // A relative `TMPDIR`, or `/`, is never a shared folder.
    assert!(!is_well_known_shared_folder(
        Path::new("temp"),
        /*home*/ None,
        Path::new("temp")
    ));
    assert!(!is_well_known_shared_folder(
        Path::new("/"),
        Some(Path::new("/")),
        Path::new("/")
    ));
    for folder in [
        Path::new("/tmp/db"),
        Path::new("/var/tmp/db"),
        &temp.join("db"),
        &fx.user_home.join("db"),
        &fx.codex_home,
    ] {
        assert!(!shared(folder), "{}", folder.display());
    }
}

/// Issue #651 item 6 (Travis, 2026-10-13): on macOS a database folder that is
/// a well-known shared folder (here the temporary folder, and the user's
/// home) is denied one entry at a time, not whole: other files stay readable
/// and writable; the databases (reads also of one created later, and each
/// runtime database's name with its `-wal`, `-shm` and `-journal` files before
/// they exist), the init lock, the GPU controller's lock, pending accounting
/// records and backups stay denied. On Linux the temporary folder stays
/// denied whole (bubblewrap can't deny a name before it exists); the home
/// holds the Corbanu home, so it is denied one entry at a time there too.
/// Another folder is still denied whole.
#[test]
fn issue_651_a_shared_database_folder_is_denied_entry_by_entry() {
    let fx = fixture();
    let temp = fx.user_home.join("temp");
    std::fs::create_dir_all(temp.join("accounting-pending")).unwrap();
    std::fs::write(temp.join("state_5.sqlite"), "x").unwrap();
    std::fs::create_dir_all(fx.user_home.join("db")).unwrap();
    let project = abs(&fx.user_home.join("project"));
    let profile = PermissionProfile::Disabled;
    let denied = |databases: &Path, name: &str| {
        let mut denials = ReadDenials::collect(
            &fx.codex_home,
            Some(&fx.user_home),
            /*claude_config_dir*/ None,
            std::slice::from_ref(&project),
        );
        denials.protect_database_folder(
            &fx.codex_home,
            databases,
            std::slice::from_ref(&project),
            Some(&fx.user_home),
            &temp,
        );
        let policy = denials
            .apply(&profile)
            .unwrap()
            .file_system_sandbox_policy();
        let path = databases.join(name);
        let read = denials.matcher().unwrap().is_read_denied(&path);
        let write = !policy.can_write_path_with_cwd(&path, project.as_path());
        (read, write)
    };
    let macos = cfg!(target_os = "macos");
    for shared in [temp.as_path(), fx.user_home.as_path()] {
        let whole = !macos && shared == temp;
        for (name, expected) in [
            // Only the temporary folder holds one; the pattern covers both.
            ("state_5.sqlite", (true, shared == temp || macos)),
            ("later.sqlite-wal", (true, whole)),
            ("pfterminal_state_5.sqlite-journal", (true, macos || whole)),
            ("pfterminal_logs_2.sqlite-wal", (true, macos || whole)),
            ("state_5.sqlite-shm", (true, macos || whole)),
            ("pfterminal_db_init.lock", (true, true)),
            ("gpu-controller.lock", (true, true)),
            ("accounting-pending/x.json", (true, true)),
            ("db-backups/x", (true, true)),
            ("notes.txt", (whole, whole)),
            ("build/out.o", (whole, whole)),
        ] {
            assert_eq!(
                denied(shared, name),
                expected,
                "{} {name}",
                shared.display()
            );
        }
    }
    // A folder of its own is still denied whole.
    for name in ["notes.txt", "state_5.sqlite", "gpu-controller.lock"] {
        assert_eq!(
            denied(&fx.user_home.join("db"), name),
            (true, true),
            "{name}"
        );
    }
}

/// #766: under Moderate the command may not read the home's Moderate marker
/// (made when missing) and may not write the registry of Moderate homes,
/// which records this home; a person's approval of the command lifts
/// neither. Other runtime files stay readable.
#[test]
fn issue_766_moderate_marker_denied_and_registry_read_only() {
    let fx = fixture();
    // The registry inside the workspace, where the profile could write it.
    let cwd = fx.user_home.join("project");
    let registry = cwd.join("registry");
    let marker = fx
        .codex_home
        .join(codex_security_level::nested::MODERATE_MARKER);
    for approved in [false, true] {
        let mut denials = ReadDenials::collect(
            &fx.codex_home,
            Some(&fx.user_home),
            /*claude_config_dir*/ None,
            &[abs(&cwd)],
        );
        if approved {
            denials = denials.runtime_only();
        }
        denials.mark_moderate_with(&fx.codex_home, Some(&registry));
        let policy = denials
            .apply(&workspace(&cwd))
            .unwrap()
            .file_system_sandbox_policy();
        assert_eq!(
            (
                marker.is_file(),
                policy.can_read_path_with_cwd(&marker, &cwd),
                policy.can_read_path_with_cwd(&fx.codex_home.join("tmp/arg0"), &cwd),
                policy.can_write_path_with_cwd(&registry.join("entry"), &cwd),
                policy.can_write_path_with_cwd(&cwd.join("file"), &cwd),
                codex_security_level::nested::registered_homes(&registry),
            ),
            (true, false, true, false, true, vec![fx.codex_home.clone()]),
            "approved: {approved}"
        );
    }
}

/// #766 round 3: a home whose stored level is Aggressive (or unreadable),
/// or that still holds an Aggressive session's rule file, is not marked,
/// even where Core's level is only Moderate.
#[test]
fn issue_766_no_marker_where_aggressive_is_stored() {
    use codex_security_level::level;
    let marker = codex_security_level::nested::MODERATE_MARKER;
    for (stored, rules) in [
        (Some("version = 1\nlevel = \"aggressive\"\n"), false),
        (Some("not a level"), false),
        (None, true),
    ] {
        let fx = fixture();
        if let Some(contents) = stored {
            std::fs::write(level::state_path(&fx.codex_home), contents).unwrap();
        }
        if rules {
            std::fs::create_dir_all(fx.codex_home.join(level::RULES_DIR)).unwrap();
            std::fs::write(level::rules_path(&fx.codex_home), level::rules_contents()).unwrap();
        }
        let cwd = fx.user_home.join("project");
        let registry = cwd.join("registry");
        let mut denials = ReadDenials::collect(
            &fx.codex_home,
            Some(&fx.user_home),
            /*claude_config_dir*/ None,
            &[abs(&cwd)],
        );
        denials.mark_moderate_with(&fx.codex_home, Some(&registry));
        assert_eq!(
            (
                fx.codex_home.join(marker).exists(),
                registry.exists(),
                !denials.moderate_marker.is_empty()
            ),
            (false, false, false),
            "{stored:?} {rules}"
        );
    }
}

/// #766 round 4: where Aggressive is stored, a marker someone made
/// unreadable is made readable again (it would pass for Moderate's on
/// Linux), and nothing is marked.
#[cfg(target_os = "linux")]
#[test]
fn issue_766_unreadable_marker_is_restored_where_aggressive_is_stored() {
    use codex_security_level::level;
    use std::os::unix::fs::PermissionsExt;
    let fx = fixture();
    std::fs::write(
        level::state_path(&fx.codex_home),
        "version = 1\nlevel = \"aggressive\"\n",
    )
    .unwrap();
    let marker = fx
        .codex_home
        .join(codex_security_level::nested::MODERATE_MARKER);
    std::fs::write(&marker, "").unwrap();
    std::fs::set_permissions(&marker, std::fs::Permissions::from_mode(0o000)).unwrap();
    let cwd = fx.user_home.join("project");
    let mut denials = ReadDenials::collect(
        &fx.codex_home,
        Some(&fx.user_home),
        /*claude_config_dir*/ None,
        &[abs(&cwd)],
    );
    denials.mark_moderate_with(&fx.codex_home, Some(&cwd.join("registry")));
    assert_eq!(
        (
            std::fs::metadata(&marker).unwrap().permissions().mode() & 0o777,
            !denials.moderate_marker.is_empty(),
        ),
        (0o644, false)
    );
}
