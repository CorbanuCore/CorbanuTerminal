//! #348 (Travis, 2026-10-20, D55): a placeholder a run made is removed by a
//! later run only from the record it kept under the Corbanu home, also when
//! the temporary folder's registry was lost (a reboot), and only when it is
//! still exactly what the run made and no live run uses it.
//!
//! Each run here has a temporary folder of its own; removing a killed run's
//! folder stands in for a reboot or a sweep of `/tmp`.
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
use codex_sandboxing::landlock::insert_placeholder_record_dir_arg;
use codex_utils_absolute_path::AbsolutePathBuf;
use pretty_assertions::assert_eq;
use std::io::BufRead;
use std::io::BufReader;
use std::io::Write;
use std::os::unix::fs::MetadataExt;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;
use std::process::Child;
use std::process::Command;
use std::process::Output;
use std::process::Stdio;
use std::sync::mpsc;
use std::time::Duration;
use tempfile::TempDir;

/// How long the sandbox may take to start.
const BOUND: Duration = Duration::from_secs(20);

fn helper() -> PathBuf {
    let helper = PathBuf::from(env!("CARGO_BIN_EXE_codex-linux-sandbox"));
    helper.canonicalize().unwrap_or(helper)
}

struct Setup {
    _dir: TempDir,
    base: PathBuf,
    /// The person's home, where the denied `.pfterminal` is missing.
    home: PathBuf,
    /// The Corbanu home the record is kept in.
    corbanu: PathBuf,
}

impl Setup {
    fn new() -> Self {
        let dir = TempDir::new().unwrap();
        let base = dir.path().canonicalize().unwrap();
        let home = base.join("home");
        let corbanu = base.join("corbanu");
        std::fs::create_dir(&home).unwrap();
        std::fs::create_dir(&corbanu).unwrap();
        Self {
            _dir: dir,
            base,
            home,
            corbanu,
        }
    }

    fn placeholder(&self) -> PathBuf {
        self.home.join(".pfterminal")
    }

    /// A temporary folder of a run's own.
    fn tmp(&self, name: &str) -> PathBuf {
        let tmp = self.base.join(format!("tmp-{name}"));
        std::fs::create_dir_all(&tmp).unwrap();
        tmp
    }

    /// The helper running `script` in `base` (writable), with `.pfterminal`
    /// denied, recording its placeholders in the Corbanu home.
    fn command(&self, script: &str, tmp: &PathBuf) -> Command {
        self.command_with(script, tmp, Vec::new())
    }

    /// [`Self::command`] with `extra` entries in its policy.
    fn command_with(
        &self,
        script: &str,
        tmp: &PathBuf,
        extra: Vec<FileSystemSandboxEntry>,
    ) -> Command {
        let deny = FileSystemSandboxEntry::skip_missing_path(
            FileSystemPath::Path {
                path: AbsolutePathBuf::try_from(self.placeholder()).unwrap(),
            },
            FileSystemAccessMode::Deny,
        );
        // `extra` with a write entry for the root: full access with its
        // denials (Moderate's), in place of `base` alone.
        let full_write = extra.iter().any(|entry| {
            entry.access == FileSystemAccessMode::Write
                && matches!(
                    entry.path,
                    FileSystemPath::Special {
                        value: FileSystemSpecialPath::Root
                    }
                )
        });
        let mut entries = if full_write {
            Vec::new()
        } else {
            vec![
                FileSystemSandboxEntry::new(
                    FileSystemPath::Special {
                        value: FileSystemSpecialPath::Root,
                    },
                    FileSystemAccessMode::Read,
                ),
                FileSystemSandboxEntry::new(
                    FileSystemPath::Path {
                        path: AbsolutePathBuf::try_from(self.base.as_path()).unwrap(),
                    },
                    FileSystemAccessMode::Write,
                ),
            ]
        };
        entries.push(deny);
        entries.extend(extra);
        let profile = PermissionProfile::from_runtime_permissions(
            &FileSystemSandboxPolicy::restricted(entries),
            NetworkSandboxPolicy::Restricted,
        );
        let mut args = create_linux_sandbox_command_args_for_permission_profile(
            vec!["/bin/sh".to_string(), "-c".to_string(), script.to_string()],
            &self.base,
            &profile,
            &self.base,
            /*use_legacy_landlock*/ false,
            /*allow_network_for_proxy*/ false,
        );
        insert_placeholder_record_dir_arg(&mut args, &self.corbanu);
        let mut command = Command::new(helper());
        command
            .args(args)
            .current_dir(&self.base)
            .env("TMPDIR", tmp)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        command
    }

