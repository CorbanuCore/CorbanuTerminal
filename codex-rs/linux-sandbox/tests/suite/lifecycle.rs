//! #935 (Travis, 2026-10-19, D35 option A): a sandboxed command cannot
//! outlive its session. When the helper's parent exits, when the helper is
//! killed, when bubblewrap dies early or when the command exits leaving a
//! daemon behind, every process in the sandbox stops, double-forked daemons
//! included.
//!
//! Each command holds the helper's stdout open in every process it starts,
//! so the pipe's end of file means none is left. The cases where
//! bubblewrap dies before binding the sandbox's PID 1 to its parent (the
//! race in #935) run a real bubblewrap through a wrapper that drops
//! `--die-with-parent`, which leaves the sandbox in that state for good.
//!
//! #1006 (Travis, 2026-10-20, D47 option B): the same holds for commands run
//! without bubblewrap (full disk write access without managed-proxy
//! routing, legacy Landlock), which also keep the signals the session and
//! the terminal send them.
#![cfg(target_os = "linux")]
#![allow(clippy::unwrap_used)]

use codex_protocol::models::PermissionProfile;
use codex_protocol::permissions::FileSystemAccessMode;
use codex_protocol::permissions::FileSystemPath;
use codex_protocol::permissions::FileSystemSandboxEntry;
use codex_protocol::permissions::FileSystemSandboxPolicy;
use codex_protocol::permissions::FileSystemSpecialPath;
use codex_protocol::permissions::NetworkSandboxPolicy;
use codex_sandboxing::landlock::create_linux_sandbox_command_args_for_permission_profile;
use codex_utils_absolute_path::AbsolutePathBuf;
use std::io::BufRead;
use std::io::BufReader;
use std::os::fd::AsRawFd;
use std::os::fd::FromRawFd;
use std::os::fd::OwnedFd;
use std::path::Path;
use std::path::PathBuf;
use std::process::Child;
use std::process::ChildStdout;
use std::process::Command;
use std::process::Stdio;
use std::sync::atomic::AtomicU32;
use std::sync::atomic::Ordering;
use std::sync::mpsc;
use std::time::Duration;
use tempfile::TempDir;

/// How long the sandbox may take to start, and to be gone once stopped.
const BOUND: Duration = Duration::from_secs(20);

fn helper() -> PathBuf {
    let helper = PathBuf::from(env!("CARGO_BIN_EXE_codex-linux-sandbox"));
    helper.canonicalize().unwrap_or(helper)
}

/// The system bubblewrap, which the wrapper runs.
fn system_bwrap() -> Option<PathBuf> {
    std::env::split_paths(&std::env::var_os("PATH")?)
        .map(|dir| dir.join("bwrap"))
        .find(|path| path.is_file())
}

/// A unique `sleep` duration, to find the process by its command line.
fn marker() -> String {
    static NEXT: AtomicU32 = AtomicU32::new(0);
    format!(
        "86400.{}{:04}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::SeqCst)
    )
}

/// Pids of processes with `marker` as a whole argument.
fn processes_with(marker: &str) -> Vec<i32> {
    let mut pids = Vec::new();
    for entry in std::fs::read_dir("/proc").unwrap().flatten() {
        let Some(pid) = entry.file_name().to_str().and_then(|n| n.parse().ok()) else {
            continue;
        };
        let Ok(cmdline) = std::fs::read(entry.path().join("cmdline")) else {
            continue;
        };
        if cmdline
            .split(|byte| *byte == 0)
            .any(|arg| arg == marker.as_bytes())
        {
            pids.push(pid);
        }
    }
    pids
}

fn kill(pid: i32) {
    // SAFETY: plain syscall.
    unsafe {
        libc::kill(pid, libc::SIGKILL);
    }
}

/// Writes an executable file from another process (#635): a descriptor
/// open for writing here would leak into other tests' forks and make
/// executing the file fail with `ETXTBSY`.
fn write_executable(path: &Path, contents: &str) {
    let status = Command::new("/bin/sh")
        .args([
            "-c",
            "printf '%s' \"$1\" > \"$2\" && chmod 755 \"$2\"",
            "sh",
        ])
        .arg(contents)
        .arg(path)
        .status()
        .unwrap();
    assert!(status.success(), "write {}", path.display());
}

