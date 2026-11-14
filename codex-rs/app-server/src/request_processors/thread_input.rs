use super::invalid_request;
use codex_app_server_protocol::JSONRPCErrorError;
use codex_core::CodexThread;
use codex_protocol::crew::AgentClass;
use codex_protocol::protocol::MultiAgentVersion;
use codex_protocol::protocol::SessionSource;
use codex_protocol::protocol::SubAgentSource;

pub(super) const DIRECT_INPUT_TO_MULTI_AGENT_V2_SUBAGENT_ERROR: &str =
    "direct app-server input is not allowed for multi-agent v2 sub-agents";

/// Mirrors the ownership policy in request validation and thread capability responses.
pub(super) fn can_accept_direct_input(
    multi_agent_version: Option<MultiAgentVersion>,
    session_source: &SessionSource,
) -> bool {
    multi_agent_version != Some(MultiAgentVersion::V2)
        || !matches!(
            session_source,
            SessionSource::SubAgent(SubAgentSource::ThreadSpawn { .. })
        )
        || human_addressable_crew_member(session_source)
}

fn human_addressable_crew_member(session_source: &SessionSource) -> bool {
    matches!(
        session_source,
        SessionSource::SubAgent(SubAgentSource::ThreadSpawn {
            agent_class: Some(AgentClass::CrewMember {
                human_addressable: true,
                ..
            }),
            ..
        })
    )
}

/// Turn input from `app_server_client_name`: the Corbanu TUI may also address
/// a multi-agent v2 sub-agent directly (its agent panes).
pub(super) fn direct_input_to_v2_thread_spawn_is_blocked(
    multi_agent_version: Option<MultiAgentVersion>,
    session_source: &SessionSource,
    app_server_client_name: Option<&str>,
) -> bool {
    !can_accept_direct_input(multi_agent_version, session_source)
        && app_server_client_name != Some("codex-tui")
}

pub(super) async fn ensure_direct_input_allowed(
    thread: &CodexThread,
) -> Result<(), JSONRPCErrorError> {
    ensure_direct_input_allowed_from_client(thread, /*app_server_client_name*/ None).await
}

pub(super) async fn ensure_direct_input_allowed_from_client(
    thread: &CodexThread,
    app_server_client_name: Option<&str>,
) -> Result<(), JSONRPCErrorError> {
    let config_snapshot = thread.config_snapshot().await;
    if direct_input_to_v2_thread_spawn_is_blocked(
        thread.multi_agent_version(),
        &config_snapshot.session_source,
        app_server_client_name,
    ) {
        return Err(invalid_request(
            DIRECT_INPUT_TO_MULTI_AGENT_V2_SUBAGENT_ERROR,
        ));
    }

    Ok(())
}
