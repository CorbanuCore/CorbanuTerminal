use super::everyone_writable_folders;
use super::gather_candidates;
use pretty_assertions::assert_eq;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

/// Grants Everyone modify on `path` and its children (`icacls`, the user's
/// own temp folder only).
pub(crate) fn grant_everyone_modify(path: &Path) {
    let output = std::process::Command::new("icacls")
        .arg(path)
        .args(["/grant", "*S-1-1-0:(OI)(CI)M"])
        .output()
        .expect("run icacls");
    assert!(output.status.success(), "icacls failed: {output:?}");
}

#[test]
fn gathers_path_entries_by_list_separator() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let dir_a = tmp.path().join("Tools");
    let dir_b = tmp.path().join("Bin");
    let dir_space = tmp.path().join("Program Files");
    for dir in [&dir_a, &dir_b, &dir_space] {
        fs::create_dir_all(dir).expect("dir");
    }
    let env = HashMap::from([(
        "PATH".to_string(),
        format!(
            "{};{};{}",
            dir_a.display(),
            dir_b.display(),
            dir_space.display()
        ),
    )]);

    let candidates = gather_candidates(tmp.path(), &env);
    for dir in [&dir_a, &dir_b, &dir_space] {
        let canonical = dunce::canonicalize(dir).expect("canonical");
        assert!(candidates.contains(&canonical), "{}", canonical.display());
    }
}

/// D607: a folder that grants Everyone write access is flagged, from the
/// working folder's children and from `PATH`; a folder that does not, and a
/// link to the flagged one, are not.
#[test]
fn d607_everyone_writable_folders_are_flagged() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let workspace = tmp.path().join("workspace");
    let open = workspace.join("open");
    let closed = workspace.join("closed");
    let tools = tmp.path().join("tools");
    for dir in [&open, &closed, &tools] {
        fs::create_dir_all(dir).expect("dir");
    }
    grant_everyone_modify(&open);
    grant_everyone_modify(&tools);
    let link = workspace.join("link-to-tools");
    let output = std::process::Command::new("cmd")
        .args(["/C", "mklink", "/J"])
        .arg(&link)
        .arg(&tools)
        .output()
        .expect("create junction");
    assert!(output.status.success(), "mklink failed: {output:?}");
    let env = HashMap::from([("PATH".to_string(), tools.display().to_string())]);

    let flagged = everyone_writable_folders(&workspace, &env, /*logs_base_dir*/ None);
    let canonical = |path: &Path| dunce::canonicalize(path).expect("canonical");
    let flagged: Vec<_> = flagged.iter().map(|path| canonical(path)).collect();
    for expected in [&open, &tools] {
        assert!(
            flagged.contains(&canonical(expected)),
            "{} not flagged: {flagged:?}",
            expected.display()
        );
    }
    assert!(!flagged.contains(&canonical(&closed)), "{flagged:?}");
    assert_eq!(
        flagged
            .iter()
            .filter(|path| **path == canonical(&tools))
            .count(),
        1,
        "the link is not followed and the PATH entry is flagged once"
    );
    fs::remove_dir(&link).expect("remove junction");
}
