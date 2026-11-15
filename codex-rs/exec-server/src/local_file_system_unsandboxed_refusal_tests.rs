//! #1976 (D491): a context carrying `unsandboxed_refusal` is refused, with
//! that text, where the host has no platform sandbox for it (Windows without
//! its sandbox), and is served as before everywhere else.

use codex_file_system::WindowsSandboxSelection;
use codex_protocol::models::PermissionProfile;
use pretty_assertions::assert_eq;

use super::*;

#[tokio::test]
async fn issue_1976_a_refusal_applies_only_without_a_platform_sandbox() -> io::Result<()> {
    let temp = tempfile::TempDir::new()?;
    let file = temp.path().join("plain.txt");
    std::fs::write(&file, "plain")?;
    let uri = PathUri::from_abs_path(&AbsolutePathBuf::from_absolute_path(&file)?);
    let fs = LocalFileSystem::unsandboxed();
    // Full access: this context never picks the sandboxed file system.
    let cwd = PathUri::from_abs_path(&AbsolutePathBuf::from_absolute_path(temp.path())?);
    let mut context =
        FileSystemSandboxContext::from_permission_profile(PermissionProfile::Disabled, cwd);
    assert_eq!(
        fs.read_file(&uri, ReadFileOptions::default(), Some(&context))
            .await?,
        b"plain".to_vec()
    );

    context.unsandboxed_refusal = Some("refused for #1976".to_string());
    let read = fs
        .read_file(&uri, ReadFileOptions::default(), Some(&context))
        .await;
    let opened = fs.open_file(&uri, FsOpenMode::Read, Some(&context)).await;
    if cfg!(windows) {
        for error in [read.map(|_| ()), opened.map(|_| ())] {
            let error = error.expect_err("refused without the Windows sandbox");
            assert_eq!(
                (error.kind(), error.to_string()),
                (
                    io::ErrorKind::PermissionDenied,
                    "refused for #1976".to_string()
                )
            );
        }
    } else {
        assert_eq!(read?, b"plain".to_vec());
        assert!(opened.is_ok());
    }

    // D522 (a): MXC counts as no sandbox here.
    context.windows_sandbox_selection = WindowsSandboxSelection::Mxc;
    let read = fs
        .read_file(&uri, ReadFileOptions::default(), Some(&context))
        .await;
    if cfg!(windows) {
        let error = read.expect_err("refused under MXC");
        assert_eq!(error.kind(), io::ErrorKind::PermissionDenied);
        assert_eq!(error.to_string(), "refused for #1976");
    } else {
        assert_eq!(read?, b"plain".to_vec());
    }

    // The non-admin sandbox needs no setup.
    context.windows_sandbox_selection = WindowsSandboxSelection::RestrictedToken;
    assert_eq!(
        fs.read_file(&uri, ReadFileOptions::default(), Some(&context))
            .await?,
        b"plain".to_vec()
    );

    // #2088: merely asking for Elevated is insufficient. Recheck on the
    // receiving executor even when a full-disk profile skips the helper.
    context.windows_sandbox_selection = WindowsSandboxSelection::Elevated;
    let read = fs
        .read_file(&uri, ReadFileOptions::default(), Some(&context))
        .await;
    let opened = fs.open_file(&uri, FsOpenMode::Read, Some(&context)).await;
    if cfg!(windows) && !codex_sandboxing::windows_elevated_sandbox_set_up() {
        for result in [read.map(|_| ()), opened.map(|_| ())] {
            let error = result.expect_err("elevated sandbox not set up");
            assert_eq!(error.kind(), io::ErrorKind::PermissionDenied);
            assert_eq!(error.to_string(), "refused for #1976");
        }
    } else {
        assert_eq!(read?, b"plain".to_vec());
        assert!(opened.is_ok());
    }
    // Unmarked requests (Permissive) retain their old behavior.
    context.unsandboxed_refusal = None;
    assert_eq!(
        fs.read_file(&uri, ReadFileOptions::default(), Some(&context))
            .await?,
        b"plain".to_vec()
    );
    Ok(())
}
