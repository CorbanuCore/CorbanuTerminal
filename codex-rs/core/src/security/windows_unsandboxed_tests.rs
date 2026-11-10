use super::*;
use pretty_assertions::assert_eq;

/// #1976: only Moderate and Aggressive on Windows with no Windows sandbox;
/// the non-admin and elevated sandboxes and Permissive are unchanged.
#[test]
fn issue_1976_refuses_only_protected_levels_on_windows_without_a_sandbox() {
    let mut refused = Vec::new();
    for windows in [false, true] {
        for level in [
            SecurityLevel::Permissive,
            SecurityLevel::Moderate,
            SecurityLevel::Aggressive,
        ] {
            for sandbox in [
                WindowsSandboxLevel::Disabled,
                WindowsSandboxLevel::RestrictedToken,
                WindowsSandboxLevel::Elevated,
            ] {
                if applies_on(windows, level, sandbox) {
                    refused.push((level, sandbox));
                }
            }
        }
    }
    assert_eq!(
        refused,
        vec![
            (SecurityLevel::Moderate, WindowsSandboxLevel::Disabled),
            (SecurityLevel::Aggressive, WindowsSandboxLevel::Disabled),
        ]
    );
    assert_eq!(
        applies(SecurityLevel::Moderate, WindowsSandboxLevel::Disabled),
        cfg!(windows)
    );
}

/// #1976: each refusal names the level and the setup steps.
#[test]
fn issue_1976_refusals_name_the_level_and_the_setup_steps() {
    for (level, name) in [
        (SecurityLevel::Moderate, "Moderate"),
        (SecurityLevel::Aggressive, "Aggressive"),
    ] {
        for (text, start) in [
            (command_refusal(level), "refused this command"),
            (file_refusal(level), "refused this file access"),
            (startup_notice(level), "needs the default Windows sandbox"),
        ] {
            assert!(
                text.starts_with(&format!("Security level {name} {start}")),
                "{text}"
            );
            for part in [
                "Set up default sandbox",
                "Administrator",
                "this session uses it from the next message",
                "`[windows]`",
                "switch to Permissive in /security",
            ] {
                assert!(text.contains(part), "{part}: {text}");
            }
        }
    }
}
