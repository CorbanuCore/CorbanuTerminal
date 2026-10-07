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

    for n in 0..=MAX_MODULE_DIRS {
        std::fs::create_dir_all(git.join(format!("modules/fake/f{n}")).as_path()).unwrap();
    }
    assert_eq!(module_paths(&git), vec![git.join("modules")]);
}
