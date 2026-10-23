//! #826 (Travis, 2026-10-15): a sandbox whose policy denies user namespaces
//! (Core adds the entry under Moderate and Aggressive) stops the command
//! creating one, so it cannot open what bubblewrap hides (`unshare -Ur`) or
//! mount over it (`unshare -Urm`). Everyday programs keep working (`clone3`
//! falls back to `clone`), and a sandbox started inside says why it cannot
//! start. Without the entry (Permissive) nothing changes.
#![cfg(target_os = "linux")]
#![allow(clippy::unwrap_used)]

use codex_core::exec::ExecCapturePolicy;
use codex_core::exec::ExecParams;
use codex_core::exec::process_exec_tool_call;
use codex_core::exec_env::create_env;
use codex_core::sandboxing::SandboxPermissions;
use codex_protocol::config_types::ShellEnvironmentPolicy;
use codex_protocol::config_types::WindowsSandboxLevel;
use codex_protocol::error::CodexErrorDetails;
use codex_protocol::error::SandboxErr;
use codex_protocol::exec_output::ExecToolCallOutput;
use codex_protocol::models::PermissionProfile;
use codex_protocol::permissions::FileSystemAccessMode;
use codex_protocol::permissions::FileSystemPath;
use codex_protocol::permissions::FileSystemSandboxEntry;
use codex_protocol::permissions::FileSystemSandboxPolicy;
use codex_protocol::permissions::FileSystemSpecialPath;
use codex_protocol::permissions::NetworkSandboxPolicy;
use codex_utils_absolute_path::AbsolutePathBuf;
use pretty_assertions::assert_eq;
use std::path::Path;
use std::path::PathBuf;
use tempfile::TempDir;

const TIMEOUT_MS: u64 = 10_000;
const SECRET: &str = "secret-826";
const DENIED: &str = "Operation not permitted";
const NESTED_REFUSED: &str = "Corbanu's Linux sandbox cannot start here";
/// Set in the command's environment where the sandbox denies namespaces.
const MARKER_ENV: &str = "CODEX_SANDBOX_USER_NAMESPACES_DENIED";

fn helper() -> PathBuf {
    let helper = PathBuf::from(env!("CARGO_BIN_EXE_codex-linux-sandbox"));
    helper.canonicalize().unwrap_or(helper)
}

/// Reads everything but `denied`; with `deny_user_namespaces`, the entry
/// Core adds under Moderate and Aggressive.
fn profile(
    denied: &Path,
    deny_user_namespaces: bool,
    network: NetworkSandboxPolicy,
) -> PermissionProfile {
    let mut entries = vec![
        FileSystemSandboxEntry::new(
            FileSystemPath::Special {
                value: FileSystemSpecialPath::Root,
            },
            FileSystemAccessMode::Read,
        ),
        FileSystemSandboxEntry::new(
            FileSystemPath::Path {
                path: AbsolutePathBuf::try_from(denied).unwrap(),
            },
            FileSystemAccessMode::Deny,
        ),
    ];
    if deny_user_namespaces {
        entries.push(FileSystemSandboxEntry::deny_user_namespaces());
    }
    PermissionProfile::from_runtime_permissions(
        &FileSystemSandboxPolicy::restricted(entries),
        network,
    )
}

/// Runs `command` in the sandbox; a failed command's output too.
async fn run(command: Vec<String>, cwd: &Path, profile: &PermissionProfile) -> ExecToolCallOutput {
    let cwd = AbsolutePathBuf::try_from(cwd).unwrap();
    let params = ExecParams {
        command,
        cwd: cwd.clone(),
        expiration: TIMEOUT_MS.into(),
        capture_policy: ExecCapturePolicy::ShellTool,
        env: create_env(&ShellEnvironmentPolicy::default(), /*thread_id*/ None),
        network: None,
        network_environment_id: None,
        sandbox_permissions: SandboxPermissions::UseDefault,
        windows_sandbox_level: WindowsSandboxLevel::Disabled,
        windows_sandbox_private_desktop: false,
        justification: None,
        arg0: None,
    };
    match process_exec_tool_call(
        params,
        profile,
        &cwd,
        std::slice::from_ref(&cwd),
        &Some(helper()),
        /*use_legacy_landlock*/ false,
        /*stdout_stream*/ None,
    )
    .await
    {
        Ok(output) => output,
        Err(err) => match err.details() {
            CodexErrorDetails::Sandbox(SandboxErr::Denied { output, .. }) => {
                output.as_ref().clone()
            }
            details => panic!("sandboxed command did not run: {details:?}"),
        },
    }
}

fn sh(script: &str) -> Vec<String> {
    vec!["sh".to_string(), "-c".to_string(), script.to_string()]
}

/// A folder with a file the profile denies, and whether bubblewrap runs
/// here at all.
struct Fixture {
    dir: TempDir,
    denied: PathBuf,
}

