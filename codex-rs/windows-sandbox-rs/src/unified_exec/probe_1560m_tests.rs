//! PROBE ONLY (win-1560m, never merged): compatibility of the #1560 token
//! hardening (D368) and #1721 option (a) (D367) with everyday workloads, on
//! both backends, plus the thread-impersonation borrowing route (#1560).
//! Every test here only reports (`W1560M|...` lines); none asserts a result.

use super::*;
use codex_protocol::permissions::FileSystemAccessMode;
use codex_protocol::permissions::FileSystemPath;
use codex_protocol::permissions::FileSystemSandboxEntry;
use codex_protocol::permissions::FileSystemSandboxPolicy;
use codex_protocol::permissions::FileSystemSpecialPath;
use codex_protocol::permissions::NetworkSandboxPolicy;

const FILES_ENV: &str = "CODEX_W1560M_FILES";

#[derive(Clone, Copy, PartialEq)]
enum Backend {
    Legacy,
    Elevated,
}

impl Backend {
    fn name(self) -> &'static str {
        match self {
            Backend::Legacy => "legacy",
            Backend::Elevated => "elevated",
        }
    }
}

fn session() -> &'static str {
    if crate::setup::is_elevated().expect("elevation") {
        "admin"
    } else {
        "normal"
    }
}

fn report(backend: Backend, case: &str, ok: bool, detail: &str) {
    let flat: Vec<char> = detail.replace(['\r', '\n'], " ").chars().collect();
    // Head and tail: Python and cargo put the cause first, tools the last word.
    let detail: String = if flat.len() > 1400 {
        let head: String = flat[..600].iter().collect();
        let tail: String = flat[flat.len() - 800..].iter().collect();
        format!("{head} [...] {tail}")
    } else {
        flat.into_iter().collect()
    };
    println!(
        "\nW1560M|{}|{}|{}|{case}|{}|{detail}",
        crate::token::probe_variant(),
        backend.name(),
        session(),
        if ok { "works" } else { "BREAKS" }
    );
}

async fn run_in(
    backend: Backend,
    profile: &PermissionProfile,
    root: &Path,
    codex_home: &Path,
    argv: Vec<String>,
    env: &[(&str, String)],
    tty: bool,
) -> (String, i32) {
    let mut env_map: HashMap<String, String> = std::env::vars().collect();
    // CI sets TEMP/TMP with forward slashes (D:/tmp); users have backslashes.
    for key in ["TEMP", "TMP"] {
        if let Some(value) = env_map.get_mut(key) {
            *value = value.replace('/', "\\");
        }
    }
    for (key, value) in env {
        env_map.insert((*key).to_string(), value.clone());
    }
    let spawned = match backend {
        Backend::Legacy => spawn_windows_sandbox_session_legacy(
            profile,
            workspace_roots_for(root).as_slice(),
            codex_home,
            argv,
            root,
            env_map,
            Some(60_000),
            &[],
            &[],
            tty,
            /*stdin_open*/ false,
            /*use_private_desktop*/ true,
        )
        .await
        .map_err(|err| format!("{err:#}")),
        Backend::Elevated => spawn_windows_sandbox_session_elevated_for_permission_profile(
            profile,
            workspace_roots_for(root).as_slice(),
            codex_home,
            argv,
            root,
            env_map,
            /*proxy_enforced*/ false,
            /*network_proxy_restricting_sid*/ None,
            Some(60_000),
            /*read_roots_override*/ None,
            /*read_roots_include_platform_defaults*/ true,
            /*write_roots_override*/ None,
            &crate::DenyReadTargets::default(),
            &[],
            tty,
            tty,
            /*use_private_desktop*/ true,
        )
        .await
        .map_err(|err| format!("{err:#}")),
    };
    match spawned {
        Ok(spawned) => {
            let (stdout, code) =
                collect_stdout_and_exit(spawned, codex_home, Duration::from_secs(/*secs*/ 90))
                    .await;
            (String::from_utf8_lossy(&stdout).into_owned(), code)
        }
        Err(err) => (format!("spawn failed: {err}"), -999),
    }
}

/// `name` on `PATH`, if there.
fn on_path(name: &str) -> Option<PathBuf> {
    std::env::split_paths(&std::env::var_os("PATH")?)
        .map(|dir| dir.join(name))
        .find(|path| path.is_file())
}

