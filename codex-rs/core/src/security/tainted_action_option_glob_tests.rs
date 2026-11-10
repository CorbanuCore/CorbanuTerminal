//! #1912: a glob may expand to a file named as an option of a command that
//! runs a program it names (`tar cf a.tar *` with `--checkpoint-action=exec=sh
//! x`, `rg data *` with `--pre=sh`, `sed -i s/a/b/ *` with `--expression=1e
//! sh x`). #1910: a quoted value with spaces before a git transport's remote
//! no longer hides it. #1897 follow-up: the prompt hint for `git push origin
//! "$b"`. In its own file: `tainted_action_tests.rs` is close to the
//! repository's blob size limit.

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

/// `<shell> -lc <line>` in `/work`.
fn action(shell: &str, line: &str) -> ApprovalAction {
    ApprovalAction::Shell {
        id: "call".into(),
        environment_id: "local".into(),
        command: [shell, "-lc", line].map(str::to_string).to_vec(),
        cwd: PathUri::from_abs_path(
            &AbsolutePathBuf::from_absolute_path_checked(fixture("/work")).expect("absolute"),
        ),
        sandbox_permissions: SandboxPermissions::UseDefault,
        additional_permissions: None,
        justification: None,
    }
}

/// The verdicts on a line under bash and under zsh.
fn verdicts(line: &str) -> [Option<ProtectedActionKind>; 2] {
    ["bash", "zsh"].map(|shell| {
        classify_with(
            &action(shell, line),
            &fixture(HOME),
            Some(&fixture(USER_HOME)),
        )
    })
}

/// Lines whose verdict under either shell is not `expected`.
fn wrong(lines: &[&str], expected: Option<ProtectedActionKind>) -> Vec<String> {
    lines
        .iter()
        .filter(|line| verdicts(line) != [expected; 2])
        .map(|line| format!("{line} -> {:?}", verdicts(line)))
        .collect()
}

#[test]
fn issue_1912_globs_that_may_name_an_option_ask() {
    let lines = [
        "tar cf a.tar *",
        "tar czf a.tgz *",
        "tar -cf a.tar *",
        "tar -czvf a.tgz *",
        "tar cf a.tar *.txt",
        "tar --create --file a.tar *",
        "tar xf a.tar *",
        "tar -C d -cf a.tar *",
        "tar -cf -- *",
        "tar cf -- *",
        "gtar cf a.tar *",
        "bsdtar cf a.tar *",
        "/usr/bin/tar cf a.tar *",
        "sudo tar cf a.tar *",
        "env tar cf a.tar *",
        "cd sub && tar cf ../a.tar *",
        "tar cf - * | gzip > a.tgz",
        "bash -c 'tar cf a.tar *'",
        "tar cf a.tar -*",
        "tar cf a.tar --check*",
        "tar cf a.tar ?*",
        "tar cf a.tar [-]*",
        "tar cf a.tar \"-\"*",
        "tar cf a.tar {a,*}",
        "tar cf a.tar $d*",
        "rg data *",
        "rg -n data -*",
        "rg data *.r?",
        "sed -i s/a/b/ *",
        "sed -n p *",
        "gsed -i s/a/b/ *.md",
        "sort *",
        "gsort -u x *",
        "rsync -a * /tmp/dst/",
        "TAR cf a.tar *",
        "timeout 9 tar cf a.tar *",
        "nice -n 5 tar cf a.tar *",
        "sed -ne=* a.txt",
    ];
    assert_eq!(
        wrong(&lines, Some(ProtectedActionKind::UnseenCode)),
        Vec::<String>::new()
    );
}

#[test]
fn issue_1912_zsh_globs_ask() {
    for line in [
        "tar cf a.tar **/*",
        "tar cf a.tar *(.)",
        "rg data ^a.txt",
        "rg data *~a.rs",
        "tar cf a.tar x#-*",
    ] {
        assert_eq!(
            classify_with(
                &action("zsh", line),
                &fixture(HOME),
                Some(&fixture(USER_HOME))
            ),
            Some(ProtectedActionKind::UnseenCode),
            "{line}"
        );
    }
}