/// How bubblewrap runs.
#[derive(Clone, Copy, PartialEq)]
enum Bwrap {
    /// As the helper finds it.
    Real,
    /// Without `--die-with-parent`: PID 1 never binds to its parent.
    Unbound,
    /// Must not run (#1006): a wrapper that records its pid and fails.
    Unused,
}

/// #1006: how a command runs without bubblewrap.
#[derive(Clone, Copy, Debug)]
enum Unwrapped {
    /// Full disk write access, network restricted, no proxy routing.
    FullDisk,
    /// Legacy Landlock (`--use-legacy-landlock`), read-only.
    LegacyLandlock,
}

const UNWRAPPED: [Unwrapped; 2] = [Unwrapped::FullDisk, Unwrapped::LegacyLandlock];

struct Fixture {
    work: TempDir,
    wrapper_dir: TempDir,
    /// The command's daemon and foreground `sleep` markers.
    daemon: String,
    foreground: String,
}

impl Fixture {
    fn new() -> Self {
        Self {
            work: TempDir::new().unwrap(),
            wrapper_dir: TempDir::new().unwrap(),
            daemon: marker(),
            foreground: marker(),
        }
    }

    fn pid_file(&self) -> PathBuf {
        self.wrapper_dir.path().join("bwrap-pids")
    }

