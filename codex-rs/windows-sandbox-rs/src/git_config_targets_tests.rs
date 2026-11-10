use super::*;
use pretty_assertions::assert_eq;
use std::fs;

fn tempdir() -> (tempfile::TempDir, PathBuf) {
    let dir = tempfile::tempdir().expect("tempdir");
    let path = dunce::canonicalize(dir.path()).expect("canonical");
    (dir, path)
}

/// `(path below base, why, kind)` of each target.
fn shown(found: Vec<Target>, base: &Path) -> Vec<(PathBuf, String, TargetKind)> {
    found
        .into_iter()
        .map(|target| {
            (
                target
                    .path
                    .strip_prefix(base)
                    .expect("below base")
                    .to_path_buf(),
                target.why,
                target.kind,
            )
        })
        .collect()
}

#[test]
fn config_entries_parse_like_git() {
    let text = concat!(
        "\u{FEFF}[core]\r\n",
        "\thooksPath = .husky/_ ; trailing comment\r\n",
        "\tbare\n",
        "# comment\n",
        "[remote \"Origin\"]\n",
        "\turl = \"C:\\\\a b\" # c\n",
        "[alias]\n",
        "\tx = !\"./run me.sh\" \\\n",
        "\t--flag\n",
        "[includeIf \"gitdir:~/w/\"]\n",
        "\tpath = extra.cfg\n",
        "[Diff.pdf]\n",
        "\ttextconv = ./tools/conv\n",
    );
    assert_eq!(
        parse(text)
            .into_iter()
            .map(|entry| (entry.name, entry.value))
            .collect::<Vec<_>>(),
        vec![
            ("core.hookspath".to_string(), ".husky/_".to_string()),
            ("core.bare".to_string(), "true".to_string()),
            ("remote.Origin.url".to_string(), "C:\\a b".to_string()),
            // Git keeps the blanks around a continuation, as spaces.
            ("alias.x".to_string(), "!./run me.sh  --flag".to_string()),
            (
                "includeif.gitdir:~/w/.path".to_string(),
                "extra.cfg".to_string()
            ),
            ("diff.pdf.textconv".to_string(), "./tools/conv".to_string()),
        ]
    );
}

/// D422: `core.hooksPath` (with Husky's folders), includes and every path in
/// a program value are targets, also inside a nested repository's `.git`;
/// a bare program name, a URL, a path outside the roots and one inside a
/// root's own `.git` aren't.
#[test]
fn targets_are_what_configs_name_inside_the_roots() {
    let (_dir, base) = tempdir();
    let work = base.join("proj");
    let git = work.join(".git");
    fs::create_dir_all(&git).expect("git folder");
    fs::write(
        git.join("config"),
        concat!(
            "[core]\n\thooksPath = .husky/_\n\tpager = less\n\tfsmonitor = ./fsmon.exe\n",
            "[include]\n\tpath = ../team.gitconfig\n",
            "[alias]\n\tst = !\"tools/st.sh\" -s\n\tlg = log\n\tf = \"!f() { sh ./x.sh \\\"$@\\\"; }; f\"\n",
            "[filter \"lfs\"]\n\tsmudge = git-lfs smudge\n",
            "[gpg]\n\tprogram = /elsewhere/gpg.exe\n",
            "[gpg \"x\"]\n\tprogram = ../../outside/gpg.exe\n",
            "[credential]\n\thelper = node scripts/cred.js https://example.com/x\n",
            "[diff \"x\"]\n\ttextconv = .git/conv\n",
        ),
    )
    .expect("config");
    fs::write(
        work.join("team.gitconfig"),
        "[sequence]\n\teditor = ./seq-editor\n",
    )
    .expect("include");
    let found = targets(
        &git,
        Some(&work),
        &[],
        &[],
        std::slice::from_ref(&base),
        &mut ConfigBudget::default(),
    );
    let file = |path: &str, why: &str| (PathBuf::from(path), why.to_string(), TargetKind::File);
    assert_eq!(
        shown(found, &work),
        vec![
            (
                PathBuf::from(".husky"),
                "core.hooksPath".to_string(),
                TargetKind::HuskyHooks
            ),
            (
                Path::new(".husky").join("_").join("_"),
                "core.hooksPath".to_string(),
                TargetKind::Hooks
            ),
            (
                Path::new(".husky").join("_"),
                "core.hooksPath".to_string(),
                TargetKind::Hooks
            ),
            file("fsmon.exe", "core.fsmonitor"),
            file("team.gitconfig", "include.path"),
            file("seq-editor", "sequence.editor"),
            file("tools/st.sh", "alias.st"),
            // Every later word may be a file (`sh` and `f` here aren't).
            file("sh", "alias.f"),
            file("x.sh", "alias.f"),
            file("f", "alias.f"),
            file("smudge", "filter.lfs.smudge"),
            // A rooted word is also tried below the work tree
            // (`"$(git rev-parse --show-toplevel)/x.sh"`).
            file("elsewhere/gpg.exe", "gpg.program"),
            file("scripts/cred.js", "credential.helper"),
            file(".git/conv", "diff.x.textconv"),
            // `receive-pack` runs hooks from the git folder.
            (
                Path::new(".git").join(".husky"),
                "core.hooksPath".to_string(),
                TargetKind::HuskyHooks
            ),
            (
                Path::new(".git").join(".husky").join("_").join("_"),
                "core.hooksPath".to_string(),
                TargetKind::Hooks
            ),
            (
                Path::new(".git").join(".husky").join("_"),
                "core.hooksPath".to_string(),
                TargetKind::Hooks
            ),
        ]
    );
}

