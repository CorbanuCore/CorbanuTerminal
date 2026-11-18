#![allow(clippy::expect_used)]
use std::any::Any;
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;

use anyhow::Context;
use anyhow::Result;
use codex_execpolicy::Decision;
use codex_execpolicy::Error;
use codex_execpolicy::Evaluation;
use codex_execpolicy::MatchOptions;
use codex_execpolicy::NetworkRuleProtocol;
use codex_execpolicy::PatternToken;
use codex_execpolicy::Policy;
use codex_execpolicy::PolicyParser;
use codex_execpolicy::PrefixPattern;
use codex_execpolicy::PrefixRule;
use codex_execpolicy::RequirementsExecPolicy;
use codex_execpolicy::RuleMatch;
use codex_execpolicy::RuleRef;
use codex_execpolicy::blocking_append_allow_prefix_rule;
use codex_utils_absolute_path::AbsolutePathBuf;
use pretty_assertions::assert_eq;
use tempfile::tempdir;

fn tokens(cmd: &[&str]) -> Vec<String> {
    cmd.iter().map(std::string::ToString::to_string).collect()
}

fn allow_all(_: &[String]) -> Decision {
    Decision::Allow
}

fn prompt_all(_: &[String]) -> Decision {
    Decision::Prompt
}

fn absolute_path(path: &str) -> AbsolutePathBuf {
    AbsolutePathBuf::try_from(path.to_string()).expect("path should be absolute")
}

