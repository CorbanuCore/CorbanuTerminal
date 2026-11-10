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

    // With a Windows sandbox selected the refusal does not apply here.
    context.windows_sandbox_selection = WindowsSandboxSelection::Elevated;
    assert_eq!(
        fs.read_file(&uri, ReadFileOptions::default(), Some(&context))
            .await?,
        b"plain".to_vec()
    );
    Ok(())
}
