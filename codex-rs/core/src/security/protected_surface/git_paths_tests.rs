#![cfg(unix)]

use super::*;
use pretty_assertions::assert_eq;

fn abs(path: &Path) -> AbsolutePathBuf {
    AbsolutePathBuf::from_absolute_path(path).unwrap()
}

#[test]
fn pf_23_s02_hooks_path_resolves_like_git() {
    let dir = tempfile::tempdir().unwrap();
    let base = abs(&dir.path().canonicalize().unwrap());
    let home = base.join("home");
    let config = base.join("config");
    let read = |text: &str| {
        std::fs::write(config.as_path(), text).unwrap();
        hooks_path(&config, &base, Some(&home))
    };
    assert_eq!(
        read("[core]\n  hookspath = tools/hooks\n"),
        Some(base.join("tools/hooks"))
    );
    assert_eq!(
        read("[core]\nhooksPath = /opt/hooks\n"),
        Some(abs(Path::new("/opt/hooks")))
    );
    assert_eq!(
        read("[core] hooksPath = \".husky\" # managed by husky\n"),
        Some(base.join(".husky"))
    );
    assert_eq!(
        read("[core] # comment\n\thooksPath = a ; old\n[CORE]\nhooksPath = b\n"),
        Some(base.join("b"))
    );
    assert_eq!(
        read("[core]\nhooksPath = ~/.githooks\n"),
        Some(home.join(".githooks"))
    );
    assert_eq!(read("[user]\nhooksPath = x\n"), None);
    assert_eq!(read("[core \"x\"]\nhooksPath = x\n"), None);
    assert_eq!(read("[core]\nhooksPath = ~other/hooks\n"), None);
}

/// Every folder below `.git/modules` has its entries protected, without
/// following links; past the limits the whole `modules` folder is.
#[test]
fn pf_23_s02_module_walk_is_bounded_and_does_not_follow_links() {
    let dir = tempfile::tempdir().unwrap();
    let git = abs(&dir.path().canonicalize().unwrap()).join(".git");
    for module in ["modules/a", "modules/libs/b"] {
        for entry in ["objects", "hooks"] {
            std::fs::create_dir_all(git.join(module).join(entry)).unwrap();
        }
        std::fs::write(git.join(module).join("config"), "").unwrap();
    }
    // A link is never followed (`/` would walk the whole disk).
    std::os::unix::fs::symlink("/", git.join("modules/root-link").as_path()).unwrap();
    // No HEAD anywhere: entries are protected whether or not one exists.
    let paths = module_paths(&git);
    for protected in [
        "modules/a/hooks",
        "modules/a/config",
        "modules/libs/b/hooks",
    ] {
        assert!(
            paths.contains(&git.join(protected)),
            "{protected}: {paths:?}"
        );
    }
    assert!(
        paths
            .iter()
            .all(|path| !path.as_path().starts_with(git.join("modules/root-link"))),
        "{paths:?}"
    );
    assert!(
        paths
            .iter()
            .all(|path| !path.as_path().starts_with(git.join("modules/a/objects"))),
        "{paths:?}"
    );

    // A fake `objects` in an intermediate folder hides nothing below it.
    std::fs::create_dir_all(git.join("modules/libs/objects").as_path()).unwrap();
    assert!(module_paths(&git).contains(&git.join("modules/libs/b/hooks")));
    // A submodule named like a git folder is still walked.
    std::fs::create_dir_all(git.join("modules/hooks/hooks").as_path()).unwrap();
    assert!(module_paths(&git).contains(&git.join("modules/hooks/hooks")));

    // A folder that cannot be read closes the whole of `modules`.
    {
        use std::os::unix::fs::PermissionsExt;
        let libs = git.join("modules/libs");
        std::fs::set_permissions(libs.as_path(), std::fs::Permissions::from_mode(0o000)).unwrap();
        // Root reads it anyway; there is nothing to check then.
        let unreadable = std::fs::read_dir(libs.as_path()).is_err();
        let paths = module_paths(&git);
        std::fs::set_permissions(libs.as_path(), std::fs::Permissions::from_mode(0o755)).unwrap();
        if unreadable {
            assert_eq!(paths, vec![git.join("modules")]);
        }
    }

    for n in 0..=MAX_MODULE_DIRS {
        std::fs::create_dir_all(git.join(format!("modules/fake/f{n}")).as_path()).unwrap();
    }
    assert_eq!(module_paths(&git), vec![git.join("modules")]);
}

/// An unreadable workspace folder or `.git` hides nothing: the `.git` becomes
/// read-only whole, and a `hooksPath` read before still holds.
#[test]
fn pf_23_s02_unreadable_repository_fails_closed() {
    use std::os::unix::fs::PermissionsExt;
    let dir = tempfile::tempdir().unwrap();
    let base = abs(&dir.path().canonicalize().unwrap());
    let repo = base.join("repo");
    let git = repo.join(".git");
    std::fs::create_dir_all(git.join("hooks").as_path()).unwrap();
    std::fs::write(
        git.join("config").as_path(),
        "[core]\n\thooksPath = .husky\n",
    )
    .unwrap();
    std::fs::create_dir_all(repo.join(".husky").as_path()).unwrap();
    let cwd = repo.join("src");
    std::fs::create_dir_all(cwd.as_path()).unwrap();
    assert!(git_persistence_paths(&cwd, None).contains(&repo.join(".husky")));

    let lock = |path: &AbsolutePathBuf, mode| {
        std::fs::set_permissions(path.as_path(), std::fs::Permissions::from_mode(mode)).unwrap()
    };
    lock(&git, 0o000);
    let unreadable = std::fs::read_dir(git.as_path()).is_err();
    let paths = git_persistence_paths(&cwd, None);
    lock(&git, 0o755);
    if unreadable {
        assert!(paths.contains(&repo.join(".husky")), "{paths:?}");
    }

    lock(&repo, 0o000);
    let hidden = std::fs::symlink_metadata(git.as_path()).is_err();
    let paths = git_persistence_paths(&cwd, None);
    lock(&repo, 0o755);
    if hidden {
        assert!(paths.contains(&git), "{paths:?}");
        assert!(paths.contains(&base.join(".git")), "{paths:?}");
    }
}
