//! #1973: data exposed by quote stripping cannot resolve a git remote.
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
fn issue_1973_quoted_assignments_cannot_hide_remotes() {
    for line in [
        r#"echo 'x;r=o'; git fetch e"$r""#,
        r#"echo "x;r=o"; git fetch e"$r""#,
        r#"echo 'x; r=o'; git fetch e"$r""#,
        "echo 'x\nr=o'; git fetch e\"$r\"",
        r#"echo 'x;export r=o'; git fetch e"$r""#,
        r#"echo 'x; r=o; '; git pull e"$r""#,
        r#"echo "x;'r'=o"; git ls-remote e"$r""#,
        r#"echo 'x; r=o'; git push e"$r" HEAD:main"#,
        r#"echo 'x; r=o'; r=origin; git fetch e"$r""#,
    ] {
        assert_eq!(
            verdicts(line),
            // git pull also changes persistent repository state. Once the
            // action asks, the existing D191 held finding is retained.
            vec![
                Some(if line.contains("git pull") {
                    ProtectedActionKind::Persistence
                } else {
                    ProtectedActionKind::UnseenCode
                });
                4
            ],
            "{line}"
        );
    }
}

#[test]
fn issue_1973_review_ambiguous_quote_contexts_ask() {
    for line in [
        r#"echo x\;r=o; git fetch e"$r""#,
        "echo a #'\necho 'x;r=o'; git fetch e\"$r\"",
        "cat <<E\ndon't\nE\necho 'x;r=o'; git fetch e\"$r\"",
        r#"echo $'\'' 'x;r=o'; git fetch e"$r""#,
        "echo a #x;r=o\ngit fetch e\"$r\"",
        r#"r""=o; git fetch e"$r""#,
        r#"r\=o; git fetch e"$r""#,
        r#"r'='o; git fetch e"$r""#,
        r#"a=1 'r=o'; git fetch e"$r""#,
        "echo \\ #'\n;r=o'; git fetch e\"$r\"",
        "echo \\\n#'\necho 'x;r=o'; git fetch e\"$r\"",
        "echo \u{a0}#'\n;r=o'; git fetch e\"$r\"",
        "echo $(true)#'\n;r=o'; git fetch e\"$r\"",
        r#"echo "$(case x in a) echo "x;r=o";; esac)"; git fetch e"$r""#,
        "echo \"$(echo #)\necho \"x;r=o\")\"; git fetch e\"$r\"",
    ] {
        assert_eq!(
            verdicts(line),
            vec![Some(ProtectedActionKind::UnseenCode); 4],
            "{line}"
        );
    }
}

#[test]
fn issue_1973_real_assignments_and_literal_remotes_stay_quiet() {
    for line in [
        r#"r=origin; git fetch "$r""#,
        r#"r='origin'; git fetch "$r""#,
        r#"r="origin"; git fetch "$r""#,
        r#"export r='origin'; git fetch "$r""#,
        r#"echo 'x;y'; git fetch origin"#,
        r#"echo "x;y"; r=origin; git fetch "$r""#,
        r#"echo 'r=o'; git fetch origin"#,
    ] {
        assert_eq!(verdicts(line), vec![None; 4], "{line}");
    }
}

#[test]
fn issue_1973_keeps_stronger_literal_path_findings() {
    for (line, baseline) in [
        (
            "echo 'x;a=.a'; b=ws; cat ~/$a$b/credentials",
            "a=.a; b=ws; cat ~/$a$b/credentials",
        ),
        (
            "echo 'x;a=.co'; b=rbanu; cat ~/$a$b/config.toml",
            "a=.co; b=rbanu; cat ~/$a$b/config.toml",
        ),
    ] {
        let expected = verdicts(baseline);
        assert!(expected.iter().all(Option::is_some));
        assert_eq!(verdicts(line), expected, "{line}");
    }
}

#[test]
fn issue_1973_quote_scan_is_linear() {
    tracing::subscriber::with_default(tracing::subscriber::NoSubscriber::default(), || {
        work::assert_linear("quoted assignments", |scale| {
            let line = format!("{}git fetch e\"$r\"", "echo 'x;r=o'; ".repeat(100 * scale));
            let _ = verdicts(&line);
        });
    });
}
