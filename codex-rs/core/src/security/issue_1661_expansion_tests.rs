//! Add-prompt regressions for the remaining #1661 shell expansions.
use super::*;
use pretty_assertions::assert_eq;
use test_case::test_case;

#[test_case("find . -exec cp {} =mytool \\;", "bash"; "find_cp_bash")]
#[test_case("find . -exec cp {} =mytool \\;", "zsh"; "find_cp_zsh")]
#[test_case("sort -o =mytool input", "bash"; "sort_bash")]
#[test_case("sort -o =mytool input", "zsh"; "sort_zsh")]
#[test_case("cc -o =mytool input.c", "bash"; "cc_bash")]
#[test_case("cc -o =mytool input.c", "zsh"; "cc_zsh")]
#[test_case("strip =mytool", "bash"; "strip_bash")]
#[test_case("strip =mytool", "zsh"; "strip_zsh")]
#[test_case("uniq input =mytool", "bash"; "uniq_bash")]
#[test_case("uniq input =mytool", "zsh"; "uniq_zsh")]
#[test_case("print x >> {,}=ls", "bash"; "brace_redirect_bash")]
#[test_case("print x >> {,}=ls", "zsh"; "brace_redirect_zsh")]
#[test_case("x=; print x >> $x=ls", "bash"; "empty_redirect_bash")]
#[test_case("x=; print x >> $x=ls", "zsh"; "empty_redirect_zsh")]
#[test_case("tee -a $nope=mytool", "bash"; "empty_tee_bash")]
#[test_case("tee -a $nope=mytool", "zsh"; "empty_tee_zsh")]
#[test_case("find . -exec sort -o =mytool {} +", "bash"; "find_sort_bash")]
#[test_case("find . -exec sort -o =mytool {} +", "zsh"; "find_sort_zsh")]
#[test_case("sort -o {,}=mytool input", "bash"; "brace_sort_bash")]
#[test_case("sort -o {,}=mytool input", "zsh"; "brace_sort_zsh")]
#[test_case("cc -o ${nope}=mytool input.c", "bash"; "empty_cc_bash")]
#[test_case("cc -o ${nope}=mytool input.c", "zsh"; "empty_cc_zsh")]
#[test_case("eval 'sort -o =mytool input'", "bash"; "eval_sort_bash")]
#[test_case("eval 'sort -o =mytool input'", "zsh"; "eval_sort_zsh")]
#[test_case("trap 'print x >> {,}=ls' EXIT", "bash"; "trap_redirect_bash")]
#[test_case("trap 'print x >> {,}=ls' EXIT", "zsh"; "trap_redirect_zsh")]
#[test_case("alias save='uniq input =mytool'", "bash"; "alias_uniq_bash")]
#[test_case("alias save='uniq input =mytool'", "zsh"; "alias_uniq_zsh")]
fn issue1661_generated_writer_targets_ask(line: &str, shell: &str) {
    // As for #1602/#1651 and D327, bash retains the stricter zsh judgment.
    assert!(kind(&[shell, "-lc", line]).is_some(), "{shell}: {line}");
}

#[test_case("git clone -b {,--upload-pack=./x} r d", "bash"; "empty_brace_bash")]
#[test_case("git clone -b {,--upload-pack=./x} r d", "zsh"; "empty_brace_zsh")]
#[test_case("git clone [-]-upload-pack=./x r d", "bash"; "glob_bash")]
#[test_case("git clone [-]-upload-pack=./x r d", "zsh"; "glob_zsh")]
#[test_case("N=; git clone ${N/#/--upload-pack=./x} r d", "bash"; "operator_bash")]
#[test_case("N=; git clone ${N/#/--upload-pack=./x} r d", "zsh"; "operator_zsh")]
#[test_case("git clone {-b,,--upload-pack=./x} ./r d", "bash"; "brace_value_bash")]
#[test_case("git clone {-b,,--upload-pack=./x} ./r d", "zsh"; "brace_value_zsh")]
#[test_case(r#"N=; git clone "${N/#/--upload-pack=./x}" r d"#, "bash"; "quoted_operator_bash")]
#[test_case(r#"N=; git clone "${N/#/--upload-pack=./x}" r d"#, "zsh"; "quoted_operator_zsh")]
#[test_case("git {--namespace,,-c,core.pager=./x,log}", "bash"; "empty_brace_config_bash")]
#[test_case("git {--namespace,,-c,core.pager=./x,log}", "zsh"; "empty_brace_config_zsh")]
fn issue1661_generated_git_options_ask(line: &str, shell: &str) {
    assert!(kind(&[shell, "-lc", line]).is_some(), "{shell}: {line}");
}

