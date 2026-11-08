//! PROBE ONLY (win-377, never merged): D377 (#304). Measures, for elevated
//! workspace-write commands,
//! 1. what a command can open in the user's `%TEMP%` when `%TEMP%` itself is
//!    the temp root (main today: setup grants it Modify), and
//! 2. which common tools still work when `TEMP`/`TMP` name a per-session
//!    subfolder of `%TEMP%` and only that subfolder is granted.
//!
//! Reports only (`P377|` lines); asserts nothing.

use super::*;
use std::os::windows::fs::OpenOptionsExt;

const SCAN_ENTRY: &str =
    "unified_exec::tests::command_identity::elevated::probe_377::probe_377_scan_entry";
const SCAN_ENV: &str = "CODEX_P377_SCAN";
const SCAN_CAP: usize = 50_000;
const FILE_READ_DATA: u32 = 0x1;
const FILE_WRITE_DATA: u32 = 0x2;
const DELETE: u32 = 0x1_0000;
const FILE_FLAG_OPEN_REPARSE_POINT: u32 = 0x0020_0000;

fn session() -> &'static str {
    if crate::setup::is_elevated().expect("elevation") {
        "admin"
    } else {
        "normal"
    }
}

fn open_class(path: &Path, access: u32) -> &'static str {
    match OpenOptions::new()
        .access_mode(access)
        .share_mode(7)
        .custom_flags(FILE_FLAG_OPEN_REPARSE_POINT)
        .open(path)
    {
        Ok(_) => "ok",
        Err(err) => match err.raw_os_error() {
            Some(5) => "denied",
            Some(32) => "sharing",
            _ => "other",
        },
    }
}

#[derive(Default)]
struct Counts {
    files: usize,
    read: usize,
    write: usize,
    delete: usize,
    write_denied: usize,
    write_sharing: usize,
    write_other: usize,
}

impl Counts {
    fn add(&mut self, path: &Path) {
        self.files += 1;
        self.read += usize::from(open_class(path, FILE_READ_DATA) == "ok");
        match open_class(path, FILE_WRITE_DATA) {
            "ok" => self.write += 1,
            "denied" => self.write_denied += 1,
            "sharing" => self.write_sharing += 1,
            _ => self.write_other += 1,
        }
        self.delete += usize::from(open_class(path, DELETE) == "ok");
    }

    fn line(&self) -> String {
        format!(
            "files={} read={} write={} delete={} write_denied={} write_sharing={} write_other={}",
            self.files,
            self.read,
            self.write,
            self.delete,
            self.write_denied,
            self.write_sharing,
            self.write_other
        )
    }
}

fn is_ours(name: &str) -> bool {
    name.starts_with("codex-windows-sandbox-")
        || name.starts_with("codex-p377-")
        || name.starts_with("p377-")
        || name.starts_with(".tmp")
}

/// Walks `dir` (not through links), splitting files under our own top-level
/// entries from everyone else's. Directories: can a file be made in them.
fn scan_dir(dir: &Path) -> String {
    let (mut ours, mut other) = (Counts::default(), Counts::default());
    let (mut dirs, mut dirs_create) = (0usize, 0usize);
    let mut unlistable = 0usize;
    let mut stack: Vec<(PathBuf, bool)> = Vec::new();
    match fs::read_dir(dir) {
        Ok(entries) => {
            for entry in entries.flatten() {
                let mine = is_ours(&entry.file_name().to_string_lossy());
                stack.push((entry.path(), mine));
            }
        }
        Err(err) => return format!("unlistable={err}"),
    }
    let mut seen = 0usize;
    while let Some((path, mine)) = stack.pop() {
        if seen >= SCAN_CAP {
            break;
        }
        seen += 1;
        let Ok(meta) = fs::symlink_metadata(&path) else {
            continue;
        };
        if meta.file_type().is_symlink() {
            continue;
        }
        if meta.is_dir() {
            if !mine {
                dirs += 1;
                let probe = path.join(format!("p377-create-{}", std::process::id()));
                if OpenOptions::new()
                    .write(true)
                    .create_new(true)
                    .open(&probe)
                    .is_ok()
                {
                    dirs_create += 1;
                    let _ = fs::remove_file(&probe);
                }
            }
            match fs::read_dir(&path) {
                Ok(entries) => stack.extend(entries.flatten().map(|entry| (entry.path(), mine))),
                Err(_) => unlistable += 1,
            }
        } else if mine {
            ours.add(&path);
        } else {
            other.add(&path);
        }
    }
    format!(
        "other[{}] other_dirs={dirs} other_dirs_create={dirs_create} unlistable_dirs={unlistable} ours[{}] capped={}",
        other.line(),
        ours.line(),
        seen >= SCAN_CAP
    )
}