impl Fixture {
    async fn new() -> Option<Self> {
        let dir = TempDir::new().unwrap();
        let denied = dir.path().canonicalize().unwrap().join("denied.txt");
        std::fs::write(&denied, SECRET).unwrap();
        let fixture = Self { dir, denied };
        let probe = run(
            sh("echo ready"),
            fixture.dir.path(),
            &profile(
                &fixture.denied,
                /*deny_user_namespaces*/ false,
                NetworkSandboxPolicy::Restricted,
            ),
        )
        .await;
        if probe.exit_code != 0 || !probe.stdout.text.contains("ready") {
            eprintln!(
                "skipped: bubblewrap does not run here: {}",
                probe.stderr.text
            );
            return None;
        }
        Some(fixture)
    }

    async fn run(
        &self,
        command: Vec<String>,
        deny_user_namespaces: bool,
        network: NetworkSandboxPolicy,
    ) -> ExecToolCallOutput {
        run(
            command,
            self.dir.path(),
            &profile(&self.denied, deny_user_namespaces, network),
        )
        .await
    }
}

/// What a run shows: whether it printed `marker` (the namespace did what it
/// was made for) or the secret, and whether it was refused with `EPERM`.
fn seen(output: &ExecToolCallOutput, marker: &str) -> (bool, bool) {
    let all = format!("{}{}", output.stdout.text, output.stderr.text);
    (
        all.contains(SECRET) || all.contains(marker),
        output.stderr.text.contains(DENIED),
    )
}

/// `unshare -Ur cat` of a denied file (bubblewrap's unreadable placeholder,
/// which root in a namespace of its own can open: the evidence nested-launch
/// detection relies on) and `unshare -Urm` mounting over its folder: refused with
/// `EPERM` wherever the entry is, with and without network. Without it
/// (Permissive) the namespace is created as before; where the host allows a
/// nested one, the placeholder opens (the hole #826 closes).
#[tokio::test]
async fn issue_826_denied_user_namespaces_keep_hidden_files_hidden() {
    let Some(fixture) = Fixture::new().await else {
        return;
    };
    let denied = fixture.denied.to_string_lossy().into_owned();
    let read = sh(&format!("unshare -Ur cat {denied} && echo OPENED-826"));
    // Mounting over the folder that holds it (as over a Corbanu home's
    // runtime folder).
    let folder = fixture.dir.path().canonicalize().unwrap();
    let mount = sh(&format!(
        "unshare -Urm sh -c 'mount -t tmpfs none {} && echo MOUNTED-826'",
        folder.display()
    ));
    let mut wrong = Vec::new();
    for network in [
        NetworkSandboxPolicy::Restricted,
        NetworkSandboxPolicy::Enabled,
    ] {
        for (name, command, marker) in [
            ("read", &read, "OPENED-826"),
            ("mount", &mount, "MOUNTED-826"),
        ] {
            let output = fixture
                .run(command.clone(), /*deny_user_namespaces*/ true, network)
                .await;
            let outcome = seen(&output, marker);
            if outcome != (false, true) {
                wrong.push(format!(
                    "{name} {network:?}: {outcome:?} stdout={:?} stderr={:?}",
                    output.stdout.text, output.stderr.text
                ));
            }
        }
    }
    assert_eq!(wrong, Vec::<String>::new());

    // Permissive: no filter and no marker (the namespace is made where the
    // host allows a nested one, as before).
    let before = fixture
        .run(
            sh(&format!(
                "echo marker=${{{MARKER_ENV}:-unset}}; unshare -Ur cat {denied} && echo OPENED-826"
            )),
            /*deny_user_namespaces*/ false,
            NetworkSandboxPolicy::Restricted,
        )
        .await;
    eprintln!(
        "without the entry: exit={} stdout={:?} stderr={:?}",
        before.exit_code, before.stdout.text, before.stderr.text
    );
    assert!(
        before.stdout.text.contains("marker=unset"),
        "Permissive changed: {before:?}"
    );
}

/// Every process in the sandbox is filtered, bubblewrap's own PID 1 included
/// (it is no descendant of the command, which could otherwise take it over
/// with `ptrace` or `/proc/1/mem` and make a namespace through it), and a
/// raw `clone(CLONE_NEWUSER)` is refused like `unshare`. Without the entry
/// PID 1 gets no filter, as before.
#[tokio::test]
async fn issue_826_every_process_in_the_sandbox_is_filtered() {
    let Some(fixture) = Fixture::new().await else {
        return;
    };
    // Counted against this test process's own filters, which every process
    // started from it inherits.
    let filters = |status: &str| {
        status
            .lines()
            .find_map(|line| line.strip_prefix("Seccomp_filters:"))
            .and_then(|count| count.trim().parse::<u32>().ok())
    };
    let Some(own) = filters(&std::fs::read_to_string("/proc/self/status").unwrap()) else {
        eprintln!("skipped: this kernel does not report Seccomp_filters");
        return;
    };
    let mut seen = Vec::new();
    for deny_user_namespaces in [true, false] {
        let output = fixture
            .run(
                sh("cat /proc/1/status"),
                deny_user_namespaces,
                NetworkSandboxPolicy::Enabled,
            )
            .await;
        seen.push((deny_user_namespaces, filters(&output.stdout.text)));
    }
    assert_eq!(seen, vec![(true, Some(own + 2)), (false, Some(own))]);
    if !Path::new("/usr/bin/python3").exists() {
        eprintln!("skipped the raw clone check: no /usr/bin/python3");
        return;
    }
    // `clone(CLONE_NEWUSER | SIGCHLD)`; a child that got through exits at once.
    let script = "import ctypes, os, platform\n\
        nr = {'x86_64': 56, 'aarch64': 220}[platform.machine()]\n\
        libc = ctypes.CDLL(None, use_errno=True)\n\
        rc = libc.syscall(nr, ctypes.c_ulong(0x10000000 | 17), None, None, None, None)\n\
        if rc == 0: os._exit(0)\n\
        print('clone', rc, ctypes.get_errno())\n";
    let output = fixture
        .run(
            vec![
                "/usr/bin/python3".to_string(),
                "-c".to_string(),
                script.to_string(),
            ],
            /*deny_user_namespaces*/ true,
            NetworkSandboxPolicy::Enabled,
        )
        .await;
    assert!(
        output
            .stdout
            .text
            .contains(&format!("clone -1 {}", libc::EPERM)),
        "{output:?}"
    );
}