#[test]
fn issue1661_expansion_checks_are_linear() {
    work::assert_linear("long generated target", |scale| {
        let line = format!("sort -o {}=mytool input", "{,}".repeat(100 * scale));
        assert!(script(&line).is_some());
    });
    for (prefix, unit) in [
        ("find . -exec cp ", "${nope}=mytool "),
        ("sort -o output ", "{a,b}=mytool "),
        ("git clone ", "[-]-quiet "),
        ("git clone ", "{,--quiet} "),
        ("git log ", "${N/#/--oneline} "),
    ] {
        work::assert_linear(prefix, |scale| {
            let line = format!("{prefix}{}", unit.repeat(100 * scale));
            let _ = script(&line);
        });
    }
}

#[test]
fn issue1661_ordinary_expansions_stay_quiet() {
    for shell in ["bash", "zsh"] {
        for line in [
            "echo =mytool",
            "sort =mytool",
            "cc -c =mytool",
            "uniq =mytool",
            "sort -o output input",
            "strip ./program",
            "print x >> a=ls",
            "print x >> {a,b}=ls",
            "git add src/[a].rs",
            "git clone ./r d",
            "N=; git log ${N/#/--oneline}",
            "curl --proto '=https' https://example.com",
        ] {
            assert_eq!(kind(&[shell, "-lc", line]), None, "{shell}: {line}");
        }
    }
}

/// Observe actual shell words without running git or writing installed tools.
#[cfg(unix)]
#[test]
fn issue1661_bash_and_zsh_expansion_semantics() {
    use std::os::unix::fs::PermissionsExt;
    use std::process::Command;

    let temp = tempfile::tempdir().unwrap();
    let bin = temp.path().join("bin");
    std::fs::create_dir(&bin).unwrap();
    let tool = bin.join("mytool");
    std::fs::write(&tool, "#!/bin/sh\nexit 0\n").unwrap();
    std::fs::set_permissions(&tool, std::fs::Permissions::from_mode(0o700)).unwrap();
    std::fs::create_dir(temp.path().join("--upload-pack=.")).unwrap();
    std::fs::write(temp.path().join("--upload-pack=./x"), "").unwrap();
    for (shell, flags, zsh) in [
        ("/bin/bash", vec!["--norc", "--noprofile"], false),
        ("/bin/zsh", vec!["-f"], true),
    ] {
        if !std::path::Path::new(shell).exists() {
            continue;
        }
        let expanded = format!("<{}>\n", tool.display());
        for (line, expected) in [
            (
                r#"printf '<%s>\n' {,--upload-pack=./x}"#,
                if zsh {
                    "<>\n<--upload-pack=./x>\n"
                } else {
                    "<--upload-pack=./x>\n"
                }
                .to_string(),
            ),
            (
                r#"printf '<%s>\n' [-]-upload-pack=./x"#,
                "<--upload-pack=./x>\n".to_string(),
            ),
            (
                r#"N=; printf '<%s>\n' ${N/#/--upload-pack=./x}"#,
                "<--upload-pack=./x>\n".to_string(),
            ),
            (
                r#"N=; printf '<%s>\n' "${N/#/--upload-pack=./x}""#,
                "<--upload-pack=./x>\n".to_string(),
            ),
            (
                r#"N=; printf '<%s>\n' ${N:h}"#,
                if zsh { "<.>\n" } else { "<>\n" }.to_string(),
            ),
            (
                r#"x=; printf '<%s>\n' $x=mytool"#,
                if zsh {
                    expanded.clone()
                } else {
                    "<=mytool>\n".to_string()
                },
            ),
            (
                r#"printf '<%s>\n' {,}=mytool"#,
                if zsh {
                    expanded.repeat(2)
                } else {
                    "<=mytool>\n".repeat(2)
                },
            ),
        ] {
            let output = Command::new(shell)
                .args(&flags)
                .args(["-c", line])
                .env_clear()
                .env("HOME", temp.path())
                .env("PATH", format!("{}:/usr/bin:/bin", bin.display()))
                .env("LC_ALL", "C")
                .env("CORBANU_TEST_NO_NATIVE_KEYRING", "1")
                .current_dir(temp.path())
                .output()
                .unwrap();
            assert!(
                output.status.success(),
                "{shell}: {line}: {:?}",
                output.stderr
            );
            assert_eq!(
                String::from_utf8(output.stdout).unwrap(),
                expected,
                "{shell}: {line}"
            );
        }
    }
}

#[test]
fn issue1661_existing_noise_still_asks() {
    for line in [
        r#"eval "awk 'NR>=10' f""#,
        "((a>=1))",
        "print x >> '=mytool'",
    ] {
        assert!(script(line).is_some(), "{line}");
    }
}