/// D422: what changed in a target since the session started; Husky's
/// folder counts only files named after git hooks, a rewrite with the same
/// content isn't a change, and a target that appears is reported with what
/// is in it.
#[test]
fn target_changes_compare_contents() {
    let (_dir, base) = tempdir();
    let work = base.join("proj");
    let git = work.join(".git");
    fs::create_dir_all(&git).expect("git folder");
    fs::create_dir_all(work.join(".husky").join("_")).expect("husky");
    fs::write(
        git.join("config"),
        "[core]\n\thooksPath = .husky/_\n[alias]\n\tx = !./x.sh\n",
    )
    .expect("config");
    fs::write(work.join(".husky/_/pre-commit"), "stub").expect("stub");
    let repos = [(git, Some(work.clone()))];
    let roots = [base];
    let states = || target_states_with(&repos, &roots, &[], &[]);
    let before = states();
    fs::write(work.join(".husky/_/pre-commit"), "stub").expect("same stub");
    fs::write(work.join(".husky/README.md"), "notes").expect("not a hook");
    let unchanged = target_changes(&before, states());
    fs::write(work.join(".husky/post-checkout"), "evil").expect("hook");
    fs::write(work.join(".husky/_/pre-commit"), "evil").expect("stub changed");
    fs::write(work.join("x.sh"), "evil").expect("script appears");
    let changed = target_changes(&before, states());
    assert_eq!(
        (unchanged, changed),
        (
            Vec::new(),
            vec![
                (
                    work.join(".husky"),
                    vec!["core.hooksPath: post-checkout added".to_string()]
                ),
                (
                    work.join(".husky").join("_"),
                    vec!["core.hooksPath: pre-commit changed".to_string()]
                ),
                (work.join("x.sh"), vec!["alias.x: added".to_string()]),
            ]
        )
    );
}

