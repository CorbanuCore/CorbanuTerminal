//! #1721 (a′) and #1560 (`pl`): everyday tools still work when each
//! elevated command runs on a logon of its own, with its own default DACL.
//! win-1560m2 measured this list in elevated and normal sessions; the tools
//! that failed on main as well (HKCU writes, the rustup proxy in a normal
//! session) are left out.
//!
//! The tools must be on `PATH` (Git for Windows, Node, Python, Rust), so the
//! test runs only when `CODEX_SEC_WIN_WORKLOADS` is set, as
//! `windows-security-probes` does.

use super::*;

const WORKLOADS_ENV: &str = "CODEX_SEC_WIN_WORKLOADS";

/// `name` on `PATH`, if there.
fn on_path(name: &str) -> Option<PathBuf> {
    std::env::split_paths(&std::env::var_os("PATH")?)
        .map(|dir| dir.join(name))
        .find(|path| path.is_file())
}

fn found(name: &str) -> String {
    on_path(name)
        .unwrap_or_else(|| panic!("{name} is not on PATH ({WORKLOADS_ENV} needs it)"))
        .display()
        .to_string()
}

fn cmd_c(line: &str) -> Vec<String> {
    vec![
        system32_exe("cmd.exe").display().to_string(),
        "/d".to_string(),
        "/c".to_string(),
        format!("({line}) 2>&1"),
    ]
}

/// The workloads: a name, the command, what its output must contain, and
/// whether it runs in a ConPTY.
fn workloads() -> Vec<(&'static str, Vec<String>, Vec<&'static str>, bool)> {
    let powershell = system32_exe("WindowsPowerShell\\v1.0\\powershell.exe")
        .display()
        .to_string();
    let pwsh = found("pwsh.exe");
    let node = found("node.exe");
    let python = found("python.exe");
    let git_bash = PathBuf::from(std::env::var_os("ProgramFiles").unwrap_or_default())
        .join("Git\\bin\\bash.exe")
        .display()
        .to_string();
    let sysroot = std::process::Command::new("rustc")
        .args(["--print", "sysroot"])
        .output()
        .map(|out| String::from_utf8_lossy(&out.stdout).trim().to_string())
        .expect("rustc --print sysroot");
    let ps = |exe: &str| {
        vec![
            exe.to_string(),
            "-NoProfile".to_string(),
            "-Command".to_string(),
            "$x = & cmd /c echo w-ps; $x | Select-String w-ps | ForEach-Object { $_.Line }; \
             Get-ChildItem | Out-Null; $t = [IO.Path]::GetTempFileName(); \
             Set-Content $t 'x'; Remove-Item $t; 'w-ps-done'"
                .to_string(),
        ]
    };
    let cargo = |dir: &str, extra: &str| {
        cmd_c(&format!(
            "set CARGO_TARGET_DIR=&& {extra}set RUSTC={sysroot}\\bin\\rustc.exe&& cd {dir} \
             && {sysroot}\\bin\\cargo.exe build --offline && target\\debug\\tinyc.exe"
        ))
    };
    let bash = |args: &[&str]| {
        std::iter::once(git_bash.clone())
            .chain(args.iter().map(|arg| (*arg).to_string()))
            .collect::<Vec<_>>()
    };
    vec![
        (
            "cmd pipeline",
            cmd_c("echo w-pipe| findstr w-pipe"),
            vec!["w-pipe"],
            false,
        ),
        (
            "powershell",
            ps(&powershell),
            vec!["w-ps", "w-ps-done"],
            false,
        ),
        (
            "powershell (ConPTY)",
            ps(&powershell),
            vec!["w-ps-done"],
            true,
        ),
        ("pwsh", ps(&pwsh), vec!["w-ps", "w-ps-done"], false),
        ("pwsh (ConPTY)", ps(&pwsh), vec!["w-ps-done"], true),
        (
            "git init/status/commit",
            cmd_c(
                "git init -q gw && cd gw && echo x> a.txt && git status --porcelain && git add a.txt \
                 && git -c user.name=p -c user.email=p@example.com commit -qm w-commit \
                 && git log --oneline",
            ),
            vec!["?? a.txt", "w-commit"],
            false,
        ),
        (
            "git bash",
            bash(&["-lc", "echo w-bash > b.txt && cat b.txt"]),
            vec!["w-bash"],
            false,
        ),
        (
            "git bash (ConPTY)",
            bash(&["-lc", "echo w-bash > b2.txt && cat b2.txt"]),
            vec!["w-bash"],
            true,
        ),
        (
            "git bash --norc, mktemp (ConPTY)",
            bash(&[
                "--norc",
                "--noprofile",
                "-c",
                "echo w-bash; id; mktemp && echo w-mktemp",
            ]),
            vec!["w-bash", "w-mktemp"],
            true,
        ),
        (
            "node",
            vec![
                node,
                "-e".to_string(),
                "require('fs').writeFileSync('n.txt','x');console.log('w-node')".to_string(),
            ],
            vec!["w-node"],
            false,
        ),
        (
            "npm --version",
            cmd_c("npm --version && echo w-npm"),
            vec!["w-npm"],
            false,
        ),
        (
            "python + tempfile",
            vec![
                python,
                "-c".to_string(),
                "import tempfile;f=tempfile.NamedTemporaryFile(delete=False);f.write(b'x');\
                 f.close();open('p.txt','w').write('x');print('w-py')"
                    .to_string(),
            ],
            vec!["w-py"],
            false,
        ),
        (
            "python venv + pip",
            cmd_c("python -m venv venv && venv\\Scripts\\python -m pip --version && echo w-venv"),
            vec!["w-venv"],
            false,
        ),
        (
            "python venv (no pip) + ensurepip",
            cmd_c(
                "python -m venv --without-pip v3 && echo w-venv-nopip \
                 && v3\\Scripts\\python -m ensurepip --default-pip && echo w-ensurepip",
            ),
            vec!["w-venv-nopip", "w-ensurepip"],
            false,
        ),
        ("cargo build", cargo("tinyc", ""), vec!["w-cargo"], false),
        (
            "cargo build (CARGO_HOME in the workspace)",
            cargo("tinyd", "set CARGO_HOME=%CD%\\ch&& "),
            vec!["w-cargo"],
            false,
        ),
        (
            "workspace files",
            cmd_c(
                "echo x> w.txt && mkdir wd && move w.txt wd\\w2.txt >nul && echo y>> wd\\w2.txt \
                 && type wd\\w2.txt && del wd\\w2.txt && rmdir wd && echo w-files",
            ),
            vec!["w-files"],
            false,
        ),
        (
            "%TEMP% files",
            cmd_c(
                "echo x> %TEMP%\\w1721.txt && type %TEMP%\\w1721.txt \
                 && mkdir %TEMP%\\w1721.d && rmdir %TEMP%\\w1721.d \
                 && del %TEMP%\\w1721.txt && echo w-temp",
            ),
            vec!["w-temp"],
            false,
        ),
    ]
}

