use super::SimpleCommand;
use super::UNSEEN_OUTPUT;
use super::simple_commands;
use pretty_assertions::assert_eq;

#[test]
fn issue_1020_nested_and_sibling_substitutions_preserve_the_complete_graph() {
    let lexed = simple_commands(&[
        "echo $(printf one) $(printf $(echo two)); echo $(cd /work && pwd)".to_string(),
    ]);
    assert!(!lexed.incomplete);
    assert_eq!(
        lexed.commands,
        vec![
            SimpleCommand {
                words: vec!["printf".into(), "one".into()],
                substituted: true,
                ..SimpleCommand::default()
            },
            SimpleCommand {
                words: vec!["echo".into(), "two".into()],
                substituted: true,
                ..SimpleCommand::default()
            },
            SimpleCommand {
                words: vec!["printf".into(), "two".into()],
                substituted: true,
                fed_by: vec![1],
                ..SimpleCommand::default()
            },
            SimpleCommand {
                words: vec!["echo".into(), "one".into(), "two".into()],
                fed_by: vec![0, 1, 2],
                ..SimpleCommand::default()
            },
            SimpleCommand {
                words: vec!["cd".into(), "/work".into()],
                substituted: true,
                ..SimpleCommand::default()
            },
            SimpleCommand {
                words: vec!["pwd".into()],
                substituted: true,
                ..SimpleCommand::default()
            },
            SimpleCommand {
                words: vec!["echo".into(), "/work".into()],
                fed_by: vec![4, 5],
                ..SimpleCommand::default()
            },
        ]
    );
}

#[test]
fn issue_1020_process_substitutions_keep_their_distinct_output_relationships() {
    let lexed =
        simple_commands(&["echo $(printf one >(cat)); echo $(printf two <(cat))".to_string()]);
    assert!(!lexed.incomplete);
    assert_eq!(
        lexed.commands,
        vec![
            SimpleCommand {
                words: vec!["cat".into()],
                substituted: true,
                reads_outer_output: true,
                ..SimpleCommand::default()
            },
            SimpleCommand {
                words: vec!["printf".into(), "one".into()],
                substituted: true,
                ..SimpleCommand::default()
            },
            SimpleCommand {
                words: vec!["echo".into(), UNSEEN_OUTPUT.into()],
                fed_by: vec![0, 1],
                ..SimpleCommand::default()
            },
            SimpleCommand {
                words: vec!["cat".into()],
                substituted: true,
                ..SimpleCommand::default()
            },
            SimpleCommand {
                words: vec!["printf".into(), "two".into(), UNSEEN_OUTPUT.into()],
                substituted: true,
                fed_by: vec![3],
                ..SimpleCommand::default()
            },
            SimpleCommand {
                words: vec!["echo".into(), format!("two {UNSEEN_OUTPUT}")],
                fed_by: vec![3, 4],
                ..SimpleCommand::default()
            },
        ]
    );
}

#[test]
fn issue_1020_unclosed_substitutions_and_pipes_preserve_the_complete_graph() {
    let closed = simple_commands(&["cat input | echo $(printf $(echo value)) >output".into()]);
    let unclosed = simple_commands(&["cat input | echo $(printf $(echo value".into()]);
    assert!(!closed.incomplete);
    assert!(!unclosed.incomplete);
    let mut expected = vec![
        SimpleCommand {
            words: vec!["cat".into(), "input".into()],
            ..SimpleCommand::default()
        },
        SimpleCommand {
            words: vec!["echo".into(), "value".into()],
            substituted: true,
            ..SimpleCommand::default()
        },
        SimpleCommand {
            words: vec!["printf".into(), "value".into()],
            substituted: true,
            fed_by: vec![1],
            ..SimpleCommand::default()
        },
        SimpleCommand {
            words: vec!["echo".into(), "value".into()],
            pipe_from: Some(0),
            fed_by: vec![1, 2],
            ..SimpleCommand::default()
        },
    ];
    assert_eq!(unclosed.commands, expected);
    expected[3].words.push("output".into());
    expected[3].writes_to.push(2);
    assert_eq!(closed.commands, expected);
}

fn chain(seed: &str, depth: usize, quoted: bool) -> String {
    let mut input = seed.to_string();
    for _ in 0..depth {
        input = if quoted {
            format!("printf \"%s\" \"$({input})\"")
        } else {
            format!("printf \"%s\" $({input})")
        };
    }
    input
}

