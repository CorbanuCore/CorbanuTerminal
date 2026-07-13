use crate::agent::AgentControl;
use codex_protocol::error::CodexErr;
use codex_protocol::protocol::MultiAgentVersion;
use codex_protocol::protocol::SessionSource;
use codex_protocol::protocol::SubAgentSource;
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
    let CodexErr::AgentLimitReached { max_threads } = err else {
        panic!("expected AgentLimitReached");
    };
    assert_eq!(max_threads, 1);

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
fn execution_guards_reserve_nazgul_control_path_under_worker_saturation() {
    let control = control_with_limit(/*max_threads*/ 1);
    let worker_source = SessionSource::SubAgent(SubAgentSource::ThreadSpawn {
        parent_thread_id: codex_protocol::ThreadId::new(),
        depth: 1,
        agent_path: None,
        agent_nickname: Some("Snaga".to_string()),
        agent_role: Some("orc".to_string()),
    });
    let nazgul_source = SessionSource::SubAgent(SubAgentSource::ThreadSpawn {
        parent_thread_id: codex_protocol::ThreadId::new(),
        depth: 1,
        agent_path: None,
        agent_nickname: Some("Angmar".to_string()),
        agent_role: Some("nazgul".to_string()),
    });

    let _worker = control
        .execution_guard(MultiAgentVersion::V2, &worker_source)
        .expect("worker should occupy the only worker slot");
    let Err(CodexErr::AgentLimitReached { max_threads }) =
        control.ensure_execution_capacity(MultiAgentVersion::V2, &worker_source)
    else {
        panic!("another worker should remain capacity limited");
    };
    assert_eq!(max_threads, 1);

    control
        .ensure_execution_capacity(MultiAgentVersion::V2, &nazgul_source)
        .expect("Nazgul control turns must bypass saturated worker capacity");
    assert!(
        control
            .execution_guard(MultiAgentVersion::V2, &nazgul_source)
            .is_none(),
        "Nazgul control turns must not consume worker capacity"
    );
}
