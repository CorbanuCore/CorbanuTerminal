use super::ScriptCommands;
use super::parse_shell_lc_single_command_prefix;
use super::read_script_commands;
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
        ("{ bash; } <<EOF\ngit push\nEOF", None),
        ("exec <<EOF\ngit push\nEOF\nbash", None),
        ("f() { bash; }; f <<EOF\ngit push\nEOF", None),
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
fn mixed_delimiters_fail_closed_for_the_whole_script() {
    for delimiter in ["E\"O\"F", "E'O'F", "'E\\OF'", "\"E\\OF\"", "E\\OF"] {
        let script = format!("cat <<{delimiter}\necho safe\n{delimiter}");
        assert_eq!(read_script_commands(&script), ScriptCommands::Unreadable);
        let argv = vec!["sh".into(), "-c".into(), script];
        assert_eq!(parse_shell_lc_single_command_prefix(&argv), None);
    }
}

#[test]
fn nonexact_terminators_fail_closed() {
    for end in ["EOFX", " EOF", "\tEOF", "EOF "] {
        let script = format!("cat <<EOF\necho safe\n{end}\necho later\nEOF");
        assert_eq!(read_script_commands(&script), ScriptCommands::Unreadable);
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
