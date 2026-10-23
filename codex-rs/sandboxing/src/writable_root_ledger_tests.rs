use super::WritableRootRefusal;
use super::check_writable_roots;
use super::pinned_real_location;
use super::record_writable_roots;
use codex_protocol::permissions::FileSystemAccessMode;
use codex_protocol::permissions::FileSystemPath;
use codex_protocol::permissions::FileSystemSandboxEntry;
use codex_protocol::permissions::FileSystemSandboxPolicy;
use codex_utils_absolute_path::AbsolutePathBuf;
use pretty_assertions::assert_eq;
use std::path::Path;
use std::path::PathBuf;

fn write(path: &Path) -> FileSystemSandboxEntry {
    FileSystemSandboxEntry::new(
        FileSystemPath::Path {
            path: AbsolutePathBuf::from_absolute_path(path).expect("absolute"),
        },
        FileSystemAccessMode::Write,
    )
}

fn base() -> (tempfile::TempDir, PathBuf) {
    let dir = tempfile::tempdir().expect("tempdir");
    let base = dir.path().canonicalize().expect("canonical");
    (dir, base)
}

/// #536: a configured root that does not exist yet, created as a link
/// during the session, is refused from then on, wherever the link leads.
#[test]
fn a_missing_root_created_as_a_link_is_refused() {
    let (_dir, base) = base();
    let workspace = base.join("ws");
    let elsewhere = base.join("elsewhere");
    std::fs::create_dir_all(&workspace).expect("ws");
    std::fs::create_dir_all(&elsewhere).expect("elsewhere");
    let target = workspace.join("target");
    let inside = workspace.join("inside");
    let policy = FileSystemSandboxPolicy::restricted(vec![
        write(&workspace),
        write(&target),
        write(&inside),
    ]);
    record_writable_roots(&policy, &workspace);
    assert!(check_writable_roots(&policy, &workspace).is_ok());

    std::os::unix::fs::symlink(&elsewhere, &target).expect("link");
    assert_eq!(
        check_writable_roots(&policy, &workspace),
        Err(WritableRootRefusal::Changed {
            root: target.clone(),
            link: target.clone(),
            was: target.clone(),
            now: elsewhere,
        })
    );
    // Even a link that stays inside the workspace was made after the start.
    std::fs::remove_file(&target).expect("unlink");
    std::fs::create_dir(workspace.join("real")).expect("real");
    std::os::unix::fs::symlink(workspace.join("real"), &inside).expect("inner link");
    assert!(matches!(
        check_writable_roots(&policy, &workspace),
        Err(WritableRootRefusal::Changed { root, .. }) if root == inside
    ));
    // Removing the link restores the session's roots.
    std::fs::remove_file(&inside).expect("unlink");
    assert!(check_writable_roots(&policy, &workspace).is_ok());
}

/// #536: a held root that is a link leading outside every root holding it is
/// refused even when it existed before the session; one created as a real
/// folder, or one no root holds, stays writable.
#[test]
fn a_held_link_leading_outside_its_holders_is_refused() {
    let (_dir, base) = base();
    let workspace = base.join("ws");
    let elsewhere = base.join("elsewhere");
    std::fs::create_dir_all(&workspace).expect("ws");
    std::fs::create_dir_all(&elsewhere).expect("elsewhere");
    let cache = workspace.join("cache");
    std::os::unix::fs::symlink(&elsewhere, &cache).expect("link");
    let free = base.join("free-link");
    std::os::unix::fs::symlink(&elsewhere, &free).expect("free link");
    let build = workspace.join("build");

    let held = FileSystemSandboxPolicy::restricted(vec![write(&workspace), write(&cache)]);
    record_writable_roots(&held, &workspace);
    assert_eq!(
        check_writable_roots(&held, &workspace),
        Err(WritableRootRefusal::OutsideHolder {
            root: cache.clone(),
            link: cache,
            real: elsewhere.clone(),
            holder: workspace.clone(),
        })
    );

    let fine =
        FileSystemSandboxPolicy::restricted(vec![write(&workspace), write(&free), write(&build)]);
    record_writable_roots(&fine, &workspace);
    std::fs::create_dir(&build).expect("build");
    let pinned = check_writable_roots(&fine, &workspace).expect("ordinary roots");
    let reals: Vec<PathBuf> = pinned.into_iter().map(|pinned| pinned.real).collect();
    assert_eq!(reals, vec![workspace, elsewhere.clone(), build]);
    assert_eq!(pinned_real_location(&free), Some(elsewhere));
}

/// #536: a workspace spelled through a link (as `/tmp` on macOS) and a git
/// worktree with its main checkout pass, pinned at their real locations.
#[test]
fn aliases_and_worktrees_pass() {
    let (_dir, base) = base();
    let real = base.join("real");
    let main = real.join("main");
    let worktree = real.join("wt");
    std::fs::create_dir_all(main.join(".git/worktrees/wt")).expect("gitdir");
    std::fs::create_dir_all(&worktree).expect("worktree");
    std::fs::write(
        worktree.join(".git"),
        format!("gitdir: {}\n", main.join(".git/worktrees/wt").display()),
    )
    .expect(".git file");
    let alias = base.join("alias");
    std::os::unix::fs::symlink(&real, &alias).expect("alias");
    let spelled = alias.join("wt");
    let policy = FileSystemSandboxPolicy::restricted(vec![
        write(&spelled),
        write(&alias.join("main")),
        write(&base),
    ]);
    record_writable_roots(&policy, &spelled);
    let pinned = check_writable_roots(&policy, &spelled).expect("aliases pass");
    let mut reals: Vec<PathBuf> = pinned.into_iter().map(|pinned| pinned.real).collect();
    reals.sort();
    assert_eq!(reals, vec![base, main, worktree]);
}

