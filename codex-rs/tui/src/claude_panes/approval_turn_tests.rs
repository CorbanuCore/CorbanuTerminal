//! #218 part 2: a contained turn's tool approvals, end to end against a fake
//! Claude Code (a shell script speaking the stream-json control protocol).

use std::sync::Mutex as StdMutex;

use pretty_assertions::assert_eq;

use super::*;
use crate::app_event::AppEvent;
use crate::app_event_sender::AppEventSender;
use crate::claude_panes::approval::ApprovalResponder;
use crate::claude_panes::approval::Denial;
use crate::claude_panes::execution::parse_claude_version;
use crate::claude_panes::execution::resolve_contained_claude;
use crate::claude_panes::execution::wait_for_decision;

const RESULT: &str = r#"{"type":"result","subtype":"success","is_error":false,"result":"done","session_id":"33333333-3333-4333-8333-333333333333"}"#;
const SECRET: &str = "bridge-secret-for-approvals";

/// One popup: tool, `tool_use_id`, details, and its answer.
type Asked = (String, Option<String>, String, ApprovalResponder);

/// What the person saw, and how many popups were closed for them.
#[derive(Default)]
struct Person {
    asked: StdMutex<Vec<Asked>>,
    settled: StdMutex<usize>,
}

impl Person {
    fn asked(&self) -> Vec<(String, Option<String>, String)> {
        self.asked
            .lock()
            .unwrap()
            .iter()
            .map(|(tool, id, details, _)| (tool.clone(), id.clone(), details.clone()))
            .collect()
    }

    fn responder(&self, index: usize) -> ApprovalResponder {
        self.asked.lock().unwrap()[index].3.clone()
    }
}

/// A person who answers each request with `answer(index)`; `None` leaves it
/// open.
fn person(
    answer: impl Fn(usize) -> Option<bool> + Send + 'static,
) -> (AppEventSender, Arc<Person>) {
    let person = Arc::new(Person::default());
    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
    let seen = Arc::clone(&person);
    tokio::spawn(async move {
        while let Some(event) = rx.recv().await {
            match event {
                AppEvent::ClaudePaneApprovalRequested(request) => {
                    let details = request
                        .details
                        .lines
                        .iter()
                        .map(ToString::to_string)
                        .collect::<Vec<_>>()
                        .join("\n");
                    let index = {
                        let mut asked = seen.asked.lock().unwrap();
                        asked.push((
                            request.tool_name.clone(),
                            request.tool_use_id.clone(),
                            details,
                            request.responder.clone(),
                        ));
                        asked.len() - 1
                    };
                    assert_eq!(request.pane_id, "claude-redaction-test");
                    if let Some(allow) = answer(index) {
                        request.responder.respond(allow);
                    }
                }
                AppEvent::ClaudePaneApprovalsSettled => *seen.settled.lock().unwrap() += 1,
                _ => {}
            }
        }
    });
    (AppEventSender::new(tx), person)
}

fn request(id: &str, tool: &str, command: &str) -> String {
    json!({
        "type": "control_request",
        "request_id": id,
        "request": {
            "subtype": "can_use_tool",
            "tool_name": tool,
            "tool_use_id": format!("toolu_{id}"),
            "input": { "command": command },
        },
    })
    .to_string()
}

/// `printf` lines for the fake Claude Code.
fn emit(lines: &[&str]) -> String {
    lines
        .iter()
        .map(|line| format!("printf '%s\\n' '{line}'\n"))
        .collect()
}

fn plan_for(dir: &tempfile::TempDir, script: String) -> ClaudeCommandPlan {
    let mut plan = bridge_redaction_plan(dir, script, SECRET);
    plan.stdin_prompt = Some("please".to_string());
    plan
}

async fn run(
    plan: ClaudeCommandPlan,
    cancel: CancellationToken,
    tx: Option<AppEventSender>,
) -> ClaudePaneTurnOutput {
    tokio::time::timeout(
        Duration::from_secs(20),
        run_claude_command_plan(plan, cancel, tx),
    )
    .await
    .expect("the turn ends")
    .expect("turn output")
}

fn read_json(dir: &tempfile::TempDir, name: &str) -> Value {
    serde_json::from_str(&std::fs::read_to_string(dir.path().join(name)).expect(name))
        .expect("json")
}

