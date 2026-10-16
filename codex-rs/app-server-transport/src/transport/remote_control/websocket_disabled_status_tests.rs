use super::tests::remote_control_auth_manager;
use super::tests::remote_control_status_channel;
use super::tests::test_current_enrollment;
use super::*;
use pretty_assertions::assert_eq;
use tokio::time::Duration;
use tokio::time::timeout;

// Only test builds can pause the real loop between a Disabled connect outcome
// and its status publication, and observe it before a new cycle self-heals.
pub(super) struct DisabledPublishBarrier {
    pub(super) reached: oneshot::Sender<()>,
    pub(super) resume: oneshot::Receiver<()>,
    pub(super) published: oneshot::Sender<()>,
    pub(super) finish: oneshot::Receiver<()>,
}

async fn disabled_outcome_status(racing_enable: bool) -> RemoteControlStatusChangedNotification {
    let (status_publisher, mut status_rx) = remote_control_status_channel();
    let (transport_event_tx, _transport_event_rx) = mpsc::channel(1);
    let (desired_state_tx, _desired_state_rx) =
        watch::channel(RemoteControlDesiredState::Enabled {
            persistence_preference: None,
        });
    let desired_state_tx = Arc::new(desired_state_tx);
    let shutdown_token = CancellationToken::new();
    let (reached_tx, reached_rx) = oneshot::channel();
    let (resume_tx, resume_rx) = oneshot::channel();
    let (published_tx, published_rx) = oneshot::channel();
    let (finish_tx, finish_rx) = oneshot::channel();
    let mut websocket = RemoteControlWebsocket::new(
        RemoteControlWebsocketConfig {
            // Invalid before authentication or any network request. connect()
            // waits for a disable after publishing Errored on this real path.
            remote_control_url: "not a remote-control URL".to_string(),
            installation_id: super::tests::TEST_INSTALLATION_ID.to_string(),
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
    websocket.disabled_status_barrier = Some(DisabledPublishBarrier {
        reached: reached_tx,
        resume: resume_rx,
        published: published_tx,
        finish: finish_rx,
    });
    let task = tokio::spawn(websocket.run(/*app_server_client_name_rx*/ None));
    {
        let _status = timeout(
            Duration::from_secs(5),
            status_rx.wait_for(|status| status.status == RemoteControlConnectionStatus::Errored),
        )
        .await
        .expect("actual connect must reach its invalid-URL wait")
        .expect("status channel stays open");
    }
    desired_state_tx.send_replace(RemoteControlDesiredState::Disabled);
    status_publisher.publish_status(RemoteControlConnectionStatus::Disabled);
    timeout(Duration::from_secs(5), reached_rx)
        .await
        .expect("actual Disabled outcome must reach the branch barrier")
        .expect("branch barrier stays open");
    if racing_enable {
        desired_state_tx.send_replace(RemoteControlDesiredState::Enabled {
            persistence_preference: None,
        });
    }
    // The race starts with the enable's Connecting status. The control starts
    // with the same stale status but still desires Disabled, so it must publish.
    status_publisher.publish_status(RemoteControlConnectionStatus::Connecting);
    assert_eq!(desired_state_tx.borrow().is_enabled(), racing_enable);
    resume_tx
        .send(())
        .expect("release before-publication barrier");
    timeout(Duration::from_secs(5), published_rx)
        .await
        .expect("actual branch must finish publication")
        .expect("publication barrier stays open");
    let observed = status_rx.borrow().clone();
    shutdown_token.cancel();
    finish_tx
        .send(())
        .expect("release after-publication barrier");
    timeout(Duration::from_secs(5), task)
        .await
        .expect("websocket loop shuts down")
        .expect("websocket task joins");
    observed
}

#[tokio::test]
async fn a_disabled_outcome_never_overwrites_a_racing_enable() {
    let actual = disabled_outcome_status(/*racing_enable*/ true).await;
    let (_, status_rx) = remote_control_status_channel();
    assert_eq!(actual, status_rx.borrow().clone());
}

#[tokio::test]
async fn a_disabled_outcome_publishes_while_still_disabled() {
    let actual = disabled_outcome_status(/*racing_enable*/ false).await;
    let (publisher, status_rx) = remote_control_status_channel();
    publisher.publish_status(RemoteControlConnectionStatus::Disabled);
    assert_eq!(actual, status_rx.borrow().clone());
}
