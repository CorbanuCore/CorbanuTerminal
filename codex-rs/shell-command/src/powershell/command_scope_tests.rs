use super::*;
use pretty_assertions::assert_eq;

#[test]
fn a_remote_command_finds_no_names_and_no_scope_looks_none_up() {
    assert!(names_resolve(&["zz-d729-missing".to_string()], &[]));
    let _scope = enter_powershell_command_scope(PowershellCommandScope::new(
        /*cwd*/ None, /*env*/ None,
    ));
    assert!(!names_resolve(&["git".to_string()], &[]));
    assert!(names_resolve(&[], &[]));
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
            // Lower case: a test folder may be case-sensitive, Windows is not.
            Some(".com;.exe;.bat;.cmd"),
        )
    };
    let local = lookup(vec![dir.path().to_path_buf()]);
    assert_eq!(local.find("tool"), Found::Yes);
    assert_eq!(local.find("tool.exe"), Found::Yes);
    assert_eq!(local.find("helper"), Found::Yes);
    assert_eq!(local.find("Get-Content"), Found::Yes);
    assert_eq!(local.find("inmo"), Found::No);
    assert_eq!(local.find(r"..\tool"), Found::Unknown);
    // A relative folder is not looked in, so what comes after it is unknown.
    let relative = lookup(vec!["bin".into(), dir.path().to_path_buf()]);
    assert_eq!(relative.find("tool"), Found::Unknown);
    assert_eq!(relative.find("inmo"), Found::Unknown);
    let relative_last = lookup(vec![dir.path().to_path_buf(), "bin".into()]);
    assert_eq!(relative_last.find("tool"), Found::Yes);
}

/// #2756: a command whose environment has no `PATH` finds no name, so no
/// module cmdlet counts as unshadowed (the spawn may fill `PATH` in); with a
/// `PATH` that holds no program of the cmdlet's name, it does.
#[test]
fn a_command_without_path_finds_no_names() {
    let cwd = AbsolutePathBuf::from_absolute_path(std::env::temp_dir()).expect("cwd");
    let cmdlets = ["get-content".to_string()];
    let no_path = std::collections::HashMap::from([("PATHEXT".to_string(), ".EXE".to_string())]);
    let scope =
        enter_powershell_command_scope(PowershellCommandScope::new(Some(&cwd), Some(&no_path)));
    assert!(!names_resolve(&[], &cmdlets));
    assert!(names_resolve(&[], &[]));
    drop(scope);
    let dir = tempfile::tempdir().expect("tempdir");
    let with_path = std::collections::HashMap::from([(
        "Path".to_string(),
        dir.path().to_string_lossy().into_owned(),
    )]);
    let _scope =
        enter_powershell_command_scope(PowershellCommandScope::new(Some(&cwd), Some(&with_path)));
    assert!(names_resolve(&[], &cmdlets));
}