/// D422 review: a global config (and what it includes, `~/` expanded) that
/// lies in a writable root is a target; its relative `core.hooksPath`
/// applies to every repository, so a hook in a new repository's folder
/// counts; Husky v8's `_` folder and a custom Husky v9 folder count; an
/// oversized hooks folder counts every time.
#[test]
fn global_configs_new_repositories_and_husky_layouts() {
    let (_dir, base) = tempdir();
    let home = base.join("home");
    fs::create_dir_all(&home).expect("home");
    fs::write(
        home.join(".gitconfig"),
        "[core]\n\thooksPath = .githooks\n[include]\n\tpath = ~/extra.cfg\n",
    )
    .expect("global");
    fs::write(home.join("extra.cfg"), "[user]\n\tname = x\n").expect("included");
    let outside = [home.join(".gitconfig")];
    let homes = [home.clone()];
    let roots = [base.clone()];
    let repo = base.join("new");
    fs::create_dir_all(repo.join(".git")).expect("repo");
    let repos = [(repo.join(".git"), Some(repo.clone()))];
    let states = || target_states_with(&repos, &roots, &outside, &homes);
    let before = states();
    fs::create_dir_all(repo.join(".githooks")).expect("hooks");
    fs::write(repo.join(".githooks").join("pre-commit"), "evil").expect("hook");
    fs::write(home.join("extra.cfg"), "[core]\n\tfsmonitor = evil.exe\n").expect("edit");
    let mut changed = target_changes(&before, states());
    changed.sort();

    let v8 = base.join("v8");
    fs::create_dir_all(v8.join(".git")).expect("v8 repo");
    fs::create_dir_all(v8.join(".config").join("husky").join("_")).expect("husky");
    fs::write(
        v8.join(".git").join("config"),
        "[core]\n\thooksPath = .config/husky\n",
    )
    .expect("v8 config");
    let v8_targets = targets(
        &v8.join(".git"),
        Some(&v8),
        &[],
        &[],
        &roots,
        &mut ConfigBudget::default(),
    );
    let big = base.join("big");
    fs::create_dir_all(big.join(".git")).expect("big repo");
    fs::create_dir_all(big.join("hooks")).expect("big hooks");
    fs::write(
        big.join(".git").join("config"),
        "[core]\n\thooksPath = hooks\n",
    )
    .expect("big");
    for n in 0..1100 {
        fs::write(big.join("hooks").join(format!("f{n}")), "x").expect("file");
    }
    let big_repos = [(big.join(".git"), Some(big.clone()))];
    let big_before = target_states_with(&big_repos, &roots, &[], &[]);
    let big_again = target_changes(
        &big_before,
        target_states_with(&big_repos, &roots, &[], &[]),
    );

    assert_eq!(
        (changed, shown(v8_targets, &v8), big_again.len(),),
        (
            vec![
                (
                    home.join("extra.cfg"),
                    vec!["git config: changed".to_string()]
                ),
                (
                    repo.join(".githooks"),
                    vec!["core.hooksPath: pre-commit added".to_string()]
                ),
            ],
            vec![
                (
                    Path::new(".config").join("husky").join("_"),
                    "core.hooksPath".to_string(),
                    TargetKind::Hooks
                ),
                (
                    Path::new(".config").join("husky"),
                    "core.hooksPath".to_string(),
                    TargetKind::Hooks
                ),
                (
                    Path::new(".git").join(".config").join("husky").join("_"),
                    "core.hooksPath".to_string(),
                    TargetKind::Hooks
                ),
                (
                    Path::new(".git").join(".config").join("husky"),
                    "core.hooksPath".to_string(),
                    TargetKind::Hooks
                ),
            ],
            1,
        )
    );
}

/// A `.git` file's repository is its `gitdir:` folder; a bare one has no
/// work tree.
#[test]
fn repo_dirs_follow_git_files() {
    let (_dir, base) = tempdir();
    let checkout = base.join("wt");
    fs::create_dir_all(&checkout).expect("checkout");
    fs::write(checkout.join(".git"), "gitdir: ../main/.git/worktrees/wt\n").expect(".git file");
    let bare = base.join("bare");
    fs::create_dir_all(&bare).expect("bare");
    assert_eq!(
        repo_dirs(&[checkout.join(".git"), bare.clone()]),
        vec![
            (
                checkout.join("../main/.git/worktrees/wt"),
                Some(checkout.clone())
            ),
            (bare, None),
        ]
    );
}

