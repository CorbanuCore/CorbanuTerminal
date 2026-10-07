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
    let denials = ReadDenials::collect(&fx.codex_home, Some(&fx.user_home), None, &[abs(&cwd)]);
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
    let denials = ReadDenials::collect(&fx.codex_home, Some(&fx.user_home), None, &[abs(&cwd)]);
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
    let denials = ReadDenials::collect(&fx.codex_home, Some(&fx.user_home), None, &[abs(&cwd)]);
    assert!(denials.skipped.contains(&abs(&cwd)));
    assert!(denials.paths().all(|path| !cwd.starts_with(path.as_path())));
    // The working folder inside the Corbanu home skips only its own entry.
    let cwd = fx.codex_home.join("notes.txt");
    let denials = ReadDenials::collect(&fx.codex_home, Some(&fx.user_home), None, &[abs(&cwd)]);
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
/// view, extension tools) get the same denials once the session is tainted,
/// and nothing changes before that.
#[tokio::test]
async fn pf_23_s01_file_tools_get_the_denials_after_taint() {
    let (session, turn) = crate::session::tests::make_session_and_context().await;
    let client = (*session.services.model_client())
        .clone()
        .with_ingress_level(codex_security_policy::SecurityLevel::Moderate)
        .with_source_envelopes(true);
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
    let context = || turn.file_system_sandbox_context(None, &environment);
    let before = protect_file_tool_context(&session, &turn, context());
    assert_eq!(before, context());

    session
        .services
        .model_client()
        .note_unrecorded_input_for_taint();
    let after = protect_file_tool_context(&session, &turn, context());
    assert!(readable(context()));
    assert!(!readable(after));
}

/// Review 2: host reads of model-named files (Codex Apps uploads) follow the
/// same denials after taint, through symlinks too.
#[tokio::test]
async fn pf_23_s01_upload_reads_follow_the_denials_after_taint() {
    let (session, turn) = crate::session::tests::make_session_and_context().await;
    let client = (*session.services.model_client())
        .clone()
        .with_ingress_level(codex_security_policy::SecurityLevel::Moderate)
        .with_source_envelopes(true);
    session.services.replace_model_client(client);
    assert_eq!(post_taint_read_policy(&session, &turn), None);
    session
        .services
        .model_client()
        .note_unrecorded_input_for_taint();
    let policy = post_taint_read_policy(&session, &turn).expect("tainted");
    let home = turn.config.codex_home.to_path_buf();
    std::fs::create_dir_all(&home).unwrap();
    std::fs::write(home.join("auth.json"), "x").unwrap();
    let outside = tempfile::tempdir().unwrap();
    let link = outside.path().join("innocent.txt");
    std::os::unix::fs::symlink(home.join("auth.json"), &link).unwrap();
    let plain = outside.path().join("plain.txt");
    std::fs::write(&plain, "x").unwrap();
    #[allow(deprecated)]
    let cwd = turn.cwd;
    assert!(!readable_under(
        &policy,
        &home.join("auth.json"),
        cwd.as_path()
    ));
    assert!(!readable_under(&policy, &link, cwd.as_path()));
    assert!(readable_under(&policy, &plain, cwd.as_path()));
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
    let denials = ReadDenials::collect(&fx.codex_home, Some(&fx.user_home), None, &[abs(&cwd)]);
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
    let denials = ReadDenials::collect(&fx.codex_home, Some(&fx.user_home), None, &[abs(&cwd)]);
    let policy = denials.apply(&base).unwrap().file_system_sandbox_policy();
    for unreadable in [fx.user_home.join(".zshrc"), fx.codex_home.join("skills")] {
        assert!(!policy.can_read_path_with_cwd(&unreadable, &cwd));
    }
    assert!(!policy.can_read_path_with_cwd(&secret, &cwd));
    assert!(policy.can_write_path_with_cwd(&cwd.join("a.txt"), &cwd));
    assert!(!policy.can_write_path_with_cwd(&cwd.join(".codex/config.toml"), &cwd));
}

/// PF-23-S02: under Aggressive the rules apply from the start of the
/// session, before any untrusted content; under Moderate they wait for it.
#[tokio::test]
async fn pf_23_s02_aggressive_applies_the_rules_from_the_start() {
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
    for (level, protected) in [
        (codex_security_policy::SecurityLevel::Moderate, false),
        (codex_security_policy::SecurityLevel::Aggressive, true),
    ] {
        let client = (*session.services.model_client())
            .clone()
            .with_ingress_level(level)
            .with_source_envelopes(true);
        session.services.replace_model_client(client);
        let context = turn.file_system_sandbox_context(None, &environment);
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

    let denials = ReadDenials::collect(&fx.codex_home, Some(&fx.user_home), None, &[abs(&cwd)]);
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
    let denials = ReadDenials::collect(&fx.codex_home, Some(&fx.user_home), None, &[abs(&cwd)]);
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

    let denials = ReadDenials::collect(&fx.codex_home, Some(&fx.user_home), None, &[abs(&cwd)]);
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
