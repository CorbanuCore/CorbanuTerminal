use codex_model_provider_info::CLAUDE_FABLE_5_PLAN_MODEL;
#[cfg(test)]
use codex_model_provider_info::CLAUDE_PLAN_MODEL;
use codex_model_provider_info::CLAUDE_PLAN_PROVIDER_ID;
use codex_model_provider_info::CORBANU_API_GLM_5_3_FLASH_MODEL;
use codex_model_provider_info::CORBANU_API_GPT_5_6_LUNA_MODEL;
use codex_model_provider_info::CORBANU_API_KIMI_K3_MODEL;
#[cfg(test)]
use codex_model_provider_info::KIMI_CODE_K3_MODEL;
#[cfg(test)]
use codex_model_provider_info::KIMI_CODE_PROVIDER_ID;
use codex_model_provider_info::OPENAI_PROVIDER_ID;
use codex_model_provider_info::OPENROUTER_GROK_4_6_MODEL;
use codex_model_provider_info::OPENROUTER_PROVIDER_ID;
use codex_model_provider_info::PFTERMINAL_PLAN_ANTHROPIC_PROVIDER_ID;
use codex_model_provider_info::PFTERMINAL_PLAN_PROVIDER_ID;
use codex_protocol::crew::CURRENT_CREW_SCHEMA_VERSION;
use codex_protocol::crew::CrewMemberSpec;
use codex_protocol::crew::CrewPolicy;
use codex_protocol::crew::CrewSpec;
use codex_protocol::crew::DelegationMode;
use codex_protocol::crew::RuntimeRequest;
use codex_protocol::openai_models::ReasoningEffort;

pub(crate) const STANDARD_NAZGUL_MODEL: &str = CLAUDE_FABLE_5_PLAN_MODEL;
pub(crate) const STANDARD_TROLL_MODEL: &str = "gpt-5.6-sol";
pub(crate) const STANDARD_ORC_MODEL: &str = "gpt-5.6-luna";
pub(crate) const STANDARD_ORC_2_MODEL: &str = "gpt-5.6-terra";
pub(crate) const STANDARD_ORC_3_MODEL: &str = OPENROUTER_GROK_4_6_MODEL;
pub(crate) const CORBANU_API_NAZGUL_MODEL: &str = CORBANU_API_KIMI_K3_MODEL;
pub(crate) const CORBANU_API_TROLL_MODEL: &str = CORBANU_API_GPT_5_6_LUNA_MODEL;
pub(crate) const CORBANU_API_ORC_MODEL: &str = CORBANU_API_GLM_5_3_FLASH_MODEL;

pub(crate) fn standard_crew_spec() -> CrewSpec {
    CrewSpec {
        schema_version: CURRENT_CREW_SCHEMA_VERSION,
        crew_id: "builtin-standard-v1".to_string(),
        preset_id: Some("standard-v1".to_string()),
        members: vec![
            member(
                "nazgul",
                "Nazgul",
                "nazgul",
                /*parent_member_id*/ None,
                RuntimeRequest::exact(
                    CLAUDE_PLAN_PROVIDER_ID,
                    STANDARD_NAZGUL_MODEL,
                    /*reasoning_effort*/ None,
                ),
            ),
            member(
                "troll",
                "Troll",
                "troll",
                Some("nazgul"),
                RuntimeRequest::exact(
                    OPENAI_PROVIDER_ID,
                    STANDARD_TROLL_MODEL,
                    Some(ReasoningEffort::XHigh),
                ),
            ),
            member(
                "orc-1",
                "Orc 1",
                "orc",
                Some("troll"),
                RuntimeRequest::exact(
                    OPENAI_PROVIDER_ID,
                    STANDARD_ORC_MODEL,
                    Some(ReasoningEffort::XHigh),
                ),
            ),
            member(
                "orc-2",
                "Orc 2",
                "orc",
                Some("troll"),
                RuntimeRequest::exact(
                    OPENAI_PROVIDER_ID,
                    STANDARD_ORC_2_MODEL,
                    Some(ReasoningEffort::XHigh),
                ),
            ),
            member(
                "orc-3",
                "Orc 3",
                "orc",
                Some("troll"),
                RuntimeRequest::exact(
                    OPENROUTER_PROVIDER_ID,
                    STANDARD_ORC_3_MODEL,
                    /*reasoning_effort*/ None,
                ),
            ),
        ],
        policy: CrewPolicy {
            delegation_mode: DelegationMode::Proactive,
            allow_ephemeral_descendants: true,
            provider_allowlist: vec![
                CLAUDE_PLAN_PROVIDER_ID.to_string(),
                OPENAI_PROVIDER_ID.to_string(),
                OPENROUTER_PROVIDER_ID.to_string(),
            ],
            maximum_spend_usd: None,
        },
    }
}

