//! D713: the forms the minimal lexical set reads precisely besides its
//! literal list, and the shapes around them that still fail closed (ask
//! whenever a prompt or forbidden rule exists). Scripts are read as bash
//! runs them unless a test says otherwise.

use super::ScriptShell;
use super::read_script;
use super::read_script_as;
use super::shell_lc_script_read;
use pretty_assertions::assert_eq;

fn read_posix(script: &str) -> super::ScriptRead {
    read_script_as(script, ScriptShell::Posix)
}

/// The script's commands as text, `?` for a word known only at run time;
/// `None` where the script fails closed.
fn commands(script: &str) -> Option<Vec<String>> {
    let read = read_posix(script);
    (!read.fails_closed()).then(|| {
        read.commands
            .iter()
            .map(|command| {
                command
                    .words
                    .iter()
                    .map(|word| word.as_deref().unwrap_or("?"))
                    .collect::<Vec<_>>()
                    .join(" ")
            })
            .collect()
    })
}

fn assert_read(cases: &[(&str, &[&str])]) {
    for (script, expected) in cases {
        let expected: Vec<String> = expected.iter().map(ToString::to_string).collect();
        assert_eq!(commands(script), Some(expected), "{script:?}");
    }
}

fn assert_fails_closed(scripts: &[&str]) {
    for script in scripts {
        assert!(read_posix(script).fails_closed(), "{script:?}");
    }
}

#[test]
fn parameter_expansions_as_arguments_are_read() {
    assert_read(&[
        ("echo $HOME", &["echo ?"]),
        ("echo \"$HOME/x\" ${HOME}/y $_", &["echo ? ? ?"]),
        ("git push $remote", &["git push ?"]),
        ("x=$y; echo \"$x\"", &["echo ?"]),
        (
            "echo ${HOME:-/tmp} ${x#*/} ${x%%.*} ${#x} ${x##*/} ${x%/*}",
            &["echo ? ? ? ? ? ?"],
        ),
        (
            "echo ${XDG:-$HOME/.config} ${x:+y} ${x:?} ${x-} ${x:=y}",
            &["echo ? ? ? ? ?"],
        ),
        ("echo ${10} ${#1} $x:h $x.y", &["echo ? ? ? ?"]),
        ("for f in *.rs; do wc -l \"$f\"; done", &["wc -l ?"]),
        ("X=$y git push", &["git push"]),
    ]);
}

#[test]
fn expansions_that_run_code_or_name_the_command_fail_closed() {
    assert_fails_closed(&[
        // The command's name.
        "$cmd push",
        "${cmd} push",
        "\"$cmd\" push",
        "$(echo git) push",
        "`echo git` push",
        "$((1)) push",
        "x=1; $x push",
        "true && $cmd push",
        "if true; then $cmd; fi",
        "x+=1 $cmd",
        "2>/dev/null $cmd push",
        "true; 2>&1 ${cmd} push",
        // Not an assignment: bash runs `1=…` as a command.
        "1=$x push",
        // zsh subscripts, which evaluate arithmetic.
        "echo \"$x[1]\"",
        "echo ${x}[1]",
        "echo \"${x}[1]\"",
        "echo $x'y'",
        // Forms outside the list: substrings, replacements, indirection,
        // transformations, zsh flags, quotes or blanks in the word.
        "echo ${x:1}",
        "echo ${x/a/b}",
        "echo ${!x}",
        "echo ${x@P}",
        "echo ${(e)x}",
        "echo ${x^^}",
        "echo ${x:-a b}",
        "echo \"${x#\"a\"}\"",
        "echo ${x#$y}",
        "echo ${x:-$(git push)}",
        "echo ${x:-a}b}",
        // Assigning a variable a shell evaluates.
        "echo ${OPTIND:=x}",
        "echo ${PS4=x}",
    ]);
}

#[test]
fn constant_arithmetic_is_read_and_names_fail_closed() {
    assert_read(&[
        (
            "echo $((1 + 2)) $(( $? * 3 )) $((10/3%2-1))",
            &["echo ? ? ?"],
        ),
        ("x=$(($# + 1)) git push", &["git push"]),
    ]);
    // A name's value is evaluated as an expression: `x='a[$(git push)]'`.
    assert_fails_closed(&[
        "x='a[$(git push)]'; echo $((x))",
        "echo $(($x + 1))",
        "echo $((x++))",
        "echo $(( (1) ))",
        "echo $((1<<2))",
        "echo $((1)",
        "echo $[1+1]",
    ]);
}

