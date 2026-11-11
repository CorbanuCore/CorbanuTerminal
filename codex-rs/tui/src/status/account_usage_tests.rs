use super::*;
use codex_app_server_protocol::TokenUsageBreakdown;
use pretty_assertions::assert_eq;

fn usage(
    provider_id: &str,
    account: &str,
    [input, cached, output, reasoning]: [i64; 4],
) -> ThreadAccountTokenUsage {
    ThreadAccountTokenUsage {
        provider_id: provider_id.to_string(),
        account: account.to_string(),
        usage: TokenUsageBreakdown {
            total_tokens: input + output,
            input_tokens: input,
            cached_input_tokens: cached,
            cache_write_input_tokens: 0,
            output_tokens: output,
            reasoning_output_tokens: reasoning,
        },
    }
}

fn tokens([input, cached, output, reasoning]: [i64; 4]) -> TokenUsage {
    TokenUsage {
        input_tokens: input,
        cached_input_tokens: cached,
        output_tokens: output,
        reasoning_output_tokens: reasoning,
        total_tokens: input + output,
    }
}

fn text(lines: Vec<Vec<Span<'static>>>) -> Vec<String> {
    lines
        .into_iter()
        .map(|spans| spans.iter().map(|span| span.content.as_ref()).collect())
        .collect()
}

/// A spawned agent on another account counts toward that account; another
/// provider's accounts are not shown; each report is a running total.
#[test]
fn threads_add_up_per_account_of_the_active_provider() {
    let coordinator = ThreadId::new();
    let worker = ThreadId::new();
    let other_provider = ThreadId::new();
    let mut session = SessionAccountUsage::default();
    session.record(coordinator, vec![usage("zai", "default", [10, 0, 1, 0])]);
    // A later report from the same thread replaces the earlier one.
    session.record(coordinator, vec![usage("zai", "default", [100, 20, 10, 5])]);
    session.record(
        worker,
        vec![
            usage("zai", "work", [50, 0, 5, 0]),
            usage("kimi", "alt", [7, 0, 7, 0]),
        ],
    );
    session.record(other_provider, vec![usage("kimi", "work", [9, 0, 9, 0])]);
    session.record(ThreadId::new(), vec![usage("zai", "work", [1, 1, 1, 1])]);

    assert_eq!(
        session.for_provider("zai", "default"),
        StatusAccountUsage {
            named_account_in_home: None,
            rows: vec![
                ("default".to_string(), tokens([100, 20, 10, 5])),
                ("work".to_string(), tokens([51, 1, 6, 1])),
            ],
        }
    );
    // The active account comes first, then the others by name.
    assert_eq!(
        session
            .for_provider("zai", "work")
            .rows
            .iter()
            .map(|(account, _)| account.as_str())
            .collect::<Vec<_>>(),
        vec!["work", "default"]
    );
    session.clear();
    assert_eq!(
        session.for_provider("zai", "work"),
        StatusAccountUsage {
            named_account_in_home: None,
            rows: vec![("work".to_string(), TokenUsage::default())],
        }
    );
}

#[test]
fn one_account_is_one_line() {
    let summary = StatusAccountUsage {
        named_account_in_home: None,
        rows: vec![("work".to_string(), tokens([40_100, 12_000, 6_015, 300]))],
    };
    assert_eq!(
        text(summary.lines()),
        vec!["work · 28,100 input · 12,000 cached input · 6,015 output (300 reasoning)"]
    );
    // Cached input and reasoning appear only when reported.
    let summary = StatusAccountUsage {
        named_account_in_home: None,
        rows: vec![("work".to_string(), tokens([700, 0, 32, 0]))],
    };
    assert_eq!(text(summary.lines()), vec!["work · 700 input · 32 output"]);
    let summary = StatusAccountUsage {
        named_account_in_home: None,
        rows: vec![("default".to_string(), TokenUsage::default())],
    };
    assert_eq!(text(summary.lines()), vec!["default · no usage yet"]);
}

#[test]
fn several_accounts_are_an_aligned_table_with_the_active_one_marked() {
    let summary = StatusAccountUsage {
        named_account_in_home: None,
        rows: vec![
            ("default".to_string(), tokens([40_100, 12_000, 6_015, 0])),
            ("work".to_string(), tokens([7_000, 0, 232, 0])),
        ],
    };
    assert_eq!(
        text(summary.lines()),
        vec![
            "account            input  cached input  output",
            "default (active)  28,100        12,000   6,015",
            "work               7,000             0     232",
        ]
    );
}