    fn run(&self, script: &str, tmp: &PathBuf) -> Output {
        self.command(script, tmp).output().unwrap()
    }

    /// Starts a run that holds its sandbox (and placeholder) until its
    /// standard input closes, once it is set up.
    fn start(&self, tmp: &PathBuf) -> Child {
        self.start_with(tmp, Vec::new())
    }

    /// [`Self::start`] with `extra` entries in its policy.
    fn start_with(&self, tmp: &PathBuf, extra: Vec<FileSystemSandboxEntry>) -> Child {
        let mut child = self
            .command_with("echo ready; read line; echo done", tmp, extra)
            .stdin(Stdio::piped())
            .spawn()
            .unwrap();
        let stdout = child.stdout.take().unwrap();
        let (tx, rx) = mpsc::channel();
        std::thread::spawn(move || {
            let mut lines = BufReader::new(stdout).lines();
            let _ = tx.send(matches!(lines.next(), Some(Ok(line)) if line == "ready"));
            for _ in lines {}
        });
        assert!(
            rx.recv_timeout(BOUND).unwrap_or(false),
            "the sandboxed command did not start"
        );
        child
    }

    /// A run killed mid-command (SIGKILL, as on a timeout), its temporary
    /// folder then lost: its placeholder stays.
    fn crash_and_lose_registry(&self) -> std::fs::Metadata {
        let tmp = self.tmp("crashed");
        let mut crashed = self.start(&tmp);
        crashed.kill().unwrap();
        crashed.wait().unwrap();
        let left = std::fs::symlink_metadata(self.placeholder()).expect("the placeholder stays");
        assert!(left.is_file() && left.len() == 0, "{left:?}");
        std::fs::remove_dir_all(&tmp).unwrap();
        left
    }

    fn records(&self) -> Vec<String> {
        let Ok(entries) = std::fs::read_dir(self.corbanu.join("sandbox-placeholders")) else {
            return Vec::new();
        };
        let mut names = entries
            .flatten()
            .filter_map(|entry| entry.file_name().into_string().ok())
            .filter(|name| name.ends_with(".rec"))
            .collect::<Vec<_>>();
        names.sort();
        names
    }

    /// Whether the sandbox runs here; `false` skips the test.
    fn sandbox_runs(&self) -> bool {
        let output = self.run("echo ok", &self.tmp("probe"));
        if !output.status.success() || output.stdout != b"ok\n" {
            eprintln!(
                "skipping: the sandbox does not run here ({}, {})",
                output.status,
                String::from_utf8_lossy(&output.stderr)
            );
            return false;
        }
        assert!(
            std::fs::symlink_metadata(self.placeholder()).is_err(),
            "a run removes its own placeholder"
        );
        assert_eq!(self.records(), Vec::<String>::new());
        true
    }
}

fn finish(mut child: Child) -> Output {
    drop(child.stdin.take());
    child.wait_with_output().unwrap()
}

fn assert_ran(output: &Output) {
    assert!(
        output.status.success(),
        "{}: {}",
        output.status,
        String::from_utf8_lossy(&output.stderr)
    );
}

/// The gate's fail-before case: without the record kept in the Corbanu
/// home, the placeholder stayed for good once the killed run's temporary
/// folder was lost.
#[test]
fn a_placeholder_left_by_a_killed_run_is_removed_at_the_next_start() {
    let setup = Setup::new();
    if !setup.sandbox_runs() {
        return;
    }
    setup.crash_and_lose_registry();
    assert_eq!(setup.records().len(), 1, "the killed run's record stays");

    let output = setup.run("echo ran", &setup.tmp("next"));
    assert_ran(&output);
    assert!(
        std::fs::symlink_metadata(setup.placeholder()).is_err(),
        "removed at the next start (and the next run removed its own)"
    );
    assert_eq!(setup.records(), Vec::<String>::new());
    assert_eq!(String::from_utf8_lossy(&output.stderr), "");
}

