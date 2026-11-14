//! #2001: zstyle -e values run as shell code on a later lookup.
use super::*;
use crate::sandboxing::SandboxPermissions;
use codex_utils_absolute_path::AbsolutePathBuf;
use pretty_assertions::assert_eq;

fn verdicts(line: &str) -> Vec<Option<ProtectedActionKind>> {
    let root = if cfg!(windows) { "C:/work" } else { "/work" };
    let home = Path::new(root).join("home/.corbanu");
    let user_home = Path::new(root).join("home");
    ["bash", "zsh"]
        .into_iter()
        .flat_map(|shell| {
            let action = ApprovalAction::Shell {
                id: "call".into(),
                environment_id: "local".into(),
                command: [shell, "-lc", line].map(str::to_string).to_vec(),
                cwd: PathUri::from_abs_path(
                    &AbsolutePathBuf::from_absolute_path_checked(root).expect("absolute"),
                ),
                sandbox_permissions: SandboxPermissions::UseDefault,
                additional_permissions: None,
                justification: None,
            };
            [
                classify_with(&action, &home, Some(&user_home)),
                classify_as_named_with(&action, &home, Some(&user_home)),
            ]
        })
        .collect()
}

#[test]
fn issue_2001_style_code_is_read_before_lookup() {
    for line in [
        "zstyle -e :x y 'eval $q'; zstyle -s :x y v",
        "zstyle -e :x y 'eval $q'",
        "zstyle -e :x y eval '$q'; zstyle -s :x y v",
        "builtin zstyle -e :x y 'eval $q'; zstyle -a :x y v",
        "command zstyle -e :x y 'eval $q'; zstyle -b :x y v",
        "zs''tyle -e :x y 'eval $q'; zstyle -T :x y",
        "zstyle -e :x y 'git fetch' 'e\"$r\"'; zstyle -t :x y",
        "zstyle -e ':x y' 'a b' 'eval $q'; zstyle -s ':x y' 'a b' v",
    ] {
        assert_eq!(
            verdicts(line),
            vec![Some(ProtectedActionKind::UnseenCode); 4],
            "{line}"
        );
    }
}

#[test]
fn issue_2001_review_dynamic_eval_options_ask() {
    for line in [
        "o=-e; zstyle $o :x y 'eval $q'; zstyle -s :x y v",
        "zstyle \"$o\" :x y 'eval $q'; zstyle -s :x y v",
        "zstyle \"$(printf %s -e)\" :x y 'eval $q'; zstyle -s :x y v",
        "o=-e; zstyle $o :x y 'zm* zsh/st?t'; zstyle -s :x y v; stat -A functions x",
        "zstyle -$o :x y 'eval $q'; zstyle -s :x y v",
    ] {
        assert_eq!(
            verdicts(line),
            vec![Some(ProtectedActionKind::UnseenCode); 4],
            "{line}"
        );
    }
}

#[test]
fn issue_2001_style_may_load_stat_before_table_writes() {
    for lookup in ["-s :x y v", "-a :x y v", "-b :x y v", "-T :x y", "-t :x y"] {
        for writer in ["stat -A functions x", "stat -H aliases x"] {
            let line = format!("zstyle -e :x y 'zm* zsh/st?t'; zstyle {lookup}; {writer}");
            assert_eq!(
                verdicts(&line),
                vec![Some(ProtectedActionKind::UnseenCode); 4],
                "{line}"
            );
        }
    }
}

#[test]
fn issue_2001_literal_styles_and_visible_code_stay_quiet() {
    for line in [
        "zstyle :x y 'eval $q'; zstyle -s :x y v",
        "zstyle -- :x y 'eval $q'; zstyle -s :x y v",
        "zstyle \":ctx:$x\" y 'eval $q'; zstyle -s \":ctx:$x\" y v",
        "zstyle -e :x y 'print -r -- ok'; zstyle -s :x y v",
        "zstyle -e :x y print -r -- ok",
    ] {
        assert_eq!(verdicts(line), vec![None; 4], "{line}");
    }
}

#[test]
fn issue_2001_preserves_stronger_findings_in_stored_code() {
    for (line, baseline) in [
        (
            "zstyle -e :x y 'cat ~/.aws/credentials'",
            "cat ~/.aws/credentials",
        ),
        (
            "zstyle -e :x y 'echo x >> ~/.zshenv'",
            "echo x >> ~/.zshenv",
        ),
    ] {
        let expected = verdicts(baseline);
        assert!(expected.iter().all(Option::is_some));
        assert_eq!(verdicts(line), expected, "{line}");
    }
}

#[test]
fn issue_2001_deferred_style_scan_is_linear() {
    tracing::subscriber::with_default(tracing::subscriber::NoSubscriber::default(), || {
        work::assert_linear("deferred styles", |scale| {
            let line = (0..50 * scale)
                .map(|index| format!("zstyle -e :x y{index} 'eval $q{index}'; "))
                .collect::<String>();
            let _ = verdicts(&line);
        });
    });
}
