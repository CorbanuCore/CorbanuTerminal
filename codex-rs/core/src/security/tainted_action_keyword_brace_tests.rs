use super::tainted_action_redirect_tests::HOME;
use super::tainted_action_redirect_tests::SUB_EVIL;
use super::tainted_action_redirect_tests::action_in;
use super::tainted_action_redirect_tests::workspace;
use super::*;
use pretty_assertions::assert_eq;

const GROUPS: &[&str] = &[
    "if true; then { echo ok; }; fi",
    "for x in a; do { echo ok; }; done",
    "if false; then :; else { echo ok; }; fi",
    "if false; then :; elif true; then { echo ok; }; fi",
    "! { echo ok; }",
    "time { echo ok; }",
    "time ! { echo ok; }",
    "{ if true; then echo ok; fi }",
    "{ for x in a; do echo ok; done }",
    "{ case x in (x) echo ok;; esac }",
];

#[test]
fn issue_2018_keyword_groups_are_read_as_commands() {
    for name in ["bash", "zsh"] {
        for line in GROUPS {
            let argv = [name, "-lc", line].map(str::to_owned);
            let main = shell::simple_commands_read(&argv, shell::Reading::LiteralBraces);
            let fixed = shell::simple_commands_read(&argv, shell::Reading::KeywordBraces);
            assert!(main.keyword_braces, "{name}: {line}");
            assert!(main.braces_read_newly, "{name}: {line}");
            assert!(!fixed.braces_read_newly, "{name}: {line}");
            assert!(
                fixed.commands.iter().any(|command| {
                    command
                        .words
                        .windows(2)
                        .any(|words| words == ["echo", "ok"])
                }),
                "{name}: {line}"
            );
        }
    }
}

#[test]
fn issue_2018_arguments_keep_literal_braces() {
    for name in ["bash", "zsh"] {
        for line in [
            "echo time { echo ok; }",
            "'time' { echo ok; }",
            r"\! { echo ok; }",
            "echo 'then { inside quotes'",
            "echo time > {",
            "{ if true; then :; fi '}'; }",
        ] {
            let argv = [name, "-lc", line].map(str::to_owned);
            let main = shell::simple_commands_read(&argv, shell::Reading::LiteralBraces);
            let fixed = shell::simple_commands_read(&argv, shell::Reading::KeywordBraces);
            let words = |lexed: shell::Lexed| {
                lexed
                    .commands
                    .into_iter()
                    .map(|command| command.words)
                    .collect::<Vec<_>>()
            };
            assert!(!main.keyword_braces, "{name}: {line}");
            assert_eq!(words(fixed), words(main), "{name}: {line}");
        }
    }
}

#[test]
fn issue_2018_keeps_main_verdict_in_bash_and_zsh() {
    let root = workspace(SUB_EVIL);
    let homes = || Homes::new(Path::new(HOME), Some(root.path()));
    for name in ["bash", "zsh"] {
        for as_named in [false, true] {
            for line in GROUPS.iter().copied().chain([
                "! { echo ok; } | bash",
                "time { echo ok; } | bash",
                "if true; then { >log cd sub; ./run.sh; }; fi",
                "! { >log cd sub; ./run.sh; }",
                "time { >log cd sub; ./run.sh; }",
                "r=origin; ! { echo ok; }; git fetch -o '{}' -- \"$r\"",
            ]) {
                let action = action_in(root.path(), &[name, "-lc", line]);
                let main = judged_readings(|reading, assignments| {
                    assert!(reading != shell::Reading::KeywordBraces);
                    let mut result = judged(&action, homes(), as_named, reading, assignments);
                    result.keyword_braces = false;
                    result
                })
                .found;
                let fixed = judged_both(&action, homes, as_named).found;
                if line.ends_with("| bash") {
                    assert_eq!(
                        main,
                        Some(ProtectedActionKind::UnseenCode),
                        "{name}: {line}"
                    );
                }
                assert!(
                    fixed.map_or(0, |kind| rank(kind) + 1) >= main.map_or(0, |kind| rank(kind) + 1),
                    "{name} as_named={as_named}: {line}: main={main:?}, fixed={fixed:?}"
                );
            }
        }
    }
}