#[test]
fn substitutions_are_read_as_scripts() {
    assert_read(&[
        (
            "x=$(git rev-parse HEAD); echo \"$x\"",
            &["git rev-parse HEAD", "echo ?"],
        ),
        (
            "echo $(git log -1 2>/dev/null) \"$(date)\" $(a | b && c)",
            &["echo ? ? ?", "git log -1", "date", "a", "b", "c"],
        ),
        ("echo $(git push)", &["echo ?", "git push"]),
        ("echo \"$(git push)\"", &["echo ?", "git push"]),
        ("echo `git push`", &["echo ?", "git push"]),
        ("echo \"a `git push` b\"", &["echo ?", "git push"]),
        ("x=`git push`", &["git push"]),
        ("echo $(echo `git push`)", &["echo ?", "echo ?", "git push"]),
        (
            "cd \"$(dirname \"$0\")\" && make",
            &["cd ?", "dirname ?", "make"],
        ),
        (
            "echo $(if true; then git push; fi)",
            &["echo ?", "true", "git push"],
        ),
        ("echo $(\ngit push\n)", &["echo ?", "git push"]),
        // bash 3.2 scans a quoted string in `$(…)` as one.
        (
            "echo $(echo 'a)b' \"c)d\" \"it's\" 'say \"x\"' '#\\t`' \"(#)\")",
            &["echo ?", "echo a)b c)d it's say \"x\" #\\t` (#)"],
        ),
    ]);
}

/// bash 3.2 and 4.x find the end of `$(…)` with scans of their own (review
/// rounds 6 and 7 of #2548); nothing they could read differently is read.
#[test]
fn substitutions_scanned_differently_fail_closed() {
    assert_fails_closed(&[
        "echo $(echo a # b\n)",
        "echo $(echo a#b)",
        "echo $(echo x\\ y)",
        "echo $(echo \"a\\\"b\")",
        "echo $(case x in x) git push;; esac)",
        "echo $(echo case)",
        "echo $(echo {})",
        "echo $(cat <<EOF\nx\nEOF\n)",
        "echo $( (git push) )",
        "echo $()",
        "echo $(echo $$(git push))",
        "echo $(echo $'\\'' )\ngit push\n')",
        // Backquotes hold no quotes, escapes or `$(…)`, and do not nest.
        "echo `echo \"x\"`",
        "echo `echo 'x'`",
        "echo `echo \\`git push\\``",
        "echo `echo $(git push)`",
        "echo ``",
        "echo \"`echo \"x\"`\"",
        // Unterminated.
        "echo $(git push",
        "echo `git push",
        // Nine deep.
        "echo $(echo $(echo $(echo $(echo $(echo $(echo $(echo $(echo $(git push)))))))))",
    ]);
    assert_read(&[(
        "echo $(echo $(echo $(echo $(echo $(echo $(echo $(echo $(git push))))))))",
        &[
            "echo ?", "echo ?", "echo ?", "echo ?", "echo ?", "echo ?", "echo ?", "echo ?",
            "git push",
        ],
    )]);
}

#[test]
fn tests_are_read() {
    assert_read(&[
        ("[[ -n $x ]] && git push", &["git push"]),
        ("[[ $x == *.rs ]]; git push", &["git push"]),
        ("[[ $x != y ]] || git push", &["git push"]),
        ("[[ -z ${x:-} ]]", &[]),
        ("[[ x == \"a b\" ]]", &[]),
        ("[[ ! -f x ]]", &[]),
        ("[[ $? -ne 0 ]]", &[]),
        ("if [[ -n \"$x\" ]]; then git push; fi", &["git push"]),
        ("[ -f x ] && git push", &["git push"]),
        ("[ x = \"a b\" ] && echo", &["echo"]),
        ("[ ! -f x -a -d y ]", &[]),
        ("[ $? -eq 0 ] 2>/dev/null", &[]),
        (
            "[ -n \"$x\" ] && [ ! -d \"${d:-x}/y\" ] && [ \"$a\" != \"b$1\" ]",
            &[],
        ),
        ("[ \"$x\" ] || [ ! \"$*\" ] || [ ! -f \"$f\" ]", &[]),
        ("while [ $# -gt 0 ]; do shift; done", &["shift"]),
        ("echo $([ -f x ] && echo y)", &["echo ?", "echo y"]),
    ]);
}

