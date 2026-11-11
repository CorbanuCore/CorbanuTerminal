use super::*;
use pretty_assertions::assert_eq;

/// Each command's words as (text, data), substituted commands marked `$()`.
fn words(text: &str) -> Vec<Vec<(String, bool)>> {
    read(text)
        .expect("readable")
        .commands
        .into_iter()
        .map(|command| {
            let mut words: Vec<(String, bool)> = command
                .words
                .into_iter()
                .map(|word| (word.text, word.data))
                .collect();
            if command.substituted {
                words.insert(0, ("$()".to_string(), true));
            }
            words
        })
        .collect()
}

/// A word that is data.
fn d(text: &str) -> (String, bool) {
    (text.to_string(), true)
}

#[test]
fn quoted_and_plain_words_are_data() {
    assert_eq!(
        words(r#"git commit -m 'a; b $(c)' "d e" f=g:h/i,j"#),
        vec![vec![
            d("git"),
            d("commit"),
            d("-m"),
            d("a; b $(c)"),
            d("d e"),
            d("f=g:h/i,j"),
        ]]
    );
}

#[test]
fn expansions_globs_and_escapes_are_not_data() {
    let line = r#"echo "$x" "a\"b" `id` $'\x41' ~/f *.rs {a,b} =ls a\ b"#;
    let data: Vec<bool> = words(line)[0].iter().map(|(_, data)| *data).collect();
    assert_eq!(
        data,
        [
            true, false, false, false, false, false, false, false, false, false
        ]
    );
}

#[test]
fn parts_with_nothing_between_them_are_one_word() {
    let reading = read("cat ~/$a$b/c.toml $\"t\" x a#b").expect("readable");
    let words: Vec<(bool, bool)> = reading.commands[0]
        .words
        .iter()
        .map(|word| (word.data, word.dynamic))
        .collect();
    assert_eq!(
        words,
        [
            (true, false),
            (false, true),
            (false, true),
            (true, false),
            (false, false)
        ]
    );
    assert_eq!(reading.commands[0].words[4].text, "a#b");
}

#[test]
fn substitutions_are_commands_of_their_own() {
    let line = "a $(b \"$(c)\") <(d) <<EOF | e\n$(f)\nEOF\ng() { h; }; for i in j; do k; done; export l=$(m)";
    let names: Vec<(bool, String)> = read(line)
        .expect("readable")
        .commands
        .into_iter()
        .map(|command| (command.substituted, command.words[0].text.clone()))
        .collect();
    let expected: Vec<(bool, String)> = [
        (false, "a"),
        (true, "b"),
        (true, "c"),
        (true, "d"),
        (false, "e"),
        (true, "f"),
        (false, "h"),
        (false, "k"),
        (false, "export"),
        (true, "m"),
    ]
    .into_iter()
    .map(|(substituted, name)| (substituted, name.to_string()))
    .collect();
    assert_eq!(names, expected);
}

#[test]
fn unreadable_text_is_refused() {
    for text in ["print('ok')", "+.,/()[]"] {
        assert!(read(text).is_err(), "{text:?}");
    }
    for (text, refusal) in [
        ("echo $(", Refusal::Syntax),
        ("echo 'a", Refusal::Syntax),
        ("if x; then y", Refusal::Syntax),
        ("echo a\u{1}b", Refusal::Characters),
        ("echo \u{e9}|sh", Refusal::Characters),
        ("echo \u{e9};id", Refusal::Characters),
        ("echo a\\\nb", Refusal::LineContinuation),
        ("done x", Refusal::Words),
        ("coproc id", Refusal::Words),
        ("} x", Refusal::Words),
        ("[v]", Refusal::Words),
        ("--expr=1e x", Refusal::Words),
        ("\\ =git x", Refusal::Words),
        ("echo;; id", Refusal::Syntax),
        ("cat <<<\nx", Refusal::Syntax),
    ] {
        assert_eq!(read(text).err(), Some(refusal), "{text:?}");
    }
    assert_eq!(
        read(&"a".repeat(MAX_BYTES + 1)).err(),
        Some(Refusal::TooLarge)
    );
    // Non-ASCII text is read when nothing a shell may join to it follows.
    assert!(read("echo r\u{e9}sum\u{e9} x").is_ok());
}

#[test]
fn deep_nesting_is_refused_not_walked() {
    let depth = 500;
    let text = format!("{}x{}", "echo $(".repeat(depth), ")".repeat(depth));
    assert!(text.len() <= MAX_BYTES);
    assert_eq!(read(&text).err(), Some(Refusal::TooDeep));
}

#[test]
fn zsh_differences_are_flagged() {
    for text in [
        "repeat 2 echo x",
        "echo ${(e)x}",
        "echo $x[1] $y",
        "echo $x:h",
        "{ echo }",
        "builtin print x",
    ] {
        // Flagged, or not read at all.
        assert!(
            read(text).ok().is_none_or(|reading| reading.zsh_unsure),
            "{text:?}"
        );
    }
    for text in ["echo a b", "git log -- x", "f() { g; }", "echo \"$x\" ${y}"] {
        assert!(
            read(text).is_ok_and(|reading| !reading.zsh_unsure),
            "{text:?}"
        );
    }
}

#[test]
fn parse_work_is_bounded_by_the_budget() {
    // Error recovery on unbalanced text is where a parse could do the most
    // work; it stops at the first error, within the budget.
    for unit in [
        "$((",
        "'\"`",
        "case x in (",
        "${x:-'",
        "a=( ",
        "x | ",
        "f() ",
    ] {
        let text = unit.repeat(MAX_BYTES / unit.len());
        match read(&text) {
            Ok(reading) => assert!(reading.parse_checks <= parse_budget(text.len())),
            Err(refusal) => assert!(
                matches!(
                    refusal,
                    Refusal::Syntax | Refusal::ParseBudget | Refusal::TooDeep
                ),
                "{unit:?}: {refusal:?}"
            ),
        }
    }
}

#[test]
fn here_documents_the_scanner_may_forget_are_refused() {
    let nine = "cat <<a\nx\na\n".repeat(9);
    assert_eq!(read(&nine).err(), Some(Refusal::HereDocuments));
    let long = format!("cat <<{}\nx\n", "d".repeat(MAX_DELIMITER_BYTES + 1));
    assert_eq!(read(&long).err(), Some(Refusal::HereDocuments));
    assert!(read(&"cat <<a\nx\na\n".repeat(8)).is_ok());
    // An extended glob pattern bash refuses by default.
    assert_eq!(
        read("case x in @(a|b)) ;; esac").err(),
        Some(Refusal::Unsupported)
    );
}

#[test]
fn here_documents_end_where_the_shells_end_them() {
    for text in [
        // A line that only starts with the delimiter does not end it.
        "cat <<'EOF'\nEOF; cat <<'Z'\nEOF\necho x\nZ\n",
        // Quotes inside the delimiter are removed by the shells.
        "cat <<E\"O\"F\nx\nEOF\necho x\nE\"O\"F\n",
        "cat <<E\\OF\nx\nEOF\n",
        // `<<-` strips tabs only.
        "cat <<-EOF\n  EOF\necho x\nEOF\n",
        // Backquotes and backslashes in an unquoted body.
        "cat <<EOF\n`echo x`\nEOF\n",
        "cat <<EOF\n\\$(echo x)\nEOF\n",
        // Two on one line, or one without its end.
        "cat <<A <<B\na\nA\nb\nB\n",
        "cat <<EOF\nx\n",
    ] {
        assert!(read(text).is_err(), "{text:?}");
    }
    let reading =
        read("cat <<EOF | sh\na $(b)\nEOF\ncat <<'Q'\n$(c) `d`\nQ\n\tcat <<-\"R\"\n\tr\n\tR\n")
            .expect("readable");
    let names: Vec<&str> = reading
        .commands
        .iter()
        .map(|command| command.words[0].text.as_str())
        .collect();
    assert_eq!(names, ["cat", "sh", "b", "cat", "cat"]);
}

#[test]
fn expansions_inside_patterns_are_refused() {
    for text in [
        "x=a; echo ${x#$(echo x)}",
        "echo ${x%`id`}",
        "echo ${x/a$(id)/y}",
        "echo ${x,,$y}",
        "echo $(echo == $(id))",
    ] {
        assert_eq!(read(text).err(), Some(Refusal::Words), "{text:?}");
    }
    assert!(read("echo ${x#a} ${x%.rs}").is_ok());
}

#[test]
fn words_a_redirection_or_a_quote_would_hide_are_refused() {
    for text in [
        // The grammar gives the redirection every word after it.
        "rm >/dev/null -rf ~/x",
        "cat >&- foo",
        "cat <<EOF -n\nx\nEOF\n",
        // Single quotes are ordinary characters in these places.
        "echo \"${x:-'$(id)'}\"",
        "cat <<EOF\n${x:-'$(id)'}\nEOF\n",
        "echo $(( '$(id)' ))",
        "(( '$(id)' ))",
        // A variable to hold a descriptor.
        "echo {x}>f",
    ] {
        assert!(read(text).is_err(), "{text:?}");
    }
    assert!(read("echo >/dev/null 2>&1; cat <<'EOF' >f\nx\nEOF\n").is_ok());
    assert!(read("echo ${x:-'a'} '$(id)'").is_ok());
}

#[test]
fn commands_that_change_how_lines_are_read_are_refused() {
    for text in [
        "shopt -s expand_aliases",
        "builtin alias l=ls",
        "command setopt globsubst",
        "$c x",
        "\"$c\" x",
    ] {
        assert_eq!(read(text).err(), Some(Refusal::Reparse), "{text:?}");
    }
}

#[test]
fn tokens_the_shells_split_are_refused() {
    for text in [
        "(echo == x; rm -rf ~)",
        "echo $(echo == x; rm -rf ~)",
        "cat <(echo =~ x; id)",
        "[ a > b ]",
        "[ x == a; rm -rf ~ ]",
        "[[ x =~ a&&b ]]",
        "[[ x =~ '([['a ]] || echo x '' ]]",
        "[[ x =~ '('a\n]] || echo x ]]",
        "{echo,hi}\n}",
    ] {
        assert!(read(text).is_err(), "{text:?}");
    }
    // Blanks inside a test's pattern or `${...}` are the shells' too.
    assert!(read("[[ $x =~ a\\ b ]] && echo ${x/ /_}").is_ok());
    assert!(read("[[ $x =~ ^(a|b c)$ ]] && [[ $y == 'p q' ]]").is_ok());
    // Only a C-style `for`'s header is arithmetic.
    assert!(read("for ((i=0; i<2; i++)); do echo '$x'; done").is_ok());
}

#[test]
fn set_is_read_only_with_harmless_options() {
    for text in [
        "export A=$(date) B=\"$x\"; printf '%s' \"$y\"",
        "set -euo pipefail; echo x",
        "set -e +u",
        "set -- a b",
        "set",
    ] {
        assert!(read(text).is_ok(), "{text:?}");
    }
    for text in [
        "set -k; git fetch origin X=y",
        "set -o posix",
        "builtin set -o keyword",
        "set -o",
        "SHELLOPTS=keyword bash -c x",
        "env BASHOPTS=expand_aliases bash -c x",
        "POSIXLY_CORRECT=1",
        "export POSIXLY_CORRECT",
        "BASH_ALIASES[ls]='rm x'",
        "BASH_CMDS[ls]=/bin/rm",
        "command {shopt,-s,expand_aliases}",
        "command {alias,ls=echo\\ x}",
        "c=shopt; command $c -s expand_aliases",
        "export \"$v=1\"",
        "read \"$v\"",
        ": ${POSIXLY_CORRECT:=1}",
        "declare -n p=BASH_ALIASES",
        "command -p set -o posix",
        "command -- set -o posix",
        "time -p set -o posix",
        "PS4='$(id)'; set -e; ls",
        "set -x",
        "printf -v \"$n\" %s x",
        "declare -n q=POSIX\"LY_CORRECT\"",
        "declare -n q=x",
        "export BASH_\"ENV\"=f",
    ] {
        assert_eq!(read(text).err(), Some(Refusal::Reparse), "{text:?}");
    }
}

#[test]
fn quotes_some_shells_do_not_honour_are_refused() {
    for text in [
        // The first backquote ends the substitution, in quotes or a comment.
        "echo `: #`;rm -rf ~\n` #`",
        "echo `echo '`;rm -rf ~;echo '` #`",
        // zsh and dash end arithmetic and double-quoted `${...}` at a quoted
        // `))` or `}`.
        "false && (( ')) ; rm -rf ~ ; (( ' ))",
        "false && echo $(( ')) ; rm -rf ~ ; echo $(( ' ))",
        "echo \"${x:-'}\"; rm -rf ~; echo \"'}\"",
        "x=POSIXLY_CORRE; : $(( ${x}CT = 1 ))",
        "x=POSIXLY_CORRE; : $(( ${x}CT <<= 1 ))",
        "for (( i=$((1)); i<\"))\"; i++ )) do :; done",
    ] {
        assert!(read(text).is_err(), "{text:?}");
    }
    assert!(read("echo `date` \"${x:-a}\" $((1 + 2)) $(($n >= 2))").is_ok());
}

#[test]
fn text_dash_reads_differently_is_flagged() {
    for text in [
        "[[ -z x || sh -ef p.sh ]]",
        "(( x ))",
        "cat <<<x",
        "diff <(a) <(b)",
    ] {
        assert!(
            read(text).is_ok_and(|reading| reading.bash_only),
            "{text:?}"
        );
    }
    assert!(read("[ -z x ] || echo y").is_ok_and(|reading| !reading.bash_only));
}

#[test]
fn nested_process_substitutions_are_read_in_linear_work() {
    work::assert_linear("grammar process substitutions", |scale| {
        let depth = 10 * scale;
        let _ = read(&format!(
            "{}x{}",
            "export a=<(".repeat(depth),
            ")".repeat(depth)
        ));
    });
}

#[test]
fn nested_expansions_are_read_in_linear_work() {
    work::assert_linear("grammar nesting", |scale| {
        let depth = 10 * scale;
        let _ = read(&format!(
            "echo {}x{}",
            "${x:-$(echo ".repeat(depth),
            ")}".repeat(depth)
        ));
    });
}

#[test]
fn reading_work_is_linear() {
    work::assert_linear("grammar", |scale| {
        let _ = read(&"a $(b 'c' \"$d\") | e; ".repeat(20 * scale));
    });
}
