//! D730 (#2702): a command run through `cmd.exe /C` in a sandbox session
//! reaches cmd as written, and a program's arguments reach it unchanged,
//! on both backends, with and without a ConPTY. Nothing injected runs.

use super::SandboxHome;
use super::collect_stdout_and_exit;
use super::command_identity::elevated::elevated_home;
use super::command_identity::elevated::run_elevated;
use super::command_identity::elevated::try_spawn_elevated;
use super::command_identity::run_legacy_with_tty;
use super::current_thread_runtime;
use super::legacy_process_test_guard;
use super::sandbox_home;
use super::spawn_windows_sandbox_session_legacy;
use super::system32_exe;
use super::user_owned_test_root;
use super::with_session_temp;
use super::workspace_roots_for;
use codex_protocol::models::PermissionProfile;
use pretty_assertions::assert_eq;
use std::fs;
use std::path::Path;
use std::time::Duration;

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
        // On main's CRT join this one ran its `echo` (#2702).
        "x\" & echo INJECTED>injected.txt & rem \"",
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
        assert!(
            !root.path().join(MARKER).exists(),
            "{label} argv: an injected command ran; stdout={stdout:?}"
        );
        assert_eq!(exit_code, 0, "{label} argv: stdout={stdout:?}");
        assert_eq!(read_dump(&dump, &stdout), arguments, "{label} argv");

        // A program path with spaces, given as its own element.
        let spaced = root.path().join("dir with space");
        fs::create_dir_all(&spaced).expect("create spaced dir");
        let spaced_exe = spaced.join("dump copy.exe");
        fs::copy(test_exe(), &spaced_exe).expect("copy the test exe");
        let dump = root.path().join("spaced.json");
        let spaced_arguments = vec!["a&b".to_string(), "x\" & echo INJECTED>injected.txt & rem \"".to_string()];
        let mut argv = vec![
            cmd(),
            "/c".to_string(),
            spaced_exe.display().to_string(),
            "--exact".to_string(),
            DUMP_ENTRY.to_string(),
            format!("{DUMP_ARG}{}", dump.display()),
        ];
        argv.extend(spaced_arguments.iter().cloned());
        let (stdout, exit_code) = run(backend, root.path(), home.path(), argv, tty).await;
        assert_eq!(exit_code, 0, "{label} spaced program: stdout={stdout:?}");
        assert_eq!(read_dump(&dump, &stdout), spaced_arguments, "{label} spaced program");

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
fn d730_legacy_tty_cmd_lines_arrive_unchanged() {
    let _guard = legacy_process_test_guard();
    let home = sandbox_home("d730-legacy-tty");
    check(Backend::Legacy, /*tty*/ true, &home);
}

#[test]
fn d730_elevated_cmd_lines_arrive_unchanged() {
    let Some(home) = elevated_home("d730-elevated") else {
        return;
    };
    let _guard = legacy_process_test_guard();
    check(Backend::Elevated, /*tty*/ false, &home);
}

#[test]
fn d730_elevated_tty_cmd_lines_arrive_unchanged() {
    let Some(home) = elevated_home("d730-elevated-tty") else {
        return;
    };
    let _guard = legacy_process_test_guard();
    check(Backend::Elevated, /*tty*/ true, &home);
}

/// Starts `argv` without a ConPTY: `None` when the sandbox refused to start
/// it, else its stdout and exit code.
async fn try_run(
    backend: Backend,
    root: &Path,
    home: &Path,
    argv: Vec<String>,
) -> Option<(String, i32)> {
    let spawned = match backend {
        Backend::Legacy => spawn_windows_sandbox_session_legacy(
            &PermissionProfile::workspace_write(),
            workspace_roots_for(root).as_slice(),
            home,
            argv,
            root,
            with_session_temp(home, std::env::vars().collect()),
            /*timeout_ms*/ Some(60_000),
            &[],
            &[],
            /*tty*/ false,
            /*stdin_open*/ false,
        )
        .await
        .ok()?,
        Backend::Elevated => try_spawn_elevated(root, home, argv, &[], /*tty*/ false)
            .await
            .ok()?,
    };
    let (stdout, exit_code) =
        collect_stdout_and_exit(spawned, home, Duration::from_secs(/*secs*/ 90)).await;
    Some((String::from_utf8_lossy(&stdout).into_owned(), exit_code))
}

/// #2718: a nested cmd (also behind `if`), `start`, and a batch file cmd
/// finds through PATHEXT
/// get their arguments with cmd's escapes removed and parse them again. With
/// an argument cmd acts on they are refused; nothing injected runs. The same
/// commands with a plain argument still run.
fn check_reparsing_programs(backend: Backend, home: &SandboxHome) {
    let root = user_owned_test_root();
    fs::write(root.path().join("x.bat"), "@exit /b 0\r\n").expect("write x.bat");
    fs::write(root.path().join("tool.cmd"), "@echo %*\r\n").expect("write tool.cmd");
    let injection = format!("x&echo.INJECTED>{MARKER}");
    let cases: [(&str, Vec<&str>, Option<&str>); 4] = [
        ("nested cmd", vec!["cmd", "/c", "echo.ran"], Some("ran ")),
        (
            "if",
            vec!["if", "1==1", "cmd", "/c", "echo.ran"],
            Some("ran "),
        ),
        ("start", vec!["start", "/b", "/wait", "x.bat"], None),
        ("PATHEXT batch", vec!["tool"], Some("")),
    ];
    current_thread_runtime().block_on(async {
        for (label, program, echoed) in &cases {
            let label = format!("{backend:?} {label}");
            let argv = |last: &str| {
                let mut argv = vec![cmd(), "/c".to_string()];
                argv.extend(program.iter().map(|part| (*part).to_string()));
                argv.push(last.to_string());
                argv
            };
            let outcome = try_run(backend, root.path(), home.path(), argv(&injection)).await;
            assert!(
                !root.path().join(MARKER).exists(),
                "{label}: an injected command ran; {outcome:?}"
            );
            if let Some((stdout, exit_code)) = &outcome {
                assert_ne!(*exit_code, 0, "{label}: not refused; stdout={stdout:?}");
            }

            let (stdout, exit_code) = try_run(backend, root.path(), home.path(), argv("plain"))
                .await
                .unwrap_or_else(|| panic!("{label}: a plain argument was refused"));
            assert_eq!(exit_code, 0, "{label} plain: stdout={stdout:?}");
            if let Some(echoed) = echoed {
                assert!(
                    stdout.contains(&format!("{echoed}plain")),
                    "{label} plain: stdout={stdout:?}"
                );
            }
        }
    });
}

#[test]
fn d2718_legacy_reparsing_programs_are_refused() {
    let _guard = legacy_process_test_guard();
    let home = sandbox_home("d2718-legacy");
    check_reparsing_programs(Backend::Legacy, &home);
}

#[test]
fn d2718_elevated_reparsing_programs_are_refused() {
    let Some(home) = elevated_home("d2718-elevated") else {
        return;
    };
    let _guard = legacy_process_test_guard();
    check_reparsing_programs(Backend::Elevated, &home);
}
