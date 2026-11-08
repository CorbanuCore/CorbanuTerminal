//! D419 (#1775), measurement 1 of `docs/design/windows-git-watch-gaps.md`:
//! can a Low-integrity (non-admin sandbox) command delete and remake, or
//! retarget in place, a `.git` junction or symbolic link at the top of a
//! writable root, and does every such change move what rule 3 records (the
//! link's own identity, its target text, its target's identity)? Plus the
//! cost of rule 3 per link and of the D420 walk on a large tree.
//!
//! Each probe prints `D419-MEASURE` lines (run with `--nocapture`); the
//! assertion is the property rule 3 needs: whatever the command managed to
//! change is recorded as changed.

use super::*;
use pretty_assertions::assert_eq;
use std::time::Instant;

fn canonical(path: &Path) -> PathBuf {
    dunce::canonicalize(path).expect("canonical")
}

fn junction(link: &Path, target: &Path) {
    let status = std::process::Command::new("cmd")
        .args(["/d", "/c", "mklink", "/J"])
        .arg(link)
        .arg(target)
        .stdout(std::process::Stdio::null())
        .status()
        .expect("mklink");
    assert!(status.success(), "mklink /J failed");
}

/// Rewrites the junction at `link` in place to point at `target`
/// (`FSCTL_SET_REPARSE_POINT`), keeping the link's own file. Returns the
/// Win32 error, 0 on success.
fn retarget_junction(link: &Path, target: &Path) -> u32 {
    use std::os::windows::ffi::OsStrExt as _;
    use windows_sys::Win32::Foundation::CloseHandle;
    use windows_sys::Win32::Foundation::GetLastError;
    use windows_sys::Win32::Foundation::INVALID_HANDLE_VALUE;
    use windows_sys::Win32::Storage::FileSystem::CreateFileW;
    use windows_sys::Win32::Storage::FileSystem::FILE_FLAG_BACKUP_SEMANTICS;
    use windows_sys::Win32::Storage::FileSystem::FILE_FLAG_OPEN_REPARSE_POINT;
    use windows_sys::Win32::Storage::FileSystem::FILE_SHARE_DELETE;
    use windows_sys::Win32::Storage::FileSystem::FILE_SHARE_READ;
    use windows_sys::Win32::Storage::FileSystem::FILE_SHARE_WRITE;
    use windows_sys::Win32::Storage::FileSystem::OPEN_EXISTING;
    use windows_sys::Win32::System::IO::DeviceIoControl;
    const GENERIC_WRITE: u32 = 0x4000_0000;
    const FSCTL_SET_REPARSE_POINT: u32 = 0x0009_00A4;
    const IO_REPARSE_TAG_MOUNT_POINT: u32 = 0xA000_0003;
    let substitute: Vec<u16> = format!(r"\??\{}", target.display())
        .encode_utf16()
        .collect();
    let print: Vec<u16> = target.display().to_string().encode_utf16().collect();
    let names = (substitute.len() + 1 + print.len() + 1) * 2;
    let mut buffer: Vec<u8> = Vec::with_capacity(16 + names);
    buffer.extend(IO_REPARSE_TAG_MOUNT_POINT.to_le_bytes());
    buffer.extend(((8 + names) as u16).to_le_bytes());
    buffer.extend(0_u16.to_le_bytes());
    buffer.extend(0_u16.to_le_bytes());
    buffer.extend(((substitute.len() * 2) as u16).to_le_bytes());
    buffer.extend((((substitute.len() + 1) * 2) as u16).to_le_bytes());
    buffer.extend(((print.len() * 2) as u16).to_le_bytes());
    for unit in substitute
        .iter()
        .chain(&[0_u16])
        .chain(&print)
        .chain(&[0_u16])
    {
        buffer.extend(unit.to_le_bytes());
    }
    let wide: Vec<u16> = link.as_os_str().encode_wide().chain(Some(0)).collect();
    // SAFETY: valid, NUL-terminated path and buffer; the handle is closed
    // here.
    unsafe {
        let handle = CreateFileW(
            wide.as_ptr(),
            GENERIC_WRITE,
            FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE,
            std::ptr::null_mut(),
            OPEN_EXISTING,
            FILE_FLAG_BACKUP_SEMANTICS | FILE_FLAG_OPEN_REPARSE_POINT,
            0,
        );
        if handle == INVALID_HANDLE_VALUE {
            return GetLastError();
        }
        let mut returned = 0_u32;
        let ok = DeviceIoControl(
            handle,
            FSCTL_SET_REPARSE_POINT,
            buffer.as_ptr().cast(),
            buffer.len() as u32,
            std::ptr::null_mut(),
            0,
            &mut returned,
            std::ptr::null_mut(),
        );
        let err = if ok == 0 { GetLastError() } else { 0 };
        CloseHandle(handle);
        err
    }
}

