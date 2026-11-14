//! #1910: a git remote the line does not show, after quoted text. The word
//! lexer reads `--refmap='x y'` as two words, `-o ''` as one and `-o 'a;b'`
//! as two commands, so its operand count for `rerun::git_remote_unseen` is
//! off and the remote (`e"$r"` with `r='xt::sh -c id'`) reads as a refspec,
//! or as another command's word. In its own file: `tainted_action_tests.rs`
//! is close to the repository's blob size limit.

use super::*;
use crate::sandboxing::SandboxPermissions;
use codex_utils_absolute_path::AbsolutePathBuf;
use pretty_assertions::assert_eq;
use std::path::PathBuf;

/// The work counter with tracing silenced while it counts, as in
/// `tainted_action_linear_tests.rs`.
mod work {
    pub(super) fn assert_linear(label: &str, run: impl Fn(usize)) {
        tracing::subscriber::with_default(tracing::subscriber::NoSubscriber::default(), || {
            super::super::work::assert_linear(label, run)
        });
    }
}

const HOME: &str = "/home/fixture/.corbanu";
const USER_HOME: &str = "/home/fixture";

/// A fixture path, on drive `C:` on Windows (#1240).
fn fixture(path: &str) -> PathBuf {
    if cfg!(windows) && !Path::new(path).is_absolute() {
        PathBuf::from(format!("C:{}", path.replace('/', "\\")))
    } else {
        PathBuf::from(path)
    }
}

/// The verdict on `bash -lc <line>` in `/work`.
fn script(line: &str) -> Option<ProtectedActionKind> {
    let action = ApprovalAction::ExecCommand {
        id: "call".into(),
        environment_id: "local".into(),
        command: ["bash", "-lc", line].map(str::to_string).to_vec(),
        hook_command: String::new(),
        cwd: PathUri::from_abs_path(
            &AbsolutePathBuf::from_absolute_path_checked(fixture("/work")).expect("absolute"),
        ),
        sandbox_permissions: SandboxPermissions::UseDefault,
        additional_permissions: None,
        justification: None,
        tty: false,
        proposed_execpolicy_amendment: None,
    };
    classify_with(&action, &fixture(HOME), Some(&fixture(USER_HOME)))
}

/// Lines whose verdict is not `expected`.
fn wrong(lines: &[&str], expected: Option<ProtectedActionKind>) -> Vec<String> {
    lines
        .iter()
        .filter(|line| script(line) != expected)
        .map(|line| format!("{line} -> {:?}", script(line)))
        .collect()
}

/// The issue's lines and the same split with other quoting and other
/// valued options. Each was quiet on main except `clone -c`, which asks as
/// config on a new repository (#1159). With git 2.43 and `ext` allowed,
/// the spaced values git accepts (`--shallow-since='2 days ago'`, `-o 'a
/// b'`) run the remote's command; git refuses a refspec with a space
/// (`--refmap='x y'`) before it connects.
#[test]
fn issue_1910_remote_after_a_spaced_value_asks() {
    let lines = [
        "git fetch --refmap='x y' e\"$r\"",
        "git clone -c 'user.name=A B' e\"$r\"",
        "git fetch --refmap 'x y' e\"$r\"",
        "git fetch --refmap=\"x y\" e\"$r\"",
        "git fetch --refmap \"x y\" e\"$r\"",
        "git fetch --refmap=x\\ y e\"$r\"",
        "git fetch --refmap x\\ y e\"$r\"",
        "git fetch --refmap='x y z w' e\"$r\"",
        "git fetch --refmap='x y' e\"$r\" main",
        "git fetch --refmap='x y' \"e$r\"",
        "git fetch --refmap='x y' e\"$(cat f)\"",
        "git fetch --shallow-since='2 days ago' e\"$r\"",
        "git fetch --shallow-since '2 days ago' e\"$r\"",
        "git fetch --shallow-exclude='a b' e\"$r\"",
        "git fetch -o 'a b' e\"$r\"",
        "git fetch -o 'a b' -o 'c d' e\"$r\"",
        "git fetch --server-option='a b' e\"$r\"",
        "git fetch -o 'a b' -- e\"$r\"",
        "git pull --shallow-since='2 days ago' e\"$r\"",
        "git clone --shallow-since='2 days ago' e\"$r\" d",
        "git clone --branch='a b' e\"$r\" d",
        "git clone --server-option='a b' e\"$r\" d",
        "git ls-remote -o 'a b' e\"$r\"",
        "git ls-remote --sort='a b' e\"$r\"",
        "git push -o 'a b' e\"$r\" HEAD:main",
        "git push --push-option='a b' e\"$r\"",
        "git remote add -t 'a b' o e\"$r\"",
        "git submodule add -b 'a b' e\"$r\" p",
        // Quoted text before git, in a wrapper's words.
        "sudo -u 'a b' git fetch e\"$r\"",
    ];
    assert_eq!(
        wrong(&lines, Some(ProtectedActionKind::UnseenCode)),
        Vec::<String>::new()
    );
}