/// D422 review 2: an include cycle is read once and a config past the
/// limits counts every time; a value naming a network or device path is
/// never opened; an absolute target a global config names counts with no
/// repository.
#[test]
fn includes_are_bounded_and_never_remote() {
    let (_dir, base) = tempdir();
    let roots = [base.clone()];
    let home = base.join("home");
    fs::create_dir_all(&home).expect("home");
    let tools = base.join("tools");
    fs::create_dir_all(&tools).expect("tools");
    fs::write(
        home.join(".gitconfig"),
        format!(
            "[include]\n\tpath = .gitconfig\n\tpath = //server/share/x\n\tpath = \\\\\\\\.\\\\CONIN$\n[credential]\n\thelper = {}\n",
            tools.join("cred.sh").display().to_string().replace('\\', "/")
        ),
    )
    .expect("global");
    let outside = [home.join(".gitconfig")];
    let mut budget = ConfigBudget::default();
    let (entries, files) = read_configs(&outside, &[], &mut budget);
    let states = target_states_with(&[], &roots, &outside, &[]);
    fs::write(tools.join("cred.sh"), "evil").expect("helper");
    let changed = target_changes(&states, target_states_with(&[], &roots, &outside, &[]));

    let big = base.join("big.cfg");
    fs::write(&big, "[alias]\n\tx = !./x\n".repeat(60_000)).expect("big config");
    let big_states = target_states_with(&[], &roots, std::slice::from_ref(&big), &[]);
    let big_again = target_changes(
        &big_states,
        target_states_with(&[], &roots, std::slice::from_ref(&big), &[]),
    );
    assert_eq!(
        (
            entries.len(),
            files,
            changed,
            big_again.iter().any(|(path, _)| *path == big),
            crate::git_watch::is_remote_or_device(Path::new("//server/share")),
            crate::git_watch::is_remote_or_device(Path::new(r"\\.\CONIN$")),
            crate::git_watch::is_remote_or_device(&base),
        ),
        (
            4,
            vec![home.join(".gitconfig")],
            vec![(
                tools.join("cred.sh"),
                vec!["credential.helper: added".to_string()]
            )],
            true,
            true,
            true,
            false,
        )
    );
}

/// D422 review 3: a `.git` file naming a network share isn't followed, NT
/// and device names count as never opened, and a path named both as a
/// hooks folder and as a file is watched as a hooks folder.
#[test]
fn remote_gitdirs_device_names_and_kinds() {
    let (_dir, base) = tempdir();
    let checkout = base.join("remote");
    fs::create_dir_all(&checkout).expect("checkout");
    fs::write(checkout.join(".git"), "gitdir: //server/share/x\n").expect(".git file");
    let work = base.join("proj");
    fs::create_dir_all(work.join(".git")).expect("repo");
    fs::write(
        work.join(".git").join("config"),
        "[alias]\n\thooks = config core.hooksPath .githooks\n[core]\n\thooksPath = .githooks\n",
    )
    .expect("config");
    let found = targets(
        &work.join(".git"),
        Some(&work),
        &[],
        &[],
        std::slice::from_ref(&base),
        &mut ConfigBudget::default(),
    );
    let githooks = found
        .iter()
        .find(|target| target.path == work.join(".githooks"))
        .map(|target| target.kind);
    let remote = |path: &str| crate::git_watch::is_remote_or_device(Path::new(path));
    assert_eq!(
        (
            repo_dirs(&[checkout.join(".git")]),
            githooks,
            remote(r"\??\UNC\evil\s\x"),
            remote("/??/UNC/evil/s/x"),
            remote(r"C:\work\CON"),
            remote(r"C:\work\nul.txt"),
            remote(r"C:\work\COM1"),
            remote(r"C:\work\console"),
            remote(r"C:\work\com10"),
        ),
        (
            Vec::new(),
            Some(TargetKind::Hooks),
            true,
            true,
            true,
            true,
            true,
            false,
            false
        )
    );
}