/// Run by the sandboxed command (this test binary, started again inside
/// the sandbox): retargets `D419_RETARGET_LINK` in place to
/// `D419_RETARGET_TARGET` and prints the Win32 error. Does nothing when
/// they aren't set.
#[test]
fn d419_probe_helper_retarget() {
    let (Some(link), Some(target)) = (
        std::env::var_os("D419_RETARGET_LINK"),
        std::env::var_os("D419_RETARGET_TARGET"),
    ) else {
        return;
    };
    let err = retarget_junction(Path::new(&link), Path::new(&target));
    println!("D419-OP in-place.fsctl rc={err}");
}

/// Control: retargeted in place (here by this Medium process), a junction
/// keeps its own identity and only its target text changes, which rule 3
/// records, so it is flagged.
#[test]
fn d419_probe_in_place_retarget_is_seen_by_its_target_text() {
    let dir = tempfile::tempdir().expect("tempdir");
    let base = canonical(dir.path());
    let (root, a, b) = (base.join("root"), base.join("a"), base.join("b"));
    for path in [&root, &a, &b] {
        fs::create_dir_all(path).expect("create");
    }
    junction(&root.join(".git"), &a);
    let roots = [root.clone()];
    let before = crate::nested_metadata::top_level_git(&roots);
    let err = retarget_junction(&root.join(".git"), &b);
    let after = crate::nested_metadata::top_level_git(&roots);
    eprintln!(
        "D419-MEASURE medium in-place retarget rc={err} same_own_identity={} target_text_changed={}",
        before[0].identity == after[0].identity,
        before[0].link.as_ref().map(|link| &link.text)
            != after[0].link.as_ref().map(|link| &link.text),
    );
    assert_eq!(
        (
            err,
            before[0].identity == after[0].identity,
            crate::nested_metadata::appeared(&before, after),
        ),
        (0, true, vec![root.join(".git")])
    );
}