/// The prompt goes in on stdin, each `can_use_tool` request reaches a person
/// with its details (redacted), the answer goes back on stdin, and stdin
/// closes after the result. With nobody to ask, tools are denied.
#[tokio::test]
async fn contained_turn_asks_a_person_before_each_tool() {
    for (ask_a_person, expected) in [(true, "allow"), (false, "deny")] {
        let dir = tempfile::tempdir().expect("tempdir");
        let script = format!(
            r#"IFS= read -r first; printf '%s\n' "$first" > prompt.json
{}IFS= read -r answer; printf '%s\n' "$answer" > answer.json
{}IFS= read -r other; printf '%s\n' "$other" > other.json
{}cat > /dev/null; echo closed > closed.txt"#,
            emit(&[&request(
                "req-1",
                "Bash",
                &format!("touch approved.txt # {SECRET}")
            )]),
            emit(&[
                r#"{"type":"control_request","request_id":"req-2","request":{"subtype":"mcp_message","server_name":"x"}}"#
            ]),
            emit(&[RESULT]),
        );
        let (tx, person) = person(|_| Some(true));
        let output = run(
            plan_for(&dir, script),
            CancellationToken::new(),
            ask_a_person.then_some(tx),
        )
        .await;

        assert_eq!(output.status, ClaudePaneTurnStatus::Success, "{output:?}");
        assert_eq!(
            read_json(&dir, "prompt.json"),
            json!({ "type": "user", "message": { "role": "user", "content": "please" } })
        );
        let answer = read_json(&dir, "answer.json");
        assert_eq!(answer["response"]["request_id"], json!("req-1"));
        assert_eq!(answer["response"]["response"]["behavior"], json!(expected));
        assert_eq!(
            read_json(&dir, "other.json")["response"]["subtype"],
            json!("error")
        );
        assert_eq!(
            std::fs::read_to_string(dir.path().join("closed.txt")).expect("stdin closed"),
            "closed\n"
        );
        if ask_a_person {
            assert_eq!(
                person.asked(),
                vec![(
                    "Bash".to_string(),
                    Some("toolu_req-1".to_string()),
                    "command: touch approved.txt # [REDACTED_SECRET]".to_string()
                )]
            );
            // The allowed input is the redacted one the person saw.
            assert_eq!(
                answer["response"]["response"]["updatedInput"]["command"],
                json!("touch approved.txt # [REDACTED_SECRET]")
            );
        }
    }
}

/// Review finding 6: a repeated request id opens no popup and gets no
/// answer, so it cannot take over the pending request's answer.
#[tokio::test]
async fn repeated_request_ids_are_dropped() {
    let dir = tempfile::tempdir().expect("tempdir");
    let script = format!(
        r#"IFS= read -r prompt
{}IFS= read -r a; IFS= read -r b; printf '%s\n%s\n' "$a" "$b" > answers.jsonl
{}cat > /dev/null"#,
        emit(&[
            &request("req-1", "Bash", "touch one.txt"),
            &request("req-1", "Read", "rm -rf ."),
            &request("req-2", "Bash", "touch two.txt"),
        ]),
        emit(&[RESULT]),
    );
    let (tx, person) = person(|_| Some(true));
    let output = run(plan_for(&dir, script), CancellationToken::new(), Some(tx)).await;
    assert_eq!(output.status, ClaudePaneTurnStatus::Success, "{output:?}");
    let asked = person.asked();
    assert_eq!(
        asked
            .iter()
            .map(|(tool, id, _)| (tool.as_str(), id.as_deref()))
            .collect::<Vec<_>>(),
        vec![("Bash", Some("toolu_req-1")), ("Bash", Some("toolu_req-2"))]
    );
    let answers = std::fs::read_to_string(dir.path().join("answers.jsonl")).expect("answers");
    let ids = answers
        .lines()
        .map(|line| {
            serde_json::from_str::<Value>(line).expect("json")["response"]["request_id"].clone()
        })
        .collect::<Vec<_>>();
    assert_eq!(ids, vec![json!("req-1"), json!("req-2")]);
}

/// Review finding 5: a control request without an id ends the turn instead
/// of hanging it.
#[tokio::test]
async fn a_request_without_an_id_ends_the_turn() {
    let dir = tempfile::tempdir().expect("tempdir");
    let script = format!(
        "{}sleep 30",
        emit(&[
            r#"{"type":"control_request","request":{"subtype":"can_use_tool","tool_name":"Bash","input":{}}}"#
        ])
    );
    let (tx, person) = person(|_| Some(true));
    let output = run(plan_for(&dir, script), CancellationToken::new(), Some(tx)).await;
    assert_eq!(
        output.status,
        ClaudePaneTurnStatus::ProviderError,
        "{output:?}"
    );
    assert!(
        output
            .error_summary
            .as_deref()
            .is_some_and(|error| error.contains("without a request id")),
        "{output:?}"
    );
    assert!(person.asked().is_empty());
}

