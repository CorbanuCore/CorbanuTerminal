//! #908: every desired-state RPC stores the desired state and then publishes
//! the matching status. An ephemeral enable didn't take
//! `desired_state_rpc_lock`, so it could land between a disable's two steps
//! (or a disable between its own), and the later publish overwrote the earlier
//! RPC's status: status `Disabled` while remote control is desired, or
//! `Connecting` while it is not. A test-only hook starts the other RPC at
//! exactly that point and polls it once; with the lock it can't proceed until
//! the first RPC has published.

use super::*;
use pretty_assertions::assert_eq;
use std::future::Future;
use std::pin::Pin;
use std::sync::Mutex as StdMutex;
use std::task::Context;
use std::task::Poll;
use std::task::Waker;

type Rpc = Pin<Box<dyn Future<Output = RemoteControlStatusChangedNotification> + Send>>;
/// The racing RPC and the result of its first poll.
type Started = Arc<StdMutex<Option<(Rpc, Poll<RemoteControlStatusChangedNotification>)>>>;

struct Interleaving {
    /// The status the first RPC returned.
    first: RemoteControlStatusChangedNotification,
    /// Whether the racing RPC had to wait when it started inside the first.
    racing_rpc_waited: bool,
    /// The status the racing RPC returned.
    racing: RemoteControlStatusChangedNotification,
}

/// Runs `first` on `handle`; once it has stored `stored`, and before it
/// publishes its status, starts `racing` on a clone and polls it once.
async fn interleave(
    mut handle: RemoteControlHandle,
    stored: RemoteControlDesiredState,
    first: impl FnOnce(RemoteControlHandle) -> Rpc,
    racing: impl FnOnce(RemoteControlHandle) -> Rpc + Send + 'static,
) -> Interleaving {
    // The racing RPC runs on a clone made before the hook is installed, so
    // it doesn't call the hook itself.
    let racing = StdMutex::new(Some((racing, handle.clone())));
    let started: Started = Arc::new(StdMutex::new(None));
    let hook_started = started.clone();
    handle.desired_state_stored_hook = Some(Arc::new(move |desired_state| {
        if desired_state != stored {
            return;
        }
        let Some((racing, racing_handle)) = racing.lock().expect("racing").take() else {
            return;
        };
        let mut rpc = racing(racing_handle);
        let mut cx = Context::from_waker(Waker::noop());
        let poll = rpc.as_mut().poll(&mut cx);
        *hook_started.lock().expect("started") = Some((rpc, poll));
    }));

    let first = first(handle).await;
    let (rpc, poll) = started
        .lock()
        .expect("started")
        .take()
        .expect("the first RPC stored the desired state");
    let (racing_rpc_waited, racing) = match poll {
        Poll::Ready(status) => (false, status),
        Poll::Pending => (true, rpc.await),
    };
    Interleaving {
        first,
        racing_rpc_waited,
        racing,
    }
}

async fn handle_with_state_db(
    codex_home: &TempDir,
    desired_state: RemoteControlDesiredState,
    status: RemoteControlConnectionStatus,
) -> RemoteControlHandle {
    let mut handle = remote_control_handle_with_current_enrollment(
        TEST_REMOTE_CONTROL_URL,
        remote_control_auth_manager(),
    );
    handle.state_db = Some(remote_control_state_runtime(codex_home).await);
    handle.desired_state_tx.send_replace(desired_state);
    handle
        .status_tx
        .send_modify(|current| current.status = status);
    handle
}

fn enable_ephemeral(handle: RemoteControlHandle) -> Rpc {
    Box::pin(async move {
        handle
            .enable_ephemeral()
            .await
            .expect("ephemeral enable should succeed")
    })
}

fn disable_ephemeral(handle: RemoteControlHandle) -> Rpc {
    Box::pin(async move { handle.disable_ephemeral().await })
}

/// The status and whether remote control is desired, once both RPCs finished.
fn settled(handle: &RemoteControlHandle) -> (RemoteControlConnectionStatus, bool) {
    (
        handle.status().status,
        handle.desired_state_tx.borrow().is_enabled(),
    )
}

#[tokio::test]
async fn ephemeral_enable_waits_for_a_disable_in_progress() {
    let codex_home = TempDir::new().expect("temp dir should create");
    let handle = handle_with_state_db(
        &codex_home,
        RemoteControlDesiredState::Enabled {
            persistence_preference: None,
        },
        RemoteControlConnectionStatus::Connected,
    )
    .await;

    let run = interleave(
        handle.clone(),
        RemoteControlDesiredState::Disabled,
        disable_ephemeral,
        enable_ephemeral,
    )
    .await;

    // The enable came last, so its status stands.
    assert_eq!(
        settled(&handle),
        (RemoteControlConnectionStatus::Connecting, true)
    );
    assert!(
        run.racing_rpc_waited,
        "the enable must wait for the disable"
    );
    assert_eq!(run.first.status, RemoteControlConnectionStatus::Disabled);
    assert_eq!(run.racing.status, RemoteControlConnectionStatus::Connecting);
}

#[tokio::test]
async fn disable_waits_for_an_ephemeral_enable_in_progress() {
    let codex_home = TempDir::new().expect("temp dir should create");
    let handle = handle_with_state_db(
        &codex_home,
        RemoteControlDesiredState::Disabled,
        RemoteControlConnectionStatus::Disabled,
    )
    .await;

    let run = interleave(
        handle.clone(),
        RemoteControlDesiredState::Enabled {
            persistence_preference: None,
        },
        enable_ephemeral,
        disable_ephemeral,
    )
    .await;

    // The disable came last, so its status stands.
    assert_eq!(
        settled(&handle),
        (RemoteControlConnectionStatus::Disabled, false)
    );
    assert!(
        run.racing_rpc_waited,
        "the disable must wait for the enable"
    );
    assert_eq!(run.first.status, RemoteControlConnectionStatus::Connecting);
    assert_eq!(run.racing.status, RemoteControlConnectionStatus::Disabled);
}
