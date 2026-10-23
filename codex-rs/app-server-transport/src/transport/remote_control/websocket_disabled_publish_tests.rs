//! #857: a connection cycle that ended because remote control was disabled
//! must not overwrite the status of an enable that landed before the cycle
//! published `Disabled`. Both places the loop publishes it (a `connect()` that
//! returns `Disabled`, and a disable that ends a live connection) are driven
//! for real; a test-only hook runs the enable between the cycle seeing the
//! disable and its publish, and records the status right after the publish,
//! before a new cycle could change it.

use super::tests::TEST_INSTALLATION_ID;
use super::tests::connected_websocket_pair;
use super::tests::remote_control_auth_manager;
use super::tests::remote_control_status_channel;
use super::tests::test_current_enrollment;
use super::*;
use pretty_assertions::assert_eq;
use std::sync::Mutex as StdMutex;
use tokio::sync::Notify;
use tokio::time::Duration;
use tokio::time::timeout;

/// How long the racing enable may wait to store its desired state. It never
/// waits unless the loop holds a borrow of the desired state while it
/// publishes, which would deadlock the enable.
const ENABLE_WAIT: Duration = Duration::from_secs(5);

struct Race {
    /// Taken by `run()`.
    websocket: Option<RemoteControlWebsocket>,
    desired_state_tx: Arc<watch::Sender<RemoteControlDesiredState>>,
    status_publisher: RemoteControlStatusPublisher,
    status_rx: watch::Receiver<RemoteControlStatusChangedNotification>,
    shutdown_token: CancellationToken,
    /// The status right after the cycle's publish.
    published: Arc<StdMutex<Option<RemoteControlConnectionStatus>>>,
    published_notify: Arc<Notify>,
}

/// A loop for `remote_control_url`, enabled, whose disabled cycles race an
/// enable (when `racing_enable`) just before they publish.
fn race(remote_control_url: &str, racing_enable: bool) -> Race {
    let (status_publisher, status_rx) = remote_control_status_channel();
    let (transport_event_tx, _transport_event_rx) = mpsc::channel(1);
    let desired_state_tx = Arc::new(
        watch::channel(RemoteControlDesiredState::Enabled {
            persistence_preference: None,
        })
        .0,
    );
    let shutdown_token = CancellationToken::new();
    let mut websocket = RemoteControlWebsocket::new(
        RemoteControlWebsocketConfig {
            remote_control_url: remote_control_url.to_string(),
            installation_id: TEST_INSTALLATION_ID.to_string(),
            remote_control_target: None,
            server_name: "test-server".to_string(),
        },
        /*state_db*/ None,
        remote_control_auth_manager(),
        RemoteControlChannels {
            transport_event_tx,
            status_publisher: status_publisher.clone(),
            current_enrollment: test_current_enrollment(/*enrollment*/ None),
            pairing_persistence_key: watch::channel(None).0,
            desired_state_persistence_lock: Arc::new(Semaphore::new(1)),
        },
        shutdown_token.clone(),
        desired_state_tx.clone(),
    );
    let published = Arc::new(StdMutex::new(None));
    let published_notify = Arc::new(Notify::new());
    let hook_published_notify = published_notify.clone();
    let hook_desired_state_tx = desired_state_tx.clone();
    let hook_status_publisher = status_publisher.clone();
    let hook_status_rx = status_rx.clone();
    let hook_published = published.clone();
    websocket.disabled_publish_hook = Some(Arc::new(move |phase| match phase {
        DisabledPublish::Before if racing_enable => {
            // The disable that woke the loop publishes `Disabled` after
            // storing its desired state; the enable comes after both (the
            // handle serializes them).
            wait_for_status(&hook_status_rx, RemoteControlConnectionStatus::Disabled);
            // The enable, in its order (desired state, then `Connecting`), on
            // another thread: it must not wait for the loop.
            let desired_state_tx = hook_desired_state_tx.clone();
            let status_publisher = hook_status_publisher.clone();
            let (done_tx, done_rx) = std::sync::mpsc::channel();
            std::thread::spawn(move || {
                desired_state_tx.send_replace(RemoteControlDesiredState::Enabled {
                    persistence_preference: None,
                });
                status_publisher.publish_status(RemoteControlConnectionStatus::Connecting);
                let _ = done_tx.send(());
            });
            done_rx
                .recv_timeout(ENABLE_WAIT)
                .expect("the enable must not wait for the loop's borrow of the desired state");
        }
        DisabledPublish::Before => {}
        DisabledPublish::After => {
            *hook_published.lock().expect("published") = Some(hook_status_rx.borrow().status);
            hook_published_notify.notify_one();
        }
    }));
    Race {
        websocket: Some(websocket),
        desired_state_tx,
        status_publisher,
        status_rx,
        shutdown_token,
        published,
        published_notify,
    }
}

