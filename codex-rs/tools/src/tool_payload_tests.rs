use super::ToolPayload;
use pretty_assertions::assert_eq;

fn function(arguments: &str) -> ToolPayload {
    ToolPayload::Function {
        arguments: arguments.to_string(),
    }
}

#[test]
fn freeform_input_reads_custom_payloads_verbatim() {
    let payload = ToolPayload::Custom {
        input: "// @exec: {\"yield_time_ms\": 10}\ntext(1)".to_string(),
    };

    assert_eq!(
        payload.freeform_input().as_deref(),
        Some("// @exec: {\"yield_time_ms\": 10}\ntext(1)")
    );
}

#[test]
fn freeform_input_reads_function_wrapped_inputs() {
    assert_eq!(
        function(r#"{"input":"const x = 1;\ntext(x)"}"#)
            .freeform_input()
            .as_deref(),
        Some("const x = 1;\ntext(x)")
    );
    assert_eq!(
        function(r#""*** Begin Patch\n*** End Patch""#)
            .freeform_input()
            .as_deref(),
        Some("*** Begin Patch\n*** End Patch")
    );
}

#[test]
fn freeform_input_rejects_function_arguments_without_string_input() {
    for arguments in [
        "",
        "not json",
        "{}",
        r#"{"code":"text(1)"}"#,
        r#"{"input":42}"#,
        r#"["text(1)"]"#,
    ] {
        assert_eq!(function(arguments).freeform_input(), None, "{arguments}");
    }
}
