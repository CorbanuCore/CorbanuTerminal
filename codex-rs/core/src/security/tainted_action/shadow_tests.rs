use super::*;
use crate::sandboxing::SandboxPermissions;
use crate::security::tainted_action::ProtectedActionKind;
use crate::security::tainted_action::classify_with;
use crate::tools::sandboxing::ApprovalAction;
use codex_utils_absolute_path::AbsolutePathBuf;
use codex_utils_path_uri::PathUri;
use pretty_assertions::assert_eq;
use std::cell::RefCell;

thread_local! {
    /// Comparisons made on this thread, for these tests.
    pub(super) static RECORDED: RefCell<Vec<Comparison>> = const { RefCell::new(Vec::new()) };
}

/// A fixture path; on Windows (#1240) on drive `C:` (`/work` is `C:\work`).
fn fixture(path: &str) -> std::path::PathBuf {
    if cfg!(windows) {
        format!("C:{}", path.replace('/', "\\")).into()
    } else {
        path.into()
    }
}

/// The classification of `bash -lc script`, and the comparisons it made.
fn classified(script: &str) -> (Option<ProtectedActionKind>, Vec<Comparison>) {
    RECORDED.with(|recorded| recorded.borrow_mut().clear());
    let action = ApprovalAction::Shell {
        id: "call".into(),
        environment_id: "local".into(),
        command: vec!["bash".into(), "-lc".into(), script.into()],
        cwd: PathUri::from_abs_path(
            &AbsolutePathBuf::from_absolute_path_checked(fixture("/work")).expect("absolute"),
        ),
        sandbox_permissions: SandboxPermissions::UseDefault,
        additional_permissions: None,
        justification: None,
    };
    let kind = classify_with(
        &action,
        &fixture("/home/fixture/.corbanu"),
        Some(&fixture("/home/fixture")),
    );
    (kind, RECORDED.with(RefCell::take))
}

#[test]
fn every_command_line_is_compared_and_nothing_changes() {
    assert_eq!(
        classified("git status && echo 'a; b'"),
        (None, vec![Comparison::Agree])
    );
    assert_eq!(
        classified("corbanu vault list"),
        (Some(ProtectedActionKind::Vault), vec![Comparison::Agree])
    );
    // A nested shell's script is a command line of its own.
    let (_, comparisons) = classified("bash -c 'git log' && ls");
    assert_eq!(comparisons, vec![Comparison::Agree, Comparison::Agree]);
}

#[test]
fn differences_are_recorded_by_kind() {
    // The reader reads arithmetic as commands on purpose (#937).
    assert_eq!(
        classified("v=1; echo $((v))").1,
        vec![Comparison::ReaderFindsMore]
    );
    assert_eq!(
        classified("echo a\\\nb").1,
        vec![Comparison::GrammarRefused(Refusal::LineContinuation)]
    );
}

#[test]
fn the_budget_bounds_the_lines_compared() {
    let reading = quoted::commands("ls");
    let mut budget = LINE_COST + 3;
    assert_eq!(compare("ls", &reading, &mut budget).0, Comparison::Agree);
    assert_eq!(budget, 1);
    assert_eq!(
        compare("ls", &reading, &mut budget).0,
        Comparison::NotCompared
    );
    assert_eq!(budget, 1);
    // A line too long to read costs nothing.
    let long = "a".repeat(grammar::MAX_BYTES + 1);
    let mut budget = LINE_COST;
    assert_eq!(
        compare(&long, &quoted::commands(&long), &mut budget).0,
        Comparison::GrammarRefused(Refusal::TooLarge)
    );
    assert_eq!(budget, LINE_COST);
}

#[test]
fn empty_scripts_cannot_multiply_the_comparisons() {
    let mut command = vec!["bash".to_string()];
    for _ in 0..2000 {
        command.push("-c".to_string());
        command.push(String::new());
    }
    RECORDED.with(|recorded| recorded.borrow_mut().clear());
    let action = ApprovalAction::Shell {
        id: "call".into(),
        environment_id: "local".into(),
        command,
        cwd: PathUri::from_abs_path(
            &AbsolutePathBuf::from_absolute_path_checked(fixture("/work")).expect("absolute"),
        ),
        sandbox_permissions: SandboxPermissions::UseDefault,
        additional_permissions: None,
        justification: None,
    };
    let _ = classify_with(
        &action,
        &fixture("/home/fixture/.corbanu"),
        Some(&fixture("/home/fixture")),
    );
    let recorded = RECORDED.with(RefCell::take);
    let compared = recorded
        .iter()
        .filter(|comparison| **comparison != Comparison::NotCompared)
        .count();
    assert!(
        compared <= MAX_COMPARED_BYTES / LINE_COST,
        "{compared} lines compared"
    );
}

#[test]
fn missing_counts_repeats() {
    let names = |names: &[&str]| {
        names
            .iter()
            .map(|name| (*name).to_string())
            .collect::<Vec<_>>()
    };
    assert!(!missing(&names(&["a", "b"]), &names(&["a", "b", "c"])));
    assert!(missing(&names(&["a", "a"]), &names(&["a", "b"])));
    assert!(missing(&names(&["c"]), &names(&["a", "b"])));
    assert!(!missing(&[], &names(&["a"])));
}