/// Runs `argv` as an elevated workspace-write command on `root`, with
/// `TEMP`/`TMP` in backslashes as users have them (CI sets `D:/tmp`).
async fn run(root: &Path, codex_home: &Path, argv: Vec<String>, tty: bool) -> (String, i32) {
    let env: Vec<(&str, String)> = ["TEMP", "TMP"]
        .into_iter()
        .filter_map(|key| Some((key, std::env::var(key).ok()?.replace('/', "\\"))))
        .collect();
    run_elevated(root, codex_home, argv, &env, tty).await
}

#[test]
fn elevated_workloads_work_on_their_own_logon() {
    if std::env::var_os(WORKLOADS_ENV).is_none() {
        eprintln!("elevated workloads: {WORKLOADS_ENV} is not set; skipped");
        return;
    }
    let Some(codex_home) = elevated_home("1721-workloads") else {
        return;
    };
    let _guard = legacy_process_test_guard();
    let root = user_owned_test_root();
    // Tiny crates, each its own workspace (the test root sits inside ours).
    for name in ["tinyc", "tinyd"] {
        let dir = root.path().join(name);
        fs::create_dir_all(dir.join("src")).expect("crate dir");
        fs::write(
            dir.join("Cargo.toml"),
            "[package]\nname = \"tinyc\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[workspace]\n",
        )
        .expect("Cargo.toml");
        fs::write(
            dir.join("src").join("main.rs"),
            "fn main() { println!(\"w-cargo\"); }\n",
        )
        .expect("main.rs");
    }
    let runtime = current_thread_runtime();
    let mut broken = Vec::new();
    for (name, argv, expected, tty) in workloads() {
        let started = Instant::now();
        let (stdout, code) = runtime.block_on(run(root.path(), codex_home.path(), argv, tty));
        let works = code == 0 && expected.iter().all(|part| stdout.contains(part));
        println!(
            "\nelevated workload {name}: {} exit={code} secs={}",
            if works { "works" } else { "BREAKS" },
            started.elapsed().as_secs()
        );
        if !works {
            broken.push(format!("{name}: exit={code} out={stdout}"));
        }
    }
    record_setup(codex_home.path());
    assert!(
        broken.is_empty(),
        "elevated workloads broke:\n{}\n{}",
        broken.join("\n"),
        sandbox_log(codex_home.path())
    );
}
