use super::quote_crt_arg;
use super::sandbox_command_line;
use pretty_assertions::assert_eq;
use std::collections::HashMap;
use std::path::Path;

/// A working directory without any batch file in it.
fn no_cwd() -> &'static Path {
    Path::new("corbanu-sec-2718-missing-dir")
}

fn argv(items: &[&str]) -> Vec<String> {
    items.iter().map(|item| (*item).to_string()).collect()
}

fn line(items: &[&str]) -> String {
    sandbox_command_line(&argv(items), &HashMap::new(), no_cwd()).expect("serialize")
}

fn refused(items: &[&str], env: &HashMap<String, String>) -> String {
    format!(
        "{:#}",
        sandbox_command_line(&argv(items), env, no_cwd()).expect_err("must be refused")
    )
}

/// Splits a command line the way the CRT (and Rust's std) does for a
/// program's arguments after its name.
fn crt_split(text: &str) -> Vec<String> {
    let mut args = Vec::new();
    let mut chars = text.chars().peekable();
    loop {
        while chars.peek().is_some_and(|c| *c == ' ' || *c == '\t') {
            chars.next();
        }
        if chars.peek().is_none() {
            return args;
        }
        let mut arg = String::new();
        let mut in_quotes = false;
        while let Some(&c) = chars.peek() {
            if !in_quotes && (c == ' ' || c == '\t') {
                break;
            }
            chars.next();
            match c {
                '\\' => {
                    let mut backslashes = 1;
                    while chars.peek() == Some(&'\\') {
                        chars.next();
                        backslashes += 1;
                    }
                    if chars.peek() == Some(&'"') {
                        arg.push_str(&"\\".repeat(backslashes / 2));
                        if backslashes % 2 == 1 {
                            chars.next();
                            arg.push('"');
                        }
                    } else {
                        arg.push_str(&"\\".repeat(backslashes));
                    }
                }
                '"' if in_quotes && chars.peek() == Some(&'"') => {
                    chars.next();
                    arg.push('"');
                }
                '"' => in_quotes = !in_quotes,
                _ => arg.push(c),
            }
        }
        args.push(arg);
    }
}

/// What cmd hands the program for an escaped tail, failing on anything cmd
/// would act on: a `%` that could start an expansion of an existing name,
/// or an unescaped quote, operator, redirection, parenthesis or `!`.
fn cmd_unescape(text: &str) -> String {
    // A program token in real quotes: literal to cmd, but for `%`.
    let (program, text) = match text.strip_prefix('"') {
        Some(rest) => {
            let (program, rest) = rest.split_once('"').expect("closing program quote");
            assert!(
                !program.contains('%'),
                "a `%` in the quoted program {program:?}"
            );
            (format!("\"{program}\""), rest)
        }
        None => (String::new(), text),
    };
    program + &cmd_unescape_rest(text)
}

fn cmd_unescape_rest(text: &str) -> String {
    let chars: Vec<char> = text.chars().collect();
    for (index, c) in chars.iter().enumerate() {
        if *c == '%' {
            assert_eq!(
                chars.get(index + 1),
                Some(&'^'),
                "a `%` not followed by `^` in {text:?}"
            );
        }
    }
    let mut out = String::new();
    let mut iter = chars.into_iter();
    while let Some(c) = iter.next() {
        match c {
            '^' => out.push(iter.next().expect("a caret escapes a character")),
            '&' | '|' | '<' | '>' | '(' | ')' | '"' | '!' => {
                panic!("unescaped {c:?} in {text:?}")
            }
            _ => out.push(c),
        }
    }
    out
}

/// The tail cmd parses after `/s /c "`, with `/S` removing only the quotes
/// this module added.
fn cmd_tail(serialized: &str) -> &str {
    let (_, rest) = serialized
        .split_once(" /c \"")
        .or_else(|| serialized.split_once(" /C \""))
        .expect("command switch");
    rest.strip_suffix('"').expect("closing quote")
}

