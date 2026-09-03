use super::*;
use codex_protocol::crew::RuntimeRequest;
use pretty_assertions::assert_eq;

#[test]
fn standard_crew_is_valid_and_topologically_ordered() {
    let crew = standard_crew_spec();
    crew.validate().expect("valid standard crew");

    assert_eq!(
        crew.members
            .iter()
            .map(|member| (
                member.logical_member_id.as_str(),
                member.parent_member_id.as_deref(),
                member.role_profile.as_str(),
                member.runtime_request.exact_parts(),
            ))
            .collect::<Vec<_>>(),
        vec![
            (
                "nazgul",
                None,
                "nazgul",
                Some((CLAUDE_PLAN_PROVIDER_ID, CLAUDE_FABLE_5_PLAN_MODEL, None)),
            ),
            (
                "troll",
                Some("nazgul"),
                "troll",
                Some((
                    OPENAI_PROVIDER_ID,
                    "gpt-5.6-sol",
                    Some(ReasoningEffort::XHigh)
                )),
            ),
            (
                "orc-1",
                Some("troll"),
                "orc",
                Some((
                    OPENAI_PROVIDER_ID,
                    "gpt-5.6-luna",
                    Some(ReasoningEffort::XHigh)
                )),
            ),
            (
                "orc-2",
                Some("troll"),
                "orc",
                Some((
                    OPENAI_PROVIDER_ID,
                    "gpt-5.6-terra",
                    Some(ReasoningEffort::XHigh)
                )),
            ),
            (
                "orc-3",
                Some("troll"),
                "orc",
                Some((OPENROUTER_PROVIDER_ID, "x-ai/grok-4.6", None)),
            ),
        ]
    );
}

#[test]
fn corbanu_api_crew_is_valid_and_uses_the_requested_runtime_for_every_role() {
    let crew = corbanu_api_crew_spec();
    crew.validate().expect("valid Corbanu API crew");

    assert_eq!(
        crew.members
            .iter()
            .map(|member| (
                member.logical_member_id.as_str(),
                member.parent_member_id.as_deref(),
                member.role_profile.as_str(),
                member.runtime_request.exact_parts(),
            ))
            .collect::<Vec<_>>(),
        vec![
            (
                "nazgul",
                None,
                "nazgul",
                Some((
                    PFTERMINAL_PLAN_ANTHROPIC_PROVIDER_ID,
                    CORBANU_API_KIMI_K3_MODEL,
                    None,
                )),
            ),
            (
                "troll",
                Some("nazgul"),
                "troll",
                Some((
                    PFTERMINAL_PLAN_PROVIDER_ID,
                    CORBANU_API_GPT_5_6_LUNA_MODEL,
                    Some(ReasoningEffort::XHigh),
                )),
            ),
            (
                "orc-1",
                Some("troll"),
                "orc",
                Some((
                    PFTERMINAL_PLAN_PROVIDER_ID,
                    CORBANU_API_GLM_5_3_FLASH_MODEL,
                    None,
                )),
            ),
            (
                "orc-2",
                Some("troll"),
                "orc",
                Some((
                    PFTERMINAL_PLAN_PROVIDER_ID,
                    CORBANU_API_GLM_5_3_FLASH_MODEL,
                    None,
                )),
            ),
            (
                "orc-3",
                Some("troll"),
                "orc",
                Some((
                    PFTERMINAL_PLAN_PROVIDER_ID,
                    CORBANU_API_GLM_5_3_FLASH_MODEL,
                    None,
                )),
            ),
        ]
    );
    assert_eq!(
        crew.policy.provider_allowlist,
        vec![
            PFTERMINAL_PLAN_ANTHROPIC_PROVIDER_ID.to_string(),
            PFTERMINAL_PLAN_PROVIDER_ID.to_string(),
        ]
    );
}

#[test]
fn qualification_crew_covers_each_required_runtime_pair() {
    let crew = multimodel_qualification_crew_spec();
    crew.validate().expect("valid qualification crew");
    let runtimes = crew
        .members
        .iter()
        .filter_map(|member| {
            let RuntimeRequest::Exact {
                provider_id,
                model_id,
                ..
            } = &member.runtime_request
            else {
                return None;
            };
            Some((provider_id.as_str(), model_id.as_str()))
        })
        .collect::<Vec<_>>();

    assert_eq!(
        runtimes,
        vec![
            (CLAUDE_PLAN_PROVIDER_ID, CLAUDE_FABLE_5_PLAN_MODEL),
            (CLAUDE_PLAN_PROVIDER_ID, CLAUDE_PLAN_MODEL),
            (OPENROUTER_PROVIDER_ID, "x-ai/grok-4.6"),
            (KIMI_CODE_PROVIDER_ID, KIMI_CODE_K3_MODEL),
        ]
    );
}