fn cmd_c(line: &str) -> Vec<String> {
    vec![
        system32_exe("cmd.exe").display().to_string(),
        "/d".to_string(),
        "/c".to_string(),
        format!("({line}) 2>&1"),
    ]
}

fn workload_cases() -> Vec<(&'static str, Vec<String>, Vec<&'static str>, bool)> {
    let powershell = system32_exe("WindowsPowerShell\\v1.0\\powershell.exe")
        .display()
        .to_string();
    let found = |name: &str| {
        on_path(name)
            .map(|path| path.display().to_string())
            .unwrap_or_else(|| format!("{name}-missing"))
    };
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
        .unwrap_or_default();
    let ps = |exe: &str, script: &str| {
        vec![
            exe.to_string(),
            "-NoProfile".to_string(),
            "-Command".to_string(),
            script.to_string(),
        ]
    };
    let ps_script = "$x = & cmd /c echo w-ps; $x | Select-String w-ps | ForEach-Object { $_.Line }; \
                     Get-ChildItem | Out-Null; $t = [IO.Path]::GetTempFileName(); \
                     Set-Content $t 'x'; Remove-Item $t; 'w-ps-done'";
    let direct_cargo = |dir: &str, extra: &str| {
        cmd_c(&format!(
            "set CARGO_TARGET_DIR=&& {extra}set RUSTC={sysroot}\\bin\\rustc.exe&& cd {dir} \
             && {sysroot}\\bin\\cargo.exe build --offline && target\\debug\\tinyc.exe"
        ))
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
            ps(&powershell, ps_script),
            vec!["w-ps", "w-ps-done"],
            false,
        ),
        (
            "powershell (ConPTY)",
            ps(&powershell, ps_script),
            vec!["w-ps-done"],
            true,
        ),
        (
            "pwsh",
            ps(&pwsh, ps_script),
            vec!["w-ps", "w-ps-done"],
            false,
        ),
        (
            "pwsh (ConPTY)",
            ps(&pwsh, ps_script),
            vec!["w-ps-done"],
            true,
        ),
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
            vec![
                git_bash.clone(),
                "-lc".to_string(),
                "echo w-bash > b.txt && cat b.txt".to_string(),
            ],
            vec!["w-bash"],
            false,
        ),
        (
            "git bash (ConPTY, stderr shown)",
            vec![
                git_bash.clone(),
                "-lc".to_string(),
                "echo w-bash > b2.txt && cat b2.txt".to_string(),
            ],
            vec!["w-bash"],
            true,
        ),
        (
            "git bash --norc -c (ConPTY)",
            vec![
                git_bash,
                "--norc".to_string(),
                "--noprofile".to_string(),
                "-c".to_string(),
                "echo w-bash; id; mktemp && echo w-mktemp".to_string(),
            ],
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
        (
            "cargo build (rustup proxy)",
            cmd_c(
                "set CARGO_TARGET_DIR=&& cd tinyc && cargo build --offline \
                 && target\\debug\\tinyc.exe",
            ),
            vec!["w-cargo"],
            false,
        ),
        (
            "cargo build (toolchain cargo)",
            direct_cargo("tinyd", ""),
            vec!["w-cargo"],
            false,
        ),
        (
            "cargo build (toolchain cargo, CARGO_HOME in workspace)",
            direct_cargo("tinye", "set CARGO_HOME=%CD%\\ch&& "),
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
                "echo TEMP=%TEMP% TMP=%TMP% && echo x> %TEMP%\\w1560m.txt && type %TEMP%\\w1560m.txt \
                 && mkdir %TEMP%\\w1560m.d && rmdir %TEMP%\\w1560m.d \
                 && del %TEMP%\\w1560m.txt && echo w-temp",
            ),
            vec!["w-temp"],
            false,
        ),
        (
            "HKCU write",
            cmd_c("reg add HKCU\\Software\\Corbanu1560m /v x /d 1 /f && echo w-reg"),
            vec!["w-reg"],
            false,
        ),
    ]
}