/// Waits (bounded) until `status_rx` shows `status`; for the hook, which runs
/// synchronously in the loop.
fn wait_for_status(
    status_rx: &watch::Receiver<RemoteControlStatusChangedNotification>,
    status: RemoteControlConnectionStatus,
) {
    let deadline = std::time::Instant::now() + ENABLE_WAIT;
    while status_rx.borrow().status != status {
        assert!(
            std::time::Instant::now() < deadline,
            "status never became {status:?}"
        );
        std::thread::yield_now();
    }
}

/// A disable, as `transition_disabled` does it.
fn disable(
    desired_state_tx: &watch::Sender<RemoteControlDesiredState>,
    status_publisher: &RemoteControlStatusPublisher,
) {
    desired_state_tx.send_replace(RemoteControlDesiredState::Disabled);
    status_publisher.publish_status(RemoteControlConnectionStatus::Disabled);
}

impl Race {
    /// The status right after the cycle's publish, once it happened.
    async fn published(&self) -> RemoteControlConnectionStatus {
        timeout(ENABLE_WAIT, self.published_notify.notified())
            .await
            .expect("the cycle publishes");
        self.published
            .lock()
            .expect("published")
            .expect("recorded with the notification")
    }
}

/// `run()`'s branch: `connect()` returns `Disabled`. An invalid URL makes
/// `connect()` publish `Errored` and wait for a disable, before any network
/// or authentication step.
async fn connect_returned_disabled(racing_enable: bool) -> RemoteControlConnectionStatus {
    let mut race = race("not a remote-control URL", racing_enable);
    let websocket = race.websocket.take().expect("the loop");
    let task = tokio::spawn(websocket.run(/*app_server_client_name_rx*/ None));
    timeout(
        ENABLE_WAIT,
        race.status_rx
            .wait_for(|status| status.status == RemoteControlConnectionStatus::Errored),
    )
    .await
    .expect("connect() reaches its wait for a disable")
    .expect("status channel stays open");
    disable(&race.desired_state_tx, &race.status_publisher);
    let published = race.published().await;
    race.shutdown_token.cancel();
    timeout(ENABLE_WAIT, task)
        .await
        .expect("the loop stops")
        .expect("the loop does not panic");
    published
}

/// `run_connection()`'s branch: a disable ends a live connection.
async fn connection_ended_by_disable(racing_enable: bool) -> RemoteControlConnectionStatus {
    let race = race("http://127.0.0.1:1/", racing_enable);
    let (client, _server) = connected_websocket_pair().await;
    let websocket = race.websocket.as_ref().expect("the loop");
    let (reason, ()) = tokio::join!(
        websocket.run_connection(client, race.shutdown_token.child_token()),
        async { disable(&race.desired_state_tx, &race.status_publisher) },
    );
    assert!(
        matches!(reason, ConnectionEndReason::Disabled),
        "{reason:?}"
    );
    race.published().await
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn a_disabled_connect_keeps_a_racing_enables_status() {
    assert_eq!(
        connect_returned_disabled(/*racing_enable*/ true).await,
        RemoteControlConnectionStatus::Connecting
    );
    assert_eq!(
        connect_returned_disabled(/*racing_enable*/ false).await,
        RemoteControlConnectionStatus::Disabled
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn a_disable_ending_a_connection_keeps_a_racing_enables_status() {
    assert_eq!(
        connection_ended_by_disable(/*racing_enable*/ true).await,
        RemoteControlConnectionStatus::Connecting
    );
    assert_eq!(
        connection_ended_by_disable(/*racing_enable*/ false).await,
        RemoteControlConnectionStatus::Disabled
    );
}
