use super::*;
use core_test_support::PathBufExt;
use pretty_assertions::assert_eq;
use std::sync::mpsc;
use tokio::sync::oneshot;

fn filter_snapshot(raw: &str, removed: &[String]) -> Result<String> {
    filter_captured_snapshot(
        raw,
        ShellType::Bash,
        &ShellEnvironmentPolicy::default(),
        /*credential_broker*/ None,
        /*broker_config_revision*/ 0,
        removed,
        &tokio::runtime::Handle::current(),
    )
    .map(|(snapshot, _)| snapshot)
}

// Exercise the same filter-await -> write boundary used by capture_snapshot
// and write_shell_snapshot, with a controllable blocking scanner.
async fn filter_and_write_snapshot(
    path: &AbsolutePathBuf,
    scan_timeout: Duration,
    filter: impl FnOnce() -> Result<String> + Send + 'static,
) -> Result<()> {
    let snapshot = run_snapshot_filter(scan_timeout, filter).await?;
    write_filtered_snapshot(path, snapshot).await
}

// A channel rendezvous makes responsiveness deterministic, including on slow
// debug runners. If filtering is moved back onto this current-thread runtime,
// the blocking receive expires before the async sibling can release it.
#[tokio::test(flavor = "current_thread")]
async fn large_filter_leaves_async_worker_responsive() -> Result<()> {
    let dir = tempfile::tempdir()?;
    let path = dir.path().join("snapshot.sh").abs();
    let raw = zeroize::Zeroizing::new(format!(
        "# Snapshot file\n{}\0\0#values\0SAFE\0ordinary\0#end\x000\0\0",
        "# ordinary shell function and alias text\n".repeat(80_000)
    ));
    assert!(raw.len() > 3 * 1024 * 1024);
    let expected = filter_snapshot(&raw, &[])?;
    let (started_tx, started_rx) = oneshot::channel();
    let (release_tx, release_rx) = mpsc::channel();
    let writing = filter_and_write_snapshot(&path, Duration::from_secs(15), move || {
        let _ = started_tx.send(());
        release_rx.recv_timeout(Duration::from_secs(5))?;
        filter_snapshot(&raw, &[])
    });
    let sibling = async {
        started_rx.await?;
        tokio::time::sleep(Duration::from_millis(10)).await;
        assert!(!path.exists());
        release_tx.send(())?;
        anyhow::Ok(())
    };
    let (written, progressed) = tokio::join!(writing, sibling);
    progressed?;
    written?;
    assert_eq!(fs::read_to_string(&path).await?, expected);
    Ok(())
}

#[tokio::test(flavor = "current_thread")]
async fn timed_out_filter_never_writes_a_late_result() -> Result<()> {
    let dir = tempfile::tempdir()?;
    let path = dir.path().join("not-created/snapshot.sh").abs();
    let (release_tx, release_rx) = mpsc::channel();
    let (started_tx, started_rx) = oneshot::channel();
    let (finished_tx, finished_rx) = oneshot::channel();
    let writing = filter_and_write_snapshot(&path, Duration::from_millis(200), move || {
        let _ = started_tx.send(());
        release_rx.recv_timeout(Duration::from_secs(5))?;
        let _ = finished_tx.send(());
        Ok("# Snapshot file\nlate result".to_string())
    });
    let (result, started) = tokio::join!(writing, started_rx);
    started?;
    assert_eq!(
        result.unwrap_err().to_string(),
        "Snapshot credential scan timed out"
    );
    assert!(!path.exists());
    assert!(!dir.path().join("not-created").exists());
    // Releasing an already-running scan after the deadline cannot write.
    release_tx.send(())?;
    finished_rx.await?;
    assert!(!path.exists());
    Ok(())
}

#[tokio::test(flavor = "current_thread")]
async fn cancelled_filter_never_writes_a_late_result() -> Result<()> {
    let dir = tempfile::tempdir()?;
    let path = dir.path().join("snapshot.sh").abs();
    let output = path.clone();
    let (started_tx, started_rx) = oneshot::channel();
    let (release_tx, release_rx) = mpsc::channel();
    let (finished_tx, finished_rx) = oneshot::channel();
    let caller = tokio::spawn(async move {
        filter_and_write_snapshot(&output, Duration::from_secs(15), move || {
            let _ = started_tx.send(());
            release_rx.recv_timeout(Duration::from_secs(5))?;
            let _ = finished_tx.send(());
            Ok("# Snapshot file\nlate result".to_string())
        })
        .await
    });
    started_rx.await?;
    caller.abort();
    assert!(caller.await.unwrap_err().is_cancelled());
    assert!(!path.exists());
    release_tx.send(())?;
    finished_rx.await?;
    assert!(!path.exists());
    Ok(())
}

#[tokio::test]
async fn filter_refusal_and_panic_never_write() -> Result<()> {
    let dir = tempfile::tempdir()?;
    let path = dir.path().join("snapshot.sh").abs();
    let refusal = filter_and_write_snapshot(&path, Duration::from_secs(5), || {
        filter_snapshot("invalid capture", &[])
    })
    .await;
    assert!(refusal.is_err());
    assert!(!path.exists());
    let panic = filter_and_write_snapshot(&path, Duration::from_secs(5), || {
        panic!("synthetic filter panic")
    })
    .await;
    assert_eq!(
        panic.unwrap_err().to_string(),
        "Snapshot credential scan failed"
    );
    assert!(!path.exists());
    Ok(())
}

#[tokio::test]
async fn blocking_filter_matches_synchronous_filter_on_randomized_captures() -> Result<()> {
    use rand::Rng;
    use rand::SeedableRng;
    let mut rng = rand::rngs::StdRng::seed_from_u64(/*state*/ 2541);
    let dir = tempfile::tempdir()?;
    for case in 0..100 {
        let text: String = (0..rng.random_range(0..2048))
            .map(|_| char::from(rng.random_range(b'a'..=b'z')))
            .collect();
        let raw = format!(
            "# Snapshot file\n# {text}\n\0\0#values\0SAFE\0ordinary\0SYNTHETIC_TOKEN\0synthetic-2541-value\0COPY\0{}\0#end\x001\0COPY\0export COPY='{}'\n\0\0",
            if case % 2 == 0 {
                "synthetic-2541-value"
            } else {
                "ordinary"
            },
            if case % 2 == 0 {
                "synthetic-2541-value"
            } else {
                "ordinary"
            },
        );
        // Include malformed framing and metadata to compare refusals too.
        let raw = if case % 3 == 0 {
            raw[..raw.len() - 1].to_string()
        } else {
            raw
        };
        let expected = filter_snapshot(&raw, &[]);
        let path = dir.path().join(format!("{case}.sh")).abs();
        let raw = zeroize::Zeroizing::new(raw);
        let actual = filter_and_write_snapshot(&path, Duration::from_secs(5), move || {
            filter_snapshot(&raw, &[])
        })
        .await;
        match expected {
            Ok(expected) => {
                actual?;
                assert_eq!(fs::read_to_string(&path).await?, expected);
            }
            Err(expected) => {
                assert_eq!(actual.unwrap_err().to_string(), expected.to_string());
                assert!(!path.exists());
            }
        }
    }
    Ok(())
}