#[test]
fn issue_1020_intent_corpus_records_full_reference_observations() {
    use super::substitution_observer::observe;
    // The versioned validation method pairs these actual observations from
    // the original and candidate. No generic safe/unsafe label is invented.
    for (id, input) in [
        ("E01/0", ""),
        ("E01/1", " "),
        ("E01/2", "\t\n"),
        ("E01/3", "pwd"),
        ("E01/4", "rm -rf -- ./issue1020-owned-marker"),
        ("E01/5", "__issue1020_unknown_command__"),
        (
            "E03/0",
            "printf '%s' '$(rm -rf -- ./issue1020-owned-marker)'",
        ),
        ("E03/1", "printf \"%s\" \"$(pwd)\""),
        ("E03/2", "printf \"%s\" \\$(pwd)"),
        ("E03/3", "printf \"%s\" \"${ISSUE1020_VALUE:-$(pwd)}\""),
        ("E03/4", "printf \"%s\" \"$((1 + 2))\""),
        ("E03/5", "printf \"%s\" \"$(printf '%s' '')\""),
        ("E03/6", "printf \"%s\" \"$(printf '%s' 'α中🙂')\""),
        ("E03/7", "printf \"%s\" \"$(printf '%s' 'é')\""),
        ("E03/8", "printf \"%s\" \"$(printf '%s' 'é')\""),
        ("E03/9", "printf \"%s\" \"$( )\""),
        ("E03/10", "printf \"%s\" \"$()\""),
        ("E03/11", "printf \"%s\" \"$(pwd\""),
        (
            "E04/0",
            "printf \"%s\" \"$(pwd)\" \"$(__issue1020_unknown_command__)\"",
        ),
        (
            "E04/1",
            "printf \"%s\" \"$(__issue1020_unknown_command__)\" \"$(pwd)\"",
        ),
        (
            "E04/2",
            "printf \"%s\" \"$(pwd)\" \"$(rm -rf -- ./issue1020-owned-marker)\"",
        ),
        (
            "E04/3",
            "printf \"%s\" \"$(rm -rf -- ./issue1020-owned-marker)\" \"$(pwd)\"",
        ),
        ("E04/4", "printf '%s' '$(pwd)' \"$(pwd)\""),
        ("E04/5", "printf \"%s\" \"$(pwd)\" \"$(pwd)\""),
    ] {
        observe(id, input);
    }
    for (seed_index, seed) in [
        "pwd",
        "rm -rf -- ./issue1020-owned-marker",
        "__issue1020_unknown_command__",
    ]
    .iter()
    .enumerate()
    {
        for quoted in [false, true] {
            for depth in [0, 1, 2, 4, 8] {
                let id = format!("E02/{seed_index}/{quoted}/{depth}");
                observe(&id, &chain(seed, depth, quoted));
            }
        }
    }
}

#[expect(
    clippy::print_stdout,
    reason = "Emit bounded test observations for the external paired-source validator"
)]
fn check_scaling(family: &str, samples: &[(usize, [u64; 7])], failures: &mut Vec<String>) {
    let fixed = samples[0].1;
    for pair in samples.windows(2) {
        let [(left_size, left), (right_size, right)] = pair else {
            unreachable!("adjacent pair");
        };
        if *left_size < 8 || *right_size != 2 * left_size {
            continue;
        }
        for dimension in 0..fixed.len() {
            let left_excess = left[dimension].saturating_sub(fixed[dimension]);
            let right_excess = right[dimension].saturating_sub(fixed[dimension]);
            let limit = 6 * left_excess.max(1) + 128;
            println!(
                "ISSUE1020_SCALING {}",
                serde_json::json!({
                    "family": family, "dimension": dimension, "leftSize": left_size,
                    "rightSize": right_size, "fixed": fixed[dimension],
                    "leftExcess": left_excess, "rightExcess": right_excess, "limit": limit,
                })
            );
            if right_excess > limit {
                failures.push(format!("{family}/{dimension}: {right_excess} > {limit}"));
            }
        }
    }
}

#[test]
fn issue_1020_nested_and_wide_actual_work_has_bounded_scaling() {
    use super::substitution_observer::observe;
    let mut failures = Vec::new();
    for quoted in [false, true] {
        let family = format!("C01/{quoted}");
        let samples: Vec<_> = [0, 4, 8, 16, 32, 64]
            .into_iter()
            .map(|depth| {
                let id = format!("{family}/{depth}");
                (depth, observe(&id, &chain("pwd", depth, quoted)))
            })
            .collect();
        check_scaling(&family, &samples, &mut failures);
    }
    let cold = observe("C02/fresh-before", "pwd");
    let wide: Vec<_> = [1, 8, 16, 32, 64, 128]
        .into_iter()
        .map(|width| {
            let input = format!("printf \"%s\" {}", vec!["\"$(pwd)\""; width].join(" "));
            (width, observe(&format!("C02/{width}"), &input))
        })
        .collect();
    check_scaling("C02", &wide, &mut failures);
    assert_eq!(observe("C02/fresh-after", "pwd"), cold);
    // All completed observations are emitted before the structural verdict,
    // so an actual baseline failure cannot erase its reference results.
    assert_eq!(failures, Vec::<String>::new());
}

#[test]
#[expect(
    clippy::print_stdout,
    reason = "Emit bounded test observations for the external paired-source validator"
)]
fn issue_1020_depth_2000_records_separate_bounded_timing_diagnostic() {
    // The public issue's original input and suggested measurement. The
    // validation plan selects this once on the candidate with an external
    // deadline, separately from the depth64 structural regression campaign.
    let depth = 2_000;
    let script = format!("echo $(( {}", "$(".repeat(depth));
    let (lexed, elapsed, counts, operation_limit) =
        super::substitution_observer::observe_depth_2000(script);
    println!(
        "ISSUE1020_TIMING {}",
        serde_json::json!({
            "depth": depth, "elapsedSeconds": elapsed.as_secs_f64(),
            "belowIssueExampleOneSecond": elapsed.as_secs_f64() < 1.0,
            "counts": counts, "operationLimit": operation_limit,
            "commands": lexed.commands.len(), "incomplete": lexed.incomplete,
            "purpose": "supplementary measurement; no elapsed-time success assertion",
        })
    );
    assert!(!lexed.incomplete);
    assert_eq!(lexed.commands.len(), depth + 1);
    for (index, command) in lexed.commands.iter().enumerate() {
        assert_eq!(
            command,
            &SimpleCommand {
                words: if index == depth {
                    vec!["echo".into(), UNSEEN_OUTPUT.into()]
                } else {
                    vec![UNSEEN_OUTPUT.into()]
                },
                substituted: index < depth,
                fed_by: (0..index).collect(),
                ..SimpleCommand::default()
            }
        );
    }
}