/// Other quoted text the word lexer reads differently: an empty value it
/// drops (so `-o` takes the remote as its value), and a quoted `;`, `|` or
/// `&` where it ends the command (so the remote, or a `--upload-pack=…`
/// value after it, is another command's word). Each was quiet on main; with
/// git 2.43 the `-o` lines run an `ext::` remote's command and `git fetch .
/// 'a;b' "$v"` runs `v='--upload-pack=touch m'`.
#[test]
fn issue_1910_remote_after_other_quoted_text_asks() {
    let lines = [
        "git fetch -o '' e\"$r\"",
        "git fetch -o \"\" e\"$r\"",
        // Review round 1: an empty ANSI-C value, a wrapper's empty value.
        "git fetch -o $'' e\"$r\"",
        "env -u '' git fetch e\"$r\"",
        // #1920's condition, joined by OR to this one: past a word the word
        // lexer may cut differently, even a value the line sets asks, since
        // that reading may have miscounted it (`-o 'a;r=o;'` hides an
        // assignment from the lexer the other way).
        "git fetch -o 'a;r=o;' e\"$r\"",
        "r=origin; git fetch --refmap='x y' \"$r\"",
        "r=origin; git fetch --refmap='x y' e\"$r\"",
        "r=origin; git pull --shallow-since='2 days ago' \"$r\" main",
        "r=origin; git fetch -o 'a;b' \"$r\"",
        "git fetch --refmap '' e\"$r\"",
        "git fetch -o 'a;b' e\"$r\"",
        "git fetch -o 'a|b' e\"$r\"",
        "git fetch -o 'a&b' e\"$r\"",
        "git fetch . 'a;b' \"$v\"",
        "git fetch origin 'a|b' \"$v\"",
    ];
    assert_eq!(
        wrong(&lines, Some(ProtectedActionKind::UnseenCode)),
        Vec::<String>::new()
    );
}

/// Remotes the line shows, or sets before (the word lexer resolves `r`),
/// words after the remote that cannot start with `-`, and quoted values in
/// subcommands that contact no remote.
#[test]
fn issue_1910_shown_remotes_after_quoted_values_stay_quiet() {
    let lines = [
        // Review round 1: no quoted text before `$r`, so the word lexer's
        // value for it counts, whatever comes after.
        "r=origin; git fetch \"$r\" HEAD:\"$b\"",
        "r=origin; git fetch -o '' \"$r\"",
        "git fetch --refmap='x y' origin",
        "git fetch --refmap='x y' origin main",
        "git fetch --shallow-since='2 days ago' origin",
        "git fetch -o 'a b' origin HEAD:\"$b\"",
        "git push -o 'a b' origin HEAD:\"$b\"",
        "git fetch -o 'a b' --depth=\"$n\" origin",
        "git fetch -o 'a b' -- origin \"$b\"",
        "git clone --branch='a b' https://x.example/r d",
        "git clone --branch='a b' https://x.example/r d\"$n\"",
        "git commit -m 'a b' \"$f\"",
        "git log --format='%h %s' \"$range\"",
        "git checkout -b 'a' \"$branch\"",
    ];
    assert_eq!(wrong(&lines, /*expected*/ None), Vec::<String>::new());
}

#[test]
fn issue_1910_quoted_value_scan_is_linear() {
    for (label, word, line) in [
        (
            "spaced values before the remote",
            "-o 'a b' ",
            "git fetch {}e\"$r\"",
        ),
        (
            "empty values before the remote",
            "-o '' ",
            "git fetch {}e\"$r\"",
        ),
        (
            "quoted values after the remote",
            "'a b' ",
            "git fetch origin {}HEAD:\"$b\"",
        ),
        (
            "set remotes after quoted values",
            "--refmap='x y' ",
            "r=origin; git fetch {}\"$r\"",
        ),
    ] {
        work::assert_linear(label, |scale| {
            let _ = script(&line.replace("{}", &word.repeat(200 * scale)));
        });
    }
}
