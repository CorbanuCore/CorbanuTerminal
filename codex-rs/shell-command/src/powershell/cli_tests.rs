//! #2568: PowerShell command lines read as PowerShell reads them.

use super::*;
use pretty_assertions::assert_eq;

fn words(line: &[&str]) -> Vec<String> {
    line.iter().map(ToString::to_string).collect()
}

fn run_of(line: &[&str]) -> String {
    let command = words(line);
    let kind = PowershellKind::of(&command[0]);
    format!("{:?}", read(&command, kind).run)
}

fn script_of(line: &[&str]) -> Option<String> {
    let command = words(line);
    let kind = PowershellKind::of(&command[0]);
    match read(&command, kind).run {
        Run::Command { script, .. } => Some(script.into_owned()),
        Run::Encoded(script) => Some(script),
        Run::File(_) | Run::Unreadable | Run::Interactive => None,
    }
}

fn is_unreadable(line: &[&str]) -> bool {
    let command = words(line);
    let kind = PowershellKind::of(&command[0]);
    read(&command, kind).run == Run::Unreadable
}

/// Base64 of UTF-16LE, as `-EncodedCommand` takes it.
fn encode(script: &str) -> String {
    let bytes: Vec<u8> = script.encode_utf16().flat_map(u16::to_le_bytes).collect();
    base64::engine::general_purpose::STANDARD.encode(bytes)
}

#[test]
fn command_joins_every_remaining_argument() {
    for shell in ["pwsh", "powershell.exe"] {
        let command = words(&[shell, "-Command", "git status", "; git push"]);
        let kind = PowershellKind::of(shell);
        assert_eq!(
            read(&command, kind),
            Invocation {
                run: Run::Command {
                    start: 2,
                    script: Cow::Owned("git status ; git push".to_string()),
                },
                plain: true,
            },
            "{shell}"
        );
        // Joined with single spaces, nothing trimmed or quoted.
        assert_eq!(
            script_of(&[shell, "-c", "Write-Output", "'a b'", "", " x "]).as_deref(),
            Some("Write-Output 'a b'   x "),
            "{shell}"
        );
        assert_eq!(
            script_of(&[shell, "-c", "git", "push"]).as_deref(),
            Some("git push")
        );
        // Words after the script that look like parameters are script.
        assert_eq!(
            script_of(&[shell, "-c", "git status", "-NoExit", "-File", "x"]).as_deref(),
            Some("git status -NoExit -File x")
        );
    }
}

#[test]
fn single_script_argument_is_borrowed() {
    let command = words(&["pwsh", "-NoProfile", "-Command", "Get-ChildItem"]);
    assert_eq!(
        read(&command, PowershellKind::Pwsh),
        Invocation {
            run: Run::Command {
                start: 3,
                script: Cow::Borrowed("Get-ChildItem"),
            },
            plain: true,
        }
    );
}

#[test]
fn command_spellings() {
    for shell in ["pwsh", "powershell"] {
        for flag in [
            "-c",
            "-co",
            "-com",
            "-comm",
            "-comma",
            "-comman",
            "-command",
            "-Command",
            "-COMMAND",
            "-cOmM",
            "--command",
            "--c",
            "/c",
            "/Command",
            "\u{2013}c",
            "\u{2014}Command",
            "\u{2015}command",
            "\u{2013}\u{2013}command",
            " -Command ",
            "\t-c\n",
        ] {
            assert_eq!(
                script_of(&[shell, flag, "git push"]).as_deref(),
                Some("git push"),
                "{shell} {flag:?}"
            );
        }
    }
}

#[test]
fn switches_before_the_script() {
    for shell in ["pwsh", "powershell.exe"] {
        for flags in [
            &["-NoProfile"][..],
            &["-nop"],
            &["-NoLogo", "-NonInteractive"],
            &["-nol", "-noni"],
            &["-STA"],
            &["-mta"],
            &["", "-NoProfile", " "],
            &["-ExecutionPolicy", "Bypass"],
            &["-ex", "Bypass"],
            &["-ep", "RemoteSigned"],
            &["-WindowStyle", "Hidden"],
            &["-w", "Hidden"],
            &["-OutputFormat", "Text"],
            &["-o", "Text"],
            &["-of", "XML"],
            &["-InputFormat", "Text"],
            &["-inp", "Text"],
            &["-if", "Text"],
            // A value is never a parameter.
            &["-ep", "-Command"],
        ] {
            let mut line = vec![shell];
            line.extend_from_slice(flags);
            line.extend(["-c", "git push"]);
            assert_eq!(script_of(&line).as_deref(), Some("git push"), "{line:?}");
        }
    }
    // PowerShell 7 only.
    for flags in [
        &["-WorkingDirectory", "C:\\"][..],
        &["-wd", "C:\\"],
        &["-wo", "C:\\"],
        &["-NoProfileLoadTime"],
    ] {
        let mut line = vec!["pwsh"];
        line.extend_from_slice(flags);
        line.extend(["-c", "git push"]);
        assert_eq!(script_of(&line).as_deref(), Some("git push"), "{line:?}");
        line[0] = "powershell";
        assert!(is_unreadable(&line), "{line:?}");
    }
}