fn create_in(dir: &Path) -> &'static str {
    let probe = dir.join(format!("p377-create-{}", std::process::id()));
    match OpenOptions::new().write(true).create_new(true).open(&probe) {
        Ok(_) => {
            let _ = fs::remove_file(&probe);
            "ok"
        }
        Err(err) if err.raw_os_error() == Some(5) => "denied",
        Err(_) => "other",
    }
}

/// The sandboxed scanner. `CODEX_P377_SCAN`: `label|kind|path` entries
/// separated by `;`, kind one of `tree`, `file`, `dir`.
#[test]
fn probe_377_scan_entry() {
    let Some(spec) = std::env::var_os(SCAN_ENV) else {
        return;
    };
    for entry in spec.to_string_lossy().split(';') {
        let mut parts = entry.splitn(3, '|');
        let (Some(label), Some(kind), Some(path)) = (parts.next(), parts.next(), parts.next())
        else {
            continue;
        };
        let path = Path::new(path);
        let report = match kind {
            "tree" => scan_dir(path),
            "file" => format!(
                "exists={} read={} write={} delete={}",
                path.exists(),
                open_class(path, FILE_READ_DATA),
                open_class(path, FILE_WRITE_DATA),
                open_class(path, DELETE)
            ),
            _ => format!("create={}", create_in(path)),
        };
        println!("\nP377SCAN|{label}|{report}");
    }
}

async fn run(
    root: &Path,
    codex_home: &Path,
    argv: Vec<String>,
    env: &[(&str, String)],
) -> (String, i32) {
    let mut env_map: HashMap<String, String> = std::env::vars().collect();
    for (key, value) in env {
        env_map.insert((*key).to_string(), value.clone());
    }
    let spawned = spawn_windows_sandbox_session_elevated_for_permission_profile(
        &PermissionProfile::workspace_write(),
        workspace_roots_for(root).as_slice(),
        codex_home,
        argv,
        root,
        env_map,
        /*proxy_enforced*/ false,
        /*network_proxy_restricting_sid*/ None,
        Some(300_000),
        /*read_roots_override*/ None,
        /*read_roots_include_platform_defaults*/ true,
        /*write_roots_override*/ None,
        &crate::DenyReadTargets::default(),
        &[],
        /*tty*/ false,
        /*stdin_open*/ false,
        /*use_private_desktop*/ true,
    )
    .await;
    match spawned {
        Ok(spawned) => {
            let (stdout, code) =
                collect_stdout_and_exit(spawned, codex_home, Duration::from_secs(/*secs*/ 330))
                    .await;
            (String::from_utf8_lossy(&stdout).into_owned(), code)
        }
        Err(err) => (format!("start error: {err:#}"), -1),
    }
}

fn cmd_argv(script: &Path) -> Vec<String> {
    vec![
        system32_exe("cmd.exe").display().to_string(),
        "/d".to_string(),
        "/c".to_string(),
        script.display().to_string(),
    ]
}

fn tail(text: &str) -> String {
    let flat = text.replace(['\r', '\n'], " ");
    let chars: Vec<char> = flat.chars().collect();
    chars[chars.len().saturating_sub(400)..].iter().collect()
}

const MKWHEEL: &str = r#"import zipfile, base64, hashlib
n, v = 'p377pkg', '0.1'
files = {n + '/__init__.py': b'X = 1\n',
         n + '-' + v + '.dist-info/METADATA': b'Metadata-Version: 2.1\nName: p377pkg\nVersion: 0.1\n',
         n + '-' + v + '.dist-info/WHEEL': b'Wheel-Version: 1.0\nGenerator: p377\nRoot-Is-Purelib: true\nTag: py3-none-any\n'}
rec = []
with zipfile.ZipFile(n + '-' + v + '-py3-none-any.whl', 'w') as z:
    for k, b in files.items():
        z.writestr(k, b)
        h = base64.urlsafe_b64encode(hashlib.sha256(b).digest()).rstrip(b'=').decode()
        rec.append(k + ',sha256=' + h + ',' + str(len(b)))
    rec.append(n + '-' + v + '.dist-info/RECORD,,')
    z.writestr(n + '-' + v + '.dist-info/RECORD', '\n'.join(rec) + '\n')
"#;