    /// The helper's arguments for `script`, writable in the work folder.
    fn helper_args(&self, script: &str) -> Vec<String> {
        let work = self.work.path().canonicalize().unwrap();
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
                        path: AbsolutePathBuf::try_from(work.as_path()).unwrap(),
                    },
                    FileSystemAccessMode::Write,
                ),
            ]),
            NetworkSandboxPolicy::Restricted,
        );
        create_linux_sandbox_command_args_for_permission_profile(
            vec!["/bin/sh".to_string(), "-c".to_string(), script.to_string()],
            &work,
            &profile,
            &work,
            /*use_legacy_landlock*/ false,
            /*allow_network_for_proxy*/ false,
        )
    }

    /// The helper's arguments for `script` run without bubblewrap.
    fn unwrapped_args(&self, mode: Unwrapped, script: &str) -> Vec<String> {
        let work = self.work.path().canonicalize().unwrap();
        let (profile, use_legacy_landlock) = match mode {
            Unwrapped::FullDisk => (
                PermissionProfile::from_runtime_permissions(
                    &FileSystemSandboxPolicy::restricted(vec![FileSystemSandboxEntry::new(
                        FileSystemPath::Special {
                            value: FileSystemSpecialPath::Root,
                        },
                        FileSystemAccessMode::Write,
                    )]),
                    NetworkSandboxPolicy::Restricted,
                ),
                false,
            ),
            Unwrapped::LegacyLandlock => (
                PermissionProfile::from_runtime_permissions(
                    &FileSystemSandboxPolicy::restricted(vec![FileSystemSandboxEntry::new(
                        FileSystemPath::Special {
                            value: FileSystemSpecialPath::Root,
                        },
                        FileSystemAccessMode::Read,
                    )]),
                    NetworkSandboxPolicy::Restricted,
                ),
                true,
            ),
        };
        create_linux_sandbox_command_args_for_permission_profile(
            vec!["/bin/sh".to_string(), "-c".to_string(), script.to_string()],
            &work,
            &profile,
            &work,
            use_legacy_landlock,
            /*allow_network_for_proxy*/ false,
        )
    }

    /// `program` with `args`, finding bubblewrap as `bwrap` says.
    fn command(&self, bwrap: Bwrap, program: &Path, args: &[String]) -> Command {
        let mut command = Command::new(program);
        command
            .args(args)
            .current_dir(self.work.path())
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit());
        if bwrap == Bwrap::Unbound {
            let real = system_bwrap().unwrap();
            let wrapper = self.wrapper_dir.path().join("bwrap");
            write_executable(
                &wrapper,
                &format!(
                    "#!/bin/sh\necho $$ >> '{}'\nfor arg do\n  shift\n  [ \"$arg\" = --die-with-parent ] && continue\n  set -- \"$@\" \"$arg\"\ndone\nexec '{}' \"$@\"\n",
                    self.pid_file().display(),
                    real.display()
                ),
            );
            let path = std::env::var_os("PATH").unwrap_or_default();
            let mut dirs = vec![self.wrapper_dir.path().to_path_buf()];
            dirs.extend(std::env::split_paths(&path));
            command.env("PATH", std::env::join_paths(dirs).unwrap());
        }
        if bwrap == Bwrap::Unused {
            let wrapper = self.wrapper_dir.path().join("bwrap");
            write_executable(
                &wrapper,
                &format!(
                    "#!/bin/sh\necho $$ >> '{}'\nexit 99\n",
                    self.pid_file().display()
                ),
            );
            let path = std::env::var_os("PATH").unwrap_or_default();
            let mut dirs = vec![self.wrapper_dir.path().to_path_buf()];
            dirs.extend(std::env::split_paths(&path));
            command.env("PATH", std::env::join_paths(dirs).unwrap());
        }
        command
    }

    /// A command that leaves a double-forked daemon, which says `ready` once
    /// in a session of its own (out of reach of a process-group kill), waits
    /// for it to say so (end of file on a pipe it holds until then), then
    /// sleeps in the foreground (or exits, without `foreground`).
    fn script(&self, foreground: bool) -> String {
        let mut script = format!(
            "exec 3>&1; (setsid /bin/sh -c 'echo ready; exec sleep {} 4>&-' 4>&1 1>&3 3>&- &) | read _",
            self.daemon
        );
        if foreground {
            script.push_str(&format!("; exec sleep {}", self.foreground));
        }
        script
    }

    /// Whether the sandbox runs here; `false` skips the test.
    fn sandbox_runs(&self, bwrap: Bwrap) -> bool {
        if bwrap == Bwrap::Unbound && system_bwrap().is_none() {
            eprintln!("skipping: no system bubblewrap");
            return false;
        }
        self.probe(bwrap, &self.helper_args("echo ok"))
    }

    /// Whether the command runs here without bubblewrap; `false` skips the
    /// test.
    fn unwrapped_runs(&self, mode: Unwrapped) -> bool {
        let runs = self.probe(Bwrap::Unused, &self.unwrapped_args(mode, "echo ok"));
        assert!(
            !self.pid_file().exists(),
            "{mode:?}: the helper ran bubblewrap"
        );
        runs
    }

    fn probe(&self, bwrap: Bwrap, args: &[String]) -> bool {
        let mut probe = self.command(bwrap, &helper(), args).spawn().unwrap();
        let mut stdout = probe.stdout.take().unwrap();
        let (tx, rx) = mpsc::channel();
        std::thread::spawn(move || {
            let mut text = String::new();
            let _ = std::io::Read::read_to_string(&mut stdout, &mut text);
            let _ = tx.send(text);
        });
        let Ok(text) = rx.recv_timeout(BOUND) else {
            let _ = probe.kill();
            reap(&mut probe);
            panic!("the sandbox probe did not finish");
        };
        let status = reap(&mut probe);
        if !status.success() || text != "ok\n" {
            eprintln!("skipping: the sandbox does not run here ({status}, {text:?})");
            return false;
        }
        // Otherwise the case would test nothing.
        assert!(
            bwrap != Bwrap::Unbound || self.pid_file().is_file(),
            "the helper did not run bubblewrap through the wrapper"
        );
        true
    }

    /// Kills what is left of the command's processes.
    fn kill_marked(&self) -> Vec<i32> {
        let mut left: Vec<i32> = [&self.daemon, &self.foreground]
            .iter()
            .flat_map(|marker| processes_with(marker))
            .collect();
        left.sort_unstable();
        left.dedup();
        for pid in &left {
            kill(*pid);
        }
        left
    }

    /// Fails unless every process holding `stdout` is gone within
    /// [`BOUND`] and no marked process is left; kills what is left either
    /// way.
    fn assert_all_gone(&self, ended: &mpsc::Receiver<()>, what: &str) {
        let gone = ended.recv_timeout(BOUND).is_ok();
        let left = self.kill_marked();
        assert!(
            gone && left.is_empty(),
            "{what}: sandboxed processes outlived it (stdout closed: {gone}, marked processes left: {left:?})"
        );
    }
}

impl Drop for Fixture {
    /// A failed test leaves nothing running either: also no shell whose
    /// script names a marker (one that keeps starting them).
    fn drop(&mut self) {
        for _ in 0..50 {
            let mut left = self.kill_marked();
            for marker in [&self.daemon, &self.foreground] {
                for pid in processes_naming(marker) {
                    kill(pid);
                    left.push(pid);
                }
            }
            if left.is_empty() {
                return;
            }
        }
    }
}

