//! Source-informed Issue822 command-cell regressions through native server notifications.
use super::*;

#[tokio::test]
async fn issue822_refused_command_preserves_reason_and_never_claims_execution() {
    for (name, command) in [("shell", "echo refused"), ("exploring", "cat refused.txt")] {
        let (mut chat, mut rx, _ops) = make_chatwidget_manual(/*model_override*/ None).await;
        chat.on_task_started();
        let mut item = begin_exec(&mut chat, "refused", command);
        let AppServerThreadItem::CommandExecution {
            status,
            aggregated_output,
            exit_code,
            duration_ms,
            ..
        } = &mut item
        else {
            panic!("expected command")
        };
        *status = serde_json::from_value(serde_json::json!("refused"))
            .expect("policy refusal must survive the app-server wire");
        *aggregated_output = Some("Synthetic policy refused escalation before approval".into());
        // The synthetic -1 sentinel must never be presented as an actual execution.
        *exit_code = Some(-1);
        *duration_ms = Some(0);
        handle_exec_end(&mut chat, item);
        let cell = std::iter::from_fn(|| rx.try_recv().ok())
            .find_map(|event| match event {
                AppEvent::InsertHistoryCell(cell) => Some(cell),
                _ => None,
            })
            .expect("refused command must flush to history");
        let display = cell.display_lines(/*width*/ 80);
        let transcript = cell.transcript_lines(/*width*/ 80);
        for lines in [&display, &transcript] {
            let text = lines_to_single_string(lines);
            // #873: neutral (a network-policy refusal can come after the
            // command ran), so it never says whether the command ran.
            assert!(!text.contains("Did not run"), "{text}");
            assert!(text.contains("Refused"), "{text}");
            assert!(text.contains("Synthetic policy refused"), "{text}");
            for false_claim in [
                "Approval declined",
                "Ran ",
                "Explored",
                "(no output)",
                "exited",
            ] {
                assert!(!text.contains(false_claim), "{text}");
            }
            assert!(!lines.iter().flat_map(|line| &line.spans).any(|span| {
                span.content.contains(['•', '✓', '✔'])
                    && span.style.fg == Some(ratatui::style::Color::Green)
            }));
        }
        // #873: one detail line carries the label and the reason.
        let display_text = lines_to_single_string(&display);
        assert_eq!(display_text.matches('└').count(), 1, "{display_text}");
        assert!(
            display_text.contains("  └ Refused: Synthetic policy refused"),
            "{display_text}"
        );
        insta::assert_snapshot!(
            format!("issue822_refused_{name}"),
            format!(
                "display:\n{}transcript:\n{}narrow:\n{}",
                lines_to_single_string(&display),
                lines_to_single_string(&transcript),
                lines_to_single_string(&cell.display_lines(/*width*/ 32)),
            )
        );
    }
}
