//! #1808: chroot after untrusted content: its command is judged as a
//! command line of its own, besides the line as read before, and a chroot
//! whose command cannot be judged asks.
use super::*;
use pretty_assertions::assert_eq;

/// #1808: after untrusted content, a chroot's command is judged as a
/// command line of its own (`chroot / ./x` asks as `./x` does), besides the
/// line as read before, and it asks when the command cannot be judged (a
/// root other than `/`, an unseen root, or no command). In bash and zsh.
/// Only adds prompts; `still_ask` holds lines that ask on main.
#[test]
fn issue1808_chroot_judges_its_command() {
    use ProtectedActionKind::UnseenCode;
    let asks = [
        // The reported case and its option spellings.
        "chroot / ./x",
        "chroot --userspec=a:b / ./x",
        "chroot --userspec a:b / ./x",
        "chroot --groups g / ./x",
        "chroot --skip-chdir / ./x",
        "chroot -- / ./x",
        "sudo chroot / ./x",
        "env chroot / ./x",
        // Review round 2: a system chroot named by its path is the wrapper.
        "/usr/sbin/chroot / ./x",
        "sudo /usr/sbin/chroot / ./x",
        "env /usr/sbin/chroot / ./x",
        "true && chroot / ./x",
        "eval chroot / ./x",
        // Unjudgeable wrapped code.
        "chroot / sh -c \"$(cat f)\"",
        "chroot /tmp/r ls",
        "chroot ./root true",
        "chroot \"$(cat f)\" true",
        "chroot /",
        "chroot /tmp/r",
    ];
    // Unchanged must-ask lines (controls) and harmless lines that stay quiet.
    let unchanged = [
        "./x",
        "sh -c \"$(cat f)\"",
        "chroot / git -c core.pager=sh log",
    ];
    let quiet = [
        "chroot / true",
        "chroot / ls -la",
        // Colon-free: on Windows `a:b` reads as a drive path (stricter there).
        "chroot --userspec=user / true",
        "chroot --skip-chdir / echo hi",
        "echo chroot / ./x",
        "grep -r chroot .",
        "env true",
    ];
    // Review round 1: lines that ask on main must still ask.
    let still_ask = [
        "find . -exec chroot A=B {} \\;",
        "find . -exec sudo chroot /srv/jail=1 {} \\;",
        "fd . -x chroot A=B {}",
        "printf x | strace xargs -I% chroot A=B %",
        "xargs -I% chroot A=B %",
        "chroot / =git log",
        "sudo chroot --skip-chdir / =git log",
        "env chroot -- / =git log",
        "xargs chroot / =git log < f",
        "./chroot / true",
        // Review round 2: dash words after the root under a launcher.
        "xargs -I% chroot / -x % < f",
        "printf x | xargs -I% chroot / -x %",
        "fd . -x chroot / --userspec u {}",
        "parallel chroot / -x {} ::: a",
        "find . -exec chroot A=B -x {} \\;",
        "xargs -I% chroot / -- % < f",
        // Review round 3: an assignment word after the root under a
        // launcher, and git config writes behind `command -v`/`env --help`.
        "xargs -I% chroot / A=% x < f",
        "printf x | xargs -I% chroot / A=% true",
        "fd . -x chroot / A={} x",
        "parallel chroot / A={} x ::: a",
        "chroot / command -v git config pager.log marker",
        "chroot / env --help git config pager.log marker",
        "sudo chroot / command -V git config pager.log marker",
    ];
    // Review round 4: quiet on main, ask now. A redirection after the root
    // leaves chroot no command (it runs the root's shell, which may read the
    // redirected file); its command runs in `/` unless `--skip-chdir`; an
    // option this reading does not know.
    let round4 = [
        "chroot / <x",
        "sudo chroot / <x 2>/dev/null",
        "chroot / >out",
        "chroot / 2>&1",
        "chroot / cp x home/fixture/.zshrc",
        "chroot --group / ls",
        "chroot -u / ls",
    ];
    let mut wrong = Vec::new();
    for shell in ["/bin/bash", "/bin/zsh"] {
        for command in round4 {
            if kind(&[shell, "-lc", command]).is_none() {
                wrong.push(format!("round 4 asks {shell}: {command}"));
            }
        }
        // Controls: the same write without chroot asks; with `--skip-chdir`
        // the relative path stays under the current folder.
        if kind(&[shell, "-lc", "cp x /home/fixture/.zshrc"]).is_none() {
            wrong.push(format!("control asks {shell}: cp x /home/fixture/.zshrc"));
        }
        if kind(&[
            shell,
            "-lc",
            "chroot --skip-chdir / cp x home/fixture/.zshrc",
        ])
        .is_some()
        {
            wrong.push(format!(
                "control quiet {shell}: chroot --skip-chdir / cp x home/fixture/.zshrc"
            ));
        }
        for command in still_ask {
            if kind(&[shell, "-lc", command]).is_none() {
                wrong.push(format!("still asks {shell}: {command}"));
            }
        }
        for command in asks.iter().chain(&unchanged) {
            if kind(&[shell, "-lc", command]) != Some(UnseenCode) {
                wrong.push(format!("asks {shell}: {command}"));
            }
        }
        for command in quiet {
            if kind(&[shell, "-lc", command]).is_some() {
                wrong.push(format!("quiet {shell}: {command}"));
            }
        }
    }
    assert_eq!(wrong, Vec::<String>::new());
}

