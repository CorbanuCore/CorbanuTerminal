use super::*;
use pretty_assertions::assert_eq;

fn path(raw: &str) -> WinPath {
    WinPath::parse(raw, None, Strictness::Lax).expect("absolute path")
}

fn workspace_root() -> Root {
    Root {
        root: path(r"C:\work"),
        read_only: vec![path(r"C:\work\.git"), path(r"C:\work\.codex")],
        metadata_names: vec![".git".to_string(), ".codex".to_string()],
    }
}

fn protected() -> Vec<WinPath> {
    vec![
        path(r"C:\Users\User\Documents\WindowsPowerShell\Modules"),
        path(r"C:\Users\User\Documents\WindowsPowerShell"),
        path(r"C:\Users\User\.cargo\bin"),
    ]
}

fn roots() -> Vec<Root> {
    vec![
        workspace_root(),
        Root {
            root: path(r"C:\Users\User"),
            read_only: Vec::new(),
            metadata_names: Vec::new(),
        },
    ]
}

fn decide(cwd: &str, writes: &[&str], removes: &[&str]) -> bool {
    let words = |paths: &[&str]| paths.iter().map(ToString::to_string).collect::<Vec<_>>();
    allows(
        &words(writes),
        &words(removes),
        &Places {
            cwd: &path(cwd),
            roots: &roots(),
            protected: &protected(),
            extensions: &executable_extensions(None),
        },
    )
}

fn allowed_in(cwd: &str, write: &str) -> bool {
    decide(cwd, &[write], &[])
}

#[test]
fn writes_inside_a_writable_root_are_allowed() {
    for write in [
        "out.txt",
        r".\dir\out.txt",
        "dir/sub/out.txt",
        r"C:\work\out.txt",
        r"c:\WORK\Out.txt",
        r"a\..\b.txt",
        r"C:\work\.github\x.yml",
        r"C:\Users\User\notes.txt",
    ] {
        assert!(allowed_in(r"C:\work", write), "{write}");
    }
}

#[test]
fn writes_elsewhere_or_unreadable_are_refused() {
    for write in [
        // Outside every root, also through `..` and other spellings.
        r"C:\Windows\x.txt",
        r"..\..\Windows\x.txt",
        r"D:\x.txt",
        // Read-only parts and metadata of a root, in any case.
        r".git\hooks\pre-commit",
        r".GIT\config",
        r"C:\work\.codex\config.toml",
        // Module, profile and PATH folders, and folders above them (a tree copied there).
        r"C:\Users\User\Documents\WindowsPowerShell\Modules\Z9\Z9.psm1",
        r"C:/Users/User/Documents/windowspowershell/modules/z9/z9.psm1",
        r"C:\Users\User\Documents\WindowsPowerShell\Microsoft.PowerShell_profile.ps1",
        r"C:\Users\User\Documents",
        r"C:\Users\User",
        r"C:\Users\User\.cargo\bin\rg.cmd",
        r"..\Users\User\Documents\WindowsPowerShell",
        // Other providers, streams, devices and forms Windows reads its own way.
        "env:PSModulePath",
        "HKCU:\\Software\\x",
        "Microsoft.PowerShell.Core\\FileSystem::C:\\work\\x",
        "FileSystem::C:\\work\\x",
        "variable:x",
        "alias:sc",
        "out.txt:stream",
        r"\\server\share\x",
        r"\\?\C:\work\x",
        r"\\.\C:\work\x",
        r"\x.txt",
        "C:x.txt",
        "NUL",
        "con.txt",
        r"dir\COM1",
        "x.",
        "x ",
        r"dir.\x",
        // Wildcards, escapes, home and 8.3 names.
        "*.psm1",
        "x?.txt",
        "[a]b.txt",
        "a`b.txt",
        r"~\x.txt",
        r"C:\Users\USER~1\Documents\x",
        "",
    ] {
        assert!(!allowed_in(r"C:\work", write), "{write}");
    }
}

