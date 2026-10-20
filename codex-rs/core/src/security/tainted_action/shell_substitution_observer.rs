//! Test-only observations of the real classifier's lexer calls.
//! No supplied command is executed. A fresh thread-local scope is used for
//! every input; no counter or graph changes production decisions.

use super::super::classify_with;
use super::Lexed;
use crate::sandboxing::SandboxPermissions;
use crate::tools::sandboxing::ApprovalAction;
use codex_utils_absolute_path::AbsolutePathBuf;
use codex_utils_path_uri::PathUri;
use serde_json::Value;
use serde_json::json;
use sha2::Digest;
use sha2::Sha256;
use std::cell::RefCell;
use std::path::Path;
use std::time::Duration;
use std::time::Instant;

#[derive(Clone, Copy)]
pub(super) enum Work {
    ParserCalls,
    ParserBytes,
    LexerChars,
    RangeVisits,
    MembershipProbes,
    FedEdgeVisits,
    FedAppendVisits,
}

struct Observation {
    counts: [u64; 7],
    graphs: Vec<Value>,
    operation_limit: u64,
    capture_graphs: bool,
}

impl Default for Observation {
    fn default() -> Self {
        Self {
            counts: [0; 7],
            graphs: Vec::new(),
            operation_limit: 2_000_000,
            capture_graphs: true,
        }
    }
}

thread_local! {
    static CURRENT: RefCell<Option<Observation>> = const { RefCell::new(None) };
}

pub(super) fn record(work: Work, amount: usize) {
    CURRENT.with(|slot| {
        if let Some(observation) = slot.borrow_mut().as_mut() {
            observation.counts[work as usize] += amount as u64;
            // A resource guard, not a performance-success threshold. A stop
            // reports charged work; the operation being charged may not run.
            let operations: u64 = observation.counts[2..].iter().sum();
            assert!(
                operations <= observation.operation_limit,
                "ISSUE1020_OPERATION_BUDGET_STOP limit={} counts={:?}",
                observation.operation_limit,
                observation.counts
            );
        }
    });
}

pub(super) fn record_graph(lexed: &Lexed) {
    CURRENT.with(|slot| {
        if let Some(observation) = slot.borrow_mut().as_mut() {
            if !observation.capture_graphs {
                return;
            }
            let commands: Vec<_> = lexed
                .commands
                .iter()
                .map(|command| {
                    json!({
                        "words": command.words, "pipeFrom": command.pipe_from,
                        "substituted": command.substituted, "fedBy": command.fed_by,
                        "readsOuterOutput": command.reads_outer_output,
                        "stdinFrom": command.stdin_from, "hereString": command.here_string,
                        "writesTo": command.writes_to,
                    })
                })
                .collect();
            observation.graphs.push(json!({
                "incomplete": lexed.incomplete, "commands": commands,
            }));
        }
    });
}

struct Reset;

impl Drop for Reset {
    fn drop(&mut self) {
        CURRENT.with(|slot| {
            let _ = slot.replace(None);
        });
    }
}

pub(super) fn observe_depth_2000(script: String) -> (Lexed, Duration, [u64; 7], u64) {
    // This supplementary diagnostic has exactly one fixed input. Its graph
    // has at most sum(0..=2000) edges; range collection, membership lookup,
    // edge registration and edge append each visit at most that many items.
    // The lexer visits at most one position per input byte. This separate
    // finite budget does not change the structural corpus's 2M default.
    let depth = 2_000_u64;
    assert_eq!(script, format!("echo $(( {}", "$(".repeat(depth as usize)));
    let edges = depth * (depth + 1) / 2;
    let operation_limit = 4 * edges + script.len() as u64;
    CURRENT.with(|slot| {
        assert!(
            slot.replace(Some(Observation {
                operation_limit,
                // The test inspects the returned graph directly. Avoid making
                // another quadratic JSON copy solely for the timing report.
                capture_graphs: false,
                ..Observation::default()
            }))
            .is_none()
        );
    });
    let _reset = Reset;
    let started = Instant::now();
    let lexed = super::simple_commands(&[script]);
    let elapsed = started.elapsed();
    let Some(observation) = CURRENT.with(RefCell::take) else {
        panic!("diagnostic observation scope must remain active");
    };
    assert_eq!(observation.counts[Work::ParserCalls as usize], 1);
    (lexed, elapsed, observation.counts, operation_limit)
}

#[expect(
    clippy::print_stdout,
    reason = "Emit bounded test observations for the external paired-source validator"
)]
pub(super) fn observe(id: &str, input: &str) -> [u64; 7] {
    assert!(input.len() <= 16_384);
    let input_sha256 = format!("{:x}", Sha256::digest(input.as_bytes()));
    println!(
        "ISSUE1020_INPUT {}",
        json!({
            "id": id, "input": input, "utf8Bytes": input.len(), "inputSha256": input_sha256,
        })
    );
    CURRENT.with(|slot| assert!(slot.replace(Some(Observation::default())).is_none()));
    let _reset = Reset;
    let (cwd, home, user_home) = if cfg!(windows) {
        (
            "C:\\work",
            "C:\\home\\fixture\\.corbanu",
            "C:\\home\\fixture",
        )
    } else {
        ("/work", "/home/fixture/.corbanu", "/home/fixture")
    };
    let Ok(cwd) = AbsolutePathBuf::from_absolute_path_checked(cwd) else {
        panic!("fixed absolute fixture path must be valid");
    };
    let result = classify_with(
        &ApprovalAction::Shell {
            id: "issue1020".into(),
            environment_id: "local".into(),
            command: vec!["bash".into(), "-lc".into(), input.into()],
            cwd: PathUri::from_abs_path(&cwd),
            sandbox_permissions: SandboxPermissions::UseDefault,
            additional_permissions: None,
            justification: None,
        },
        Path::new(home),
        Some(Path::new(user_home)),
    );
    let Some(observation) = CURRENT.with(RefCell::take) else {
        panic!("observation scope must remain active");
    };
    assert!(observation.counts[Work::ParserCalls as usize] > 0);
    println!(
        "ISSUE1020_OBSERVATION {}",
        json!({
            "id": id, "inputSha256": input_sha256, "utf8Bytes": input.len(),
            "classification": format!("{result:?}"), "counts": observation.counts,
            "graphs": observation.graphs,
        })
    );
    observation.counts
}