#[tokio::test]
async fn a_second_result_is_an_error() {
    let dir = tempfile::tempdir().expect("tempdir");
    let script = format!("{}sleep 30", emit(&[RESULT, RESULT]));
    let output = run(plan_for(&dir, script), CancellationToken::new(), None).await;
    assert_eq!(
        output.status,
        ClaudePaneTurnStatus::ProviderError,
        "{output:?}"
    );
    assert!(
        output
            .error_summary
            .as_deref()
            .is_some_and(|error| error.contains("second result")),
        "{output:?}"
    );
}

/// Review finding 5: an interrupt with a popup open settles the request, so
/// its popup closes, and sends no answer.
#[tokio::test]
async fn interrupting_a_turn_closes_its_open_popups() {
    let dir = tempfile::tempdir().expect("tempdir");
    let script = format!(
        "IFS= read -r prompt\n{}cat > rest.jsonl",
        emit(&[&request("req-1", "Bash", "touch x")])
    );
    let (tx, person) = person(|_| None);
    let cancel = CancellationToken::new();
    let turn = tokio::spawn(run(plan_for(&dir, script), cancel.clone(), Some(tx)));
    for _ in 0..200 {
        if !person.asked().is_empty() {
            break;
        }
        tokio::time::sleep(Duration::from_millis(25)).await;
    }
    assert_eq!(person.asked().len(), 1);
    assert!(!person.responder(0).is_settled());
    cancel.cancel();
    let output = turn.await.expect("turn task");
    assert_eq!(
        output.status,
        ClaudePaneTurnStatus::Interrupted,
        "{output:?}"
    );
    for _ in 0..200 {
        if *person.settled.lock().unwrap() > 0 {
            break;
        }
        tokio::time::sleep(Duration::from_millis(25)).await;
    }
    assert!(person.responder(0).is_settled());
    assert_eq!(*person.settled.lock().unwrap(), 1);
}

/// Review finding 5: `control_cancel_request` withdraws a request: its popup
/// closes and it gets no answer.
#[tokio::test]
async fn claude_code_can_withdraw_a_request() {
    let dir = tempfile::tempdir().expect("tempdir");
    let script = format!(
        r#"IFS= read -r prompt
{}IFS= read -r answer; printf '%s\n' "$answer" > answer.json
{}cat > rest.jsonl"#,
        emit(&[
            &request("req-1", "Bash", "touch x"),
            r#"{"type":"control_cancel_request","request_id":"req-1"}"#,
            &request("req-2", "Bash", "touch y"),
        ]),
        emit(&[RESULT]),
    );
    // The second request is answered only once the first is settled.
    let (tx, person) = person(|_| None);
    let answering = Arc::clone(&person);
    let answer_second = tokio::spawn(async move {
        loop {
            let asked = answering.asked.lock().unwrap().len();
            if asked == 2 && answering.responder(0).is_settled() {
                answering.responder(1).respond(/*allow*/ false);
                return;
            }
            tokio::time::sleep(Duration::from_millis(25)).await;
        }
    });
    let output = run(plan_for(&dir, script), CancellationToken::new(), Some(tx)).await;
    tokio::time::timeout(Duration::from_secs(5), answer_second)
        .await
        .expect("first request settled")
        .expect("task");
    assert_eq!(output.status, ClaudePaneTurnStatus::Success, "{output:?}");
    assert_eq!(
        read_json(&dir, "answer.json")["response"]["request_id"],
        json!("req-2")
    );
    // Nothing was ever sent for the withdrawn request.
    assert_eq!(
        std::fs::read_to_string(dir.path().join("rest.jsonl")).expect("rest"),
        ""
    );
}

/// Claude Code that exits without reading its prompt ends the turn; writing
/// the prompt fails quietly.
#[tokio::test]
async fn claude_exiting_before_reading_the_prompt_ends_the_turn() {
    let dir = tempfile::tempdir().expect("tempdir");
    let mut plan = plan_for(&dir, "exit 3".to_string());
    plan.stdin_prompt = Some("p".repeat(1 << 20));
    let output = run(plan, CancellationToken::new(), None).await;
    assert_ne!(output.status, ClaudePaneTurnStatus::Success, "{output:?}");
}