const NASTY: &[&str] = &[
    "plain",
    "",
    " ",
    "two words",
    "q\"q",
    "\"",
    "\"\"",
    "trailing\\",
    "C:\\Program Files\\x\\",
    "back\\\\\"slash",
    "a&b",
    "a && b",
    "a|b",
    "a || b",
    "<in",
    ">out",
    "2>&1",
    "(group)",
    "^caret^^",
    "%PATH%",
    "%PATH:~0,3%",
    "%%",
    "%",
    "end%",
    "%\"",
    "!PATH!",
    "!",
    "@echo",
    ":label",
    ",;=",
    "x\" & echo INJECTED & \"",
    "\" | calc | \"",
    "x\" & echo INJECTED>injected.txt & rem \"",
    "^\" & echo INJECTED",
    "%COMSPEC% /c echo INJECTED",
    "h\u{e9}llo \u{2713} \u{65e5}\u{672c}",
    "z\u{2603}%\u{2603}%",
];

#[test]
fn ordinary_programs_keep_the_crt_join() {
    assert_eq!(
        line(&["pwsh.exe", "-Command", "Write-Output \"hello world\""]),
        "pwsh.exe -Command \"Write-Output \\\"hello world\\\"\""
    );
    // Not cmd, however close the name.
    assert_eq!(line(&["mycmd.exe", "/c", "a&b"]), "mycmd.exe /c a&b");
    assert_eq!(
        line(&["C:\\cmd.exe.d\\tool.exe", "/c", "q\"q"]),
        "C:\\cmd.exe.d\\tool.exe /c \"q\\\"q\""
    );
}

#[test]
fn a_single_command_reaches_cmd_verbatim() {
    // #2702: CRT quoting turned these quotes into `\"`, which cmd doesn't
    // understand.
    assert_eq!(
        line(&[
            "cmd.exe",
            "/c",
            "\"C:\\Program Files\\PowerShell\\7\\pwsh.exe\" -NoProfile -EncodedCommand abc==",
        ]),
        "cmd.exe /d /v:off /s /c \"\"C:\\Program Files\\PowerShell\\7\\pwsh.exe\" -NoProfile -EncodedCommand abc==\""
    );
    assert_eq!(
        line(&[
            "C:\\Windows\\System32\\cmd.exe",
            "/d",
            "/c",
            "echo %CODEX_ELEVATED_TEST% & exit /b 23",
        ]),
        "C:\\Windows\\System32\\cmd.exe /d /v:off /s /c \"echo %CODEX_ELEVATED_TEST% & exit /b 23\""
    );
    for text in NASTY {
        let serialized = line(&["cmd", "/c", text]);
        assert_eq!(cmd_tail(&serialized), *text, "{serialized}");
    }
}

#[test]
fn caller_switches_are_kept_and_not_repeated() {
    assert_eq!(
        line(&[
            "C:\\Program Files\\cmd.exe",
            "/D",
            "/S",
            "/V:ON",
            "/Q",
            "/C",
            "echo !x!"
        ]),
        "\"C:\\Program Files\\cmd.exe\" /D /S /V:ON /Q /C \"echo !x!\""
    );
    assert_eq!(
        line(&["CMD.EXE", "/v:off", "/e:on", "/k", "dir"]),
        "CMD.EXE /d /v:off /e:on /s /k \"dir\""
    );
    assert_eq!(line(&["cmd", "/r"]), "cmd /d /v:off /s /r \"\"");
    // No command switch: cmd parses no command line.
    assert_eq!(line(&["cmd.exe", "/q"]), "cmd.exe /q");
}