/// #1877 item 7(b): a path a config writes with a `\\?\` drive-letter
/// prefix (`\\?\C:\proj\.githooks`) is read as the plain path, so it is
/// watched; a `\\?\UNC\` one is still never opened, and one with `..`
/// (which `\\?\` takes literally) is still dropped. Windows only (#2027):
/// `\\?\` paths mean nothing elsewhere (`dunce::simplified` returns other
/// platforms' paths unchanged), and this code only runs on Windows.
#[cfg(windows)]
#[test]
fn verbatim_drive_paths_are_read_as_plain_ones() {
    let (_dir, base) = tempdir();
    let work = base.join("proj");
    fs::create_dir_all(work.join(".git")).expect("repo");
    // As git reads a config: `\\` is one backslash.
    let verbatim = |name: &str| format!(r"\\?\{}", work.join(name).display()).replace('\\', r"\\");
    fs::write(
        work.join(".git").join("config"),
        format!(
            "[core]\n\thooksPath = {}\n[include]\n\tpath = {}\n[alias]\n\tt = !{}\n\tu = !{}\n\tv = !{}\n",
            verbatim(".githooks"),
            verbatim("extra.cfg"),
            verbatim("t.sh"),
            r"\\\\?\\UNC\\server\\share\\x.sh",
            verbatim("..\\proj\\v.sh"),
        ),
    )
    .expect("config");
    fs::write(work.join("extra.cfg"), "[core]\n\tfsmonitor = ./fsm.exe\n").expect("include");
    let found = targets(
        &work.join(".git"),
        Some(&work),
        &[],
        &[],
        std::slice::from_ref(&base),
        &mut ConfigBudget::default(),
    );
    let file = |name: &str, why: &str| (PathBuf::from(name), why.to_string(), TargetKind::File);
    assert_eq!(
        shown(found, &work),
        vec![
            (
                PathBuf::from(r".githooks\_"),
                "core.hooksPath".to_string(),
                TargetKind::Hooks
            ),
            (
                PathBuf::from(".githooks"),
                "core.hooksPath".to_string(),
                TargetKind::Hooks
            ),
            file("extra.cfg", "include.path"),
            file("fsm.exe", "core.fsmonitor"),
            file("t.sh", "alias.t"),
        ]
    );
}

/// #1877 item 6: configs that are each under 1 MiB but together over
/// 16 MiB count as changed after every look, and none is read past that.
#[test]
fn configs_too_large_together_count_every_time() {
    let (_dir, base) = tempdir();
    let roots = [base.clone()];
    let global = base.join("global.cfg");
    let mut includes = String::from("[include]\n");
    for n in 0..20 {
        let part = base.join(format!("part{n}.cfg"));
        fs::write(&part, format!("# {}\n", "x".repeat(1_000_000))).expect("part");
        includes.push_str(&format!(
            "\tpath = {}\n",
            part.display().to_string().replace('\\', "/")
        ));
    }
    fs::write(&global, includes).expect("global");
    let look = || target_states_with(&[], &roots, std::slice::from_ref(&global), &[]);
    let first = look();
    let again = target_changes(&first, look());
    // Nothing more is read once over: the global config and 17 parts.
    let mut budget = ConfigBudget::default();
    read_configs(std::slice::from_ref(&global), &[], &mut budget);
    assert_eq!(
        (
            first
                .iter()
                .filter(|state| state.why == "git config too large to check")
                .map(|state| state.path.clone())
                .collect::<Vec<_>>(),
            again
                .iter()
                .any(|(path, _)| *path == base.join("part16.cfg")),
            budget.files,
            budget.bytes <= MAX_CONFIG_TOTAL_BYTES,
        ),
        (vec![base.join("part16.cfg")], true, 18, true)
    );
}