/// The command cannot write the record (here its writable root holds the
/// Corbanu home): a record it forged would have a later run remove a file.
#[test]
fn the_command_cannot_write_the_record() {
    let setup = Setup::new();
    if !setup.sandbox_runs() {
        return;
    }
    let output = setup.run(
        "echo forged > corbanu/sandbox-placeholders/1-1-1.rec; echo write=$?; \
         rm corbanu/sandbox-placeholders/lock; echo rm=$?; \
         mv corbanu/sandbox-placeholders corbanu/moved; echo mv=$?",
        &setup.tmp("forger"),
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    let output = setup.run(
        "(exec 3<corbanu/sandbox-placeholders/lock) 2>/dev/null; echo read=$?; \
         (exec 4>corbanu/sandbox-placeholders/lock) 2>/dev/null; echo write=$?",
        &setup.tmp("locker"),
    );
    let locks = String::from_utf8_lossy(&output.stdout);
    // Root could read it whatever its mode; the read-only bind still stops
    // writing.
    // SAFETY: `geteuid` has no preconditions.
    let steps: &[&str] = if unsafe { libc::geteuid() } == 0 {
        &["write"]
    } else {
        &["read", "write"]
    };
    for step in steps {
        assert!(
            locks.contains(&format!("{step}=")) && !locks.contains(&format!("{step}=0")),
            "the command cannot open the lock file ({step}): {locks}"
        );
    }
    for step in ["write", "rm", "mv"] {
        assert!(
            stdout.contains(&format!("{step}=")) && !stdout.contains(&format!("{step}=0")),
            "{step}: {stdout}"
        );
    }
    assert_eq!(setup.records(), Vec::<String>::new());
    assert!(setup.corbanu.join("sandbox-placeholders/lock").is_file());
}

/// A person's own empty, read-only file with the placeholder's name (made
/// after removing the placeholder) is not removed: it is not what the run
/// recorded. It is reported once.
#[test]
fn a_persons_own_empty_read_only_file_with_the_name_is_kept() {
    let setup = Setup::new();
    if !setup.sandbox_runs() {
        return;
    }
    setup.crash_and_lose_registry();
    std::fs::remove_file(setup.placeholder()).unwrap();
    std::thread::sleep(Duration::from_millis(50));
    std::fs::write(setup.placeholder(), "").unwrap();
    std::fs::set_permissions(setup.placeholder(), std::fs::Permissions::from_mode(0o444)).unwrap();
    let own = std::fs::symlink_metadata(setup.placeholder()).unwrap();

    let output = setup.run("echo ran", &setup.tmp("next"));
    assert_ran(&output);
    let kept = std::fs::symlink_metadata(setup.placeholder()).expect("kept");
    assert_eq!((kept.ino(), kept.len()), (own.ino(), 0));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("left ") && stderr.contains(".pfterminal"),
        "{stderr}"
    );
    assert_eq!(setup.records(), Vec::<String>::new());

    // Reported once, and kept by later runs too.
    let output = setup.run("echo ran", &setup.tmp("later"));
    assert_ran(&output);
    assert_eq!(String::from_utf8_lossy(&output.stderr), "");
    assert_eq!(
        std::fs::symlink_metadata(setup.placeholder())
            .unwrap()
            .ino(),
        own.ino()
    );
}