/// Pids of processes with an argument containing `marker` (not this one).
fn processes_naming(marker: &str) -> Vec<i32> {
    let me = std::process::id() as i32;
    let mut pids = Vec::new();
    for entry in std::fs::read_dir("/proc").unwrap().flatten() {
        let Some(pid) = entry.file_name().to_str().and_then(|n| n.parse().ok()) else {
            continue;
        };
        let Ok(cmdline) = std::fs::read(entry.path().join("cmdline")) else {
            continue;
        };
        if pid != me
            && cmdline
                .split(|byte| *byte == 0)
                .any(|arg| arg.windows(marker.len()).any(|w| w == marker.as_bytes()))
        {
            pids.push(pid);
        }
    }
    pids
}

/// Waits for `ready`, then signals when `stdout` reaches end of file.
fn watch(stdout: ChildStdout) -> mpsc::Receiver<()> {
    let (ready_tx, ready_rx) = mpsc::channel();
    let (ended_tx, ended_rx) = mpsc::channel();
    std::thread::spawn(move || {
        let mut lines = BufReader::new(stdout).lines();
        if matches!(lines.next(), Some(Ok(line)) if line == "ready") {
            let _ = ready_tx.send(());
        }
        for _ in lines.by_ref() {}
        let _ = ended_tx.send(());
    });
    ready_rx
        .recv_timeout(BOUND)
        .expect("the sandboxed command did not start");
    ended_rx
}

fn reap(child: &mut Child) -> std::process::ExitStatus {
    child.wait().unwrap()
}

/// The helper's parent exits (the session's process dies): with a real
/// bubblewrap, the helper notices and stops the sandbox.
#[test]
fn the_sandbox_stops_when_the_helpers_parent_exits() {
    let fixture = Fixture::new();
    if !fixture.sandbox_runs(Bwrap::Real) {
        return;
    }
    let mut args = vec![
        "-c".to_string(),
        "\"$@\" & wait".to_string(),
        "sh".to_string(),
        helper().display().to_string(),
    ];
    args.extend(fixture.helper_args(&fixture.script(/*foreground*/ true)));
    let mut parent = fixture
        .command(Bwrap::Real, Path::new("/bin/sh"), &args)
        .spawn()
        .unwrap();
    let ended = watch(parent.stdout.take().unwrap());
    parent.kill().unwrap();
    reap(&mut parent);
    fixture.assert_all_gone(&ended, "the helper's parent exited");
}

/// The helper is killed (as a session's process-group kill does) while
/// bubblewrap has not bound the sandbox to itself.
#[test]
fn the_sandbox_stops_when_the_helper_is_killed() {
    let fixture = Fixture::new();
    if !fixture.sandbox_runs(Bwrap::Unbound) {
        return;
    }
    let mut helper = fixture
        .command(
            Bwrap::Unbound,
            &helper(),
            &fixture.helper_args(&fixture.script(/*foreground*/ true)),
        )
        .spawn()
        .unwrap();
    let ended = watch(helper.stdout.take().unwrap());
    helper.kill().unwrap();
    reap(&mut helper);
    fixture.assert_all_gone(&ended, "the helper was killed");
}

/// Bubblewrap dies before binding the sandbox to itself: the helper stops
/// the sandbox and exits as bubblewrap did.
#[test]
fn the_sandbox_stops_when_bubblewrap_dies_early() {
    use std::os::unix::process::ExitStatusExt;

    let fixture = Fixture::new();
    if !fixture.sandbox_runs(Bwrap::Unbound) {
        return;
    }
    std::fs::remove_file(fixture.pid_file()).unwrap();
    let mut helper = fixture
        .command(
            Bwrap::Unbound,
            &helper(),
            &fixture.helper_args(&fixture.script(/*foreground*/ true)),
        )
        .spawn()
        .unwrap();
    let ended = watch(helper.stdout.take().unwrap());
    // The last bubblewrap started is the command's (probes run first).
    let pids = std::fs::read_to_string(fixture.pid_file()).unwrap();
    kill(pids.lines().last().unwrap().parse().unwrap());
    fixture.assert_all_gone(&ended, "bubblewrap died");
    assert_eq!(reap(&mut helper).signal(), Some(libc::SIGKILL));
}

