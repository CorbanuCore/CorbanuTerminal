use super::DangerousCommandMatch;
use super::dangerous_command_match;
use pretty_assertions::assert_eq;

fn check(words: &[&str], expected: Option<DangerousCommandMatch>) {
    let command = words
        .iter()
        .map(|word| (*word).to_owned())
        .collect::<Vec<_>>();
    assert_eq!(dangerous_command_match(&command), expected, "{words:?}");
}

#[test]
fn issue_2285_force_prefixes() {
    for option in ["--f", "--fo", "--for", "--forc", "--force", "-rf", "-vfr"] {
        check(
            &["rm", option, "target"],
            Some(DangerousCommandMatch::ForcedRm),
        );
        check(
            &["rm", "-r", "target", option],
            Some(DangerousCommandMatch::ForcedRm),
        );
        check(&["rm", "--", option], /*expected*/ None);
    }
    for option in [
        "--fo?",
        "--forces",
        "--format",
        "--",
        "--recursive",
        "-r",
        "-i",
    ] {
        check(&["rm", option, "target"], /*expected*/ None);
    }
}

#[test]
fn issue_2285_quoted_force_flags() {
    for script in [
        "rm $'-rf' target",
        r#"rm $"-rf" target"#,
        r"rm $'\x2dr\146' target",
        r"rm $'\055\x66' target",
        r"rm $'\u002df' target",
        r"rm $'\U0000002df' target",
        r"rm -$'r'$'f' target",
        r#"rm -"$" $'-f' target"#,
        r"rm --fo\rc target",
        r"rm '--fo'rc target",
        r"rm --$'forc' target",
        r"rm -r\f target",
        r"$'rm' $'-rf' target",
        r#"env X=1 rm $"--for" target"#,
        r#"sudo rm $'-rf' target"#,
        r#"if true; then rm $'-rf' target; fi"#,
        r#"echo "$(rm $'-rf' target)""#,
        r#"bash -c "rm \$'-rf' target""#,
        r#"trap "rm \$'-rf' target" EXIT"#,
        r"bash -c $'rm -rf target;'#",
        r"trap $'rm -rf target;'# EXIT",
    ] {
        check(
            &["bash", "-lc", script],
            Some(DangerousCommandMatch::ForcedRm),
        );
    }
}

#[test]
fn issue_2285_non_forced_commands_stay_quiet() {
    for script in [
        "rm target",
        r"bash -c $'rm -r target;'#",
        r"trap $'echo rm -rf target;'# EXIT",
        "rm -r target",
        "rm $'-r' target",
        r#"rm $"-r" target"#,
        r"rm $'\x2dr' target",
        r"rm $'target\nf' other",
        r"rm $'target\0-rf' other",
        r"rm -- $'-rf'",
        r#"rm $"--" -f"#,
        r"rm -- '--forc'",
        r#"echo "rm $'-rf' target""#,
        r#"printf '%s' $'-rf'"#,
        r#"rm "$FLAGS" target"#,
        r#"rm $"$FLAGS" target"#,
        r#"rm $'\\x2df' target"#,
    ] {
        check(&["bash", "-lc", script], /*expected*/ None);
    }
}

#[test]
fn issue_2285_existing_detections_are_not_relaxed() {
    // The old danger-only extraction omits these newly decoded terminators.
    // Keep that conservative match as well as checking decoded spellings.
    for script in ["rm $'--' -f", r"rm \-\- -f"] {
        check(
            &["bash", "-lc", script],
            Some(DangerousCommandMatch::ForcedRm),
        );
    }
}