#[test]
fn plain_is_only_a_command_with_plain_switches() {
    let plain = |line: &[&str]| {
        let command = words(line);
        read(&command, PowershellKind::of(&command[0])).plain
    };
    assert!(plain(&["pwsh", "-c", "x"]));
    assert!(plain(&[
        "pwsh",
        "-NoLogo",
        "-NoProfile",
        "-NonInteractive",
        "-c",
        "x"
    ]));
    assert!(plain(&["powershell", "-sta", "-Command", "x"]));
    assert!(!plain(&["pwsh", "-ep", "Bypass", "-c", "x"]));
    assert!(!plain(&["pwsh", "-wd", "C:\\", "-c", "x"]));
    assert!(!plain(&["pwsh", "-NoProfileLoadTime", "-c", "x"]));
    assert!(!plain(&["pwsh", "-cwa", "x"]));
    assert!(!plain(&["pwsh", "-ec", &encode("x")]));
    assert!(!plain(&["pwsh", "-File", "x.ps1"]));
}

#[test]
fn encoded_command_is_decoded() {
    let encoded = encode("git push; Write-Output 'é ✓'");
    for shell in ["pwsh", "powershell.exe"] {
        for flag in [
            "-EncodedCommand",
            "-encodedcommand",
            "-e",
            "-en",
            "-enc",
            "-encodedc",
            "-ec",
            "--ec",
            "/e",
        ] {
            assert_eq!(
                script_of(&[shell, "-NoProfile", flag, &encoded]).as_deref(),
                Some("git push; Write-Output 'é ✓'"),
                "{shell} {flag}"
            );
        }
        // Parameters with no effect on what runs may follow it.
        assert_eq!(
            script_of(&[shell, "-ec", &encoded, "-NoProfile", "-ep", "Bypass"]).as_deref(),
            Some("git push; Write-Output 'é ✓'"),
            "{shell}"
        );
    }
}

#[test]
fn encoded_command_fails_closed() {
    for shell in ["pwsh", "powershell.exe"] {
        for line in [
            // Not base64, not padded, not UTF-16LE (odd length, lone
            // surrogate), or no value.
            &[shell, "-ec", "!!!"][..],
            &[shell, "-ec", "Z2l0"],
            &[shell, "-ec", "Zw"],
            &[shell, "-ec", "Zw=="],
            &[shell, "-ec", "ANg="],
            &[shell, "-ec", " Z2kA "],
            &[shell, "-ec"],
            // Another script or file after it, which PowerShell refuses or
            // reads differently.
            &[shell, "-ec", "ZwBpAHQA", "-c", "git push"],
            &[shell, "-ec", "ZwBpAHQA", "-ec", "ZwBpAHQA"],
            &[shell, "-ec", "ZwBpAHQA", "-File", "x.ps1"],
            &[shell, "-ec", "ZwBpAHQA", "push"],
            &[shell, "-ec", "ZwBpAHQA", "-NoExit"],
            &[shell, "-ec", "ZwBpAHQA", "-ep"],
            // `-EncodedArguments`.
            &[shell, "-ea", "ZwBpAHQA", "-c", "git push"],
        ] {
            assert!(is_unreadable(line), "{line:?}: {}", run_of(line));
        }
    }
    // A negative control: the same value decodes.
    assert_eq!(
        script_of(&["pwsh", "-ec", "ZwBpAHQA"]).as_deref(),
        Some("git")
    );
}

#[test]
fn command_with_args() {
    for flag in [
        "-CommandWithArgs",
        "-commandwithargs",
        "-cwa",
        "-CWA",
        "--cwa",
    ] {
        assert_eq!(
            script_of(&["pwsh", flag, "git push"]).as_deref(),
            Some("git push"),
            "{flag}"
        );
        // Its arguments are `$args`, which the script may run.
        assert!(
            is_unreadable(&["pwsh", flag, "& $args[0] $args[1]", "git", "push"]),
            "{flag}"
        );
        // Windows PowerShell has no `-CommandWithArgs`.
        assert!(is_unreadable(&["powershell", flag, "git push"]), "{flag}");
    }
    // `-commandw` abbreviates nothing.
    assert!(is_unreadable(&["pwsh", "-commandw", "git push"]));
}