#[test]
fn several_elements_reach_the_program_unchanged() {
    assert_eq!(
        line(&[
            "cmd.exe", "/c", "prog.exe", "a&b", "x y", "q\"q", "", "%PATH%", "!X!"
        ]),
        "cmd.exe /d /v:off /s /c \"prog.exe a^&b ^\"x y^\" ^\"q\\^\"q^\" ^\"^\" ^\"%^PATH%^\" ^!X^!\""
    );
    for text in NASTY {
        let items = ["cmd.exe", "/c", "C:\\tools\\dump.exe", text, "next"];
        let serialized = line(&items);
        let handed_on = cmd_unescape(cmd_tail(&serialized));
        assert_eq!(
            crt_split(&handed_on),
            argv(&items[2..]),
            "{text:?} -> {serialized}"
        );
    }
    // Every pair, so no argument can change how the next one is read.
    for first in NASTY {
        for second in NASTY {
            let items = ["cmd.exe", "/c", "dump.exe", first, second];
            let handed_on = cmd_unescape(cmd_tail(&line(&items)));
            assert_eq!(crt_split(&handed_on), argv(&items[2..]));
        }
    }
}

#[test]
fn awkward_program_tokens_are_quoted() {
    for program in [
        "@prog",
        ":prog",
        "(prog",
        "a,b.exe",
        "a=b.exe",
        "a;b.exe",
        "a b.exe",
        "C:\\Program Files (x86)\\a&b\\x.exe",
    ] {
        let items = ["cmd", "/c", program, "arg"];
        let handed_on = cmd_unescape(cmd_tail(&line(&items)));
        assert!(handed_on.starts_with('"'), "{program:?} -> {handed_on}");
        assert_eq!(crt_split(&handed_on), argv(&items[2..]));
    }
    let none = HashMap::new();
    for program in ["p%x%.exe", "p\"q.exe"] {
        assert!(
            refused(&["cmd", "/c", program, "arg"], &none).contains("program"),
            "{program:?}"
        );
    }
    // Builtins stay bare so cmd still finds them.
    assert_eq!(
        line(&["cmd", "/c", "echo", "a&b"]),
        "cmd /d /v:off /s /c \"echo a^&b\""
    );
}

#[test]
fn very_long_lines_are_passed_whole() {
    let long = "A".repeat(9_000);
    let serialized = line(&["cmd", "/c", &long]);
    assert_eq!(cmd_tail(&serialized), long);
    let items = ["cmd", "/c", "dump.exe", &long, "&echo INJECTED"];
    let handed_on = cmd_unescape(cmd_tail(&line(&items)));
    assert_eq!(crt_split(&handed_on), argv(&items[2..]));
}

#[test]
fn what_cannot_be_serialized_faithfully_is_refused() {
    let none = HashMap::new();
    assert!(refused(&["cmd", "/c", "echo a\0& calc"], &none).contains("NUL"));
    assert!(refused(&["cmd", "/c echo hi"], &none).contains("unsupported"));
    assert!(refused(&["cmd", "/c:x", "dir"], &none).contains("unsupported"));
    assert!(refused(&["cmd", "/v:maybe", "/c", "dir"], &none).contains("unsupported"));
    assert!(refused(&["cmd", "\"/c\"", "dir"], &none).contains("unsupported"));
    assert!(refused(&["cmd", "/v:on", "/c", "dump.exe", "!x!"], &none).contains("/V:ON"));
    assert!(refused(&["cmd", "/v", "/c", "dump.exe", "x"], &none).contains("/V:ON"));
    assert!(refused(&["cmd", "/c", "dump.exe", "a\nb"], &none).contains("line break"));
    assert!(refused(&["cmd", "/c", "dump.exe", "a\rb"], &none).contains("line break"));
    let caret_var = HashMap::from([("^PATH".to_string(), "x & calc".to_string())]);
    assert!(refused(&["cmd", "/c", "dump.exe", "%PATH%"], &caret_var).contains("^PATH"));
    // Without a `%` that variable can't be reached.
    assert_eq!(
        sandbox_command_line(&argv(&["cmd", "/c", "dump.exe", "x"]), &caret_var, no_cwd())
            .expect("ok"),
        "cmd /d /v:off /s /c \"dump.exe x\""
    );
}