const PS1: &str = r#"$ErrorActionPreference = 'Stop'
[IO.Path]::GetTempPath()
$f = New-TemporaryFile; Remove-Item $f
$t = [IO.Path]::GetTempFileName(); Remove-Item $t
$d = Join-Path ([IO.Path]::GetTempPath()) ([Guid]::NewGuid()); New-Item -ItemType Directory $d | Out-Null; Remove-Item $d
Get-FileHash -Algorithm SHA256 $PSCommandPath | Out-Null
'P377OK'
"#;

fn find_msbuild() -> Option<PathBuf> {
    let vswhere = PathBuf::from(std::env::var_os("ProgramFiles(x86)")?)
        .join("Microsoft Visual Studio")
        .join("Installer")
        .join("vswhere.exe");
    let out = std::process::Command::new(vswhere)
        .args(["-latest", "-find", "MSBuild\\**\\Bin\\MSBuild.exe"])
        .output()
        .ok()?;
    String::from_utf8_lossy(&out.stdout)
        .lines()
        .map(str::trim)
        .find(|line| !line.is_empty())
        .map(PathBuf::from)
}

/// Writes each workload into `dir`; returns (name, script).
fn workloads(dir: &Path) -> Vec<(&'static str, PathBuf)> {
    let w = |name: &str, body: &str| {
        let script = dir.join(format!("w-{name}.cmd"));
        fs::write(
            &script,
            format!(
                "@echo off\r\ncd /d \"{}\"\r\necho TEMP=%TEMP% TMP=%TMP%\r\n{body}\r\n",
                dir.display()
            ),
        )
        .expect("write workload");
        script
    };
    fs::write(dir.join("mkwheel.py"), MKWHEEL).expect("mkwheel");
    fs::write(dir.join("t.ps1"), PS1).expect("ps1");
    let crate_dir = dir.join("c377");
    fs::create_dir_all(crate_dir.join("src")).expect("crate dir");
    fs::write(
        crate_dir.join("Cargo.toml"),
        "[package]\nname = \"c377\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[workspace]\n",
    )
    .expect("Cargo.toml");
    fs::write(
        crate_dir.join("src").join("main.rs"),
        "fn main() { let d = std::env::temp_dir(); let f = d.join(\"p377-rs.txt\"); std::fs::write(&f, b\"x\").unwrap(); std::fs::remove_file(&f).unwrap(); println!(\"P377OK\"); }\n",
    )
    .expect("main.rs");
    let pkg = dir.join("pkga");
    fs::create_dir_all(&pkg).expect("pkga");
    fs::write(
        pkg.join("package.json"),
        "{\"name\":\"pkga\",\"version\":\"1.0.0\",\"main\":\"index.js\"}",
    )
    .expect("pkga json");
    fs::write(pkg.join("index.js"), "module.exports = 377;\n").expect("pkga js");
    for app in ["app", "app2"] {
        let app = dir.join(app);
        fs::create_dir_all(&app).expect("app");
        fs::write(
            app.join("package.json"),
            "{\"name\":\"app\",\"version\":\"1.0.0\",\"dependencies\":{\"pkga\":\"file:../pkga\"}}",
        )
        .expect("app json");
    }
    fs::write(
        dir.join("t.proj"),
        "<Project DefaultTargets=\"T\"><Target Name=\"T\"><Exec Command=\"echo P377-exec\" /><WriteLinesToFile File=\"$(TEMP)\\p377-msb.txt\" Lines=\"x\" Overwrite=\"true\" /><Delete Files=\"$(TEMP)\\p377-msb.txt\" /></Target></Project>",
    )
    .expect("proj");
    let mut list = vec![
        (
            "cmd-temp",
            w(
                "cmd-temp",
                "echo x> \"%TEMP%\\p377-cmd.txt\" || exit /b 1\r\ndel \"%TEMP%\\p377-cmd.txt\" || exit /b 1\r\necho P377OK",
            ),
        ),
        (
            "python-tempfile",
            w(
                "python-tempfile",
                "python -c \"import tempfile,os;print(tempfile.gettempdir());f=tempfile.NamedTemporaryFile(delete=False);f.write(b'x');f.close();os.unlink(f.name);d=tempfile.mkdtemp();os.rmdir(d);print('P377OK')\" 2>&1",
            ),
        ),
        (
            "python-venv-pip",
            w(
                "python-venv-pip",
                "python -m venv v 2>&1 || exit /b 1\r\npython mkwheel.py 2>&1 || exit /b 1\r\nv\\Scripts\\python.exe -m pip install --no-index --no-cache-dir --disable-pip-version-check p377pkg-0.1-py3-none-any.whl 2>&1 || exit /b 1\r\nv\\Scripts\\python.exe -c \"import p377pkg;print('P377OK')\" 2>&1",
            ),
        ),
        (
            "python-pip-default-cache",
            w(
                "python-pip-default-cache",
                "python mkwheel.py 2>&1 || exit /b 1\r\npython -m pip install --no-index --disable-pip-version-check --target pt p377pkg-0.1-py3-none-any.whl 2>&1 || exit /b 1\r\nif exist pt\\p377pkg\\__init__.py echo P377OK",
            ),
        ),
        (
            "cargo-build",
            w(
                "cargo-build",
                "set CARGO_TARGET_DIR=\r\nset RUSTC_WRAPPER=\r\nset CARGO_BUILD_RUSTC_WRAPPER=\r\ncd c377\r\ncargo build --offline 2>&1 || exit /b 1\r\ntarget\\debug\\c377.exe 2>&1",
            ),
        ),
        (
            "node-tmpdir",
            w(
                "node-tmpdir",
                "node -e \"const os=require('os'),fs=require('fs'),p=require('path');const d=fs.mkdtempSync(p.join(os.tmpdir(),'n'));fs.writeFileSync(p.join(d,'f'),'x');fs.rmSync(d,{recursive:true});console.log(os.tmpdir(),'P377OK')\" 2>&1",
            ),
        ),
        (
            "npm-install-local",
            w(
                "npm-install-local",
                "cd app\r\ncall npm install --offline --no-audit --no-fund --cache ..\\npmcache 2>&1 || exit /b 1\r\nnode -e \"console.log(require('pkga')===377?'P377OK':'bad')\" 2>&1",
            ),
        ),
        (
            "npm-install-default-cache",
            w(
                "npm-install-default-cache",
                "cd app2\r\ncall npm install --offline --no-audit --no-fund 2>&1 || exit /b 1\r\nnode -e \"console.log(require('pkga')===377?'P377OK':'bad')\" 2>&1",
            ),
        ),
        (
            "npm-pack",
            w(
                "npm-pack",
                "cd pkga\r\ncall npm pack --cache ..\\npmcache 2>&1 || exit /b 1\r\nif exist pkga-1.0.0.tgz echo P377OK",
            ),
        ),
        (
            "powershell",
            w(
                "powershell",
                &format!(
                    "\"{}\" -NoProfile -ExecutionPolicy Bypass -File t.ps1 2>&1",
                    system32_exe("WindowsPowerShell\\v1.0\\powershell.exe").display()
                ),
            ),
        ),
        (
            "pwsh",
            w(
                "pwsh",
                "where pwsh >nul 2>&1 || (echo no-pwsh & exit /b 9)\r\npwsh -NoProfile -ExecutionPolicy Bypass -File t.ps1 2>&1",
            ),
        ),
        (
            "git",
            w(
                "git",
                "git init -q g 2>&1 || exit /b 1\r\ncd g\r\necho a> a.txt\r\ngit add a.txt 2>&1 || exit /b 1\r\ngit -c user.name=p -c user.email=p@p commit -q -m m 2>&1 || exit /b 1\r\ngit log --oneline 2>&1 && echo P377OK",
            ),
        ),
    ];
    if let Some(bash) = command_identity::git_for_windows("bin\\bash.exe") {
        fs::write(
            dir.join("t.sh"),
            "t=$(mktemp) && d=$(mktemp -d) || exit 1\ncat <<EOF > \"$t\"\nx\nEOF\nrm -rf \"$t\" \"$d\" && echo P377OK\n",
        )
        .expect("t.sh");
        list.push((
            "git-bash-heredoc-mktemp",
            w("git-bash", &format!("\"{}\" t.sh 2>&1", bash.display())),
        ));
    }
    match find_msbuild() {
        Some(msbuild) => list.push((
            "msbuild-exec",
            w(
                "msbuild",
                &format!(
                    "\"{}\" -nologo -nodeReuse:false t.proj 2>&1 || exit /b 1\r\necho P377OK",
                    msbuild.display()
                ),
            ),
        )),
        None => println!("\nP377|{}|msbuild not found", session()),
    }
    list.push((
        "dotnet-msbuild",
        w(
            "dotnet-msbuild",
            "set DOTNET_CLI_TELEMETRY_OPTOUT=1\r\nset DOTNET_NOLOGO=1\r\nset DOTNET_SKIP_FIRST_TIME_EXPERIENCE=1\r\nwhere dotnet >nul 2>&1 || (echo no-dotnet & exit /b 9)\r\ndotnet msbuild -nologo -nodeReuse:false t.proj 2>&1 || exit /b 1\r\necho P377OK",
        ),
    ));
    list
}