/// Measurement 1: what a non-admin sandbox command can do to a top-level
/// `.git` link, and whether rule 3 sees it.
#[test]
fn legacy_d419_probe_sandboxed_command_and_git_links() {
    let _guard = legacy_process_test_guard();
    let test_root = user_owned_test_root();
    let base = test_root.path().join("d419");
    let names = [
        "remade",
        "moved",
        "stripped",
        "target-moved",
        "symlinked",
        "in-place",
    ];
    for name in names {
        fs::create_dir_all(base.join(name)).expect("create root");
    }
    for name in ["a", "b"] {
        fs::create_dir_all(base.join(name)).expect("create target");
    }
    let base = canonical(&base);
    let roots: Vec<PathBuf> = names.iter().map(|name| base.join(name)).collect();
    let (a, b) = (base.join("a"), base.join("b"));
    // `target-moved` links to a folder inside its own root (as the `repo`
    // tool's `.repo\projects`); the others to `a`, outside every root.
    fs::create_dir_all(roots[3].join("real")).expect("inner target");
    for (index, root) in roots.iter().enumerate() {
        match index {
            3 => junction(&root.join(".git"), &root.join("real")),
            4 => {}
            _ => junction(&root.join(".git"), &a),
        }
    }
    let before = crate::nested_metadata::top_level_git(&roots);

    let codex_home = sandbox_home("d419-probe");
    let helper = std::env::current_exe().expect("test binary");
    let script = format!(
        concat!(
            "@echo off\r\n",
            "rmdir \"{r0}\\.git\"\r\n",
            "echo D419-OP remade.rmdir rc=%errorlevel%\r\n",
            "mklink /J \"{r0}\\.git\" \"{b}\" >nul\r\n",
            "echo D419-OP remade.mklink rc=%errorlevel%\r\n",
            "move \"{r1}\\.git\" \"{r1}\\moved\" >nul\r\n",
            "echo D419-OP moved.move rc=%errorlevel%\r\n",
            "mklink /J \"{r1}\\.git\" \"{a}\" >nul\r\n",
            "echo D419-OP moved.mklink rc=%errorlevel%\r\n",
            "fsutil reparsepoint delete \"{r2}\\.git\" >nul\r\n",
            "echo D419-OP stripped.fsutil rc=%errorlevel%\r\n",
            "move \"{r3}\\real\" \"{r3}\\real-old\" >nul\r\n",
            "echo D419-OP target-moved.move rc=%errorlevel%\r\n",
            "mkdir \"{r3}\\real\"\r\n",
            "echo D419-OP target-moved.mkdir rc=%errorlevel%\r\n",
            "mklink /D \"{r4}\\.git\" \"{a}\" >nul\r\n",
            "echo D419-OP symlinked.mklink-d rc=%errorlevel%\r\n",
            "set D419_RETARGET_LINK={r5}\\.git\r\n",
            "set D419_RETARGET_TARGET={b}\r\n",
            "\"{helper}\" --exact unified_exec::tests::d419_probe::d419_probe_helper_retarget --nocapture --test-threads=1 2>nul | findstr D419-OP\r\n",
            "echo D419-OP in-place.helper rc=%errorlevel%\r\n",
            "exit /b 0\r\n",
        ),
        r0 = roots[0].display(),
        r1 = roots[1].display(),
        r2 = roots[2].display(),
        r3 = roots[3].display(),
        r4 = roots[4].display(),
        r5 = roots[5].display(),
        a = a.display(),
        b = b.display(),
        helper = helper.display(),
    );
    let script_path = codex_home.path().join("d419.cmd");
    fs::write(&script_path, script).expect("write script");
    let workspace_roots: Vec<AbsolutePathBuf> = roots
        .iter()
        .map(|root| AbsolutePathBuf::from_absolute_path(root).expect("absolute"))
        .collect();
    let runtime = current_thread_runtime();
    let (stdout, exit_code) = runtime.block_on(async {
        let spawned = spawn_windows_sandbox_session_legacy(
            &PermissionProfile::workspace_write(),
            workspace_roots.as_slice(),
            codex_home.path(),
            vec![
                "C:\\Windows\\System32\\cmd.exe".to_string(),
                "/d".to_string(),
                "/c".to_string(),
                script_path.display().to_string(),
            ],
            &roots[0],
            HashMap::new(),
            /*timeout_ms*/ Some(120_000),
            &[],
            &[],
            /*tty*/ false,
            /*stdin_open*/ false,
            /*use_private_desktop*/ true,
        )
        .await
        .expect("spawn legacy session");
        collect_stdout_and_exit(
            spawned,
            codex_home.path(),
            Duration::from_secs(/*secs*/ 150),
        )
        .await
    });
    let stdout = String::from_utf8_lossy(&stdout);
    for line in stdout.lines().filter(|line| line.contains("D419-OP")) {
        eprintln!("D419-MEASURE {line}");
    }

    let after = crate::nested_metadata::top_level_git(&roots);
    let mut changed_on_disk = Vec::new();
    for root in &roots {
        let git = root.join(".git");
        let old = before.iter().find(|entry| entry.path == git);
        let new = after.iter().find(|entry| entry.path == git);
        let target_now = std::fs::read_link(&git)
            .ok()
            .map(|target| target.to_string_lossy().into_owned());
        let differs = old.map(|e| (&e.identity, &e.link)) != new.map(|e| (&e.identity, &e.link));
        eprintln!(
            "D419-MEASURE root={} before={:?} after={:?} target_now={:?} rule3_changed={differs}",
            root.file_name().unwrap_or_default().to_string_lossy(),
            old.map(|e| (&e.identity, &e.link)),
            new.map(|e| (&e.identity, &e.link)),
            target_now,
        );
        // A `.git` there now that git would see differently from the
        // start: another link target, no longer a link, a new one, or the
        // same link to a replaced folder. (One removed is no threat: git
        // then finds no repository there.)
        let was = old.and_then(|e| e.link.as_ref().map(|link| link.text.clone()));
        let exists = fs::symlink_metadata(&git).is_ok();
        if exists
            && (target_now != was
                || (root.ends_with("target-moved")
                    && fs::symlink_metadata(root.join("real-old")).is_ok()))
        {
            changed_on_disk.push(root.clone());
        }
    }
    let flagged = crate::nested_metadata::appeared(&before, after);
    eprintln!(
        "D419-MEASURE exit={exit_code} changed_on_disk={changed_on_disk:?} flagged={flagged:?}"
    );

    // Rule 3's property: everything a command changed is flagged.
    let missed: Vec<&PathBuf> = changed_on_disk
        .iter()
        .filter(|root| !flagged.contains(&root.join(".git")))
        .collect();
    assert_eq!((exit_code, missed), (0, Vec::<&PathBuf>::new()));
}