#[test]
fn crt_quoting_is_unchanged() {
    assert_eq!(quote_crt_arg("plain"), "plain");
    assert_eq!(quote_crt_arg(""), "\"\"");
    assert_eq!(quote_crt_arg("a b\\"), "\"a b\\\\\"");
    assert_eq!(quote_crt_arg("q\"q"), "\"q\\\"q\"");
    for text in NASTY {
        assert_eq!(crt_split(&quote_crt_arg(text)), vec![(*text).to_string()]);
    }
}

#[test]
fn cmd_spellings_windows_accepts_are_recognized() {
    for program in [
        "cmd.exe.",
        "cmd.exe ",
        "cmd.exe. .",
        "C:\\Windows\\System32\\cmd.exe::$DATA",
        "C:cmd.exe",
        "c:/windows/system32/CMD",
    ] {
        let serialized = line(&[program, "/c", "dump.exe", "a&b"]);
        assert!(
            serialized.ends_with("/s /c \"dump.exe a^&b\""),
            "{program:?} -> {serialized}"
        );
    }
}

#[test]
fn call_is_refused_with_separate_arguments() {
    let none = HashMap::new();
    for program in ["call", "CALL", "call.", "call:x", "Call\\prog.exe"] {
        assert!(
            refused(&["cmd", "/c", program, "prog.exe", "%SECRET%"], &none).contains("CALL"),
            "{program:?}"
        );
    }
    // Other names starting with the same letters are programs.
    assert_eq!(
        line(&["cmd", "/c", "callme.exe", "a&b"]),
        "cmd /d /v:off /s /c \"callme.exe a^&b\""
    );
    // As a typed command line, CALL keeps its cmd meaning.
    assert_eq!(
        line(&["cmd", "/c", "call prog.exe %X%"]),
        "cmd /d /v:off /s /c \"call prog.exe %X%\""
    );
}

#[test]
fn batch_files_refuse_what_cmd_would_act_on() {
    let none = HashMap::new();
    for program in ["x.bat", "C:\\s\\X.CMD", "x.bat.", "x.cmd::$DATA"] {
        for arg in [
            "a\" & calc & \"",
            "a&calc",
            "%PATH%",
            "!x!",
            "a|b",
            "(x)",
            "a^b",
            "a\nb",
        ] {
            assert!(
                refused(&[program, arg], &none).contains("batch file"),
                "{program:?} {arg:?}"
            );
        }
    }
    assert!(refused(&["C:\\a&b\\x.bat", "plain"], &none).contains("batch file"));
    // A named stream, as Windows reads it.
    assert!(refused(&["C:\\w\\x.txt:y.bat", "a&b"], &none).contains("batch file"));
    // A batch file run by cmd /c gets the same refusal.
    for items in [
        ["cmd", "/c", "x.bat", "a&b"],
        ["cmd", "/c", "C:\\dir x\\y.cmd", "%X%"],
        ["cmd", "/c", "x.cmd.", "!x!"],
    ] {
        assert!(refused(&items, &none).contains("batch file"), "{items:?}");
    }
    assert_eq!(
        line(&["cmd", "/c", "C:\\dir x\\y.cmd", "plain", "two words"]),
        "cmd /d /v:off /s /c \"\"C:\\dir x\\y.cmd\" plain ^\"two words^\"\""
    );
    // Plain arguments keep the CRT join.
    assert_eq!(
        line(&["C:\\scripts\\x.bat", "plain", "two words", ""]),
        "C:\\scripts\\x.bat plain \"two words\" \"\""
    );
}

