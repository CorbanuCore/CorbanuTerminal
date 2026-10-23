use super::excluded_writable_link_warnings;
use codex_protocol::ThreadId;
use codex_protocol::permissions::FileSystemAccessMode;
use codex_protocol::permissions::FileSystemPath;
use codex_protocol::permissions::FileSystemSandboxEntry;
use codex_protocol::permissions::FileSystemSandboxPolicy;
use codex_utils_absolute_path::AbsolutePathBuf;
use pretty_assertions::assert_eq;

/// #535: a writable link into another root's read-only path is explained
/// once per thread, naming the link and the path that stays read-only.
#[test]
fn excluded_writable_links_are_explained_once_per_thread() {
    let dir = tempfile::tempdir().expect("tempdir");
    let base = dir.path().canonicalize().expect("canonical");
    let workspace = base.join("ws");
    let hooks = workspace.join(".git").join("hooks");
    std::fs::create_dir_all(&hooks).expect("hooks");
    let link = base.join("hooks-link");
    std::os::unix::fs::symlink(&hooks, &link).expect("link");
    let write = |path: &std::path::Path| {
        FileSystemSandboxEntry::new(
            FileSystemPath::Path {
                path: AbsolutePathBuf::from_absolute_path(path).expect("absolute"),
            },
            FileSystemAccessMode::Write,
        )
    };
    let policy = FileSystemSandboxPolicy::restricted(vec![write(&workspace), write(&link)]);
    let thread = ThreadId::new();
    let expected = vec![format!(
        "Write access to `{}` is not granted: it is, or passes through, a link into `{}`, a \
         path another writable folder protects. Name the folder by its real path if it should \
         be writable.",
        link.display(),
        workspace.join(".git").display(),
    )];
    assert_eq!(
        excluded_writable_link_warnings(thread, &policy, &workspace),
        expected
    );
    assert_eq!(
        excluded_writable_link_warnings(thread, &policy, &workspace),
        Vec::<String>::new()
    );
    assert_eq!(
        excluded_writable_link_warnings(ThreadId::new(), &policy, &workspace),
        expected
    );
    let plain = FileSystemSandboxPolicy::restricted(vec![write(&workspace)]);
    assert_eq!(
        excluded_writable_link_warnings(ThreadId::new(), &plain, &workspace),
        Vec::<String>::new()
    );
}
