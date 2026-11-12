//! #1974: a post-taint assignment that may not run, or that runs only in
//! a subshell, must not count as resolving the variable. The word lexer
//! records every assignment it reads (`r=origin`) into the classifier's
//! variables, so a later `git fetch e"$r"` reads as a resolved remote and
//! runs without a prompt, while the variable can still hold anything. In
//! its own file: `tainted_action_tests.rs` is close to the repository's
//! blob size limit.

use super::*;
use crate::sandboxing::SandboxPermissions;
use codex_utils_absolute_path::AbsolutePathBuf;
use pretty_assertions::assert_eq;
use std::path::PathBuf;

/// The work counter with tracing silenced while it counts, as in
/// `tainted_action_linear_tests.rs`.
mod work {
    pub(super) fn assert_linear(label: &str, run: impl Fn(usize)) {
        tracing::subscriber::with_default(tracing::subscriber::NoSubscriber::default(), || {
            super::super::work::assert_linear(label, run)
        });
    }
}

const HOME: &str = "/home/fixture/.corbanu";
const USER_HOME: &str = "/home/fixture";

/// A fixture path, on drive `C:` on Windows (#1240).
fn fixture(path: &str) -> PathBuf {
    if cfg!(windows) && !Path::new(path).is_absolute() {
        PathBuf::from(format!("C:{}", path.replace('/', "\\")))
    } else {
        PathBuf::from(path)
    }
}

/// The verdict on `bash -lc <line>` in `/work`.
fn script(line: &str) -> Option<ProtectedActionKind> {
    let action = ApprovalAction::Shell {
        id: "call".into(),
        environment_id: "local".into(),
        command: ["bash", "-lc", line].map(str::to_string).to_vec(),
        cwd: PathUri::from_abs_path(
            &AbsolutePathBuf::from_absolute_path_checked(fixture("/work")).expect("absolute"),
        ),
        sandbox_permissions: SandboxPermissions::UseDefault,
        additional_permissions: None,
        justification: None,
    };
    classify_with(&action, &fixture(HOME), Some(&fixture(USER_HOME)))
}

/// Lines whose verdict is not `expected`.
fn wrong(lines: &[&str], expected: Option<ProtectedActionKind>) -> Vec<String> {
    lines
        .iter()
        .filter(|line| script(line) != expected)
        .map(|line| format!("{line} -> {:?}", script(line)))
        .collect()
}

/// The issue's four bypasses: the assignment runs only when the condition
/// allows it (`||` / `&&`), or only inside a subshell (`(...)`), so the
/// remote `e"$r"` can still be anything. With git 2.43 and `ext` allowed,
/// each of these runs an `ext::` remote's command.
#[test]
fn issue_1974_conditional_and_subshell_assignments_ask() {
    let lines = [
        "[ -n \"$r\" ] || r=origin; git fetch e\"$r\"",
        "true || r=origin; git fetch e\"$r\"",
        "false && r=origin; git fetch e\"$r\"",
        "(r=origin); git fetch e\"$r\"",
    ];
    assert_eq!(
        wrong(&lines, Some(ProtectedActionKind::UnseenCode)),
        Vec::<String>::new()
    );
}

/// The related forms from the issue: a conditional body (`if`/`while`,
/// one-line and newline forms), a backgrounded group (`{ } &`, whose
/// assignments run in a subshell), a backgrounded assignment on its own,
/// and a command substitution (`$(...)`, whose assignments never reach
/// the parent shell).
#[test]
fn issue_1974_related_conditional_forms_ask() {
    let lines = [
        "if true; then r=origin; fi; git fetch e\"$r\"",
        "if true; then\nr=origin\nfi; git fetch e\"$r\"",
        "if false; then r=origin; else r=other; fi; git fetch e\"$r\"",
        "while false; do r=origin; done; git fetch e\"$r\"",
        "for i in; do r=origin; done; git fetch e\"$r\"",
        "{ r=origin; } & git fetch e\"$r\"",
        "r=origin & git fetch e\"$r\"",
        "$(r=origin); git fetch e\"$r\"",
        "x=$(r=origin); git fetch e\"$r\"",
    ];
    assert_eq!(
        wrong(&lines, Some(ProtectedActionKind::UnseenCode)),
        Vec::<String>::new()
    );
}