/// The command exits and leaves a daemon: the daemon is stopped and the
/// helper exits with the command's status, also when bubblewrap never
/// bound the sandbox to itself.
#[test]
fn a_daemon_left_by_a_finished_command_is_stopped() {
    for bwrap in [Bwrap::Real, Bwrap::Unbound] {
        let fixture = Fixture::new();
        if !fixture.sandbox_runs(bwrap) {
            return;
        }
        let script = format!("{}; exit 7", fixture.script(/*foreground*/ false));
        let mut helper = fixture
            .command(bwrap, &helper(), &fixture.helper_args(&script))
            .spawn()
            .unwrap();
        let ended = watch(helper.stdout.take().unwrap());
        let what = match bwrap {
            Bwrap::Real => "the command exited",
            Bwrap::Unbound | Bwrap::Unused => "the command exited (unbound sandbox)",
        };
        fixture.assert_all_gone(&ended, what);
        assert_eq!(reap(&mut helper).code(), Some(7));
    }
}

fn signal_group(group: i32, signal: libc::c_int) {
    // SAFETY: plain syscall.
    assert_eq!(unsafe { libc::kill(-group, signal) }, 0, "signal the group");
}

/// #1006: the helper's parent exits: a command run without bubblewrap stops,
/// with the daemon it left.
#[test]
fn a_command_without_bubblewrap_stops_when_the_helpers_parent_exits() {
    for mode in UNWRAPPED {
        let fixture = Fixture::new();
        if !fixture.unwrapped_runs(mode) {
            continue;
        }
        let mut args = vec![
            "-c".to_string(),
            "\"$@\" & wait".to_string(),
            "sh".to_string(),
            helper().display().to_string(),
        ];
        args.extend(fixture.unwrapped_args(mode, &fixture.script(/*foreground*/ true)));
        let mut parent = fixture
            .command(Bwrap::Unused, Path::new("/bin/sh"), &args)
            .spawn()
            .unwrap();
        let ended = watch(parent.stdout.take().unwrap());
        parent.kill().unwrap();
        reap(&mut parent);
        fixture.assert_all_gone(&ended, &format!("{mode:?}: the helper's parent exited"));
    }
}

/// #1006: the helper alone is killed.
#[test]
fn a_command_without_bubblewrap_stops_when_the_helper_is_killed() {
    for mode in UNWRAPPED {
        let fixture = Fixture::new();
        if !fixture.unwrapped_runs(mode) {
            continue;
        }
        let mut helper = fixture
            .command(
                Bwrap::Unused,
                &helper(),
                &fixture.unwrapped_args(mode, &fixture.script(/*foreground*/ true)),
            )
            .spawn()
            .unwrap();
        let ended = watch(helper.stdout.take().unwrap());
        helper.kill().unwrap();
        reap(&mut helper);
        fixture.assert_all_gone(&ended, &format!("{mode:?}: the helper was killed"));
    }
}

/// #1006: the session kills the helper's process group, as Corbanu does.
#[test]
fn a_command_without_bubblewrap_stops_when_its_process_group_is_killed() {
    use std::os::unix::process::CommandExt;

    for mode in UNWRAPPED {
        let fixture = Fixture::new();
        if !fixture.unwrapped_runs(mode) {
            continue;
        }
        let mut helper = fixture
            .command(
                Bwrap::Unused,
                &helper(),
                &fixture.unwrapped_args(mode, &fixture.script(/*foreground*/ true)),
            )
            .process_group(0)
            .spawn()
            .unwrap();
        let ended = watch(helper.stdout.take().unwrap());
        signal_group(helper.id() as i32, libc::SIGKILL);
        reap(&mut helper);
        fixture.assert_all_gone(&ended, &format!("{mode:?}: the process group was killed"));
    }
}

/// #1006: the command exits and leaves a daemon: the daemon is stopped and
/// the helper exits with the command's status.
#[test]
fn a_daemon_left_by_a_command_without_bubblewrap_is_stopped() {
    for mode in UNWRAPPED {
        let fixture = Fixture::new();
        if !fixture.unwrapped_runs(mode) {
            continue;
        }
        let script = format!("{}; exit 7", fixture.script(/*foreground*/ false));
        let mut helper = fixture
            .command(
                Bwrap::Unused,
                &helper(),
                &fixture.unwrapped_args(mode, &script),
            )
            .spawn()
            .unwrap();
        let ended = watch(helper.stdout.take().unwrap());
        fixture.assert_all_gone(&ended, &format!("{mode:?}: the command exited"));
        assert_eq!(reap(&mut helper).code(), Some(7), "{mode:?}");
    }
}

