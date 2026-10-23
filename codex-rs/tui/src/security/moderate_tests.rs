use pretty_assertions::assert_eq;

use super::*;

/// #750 (D1, D3): no Moderate row says a command can leave the sandbox, the
/// flag-dependent rows say when their flag is off, and Moderate is not saved
/// without the untrusted-content labels.
#[test]
fn issue_750_rows_claim_only_what_is_enforced() {
    let on = Facts {
        untrusted_content: true,
        secretless_launch: true,
        output_gate: true,
    };
    for facts in [on, Facts::default()] {
        let rows = rows(facts);
        let labels = rows.iter().map(|(label, _)| *label).collect::<Vec<_>>();
        assert_eq!(
            labels,
            vec![
                "Sandbox",
                "Protected files",
                "Untrusted content",
                "Secrets",
                "Model keys",
                "Child agents",
                "Kill switch",
            ]
        );
        for (label, value) in &rows {
            assert!(
                !value.contains("can run outside the sandbox"),
                "{label}: {value}"
            );
        }
        let secrets = &rows[3].1;
        assert_eq!(
            secrets.contains("secretless_agent_launch off"),
            !facts.secretless_launch,
            "{secrets}"
        );
        assert_eq!(blocker(facts).is_some(), !facts.untrusted_content);
    }
}
