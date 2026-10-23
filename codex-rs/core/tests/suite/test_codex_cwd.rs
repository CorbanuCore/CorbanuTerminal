//! sec-745 follow-up: the config `TestCodex` builds is not built for the
//! test process's own working folder (the crate's source tree, e.g.
//! `codex-rs/core`). The folder a config is built for is its workspace root,
//! so sandboxed commands in tests could write into the checkout. #1120: a
//! local session's own folder is its workspace root by default, as in a real
//! session.

use codex_utils_absolute_path::AbsolutePathBuf;
use core_test_support::load_default_config_for_test_in;
use core_test_support::responses::start_mock_server;
use core_test_support::skip_if_no_network;
use core_test_support::skip_if_remote;
use core_test_support::test_codex::test_codex;
use tempfile::TempDir;

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_codex_config_does_not_make_the_source_tree_a_workspace_root() -> anyhow::Result<()> {
    skip_if_no_network!(Ok(()));

    let server = start_mock_server().await;
    let test = test_codex().build(&server).await?;
    let source_tree = AbsolutePathBuf::current_dir()?;
    let roots = test.config.permissions.workspace_roots();
    assert!(
        !roots
            .iter()
            .any(|root| source_tree.as_path().starts_with(root.as_path())),
        "{source_tree:?} is under a workspace root: {roots:?}"
    );
    Ok(())
}

fn is_workspace_root(roots: &[AbsolutePathBuf], folder: &std::path::Path) -> bool {
    let folder = std::fs::canonicalize(folder).expect("canonical folder");
    roots
        .iter()
        .any(|root| std::fs::canonicalize(root.as_path()).ok().as_ref() == Some(&folder))
}

/// #1120: by default a local session's own folder is its workspace root, as
/// in a real session.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_codex_session_folder_is_its_workspace_root_by_default() -> anyhow::Result<()> {
    skip_if_no_network!(Ok(()));
    skip_if_remote!(Ok(()), "a remote session's folder is not on this machine");

    let server = start_mock_server().await;
    let test = test_codex().build(&server).await?;
    let roots = test.config.permissions.workspace_roots();
    assert!(
        is_workspace_root(roots, test.cwd_path()),
        "{:?} is not a workspace root: {roots:?}",
        test.cwd_path()
    );
    Ok(())
}

/// A test that builds its config for another folder keeps its session
/// outside the workspace roots.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_codex_config_folder_keeps_the_session_outside_its_workspace_roots()
-> anyhow::Result<()> {
    skip_if_no_network!(Ok(()));
    skip_if_remote!(Ok(()), "a remote session's folder is not on this machine");

    let server = start_mock_server().await;
    let config_folder = TempDir::new()?;
    let test = test_codex()
        .with_config_folder(config_folder.path())
        .build(&server)
        .await?;
    let roots = test.config.permissions.workspace_roots();
    assert!(is_workspace_root(roots, config_folder.path()), "{roots:?}");
    assert!(
        !is_workspace_root(roots, test.cwd_path()),
        "{:?} is a workspace root: {roots:?}",
        test.cwd_path()
    );
    Ok(())
}

/// #1120: a config built directly by `load_default_config_for_test_in` is
/// built for the given folder, never the source tree.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn direct_test_config_is_built_for_the_given_folder() {
    let home = TempDir::new().expect("home");
    let cwd = TempDir::new().expect("cwd");
    let config = load_default_config_for_test_in(&home, Default::default(), cwd.path()).await;
    let roots = config.permissions.workspace_roots();
    assert!(is_workspace_root(roots, cwd.path()), "{roots:?}");
    let source_tree = AbsolutePathBuf::current_dir().expect("current dir");
    assert!(
        !roots
            .iter()
            .any(|root| source_tree.as_path().starts_with(root.as_path())),
        "{source_tree:?} is under a workspace root: {roots:?}"
    );
}