fn unique_id() -> String {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    format!("{:012x}", (nanos as u64) & 0xffff_ffff_ffff)
}

fn temp_env(dir: &Path) -> Vec<(&'static str, String)> {
    let value = dir.display().to_string();
    vec![("TEMP", value.clone()), ("TMP", value)]
}

#[test]
fn probe_377_temp_measure() {
    let _guard = legacy_process_test_guard();
    let (Some(home1), Some(home2)) = (elevated_home("p377a"), elevated_home("p377b")) else {
        println!("\nP377|{}|skipped: no elevated setup", session());
        return;
    };
    let runtime = current_thread_runtime();
    let real_temp =
        dunce::canonicalize(std::env::var_os("TEMP").expect("TEMP")).expect("TEMP real");
    println!(
        "\nP377|{}|temp={} tmp={:?}",
        session(),
        real_temp.display(),
        std::env::var_os("TMP")
    );
    let id = unique_id();
    let user_pre = real_temp.join(format!("p377-user-pre-{id}.txt"));
    fs::write(&user_pre, "user").expect("user pre");
    // "subfolder" first: on a fresh runner `%TEMP%` is not granted yet.
    for variant in ["subfolder", "whole"] {
        let root1 = user_owned_test_root();
        let root2 = user_owned_test_root();
        let (t1, t2) = if variant == "subfolder" {
            let t1 = real_temp.join(format!("codex-p377-{id}-s1"));
            let t2 = real_temp.join(format!("codex-p377-{id}-s2"));
            fs::create_dir_all(&t1).expect("t1");
            fs::create_dir_all(&t2).expect("t2");
            (t1, t2)
        } else {
            (real_temp.clone(), real_temp.clone())
        };
        let s2_file = t2.join(format!("p377-s2-{variant}-{id}.txt"));
        let s2_script = root2.path().join("s2.cmd");
        fs::write(
            &s2_script,
            format!(
                "@echo off\r\necho s2> \"{}\" && echo P377S2OK\r\n",
                s2_file.display()
            ),
        )
        .expect("s2 script");
        let (out, code) = runtime.block_on(run(
            root2.path(),
            home2.path(),
            cmd_argv(&s2_script),
            &temp_env(&t2),
        ));
        println!(
            "\nP377|{}|{variant}|session2-make exit={code} ok={} out={}",
            session(),
            out.contains("P377S2OK"),
            tail(&out)
        );
        // Another program's file, made after the grants.
        let user_post = real_temp.join(format!("p377-user-post-{variant}-{id}.txt"));
        fs::write(&user_post, "user").expect("user post");
        let spec = [
            format!("temp-tree|tree|{}", real_temp.display()),
            format!("user-pre|file|{}", user_pre.display()),
            format!("user-post|file|{}", user_post.display()),
            format!("session2-file|file|{}", s2_file.display()),
            format!("session2-dir|dir|{}", t2.display()),
            format!("temp-root|dir|{}", real_temp.display()),
            format!("own-temp|dir|{}", t1.display()),
        ]
        .join(";");
        let mut env = temp_env(&t1);
        env.push((SCAN_ENV, spec));
        let (out, code) = runtime.block_on(run(
            root1.path(),
            home1.path(),
            probe_argv(SCAN_ENTRY),
            &env,
        ));
        let lines: Vec<&str> = out
            .lines()
            .filter(|line| line.starts_with("P377SCAN|"))
            .collect();
        if lines.is_empty() {
            println!(
                "\nP377|{}|{variant}|scan exit={code} out={}",
                session(),
                tail(&out)
            );
        }
        for line in lines {
            println!("\nP377|{}|{variant}|{line}", session());
        }
        let work = root1.path().join("work");
        fs::create_dir_all(&work).expect("work");
        let mut passed = 0;
        let list = workloads(&work);
        let total = list.len();
        for (name, script) in list {
            let started = Instant::now();
            let (out, code) = runtime.block_on(run(
                root1.path(),
                home1.path(),
                cmd_argv(&script),
                &temp_env(&t1),
            ));
            let ok = code == 0 && out.contains("P377OK");
            passed += usize::from(ok);
            println!(
                "\nP377W|{}|{variant}|{name}|ok={ok} exit={code} secs={} out={}",
                session(),
                started.elapsed().as_secs(),
                tail(&out)
            );
        }
        println!(
            "\nP377|{}|{variant}|workloads passed={passed}/{total}",
            session()
        );
        if variant == "subfolder" {
            let left = fs::read_dir(&t1).map(|dir| dir.count()).unwrap_or(0);
            println!("\nP377|{}|{variant}|left-in-own-temp={left}", session());
        }
    }
    record_setup(home1.path());
}

