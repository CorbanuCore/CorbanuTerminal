//! Guard outcome controls; no transport-drop fallback participates in these tests.
use super::*;
use codex_exec_server::ExecServerError;
use pretty_assertions::assert_eq;
use tokio::process::Child;
use tokio::sync::Mutex;

#[derive(Clone, Copy)]
enum FirstTerminate {
    Success,
    Error,
    Pending,
}

struct ControlledProcess {
    id: ProcessId,
    first: FirstTerminate,
    calls: watch::Sender<usize>,
    child: Mutex<Option<Child>>,
}

impl ControlledProcess {
    // Upstream's executor launcher hands out `Arc<StdioServerProcessHandle>`; cloning the
    // `Arc` is the shared handle main's `Clone` gives, and the struct's `Drop` runs on the last
    // one.
    fn handle(first: FirstTerminate) -> (Arc<Self>, Arc<StdioServerProcessHandle>) {
        let process = Arc::new(Self {
            id: ProcessId::from("owned-test-process"),
            first,
            calls: watch::channel(0).0,
            child: Mutex::new(None),
        });
        let handle =
            StdioServerProcessHandle::executor("owned-test-process".into(), process.clone());
        (process, handle)
    }
}

impl ExecProcess for ControlledProcess {
    fn process_id(&self) -> &ProcessId {
        &self.id
    }
    fn subscribe_wake(&self) -> watch::Receiver<u64> {
        watch::channel(0).1
    }
    fn subscribe_events(&self) -> ExecProcessEventReceiver {
        ExecProcessEventReceiver::empty()
    }
    fn read(
        &self,
        _: Option<u64>,
        _: Option<usize>,
        _: Option<u64>,
    ) -> ExecProcessFuture<'_, ReadResponse> {
        Box::pin(std::future::pending())
    }
    fn write(&self, _: Vec<u8>) -> ExecProcessFuture<'_, WriteResponse> {
        Box::pin(async {
            Ok(WriteResponse {
                status: WriteStatus::Accepted,
            })
        })
    }
    fn signal(&self, _: ProcessSignal) -> ExecProcessFuture<'_, ()> {
        Box::pin(async { Ok(()) })
    }
    fn terminate(&self) -> ExecProcessFuture<'_, ()> {
        Box::pin(async move {
            // Count delivery only when the returned future is polled.
            self.calls.send_modify(|count| *count += 1);
            if *self.calls.borrow() == 1 {
                match self.first {
                    FirstTerminate::Success => {}
                    FirstTerminate::Error => {
                        return Err(ExecServerError::Protocol("synthetic rejection".into()));
                    }
                    FirstTerminate::Pending => std::future::pending::<()>().await,
                }
            }
            let child = self.child.lock().await.take();
            if let Some(mut child) = child {
                child
                    .kill()
                    .await
                    .map_err(|error| ExecServerError::Protocol(error.to_string()))?;
                *self.child.lock().await = Some(child);
            }
            Ok(())
        })
    }
}

#[tokio::test]
async fn successful_termination_is_not_repeated_by_calls_or_last_drop() {
    let (process, handle) = ControlledProcess::handle(FirstTerminate::Success);
    let clone = handle.clone();
    handle.terminate().await.expect("terminate");
    clone.terminate().await.expect("already terminated");
    drop(handle);
    drop(clone);
    tokio::task::yield_now().await;
    assert_eq!(*process.calls.borrow(), 1);
}

#[tokio::test]
async fn failed_termination_can_be_explicitly_retried() {
    let (process, handle) = ControlledProcess::handle(FirstTerminate::Error);
    assert!(handle.terminate().await.is_err());
    handle.terminate().await.expect("retry succeeds");
    drop(handle);
    tokio::task::yield_now().await;
    assert_eq!(*process.calls.borrow(), 2);
}

#[tokio::test(start_paused = true)]
async fn cancelled_termination_can_be_explicitly_retried() {
    let (process, handle) = ControlledProcess::handle(FirstTerminate::Pending);
    assert!(
        tokio::time::timeout(STARTUP_TIMEOUT, handle.terminate())
            .await
            .is_err()
    );
    handle.terminate().await.expect("retry succeeds");
    drop(handle);
    tokio::task::yield_now().await;
    assert_eq!(*process.calls.borrow(), 2);
}

#[tokio::test(start_paused = true)]
async fn cancelled_termination_is_retried_only_on_last_handle_drop() {
    let (process, handle) = ControlledProcess::handle(FirstTerminate::Pending);
    let clone = handle.clone();
    assert!(
        tokio::time::timeout(STARTUP_TIMEOUT, handle.terminate())
            .await
            .is_err()
    );
    drop(handle);
    tokio::task::yield_now().await;
    assert_eq!(*process.calls.borrow(), 1);
    let mut calls = process.calls.subscribe();
    drop(clone);
    tokio::time::timeout(HANG_DEADLINE, calls.wait_for(|count| *count == 2))
        .await
        .expect("last-drop retry")
        .expect("counter retained");
}

/// Spawned only by the owned-child test below, never as an ordinary test.
#[tokio::test]
#[ignore = "blocking child fixture, launched explicitly by owned-child cleanup test"]
async fn owned_process_fixture() {
    std::future::pending::<()>().await;
}

#[tokio::test]
async fn cancelled_handle_last_drop_reaps_its_owned_executor_child() -> anyhow::Result<()> {
    let (process, handle) = ControlledProcess::handle(FirstTerminate::Pending);
    let child = tokio::process::Command::new(std::env::current_exe()?)
        .args([
            "--ignored",
            "--exact",
            "rmcp_client::lifecycle_tests::terminate_tests::owned_process_fixture",
        ])
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .kill_on_drop(true)
        .spawn()?;
    *process.child.lock().await = Some(child);
    assert!(
        process
            .child
            .lock()
            .await
            .as_mut()
            .expect("owned child")
            .try_wait()?
            .is_none()
    );
    let mut cancelled = Box::pin(handle.terminate());
    assert!(futures::poll!(cancelled.as_mut()).is_pending());
    assert_eq!(*process.calls.borrow(), 1);
    drop(cancelled);
    drop(handle);
    tokio::time::timeout(Duration::from_secs(10), async {
        loop {
            if process
                .child
                .lock()
                .await
                .as_mut()
                .map(Child::try_wait)
                .transpose()?
                .flatten()
                .is_some()
            {
                break Ok::<_, io::Error>(());
            }
            tokio::task::yield_now().await;
        }
    })
    .await??;
    assert_eq!(*process.calls.borrow(), 2);
    Ok(())
}
