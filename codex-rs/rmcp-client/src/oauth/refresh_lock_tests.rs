use super::RefreshCredentialLock;
use anyhow::Result;
use std::time::Duration;
use tempfile::tempdir;

#[tokio::test]
async fn acquisition_times_out_without_stealing() -> Result<()> {
    let codex_home = tempdir()?;
    let store_key = "test-store-key";
    let held_lock = RefreshCredentialLock::acquire_in(
        codex_home.path(),
        store_key,
        Duration::from_millis(/*millis*/ 100),
    )
    .await?;

    let error = RefreshCredentialLock::acquire_in(
        codex_home.path(),
        store_key,
        Duration::from_millis(/*millis*/ 50),
    )
    .await
    .err()
    .expect("contending lock acquisition should time out");
    assert!(
        error
            .to_string()
            .contains("timed out after 50ms waiting for OAuth refresh lock"),
        "unexpected error: {error:#}"
    );

    drop(held_lock);
    let _reacquired = RefreshCredentialLock::acquire_in(
        codex_home.path(),
        store_key,
        Duration::from_millis(/*millis*/ 100),
    )
    .await?;
    Ok(())
}

// Issue928: native retained-descriptor lifetime through the actual refresh guard.
// An error return must drop the actual guard before the caller observes the error.
fn refresh_error_return(_guard: RefreshCredentialLock) -> std::result::Result<(), &'static str> {
    Err("synthetic transaction error")
}

#[derive(Clone, Copy, Debug)]
enum LockScopeExit {
    Normal,
    Error,
    Unwind,
}

#[tokio::test]
async fn issue928_mcp_refresh_guard_scope_releases_with_retained_descriptor() -> Result<()> {
    for exit in [
        LockScopeExit::Normal,
        LockScopeExit::Error,
        LockScopeExit::Unwind,
    ] {
        let home = tempdir()?;
        let key = "synthetic-refresh-lifetime";
        let guard = RefreshCredentialLock::acquire_in(
            home.path(),
            key,
            Duration::from_millis(/*millis*/ 100),
        )
        .await?;
        let retained = guard._file.try_clone()?;
        let before = retained.metadata()?;
        match exit {
            LockScopeExit::Normal => drop(guard),
            LockScopeExit::Error => {
                let result = refresh_error_return(guard);
                pretty_assertions::assert_eq!(result, Err("synthetic transaction error"));
            }
            LockScopeExit::Unwind => {
                let result = std::panic::catch_unwind(move || {
                    let _owned = guard;
                    panic!("synthetic transaction unwind");
                });
                assert!(result.is_err());
            }
        }
        let reacquired = RefreshCredentialLock::acquire_in(
            home.path(),
            key,
            Duration::from_millis(/*millis*/ 100),
        )
        .await;
        assert!(
            reacquired.is_ok(),
            "real refresh guard must release on {exit:?} while its duplicate remains open: {:?}",
            reacquired.as_ref().err()
        );
        let reacquired = reacquired?;
        let after = reacquired._file.metadata()?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::MetadataExt;
            pretty_assertions::assert_eq!((before.dev(), before.ino()), (after.dev(), after.ino()));
        }
        #[cfg(not(unix))]
        pretty_assertions::assert_eq!(before.len(), after.len());
        drop(retained);
    }
    Ok(())
}

#[tokio::test]
async fn issue928_mcp_live_refresh_guard_exclusion_and_independence() -> Result<()> {
    let home = tempdir()?;
    let key = "synthetic-shared-refresh-key";
    let held =
        RefreshCredentialLock::acquire_in(home.path(), key, Duration::from_millis(/*millis*/ 100))
            .await?;
    drop(held._file.try_clone()?);
    let _independent = RefreshCredentialLock::acquire_in(
        home.path(),
        "synthetic-other-refresh-key",
        Duration::from_millis(/*millis*/ 100),
    )
    .await?;
    let error = RefreshCredentialLock::acquire_in(home.path(), key, Duration::ZERO)
        .await
        .err()
        .expect("same-resource contender must time out while guard lives");
    assert!(
        error
            .to_string()
            .contains("timed out after 0ns waiting for OAuth refresh lock")
    );
    drop(held);
    let _reacquired =
        RefreshCredentialLock::acquire_in(home.path(), key, Duration::from_millis(/*millis*/ 100))
            .await?;
    Ok(())
}

#[tokio::test]
async fn issue928_mcp_refresh_acquisition_io_errors_are_preserved() -> Result<()> {
    let home = tempdir()?;
    let blocked_home = home.path().join("not-a-directory");
    std::fs::write(&blocked_home, b"synthetic sentinel")?;
    let error = RefreshCredentialLock::acquire_in(
        &blocked_home,
        "synthetic-io-error",
        Duration::from_millis(/*millis*/ 100),
    )
    .await
    .err()
    .expect("file-as-home must fail directory creation");
    assert!(error.downcast_ref::<std::io::Error>().is_some());
    pretty_assertions::assert_eq!(std::fs::read(&blocked_home)?, b"synthetic sentinel");
    let guard = RefreshCredentialLock::acquire_in(
        home.path(),
        "synthetic-io-error",
        Duration::from_millis(/*millis*/ 100),
    )
    .await?;
    drop(guard);
    let mut entries = std::fs::read_dir(home.path().join(super::REFRESH_LOCK_DIR))?;
    let path = entries
        .next()
        .expect("actual acquire creates a lock file")?
        .path();
    assert!(entries.next().is_none());
    std::fs::remove_file(&path)?;
    std::fs::create_dir(&path)?;
    let error = RefreshCredentialLock::acquire_in(
        home.path(),
        "synthetic-io-error",
        Duration::from_millis(/*millis*/ 100),
    )
    .await
    .err()
    .expect("directory-as-lock must fail opening the file");
    assert!(
        error
            .to_string()
            .contains("failed to open OAuth refresh lock")
    );
    assert!(path.is_dir());
    Ok(())
}