/// Costs: rule 3 per link, and the D420 walk and fingerprints on a tree
/// about this repository's size (38k files) with 100 nested repositories.
#[test]
fn d419_probe_costs() {
    let dir = tempfile::tempdir().expect("tempdir");
    let base = canonical(dir.path());
    let target = base.join("target");
    fs::create_dir_all(&target).expect("target");
    let links: Vec<PathBuf> = (0..200)
        .map(|n| {
            let root = base.join(format!("l{n}"));
            fs::create_dir_all(&root).expect("root");
            junction(&root.join(".git"), &target);
            root
        })
        .collect();
    let start = Instant::now();
    let states = crate::nested_metadata::top_level_git(&links);
    let per_link = start.elapsed() / 200;
    assert_eq!(states.iter().filter(|s| s.link.is_some()).count(), 200);

    let tree = base.join("tree");
    for project in 0..100 {
        let project_dir = tree.join(format!("p{project}"));
        let git = project_dir.join(".git");
        for folder in ["hooks", "objects\\pack", "refs\\heads", "info"] {
            fs::create_dir_all(git.join(folder)).expect("git folder");
        }
        fs::write(git.join("HEAD"), "ref: refs/heads/main\n").expect("HEAD");
        fs::write(git.join("config"), "[core]\n\tbare = false\n").expect("config");
        for sample in 0..10 {
            fs::write(git.join(format!("hooks\\h{sample}.sample")), "#!/bin/sh\n").expect("hook");
        }
        for sub in 0..19 {
            let folder = project_dir.join(format!("src{sub}"));
            fs::create_dir_all(&folder).expect("src");
            for file in 0..20 {
                fs::write(folder.join(format!("f{file}.rs")), "x").expect("file");
            }
        }
    }
    let roots = [tree];
    let start = Instant::now();
    let found = crate::nested_metadata::find(&roots);
    let walk = start.elapsed();
    let start = Instant::now();
    let repos = crate::nested_metadata::repo_states(&found.repos, &roots);
    let fingerprints = start.elapsed();
    eprintln!(
        "D419-MEASURE cost per_link_rule3={per_link:?} files={} repos={} walk={walk:?} fingerprints={fingerprints:?}",
        100 * 19 * 20,
        repos.len(),
    );
    assert_eq!(repos.len(), 100);
}