pub(crate) fn corbanu_api_crew_spec() -> CrewSpec {
    CrewSpec {
        schema_version: CURRENT_CREW_SCHEMA_VERSION,
        crew_id: "builtin-corbanu-api-v1".to_string(),
        preset_id: Some("corbanu-api-v1".to_string()),
        members: vec![
            member(
                "nazgul",
                "Nazgul",
                "nazgul",
                /*parent_member_id*/ None,
                RuntimeRequest::exact(
                    PFTERMINAL_PLAN_ANTHROPIC_PROVIDER_ID,
                    CORBANU_API_NAZGUL_MODEL,
                    /*reasoning_effort*/ None,
                ),
            ),
            member(
                "troll",
                "Troll",
                "troll",
                Some("nazgul"),
                RuntimeRequest::exact(
                    PFTERMINAL_PLAN_PROVIDER_ID,
                    CORBANU_API_TROLL_MODEL,
                    Some(ReasoningEffort::XHigh),
                ),
            ),
            member(
                "orc-1",
                "Orc 1",
                "orc",
                Some("troll"),
                RuntimeRequest::exact(
                    PFTERMINAL_PLAN_PROVIDER_ID,
                    CORBANU_API_ORC_MODEL,
                    /*reasoning_effort*/ None,
                ),
            ),
            member(
                "orc-2",
                "Orc 2",
                "orc",
                Some("troll"),
                RuntimeRequest::exact(
                    PFTERMINAL_PLAN_PROVIDER_ID,
                    CORBANU_API_ORC_MODEL,
                    /*reasoning_effort*/ None,
                ),
            ),
            member(
                "orc-3",
                "Orc 3",
                "orc",
                Some("troll"),
                RuntimeRequest::exact(
                    PFTERMINAL_PLAN_PROVIDER_ID,
                    CORBANU_API_ORC_MODEL,
                    /*reasoning_effort*/ None,
                ),
            ),
        ],
        policy: CrewPolicy {
            delegation_mode: DelegationMode::Proactive,
            allow_ephemeral_descendants: true,
            provider_allowlist: vec![
                PFTERMINAL_PLAN_ANTHROPIC_PROVIDER_ID.to_string(),
                PFTERMINAL_PLAN_PROVIDER_ID.to_string(),
            ],
            maximum_spend_usd: None,
        },
    }
}

/// A non-default preset used to qualify all runtime families required by the cutover.
#[cfg(test)]
pub(crate) fn multimodel_qualification_crew_spec() -> CrewSpec {
    let mut crew = standard_crew_spec();
    crew.crew_id = "builtin-multimodel-qualification-v1".to_string();
    crew.preset_id = Some("multimodel-qualification-v1".to_string());
    crew.members = vec![
        member(
            "manager",
            "Manager",
            "nazgul",
            /*parent_member_id*/ None,
            RuntimeRequest::exact(
                CLAUDE_PLAN_PROVIDER_ID,
                CLAUDE_FABLE_5_PLAN_MODEL,
                /*reasoning_effort*/ None,
            ),
        ),
        member(
            "opus-reviewer",
            "Opus reviewer",
            "orc",
            Some("manager"),
            RuntimeRequest::exact(
                CLAUDE_PLAN_PROVIDER_ID,
                CLAUDE_PLAN_MODEL,
                /*reasoning_effort*/ None,
            ),
        ),
        member(
            "grok-reviewer",
            "Grok reviewer",
            "orc",
            Some("manager"),
            RuntimeRequest::exact(
                OPENROUTER_PROVIDER_ID,
                OPENROUTER_GROK_4_6_MODEL,
                /*reasoning_effort*/ None,
            ),
        ),
        member(
            "kimi-reviewer",
            "Kimi reviewer",
            "orc",
            Some("manager"),
            RuntimeRequest::exact(
                KIMI_CODE_PROVIDER_ID,
                KIMI_CODE_K3_MODEL,
                /*reasoning_effort*/ None,
            ),
        ),
    ];
    crew.policy.provider_allowlist = vec![
        CLAUDE_PLAN_PROVIDER_ID.to_string(),
        OPENROUTER_PROVIDER_ID.to_string(),
        KIMI_CODE_PROVIDER_ID.to_string(),
    ];
    crew
}

fn member(
    logical_member_id: &str,
    display_name: &str,
    role_profile: &str,
    parent_member_id: Option<&str>,
    runtime_request: RuntimeRequest,
) -> CrewMemberSpec {
    CrewMemberSpec {
        logical_member_id: logical_member_id.to_string(),
        display_name: display_name.to_string(),
        role_profile: role_profile.to_string(),
        parent_member_id: parent_member_id.map(str::to_string),
        runtime_request,
    }
}

#[cfg(test)]
#[path = "crew_presets_tests.rs"]
mod tests;