#[test]
fn nested_cmd_and_start_are_refused_with_metacharacters() {
    let none = HashMap::new();
    // The #2718 reproducers.
    assert!(refused(&["cmd", "/c", "cmd", "/c", "x", "a&b"], &none).contains("re-parse"));
    assert!(refused(&["cmd", "/c", "start", "x.bat", "a&b"], &none).contains("re-parse"));
    for program in [
        "cmd",
        "CMD",
        "cmd.exe",
        "Cmd.Exe.",
        "C:\\Windows\\System32\\cmd.exe",
        "c:/windows/system32/CMD.EXE",
        "C:\\Windows\\System32\\cmd.exe::$DATA",
        "C:\\Program Files\\x\\cmd.exe",
        "cmd.exe/c",
        "cmd/c",
        "start",
        "START",
        "Start.",
        "start/b",
        "start+x",
        "start\\..\\x.exe",
        "cmd.exe+x",
        " cmd.exe",
        "cmd.exe ",
        "C:cmd",
        // Not a plain path: cmd may cut a command name from it at any
        // whitespace or other character (#2718 review r2).
        "C:\\Windows\\System32\\cmd.exe\u{0B}",
        "cmd\u{0C}",
        "cmd.exe\u{A0}",
        "cmd\u{FF}x",
        "npm\tx",
        "x=y",
        "@x",
        "C:\\Program Files (x86)\\x\\tool.exe",
        "C:\\Users\\Jos\u{e9}\\tool.exe",
        // A `.`, `..` or empty component can make Windows read another one
        // as the file name.
        "C:\\Windows\\System32\\cmd.exe\\.",
        "C:\\Windows\\System32\\cmd.exe\\x\\..",
        "cmd.exe\\",
        ".\\tool.exe",
        "C:\\a\\\\cmd.exe",
        "",
    ] {
        for arg in [
            "a\"b", "%PATH%", "!x!", "a^b", "a&b", "a|b", "a<b", "a>b", "(x", "x)", "a\nb", "a\rb",
        ] {
            assert!(
                refused(&["cmd", "/c", program, "/c", "dump.exe", arg], &none).contains("re-parse"),
                "{program:?} {arg:?}"
            );
        }
    }
    // Without anything cmd acts on, they still run.
    assert_eq!(
        line(&["cmd", "/c", "cmd", "/c", "dump.exe", "two words"]),
        "cmd /d /v:off /s /c \"cmd /c dump.exe ^\"two words^\"\""
    );
    assert_eq!(
        line(&["cmd", "/c", "start", "/b", "dump.exe", "x"]),
        "cmd /d /v:off /s /c \"start /b dump.exe x\""
    );
    // `if` and `for` run a command taken from their arguments (#2718 review).
    for items in [
        &["cmd", "/c", "if", "1==1", "cmd", "/c", "dump.exe", "a&b"][..],
        &["cmd", "/c", "IF", "/i", "a==a", "npm", "a|b"],
        &["cmd", "/c", "if", "exist", "f", "x.bat", "a&b"],
        &["cmd", "/c", "if/i", "a==a", "dump.exe", "%X%"],
        &[
            "cmd", "/c", "if", "not", "defined", "X", "start", "x", "a&b",
        ],
        &["cmd", "/c", "for", "/f", "%i", "in", "(x)", "do", "y"],
        &["cmd", "/c", "@start", "x", "a&b"],
    ] {
        assert!(refused(items, &none).contains("re-parse"), "{items:?}");
    }
    assert_eq!(
        line(&["cmd", "/c", "if", "1==1", "dump.exe", "x"]),
        "cmd /d /v:off /s /c \"if 1==1 dump.exe x\""
    );
    // Other names containing the same letters are programs, and so is a
    // program in a directory named `cmd` or `start`.
    for program in [
        "cmdline.exe",
        "restart.exe",
        "startup",
        "iffy.exe",
        "format.exe",
        "foo.cmd.exe",
        "C:\\cmds\\x.exe",
        "C:\\Program Files\\Git\\cmd\\git.exe",
        "C:\\Start Menu\\start\\x.exe",
        "\\\\server\\share\\x-y_z.exe",
        "\\tools\\x.exe",
    ] {
        let items = ["cmd", "/c", program, "a&b"];
        assert_eq!(
            crt_split(&cmd_unescape(cmd_tail(&line(&items)))),
            argv(&items[2..]),
            "{program:?}"
        );
    }
    // A typed command line keeps its cmd meaning.
    assert_eq!(
        line(&["cmd", "/c", "cmd /c x a&b"]),
        "cmd /d /v:off /s /c \"cmd /c x a&b\""
    );
}