/// A placeholder filled or replaced since is not removed.
#[test]
fn a_placeholder_filled_or_replaced_since_is_kept() {
    let setup = Setup::new();
    if !setup.sandbox_runs() {
        return;
    }
    setup.crash_and_lose_registry();
    std::fs::set_permissions(setup.placeholder(), std::fs::Permissions::from_mode(0o644)).unwrap();
    std::fs::OpenOptions::new()
        .append(true)
        .open(setup.placeholder())
        .unwrap()
        .write_all(b"data")
        .unwrap();
    let output = setup.run("echo ran", &setup.tmp("filled"));
    assert_ran(&output);
    assert_eq!(std::fs::read(setup.placeholder()).unwrap(), b"data");
    std::fs::remove_file(setup.placeholder()).unwrap();

    setup.crash_and_lose_registry();
    let other = setup.home.join("other");
    std::fs::write(&other, "").unwrap();
    std::fs::set_permissions(&other, std::fs::Permissions::from_mode(0o444)).unwrap();
    let replacement = std::fs::symlink_metadata(&other).unwrap().ino();
    std::fs::rename(&other, setup.placeholder()).unwrap();
    let output = setup.run("echo ran", &setup.tmp("replaced"));
    assert_ran(&output);
    assert_eq!(
        std::fs::symlink_metadata(setup.placeholder())
            .unwrap()
            .ino(),
        replacement
    );
    assert_eq!(setup.records(), Vec::<String>::new());
}

/// While a live run (with a temporary folder of its own, so its registry
/// cannot say so) mounts over a killed run's placeholder, later runs leave
/// it in place; once that run ends, the next start removes it.
#[test]
fn a_placeholder_a_live_run_mounts_over_is_kept_until_it_ends() {
    let setup = Setup::new();
    if !setup.sandbox_runs() {
        return;
    }
    let tmp = setup.tmp("maker");
    let mut maker = setup.start(&tmp);
    let user = setup.start(&setup.tmp("user"));
    maker.kill().unwrap();
    maker.wait().unwrap();
    std::fs::remove_dir_all(&tmp).unwrap();

    let output = setup.run("echo ran", &setup.tmp("next"));
    assert_ran(&output);
    assert!(
        setup.placeholder().is_file(),
        "kept while a live run mounts over it"
    );

    assert_ran(&finish(user));
    let output = setup.run("echo ran", &setup.tmp("after"));
    assert_ran(&output);
    assert!(std::fs::symlink_metadata(setup.placeholder()).is_err());
    assert_eq!(setup.records(), Vec::<String>::new());
}

/// Runs starting at once after a crash: one removes the placeholder, every
/// run works, and nothing is left. (A run whose placeholder another run
/// still held when it ended keeps its record until the next start.)
#[test]
fn runs_starting_at_once_after_a_crash_all_work_and_leave_nothing() {
    let setup = Setup::new();
    if !setup.sandbox_runs() {
        return;
    }
    setup.crash_and_lose_registry();
    let tmp = setup.tmp("at-once");
    let runs = (0..4)
        .map(|_| setup.command("echo ran", &tmp).spawn().unwrap())
        .collect::<Vec<_>>();
    for run in runs {
        let output = run.wait_with_output().unwrap();
        assert_ran(&output);
        assert_eq!(output.stdout, b"ran\n");
    }
    assert!(std::fs::symlink_metadata(setup.placeholder()).is_err());
    assert_ran(&setup.run("echo ran", &tmp));
    assert!(std::fs::symlink_metadata(setup.placeholder()).is_err());
    assert_eq!(setup.records(), Vec::<String>::new());
}

/// A read-only entry for `path` (as core protects a repository's git files).
fn read_only(path: PathBuf) -> FileSystemSandboxEntry {
    FileSystemSandboxEntry::new(
        FileSystemPath::Path {
            path: AbsolutePathBuf::try_from(path).unwrap(),
        },
        FileSystemAccessMode::Read,
    )
}

