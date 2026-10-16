//! Issue941 synthetic refusal diagnostics; no commands or credential reads execute.
use super::*;
use pretty_assertions::assert_eq;

fn action(level: SecurityLevel, kind: ProtectedActionKind) -> PostTaintAction {
    PostTaintAction {
        kind,
        state: PostTaintState {
            taint_generation: 1,
            policy: PolicyBinding::Unbound,
            level,
        },
    }
}

#[test]
fn issue941_approvals_off_names_the_actual_level() {
    let mut output = Vec::new();
    for level in [SecurityLevel::Moderate, SecurityLevel::Aggressive] {
        let reason = action(level, ProtectedActionKind::Credentials).approvals_off_rejection();
        assert!(
            reason.contains(&format!("Security level {level}:")),
            "{reason}"
        );
        assert!(reason.contains("approval_policy = never"), "{reason}");
        output.push(reason);
    }
    insta::assert_snapshot!("issue941_approvals_off_levels", output.join("\n"));
}

#[test]
fn issue941_approvals_off_explains_review_and_manual_recovery() {
    for kind in [
        ProtectedActionKind::Credentials,
        ProtectedActionKind::Disclosure,
    ] {
        let reason = action(SecurityLevel::Moderate, kind).approvals_off_rejection();
        let lower = reason.to_lowercase();
        assert!(lower.contains("review"), "{reason}");
        assert!(lower.contains("approval prompts"), "{reason}");
        assert!(lower.contains("asking again"), "{reason}");
        assert!(
            lower.contains("other security protections still apply"),
            "{reason}"
        );
    }
}

#[test]
fn issue941_diagnostic_repetition_preserves_captured_authority() {
    for level in [SecurityLevel::Moderate, SecurityLevel::Aggressive] {
        for kind in [
            ProtectedActionKind::Credentials,
            ProtectedActionKind::Disclosure,
        ] {
            let action = action(level, kind);
            let before = action.clone();
            let denied_read = action.reads_stay_denied();
            let reason = action.approvals_off_rejection();
            assert_eq!(reason, action.approvals_off_rejection());
            assert_eq!(action, before);
            assert_eq!(action.reads_stay_denied(), denied_read);
            assert!(
                !action
                    .state
                    .human_approval_lifts_rules(/*approved_at*/ None)
            );
            assert_eq!(action.refused_up_front(), None);
        }
    }
}

#[test]
fn issue941_existing_approval_prompt_does_not_gain_approvals_off_guidance() {
    for level in [SecurityLevel::Moderate, SecurityLevel::Aggressive] {
        let action = action(level, ProtectedActionKind::Disclosure);
        let prompt = action.approval_reason();
        assert!(prompt.starts_with("Security level:"), "{prompt}");
        assert!(
            prompt.contains("a cached or automatic approval does not count"),
            "{prompt}"
        );
        assert!(!prompt.contains("approvals are off"), "{prompt}");
        assert!(!prompt.contains("approval prompts"), "{prompt}");
    }
}