#[test]
fn batch_files_found_through_pathext_are_refused_with_metacharacters() {
    let root = tempfile::tempdir().expect("tempdir");
    let cwd = root.path().join("work");
    let bin = root.path().join("bin dir");
    let quoted = root.path().join("quoted");
    for dir in [&cwd, &bin, &quoted, &cwd.join("sub")] {
        std::fs::create_dir_all(dir).expect("mkdir");
    }
    for file in [
        bin.join("Shout.CMD"),
        bin.join("npm.cmd"),
        bin.join("git.exe"),
        quoted.join("yarn.bat"),
        cwd.join("local.cmd"),
        // `sub\rel.bat` in `sub` on Windows; a file of that name elsewhere.
        cwd.join("sub\\rel.bat"),
    ] {
        std::fs::write(file, "@echo off\r\n").expect("write");
    }
    // Two spellings of PATH: the child gets one of them, so both count.
    let env = HashMap::from([
        ("PATH".to_string(), "relative-missing".to_string()),
        (
            "Path".to_string(),
            format!("{};;\"{}\"", bin.display(), quoted.display()),
        ),
    ]);
    let run = |items: &[&str]| sandbox_command_line(&argv(items), &env, &cwd);
    for items in [
        ["cmd", "/c", "npm", "a&b"],
        ["cmd", "/c", "Shout", "a&b"],
        ["cmd", "/c", "yarn", "%X%"],
        ["cmd", "/c", "C:npm", "a&b"],
        ["cmd", "/c", "local", "a|b"],
        ["cmd", "/c", "sub\\rel", "!x!"],
    ] {
        let error = format!("{:#}", run(&items).expect_err("must be refused"));
        assert!(
            error.contains("resolves to a batch file"),
            "{items:?}: {error}"
        );
    }
    // Whatever cmd might cut from a token that isn't a plain path (`npm+x`,
    // `npm<VT>`) is refused before any lookup.
    for program in ["npm\u{0B}", "npm/x", "npm+x", "npm[x"] {
        let error = format!(
            "{:#}",
            run(&["cmd", "/c", program, "a&b"]).expect_err("must be refused")
        );
        assert!(error.contains("plain path"), "{program:?}: {error}");
    }
    // An empty component is named in the refusal (#2745).
    let error = format!(
        "{:#}",
        run(&["cmd", "/c", "bin\\\\npm", "a&b"]).expect_err("must be refused")
    );
    assert!(error.contains("no empty, `.` or `..` component"), "{error}");
    // A spelled-out extension with trailing dots or spaces runs the batch
    // file; cmd doesn't trim a bare `npm.` before PATHEXT (#2745).
    for program in ["npm.cmd.", "npm.cmd ", "npm.CMD. ."] {
        let error = format!(
            "{:#}",
            run(&["cmd", "/c", program, "a&b"]).expect_err("must be refused")
        );
        assert!(error.contains("batch file"), "{program:?}: {error}");
    }
    // An absolute path is refused either way: as a batch file, or as a
    // path that isn't plain (`/` off Windows, an 8.3 or non-ASCII TEMP).
    let rel = cwd.join("sub\\rel").display().to_string();
    let error = format!(
        "{:#}",
        run(&["cmd", "/c", &rel, "(x)"]).expect_err("refused")
    );
    assert!(
        error.contains("batch file") || error.contains("plain path"),
        "{error}"
    );
    // Plain arguments, and programs with no batch file, still run.
    assert_eq!(
        run(&["cmd", "/c", "npm", "install", "two words"]).expect("ok"),
        "cmd /d /v:off /s /c \"npm install ^\"two words^\"\""
    );
    assert_eq!(
        run(&["cmd", "/c", "git", "commit", "-m", "a&b"]).expect("ok"),
        "cmd /d /v:off /s /c \"git commit -m a^&b\""
    );
}