/// Threads and child processes still start under the denial: `clone3` fails
/// with `ENOSYS`, so the C library falls back to `clone`.
#[tokio::test]
async fn issue_826_everyday_programs_still_run() {
    let Some(fixture) = Fixture::new().await else {
        return;
    };
    let shell = fixture
        .run(
            sh("(echo child) & wait; echo shell-ok"),
            /*deny_user_namespaces*/ true,
            NetworkSandboxPolicy::Restricted,
        )
        .await;
    assert_eq!(
        (shell.exit_code, shell.stdout.text.contains("shell-ok")),
        (0, true),
        "{shell:?}"
    );
    if !Path::new("/usr/bin/python3").exists() {
        eprintln!("skipped the python3 checks: no /usr/bin/python3");
        return;
    }
    let script = "import ctypes, subprocess, threading\n\
        libc = ctypes.CDLL(None, use_errno=True)\n\
        rc = libc.syscall(435, None, 0)\n\
        print('clone3', rc, ctypes.get_errno())\n\
        t = threading.Thread(target=lambda: None); t.start(); t.join()\n\
        subprocess.run(['true'], check=True)\n\
        print('python-ok')\n";
    let python = |deny_user_namespaces| {
        fixture.run(
            vec![
                "/usr/bin/python3".to_string(),
                "-c".to_string(),
                script.to_string(),
            ],
            deny_user_namespaces,
            NetworkSandboxPolicy::Restricted,
        )
    };
    let denied = python(/*deny_user_namespaces*/ true).await;
    assert_eq!(
        (
            denied.exit_code,
            denied.stdout.text.contains("python-ok"),
            denied
                .stdout
                .text
                .contains(&format!("clone3 -1 {}", libc::ENOSYS)),
        ),
        (0, true, true),
        "{denied:?}"
    );
    // Permissive: `clone3` reaches the kernel as before (bad arguments).
    let allowed = python(/*deny_user_namespaces*/ false).await;
    assert!(
        allowed.stdout.text.contains("python-ok")
            && !allowed
                .stdout
                .text
                .contains(&format!("clone3 -1 {}", libc::ENOSYS)),
        "{allowed:?}"
    );
}

/// A sandbox started inside one that denies user namespaces cannot create
/// the namespace bubblewrap needs; it says why, naming the levels, instead
/// of leaving it to bubblewrap's message.
#[tokio::test]
async fn issue_826_a_sandbox_inside_says_why_it_cannot_start() {
    let Some(fixture) = Fixture::new().await else {
        return;
    };
    let cwd = fixture.dir.path().canonicalize().unwrap();
    let nested = vec![
        helper().to_string_lossy().into_owned(),
        "--sandbox-policy-cwd".to_string(),
        cwd.to_string_lossy().into_owned(),
        "--permission-profile".to_string(),
        serde_json::to_string(&PermissionProfile::read_only()).unwrap(),
        "--".to_string(),
        "echo".to_string(),
        "nested-ran".to_string(),
    ];
    let output = fixture
        .run(
            nested.clone(),
            /*deny_user_namespaces*/ true,
            NetworkSandboxPolicy::Restricted,
        )
        .await;
    assert_eq!(
        (
            output.exit_code == 0,
            output.stdout.text.contains("nested-ran"),
            output.stderr.text.contains(NESTED_REFUSED),
        ),
        (false, false, true),
        "{output:?}"
    );
    // Permissive: no such message (the nested sandbox runs where the host
    // allows it, as before).
    let before = fixture
        .run(
            nested,
            /*deny_user_namespaces*/ false,
            NetworkSandboxPolicy::Restricted,
        )
        .await;
    eprintln!(
        "without the entry: exit={} stdout={:?} stderr={:?}",
        before.exit_code, before.stdout.text, before.stderr.text
    );
    assert!(!before.stderr.text.contains(NESTED_REFUSED), "{before:?}");
}
