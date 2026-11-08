use super::*;
use pretty_assertions::assert_eq;

fn junction(link: &Path, target: &Path) {
    let status = std::process::Command::new("cmd")
        .args(["/d", "/c", "mklink", "/J"])
        .arg(link)
        .arg(target)
        .stdout(std::process::Stdio::null())
        .status()
        .expect("mklink");
    assert!(status.success(), "mklink /J failed");
}

/// Finds nested entries of either name and any case, not the top-level
/// ones, nothing inside `.git` or behind a junction; a junction named
/// `.codex` is found but never matches.
#[test]
fn find_reports_nested_entries_only() {
    let dir = tempfile::tempdir().expect("tempdir");
    let root = dunce::canonicalize(dir.path()).expect("canonical");
    let outside = tempfile::tempdir().expect("outside");
    for path in [
        ".codex\\inner\\.codex",
        "a\\.codex",
        "a\\b\\.AGENTS",
        ".git\\x\\.codex",
        "target\\.codex",
        "j",
    ] {
        std::fs::create_dir_all(root.join(path)).expect("create");
    }
    std::fs::create_dir_all(outside.path().join("c\\.codex")).expect("create outside");
    junction(&root.join("link"), outside.path());
    junction(&root.join("j\\.codex"), &root.join("target"));

    let found = find(std::slice::from_ref(&root));
    assert!(found.unlistable.is_empty(), "{:?}", found.unlistable);
    let found = found.entries;
    let mut paths: Vec<(PathBuf, bool)> = found
        .iter()
        .map(|entry| (entry.path.clone(), entry.identity.is_some()))
        .collect();
    paths.sort();
    let mut expected = vec![
        (root.join("a\\.codex"), true),
        // Under the name Core opens.
        (root.join("a\\b\\.agents"), true),
        (root.join("j\\.codex"), false),
        (root.join("target\\.codex"), true),
    ];
    expected.sort();
    assert_eq!(paths, expected);
    assert_eq!(to_protect(&found).len(), 3, "the junction is not protected");
}

/// The same file in the same place matches; a new one, a replaced one and
/// one without an identity don't.
#[test]
fn appeared_compares_path_and_identity() {
    let dir = tempfile::tempdir().expect("tempdir");
    let root = dunce::canonicalize(dir.path()).expect("canonical");
    for path in ["kept\\.codex", "replaced\\.codex"] {
        std::fs::create_dir_all(root.join(path)).expect("create");
    }
    let before = find(std::slice::from_ref(&root)).entries;
    std::fs::remove_dir(root.join("replaced\\.codex")).expect("remove");
    std::fs::create_dir(root.join("replaced\\.codex")).expect("recreate");
    std::fs::create_dir_all(root.join("new\\.agents")).expect("create new");

    let mut appeared = appeared(&before, find(std::slice::from_ref(&root)).entries);
    appeared.sort();
    assert_eq!(
        appeared,
        vec![root.join("new\\.agents"), root.join("replaced\\.codex")]
    );
    let roots = [root.clone()];
    assert_eq!(
        (
            appeared_in_session(&roots, &before, &root.join("KEPT\\.codex")),
            appeared_in_session(&roots, &before, &root.join("replaced\\.codex")),
            appeared_in_session(&roots, &before, &root.join(".codex")),
        ),
        (false, true, false)
    );
}

/// A name NTFS may fold to `.agents` (`ſ` upper-cases to `S`) is found
/// under the name Core opens, and never matches; a folder the walk can't
/// list (here a root that is a file) is reported.
#[test]
fn find_reports_folded_names_and_unlistable_folders() {
    let dir = tempfile::tempdir().expect("tempdir");
    let root = dunce::canonicalize(dir.path()).expect("canonical");
    std::fs::create_dir_all(root.join("a\\.agent\u{17f}")).expect("create folded name");
    // Walked into: only `.git` itself is skipped.
    std::fs::create_dir_all(root.join(".g\u{131}t\\b\\.codex")).expect("create .gıt");
    let not_a_folder = dir.path().join("file");
    std::fs::write(&not_a_folder, "x").expect("write file");
    let mut found = find(&[root.clone(), not_a_folder.clone()]);
    found.entries.sort_by(|a, b| a.path.cmp(&b.path));

    assert_eq!(
        (
            found
                .entries
                .iter()
                .map(|entry| (entry.path.clone(), entry.identity.is_some()))
                .collect::<Vec<_>>(),
            found.unlistable,
            folds_to(".AGENT\u{17f}", ".agents"),
            folds_to(".agentss", ".agents"),
        ),
        (
            vec![
                (root.join(".g\u{131}t\\b\\.codex"), true),
                (root.join("a\\.agents"), false),
            ],
            vec![not_a_folder],
            true,
            false,
        )
    );
}

