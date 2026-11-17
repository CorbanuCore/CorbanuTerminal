use super::*;
use pretty_assertions::assert_eq;

#[test]
fn fable_5_2_always_uses_adaptive_thinking() {
    for (effort, expected) in [
        (None, "high"),
        (Some(ReasoningEffortConfig::None), "high"),
        (Some(ReasoningEffortConfig::Minimal), "high"),
        (Some(ReasoningEffortConfig::High), "high"),
        (Some(ReasoningEffortConfig::Low), "low"),
        (
            Some(ReasoningEffortConfig::Custom("max".to_string())),
            "max",
        ),
    ] {
        assert_eq!(
            anthropic_reasoning_for_model_and_effort("claude-fable-5-2", effort.as_ref()),
            (
                Some(json!({"type": "adaptive", "display": "summarized"})),
                Some(json!({"effort": expected}))
            )
        );
    }
}
