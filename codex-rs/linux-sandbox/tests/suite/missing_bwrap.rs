#![cfg(target_os = "linux")]
//! Issue #563: on a host without bubblewrap, the sandbox helper refuses a
//! command with a message that names what to install. It used to panic
//! (status 101), or abort with no message (134) where it runs from a
//! constructor, as in test binaries. The command never runs.

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
use std::process::Command;
use std::process::Output;
use tempfile::TempDir;

const RAN: &str = "issue563-command-ran";

/// The helper, copied into a folder of its own: nothing bundled next to it
/// (every place it looks for a bundled copy is inside `dir`).
fn lone_helper(dir: &Path) -> PathBuf {
    let bin = dir.join("bin");
    std::fs::create_dir(&bin).expect("create bin folder");
    let helper = bin.join("codex-linux-sandbox");
    // #635: copied by another process, so no descriptor open for writing
    // leaks into other tests' forks (executing it would fail with ETXTBSY).
    let copied = Command::new("cp")
        .arg(env!("CARGO_BIN_EXE_codex-linux-sandbox"))
        .arg(&helper)
        .status()
        .expect("copy the helper");
    assert!(copied.success(), "copy the helper");
    helper
}

fn profiles(dir: &Path) -> Vec<(&'static str, PermissionProfile)> {
    let secret = AbsolutePathBuf::from_absolute_path(dir.join("missing-secret")).expect("absolute");
    // Full write access with a denied path: the run that forks bubblewrap's
    // child to clean up placeholders after it (protected levels); the
    // refusal comes from that child.
    let protected = FileSystemSandboxPolicy::restricted(vec![
        FileSystemSandboxEntry::new(
            FileSystemPath::Special {
                value: FileSystemSpecialPath::Root,
            },
            FileSystemAccessMode::Write,
        ),
        FileSystemSandboxEntry::new(
            FileSystemPath::Path { path: secret },
            FileSystemAccessMode::Deny,
        ),
    ]);
    vec![
        ("read-only", PermissionProfile::read_only()),
        (
            "protected",
            PermissionProfile::from_runtime_permissions(
                &protected,
                NetworkSandboxPolicy::Restricted,
            ),
        ),
    ]
}

fn run(helper: &Path, path_env: &Path, cwd: &Path, profile: &PermissionProfile) -> Output {
    Command::new(helper)
        .env_clear()
        .env("PATH", path_env)
        .current_dir(cwd)
        .arg("--sandbox-policy-cwd")
        .arg(cwd)
        .arg("--permission-profile")
        .arg(serde_json::to_string(profile).expect("profile json"))
        .arg("--")
        .args(["/bin/sh", "-c", &format!("echo {RAN}")])
        .output()
        .expect("run the helper")
}

#[test]
fn without_bubblewrap_the_helper_refuses_and_names_what_to_install() {
    let dir = TempDir::new().expect("temp dir");
    let helper = lone_helper(dir.path());
    // A PATH with no `bwrap` on it.
    let empty_path = dir.path().join("empty-path");
    std::fs::create_dir(&empty_path).expect("create empty PATH folder");
    let mut wrong = Vec::new();
    for (name, profile) in profiles(dir.path()) {
        let output = run(&helper, &empty_path, dir.path(), &profile);
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        if output.status.code() != Some(1)
            || stdout.contains(RAN)
            || stderr.trim() != codex_sandboxing::BWRAP_UNAVAILABLE_MESSAGE
        {
            wrong.push(format!(
                "{name}: status={:?} stdout={stdout:?} stderr={stderr:?}",
                output.status
            ));
        }
    }
    assert_eq!(wrong, Vec::<String>::new());
}