#[test]
fn issue_1912_globs_that_cannot_name_an_option_stay_quiet() {
    let lines = [
        "tar cf a.tar -- *",
        "tar cf a.tar ./*",
        "tar cf a.tar src/*",
        "tar cf a.tar a.txt",
        "tar cf a.tar '*'",
        "tar cf a.tar \"*\"",
        "tar --exclude=*.log -cf a.tar dir",
        "tar --exclude='*.log' -cf a.tar dir",
        "tar tf a.tar",
        "rg data -- *",
        "rg data ./*",
        "rg -n data *.rs",
        "rg -n data src/*.rs *.md",
        "sort -u *.txt",
        "rg -g '*.rs' data",
        "rg --glob=*.rs data",
        "sed -i s/a/b/ -- *",
        "sed -- s/a/b/ *",
        "sed -i s/a/b/ src/*.md",
        "sed -i 's/*/x/' a.txt",
        "sort -- *",
        "rsync -a ./* /tmp/dst/",
        "ls *",
        "cat *.txt",
        "cp * /tmp/d/",
        "wc -l *",
        "echo tar cf a.tar *",
    ];
    assert_eq!(wrong(&lines, /*expected*/ None), Vec::<String>::new());
    // Under Windows path rules these may ask for credential access (a
    // path the Windows view reads); only this rule's verdict is pinned.
    for line in ["tar cf a.tar \"$PWD\"/*", "tar cf a.tar \\*"] {
        assert!(
            !verdicts(line).contains(&Some(ProtectedActionKind::UnseenCode)),
            "{line}"
        );
    }
}

#[test]
fn issue_1910_quoted_value_with_spaces_does_not_hide_the_remote() {
    let lines = [
        "git fetch --refmap='x y' e\"$r\"",
        "git fetch --refmap 'x y' e\"$r\"",
        "git clone -c 'user.name=A B' e\"$r\"",
        "git fetch --refmap=\"x y\" e\"$r\"",
        "git push --push-option='a b' e\"$r\"",
        "git fetch --refmap='x,y' e\"$r\"",
        "git fetch --refmap=x,y e\"$r\"",
        "git fetch --refmap='x(y' e\"$r\"",
        "git fetch --refmap '}' --depth 1 e\"$r\"",
        "git fetch origin \\} main \"$v\"",
        "git fetch --refmap '{' e\"$r\"",
        "git fetch --refmap='x$' e\"$r\"",
        "git fetch --refmap=\"x$'\" e\"$r\"",
        "git fetch --refmap a\u{a0}b e\"$r\"",
        "git fetch --refmap='x`y' e\"$r\"",
        "git fetch --refmap='x$(y' e\"$r\"",
        "git fetch --refmap='x)y' e\"$r\"",
        "git fetch --refmap='x;y' e\"$r\"",
        "git fetch --refmap='x|y' e\"$r\"",
        "git fetch --refmap='x&y' e\"$r\"",
        // Asks on main too (`git_runs_code_after_git`); kept asking.
        "git -C 'a b' \"$s\" *",
    ];
    assert_eq!(
        wrong(&lines, Some(ProtectedActionKind::UnseenCode)),
        Vec::<String>::new()
    );
    let quiet = [
        "git fetch --refmap='x y' origin",
        "git fetch --refmap='x y' origin main",
        "git push origin HEAD:\"$b\"",
        "git push origin -- \"$b\"",
        "git push origin refs/heads/\"$b\"",
        "git commit -m 'a b' && git push origin HEAD:\"$b\"",
    ];
    assert_eq!(wrong(&quiet, /*expected*/ None), Vec::<String>::new());
}

#[test]
fn issue_1897_push_hint_names_the_quiet_forms() {
    let hint = |line: &str| approval_hint(&action("bash", line), ProtectedActionKind::UnseenCode);
    for line in [
        "git push origin \"$branch\"",
        "git push -u origin $b",
        "cd r && git push origin \"${b}\"",
    ] {
        assert_eq!(hint(line), Some(PUSH_BRANCH_HINT), "{line}");
        // The line does ask, so the hint is shown with a prompt.
        assert_eq!(
            verdicts(line),
            [Some(ProtectedActionKind::UnseenCode); 2],
            "{line}"
        );
    }
    for line in [
        "git push origin main",
        "git fetch origin \"$b\"",
        "git push origin main; echo \"$b\"",
        "echo push \"$b\"",
        "git push \"$r\" main",
        "git push origin main\necho \"$b\"",
        "git push origin main;echo \"$b\"",
        "git log push origin \"$b\"",
    ] {
        assert_eq!(hint(line), None, "{line}");
    }
    assert_eq!(
        approval_hint(
            &action("bash", "git push origin \"$b\""),
            ProtectedActionKind::Credentials,
        ),
        None
    );
}

#[test]
fn issue_1912_and_1910_scans_are_linear() {
    for (label, line, word) in [
        ("tar-globs", "tar cf a.tar {} *", "*.txt "),
        ("tar-plain", "tar cf a.tar {} *", "a.txt "),
        ("sed-dashes", "sed -i s/a/b/ {} -- *", "-- "),
        ("git-refmap", "git fetch {} e\"$r\"", "--refmap='x y' "),
    ] {
        work::assert_linear(label, |scale| {
            let _ = verdicts(&line.replace("{}", &word.repeat(200 * scale)));
        });
    }
}