#[test]
fn tests_other_shells_or_operators_read_differently_fail_closed() {
    assert_fails_closed(&[
        // dash has no `[[`: it runs what follows `||` as a command.
        "[[ -n x || git -nt push ]]",
        "[[ -n x && git -nt push ]]",
        "[[ -n x; git push ]]",
        "[[ -n x\n]]",
        "[[ x < y ]]",
        "[[ ( x ) ]]",
        "[[ $x =~ y ]]",
        // No substitutions in tests.
        "[[ $(pwd) == x ]]",
        "[[ `pwd` == x ]]",
        "[[ $((1)) -eq 1 ]]",
        // `[` splits an expansion into words that may become operators
        // (`-v 'a[$(…)]'`): `$?`, `$#`, `$$`, `$!`, or quoted where the
        // operators around it are fixed.
        "[ -n $x ]",
        "[ $1 ]",
        "[ \"$@\" ]",
        "[ ${x} ]",
        "[ \"$x\" \"$y\" ]",
        "[ \"$x\" \"$y\" \"$z\" ]",
        "[ \"$x\" -a \"$y\" ]",
        "[ -t \"$x\" ]",
        "[ -v \"$x\" ]",
        "[ \"$x\" -eq 1 ]",
        "[ ! \"$x\" = y ]",
        "[ -n \"$x\" -o -n y ]",
        "[ -n \"$(pwd)\" ]",
        "[ x == y ]",
        // A quoted operand `[` would read as an operator.
        "[ \"-v\" x ]",
        "[ \\-v x ]",
        "[ ''-v x ]",
        "[ -n x \"-o\" -v y ]",
        "[ \"!\" x ]",
        // What the reader checks: `-v`, and arithmetic on other than numbers.
        "[[ -v x ]]",
        "[[ $x -eq 1 ]]",
        "[ x -gt 0 ]",
        // Review round 1: `[` operands are expanded as file names, which `[`
        // may read as operators (a file named `-v`).
        "[ * ]",
        "[ ! * ]",
        "[ -n x* ]",
        "[ -f a?b ]",
        "[ x = a^b ]",
        "[ x = a#b ]",
        "[ -f \\* ]",
        // A test is all of its command.
        "[ x ] y",
        "[[ x ]] y",
        "[ x ]]",
        "[x ]",
        "[ x",
        "[[ x ]",
    ]);
}

/// Found while measuring D713, on main since #2613: tree-sitter-bash joins
/// a word that starts with `\` to the command on the line before, while
/// bash runs it as a command of its own.
#[test]
fn a_line_starting_with_a_backslash_fails_closed() {
    assert_fails_closed(&[
        "true\n\\git push",
        "x=1\n\\git push",
        "true # note\n\\git push",
        "true\n\n\\git push",
        "true &&\n\\git push",
    ]);
    assert_read(&[
        ("\\git push", &["git push"]),
        ("true; \\git push", &["true", "git push"]),
        ("true\n \\git push", &["true", "git push"]),
    ]);
}

/// Review round 1: with options a user may set in zsh's startup files, an
/// unquoted parameter or substitution becomes a pattern whose qualifiers
/// run code (`glob_subst`: `x='*(e:cmd:)'; echo $x`; `glob_assign`:
/// `y=$x`), or splits into words. Where zsh (or a shell not known) runs
/// the script, only quoted ones are read.
#[test]
fn unquoted_expansions_fail_closed_where_zsh_may_run_them() {
    let argv =
        |shell: &str, script: &str| vec![shell.to_string(), "-lc".to_string(), script.to_string()];
    for script in [
        "x='*(e:git push:)'; echo $x",
        "echo ${x#a}",
        "y=$x",
        "x=$(git rev-parse HEAD)",
        "echo `pwd`",
        "for f in $(ls); do wc -l \"$f\"; done",
        "[[ -n $x ]]",
        "set -- '*(e:git push:)'; echo $1",
        "echo \"$(echo $x)\"",
    ] {
        assert!(read_script(script).fails_closed(), "{script:?}");
        for shell in ["zsh", "/bin/zsh", "ksh", "fish"] {
            let read = shell_lc_script_read(&argv(shell, script));
            assert!(
                read.is_none_or(|read| read.fails_closed()),
                "{shell} {script:?}"
            );
        }
        for shell in ["bash", "/bin/sh", "sh"] {
            let read = shell_lc_script_read(&argv(shell, script)).expect(shell);
            assert!(!read.fails_closed(), "{shell} {script:?}");
        }
    }
    // Quoted, arithmetic and `$?`-like values are read for zsh too.
    for script in [
        "echo \"$HOME\" \"${x:-a}\" \"$(git rev-parse HEAD)\" \"`pwd`\"",
        "echo $((1 + 2)) $? $#",
        "[ -n \"$x\" ] && [[ \"$x\" == y ]]",
    ] {
        assert!(!read_script(script).fails_closed(), "{script:?}");
        let read = shell_lc_script_read(&argv("zsh", script)).expect("zsh");
        assert!(!read.fails_closed(), "{script:?}");
    }
}
