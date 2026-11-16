//! Warning identities survive replay and preserve complete transcript details.

use super::*;
use pretty_assertions::assert_eq;

#[test]
fn usage_warnings_render_in_both_transcripts_including_composites() {
    let message = "Only 8% of your 5h limit remains. Run /status for details.";
    let usage = new_usage_warning_event(message.into());
    let diagnostic = new_warning_event(message.into());
    let composite = CompositeHistoryCell::new(vec![
        Box::new(PlainHistoryCell::new(vec!["Earlier output".into()])),
        Box::new(new_warning_event("Hidden diagnostic".into())),
        Box::new(new_usage_warning_event(message.into())),
    ]);
    assert_eq!(usage.warning_entries(), diagnostic.warning_entries());
    for mode in [HistoryRenderMode::Rich, HistoryRenderMode::Raw] {
        let visible = usage.display_hyperlink_lines_for_mode(/*width*/ 28, mode);
        let expected = match mode {
            HistoryRenderMode::Rich => usage.transcript_hyperlink_lines(/*width*/ 28),
            HistoryRenderMode::Raw => plain_hyperlink_lines(usage.raw_lines()),
        };
        assert_eq!(visible, expected);
        if matches!(mode, HistoryRenderMode::Rich) {
            assert_eq!(usage.compact_hyperlink_lines(/*width*/ 28), visible);
        }
        assert!(
            diagnostic
                .display_hyperlink_lines_for_mode(/*width*/ 28, mode)
                .is_empty()
        );
        assert_eq!(
            composite.display_hyperlink_lines_for_mode(/*width*/ 28, mode),
            [
                vec![
                    HyperlinkLine::from("Earlier output"),
                    HyperlinkLine::from("")
                ],
                visible
            ]
            .concat(),
        );
    }
}

#[test]
fn security_and_account_warnings_keep_chat_footer_and_viewer_details() {
    let messages = [
        crate::legacy_core::security_revocation::KILL_SWITCH_STOPPED.to_string(),
        "`[provider_accounts]` is ignored because the `named_accounts` feature is off".to_string(),
        "Security level Moderate blocked writing `.git/hooks/pre-commit`.".to_string(),
        "Security level Moderate: your Corbanu home is reached through a link.".to_string(),
        "Security level Moderate needs the source_envelopes feature.".to_string(),
        "Moderate needs the untrusted-content labels (the source_envelopes feature).".to_string(),
        "No provider is allowed for spawned agents: project restrictions leave the allowlist empty.".to_string(),
        codex_sandboxing::LEGACY_WINDOWS_SANDBOX_WARNING.to_string(),
        codex_login::OPENAI_API_KEY_ENV_FALLBACK_NOTICE.to_string(),
        "OpenAI (openai) credential was rejected. Open /providers.".to_string(),
        codex_model_provider_info::named_account_rejection_message(
            "OpenAI", "openai", "work", /*provider*/ None,
        ),
    ];
    for message in messages {
        let warning = new_warning_event(message.clone());
        assert!(warning.visible_in_transcript, "{message}");
        let startup =
            StartupWarningsCell::new(vec!["Ordinary startup diagnostic".into(), message.clone()]);
        for mode in [HistoryRenderMode::Rich, HistoryRenderMode::Raw] {
            let visible = warning.display_lines_for_mode(/*width*/ 80, mode);
            assert!(!visible.is_empty(), "{message}");
            assert_eq!(startup.display_lines_for_mode(/*width*/ 80, mode), visible);
        }
        let cells: Vec<Arc<dyn HistoryCell>> = vec![Arc::new(warning), Arc::new(startup)];
        assert_eq!(warning_count(&cells), 2);
        let entries = warning_entries(&cells);
        assert_eq!(entries[0].details, message);
        assert_eq!(entries[1].details, "Ordinary startup diagnostic");
        assert!(!cells[1].transcript_lines(/*width*/ 80).is_empty());
    }
}

#[test]
fn warning_count_deduplicates_messages_mcp_summaries_and_composites() {
    let cells: Vec<Arc<dyn HistoryCell>> = vec![
        Arc::new(new_warning_event("Repeated warning".into())),
        Arc::new(StartupWarningsCell::new(vec!["Repeated warning".into()])),
        Arc::new(StartupWarningsCell::mcp(
            vec!["MCP alpha failed".into()],
            ["alpha".into()],
            /*failure_reason*/ None,
        )),
        Arc::new(StartupWarningsCell::mcp(
            vec!["MCP startup incomplete".into()],
            ["alpha".into()],
            /*failure_reason*/ None,
        )),
        Arc::new(CompositeHistoryCell::new(vec![Box::new(
            new_warning_event("Another warning".into()),
        )])),
    ];
    assert_eq!(warning_count(&cells), 3);
    assert_eq!(
        warning_entries(&cells),
        vec![
            WarningEntry {
                id: WarningId::Message("Repeated warning".into()),
                source: "Warning".into(),
                details: "Repeated warning".into()
            },
            WarningEntry {
                id: WarningId::McpServer("alpha".into()),
                source: "MCP · alpha".into(),
                details: "MCP alpha failed\n\nMCP startup incomplete".into()
            },
            WarningEntry {
                id: WarningId::Message("Another warning".into()),
                source: "Warning".into(),
                details: "Another warning".into()
            },
        ]
    );
    let replay = [cells.clone(), cells.clone()].concat();
    assert_eq!(warning_count(&replay), 3);
    assert_eq!(warning_entries(&replay), warning_entries(&cells));
    assert_eq!(warning_count(&[]), 0);
    assert!(
        cells
            .iter()
            .all(|cell| cell.compact_hyperlink_lines(/*width*/ 40).is_empty())
    );
    assert!(cells.iter().all(|cell| {
        cell.display_lines_for_mode(/*width*/ 40, HistoryRenderMode::Raw)
            .is_empty()
    }));
    assert!(cells.iter().all(|cell| !cell.raw_lines().is_empty()));
}