/// A prompt far larger than a pipe buffer reaches Claude Code whole, while
/// its stdout is read at the same time.
#[tokio::test]
async fn prompts_larger_than_a_pipe_buffer_arrive_whole() {
    let dir = tempfile::tempdir().expect("tempdir");
    let script = format!(
        "{}IFS= read -r first; printf '%s\\n' \"$first\" > prompt.json\n{}cat > /dev/null",
        // Output first, so both pipes are busy at once.
        emit(&[
            r#"{"type":"system","subtype":"init","session_id":"33333333-3333-4333-8333-333333333333"}"#
        ]),
        emit(&[RESULT]),
    );
    let mut plan = plan_for(&dir, script);
    let prompt = "q".repeat(300 * 1024);
    plan.stdin_prompt = Some(prompt.clone());
    let output = run(plan, CancellationToken::new(), None).await;
    assert_eq!(output.status, ClaudePaneTurnStatus::Success, "{output:?}");
    assert_eq!(
        read_json(&dir, "prompt.json")["message"]["content"],
        json!(prompt)
    );
}

#[tokio::test]
async fn unanswered_requests_time_out_as_denials() {
    let cancel = CancellationToken::new();
    let (_responder, decision) = ApprovalResponder::new();
    assert_eq!(
        wait_for_decision(decision, &cancel, Duration::from_millis(20)).await,
        Some(Err(Denial::TimedOut))
    );
    let (responder, decision) = ApprovalResponder::new();
    drop(responder);
    assert_eq!(
        wait_for_decision(decision, &cancel, Duration::from_secs(60)).await,
        Some(Err(Denial::Person))
    );
    let (_responder, decision) = ApprovalResponder::new();
    cancel.cancel();
    assert_eq!(
        wait_for_decision(decision, &cancel, Duration::from_secs(60)).await,
        None
    );
}

#[test]
fn claude_versions_parse() {
    assert_eq!(
        parse_claude_version("2.1.292 (Claude Code)\n"),
        Some((2, 1, 292))
    );
    assert_eq!(
        parse_claude_version("2.1.37 (Claude Code)"),
        Some((2, 1, 37))
    );
    assert!(parse_claude_version("2.1.37 (Claude Code)") < Some((2, 1, 292)));
    assert_eq!(parse_claude_version("claude"), None);
    assert_eq!(parse_claude_version("2.1"), None);
    assert_eq!(parse_claude_version(""), None);
}

/// Review round 2, finding 3: the Claude Code that is checked and launched is
/// one file, never inside a folder the pane can write.
#[test]
fn contained_claude_is_never_taken_from_a_writable_folder() {
    let root = tempfile::tempdir().expect("tempdir");
    let bin = root.path().join("bin");
    let pane = root.path().join("pane");
    std::fs::create_dir_all(&bin).expect("bin");
    std::fs::create_dir_all(&pane).expect("pane");
    let install = |dir: &std::path::Path| {
        let path = dir.join("claude");
        std::fs::write(&path, "#!/bin/sh\necho 2.1.292\n").expect("write");
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o755)).expect("chmod");
        path
    };
    let installed = install(&bin);
    let resolved = resolve_contained_claude(installed.to_str().expect("utf-8"), &[&pane])
        .expect("an installed claude");
    assert_eq!(
        resolved,
        std::fs::canonicalize(&installed).expect("canonical")
    );

    let planted = install(&pane);
    let err = resolve_contained_claude(planted.to_str().expect("utf-8"), &[&pane])
        .expect_err("planted in the pane folder");
    assert!(
        err.to_string().contains("which the pane can write"),
        "{err}"
    );

    // A symlink out of a safe folder into the pane folder is refused too.
    let link = bin.join("claude-link");
    std::os::unix::fs::symlink(&planted, &link).expect("symlink");
    assert!(resolve_contained_claude(link.to_str().expect("utf-8"), &[&pane]).is_err());

    // Not executable, or a relative path: refused.
    let plain = bin.join("claude-plain");
    std::fs::write(&plain, "x").expect("write");
    assert!(resolve_contained_claude(plain.to_str().expect("utf-8"), &[&pane]).is_err());
    assert!(resolve_contained_claude("./claude", &[&pane]).is_err());
}
