//! #1791 and D400 (needs security sign-off): with the admin setup's
//! shared-folder denies, an elevated-backend command can't delete in folders
//! outside its writable roots that grant a group every signed-in token holds
//! delete-child (Full control), nor write in folders that grant Everyone
//! write, while listing and reading there and the everyday workloads
//! win-1791m measured still work. Without the denies (runs 42426631695 and
//! after) every delete and write below succeeded.
//!
//! The fixtures are vendor-like folders at fixed paths in `ProgramData` and
//! at the top of the system drive (D407), made by an elevated run before its
//! setup scans; a normal-session run reuses them and a setup an elevated run
//! recorded after them (`elevated_identity_tests.rs`).

use super::command_identity::elevated::elevated_home;
use super::command_identity::elevated::run_elevated;
use super::*;
use crate::SharedFolderDenyKind;
use crate::SharedFolderDenyKind::DeleteChild;
use crate::SharedFolderDenyKind::Write;
use crate::has_shared_folder_deny;
use crate::load_shared_folder_deny_record;
use crate::machine_shared_folder_record_dir;
use crate::token::LocalSid;
use pretty_assertions::assert_eq;

const VICTIM_FILE: &str = "victim-1791.txt";
const VICTIM_DIR: &str = "victim-1791-dir";
const VICTIM_SUB: &str = "victim-1791-sub";
const VICTIM_TEXT: &str = "victim";
/// Set where a skip must fail (CI).
const REQUIRE_ENV: &str = "CODEX_SEC_WIN_1791_REQUIRE";

fn env_dir(name: &str, fallback: &str) -> PathBuf {
    std::env::var_os(name)
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(fallback))
}

fn system_drive_root() -> PathBuf {
    PathBuf::from(format!("{}\\", env_dir("SystemDrive", "C:").display()))
}

type Fixture = (PathBuf, &'static str, &'static [SharedFolderDenyKind]);

/// Fixture folders, their grant, and the denies the setup must add.
fn fixtures() -> Vec<Fixture> {
    let base = env_dir("ProgramData", "C:\\ProgramData").join("CorbanuSecWin1791");
    vec![
        (
            base.join("users-full"),
            "*S-1-5-32-545:(OI)(CI)F",
            &[DeleteChild],
        ),
        (
            base.join("authenticated-users-full"),
            "*S-1-5-11:(OI)(CI)F",
            &[DeleteChild],
        ),
        (
            base.join("interactive-full"),
            "*S-1-5-4:(OI)(CI)F",
            &[DeleteChild],
        ),
        // Full control only for the subfolders directly below.
        (
            base.join("users-direct-subfolders"),
            "*S-1-5-32-545:(CI)(IO)(NP)F",
            &[DeleteChild],
        ),
        (base.join("everyone-modify"), "*S-1-1-0:(OI)(CI)M", &[Write]),
        // Everyone may also change permissions here.
        (
            base.join("everyone-full"),
            "*S-1-1-0:(OI)(CI)F",
            &[DeleteChild, Write],
        ),
        (
            system_drive_root().join("CorbanuSecWin1791Top"),
            "*S-1-5-32-545:(OI)(CI)F",
            &[DeleteChild],
        ),
    ]
}

fn sandbox_group_sid() -> String {
    let sid = crate::resolve_sid("CodexSandboxUsers").expect("the sandbox users' group");
    crate::string_from_sid_bytes(&sid).expect("SID string")
}

fn icacls(path: &Path, args: &[&str]) -> String {
    let output = std::process::Command::new(system32_exe("icacls.exe"))
        .arg(path)
        .args(args)
        .output()
        .expect("run icacls");
    let text = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(output.status.success(), "icacls {args:?} failed: {text}");
    text
}

/// Makes the fixtures (elevated: before the setup scans) or checks an
/// elevated run made them (normal session); `None` to skip.
fn prepare_fixtures() -> Option<Vec<Fixture>> {
    let fixtures = fixtures();
    if crate::setup::is_elevated().expect("elevation") {
        for (path, grant, _) in &fixtures {
            fs::create_dir_all(path).expect("create a fixture");
            icacls(path, &["/grant", grant]);
        }
    } else if fixtures.iter().any(|(path, _, _)| !path.is_dir()) {
        assert!(
            std::env::var_os(REQUIRE_ENV).is_none(),
            "sec-win-1791: no fixtures; an elevated run must make them first"
        );
        eprintln!("sec-win-1791: no fixtures from an elevated run; skipped");
        return None;
    }
    Some(fixtures)
}

