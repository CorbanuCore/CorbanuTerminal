//! D730 (#2702): a command run through `cmd.exe /C` in a sandbox session
//! reaches cmd as written, and a program's arguments reach it unchanged,
//! on both backends, with and without a ConPTY. Nothing injected runs.

use super::SandboxHome;
use super::command_identity::elevated::elevated_home;
use super::command_identity::elevated::run_elevated;
use super::command_identity::run_legacy_with_tty;
use super::current_thread_runtime;
use super::legacy_process_test_guard;
use super::sandbox_home;
use super::system32_exe;
use super::user_owned_test_root;
use pretty_assertions::assert_eq;
use std::fs;
use std::path::Path;

const DUMP_ENTRY: &str = "unified_exec::tests::cmd_line::cmd_line_dump_entry";
/// Precedes the dump file in the entry's arguments; what follows is dumped.
const DUMP_ARG: &str = "codex-2702-dump=";
const VAR: &str = "CODEX_2702_VAR";
/// Written in the command's working directory only if something injected ran.
const MARKER: &str = "injected.txt";
/// Expanding this would run the injection.
const VAR_VALUE: &str = "v & echo INJECTED > injected.txt";

/// Writes its arguments after [`DUMP_ARG`] as JSON to the file named there.
/// Does nothing otherwise.
#[test]
fn cmd_line_dump_entry() {
    let args: Vec<String> = std::env::args().collect();
    let Some(index) = args.iter().position(|arg| arg.starts_with(DUMP_ARG)) else {
        return;
    };
    let path = &args[index][DUMP_ARG.len()..];
    let json = serde_json::to_string(&args[index + 1..]).expect("encode arguments");
    fs::write(path, json).expect("write the dump");
}

#[derive(Clone, Copy, Debug)]
enum Backend {
    Legacy,
    Elevated,
}

async fn run(
    backend: Backend,
    root: &Path,
    home: &Path,
    argv: Vec<String>,
    tty: bool,
) -> (String, i32) {
    let env = [(VAR, VAR_VALUE.to_string())];
    match backend {
        Backend::Legacy => run_legacy_with_tty(root, home, argv, &env, tty).await,
        Backend::Elevated => run_elevated(root, home, argv, &env, tty).await,
    }
}

fn cmd() -> String {
    system32_exe("cmd.exe").display().to_string()
}

fn test_exe() -> String {
    std::env::current_exe()
        .expect("test exe")
        .display()
        .to_string()
}

fn read_dump(path: &Path, stdout: &str) -> Vec<String> {
    let json = fs::read_to_string(path)
        .unwrap_or_else(|err| panic!("no dump at {}: {err}; stdout={stdout:?}", path.display()));
    serde_json::from_str(&json).expect("decode dump")
}

/// Arguments that must reach the program byte for byte. None starts with
/// `-`, which the test harness would read as an option.
fn hostile_arguments() -> Vec<String> {
    [
        "plain",
        "two words",
        "",
        "q\"q",
        "\"",
        "trailing\\",
        "C:\\Program Files\\x\\",
        "a&b",
        "a|b",
        "<in",
        ">out",
        "2>&1",
        "(group)",
        "^caret^^",
        "%PATH%",
        "%CODEX_2702_VAR%",
        "%CODEX_2702_VAR:~0,3%",
        "%%",
        "end%",
        "!CODEX_2702_VAR!",
        "@at",
        ",;=",
        "x\" & echo INJECTED > \"injected.txt\" & \"",
        "a & echo INJECTED > injected.txt",
        "| cmd /c echo INJECTED > injected.txt",
        "^\" & echo INJECTED > injected.txt & ^\"",
        "%COMSPEC% /c echo INJECTED > injected.txt",
        "h\u{e9}llo \u{2713} \u{65e5}\u{672c} \u{1f600}",
    ]
    .into_iter()
    .map(str::to_string)
    .collect()
}