fn host_absolute_path(segments: &[&str]) -> String {
    let mut path = if cfg!(windows) {
        PathBuf::from(r"C:\")
    } else {
        PathBuf::from("/")
    };
    for segment in segments {
        path.push(segment);
    }
    path.to_string_lossy().into_owned()
}

fn host_executable_name(name: &str) -> String {
    if cfg!(windows) {
        format!("{name}.exe")
    } else {
        name.to_string()
    }
}

fn starlark_string(value: &str) -> String {
    value.replace('\\', "\\\\").replace('"', "\\\"")
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum RuleSnapshot {
    Prefix(PrefixRule),
}

fn rule_snapshots(rules: &[RuleRef]) -> Vec<RuleSnapshot> {
    rules
        .iter()
        .map(|rule| {
            let rule_any = rule.as_ref() as &dyn Any;
            if let Some(prefix_rule) = rule_any.downcast_ref::<PrefixRule>() {
                RuleSnapshot::Prefix(prefix_rule.clone())
            } else {
                panic!("unexpected rule type in RuleRef: {rule:?}");
            }
        })
        .collect()
}

#[test]
fn append_allow_prefix_rule_dedupes_existing_rule() -> Result<()> {
    let tmp = tempdir().context("create temp dir")?;
    let policy_path = tmp.path().join("rules").join("default.rules");
    let prefix = tokens(&["python3"]);

    blocking_append_allow_prefix_rule(&policy_path, &prefix)?;
    blocking_append_allow_prefix_rule(&policy_path, &prefix)?;

    let contents = fs::read_to_string(&policy_path).context("read policy")?;
    assert_eq!(
        contents,
        r#"prefix_rule(pattern=["python3"], decision="allow")
"#
    );
    Ok(())
}

#[test]
fn network_rules_compile_into_domain_lists() -> Result<()> {
    let policy_src = r#"
network_rule(host = "google.com", protocol = "http", decision = "allow")
network_rule(host = "api.github.com", protocol = "https", decision = "allow")
network_rule(host = "blocked.example.com", protocol = "https", decision = "deny")
network_rule(host = "prompt-only.example.com", protocol = "https", decision = "prompt")
    "#;
    let mut parser = PolicyParser::new();
    parser.parse("network.rules", policy_src)?;
    let policy = parser.build();

    assert_eq!(policy.network_rules().len(), 4);
    assert_eq!(
        policy.network_rules()[1].protocol,
        NetworkRuleProtocol::Https
    );

    let (allowed, denied) = policy.compiled_network_domains();
    assert_eq!(
        allowed,
        vec!["google.com".to_string(), "api.github.com".to_string()]
    );
    assert_eq!(denied, vec!["blocked.example.com".to_string()]);
    Ok(())
}

#[test]
fn network_rule_rejects_wildcard_hosts() {
    let mut parser = PolicyParser::new();
    let err = parser
        .parse(
            "network.rules",
            r#"network_rule(host="*", protocol="http", decision="allow")"#,
        )
        .expect_err("wildcard network_rule host should fail");
    assert!(err.to_string().contains("wildcards are not allowed"));
}

#[test]
fn basic_match() -> Result<()> {
    let policy_src = r#"
prefix_rule(
    pattern = ["git", "status"],
)
    "#;
    let mut parser = PolicyParser::new();
    parser.parse("test.rules", policy_src)?;
    let policy = parser.build();
    let cmd = tokens(&["git", "status"]);
    let evaluation = policy.check(&cmd, &allow_all);
    assert_eq!(
        Evaluation {
            decision: Decision::Allow,
            matched_rules: vec![RuleMatch::PrefixRuleMatch {
                matched_prefix: tokens(&["git", "status"]),
                decision: Decision::Allow,
                resolved_program: None,
                justification: None,
            }],
        },
        evaluation
    );
    Ok(())
}

#[test]
fn justification_is_attached_to_forbidden_matches() -> Result<()> {
    let policy_src = r#"
prefix_rule(
    pattern = ["rm"],
    decision = "forbidden",
    justification = "destructive command",
)
    "#;
    let mut parser = PolicyParser::new();
    parser.parse("test.rules", policy_src)?;
    let policy = parser.build();

    let evaluation = policy.check(
        &tokens(&["rm", "-rf", "/some/important/folder"]),
        &allow_all,
    );
    assert_eq!(
        Evaluation {
            decision: Decision::Forbidden,
            matched_rules: vec![RuleMatch::PrefixRuleMatch {
                matched_prefix: tokens(&["rm"]),
                decision: Decision::Forbidden,
                resolved_program: None,
                justification: Some("destructive command".to_string()),
            }],
        },
        evaluation
    );
    Ok(())
}

#[test]
fn justification_can_be_used_with_allow_decision() -> Result<()> {
    let policy_src = r#"
prefix_rule(
    pattern = ["ls"],
    decision = "allow",
    justification = "safe and commonly used",
)
    "#;
    let mut parser = PolicyParser::new();
    parser.parse("test.rules", policy_src)?;
    let policy = parser.build();

    let evaluation = policy.check(&tokens(&["ls", "-l"]), &prompt_all);
    assert_eq!(
        Evaluation {
            decision: Decision::Allow,
            matched_rules: vec![RuleMatch::PrefixRuleMatch {
                matched_prefix: tokens(&["ls"]),
                decision: Decision::Allow,
                resolved_program: None,
                justification: Some("safe and commonly used".to_string()),
            }],
        },
        evaluation
    );
    Ok(())
}

#[test]
fn justification_cannot_be_empty() {
    let policy_src = r#"
prefix_rule(
    pattern = ["ls"],
    decision = "prompt",
    justification = "   ",
)
    "#;
    let mut parser = PolicyParser::new();
    let err = parser
        .parse("test.rules", policy_src)
        .expect_err("expected parse error");
    assert!(
        err.to_string()
            .contains("invalid rule: justification cannot be empty")
    );
}

#[test]
fn add_prefix_rule_extends_policy() -> Result<()> {
    let mut policy = Policy::empty();
    policy.add_prefix_rule(&tokens(&["ls", "-l"]), Decision::Prompt)?;

    let rules = rule_snapshots(policy.rules().get_vec("ls").context("missing ls rules")?);
    assert_eq!(
        vec![RuleSnapshot::Prefix(PrefixRule {
            pattern: PrefixPattern {
                canonical_matching: false,
                first: Arc::from("ls"),
                rest: vec![PatternToken::Single(String::from("-l"))].into(),
            },
            decision: Decision::Prompt,
            justification: None,
        })],
        rules
    );

    let evaluation = policy.check(&tokens(&["ls", "-l", "/some/important/folder"]), &allow_all);
    assert_eq!(
        Evaluation {
            decision: Decision::Prompt,
            matched_rules: vec![RuleMatch::PrefixRuleMatch {
                matched_prefix: tokens(&["ls", "-l"]),
                decision: Decision::Prompt,
                resolved_program: None,
                justification: None,
            }],
        },
        evaluation
    );
    Ok(())
}

#[test]
fn add_prefix_rule_rejects_empty_prefix() -> Result<()> {
    let mut policy = Policy::empty();
    let result = policy.add_prefix_rule(&[], Decision::Allow);

    match result.unwrap_err() {
        Error::InvalidPattern(message) => assert_eq!(message, "prefix cannot be empty"),
        other => panic!("expected InvalidPattern(..), got {other:?}"),
    }
    Ok(())
}

#[test]
fn parses_multiple_policy_files() -> Result<()> {
    let first_policy = r#"
prefix_rule(
    pattern = ["git"],
    decision = "prompt",
)
    "#;
    let second_policy = r#"
prefix_rule(
    pattern = ["git", "commit"],
    decision = "forbidden",
)
    "#;
    let mut parser = PolicyParser::new();
    parser.parse("first.rules", first_policy)?;
    parser.parse("second.rules", second_policy)?;
    let policy = parser.build();

    let git_rules = rule_snapshots(policy.rules().get_vec("git").context("missing git rules")?);
    assert_eq!(
        vec![
            RuleSnapshot::Prefix(PrefixRule {
                pattern: PrefixPattern {
                    canonical_matching: false,
                    first: Arc::from("git"),
                    rest: Vec::<PatternToken>::new().into(),
                },
                decision: Decision::Prompt,
                justification: None,
            }),
            RuleSnapshot::Prefix(PrefixRule {
                pattern: PrefixPattern {
                    canonical_matching: false,
                    first: Arc::from("git"),
                    rest: vec![PatternToken::Single("commit".to_string())].into(),
                },
                decision: Decision::Forbidden,
                justification: None,
            }),
        ],
        git_rules
    );

    let status_eval = policy.check(&tokens(&["git", "status"]), &allow_all);
    assert_eq!(
        Evaluation {
            decision: Decision::Prompt,
            matched_rules: vec![RuleMatch::PrefixRuleMatch {
                matched_prefix: tokens(&["git"]),
                decision: Decision::Prompt,
                resolved_program: None,
                justification: None,
            }],
        },
        status_eval
    );

    let commit_eval = policy.check(&tokens(&["git", "commit", "-m", "hi"]), &allow_all);
    assert_eq!(
        Evaluation {
            decision: Decision::Forbidden,
            matched_rules: vec![
                RuleMatch::PrefixRuleMatch {
                    matched_prefix: tokens(&["git"]),
                    decision: Decision::Prompt,
                    resolved_program: None,
                    justification: None,
                },
                RuleMatch::PrefixRuleMatch {
                    matched_prefix: tokens(&["git", "commit"]),
                    decision: Decision::Forbidden,
                    resolved_program: None,
                    justification: None,
                },
            ],
        },
        commit_eval
    );
    Ok(())
}

#[test]
fn only_first_token_alias_expands_to_multiple_rules() -> Result<()> {
    let policy_src = r#"
prefix_rule(
    pattern = [["bash", "sh"], ["-c", "-l"]],
)
    "#;
    let mut parser = PolicyParser::new();
    parser.parse("test.rules", policy_src)?;
    let policy = parser.build();

    let bash_rules = rule_snapshots(
        policy
            .rules()
            .get_vec("bash")
            .context("missing bash rules")?,
    );
    let sh_rules = rule_snapshots(policy.rules().get_vec("sh").context("missing sh rules")?);
    assert_eq!(
        vec![RuleSnapshot::Prefix(PrefixRule {
            pattern: PrefixPattern {
                canonical_matching: false,
                first: Arc::from("bash"),
                rest: vec![PatternToken::Alts(vec!["-c".to_string(), "-l".to_string()])].into(),
            },
            decision: Decision::Allow,
            justification: None,
        })],
        bash_rules
    );
    assert_eq!(
        vec![RuleSnapshot::Prefix(PrefixRule {
            pattern: PrefixPattern {
                canonical_matching: false,
                first: Arc::from("sh"),
                rest: vec![PatternToken::Alts(vec!["-c".to_string(), "-l".to_string()])].into(),
            },
            decision: Decision::Allow,
            justification: None,
        })],
        sh_rules
    );

    let bash_eval = policy.check(&tokens(&["bash", "-c", "echo", "hi"]), &allow_all);
    assert_eq!(
        Evaluation {
            decision: Decision::Allow,
            matched_rules: vec![RuleMatch::PrefixRuleMatch {
                matched_prefix: tokens(&["bash", "-c"]),
                decision: Decision::Allow,
                resolved_program: None,
                justification: None,
            }],
        },
        bash_eval
    );

    let sh_eval = policy.check(&tokens(&["sh", "-l", "echo", "hi"]), &allow_all);
    assert_eq!(
        Evaluation {
            decision: Decision::Allow,
            matched_rules: vec![RuleMatch::PrefixRuleMatch {
                matched_prefix: tokens(&["sh", "-l"]),
                decision: Decision::Allow,
                resolved_program: None,
                justification: None,
            }],
        },
        sh_eval
    );
    Ok(())
}

#[test]
fn tail_aliases_are_not_cartesian_expanded() -> Result<()> {
    let policy_src = r#"
prefix_rule(
    pattern = ["npm", ["i", "install"], ["--legacy-peer-deps", "--no-save"]],
)
    "#;
    let mut parser = PolicyParser::new();
    parser.parse("test.rules", policy_src)?;
    let policy = parser.build();

    let rules = rule_snapshots(policy.rules().get_vec("npm").context("missing npm rules")?);
    assert_eq!(
        vec![RuleSnapshot::Prefix(PrefixRule {
            pattern: PrefixPattern {
                canonical_matching: false,
                first: Arc::from("npm"),
                rest: vec![
                    PatternToken::Alts(vec!["i".to_string(), "install".to_string()]),
                    PatternToken::Alts(vec![
                        "--legacy-peer-deps".to_string(),
                        "--no-save".to_string(),
                    ]),
                ]
                .into(),
            },
            decision: Decision::Allow,
            justification: None,
        })],
        rules
    );

    let npm_i = policy.check(&tokens(&["npm", "i", "--legacy-peer-deps"]), &allow_all);
    assert_eq!(
        Evaluation {
            decision: Decision::Allow,
            matched_rules: vec![RuleMatch::PrefixRuleMatch {
                matched_prefix: tokens(&["npm", "i", "--legacy-peer-deps"]),
                decision: Decision::Allow,
                resolved_program: None,
                justification: None,
            }],
        },
        npm_i
    );

    let npm_install = policy.check(
        &tokens(&["npm", "install", "--no-save", "leftpad"]),
        &allow_all,
    );
    assert_eq!(
        Evaluation {
            decision: Decision::Allow,
            matched_rules: vec![RuleMatch::PrefixRuleMatch {
                matched_prefix: tokens(&["npm", "install", "--no-save"]),
                decision: Decision::Allow,
                resolved_program: None,
                justification: None,
            }],
        },
        npm_install
    );
    Ok(())
}

#[test]
fn match_and_not_match_examples_are_enforced() -> Result<()> {
    let policy_src = r#"
prefix_rule(
    pattern = ["git", "status"],
    match = [["git", "status"], "git status"],
    not_match = [
        ["git", "--config", "color.status=always", "status"],
        "git --config color.status=always status",
    ],
)
    "#;
    let mut parser = PolicyParser::new();
    parser.parse("test.rules", policy_src)?;
    let policy = parser.build();
    let match_eval = policy.check(&tokens(&["git", "status"]), &allow_all);
    assert_eq!(
        Evaluation {
            decision: Decision::Allow,
            matched_rules: vec![RuleMatch::PrefixRuleMatch {
                matched_prefix: tokens(&["git", "status"]),
                decision: Decision::Allow,
                resolved_program: None,
                justification: None,
            }],
        },
        match_eval
    );

    let no_match_eval = policy.check(
        &tokens(&["git", "--config", "color.status=always", "status"]),
        &allow_all,
    );
    assert_eq!(
        Evaluation {
            decision: Decision::Allow,
            matched_rules: vec![RuleMatch::HeuristicsRuleMatch {
                command: tokens(&["git", "--config", "color.status=always", "status",]),
                decision: Decision::Allow,
            }],
        },
        no_match_eval
    );
    Ok(())
}

#[test]
fn strictest_decision_wins_across_matches() -> Result<()> {
    let policy_src = r#"
prefix_rule(
    pattern = ["git"],
    decision = "prompt",
)
prefix_rule(
    pattern = ["git", "commit"],
    decision = "forbidden",
)
    "#;
    let mut parser = PolicyParser::new();
    parser.parse("test.rules", policy_src)?;
    let policy = parser.build();

    let commit = policy.check(&tokens(&["git", "commit", "-m", "hi"]), &allow_all);
    assert_eq!(
        Evaluation {
            decision: Decision::Forbidden,
            matched_rules: vec![
                RuleMatch::PrefixRuleMatch {
                    matched_prefix: tokens(&["git"]),
                    decision: Decision::Prompt,
                    resolved_program: None,
                    justification: None,
                },
                RuleMatch::PrefixRuleMatch {
                    matched_prefix: tokens(&["git", "commit"]),
                    decision: Decision::Forbidden,
                    resolved_program: None,
                    justification: None,
                },
            ],
        },
        commit
    );
    Ok(())
}

#[test]
fn strictest_decision_across_multiple_commands() -> Result<()> {
    let policy_src = r#"
prefix_rule(
    pattern = ["git"],
    decision = "prompt",
)
prefix_rule(
    pattern = ["git", "commit"],
    decision = "forbidden",
)
    "#;
    let mut parser = PolicyParser::new();
    parser.parse("test.rules", policy_src)?;
    let policy = parser.build();

    let commands = vec![
        tokens(&["git", "status"]),
        tokens(&["git", "commit", "-m", "hi"]),
    ];

    let evaluation = policy.check_multiple(&commands, &allow_all);
    assert_eq!(
        Evaluation {
            decision: Decision::Forbidden,
            matched_rules: vec![
                RuleMatch::PrefixRuleMatch {
                    matched_prefix: tokens(&["git"]),
                    decision: Decision::Prompt,
                    resolved_program: None,
                    justification: None,
                },
                RuleMatch::PrefixRuleMatch {
                    matched_prefix: tokens(&["git"]),
                    decision: Decision::Prompt,
                    resolved_program: None,
                    justification: None,
                },
                RuleMatch::PrefixRuleMatch {
                    matched_prefix: tokens(&["git", "commit"]),
                    decision: Decision::Forbidden,
                    resolved_program: None,
                    justification: None,
                },
            ],
        },
        evaluation
    );
    Ok(())
}

#[test]
fn heuristics_match_is_returned_when_no_policy_matches() {
    let policy = Policy::empty();
    let command = tokens(&["python"]);

    let evaluation = policy.check(&command, &prompt_all);
    assert_eq!(
        Evaluation {
            decision: Decision::Prompt,
            matched_rules: vec![RuleMatch::HeuristicsRuleMatch {
                command,
                decision: Decision::Prompt,
            }],
        },
        evaluation
    );
}

#[test]
fn parses_host_executable_paths() -> Result<()> {
    let homebrew_git = host_absolute_path(&["opt", "homebrew", "bin", "git"]);
    let usr_git = host_absolute_path(&["usr", "bin", "git"]);
    let homebrew_git_literal = starlark_string(&homebrew_git);
    let usr_git_literal = starlark_string(&usr_git);
    let policy_src = format!(
        r#"
host_executable(
    name = "git",
    paths = [
        "{homebrew_git_literal}",
        "{usr_git_literal}",
        "{usr_git_literal}",
    ],
)
"#
    );
    let mut parser = PolicyParser::new();
    parser.parse("test.rules", &policy_src)?;
    let policy = parser.build();

    assert_eq!(
        policy
            .host_executables()
            .get("git")
            .expect("missing git host executable")
            .as_ref(),
        [absolute_path(&homebrew_git), absolute_path(&usr_git)]
    );
    Ok(())
}

#[test]
fn host_executable_rejects_non_absolute_path() {
    let policy_src = r#"
host_executable(name = "git", paths = ["git"])
"#;
    let mut parser = PolicyParser::new();
    let err = parser
        .parse("test.rules", policy_src)
        .expect_err("expected parse error");
    assert!(
        err.to_string()
            .contains("host_executable paths must be absolute")
    );
}

#[test]
fn host_executable_rejects_name_with_path_separator() {
    let git_path = host_absolute_path(&["usr", "bin", "git"]);
    let git_path_literal = starlark_string(&git_path);
    let policy_src =
        format!(r#"host_executable(name = "{git_path_literal}", paths = ["{git_path_literal}"])"#);
    let mut parser = PolicyParser::new();
    let err = parser
        .parse("test.rules", &policy_src)
        .expect_err("expected parse error");
    assert!(
        err.to_string()
            .contains("host_executable name must be a bare executable name")
    );
}

#[test]
fn host_executable_rejects_path_with_wrong_basename() {
    let rg_path = host_absolute_path(&["usr", "bin", "rg"]);
    let rg_path_literal = starlark_string(&rg_path);
    let policy_src = format!(r#"host_executable(name = "git", paths = ["{rg_path_literal}"])"#);
    let mut parser = PolicyParser::new();
    let err = parser
        .parse("test.rules", &policy_src)
        .expect_err("expected parse error");
    assert!(err.to_string().contains("must have basename `git`"));
}

#[test]
fn host_executable_last_definition_wins() -> Result<()> {
    let usr_git = host_absolute_path(&["usr", "bin", "git"]);
    let homebrew_git = host_absolute_path(&["opt", "homebrew", "bin", "git"]);
    let usr_git_literal = starlark_string(&usr_git);
    let homebrew_git_literal = starlark_string(&homebrew_git);
    let mut parser = PolicyParser::new();
    parser.parse(
        "shared.rules",
        &format!(r#"host_executable(name = "git", paths = ["{usr_git_literal}"])"#),
    )?;
    parser.parse(
        "user.rules",
        &format!(r#"host_executable(name = "git", paths = ["{homebrew_git_literal}"])"#),
    )?;
    let policy = parser.build();

    assert_eq!(
        policy
            .host_executables()
            .get("git")
            .expect("missing git host executable")
            .as_ref(),
        [absolute_path(&homebrew_git)]
    );
    Ok(())
}

#[test]
fn host_executable_resolution_uses_basename_rule_when_allowed() -> Result<()> {
    let git_name = host_executable_name("git");
    let git_path = host_absolute_path(&["usr", "bin", &git_name]);
    let git_path_literal = starlark_string(&git_path);
    let policy_src = format!(
        r#"
prefix_rule(pattern = ["git", "status"], decision = "prompt")
host_executable(name = "git", paths = ["{git_path_literal}"])
"#
    );
    let mut parser = PolicyParser::new();
    parser.parse("test.rules", &policy_src)?;
    let policy = parser.build();

    let evaluation = policy.check_with_options(
        &[git_path.clone(), "status".to_string()],
        &allow_all,
        &MatchOptions {
            resolve_host_executables: true,
            fold_program_case: false,
        },
    );
    assert_eq!(
        evaluation,
        Evaluation {
            decision: Decision::Prompt,
            matched_rules: vec![RuleMatch::PrefixRuleMatch {
                matched_prefix: tokens(&["git", "status"]),
                decision: Decision::Prompt,
                resolved_program: Some(absolute_path(&git_path)),
                justification: None,
            }],
        }
    );
    Ok(())
}

/// An absolute-path amendment may bypass a basename rule only if its restriction is preserved.
#[test]
fn inherited_absolute_amendments_preserve_managed_basename_restrictions() -> Result<()> {
    let mut parser = PolicyParser::new();
    parser.parse(
        "managed.rules",
        r#"prefix_rule(pattern = ["git", ["push", "send-pack"]], canonical_matching=True, decision = "forbidden")"#,
    )?;
    let managed = parser.build();
    let requirements = RequirementsExecPolicy::new(managed.clone());
    let git = host_absolute_path(&["usr", "bin", &host_executable_name("git")]);
    for (args, decision, accepted) in [
        (&["status"][..], Decision::Allow, true),
        (&[], Decision::Allow, false),
        (&["send-pack", "origin"], Decision::Allow, false),
        (&["PUſH"], Decision::Allow, false),
        (&["push"], Decision::Forbidden, true),
    ] {
        let mut snapshot = managed.clone();
        let mut prefix = vec![git.clone()];
        prefix.extend(tokens(args));
        snapshot.add_prefix_rule(&prefix, decision)?;
        assert_eq!(
            snapshot.preserves_managed_requirements(&requirements),
            accepted
        );
    }
    let path = starlark_string(&git);
    for (insensitive, accepted) in [("True", true), ("False", false)] {
        let mut parser = PolicyParser::new();
        parser.parse("path.rules", &format!(
            r#"prefix_rule(pattern=["{path}", ["push", "send-pack"]], canonical_matching={insensitive}, decision="forbidden")"#
        ))?;
        let managed = managed.merge_overlay(&parser.build());
        let mut snapshot = managed.clone();
        snapshot.add_prefix_rule(std::slice::from_ref(&git), Decision::Allow)?;
        assert_eq!(
            snapshot.preserves_managed_requirements(&RequirementsExecPolicy::new(managed)),
            accepted
        );
    }
    Ok(())
}

#[test]
fn inherited_absolute_amendment_respects_existing_exact_denials() -> Result<()> {
    let git = host_absolute_path(&["usr", "bin", &host_executable_name("git")]);
    for (denied_args, accepted) in [(&["push"][..], true), (&["push", "origin"][..], false)] {
        let mut managed = Policy::empty();
        managed.add_prefix_rule(&tokens(&["git", "push"]), Decision::Forbidden)?;
        let mut exact = vec![git.clone()];
        exact.extend(tokens(denied_args));
        managed.add_prefix_rule(&exact, Decision::Forbidden)?;

        let mut snapshot = managed.clone();
        snapshot.add_prefix_rule(std::slice::from_ref(&git), Decision::Allow)?;
        assert_eq!(
            snapshot.preserves_managed_requirements(&RequirementsExecPolicy::new(managed)),
            accepted
        );
    }
    Ok(())
}

#[test]
fn prefix_rule_examples_honor_host_executable_resolution() -> Result<()> {
    let allowed_git_name = host_executable_name("git");
    let allowed_git = host_absolute_path(&["usr", "bin", &allowed_git_name]);
    let other_git = host_absolute_path(&["opt", "homebrew", "bin", &allowed_git_name]);
    let allowed_git_literal = starlark_string(&allowed_git);
    let other_git_literal = starlark_string(&other_git);
    let policy_src = format!(
        r#"
prefix_rule(
    pattern = ["git", "status"],
    match = [["{allowed_git_literal}", "status"]],
    not_match = [["{other_git_literal}", "status"]],
)
host_executable(name = "git", paths = ["{allowed_git_literal}"])
"#
    );

    let mut parser = PolicyParser::new();
    parser.parse("test.rules", &policy_src)?;

    Ok(())
}

#[test]
fn canonical_host_executable_resolution_uses_only_relevant_declarations() -> Result<()> {
    let bash_path = starlark_string(&host_absolute_path(&[
        "usr",
        "bin",
        &host_executable_name("bash"),
    ]));
    // The long s keeps the executable lookup keys distinct on Windows too.
    let long_s_bash_path = starlark_string(&host_absolute_path(&[
        "usr",
        "bin",
        &host_executable_name("baſh"),
    ]));
    let policy_src = format!(
        r#"
prefix_rule(pattern = ["bash"], canonical_matching = True, decision = "forbidden",
            match = [["{bash_path}"]], not_match = [["{long_s_bash_path}"]])
host_executable(name = "bash", paths = ["{bash_path}"])
host_executable(name = "baſh", paths = ["{long_s_bash_path}"])
"#
    );
    PolicyParser::new().parse("test.rules", &policy_src)?;
    Ok(())
}

#[test]
fn host_executable_resolution_respects_explicit_empty_allowlist() -> Result<()> {
    let policy_src = r#"
prefix_rule(pattern = ["git"], decision = "prompt")
host_executable(name = "git", paths = [])
"#;
    let mut parser = PolicyParser::new();
    parser.parse("test.rules", policy_src)?;
    let policy = parser.build();
    let git_path = host_absolute_path(&["usr", "bin", "git"]);

    let evaluation = policy.check_with_options(
        &[git_path.clone(), "status".to_string()],
        &allow_all,
        &MatchOptions {
            resolve_host_executables: true,
            fold_program_case: false,
        },
    );
    assert_eq!(
        evaluation,
        Evaluation {
            decision: Decision::Allow,
            matched_rules: vec![RuleMatch::HeuristicsRuleMatch {
                command: vec![git_path, "status".to_string()],
                decision: Decision::Allow,
            }],
        }
    );
    Ok(())
}

#[test]
fn host_executable_resolution_ignores_path_not_in_allowlist() -> Result<()> {
    let allowed_git = host_absolute_path(&["usr", "bin", "git"]);
    let other_git = host_absolute_path(&["opt", "homebrew", "bin", "git"]);
    let allowed_git_literal = starlark_string(&allowed_git);
    let policy_src = format!(
        r#"
prefix_rule(pattern = ["git"], decision = "prompt")
host_executable(name = "git", paths = ["{allowed_git_literal}"])
"#
    );
    let mut parser = PolicyParser::new();
    parser.parse("test.rules", &policy_src)?;
    let policy = parser.build();

    let evaluation = policy.check_with_options(
        &[other_git.clone(), "status".to_string()],
        &allow_all,
        &MatchOptions {
            resolve_host_executables: true,
            fold_program_case: false,
        },
    );
    assert_eq!(
        evaluation,
        Evaluation {
            decision: Decision::Allow,
            matched_rules: vec![RuleMatch::HeuristicsRuleMatch {
                command: vec![other_git, "status".to_string()],
                decision: Decision::Allow,
            }],
        }
    );
    Ok(())
}

#[test]
fn host_executable_resolution_falls_back_without_mapping() -> Result<()> {
    let policy_src = r#"
prefix_rule(pattern = ["git"], decision = "prompt")
"#;
    let mut parser = PolicyParser::new();
    parser.parse("test.rules", policy_src)?;
    let policy = parser.build();
    let git_path = host_absolute_path(&["usr", "bin", "git"]);

    let evaluation = policy.check_with_options(
        &[git_path.clone(), "status".to_string()],
        &allow_all,
        &MatchOptions {
            resolve_host_executables: true,
            fold_program_case: false,
        },
    );
    assert_eq!(
        evaluation,
        Evaluation {
            decision: Decision::Prompt,
            matched_rules: vec![RuleMatch::PrefixRuleMatch {
                matched_prefix: tokens(&["git"]),
                decision: Decision::Prompt,
                resolved_program: Some(absolute_path(&git_path)),
                justification: None,
            }],
        }
    );
    Ok(())
}

#[test]
fn host_executable_resolution_does_not_override_exact_match() -> Result<()> {
    let git_path = host_absolute_path(&["usr", "bin", "git"]);
    let git_path_literal = starlark_string(&git_path);
    let policy_src = format!(
        r#"
prefix_rule(pattern = ["{git_path_literal}"], decision = "allow")
prefix_rule(pattern = ["git"], decision = "prompt")
host_executable(name = "git", paths = ["{git_path_literal}"])
"#
    );
    let mut parser = PolicyParser::new();
    parser.parse("test.rules", &policy_src)?;
    let policy = parser.build();

    let evaluation = policy.check_with_options(
        &[git_path.clone(), "status".to_string()],
        &allow_all,
        &MatchOptions {
            resolve_host_executables: true,
            fold_program_case: false,
        },
    );
    assert_eq!(
        evaluation,
        Evaluation {
            decision: Decision::Allow,
            matched_rules: vec![RuleMatch::PrefixRuleMatch {
                matched_prefix: vec![git_path],
                decision: Decision::Allow,
                resolved_program: None,
                justification: None,
            }],
        }
    );
    Ok(())
}

#[test]
fn canonical_matching_prefix_matches_each_token() -> Result<()> {
    let mut parser = PolicyParser::new();
    parser.parse(
        "case.rules",
        r#"
prefix_rule(pattern=[["Tool", "bash"], "Run", ["ONE", "two", "set"]], canonical_matching=True,
            match=["TOOL rUn Two extra", "bAſh RUN one", "bash run ſet"],
            not_match=["TOOL run three", "tool runner two"])
prefix_rule(pattern=["café", ["résumé", "run"]], canonical_matching=True,
            match=["CAFÉ RÉSUMÉ"], not_match=["cafe resume", "ｃafé résumé"])
# Full case folding can expand one character into two, within a single token.
prefix_rule(pattern=["straße"], canonical_matching=True,
            match=["STRASSE"], not_match=["STRAS SE"])
# U+0345 folds to a spacing iota. Reorder the acute accent before folding
# so it stays attached to the alpha.
prefix_rule(pattern=["α\u0345\u0301"], canonical_matching=True,
            match=["άι"], not_match=["αί"])
prefix_rule(pattern=["café"], not_match=["café"])
prefix_rule(pattern=["set-content"], canonical_matching=True, not_match=["set-cøntent"])
prefix_rule(pattern=["bash"], not_match=["baſh"])
prefix_rule(pattern=["Tool", "Run"], not_match=["tool Run", "Tool run"])
"#,
    )?;
    Ok(())
}

/// #2506: each rule's decision is mapped before the strictest is taken, so a
/// weakened forbidden rule cannot hide a prompt-user rule.
#[test]
fn strictest_rule_matching_more_args_by_maps_each_rule() -> Result<()> {
    let mut policy = Policy::empty();
    policy.add_prefix_rule(&tokens(&["git", "push"]), Decision::PromptUser)?;
    policy.add_prefix_rule(&tokens(&["git", "push", "--force"]), Decision::Forbidden)?;
    policy.add_prefix_rule(&tokens(&["git", "status", "-s"]), Decision::Allow)?;
    let options = MatchOptions {
        resolve_host_executables: false,
        fold_program_case: false,
    };
    let weaken_forbidden = |decision| match decision {
        Decision::Allow => None,
        Decision::Forbidden => Some(Decision::Prompt),
        other => Some(other),
    };
    for (known, expected) in [
        (vec![], Some(Decision::PromptUser)),
        (vec!["git"], Some(Decision::PromptUser)),
        (vec!["git", "push"], Some(Decision::Prompt)),
        (vec!["git", "status"], None),
    ] {
        assert_eq!(
            policy.strictest_rule_matching_more_args_by(
                &tokens(&known),
                &options,
                weaken_forbidden
            ),
            expected,
            "{known:?}"
        );
    }
    assert_eq!(
        policy.strictest_rule_matching_more_args(&tokens(&["git"]), &options),
        Some(Decision::Forbidden)
    );
    Ok(())
}

#[test]
fn strictest_rule_matching_more_args_counts_only_longer_rules() -> Result<()> {
    let mut parser = PolicyParser::new();
    parser.parse(
        "test.rules",
        r#"prefix_rule(pattern=["git", ["push", "fetch"], "--force"], decision="forbidden")
prefix_rule(pattern=["git", "push"], decision="prompt")
prefix_rule(pattern=["git", "status"], decision="allow")
prefix_rule(pattern=["ls"], decision="allow")"#,
    )?;
    let policy = parser.build();
    let options = MatchOptions {
        resolve_host_executables: true,
        fold_program_case: false,
    };
    let git = host_absolute_path(&["usr", "bin", "git"]);
    for (known, expected) in [
        (vec![], Some(Decision::Forbidden)),
        (vec!["git"], Some(Decision::Forbidden)),
        (vec!["git", "fetch"], Some(Decision::Forbidden)),
        (vec![git.as_str(), "push"], Some(Decision::Forbidden)),
        // Already matched (left to ordinary matching), or never matching.
        (vec!["git", "push", "--force"], None),
        (vec!["git", "status"], None),
        (vec!["git", "push", "-f"], None),
        (vec!["gitx"], None),
        (vec!["ls"], None),
        (vec!["cat"], None),
    ] {
        assert_eq!(
            policy.strictest_rule_matching_more_args(&tokens(&known), &options),
            expected,
            "{known:?}"
        );
    }
    let mut parser = PolicyParser::new();
    parser.parse(
        "test.rules",
        r#"prefix_rule(pattern=["git", "push"], decision="prompt")
prefix_rule(pattern=["git", "status"], decision="allow")"#,
    )?;
    let policy = parser.build();
    assert_eq!(
        policy.strictest_rule_matching_more_args(&tokens(&["git"]), &options),
        Some(Decision::Prompt)
    );
    assert_eq!(
        Policy::empty().strictest_rule_matching_more_args(&[], &options),
        None
    );
    // Canonical rules count under their canonical name and compare
    // canonically, as when matching (sync side's canonical_matching).
    let mut parser = PolicyParser::new();
    parser.parse(
        "test.rules",
        r#"prefix_rule(pattern=["Tool", "Run", "--force"], decision="forbidden", canonical_matching=True)
prefix_rule(pattern=["tool", "status"], decision="allow")"#,
    )?;
    let policy = parser.build();
    for (known, expected) in [
        (vec!["TOOL"], Some(Decision::Forbidden)),
        (vec!["tool", "rUn"], Some(Decision::Forbidden)),
        (vec!["tool", "status"], None),
        (vec!["tool", "walk"], None),
    ] {
        assert_eq!(
            policy.strictest_rule_matching_more_args(&tokens(&known), &options),
            expected,
            "{known:?}"
        );
    }
    Ok(())
}

/// D696: a `canonical_matching` allow rule asks for caseless names, so it
/// still approves `RM`, but not the `.exe` spellings Windows' host lookup
/// also reaches.
#[test]
fn canonical_allow_rules_fold_case_but_keep_the_suffix() -> Result<()> {
    let mut parser = PolicyParser::new();
    parser.parse(
        "test.rules",
        r#"prefix_rule(pattern=["rm"], decision="allow", canonical_matching=True)"#,
    )?;
    let policy = parser.build();
    let options = MatchOptions {
        resolve_host_executables: true,
        fold_program_case: false,
    };
    let decision = |cmd: &[&str]| {
        policy
            .check_with_options(&tokens(cmd), &ask_all, &options)
            .decision
    };
    assert_eq!(decision(&["rm", "x"]), Decision::Allow);
    assert_eq!(decision(&["RM", "x"]), Decision::Allow);
    for program in [
        "RM.EXE".to_string(),
        "rm.exe".to_string(),
        host_absolute_path(&["Windows", "RM.exe"]),
    ] {
        assert_eq!(
            decision(&[program.as_str(), "x"]),
            Decision::Prompt,
            "{program}"
        );
    }
    Ok(())
}

const FOLD: MatchOptions = MatchOptions {
    resolve_host_executables: true,
    fold_program_case: true,
};

fn folded_decisions(policy: &Policy, cmd: &[&str], options: &MatchOptions) -> Vec<Decision> {
    policy
        .case_folded_matches(&tokens(cmd), options)
        .iter()
        .map(RuleMatch::decision)
        .collect()
}

/// D692: forbidden and prompt rules match the program name in any case;
/// allow rules and arguments stay exact.
#[test]
fn case_folded_matches_only_strict_rules() -> Result<()> {
    let mut policy = Policy::empty();
    policy.add_prefix_rule(&tokens(&["rm"]), Decision::Forbidden)?;
    policy.add_prefix_rule(&tokens(&["git", "push"]), Decision::Prompt)?;
    policy.add_prefix_rule(&tokens(&["ls"]), Decision::Allow)?;
    policy.add_prefix_rule(&tokens(&["Rscript"]), Decision::Forbidden)?;
    policy.add_prefix_rule(&tokens(&["/bin/sudo"]), Decision::Prompt)?;
    policy.add_prefix_rule(&tokens(&["sudo"]), Decision::Allow)?;
    for (cmd, expected) in [
        (&["RM", "-rf", "x"][..], vec![Decision::Forbidden]),
        (&["Rm"], vec![Decision::Forbidden]),
        (&["GIT", "push"], vec![Decision::Prompt]),
        (&["rscript", "x"], vec![Decision::Forbidden]),
        (&["RSCRIPT"], vec![Decision::Forbidden]),
        (&["/BIN/SUDO", "ls"], vec![Decision::Prompt]),
        (&["\u{17f}\u{75}do"], vec![]),
        // The exact name is left to the exact lookup.
        (&["rm", "x"], vec![]),
        // Arguments stay exact.
        (&["GIT", "PUSH"], vec![]),
        // Allow rules never match another case.
        (&["LS"], vec![]),
        (&["SUDO"], vec![]),
        (&["RMX"], vec![]),
    ] {
        assert_eq!(folded_decisions(&policy, cmd, &FOLD), expected, "{cmd:?}");
        assert_eq!(
            folded_decisions(&policy, cmd, &MatchOptions::default()),
            Vec::<Decision>::new(),
            "{cmd:?}"
        );
        // The exact lookup is unchanged by the option.
        assert_eq!(
            policy.matches_for_command_with_options(
                &tokens(cmd),
                /*heuristics_fallback*/ None,
                &FOLD
            ),
            policy.matches_for_command_with_options(
                &tokens(cmd),
                /*heuristics_fallback*/ None,
                &MatchOptions {
                    resolve_host_executables: true,
                    fold_program_case: false,
                }
            ),
            "{cmd:?}"
        );
    }
    let rule_match = policy.case_folded_matches(&tokens(&["RM", "-rf"]), &FOLD);
    assert_eq!(
        rule_match,
        vec![RuleMatch::PrefixRuleMatch {
            matched_prefix: tokens(&["rm"]),
            decision: Decision::Forbidden,
            resolved_program: None,
            justification: None,
        }]
    );
    // Unicode folds that lowercasing misses.
    let mut policy = Policy::empty();
    policy.add_prefix_rule(&tokens(&["sudo"]), Decision::Forbidden)?;
    policy.add_prefix_rule(&tokens(&["timeout"]), Decision::Forbidden)?;
    for cmd in ["\u{17f}\u{75}do", "SUDO", "t\u{131}meout"] {
        assert_eq!(
            folded_decisions(&policy, &[cmd], &FOLD),
            vec![Decision::Forbidden],
            "{cmd}"
        );
    }
    Ok(())
}

#[test]
fn case_folded_matches_resolve_host_executables() -> Result<()> {
    let rm = host_executable_name("rm");
    let listed = host_absolute_path(&["usr", "bin", &rm]);
    let listed_upper = host_absolute_path(&["USR", "BIN", &rm.to_ascii_uppercase()]);
    let other_upper = host_absolute_path(&["opt", "bin", &rm.to_ascii_uppercase()]);
    let mut policy = Policy::empty();
    policy.add_prefix_rule(&tokens(&["rm"]), Decision::Forbidden)?;
    let unlisted = policy.clone();
    policy.set_host_executable_paths("rm".to_string(), vec![absolute_path(&listed)]);
    let found = policy.case_folded_matches(&[listed_upper.clone(), "x".to_string()], &FOLD);
    assert_eq!(
        found,
        vec![RuleMatch::PrefixRuleMatch {
            matched_prefix: tokens(&["rm"]),
            decision: Decision::Forbidden,
            resolved_program: Some(absolute_path(&listed_upper)),
            justification: None,
        }]
    );
    // Another directory is another program when paths are listed.
    assert_eq!(
        policy.case_folded_matches(std::slice::from_ref(&other_upper), &FOLD),
        Vec::new()
    );
    // With no list any directory counts. (On Windows the basename key is
    // already lower case, so the exact lookup matches it.)
    assert_eq!(
        unlisted
            .case_folded_matches(&[other_upper], &FOLD)
            .iter()
            .map(RuleMatch::decision)
            .collect::<Vec<_>>(),
        if cfg!(windows) {
            Vec::new()
        } else {
            vec![Decision::Forbidden]
        }
    );
    // Relative paths read by their basename, as the exact lookup does.
    for cmd in ["./RM", "../bin/RM"] {
        assert_eq!(
            unlisted
                .case_folded_matches(&tokens(&[cmd, "x"]), &FOLD)
                .iter()
                .map(RuleMatch::decision)
                .collect::<Vec<_>>(),
            if cfg!(windows) {
                Vec::new()
            } else {
                vec![Decision::Forbidden]
            },
            "{cmd}"
        );
    }
    // Without host resolution, only the word itself folds.
    assert_eq!(
        policy.case_folded_matches(
            &[listed_upper],
            &MatchOptions {
                resolve_host_executables: false,
                fold_program_case: true,
            }
        ),
        Vec::new()
    );
    Ok(())
}

#[test]
fn strictest_rule_matching_more_args_folds_only_strict_rules() -> Result<()> {
    let mut policy = Policy::empty();
    policy.add_prefix_rule(&tokens(&["rm", "-rf"]), Decision::Forbidden)?;
    policy.add_prefix_rule(&tokens(&["ls", "-l"]), Decision::Allow)?;
    let exact = MatchOptions {
        resolve_host_executables: true,
        fold_program_case: false,
    };
    assert_eq!(
        policy.strictest_rule_matching_more_args(&tokens(&["RM"]), &FOLD),
        Some(Decision::Forbidden)
    );
    assert_eq!(
        policy.strictest_rule_matching_more_args(&tokens(&["RM"]), &exact),
        // Windows host-executable lookup folds the basename even when the
        // separate program-case matching option is disabled (D696).
        if cfg!(windows) {
            Some(Decision::Forbidden)
        } else {
            None
        }
    );
    assert_eq!(
        policy.strictest_rule_matching_more_args(&tokens(&["LS"]), &FOLD),
        None
    );
    assert_eq!(
        policy.strictest_rule_matching_more_args(&tokens(&["ls"]), &FOLD),
        Some(Decision::Allow)
    );
    Ok(())
}

#[test]
fn partial_allow_rules_need_exact_host_spelling() -> Result<()> {
    let mut parser = PolicyParser::new();
    parser.parse(
        "test.rules",
        r#"prefix_rule(pattern=["git", "push"], decision="allow")"#,
    )?;
    let policy = parser.build();
    for fold_program_case in [false, true] {
        let options = MatchOptions {
            resolve_host_executables: true,
            fold_program_case,
        };
        for program in ["git".to_string(), host_absolute_path(&["tools", "git"])] {
            assert_eq!(
                policy.strictest_rule_matching_more_args(&tokens(&[&program]), &options),
                Some(Decision::Allow),
                "{program}"
            );
        }
        for program in [
            "GIT".to_string(),
            "git.exe".to_string(),
            "GIT.EXE".to_string(),
            host_absolute_path(&["tools", "GIT"]),
            host_absolute_path(&["tools", "git.exe"]),
        ] {
            assert_eq!(
                policy.strictest_rule_matching_more_args(&tokens(&[&program]), &options),
                None,
                "{program}"
            );
            // Eligibility is checked before callers map rule decisions.
            assert_eq!(
                policy.strictest_rule_matching_more_args_by(&tokens(&[&program]), &options, |_| {
                    Some(Decision::Forbidden)
                },),
                None,
                "{program}"
            );
        }
    }
    Ok(())
}

fn ask_all(_: &[String]) -> Decision {
    Decision::Prompt
}

/// D696: allow rules approve only the program as they spell it; Windows'
/// host lookup (lower-cased, `.exe`/`.cmd`/`.bat`/`.com` dropped) now feeds
/// only forbidden and prompt rules. Off Windows the lookup key is the file
/// name itself, so every case below behaves the same there.
#[test]
fn allow_rules_need_the_exact_spelling() -> Result<()> {
    let mut parser = PolicyParser::new();
    parser.parse(
        "test.rules",
        r#"prefix_rule(pattern=["rm"], decision="allow")
prefix_rule(pattern=["git", "push"], decision="allow")
prefix_rule(pattern=["git", "push", "--force"], decision="forbidden")"#,
    )?;
    let policy = parser.build();
    for fold_program_case in [false, true] {
        let options = MatchOptions {
            resolve_host_executables: true,
            fold_program_case,
        };
        let decision = |cmd: &[&str]| {
            policy
                .check_with_options(&tokens(cmd), &ask_all, &options)
                .decision
        };
        let git = host_absolute_path(&["usr", "bin", "git"]);
        // The exact spelling, bare or as a path (the control).
        assert_eq!(decision(&["rm", "x"]), Decision::Allow);
        assert_eq!(decision(&["git", "push"]), Decision::Allow);
        assert_eq!(decision(&[git.as_str(), "push"]), Decision::Allow);
        for program in [
            "RM".to_string(),
            "Rm".to_string(),
            "rm.exe".to_string(),
            "RM.EXE".to_string(),
            host_absolute_path(&["usr", "bin", "RM"]),
            host_absolute_path(&["usr", "bin", "rm.exe"]),
        ] {
            assert_eq!(
                decision(&[program.as_str(), "x"]),
                Decision::Prompt,
                "{program}"
            );
        }
        for program in [
            "Git".to_string(),
            "GIT.EXE".to_string(),
            "git.exe".to_string(),
            host_absolute_path(&["Program Files", "Git", "cmd", "git.exe"]),
            host_absolute_path(&["usr", "bin", "GIT"]),
        ] {
            assert_eq!(
                decision(&[program.as_str(), "push"]),
                Decision::Prompt,
                "{program}"
            );
            // Forbidden rules keep Windows' lookup.
            let forbidden = if cfg!(windows) {
                Decision::Forbidden
            } else {
                Decision::Prompt
            };
            assert_eq!(
                decision(&[program.as_str(), "push", "--force"]),
                forbidden,
                "{program}"
            );
        }
    }
    Ok(())
}

/// D696: `match` and `not_match` examples keep reading programs as before,
/// so a rules file that parsed still parses.
#[test]
fn allow_rule_examples_keep_the_host_lookup() -> Result<()> {
    let mut parser = PolicyParser::new();
    let parsed = parser.parse(
        "test.rules",
        r#"prefix_rule(pattern=["git", "push"], decision="allow", match=[["GIT.EXE", "push"]])"#,
    );
    // Windows reads `GIT.EXE` by its lookup key `git`; elsewhere it never
    // matched.
    assert_eq!(parsed.is_ok(), cfg!(windows), "{parsed:?}");
    Ok(())
}