/// #745: a run whose policy protects paths in another live run's empty
/// placeholder folder (the `.git` it made, which core takes for a
/// repository's once it exists, protecting its hooks and config) holds that
/// folder read-only instead of making placeholders in it. Before, those
/// changed the folder, so its maker could not remove it, its record dropped
/// it, and it was left behind for good. Each run has a temporary folder of
/// its own; the last starts after both have ended, as after a reboot.
#[test]
fn a_placeholder_folder_another_run_protects_paths_in_is_not_left_behind() {
    let setup = Setup::new();
    if !setup.sandbox_runs() {
        return;
    }
    let git = setup.base.join(".git");
    // Full access with denials (Moderate's): the root is writable, `.git`
    // too; its hooks and config are read-only, as core protects them once
    // `.git` exists (or, in one case, a path in it is denied instead).
    let protects = |deny: bool| {
        let mut entries = vec![FileSystemSandboxEntry::new(
            FileSystemPath::Special {
                value: FileSystemSpecialPath::Root,
            },
            FileSystemAccessMode::Write,
        )];
        if deny {
            entries.push(FileSystemSandboxEntry::new(
                FileSystemPath::Path {
                    path: AbsolutePathBuf::try_from(git.join("info/secret")).unwrap(),
                },
                FileSystemAccessMode::Deny,
            ));
        } else {
            entries.push(read_only(git.join("hooks")));
            entries.push(read_only(git.join("config")));
        }
        entries
    };
    // The other run in a temporary folder of its own (found from the
    // record kept in the Corbanu home) or in the maker's (found from its
    // registry); the next run in a third, as after a reboot.
    for (case, shared, deny) in [
        ("own temporary folder", false, false),
        ("shared temporary folder", true, false),
        ("a denied path in it", false, true),
    ] {
        let maker_tmp = setup.tmp(&format!("maker-{case}"));
        let maker = setup.start(&maker_tmp);
        assert!(git.is_dir(), "{case}: the maker's placeholder folder");
        let user_tmp = if shared {
            maker_tmp.clone()
        } else {
            setup.tmp(&format!("user-{case}"))
        };
        let user = setup.start_with(&user_tmp, protects(deny));
        assert_ran(&finish(maker));
        assert_ran(&finish(user));
        let output = setup.run("echo ran", &setup.tmp(&format!("next-{case}")));
        assert_ran(&output);
        assert!(
            std::fs::symlink_metadata(&git).is_err(),
            "{case}: not left behind: {:?}",
            std::fs::read_dir(&git).map(Iterator::collect::<Vec<_>>)
        );
        assert_eq!(setup.records(), Vec::<String>::new(), "{case}");
        assert_eq!(String::from_utf8_lossy(&output.stderr), "", "{case}");
    }

    // The command can create nothing in it, as without the other run.
    let maker = setup.start(&setup.tmp("maker-2"));
    let output = setup
        .command_with(
            "touch .git/hooks 2>/dev/null; echo hooks=$?; \
             mkdir .git/objects 2>/dev/null; echo objects=$?",
            &setup.tmp("writer"),
            protects(/*deny*/ false),
        )
        .output()
        .unwrap();
    assert_ran(&output);
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("hooks=") && !stdout.contains("hooks=0") && !stdout.contains("objects=0"),
        "{stdout}"
    );
    assert_eq!(std::fs::read_dir(&git).unwrap().count(), 0, "unchanged");
    assert_ran(&finish(maker));
    assert!(
        std::fs::symlink_metadata(&git).is_err(),
        "its maker removed it"
    );
}

/// #745: a placeholder folder that was not empty when its run's cleanup came
/// to it (the person put a file in it) stays in that run's record, so the
/// next start reports it once and leaves it. Before, the record dropped it
/// unreported.
#[test]
fn a_placeholder_folder_its_run_could_not_remove_is_reported() {
    let setup = Setup::new();
    if !setup.sandbox_runs() {
        return;
    }
    let git = setup.base.join(".git");
    let maker = setup.start(&setup.tmp("maker"));
    std::fs::write(git.join("notes"), "mine").unwrap();
    assert_ran(&finish(maker));
    assert_eq!(setup.records().len(), 1, "kept in the record");

    let output = setup.run("echo ran", &setup.tmp("next"));
    assert_ran(&output);
    assert_eq!(std::fs::read(git.join("notes")).unwrap(), b"mine");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("left ") && stderr.contains(".git"),
        "{stderr}"
    );
    assert_eq!(setup.records(), Vec::<String>::new());
}

/// Makes the person's own empty, read-only file at `path` (another file, so
/// another inode, than the placeholder there), as an editor saving it would.
fn replace_with_own_empty_file(path: &PathBuf) -> u64 {
    let own = path.with_extension("own");
    std::fs::write(&own, "").unwrap();
    std::fs::set_permissions(&own, std::fs::Permissions::from_mode(0o444)).unwrap();
    let ino = std::fs::symlink_metadata(&own).unwrap().ino();
    std::fs::rename(&own, path).unwrap();
    ino
}

