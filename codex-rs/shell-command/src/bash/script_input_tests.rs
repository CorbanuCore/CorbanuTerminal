use super::script_input::ScriptInput;
use super::shell_lc_script_inputs;
use pretty_assertions::assert_eq;

#[test]
fn stdin_syntax_keeps_literal_bodies_and_unknown_producers_distinct() {
    for (script, literal) in [
        ("bash <<EOF\ngit push\nEOF", Some("git push\n")),
        ("bash <<'EOF'\n$cmd\nEOF", Some("$cmd\n")),
        ("bash <<EOF\n$cmd\nEOF", None),
        ("bash <<EOF\n`cmd`\nEOF", None),
        ("echo hi | bash", None),
        ("echo hi | (bash)", None),
        ("cat <<EOF | bash\ngit push\nEOF", None),
        ("bash <<< 'git push'", None),
        ("bash < file", None),
        ("bash <<EOF < file\necho safe\nEOF", None),
    ] {
        let argv = vec!["bash".into(), "-c".into(), script.into()];
        let inputs = shell_lc_script_inputs(&argv);
        let shell_inputs: Vec<_> = inputs
            .into_iter()
            .filter(|input| input.words.first() == Some(&Some("bash".to_string())))
            .collect();
        assert_eq!(
            shell_inputs,
            vec![ScriptInput {
                words: vec![Some("bash".into())],
                literal: literal.map(str::to_string),
            }],
            "{script:?}"
        );
    }
}

#[test]
fn output_redirections_and_first_pipeline_command_do_not_supply_stdin() {
    for script in ["bash > log", "bash 2> log", "bash | cat", "bash"] {
        let argv = vec!["sh".into(), "-c".into(), script.into()];
        assert!(
            shell_lc_script_inputs(&argv)
                .into_iter()
                .all(|input| input.words.first() != Some(&Some("bash".to_string()))),
            "{script:?}"
        );
    }
}
