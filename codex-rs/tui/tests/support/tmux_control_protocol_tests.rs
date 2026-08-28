use pretty_assertions::assert_eq;

use super::*;

fn parse(parser: &mut ControlParser, line: &[u8]) -> Option<ParsedItem> {
    parser.parse_line(line).expect("control line should parse")
}

#[test]
fn command_blocks_are_correlated_into_complete_results() {
    let mut parser = ControlParser::default();
    assert_eq!(parse(&mut parser, b"%begin 100 7 1"), None);
    assert_eq!(parse(&mut parser, b"first line"), None);
    assert_eq!(parse(&mut parser, b"second line"), None);
    assert_eq!(
        parse(&mut parser, b"%end 100 7 1"),
        Some(ParsedItem {
            retained_bytes: 47,
            value: ControlItem::Command(CommandResult {
                time: 100,
                number: 7,
                flags: 1,
                status: CommandStatus::Success,
                output: vec![b"first line".to_vec(), b"second line".to_vec()],
            }),
        })
    );

    assert_eq!(parse(&mut parser, b"%begin 101 8 0"), None);
    assert_eq!(parse(&mut parser, b"bad target"), None);
    assert_eq!(
        parse(&mut parser, b"%error 101 8 0"),
        Some(ParsedItem {
            retained_bytes: 38,
            value: ControlItem::Command(CommandResult {
                time: 101,
                number: 8,
                flags: 0,
                status: CommandStatus::Error,
                output: vec![b"bad target".to_vec()],
            }),
        })
    );
}

#[test]
fn required_notifications_parse_as_typed_events() {
    let mut parser = ControlParser::default();
    let fixtures = [
        (
            b"%layout-change @1 layout visible flags".as_slice(),
            ControlEvent::LayoutChange {
                window_id: "@1".into(),
                layout: "layout".into(),
                visible_layout: "visible".into(),
                flags: "flags".into(),
            },
        ),
        (
            b"%window-pane-changed @1 %2".as_slice(),
            ControlEvent::WindowPaneChanged {
                window_id: "@1".into(),
                pane_id: "%2".into(),
            },
        ),
        (
            b"%pane-mode-changed %2".as_slice(),
            ControlEvent::PaneModeChanged {
                pane_id: "%2".into(),
            },
        ),
        (
            b"%pause %2".as_slice(),
            ControlEvent::Pause {
                pane_id: "%2".into(),
            },
        ),
        (
            b"%continue %2".as_slice(),
            ControlEvent::Continue {
                pane_id: "%2".into(),
            },
        ),
        (
            b"%exit detached".as_slice(),
            ControlEvent::Exit {
                reason: b"detached".to_vec(),
            },
        ),
    ];

    for (line, expected) in fixtures {
        assert_eq!(
            parse(&mut parser, line).map(|item| item.value),
            Some(ControlItem::Event(expected))
        );
    }
}

#[test]
fn pane_output_decodes_octal_and_preserves_future_fields() {
    let mut parser = ControlParser::default();
    assert_eq!(
        parse(&mut parser, b"%output %3 hello\\040world\\015\\012").map(|item| item.value),
        Some(ControlItem::Event(ControlEvent::Output {
            pane_id: "%3".into(),
            data: b"hello world\r\n".to_vec(),
        }))
    );
    assert_eq!(
        parse(
            &mut parser,
            b"%extended-output %3 42 future-a future-b : chunk\\134tail"
        )
        .map(|item| item.value),
        Some(ControlItem::Event(ControlEvent::ExtendedOutput {
            pane_id: "%3".into(),
            age_millis: 42,
            future: vec!["future-a".into(), "future-b".into()],
            data: b"chunk\\tail".to_vec(),
        }))
    );
}

#[test]
fn unknown_notifications_are_preserved() {
    let mut parser = ControlParser::default();
    assert_eq!(
        parse(&mut parser, b"%future-notice value with spaces").map(|item| item.value),
        Some(ControlItem::Event(ControlEvent::Unknown {
            name: "%future-notice".into(),
            arguments: b"value with spaces".to_vec(),
        }))
    );
}

#[test]
fn malformed_framing_and_escapes_return_structured_errors() {
    let mut parser = ControlParser::default();
    assert_eq!(
        parser
            .parse_line(b"orphan output")
            .expect_err("orphan output should fail")
            .to_string(),
        "unexpected tmux control output outside a command block"
    );
    assert_eq!(
        parser
            .parse_line(b"%end 100 7 1")
            .expect_err("stray end should fail")
            .to_string(),
        "tmux command boundary had no matching %begin marker"
    );

    let mut parser = ControlParser::default();
    assert_eq!(parse(&mut parser, b"%begin 100 7 1"), None);
    assert_eq!(
        parser
            .parse_line(b"%begin 101 8 1")
            .expect_err("nested begin should fail")
            .to_string(),
        "tmux command block contained a nested %begin marker"
    );

    let mut parser = ControlParser::default();
    assert_eq!(parse(&mut parser, b"%begin 100 7 1"), None);
    assert_eq!(
        parser
            .parse_line(b"%end 100 8 1")
            .expect_err("mismatched marker should fail")
            .to_string(),
        "tmux command boundary did not match its %begin marker"
    );

    let mut parser = ControlParser::default();
    assert_eq!(
        parser
            .parse_line(b"%output %1 bad\\09x")
            .expect_err("invalid octal should fail")
            .to_string(),
        "invalid tmux octal escape"
    );

    let mut parser = ControlParser::default();
    assert_eq!(parse(&mut parser, b"%begin 100 7 1"), None);
    assert_eq!(
        parser
            .finish()
            .expect_err("unfinished command should fail at EOF")
            .to_string(),
        "tmux control stream ended inside a command block"
    );
}

#[test]
fn line_and_command_output_limits_fail_at_the_boundary() {
    let mut parser = ControlParser::default();
    let oversized = vec![b'x'; MAX_CONTROL_LINE_BYTES + 1];
    assert_eq!(
        parser
            .parse_line(&oversized)
            .expect_err("oversized line should fail")
            .to_string(),
        format!("tmux control line exceeded {MAX_CONTROL_LINE_BYTES} bytes")
    );

    assert_eq!(parse(&mut parser, b"%begin 1 1 0"), None);
    for _ in 0..MAX_CONTROL_EVENTS {
        assert_eq!(parse(&mut parser, b"x"), None);
    }
    assert_eq!(
        parser
            .parse_line(b"x")
            .expect_err("too many command lines should fail")
            .to_string(),
        format!("tmux command output exceeded {MAX_CONTROL_EVENTS} lines")
    );

    let mut parser = ControlParser::default();
    assert_eq!(parse(&mut parser, b"%begin 1 1 0"), None);
    let chunk = vec![b'x'; MAX_CONTROL_LINE_BYTES];
    for _ in 0..15 {
        assert_eq!(parse(&mut parser, &chunk), None);
    }
    let remaining = MAX_CONTROL_BACKLOG_BYTES - b"%begin 1 1 0".len() - 15 * chunk.len();
    assert_eq!(parse(&mut parser, &vec![b'x'; remaining]), None);
    assert_eq!(
        parser
            .parse_line(b"%end 1 1 0")
            .expect_err("closing marker beyond byte limit should fail")
            .to_string(),
        format!("tmux command output exceeded {MAX_CONTROL_BACKLOG_BYTES} bytes")
    );
}