/// Shared folders of the machine itself; any delete there is a hole too.
fn machine_shared_folders() -> Vec<PathBuf> {
    let public = env_dir("PUBLIC", "C:\\Users\\Public");
    [
        public.join("Documents"),
        public,
        env_dir("SystemRoot", "C:\\Windows").join("Temp"),
    ]
    .into_iter()
    .filter(|path| path.is_dir())
    .collect()
}

/// A file, an empty folder and a file in a subfolder, made by this
/// (unsandboxed) test; false if it can't make them there.
fn ensure_victims(target: &Path) -> bool {
    let made = (|| -> std::io::Result<()> {
        fs::write(target.join(VICTIM_FILE), VICTIM_TEXT)?;
        fs::create_dir_all(target.join(VICTIM_DIR))?;
        fs::create_dir_all(target.join(VICTIM_SUB))?;
        fs::write(target.join(VICTIM_SUB).join(VICTIM_FILE), VICTIM_TEXT)
    })();
    if let Err(err) = &made {
        eprintln!("sec-win-1791: no victims in {}: {err}", target.display());
    }
    made.is_ok()
}

fn cmd_argv(root: &Path, file: &str, body: &str) -> Vec<String> {
    let path = root.join(file);
    fs::write(&path, body.replace('\n', "\r\n")).expect("write a script");
    vec![
        system32_exe("cmd.exe").display().to_string(),
        "/d".to_string(),
        "/c".to_string(),
        path.display().to_string(),
    ]
}

fn session() -> &'static str {
    if crate::setup::is_elevated().expect("elevation") {
        "elevated session"
    } else {
        "normal session"
    }
}

