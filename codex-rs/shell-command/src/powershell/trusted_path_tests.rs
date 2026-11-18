//! #2568: the trusted parser is never found in the current directory, however
//! it is spelled, and the fixed install paths come only from local absolute
//! paths.

use super::PowershellKind;
use super::fixed_powershell_paths;
use super::is_local_absolute;
use super::trusted_on_path;
use pretty_assertions::assert_eq;
use std::ffi::OsString;
use std::os::windows::process::CommandExt;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;

/// A directory with a fake `pwsh.exe` in it.
fn dir_with_pwsh(parent: &Path, name: &str) -> PathBuf {
    let dir = parent.join(name);
    std::fs::create_dir(&dir).expect("create dir");
    std::fs::write(dir.join("pwsh.exe"), b"MZ").expect("write fake pwsh.exe");
    dir
}

fn path_of<T: AsRef<std::ffi::OsStr>>(dirs: &[T]) -> OsString {
    std::env::join_paths(dirs).expect("join PATH")
}

/// An environment holding only `pairs`.
fn env_of<'a>(pairs: &'a [(&'a str, &'a str)]) -> impl Fn(&str) -> Option<OsString> + 'a {
    move |name: &str| {
        pairs
            .iter()
            .find(|(key, _)| *key == name)
            .map(|(_, value)| OsString::from(value))
    }
}

/// `dir`'s 8.3 short name, where the volume makes them.
fn short_name(dir: &Path) -> Option<PathBuf> {
    let output = Command::new("cmd")
        .raw_arg(format!(
            "/d /c for %I in (\"{}\") do @echo %~sI",
            dir.display()
        ))
        .output()
        .ok()?;
    let short = String::from_utf8(output.stdout).ok()?.trim().to_string();
    let short = PathBuf::from(short);
    (output.status.success() && short.is_dir() && short != dir).then_some(short)
}

fn junction(link: &Path, target: &Path) {
    let status = Command::new("cmd")
        .raw_arg(format!(
            "/d /c mklink /J \"{}\" \"{}\"",
            link.display(),
            target.display()
        ))
        .output()
        .expect("run mklink")
        .status;
    assert!(status.success(), "mklink /J failed: {status:?}");
}

#[test]
fn path_lookup_never_finds_the_current_directory() {
    let root = tempfile::tempdir().expect("tempdir");
    let current = dir_with_pwsh(root.path(), "current directory with a long name");
    let elsewhere = dir_with_pwsh(root.path(), "elsewhere");

    // Positive control: found outside the current directory.
    assert_eq!(
        trusted_on_path("pwsh.exe", &path_of(&[&current, &elsewhere]), &current),
        Some(elsewhere.join("pwsh.exe"))
    );

    // The current directory as written, in another case, with a trailing
    // separator, or relative.
    let upper = PathBuf::from(current.to_string_lossy().to_uppercase());
    let trailing = PathBuf::from(format!("{}\\", current.display()));
    for entry in [current.clone(), upper.clone(), trailing, PathBuf::from(".")] {
        assert_eq!(
            trusted_on_path("pwsh.exe", &path_of(&[&entry]), &current),
            None,
            "PATH={entry:?}"
        );
    }
    assert_eq!(
        trusted_on_path("pwsh.exe", &path_of(&[&current]), &upper),
        None,
        "cwd in another case"
    );
    // A relative entry names a directory under the current one.
    let nested = dir_with_pwsh(&current, "bin");
    assert_eq!(
        trusted_on_path("pwsh.exe", &path_of(&[Path::new("bin")]), &current),
        None
    );
    // Nor an absolute entry under it, where a command may write.
    assert_eq!(
        trusted_on_path("pwsh.exe", &path_of(&[&nested]), &current),
        None
    );

    // Its 8.3 short name, either side.
    match short_name(&current) {
        Some(short) => {
            assert_eq!(
                trusted_on_path("pwsh.exe", &path_of(&[&short]), &current),
                None,
                "PATH={short:?}"
            );
            assert_eq!(
                trusted_on_path("pwsh.exe", &path_of(&[&current]), &short),
                None,
                "cwd={short:?}"
            );
        }
        None => eprintln!("no 8.3 short names on this volume; that case is not exercised"),
    }

    // A junction to it, either side.
    let link = root.path().join("junction");
    junction(&link, &current);
    assert_eq!(
        trusted_on_path("pwsh.exe", &path_of(&[&link]), &current),
        None
    );
    assert_eq!(
        trusted_on_path("pwsh.exe", &path_of(&[&current]), &link),
        None
    );
    assert_eq!(
        trusted_on_path("pwsh.exe", &path_of(&[&link, &elsewhere]), &current),
        Some(elsewhere.join("pwsh.exe"))
    );

    // An unresolvable current directory trusts nothing on PATH.
    assert_eq!(
        trusted_on_path(
            "pwsh.exe",
            &path_of(&[&elsewhere]),
            &root.path().join("missing")
        ),
        None
    );
}

#[test]
fn fixed_paths_come_from_local_absolute_paths_only() {
    let windows_default =
        PathBuf::from(r"C:\Windows\System32\WindowsPowerShell\v1.0\powershell.exe");
    let pwsh_default = PathBuf::from(r"C:\Program Files\PowerShell\7\pwsh.exe");

    // Local absolute paths are used, defaults after them.
    assert_eq!(
        fixed_powershell_paths(
            PowershellKind::WindowsPowershell,
            &env_of(&[("SystemRoot", r"D:\Win"), ("windir", r"E:\Win")])
        ),
        vec![
            PathBuf::from(r"D:\Win\System32\WindowsPowerShell\v1.0\powershell.exe"),
            PathBuf::from(r"E:\Win\System32\WindowsPowerShell\v1.0\powershell.exe"),
            windows_default.clone(),
        ]
    );
    assert_eq!(
        fixed_powershell_paths(
            PowershellKind::Pwsh,
            &env_of(&[("ProgramFiles", r"D:\Programs")])
        ),
        vec![
            PathBuf::from(r"D:\Programs\PowerShell\7\pwsh.exe"),
            pwsh_default.clone(),
        ]
    );

    // UNC, device, relative and `..` paths are not; the defaults remain.
    for value in [
        r"\\server\share\Windows",
        r"\\?\C:\Windows",
        r"\\.\C:\Windows",
        r"\\?\UNC\server\share",
        r"//server/share/Windows",
        r"Windows",
        r"C:Windows",
        r"\Windows",
        r"C:\Windows\..\Users\x",
        "",
    ] {
        let pairs = [
            ("SystemRoot", value),
            ("windir", value),
            ("ProgramFiles", value),
            ("ProgramW6432", value),
        ];
        assert_eq!(
            fixed_powershell_paths(PowershellKind::WindowsPowershell, &env_of(&pairs)),
            vec![windows_default.clone()],
            "{value:?}"
        );
        assert_eq!(
            fixed_powershell_paths(PowershellKind::Pwsh, &env_of(&pairs)),
            vec![pwsh_default.clone()],
            "{value:?}"
        );
        assert!(!is_local_absolute(Path::new(value)), "{value:?}");
    }
    assert!(is_local_absolute(Path::new(r"C:\Windows")));
    assert!(is_local_absolute(Path::new(r"c:\")));
}