// ---------------------------------------------------------------------------
// PROBE ONLY (win-377b, #1871, never merged): what an elevated pre-upgrade
// session leaves in the user's `%TEMP%` that the sandbox's accounts own, what
// a sandboxed command can still do to it once the old `%TEMP%` grant is gone
// (#1857's upgrade step), and what taking it for the user changes.
// Reports only (`P377B|` lines).

const SCAN_B_ENTRY: &str =
    "unified_exec::tests::command_identity::elevated::probe_377::probe_377b_scan_entry";
const LIST_ENV: &str = "CODEX_P377B_LIST";
const READ_CONTROL_B: u32 = 0x2_0000;
const WRITE_DAC_B: u32 = 0x4_0000;
const WRITE_OWNER_B: u32 = 0x8_0000;
const READ_ATTRIBUTES_B: u32 = 0x80;
const BACKUP_SEMANTICS_B: u32 = 0x0200_0000;

fn open_b(path: &Path, access: u32) -> std::io::Result<fs::File> {
    OpenOptions::new()
        .access_mode(access)
        .share_mode(7)
        .custom_flags(FILE_FLAG_OPEN_REPARSE_POINT | BACKUP_SEMANTICS_B)
        .open(path)
}

/// The sandboxed side: for each listed path, can it be opened to change its
/// DACL, to write (or add a file, for a folder), to delete.
#[test]
fn probe_377b_scan_entry() {
    let Some(list) = std::env::var_os(LIST_ENV) else {
        return;
    };
    let text = fs::read_to_string(list).unwrap_or_default();
    let (mut total, mut write_dac, mut write, mut delete, mut regrant) = (0, 0, 0, 0, 0);
    for line in text.lines().filter(|line| !line.is_empty()) {
        let path = Path::new(line);
        total += 1;
        write_dac += usize::from(open_b(path, WRITE_DAC_B).is_ok());
        write += usize::from(open_b(path, FILE_WRITE_DATA).is_ok());
        delete += usize::from(open_b(path, DELETE).is_ok());
        // What an owner does with WRITE_DAC: grant itself access again
        // (only on the first five, to keep the state comparable).
        if regrant < 5 && open_b(path, WRITE_DAC_B).is_ok() {
            let me = crate::deny_read_sessions::current_user_sid().unwrap_or_default();
            let out = std::process::Command::new(system32_exe("icacls.exe"))
                .arg(path)
                .arg("/grant")
                .arg(format!("*{me}:(M)"))
                .output();
            if out.is_ok_and(|out| out.status.success()) && open_b(path, FILE_WRITE_DATA).is_ok() {
                regrant += 1;
            }
        }
    }
    println!(
        "\nP377BSCAN|total={total} write_dac={write_dac} write={write} delete={delete} regranted_then_writable={regrant}"
    );
}