fn workloads(backend: Backend, codex_home: &Path) {
    let root = user_owned_test_root();
    let runtime = current_thread_runtime();
    let profile = PermissionProfile::workspace_write();
    // Tiny crates, each its own workspace (the test root sits inside ours).
    for name in ["tinyc", "tinyd", "tinye"] {
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
    for (name, argv, expected, tty) in workload_cases() {
        let started = Instant::now();
        let (stdout, code) = runtime.block_on(run_in(
            backend,
            &profile,
            root.path(),
            codex_home,
            argv,
            &[],
            tty,
        ));
        let ok = code == 0 && expected.iter().all(|part| stdout.contains(part));
        let log = if ok {
            String::new()
        } else {
            let log = sandbox_log(codex_home);
            let lines: Vec<&str> = log.lines().collect();
            lines[lines.len().saturating_sub(6)..].join(" / ")
        };
        report(
            backend,
            name,
            ok,
            &format!(
                "exit={code} secs={} out={stdout} log={log}",
                started.elapsed().as_secs()
            ),
        );
    }
    // The probes this module adds, run inside the sandbox.
    for (name, mode) in [
        ("self token (Chromium-style)", "self_token"),
        ("self process", "self"),
    ] {
        let child_dir = root.path().join(format!("child-{mode}"));
        fs::create_dir_all(&child_dir).expect("child dir");
        let (stdout, code) = runtime.block_on(run_in(
            backend,
            &profile,
            root.path(),
            codex_home,
            probe_argv(PROBE_ENTRY),
            &[
                (PROBE_ENV, mode.to_string()),
                (CHILD_ENV, child_dir.display().to_string()),
            ],
            false,
        ));
        let line = probe_line(&stdout).unwrap_or_else(|| format!("no line exit={code} {stdout}"));
        let ok = line
            .split(' ')
            .filter(|part| part.contains('='))
            .all(|part| part.ends_with("=ok") || part.ends_with("=true"));
        report(backend, name, ok, &line);
    }
    files(backend, codex_home);
}

/// Workspace writes, a pre-existing file, configured read-only paths, `.git`,
/// `%TEMP%`, a folder outside every writable root and HKCU.
fn files(backend: Backend, codex_home: &Path) {
    let test_root = user_owned_test_root();
    let workspace = test_root.path().join("workspace");
    let temp = test_root.path().join("temp");
    let read_only = workspace.join("ro");
    let outside = match backend {
        Backend::Elevated => authenticated_users_folder(),
        Backend::Legacy => user_owned_test_root(),
    };
    fs::create_dir_all(&read_only).expect("read-only dir");
    fs::create_dir_all(workspace.join(".git")).expect(".git");
    fs::create_dir_all(&temp).expect("temp");
    fs::write(read_only.join("orig.txt"), "orig").expect("seed");
    fs::write(workspace.join("pre.txt"), "pre").expect("seed");
    let absolute = |path: &Path| AbsolutePathBuf::from_absolute_path(path).expect("absolute");
    let profile = PermissionProfile::from_runtime_permissions(
        &FileSystemSandboxPolicy::restricted(vec![
            FileSystemSandboxEntry::new(
                FileSystemPath::Special {
                    value: FileSystemSpecialPath::Root,
                },
                FileSystemAccessMode::Read,
            ),
            FileSystemSandboxEntry::new(
                FileSystemPath::Path {
                    path: absolute(&workspace),
                },
                FileSystemAccessMode::Write,
            ),
            FileSystemSandboxEntry::new(
                FileSystemPath::Special {
                    value: FileSystemSpecialPath::Tmpdir,
                },
                FileSystemAccessMode::Write,
            ),
            FileSystemSandboxEntry::new(
                FileSystemPath::Path {
                    path: absolute(&read_only),
                },
                FileSystemAccessMode::Read,
            ),
        ]),
        NetworkSandboxPolicy::Restricted,
    );
    let runtime = current_thread_runtime();
    let temp_value = temp.display().to_string();
    let spec = format!(
        "{}|{}|{}",
        workspace.display(),
        read_only.display(),
        outside.path().display()
    );
    // Twice: the second run modifies what the first created.
    for round in ["first", "second"] {
        let (stdout, code) = runtime.block_on(run_in(
            backend,
            &profile,
            &workspace,
            codex_home,
            probe_argv(PROBE_ENTRY),
            &[
                (PROBE_ENV, "files".to_string()),
                (FILES_ENV, spec.clone()),
                ("TEMP", temp_value.clone()),
                ("TMP", temp_value.clone()),
            ],
            false,
        ));
        let line = probe_line(&stdout).unwrap_or_else(|| format!("no line exit={code} {stdout}"));
        // Every `want_err` part must be refused, every other part allowed.
        let ok = line
            .split(' ')
            .filter(|part| part.contains('='))
            .all(|part| {
                let (name, value) = part.split_once('=').expect("pair");
                if name.starts_with("deny_") {
                    value.starts_with("err")
                } else {
                    value == "ok"
                }
            });
        report(backend, &format!("file access ({round} run)"), ok, &line);
    }
}

/// `ok` or the OS error for `op`.
fn io_result(op: std::io::Result<()>) -> String {
    match op {
        Ok(()) => "ok".to_string(),
        Err(err) => format!("err{}", err.raw_os_error().unwrap_or(-1)),
    }
}

/// The `files` probe: `CODEX_W1560M_FILES` is `workspace|read_only|outside`.
pub(crate) fn files_report() -> String {
    let spec = std::env::var(FILES_ENV).expect("files spec");
    let mut parts = spec.split('|').map(PathBuf::from);
    let (workspace, read_only, outside) = (
        parts.next().expect("workspace"),
        parts.next().expect("read-only"),
        parts.next().expect("outside"),
    );
    let temp = std::env::temp_dir();
    let append = |path: &Path| -> std::io::Result<()> {
        use std::io::Write;
        fs::OpenOptions::new()
            .append(true)
            .open(path)?
            .write_all(b"+")
    };
    let out = vec![
        format!(
            "ws_create={}",
            io_result(fs::write(workspace.join("new.txt"), "x"))
        ),
        format!(
            "ws_modify_preexisting={}",
            io_result(append(&workspace.join("pre.txt")))
        ),
        format!(
            "ws_modify_earlier_command={}",
            io_result(fs::write(workspace.join("new.txt"), "y"))
        ),
        format!(
            "ws_mkdir_rename_delete={}",
            io_result((|| {
                fs::create_dir_all(workspace.join("d1"))?;
                fs::write(workspace.join("d1").join("f"), "x")?;
                fs::rename(workspace.join("d1"), workspace.join("d2"))?;
                fs::remove_dir_all(workspace.join("d2"))
            })())
        ),
        format!(
            "temp_create_delete={}",
            io_result((|| {
                let path = temp.join(format!("w1560m-{}.txt", std::process::id()));
                fs::write(&path, "x")?;
                fs::remove_file(path)
            })())
        ),
        format!(
            "ro_read={}",
            io_result(fs::read(read_only.join("orig.txt")).map(drop))
        ),
        format!(
            "deny_ro_create={}",
            io_result(fs::write(read_only.join("new.txt"), "x"))
        ),
        format!(
            "deny_ro_modify={}",
            io_result(fs::write(read_only.join("orig.txt"), "x"))
        ),
        format!(
            "deny_git_write={}",
            io_result(fs::write(workspace.join(".git").join("probe"), "x"))
        ),
        format!(
            "deny_outside_write={}",
            io_result(fs::write(outside.join("x.txt"), "x"))
        ),
    ];
    out.join(" ")
}

/// The `self_token` probe: what a Chromium/Electron-style sandbox does with
/// its own token (open it for all access, restrict it, start a child with a
/// primary copy).
pub(crate) fn self_token_report() -> String {
    use windows_sys::Win32::Security::CreateRestrictedToken;
    use windows_sys::Win32::Security::TOKEN_ALL_ACCESS;
    use windows_sys::Win32::Security::TOKEN_ASSIGN_PRIMARY;
    // SAFETY: every handle is closed below.
    unsafe {
        let mut parts = Vec::new();
        let mut all: HANDLE = 0;
        if OpenProcessToken(GetCurrentProcess(), TOKEN_ALL_ACCESS, &mut all) == 0 {
            parts.push(format!("open_all_access=err{}", GetLastError()));
        } else {
            CloseHandle(all);
            parts.push("open_all_access=ok".to_string());
        }
        let mut token: HANDLE = 0;
        if OpenProcessToken(
            GetCurrentProcess(),
            TOKEN_DUPLICATE | TOKEN_QUERY | TOKEN_ASSIGN_PRIMARY,
            &mut token,
        ) == 0
        {
            parts.push(format!("open_duplicate=err{}", GetLastError()));
            return parts.join(" ");
        }
        parts.push("open_duplicate=ok".to_string());
        // DISABLE_MAX_PRIVILEGE, then also WRITE_RESTRICTED (the source token is).
        for (name, flags) in [
            ("create_restricted", 0x1u32),
            ("create_write_restricted", 0x9u32),
        ] {
            let mut restricted: HANDLE = 0;
            if CreateRestrictedToken(
                token,
                flags,
                0,
                std::ptr::null(),
                0,
                std::ptr::null(),
                0,
                std::ptr::null(),
                &mut restricted,
            ) == 0
            {
                parts.push(format!("{name}=err{}", GetLastError()));
            } else {
                CloseHandle(restricted);
                parts.push(format!("{name}=ok"));
            }
        }
        // As Chromium does: restricting SIDs (Everyone here).
        let mut everyone = crate::token::world_sid().expect("Everyone SID");
        let restrict = windows_sys::Win32::Security::SID_AND_ATTRIBUTES {
            Sid: everyone.as_mut_ptr().cast(),
            Attributes: 0,
        };
        let mut restricted: HANDLE = 0;
        if CreateRestrictedToken(
            token,
            0,
            0,
            std::ptr::null(),
            0,
            std::ptr::null(),
            1,
            &restrict,
            &mut restricted,
        ) == 0
        {
            parts.push(format!("create_restricted_sid=err{}", GetLastError()));
        } else {
            CloseHandle(restricted);
            parts.push("create_restricted_sid=ok".to_string());
        }
        parts.push(format!("start_child_with_copy={}", primary_result(token)));
        CloseHandle(token);
        // Informational (no `=`, so not scored): this token object's own DACL.
        parts.push(format!("token_dacl:{}", own_token_dacl_sddl()));
        parts.join(" ")
    }
}

/// This process's token object's DACL as SDDL, or `err<code>`.
fn own_token_dacl_sddl() -> String {
    use windows_sys::Win32::Foundation::HLOCAL;
    use windows_sys::Win32::Foundation::LocalFree;
    use windows_sys::Win32::Security::Authorization::ConvertSecurityDescriptorToStringSecurityDescriptorW;
    use windows_sys::Win32::Security::Authorization::GetSecurityInfo;
    use windows_sys::Win32::Security::Authorization::SDDL_REVISION_1;
    use windows_sys::Win32::Security::Authorization::SE_KERNEL_OBJECT;
    use windows_sys::Win32::Security::DACL_SECURITY_INFORMATION;
    // SAFETY: every handle and buffer is checked and freed here.
    unsafe {
        let mut token: HANDLE = 0;
        if OpenProcessToken(GetCurrentProcess(), 0x0002_0000 | TOKEN_QUERY, &mut token) == 0 {
            return format!("open_err{}", GetLastError());
        }
        let mut sd: *mut std::ffi::c_void = std::ptr::null_mut();
        let status = GetSecurityInfo(
            token,
            SE_KERNEL_OBJECT,
            DACL_SECURITY_INFORMATION,
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            &mut sd,
        );
        CloseHandle(token);
        if status != 0 {
            return format!("err{status}");
        }
        let mut text: *mut u16 = std::ptr::null_mut();
        let converted = ConvertSecurityDescriptorToStringSecurityDescriptorW(
            sd,
            SDDL_REVISION_1,
            DACL_SECURITY_INFORMATION,
            &mut text,
            std::ptr::null_mut(),
        ) != 0;
        let sddl = if converted {
            let len = (0..).take_while(|&i| *text.add(i) != 0).count();
            let sddl = String::from_utf16_lossy(std::slice::from_raw_parts(text, len));
            LocalFree(text as HLOCAL);
            sddl
        } else {
            format!("convert_err{}", GetLastError())
        };
        LocalFree(sd as HLOCAL);
        sddl
    }
}

/// The borrow probes with the holder and the borrower in a ConPTY (the
/// mode that still starts under `d368`); results go through a file.
#[test]
fn probe_1560m_elevated_tty_borrow() {
    let Some(codex_home) = elevated_home("w1560m-elevated-tty") else {
        return;
    };
    let _guard = legacy_process_test_guard();
    let holder_root = user_owned_test_root();
    let root = user_owned_test_root();
    let runtime = current_thread_runtime();
    let profile = PermissionProfile::workspace_write();
    for mode in ["borrow", "thread_borrow", "self_token"] {
        let target = holder_root.path().join(format!("{mode}-tty.txt"));
        let out = root.path().join(format!("{mode}.out"));
        let child_dir = root.path().join(format!("child-{mode}"));
        fs::create_dir_all(&child_dir).expect("child dir");
        let (_stdout, code) = runtime.block_on(with_elevated_holder_tty(
            holder_root.path(),
            codex_home.path(),
            /*tty*/ true,
            async |pid| {
                run_in(
                    Backend::Elevated,
                    &profile,
                    root.path(),
                    codex_home.path(),
                    probe_argv(PROBE_ENTRY),
                    &[
                        (PROBE_ENV, mode.to_string()),
                        (TARGET_ENV, pid.to_string()),
                        (FILE_ENV, target.display().to_string()),
                        (OUT_ENV, out.display().to_string()),
                        (CHILD_ENV, child_dir.display().to_string()),
                    ],
                    /*tty*/ true,
                )
                .await
            },
        ));
        let line = fs::read_to_string(&out).unwrap_or_else(|_| format!("no result exit={code}"));
        let ok = if mode == "self_token" {
            line.split(' ')
                .filter(|part| part.contains('='))
                .all(|part| part.ends_with("=ok"))
        } else {
            !target.exists() && !line.contains("create_process=ok") && !line.contains("no result")
        };
        let name = if mode == "self_token" {
            "self token (ConPTY)".to_string()
        } else {
            format!("{mode} closed (ConPTY)")
        };
        report(
            Backend::Elevated,
            &name,
            ok,
            &format!("wrote={} {line}", target.exists()),
        );
    }
    record_setup(codex_home.path());
}

/// The `thread_borrow` probe (#1560, review of #1717): opens a thread of
/// process `pid` (another workspace's command) for direct impersonation and
/// impersonates it with `NtImpersonateThread`, then writes `path` (in that
/// workspace) and opens that process for `PROCESS_CREATE_PROCESS`.
pub(crate) fn thread_borrow_report(pid: u32, path: &Path) -> String {
    use windows_sys::Win32::Security::SECURITY_QUALITY_OF_SERVICE;
    use windows_sys::Win32::Security::TokenImpersonationLevel;
    use windows_sys::Win32::System::Threading::GetCurrentThread;
    use windows_sys::Win32::System::Threading::OpenThread;
    use windows_sys::Win32::System::Threading::OpenThreadToken;
    type NtImpersonateThread =
        unsafe extern "system" fn(HANDLE, HANDLE, *const SECURITY_QUALITY_OF_SERVICE) -> i32;
    let own = write_result(path);
    let threads = threads_of(&std::collections::HashSet::from([pid]));
    let open = |thread: u32, access: u32| -> String {
        // SAFETY: the handle is closed right away.
        unsafe {
            let handle = OpenThread(access, 0, thread);
            if handle == 0 {
                format!("err{}", GetLastError())
            } else {
                CloseHandle(handle);
                "ok".to_string()
            }
        }
    };
    let Some((thread, _)) = threads.first().copied() else {
        return format!("own_write={own} threads=0");
    };
    let mut parts = vec![
        format!("own_write={own}"),
        format!("threads={}", threads.len()),
        format!("direct_impersonation={}", open(thread, 0x0200)),
        format!("impersonate={}", open(thread, 0x0100)),
        format!("set_thread_token={}", open(thread, 0x0080)),
        format!("set_context={}", open(thread, 0x0010)),
        format!("query_limited={}", open(thread, 0x0800)),
    ];
    // SAFETY: every handle is closed below; the thread reverts before returning.
    unsafe {
        let handle = OpenThread(0x0200, 0, thread);
        if handle == 0 {
            parts.push("borrow=not_opened".to_string());
            return parts.join(" ");
        }
        let library = crate::winutil::to_wide("ntdll.dll");
        let module = windows_sys::Win32::System::LibraryLoader::GetModuleHandleW(library.as_ptr());
        let Some(proc) = windows_sys::Win32::System::LibraryLoader::GetProcAddress(
            module,
            c"NtImpersonateThread".as_ptr().cast(),
        ) else {
            CloseHandle(handle);
            parts.push("borrow=no_ntimpersonatethread".to_string());
            return parts.join(" ");
        };
        let nt_impersonate: NtImpersonateThread = std::mem::transmute(proc);
        let qos = SECURITY_QUALITY_OF_SERVICE {
            Length: std::mem::size_of::<SECURITY_QUALITY_OF_SERVICE>() as u32,
            ImpersonationLevel: SecurityImpersonation,
            ContextTrackingMode: 0,
            EffectiveOnly: 0,
        };
        let status = nt_impersonate(GetCurrentThread(), handle, &qos);
        CloseHandle(handle);
        if status != 0 {
            parts.push(format!("borrow=nt_status_{status:#x}"));
            return parts.join(" ");
        }
        let mut token: HANDLE = 0;
        let level = if OpenThreadToken(GetCurrentThread(), TOKEN_QUERY, 1, &mut token) == 0 {
            format!("token_err{}", GetLastError())
        } else {
            let mut level: i32 = -1;
            let mut needed = 0;
            GetTokenInformation(
                token,
                TokenImpersonationLevel,
                (&mut level as *mut i32).cast(),
                std::mem::size_of::<i32>() as u32,
                &mut needed,
            );
            CloseHandle(token);
            // 1 identification, 2 impersonation, 3 delegation.
            format!("{level}")
        };
        let write = write_result(path);
        let create_process = open_result(pid, 0x0080);
        RevertToSelf();
        parts.push(format!(
            "borrow=impersonated level={level} write={write} create_process={create_process}"
        ));
    }
    parts.join(" ")
}

#[test]
fn probe_1560m_legacy_workloads() {
    let _guard = legacy_process_test_guard();
    let codex_home = sandbox_home("w1560m-legacy");
    workloads(Backend::Legacy, codex_home.path());
}

#[test]
fn probe_1560m_elevated_workloads() {
    let Some(codex_home) = elevated_home("w1560m-elevated") else {
        return;
    };
    let _guard = legacy_process_test_guard();
    workloads(Backend::Elevated, codex_home.path());
    record_setup(codex_home.path());
}

fn thread_borrow(backend: Backend, codex_home: &Path) {
    let holder_root = user_owned_test_root();
    let root = user_owned_test_root();
    let target = holder_root.path().join("thread-borrowed.txt");
    let runtime = current_thread_runtime();
    let profile = PermissionProfile::workspace_write();
    let probe = async |holder_pid: u32| {
        run_in(
            backend,
            &profile,
            root.path(),
            codex_home,
            probe_argv(PROBE_ENTRY),
            &[
                (PROBE_ENV, "thread_borrow".to_string()),
                (TARGET_ENV, holder_pid.to_string()),
                (FILE_ENV, target.display().to_string()),
            ],
            false,
        )
        .await
    };
    let (stdout, code) = match backend {
        Backend::Legacy => runtime.block_on(with_holder(holder_root.path(), codex_home, probe)),
        Backend::Elevated => {
            runtime.block_on(with_elevated_holder(holder_root.path(), codex_home, probe))
        }
    };
    let line = probe_line(&stdout).unwrap_or_else(|| format!("no line exit={code} {stdout}"));
    // "works" here means the route is closed.
    let closed = !target.exists() && !line.contains("create_process=ok");
    report(
        backend,
        "thread impersonation borrow closed",
        closed,
        &format!("wrote={} {line}", target.exists()),
    );
}

#[test]
fn probe_1560m_legacy_thread_borrow() {
    let _guard = legacy_process_test_guard();
    let codex_home = sandbox_home("w1560m-legacy-thread");
    thread_borrow(Backend::Legacy, codex_home.path());
}

#[test]
fn probe_1560m_elevated_thread_borrow() {
    let Some(codex_home) = elevated_home("w1560m-elevated-thread") else {
        return;
    };
    let _guard = legacy_process_test_guard();
    thread_borrow(Backend::Elevated, codex_home.path());
    record_setup(codex_home.path());
}