/// A chroot child inherits the enclosing command's script input, even when
/// redirection operands precede the wrapper or are not child arguments.
#[test]
fn issue1808_chroot_inherited_script_input_asks() {
    let asks = [
        "cat payload | chroot / sh",
        "chroot / sh -s <payload",
        "chroot / sh <payload",
        "<payload chroot / sh",
        "cat payload | chroot / python3",
        "cat payload | chroot / chroot / sh",
        "chroot / bash <<< 'echo hello'",
        "cat payload > >(chroot / sh)",
    ];
    let quiet = [
        "cat payload | chroot / cat",
        "chroot / cat <payload",
        "chroot / sh -c 'echo ok' <payload",
        "echo text | chroot / true",
    ];
    let mut wrong = Vec::new();
    for shell in ["/bin/bash", "/bin/zsh"] {
        for command in asks {
            if kind(&[shell, "-lc", command]).is_none() {
                wrong.push(format!("inherited input must ask {shell}: {command}"));
            }
        }
        for command in quiet {
            if kind(&[shell, "-lc", command]).is_some() {
                wrong.push(format!("data-only input stays quiet {shell}: {command}"));
            }
        }
    }
    assert_eq!(wrong, Vec::<String>::new());
}

/// #1808: chroot reading stays linear, with chained chroots and long
/// option runs.
#[test]
fn issue1808_chroot_reading_is_linear() {
    // Review round 4: these nest (`chroot / chroot / … true`, and through
    // `eval`), and option runs.
    for (label, unit, tail) in [
        ("nested chroots", "chroot / ", "true"),
        ("nested through eval", "eval chroot / ", "true"),
        (
            "option runs",
            "--userspec a --groups g --skip-chdir ",
            "/ true",
        ),
    ] {
        work::assert_linear(label, |scale| {
            let line = if label == "option runs" {
                format!("chroot {}{tail}", unit.repeat(100 * scale))
            } else {
                format!("{}{tail}", unit.repeat(100 * scale))
            };
            let _ = script(&line);
        });
    }
}

/// #1808 review: argv forms (no shell) ask: `=git` after chroot's root and a
/// chroot named by a workspace path (as on main), and a system chroot named
/// by its path, whose command is judged.
#[test]
fn issue1808_chroot_argv_forms_keep_main_verdicts() {
    for command in [
        &["chroot", "/", "=git", "log"][..],
        &["./chroot", "/", "true"][..],
        &["/usr/sbin/chroot", "/", "./x"][..],
    ] {
        assert!(kind(command).is_some(), "{command:?}");
    }
}
