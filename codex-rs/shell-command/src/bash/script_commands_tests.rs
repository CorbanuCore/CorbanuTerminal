//! #2543: [`read_script`] reads quoted words as bash does, and never reads
//! a word bash still changes as written.

use super::script_read::read_tree;
use super::shell_lc_script_read;
use pretty_assertions::assert_eq;

/// What these tests check of a [`super::ScriptRead`].
#[derive(Debug, PartialEq, Eq)]
enum ScriptCommands {
    Plain(Vec<Vec<String>>),
    Partial(Vec<Vec<Option<String>>>),
    Unreadable,
}

/// By the syntax tree alone: the lexical pre-filter is tested in
/// `script_read_tests`.
fn read_script_commands(script: &str) -> ScriptCommands {
    summary(read_tree(script))
}

fn shell_lc_script_commands(command: &[String]) -> Option<ScriptCommands> {
    shell_lc_script_read(command).map(summary)
}

fn summary(read: super::ScriptRead) -> ScriptCommands {
    let words = read.commands.into_iter().map(|command| command.words);
    if read.unread {
        ScriptCommands::Unreadable
    } else if read.plain {
        ScriptCommands::Plain(
            words
                .map(|words| words.into_iter().flatten().collect())
                .collect(),
        )
    } else {
        ScriptCommands::Partial(words.collect())
    }
}

fn plain(commands: &[&[&str]]) -> ScriptCommands {
    ScriptCommands::Plain(
        commands
            .iter()
            .map(|words| words.iter().map(ToString::to_string).collect())
            .collect(),
    )
}

/// `None` words are written `?`.
fn partial(commands: &[&[&str]]) -> ScriptCommands {
    ScriptCommands::Partial(
        commands
            .iter()
            .map(|words| {
                words
                    .iter()
                    .map(|word| (*word != "?").then(|| word.to_string()))
                    .collect()
            })
            .collect(),
    )
}

#[test]
fn quoted_words_read_as_bash_runs_them() {
    for (script, expected) in [
        ("git push", plain(&[&["git", "push"]])),
        ("'git' push", plain(&[&["git", "push"]])),
        ("\"git\" push", plain(&[&["git", "push"]])),
        ("g'i't push", plain(&[&["git", "push"]])),
        ("g\"i\"t p'u'sh", plain(&[&["git", "push"]])),
        ("'/usr/bin/git' push", plain(&[&["/usr/bin/git", "push"]])),
        (
            "echo hi; 'git' push",
            plain(&[&["echo", "hi"], &["git", "push"]]),
        ),
        (
            "echo hi && \"git\" push || true | 'cat'",
            plain(&[&["echo", "hi"], &["git", "push"], &["true"], &["cat"]]),
        ),
        ("g\\it push", plain(&[&["git", "push"]])),
        ("\"g\\it\" push", plain(&[&["g\\it", "push"]])),
        ("\"a\\\"b\\\\c\\$d\"", plain(&[&["a\"b\\c$d"]])),
        (
            "'C:\\Tools\\Zsh.exe' -c x",
            plain(&[&["C:\\Tools\\Zsh.exe", "-c", "x"]]),
        ),
        ("ls {} -g'*.py'", plain(&[&["ls", "{}", "-g*.py"]])),
        ("git a=b \\=c d+e", plain(&[&["git", "a=b", "=c", "d+e"]])),
        (
            "test -f x && ls a\\[b]",
            plain(&[&["test", "-f", "x"], &["ls", "a[b]"]]),
        ),
    ] {
        assert_eq!(read_script_commands(script), expected, "{script}");
    }
}

#[test]
fn words_bash_builds_at_run_time_are_unknown() {
    for (script, expected) in [
        ("$cmd push", partial(&[&["?", "push"]])),
        ("\"$cmd\" push", partial(&[&["?", "push"]])),
        (
            "git \"$(echo push)\"",
            partial(&[&["git", "?"], &["echo", "push"]]),
        ),
        ("/usr/bin/gi? push", partial(&[&["?", "push"]])),
        ("/usr/bin/g*t push", partial(&[&["?", "push"]])),
        ("/usr/bin/gi[t] push", partial(&[&["?", "push"]])),
        ("git {pu,}sh", partial(&[&["git", "?"]])),
        ("~/bin/git push", partial(&[&["?", "push"]])),
        ("=git push", partial(&[&["?", "push"]])),
        ("[ -f x ] && 'git' push", partial(&[&["git", "push"]])),
        ("$'git' push", partial(&[&["git", "push"]])),
        ("git push > /dev/null", partial(&[&["git", "push"]])),
        ("A=1 'git' push 2>&1", partial(&[&["git", "push"]])),
        (
            "for f in *; do 'git' push; done",
            partial(&[&["git", "push"]]),
        ),
        (
            "x=$('git' push); f() { \"rm\" -rf x; }",
            partial(&[&["git", "push"], &["rm", "-rf", "x"]]),
        ),
        (
            "cat <(git push)",
            partial(&[&["cat", "?"], &["git", "push"]]),
        ),
    ] {
        assert_eq!(read_script_commands(script), expected, "{script}");
    }
}

#[test]
fn unparsable_scripts_are_unreadable() {
    // The grammar does not read a brace expansion as a command name.
    for script in ["git push (", "'git push", "if then fi", "{git,} push"] {
        assert_eq!(
            read_script_commands(script),
            ScriptCommands::Unreadable,
            "{script}"
        );
    }
}

#[test]
fn only_shell_scripts_are_read() {
    let words = |line: &[&str]| line.iter().map(ToString::to_string).collect::<Vec<_>>();
    assert_eq!(
        shell_lc_script_commands(&words(&["bash", "-lc", "'git' push"])),
        Some(plain(&[&["git", "push"]]))
    );
    assert_eq!(
        shell_lc_script_commands(&words(&["sh", "-c", "\"git\" push"])),
        Some(plain(&[&["git", "push"]]))
    );
    assert_eq!(shell_lc_script_commands(&words(&["git", "push"])), None);
}