/// One backend and console mode: a program with hostile arguments, a typed
/// command line, and a control showing the marker would appear if
/// something injected ran.
fn check(backend: Backend, tty: bool, home: &SandboxHome) {
    let root = user_owned_test_root();
    let label = format!("{backend:?} tty={tty}");
    current_thread_runtime().block_on(async {
        // Several elements after /c: a program and its arguments.
        let dump = root.path().join("argv.json");
        let arguments = hostile_arguments();
        let mut argv = vec![
            cmd(),
            "/c".to_string(),
            test_exe(),
            "--exact".to_string(),
            DUMP_ENTRY.to_string(),
            format!("{DUMP_ARG}{}", dump.display()),
        ];
        argv.extend(arguments.iter().cloned());
        let (stdout, exit_code) = run(backend, root.path(), home.path(), argv, tty).await;
        assert_eq!(exit_code, 0, "{label} argv: stdout={stdout:?}");
        assert_eq!(read_dump(&dump, &stdout), arguments, "{label} argv");

        // One element after /c: a command line cmd parses as typed.
        // %VAR% expands (inside quotes, so its `&` stays literal) and
        // `!VAR!` doesn't (delayed expansion off).
        let dump = root.path().join("line.json");
        let line = format!(
            "\"{}\" --exact {DUMP_ENTRY} \"{DUMP_ARG}{}\" \"a & echo INJECTED > {MARKER} & b\" \"%{VAR}%\" \"!{VAR}!\" plain^&caret \"h\u{e9}llo \u{2713}\"",
            test_exe(),
            dump.display(),
        );
        let (stdout, exit_code) = run(
            backend,
            root.path(),
            home.path(),
            vec![cmd(), "/c".to_string(), line],
            tty,
        )
        .await;
        assert_eq!(exit_code, 0, "{label} line: stdout={stdout:?}");
        assert_eq!(
            read_dump(&dump, &stdout),
            [
                format!("a & echo INJECTED > {MARKER} & b"),
                VAR_VALUE.to_string(),
                format!("!{VAR}!"),
                "plain&caret".to_string(),
                "h\u{e9}llo \u{2713}".to_string(),
            ],
            "{label} line"
        );
        assert!(
            !root.path().join(MARKER).exists(),
            "{label}: an injected command ran"
        );

        // Control: the same redirection, typed as a command, does write.
        let control = "control.txt";
        let (stdout, exit_code) = run(
            backend,
            root.path(),
            home.path(),
            vec![
                cmd(),
                "/c".to_string(),
                format!("echo CONTROL > {control} & exit /b 7"),
            ],
            tty,
        )
        .await;
        assert_eq!(exit_code, 7, "{label} control: stdout={stdout:?}");
        assert!(
            root.path().join(control).exists(),
            "{label}: the control command did not write"
        );
    });
}

#[test]
fn d730_legacy_cmd_lines_arrive_unchanged() {
    let _guard = legacy_process_test_guard();
    let home = sandbox_home("d730-legacy");
    check(Backend::Legacy, /*tty*/ false, &home);
    check(Backend::Legacy, /*tty*/ true, &home);

    // Past cmd's 8191-character limit cmd refuses the line: nothing runs.
    let root = user_owned_test_root();
    let dump = root.path().join("long.json");
    let argv = vec![
        cmd(),
        "/c".to_string(),
        test_exe(),
        "--exact".to_string(),
        DUMP_ENTRY.to_string(),
        format!("{DUMP_ARG}{}", dump.display()),
        "A".repeat(9_000),
        format!("& echo INJECTED > {MARKER}"),
    ];
    let (stdout, exit_code) = current_thread_runtime().block_on(run(
        Backend::Legacy,
        root.path(),
        home.path(),
        argv,
        /*tty*/ false,
    ));
    assert_ne!(exit_code, 0, "stdout={stdout:?}");
    assert!(!dump.exists(), "an overlong line ran");
    assert!(
        !root.path().join(MARKER).exists(),
        "an injected command ran"
    );
}

#[test]
fn d730_elevated_cmd_lines_arrive_unchanged() {
    let Some(home) = elevated_home("d730-elevated") else {
        return;
    };
    let _guard = legacy_process_test_guard();
    check(Backend::Elevated, /*tty*/ false, &home);
    check(Backend::Elevated, /*tty*/ true, &home);
}
