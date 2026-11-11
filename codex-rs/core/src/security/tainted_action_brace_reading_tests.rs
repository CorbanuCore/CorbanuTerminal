//! #1975 review 3: preserve the full pre-brace-change verdict, including
//! #1772's redirection reading, for groups after shell control words.

#![cfg(unix)]

use super::tainted_action_redirect_tests::HOME;
use super::tainted_action_redirect_tests::SUB_EVIL;
use super::tainted_action_redirect_tests::action_in;
use super::tainted_action_redirect_tests::workspace;
use super::*;
use pretty_assertions::assert_eq;

const LINES: &[&str] = &[
    ">log cd sub; ./run.sh",
    "{ >log cd sub; ./run.sh; }",
    "if true; then { >log cd sub; ./run.sh; }; fi",
    "for i in 1; do { >log cd sub; ./run.sh; }; done",
    "while true; do { >log cd sub; ./run.sh; }; break; done",
    "until false; do { >log cd sub; ./run.sh; }; break; done",
    "! { >log cd sub; ./run.sh; }",
    "time { >log cd sub; ./run.sh; }",
    "if false; then :; else { >log cd sub; ./run.sh; }; fi",
    "if false; then :; elif true; then { >log cd sub; ./run.sh; }; fi",
    "true && { >log cd sub; ./run.sh; }",
    "( >log cd sub; ./run.sh )",
    "{ for i in 1; do >log cd sub; done }; ./run.sh",
    "{ if true; then >log cd sub; fi }; ./run.sh",
    "{ case x in x) >log cd sub;; esac }; ./run.sh",
];

/// Both public classification paths and the forced-shell measuring harness
/// must retain the Persistence finding. No fixture script is executed.
fn wrong_verdicts(
    read: impl Fn(&ApprovalAction, &Path) -> Option<ProtectedActionKind>,
) -> Vec<String> {
    let root = workspace(SUB_EVIL);
    let mut wrong = Vec::new();
    for shell in ["bash", "zsh"] {
        for line in LINES {
            for cd in [">log cd sub", "cd sub >log", "cd sub"] {
                let line = line.replace(">log cd sub", cd);
                let action = action_in(root.path(), &[shell, "-lc", &line]);
                let found = read(&action, root.path());
                if found != Some(ProtectedActionKind::Persistence) {
                    wrong.push(format!("{shell}: {line} -> {found:?}"));
                }
            }
        }
    }
    wrong
}

#[test]
fn issue_1975_brace_groups_keep_full_main_verdict() {
    assert_eq!(
        wrong_verdicts(|action, root| classify_with(action, Path::new(HOME), Some(root))),
        Vec::<String>::new()
    );
}

#[test]
fn issue_1975_named_brace_groups_keep_full_main_verdict() {
    assert_eq!(
        wrong_verdicts(|action, root| classify_as_named_with(action, Path::new(HOME), Some(root))),
        Vec::<String>::new()
    );
}

#[test]
fn issue_1975_measuring_harness_keeps_full_main_verdict() {
    for avoids_zsh in [false, true] {
        assert_eq!(
            wrong_verdicts(|action, root| judged_avoiding(
                action,
                || Homes::new(Path::new(HOME), Some(root)),
                avoids_zsh,
            )
            .found),
            Vec::<String>::new(),
            "avoids_zsh={avoids_zsh}"
        );
    }
}