/// D332: a root's top-level `.git` is reported when it exists, with its
/// identity (D419: a junction too, with what it points to); a missing one
/// isn't.
/// One counts as appeared in a session unless it is there, as the same
/// file, from the start; a `.git` below the top of a root isn't D332's.
#[test]
fn top_level_git_is_reported_and_compared() {
    let dir = tempfile::tempdir().expect("tempdir");
    let base = dunce::canonicalize(dir.path()).expect("canonical");
    let (none, folder, file, linked, target) = (
        base.join("none"),
        base.join("folder"),
        base.join("file"),
        base.join("linked"),
        base.join("target"),
    );
    for path in [&none, &folder, &file, &linked, &target] {
        std::fs::create_dir_all(path).expect("create");
    }
    std::fs::create_dir(folder.join(".git")).expect(".git folder");
    std::fs::write(file.join(".git"), b"gitdir: elsewhere").expect(".git file");
    junction(&linked.join(".git"), &target);
    let roots = vec![none.clone(), folder.clone(), file.clone(), linked.clone()];

    let before = top_level_git(&roots);
    let reported: Vec<(PathBuf, bool)> = before
        .iter()
        .map(|entry| (entry.path.clone(), entry.identity.is_some()))
        .collect();
    let appeared = |path: &Path| git_appeared_in_session(&roots, &before, path);
    let unchanged = (
        appeared(&none.join(".git")),
        appeared(&folder.join(".git")),
        appeared(&file.join(".git")),
        appeared(&linked.join(".git")),
        appeared(&folder.join("sub\\.git")),
        appeared(&folder.join(".codex")),
    );
    std::fs::create_dir(none.join(".git")).expect("plant");
    std::fs::remove_dir(folder.join(".git")).expect("remove");
    std::fs::create_dir(folder.join(".git")).expect("replace");
    let changed = (
        appeared(&none.join(".git")),
        appeared(&folder.join(".GIT")),
        crate::nested_metadata::appeared(&before, top_level_git(&roots)),
    );

    assert_eq!(
        (reported, unchanged, changed),
        (
            vec![
                (folder.join(".git"), true),
                (file.join(".git"), true),
                (linked.join(".git"), true),
            ],
            // D419 (rule 3): an unchanged junction no longer counts.
            (true, false, false, false, false, false),
            (true, true, vec![none.join(".git"), folder.join(".git")]),
        )
    );
}

/// D419 (rule 3, #1775): a `.git` junction counts as changed when it is
/// remade (same target), pointed elsewhere, or its target folder is
/// replaced, and not otherwise.
#[test]
fn top_level_git_link_counts_only_when_it_or_its_target_changes() {
    let dir = tempfile::tempdir().expect("tempdir");
    let base = dunce::canonicalize(dir.path()).expect("canonical");
    let roots: Vec<PathBuf> = ["remade", "retargeted", "target-replaced", "kept"]
        .iter()
        .map(|name| base.join(name))
        .collect();
    let (a, b) = (base.join("a"), base.join("b"));
    for path in roots.iter().chain([&a, &b]) {
        std::fs::create_dir_all(path).expect("create");
    }
    let targets = [a.clone(), a.clone(), base.join("t"), a.clone()];
    std::fs::create_dir_all(&targets[2]).expect("target");
    for (root, target) in roots.iter().zip(&targets) {
        junction(&root.join(".git"), target);
    }
    let before = top_level_git(&roots);

    // Remade with the same target: a new link, so a new identity.
    std::fs::remove_dir(roots[0].join(".git")).expect("remove");
    junction(&roots[0].join(".git"), &a);
    // Pointed elsewhere.
    std::fs::remove_dir(roots[1].join(".git")).expect("remove");
    junction(&roots[1].join(".git"), &b);
    // The target folder replaced by another at the same path.
    std::fs::rename(&targets[2], base.join("t-old")).expect("move target");
    std::fs::create_dir(&targets[2]).expect("new target");

    assert_eq!(
        crate::nested_metadata::appeared(&before, top_level_git(&roots)),
        vec![
            roots[0].join(".git"),
            roots[1].join(".git"),
            roots[2].join(".git")
        ]
    );
}