/// #536: a link found at the start that leads outside its holder is refused;
/// once the person removes it, commands run again, and a link made again is
/// refused as made during the session.
#[test]
fn removing_a_refused_link_lets_commands_run_again() {
    let (_dir, base) = base();
    let workspace = base.join("ws");
    let elsewhere = base.join("elsewhere");
    std::fs::create_dir_all(&workspace).expect("ws");
    std::fs::create_dir_all(&elsewhere).expect("elsewhere");
    let target = workspace.join("target");
    std::os::unix::fs::symlink(&elsewhere, &target).expect("link");
    let policy = FileSystemSandboxPolicy::restricted(vec![write(&workspace), write(&target)]);
    record_writable_roots(&policy, &workspace);
    assert!(matches!(
        check_writable_roots(&policy, &workspace),
        Err(WritableRootRefusal::OutsideHolder { .. })
    ));
    std::fs::remove_file(&target).expect("unlink");
    assert!(check_writable_roots(&policy, &workspace).is_ok());
    std::fs::create_dir(&target).expect("real folder");
    assert!(check_writable_roots(&policy, &workspace).is_ok());
    std::fs::remove_dir(&target).expect("rmdir");
    std::os::unix::fs::symlink(workspace.join(".."), &target).expect("link again");
    assert!(matches!(
        check_writable_roots(&policy, &workspace),
        Err(WritableRootRefusal::Changed { .. })
    ));
}

/// #536: the refusal names the link on the path (here a folder above the
/// root), quoted for the shell.
#[test]
fn the_refusal_names_the_link_on_the_path() {
    let (_dir, base) = base();
    let workspace = base.join("ws");
    let elsewhere = base.join("elsewhere");
    std::fs::create_dir_all(&workspace).expect("ws");
    std::fs::create_dir_all(elsewhere.join("deep")).expect("elsewhere");
    let parent = workspace.join("a dir");
    let root = parent.join("deep");
    let policy = FileSystemSandboxPolicy::restricted(vec![write(&workspace), write(&root)]);
    record_writable_roots(&policy, &workspace);
    std::os::unix::fs::symlink(&elsewhere, &parent).expect("link above the root");
    let refusal = check_writable_roots(&policy, &workspace).expect_err("refused");
    assert_eq!(
        refusal,
        WritableRootRefusal::Changed {
            root: root.clone(),
            link: parent.clone(),
            was: root,
            now: elsewhere.join("deep"),
        }
    );
    assert!(
        refusal
            .to_string()
            .contains(&format!("`!rm '{}'`", parent.display())),
        "{refusal}"
    );
}

/// #536 review: with the workspace itself reached through a link (a linked
/// code folder), the refusal names the link made in the workspace, not the
/// person's folder link above it.
#[test]
fn the_refusal_names_the_link_that_changed() {
    let (_dir, base) = base();
    let real_code = base.join("mnt-code");
    std::fs::create_dir_all(real_code.join("ws")).expect("ws");
    let elsewhere = base.join("elsewhere");
    std::fs::create_dir_all(&elsewhere).expect("elsewhere");
    let code = base.join("code");
    std::os::unix::fs::symlink(&real_code, &code).expect("code link");
    let workspace = code.join("ws");
    let target = workspace.join("target");
    let policy = FileSystemSandboxPolicy::restricted(vec![write(&workspace), write(&target)]);
    record_writable_roots(&policy, &workspace);
    std::os::unix::fs::symlink(&elsewhere, &target).expect("new link");
    let refusal = check_writable_roots(&policy, &workspace).expect_err("refused");
    let WritableRootRefusal::Changed { link, .. } = &refusal else {
        panic!("{refusal:?}");
    };
    assert_eq!(link, &target);
    assert!(
        refusal
            .to_string()
            .contains(&format!("`!rm '{}'`", target.display())),
        "{refusal}"
    );
}

/// #536 review: a root that was already a link to a folder inside the
/// workspace (reached through a linked code folder) and is pointed elsewhere
/// names that link, the only one a command could have made.
#[test]
fn the_refusal_names_a_repointed_link() {
    let (_dir, base) = base();
    let real_code = base.join("mnt-code");
    std::fs::create_dir_all(real_code.join("ws/build/out")).expect("ws");
    let elsewhere = base.join("elsewhere");
    std::fs::create_dir_all(&elsewhere).expect("elsewhere");
    let code = base.join("code");
    std::os::unix::fs::symlink(&real_code, &code).expect("code link");
    let workspace = code.join("ws");
    let target = workspace.join("target");
    std::os::unix::fs::symlink(workspace.join("build/out"), &target).expect("inner link");
    let policy = FileSystemSandboxPolicy::restricted(vec![write(&workspace), write(&target)]);
    record_writable_roots(&policy, &workspace);
    assert!(check_writable_roots(&policy, &workspace).is_ok());
    std::fs::remove_file(&target).expect("unlink");
    std::os::unix::fs::symlink(&elsewhere, &target).expect("repointed");
    let refusal = check_writable_roots(&policy, &workspace).expect_err("refused");
    let WritableRootRefusal::Changed { link, .. } = &refusal else {
        panic!("{refusal:?}");
    };
    assert_eq!(link, &target);
}