/// #1877 item 7(b): only a `\\?\` drive-letter path from the top of the
/// drive is simplified; the volume itself (`\\?\C:`, which would become
/// the relative `C:`), network paths and names `\\?\` takes literally stay
/// as they are (and are never opened). Windows only, as above (#2027).
#[cfg(windows)]
#[test]
fn local_simplifies_only_rooted_drive_paths() {
    let local = |path: &str| local(Path::new(path)).display().to_string();
    assert_eq!(
        [
            local(r"\\?\C:\proj\x.sh"),
            local(r"\\?\C:"),
            local(r"\\?\UNC\server\share\x"),
            local(r"\\.\C:\x"),
            local(r"\\?\C:\proj\x."),
            local(r"\\?\C:\proj/x"),
        ],
        [
            r"C:\proj\x.sh",
            r"\\?\C:",
            r"\\?\UNC\server\share\x",
            r"\\.\C:\x",
            r"\\?\C:\proj\x.",
            r"\\?\C:\proj/x",
        ]
    );
}

/// D517: `path` (on this machine) as a network path to the same file:
/// `\\localhost\C$\...` on Windows (the loopback admin share), `//...`
/// elsewhere (which POSIX may read as `/...`, and this crate counts as a
/// network path).
fn on_share(path: &Path) -> PathBuf {
    let text = path.display().to_string();
    if cfg!(windows) {
        let (drive, rest) = text.split_once(':').expect("a drive letter");
        PathBuf::from(format!(r"\\localhost\{drive}${rest}"))
    } else {
        PathBuf::from(format!("/{text}"))
    }
}

/// D517: the configs on a share flagged as not read in time.
fn unread(states: &[TargetState]) -> Vec<PathBuf> {
    states
        .iter()
        .filter(|state| state.why == "git config on a network share not read in time")
        .map(|state| state.path.clone())
        .collect()
}

/// D517 (#1877 item 7(a)): a read from a share is waited on only until the
/// deadline; while it still hangs, the next look doesn't start another for
/// the same path; once it ends, the path is read again. A missing file
/// isn't a failure; any other error is. Past the deadline a read still
/// starts (so the next look finds it done or waiting) but isn't waited for.
#[test]
fn share_reads_wait_until_the_deadline_only() {
    let soon = || Instant::now() + Duration::from_millis(200);
    let path = Path::new("//server/share/.gitconfig");
    let (release, hung) = std::sync::mpsc::channel::<()>();
    let started = Instant::now();
    let first = read_by(path, soon(), move || {
        let _ = hung.recv();
        Ok(b"late".to_vec())
    });
    let waited = started.elapsed();
    let started = Instant::now();
    let again = read_by(path, soon(), || Ok(b"x".to_vec()));
    let refused_at_once = started.elapsed() < Duration::from_millis(100);
    release.send(()).expect("release");
    let mut after = Err(());
    for _ in 0..100 {
        after = read_by(path, soon(), || Ok(b"x".to_vec()));
        if after.is_ok() {
            break;
        }
        std::thread::sleep(Duration::from_millis(20));
    }
    // `ERROR_FILE_NOT_FOUND` / `ENOENT`; `ERROR_BAD_NETPATH` / `EACCES`.
    let missing = read_by(path, soon(), || Err(std::io::Error::from_raw_os_error(2)));
    let unreachable = read_by(path, soon(), || {
        Err(std::io::Error::from_raw_os_error(if cfg!(windows) {
            53
        } else {
            13
        }))
    });
    // Past the deadline the read still starts, but isn't waited for.
    let other = Path::new("//server/share/.config/git/config");
    let (finish, hung) = std::sync::mpsc::channel::<()>();
    let past = read_by(other, Instant::now(), move || {
        let _ = hung.recv();
        Ok(b"late".to_vec())
    });
    let still_waiting = read_by(other, soon(), || Ok(b"x".to_vec()));
    finish.send(()).expect("finish");
    assert_eq!(
        (
            first,
            waited >= Duration::from_millis(150) && waited < Duration::from_secs(2),
            again,
            refused_at_once,
            after,
            missing,
            unreachable,
            past,
            still_waiting,
        ),
        (
            Err(()),
            true,
            Err(()),
            true,
            Ok(Some(b"x".to_vec())),
            Ok(None),
            Err(()),
            Err(()),
            Err(()),
        )
    );
}

