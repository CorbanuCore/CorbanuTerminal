//! #2548: the reader accepts only shapes it fully understands, reports where
//! each command's input comes from, and reads bash's reserved prefixes.

use super::CommandStdin;
use super::parse_shell_lc_single_command_prefix;
use super::read_script;
use super::script_read::read_tree;
use pretty_assertions::assert_eq;

fn words(script: &str) -> Vec<Vec<Option<String>>> {
    read_script(script)
        .commands
        .into_iter()
        .map(|command| command.words)
        .collect()
}

fn known(words: &[&str]) -> Vec<Option<String>> {
    words.iter().map(|word| Some(word.to_string())).collect()
}

/// The input of the `bash` command in `script`.
fn bash_stdin(script: &str) -> CommandStdin {
    let read = read_script(script);
    let mut found = read
        .commands
        .into_iter()
        .filter(|command| command.words.first() == Some(&Some("bash".to_string())));
    let command = found.next().expect(script);
    assert_eq!(found.next(), None, "{script:?}");
    command.stdin
}

#[test]
fn stdin_is_literal_only_for_a_commands_own_literal_input() {
    let literal = |text: &str| CommandStdin::Literal(text.to_string());
    for (script, stdin) in [
        ("bash <<EOF\ngit push\nEOF", literal("git push\n")),
        ("bash <<'EOF'\n$cmd\nEOF", literal("$cmd\n")),
        ("bash <<-'EOF'\n\tgit push\n\tEOF", literal("git push\n")),
        ("bash <<< 'git push'", literal("git push\n")),
        (
            "{ bash <<EOF\ngit push\nEOF\n} < /dev/null",
            literal("git push\n"),
        ),
        ("bash <<EOF\n$cmd\nEOF", CommandStdin::Redirected),
        ("bash <<EOF\n`cmd`\nEOF", CommandStdin::Redirected),
        ("bash <<< \"$x\"", CommandStdin::Redirected),
        ("echo hi | bash", CommandStdin::Redirected),
        // The grammar puts this here-document on the whole pipeline, so it
        // is not taken as the shell's text.
        (
            "echo hi | bash <<EOF\ngit push\nEOF",
            CommandStdin::Redirected,
        ),
        ("bash < file", CommandStdin::Redirected),
        ("bash <&3", CommandStdin::Redirected),
        ("bash 0<file", CommandStdin::Redirected),
        (
            "bash <<EOF < file\necho safe\nEOF",
            CommandStdin::Redirected,
        ),
        // Any redirection onto fd 0 replaces the literal input (review r1).
        ("bash <<< 'echo hi' 0<&3", CommandStdin::Redirected),
        ("bash <<< 'echo hi' {fd}<file", CommandStdin::Redirected),
        ("coproc bash", CommandStdin::Redirected),
        ("echo hi | (bash)", CommandStdin::Inherited),
        ("{ bash; } <<EOF\ngit push\nEOF", CommandStdin::Inherited),
        (
            "f() { bash; }; f <<EOF\ngit push\nEOF",
            CommandStdin::Inherited,
        ),
        ("bash", CommandStdin::Inherited),
        ("bash > log", CommandStdin::Inherited),
        ("bash 2> log 3< file", CommandStdin::Inherited),
        ("bash | cat", CommandStdin::Inherited),
    ] {
        assert_eq!(bash_stdin(script), stdin, "{script:?}");
    }
}

#[test]
fn reserved_prefixes_are_read_in_any_order() {
    for script in [
        "time ! git push",
        "! time git push",
        "time -p -- git push",
        "! ! git push",
        "time ! time -p ! git push",
        "coproc git push",
        "time coproc git push",
    ] {
        let read = read_tree(script);
        assert!(!read.unread, "{script:?}");
        assert_eq!(
            read.commands
                .iter()
                .map(|command| command.words.clone())
                .collect::<Vec<_>>(),
            vec![known(&["git", "push"])],
            "{script:?}"
        );
    }
    // The minimal lexical set has no `coproc`.
    assert!(read_script("coproc git push").fails_closed());
    // Quoted, they are programs (`'time'` is /usr/bin/time).
    assert_eq!(
        words("'time' git push"),
        vec![known(&["time", "git", "push"])]
    );
}