fn bucket(name: &str) -> String {
    let mut out = String::new();
    for ch in name.chars().take(28) {
        out.push(if ch.is_ascii_digit() { '#' } else { ch });
    }
    out
}

#[derive(Default)]
struct Inventory {
    entries: usize,
    owners: std::collections::BTreeMap<&'static str, usize>,
    sandbox: Vec<PathBuf>,
    sandbox_files: usize,
    sandbox_dirs: usize,
    sandbox_links: usize,
    sandbox_takeable: usize,
    buckets: std::collections::BTreeMap<String, usize>,
}

fn inventory(base: &Path, accounts: &[String]) -> Inventory {
    let user = crate::deny_read_sessions::current_user_sid().unwrap_or_default();
    let mut inv = Inventory::default();
    let mut stack = vec![base.to_path_buf()];
    while let Some(dir) = stack.pop() {
        let Ok(entries) = fs::read_dir(&dir) else {
            continue;
        };
        for entry in entries.flatten() {
            if inv.entries >= 200_000 {
                return inv;
            }
            inv.entries += 1;
            let path = entry.path();
            let Ok(look) = open_b(&path, READ_CONTROL_B | READ_ATTRIBUTES_B) else {
                *inv.owners.entry("unreadable").or_default() += 1;
                continue;
            };
            let attrs = look.metadata().map(|m| {
                use std::os::windows::fs::MetadataExt;
                m.file_attributes()
            });
            let attrs = attrs.unwrap_or(0);
            let is_link = attrs & 0x400 != 0;
            let is_dir = attrs & 0x10 != 0;
            let owner = crate::deny_read_sessions::owner_sid(&look).unwrap_or_default();
            let class = if accounts.iter().any(|a| a.eq_ignore_ascii_case(&owner)) {
                "sandbox"
            } else if owner == user {
                "user"
            } else if owner == "S-1-5-32-544" {
                "admins"
            } else if owner == "S-1-5-18" {
                "system"
            } else {
                "other"
            };
            *inv.owners.entry(class).or_default() += 1;
            if class == "sandbox" {
                inv.sandbox.push(path.clone());
                if is_link {
                    inv.sandbox_links += 1;
                } else if is_dir {
                    inv.sandbox_dirs += 1;
                } else {
                    inv.sandbox_files += 1;
                }
                inv.sandbox_takeable += usize::from(
                    open_b(&path, READ_CONTROL_B | WRITE_DAC_B | WRITE_OWNER_B).is_ok(),
                );
                let top = path
                    .strip_prefix(base)
                    .ok()
                    .and_then(|rel| rel.components().next())
                    .map(|c| c.as_os_str().to_string_lossy().into_owned())
                    .unwrap_or_default();
                *inv.buckets.entry(bucket(&top)).or_default() += 1;
            }
            if is_dir && !is_link {
                stack.push(path);
            }
        }
    }
    inv
}