#[test]
fn file_reads_as_the_file_and_its_arguments() {
    for shell in ["pwsh", "powershell.exe"] {
        for flag in ["-File", "-file", "-f", "-fi", "-fil", "--file", "/File"] {
            let command = words(&[shell, "-NoProfile", flag, "deploy.ps1", "-Force", "x"]);
            assert_eq!(
                read(&command, PowershellKind::of(shell)).run,
                Run::File(&command[3..]),
                "{shell} {flag}"
            );
        }
        // From stdin, or no file.
        assert!(is_unreadable(&[shell, "-File", "-"]), "{shell}");
        assert!(is_unreadable(&[shell, "-File"]), "{shell}");
    }
    // PowerShell 7 runs its first positional argument as a file.
    let command = words(&["pwsh", "-NoProfile", "deploy.ps1", "-c", "x"]);
    assert_eq!(
        read(&command, PowershellKind::Pwsh).run,
        Run::File(&command[2..])
    );
    // Windows PowerShell reads it as a script, but a `powershell` may be
    // PowerShell 7: neither is assumed.
    assert!(is_unreadable(&["powershell.exe", "git", "push"]));
    assert!(is_unreadable(&["powershell.exe", "-NoProfile", "git push"]));
}

#[test]
fn unknown_and_unsupported_parameters_fail_closed() {
    for shell in ["pwsh", "powershell.exe"] {
        for flags in [
            &["-NoExit"][..],
            &["-noe"],
            &["-Interactive"],
            &["-Login"],
            &["-Version", "2"],
            &["-v"],
            &["-Help"],
            &["-?"],
            &["-ConfigurationName", "x"],
            &["-SettingsFile", "x.json"],
            &["-ConfigurationFile", "x.pssc"],
            &["-CustomPipeName", "x"],
            &["-PSConsoleFile", "x.psc1"],
            &["-ServerMode"],
            &["-s"],
            &["-Foo"],
            &["-NonInteractiv3"],
            // Too short to abbreviate anything, or ambiguous.
            &["-no"],
            &["-n"],
            &["-st"],
            &["-in"],
            &["-con"],
            // Not a parameter PowerShell splits at `:`.
            &["-Command:git"],
            // Only dashes.
            &["-"],
            &["--"],
            &["\u{2013}"],
            // Letters .NET folds differently (`ı` → `I`, `K` → `k`).
            &["-noprofıle"],
            &["-\u{212a}"],
            // White space PowerShells trim differently.
            &["\u{a0}-NoProfile"],
            &["-NoProfile\u{2003}"],
            &["\u{2003}"],
        ] {
            let mut line = vec![shell];
            line.extend_from_slice(flags);
            line.extend(["-Command", "git push"]);
            assert!(is_unreadable(&line), "{line:?}: {}", run_of(&line));
        }
        // A value missing.
        assert!(is_unreadable(&[shell, "-ExecutionPolicy"]), "{shell}");
        assert!(is_unreadable(&[shell, "-nop", "-w"]), "{shell}");
        // From stdin, or no script.
        assert!(is_unreadable(&[shell, "-Command", "-"]), "{shell}");
        assert!(is_unreadable(&[shell, "-c"]), "{shell}");
        assert!(is_unreadable(&[shell, "-NoProfile", "-c"]), "{shell}");
    }
    // Where PowerShell 7 and Windows PowerShell read a parameter
    // differently, `powershell` is not read: `-i` is `-Interactive` in 7
    // and `-InputFormat` in 5.1; `-—c` is a file in 7.
    assert!(is_unreadable(&[
        "powershell",
        "-i",
        "Text",
        "-c",
        "git push"
    ]));
    assert!(is_unreadable(&["powershell", "-\u{2014}c", "git push"]));
    assert!(is_unreadable(&["powershell", "/-c", "git push"]));
    assert!(is_unreadable(&["pwsh", "-\u{2014}c", "git push"]));
}

#[test]
fn no_script_is_interactive() {
    for line in [
        &["pwsh"][..],
        &["powershell.exe", "-NoProfile"],
        &["pwsh", "-NoLogo", "", "-ep", "Bypass"],
    ] {
        let command = words(line);
        assert_eq!(
            read(&command, PowershellKind::of(&command[0])).run,
            Run::Interactive,
            "{line:?}"
        );
    }
}