fn ino(path: &PathBuf) -> Option<u64> {
    std::fs::symlink_metadata(path)
        .ok()
        .map(|metadata| metadata.ino())
}

/// #607 (item 5): a killed run's placeholder that the person replaced with
/// their own empty file soon after (within the temporary folder's registry's
/// window) is not removed by the next start in that temporary folder: the
/// record kept in the Corbanu home says it is not the placeholder made
/// there. It is reported once.
#[test]
fn a_persons_own_file_in_place_of_a_killed_runs_placeholder_is_kept() {
    let setup = Setup::new();
    if !setup.sandbox_runs() {
        return;
    }
    let tmp = setup.tmp("shared");
    let mut crashed = setup.start(&tmp);
    crashed.kill().unwrap();
    crashed.wait().unwrap();
    let own = replace_with_own_empty_file(&setup.placeholder());

    let output = setup.run("echo ran", &tmp);
    assert_ran(&output);
    assert_eq!(ino(&setup.placeholder()), Some(own), "kept");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains(".pfterminal"), "{stderr}");
    assert_eq!(setup.records(), Vec::<String>::new());
}

/// #607 (item 4): the last run out that shares another run's placeholder
/// does not remove it once the person has replaced it with their own empty
/// file while the command ran.
#[test]
fn the_last_run_out_keeps_a_shared_placeholder_the_person_replaced() {
    let setup = Setup::new();
    if !setup.sandbox_runs() {
        return;
    }
    let tmp = setup.tmp("shared");
    let mut maker = setup.start(&tmp);
    let holder = setup.start(&tmp);
    maker.kill().unwrap();
    maker.wait().unwrap();
    let own = replace_with_own_empty_file(&setup.placeholder());

    assert_ran(&finish(holder));
    assert_eq!(ino(&setup.placeholder()), Some(own), "kept");
    let output = setup.run("echo ran", &setup.tmp("next"));
    assert_ran(&output);
    assert_eq!(ino(&setup.placeholder()), Some(own));
    assert_eq!(setup.records(), Vec::<String>::new());
}

/// #607 (item 4): a run that holds a killed run's placeholder (left in
/// place while a live run of another temporary folder mounts over it) does
/// not remove the path once the person has replaced it while its command
/// ran.
#[test]
fn a_held_placeholder_the_person_replaced_is_kept() {
    let setup = Setup::new();
    if !setup.sandbox_runs() {
        return;
    }
    let tmp = setup.tmp("shared");
    let mut maker = setup.start(&tmp);
    let user = setup.start(&setup.tmp("user"));
    maker.kill().unwrap();
    maker.wait().unwrap();
    let holder = setup.start(&tmp);
    let own = replace_with_own_empty_file(&setup.placeholder());

    assert_ran(&finish(user));
    assert_ran(&finish(holder));
    assert_eq!(ino(&setup.placeholder()), Some(own), "kept");
    let output = setup.run("echo ran", &setup.tmp("next"));
    assert_ran(&output);
    assert_eq!(ino(&setup.placeholder()), Some(own));
    assert_eq!(setup.records(), Vec::<String>::new());
}

/// #607: a run's own placeholder that the person replaced with their own
/// empty file while the command ran is left to them, and reported at the
/// next start. Before, the run's cleanup removed any empty file there.
#[test]
fn a_runs_own_placeholder_the_person_replaced_is_kept() {
    let setup = Setup::new();
    if !setup.sandbox_runs() {
        return;
    }
    let maker = setup.start(&setup.tmp("maker"));
    let own = replace_with_own_empty_file(&setup.placeholder());
    assert_ran(&finish(maker));
    assert_eq!(ino(&setup.placeholder()), Some(own), "kept");

    let output = setup.run("echo ran", &setup.tmp("next"));
    assert_ran(&output);
    assert_eq!(ino(&setup.placeholder()), Some(own));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains(".pfterminal"), "{stderr}");
    assert_eq!(setup.records(), Vec::<String>::new());
}
