use super::*;
use pretty_assertions::assert_eq;

fn strings(values: &[&str]) -> Vec<String> {
    values.iter().map(|value| (*value).to_string()).collect()
}

#[test]
fn detects_direct_and_script_wrapped_worktree_creation() {
    let commands = [
        strings(&["git", "worktree", "add", "/tmp/proof", "HEAD"]),
        strings(&[
            "git",
            "-C",
            "/repo",
            "worktree",
            "add",
            "/tmp/proof",
            "HEAD",
        ]),
        strings(&[
            "/bin/bash",
            "-lc",
            "set -euo pipefail\nprintf 'command=git worktree add --detach /tmp/proof HEAD\\n'\ngit worktree add --detach \"$WT\" HEAD",
        ]),
        strings(&[
            "/bin/bash",
            "-lc",
            "printf '%s' 'git worktree add is documentation'; set +e; git worktree add /tmp/proof HEAD",
        ]),
    ];

    assert_eq!(
        commands.map(|command| contains_git_worktree_add(&command)),
        [true, true, true, true]
    );
}

#[test]
fn ignores_diagnostics_and_unrelated_git_commands() {
    let commands = [
        strings(&["git", "worktree", "list"]),
        strings(&["git", "status", "--short"]),
        strings(&[
            "/bin/bash",
            "-lc",
            "printf 'git worktree add /tmp/proof HEAD\\n'\ngit worktree list",
        ]),
    ];

    assert_eq!(
        commands.map(|command| contains_git_worktree_add(&command)),
        [false, false, false]
    );
}

#[test]
fn hierarchy_scope_is_explicit() {
    assert_eq!(
        [
            is_hierarchy_role(Some("nazgul")),
            is_hierarchy_role(Some("troll")),
            is_hierarchy_role(Some("orc")),
            is_hierarchy_role(Some("worker")),
            is_hierarchy_role(None),
        ],
        [true, true, true, false, false]
    );
}

#[test]
fn working_tree_measurement_excludes_git_metadata() -> anyhow::Result<()> {
    let root = tempfile::tempdir()?;
    std::fs::write(root.path().join("tracked.bin"), vec![0_u8; 17])?;
    std::fs::create_dir(root.path().join("nested"))?;
    std::fs::write(root.path().join("nested/artifact.bin"), vec![0_u8; 23])?;
    std::fs::create_dir(root.path().join(".git"))?;
    std::fs::write(root.path().join(".git/object"), vec![0_u8; 101])?;

    assert_eq!(measure_working_tree_bytes(root.path())?, 40);
    Ok(())
}
