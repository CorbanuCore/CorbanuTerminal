use super::*;
use pretty_assertions::assert_eq;

fn key(tool: &str) -> McpToolApprovalKey {
    McpToolApprovalKey {
        server: "docs".to_string(),
        connector_id: None,
        tool_name: tool.to_string(),
    }
}

#[tokio::test]
async fn issue_1001_recorded_tools_are_approved_and_others_not() {
    let home = tempfile::tempdir().expect("home");
    assert!(!approved(home.path(), &key("search")).await, "no record");
    record(home.path(), key("search")).await.expect("recorded");
    record(home.path(), key("search"))
        .await
        .expect("idempotent");
    assert!(approved(home.path(), &key("search")).await);
    assert!(!approved(home.path(), &key("write")).await);
    let text = std::fs::read_to_string(home.path().join(FILE)).expect("file");
    assert_eq!(text.matches("\"search\"").count(), 1, "{text}");

    // An unreadable record approves nothing, and is not replaced.
    std::fs::write(home.path().join(FILE), "not json").expect("write");
    assert!(!approved(home.path(), &key("search")).await);
    assert!(record(home.path(), key("write")).await.is_err());
    assert_eq!(
        std::fs::read_to_string(home.path().join(FILE)).expect("file"),
        "not json"
    );
}