/// D517 (#1877 item 7(a)): global configs on a share that can't be reached
/// count as changed after every look; a look waits about [`SHARE_TIMEOUT`]
/// at most in all, and the next one not at all while those reads still
/// hang. Windows: a TEST-NET-1 server (RFC 5737; never a real host, so no
/// credentials go anywhere; a first read hangs for about 21 s, later ones
/// fail at once; both count). Elsewhere: FIFOs no one writes, so
/// opening them hangs.
#[test]
fn global_config_on_an_unreachable_share_counts_as_changed() {
    let (_dir, base) = tempdir();
    let roots = [base.join("work")];
    fs::create_dir_all(&roots[0]).expect("work");
    #[cfg(windows)]
    let (globals, fifos) = (
        vec![
            PathBuf::from(r"\\192.0.2.1\share\.config\git\config"),
            PathBuf::from(r"\\192.0.2.1\share\.gitconfig"),
        ],
        Vec::<PathBuf>::new(),
    );
    #[cfg(not(windows))]
    let (globals, fifos) = {
        let fifos = vec![base.join("hung-xdg-config"), base.join("hung.gitconfig")];
        for fifo in &fifos {
            let made = std::process::Command::new("mkfifo")
                .arg(fifo)
                .status()
                .expect("mkfifo");
            assert!(made.success(), "mkfifo");
        }
        (
            fifos.iter().map(|fifo| on_share(fifo)).collect::<Vec<_>>(),
            fifos,
        )
    };
    let look = || {
        let started = Instant::now();
        let states = target_states_with(&[], &roots, &globals, &[]);
        (states, started.elapsed())
    };
    let (first, first_took) = look();
    let (second, second_took) = look();
    let changed = target_changes(&first, second.clone());
    for fifo in fifos {
        // Let a hung read end (read and write, so this open never waits).
        drop(fs::OpenOptions::new().read(true).write(true).open(fifo));
    }
    assert_eq!(
        (
            unread(&first),
            unread(&second),
            changed
                .iter()
                .map(|(path, _)| path.clone())
                .collect::<Vec<_>>(),
            first_took < SHARE_TIMEOUT + Duration::from_secs(1),
            second_took < Duration::from_secs(1),
        ),
        (globals.clone(), globals.clone(), globals, true, true)
    );
}

