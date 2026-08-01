use super::*;
use crate::AgentPath;
use crate::ThreadId;
use crate::protocol::SubAgentActivityEvent;
use crate::protocol::SubAgentActivityKind;
use pretty_assertions::assert_eq;

#[test]
fn subagent_activity_legacy_event_preserves_available_identity() {
    let agent_thread_id = ThreadId::new();
    let agent_path = AgentPath::try_from("/root/worker").expect("valid agent path");
    let item = SubAgentActivityItem {
        id: "activity-1".to_string(),
        kind: SubAgentActivityKind::Interacted,
        agent_thread_id,
        agent_path: agent_path.clone(),
    };

    let EventMsg::SubAgentActivity(actual) = item.as_legacy_event(123) else {
        panic!("expected sub-agent activity event");
    };
    assert_eq!(
        actual,
        SubAgentActivityEvent {
            event_id: "activity-1".to_string(),
            occurred_at_ms: 123,
            agent_thread_id,
            agent_path,
            agent_nickname: None,
            agent_role: None,
            task_preview: None,
            kind: SubAgentActivityKind::Interacted,
        }
    );
}
