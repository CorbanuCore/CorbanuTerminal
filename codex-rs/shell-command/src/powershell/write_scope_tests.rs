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

fn allowed_in(cwd: &str, write: &str) -> bool {
    allows(
        &[write.to_string()],
        &path(cwd),
        &[
            workspace_root(),
            Root {
                root: path(r"C:\Users\User"),
                read_only: Vec::new(),
                metadata_names: Vec::new(),
            },
        ],
        &protected(),
    )
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
    assert!(!writes_allowed(&["out.txt".to_string()], &[]));
}