/// D517 (#1877 item 7(a)): a global config on a share that can be reached
/// is read, and what it names is watched; the include paths it names on a
/// share, or on Windows with no drive (`/x`, the current drive), are never
/// opened, while a local one is followed. A repository's
/// own config on a share is still never read (only the global and system
/// configs from the environment are).
#[test]
fn global_config_on_a_share_is_read_but_not_its_share_includes() {
    let (_dir, base) = tempdir();
    let work = base.join("work");
    let share = base.join("share");
    fs::create_dir_all(work.join("hooks")).expect("hooks");
    fs::create_dir_all(&share).expect("share");
    let slash = |path: &Path| path.display().to_string().replace('\\', "/");
    fs::write(
        share.join("inc.cfg"),
        format!(
            "[core]\n\tfsmonitor = {}\n",
            slash(&work.join("from-share.exe"))
        ),
    )
    .expect("share include");
    fs::write(
        base.join("local.cfg"),
        format!(
            "[core]\n\tfsmonitor = {}\n",
            slash(&work.join("from-local.exe"))
        ),
    )
    .expect("local include");
    fs::write(
        base.join("rootless.cfg"),
        format!(
            "[core]\n\tfsmonitor = {}\n",
            slash(&work.join("from-rootless.exe"))
        ),
    )
    .expect("rootless include");
    // `/...` with no drive: on Windows the current drive, maybe a mapped one.
    let rootless = slash(&base.join("rootless.cfg"));
    let rootless = rootless
        .split_once(':')
        .map_or(rootless.clone(), |(_, rest)| rest.to_string());
    fs::write(
        share.join(".gitconfig"),
        format!(
            "[core]\n\thooksPath = {}\n[include]\n\tpath = inc.cfg\n\tpath = {}\n\tpath = {}\n\tpath = {rootless}\n",
            slash(&work.join("hooks")),
            slash(&on_share(&share.join("inc.cfg"))),
            slash(&base.join("local.cfg")),
        ),
    )
    .expect("global");
    // Outside the root, so a path a config names from `/` is not also
    // read below it (`"$(git rev-parse --show-toplevel)/x.sh"`).
    let repo = base.join("repo");
    fs::create_dir_all(repo.join(".git")).expect("repo");
    fs::write(
        repo.join(".git").join("config"),
        format!("[alias]\n\tx = !{}\n", slash(&work.join("own.sh"))),
    )
    .expect("own config");
    let repos = [(on_share(&repo.join(".git")), Some(on_share(&repo)))];
    let outside = [on_share(&share.join(".gitconfig"))];
    let roots = [work.clone()];
    let look = || target_states_with(&repos, &roots, &outside, &[]);
    let first = look();
    let unchanged = target_changes(&first, look());
    fs::write(work.join("hooks").join("pre-commit"), "evil").expect("hook");
    let changed = target_changes(&first, look());
    let mut watched: Vec<(PathBuf, String)> = first
        .iter()
        .map(|state| (state.path.clone(), state.why.clone()))
        .collect();
    watched.sort();
    let mut expected = vec![
        (work.join("from-local.exe"), "core.fsmonitor".to_string()),
        (work.join("hooks"), "core.hooksPath".to_string()),
        (work.join("hooks").join("_"), "core.hooksPath".to_string()),
    ];
    // Elsewhere `/...` is a full path on this machine.
    if !cfg!(windows) {
        expected.insert(
            1,
            (work.join("from-rootless.exe"), "core.fsmonitor".to_string()),
        );
    }
    assert_eq!(
        (watched, unchanged, changed),
        (
            expected,
            Vec::new(),
            vec![(
                work.join("hooks"),
                vec!["core.hooksPath: pre-commit added".to_string()]
            )],
        )
    );
}

/// D517: a global config on this machine is read as before (no thread, no
/// flag), and so are the includes it names on this machine; one it names
/// on a share is still never opened.
#[test]
fn local_global_config_reads_as_before() {
    let (_dir, base) = tempdir();
    let work = base.join("work");
    fs::create_dir_all(work.join("hooks")).expect("hooks");
    let slash = |path: &Path| path.display().to_string().replace('\\', "/");
    fs::write(
        base.join("local.cfg"),
        format!(
            "[core]\n\tfsmonitor = {}\n",
            slash(&work.join("from-local.exe"))
        ),
    )
    .expect("local include");
    fs::write(
        base.join("share.cfg"),
        format!(
            "[core]\n\tfsmonitor = {}\n",
            slash(&work.join("from-share.exe"))
        ),
    )
    .expect("share include");
    let global = base.join(".gitconfig");
    fs::write(
        &global,
        format!(
            "[core]\n\thooksPath = {}\n[include]\n\tpath = local.cfg\n\tpath = {}\n",
            slash(&work.join("hooks")),
            slash(&on_share(&base.join("share.cfg"))),
        ),
    )
    .expect("global");
    let mut budget = ConfigBudget::default();
    let (entries, files) = read_outside_configs(std::slice::from_ref(&global), &[], &mut budget);
    let states = target_states_with(
        &[],
        std::slice::from_ref(&work),
        std::slice::from_ref(&global),
        &[],
    );
    let mut watched: Vec<PathBuf> = states.iter().map(|state| state.path.clone()).collect();
    watched.sort();
    assert_eq!(
        (
            entries.len(),
            files,
            budget.share_deadline.is_none(),
            budget.unread,
            watched,
        ),
        (
            4,
            vec![global.clone(), base.join("local.cfg")],
            true,
            Vec::<PathBuf>::new(),
            vec![
                work.join("from-local.exe"),
                work.join("hooks"),
                work.join("hooks").join("_"),
            ],
        )
    );
}
