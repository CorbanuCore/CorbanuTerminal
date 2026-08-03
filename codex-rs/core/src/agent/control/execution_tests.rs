use crate::agent::AgentControl;
use codex_protocol::error::CodexErrorDetails;
use codex_protocol::protocol::MultiAgentVersion;
use codex_protocol::protocol::Op;
use codex_protocol::protocol::SessionSource;
use codex_protocol::protocol::SubAgentSource;
use codex_protocol::user_input::UserInput;
use pretty_assertions::assert_eq;

fn control_with_limit(max_threads: usize) -> AgentControl {
    let control = AgentControl::default();
    control.agent_execution_limiter.initialize(max_threads);
    control
}

#[test]
fn execution_guards_count_active_v2_subagent_turns() {
    let control = control_with_limit(/*max_threads*/ 1);
    // Child role configs cannot replace the root-derived session limit.
    control
        .agent_execution_limiter
        .initialize(/*max_threads*/ 2);
    let source = SessionSource::SubAgent(SubAgentSource::Other("worker".to_string()));

    control
        .ensure_execution_capacity(MultiAgentVersion::V2, &source)
        .expect("first active turn should fit");
    let first = control
        .execution_guard(MultiAgentVersion::V2, &source)
        .expect("v2 subagent execution should be counted");
    let Err(err) = control.ensure_execution_capacity(MultiAgentVersion::V2, &source) else {
        panic!("second active turn should exceed the derived non-root cap");
    };
    let CodexErrorDetails::AgentLimitReached { max_threads } = err.details() else {
        panic!("expected AgentLimitReached");
    };
    assert_eq!(*max_threads, 1);

    drop(first);
    control
        .ensure_execution_capacity(MultiAgentVersion::V2, &source)
        .expect("capacity should be released when the running task drops");
}

#[test]
fn execution_guards_ignore_root_and_v1_turns() {
    let control = control_with_limit(/*max_threads*/ 0);

    assert!(
        control
            .execution_guard(MultiAgentVersion::V2, &SessionSource::Cli)
            .is_none()
    );
    assert!(
        control
            .execution_guard(
                MultiAgentVersion::V1,
                &SessionSource::SubAgent(SubAgentSource::Other("worker".to_string())),
            )
            .is_none()
    );
}

#[test]
fn execution_guards_do_not_derive_capacity_policy_from_role_names() {
    let control = control_with_limit(/*max_threads*/ 1);
    let worker_source = SessionSource::SubAgent(SubAgentSource::ThreadSpawn {
        parent_thread_id: codex_protocol::ThreadId::new(),
        depth: 1,
        agent_path: None,
        agent_nickname: Some("Snaga".to_string()),
        agent_role: Some("orc".to_string()),
        agent_class: None,
    });
    let nazgul_source = SessionSource::SubAgent(SubAgentSource::ThreadSpawn {
        parent_thread_id: codex_protocol::ThreadId::new(),
        depth: 1,
        agent_path: None,
        agent_nickname: Some("Angmar".to_string()),
        agent_role: Some("nazgul".to_string()),
        agent_class: None,
    });

    let _worker = control
        .execution_guard(MultiAgentVersion::V2, &worker_source)
        .expect("worker should occupy the only worker slot");
    let Err(err) = control.ensure_execution_capacity(MultiAgentVersion::V2, &worker_source) else {
        panic!("another worker should remain capacity limited");
    };
    let CodexErrorDetails::AgentLimitReached { max_threads } = err.details() else {
        panic!("expected AgentLimitReached");
    };
    assert_eq!(*max_threads, 1);

    let Err(err) = control.ensure_execution_capacity(MultiAgentVersion::V2, &nazgul_source) else {
        panic!("display roles must not bypass native execution capacity");
    };
    let CodexErrorDetails::AgentLimitReached { max_threads } = err.details() else {
        panic!("expected AgentLimitReached");
    };
    assert_eq!(*max_threads, 1);
    let err = match control.try_execution_guard(MultiAgentVersion::V2, &nazgul_source) {
        Ok(_) => panic!("display roles must remain capacity limited"),
        Err(err) => err,
    };
    assert!(matches!(
        err.details(),
        CodexErrorDetails::AgentLimitReached { max_threads: 1 }
    ));
}

#[test]
fn human_input_is_control_plane_work_not_worker_execution() {
    let user_input = Op::UserInput {
        items: vec![UserInput::Text {
            text: "Reprioritize the crew while every worker slot is occupied.".to_string(),
            text_elements: Vec::new(),
        }],
        final_output_json_schema: None,
        responsesapi_client_metadata: None,
        additional_context: Default::default(),
        thread_settings: Default::default(),
    };

    assert!(!super::op_starts_worker_turn(&user_input));
    assert!(super::op_starts_worker_turn(&Op::WakePendingWork));
}

#[tokio::test]
async fn capacity_waiter_unblocks_after_atomic_worker_reservation_is_released() {
    let control = control_with_limit(/*max_threads*/ 1);
    let source = SessionSource::SubAgent(SubAgentSource::Other("worker".to_string()));
    let reservation = control
        .try_execution_guard(MultiAgentVersion::V2, &source)
        .expect("first reservation")
        .expect("worker reservation");
    let err = match control.try_execution_guard(MultiAgentVersion::V2, &source) {
        Ok(_) => panic!("second worker must be capacity limited"),
        Err(err) => err,
    };
    assert!(matches!(
        err.details(),
        CodexErrorDetails::AgentLimitReached { max_threads: 1 }
    ));

    let waiting_control = control.clone();
    let waiter = tokio::spawn(async move {
        waiting_control.wait_for_execution_capacity().await;
        waiting_control
            .try_execution_guard(MultiAgentVersion::V2, &source)
            .expect("reservation after wake")
            .expect("worker reservation after wake")
    });
    assert!(
        tokio::time::timeout(std::time::Duration::from_millis(25), async {
            while !waiter.is_finished() {
                tokio::task::yield_now().await;
            }
        })
        .await
        .is_err()
    );
    drop(reservation);
    let resumed_reservation = tokio::time::timeout(std::time::Duration::from_secs(1), waiter)
        .await
        .expect("capacity waiter should wake")
        .expect("capacity waiter task");
    drop(resumed_reservation);
}