#[test]
fn shapes_not_fully_understood_are_unread() {
    for script in [
        // Arithmetic evaluates subscripts in variables' values.
        "echo $((x))",
        "((x++))",
        "for ((i=0; i<2; i++)); do :; done",
        "[[ $x -eq 1 ]]",
        "[ \"$n\" -gt 0 ]",
        "[[ -v x ]]",
        "echo ${x:1}",
        "echo ${a[1]}",
        "echo ${!x}",
        "echo ${x@P}",
        "echo ${(e)x}",
        "a[$i]=1",
        "a=([1]=x)",
        "declare -i n",
        "declare -n r=x",
        "local -a a",
        "export \"$x\"",
        "unset 'a[1]'",
        "unset \"$x\"",
        // Backquotes nest by escaping, which the grammar misreads.
        "echo `echo \\`git push\\``",
        // Variables that run code.
        "BASH_ENV=./x.sh bash -c true",
        "export PS4='$(git push)'",
        "SHELLOPTS=xtrace true",
        // Reserved words the reader does not strip.
        "coproc NAME { git push; }",
        "repeat 2 git push",
        // Words after a here-document's start nest in it.
        "bash <<'A' -c 'git push'\necho safe\nA",
        "git <<'A' push --force\nA",
        // Here-documents the grammar may end in the wrong place.
        "cat <<E\"O\"F\necho safe\nE\"O\"F",
        "cat <<EOF\nx\nEO\\\nF\ngit push\nEOF",
        "cat <<EOF\necho safe\nEOFX\necho later\nEOF",
        // The grammar takes this `0` for an argument after the start marker.
        "bash <<'EOF' 0>&3\necho hi\nEOF",
        // Backquotes the grammar leaves inside tokens (review r1).
        "cat <<EOF\n`git push`\nEOF",
        "cat <<EOF > notes.md\n`git push`\nEOF",
        "[[ x =~ (`git push`) ]]",
        "case x in ?a`{git,push}`) ;; esac",
        // Integer specials evaluate what is assigned (review r1).
        "OPTIND='a[$(git push)]'",
        "for SECONDS in x; do :; done",
        "export HISTCMD=1",
        "echo ${OPTIND:=x}",
        "coproc OPTIND='a[$(git push)]'",
        "time BASH_ENV=./x.sh bash -c true",
        // A comment where bash sees none.
        "x=(a)#; git push",
        "echo a\\\n#b; git push",
        // Review round 2: the grammar and bash split these differently.
        "git 2>&1 push origin",
        "echo a\\\r\ngit push",
        "git pu\\\nsh",
        "(echo == x; git push)",
        "echo $(echo =~ x; git push)",
        "echo ${x#{}; git push; echo }",
        "[[ -t 'path[$(git push)]' ]]",
        "[ -t x ]",
        "echo $HOME['$(git push)']",
        "echo \"$HOME[1]\"",
        "nocorrect X=1 git push",
        "NULLCMD=reboot",
        // Review round 3.
        "cat <<'Q' &&\nQ\ngit push\nQ",
        "cat <<'Q' && echo $(\ncat <<EOF\n$(echo '\nEOF\ngit push\n#')\nEOF\nQ\n)\nQ",
        "[ x | git push ]",
        "commands=(ls /usr/bin/git)",
        "functions+=(x y)",
        "PROMPT4='$(git push)'",
        // `[` ends at a newline for bash.
        "[\n! sh -c 'git push' ]",
        "[ x < y ]",
        "[ ( x ) ]",
        // Review round 4.
        "[ x != ab<(git$IFS'push') ]",
        "case x in ab<(git$IFS'push')) ;; esac",
        "cat <<'A' && bash <<'A'\necho hi\nA\ngit push\nA",
        "[ x |=sh -c 'git push' ]",
        "[[ x |= y ]]",
        "cat <<Q\nz $#a[x]\nQ",
        "echo \"$#a[x]\"",
        "echo $(case x in x) echo;; esac)",
        "coproc nocorrect X=1 git push",
        "echo /(e:'git push':)",
        "X==(git push)",
        "precmd_functions=(x)",
        // D713: `[` splits expansions into words that may become operators,
        // and a quoted operand may read as one.
        "[ -n \"$(git status --porcelain)\" ]",
        "[ -n $x ]",
        "[ \"$a\" \"$b\" ]",
        "[ x \"-eq\" 1 ]",
        // An operator where operands are arithmetic is one too (`-n`
        // negates the value of `n`).
        "[[ 1 -eq -n ]]",
        "[ 1 -eq 1 -a x ]",
        // Does not parse.
        "git push (",
        "echo *(e:'git push':)",
    ] {
        assert!(read_script(script).fails_closed(), "{script:?}");
    }
}

#[test]
fn understood_shapes_are_read() {
    for script in [
        "git status",
        "x=$(git log -1); echo \"$x\"",
        "for f in *.rs; do wc -l \"$f\"; done",
        "while read -r line; do echo \"$line\"; done < file",
        "if [ -f x ]; then echo a; elif [[ -d y && -n $z ]]; then :; else exit 1; fi",
        "case $x in a|b) echo a;; *) :;; esac",
        "f() { echo \"$1\"; }; f x",
        "export PATH=\"$HOME/bin:$PATH\"; local x=1; readonly R=2; declare -x A",
        "unset x y",
        "echo ${x:-default} ${x:=b} \"${@}\" $1 $?",
        "[ $? -eq 0 ] && [[ \"$#\" -gt 1 ]]",
        "cat <<'EOF' > out.txt\nhello $x\nEOF",
        "diff <(sort a) <(sort b) 2>&1 | head",
        "cmd & other; wait",
        "a=(x y \"$z\")",
        "set -euo pipefail",
        "cat <<'EOF' > f.py && python3 f.py\nprint(1)\nEOF",
        "cat <<'EOF' | python3\nprint(1)\nEOF",
        "cat <<'EOF'\nuse `code` and $(this) literally\nEOF",
        "true;#note",
        "a+=(c); f() { :; }; [[ -n $x || -z $y ]]",
        "case $x in a|b*) echo;; esac",
        "[ -t 1 ] && [[ -t 0 ]]",
    ] {
        let read = read_tree(script);
        assert!(!read.unread, "{script:?}: {read:?}");
    }
}