/// #1006: a command run without bubblewrap that dies of a signal: the
/// helper dies of it too.
#[test]
fn the_helper_dies_of_the_signal_a_command_without_bubblewrap_died_of() {
    use std::os::unix::process::ExitStatusExt;

    for mode in UNWRAPPED {
        let fixture = Fixture::new();
        if !fixture.unwrapped_runs(mode) {
            continue;
        }
        let status = fixture
            .command(
                Bwrap::Unused,
                &helper(),
                &fixture.unwrapped_args(mode, "kill -TERM $$"),
            )
            .status()
            .unwrap();
        assert_eq!(status.signal(), Some(libc::SIGTERM), "{mode:?}");
    }
}

/// #1006: signals sent to the helper's process group reach a command run
/// without bubblewrap once each, also when it has moved to a session and
/// process group of its own (as an interactive shell moves to a group of its
/// own); the helper exits as the command did.
#[test]
fn a_command_without_bubblewrap_gets_its_groups_signals_once() {
    use std::os::unix::process::CommandExt;

    let has_setsid = Command::new("/bin/sh")
        .args(["-c", "command -v setsid"])
        .stdout(Stdio::null())
        .status()
        .unwrap()
        .success();
    for mode in UNWRAPPED {
        for moves in [false, true] {
            if moves && !has_setsid {
                eprintln!("skipping the moved case: no setsid");
                continue;
            }
            let fixture = Fixture::new();
            if !fixture.unwrapped_runs(mode) {
                continue;
            }
            let shell = format!(
                "n=0; trap 'n=$((n+1)); echo interrupted' INT; trap 'echo \"$n\"; exit 3' TERM; echo ready; while :; do sleep {} & wait; done",
                fixture.foreground
            );
            let script = if moves {
                // Not the group leader, so `setsid` does not fork.
                format!("exec setsid /bin/sh -c '{}'", shell.replace('\'', "'\\''"))
            } else {
                shell
            };
            let mut helper = fixture
                .command(
                    Bwrap::Unused,
                    &helper(),
                    &fixture.unwrapped_args(mode, &script),
                )
                .process_group(0)
                .spawn()
                .unwrap();
            let group = helper.id() as i32;
            let (tx, rx) = mpsc::channel();
            let stdout = helper.stdout.take().unwrap();
            std::thread::spawn(move || {
                for line in BufReader::new(stdout).lines() {
                    let Ok(line) = line else { break };
                    if tx.send(line).is_err() {
                        break;
                    }
                }
            });
            let next = |what: &str| {
                rx.recv_timeout(BOUND)
                    .unwrap_or_else(|_| panic!("{mode:?} (moved: {moves}): no {what}"))
            };
            assert_eq!(next("ready"), "ready");
            signal_group(group, libc::SIGINT);
            assert_eq!(next("interrupt"), "interrupted");
            // Time for a second (relayed) SIGINT to arrive, if one would.
            std::thread::sleep(Duration::from_millis(300));
            signal_group(group, libc::SIGTERM);
            assert_eq!(
                next("count"),
                "1",
                "{mode:?} (moved: {moves}): SIGINT reached the command more than once"
            );
            assert_eq!(
                reap(&mut helper).code(),
                Some(3),
                "{mode:?} (moved: {moves})"
            );
            assert!(
                fixture.kill_marked().is_empty(),
                "{mode:?} (moved: {moves}): the command's sleep outlived it"
            );
        }
    }
}

/// A pseudo-terminal: the master, and the slave to become a controlling
/// terminal; `None` (the test skips) where none can be opened.
fn open_pty() -> Option<(OwnedFd, OwnedFd)> {
    // SAFETY: plain calls; the descriptors are checked and owned here.
    unsafe {
        let master = libc::posix_openpt(libc::O_RDWR | libc::O_NOCTTY | libc::O_CLOEXEC);
        if master < 0 {
            return None;
        }
        let master = OwnedFd::from_raw_fd(master);
        let mut name = [0 as libc::c_char; 128];
        if libc::grantpt(master.as_raw_fd()) != 0
            || libc::unlockpt(master.as_raw_fd()) != 0
            || libc::ptsname_r(master.as_raw_fd(), name.as_mut_ptr(), name.len()) != 0
        {
            return None;
        }
        let slave = libc::open(
            name.as_ptr(),
            libc::O_RDWR | libc::O_NOCTTY | libc::O_CLOEXEC,
        );
        (slave >= 0).then(|| (master, OwnedFd::from_raw_fd(slave)))
    }
}