fn print_inventory(label: &str, inv: &Inventory) {
    println!(
        "\nP377B|{}|{label}|entries={} owners={:?} sandbox_files={} sandbox_dirs={} sandbox_links={} sandbox_takeable={}/{}",
        session(),
        inv.entries,
        inv.owners,
        inv.sandbox_files,
        inv.sandbox_dirs,
        inv.sandbox_links,
        inv.sandbox_takeable,
        inv.sandbox.len()
    );
    println!(
        "\nP377B|{}|{label}|top-level buckets={:?}",
        session(),
        inv.buckets
    );
}

#[test]
fn probe_377b_leftovers() {
    let _guard = legacy_process_test_guard();
    let Some(home) = elevated_home("p377b") else {
        println!("\nP377B|{}|skipped: no elevated setup", session());
        return;
    };
    let runtime = current_thread_runtime();
    let real_temp =
        dunce::canonicalize(std::env::var_os("TEMP").expect("TEMP")).expect("TEMP real");
    let id = unique_id();
    println!("\nP377B|{}|temp={}", session(), real_temp.display());
    let accounts = crate::session_temp_leftovers::sandbox_account_sids();
    println!("\nP377B|{}|accounts(before)={accounts:?}", session());
    let user_dir = real_temp.join(format!("p377b-user-{id}"));
    fs::create_dir_all(&user_dir).expect("user dir");
    fs::write(user_dir.join("user.txt"), "user").expect("user file");
    let root = user_owned_test_root();
    let work = root.path().join("work");
    fs::create_dir_all(&work).expect("work");
    // A pre-upgrade session: the usual tools with TEMP = %TEMP%.
    let list = workloads(&work);
    let total = list.len();
    let mut passed = 0;
    for (name, script) in list {
        let (out, code) = runtime.block_on(run(
            root.path(),
            home.path(),
            cmd_argv(&script),
            &temp_env(&real_temp),
        ));
        let ok = code == 0 && out.contains("P377OK");
        passed += usize::from(ok);
        println!("\nP377BW|{}|{name}|ok={ok} exit={code}", session());
    }
    println!("\nP377B|{}|workloads passed={passed}/{total}", session());
    // Things a session plausibly leaves behind (crashes, tools that keep
    // their temp files, files written on purpose).
    let bash = command_identity::git_for_windows("bin\\bash.exe")
        .map(|bash| {
            format!(
                "\"{}\" -c \"d=$(mktemp -d) && echo x > $d/f && t=$(mktemp) && echo $d $t\" 2>&1",
                bash.display()
            )
        })
        .unwrap_or_default();
    let left = work.join("w-left.cmd");
    fs::write(
        &left,
        format!(
            "@echo off\r\n\
             echo x> \"%TEMP%\\p377b-cmd-left-{id}.txt\"\r\n\
             mkdir \"%TEMP%\\p377b-cmd-dir-{id}\" && echo x> \"%TEMP%\\p377b-cmd-dir-{id}\\f.txt\"\r\n\
             echo x> \"{ud}\\sandbox-made.txt\"\r\n\
             mkdir \"{ud}\\sandbox-sub\" && echo x> \"{ud}\\sandbox-sub\\f.txt\"\r\n\
             python -c \"import tempfile,os,sys;print(sys.version);f=tempfile.NamedTemporaryFile(delete=False,prefix='p377b-py-');f.write(b'x');f.close();d=tempfile.mkdtemp(prefix='p377b-pyd-');open(os.path.join(d,'f'),'w').write('x');os.mkdir(os.path.join(tempfile.gettempdir(),'p377b-py700-{id}'),0o700)\" 2>&1\r\n\
             node -e \"const os=require('os'),fs=require('fs'),p=require('path');const d=fs.mkdtempSync(p.join(os.tmpdir(),'p377b-node-'));fs.writeFileSync(p.join(d,'f'),'x')\" 2>&1\r\n\
             \"{ps}\" -NoProfile -Command \"$f=New-TemporaryFile; Set-Content $f x; $f.FullName\" 2>&1\r\n\
             {bash}\r\n\
             echo P377OK\r\n",
            ud = user_dir.display(),
            ps = system32_exe("WindowsPowerShell\\v1.0\\powershell.exe").display(),
        ),
    )
    .expect("left script");
    let (out, code) = runtime.block_on(run(
        root.path(),
        home.path(),
        cmd_argv(&left),
        &temp_env(&real_temp),
    ));
    println!(
        "\nP377B|{}|leave-behind exit={code} out={}",
        session(),
        tail(&out)
    );
    let accounts = crate::session_temp_leftovers::sandbox_account_sids();
    println!("\nP377B|{}|accounts={accounts:?}", session());
    let accounts = accounts.unwrap_or_default();
    let before = inventory(&real_temp, &accounts);
    print_inventory("after-session", &before);
    let list_file = root.path().join("p377b-list.txt");
    fs::write(
        &list_file,
        before
            .sandbox
            .iter()
            .map(|p| p.display().to_string())
            .collect::<Vec<_>>()
            .join("\n"),
    )
    .expect("list");
    let scan = |label: &str| {
        let mut env = temp_env(&work);
        env.push((LIST_ENV, list_file.display().to_string()));
        let (out, code) = runtime.block_on(run(
            root.path(),
            home.path(),
            probe_argv(SCAN_B_ENTRY),
            &env,
        ));
        let line = out
            .lines()
            .find(|line| line.starts_with("P377BSCAN|"))
            .map(str::to_string)
            .unwrap_or_else(|| format!("no scan line exit={code} {}", tail(&out)));
        println!("\nP377B|{}|{label}|{line}", session());
    };
    scan("sandbox-with-old-grant");
    // #1857's upgrade step: the group's and the home's grants on %TEMP% go.
    let caps = crate::cap::load_or_create_cap_sids(home.path()).expect("caps");
    let mut sids: Vec<String> = caps
        .writable_root_by_path
        .values()
        .chain(caps.workspace_by_cwd.values())
        .cloned()
        .collect();
    if let Ok(group) = crate::winutil::resolve_sid(crate::token::SANDBOX_USERS_GROUP)
        && let Ok(group) = crate::winutil::string_from_sid_bytes(&group)
    {
        sids.push(group);
    }
    let psids: Vec<crate::token::LocalSid> = sids
        .iter()
        .filter_map(|sid| crate::token::LocalSid::from_string(sid).ok())
        .collect();
    let ptrs: Vec<_> = psids.iter().map(|sid| sid.as_ptr()).collect();
    // SAFETY: valid SIDs; the folder exists.
    let removed = unsafe { crate::acl::remove_explicit_allow_aces(&real_temp, &ptrs) };
    println!("\nP377B|{}|old grant removed={removed:?}", session());
    scan("sandbox-after-grant-removed");
    let started = Instant::now();
    let taken = crate::session_temp_leftovers::take_sandbox_leftovers(&real_temp, &accounts);
    println!(
        "\nP377B|{}|take secs={:.2} result={taken:?}",
        session(),
        started.elapsed().as_secs_f64()
    );
    scan("sandbox-after-take");
    let after = inventory(&real_temp, &accounts);
    print_inventory("after-take", &after);
    println!(
        "\nP377B|{}|user-dir user.txt intact={}",
        session(),
        fs::read_to_string(user_dir.join("user.txt")).is_ok_and(|text| text == "user")
    );
    record_setup(home.path());
}