#[test]
fn a_working_directory_outside_the_roots_writes_nowhere() {
    assert!(!allowed_in(r"C:\Windows\Temp", "out.txt"));
    assert!(!allowed_in(
        r"C:\Users\User\Documents\WindowsPowerShell\Modules\Z9",
        "Z9.psm1"
    ));
}

#[test]
fn parts_are_normalised_and_folded() {
    assert_eq!(
        WinPath::parse(r"a\.\b\..\C", Some(&path(r"c:\Work")), Strictness::Write),
        Some(path(r"C:\work\a\c"))
    );
    assert_eq!(
        WinPath::parse(r"..\..", Some(&path(r"C:\work")), Strictness::Write),
        None
    );
    assert_eq!(
        WinPath::parse(r"\\?\C:\x", None, Strictness::Lax),
        Some(path(r"C:\x"))
    );
}

#[test]
fn no_write_is_allowed_outside_a_scope() {
    assert!(!writes_allowed(&["out.txt".to_string()], &[], &[]));
    assert!(!writes_allowed(&[], &["out.txt".to_string()], &[]));
}

#[test]
fn files_windows_runs_from_a_folder_are_not_written() {
    for write in [
        "node.cmd",
        "x.EXE",
        r"bin\tool.bat",
        "a.dll",
        "s.vbs",
        "x.com",
    ] {
        assert!(!allowed_in(r"C:\work", write), "{write}");
    }
    for write in ["notes.ps1", "x.txt", "x.exe.txt", "build.rs"] {
        assert!(allowed_in(r"C:\work", write), "{write}");
    }
    // `PATHEXT` decides.
    assert!(allows(
        &["x.js".to_string()],
        &[],
        &Places {
            cwd: &path(r"C:\work"),
            roots: &roots(),
            protected: &[],
            extensions: &executable_extensions(Some(".COM;.EXE")),
        },
    ));
}

#[test]
fn removing_a_folder_above_a_read_only_part_is_refused() {
    assert!(decide(r"C:\work", &[], &[r"build"]));
    for remove in [".", r"C:\work", r"C:\", r".git", r".git\hooks"] {
        assert!(!decide(r"C:\work", &[], &[remove]), "{remove}");
    }
    // Moving `s` to `.` writes `.\s`, and removes `s`.
    assert!(decide(r"C:\work", &[".", r".\s"], &["s"]));
    // A copy into `.` named like a read-only part is refused.
    assert!(!decide(r"C:\work", &[".", r".\.git"], &[]));
}

#[test]
fn names_resolve_as_powershell_finds_them_on_path() {
    let dir = tempfile::tempdir().expect("tempdir");
    std::fs::write(dir.path().join("tool.exe"), "").expect("write");
    std::fs::write(dir.path().join("helper.ps1"), "").expect("write");
    std::fs::write(dir.path().join("Get-Content.ps1"), "").expect("write");
    let lookup = |path: Vec<std::path::PathBuf>| {
        PathLookup::new(
            Some(std::env::join_paths(path).expect("PATH").as_os_str()),
            Some(".COM;.EXE;.BAT;.CMD"),
        )
    };
    let local = lookup(vec![dir.path().to_path_buf()]);
    assert_eq!(local.find("tool"), Found::Yes);
    assert_eq!(local.find("tool.exe"), Found::Yes);
    assert_eq!(local.find("helper"), Found::Yes);
    assert_eq!(local.find("Get-Content"), Found::Yes);
    assert_eq!(local.find("inmo"), Found::No);
    assert_eq!(local.find(r"..\tool"), Found::Unknown);
    // A relative folder is not looked in.
    let relative = lookup(vec!["bin".into(), dir.path().to_path_buf()]);
    assert_eq!(relative.find("tool"), Found::Yes);
    assert_eq!(relative.find("inmo"), Found::Unknown);
}