#[test]
fn unread_parts_still_report_their_commands() {
    assert_eq!(
        words("echo $((1)) && git push; for ((;;)); do git push --force; done"),
        vec![
            vec![Some("echo".to_string()), None],
            known(&["git", "push"]),
            known(&["git", "push", "--force"]),
        ]
    );
}

#[test]
fn ambiguous_heredocs_are_not_reduced_to_one_command() {
    for script in [
        "cat <<E\"O\"F\necho safe\nE\"O\"F",
        "cat <<EOF\nx\nEO\\\nF\ngit push\nEOF",
        "bash <<'A' -c 'git push'\necho safe\nA",
    ] {
        let argv = vec!["sh".to_string(), "-c".to_string(), script.to_string()];
        assert_eq!(
            parse_shell_lc_single_command_prefix(&argv),
            None,
            "{script:?}"
        );
    }
}

/// The lexical pre-filter (`script_lexical`): shapes the syntax tree reads,
/// but outside the safe set, fail closed.
#[test]
fn lexically_unsafe_scripts_are_unread() {
    for script in [
        "[[ $x =~ ^a(b|c)$ ]]",
        "echo ${x#*/} ${x%%.*} ${#x} ${x/a/b}",
        "echo a \\\n  b",
        "echo \"a\\\nb\"",
        "ls /usr/bin/gi[t]",
        "echo {a,b}",
        "echo $((x + 2))",
        "cat <<EOF\n$(date)\nEOF",
        "echo na\u{ef}ve",
        "echo a\u{a0}b",
        "echo a\r",
        "cat <<E\"O\"F\nx\nE\"O\"F",
    ] {
        assert!(read_script(script).fails_closed(), "{script:?}");
    }
}

/// Shapes outside the minimal lexical set (most of which the syntax tree
/// reads): they ask whenever a prompt or forbidden rule exists.
#[test]
fn minimal_set_refuses_more_than_the_tree() {
    for script in [
        "echo $'\\\\' ; git push ; echo ' #'",
        "echo hi &> log",
        "echo hi |& cat",
        "cat <<< hi",
        "cat <(echo hi)",
        "cat <<'EOF'\nhi\nEOF",
        "bash <<EOF\necho safe\nEOF",
        "case x in a) echo;; esac",
        "f() { echo; }",
        "( cd x && make )",
        "{ echo; }",
        "a=(x y)",
        "true;#note",
        "cmd >/dev/null 2>&1",
        "cmd > \"out\"",
        // D713: expansions only as arguments, tests only on one line.
        "$cmd push",
        "x=$(git log -1) $x",
        "[ \"$a\" \"$b\" ]",
        "[[ -n x || -z y ]]",
        "sleep 1 & wait",
        "\"\"=git push",
        // Review round 8: an escaped blank, then `#`.
        "echo \\ #; git push",
        "echo x\\\t#; git push",
        "true \\ #&& git push",
        "echo \"na\u{ef}ve\"",
    ] {
        assert!(read_script(script).fails_closed(), "{script:?}");
    }
    for script in [
        "git status && echo ok",
        "cargo build 2>&1 | tail -n 5",
        "ls > /dev/null; echo \"exit=$?\" $1 \"$$\"",
        "find . -name '*.rs' -exec wc -l {} \\;",
        "if test -f x; then echo a; else echo b; fi # done",
        "for f in a b; do wc -l f; done",
        "echo 'na\u{ef}ve $HOME $(x)' a\\b",
        // D713.
        "if [ -f x ]; then echo a; fi",
        "[[ -n x ]]",
        "echo ${x:-default}",
        "x=$(git log -1); echo \"$x\"",
        "echo \"$HOME\" \"${HOME}\"",
        "for f in *.rs; do wc -l \"$f\"; done",
    ] {
        let read = super::read_script_as(script, super::ScriptShell::Posix);
        assert!(!read.fails_closed(), "{script:?}");
    }
}

/// Where `time` is not a keyword (dash) it is `/usr/bin/time`: one followed
/// by its own options stays a word for the launcher reading (review r6).
#[test]
fn time_with_options_stays_a_program() {
    assert_eq!(
        words("time -l git push"),
        vec![known(&["time", "-l", "git", "push"])]
    );
    assert_eq!(words("time -p git push"), vec![known(&["git", "push"])]);
    // zsh's extended_glob makes `#` and `^` glob characters in a name.
    assert_eq!(
        words("gi^t push"),
        vec![vec![None, Some("push".to_string())]]
    );
    assert!(read_script("gi#t push").fails_closed());
}