/// Reads `master` into `text` until it contains `needle`.
fn read_until(master: &OwnedFd, text: &mut String, needle: &str) {
    let deadline = std::time::Instant::now() + BOUND;
    while !text.contains(needle) {
        let left = deadline.saturating_duration_since(std::time::Instant::now());
        assert!(!left.is_zero(), "no {needle:?} on the terminal: {text:?}");
        let mut poll_fd = libc::pollfd {
            fd: master.as_raw_fd(),
            events: libc::POLLIN,
            revents: 0,
        };
        // SAFETY: one valid pollfd.
        if unsafe { libc::poll(&mut poll_fd, 1, left.as_millis() as libc::c_int) } <= 0 {
            continue;
        }
        let mut buffer = [0u8; 1024];
        // SAFETY: reads at most `buffer.len()` bytes into `buffer`.
        let read =
            unsafe { libc::read(master.as_raw_fd(), buffer.as_mut_ptr().cast(), buffer.len()) };
        assert!(read > 0, "the terminal closed: {text:?}");
        text.push_str(&String::from_utf8_lossy(&buffer[..read as usize]));
    }
}

fn write_all(master: &OwnedFd, bytes: &[u8]) {
    // SAFETY: writes `bytes` (a few bytes: fits the terminal's buffer).
    let written = unsafe { libc::write(master.as_raw_fd(), bytes.as_ptr().cast(), bytes.len()) };
    assert_eq!(written, bytes.len() as isize, "write to the terminal");
}

/// #1006: a command run without bubblewrap on a terminal, as Corbanu runs
/// one (the helper leads the session): the terminal's stop signal does not
/// stop it, as before (its process group stays orphaned), and a hangup
/// still reaches it (it went to the session leader, which it was).
#[test]
fn a_command_without_bubblewrap_keeps_its_terminal() {
    use std::os::unix::process::CommandExt;
    use std::os::unix::process::ExitStatusExt;

    for mode in UNWRAPPED {
        let fixture = Fixture::new();
        if !fixture.unwrapped_runs(mode) {
            continue;
        }
        let Some((master, slave)) = open_pty() else {
            eprintln!("skipping: no pseudo-terminal");
            return;
        };
        let script = format!(
            "echo ready; read line; echo \"got $line\"; exec sleep {}",
            fixture.foreground
        );
        let mut command = fixture.command(
            Bwrap::Unused,
            &helper(),
            &fixture.unwrapped_args(mode, &script),
        );
        command
            .stdin(Stdio::from(slave.try_clone().unwrap()))
            .stdout(Stdio::from(slave.try_clone().unwrap()))
            .stderr(Stdio::from(slave));
        // SAFETY: async-signal-safe calls between fork and exec.
        unsafe {
            command.pre_exec(|| {
                if libc::setsid() < 0 || libc::ioctl(0, libc::TIOCSCTTY, 0) < 0 {
                    return Err(std::io::Error::last_os_error());
                }
                Ok(())
            });
        }
        let mut helper = command.spawn().unwrap();
        // Only the helper holds the slave now.
        drop(command);
        let mut text = String::new();
        read_until(&master, &mut text, "ready");
        write_all(&master, b"\x1a");
        write_all(&master, b"x\n");
        read_until(&master, &mut text, "got x");
        // The last descriptor of the master: the slave hangs up (once a
        // parallel test's fork, which may hold a copy until it execs, has
        // let go of it).
        drop(master);
        let (tx, rx) = mpsc::channel();
        std::thread::spawn(move || {
            let _ = tx.send(reap(&mut helper));
        });
        let status = rx
            .recv_timeout(BOUND)
            .unwrap_or_else(|_| panic!("{mode:?}: the command outlived its terminal's hangup"));
        assert_eq!(status.signal(), Some(libc::SIGHUP), "{mode:?}: {status}");
        assert!(fixture.kill_marked().is_empty(), "{mode:?}");
    }
}