fn command_env(temp: &Path) -> [(&'static str, String); 2] {
    let temp = temp.display().to_string();
    [("TEMP", temp.clone()), ("TMP", temp)]
}

#[test]
fn sec_win_1791_elevated_command_cannot_delete_or_write_in_shared_folders() {
    let Some(codex_home) = elevated_home("sec-win-1791-boundary") else {
        return;
    };
    let Some(fixtures) = prepare_fixtures() else {
        return;
    };
    let _guard = legacy_process_test_guard();
    let root = user_owned_test_root();
    let temp = user_owned_test_root();
    let mut deletes: Vec<PathBuf> = fixtures.iter().map(|(path, _, _)| path.clone()).collect();
    for fixture in &deletes {
        assert!(
            ensure_victims(fixture),
            "sec-win-1791: no victims in the fixture {}",
            fixture.display()
        );
    }
    deletes.extend(
        machine_shared_folders()
            .into_iter()
            .filter(|target| ensure_victims(target)),
    );
    let writes: Vec<PathBuf> = fixtures
        .iter()
        .filter(|(_, _, kinds)| kinds.contains(&Write))
        .map(|(path, _, _)| path.clone())
        .collect();
    // The first command runs the setup (in an elevated session, with the
    // scan), which may still add the denies; compare permissions after it.
    let (stdout, exit_code) = current_thread_runtime().block_on(run_elevated(
        root.path(),
        codex_home.path(),
        cmd_argv(root.path(), "setup.cmd", "@echo off\necho setup\n"),
        &command_env(temp.path()),
        /*tty*/ false,
    ));
    assert_eq!(
        exit_code,
        0,
        "stdout={stdout:?}\n{}",
        sandbox_log(codex_home.path())
    );
    // The command also tries to take the denies out (or replace the DACL or
    // the owner) on every fixture and its subfolder.
    let acls_before: Vec<(PathBuf, String)> = fixtures
        .iter()
        .flat_map(|(path, _, _)| [path.clone(), path.join(VICTIM_SUB)])
        .map(|path| {
            let acl = icacls(&path, &[]);
            (path, acl)
        })
        .collect();
    let mut script = String::from("@echo off\n");
    for (path, _) in &acls_before {
        let t = path.display();
        script.push_str(&format!(
            "icacls \"{t}\" /remove:d CodexSandboxUsers 2>&1\n\
             icacls \"{t}\" /grant *S-1-1-0:(OI)(CI)F 2>&1\n\
             icacls \"{t}\" /inheritance:r 2>&1\n\
             icacls \"{t}\" /setowner CodexSandboxOffline 2>&1\n"
        ));
    }
    for (i, target) in deletes.iter().enumerate() {
        let t = target.display();
        script.push_str(&format!(
            "dir /b \"{t}\" >nul 2>&1 && (echo READ {i} list ok) || (echo READ {i} list FAIL)\n\
             type \"{t}\\{VICTIM_FILE}\" >nul 2>&1 && (echo READ {i} file ok) || (echo READ {i} file FAIL)\n\
             del /q \"{t}\\{VICTIM_FILE}\" 2>&1\n\
             rmdir \"{t}\\{VICTIM_DIR}\" 2>&1\n\
             del /q \"{t}\\{VICTIM_SUB}\\{VICTIM_FILE}\" 2>&1\n\
             rmdir /s /q \"{t}\\{VICTIM_SUB}\" 2>&1\n"
        ));
    }
    for target in &writes {
        let t = target.display();
        script.push_str(&format!(
            "(echo x>\"{t}\\written-1791.txt\") 2>&1\n\
             mkdir \"{t}\\written-1791-dir\" 2>&1\n\
             (echo x>>\"{t}\\{VICTIM_FILE}\") 2>&1\n"
        ));
    }
    script.push_str("echo inside>inside.txt\nexit /b 0\n");
    let (stdout, exit_code) = current_thread_runtime().block_on(run_elevated(
        root.path(),
        codex_home.path(),
        cmd_argv(root.path(), "sec-win-1791.cmd", &script),
        &command_env(temp.path()),
        /*tty*/ false,
    ));
    assert!(
        exit_code == 0 && root.path().join("inside.txt").is_file(),
        "the command didn't run or couldn't write in its workspace: exit={exit_code} \
         stdout={stdout:?}\n{}",
        sandbox_log(codex_home.path())
    );

    let mut holes = Vec::new();
    for target in &deletes {
        for (what, path) in [
            ("a file", target.join(VICTIM_FILE)),
            ("an empty folder", target.join(VICTIM_DIR)),
            (
                "a file in a subfolder",
                target.join(VICTIM_SUB).join(VICTIM_FILE),
            ),
        ] {
            if !path.exists() {
                holes.push(format!("deleted {what} in {}", target.display()));
            }
        }
    }
    for target in &writes {
        for (what, written) in [
            ("a file", target.join("written-1791.txt").exists()),
            ("a folder", target.join("written-1791-dir").exists()),
            (
                "to a file",
                fs::read_to_string(target.join(VICTIM_FILE)).unwrap_or_default() != VICTIM_TEXT,
            ),
        ] {
            if written {
                holes.push(format!("wrote {what} in {}", target.display()));
            }
        }
    }
    let unreadable: Vec<&str> = stdout
        .lines()
        .filter(|line| line.starts_with("READ ") && line.ends_with(" FAIL"))
        .collect();
    assert!(
        holes.is_empty() && unreadable.is_empty(),
        "#1791/D400: the elevated command {holes:?}; reads failed: {unreadable:?}\n\
         stdout={stdout}\n{}",
        sandbox_log(codex_home.path())
    );
    let reads = stdout
        .lines()
        .filter(|line| line.starts_with("READ "))
        .count();
    assert_eq!(reads, deletes.len() * 2, "stdout={stdout}");

    for (path, before) in &acls_before {
        assert_eq!(
            &icacls(path, &[]),
            before,
            "#1791/D400: the elevated command changed the permissions of {}\nstdout={stdout}",
            path.display()
        );
    }

    // Each fixture's denies are in place, for the sandbox users' group.
    let group_sid = sandbox_group_sid();
    let group = LocalSid::from_string(&group_sid).expect("group SID");
    for (path, _, kinds) in &fixtures {
        for kind in *kinds {
            // SAFETY: a valid SID and an existing folder.
            let present = unsafe { has_shared_folder_deny(path, group.as_ptr(), *kind) };
            assert!(
                present.expect("read DACL"),
                "{kind:?} missing on {}: {}",
                path.display(),
                icacls(path, &[])
            );
        }
    }
    // An elevated run's own setup scanned and recorded them.
    if crate::setup::is_elevated().expect("elevation") {
        let record_dir = machine_shared_folder_record_dir().expect("record folder");
        let record = load_shared_folder_deny_record(&record_dir).expect("record");
        for (path, _, kinds) in &fixtures {
            for kind in *kinds {
                assert!(
                    record.entries.iter().any(|entry| entry.path == *path
                        && entry.kind == *kind
                        && entry.sid == group_sid
                        && !entry.homes.is_empty()),
                    "{kind:?} on {} not recorded",
                    path.display()
                );
            }
        }
        // What the scan found and cost on this machine, and every folder
        // this machine's setups have denied so far.
        for line in sandbox_log(codex_home.path()).lines() {
            if line.contains("shared-folder") {
                println!("sec-win-1791 setup log: {line}");
            }
        }
        for entry in &record.entries {
            println!(
                "sec-win-1791 recorded: {:?} on {} ({} homes)",
                entry.kind,
                entry.path.display(),
                entry.homes.len()
            );
        }
    }
    println!(
        "sec-win-1791 {}: {} folders kept their files and {} refused writes; reads worked",
        session(),
        deletes.len(),
        writes.len()
    );
}

const CMD_WORKLOAD: &str = r#"@echo off
echo hi>a.txt
copy /y a.txt b.txt >nul 2>&1
ren b.txt c.txt >nul 2>&1
if exist c.txt (echo STEP write-copy-rename ok) else (echo STEP write-copy-rename FAIL)
del /q a.txt c.txt >nul 2>&1
if exist c.txt (echo STEP delete-own FAIL) else if exist a.txt (echo STEP delete-own FAIL) else (echo STEP delete-own ok)
mkdir d\e >nul 2>&1
echo x>d\e\f.txt
rmdir /s /q d >nul 2>&1
if exist d (echo STEP rmdir-own-tree FAIL) else (echo STEP rmdir-own-tree ok)
del /q pre.txt >nul 2>&1
if exist pre.txt (echo STEP delete-preexisting-file FAIL) else (echo STEP delete-preexisting-file ok)
rmdir /s /q pre-dir >nul 2>&1
if exist pre-dir (echo STEP delete-preexisting-tree FAIL) else (echo STEP delete-preexisting-tree ok)
echo t>"%TEMP%\w1791-cmd.txt"
if exist "%TEMP%\w1791-cmd.txt" (echo STEP temp-write ok) else (echo STEP temp-write FAIL)
del /q "%TEMP%\w1791-cmd.txt" >nul 2>&1
if exist "%TEMP%\w1791-cmd.txt" (echo STEP temp-delete FAIL) else (echo STEP temp-delete ok)
powershell -NoProfile -NonInteractive -Command "Write-Output psok" > psout.txt 2>&1
findstr /c:psok psout.txt >nul && (echo STEP powershell-from-cmd ok) || (echo STEP powershell-from-cmd FAIL & type psout.txt)
"#;

const CMD_STEPS: [&str; 8] = [
    "write-copy-rename",
    "delete-own",
    "rmdir-own-tree",
    "delete-preexisting-file",
    "delete-preexisting-tree",
    "temp-write",
    "temp-delete",
    "powershell-from-cmd",
];

// win-1791m's `Start-Process` step is left out: it fails before and after any
// change, on a duplicate `Path`/`PATH` in the test environment.
const PS_WORKLOAD: &str = r#"$ErrorActionPreference = 'Stop'
function Step($n, [scriptblock]$b) { try { & $b | Out-Null; "STEP $n ok" } catch { "STEP $n FAIL $($_.Exception.Message)" } }
Step fileops { Set-Content p.txt hi; Copy-Item p.txt q.txt; Rename-Item q.txt r.txt; Remove-Item p.txt, r.txt; if (Test-Path r.txt) { throw 'still there' } }
Step tree { New-Item -ItemType Directory pd\e -Force; Set-Content pd\e\f.txt x; Remove-Item -Recurse -Force pd; if (Test-Path pd) { throw 'still there' } }
Step tempfile { $t = [IO.Path]::GetTempFileName(); Set-Content $t x; Remove-Item $t }
Step child { $o = & cmd.exe /c echo childok; if ("$o" -notmatch 'childok') { throw "got $o" } }
Step read-system { Get-ChildItem "$env:SystemRoot\System32\drivers\etc" }
"#;

const PS_STEPS: [&str; 5] = ["fileops", "tree", "tempfile", "child", "read-system"];

const GIT_WORKLOAD: &str = r#"@echo off
set GIT_TERMINAL_PROMPT=0
git init -q repo >g.log 2>&1 && (echo STEP init ok) || (echo STEP init FAIL & type g.log)
cd repo
echo x>f.txt
git add f.txt >g.log 2>&1 && git -c user.name=p -c user.email=p@e commit -qm one >>g.log 2>&1 && (echo STEP commit ok) || (echo STEP commit FAIL & type g.log)
git checkout -qb br >g.log 2>&1 && git rm -q f.txt >>g.log 2>&1 && git -c user.name=p -c user.email=p@e commit -qm two >>g.log 2>&1 && (echo STEP branch-rm-commit ok) || (echo STEP branch-rm-commit FAIL & type g.log)
git gc -q >g.log 2>&1 && (echo STEP gc ok) || (echo STEP gc FAIL & type g.log)
cd ..
git clone -q repo repo2 >g.log 2>&1 && (echo STEP clone-local ok) || (echo STEP clone-local FAIL & type g.log)
rmdir /s /q repo2 >nul 2>&1
if exist repo2 (echo STEP delete-clone FAIL) else (echo STEP delete-clone ok)
if exist "%ProgramFiles%\Git\bin\bash.exe" ("%ProgramFiles%\Git\bin\bash.exe" -c "echo bashok" > b.log 2>&1) else (echo nobash> b.log)
findstr /c:bashok b.log >nul && (echo STEP git-bash ok) || (echo STEP git-bash FAIL & type b.log)
"#;

const GIT_STEPS: [&str; 7] = [
    "init",
    "commit",
    "branch-rm-commit",
    "gc",
    "clone-local",
    "delete-clone",
    "git-bash",
];

const NODE_JS: &str = r#"const fs = require('fs'), os = require('os'), path = require('path'), cp = require('child_process');
function step(n, f) { try { f(); console.log('STEP ' + n + ' ok'); } catch (e) { console.log('STEP ' + n + ' FAIL ' + String(e && e.message).split('\n')[0]); } }
step('fs', () => { fs.mkdirSync('nd/e', { recursive: true }); fs.writeFileSync('nd/e/f', 'x'); fs.renameSync('nd/e/f', 'nd/e/g'); fs.rmSync('nd', { recursive: true }); if (fs.existsSync('nd')) throw new Error('still there'); });
step('tmpdir', () => { const d = fs.mkdtempSync(path.join(os.tmpdir(), 'n1791-')); fs.writeFileSync(path.join(d, 'f'), 'x'); fs.rmSync(d, { recursive: true }); });
step('child-exec', () => { const o = cp.execSync('cmd /c echo childok').toString(); if (!o.includes('childok')) throw new Error(o); });
step('child-spawn', () => { const r = cp.spawnSync(process.execPath, ['-e', 'console.log(42)']); if (r.error) throw r.error; if (!String(r.stdout).includes('42')) throw new Error('status ' + r.status + ' ' + r.stderr); });
"#;

const NODE_WORKLOAD: &str = r#"@echo off
set npm_config_cache=%CD%\.npm-cache
set npm_config_update_notifier=false
node w.js
cd app
call npm install ..\lib --no-audit --no-fund --offline >n.log 2>&1 && (echo STEP npm-install-local ok) || (echo STEP npm-install-local FAIL & type n.log)
node -e "console.log('v' + require('lib1791'))" > r.log 2>&1
findstr /c:v7 r.log >nul && (echo STEP npm-require ok) || (echo STEP npm-require FAIL & type r.log)
call npm run t >n.log 2>&1 && (echo STEP npm-run ok) || (echo STEP npm-run FAIL & type n.log)
cd ..
rmdir /s /q app >nul 2>&1
if exist app (echo STEP delete-node_modules FAIL) else (echo STEP delete-node_modules ok)
"#;

const NODE_STEPS: [&str; 8] = [
    "fs",
    "tmpdir",
    "child-exec",
    "child-spawn",
    "npm-install-local",
    "npm-require",
    "npm-run",
    "delete-node_modules",
];

const PYTHON_PY: &str = r#"import os, shutil, subprocess, sys, tempfile

def step(n, f):
    try:
        f()
        print(f"STEP {n} ok", flush=True)
    except Exception as e:
        msg = str(e).splitlines()[0] if str(e) else ""
        print(f"STEP {n} FAIL {type(e).__name__}: {msg}", flush=True)

def fs_ops():
    os.makedirs("pd/e", exist_ok=True)
    open("pd/e/f", "w").write("x")
    os.replace("pd/e/f", "pd/e/g")
    shutil.rmtree("pd")
    assert not os.path.exists("pd")

def temp():
    with tempfile.NamedTemporaryFile() as f:
        f.write(b"x")
    with tempfile.TemporaryDirectory() as d:
        open(os.path.join(d, "f"), "w").write("x")

def child():
    out = subprocess.run(["cmd", "/c", "echo", "childok"], capture_output=True, text=True, check=True).stdout
    assert "childok" in out, out

def sq(x):
    return x * x

def pool():
    import multiprocessing as mp
    with mp.Pool(2) as p:
        assert p.map(sq, [1, 2, 3]) == [1, 4, 9]

def venv():
    subprocess.run([sys.executable, "-m", "venv", "--without-pip", "venv"], check=True, capture_output=True)
    shutil.rmtree("venv")

if __name__ == "__main__":
    for n, f in [("fs", fs_ops), ("tempfile", temp), ("subprocess", child), ("multiprocessing-pool", pool), ("venv", venv)]:
        step(n, f)
"#;

const PYTHON_STEPS: [&str; 5] = [
    "fs",
    "tempfile",
    "subprocess",
    "multiprocessing-pool",
    "venv",
];

const CARGO_WORKLOAD: &str = r#"@echo off
set CARGO_TARGET_DIR=%CD%\target
set CARGO_TERM_COLOR=never
cargo new --vcs none --quiet hello >c.log 2>&1 && (echo STEP new ok) || (echo STEP new FAIL & type c.log)
cd hello
cargo build --offline --quiet >c.log 2>&1 && (echo STEP build ok) || (echo STEP build FAIL & type c.log)
cargo run --offline --quiet >c.log 2>&1 && (echo STEP run ok) || (echo STEP run FAIL & type c.log)
cargo test --offline --quiet >c.log 2>&1 && (echo STEP test ok) || (echo STEP test FAIL & type c.log)
cd ..
rmdir /s /q target hello >nul 2>&1
if exist hello (echo STEP delete-project FAIL) else (echo STEP delete-project ok)
"#;

const CARGO_STEPS: [&str; 5] = ["new", "build", "run", "test", "delete-project"];

/// Runs a workload; returns `(name/step, ok)` for each step it reports and
/// any expected step it didn't.
fn workload(
    codex_home: &Path,
    name: &str,
    root: &Path,
    temp: &Path,
    argv: Vec<String>,
    expected: &[&str],
) -> Vec<String> {
    let (stdout, exit_code) = current_thread_runtime().block_on(run_elevated(
        root,
        codex_home,
        argv,
        &command_env(temp),
        /*tty*/ false,
    ));
    let mut failed = Vec::new();
    for step in expected {
        let ok = format!("STEP {step} ok");
        if !stdout.lines().any(|line| line.trim() == ok) {
            failed.push(format!("{name}/{step}"));
        }
    }
    println!(
        "sec-win-1791 workload {name}: {}/{} steps ok (exit {exit_code})",
        expected.len() - failed.len(),
        expected.len()
    );
    if !failed.is_empty() {
        println!("--- {name} stdout ---\n{stdout}\n--- end ---");
    }
    failed
}

/// A workspace like a user's project folder, outside the checkout (whose
/// `rust-toolchain.toml` would make rustup install components).
fn private_root() -> TempDir {
    let root = tempfile::Builder::new()
        .prefix("sec-win-1791-")
        .tempdir_in(std::env::temp_dir().to_string_lossy().replace('/', "\\"))
        .expect("a workspace");
    let user = format!("*{}:(OI)(CI)F", current_user_sid());
    icacls(
        root.path(),
        &[
            "/inheritance:r",
            "/grant:r",
            "*S-1-5-18:(OI)(CI)F",
            "*S-1-5-32-544:(OI)(CI)F",
            user.as_str(),
        ],
    );
    icacls(root.path(), &["/setintegritylevel", "(OI)(CI)M"]);
    root
}

fn prepare_cmd_project(root: &Path) {
    fs::write(root.join("pre.txt"), "pre").expect("pre.txt");
    fs::create_dir_all(root.join("pre-dir").join("sub")).expect("pre-dir");
    fs::write(root.join("pre-dir").join("sub").join("f.txt"), "pre").expect("pre-dir file");
}

/// win-1791m's everyday list (58 of its 60 steps), with whatever shared-folder
/// denies the setup added on this machine.
#[test]
fn sec_win_1791_elevated_everyday_workloads_still_work() {
    let Some(codex_home) = elevated_home("sec-win-1791-workloads") else {
        return;
    };
    if prepare_fixtures().is_none() {
        return;
    }
    let _guard = legacy_process_test_guard();
    let home = codex_home.path();
    let root_dir = private_root();
    let temp_dir = private_root();
    let (root, temp) = (root_dir.path(), temp_dir.path());
    let mut failed = Vec::new();

    prepare_cmd_project(root);
    failed.extend(workload(
        home,
        "cmd",
        root,
        temp,
        cmd_argv(root, "w-cmd.cmd", CMD_WORKLOAD),
        &CMD_STEPS,
    ));

    fs::write(root.join("w.ps1"), PS_WORKLOAD).expect("w.ps1");
    let ps_args = |exe: PathBuf| {
        vec![
            exe.display().to_string(),
            "-NoProfile".to_string(),
            "-NonInteractive".to_string(),
            "-ExecutionPolicy".to_string(),
            "Bypass".to_string(),
            "-File".to_string(),
            root.join("w.ps1").display().to_string(),
        ]
    };
    let powershell = env_dir("SystemRoot", "C:\\Windows")
        .join("System32\\WindowsPowerShell\\v1.0\\powershell.exe");
    failed.extend(workload(
        home,
        "powershell",
        root,
        temp,
        ps_args(powershell),
        &PS_STEPS,
    ));
    match pwsh_path() {
        Some(pwsh) => failed.extend(workload(home, "pwsh", root, temp, ps_args(pwsh), &PS_STEPS)),
        None => failed.push("pwsh (not installed)".to_string()),
    }

    failed.extend(workload(
        home,
        "git",
        root,
        temp,
        cmd_argv(root, "w-git.cmd", GIT_WORKLOAD),
        &GIT_STEPS,
    ));

    fs::write(root.join("w.js"), NODE_JS).expect("w.js");
    fs::create_dir_all(root.join("lib")).expect("lib");
    fs::write(
        root.join("lib").join("package.json"),
        r#"{"name":"lib1791","version":"1.0.0","main":"i.js"}"#,
    )
    .expect("lib package.json");
    fs::write(root.join("lib").join("i.js"), "module.exports = 7;\n").expect("lib i.js");
    fs::create_dir_all(root.join("app")).expect("app");
    fs::write(
        root.join("app").join("package.json"),
        r#"{"name":"app1791","version":"1.0.0","private":true,"scripts":{"t":"node -e \"console.log(8)\""}}"#,
    )
    .expect("app package.json");
    failed.extend(workload(
        home,
        "node",
        root,
        temp,
        cmd_argv(root, "w-node.cmd", NODE_WORKLOAD),
        &NODE_STEPS,
    ));

    fs::write(root.join("w.py"), PYTHON_PY).expect("w.py");
    failed.extend(workload(
        home,
        "python",
        root,
        temp,
        cmd_argv(root, "w-py.cmd", "@echo off\npython w.py\n"),
        &PYTHON_STEPS,
    ));

    failed.extend(workload(
        home,
        "cargo",
        root,
        temp,
        cmd_argv(root, "w-cargo.cmd", CARGO_WORKLOAD),
        &CARGO_STEPS,
    ));

    // A project kept in a shared folder, its ACL inherited.
    let public_documents = env_dir("PUBLIC", "C:\\Users\\Public").join("Documents");
    let project_dir = tempfile::Builder::new()
        .prefix("sec-win-1791-")
        .tempdir_in(&public_documents)
        .expect("a project in Public\\Documents");
    let project = project_dir.path();
    prepare_cmd_project(project);
    failed.extend(workload(
        home,
        "public-project-cmd",
        project,
        temp,
        cmd_argv(project, "w-cmd.cmd", CMD_WORKLOAD),
        &CMD_STEPS,
    ));
    failed.extend(workload(
        home,
        "public-project-git",
        project,
        temp,
        cmd_argv(project, "w-git.cmd", GIT_WORKLOAD),
        &GIT_STEPS,
    ));

    let total = CMD_STEPS.len() * 2
        + PS_STEPS.len() * 2
        + GIT_STEPS.len() * 2
        + NODE_STEPS.len()
        + PYTHON_STEPS.len()
        + CARGO_STEPS.len();
    assert!(
        failed.is_empty(),
        "#1791/D400: everyday steps failed: {failed:?}\n{}",
        sandbox_log(home)
    );
    println!(
        "sec-win-1791 {}: all {total} everyday steps worked",
        session()
    );
}

/// The removal (for uninstalling) takes out exactly what the setup added.
/// Elevated only; it removes this home's entries everywhere, so CI runs it
/// after the tests that rely on them.
#[test]
fn sec_win_1791_removal_takes_out_recorded_denies() {
    if !crate::setup::is_elevated().expect("elevation") {
        assert!(
            std::env::var_os(REQUIRE_ENV).is_none(),
            "sec-win-1791: the removal test needs an elevated run"
        );
        return;
    }
    let shared = tempfile::Builder::new()
        .prefix("CorbanuSecWin1791Removal-")
        .tempdir_in(env_dir("ProgramData", "C:\\ProgramData"))
        .expect("a shared folder");
    icacls(shared.path(), &["/grant", "*S-1-5-32-545:(OI)(CI)F"]);
    let before = icacls(shared.path(), &[]);
    let Some(codex_home) = elevated_home("sec-win-1791-removal") else {
        return;
    };
    let _guard = legacy_process_test_guard();
    let root = user_owned_test_root();
    let temp = user_owned_test_root();
    let (stdout, exit_code) = current_thread_runtime().block_on(run_elevated(
        root.path(),
        codex_home.path(),
        cmd_argv(root.path(), "noop.cmd", "@echo off\necho ran\n"),
        &command_env(temp.path()),
        /*tty*/ false,
    ));
    assert_eq!(exit_code, 0, "stdout={stdout:?}");
    let record_dir = machine_shared_folder_record_dir().expect("record folder");
    let record = load_shared_folder_deny_record(&record_dir).expect("record");
    let entry = record
        .entries
        .iter()
        .find(|entry| entry.path == shared.path())
        .unwrap_or_else(|| panic!("{} not recorded: {record:?}", shared.path().display()))
        .clone();
    assert_eq!(entry.kind, SharedFolderDenyKind::DeleteChild);
    let group = LocalSid::from_string(&entry.sid).expect("group SID");
    // SAFETY: a valid SID and an existing folder.
    let present = unsafe { has_shared_folder_deny(shared.path(), group.as_ptr(), entry.kind) };
    assert!(present.expect("read DACL"));
    assert_ne!(icacls(shared.path(), &[]), before);

    crate::setup::run_elevated_shared_folder_deny_removal(codex_home.path())
        .unwrap_or_else(|err| panic!("removal: {err:#}\n{}", sandbox_log(codex_home.path())));
    // Only this home relied on the new folder's deny: it is gone, exactly.
    assert_eq!(icacls(shared.path(), &[]), before);
    let left = load_shared_folder_deny_record(&record_dir).expect("record");
    assert!(
        !left.entries.iter().any(|entry| entry.path == shared.path()),
        "{left:?}"
    );
    // What other homes' setups rely on stays, in place.
    for entry in &left.entries {
        let group = LocalSid::from_string(&entry.sid).expect("group SID");
        // SAFETY: as above.
        let present = unsafe { has_shared_folder_deny(&entry.path, group.as_ptr(), entry.kind) };
        assert!(
            !entry.path.exists() || present.unwrap_or(false),
            "{:?} on {} went with another home's removal",
            entry.kind,
            entry.path.display()
        );
    }
    println!(
        "sec-win-1791: the removal took out the deny only its home relied on and kept {} others",
        left.entries.len()
    );
}