/// With a bubblewrap next to the helper (as the release packages ship it,
/// in `codex-resources/`), the helper uses it when none is on PATH. Runs
/// where the `bwrap` binary was built next to the helper
/// (`cargo build --bin bwrap`) and the host lets it create user namespaces.
#[test]
fn without_bubblewrap_on_path_the_helper_uses_its_bundled_copy() {
    let built = Path::new(env!("CARGO_BIN_EXE_codex-linux-sandbox")).with_file_name("bwrap");
    if !built.is_file() {
        eprintln!("skipped: no bwrap built at {}", built.display());
        return;
    }
    let dir = TempDir::new().expect("temp dir");
    let helper = lone_helper(dir.path());
    let resources = dir.path().join("bin/codex-resources");
    std::fs::create_dir(&resources).expect("create codex-resources");
    std::fs::copy(&built, resources.join("bwrap")).expect("copy bwrap");
    let empty_path = dir.path().join("empty-path");
    std::fs::create_dir(&empty_path).expect("create empty PATH folder");
    let probe = run(
        &helper,
        &empty_path,
        dir.path(),
        &PermissionProfile::read_only(),
    );
    let stderr = String::from_utf8_lossy(&probe.stderr);
    // A bubblewrap outside the system's AppArmor allowance cannot create
    // the namespaces where unprivileged user namespaces are restricted
    // (Ubuntu 24.04 by default).
    if ["namespace", "uid map", "RTM_NEWADDR", "RTM_NEWLINK"]
        .iter()
        .any(|failure| stderr.contains(failure))
    {
        eprintln!("skipped: this host cannot create user namespaces: {stderr}");
        return;
    }
    let mut wrong = Vec::new();
    for (name, profile) in profiles(dir.path()) {
        let output = run(&helper, &empty_path, dir.path(), &profile);
        let stdout = String::from_utf8_lossy(&output.stdout);
        if !output.status.success() || !stdout.contains(RAN) {
            wrong.push(format!(
                "{name}: status={:?} stdout={stdout:?} stderr={:?}",
                output.status,
                String::from_utf8_lossy(&output.stderr)
            ));
        }
    }
    assert_eq!(wrong, Vec::<String>::new());
}

/// #581: where AppArmor restricts unprivileged user namespaces (Ubuntu
/// 24.04 by default), the bundled copy could not start (`loopback: Failed
/// RTM_NEWADDR`), so every sandboxed command failed. It now runs through
/// `aa-exec -p rootlesskit`, from its verified descriptor, which the
/// command does not inherit. Runs where `bwrap` was built next to the
/// helper (`cargo build -p codex-bwrap`), the host restricts user
/// namespaces and the `rootlesskit` profile lets a bubblewrap start.
#[test]
fn where_apparmor_restricts_user_namespaces_the_bundled_copy_runs_through_aa_exec() {
    let built = Path::new(env!("CARGO_BIN_EXE_codex-linux-sandbox")).with_file_name("bwrap");
    if !built.is_file() {
        eprintln!("skipped: no bwrap built at {}", built.display());
        return;
    }
    let restricted =
        std::fs::read_to_string("/proc/sys/kernel/apparmor_restrict_unprivileged_userns")
            .is_ok_and(|value| value.trim() == "1");
    let through_aa_exec = Command::new("aa-exec")
        .args(["-p", "rootlesskit", "--"])
        .arg(&built)
        .args(["--unshare-user", "--unshare-net", "--ro-bind", "/", "/"])
        .args(["--", "/bin/true"])
        .output()
        .is_ok_and(|output| output.status.success());
    if !restricted || !through_aa_exec {
        eprintln!(
            "skipped: user namespaces restricted={restricted}, bubblewrap through aa-exec={through_aa_exec}"
        );
        return;
    }
    let dir = TempDir::new().expect("temp dir");
    let helper = lone_helper(dir.path());
    let resources = dir.path().join("bin/codex-resources");
    std::fs::create_dir(&resources).expect("create codex-resources");
    std::fs::copy(&built, resources.join("bwrap")).expect("copy bwrap");
    let empty_path = dir.path().join("empty-path");
    std::fs::create_dir(&empty_path).expect("create empty PATH folder");
    let mut wrong = Vec::new();
    for (name, profile) in profiles(dir.path()) {
        let output = Command::new(&helper)
            .env_clear()
            .env("PATH", &empty_path)
            .current_dir(dir.path())
            .arg("--sandbox-policy-cwd")
            .arg(dir.path())
            .arg("--permission-profile")
            .arg(serde_json::to_string(&profile).expect("profile json"))
            .arg("--")
            .args([
                "/bin/sh",
                "-c",
                &format!("echo {RAN}; /bin/ls -l /proc/self/fd/"),
            ])
            .output()
            .expect("run the helper");
        let stdout = String::from_utf8_lossy(&output.stdout);
        // The command inherits no descriptor of the bundled copy.
        if !output.status.success() || !stdout.contains(RAN) || stdout.contains("bwrap") {
            wrong.push(format!(
                "{name}: status={:?} stdout={stdout:?} stderr={:?}",
                output.status,
                String::from_utf8_lossy(&output.stderr)
            ));
        }
    }
    assert_eq!(wrong, Vec::<String>::new());
}