/// The controls that must stay quiet: an assignment that always runs in
/// the current shell (`r=origin`, a brace group, `export`, an assignment
/// before `&&`), and an assignment that runs as the `if` condition itself
/// (it runs even when the branch does not).
#[test]
fn issue_1974_plain_assignment_control_stays_quiet() {
    let lines = [
        "r=origin; git fetch e\"$r\"",
        "{ r=origin; }; git fetch e\"$r\"",
        "export r=origin; git fetch e\"$r\"",
        "r=origin && git fetch e\"$r\"",
        "r=origin; git fetch \"$r\"",
        "if r=origin; then git fetch e\"$r\"; fi",
    ];
    assert_eq!(wrong(&lines, /*expected*/ None), Vec::<String>::new());
}

#[test]
fn review_assignment_contexts_never_resolve_an_unseen_remote() {
    let lines = [
        "r=origin | cat; git fetch e\"$r\"",
        "f() { r=origin; }; git fetch e\"$r\"",
        "function f { r=origin; }; git fetch e\"$r\"",
        "false && { r=origin; }; git fetch e\"$r\"",
        "false &&\nr=origin; git fetch e\"$r\"",
        "r=origin && true & git fetch e\"$r\"",
        "r=origin && tmp=$(echo x) & git fetch e\"$r\"",
        "false && # continuation\nr=origin; git fetch e\"$r\"",
        "false && # ; r=origin\n{ r=origin; }; git fetch e\"$r\"",
        "r=origin && { true; } >log & git fetch e\"$r\"",
        "if r=origin; then :; fi | cat; git fetch e\"$r\"",
        "while r=origin; do break; done | cat; git fetch e\"$r\"",
        "r=origin && if true; then :; fi >log & git fetch e\"$r\"",
        "false && r=$(echo origin); git fetch e\"$r\"",
        "{ r=origin; } | cat; git fetch e\"$r\"",
        "true || { r=origin; }; git fetch e\"$r\"",
        "false ||\nr=origin; git fetch e\"$r\"",
    ];
    assert_eq!(
        wrong(&lines, Some(ProtectedActionKind::UnseenCode)),
        Vec::<String>::new()
    );
}

#[test]
fn review_possible_literals_keep_protected_reads_visible() {
    for (line, baseline) in [
        (
            "true && a=.a; b=ws; cat ~/$a$b/credentials",
            "a=.a; b=ws; cat ~/$a$b/credentials",
        ),
        (
            "true && a=.co; b=dex; cat ~/$a$b/config.toml",
            "a=.co; b=dex; cat ~/$a$b/config.toml",
        ),
    ] {
        let expected = script(baseline).expect("baseline protects this path");
        assert_eq!(script(line), Some(expected), "{line}");
    }
}

#[test]
fn review_nested_background_group_scan_is_linear() {
    work::assert_linear("nested background groups", |scale| {
        let n = 200 * scale;
        let line = format!(
            "{}true; {}git fetch e\"$r\"",
            "{ r=origin; ".repeat(n),
            "} & ".repeat(n)
        );
        let _ = shell::simple_commands(&[line]);
    });
}

#[test]
fn issue_1974_conditional_assignment_scan_is_linear() {
    for (label, word, line) in [
        (
            "conditional assignments before the remote",
            "false && r=origin; ",
            "{}git fetch e\"$r\"",
        ),
        (
            "subshell assignments before the remote",
            "(r=origin); ",
            "{}git fetch e\"$r\"",
        ),
    ] {
        work::assert_linear(label, |scale| {
            let _ = script(&line.replace("{}", &word.repeat(200 * scale)));
        });
    }
}
