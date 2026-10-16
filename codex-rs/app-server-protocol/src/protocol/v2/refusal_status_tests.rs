use super::*;

#[test]
fn issue822_command_status_conversions_preserve_refusal_and_decline() {
    for (core, expected) in [
        (CoreExecCommandStatus::Refused, "refused"),
        (CoreExecCommandStatus::Declined, "declined"),
        (CoreExecCommandStatus::Completed, "completed"),
        (CoreExecCommandStatus::Failed, "failed"),
    ] {
        let item = CoreCommandExecutionStatus::from(core.clone());
        let legacy = CommandExecutionStatus::from(core);
        let typed = CommandExecutionStatus::from(item);
        assert_eq!(typed, legacy);
        let wire = serde_json::to_value(&typed).expect("serialize status");
        assert_eq!(wire, expected);
        assert_eq!(
            serde_json::from_value::<CommandExecutionStatus>(wire).unwrap(),
            typed
        );
    }
}
